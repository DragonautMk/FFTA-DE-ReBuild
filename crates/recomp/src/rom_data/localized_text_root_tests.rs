//! Verify every declared European text selector and the two keyboard root loads.

use super::{
    attach_from_directory, cmd, encode_thumb_pc_literals, parse, parse_thumb_pc_literals,
    text_catalog_tests::invoke, Extraction, Manifest, MANIFEST_FILE, THUMB_PC_LITERALS_FILE,
    THUMB_PC_LITERALS_ID,
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
    selections: Vec<Selection>,
    unowned: Vec<Selection>,
}

#[derive(Deserialize)]
struct Selection {
    family: String,
    entry: u32,
    language: Option<u8>,
    offset: u32,
    reader: u32,
    root: u32,
    sha256: String,
}

fn input(name: &str) -> PathBuf {
    std::env::var_os(name).unwrap().into()
}

fn execute(machine: &mut Machine, selection: &Selection, language: Option<u8>) -> u32 {
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs.fill(0);
    machine.cpu.branch = None;
    if let Some(language) = language {
        machine.bus.write8(LANGUAGE, language);
        invoke(machine, BASE + selection.entry, [0; 4]);
        machine.cpu.regs[0]
    } else {
        assert_eq!(selection.family, "name-keyboard");
        machine.cpu.regs[15] = BASE + selection.reader;
        machine.cpu.cpsr = 0xbf;
        machine.step();
        assert_eq!(machine.cpu.regs[15], BASE + selection.reader + 2);
        machine.cpu.regs[1]
    }
}

fn check_selection(
    raw: &mut Machine,
    managed: &mut Machine,
    selection: &Selection,
    language: Option<u8>,
) {
    let expected = if selection.family == "text-pointer-table" {
        let start = selection.root as usize;
        u32::from_le_bytes(raw.bus.rom[start..start + 4].try_into().unwrap())
    } else {
        BASE + selection.root
    };
    let capture = TraceCapture::start(
        TraceFilter {
            readers: Vec::new(),
            ranges: vec![(selection.offset, selection.offset + 4)],
        },
        4,
    )
    .unwrap();
    capture.frame(Some(0));
    assert_eq!(execute(raw, selection, language), expected);
    assert_eq!(execute(managed, selection, language), expected);
    assert_eq!(raw.cpu.regs, managed.cpu.regs);
    assert_eq!(raw.cpu.cpsr, managed.cpu.cpsr);
    assert_eq!(raw.bus.clock, managed.bus.clock);
    let trace = capture.finish();
    assert!(!trace.truncated);
    assert_eq!(trace.matched_requests, 2);
    assert_eq!(trace.events.len(), 2);
    for service in [Service::Raw, Service::Typed] {
        let event = trace
            .events
            .iter()
            .find(|row| row.key.service == service)
            .unwrap();
        assert_eq!(event.key.read.physical_offset, selection.offset);
        assert_eq!(event.key.read.length, 4);
        assert_eq!(event.key.instruction.unwrap().pc, BASE + selection.reader);
        assert_eq!(event.requests, 1);
    }
}

#[test]
#[ignore = "exact EUR ROM, E504 plan and E503 predecessor manifest"]
fn european_localized_text_root_publication_and_original_selectors() {
    let rom_path = input("FFTA_STATE_TEST_ROM");
    let rom = fs::read(&rom_path).unwrap();
    let work = input("FFTA_LOCALIZED_TEXT_ROOTS_WORK");
    let plan: Plan =
        serde_json::from_slice(&fs::read(input("FFTA_LOCALIZED_TEXT_ROOTS_PLAN")).unwrap())
            .unwrap();
    let predecessor_bytes = fs::read(input("FFTA_LOCALIZED_TEXT_ROOTS_PREDECESSOR")).unwrap();
    let predecessor: Manifest = parse(&predecessor_bytes, "E503 manifest").unwrap();
    assert_eq!(
        (predecessor.extractor_revision, predecessor.decoder_revision),
        (314, 313)
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
        .join("v1/e315-d314");
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
    let manifest: Manifest = parse(&manifest_bytes, "E505 manifest").unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (315, 314)
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
    assert_eq!(
        (added.len(), plan.unowned.len(), plan.selections.len()),
        (192, 192, 197)
    );
    for (source, witness) in added.iter().zip(&plan.unowned) {
        assert_eq!(
            (source.offset, source.length, &source.sha256),
            (witness.offset, 4, &witness.sha256)
        );
        assert_eq!(source.role, "thumb-pc-literal");
    }

    let literal_path = dataset.join(THUMB_PC_LITERALS_FILE);
    let literal_bytes = fs::read(&literal_path).unwrap();
    let previous_extraction = Extraction::from_rom_for_manifest(&rom, &predecessor).unwrap();
    let prior_literals = previous_extraction.thumb_pc_literals.unwrap().decoded;
    assert!(prior_literals.localized_text_roots.is_empty());
    assert_eq!(prior_literals.custom_decoder_roots.len(), 31);
    let previous_file = encode_thumb_pc_literals(&prior_literals).unwrap();
    assert_eq!(digest(&previous_file), old.sha256);
    let mut managed = Machine::new(rom.clone());
    fs::write(&manifest_path, predecessor_bytes).unwrap();
    fs::write(&literal_path, previous_file).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let previous_owner = managed.bus.rom_resources().unwrap().clone();
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    fs::write(&literal_path, &literal_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    assert!(!Arc::ptr_eq(&previous_owner, &owner));

    let mut edited = parse_thumb_pc_literals(&literal_bytes).unwrap();
    edited.localized_text_roots[0].value ^= 4;
    edited.validate().unwrap();
    let edited = encode_thumb_pc_literals(&edited).unwrap();
    let mut forged = manifest.clone();
    forged.resources[index].sha256 = digest(&edited);
    fs::write(&manifest_path, toml::to_string_pretty(&forged).unwrap()).unwrap();
    fs::write(&literal_path, edited).unwrap();
    assert!(attach_from_directory(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(&owner, managed.bus.rom_resources().unwrap()));
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    fs::write(&literal_path, literal_bytes).unwrap();

    for source in added {
        for byte in &mut managed.bus.rom[source.offset as usize..(source.offset + 4) as usize] {
            *byte ^= 0xff;
        }
    }
    let mut raw = Machine::new(rom);
    let mut roots = BTreeSet::new();
    for selection in &plan.selections {
        assert!(roots.insert(selection.offset));
        check_selection(&mut raw, &mut managed, selection, selection.language);
    }
    let mut fallback_cases = 0;
    for family in [
        "text-pointer-table",
        "text-archive-index",
        "script-text-language",
    ] {
        let first = plan
            .selections
            .iter()
            .find(|row| row.family == family && row.language == Some(0))
            .unwrap();
        for unknown in [5, 255] {
            check_selection(&mut raw, &mut managed, first, Some(unknown));
            fallback_cases += 1;
        }
    }
    assert!(added.iter().all(|source| roots.contains(&source.offset)));

    let result = serde_json::json!({
        "status": "PASS", "revision": [315, 314], "source_bytes": 768, "new_roots": added.len(),
        "original_loads": roots.len(), "fallback_cases": fallback_cases,
        "predecessor_attach": true, "rehashed_edit_rejected": true,
        "manifest_sha256": digest(&manifest_bytes), "literal_file_sha256": new.sha256,
    });
    fs::write(work.join("manifest.toml"), manifest_bytes).unwrap();
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
