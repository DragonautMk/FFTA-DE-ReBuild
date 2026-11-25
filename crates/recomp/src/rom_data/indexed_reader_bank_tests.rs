//! Use the E506 census and original instructions to verify the complete banks.
use super::{
    Extraction, INDEXED_GRAPHICS_FILE, INDEXED_GRAPHICS_ID, MANIFEST_FILE, Manifest,
    attach_from_directory, cmd, encode_indexed_graphics, parse, parse_indexed_graphics,
};
use gba_core::{Bus, Machine, ffta_rom::digest};
use serde::Deserialize;
use std::{collections::BTreeMap, fs, path::PathBuf, sync::Arc};

const BASE: u32 = 0x0800_0000;
const LANGUAGE: u32 = 0x0200_1f6e;
const DESTINATION: u32 = 0x0201_0000;
const RETURN: u32 = BASE + 0x300;
const DECODER: u32 = BASE + 0x537c;
const UNROOTED: u32 = 0x3b8fbc;

#[derive(Deserialize)]
struct Plan {
    archives: Vec<ArchiveWitness>,
}

#[derive(Deserialize)]
struct ArchiveWitness {
    offset: u32,
    end: u32,
    unowned_bytes: u32,
    programs: usize,
    decoded_bytes: usize,
    source_sha256: String,
    decoded_sha256: String,
}

#[derive(Deserialize)]
struct Readers {
    literal_loads: Vec<LiteralLoad>,
    pointer_tables: BTreeMap<String, Vec<u32>>,
}

#[derive(Deserialize)]
struct LiteralLoad {
    instruction: u32,
    literal: u32,
    source: u32,
}

fn input(name: &str) -> PathBuf {
    std::env::var_os(name).unwrap().into()
}

pub(super) fn initialize(machine: &mut Machine, start: u32) {
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs.fill(0);
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = start;
    machine.cpu.cpsr = 0xbf;
    machine.cpu.branch = None;
}

pub(super) fn execute_until(machine: &mut Machine, stop: u32, step_limit: usize) {
    for _ in 0..step_limit {
        if machine.cpu.regs[15] == stop {
            return;
        }
        machine.bus.frame_ready = false;
        machine.step();
    }
    panic!("Indexed reader escaped at {:08x}", machine.cpu.regs[15]);
}

fn selection(machine: &mut Machine, load: &LiteralLoad, slot: u8) -> u32 {
    // Enter immediately before the original literal load or angle projection.
    // Stop on the wrapper call or after the lookup in the pointer-return path.
    let entry = load.instruction;
    let (start, stop) = match entry {
        0x03287e => (entry, 0x032886),
        0x068fe2 => (entry, 0x068ffa),
        0x070e50 => (entry, 0x070e60),
        0x0737ea => (entry, 0x0737f8),
        0x07380c => (entry, 0x073814),
        0x07e64e => (entry, 0x07e662),
        0x07e836 => (entry, 0x07e84a),
        0x08a97e => (entry, 0x08a984),
        0x08aa16 => (entry, 0x08aa1c),
        0x08ab46 => (entry, 0x08ab4c),
        0x08f372 => (entry, 0x08f386),
        0x09276a => (entry, 0x092778),
        0x0a47a8 => (0x0a479c, 0x0a47b6),
        0x0abbca => (0x0abbbe, 0x0abbd2),
        0x0cf3b8 => (entry, 0x0cf3be),
        0x12f040 => (entry, 0x12f04c),
        _ => panic!("Unexpected E506 literal load at {entry:06x}"),
    };
    initialize(machine, BASE + start);
    machine.bus.write8(LANGUAGE, slot);
    machine.cpu.regs[0] = u32::from(slot) << 12;
    machine.cpu.regs[4] = if entry == 0x070e50 {
        LANGUAGE
    } else {
        DESTINATION
    };
    machine.cpu.regs[5] = if entry == 0x09276a {
        LANGUAGE
    } else {
        DESTINATION
    };
    machine.cpu.regs[7] = DESTINATION;
    machine.cpu.regs[9] = DESTINATION;
    execute_until(machine, BASE + stop, 40);
    machine.cpu.regs[0]
}

#[test]
#[ignore = "exact EUR ROM, E506 evidence and E505 predecessor manifest"]
fn european_indexed_reader_bank_publication_and_original_decoders() {
    let work = input("FFTA_INDEXED_BANKS_WORK");
    let rom_path = input("FFTA_STATE_TEST_ROM");
    let rom = fs::read(&rom_path).unwrap();
    let plan: Plan =
        serde_json::from_slice(&fs::read(input("FFTA_INDEXED_BANKS_PLAN")).unwrap()).unwrap();
    let readers: Readers =
        serde_json::from_slice(&fs::read(input("FFTA_INDEXED_BANKS_READERS")).unwrap()).unwrap();
    let previous_bytes = fs::read(input("FFTA_INDEXED_BANKS_PREDECESSOR")).unwrap();
    let previous: Manifest = parse(&previous_bytes, "E505 manifest").unwrap();
    assert_eq!(
        (previous.extractor_revision, previous.decoder_revision),
        (315, 314)
    );

    let resources = tempfile::tempdir_in(&work).unwrap();
    let rom_arg = rom_path.to_str().unwrap().to_owned();
    let prepare = [
        "prepare".into(),
        rom_arg.clone(),
        resources.path().to_str().unwrap().into(),
    ];
    cmd(&prepare).unwrap();
    let revision = resources
        .path()
        .join(&previous.source_rom_sha256)
        .join("v1/e316-d315");
    let current = fs::read_to_string(revision.join("current")).unwrap();
    let dataset = revision.join(current.trim());
    cmd(&prepare).unwrap();
    assert_eq!(
        fs::read_to_string(revision.join("current")).unwrap(),
        current
    );
    cmd(&["verify".into(), rom_arg, dataset.to_str().unwrap().into()]).unwrap();
    let manifest_path = dataset.join(MANIFEST_FILE);
    let manifest_bytes = fs::read(&manifest_path).unwrap();
    let manifest: Manifest = parse(&manifest_bytes, "E507 manifest").unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (316, 315)
    );
    assert_eq!(manifest.coverage, previous.coverage);
    assert_eq!(manifest.resources.len(), previous.resources.len());
    let index = manifest
        .resources
        .iter()
        .position(|entry| entry.id == INDEXED_GRAPHICS_ID)
        .unwrap();
    for (i, (new, old)) in manifest
        .resources
        .iter()
        .zip(&previous.resources)
        .enumerate()
    {
        if i != index {
            assert_eq!(new, old);
        }
    }
    let old = &previous.resources[index];
    let new = &manifest.resources[index];
    assert_eq!(
        (&new.path, &new.format, &new.dependencies),
        (&old.path, &old.format, &old.dependencies)
    );
    assert!(
        old.sources
            .iter()
            .all(|source| new.sources.contains(source))
    );
    let added: Vec<_> = new
        .sources
        .iter()
        .filter(|source| !old.sources.contains(source))
        .collect();
    assert_eq!(added.len(), 50);
    assert_eq!(
        added.iter().map(|source| source.length).sum::<u32>(),
        109_421
    );
    for range in &added {
        let start = range.offset as usize;
        let end = start + range.length as usize;
        assert_eq!(digest(&rom[start..end]), range.sha256);
        assert!(
            previous
                .resources
                .iter()
                .flat_map(|entry| &entry.sources)
                .all(|prior| {
                    prior.offset + prior.length <= range.offset
                        || range.offset + range.length <= prior.offset
                })
        );
    }

    let packet_path = dataset.join(INDEXED_GRAPHICS_FILE);
    let packet_bytes = fs::read(&packet_path).unwrap();
    let mut graphics = parse_indexed_graphics(&packet_bytes).unwrap();
    let banks = graphics.reader_banks.take().unwrap();
    let previous_extraction = Extraction::from_rom_for_manifest(&rom, &previous).unwrap();
    let previous_graphics = previous_extraction.indexed_graphics.unwrap().decoded;
    assert_eq!(graphics, previous_graphics);
    let previous_packet = encode_indexed_graphics(&previous_graphics).unwrap();
    assert_eq!(digest(&previous_packet), old.sha256);

    let mut managed = Machine::new(rom.clone());
    fs::write(&manifest_path, previous_bytes).unwrap();
    fs::write(&packet_path, previous_packet).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let first = added[0].offset;
    managed.bus.rom[first as usize] ^= 0xff;
    assert_eq!(managed.bus.read8(BASE + first), !rom[first as usize]);
    managed.bus.rom[first as usize] ^= 0xff;
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    fs::write(&packet_path, &packet_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();

    let mut edited = parse_indexed_graphics(&packet_bytes).unwrap();
    edited.reader_banks.as_mut().unwrap().archives[0].opaque_header[0] ^= 1;
    edited.validate().unwrap();
    let mut forged = manifest.clone();
    let forged_archive = &edited.reader_banks.as_ref().unwrap().archives[0];
    forged.resources[index]
        .sources
        .iter_mut()
        .find(|row| row.offset == forged_archive.source_offset)
        .unwrap()
        .sha256 = digest(&forged_archive.source_bytes().unwrap());
    let edited = encode_indexed_graphics(&edited).unwrap();
    forged.resources[index].sha256 = digest(&edited);
    fs::write(&manifest_path, toml::to_string_pretty(&forged).unwrap()).unwrap();
    fs::write(&packet_path, edited).unwrap();
    assert!(attach_from_directory(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(&owner, managed.bus.rom_resources().unwrap()));
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    fs::write(&packet_path, &packet_bytes).unwrap();

    let poison = |machine: &mut Machine| {
        for span in &added {
            for byte in
                &mut machine.bus.rom[span.offset as usize..(span.offset + span.length) as usize]
            {
                *byte ^= 0xff;
            }
        }
    };
    poison(&mut managed);
    let before = owner.indexed_graphics_stats().unwrap();
    for span in &added {
        for at in span.offset..span.offset + span.length {
            assert_eq!(managed.bus.read8(BASE + at), rom[at as usize], "{at:06x}");
        }
    }
    managed.bus.rom[UNROOTED as usize] ^= 0xff;
    assert_eq!(managed.bus.read8(BASE + UNROOTED), !rom[UNROOTED as usize]);
    managed.bus.rom[UNROOTED as usize] ^= 0xff;
    let after = owner.indexed_graphics_stats().unwrap();
    assert_eq!(after.source_bytes - before.source_bytes, 109_421);
    assert_eq!(after.reader_literal_bytes - before.reader_literal_bytes, 64);
    assert_eq!(after.selector_bytes - before.selector_bytes, 104);
    assert_eq!(
        (after.archives_used, after.groups_used, after.programs_used),
        (31, 3, 852)
    );

    let mut raw = Machine::new(rom);
    let mut selector_cases = 0;
    for load in &readers.literal_loads {
        assert!(
            added
                .iter()
                .any(|span| span.offset == load.literal && span.length == 4)
        );
        let table = readers.pointer_tables.get(&format!("{:06x}", load.source));
        let choices = table
            .map(Vec::as_slice)
            .unwrap_or(std::slice::from_ref(&load.source));
        for (slot, &offset) in choices.iter().enumerate() {
            raw.bus.clock = 0;
            managed.bus.clock = 0;
            assert_eq!(selection(&mut raw, load, slot as u8), BASE + offset);
            assert_eq!(selection(&mut managed, load, slot as u8), BASE + offset);
            assert_eq!(raw.cpu.regs, managed.cpu.regs);
            assert_eq!(raw.cpu.cpsr, managed.cpu.cpsr);
            assert_eq!(raw.bus.clock, managed.bus.clock);
            selector_cases += 1;
        }
    }
    assert_eq!(selector_cases, 70);
    poison(&mut managed);

    let initial = Machine::new(raw.bus.rom.clone()).save_state().unwrap();
    let expected = plan
        .archives
        .iter()
        .filter(|row| row.unowned_bytes > 0 && row.offset != UNROOTED);
    let mut decoded_bytes = 0;
    let mut decoder_cases = 0;
    for (archive, witness) in banks.archives.iter().zip(expected) {
        assert_eq!(archive.source_offset, witness.offset);
        assert_eq!(archive.programs.len(), witness.programs);
        let range = added
            .iter()
            .find(|span| span.offset == witness.offset)
            .unwrap();
        assert_eq!(range.length, witness.end - witness.offset);
        assert_eq!(range.sha256, witness.source_sha256);
        let decoded = archive.selected_bytes(0, 0).unwrap();
        assert_eq!(decoded.len(), witness.decoded_bytes);
        assert_eq!(digest(&decoded), witness.decoded_sha256);
        raw.load_state(&initial).unwrap();
        managed.load_state(&initial).unwrap();
        poison(&mut managed);
        for machine in [&mut raw, &mut managed] {
            initialize(machine, DECODER);
            machine.cpu.regs[0] = BASE + witness.offset;
            machine.cpu.regs[1] = DESTINATION;
            execute_until(machine, RETURN, 10_000_000);
            assert_eq!(machine.cpu.regs[0], 0);
            let destination = (DESTINATION - 0x0200_0000) as usize;
            assert_eq!(
                &machine.bus.ewram[destination..destination + decoded.len()],
                decoded
            );
        }
        poison(&mut managed);
        assert_eq!(
            raw.save_state().unwrap(),
            managed.save_state().unwrap(),
            "archive {:06x}",
            witness.offset
        );
        decoded_bytes += decoded.len();
        decoder_cases += 1;
    }
    assert_eq!((decoder_cases, decoded_bytes), (31, 189_184));
    let result = serde_json::json!({
        "status": "PASS", "revision": [316, 315], "source_bytes": 109421, "spans": added.len(),
        "archives": decoder_cases, "programs": 852, "decoded_bytes": decoded_bytes,
        "original_selector_cases": selector_cases, "predecessor_attach": true,
        "rehashed_edit_rejected": true, "unrooted_fallback": true, "full_state_equal": true,
        "manifest_sha256": digest(&manifest_bytes), "packet_sha256": new.sha256,
    });
    fs::write(work.join("manifest.toml"), manifest_bytes).unwrap();
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
