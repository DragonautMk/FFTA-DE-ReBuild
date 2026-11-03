//! Ninth EUR managed file; completed original string/font proof is not repeated.
use super::*;
use gba_core::rom_data::{text_short, Resource, SourceRange, SHORT_TEXTS_FORMAT};
use std::sync::Arc;

pub(super) fn fixture() -> Extraction {
    let mut source = super::bg_transfer_managed_tests::fixture();
    let strings = text_short::STRING_STARTS
        .iter()
        .map(|&source_offset| text_short::Text {
            source_offset,
            encoding: text_short::Encoding::Compact,
            units: vec![text_short::Unit::Glyph { index: 0 }],
        })
        .collect();
    let decoded = ShortTexts {
        languages: text_short::TABLES
            .iter()
            .enumerate()
            .map(|(id, &table_offset)| text_short::Language {
                language_id: id as u8,
                table_offset,
                entries: vec![id as u16; text_short::ENTRY_COUNT],
            })
            .collect(),
        strings,
        glyphs: vec![text_short::Glyph {
            index: 0,
            width: 7,
            tile_words: vec![0x12345678; 32],
        }],
    };
    decoded.validate().unwrap();
    let mut ranges = Vec::new();
    let mut add = |offset, bytes: Vec<u8>, role| {
        ranges.push(SourceRange {
            offset,
            length: bytes.len() as u32,
            sha256: gba_core::ffta_rom::digest(&bytes),
            role,
        })
    };
    for l in &decoded.languages {
        add(
            l.table_offset,
            decoded.table_bytes(l).unwrap(),
            format!("short-text-table/{}", l.language_id),
        );
    }
    for (id, t) in decoded.strings.iter().enumerate() {
        add(
            t.source_offset,
            t.source_bytes().unwrap(),
            format!("short-text-string/{id}"),
        );
    }
    for g in &decoded.glyphs {
        add(
            g.source_offset(),
            g.source_bytes(),
            format!("short-text-glyph/{}", g.index),
        );
        add(
            g.width_offset(),
            vec![g.width],
            format!("short-text-width/{}", g.index),
        );
    }
    source.short_texts = Some(text_short::Catalog {
        decoded,
        sources: ranges.clone(),
    });
    source.manifest.coverage.push(SHORT_TEXTS_ID.into());
    source.manifest.resources.push(Resource {
        id: SHORT_TEXTS_ID.into(),
        path: SHORT_TEXTS_FILE.into(),
        format: SHORT_TEXTS_FORMAT.into(),
        sha256: String::new(),
        dependencies: Vec::new(),
        sources: ranges,
    });
    source
}

#[test]
fn ninth_text_file_publication_and_no_write_reuse_are_deterministic() {
    let root = tempfile::tempdir().unwrap();
    let mut source = fixture();
    let a = root.path().join("first");
    let b = root.path().join("second");
    assert_eq!(extract(&a, &mut source).unwrap(), "created");
    assert_eq!(extract(&b, &mut source).unwrap(), "created");
    assert_eq!(fs::read_dir(&a).unwrap().count(), 9);
    for entry in fs::read_dir(&a).unwrap() {
        let name = entry.unwrap().file_name();
        assert_eq!(
            fs::read(a.join(&name)).unwrap(),
            fs::read(b.join(name)).unwrap()
        );
    }
    let first = preparation::prepare(root.path(), &mut source, &mut |_| Ok(())).unwrap();
    let reused = preparation::prepare(root.path(), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(reused.action, "reused");
    assert_eq!(reused.directory, first.directory);
    verify(&a, &source).unwrap();
}

#[test]
#[ignore = "requires exact EUR ROM, retained e10 and typed02 files, absent FFTA_STATE_TEST_TEXT_MANAGED_ROOT"]
fn european_text_dataset_authenticates_repairs_moves_and_retains_e10() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_TEXT_MANAGED_ROOT").unwrap());
    assert!(!root.exists());
    let old = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_OLD_E10_DATASET").unwrap());
    let typed = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_TEXT_TYPED_JSON").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let names = [
        MANIFEST_FILE,
        OBJ_DIMENSIONS_FILE,
        BG_DESCRIPTORS_FILE,
        BG_BASE_PALETTES_FILE,
        BG_ALTERNATE_PALETTES_FILE,
        BG_PACKET_PROGRAMS_FILE,
        BG_MAIN_GRAPHICS_FILE,
        BG_TRANSFER_PROGRAMS_FILE,
        SHORT_TEXTS_FILE,
    ];
    let old_files: Vec<_> = names[..8]
        .iter()
        .map(|n| fs::read(old.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old_files[0], "prior e10 manifest").unwrap();
    assert_eq!(
        (
            prior.schema,
            prior.extractor_revision,
            prior.decoder_revision
        ),
        (1, 10, 9)
    );
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.schema,
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (1, 11, 10)
    );
    assert_eq!(source.manifest.resources.len(), 8);
    assert_eq!(source.manifest.resources[7].sources.len(), 670);
    let offline = root.join("offline dataset");
    extract(&offline, &mut source).unwrap();
    assert_eq!(fs::read_dir(&offline).unwrap().count(), 9);
    let files: Vec<_> = names
        .iter()
        .map(|n| fs::read(offline.join(n)).unwrap())
        .collect();
    assert_eq!(files[8], fs::read(typed).unwrap());
    for i in 1..8 {
        assert_eq!(files[i], old_files[i]);
    }
    let managed = root.join("resources");
    let retained = managed
        .join(&prior.source_rom_sha256)
        .join("v1/e10-d9/generation-retained");
    fs::create_dir_all(&retained).unwrap();
    for (name, bytes) in names[..8].iter().zip(&old_files) {
        fs::write(retained.join(name), bytes).unwrap();
    }
    fs::write(
        retained.parent().unwrap().join("current"),
        b"generation-retained\n",
    )
    .unwrap();
    let first = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(first.action, "created");
    assert!(first.directory.parent().unwrap().ends_with("v1/e11-d10"));
    let reused = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(reused.action, "reused");
    assert_eq!(reused.directory, first.directory);
    fs::rename(
        first.directory.join(SHORT_TEXTS_FILE),
        first.directory.join("missing-text.copy"),
    )
    .unwrap();
    let repaired = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(repaired.action, "repaired");
    fs::write(repaired.directory.join(SHORT_TEXTS_FILE), b"partial").unwrap();
    let repaired2 = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(repaired2.action, "repaired");
    assert_eq!(
        fs::read(repaired.directory.join(SHORT_TEXTS_FILE)).unwrap(),
        b"partial"
    );
    let mut machine = crate::make_machine(rom, None);
    let snapshot = machine.save_state().unwrap();
    attach_from_directory(&mut machine, &offline).unwrap();
    let owner = machine.bus.rom_resources().unwrap().clone();
    let original = parse_short_texts(&files[8]).unwrap();
    let mut forged = original.clone();
    forged.glyphs[0].width ^= 1;
    let mut manifest = source.manifest.clone();
    for case in 0..6 {
        manifest.clone_from(&source.manifest);
        let mut bytes = files[8].clone();
        match case {
            0 => {
                bytes = encode_short_texts(&forged).unwrap();
                manifest.resources[7].sha256 = gba_core::ffta_rom::digest(&bytes);
            }
            1 => manifest.resources[7].path = "../outside.json".into(),
            2 => {
                manifest.resources.pop();
                manifest.coverage.pop();
            }
            3 => manifest.resources[7].sources[0].offset += 4,
            4 => manifest.resources[7].dependencies.push(BG_DESCRIPTORS_ID.into()),
            _ => {
                bytes.truncate(bytes.len() - 1);
                manifest.resources[7].sha256 = gba_core::ffta_rom::digest(&bytes);
            }
        }
        fs::write(offline.join(SHORT_TEXTS_FILE), bytes).unwrap();
        fs::write(
            offline.join(MANIFEST_FILE),
            toml::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert!(attach_from_directory(&mut machine, &offline).is_err());
        assert!(Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
        assert_eq!(machine.save_state().unwrap(), snapshot);
    }
    manifest.clone_from(&source.manifest);
    let mut annotated = files[8].clone();
    annotated.extend_from_slice(b"\n \n");
    manifest.resources[7].sha256 = gba_core::ffta_rom::digest(&annotated);
    fs::write(offline.join(SHORT_TEXTS_FILE), annotated).unwrap();
    fs::write(
        offline.join(MANIFEST_FILE),
        toml::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
    attach_from_directory(&mut machine, &offline).unwrap();
    let current = machine.bus.rom_resources().unwrap().clone();
    assert_ne!(current.dataset_sha256(), owner.dataset_sha256());
    assert_eq!(current.transfer_file_sha256(), owner.transfer_file_sha256());
    assert_eq!(machine.save_state().unwrap(), snapshot);
    fs::File::options()
        .write(true)
        .open(offline.join(SHORT_TEXTS_FILE))
        .unwrap()
        .set_len(MAX_SHORT_TEXTS_FILE_BYTES + 1)
        .unwrap();
    assert!(attach_from_directory(&mut machine, &offline).is_err());
    assert!(Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &current));
    for (name, bytes) in names.iter().zip(&files) {
        fs::write(offline.join(name), bytes).unwrap();
    }
    let relative = repaired2
        .directory
        .strip_prefix(&managed)
        .unwrap()
        .to_path_buf();
    let prior_relative = retained.strip_prefix(&managed).unwrap().to_path_buf();
    let moved = root.join("moved resources");
    fs::rename(managed, &moved).unwrap();
    let relocated = preparation::prepare(&moved, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(relocated.action, "reused");
    assert_eq!(relocated.directory, moved.join(relative));
    for (name, bytes) in names.iter().zip(&files) {
        assert_eq!(fs::read(relocated.directory.join(name)).unwrap(), *bytes);
    }
    for (name, bytes) in names[..8].iter().zip(&old_files) {
        assert_eq!(
            fs::read(moved.join(&prior_relative).join(name)).unwrap(),
            *bytes
        );
        assert_eq!(fs::read(old.join(name)).unwrap(), *bytes);
    }
    let report = serde_json::json!({"region":"europe","files":9,"resources":8,"text_sources":670,"strings":535,"glyphs":65,
        "text_json_sha256":gba_core::ffta_rom::digest(&files[8]),"retained_e10_unchanged":true,"prior_resource_files_unchanged":true,
        "create_reuse_missing_corrupt_repair_move":true,"whole_set_identity":true,"activation_transactional":true,"coherent_edit_rejected":true,
        "fixed_paths_and_file_guards":true,"texts_runtime":"verified-only","linux":"pending","usa":"pending"});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("SHORT_TEXT_MANAGED EUR files=9 sources=670 retained_e10=true create/reuse/repair/move/identity/transactional_activation=pass");
}
