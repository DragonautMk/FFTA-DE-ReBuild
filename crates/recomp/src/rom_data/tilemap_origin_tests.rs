//! Wine proof for the authenticated tilemap origin literal bank.
use super::{
    attach_from_directory, parse, preparation,
    tilemap_transfer_directory_tests::check_stream_bus_reads, Extraction, Manifest, MANIFEST_FILE,
    TILEMAP_DIRECT_FILE, TILEMAP_DIRECT_ID,
};
use gba_core::{ffta_rom::digest, rom_data::SourceRange, Bus, Machine};
use serde::Deserialize;
use std::{fs, path::PathBuf, sync::Arc};

#[derive(Deserialize)]
pub(super) struct Literal {
    reader: u32,
    offset: u32,
    pointer: u32,
    from_direct_file: bool,
}

#[derive(Deserialize)]
struct Projection {
    sources: Vec<SourceRange>,
    literals: Vec<Literal>,
}

fn input(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os(name).unwrap_or_else(|| panic!("Missing {name}")))
}

pub(super) fn read_literals(
    raw: &mut Machine,
    managed: &mut Machine,
    rom: &[u8],
    literals: &[Literal],
) -> u64 {
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = owner.tilemap_direct_bus_bytes().unwrap();
    for (index, literal) in literals.iter().enumerate() {
        let mirror = [0x0800_0000, 0x0a00_0000, 0x0c00_0000][index % 3];
        let reader = (mirror + literal.reader) as usize;
        let opcode_at = literal.reader as usize;
        let opcode = u16::from_le_bytes(rom[opcode_at..opcode_at + 2].try_into().unwrap());
        assert_eq!(opcode & 0xf800, 0x4800);
        assert_eq!(
            ((literal.reader + 4) & !3) + 4 * u32::from(opcode & 0xff),
            literal.offset
        );
        let register = usize::from((opcode >> 8) & 7);
        let before = owner.tilemap_direct_bus_bytes().unwrap();
        for machine in [&mut *raw, &mut *managed] {
            machine.cpu.regs = [0; 16];
            machine.cpu.regs[15] = reader as u32;
            machine.cpu.cpsr = 0xbf;
            machine.bus.frame_ready = false;
            machine.step();
            assert_eq!(
                machine.cpu.regs[register], literal.pointer,
                "Literal {:#x}",
                literal.offset
            );
            assert_eq!(machine.cpu.regs[15], reader as u32 + 2);
        }
        assert_eq!(
            owner.tilemap_direct_bus_bytes().unwrap() - before,
            if literal.from_direct_file { 4 } else { 0 },
            "Origin literal owner for {:#x}",
            literal.offset,
        );
        assert_eq!(managed.cpu.regs, raw.cpu.regs);
    }
    owner.tilemap_direct_bus_bytes().unwrap() - initial
}

#[test]
#[ignore = "exact EUR ROM and E458/E459 origin literal projection"]
fn european_tilemap_origin_literals_and_original_loads() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    let work = input("FFTA_TILEMAP_ORIGIN_WORK");
    let expected: Projection =
        serde_json::from_slice(&fs::read(input("FFTA_TILEMAP_ORIGIN_PLAN")).unwrap()).unwrap();
    let previous_bytes = fs::read(input("FFTA_TILEMAP_ORIGIN_PREDECESSOR")).unwrap();
    let previous: Manifest = parse(&previous_bytes, "E308 manifest").unwrap();
    assert_eq!(
        (previous.extractor_revision, previous.decoder_revision),
        (308, 307)
    );

    let root = tempfile::tempdir_in(&work).unwrap();
    let mut revision = previous.clone();
    (revision.extractor_revision, revision.decoder_revision) = (309, 308);
    let mut extraction = Extraction::from_rom_for_manifest(&rom, &revision).unwrap();
    let prepared = preparation::prepare(root.path(), &mut extraction, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    let dataset = &prepared.directory;
    let manifest_path = dataset.join(MANIFEST_FILE);
    let manifest_bytes = fs::read(&manifest_path).unwrap();
    let manifest: Manifest = parse(&manifest_bytes, "E309 manifest").unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (309, 308)
    );
    assert_eq!(manifest.resources.len(), 155);
    assert_eq!(manifest.resources[..154], previous.resources[..154]);
    assert_eq!(manifest.coverage, previous.coverage);
    let resource = &manifest.resources[154];
    assert_eq!(resource.id, TILEMAP_DIRECT_ID);
    assert_eq!(resource.sources, expected.sources);
    let path = dataset.join(TILEMAP_DIRECT_FILE);
    let mut file = fs::read(&path).unwrap();
    assert_eq!(file.len(), 3_141);
    assert_eq!(digest(&file), resource.sha256);
    let reused = preparation::prepare(root.path(), &mut extraction, &mut |_| Ok(())).unwrap();
    assert_eq!(reused.action, "reused");
    assert_eq!(&reused.directory, dataset);
    drop(extraction);

    let old = &previous.resources[154];
    assert_eq!(old.id, TILEMAP_DIRECT_ID);
    let raw_range = |span: &SourceRange| {
        let start = span.offset as usize;
        &rom[start..start + span.length as usize]
    };
    let old_file: Vec<_> = old.sources.iter().flat_map(raw_range).copied().collect();
    assert_eq!(digest(&old_file), old.sha256);
    let mut managed = Machine::new(rom.clone());
    fs::write(&path, old_file).unwrap();
    fs::write(&manifest_path, previous_bytes).unwrap();
    attach_from_directory(&mut managed, dataset).unwrap();
    let old_owner = managed.bus.rom_resources().unwrap().clone();
    let new_literal = expected
        .literals
        .iter()
        .find(|literal| literal.from_direct_file)
        .unwrap();
    let offset = new_literal.offset as usize;
    managed.bus.rom[offset] ^= 0xff;
    assert_eq!(
        managed.bus.read8(0x0c00_0000 + new_literal.offset),
        !rom[offset]
    );
    assert_eq!(old_owner.tilemap_direct_bus_bytes(), Some(0));
    managed.bus.rom[offset] = rom[offset];

    fs::write(&path, &file).unwrap();
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    attach_from_directory(&mut managed, dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let edited_byte: usize = resource
        .sources
        .iter()
        .take_while(|span| span.role != "origin-literal")
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
    assert_eq!(expected.literals.len(), 153);
    for literal in &expected.literals {
        let at = literal.offset as usize;
        for byte in &mut managed.bus.rom[at..at + 4] {
            *byte ^= 0xff;
        }
    }
    for literal in &expected.literals {
        check_stream_bus_reads(&mut raw, &mut managed, literal.offset, literal.offset + 4);
    }
    let served = read_literals(&mut raw, &mut managed, &rom, &expected.literals);
    assert_eq!(served, 604);
    managed.bus.rom.copy_from_slice(&rom);
    let endpoint = managed.save_state().unwrap();
    assert!(
        endpoint == raw.save_state().unwrap(),
        "Final literal reader states differ"
    );
    let result = serde_json::json!({
        "status": "PASS",
        "revision": [309, 308],
        "published_bytes": file.len(),
        "published_manifest_sha256": digest(&manifest_bytes),
        "source_file_sha256": resource.sha256,
        "original_ldr_calls": expected.literals.len(),
        "new_ldr_bytes": served,
        "guest_state_sha256": digest(&endpoint),
        "e308_attach": true,
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
