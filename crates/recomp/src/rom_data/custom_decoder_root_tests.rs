//! Verify the E502 source-base literals through the original table readers.

use super::{
    attach_from_directory, cmd, encode_thumb_pc_literals, parse, parse_thumb_pc_literals,
    Extraction, Manifest, MANIFEST_FILE, THUMB_PC_LITERALS_FILE, THUMB_PC_LITERALS_ID,
};
use gba_core::{
    ffta_rom::digest,
    rom_data::read_audit::{Service, TraceCapture, TraceFilter},
    Bus, Machine,
};
use serde::Deserialize;
use std::{collections::BTreeSet, fs, path::PathBuf, sync::Arc};

const BASE: u32 = 0x08000000;
const LANGUAGE: u32 = 0x02001f6e;

#[derive(Deserialize)]
struct Plan {
    sites: Vec<Site>,
    unowned: Vec<Literal>,
}

#[derive(Deserialize)]
struct Site {
    family: String,
    literal: Option<Literal>,
    index_reader: Option<u32>,
}

#[derive(Deserialize)]
struct Literal {
    offset: u32,
    reader: u32,
    opcode: u16,
    root: u32,
    sha256: String,
    owner: Option<String>,
}

fn input(name: &str) -> PathBuf {
    std::env::var_os(name).unwrap().into()
}

fn half(rom: &[u8], at: u32) -> u16 {
    let at = at as usize;
    u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())
}

fn word(rom: &[u8], at: u32) -> u32 {
    let at = at as usize;
    u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
}

fn table_read(machine: &mut Machine, site: &Site, literal: &Literal, index: u8, expected: u32) {
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs.fill(0);
    machine.cpu.regs[13] = 0x03007c00;
    machine.cpu.regs[15] = BASE + literal.reader;
    machine.cpu.cpsr = 0xbf;
    machine.cpu.branch = None;
    let table_register = usize::from((literal.opcode >> 8) & 7);
    if site.family == "alternate-bg-palette" {
        machine.cpu.regs[1] = u32::from(index);
    }
    machine.step();
    assert_eq!(machine.cpu.regs[table_register], BASE + literal.root);
    let (reader, target) = if let Some(reader) = site.index_reader {
        assert_eq!(site.family, "language-table");
        // Enter the original lookup with its language address established.
        // This skips earlier setup calls for the two preserved-register paths.
        let opcode = half(&machine.bus.rom, reader);
        let index_address_register = usize::from((opcode >> 3) & 7);
        assert_ne!(index_address_register, table_register);
        machine.cpu.regs[index_address_register] = LANGUAGE;
        machine.bus.write8(LANGUAGE, index);
        let target = usize::from(half(&machine.bus.rom, reader + 6) & 7);
        (reader, target)
    } else {
        assert_eq!(site.family, "alternate-bg-palette");
        (literal.reader + 2, 1)
    };
    machine.cpu.regs[15] = BASE + reader;
    for _ in 0..4 {
        machine.step();
    }
    assert_eq!(machine.cpu.regs[15], BASE + reader + 8);
    assert_eq!(machine.cpu.regs[target], expected);
}

#[test]
#[ignore = "exact EUR ROM, E502 plan and E501 predecessor manifest"]
fn european_custom_decoder_root_publication_and_source_reads() {
    let rom_path = input("FFTA_STATE_TEST_ROM");
    let rom = fs::read(&rom_path).unwrap();
    let work = input("FFTA_CUSTOM_DECODER_ROOTS_WORK");
    let plan: Plan =
        serde_json::from_slice(&fs::read(input("FFTA_CUSTOM_DECODER_ROOTS_PLAN")).unwrap())
            .unwrap();
    let predecessor_bytes = fs::read(input("FFTA_CUSTOM_DECODER_ROOTS_PREDECESSOR")).unwrap();
    let predecessor: Manifest = parse(&predecessor_bytes, "E501 manifest").unwrap();
    assert_eq!(
        (predecessor.extractor_revision, predecessor.decoder_revision),
        (313, 312)
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
        .join(&predecessor.source_rom_sha256)
        .join("v1/e314-d313");
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
    let manifest: Manifest = parse(&manifest_bytes, "E503 manifest").unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (314, 313)
    );
    assert_eq!(manifest.coverage, predecessor.coverage);
    assert_eq!(manifest.resources.len(), predecessor.resources.len());
    let index = manifest
        .resources
        .iter()
        .position(|entry| entry.id == THUMB_PC_LITERALS_ID)
        .unwrap();
    for (row, (new, old)) in manifest
        .resources
        .iter()
        .zip(&predecessor.resources)
        .enumerate()
    {
        if row != index {
            assert_eq!(new, old);
        }
    }
    let old = &predecessor.resources[index];
    let new = &manifest.resources[index];
    assert_eq!(
        (&new.path, &new.format, &new.dependencies),
        (&old.path, &old.format, &old.dependencies)
    );
    assert!(new.sources.starts_with(&old.sources));
    let added = &new.sources[old.sources.len()..];
    assert_eq!(added.len(), 31);
    assert_eq!(plan.unowned.len(), added.len());
    for (source, witness) in added.iter().zip(&plan.unowned) {
        assert_eq!(
            (source.offset, source.length, &source.sha256),
            (witness.offset, 4, &witness.sha256)
        );
        assert_eq!(source.role, "thumb-pc-literal");
    }

    let literal_path = dataset.join(THUMB_PC_LITERALS_FILE);
    let original_file = fs::read(&literal_path).unwrap();
    let previous_extraction = Extraction::from_rom_for_manifest(&rom, &predecessor).unwrap();
    let prior_literals = previous_extraction.thumb_pc_literals.unwrap().decoded;
    assert!(prior_literals.custom_decoder_roots.is_empty());
    let previous_file = encode_thumb_pc_literals(&prior_literals).unwrap();
    assert_eq!(digest(&previous_file), old.sha256);
    let mut managed = Machine::new(rom.clone());
    fs::write(&manifest_path, predecessor_bytes).unwrap();
    fs::write(&literal_path, previous_file).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let prior_owner = managed.bus.rom_resources().unwrap().clone();
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    fs::write(&literal_path, &original_file).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    assert!(!Arc::ptr_eq(&prior_owner, &owner));

    let mut edited = parse_thumb_pc_literals(&original_file).unwrap();
    edited.custom_decoder_roots[0].value ^= 4;
    edited.validate().unwrap();
    let edited = encode_thumb_pc_literals(&edited).unwrap();
    let mut forged = manifest.clone();
    forged.resources[index].sha256 = digest(&edited);
    fs::write(&manifest_path, toml::to_string_pretty(&forged).unwrap()).unwrap();
    fs::write(&literal_path, edited).unwrap();
    assert!(attach_from_directory(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(&owner, managed.bus.rom_resources().unwrap()));
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    fs::write(&literal_path, original_file).unwrap();

    let ranges: Vec<_> = added
        .iter()
        .map(|source| (source.offset, source.offset + 4))
        .collect();
    for &(start, end) in &ranges {
        for byte in &mut managed.bus.rom[start as usize..end as usize] {
            *byte ^= 0xff;
        }
    }
    let mut raw = Machine::new(rom);
    let capture = TraceCapture::start(
        TraceFilter {
            readers: Vec::new(),
            ranges,
        },
        256,
    )
    .unwrap();
    capture.frame(Some(0));
    let mut site_count = 0;
    let selected_sites = plan.sites.iter().filter_map(|site| {
        site.literal
            .as_ref()
            .filter(|literal| literal.owner.is_none())
            .map(|literal| (site, literal))
    });
    for (site, literal) in selected_sites {
        let last = match site.family.as_str() {
            "language-table" => 4,
            "alternate-bg-palette" => 156,
            family => panic!("Unexpected E502 source family {family}"),
        };
        for index in [0, last] {
            let expected = if last == 4 {
                word(&raw.bus.rom, literal.root + 4 * u32::from(index))
            } else {
                BASE + literal.root
                    + u32::from(half(&raw.bus.rom, literal.root + 2 * u32::from(index)))
            };
            table_read(&mut raw, site, literal, index, expected);
            table_read(&mut managed, site, literal, index, expected);
            assert_eq!(raw.cpu.regs, managed.cpu.regs);
            assert_eq!(raw.cpu.cpsr, managed.cpu.cpsr);
            assert_eq!(raw.bus.clock, managed.bus.clock);
        }
        site_count += 1;
    }
    let trace = capture.finish();
    assert!(!trace.truncated);
    assert_eq!(site_count, 32);
    assert_eq!(trace.matched_requests, 128);
    for service in [Service::Raw, Service::Typed] {
        let offsets: BTreeSet<_> = trace
            .events
            .iter()
            .filter(|event| event.key.service == service)
            .map(|event| {
                let source = &event.key.read;
                let literal = plan
                    .unowned
                    .iter()
                    .find(|row| row.offset == source.physical_offset)
                    .unwrap();
                assert_eq!(source.length, 4);
                assert_eq!(event.key.instruction.unwrap().pc, BASE + literal.reader);
                source.physical_offset
            })
            .collect();
        assert_eq!(offsets, plan.unowned.iter().map(|row| row.offset).collect());
    }
    let stats = owner.thumb_pc_literals_stats().unwrap();
    assert_eq!(stats.source_bytes, 256);
    assert_eq!(stats.star_cross_observed_bytes, 0);

    let result = serde_json::json!({
        "status": "PASS", "revision": [314, 313], "source_bytes": 124,
        "roots": added.len(), "reader_paths": site_count, "lookup_cases": 2 * site_count,
        "typed_source_bytes": stats.source_bytes, "predecessor_attach": true, "rehashed_edit_rejected": true,
        "manifest_sha256": digest(&manifest_bytes), "literal_file_sha256": new.sha256,
    });
    fs::write(work.join("manifest.toml"), manifest_bytes).unwrap();
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
