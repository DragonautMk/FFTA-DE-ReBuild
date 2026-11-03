//! Managed alternate files, source authentication and previous revision retention.
use super::*;
use std::sync::Arc;

#[test]
#[ignore = "requires exact regional ROMs, existing e6 datasets and absent FFTA_STATE_TEST_ALT_PALETTE_DATA_ROOT"]
fn alternate_files_authenticate_whole_set_and_retain_previous_generations() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_ALT_PALETTE_DATA_ROOT").unwrap());
    assert!(!root.exists(), "preserve prior managed observations");
    let old_root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_OLD_DATASETS_ROOT").unwrap());
    let mut reports = Vec::new();
    let names = [
        MANIFEST_FILE,
        OBJ_DIMENSIONS_FILE,
        BG_DESCRIPTORS_FILE,
        BG_BASE_PALETTES_FILE,
        BG_ALTERNATE_PALETTES_FILE,
        BG_PACKET_PROGRAMS_FILE,
    ];
    for (env, profile) in [
        ("FFTA_TEST_ROM_USA", "usa"),
        ("FFTA_TEST_ROM_EUR", "europe"),
    ] {
        let rom = fs::read(std::env::var_os(env).unwrap()).unwrap();
        let mut source = Extraction::from_rom(&rom).unwrap();
        let base = root.join(profile);
        let offline = base.join("offline");
        extract(&offline, &mut source).unwrap();
        assert_eq!(fs::read_dir(&offline).unwrap().count(), 6);
        let originals: Vec<_> = names
            .iter()
            .map(|name| fs::read(offline.join(name)).unwrap())
            .collect();
        let parsed = parse_alternate_palettes(&originals[4]).unwrap();
        assert_eq!(
            &parsed,
            &source.alternate_palettes.as_ref().unwrap().decoded
        );
        assert_eq!(encode_alternate_palettes(&parsed).unwrap(), originals[4]);
        assert_eq!(source.manifest.resources[3].sources.len(), 316);
        assert_eq!(
            &source.manifest.coverage[3],
            BG_ALTERNATE_PALETTES_ID
        );
        let mut machine = crate::make_machine(rom, None);
        let snapshot = machine.save_state().unwrap();
        attach_from_directory(&mut machine, &offline).unwrap();
        let owner = machine.bus.rom_resources().unwrap().clone();
        let mut forged = parsed.clone();
        // Preserve the consumed extent so this coherent content edit reaches
        // source authentication instead of failing the overlap guard first.
        let original_palette = &parsed.catalogs[0][0];
        let changed_palette = (4..original_palette.encoded_source.len())
            .find_map(|index| {
                let mut encoded = original_palette.encoded_source.clone();
                encoded[index] ^= 0x80;
                let (candidate, consumed) =
                    gba_core::rom_data::bg_alternate_palettes::Palette::decode(&encoded).ok()?;
                (consumed == encoded.len() && candidate.banks != original_palette.banks)
                    .then_some(candidate)
            })
            .expect("at least one valid same-extent content edit");
        forged.catalogs[0][0] = changed_palette;
        let forged = encode_alternate_palettes(&forged).unwrap();
        let mut manifest = source.manifest.clone();
        manifest.resources[3].sha256 = gba_core::ffta_rom::digest(&forged);
        fs::write(offline.join(BG_ALTERNATE_PALETTES_FILE), forged).unwrap();
        fs::write(
            offline.join(MANIFEST_FILE),
            toml::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert!(attach_from_directory(&mut machine, &offline).is_err());
        assert!(Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
        assert_eq!(machine.save_state().unwrap(), snapshot);
        // Even a source re-encoding with identical banks is not source-equal.
        let mut recoded = parsed;
        recoded.catalogs[0][0].encoded_source.insert(4, 3);
        let recoded = encode_alternate_palettes(&recoded).unwrap();
        manifest.resources[3].sha256 = gba_core::ffta_rom::digest(&recoded);
        fs::write(offline.join(BG_ALTERNATE_PALETTES_FILE), recoded).unwrap();
        fs::write(
            offline.join(MANIFEST_FILE),
            toml::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert!(attach_from_directory(&mut machine, &offline).is_err());
        assert!(Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
        assert_eq!(machine.save_state().unwrap(), snapshot);
        let mut annotated = originals[4].clone();
        annotated.extend_from_slice(b"\n \n");
        manifest.resources[3].sha256 = gba_core::ffta_rom::digest(&annotated);
        fs::write(offline.join(BG_ALTERNATE_PALETTES_FILE), annotated).unwrap();
        fs::write(
            offline.join(MANIFEST_FILE),
            toml::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        attach_from_directory(&mut machine, &offline).unwrap();
        let current = machine.bus.rom_resources().unwrap().clone();
        assert_ne!(current.dataset_sha256(), owner.dataset_sha256());
        assert_eq!(current.file_sha256(), owner.file_sha256());
        assert_eq!(
            current.background_file_sha256(),
            owner.background_file_sha256()
        );
        assert_eq!(current.palette_file_sha256(), owner.palette_file_sha256());
        assert_eq!(machine.save_state().unwrap(), snapshot);
        fs::remove_file(offline.join(BG_ALTERNATE_PALETTES_FILE)).unwrap();
        assert!(attach_from_directory(&mut machine, &offline).is_err());
        assert!(Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &current));
        for (name, bytes) in names.iter().zip(&originals) {
            fs::write(offline.join(name), bytes).unwrap();
        }

        let old = old_root.join(profile).join("offline");
        let old_bytes: Vec<_> = names[..4]
            .iter()
            .map(|name| fs::read(old.join(name)).unwrap())
            .collect();
        let prior: Manifest = parse(&old_bytes[0], "previous manifest").unwrap();
        assert_eq!(
            (
                prior.schema,
                prior.extractor_revision,
                prior.decoder_revision
            ),
            (1, 6, 5)
        );
        let managed = base.join("resources");
        let retained = managed
            .join(&prior.source_rom_sha256)
            .join("v1/e6-d5/generation-retained");
        fs::create_dir_all(&retained).unwrap();
        for (name, bytes) in names[..4].iter().zip(&old_bytes) {
            fs::write(retained.join(name), bytes).unwrap();
        }
        fs::write(
            retained.parent().unwrap().join("current"),
            b"generation-retained\n",
        )
        .unwrap();
        assert!(verify(&retained, &source).is_err());
        let first = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
        assert_eq!(first.action, "created");
        assert!(first.directory.parent().unwrap().ends_with("v1/e8-d7"));
        let reused = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
        assert_eq!(reused.action, "reused");
        assert_eq!(reused.directory, first.directory);
        fs::write(
            first.directory.join(BG_ALTERNATE_PALETTES_FILE),
            b"broken alternate",
        )
        .unwrap();
        let repaired = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
        assert_eq!(repaired.action, "repaired");
        assert_eq!(
            fs::read(first.directory.join(BG_ALTERNATE_PALETTES_FILE)).unwrap(),
            b"broken alternate"
        );
        fs::remove_file(repaired.directory.join(BG_ALTERNATE_PALETTES_FILE)).unwrap();
        let ready = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
        assert_eq!(ready.action, "repaired");
        let relative = ready
            .directory
            .strip_prefix(&managed)
            .unwrap()
            .to_path_buf();
        let retained_relative = retained.strip_prefix(&managed).unwrap().to_path_buf();
        let moved = base.join("moved resources");
        fs::rename(&managed, &moved).unwrap();
        let relocated = preparation::prepare(&moved, &mut source, &mut |_| Ok(())).unwrap();
        assert_eq!(relocated.action, "reused");
        assert_eq!(relocated.directory, moved.join(relative));
        for (name, bytes) in names.iter().zip(&originals) {
            assert_eq!(fs::read(relocated.directory.join(name)).unwrap(), *bytes);
        }
        for (name, bytes) in names[..4].iter().zip(&old_bytes) {
            assert_eq!(
                fs::read(moved.join(&retained_relative).join(name)).unwrap(),
                *bytes
            );
            assert_eq!(fs::read(old.join(name)).unwrap(), *bytes);
        }
        assert_eq!(
            fs::read(
                moved
                    .join(retained_relative)
                    .parent()
                    .unwrap()
                    .join("current")
            )
            .unwrap(),
            b"generation-retained\n"
        );
        reports.push(serde_json::json!({"profile":profile,"files":6,"sources":316,"alternate_sha256":gba_core::ffta_rom::digest(&originals[4]),"e6_retained":true,"create_reuse_repair_move":true,"transactional_activation":true,"whole_set_identity":true,"alternate_runtime_consumption":false}));
    }
    assert_eq!(
        reports[0]["alternate_sha256"],
        reports[1]["alternate_sha256"]
    );
    fs::write(
        root.join("results.json"),
        serde_json::to_vec_pretty(&reports).unwrap(),
    )
    .unwrap();
}
