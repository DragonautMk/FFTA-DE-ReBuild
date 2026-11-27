//! Verify extracted dispatch tables and literals against original readers.

use super::{
    attach_dataset,
    direct_byte_copy_tests::{capture, flip, input, same_state},
    encode_thumb_pc_literals, encode_thumb_switch_tables,
    indexed_reader_bank_tests::{execute_until, initialize},
    read_dataset, THUMB_PC_LITERALS_ID, THUMB_SWITCH_TABLES_ID,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::{
        read_audit::Service, thumb_pc_literals, thumb_switch_tables, Extraction, SourceRange,
    },
    Bus, Machine,
};
use serde::Deserialize;
use std::{collections::BTreeSet, fs, sync::Arc};

const BASE: u32 = 0x0800_0000;
const PREVIOUS: (u32, u32) = (345, 344);
const CURRENT: (u32, u32) = (346, 345);
const PREVIOUS_MANIFEST_SHA256: &str =
    "79921deba3e004deaf62273ae5ac39ae65a1323282975bfdb3dc45abdc277af6";
const PLAN_SHA256: &str = "ddba2f174eb1df9023cb73d2a3cb13514e8568235b9d90cf15a399e301b61a66";

#[derive(Deserialize)]
struct Plan {
    tables: Vec<Table>,
    literal_spans: Vec<Literal>,
}

#[derive(Deserialize)]
struct Table {
    root: u32,
    code_start: u32,
    pc_literal: u32,
    length: u32,
    root_sha256: String,
    source_sha256: String,
    destinations: Vec<u32>,
    residual_bytes: u32,
}

#[derive(Deserialize)]
struct Literal {
    offset: u32,
    length: u32,
    readers: Vec<u32>,
    sha256: String,
    residual_bytes: u32,
}

struct ReaderPair {
    raw: Machine,
    extracted: Machine,
    initial: Vec<u8>,
}

impl ReaderPair {
    fn check_route(
        &mut self,
        entry: u32,
        inputs: &[u32],
        destination: u32,
        ranges: &[(u32, u32)],
        expected: &[(u32, u32, usize)],
    ) {
        let entry = BASE + entry;
        for machine in [&mut self.raw, &mut self.extracted] {
            machine.load_state(&self.initial).unwrap();
            initialize(machine, entry);
            machine.cpu.regs[..inputs.len()].copy_from_slice(inputs);
        }
        execute_until(&mut self.raw, BASE + destination, 16);
        for &(start, end) in ranges {
            flip(&mut self.extracted, start, end - start);
        }
        let trace = capture(&self.extracted, ranges.to_vec(), 4);
        execute_until(&mut self.extracted, BASE + destination, 16);
        let report = trace.finish();
        assert_eq!(report.dropped_requests, 0);
        assert_eq!(report.matched_requests as usize, expected.len());
        let observed: BTreeSet<_> = report
            .events
            .iter()
            .map(|event| {
                let site = event.key.instruction.unwrap();
                assert!(site.thumb);
                assert_eq!(event.key.service, Service::Typed, "read at {:08x}", site.pc);
                (
                    site.pc - BASE,
                    event.key.read.physical_offset,
                    event.key.read.length,
                )
            })
            .collect();
        assert_eq!(observed, expected.iter().copied().collect());
        for &(start, end) in ranges {
            flip(&mut self.extracted, start, end - start);
        }
        same_state(&self.raw, &self.extracted, entry);
    }
}

#[test]
#[ignore = "requires the exact EUR ROM, retained e345 files and the E580 source plan"]
fn european_helper_field_sources_feed_original_dispatch() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let plan = fs::read(input("FFTA_HELPER_FIELD_PLAN")).unwrap();
    assert_eq!(digest(&plan), PLAN_SHA256);
    let plan: Plan = serde_json::from_slice(&plan).unwrap();
    assert_eq!(plan.tables.len(), 6);
    assert_eq!(plan.literal_spans.len(), 24);
    let tables: Vec<_> = plan
        .tables
        .iter()
        .filter(|table| table.residual_bytes > 0)
        .collect();
    let literals: Vec<_> = plan
        .literal_spans
        .iter()
        .filter(|span| span.residual_bytes > 0)
        .collect();
    assert_eq!((tables.len(), literals.len()), (2, 10));

    let mut new_words = Vec::new();
    let mut new_tables = Vec::new();
    for table in &tables {
        assert_eq!(table.length, 4 + 4 * table.destinations.len() as u32);
        assert_eq!(table.residual_bytes, table.length);
        new_words.push(SourceRange {
            offset: table.root,
            length: 4,
            sha256: table.root_sha256.clone(),
            role: thumb_pc_literals::SOURCE_ROLE.into(),
        });
        new_tables.push(SourceRange {
            offset: table.root + 4,
            length: table.length - 4,
            sha256: table.source_sha256.clone(),
            role: thumb_switch_tables::ADDITIONAL_SOURCE_ROLE.into(),
        });
    }
    for span in &literals {
        assert_eq!(
            (span.length, span.residual_bytes, span.readers.len()),
            (4, 4, 1)
        );
        new_words.push(SourceRange {
            offset: span.offset,
            length: span.length,
            sha256: span.sha256.clone(),
            role: thumb_pc_literals::SOURCE_ROLE.into(),
        });
    }
    new_words.sort_by_key(|span| span.offset);

    let directory = input("FFTA_STATE_TEST_DATASET");
    assert_eq!(
        digest(&fs::read(directory.join("manifest.toml")).unwrap()),
        PREVIOUS_MANIFEST_SHA256
    );
    let mut dataset = read_dataset(&directory).unwrap();
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        PREVIOUS
    );
    let mut extracted = Machine::new(rom.clone());
    attach_dataset(&mut extracted, &dataset).unwrap();
    let previous = extracted.bus.rom_resources().unwrap().clone();
    let first = new_words[0].offset;
    flip(&mut extracted, first, 4);
    assert_eq!(extracted.bus.read8(BASE + first), !rom[first as usize]);
    flip(&mut extracted, first, 4);
    (
        dataset.manifest.extractor_revision,
        dataset.manifest.decoder_revision,
    ) = CURRENT;
    assert!(attach_dataset(&mut extracted, &dataset).is_err());
    assert!(Arc::ptr_eq(
        &previous,
        extracted.bus.rom_resources().unwrap()
    ));

    let source = Extraction::from_rom_for_manifest(&rom, &dataset.manifest).unwrap();
    let switches = source.thumb_switch_tables.unwrap();
    let words = source.thumb_pc_literals.unwrap();
    let mut before_switches = switches.decoded.clone();
    before_switches
        .additional_tables
        .truncate(thumb_switch_tables::WORLD_NODE_ADDITIONAL_TABLE_COUNT);
    let mut before_words = words.decoded.clone();
    before_words.helper_field_literals.clear();
    assert_eq!(Some(&before_switches), dataset.thumb_switch_tables.as_ref());
    assert_eq!(Some(&before_words), dataset.thumb_pc_literals.as_ref());
    let switch_packet = encode_thumb_switch_tables(&switches.decoded).unwrap();
    let literal_packet = encode_thumb_pc_literals(&words.decoded).unwrap();
    for (id, new_sources, additions, packet) in [
        (
            THUMB_SWITCH_TABLES_ID,
            switches.sources,
            &new_tables,
            &switch_packet,
        ),
        (
            THUMB_PC_LITERALS_ID,
            words.sources,
            &new_words,
            &literal_packet,
        ),
    ] {
        let entry = dataset
            .manifest
            .resources
            .iter_mut()
            .find(|entry| entry.id == id)
            .unwrap();
        assert!(new_sources.starts_with(&entry.sources));
        assert_eq!(&new_sources[entry.sources.len()..], additions.as_slice());
        entry.sha256 = digest(packet);
        entry.sources = new_sources;
    }
    dataset.thumb_switch_tables = Some(switches.decoded);
    dataset.thumb_pc_literals = Some(words.decoded);
    dataset.thumb_switch_tables_bytes = Some(switch_packet.clone());
    dataset.thumb_pc_literals_bytes = Some(literal_packet.clone());
    attach_dataset(&mut extracted, &dataset).unwrap();

    let raw = Machine::new(rom.clone());
    let mut pair = ReaderPair {
        initial: raw.save_state().unwrap(),
        raw,
        extracted,
    };
    let mut dispatch_cases = 0;
    let mut guard_cases = 0;
    for table in tables {
        let (bias, outside, inputs) = match table.root {
            0x096e54 => (0, 0x0972cc, [7, 8, u32::MAX]),
            0x097644 => (7, 0x09a230, [6, 18, u32::MAX]),
            root => panic!("Unexpected switch root {root:06x}"),
        };
        let ranges = [(table.root, table.root + table.length)];
        for (index, &destination) in table.destinations.iter().enumerate() {
            pair.check_route(
                table.code_start,
                &[index as u32 + bias],
                destination,
                &ranges,
                &[
                    (table.pc_literal, table.root, 4),
                    (table.pc_literal + 4, table.root + 4 + 4 * index as u32, 4),
                ],
            );
            dispatch_cases += 1;
        }
        for value in inputs {
            pair.check_route(table.code_start, &[value], outside, &ranges, &[]);
            guard_cases += 1;
        }
    }
    for word in &literals {
        let reader = word.readers[0];
        pair.check_route(
            reader,
            &[],
            reader + 2,
            &[(word.offset, word.offset + 4)],
            &[(reader, word.offset, 4)],
        );
    }
    assert_eq!((dispatch_cases, guard_cases), (18, 6));

    for gap in [0x096e74, 0x097674] {
        assert!(dataset.manifest.resources.iter().all(|entry| {
            entry
                .sources
                .iter()
                .all(|span| span.offset >= gap + 4 || span.offset + span.length <= gap)
        }));
        let at = gap as usize;
        let backing = u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        for machine in [&mut pair.raw, &mut pair.extracted] {
            flip(machine, gap, 4);
            assert_eq!(machine.bus.read32(BASE + gap), !backing);
        }
    }
    let additions: Vec<_> = new_words.iter().chain(&new_tables).collect();
    assert_eq!(additions.iter().map(|span| span.length).sum::<u32>(), 120);
    for span in &additions {
        flip(&mut pair.extracted, span.offset, span.length);
    }
    let mut boundaries = 0;
    for span in additions {
        for offset in [span.offset - 1, span.offset + span.length - 1] {
            for mirror in [BASE, 0x0a00_0000, 0x0c00_0000] {
                let address = mirror + offset;
                assert_eq!(
                    pair.extracted.bus.read16(address),
                    pair.raw.bus.read16(address)
                );
                assert_eq!(
                    pair.extracted.bus.read32(address),
                    pair.raw.bus.read32(address)
                );
                boundaries += 2;
            }
        }
    }

    let work = input("FFTA_HELPER_FIELD_WORK");
    let manifest = toml::to_string_pretty(&dataset.manifest).unwrap();
    fs::write(work.join("manifest.toml"), &manifest).unwrap();
    fs::write(work.join("thumb-switch-tables.json"), switch_packet).unwrap();
    fs::write(work.join("thumb-pc-literals.json"), literal_packet).unwrap();
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "status": "PASS", "revision": CURRENT, "manifest_sha256": digest(manifest.as_bytes()),
            "source_bytes": 120, "dispatch_cases": dispatch_cases, "guard_cases": guard_cases,
            "producer_loads": literals.len(), "typed_reads": dispatch_cases * 2 + literals.len(),
            "mixed_mirror_checks": boundaries, "raw_gaps": 2, "full_state_and_clock_equal": true,
            "predecessor_attach": true, "missing_groups_rejected": true,
        }))
        .unwrap(),
    )
    .unwrap();
}

#[derive(Deserialize)]
struct CachedObjectPlan {
    switch_tables: Vec<SwitchReader>,
    literal_ranges: Vec<Literal>,
}

#[derive(Deserialize)]
struct SwitchReader {
    reader: u32,
    offset: u32,
    length: u32,
    sha256: String,
    targets: Vec<u32>,
    residual_bytes: u32,
}

#[derive(Deserialize)]
struct IndexedExchangePlan {
    switch_tables: Vec<SwitchReader>,
    literal_ranges: Vec<Literal>,
    directions: Vec<DirectionReader>,
}

#[derive(Deserialize)]
struct DirectionReader {
    literal: u32,
    reader: u32,
    offset: u32,
    length: u32,
    sha256: String,
}

#[test]
#[ignore = "requires the exact EUR ROM, the E591 source plan and e349/e350 generations"]
fn european_indexed_exchange_sources_feed_original_readers() {
    const REVISION: (u32, u32) = (350, 349);
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let plan = fs::read(input("FFTA_INDEXED_EXCHANGE_PLAN")).unwrap();
    assert_eq!(
        digest(&plan),
        "9b776fb9e4882c6c0e1a17f23464879b14b1f353e631340c500eb50f1324c9d1"
    );
    let plan: IndexedExchangePlan = serde_json::from_slice(&plan).unwrap();
    let tables: Vec<_> = plan.switch_tables.iter().filter(|table| table.residual_bytes != 0).collect();
    let literals: Vec<_> = plan.literal_ranges.iter().filter(|literal| literal.residual_bytes == 4).collect();
    assert_eq!((tables.len(), literals.len(), plan.directions.len()), (2, 13, 2));

    let previous_dir = input("FFTA_STATE_TEST_DATASET");
    assert_eq!(
        digest(&fs::read(previous_dir.join("manifest.toml")).unwrap()),
        "9bc7675341bc90ad6761701b7b55fc88615a5220cbd52da901d8523d9b4bb0fb"
    );
    let mut previous = read_dataset(&previous_dir).unwrap();
    assert_eq!(
        (previous.manifest.extractor_revision, previous.manifest.decoder_revision),
        (349, 348)
    );
    let current = read_dataset(&input("FFTA_INDEXED_EXCHANGE_DATASET")).unwrap();
    assert_eq!(
        (current.manifest.extractor_revision, current.manifest.decoder_revision),
        REVISION
    );
    let mut extracted = Machine::new(rom.clone());
    attach_dataset(&mut extracted, &previous).unwrap();
    let prior = extracted.bus.rom_resources().unwrap().clone();
    let first = plan.directions[0].offset;
    flip(&mut extracted, first, 1);
    assert_eq!(extracted.bus.read8(BASE + first), !rom[first as usize]);
    flip(&mut extracted, first, 1);
    (previous.manifest.extractor_revision, previous.manifest.decoder_revision) = REVISION;
    assert!(attach_dataset(&mut extracted, &previous).is_err());
    assert!(Arc::ptr_eq(&prior, extracted.bus.rom_resources().unwrap()));
    attach_dataset(&mut extracted, &current).unwrap();

    let raw = Machine::new(rom.clone());
    let mut pair = ReaderPair { initial: raw.save_state().unwrap(), raw, extracted };
    let mut additions = Vec::new();
    let (mut dispatch_cases, mut guard_cases, mut direction_reads) = (0, 0, 0);
    for table in tables {
        let (entry, bias, outside, invalid): (_, _, _, &[u32]) = match table.offset {
            0x0bb5ac => (0x0bb594, 4, 0x0bb5ec, &[3, 17]),
            0x0c3084 => (0x0c3070, 0, 0x0c362a, &[5]),
            offset => panic!("Unexpected indexed exchange table {offset:06x}"),
        };
        let end = table.offset + table.length;
        assert_eq!(digest(&rom[table.offset as usize..end as usize]), table.sha256);
        assert_eq!(table.length, 4 * table.targets.len() as u32);
        additions.push((table.offset, end));
        let range = [(table.offset - 4, end)];
        for (index, &target) in table.targets.iter().enumerate() {
            pair.check_route(
                entry,
                &[index as u32 + bias],
                target,
                &range,
                &[
                    (table.reader, table.offset - 4, 4),
                    (table.reader + 4, table.offset + 4 * index as u32, 4),
                ],
            );
            dispatch_cases += 1;
        }
        for &value in invalid {
            pair.check_route(entry, &[value], outside, &range, &[]);
            guard_cases += 1;
        }
    }
    for direction in &plan.directions {
        let end = direction.offset + direction.length;
        assert_eq!(digest(&rom[direction.offset as usize..end as usize]), direction.sha256);
        let opcode = u16::from_le_bytes(
            rom[direction.literal as usize..direction.literal as usize + 2].try_into().unwrap(),
        );
        let pool = ((direction.literal + 4) & !3) + 4 * u32::from(opcode & 0xff);
        let ranges = [(pool, pool + 4), (direction.offset, end)];
        additions.push((direction.offset, end));
        for index in 0..direction.length {
            pair.check_route(
                direction.literal,
                &[0, 0, 0, 0, 0, 0, 0, index],
                direction.reader + 2,
                &ranges,
                &[(direction.literal, pool, 4), (direction.reader, direction.offset + index, 1)],
            );
            direction_reads += 1;
        }
        let guard = match direction.offset {
            0x399568 => 0x0b839a,
            0x39956c => 0x0b841e,
            offset => panic!("Unexpected direction table {offset:06x}"),
        };
        pair.check_route(guard, &[0, 0, 0, 0, 0, 0, 0, direction.length], guard + 4, &ranges, &[]);
        guard_cases += 1;
    }
    for literal in &literals {
        let end = literal.offset + literal.length;
        assert_eq!((literal.length, literal.readers.len()), (4, 1));
        assert_eq!(digest(&rom[literal.offset as usize..end as usize]), literal.sha256);
        let reader = literal.readers[0];
        additions.push((literal.offset, end));
        pair.check_route(reader, &[], reader + 2, &[(literal.offset, end)], &[(reader, literal.offset, 4)]);
    }
    assert_eq!((dispatch_cases, guard_cases, direction_reads), (18, 5, 6));
    assert_eq!(additions.iter().map(|(start, end)| end - start).sum::<u32>(), 130);

    let gap = 0x39956e;
    let expected = !u16::from_le_bytes(rom[gap as usize..gap as usize + 2].try_into().unwrap());
    for machine in [&mut pair.raw, &mut pair.extracted] {
        flip(machine, gap, 2);
        assert_eq!(machine.bus.read16(BASE + gap), expected);
        flip(machine, gap, 2);
    }
    for &(start, end) in &additions {
        flip(&mut pair.extracted, start, end - start);
    }
    let mut boundaries = 0;
    for (start, end) in additions {
        for offset in [start - 1, end - 1] {
            for mirror in [BASE, 0x0a00_0000, 0x0c00_0000] {
                let address = mirror + offset;
                assert_eq!(pair.extracted.bus.read16(address), pair.raw.bus.read16(address));
                assert_eq!(pair.extracted.bus.read32(address), pair.raw.bus.read32(address));
                boundaries += 2;
            }
        }
    }
    fs::write(
        input("FFTA_INDEXED_EXCHANGE_WORK").join("test-result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "status": "PASS", "revision": REVISION, "source_bytes": 130,
            "dispatch_cases": dispatch_cases, "guard_cases": guard_cases,
            "literal_loads": literals.len(), "direction_reads": direction_reads,
            "typed_reads": 2 * (dispatch_cases + direction_reads) + literals.len(),
            "mixed_mirror_checks": boundaries, "raw_neighbor_unchanged": true,
            "full_state_and_clock_equal": true, "predecessor_attach": true,
            "missing_groups_rejected": true,
        }))
        .unwrap(),
    )
    .unwrap();
}

#[test]
#[ignore = "requires the exact EUR ROM, the E593 source plan and e350/e351 generations"]
fn european_five_byte_list_sources_feed_original_readers() {
    const REVISION: (u32, u32) = (351, 350);
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let plan = fs::read(input("FFTA_FIVE_BYTE_LIST_PLAN")).unwrap();
    assert_eq!(
        digest(&plan),
        "2cc76ff2b5ea71263710382f76bcfe73fe67593313cccf56b5fcda94593953db"
    );
    let plan: FiveByteListPlan = serde_json::from_slice(&plan).unwrap();
    let direction = plan.direction;
    let direction_end = direction.offset + direction.length;
    assert_eq!(
        (direction.count, direction.stride, direction.length),
        (6, 4, 24)
    );
    assert_eq!(
        digest(&rom[direction.offset as usize..direction_end as usize]),
        direction.sha256
    );

    let previous_dir = input("FFTA_STATE_TEST_DATASET");
    assert_eq!(
        digest(&fs::read(previous_dir.join("manifest.toml")).unwrap()),
        "cbfc7a91f5ef5380f84029363c11429d56d55b734c4b89bd764e75854f1325e5"
    );
    let mut previous = read_dataset(&previous_dir).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (350, 349)
    );
    let current = read_dataset(&input("FFTA_FIVE_BYTE_LIST_DATASET")).unwrap();
    assert_eq!(
        (
            current.manifest.extractor_revision,
            current.manifest.decoder_revision
        ),
        REVISION
    );
    let mut extracted = Machine::new(rom.clone());
    attach_dataset(&mut extracted, &previous).unwrap();
    let prior = extracted.bus.rom_resources().unwrap().clone();
    flip(&mut extracted, direction.offset, 1);
    assert_eq!(
        extracted.bus.read8(BASE + direction.offset),
        !rom[direction.offset as usize]
    );
    flip(&mut extracted, direction.offset, 1);
    (
        previous.manifest.extractor_revision,
        previous.manifest.decoder_revision,
    ) = REVISION;
    assert!(attach_dataset(&mut extracted, &previous).is_err());
    assert!(Arc::ptr_eq(&prior, extracted.bus.rom_resources().unwrap()));
    attach_dataset(&mut extracted, &current).unwrap();

    let raw = Machine::new(rom.clone());
    let mut pair = ReaderPair {
        initial: raw.save_state().unwrap(),
        raw,
        extracted,
    };
    let table = plan.switch;
    let switch_end = table.offset + 4 * table.targets.len() as u32;
    assert_eq!((table.offset, table.targets.len()), (table.root + 4, 5));
    assert_eq!(
        digest(&rom[table.root as usize..switch_end as usize]),
        table.sha256
    );
    let switch_range = [(table.root, switch_end)];
    for (state, &target) in table.targets.iter().enumerate() {
        pair.check_route(
            table.guard - 4,
            &[0x80 | state as u32],
            target,
            &switch_range,
            &[
                (table.reader, table.root, 4),
                (table.reader + 4, table.offset + state as u32 * 4, 4),
            ],
        );
    }
    pair.check_route(table.guard - 4, &[0x85], 0x14cadc, &switch_range, &[]);

    let mut direction_paths = 0;
    for index in 0..direction.count {
        let row = direction.offset + index * direction.stride;
        pair.check_route(
            0x14c7c0,
            &[0, 0, 8, 0, 0, 0, 0, index],
            0x14c7d6,
            &[(0x14c8b0, 0x14c8b4), (direction.offset, direction_end)],
            &[
                (direction.forward_reader, 0x14c8b0, 4),
                (0x14c7c8, row, 1),
                (0x14c7d4, row + 1, 1),
            ],
        );
        direction_paths += 1;
    }
    let reverse_ranges = [(0x14c444, 0x14c448), (direction.offset, direction_end)];
    for kind in 0..4 {
        let row = direction.offset + (kind + 1) * direction.stride;
        pair.check_route(
            0x14c3fe,
            &[0, kind, 0x0200_1000],
            0x14c41a,
            &reverse_ranges,
            &[
                (direction.reverse_reader, 0x14c444, 4),
                (0x14c410, row, 1),
                (0x14c418, row + 1, 1),
            ],
        );
        direction_paths += 1;
    }
    pair.check_route(0x14c3fe, &[0, 15], 0x14c454, &reverse_ranges, &[]);

    let mut additions = vec![
        (direction.offset, direction_end),
        (table.root, table.offset),
        (table.offset, switch_end),
    ];
    let mut support_checks = 0;
    for literal in plan
        .literals
        .iter()
        .filter(|literal| literal.residual_bytes != 0)
    {
        let end = literal.offset + literal.length;
        assert_eq!(
            (
                literal.length,
                literal.residual_bytes,
                literal.readers.len()
            ),
            (4, 4, 1)
        );
        assert_eq!(
            digest(&rom[literal.offset as usize..end as usize]),
            literal.sha256
        );
        let reader = literal.readers[0];
        additions.push((literal.offset, end));
        pair.check_route(
            reader,
            &[],
            reader + 2,
            &[(literal.offset, end)],
            &[(reader, literal.offset, 4)],
        );
        support_checks += 1;
    }
    assert_eq!(support_checks, 29);
    assert_eq!(
        additions
            .iter()
            .map(|(start, end)| end - start)
            .sum::<u32>(),
        164
    );

    // Both adjoining resources must keep their own bytes when reads span the bank.
    for offset in [direction.offset - 1, direction_end] {
        flip(&mut pair.extracted, offset, 1);
        assert_eq!(
            pair.extracted.bus.read8(BASE + offset),
            rom[offset as usize]
        );
    }
    for &(start, end) in &additions {
        flip(&mut pair.extracted, start, end - start);
    }
    let mut boundaries = 0;
    let mut trailing_reads = 0;
    for mirror in [BASE, 0x0a00_0000, 0x0c00_0000] {
        for &(start, end) in &additions {
            for offset in [start - 1, end - 1] {
                let address = mirror + offset;
                assert_eq!(
                    pair.extracted.bus.read16(address),
                    pair.raw.bus.read16(address)
                );
                assert_eq!(
                    pair.extracted.bus.read32(address),
                    pair.raw.bus.read32(address)
                );
                boundaries += 2;
            }
        }
        for index in 0..direction.count {
            for tail in [2, 3] {
                let offset = direction.offset + index * direction.stride + tail;
                assert_eq!(
                    pair.extracted.bus.read8(mirror + offset),
                    rom[offset as usize]
                );
                trailing_reads += 1;
            }
        }
    }
    fs::write(
        input("FFTA_FIVE_BYTE_LIST_WORK").join("test-result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "status": "PASS", "revision": REVISION, "source_bytes": 164,
            "dispatch_cases": table.targets.len(), "guard_cases": 2,
            "literal_loads": support_checks + 1, "direction_reads": 2 * direction_paths,
            "typed_reads": 2 * table.targets.len() + 3 * direction_paths + support_checks,
            "mixed_mirror_checks": boundaries, "owned_neighbors": 2, "trailing_byte_reads": trailing_reads,
            "full_state_and_clock_equal": true, "predecessor_attach": true, "missing_groups_rejected": true,
        })).unwrap(),
    ).unwrap();
}

#[derive(Deserialize)]
struct FiveByteListPlan {
    direction: FiveByteListDirectionPlan,
    switch: FiveByteListSwitchPlan,
    literals: Vec<Literal>,
}

#[derive(Deserialize)]
struct FiveByteListDirectionPlan {
    offset: u32,
    length: u32,
    stride: u32,
    count: u32,
    sha256: String,
    forward_reader: u32,
    reverse_reader: u32,
}

#[derive(Deserialize)]
struct FiveByteListSwitchPlan {
    root: u32,
    offset: u32,
    targets: Vec<u32>,
    guard: u32,
    reader: u32,
    sha256: String,
}

#[test]
#[ignore = "requires the exact EUR ROM, the E595 source plan and e351/e352 generations"]
fn european_palette_task_literals_feed_original_readers() {
    const REVISION: (u32, u32) = (352, 351);
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let plan = fs::read(input("FFTA_PALETTE_TASK_PLAN")).unwrap();
    assert_eq!(
        digest(&plan),
        "b86363e2192abb1edd9fb34bef34431982a2ee5a5dd430307f37bddaf1d53c53"
    );
    let plan: PaletteTaskLiteralPlan = serde_json::from_slice(&plan).unwrap();
    let literals: Vec<_> = plan
        .literal_ranges
        .into_iter()
        .filter(|span| span.residual_bytes != 0)
        .collect();
    assert_eq!(literals.len(), 15);

    let previous_dir = input("FFTA_STATE_TEST_DATASET");
    assert_eq!(
        digest(&fs::read(previous_dir.join("manifest.toml")).unwrap()),
        "3bdcdbff11b8d2ac687c7d3ced6e31ec9dd92bc54de72962428dd39ebffd28a4"
    );
    let mut previous = read_dataset(&previous_dir).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (351, 350)
    );
    let current = read_dataset(&input("FFTA_PALETTE_TASK_DATASET")).unwrap();
    assert_eq!(
        (
            current.manifest.extractor_revision,
            current.manifest.decoder_revision
        ),
        REVISION
    );
    let mut extracted = Machine::new(rom.clone());
    attach_dataset(&mut extracted, &previous).unwrap();
    let prior = extracted.bus.rom_resources().unwrap().clone();
    let list_root = 0x0fa698;
    flip(&mut extracted, list_root, 1);
    assert_eq!(
        extracted.bus.read8(BASE + list_root),
        !rom[list_root as usize]
    );
    flip(&mut extracted, list_root, 1);
    (
        previous.manifest.extractor_revision,
        previous.manifest.decoder_revision,
    ) = REVISION;
    assert!(attach_dataset(&mut extracted, &previous).is_err());
    assert!(Arc::ptr_eq(&prior, extracted.bus.rom_resources().unwrap()));
    attach_dataset(&mut extracted, &current).unwrap();

    let raw = Machine::new(rom.clone());
    let mut pair = ReaderPair {
        initial: raw.save_state().unwrap(),
        raw,
        extracted,
    };
    for literal in &literals {
        let end = literal.offset + literal.length;
        assert_eq!(
            (
                literal.length,
                literal.residual_bytes,
                literal.readers.len()
            ),
            (4, 4, 1)
        );
        assert_eq!(
            digest(&rom[literal.offset as usize..end as usize]),
            literal.sha256
        );
        let reader = literal.readers[0];
        pair.check_route(
            reader,
            &[],
            reader + 2,
            &[(literal.offset, end)],
            &[(reader, literal.offset, 4)],
        );
    }

    for literal in &literals {
        flip(&mut pair.extracted, literal.offset, literal.length);
    }
    let mut mirror_checks = 0;
    for mirror in [BASE, 0x0a00_0000, 0x0c00_0000] {
        for literal in &literals {
            let start = literal.offset;
            let end = start + literal.length;
            for offset in [start - 1, end - 1] {
                let address = mirror + offset;
                assert_eq!(
                    pair.extracted.bus.read16(address),
                    pair.raw.bus.read16(address)
                );
                assert_eq!(
                    pair.extracted.bus.read32(address),
                    pair.raw.bus.read32(address)
                );
                mirror_checks += 2;
            }
        }
    }
    fs::write(
        input("FFTA_PALETTE_TASK_WORK").join("test-result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "status": "PASS", "revision": REVISION, "source_bytes": 4 * literals.len(),
            "literal_loads": literals.len(), "typed_reads": literals.len(),
            "mixed_mirror_checks": mirror_checks, "full_state_and_clock_equal": true,
            "predecessor_attach": true, "missing_group_rejected": true,
        }))
        .unwrap(),
    )
    .unwrap();
}

#[derive(Deserialize)]
struct PaletteTaskLiteralPlan {
    literal_ranges: Vec<Literal>,
}

#[derive(Deserialize)]
struct BackgroundVariantStatePlan {
    switch_targets: Vec<u32>,
    switch_unowned_spans: Vec<SourceRange>,
}

#[test]
#[ignore = "requires the exact EUR ROM, E599 source plan and e353/e354 generations"]
fn european_background_variant_state_dispatch_uses_extracted_sources() {
    const REVISION: (u32, u32) = (354, 353);
    const ENTRY: u32 = 0x02097c;
    const STATE_SELECTOR: u32 = 0x0200_80c5;
    const STATE_BASE: u32 = 0x02099c;
    const SWITCH_BASE: u32 = 0x0209a0;
    const FIRST_TARGET: u32 = 0x0209a4;
    const TARGET_END: u32 = 0x0209dc;
    const REJECTED_TARGET: u32 = 0x021014;

    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let plan = fs::read(input("FFTA_BACKGROUND_VARIANT_PLAN")).unwrap();
    assert_eq!(
        digest(&plan),
        "82dfef0f5df88ef1b406baacc7ee54a3a39be9e70b0a3cc50904777a45f3b878"
    );
    let plan: BackgroundVariantStatePlan = serde_json::from_slice(&plan).unwrap();
    assert_eq!(plan.switch_targets.len(), 14);
    let ranges = [
        (STATE_BASE, SWITCH_BASE),
        (SWITCH_BASE, FIRST_TARGET),
        (FIRST_TARGET, TARGET_END),
    ];
    assert_eq!(plan.switch_unowned_spans.len(), ranges.len());
    for (span, (start, end)) in plan.switch_unowned_spans.iter().zip(ranges) {
        assert_eq!((span.offset, span.length), (start, end - start));
        assert_eq!(digest(&rom[start as usize..end as usize]), span.sha256);
    }

    let previous_dir = input("FFTA_STATE_TEST_DATASET");
    assert_eq!(
        digest(&fs::read(previous_dir.join("manifest.toml")).unwrap()),
        "8f2c348d8980c266f6a2b32a4bcd85575e7bffd70a288ef8cbef342f401aebd8"
    );
    let mut previous = read_dataset(&previous_dir).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (353, 352)
    );
    let current = read_dataset(&input("FFTA_BACKGROUND_VARIANT_DATASET")).unwrap();
    assert_eq!(
        (
            current.manifest.extractor_revision,
            current.manifest.decoder_revision
        ),
        REVISION
    );
    let mut extracted = Machine::new(rom.clone());
    attach_dataset(&mut extracted, &previous).unwrap();
    let previous_resources = extracted.bus.rom_resources().unwrap().clone();
    flip(&mut extracted, STATE_BASE, TARGET_END - STATE_BASE);
    for offset in [STATE_BASE, SWITCH_BASE, FIRST_TARGET, TARGET_END - 1] {
        assert_eq!(extracted.bus.read8(BASE + offset), !rom[offset as usize]);
    }
    flip(&mut extracted, STATE_BASE, TARGET_END - STATE_BASE);
    (
        previous.manifest.extractor_revision,
        previous.manifest.decoder_revision,
    ) = REVISION;
    assert!(attach_dataset(&mut extracted, &previous).is_err());
    assert!(Arc::ptr_eq(
        &previous_resources,
        extracted.bus.rom_resources().unwrap()
    ));
    attach_dataset(&mut extracted, &current).unwrap();
    let raw = Machine::new(rom);
    let initial = raw.save_state().unwrap();
    let mut pair = ReaderPair {
        raw,
        extracted,
        initial: initial.clone(),
    };

    let cases = plan
        .switch_targets
        .iter()
        .copied()
        .enumerate()
        .map(|(state, target)| (state as u8, target))
        .chain([(14, REJECTED_TARGET), (255, REJECTED_TARGET)]);
    let mut typed_reads = 0;
    for (state, target) in cases {
        pair.raw.load_state(&initial).unwrap();
        pair.raw.bus.write8(STATE_SELECTOR, state);
        pair.initial = pair.raw.save_state().unwrap();
        let mut expected = vec![(0x020988, STATE_BASE, 4)];
        if usize::from(state) < plan.switch_targets.len() {
            expected.extend([
                (0x020994, SWITCH_BASE, 4),
                (0x020998, FIRST_TARGET + 4 * u32::from(state), 4),
            ]);
        }
        pair.check_route(ENTRY, &[], target, &ranges, &expected);
        typed_reads += expected.len();
    }

    flip(&mut pair.extracted, STATE_BASE, TARGET_END - STATE_BASE);
    let mut mirror_checks = 0;
    for mirror in [BASE, 0x0a00_0000, 0x0c00_0000] {
        for offset in [
            STATE_BASE - 1,
            SWITCH_BASE - 1,
            FIRST_TARGET - 1,
            TARGET_END - 1,
        ] {
            let address = mirror + offset;
            assert_eq!(
                pair.extracted.bus.read16(address),
                pair.raw.bus.read16(address)
            );
            assert_eq!(
                pair.extracted.bus.read32(address),
                pair.raw.bus.read32(address)
            );
            mirror_checks += 2;
        }
    }
    fs::write(
        input("FFTA_BACKGROUND_VARIANT_WORK").join("test-result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "status": "PASS",
            "revision": REVISION,
            "source_bytes": TARGET_END - STATE_BASE,
            "original_switch_targets": plan.switch_targets.len(),
            "guarded_states": 2,
            "typed_reads": typed_reads,
            "mixed_mirror_checks": mirror_checks,
            "full_state_and_clock_equal": true,
            "predecessor_attach": true,
            "missing_group_rejected": true,
        }))
        .unwrap(),
    )
    .unwrap();
}

#[derive(Deserialize)]
struct CpuFastSetPlan {
    literal_ranges: Vec<CpuFastSetLiteral>,
}

#[derive(Deserialize)]
struct CpuFastSetLiteral {
    offset: u32,
    length: u32,
    sha256: String,
    residual_bytes: u32,
    readers: Vec<CpuFastSetLiteralReader>,
}

#[derive(Deserialize)]
struct CpuFastSetLiteralReader {
    pc: u32,
    opcode: u16,
}

#[test]
#[ignore = "requires the exact EUR ROM, E603 source plan and e354/e355 generations"]
fn european_cpu_fast_set_literals_feed_original_uploads() {
    const REVISION: (u32, u32) = (355, 354);
    const INITIAL_VRAM: u8 = 0x5a;
    const FILL_WORD: u32 = 0x3927_4153;
    const FILL_OFFSET: usize = 0xffc0;
    const COPY_OFFSET: usize = 0x8000;
    const COPY_SOURCE: usize = 0x100;
    const COPY_BYTES: usize = 0xa40;

    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let plan = fs::read(input("FFTA_CPU_FAST_SET_PLAN")).unwrap();
    assert_eq!(
        digest(&plan),
        "a263a7815a8ac0a8f558672764e351dcb57d12b4b4ffd6895680a8b5fcef767d"
    );
    let plan: CpuFastSetPlan = serde_json::from_slice(&plan).unwrap();
    let literals: Vec<_> = plan
        .literal_ranges
        .iter()
        .filter(|span| span.residual_bytes != 0)
        .collect();
    assert_eq!((plan.literal_ranges.len(), literals.len()), (82, 65));

    let previous_dir = input("FFTA_STATE_TEST_DATASET");
    assert_eq!(
        digest(&fs::read(previous_dir.join("manifest.toml")).unwrap()),
        "8c751b704148915f3431666d2bc3e5d4598ac7bbb73095aacb23183563ac78b4"
    );
    let mut previous = read_dataset(&previous_dir).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (354, 353)
    );
    let current = read_dataset(&input("FFTA_CPU_FAST_SET_DATASET")).unwrap();
    assert_eq!(
        (
            current.manifest.extractor_revision,
            current.manifest.decoder_revision
        ),
        REVISION
    );
    let mut extracted = Machine::new(rom.clone());
    attach_dataset(&mut extracted, &previous).unwrap();
    let prior = extracted.bus.rom_resources().unwrap().clone();
    let new_word = literals[0].offset;
    flip(&mut extracted, new_word, 4);
    assert_eq!(
        extracted.bus.read8(BASE + new_word),
        !rom[new_word as usize]
    );
    flip(&mut extracted, new_word, 4);
    (
        previous.manifest.extractor_revision,
        previous.manifest.decoder_revision,
    ) = REVISION;
    assert!(attach_dataset(&mut extracted, &previous).is_err());
    assert!(Arc::ptr_eq(&prior, extracted.bus.rom_resources().unwrap()));
    attach_dataset(&mut extracted, &current).unwrap();

    let mut raw = Machine::new(rom.clone());
    raw.bus.vram.fill(INITIAL_VRAM);
    let copy_source = &mut raw.bus.ewram[COPY_SOURCE..COPY_SOURCE + COPY_BYTES];
    for (index, byte) in copy_source.iter_mut().enumerate() {
        *byte = (index as u8) ^ (index >> 8) as u8;
    }
    let copy_source = copy_source.to_vec();
    let mut pair = ReaderPair {
        initial: raw.save_state().unwrap(),
        raw,
        extracted,
    };

    let mut literal_loads = 0;
    for span in &literals {
        let end = span.offset + span.length;
        assert_eq!((span.length, span.residual_bytes), (4, 4));
        assert_eq!(
            digest(&rom[span.offset as usize..end as usize]),
            span.sha256
        );
        for reader in &span.readers {
            let at = reader.pc as usize;
            assert_eq!(rom[at..at + 2], reader.opcode.to_le_bytes());
            pair.check_route(
                reader.pc,
                &[],
                reader.pc + 2,
                &[(span.offset, end)],
                &[(reader.pc, span.offset, 4)],
            );
            literal_loads += 1;
        }
    }
    assert_eq!(literal_loads, 67);

    pair.check_route(
        0x0e51cc,
        &[0, 0, 0, 0, 0, FILL_WORD],
        0x0e51d8,
        &[(0x0e51ec, 0x0e51f4)],
        &[(0x0e51d0, 0x0e51ec, 4), (0x0e51d2, 0x0e51f0, 4)],
    );
    let fill_end = FILL_OFFSET + 64;
    for word in pair.extracted.bus.vram[FILL_OFFSET..fill_end].chunks_exact(4) {
        assert_eq!(word, FILL_WORD.to_le_bytes());
    }
    assert_eq!(pair.extracted.bus.vram[FILL_OFFSET - 1], INITIAL_VRAM);
    assert_eq!(pair.extracted.bus.vram[fill_end], INITIAL_VRAM);

    pair.check_route(
        0x03281e,
        &[0, 0, 0, 0, 0, 0x0200_0000 + COPY_SOURCE as u32],
        0x03282a,
        &[(0x0328fc, 0x032900)],
        &[(0x03281e, 0x0328fc, 4)],
    );
    let copy_end = COPY_OFFSET + COPY_BYTES;
    assert_eq!(pair.extracted.bus.vram[COPY_OFFSET..copy_end], copy_source);
    assert_eq!(pair.extracted.bus.vram[COPY_OFFSET - 1], INITIAL_VRAM);
    assert_eq!(pair.extracted.bus.vram[copy_end], INITIAL_VRAM);

    for span in &literals {
        flip(&mut pair.extracted, span.offset, span.length);
    }
    let mut mirror_checks = 0;
    for mirror in [BASE, 0x0a00_0000, 0x0c00_0000] {
        for span in &literals {
            for offset in [span.offset - 1, span.offset + span.length - 1] {
                let address = mirror + offset;
                assert_eq!(
                    pair.extracted.bus.read16(address),
                    pair.raw.bus.read16(address)
                );
                assert_eq!(
                    pair.extracted.bus.read32(address),
                    pair.raw.bus.read32(address)
                );
                mirror_checks += 2;
            }
        }
    }
    fs::write(
        input("FFTA_CPU_FAST_SET_WORK").join("test-result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "status": "PASS", "revision": REVISION,
            "source_bytes": 4 * literals.len(), "literal_loads": literal_loads,
            "original_swi_cases": 2, "transferred_bytes": 64 + COPY_BYTES,
            "mixed_mirror_checks": mirror_checks, "full_state_and_clock_equal": true,
            "predecessor_attach": true, "missing_group_rejected": true,
        }))
        .unwrap(),
    )
    .unwrap();
}

#[test]
#[ignore = "requires the exact EUR ROM, the E589 plan and e348/e349 generations"]
fn european_cached_object_sources_feed_original_dispatch() {
    const REVISION: (u32, u32) = (349, 348);
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let plan = fs::read(input("FFTA_CACHED_OBJECT_PLAN")).unwrap();
    assert_eq!(
        digest(&plan),
        "2761063bd3e59fb69105b3dd37ce1f645a6c2ad79ba014fe53867af96f33c8d3"
    );
    let plan: CachedObjectPlan = serde_json::from_slice(&plan).unwrap();
    assert_eq!(plan.switch_tables.len(), 2);
    assert_eq!(plan.literal_ranges.len(), 52);
    let literals: Vec<_> = plan
        .literal_ranges
        .iter()
        .filter(|span| span.residual_bytes == 4)
        .collect();
    assert_eq!(literals.len(), 33);

    let previous_dir = input("FFTA_STATE_TEST_DATASET");
    assert_eq!(
        digest(&fs::read(previous_dir.join("manifest.toml")).unwrap()),
        "ba39c0a6a2e19e0b9b2f5cbc7862962e84f7ee2966a614a77746ebc6b82f911d"
    );
    let mut previous = read_dataset(&previous_dir).unwrap();
    assert_eq!(
        (previous.manifest.extractor_revision, previous.manifest.decoder_revision),
        (348, 347)
    );
    let path = input("FFTA_CACHED_OBJECT_DATASET");
    let current = read_dataset(&path).unwrap();
    assert_eq!(
        (current.manifest.extractor_revision, current.manifest.decoder_revision),
        REVISION
    );
    let mut extracted = Machine::new(rom.clone());
    attach_dataset(&mut extracted, &previous).unwrap();
    let prior = extracted.bus.rom_resources().unwrap().clone();
    let first = literals[0].offset;
    flip(&mut extracted, first, 4);
    assert_eq!(extracted.bus.read8(BASE + first), !rom[first as usize]);
    flip(&mut extracted, first, 4);
    (previous.manifest.extractor_revision, previous.manifest.decoder_revision) = REVISION;
    assert!(attach_dataset(&mut extracted, &previous).is_err());
    assert!(Arc::ptr_eq(&prior, extracted.bus.rom_resources().unwrap()));
    attach_dataset(&mut extracted, &current).unwrap();

    let raw = Machine::new(rom.clone());
    let mut pair = ReaderPair { initial: raw.save_state().unwrap(), raw, extracted };
    let mut additions = Vec::new();
    let mut dispatch_cases = 0;
    for table in &plan.switch_tables {
        let (entry, outside) = match table.offset {
            0x0089dc => (0x0089c4, 0x008da8),
            0x009564 => (0x009542, 0x00970e),
            offset => panic!("Unexpected cached object table {offset:06x}"),
        };
        let end = table.offset + table.length;
        assert_eq!(digest(&rom[table.offset as usize..end as usize]), table.sha256);
        assert_eq!(table.length, 4 * table.targets.len() as u32);
        additions.push((table.offset, end));
        let range = [(table.offset - 4, end)];
        for (index, &target) in table.targets.iter().enumerate() {
            pair.check_route(
                entry,
                &[index as u32, 1],
                target,
                &range,
                &[
                    (table.reader, table.offset - 4, 4),
                    (table.reader + 4, table.offset + 4 * index as u32, 4),
                ],
            );
            dispatch_cases += 1;
        }
        pair.check_route(entry, &[table.targets.len() as u32, 1], outside, &range, &[]);
    }
    pair.check_route(0x009542, &[0, 0], 0x0095f4, &[(0x009560, 0x00958c)], &[]);
    assert_eq!(dispatch_cases, 21);

    for literal in &literals {
        let end = literal.offset + literal.length;
        assert_eq!((literal.length, literal.readers.len()), (4, 1));
        assert_eq!(digest(&rom[literal.offset as usize..end as usize]), literal.sha256);
        let reader = literal.readers[0];
        additions.push((literal.offset, end));
        pair.check_route(
            reader,
            &[],
            reader + 2,
            &[(literal.offset, end)],
            &[(reader, literal.offset, 4)],
        );
    }
    assert_eq!(additions.iter().map(|(start, end)| end - start).sum::<u32>(), 216);

    for gap in [0x008a08, 0x00958c] {
        for machine in [&mut pair.raw, &mut pair.extracted] {
            let backing = u32::from_le_bytes(rom[gap as usize..gap as usize + 4].try_into().unwrap());
            flip(machine, gap, 4);
            assert_eq!(machine.bus.read32(BASE + gap), !backing);
            flip(machine, gap, 4);
        }
    }
    for &(start, end) in &additions {
        flip(&mut pair.extracted, start, end - start);
    }
    let mut boundaries = 0;
    for (start, end) in additions {
        for offset in [start - 1, end - 1] {
            for mirror in [BASE, 0x0a00_0000, 0x0c00_0000] {
                let address = mirror + offset;
                assert_eq!(pair.extracted.bus.read16(address), pair.raw.bus.read16(address));
                assert_eq!(pair.extracted.bus.read32(address), pair.raw.bus.read32(address));
                boundaries += 2;
            }
        }
    }
    fs::write(
        input("FFTA_CACHED_OBJECT_WORK").join("test-result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "status": "PASS", "revision": REVISION, "source_bytes": 216,
            "dispatch_cases": dispatch_cases, "guard_cases": 3,
            "literal_loads": literals.len(), "typed_reads": dispatch_cases * 2 + literals.len(),
            "mixed_mirror_checks": boundaries, "raw_gaps": 2, "full_state_and_clock_equal": true,
            "predecessor_attach": true, "missing_groups_rejected": true,
        }))
        .unwrap(),
    )
    .unwrap();
}
