//! Verify all four selector values for the five paired-palette consumers.

use super::{
    attach_from_directory, cmd, encode_ui_raw_transfers, parse, parse_ui_raw_transfers, Manifest,
    MANIFEST_FILE, UI_RAW_TRANSFERS_FILE, UI_RAW_TRANSFERS_ID,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::{
        read_audit::{Service, TraceCapture, TraceFilter},
        SourceRange,
    },
    Bus, Machine,
};
use serde::Deserialize;
use std::{collections::BTreeMap, fs, path::PathBuf, sync::Arc};

const ROM_BASE: u32 = 0x0800_0000;
const CPU_SET: u32 = 0x0814_5230;
const ALIAS: (u32, u32) = (0xb4bea0, 0xb4bec0);
type Reads = BTreeMap<(u32, usize, u32), u64>;

#[derive(Deserialize)]
struct Plan {
    consumers: Vec<Consumer>,
    spans: Vec<SourceRange>,
}

#[derive(Deserialize)]
struct Consumer {
    entry: u32,
    stop: u32,
    literal_reader: u32,
    literal: u32,
    table: u32,
    index_loads: Vec<u32>,
    source_bias: u32,
    length: usize,
    destinations: Vec<u32>,
    code_sha256: String,
}

fn input(key: &str) -> PathBuf {
    std::env::var_os(key).unwrap().into()
}

fn run_consumer(machine: &mut Machine, consumer: &Consumer, mode: u32) {
    machine.bus.write8(0x0200_2fc2, 0xa0 | mode as u8);
    machine.cpu.regs[0] = 0;
    machine.cpu.regs[4] = 0x0200_0000;
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[15] = ROM_BASE + consumer.entry;
    machine.cpu.cpsr = 0xbf;
    machine.cpu.branch = None;
    for _ in 0..200 {
        if machine.cpu.regs[15] == ROM_BASE + consumer.stop {
            return;
        }
        machine.step();
    }
    panic!("Consumer {:06x} did not reach its copy end", consumer.entry);
}

fn run_all(machine: &mut Machine, rom: &[u8], plan: &Plan, service: Service) -> Reads {
    let capture = TraceCapture::start(
        TraceFilter {
            readers: vec![],
            ranges: plan
                .spans
                .iter()
                .map(|span| (span.offset, span.offset + span.length))
                .chain(std::iter::once(ALIAS))
                .collect(),
        },
        8000,
    )
    .unwrap();
    capture.frame(Some(machine.bus.frames));
    machine.bus.write32(0x0300_2810, 0x0201_0000);
    machine.bus.write32(0x0300_2818, 0x0201_4000);
    machine.bus.write32(0x0300_27d8, 0x0201_8000);

    let mut expected: Reads = BTreeMap::new();
    for mode in 0..4 {
        for consumer in &plan.consumers {
            run_consumer(machine, consumer, mode);
            *expected
                .entry((consumer.literal, 4, ROM_BASE + consumer.literal_reader))
                .or_default() += 1;
            assert_eq!(consumer.index_loads.len(), consumer.destinations.len());
            for (pair, (&destination, &reader)) in consumer
                .destinations
                .iter()
                .zip(&consumer.index_loads)
                .enumerate()
            {
                let cell = consumer.table + 8 * mode + 4 * pair as u32;
                let at = cell as usize;
                let root = u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()) - ROM_BASE;
                let source = root + consumer.source_bias;
                *expected.entry((cell, 4, ROM_BASE + reader)).or_default() += 1;
                for offset in (0..consumer.length).step_by(2) {
                    *expected
                        .entry((source + offset as u32, 2, CPU_SET))
                        .or_default() += 1;
                    assert_eq!(
                        machine.bus.read16(destination + offset as u32),
                        u16::from_le_bytes(
                            rom[source as usize + offset..source as usize + offset + 2]
                                .try_into()
                                .unwrap(),
                        ),
                        "Mode {mode}, consumer {:06x}, byte {offset}",
                        consumer.entry,
                    );
                }
            }
        }
    }
    let report = capture.finish();
    assert!(!report.truncated && report.dropped_requests == 0);
    let mut actual = BTreeMap::new();
    for event in report.events {
        let site = event.key.instruction.expect("Original palette read site");
        let read = event.key.read;
        assert!(site.thumb);
        assert_eq!(event.key.service, service, "{event:?}");
        assert!(actual
            .insert((read.physical_offset, read.length, site.pc), event.requests)
            .is_none());
    }
    assert_eq!(actual, expected);
    actual
}

#[test]
#[ignore = "exact EUR ROM, E534 color pair plan and e328-d327 predecessor manifest"]
fn european_color_mode_pairs_use_complete_extracted_banks() {
    let rom_path = input("FFTA_STATE_TEST_ROM");
    let rom = fs::read(&rom_path).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let work = input("FFTA_COLOR_PAIRS_WORK");
    let plan: Plan =
        serde_json::from_slice(&fs::read(input("FFTA_COLOR_PAIRS_PLAN")).unwrap()).unwrap();
    assert_eq!(plan.consumers.len(), 5);
    assert_eq!(plan.spans.len(), 11);
    let source_bytes = plan.spans.iter().map(|span| span.length).sum::<u32>();
    assert_eq!(source_bytes, 6228);
    for consumer in &plan.consumers {
        assert_eq!(
            digest(&rom[consumer.entry as usize..consumer.stop as usize]),
            consumer.code_sha256
        );
    }
    for span in &plan.spans {
        let start = span.offset as usize;
        assert_eq!(
            digest(&rom[start..start + span.length as usize]),
            span.sha256
        );
    }
    let old_bytes = fs::read(input("FFTA_COLOR_PAIRS_PREDECESSOR")).unwrap();
    let previous: Manifest = parse(&old_bytes, "E533 manifest").unwrap();
    assert_eq!(
        (previous.extractor_revision, previous.decoder_revision),
        (328, 327)
    );
    let resources = tempfile::tempdir_in(&work).unwrap();
    cmd(&[
        "prepare".into(),
        rom_path.to_str().unwrap().into(),
        resources.path().to_str().unwrap().into(),
    ])
    .unwrap();
    let revision = resources.path().join(EUROPE_SHA256).join("v1/e329-d328");
    let current = fs::read_to_string(revision.join("current")).unwrap();
    let dataset = revision.join(current.trim());
    let manifest_path = dataset.join(MANIFEST_FILE);
    let packet_path = dataset.join(UI_RAW_TRANSFERS_FILE);
    let manifest_bytes = fs::read(&manifest_path).unwrap();
    let packet_bytes = fs::read(&packet_path).unwrap();
    let manifest: Manifest = parse(&manifest_bytes, "E534 manifest").unwrap();
    let index = previous
        .resources
        .iter()
        .position(|entry| entry.id == UI_RAW_TRANSFERS_ID)
        .unwrap();
    let mut expected = previous.clone();
    expected.extractor_revision = 329;
    expected.decoder_revision = 328;
    expected.resources[index].sha256 = digest(&packet_bytes);
    expected.resources[index]
        .sources
        .extend(plan.spans.iter().cloned());
    expected.resources[index]
        .sources
        .sort_unstable_by_key(|span| span.offset);
    assert_eq!(manifest, expected);

    let mut old_packet = parse_ui_raw_transfers(&packet_bytes).unwrap();
    assert!(old_packet
        .halfword_copies
        .as_mut()
        .unwrap()
        .color_mode_pairs
        .take()
        .is_some());
    let old_packet = encode_ui_raw_transfers(&old_packet).unwrap();
    assert_eq!(digest(&old_packet), previous.resources[index].sha256);
    let mut managed = Machine::new(rom.clone());
    fs::write(&packet_path, old_packet).unwrap();
    fs::write(&manifest_path, old_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let mut incomplete = previous;
    incomplete.extractor_revision = 329;
    incomplete.decoder_revision = 328;
    fs::write(&manifest_path, toml::to_string_pretty(&incomplete).unwrap()).unwrap();
    assert!(attach_from_directory(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(&owner, managed.bus.rom_resources().unwrap()));
    fs::write(&packet_path, packet_bytes).unwrap();
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    drop(owner);

    let mut raw = Machine::new(rom.clone());
    let initial_state = raw.save_state().unwrap();
    for machine in [&mut raw, &mut managed] {
        machine.load_state(&initial_state).unwrap();
        machine.bus.ffta_sprites.enabled = false;
    }
    let alter_backing = |machine: &mut Machine| {
        for (start, end) in plan
            .spans
            .iter()
            .map(|span| (span.offset, span.offset + span.length))
            .chain(std::iter::once(ALIAS))
        {
            for byte in &mut machine.bus.rom[start as usize..end as usize] {
                *byte ^= 0xff;
            }
        }
    };
    alter_backing(&mut managed);
    let raw_reads = run_all(&mut raw, &rom, &plan, Service::Raw);
    assert_eq!(
        raw_reads,
        run_all(&mut managed, &rom, &plan, Service::Typed)
    );
    let resources = managed.bus.rom_resources().unwrap();
    let transfer = resources.ui_raw_transfers_stats().unwrap();
    let palette = resources.ui_palette_stats().unwrap();
    assert_eq!(
        (transfer.payload_bytes, transfer.selector_bytes),
        (12_352, 208)
    );
    assert_eq!((transfer.banks_used, transfer.groups_used), (3, 3));
    assert_eq!(palette.source_bytes, 192);
    alter_backing(&mut managed);
    assert_eq!(raw.bus.clock, managed.bus.clock);
    assert_eq!(
        digest(&raw.save_state().unwrap()),
        digest(&managed.save_state().unwrap())
    );

    fs::write(work.join("manifest.toml"), &manifest_bytes).unwrap();
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "status": "PASS",
            "revision": [329, 328],
            "manifest_sha256": digest(&manifest_bytes),
            "source_bytes": source_bytes,
            "source_spans": plan.spans.len(),
            "consumer_cases": plan.consumers.len() * 4,
            "original_reads": raw_reads.values().sum::<u64>(),
            "alias_bytes_read": palette.source_bytes,
            "predecessor_attach": true,
            "missing_group_rejected": true,
            "full_state_and_clock_equal": true
        }))
        .unwrap(),
    )
    .unwrap();
}
