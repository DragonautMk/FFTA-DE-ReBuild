//! EUR type-4 and type-5 dispatch through the persisted controls and streams.
use super::{
    attach_from_directory, parse, preparation,
    tilemap_transfer_directory_tests::check_stream_bus_reads, Extraction, Manifest, MANIFEST_FILE,
    TILEMAP_DIRECT_FILE, TILEMAP_DIRECT_ID,
};
use gba_core::{ffta_rom::digest, rom_data::SourceRange, Bus, Machine};
use serde::Deserialize;
use std::{fs, ops::Range, path::PathBuf, sync::Arc};

const CONTEXT: u32 = 0x0201_8000;
const DISPATCH: u32 = 0x080d_b2a8;
const RETURN: u32 = 0x0200_0000;
const DESTINATION: Range<usize> = 0x20000..0x30000;
const MIRRORS: [u32; 3] = [0x0800_0000, 0x0a00_0000, 0x0c00_0000];
const SPLIT_DESCRIPTOR: u32 = 0x3a_ba84;

#[derive(Deserialize)]
struct DispatchCase {
    kind: u8,
    row: u32,
    root: u32,
    descriptor: usize,
    control_bytes: u64,
    stream_file: bool,
}

#[derive(Deserialize)]
struct ExpectedBank {
    sources: Vec<SourceRange>,
    stream_ranges: Vec<Range<usize>>,
    stream_padding: usize,
    descriptor_padding: usize,
    cases: Vec<DispatchCase>,
}

fn input(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os(name).unwrap_or_else(|| panic!("Missing {name}")))
}

fn execute_dispatch(machine: &mut Machine, rom: &[u8], address: u32, case: &DispatchCase) {
    let destination = 0x0200_0000 + DESTINATION.start as u32;
    machine.bus.ewram[DESTINATION].fill(0x5a);
    machine.bus.write32(CONTEXT, address);
    machine.bus.write8(CONTEXT + 7, 0);
    machine.bus.write32(CONTEXT + 0x24, destination);
    machine.cpu.regs = [0; 16];
    machine.cpu.regs[0] = CONTEXT;
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = DISPATCH;
    machine.cpu.cpsr = 0xbf;
    machine.bus.ffta_sprites.enabled = false;
    machine.bus.frame_ready = false;

    let initializer = if case.kind == 4 {
        0x080d_a544
    } else {
        0x080d_a81c
    };
    let descriptor = case.descriptor;
    let parameter = u16::from_le_bytes(rom[descriptor + 4..descriptor + 6].try_into().unwrap());
    let mut entered = false;
    // Mode 4 can enter the full writer once for each cell of a 32-by-32 map.
    for _ in 0..2_000_000 {
        match machine.cpu.regs[15] {
            pc if pc == initializer => {
                assert!(!entered);
                assert_eq!(
                    machine.cpu.regs[..3],
                    [destination, MIRRORS[0] + case.root, u32::from(parameter)],
                    "initializer arguments for {address:#x}"
                );
                if case.kind == 4 {
                    assert_eq!(machine.cpu.regs[3], u32::from(rom[descriptor + 6]));
                }
                entered = true;
            }
            RETURN => {
                assert!(entered, "Missing initializer for {address:#x}");
                assert_eq!(machine.cpu.regs[0], 1, "Return value for {address:#x}");
                return;
            }
            _ => {}
        }
        machine.step();
    }
    panic!(
        "Original tilemap dispatcher did not return for {address:#x}; PC={:#x}, entered={entered}",
        machine.cpu.regs[15]
    );
}

fn check_dispatchers(
    raw: &mut Machine,
    managed: &mut Machine,
    rom: &[u8],
    cases: &[DispatchCase],
) -> ([u64; 2], u64) {
    let owner = managed.bus.rom_resources().unwrap().clone();
    let mut control_counts = [0, 0];
    let initial_stream = owner.tilemap_stream_bank_bytes_read().unwrap();
    for (index, case) in cases.iter().enumerate() {
        let address = MIRRORS[index % MIRRORS.len()] + case.row;
        let initial_control = owner.tilemap_direct_bus_bytes().unwrap();
        let file_before = owner.tilemap_stream_bank_bytes_read().unwrap();
        let prior_before = owner.observed_tilemap_stats().unwrap().source_bytes;
        execute_dispatch(raw, rom, address, case);
        execute_dispatch(managed, rom, address, case);
        assert!(
            managed.bus.ewram == raw.bus.ewram,
            "Tilemap output for {address:#x}"
        );
        assert_eq!(
            managed.cpu.regs, raw.cpu.regs,
            "CPU result for {address:#x}"
        );
        let controls = owner.tilemap_direct_bus_bytes().unwrap() - initial_control;
        assert_eq!(
            controls, case.control_bytes,
            "Control reads for {address:#x}"
        );
        let file = owner.tilemap_stream_bank_bytes_read().unwrap() - file_before;
        let prior = owner.observed_tilemap_stats().unwrap().source_bytes - prior_before;
        assert!(
            if case.stream_file {
                file > 0
            } else {
                prior > 0
            },
            "The stream at {:#x} did not use its registered source",
            case.root
        );
        control_counts[usize::from(case.kind - 4)] += controls;
    }
    (
        control_counts,
        owner.tilemap_stream_bank_bytes_read().unwrap() - initial_stream,
    )
}

#[test]
#[ignore = "exact EUR ROM and an E447/E450/E452/E458 and E307 control oracle"]
fn european_direct_tilemap_bank_files_and_original_dispatchers() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    let work = input("FFTA_TILEMAP_DIRECT_WORK");
    let expected: ExpectedBank =
        serde_json::from_slice(&fs::read(input("FFTA_TILEMAP_DIRECT_PLAN")).unwrap()).unwrap();
    let previous_bytes = fs::read(input("FFTA_TILEMAP_DIRECT_PREDECESSOR")).unwrap();
    let previous: Manifest = parse(&previous_bytes, "E307 manifest").unwrap();
    assert_eq!(
        (previous.extractor_revision, previous.decoder_revision),
        (307, 306)
    );

    let root = tempfile::tempdir_in(&work).unwrap();
    let mut revision = previous.clone();
    (revision.extractor_revision, revision.decoder_revision) = (308, 307);
    let mut extraction = Extraction::from_rom_for_manifest(&rom, &revision).unwrap();
    let prepared = preparation::prepare(root.path(), &mut extraction, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    let dataset = &prepared.directory;
    let manifest_path = dataset.join(MANIFEST_FILE);
    let manifest_bytes = fs::read(&manifest_path).unwrap();
    let manifest: Manifest = parse(&manifest_bytes, "E308 manifest").unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (308, 307)
    );
    assert_eq!(manifest.resources.len(), 155);
    assert_eq!(manifest.resources[..154], previous.resources);
    assert_eq!(manifest.coverage[..154], previous.coverage);
    let resource = &manifest.resources[154];
    assert_eq!(resource.id, TILEMAP_DIRECT_ID);
    assert_eq!(resource.sources, expected.sources);
    let path = dataset.join(TILEMAP_DIRECT_FILE);
    let mut file = fs::read(&path).unwrap();
    assert_eq!(file.len(), 2_537);
    assert_eq!(digest(&file), resource.sha256);
    let reused = preparation::prepare(root.path(), &mut extraction, &mut |_| Ok(())).unwrap();
    assert_eq!(reused.action, "reused");
    assert_eq!(&reused.directory, dataset);
    drop(extraction);

    let mut raw = Machine::new(rom.clone());
    let initial = raw.save_state().unwrap();
    let mut managed = Machine::new(rom.clone());
    fs::write(&manifest_path, previous_bytes).unwrap();
    attach_from_directory(&mut managed, dataset).unwrap();
    let legacy = managed.bus.rom_resources().unwrap();
    assert!(legacy.tilemap_direct_bus_bytes().is_none());
    assert!(legacy.tilemap_type2_bus_bytes().is_some());
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    attach_from_directory(&mut managed, dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    managed.load_state(&initial).unwrap();

    *file.last_mut().unwrap() ^= 1;
    fs::write(&path, &file).unwrap();
    let mut edited = manifest.clone();
    edited.resources[154].sha256 = digest(&file);
    fs::write(&manifest_path, toml::to_string_pretty(&edited).unwrap()).unwrap();
    assert_eq!(
        attach_from_directory(&mut managed, dataset).unwrap_err(),
        "Direct tilemap transfer file differs from authenticated source",
    );
    assert!(Arc::ptr_eq(&owner, managed.bus.rom_resources().unwrap()));
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
    managed.bus.rom[(SPLIT_DESCRIPTOR + 7) as usize] ^= 0xff;
    let before = owner.tilemap_direct_bus_bytes().unwrap();
    let split = MIRRORS[1] + SPLIT_DESCRIPTOR + 4;
    assert_eq!(managed.bus.read32(split), raw.bus.read32(split));
    assert_eq!(owner.tilemap_direct_bus_bytes().unwrap() - before, 3);
    for span in &resource.sources {
        check_stream_bus_reads(
            &mut raw,
            &mut managed,
            span.offset,
            span.offset + span.length,
        );
    }

    for span in expected.stream_ranges {
        managed.bus.rom[span]
            .iter_mut()
            .for_each(|byte| *byte = !*byte);
    }
    for padding in [expected.stream_padding, expected.descriptor_padding] {
        managed.bus.rom[padding] ^= 0xff;
        for mirror in MIRRORS {
            assert_eq!(managed.bus.read8(mirror + padding as u32), !rom[padding]);
        }
    }
    assert_eq!(expected.cases.len(), 145);
    let (control_counts, stream_bytes) =
        check_dispatchers(&mut raw, &mut managed, &rom, &expected.cases);
    assert!(control_counts.into_iter().all(|count| count > 0) && stream_bytes > 0);
    managed.bus.rom.copy_from_slice(&rom);
    let endpoint = managed.save_state().unwrap();
    assert!(
        endpoint == raw.save_state().unwrap(),
        "Final direct transfer states differ"
    );

    let result = serde_json::json!({
        "status": "PASS",
        "revision": [308, 307],
        "published_bytes": file.len(),
        "published_manifest_sha256": digest(&manifest_bytes),
        "source_file_sha256": resource.sha256,
        "original_dispatches": expected.cases.len(),
        "control_bytes_by_type": control_counts,
        "encoded_file_reads": stream_bytes,
        "guest_state_sha256": digest(&endpoint),
        "e307_attach": true,
        "split_descriptor_boundary": true,
        "raw_padding": true,
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
