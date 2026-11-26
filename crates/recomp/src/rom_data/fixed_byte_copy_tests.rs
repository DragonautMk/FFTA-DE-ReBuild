//! Verify the EUR fixed-size byte-copy bank against the original interpreter paths.

use super::{
    attach_dataset, encode_copied_halfword_lookups,
    indexed_reader_bank_tests::{execute_until, initialize},
    read_dataset, COPIED_HALFWORD_LOOKUPS_ID,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::{
        copied_halfword_lookups::fixed_byte_copies::{LITERAL_ROLE, PAYLOAD_ROLE},
        read_audit::{Reader, Service, TraceCapture, TraceFilter},
        Extraction, SourceRange,
    },
    Bus, Machine,
};
use serde::Deserialize;
use std::{collections::BTreeSet, fs, path::PathBuf, sync::Arc};

const BASE: u32 = 0x0800_0000;
const IWRAM: u32 = 0x0300_0000;
const CURRENT: (u32, u32) = (342, 341);
const PREVIOUS: (u32, u32) = (341, 340);
const PREVIOUS_MANIFEST_SHA256: &str =
    "dc77a74b0ebe493ced79370d8e25be11f4876e408af261fdbafc2395b275e59b";
const PLAN_SHA256: &str = "aa5ff835be13361562868356d7bf8b8b27a5059a8c2383efaaf6470c06a91560";

#[derive(Deserialize)]
struct Plan {
    verified_rows: Vec<Caller>,
    new_source_plan: PlannedBank,
}

#[derive(Deserialize)]
struct Caller {
    call: u32,
    reader: u32,
    literal: u32,
    source: u32,
    length: u32,
    guest_source: u32,
    reader_sha256: String,
    new_source_bytes: u32,
    new_literal_bytes: u32,
}

#[derive(Deserialize)]
struct PlannedBank {
    payloads: Vec<PlannedSpan>,
    literals: Vec<PlannedSpan>,
}

#[derive(Deserialize)]
struct PlannedSpan {
    offset: u32,
    length: u32,
    sha256: String,
}

fn input(name: &str) -> PathBuf {
    std::env::var_os(name).expect(name).into()
}

#[test]
#[ignore = "requires the exact EUR ROM, retained e341 files and the E568 source plan"]
fn european_fixed_byte_copies_use_extracted_sources() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let plan = fs::read(input("FFTA_FIXED_COPY_PLAN")).unwrap();
    assert_eq!(digest(&plan), PLAN_SHA256);
    let plan: Plan = serde_json::from_slice(&plan).unwrap();
    assert_eq!(plan.verified_rows.len(), 120);
    let rows: Vec<_> = plan
        .verified_rows
        .into_iter()
        .filter(|row| row.new_source_bytes != 0 || row.new_literal_bytes != 0)
        .collect();
    assert_eq!(rows.len(), 99);
    assert_eq!(plan.new_source_plan.payloads.len(), 91);
    assert_eq!(plan.new_source_plan.literals.len(), 97);
    let mut planned_spans: Vec<_> = [
        (plan.new_source_plan.payloads, PAYLOAD_ROLE),
        (plan.new_source_plan.literals, LITERAL_ROLE),
    ]
    .into_iter()
    .flat_map(|(spans, role)| {
        spans.into_iter().map(move |span| SourceRange {
            offset: span.offset,
            length: span.length,
            sha256: span.sha256,
            role: role.into(),
        })
    })
    .collect();
    planned_spans.sort_unstable_by_key(|span| span.offset);

    let dataset_path = input("FFTA_STATE_TEST_DATASET");
    assert_eq!(
        digest(&fs::read(dataset_path.join("manifest.toml")).unwrap()),
        PREVIOUS_MANIFEST_SHA256
    );
    let mut dataset = read_dataset(&dataset_path).unwrap();
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        PREVIOUS
    );
    let mut managed = Machine::new(rom.clone());
    attach_dataset(&mut managed, &dataset).unwrap();
    let predecessor = managed.bus.rom_resources().unwrap().clone();
    let old_raw = rows[0].source as usize;
    managed.bus.rom[old_raw] ^= 0xff;
    assert_eq!(managed.bus.read8(BASE + old_raw as u32), !rom[old_raw]);
    managed.bus.rom[old_raw] = rom[old_raw];

    (
        dataset.manifest.extractor_revision,
        dataset.manifest.decoder_revision,
    ) = CURRENT;
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(
        &predecessor,
        managed.bus.rom_resources().unwrap()
    ));
    let catalog = Extraction::from_rom_for_manifest(&rom, &dataset.manifest)
        .unwrap()
        .copied_halfword_lookups
        .unwrap();
    let mut without_bank = catalog.decoded.clone();
    let bank = without_bank.fixed_byte_copies.take().unwrap();
    assert_eq!(
        encode_copied_halfword_lookups(&without_bank)
            .unwrap()
            .as_slice(),
        dataset.copied_halfword_lookups_bytes.as_deref().unwrap()
    );
    let packet = encode_copied_halfword_lookups(&catalog.decoded).unwrap();
    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|resource| resource.id == COPIED_HALFWORD_LOOKUPS_ID)
        .unwrap();
    assert!(catalog.sources.starts_with(&resource.sources));
    assert_eq!(catalog.sources[resource.sources.len()..], planned_spans);
    resource.sha256 = digest(&packet);
    resource.sources = catalog.sources;
    dataset.copied_halfword_lookups = Some(catalog.decoded);
    dataset.copied_halfword_lookups_bytes = Some(packet.clone());
    attach_dataset(&mut managed, &dataset).unwrap();

    let new_spans: Vec<_> = bank.source_spans().collect();
    let flip_sources = |machine: &mut Machine| {
        for (offset, bytes, _) in &new_spans {
            let start = *offset as usize;
            for byte in &mut machine.bus.rom[start..start + bytes.len()] {
                *byte ^= 0xff;
            }
        }
    };
    let mut raw = Machine::new(rom.clone());
    let initial = raw.save_state().unwrap();
    let mut requests = 0;
    let mut payload_widths = BTreeSet::new();
    for caller in &rows {
        assert_eq!(caller.new_source_bytes, caller.length);
        assert_eq!(caller.new_literal_bytes, 4);
        assert_eq!(
            digest(&rom[caller.reader as usize..caller.call as usize + 4]),
            caller.reader_sha256
        );
        let call = BASE + caller.call;
        let stop = call + 4;
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap();
            initialize(machine, BASE + caller.reader);
            machine.cpu.regs[..13].fill(IWRAM + 0x7000);
        }
        execute_until(&mut raw, call, 4);
        let destination = raw.cpu.regs[0];
        execute_until(&mut raw, stop, 512);
        flip_sources(&mut managed);

        let source = caller.source..caller.source + caller.length;
        let literal = caller.literal..caller.literal + 4;
        let trace = TraceCapture::start(
            TraceFilter {
                readers: vec![Reader::Bus],
                ranges: vec![(source.start, source.end), (literal.start, literal.end)],
            },
            caller.length as usize + 2,
        )
        .unwrap();
        trace.frame(Some(managed.bus.frames));
        execute_until(&mut managed, call, 4);
        assert_eq!(managed.cpu.regs[0], destination);
        assert_eq!(managed.cpu.regs[1], caller.guest_source);
        assert_eq!(managed.cpu.regs[2], caller.length);
        execute_until(&mut managed, stop, 512);
        let trace = trace.finish();
        assert_eq!(trace.dropped_requests, 0);
        let mut covered = BTreeSet::new();
        for event in &trace.events {
            let access = event.key.read;
            let site = event.key.instruction.unwrap();
            let at = access.physical_offset;
            let end = at + access.length as u32;
            assert_eq!(
                event.key.service,
                Service::Typed,
                "original read at {:08x}",
                site.pc
            );
            assert!(site.thumb);
            if literal.contains(&at) {
                assert_eq!(
                    (site.pc, at, end),
                    (BASE + caller.reader, literal.start, literal.end)
                );
            } else {
                assert!(source.contains(&at) && end <= source.end);
                assert!((0x08147dc0..0x08147e22).contains(&site.pc));
                payload_widths.insert(access.length);
            }
            covered.extend(at..end);
        }
        assert_eq!(
            covered,
            source.clone().chain(literal).collect(),
            "copy at {:08x}",
            call
        );
        requests += trace.matched_requests;
        let start = destination.checked_sub(IWRAM).unwrap() as usize;
        assert_eq!(
            &managed.bus.iwram[start..start + caller.length as usize],
            &rom[source.start as usize..source.end as usize],
            "destination after {:08x}",
            call
        );
        flip_sources(&mut managed);
        assert_eq!(managed.bus.clock, raw.bus.clock);
        assert!(
            managed.save_state().unwrap() == raw.save_state().unwrap(),
            "state after {:08x}",
            call
        );
    }
    assert_eq!(payload_widths, BTreeSet::from([1, 4]));

    flip_sources(&mut managed);
    for gap in [0x14d936, 0x14d942] {
        assert!(dataset.manifest.resources.iter().all(|resource| resource
            .sources
            .iter()
            .all(|span| span.offset >= gap + 2 || span.offset + span.length <= gap)));
        let start = gap as usize;
        let original = u16::from_le_bytes(rom[start..start + 2].try_into().unwrap());
        for machine in [&mut raw, &mut managed] {
            for byte in &mut machine.bus.rom[start..start + 2] {
                *byte ^= 0xff;
            }
            assert_eq!(machine.bus.read16(BASE + gap), !original);
        }
    }
    let mut boundary_reads = 0;
    for (offset, bytes, _) in &new_spans {
        for at in [offset - 1, offset + bytes.len() as u32 - 1] {
            for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
                let address = mirror + at;
                assert_eq!(
                    managed.bus.read16(address),
                    raw.bus.read16(address),
                    "halfword at {address:08x}"
                );
                assert_eq!(
                    managed.bus.read32(address),
                    raw.bus.read32(address),
                    "word at {address:08x}"
                );
                boundary_reads += 2;
            }
        }
    }

    let added_bytes = |role| -> u32 {
        planned_spans
            .iter()
            .filter(|span| span.role == role)
            .map(|span| span.length)
            .sum()
    };
    let work = input("FFTA_FIXED_COPY_WORK");
    let manifest = toml::to_string_pretty(&dataset.manifest).unwrap();
    fs::write(work.join("manifest.toml"), &manifest).unwrap();
    fs::write(work.join("copied-halfword-lookups.json"), &packet).unwrap();
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "status": "PASS", "revision": CURRENT, "manifest_sha256": digest(manifest.as_bytes()),
            "new_source_bytes": added_bytes(PAYLOAD_ROLE), "new_literal_bytes": added_bytes(LITERAL_ROLE),
            "new_spans": new_spans.len(), "original_copy_paths": rows.len(), "typed_requests": requests,
            "payload_read_widths": payload_widths, "mixed_mirror_checks": boundary_reads,
            "raw_gaps_preserved": true, "full_state_and_clock_equal_each_copy": true,
            "predecessor_attach": true, "missing_group_rejected": true,
        }))
        .unwrap(),
    )
    .unwrap();
}
