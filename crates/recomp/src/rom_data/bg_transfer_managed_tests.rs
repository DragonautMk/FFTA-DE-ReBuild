//! Eighth EUR managed file, independent of the completed original animation proof.
use super::*;
use gba_core::rom_data::{bg_transfers, Resource, SourceRange, BG_TRANSFER_PROGRAMS_FORMAT};
use std::sync::Arc;

pub(super) fn fixture() -> Extraction {
    let mut source = super::bg_graphics_managed_tests::fixture();
    let control = bg_transfers::Control::decode(&[32, 0, 2, 0, 8, 0, 0, 0, 16, 0, 1, 0], 0x5000)
        .unwrap()
        .0;
    let tiles = bg_transfers::Tiles {
        relative_source: 0x6000,
        words: vec![0x81234567; 16],
    };
    let ranges = [
        (
            "control",
            control.relative_source,
            control.source_bytes().unwrap(),
        ),
        ("tiles", tiles.relative_source, tiles.source_bytes()),
    ]
    .into_iter()
    .map(|(kind, relative, bytes)| SourceRange {
        offset: 0x100 + relative,
        length: bytes.len() as u32,
        sha256: gba_core::ffta_rom::digest(&bytes),
        role: format!("bg-transfer-{kind}/0"),
    })
    .collect::<Vec<_>>();
    source.transfers = Some(bg_transfers::Catalog {
        sources: ranges.clone(),
        decoded: Transfers {
            descriptor_links: vec![
                [
                    Some(bg_transfers::Link {
                        control: 0,
                        tiles: 0,
                        destination: 0x06000020
                    }),
                    None,
                    None,
                    None,
                    None
                ];
                162
            ],
            variant_sources: vec![None; 162],
            controls: vec![control],
            tiles: vec![tiles],
        },
    });
    source
        .manifest
        .coverage
        .push(BG_TRANSFER_PROGRAMS_ID.into());
    source.manifest.resources.push(Resource {
        id: BG_TRANSFER_PROGRAMS_ID.into(),
        path: BG_TRANSFER_PROGRAMS_FILE.into(),
        format: BG_TRANSFER_PROGRAMS_FORMAT.into(),
        sha256: String::new(),
        dependencies: vec![BG_DESCRIPTORS_ID.into()],
        sources: ranges,
    });
    source
}

#[test]
fn eighth_transfer_file_publication_and_no_write_reuse_are_deterministic() {
    let root = tempfile::tempdir().unwrap();
    let mut source = fixture();
    let a = root.path().join("first");
    let b = root.path().join("second");
    assert_eq!(extract(&a, &mut source).unwrap(), "created");
    assert_eq!(extract(&b, &mut source).unwrap(), "created");
    assert_eq!(fs::read_dir(&a).unwrap().count(), 8);
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
#[ignore = "requires exact EUR ROM, retained e9 and typed02 files, absent FFTA_STATE_TEST_TRANSFER_MANAGED_ROOT"]
fn european_transfer_dataset_authenticates_repairs_moves_and_retains_e9() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_TRANSFER_MANAGED_ROOT").unwrap());
    assert!(!root.exists());
    let old = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_OLD_E9_DATASET").unwrap());
    let typed = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_TRANSFER_TYPED_JSON").unwrap());
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
    ];
    let old_files: Vec<_> = names[..7]
        .iter()
        .map(|n| fs::read(old.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old_files[0], "prior e9 manifest").unwrap();
    assert_eq!(
        (
            prior.schema,
            prior.extractor_revision,
            prior.decoder_revision
        ),
        (1, 9, 8)
    );
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.schema,
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (1, 10, 9)
    );
    assert_eq!(source.manifest.resources.len(), 7);
    assert_eq!(source.manifest.resources[6].sources.len(), 76);
    let offline = root.join("offline dataset");
    extract(&offline, &mut source).unwrap();
    assert_eq!(fs::read_dir(&offline).unwrap().count(), 8);
    let files: Vec<_> = names
        .iter()
        .map(|n| fs::read(offline.join(n)).unwrap())
        .collect();
    assert_eq!(files[7], fs::read(typed).unwrap());
    for i in 1..7 {
        assert_eq!(files[i], old_files[i]);
    }
    let managed = root.join("resources");
    let retained = managed
        .join(&prior.source_rom_sha256)
        .join("v1/e9-d8/generation-retained");
    fs::create_dir_all(&retained).unwrap();
    for (name, bytes) in names[..7].iter().zip(&old_files) {
        fs::write(retained.join(name), bytes).unwrap();
    }
    fs::write(
        retained.parent().unwrap().join("current"),
        b"generation-retained\n",
    )
    .unwrap();
    let first = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(first.action, "created");
    assert!(first.directory.parent().unwrap().ends_with("v1/e10-d9"));
    let reused = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(reused.action, "reused");
    assert_eq!(reused.directory, first.directory);
    fs::rename(
        first.directory.join(BG_TRANSFER_PROGRAMS_FILE),
        first.directory.join("missing-transfer.copy"),
    )
    .unwrap();
    let repaired = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(repaired.action, "repaired");
    fs::write(
        repaired.directory.join(BG_TRANSFER_PROGRAMS_FILE),
        b"partial",
    )
    .unwrap();
    let repaired2 = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(repaired2.action, "repaired");
    assert_eq!(
        fs::read(repaired.directory.join(BG_TRANSFER_PROGRAMS_FILE)).unwrap(),
        b"partial"
    );
    let mut machine = crate::make_machine(rom, None);
    let snapshot = machine.save_state().unwrap();
    attach_from_directory(&mut machine, &offline).unwrap();
    let owner = machine.bus.rom_resources().unwrap().clone();
    let original = parse_transfers(&files[7]).unwrap();
    let mut forged = original.clone();
    forged.tiles[0].words[0] ^= 0x80000000;
    let mut manifest = source.manifest.clone();
    for case in 0..6 {
        manifest.clone_from(&source.manifest);
        let mut bytes = files[7].clone();
        match case {
            0 => {
                bytes = encode_transfers(&forged).unwrap();
                manifest.resources[6].sha256 = gba_core::ffta_rom::digest(&bytes);
            }
            1 => manifest.resources[6].path = "../outside.json".into(),
            2 => {
                manifest.resources.pop();
                manifest.coverage.pop();
            }
            3 => manifest.resources[6].sources[0].offset += 4,
            4 => manifest.resources[6].dependencies.clear(),
            _ => {
                bytes.truncate(bytes.len() - 1);
                manifest.resources[6].sha256 = gba_core::ffta_rom::digest(&bytes);
            }
        }
        fs::write(offline.join(BG_TRANSFER_PROGRAMS_FILE), bytes).unwrap();
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
    let mut annotated = files[7].clone();
    annotated.extend_from_slice(b"\n \n");
    manifest.resources[6].sha256 = gba_core::ffta_rom::digest(&annotated);
    fs::write(offline.join(BG_TRANSFER_PROGRAMS_FILE), annotated).unwrap();
    fs::write(
        offline.join(MANIFEST_FILE),
        toml::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
    attach_from_directory(&mut machine, &offline).unwrap();
    let current = machine.bus.rom_resources().unwrap().clone();
    assert_ne!(current.dataset_sha256(), owner.dataset_sha256());
    assert_eq!(current.graphics_file_sha256(), owner.graphics_file_sha256());
    assert_eq!(machine.save_state().unwrap(), snapshot);
    fs::File::options()
        .write(true)
        .open(offline.join(BG_TRANSFER_PROGRAMS_FILE))
        .unwrap()
        .set_len(MAX_BG_TRANSFER_PROGRAMS_FILE_BYTES + 1)
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
    for (name, bytes) in names[..7].iter().zip(&old_files) {
        assert_eq!(
            fs::read(moved.join(&prior_relative).join(name)).unwrap(),
            *bytes
        );
        assert_eq!(fs::read(old.join(name)).unwrap(), *bytes);
    }
    let report = serde_json::json!({"region":"europe","files":8,"resources":7,"transfer_sources":76,"controls":42,"tile_banks":34,
        "transfer_json_sha256":gba_core::ffta_rom::digest(&files[7]),"retained_e9_unchanged":true,"prior_resource_files_unchanged":true,
        "create_reuse_missing_corrupt_repair_move":true,"whole_set_identity":true,"activation_transactional":true,"coherent_edit_rejected":true,
        "fixed_paths_and_file_guards":true,"transfers_runtime":"verified-only","linux":"pending","usa":"pending"});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("BG_TRANSFER_MANAGED EUR files=8 sources=76 retained_e9=true create/reuse/repair/move/identity/transactional_activation=pass");
}
