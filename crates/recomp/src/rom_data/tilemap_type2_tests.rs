//! Exercise the published EUR type-2 lists through the original guest readers.
use super::{
    attach_from_directory, parse, preparation,
    tilemap_transfer_directory_tests::check_stream_bus_reads, Extraction, Manifest, MANIFEST_FILE,
    TILEMAP_TYPE2_FILE, TILEMAP_TYPE2_ID,
};
use gba_core::{ffta_rom::digest, rom_data::SourceRange, Bus, Machine};
use serde::Deserialize;
use std::{fs, ops::Range, path::PathBuf, sync::Arc};

const CONTEXT: u32 = 0x0300_1000;
const READER: u32 = 0x080d_b374;
const RETURN: u32 = 0x0200_0000;
const MIRRORS: [u32; 3] = [0x0800_0000, 0x0a00_0000, 0x0c00_0000];
const DESTINATION: Range<usize> = 0x3860..0x3c60;
const PRIOR_RECTANGLE: Range<usize> = 0xae6db8..0xae6dc0;

#[derive(Deserialize)]
struct Copy {
    source: usize,
    destination: usize,
    length: usize,
}

#[derive(Deserialize)]
struct ListCase {
    row_offset: u32,
    copies: Vec<Copy>,
    file_bytes_read: u64,
}

#[derive(Deserialize)]
struct ExpectedBank {
    sources: Vec<SourceRange>,
    original_lists: Vec<ListCase>,
}

fn input(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os(name).unwrap_or_else(|| panic!("Missing {name}")))
}

pub(super) fn install_rectangle_copy(machine: &mut Machine, rom: &[u8]) {
    // The game installs its byte and word copy helper in IWRAM at boot.
    machine.bus.iwram[0x5ee8..0x5f26].copy_from_slice(&rom[0xd968a8..0xd968e6]);
}

fn start_list(machine: &mut Machine, row: u32) {
    machine.bus.iwram[DESTINATION].fill(0x5a);
    machine.bus.write32(CONTEXT, row);
    machine.bus.write8(CONTEXT + 7, 0);
    machine.cpu.regs = [0; 16];
    machine.cpu.regs[0] = CONTEXT;
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = READER;
    machine.cpu.cpsr = 0xbf;
    machine.bus.frame_ready = false;
}

fn finish_list(machine: &mut Machine, row: u32) {
    for _ in 0..20_000 {
        if machine.cpu.regs[15] == RETURN {
            assert_eq!(machine.cpu.regs[0], 1, "return value for {row:#x}");
            return;
        }
        machine.step();
    }
    panic!("Original list reader did not return for {row:#x}");
}

fn check_original_lists(
    raw: &mut Machine,
    managed: &mut Machine,
    rom: &[u8],
    lists: &[ListCase],
) -> (u64, usize) {
    let owner = managed.bus.rom_resources().unwrap().clone();
    let start = owner.tilemap_type2_bus_bytes().unwrap();
    for machine in [&mut *raw, &mut *managed] {
        install_rectangle_copy(machine, rom);
        machine.bus.ffta_sprites.enabled = false;
    }
    let mut copied = 0;
    for (index, list) in lists.iter().enumerate() {
        let address = MIRRORS[index % MIRRORS.len()] + list.row_offset;
        let mut expected = vec![0x5a; DESTINATION.len()];
        for copy in &list.copies {
            let dest = copy.destination - DESTINATION.start;
            expected[dest..dest + copy.length]
                .copy_from_slice(&rom[copy.source..copy.source + copy.length]);
            copied += copy.length;
        }
        let before = owner.tilemap_type2_bus_bytes().unwrap();
        for machine in [&mut *raw, &mut *managed] {
            start_list(machine, address);
            finish_list(machine, address);
            assert!(
                machine.bus.iwram[DESTINATION] == expected,
                "Rectangle destination mismatch for {address:#x}"
            );
        }
        assert_eq!(
            owner.tilemap_type2_bus_bytes().unwrap() - before,
            list.file_bytes_read,
            "Unexpected file-backed reads for {address:#x}"
        );
        assert_eq!(
            managed.cpu.regs, raw.cpu.regs,
            "CPU result for {address:#x}"
        );
    }
    (owner.tilemap_type2_bus_bytes().unwrap() - start, copied)
}

#[test]
#[ignore = "exact EUR ROM, E306 manifest and independently verified E449/E452 projection"]
fn european_type2_bank_files_and_original_lists() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    let work = input("FFTA_TYPE2_WORK");
    let expected: ExpectedBank =
        serde_json::from_slice(&fs::read(input("FFTA_TYPE2_PLAN")).unwrap()).unwrap();
    let old_bytes = fs::read(input("FFTA_TYPE2_PREDECESSOR")).unwrap();
    let old: Manifest = parse(&old_bytes, "E306 manifest").unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (306, 305));

    let root = tempfile::tempdir_in(&work).unwrap();
    let mut revision = old.clone();
    (revision.extractor_revision, revision.decoder_revision) = (307, 306);
    let mut extraction = Extraction::from_rom_for_manifest(&rom, &revision).unwrap();
    let prepared = preparation::prepare(root.path(), &mut extraction, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    let dataset = &prepared.directory;
    let manifest_path = dataset.join(MANIFEST_FILE);
    let manifest_bytes = fs::read(&manifest_path).unwrap();
    let manifest: Manifest = parse(&manifest_bytes, "E307 manifest").unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (307, 306)
    );
    assert_eq!(manifest.resources.len(), 154);
    assert_eq!(manifest.resources[..153], old.resources);
    assert_eq!(manifest.coverage[..153], old.coverage);
    let resource = &manifest.resources[153];
    assert_eq!(resource.id, TILEMAP_TYPE2_ID);
    assert_eq!(resource.sources, expected.sources);
    let path = dataset.join(TILEMAP_TYPE2_FILE);
    let mut file = fs::read(&path).unwrap();
    assert_eq!(file.len(), 14_758);
    assert_eq!(digest(&file), resource.sha256);

    let reused = preparation::prepare(root.path(), &mut extraction, &mut |_| Ok(())).unwrap();
    assert_eq!(reused.action, "reused");
    assert_eq!(&reused.directory, dataset);
    drop(extraction);

    let mut raw = Machine::new(rom.clone());
    let initial = raw.save_state().unwrap();
    let mut managed = Machine::new(rom.clone());
    fs::write(&manifest_path, old_bytes).unwrap();
    attach_from_directory(&mut managed, dataset).unwrap();
    let legacy = managed.bus.rom_resources().unwrap();
    assert!(legacy.tilemap_type2_bus_bytes().is_none());
    assert!(legacy.tilemap_type3_stats().is_some());
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    attach_from_directory(&mut managed, dataset).unwrap();
    let active = managed.bus.rom_resources().unwrap().clone();

    *file.last_mut().unwrap() ^= 1;
    fs::write(&path, &file).unwrap();
    let mut edited = manifest.clone();
    edited.resources[153].sha256 = digest(&file);
    fs::write(&manifest_path, toml::to_string_pretty(&edited).unwrap()).unwrap();
    assert_eq!(
        attach_from_directory(&mut managed, dataset).unwrap_err(),
        "Type-2 tilemap transfer file differs from authenticated source",
    );
    assert!(Arc::ptr_eq(&active, managed.bus.rom_resources().unwrap()));
    *file.last_mut().unwrap() ^= 1;

    let mut cursor = 0;
    for span in &resource.sources {
        let start = span.offset as usize;
        let length = span.length as usize;
        let end = start + length;
        assert_eq!(&file[cursor..cursor + length], &rom[start..end]);
        managed.bus.rom[start..end]
            .iter_mut()
            .for_each(|byte| *byte = !*byte);
        cursor += length;
    }
    assert_eq!(cursor, file.len());
    for byte in &mut managed.bus.rom[PRIOR_RECTANGLE] {
        *byte = !*byte;
    }
    // One word starts in the previously owned cells and ends in the new file.
    let before = active.tilemap_type2_bus_bytes().unwrap();
    let split = 0x0a00_0000 + 0xae6dbe;
    assert_eq!(managed.bus.read32(split), raw.bus.read32(split));
    assert_eq!(active.tilemap_type2_bus_bytes().unwrap() - before, 2);
    for span in &resource.sources {
        check_stream_bus_reads(
            &mut raw,
            &mut managed,
            span.offset,
            span.offset + span.length,
        );
    }

    // Both packed files are active. Verify that the type-3 BIOS input still
    // resolves by its exact start and reports the full encoded extent.
    let start = 0xafbfa8;
    let encoded = &rom[start..start + 120];
    managed.bus.rom[start..start + 120]
        .iter_mut()
        .for_each(|byte| *byte = !*byte);
    check_stream_bus_reads(&mut raw, &mut managed, start as u32, start as u32 + 120);
    let before = active.tilemap_type3_stats().unwrap();
    let bound = managed
        .bus
        .bound_tilemap_lz77(0x0c00_0000 + start as u32)
        .unwrap();
    assert_eq!(bound.bytes(), encoded);
    let after = active.tilemap_type3_stats().unwrap();
    assert_eq!(after.lz77_calls - before.lz77_calls, 1);
    assert_eq!(after.lz77_input_bytes - before.lz77_input_bytes, 120);
    managed.bus.rom[start..start + 120].copy_from_slice(encoded);

    raw.load_state(&initial).unwrap();
    let mut backing = rom.clone();
    std::mem::swap(&mut managed.bus.rom, &mut backing);
    managed.load_state(&initial).unwrap();
    std::mem::swap(&mut managed.bus.rom, &mut backing);
    assert_eq!(expected.original_lists.len(), 294);
    let (served, copied) =
        check_original_lists(&mut raw, &mut managed, &rom, &expected.original_lists);
    assert!(served > 0 && copied > 0);
    managed.bus.rom.copy_from_slice(&rom);
    let endpoint = managed.save_state().unwrap();
    assert!(
        endpoint == raw.save_state().unwrap(),
        "final original-reader states differ"
    );
    let result = serde_json::json!({
        "status": "PASS",
        "revision": [307, 306],
        "published_bytes": file.len(),
        "published_manifest_sha256": digest(&manifest_bytes),
        "source_file_sha256": resource.sha256,
        "original_list_readers": expected.original_lists.len(),
        "guest_file_bytes_read": served,
        "guest_rectangle_bytes_copied": copied,
        "guest_state_sha256": digest(&endpoint),
        "e306_attach": true,
        "preserved_type3_input": true,
        "split_rectangle_boundary": true,
        "coherent_edit_rejected": true,
        "publication_and_reuse": true,
    });
    root.close().unwrap();
    fs::write(work.join(MANIFEST_FILE), manifest_bytes).unwrap();
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
