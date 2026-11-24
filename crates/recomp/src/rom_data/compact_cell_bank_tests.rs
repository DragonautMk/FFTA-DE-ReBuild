//! Verify the European compact-cell file through the original guest reads.
use super::{
    attach_from_directory, parse, preparation,
    tilemap_transfer_directory_tests::{check_stream_bus_reads, replay_pair},
    Extraction, Manifest, COMPACT_CELL_BANK_FILE, COMPACT_CELL_BANK_ID, MANIFEST_FILE,
};
use gba_core::{
    ffta_rom::digest,
    rom_data::{compact_cell_bank, SourceRange},
    Bus, Machine,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fs, path::PathBuf, sync::Arc};

const ROM_BASE: u32 = 0x0800_0000;
const RAM: u32 = 0x0200_0000;

#[derive(Deserialize)]
struct Caller {
    load: u32,
    literal: u32,
    pointer_cell: Option<u32>,
}

#[derive(Deserialize)]
struct Frame {
    start: u32,
    extent_end: u32,
    token_end: Option<u32>,
    width: u8,
    height: u8,
}

#[derive(Deserialize)]
struct Record {
    root: u32,
    end: u32,
    directory: u32,
    sequence: u32,
    sequence_entries: u32,
    header_flags: u32,
    source_sha256: String,
    frames: Vec<Frame>,
    callers: Vec<Caller>,
}

#[derive(Deserialize)]
struct Slot {
    index: u8,
    root: u32,
    parameter: u32,
}

#[derive(Deserialize)]
struct SelectorTable {
    start: u32,
    end: u32,
    parameter_start: u32,
    pointer_literal: u32,
    parameter_literal: u32,
    selector_opcode: u16,
    call: u32,
    load: u32,
    slots: Vec<Slot>,
}

#[derive(Deserialize)]
struct Bank {
    records: Vec<Record>,
    alignment: Vec<(u32, u32)>,
    selector_tables: Vec<SelectorTable>,
}

fn input(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os(name).unwrap_or_else(|| panic!("Missing {name}")))
}

fn direct_literals(bank: &Bank) -> impl Iterator<Item = &Caller> {
    bank.records
        .iter()
        .flat_map(|record| &record.callers)
        .filter(|caller| caller.pointer_cell.is_none())
}

fn source_ranges(bank: &Bank, rom: &[u8]) -> Vec<SourceRange> {
    let mut spans = BTreeSet::new();
    for record in &bank.records {
        assert_eq!(
            digest(&rom[record.root as usize..record.end as usize]),
            record.source_sha256
        );
        spans.insert((record.root, record.end, "compact-record"));
    }
    for caller in direct_literals(bank) {
        spans.insert((caller.literal, caller.literal + 4, "constructor-literal"));
    }
    for table in &bank.selector_tables {
        spans.insert((table.start, table.end, "indexed-root-pointers"));
        spans.insert((
            table.parameter_start,
            table.parameter_start + 2 * table.slots.len() as u32,
            "indexed-parameters",
        ));
        for literal in [table.pointer_literal, table.parameter_literal] {
            spans.insert((literal, literal + 4, "constructor-literal"));
        }
    }
    spans
        .into_iter()
        .map(|(start, end, role)| SourceRange {
            offset: start,
            length: end - start,
            sha256: digest(&rom[start as usize..end as usize]),
            role: role.into(),
        })
        .collect()
}

fn original_steps(
    raw: &mut Machine,
    managed: &mut Machine,
    start: u32,
    end: u32,
    registers: &[(usize, u32)],
) {
    for machine in [&mut *raw, &mut *managed] {
        machine.cpu.regs = [0; 16];
        for &(register, value) in registers {
            machine.cpu.regs[register] = value;
        }
        machine.cpu.regs[15] = ROM_BASE + start;
        machine.cpu.cpsr = 0xbf;
        for _ in 0..32 {
            if machine.cpu.regs[15] == ROM_BASE + end {
                break;
            }
            machine.bus.frame_ready = false;
            machine.step();
        }
        assert_eq!(machine.cpu.regs[15], ROM_BASE + end, "reader at {start:#x}");
    }
    assert_eq!(managed.cpu.regs, raw.cpu.regs, "reader at {start:#x}");
    assert_eq!(managed.cpu.cpsr, raw.cpu.cpsr);
    assert_eq!(managed.bus.clock, raw.bus.clock);
}

fn original_controls(bank: &Bank, rom: &[u8], raw: &mut Machine, managed: &mut Machine) {
    for caller in direct_literals(bank) {
        let at = caller.load as usize;
        let opcode = u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        assert_eq!(opcode & 0xf800, 0x4800);
        let register = usize::from((opcode >> 8) & 7);
        let literal = caller.literal as usize;
        original_steps(raw, managed, caller.load, caller.load + 2, &[]);
        assert_eq!(
            managed.cpu.regs[register],
            u32::from_le_bytes(rom[literal..literal + 4].try_into().unwrap())
        );
    }
    for table in &bank.selector_tables {
        let register = usize::from((table.selector_opcode >> 3) & 7);
        let displacement = u32::from((table.selector_opcode >> 6) & 31);
        for slot in &table.slots {
            raw.bus.write8(RAM + displacement, slot.index);
            managed.bus.write8(RAM + displacement, slot.index);
            original_steps(raw, managed, table.load, table.call, &[(register, RAM)]);
            assert_eq!(
                &managed.cpu.regs[..4],
                &[0, ROM_BASE + slot.root, slot.parameter, 6]
            );
        }
    }
}

fn original_records(bank: &Bank, rom: &[u8], raw: &mut Machine, managed: &mut Machine) {
    for record in &bank.records {
        let directory = ROM_BASE + record.directory;
        let sequence = ROM_BASE + record.sequence;
        original_steps(
            raw,
            managed,
            0xd868c,
            0xd86a0,
            &[(4, ROM_BASE + record.root)],
        );
        assert_eq!(managed.cpu.regs[5], directory);
        assert_eq!(managed.cpu.regs[2], sequence);
        assert_eq!(managed.cpu.regs[3], record.header_flags);
        original_steps(raw, managed, 0xb08a, 0xb092, &[(0, sequence)]);
        assert_eq!(managed.cpu.regs[0], record.sequence_entries);

        for (index, frame) in record.frames.iter().enumerate() {
            original_steps(
                raw,
                managed,
                0xaf04,
                0xaf30,
                &[
                    (0, directory + 2 * index as u32),
                    (1, directory),
                    (3, RAM + 4),
                    (4, RAM),
                    (6, RAM + 6),
                    (7, RAM + 8),
                    (8, RAM + 10),
                ],
            );
            assert_eq!(managed.cpu.regs[1], ROM_BASE + frame.start);
            assert_eq!(managed.bus.read32(RAM), frame.start - record.directory);
            for byte in 0..4 {
                let output = RAM + 4 + 2 * byte;
                let expected = u16::from(rom[(frame.start + byte) as usize]);
                assert_eq!(raw.bus.read16(output), expected);
                assert_eq!(managed.bus.read16(output), expected);
            }
        }
        for index in 0..record.sequence_entries {
            let at = record.sequence + 2 + 2 * index;
            let packed = u16::from(rom[at as usize]);
            original_steps(
                raw,
                managed,
                0xae6c,
                0xae7c,
                &[(3, ROM_BASE + at), (4, RAM)],
            );
            let expected = (packed & 63) | ((packed & 0xc0) << 8);
            assert_eq!(raw.bus.read16(RAM), expected);
            assert_eq!(managed.bus.read16(RAM), expected);
            original_steps(raw, managed, 0xae80, 0xae82, &[(3, ROM_BASE + at)]);
            assert_eq!(managed.cpu.regs[0], u32::from(rom[at as usize + 1]));
        }
    }
}

#[test]
#[ignore = "exact EUR ROM, E310 manifest and E490 full compact-cell source index"]
fn european_compact_cell_bank_preparation_and_original_reads() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    let work = input("FFTA_COMPACT_CELL_BANK_WORK");
    let bank: Bank =
        serde_json::from_slice(&fs::read(input("FFTA_COMPACT_CELL_BANK_INDEX")).unwrap()).unwrap();
    let previous_bytes = fs::read(input("FFTA_COMPACT_CELL_BANK_PREDECESSOR")).unwrap();
    let previous: Manifest = parse(&previous_bytes, "E310 manifest").unwrap();
    assert_eq!(
        (
            previous.extractor_revision,
            previous.decoder_revision,
            previous.resources.len()
        ),
        (310, 309, 155)
    );
    assert_eq!(
        (
            bank.records.len(),
            bank.selector_tables.len(),
            direct_literals(&bank).count()
        ),
        (91, 9, 78)
    );

    let root = tempfile::tempdir_in(&work).unwrap();
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(root.path(), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    let dataset = &prepared.directory;
    let manifest_path = dataset.join(MANIFEST_FILE);
    let manifest_bytes = fs::read(&manifest_path).unwrap();
    let manifest: Manifest = parse(&manifest_bytes, "E311 manifest").unwrap();
    assert_eq!(
        (
            manifest.extractor_revision,
            manifest.decoder_revision,
            manifest.resources.len()
        ),
        (311, 310, 156)
    );
    assert_eq!(&manifest.resources[..155], previous.resources.as_slice());
    assert_eq!(&manifest.coverage[..155], previous.coverage.as_slice());
    let resource = &manifest.resources[155];
    assert_eq!(resource.id, COMPACT_CELL_BANK_ID);
    let sources = source_ranges(&bank, &rom);
    assert_eq!(resource.sources, sources);
    let path = dataset.join(COMPACT_CELL_BANK_FILE);
    let bytes = fs::read(&path).unwrap();
    assert_eq!(bytes.len(), 31_634);
    assert_eq!(digest(&bytes), resource.sha256);
    let expected: Vec<_> = sources
        .iter()
        .flat_map(|span| {
            rom[span.offset as usize..(span.offset + span.length) as usize]
                .iter()
                .copied()
        })
        .collect();
    assert_eq!(bytes, expected);
    let reused = preparation::prepare(root.path(), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(reused.action, "reused");
    assert_eq!(reused.directory, *dataset);

    let mut managed = Machine::new(rom.clone());
    fs::write(&manifest_path, previous_bytes).unwrap();
    fs::remove_file(&path).unwrap();
    attach_from_directory(&mut managed, dataset).unwrap();
    assert_eq!(
        managed
            .bus
            .rom_resources()
            .unwrap()
            .compact_cell_bank_bus_bytes(),
        None
    );
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    fs::write(&path, &bytes).unwrap();
    attach_from_directory(&mut managed, dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();

    let mut edited = bytes.clone();
    edited[0] ^= 1;
    let mut edited_manifest = manifest.clone();
    edited_manifest.resources[155].sha256 = digest(&edited);
    fs::write(&path, edited).unwrap();
    fs::write(
        &manifest_path,
        toml::to_string_pretty(&edited_manifest).unwrap(),
    )
    .unwrap();
    assert_eq!(
        attach_from_directory(&mut managed, dataset).unwrap_err(),
        "Compact-cell bank file differs from authenticated source"
    );
    assert!(Arc::ptr_eq(&owner, managed.bus.rom_resources().unwrap()));

    let mut raw = Machine::new(rom.clone());
    let lanes = |range: &SourceRange| range.offset as usize..(range.offset + range.length) as usize;
    for span in &sources {
        for at in lanes(span) {
            managed.bus.rom[at] ^= 0xff;
        }
    }
    for &(lo, hi) in &bank.alignment {
        for machine in [&mut raw, &mut managed] {
            machine.bus.rom[lo as usize..hi as usize].fill(0xa5);
        }
    }
    let before = owner.compact_cell_bank_bus_bytes().unwrap();
    for span in &sources {
        for at in lanes(span) {
            let address = ROM_BASE + at as u32;
            assert_eq!(
                raw.bus.read8(address),
                managed.bus.read8(address),
                "bank byte {at:#x}"
            );
        }
    }
    assert_eq!(
        owner.compact_cell_bank_bus_bytes().unwrap() - before,
        31_634
    );
    for span in &sources {
        for edge in [span.offset, span.offset + span.length] {
            check_stream_bus_reads(&mut raw, &mut managed, edge, edge);
        }
    }

    let before = owner.compact_cell_bank_bus_bytes().unwrap();
    original_controls(&bank, &rom, &mut raw, &mut managed);
    let control_reads = owner.compact_cell_bank_bus_bytes().unwrap() - before;
    assert_eq!(control_reads, 886);
    let before = owner.compact_cell_bank_bus_bytes().unwrap();
    original_records(&bank, &rom, &mut raw, &mut managed);
    let record_reads = owner.compact_cell_bank_bus_bytes().unwrap() - before;
    assert_eq!(record_reads, 4630);
    raw.bus.rom.copy_from_slice(&rom);
    managed.bus.rom.copy_from_slice(&rom);
    assert!(raw.save_state().unwrap() == managed.save_state().unwrap());

    root.close().unwrap();
    fs::write(work.join(MANIFEST_FILE), manifest_bytes).unwrap();
    let result = serde_json::json!({
        "status": "PASS", "revision": [311, 310], "published_bytes": bytes.len(),
        "source_file_sha256": resource.sha256, "prior_resources_preserved": 155,
        "original_control_read_bytes": control_reads, "original_record_read_bytes": record_reads,
        "records": 91, "frame_slots": 493, "sequence_entries": 563,
        "publication_and_reuse": true, "e310_attach": true, "coherent_edit_rejected": true,
    });
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}

const BACKGROUND: u32 = RAM + 0x9000;
const TASK: u32 = RAM + 0xa000;
const OUTPUT_MAP: u32 = RAM + 0xb000;
const STACK: u32 = RAM + 0x3f000;
const MAP_SIDE: usize = 32;

fn run_render_steps(
    machine: &mut Machine,
    start: u32,
    stop: u32,
    registers: &[(usize, u32)],
    limit: usize,
) -> Option<u32> {
    machine.cpu.regs = [0; 16];
    for &(register, value) in registers {
        machine.cpu.regs[register] = value;
    }
    machine.cpu.regs[13] = STACK;
    machine.cpu.regs[15] = ROM_BASE + start;
    machine.cpu.cpsr = 0xbf;
    let mut decoder = None;
    for _ in 0..limit {
        let pc = machine.cpu.regs[15].wrapping_sub(ROM_BASE);
        if pc == stop {
            assert_eq!(machine.cpu.regs[13], STACK);
            return decoder;
        }
        if matches!(pc, 0xa808 | 0xab40) {
            assert!(decoder.replace(pc).is_none());
        }
        machine.step();
    }
    panic!(
        "Original render path {start:#x} did not reach {stop:#x}, PC {:#x}",
        machine.cpu.regs[15]
    );
}

fn verify_frame_draw(
    machine: &mut Machine,
    root: u32,
    frame: &compact_cell_bank::CompactCellFrame,
    byte_cells: bool,
) -> usize {
    let cell_width = if byte_cells { 1 } else { 2 };
    machine.bus.write32(BACKGROUND, OUTPUT_MAP);
    machine.bus.write16(BACKGROUND + 8, u16::from(byte_cells));
    machine.bus.write8(BACKGROUND + 0xc, MAP_SIDE as u8);
    machine.bus.write8(BACKGROUND + 0xd, MAP_SIDE as u8);
    machine.bus.write16(BACKGROUND + 0x10, 0x10);
    // Use zero as the original renderer's transparent cell in both modes.
    machine.bus.write32(RAM + 0x9138, 0);
    machine.bus.write32(TASK + 0x14, ROM_BASE + root + 4);
    machine.bus.write8(TASK + 0x1e, MAP_SIDE as u8);
    machine.bus.write8(TASK + 0x1f, MAP_SIDE as u8);

    run_render_steps(machine, 0xb4ae, 0xb4e0, &[(5, TASK)], 64);
    run_render_steps(
        machine,
        0xd86da,
        0xd86f8,
        &[(4, ROM_BASE + root), (5, TASK)],
        64,
    );
    let flags = machine.bus.read16(TASK + 2);
    assert_eq!(flags, if byte_cells { 0xc006 } else { 0xc205 });

    let dimensions = (usize::from(frame.width), usize::from(frame.height));
    assert!(dimensions.0 <= MAP_SIDE && dimensions.1 <= MAP_SIDE);
    let map_start = (OUTPUT_MAP - RAM) as usize;
    let map_end = map_start + MAP_SIDE * MAP_SIDE * cell_width;
    machine.bus.ewram[map_start..map_end].fill(0xd7);
    machine
        .bus
        .write32(STACK + 0x20, frame.record_offset as u32 - 4);
    machine.bus.write32(STACK + 0x30, 0);
    let decoder = run_render_steps(
        machine,
        0xb7bc,
        0xb8a2,
        &[
            (1, BACKGROUND),
            (2, u32::from(flags)),
            (4, BACKGROUND),
            (6, TASK),
            (8, STACK + 0x1c),
            (9, STACK + 0x14),
            (10, STACK + 0x16),
        ],
        1000 + 150 * frame.cells.len(),
    );
    assert_eq!(decoder, Some(if byte_cells { 0xab40 } else { 0xa808 }));

    let mut expected = vec![0xd7; map_end - map_start];
    let mut writes = 0;
    for (y, row) in frame.cells.chunks_exact(dimensions.0).enumerate() {
        for (x, &cell) in row.iter().enumerate().filter(|(_, cell)| **cell != 0) {
            let start = (y * MAP_SIDE + x) * cell_width;
            expected[start..start + cell_width].copy_from_slice(&cell.to_le_bytes()[..cell_width]);
            writes += 1;
        }
    }
    assert_eq!(
        &machine.bus.ewram[map_start..map_end],
        expected.as_slice(),
        "Cell map differs for record {root:#x} at offset {:#x}",
        frame.record_offset
    );
    writes
}

#[derive(Default, Serialize)]
struct DecoderCoverage {
    records: usize,
    frames: usize,
    cells: usize,
    writes: usize,
}

#[test]
#[ignore = "exact EUR ROM, E490 source index and E491 file identity"]
fn european_compact_cell_bank_decodes_original_frame_output() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    let work = input("FFTA_COMPACT_CELL_BANK_WORK");
    let bank: Bank =
        serde_json::from_slice(&fs::read(input("FFTA_COMPACT_CELL_BANK_INDEX")).unwrap()).unwrap();
    let extraction = Extraction::from_rom(&rom).unwrap();
    let resource = extraction
        .manifest
        .resources
        .iter()
        .find(|r| r.id == COMPACT_CELL_BANK_ID)
        .unwrap();
    let packed = extraction.compact_cell_bank.as_ref().unwrap();
    assert_eq!(packed.len(), 31_634);
    assert_eq!(
        digest(packed),
        "776a100153f58e525a6b7350f589150d81d7e4233ce5b14da8be371eb724ef27"
    );
    let mut original = Machine::new(rom);
    let mut cursor = 0;
    let mut coverage: [DecoderCoverage; 2] = Default::default();
    for span in &resource.sources {
        let end = cursor + span.length as usize;
        let bytes = &packed[cursor..end];
        cursor = end;
        if span.role != "compact-record" {
            continue;
        }
        let record = &bank.records[bank
            .records
            .binary_search_by_key(&span.offset, |record| record.root)
            .unwrap()];
        assert_eq!(digest(bytes), record.source_sha256);
        let byte_cells = record.header_flags == 14;
        let mode = &mut coverage[usize::from(byte_cells)];
        mode.records += 1;
        let frames = compact_cell_bank::decode_frames(bytes).unwrap();
        assert_eq!(frames.len(), record.frames.len());
        mode.frames += frames.len();
        for (frame, prior) in frames.iter().zip(&record.frames) {
            let start = record.root + frame.record_offset as u32;
            assert_eq!(
                (start, frame.width, frame.height),
                (prior.start, prior.width, prior.height)
            );
            let encoded_end = start + frame.encoded_len as u32;
            let frame_padding = if matches!(start, 0xaed7ce | 0xaee2f6) {
                2
            } else {
                0
            };
            assert_eq!(
                encoded_end,
                prior.token_end.unwrap_or(prior.extent_end - frame_padding)
            );
            mode.cells += frame.cells.len();
            mode.writes += verify_frame_draw(&mut original, record.root, frame, byte_cells);
        }
    }
    assert_eq!(cursor, packed.len());
    let [halfword, byte] = coverage;
    assert_eq!(
        (halfword.records, halfword.frames, halfword.cells),
        (87, 483, 34_532)
    );
    assert_eq!((byte.records, byte.frames, byte.cells), (4, 10, 592));
    assert!(halfword.writes > 0 && halfword.writes < halfword.cells);
    assert!(byte.writes > 0 && byte.writes < byte.cells);
    let result = serde_json::json!({
        "status": "PASS",
        "packed_sha256": digest(packed), "packed_bytes": packed.len(),
        "halfword": halfword, "byte": byte,
    });
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}

#[derive(Deserialize)]
struct OrdinaryReplay {
    first_guest_frame: u64,
    snapshot_sha256: String,
    steps: Vec<(u32, u16)>,
}

#[test]
#[ignore = "exact EUR ROM, E491 manifest and the retained IN-08 action-12 replay"]
fn european_compact_cell_bank_in_ordinary_action() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};

    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    let work = input("FFTA_COMPACT_CELL_BANK_WORK");
    let expected_manifest = fs::read(input("FFTA_COMPACT_CELL_BANK_MANIFEST")).unwrap();
    let manifest: Manifest = parse(&expected_manifest, "E491 manifest").unwrap();
    let replay: OrdinaryReplay =
        serde_json::from_slice(&fs::read(input("FFTA_COMPACT_CELL_BANK_REPLAY")).unwrap()).unwrap();
    let snapshot = fs::read(input("FFTA_COMPACT_CELL_BANK_SNAPSHOT")).unwrap();
    assert_eq!(digest(&snapshot), replay.snapshot_sha256);

    let root = tempfile::tempdir_in(&work).unwrap();
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(root.path(), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(
        fs::read(prepared.directory.join(MANIFEST_FILE)).unwrap(),
        expected_manifest
    );
    let bank = manifest
        .resources
        .iter()
        .find(|resource| resource.id == COMPACT_CELL_BANK_ID)
        .unwrap();
    let altered_bytes = bank.sources.iter().map(|span| span.length).sum::<u32>();
    assert_eq!(altered_bytes, 31_634);

    let mut raw = Machine::new(rom.clone());
    let mut managed = Machine::new(rom.clone());
    for machine in [&mut raw, &mut managed] {
        machine.load_state(&snapshot).unwrap();
        assert_eq!(machine.bus.frames, replay.first_guest_frame);
    }
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let before = owner.compact_cell_bank_bus_bytes().unwrap();
    for span in &bank.sources {
        let start = span.offset as usize;
        let end = start + span.length as usize;
        for byte in &mut managed.bus.rom[start..end] {
            *byte = !*byte;
        }
    }

    let capture = TraceCapture::start(
        TraceFilter {
            readers: Vec::new(),
            // Observe both constructor candidates for the second record, plus the
            // task descriptor and the whole physical extent of the compact records.
            ranges: vec![
                (0x0e92e8, 0x0e92ec),
                (0x0e93fc, 0x0e9400),
                (0x0ed244, 0x0ed248),
                (0x399810, 0x399818),
                (0xae9ebc, 0xaf1818),
            ],
        },
        4096,
    )
    .unwrap();
    let audio_samples = replay_pair(&mut raw, &mut managed, &replay.steps, |frame| {
        capture.frame(frame)
    });
    let report = capture.finish();
    assert!(audio_samples > 0 && !report.truncated && report.dropped_requests == 0);
    let mut traced_bank_bytes = 0;
    for event in &report.events {
        let read = event.key.read;
        let start = read.physical_offset;
        let end = start + read.length as u32;
        if bank
            .sources
            .iter()
            .any(|span| span.offset <= start && end <= span.offset + span.length)
        {
            assert_eq!(event.key.service, Service::Typed, "{event:?}");
            traced_bank_bytes += event.requests * read.length as u64;
        }
    }
    let managed_bus_bytes = owner.compact_cell_bank_bus_bytes().unwrap() - before;
    assert!(traced_bank_bytes > 0 && managed_bus_bytes >= traced_bank_bytes);

    managed.bus.rom.copy_from_slice(&rom);
    assert!(raw.bus.framebuffer == managed.bus.framebuffer && raw.bus.oam == managed.bus.oam);
    let endpoint = raw.save_state().unwrap();
    assert!(
        endpoint == managed.save_state().unwrap(),
        "Ordinary replay state differs"
    );
    fs::write(
        work.join("managed-trace.json"),
        serde_json::to_vec(&report).unwrap(),
    )
    .unwrap();
    let result = serde_json::json!({
        "status": "PASS", "manifest_sha256": digest(&expected_manifest),
        "altered_bytes": altered_bytes, "managed_bus_bytes": managed_bus_bytes,
        "traced_bank_bytes": traced_bank_bytes, "audio_samples": audio_samples,
        "first_guest_frame": replay.first_guest_frame, "last_guest_frame": raw.bus.frames,
        "end_state_sha256": digest(&endpoint), "equivalent": true,
    });
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    root.close().unwrap();
}
