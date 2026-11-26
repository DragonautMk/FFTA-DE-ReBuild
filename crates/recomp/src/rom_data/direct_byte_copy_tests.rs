//! Verify all E571 literal readers and shared ROM-copy paths.

use super::{
    attach_dataset, encode_copied_halfword_lookups,
    indexed_reader_bank_tests::{execute_until, initialize},
    read_dataset, COPIED_HALFWORD_LOOKUPS_ID, THUMB_PC_LITERALS_ID,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::{
        copied_halfword_lookups::fixed_byte_copies::{DIRECT_LITERAL_ROLE, DIRECT_PAYLOAD_ROLE},
        read_audit::{Reader, Service, TraceCapture, TraceFilter},
        Extraction, SourceRange,
    },
    Bus, Machine,
};
use serde::Deserialize;
use std::{collections::BTreeSet, fs, path::PathBuf, sync::Arc};

const BASE: u32 = 0x0800_0000;
const IWRAM: u32 = 0x0300_0000;
const CURRENT: (u32, u32) = (343, 342);
const PREVIOUS: (u32, u32) = (342, 341);
const EARLIER_LITERAL: u32 = 0x12969c;
const RAW_GAP: u32 = 0x0088e0;
const PREVIOUS_MANIFEST_SHA256: &str =
    "c3f3164da2ce5dac68c3b4c10f0764e768ddd52aadef4fc05a1e5e536779eb6b";
const PLAN_SHA256: &str = "9bf364906b236124b9c46dd0bfcb0d5503f216db15a6309e4c03aa6ed8d3232d";

#[derive(Deserialize)]
struct Plan {
    rows: Vec<Caller>,
    source_ranges: Vec<PlannedSpan>,
}

#[derive(Deserialize)]
struct Caller {
    call: u32,
    length: u32,
    source_kind: String,
    source_low: u32,
    source_high: u32,
    argument_reads: Vec<LiteralRead>,
}

#[derive(Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct LiteralRead {
    reader: u32,
    offset: u32,
    length: u32,
    role: String,
}

#[derive(Deserialize)]
struct PlannedSpan {
    offset: u32,
    length: u32,
    sha256: String,
    role: String,
    callers: Vec<u32>,
    residual: Vec<[u32; 2]>,
}

pub(super) fn input(name: &str) -> PathBuf {
    std::env::var_os(name).expect(name).into()
}

pub(super) fn flip(machine: &mut Machine, offset: u32, length: u32) {
    for byte in &mut machine.bus.rom[offset as usize..(offset + length) as usize] {
        *byte ^= 0xff;
    }
}

pub(super) fn capture(machine: &Machine, ranges: Vec<(u32, u32)>, limit: usize) -> TraceCapture {
    let trace = TraceCapture::start(
        TraceFilter {
            readers: vec![Reader::Bus],
            ranges,
        },
        limit,
    )
    .unwrap();
    trace.frame(Some(machine.bus.frames));
    trace
}

pub(super) fn same_state(raw: &Machine, managed: &Machine, instruction: u32) {
    assert_eq!(
        managed.bus.clock, raw.bus.clock,
        "clock at {instruction:08x}"
    );
    assert!(
        managed.save_state().unwrap() == raw.save_state().unwrap(),
        "state at {instruction:08x}"
    );
}

#[test]
#[ignore = "requires the exact EUR ROM, retained e342 files and the E571 argument plan"]
fn european_direct_copy_arguments_use_extracted_sources() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let plan = fs::read(input("FFTA_DIRECT_COPY_PLAN")).unwrap();
    assert_eq!(digest(&plan), PLAN_SHA256);
    let plan: Plan = serde_json::from_slice(&plan).unwrap();
    assert_eq!(plan.rows.len(), 61);
    assert_eq!(plan.source_ranges.len(), 55);
    let reads: BTreeSet<_> = plan
        .rows
        .iter()
        .flat_map(|row| &row.argument_reads)
        .collect();
    assert_eq!(reads.len(), 53);

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
    let earlier_owner = dataset
        .manifest
        .resources
        .iter()
        .find(|r| r.id == THUMB_PC_LITERALS_ID)
        .unwrap();
    let mut additions = Vec::new();
    for span in &plan.source_ranges {
        assert_eq!(span.length, 4);
        let role = match span.role.as_str() {
            "pc-literal" => DIRECT_LITERAL_ROLE,
            "payload" => DIRECT_PAYLOAD_ROLE,
            role => panic!("Unexpected E571 argument role {role}"),
        };
        if span.offset == EARLIER_LITERAL {
            assert!(span.residual.is_empty());
            assert!(earlier_owner.sources.iter().any(|source| {
                source.offset == span.offset
                    && source.length == span.length
                    && source.sha256 == span.sha256
            }));
        } else {
            assert_eq!(span.residual, [[span.offset, span.offset + 4]]);
            additions.push(SourceRange {
                offset: span.offset,
                length: span.length,
                sha256: span.sha256.clone(),
                role: role.into(),
            });
        }
    }
    assert_eq!(
        additions
            .iter()
            .filter(|span| span.role == DIRECT_LITERAL_ROLE)
            .count(),
        52
    );
    assert_eq!(
        additions
            .iter()
            .filter(|span| span.role == DIRECT_PAYLOAD_ROLE)
            .count(),
        2
    );

    let mut managed = Machine::new(rom.clone());
    attach_dataset(&mut managed, &dataset).unwrap();
    let predecessor = managed.bus.rom_resources().unwrap().clone();
    let first = additions[0].offset;
    flip(&mut managed, first, 4);
    assert_eq!(managed.bus.read8(BASE + first), !rom[first as usize]);
    flip(&mut managed, first, 4);
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
    let packet = encode_copied_halfword_lookups(&catalog.decoded).unwrap();
    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|r| r.id == COPIED_HALFWORD_LOOKUPS_ID)
        .unwrap();
    assert!(catalog.sources.starts_with(&resource.sources));
    assert_eq!(catalog.sources[resource.sources.len()..], additions);
    resource.sha256 = digest(&packet);
    resource.sources = catalog.sources;
    dataset.copied_halfword_lookups = Some(catalog.decoded);
    dataset.copied_halfword_lookups_bytes = Some(packet.clone());
    attach_dataset(&mut managed, &dataset).unwrap();

    let mut raw = Machine::new(rom.clone());
    let initial = raw.save_state().unwrap();
    for read in &reads {
        assert_eq!((read.role.as_str(), read.length), ("pc-literal", 4));
        let reader = read.reader as usize;
        let opcode = u16::from_le_bytes(rom[reader..reader + 2].try_into().unwrap());
        assert_eq!(opcode & 0xf800, 0x4800);
        let register = ((opcode >> 8) & 7) as usize;
        let pc = BASE + read.reader;
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap();
            initialize(machine, pc);
        }
        execute_until(&mut raw, pc + 2, 2);
        flip(&mut managed, read.offset, 4);
        let trace = capture(&managed, vec![(read.offset, read.offset + 4)], 2);
        execute_until(&mut managed, pc + 2, 2);
        let trace = trace.finish();
        assert_eq!(
            (
                trace.matched_requests,
                trace.dropped_requests,
                trace.events.len()
            ),
            (1, 0, 1)
        );
        let event = &trace.events[0];
        let site = event.key.instruction.unwrap();
        assert_eq!(
            event.key.service,
            Service::Typed,
            "argument read at {pc:08x}"
        );
        assert_eq!((site.pc, site.thumb), (pc, true));
        assert_eq!(
            (event.key.read.physical_offset, event.key.read.length),
            (read.offset, 4)
        );
        assert_eq!(
            managed.cpu.regs[register],
            u32::from_le_bytes(
                rom[read.offset as usize..read.offset as usize + 4]
                    .try_into()
                    .unwrap()
            )
        );
        flip(&mut managed, read.offset, 4);
        same_state(&raw, &managed, pc);
    }

    let mut copy_paths = 0;
    let mut copy_requests = 0;
    for span in plan
        .source_ranges
        .iter()
        .filter(|span| span.role == "payload")
    {
        assert_eq!(span.callers.len(), 2);
        let callers: Vec<_> = span
            .callers
            .iter()
            .map(|call| plan.rows.iter().find(|row| row.call == *call).unwrap())
            .collect();
        let first = callers[0];
        let [literal] = first.argument_reads.as_slice() else {
            panic!("Expected one shared ROM argument");
        };
        assert!(callers
            .iter()
            .all(|row| row.argument_reads.as_slice() == std::slice::from_ref(literal)));
        let source = span.offset..span.offset + 4;
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap();
            initialize(machine, BASE + literal.reader);
        }
        for (index, caller) in callers.into_iter().enumerate() {
            assert_eq!(caller.source_kind, "rom");
            assert_eq!(
                (caller.source_low, caller.source_high, caller.length),
                (BASE + span.offset, BASE + span.offset, 4)
            );
            let pc = BASE + caller.call;
            execute_until(&mut raw, pc, 256);
            let destination = raw.cpu.regs[0];
            execute_until(&mut raw, pc + 4, 128);

            flip(&mut managed, literal.offset, 4);
            flip(&mut managed, span.offset, 4);
            let trace = capture(
                &managed,
                vec![
                    (literal.offset, literal.offset + 4),
                    (source.start, source.end),
                ],
                12,
            );
            execute_until(&mut managed, pc, 256);
            assert_eq!(
                (
                    managed.cpu.regs[0],
                    managed.cpu.regs[1],
                    managed.cpu.regs[2]
                ),
                (destination, BASE + span.offset, 4)
            );
            execute_until(&mut managed, pc + 4, 128);
            let trace = trace.finish();
            assert_eq!(trace.dropped_requests, 0);
            let mut payload_read = BTreeSet::new();
            let mut literal_requests = 0;
            for event in &trace.events {
                let site = event.key.instruction.unwrap();
                let at = event.key.read.physical_offset;
                assert_eq!(event.key.service, Service::Typed, "copy at {pc:08x}");
                assert!(site.thumb);
                if at == literal.offset {
                    assert_eq!((site.pc, event.key.read.length), (BASE + literal.reader, 4));
                    literal_requests += event.requests;
                } else {
                    assert!((0x08147dc0..0x08147e22).contains(&site.pc));
                    payload_read.extend(at..at + event.key.read.length as u32);
                }
            }
            assert_eq!(payload_read, source.clone().collect());
            assert_eq!(literal_requests, u64::from(index == 0));
            let start = destination.checked_sub(IWRAM).unwrap() as usize;
            assert_eq!(
                &managed.bus.iwram[start..start + 4],
                &rom[span.offset as usize..span.offset as usize + 4]
            );
            flip(&mut managed, span.offset, 4);
            flip(&mut managed, literal.offset, 4);
            same_state(&raw, &managed, pc);
            copy_paths += 1;
            copy_requests += trace.matched_requests;
        }
    }
    assert_eq!(copy_paths, 4);

    for span in &additions {
        flip(&mut managed, span.offset, span.length);
    }
    assert!(dataset.manifest.resources.iter().all(|resource| resource
        .sources
        .iter()
        .all(|span| span.offset >= RAW_GAP + 4 || span.offset + span.length <= RAW_GAP)));
    let original = u32::from_le_bytes(
        rom[RAW_GAP as usize..RAW_GAP as usize + 4]
            .try_into()
            .unwrap(),
    );
    for machine in [&mut raw, &mut managed] {
        flip(machine, RAW_GAP, 4);
        assert_eq!(machine.bus.read32(BASE + RAW_GAP), !original);
    }
    let mut boundaries = 0;
    for span in &additions {
        for at in [span.offset - 1, span.offset + span.length - 1] {
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
                boundaries += 2;
            }
        }
    }

    let work = input("FFTA_DIRECT_COPY_WORK");
    let manifest = toml::to_string_pretty(&dataset.manifest).unwrap();
    fs::write(work.join("manifest.toml"), &manifest).unwrap();
    fs::write(work.join("copied-halfword-lookups.json"), &packet).unwrap();
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "status": "PASS", "revision": CURRENT, "manifest_sha256": digest(manifest.as_bytes()),
            "new_spans": additions.len(), "new_source_bytes": 8, "new_literal_bytes": 208,
            "literal_readers": reads.len(), "earlier_literal_preserved": true,
            "original_copy_paths": copy_paths, "typed_copy_requests": copy_requests,
            "mixed_mirror_checks": boundaries, "raw_gap_preserved": true,
            "full_state_and_clock_equal_each_path": true,
            "predecessor_attach": true, "missing_group_rejected": true,
        }))
        .unwrap(),
    )
    .unwrap();
}
