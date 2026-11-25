//! Verify both original copy phases of the EUR scanline blend table.

use super::{
    attach_from_directory, cmd, encode_ui_raw_transfers, parse, parse_ui_raw_transfers,
    row_pair_copy_tests::run_thumb, Manifest, MANIFEST_FILE, UI_RAW_TRANSFERS_FILE,
    UI_RAW_TRANSFERS_ID,
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
const TABLE: u32 = 0x3a62ec;
const CPU_SET: u32 = 0x0814_5230;
const CALLBACK: u32 = 0x0810_ce10;
const STACK: u32 = 0x0300_7c00;
const STATE: u32 = 0x0201_0000;
const CONTEXT: u32 = 0x0201_0100;
const ACTOR: u32 = 0x0201_0200;
const BUFFER_STATE: u32 = 0x0201_0300;
const DESTINATION: u32 = 0x0201_0400;
const LINES: u32 = 160;
type Reads = BTreeMap<(u32, usize, u32), u64>;

#[derive(Deserialize)]
struct Plan {
    phases: [Phase; 2],
    spans: Vec<SourceRange>,
}

#[derive(Deserialize)]
struct Phase {
    init: u32,
    init_stop: u32,
    state: u16,
    start: u16,
    end: u16,
    ticks: u16,
    reader: u32,
    literal: u32,
    indexes: Vec<u16>,
}

fn input(key: &str) -> PathBuf {
    std::env::var_os(key).unwrap().into()
}

fn run_phases(machine: &mut Machine, rom: &[u8], plan: &Plan, service: Service) -> Reads {
    machine.bus.write32(CONTEXT + 0x24, ACTOR);
    // The original getter follows the task-local buffer pointer through these fields.
    machine.bus.write32(ACTOR + 0x1c, BUFFER_STATE);
    machine.bus.write16(BUFFER_STATE + 0x10, 1);
    machine.bus.write32(BUFFER_STATE + 0x14, DESTINATION);
    let capture = TraceCapture::start(
        TraceFilter {
            readers: vec![],
            ranges: plan
                .spans
                .iter()
                .map(|s| (s.offset, s.offset + s.length))
                .collect(),
        },
        600,
    )
    .unwrap();
    capture.frame(Some(machine.bus.frames));
    let mut expected: Reads = BTreeMap::new();
    for phase in &plan.phases {
        machine.bus.write32(STACK + 0x30, CONTEXT + 0x30);
        machine.cpu.regs[6] = CONTEXT;
        run_thumb(machine, ROM_BASE + phase.init, ROM_BASE + phase.init_stop);
        assert_eq!(machine.bus.read16(CONTEXT + 0x30), phase.start);
        assert_eq!(machine.bus.read16(CONTEXT + 0x32), phase.ticks);
        machine.bus.write16(STATE, phase.state);

        for (tick, &index) in phase.indexes.iter().enumerate() {
            machine.cpu.regs[..3].copy_from_slice(&[0, STATE, CONTEXT]);
            machine.cpu.regs[14] = ROM_BASE | 1;
            run_thumb(machine, CALLBACK, ROM_BASE);
            assert_eq!(machine.bus.read16(CONTEXT + 0x30), index);
            let complete = tick + 1 == phase.indexes.len();
            assert_eq!(machine.bus.read16(STATE), phase.state + u16::from(complete));
            let source = TABLE + u32::from(index) * 2;
            let dest = (DESTINATION - 0x0200_0000) as usize;
            assert_eq!(
                &machine.bus.ewram[dest..dest + (LINES * 2) as usize],
                &rom[source as usize..(source + LINES * 2) as usize],
                "State {:#x}, tick {}",
                phase.state,
                tick + 1,
            );
            *expected
                .entry((phase.literal, 4, ROM_BASE + phase.reader))
                .or_default() += 1;
            for line in 0..LINES {
                *expected.entry((source + 2 * line, 2, CPU_SET)).or_default() += 1;
            }
        }
        assert_eq!(machine.bus.read16(CONTEXT + 0x30), phase.end);
    }
    let report = capture.finish();
    assert!(!report.truncated && report.dropped_requests == 0);
    let mut actual = Reads::new();
    for event in report.events {
        let site = event
            .key
            .instruction
            .expect("Original scanline source read");
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
#[ignore = "exact EUR ROM, E535 interpolation plan and e329-d328 predecessor manifest"]
fn european_scanline_blend_phases_read_extracted_bank() {
    let rom_path = input("FFTA_STATE_TEST_ROM");
    let rom = fs::read(&rom_path).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let work = input("FFTA_SCANLINE_BLEND_WORK");
    let plan: Plan =
        serde_json::from_slice(&fs::read(input("FFTA_SCANLINE_BLEND_PLAN")).unwrap()).unwrap();
    assert_eq!(plan.spans.len(), 3);
    assert_eq!(plan.phases.each_ref().map(|p| p.indexes.len()), [60, 60],);
    let source_bytes = plan.spans.iter().map(|span| span.length).sum::<u32>();
    assert_eq!(source_bytes, 968);
    for span in &plan.spans {
        let start = span.offset as usize;
        assert_eq!(
            digest(&rom[start..start + span.length as usize]),
            span.sha256
        );
    }
    let previous_bytes = fs::read(input("FFTA_SCANLINE_BLEND_PREDECESSOR")).unwrap();
    let previous: Manifest = parse(&previous_bytes, "E534 manifest").unwrap();
    assert_eq!(
        (previous.extractor_revision, previous.decoder_revision),
        (329, 328)
    );
    let resources = tempfile::tempdir_in(&work).unwrap();
    cmd(&[
        "prepare".into(),
        rom_path.to_str().unwrap().into(),
        resources.path().to_str().unwrap().into(),
    ])
    .unwrap();
    let revision = resources.path().join(EUROPE_SHA256).join("v1/e330-d329");
    let current = fs::read_to_string(revision.join("current")).unwrap();
    let dataset = revision.join(current.trim());
    let manifest_path = dataset.join(MANIFEST_FILE);
    let packet_path = dataset.join(UI_RAW_TRANSFERS_FILE);
    let manifest_bytes = fs::read(&manifest_path).unwrap();
    let packet_bytes = fs::read(&packet_path).unwrap();
    let manifest: Manifest = parse(&manifest_bytes, "E535 manifest").unwrap();
    let index = previous
        .resources
        .iter()
        .position(|entry| entry.id == UI_RAW_TRANSFERS_ID)
        .unwrap();
    let mut expected = previous.clone();
    expected.extractor_revision = 330;
    expected.decoder_revision = 329;
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
        .scanline_blend
        .take()
        .is_some());
    let old_packet = encode_ui_raw_transfers(&old_packet).unwrap();
    assert_eq!(digest(&old_packet), previous.resources[index].sha256);
    let mut managed = Machine::new(rom.clone());
    fs::write(&packet_path, old_packet).unwrap();
    fs::write(&manifest_path, previous_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let mut incomplete = previous;
    incomplete.extractor_revision = 330;
    incomplete.decoder_revision = 329;
    fs::write(&manifest_path, toml::to_string_pretty(&incomplete).unwrap()).unwrap();
    assert!(attach_from_directory(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(&owner, managed.bus.rom_resources().unwrap()));
    fs::write(&packet_path, packet_bytes).unwrap();
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    drop(owner);

    let mut raw = Machine::new(rom.clone());
    let initial = raw.save_state().unwrap();
    for machine in [&mut raw, &mut managed] {
        machine.load_state(&initial).unwrap();
        machine.bus.ffta_sprites.enabled = false;
    }
    let flip_backing = |machine: &mut Machine| {
        for span in &plan.spans {
            let start = span.offset as usize;
            for byte in &mut machine.bus.rom[start..start + span.length as usize] {
                *byte ^= 0xff;
            }
        }
    };
    flip_backing(&mut managed);
    let actual = run_phases(&mut raw, &rom, &plan, Service::Raw);
    assert_eq!(
        actual,
        run_phases(&mut managed, &rom, &plan, Service::Typed)
    );
    let copied_halfwords = actual.keys().filter(|(_, _, pc)| *pc == CPU_SET).count();
    assert_eq!(copied_halfwords, 477);
    // The first three entries precede the first updated index.
    for at in (TABLE..TABLE + 6).step_by(2) {
        assert_eq!(
            raw.bus.read16(ROM_BASE + at),
            managed.bus.read16(ROM_BASE + at)
        );
    }
    let stats = managed
        .bus
        .rom_resources()
        .unwrap()
        .ui_raw_transfers_stats()
        .unwrap();
    assert_eq!((stats.payload_bytes, stats.selector_bytes), (38_406, 480));
    flip_backing(&mut managed);
    assert_eq!(raw.bus.clock, managed.bus.clock);
    assert_eq!(
        digest(&raw.save_state().unwrap()),
        digest(&managed.save_state().unwrap())
    );
    fs::write(work.join("manifest.toml"), &manifest_bytes).unwrap();
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "status": "PASS", "revision": [330, 329],
            "manifest_sha256": digest(&manifest_bytes),
            "source_bytes": source_bytes, "phased_halfwords": copied_halfwords,
            "original_copies": 120, "original_reads": actual.values().sum::<u64>(),
            "direct_prefix_halfwords": 3, "predecessor_attach": true,
            "missing_group_rejected": true, "full_state_and_clock_equal": true
        }))
        .unwrap(),
    )
    .unwrap();
}
