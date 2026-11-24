//! Check the European transfer-list sources through their original load instructions.
use super::{
    attach_from_directory, parse, preparation,
    tilemap_origin_tests::{read_literals, Literal},
    tilemap_transfer_directory_tests::check_stream_bus_reads,
    Extraction, Manifest, MANIFEST_FILE, TILEMAP_DIRECT_FILE, TILEMAP_DIRECT_ID,
};
use gba_core::{ffta_rom::digest, rom_data::SourceRange, Machine};
use serde::Deserialize;
use std::{fs, path::PathBuf, sync::Arc};

const MIRRORS: [u32; 3] = [0x0800_0000, 0x0a00_0000, 0x0c00_0000];

#[derive(Deserialize)]
struct TableLoad {
    reader: u32,
    offset: u32,
    width: u32,
    value: u32,
    from_direct_file: bool,
}

#[derive(Deserialize)]
struct Projection {
    sources: Vec<SourceRange>,
    control_fields: Vec<(u32, u32)>,
    literals: Vec<Literal>,
    table_loads: Vec<TableLoad>,
}

fn input(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os(name).unwrap_or_else(|| panic!("Missing {name}")))
}

fn read_table_cells(
    raw: &mut Machine,
    managed: &mut Machine,
    rom: &[u8],
    cases: &[TableLoad],
) -> u64 {
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = owner.tilemap_direct_bus_bytes().unwrap();
    for (index, case) in cases.iter().enumerate() {
        let mirror = MIRRORS[index % MIRRORS.len()];
        let pc = mirror + case.reader;
        let reader = case.reader as usize;
        let opcode = u16::from_le_bytes(rom[reader..reader + 2].try_into().unwrap());
        let family = match case.width {
            1 => 0x7800,
            4 => 0x6800,
            _ => panic!("Invalid directory read width"),
        };
        assert_eq!(opcode & 0xf800, family);
        let displacement = u32::from((opcode >> 6) & 31) * case.width;
        let address_register = usize::from((opcode >> 3) & 7);
        let value_register = usize::from(opcode & 7);
        let before = owner.tilemap_direct_bus_bytes().unwrap();
        for machine in [&mut *raw, &mut *managed] {
            machine.cpu.regs = [0; 16];
            machine.cpu.regs[15] = pc;
            machine.cpu.regs[address_register] = mirror + case.offset - displacement;
            machine.cpu.cpsr = 0xbf;
            machine.bus.frame_ready = false;
            machine.step();
            assert_eq!(
                machine.cpu.regs[value_register], case.value,
                "Directory cell {:#x} from reader {pc:#x}",
                case.offset,
            );
            assert_eq!(machine.cpu.regs[15], pc + 2);
        }
        assert_eq!(
            owner.tilemap_direct_bus_bytes().unwrap() - before,
            if case.from_direct_file {
                u64::from(case.width)
            } else {
                0
            },
            "Directory cell owner for {:#x}",
            case.offset,
        );
        assert_eq!(managed.cpu.regs, raw.cpu.regs);
    }
    owner.tilemap_direct_bus_bytes().unwrap() - initial
}

#[test]
#[ignore = "exact EUR ROM and E446/E451 directory projection"]
fn european_tilemap_directory_sources_and_original_loads() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    let work = input("FFTA_TILEMAP_LIST_WORK");
    let expected: Projection =
        serde_json::from_slice(&fs::read(input("FFTA_TILEMAP_LIST_PLAN")).unwrap()).unwrap();
    let previous_bytes = fs::read(input("FFTA_TILEMAP_LIST_PREDECESSOR")).unwrap();
    let previous: Manifest = parse(&previous_bytes, "E309 manifest").unwrap();
    assert_eq!(
        (previous.extractor_revision, previous.decoder_revision),
        (309, 308)
    );

    let root = tempfile::tempdir_in(&work).unwrap();
    let mut revision = previous.clone();
    (revision.extractor_revision, revision.decoder_revision) = (310, 309);
    let mut extraction = Extraction::from_rom_for_manifest(&rom, &revision).unwrap();
    let prepared = preparation::prepare(root.path(), &mut extraction, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    let dataset = &prepared.directory;
    let manifest_path = dataset.join(MANIFEST_FILE);
    let manifest_bytes = fs::read(&manifest_path).unwrap();
    let manifest: Manifest = parse(&manifest_bytes, "E310 manifest").unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (310, 309)
    );
    assert_eq!(manifest.resources.len(), 155);
    assert_eq!(manifest.resources[..154], previous.resources[..154]);
    assert_eq!(manifest.coverage, previous.coverage);
    let resource = &manifest.resources[154];
    assert_eq!(resource.id, TILEMAP_DIRECT_ID);
    assert_eq!(resource.sources, expected.sources);
    let path = dataset.join(TILEMAP_DIRECT_FILE);
    let mut file = fs::read(&path).unwrap();
    assert_eq!(file.len(), 4_635);
    assert_eq!(digest(&file), resource.sha256);
    let reused = preparation::prepare(root.path(), &mut extraction, &mut |_| Ok(())).unwrap();
    assert_eq!(reused.action, "reused");
    assert_eq!(&reused.directory, dataset);
    drop(extraction);

    let old = &previous.resources[154];
    assert_eq!(old.id, TILEMAP_DIRECT_ID);
    let old_file: Vec<_> = old
        .sources
        .iter()
        .flat_map(|span| {
            let start = span.offset as usize;
            rom[start..start + span.length as usize].iter().copied()
        })
        .collect();
    assert_eq!(digest(&old_file), old.sha256);
    let mut managed = Machine::new(rom.clone());
    fs::write(&path, old_file).unwrap();
    fs::write(&manifest_path, previous_bytes).unwrap();
    attach_from_directory(&mut managed, dataset).unwrap();
    assert_eq!(
        managed
            .bus
            .rom_resources()
            .unwrap()
            .tilemap_direct_bus_bytes(),
        Some(0)
    );
    fs::write(&path, &file).unwrap();
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    attach_from_directory(&mut managed, dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();

    let edited_byte: usize = resource
        .sources
        .iter()
        .take_while(|span| span.role != "directory-count")
        .map(|span| span.length as usize)
        .sum();
    file[edited_byte] ^= 1;
    fs::write(&path, &file).unwrap();
    let mut edited = manifest.clone();
    edited.resources[154].sha256 = digest(&file);
    fs::write(&manifest_path, toml::to_string_pretty(&edited).unwrap()).unwrap();
    assert_eq!(
        attach_from_directory(&mut managed, dataset).unwrap_err(),
        "Direct tilemap transfer file differs from authenticated source",
    );
    assert!(Arc::ptr_eq(&owner, managed.bus.rom_resources().unwrap()));

    let mut raw = Machine::new(rom.clone());
    for &(offset, width) in &expected.control_fields {
        let start = offset as usize;
        for byte in &mut managed.bus.rom[start..start + width as usize] {
            *byte ^= 0xff;
        }
    }
    for &(offset, width) in &expected.control_fields {
        check_stream_bus_reads(&mut raw, &mut managed, offset, offset + width);
    }
    assert_eq!(expected.literals.len(), 280);
    assert_eq!(expected.table_loads.len(), 209);
    let literal_bytes = read_literals(&mut raw, &mut managed, &rom, &expected.literals);
    let table_bytes = read_table_cells(&mut raw, &mut managed, &rom, &expected.table_loads);
    assert_eq!((literal_bytes, table_bytes), (1084, 410));
    managed.bus.rom.copy_from_slice(&rom);
    let endpoint = managed.save_state().unwrap();
    assert!(
        endpoint == raw.save_state().unwrap(),
        "Final directory reader states differ"
    );
    let result = serde_json::json!({
        "status": "PASS",
        "revision": [310, 309],
        "published_bytes": file.len(),
        "published_manifest_sha256": digest(&manifest_bytes),
        "source_file_sha256": resource.sha256,
        "original_literal_loads": expected.literals.len(),
        "original_table_loads": expected.table_loads.len(),
        "new_literal_bytes": literal_bytes,
        "new_table_bytes": table_bytes,
        "guest_state_sha256": digest(&endpoint),
        "e309_attach": true,
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
