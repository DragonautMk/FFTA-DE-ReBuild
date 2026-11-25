//! Verify the full EUR title frontend source bank and its original consumers.

use super::super::{
    attach_from_directory, cmd, encode_thumb_pc_literals, encode_thumb_switch_tables, parse,
    parse_thumb_pc_literals, parse_thumb_switch_tables, Manifest, MANIFEST_FILE,
    THUMB_PC_LITERALS_FILE, THUMB_PC_LITERALS_ID, THUMB_SWITCH_TABLES_FILE, THUMB_SWITCH_TABLES_ID,
};
use super::{
    check_reads, check_source_word, checked_trace, flip_backing, input, reach_guest_pc,
    run_literal_loads, Word, BASE,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::{
        read_audit::{Service, TraceCapture, TraceFilter, TraceReport},
        thumb_pc_literals::SOURCE_ROLE,
        thumb_switch_tables::ADDITIONAL_SOURCE_ROLE,
        SourceRange,
    },
    Machine, StepEvent,
};
use serde::Deserialize;
use std::{collections::BTreeSet, fs, sync::Arc};

const OPENING: u32 = 0x0813_a1dc;
const POSITIONS_COMPLETE: u32 = 0x0813_a306;
const FRONTEND_RANGE: (u32, u32) = (0x139508, 0x13ba90);

#[derive(Deserialize)]
struct Plan {
    words: Vec<Word>,
    readers: Vec<Word>,
    new_readers: Vec<Word>,
    tables: Vec<Switch>,
}

#[derive(Deserialize)]
struct Switch {
    root: u32,
    start: u32,
    guard: u32,
    reader: u32,
    mov: u32,
    fallback: u32,
    destinations: Vec<u32>,
    sha256: String,
}

fn run_opening(machine: &mut Machine) -> TraceReport {
    let capture = TraceCapture::start(
        TraceFilter {
            readers: Vec::new(),
            ranges: vec![FRONTEND_RANGE],
        },
        256,
    )
    .unwrap();
    reach_guest_pc(machine, POSITIONS_COMPLETE, 120, Some(&capture), |_| 0x3ff);
    capture.finish()
}

fn run_switch(machine: &mut Machine, table: &Switch, index: u32, target: u32) -> TraceReport {
    let capture = TraceCapture::start(
        TraceFilter {
            readers: Vec::new(),
            ranges: vec![(
                table.root,
                table.start + 4 * table.destinations.len() as u32,
            )],
        },
        2,
    )
    .unwrap();
    capture.frame(Some(0));
    machine.cpu.regs[0] = index;
    machine.cpu.regs[15] = BASE + table.guard;
    machine.cpu.cpsr = 0xbf;
    machine.cpu.branch = None;
    for _ in 0..8 {
        let pc = machine.cpu.regs[15];
        assert!(matches!(machine.step(), StepEvent::Instr(i) if i.addr == pc));
        let next = machine.cpu.regs[15];
        if !(BASE + table.guard..=BASE + table.mov).contains(&next) {
            assert_eq!(next, target, "Switch at {:06x}, index {index}", table.mov);
            return capture.finish();
        }
    }
    panic!("Switch at {:06x} did not dispatch", table.mov);
}

#[test]
#[ignore = "exact EUR ROM, E532 source plan and E531 predecessor manifest"]
fn european_title_frontend_banks_use_extracted_sources() {
    let rom_path = input("FFTA_STATE_TEST_ROM");
    let rom = fs::read(&rom_path).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let work = input("FFTA_TITLE_FRONTEND_WORK");
    let plan: Plan =
        serde_json::from_slice(&fs::read(input("FFTA_TITLE_FRONTEND_PLAN")).unwrap()).unwrap();
    let previous_bytes = fs::read(input("FFTA_TITLE_FRONTEND_PREDECESSOR")).unwrap();
    let previous: Manifest = parse(&previous_bytes, "E531 manifest").unwrap();
    assert_eq!(
        (previous.extractor_revision, previous.decoder_revision),
        (326, 325)
    );
    assert_eq!(
        (plan.words.len(), plan.readers.len(), plan.new_readers.len()),
        (225, 347, 256)
    );
    assert_eq!(
        plan.tables
            .iter()
            .map(|table| table.destinations.len())
            .collect::<Vec<_>>(),
        [6, 5]
    );
    for word in &plan.readers {
        check_source_word(&rom, word);
    }

    let resources = tempfile::tempdir_in(&work).unwrap();
    cmd(&[
        "prepare".into(),
        rom_path.to_str().unwrap().into(),
        resources.path().to_str().unwrap().into(),
    ])
    .unwrap();
    let revision = resources.path().join(EUROPE_SHA256).join("v1/e327-d326");
    let current = fs::read_to_string(revision.join("current")).unwrap();
    let dataset = revision.join(current.trim());
    let manifest_path = dataset.join(MANIFEST_FILE);
    let literal_path = dataset.join(THUMB_PC_LITERALS_FILE);
    let switch_path = dataset.join(THUMB_SWITCH_TABLES_FILE);
    let manifest_bytes = fs::read(&manifest_path).unwrap();
    let literal_bytes = fs::read(&literal_path).unwrap();
    let switch_bytes = fs::read(&switch_path).unwrap();
    let manifest: Manifest = parse(&manifest_bytes, "E532 manifest").unwrap();
    let literal_index = previous
        .resources
        .iter()
        .position(|entry| entry.id == THUMB_PC_LITERALS_ID)
        .unwrap();
    let switch_index = previous
        .resources
        .iter()
        .position(|entry| entry.id == THUMB_SWITCH_TABLES_ID)
        .unwrap();
    let mut expected = previous.clone();
    expected.extractor_revision = 327;
    expected.decoder_revision = 326;
    expected.resources[literal_index].sha256 = digest(&literal_bytes);
    expected.resources[switch_index].sha256 = digest(&switch_bytes);
    expected.resources[literal_index]
        .sources
        .extend(plan.words.iter().map(|word| SourceRange {
            offset: word.offset,
            length: 4,
            sha256: word.sha256.clone(),
            role: SOURCE_ROLE.into(),
        }));
    for table in &plan.tables {
        let bytes: Vec<_> = table
            .destinations
            .iter()
            .flat_map(|target| target.to_le_bytes())
            .collect();
        assert_eq!(
            rom[table.start as usize..table.start as usize + bytes.len()],
            bytes
        );
        assert_eq!(digest(&bytes), table.sha256);
        expected.resources[switch_index].sources.push(SourceRange {
            offset: table.start,
            length: bytes.len() as u32,
            sha256: table.sha256.clone(),
            role: ADDITIONAL_SOURCE_ROLE.into(),
        });
    }
    assert_eq!(manifest, expected);

    let mut old_literals = parse_thumb_pc_literals(&literal_bytes).unwrap();
    assert_eq!(old_literals.title_frontend_roots.len(), plan.words.len());
    old_literals.title_frontend_roots.clear();
    let old_literals = encode_thumb_pc_literals(&old_literals).unwrap();
    assert_eq!(
        digest(&old_literals),
        previous.resources[literal_index].sha256
    );
    let mut old_switches = parse_thumb_switch_tables(&switch_bytes).unwrap();
    assert_eq!(old_switches.additional_tables.len(), 52);
    old_switches.additional_tables.truncate(50);
    let old_switches = encode_thumb_switch_tables(&old_switches).unwrap();
    assert_eq!(
        digest(&old_switches),
        previous.resources[switch_index].sha256
    );
    let mut managed = crate::make_machine(rom.clone(), None);
    fs::write(&literal_path, old_literals).unwrap();
    fs::write(&switch_path, old_switches).unwrap();
    fs::write(&manifest_path, previous_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let previous_owner = managed.bus.rom_resources().unwrap().clone();
    let mut incomplete = previous;
    incomplete.extractor_revision = 327;
    incomplete.decoder_revision = 326;
    fs::write(&manifest_path, toml::to_string_pretty(&incomplete).unwrap()).unwrap();
    assert!(attach_from_directory(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(
        &previous_owner,
        managed.bus.rom_resources().unwrap()
    ));
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    fs::write(&literal_path, literal_bytes).unwrap();
    fs::write(&switch_path, switch_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    drop(previous_owner);

    let mut raw = crate::make_machine(rom, None);
    reach_guest_pc(&mut raw, OPENING, 4200, None, |frame| {
        if (180..182).contains(&frame) {
            0x3fe
        } else {
            0x3ff
        }
    });
    let opening_frame = raw.bus.frames;
    let origin = raw.save_state().unwrap();
    raw.load_state(&origin).unwrap();
    managed.load_state(&origin).unwrap();
    let flip_frontend = |machine: &mut Machine| {
        flip_backing(machine, &plan.words);
        for table in &plan.tables {
            let start = table.start as usize;
            for byte in &mut machine.bus.rom[start..start + table.destinations.len() * 4] {
                *byte ^= 0xff;
            }
        }
    };
    flip_frontend(&mut managed);
    let readers = plan
        .readers
        .iter()
        .map(|word| (word.offset, BASE + word.reader))
        .chain(plan.tables.iter().flat_map(|table| {
            (0..table.destinations.len() as u32)
                .map(move |index| (table.start + index * 4, BASE + table.reader))
        }))
        .collect();
    let raw_trace = run_opening(&mut raw);
    let managed_trace = run_opening(&mut managed);
    let ordinary = checked_trace(&managed_trace, &readers, Service::Typed);
    assert_eq!(checked_trace(&raw_trace, &readers, Service::Raw), ordinary);
    for (source, reader) in [(0x13a28c, 0x13a240), (0x13a314, 0x13a2c8)] {
        assert!(ordinary.contains_key(&(source, BASE + reader)));
    }
    // The original bounds check is reached after all nineteen position pairs.
    for machine in [&raw, &managed] {
        let offset = (machine.cpu.regs[6] - 0x0200_0000) as usize;
        assert_eq!(machine.bus.ewram[offset + 4..offset + 6], [19, 0]);
    }
    assert_eq!(raw.bus.frames, managed.bus.frames);
    assert_eq!(raw.bus.clock, managed.bus.clock);
    assert_eq!(raw.bus.audio_buf, managed.bus.audio_buf);
    let complete_frame = raw.bus.frames;
    flip_frontend(&mut managed);
    assert_eq!(
        digest(&raw.save_state().unwrap()),
        digest(&managed.save_state().unwrap())
    );
    flip_frontend(&mut managed);

    check_reads(
        &run_literal_loads(&mut raw, &plan.new_readers, vec![FRONTEND_RANGE]),
        &plan.new_readers,
        Service::Raw,
    );
    check_reads(
        &run_literal_loads(&mut managed, &plan.new_readers, vec![FRONTEND_RANGE]),
        &plan.new_readers,
        Service::Typed,
    );
    for table in &plan.tables {
        let count = table.destinations.len() as u32;
        for index in (0..=count).chain(std::iter::once(255)) {
            let (target, expected) = if let Some(&target) = table.destinations.get(index as usize) {
                (
                    target,
                    BTreeSet::from([
                        (table.root, BASE + table.mov - 6),
                        (table.start + 4 * index, BASE + table.reader),
                    ]),
                )
            } else {
                (table.fallback, BTreeSet::new())
            };
            let raw_trace = run_switch(&mut raw, table, index, target);
            let managed_trace = run_switch(&mut managed, table, index, target);
            let raw_access = checked_trace(&raw_trace, &expected, Service::Raw);
            let managed_access = checked_trace(&managed_trace, &expected, Service::Typed);
            assert_eq!(managed_access, raw_access);
            assert_eq!(
                managed_access.keys().copied().collect::<BTreeSet<_>>(),
                expected
            );
            assert_eq!(managed_trace.matched_requests, expected.len() as u64);
        }
    }
    let report = serde_json::json!({
        "status": "PASS", "revision": [327, 326], "new_words": 225, "switch_targets": 11,
        "source_bytes": 944, "controlled_readers": 256, "fallback_cases": 4,
        "opening_frame": opening_frame, "complete_frame": complete_frame, "position_pairs": 19,
        "ordinary_read_sites": ordinary.len(), "ordinary_typed_requests": managed_trace.matched_requests,
        "full_state_and_audio_equal": true, "predecessor_attach": true, "missing_group_rejected": true,
        "manifest_sha256": digest(&manifest_bytes),
    });
    fs::write(work.join("manifest.toml"), manifest_bytes).unwrap();
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}
