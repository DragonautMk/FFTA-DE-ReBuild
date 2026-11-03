//! Tenth EUR managed file; broad original text/archive proof remains closed.
use super::*;
use gba_core::rom_data::{text_catalogs, text_short, Resource, SourceRange, TEXT_CATALOGS_FORMAT};
use std::sync::Arc;

pub(super) fn fixture() -> Extraction {
    let mut source = super::text_short_managed_tests::fixture();
    let mut value = TextCatalogs {
        pointer_catalogs: Vec::new(),
        strings: Vec::new(),
        archive_catalogs: Vec::new(),
        glyphs: vec![text_short::Glyph {
            index: 1,
            width: 7,
            tile_words: vec![0x12345678; 32],
        }],
    };
    for category in 0..9 {
        let mut languages = Vec::new();
        for language in 0..5 {
            let id = value.strings.len() as u16;
            value.strings.push(text_short::Text {
                source_offset: text_catalogs::POINTER_STARTS[category][language],
                encoding: text_short::Encoding::Compact,
                units: vec![text_short::Unit::Glyph { index: 1 }],
            });
            languages.push(text_catalogs::PointerLanguage {
                source_offset: text_catalogs::POINTER_ROOTS[category][language],
                entries: vec![id; text_catalogs::POINTER_COUNTS[category]],
            });
        }
        value.pointer_catalogs.push(text_catalogs::PointerCatalog {
            reader: text_catalogs::POINTER_READERS[category],
            languages,
        });
    }
    for category in 0..29 {
        let languages = (0..5)
            .map(|language| {
                let root = text_catalogs::ARCHIVE_ROOTS[category][language];
                text_catalogs::Archive {
                    source_offset: root,
                    offsets: vec![4, 65535],
                    programs: vec![text_catalogs::Program {
                        source_offset: root + 4,
                        header: 0,
                        units: vec![text_catalogs::ProgramUnit::Glyph { index: 1 }],
                        decoded_padding: Vec::new(),
                        encoded: None,
                    }],
                }
            })
            .collect();
        value.archive_catalogs.push(text_catalogs::ArchiveCatalog {
            reader: text_catalogs::ARCHIVE_READERS[category],
            languages,
        });
    }
    value.validate().unwrap();
    let mut ranges = Vec::new();
    let mut add = |offset, bytes: Vec<u8>, role| {
        ranges.push(SourceRange {
            offset,
            length: bytes.len() as u32,
            sha256: gba_core::ffta_rom::digest(&bytes),
            role,
        })
    };
    for (id, c) in value.pointer_catalogs.iter().enumerate() {
        for (language, t) in c.languages.iter().enumerate() {
            add(
                t.source_offset,
                value.table_bytes(t).unwrap(),
                format!("text-pointer-table/{id}/{language}"),
            );
        }
    }
    for (id, t) in value.strings.iter().enumerate() {
        add(
            t.source_offset,
            t.source_bytes().unwrap(),
            format!("text-pointer-string/{id}"),
        );
    }
    for (id, c) in value.archive_catalogs.iter().enumerate() {
        for (language, a) in c.languages.iter().enumerate() {
            add(
                a.source_offset,
                a.offsets.iter().flat_map(|v| v.to_le_bytes()).collect(),
                format!("text-archive-index/{id}/{language}"),
            );
            for (index, p) in a.programs.iter().enumerate() {
                add(
                    p.source_offset,
                    p.source_bytes().unwrap(),
                    format!("text-archive-program/{id}/{language}/{index}"),
                );
            }
        }
    }
    for g in &value.glyphs {
        add(
            g.source_offset(),
            g.source_bytes(),
            format!("text-font-glyph/{}", g.index),
        );
        add(
            g.width_offset(),
            vec![g.width],
            format!("text-font-width/{}", g.index),
        );
    }
    source.text_catalogs = Some(text_catalogs::Catalog {
        decoded: value,
        sources: ranges.clone(),
    });
    source.manifest.coverage.push(TEXT_CATALOGS_ID.into());
    source.manifest.resources.push(Resource {
        id: TEXT_CATALOGS_ID.into(),
        path: TEXT_CATALOGS_FILE.into(),
        format: TEXT_CATALOGS_FORMAT.into(),
        sha256: String::new(),
        dependencies: vec![SHORT_TEXTS_ID.into()],
        sources: ranges,
    });
    source
}

#[test]
fn tenth_text_catalog_file_publication_and_no_write_reuse_are_deterministic() {
    let root = tempfile::tempdir().unwrap();
    let mut source = fixture();
    let a = root.path().join("first");
    let b = root.path().join("second");
    assert_eq!(extract(&a, &mut source).unwrap(), "created");
    assert_eq!(extract(&b, &mut source).unwrap(), "created");
    assert_eq!(fs::read_dir(&a).unwrap().count(), 10);
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
#[ignore = "requires exact EUR ROM, retained e11 and broad typed02 files, absent FFTA_STATE_TEST_TEXT_CATALOG_MANAGED_ROOT"]
fn european_text_catalog_dataset_authenticates_repairs_moves_and_retains_e11() {
    let root =
        PathBuf::from(std::env::var_os("FFTA_STATE_TEST_TEXT_CATALOG_MANAGED_ROOT").unwrap());
    assert!(!root.exists());
    let old = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_OLD_E11_DATASET").unwrap());
    let typed = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_TEXT_CATALOG_TYPED_JSON").unwrap());
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
        TEXT_CATALOGS_FILE,
    ];
    let old_files: Vec<_> = names[..9]
        .iter()
        .map(|n| fs::read(old.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old_files[0], "prior e11 manifest").unwrap();
    assert_eq!(
        (
            prior.schema,
            prior.extractor_revision,
            prior.decoder_revision
        ),
        (1, 11, 10)
    );
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.schema,
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (1, 12, 11)
    );
    assert_eq!(source.manifest.resources.len(), 9);
    assert_eq!(source.manifest.resources[8].sources.len(), 33581);
    let offline = root.join("offline dataset");
    extract(&offline, &mut source).unwrap();
    assert_eq!(fs::read_dir(&offline).unwrap().count(), 10);
    let files: Vec<_> = names
        .iter()
        .map(|n| fs::read(offline.join(n)).unwrap())
        .collect();
    assert_eq!(files[9], fs::read(typed).unwrap());
    for i in 1..9 {
        assert_eq!(files[i], old_files[i]);
    }
    let managed = root.join("resources");
    let retained = managed
        .join(&prior.source_rom_sha256)
        .join("v1/e11-d10/generation-retained");
    fs::create_dir_all(&retained).unwrap();
    for (name, bytes) in names[..9].iter().zip(&old_files) {
        fs::write(retained.join(name), bytes).unwrap();
    }
    fs::write(
        retained.parent().unwrap().join("current"),
        b"generation-retained\n",
    )
    .unwrap();
    let first = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(first.action, "created");
    assert!(first.directory.parent().unwrap().ends_with("v1/e12-d11"));
    let reused = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(reused.action, "reused");
    assert_eq!(reused.directory, first.directory);
    fs::rename(
        first.directory.join(TEXT_CATALOGS_FILE),
        first.directory.join("missing-text.copy"),
    )
    .unwrap();
    let repaired = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(repaired.action, "repaired");
    fs::write(repaired.directory.join(TEXT_CATALOGS_FILE), b"partial").unwrap();
    let repaired2 = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(repaired2.action, "repaired");
    assert_eq!(
        fs::read(repaired.directory.join(TEXT_CATALOGS_FILE)).unwrap(),
        b"partial"
    );
    let mut machine = crate::make_machine(rom, None);
    let snapshot = machine.save_state().unwrap();
    attach_from_directory(&mut machine, &offline).unwrap();
    let owner = machine.bus.rom_resources().unwrap().clone();
    let original = parse_text_catalogs(&files[9]).unwrap();
    let mut forged = original.clone();
    forged.glyphs[0].width ^= 1;
    let mut manifest = source.manifest.clone();
    for case in 0..6 {
        manifest.clone_from(&source.manifest);
        let mut bytes = files[9].clone();
        match case {
            0 => {
                bytes = encode_text_catalogs(&forged).unwrap();
                manifest.resources[8].sha256 = gba_core::ffta_rom::digest(&bytes);
            }
            1 => manifest.resources[8].path = "../outside.json".into(),
            2 => {
                manifest.resources.pop();
                manifest.coverage.pop();
            }
            3 => manifest.resources[8].sources[0].offset += 4,
            4 => manifest.resources[8].dependencies.clear(),
            _ => {
                bytes.truncate(bytes.len() - 1);
                manifest.resources[8].sha256 = gba_core::ffta_rom::digest(&bytes);
            }
        }
        fs::write(offline.join(TEXT_CATALOGS_FILE), bytes).unwrap();
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
    let mut annotated = files[9].clone();
    annotated.extend_from_slice(b"\n \n");
    manifest.resources[8].sha256 = gba_core::ffta_rom::digest(&annotated);
    fs::write(offline.join(TEXT_CATALOGS_FILE), annotated).unwrap();
    fs::write(
        offline.join(MANIFEST_FILE),
        toml::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
    attach_from_directory(&mut machine, &offline).unwrap();
    let current = machine.bus.rom_resources().unwrap().clone();
    assert_ne!(current.dataset_sha256(), owner.dataset_sha256());
    assert_eq!(current.text_file_sha256(), owner.text_file_sha256());
    assert_eq!(machine.save_state().unwrap(), snapshot);
    fs::File::options()
        .write(true)
        .open(offline.join(TEXT_CATALOGS_FILE))
        .unwrap()
        .set_len(MAX_TEXT_CATALOGS_FILE_BYTES + 1)
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
    for (name, bytes) in names[..9].iter().zip(&old_files) {
        assert_eq!(
            fs::read(moved.join(&prior_relative).join(name)).unwrap(),
            *bytes
        );
        assert_eq!(fs::read(old.join(name)).unwrap(), *bytes);
    }
    let report = serde_json::json!({"region":"europe","files":10,"resources":9,"text_sources":33581,"strings":16100,"archive_programs":17035,"glyphs":128,
        "text_catalog_json_sha256":gba_core::ffta_rom::digest(&files[9]),"retained_e11_unchanged":true,"prior_resource_files_unchanged":true,
        "create_reuse_missing_corrupt_repair_move":true,"whole_set_identity":true,"activation_transactional":true,"coherent_edit_rejected":true,
        "fixed_paths_and_file_guards":true,"text_catalogs_runtime":"verified-only","linux":"pending","usa":"pending"});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("TEXT_CATALOG_MANAGED EUR files=10 sources=33581 retained_e11=true create/reuse/repair/move/identity/transactional_activation=pass");
}
