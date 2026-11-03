use super::*;

fn manifest(rom: &[u8]) -> Vec<u8> {
    let m = gba_core::rom_data::Manifest {
        schema: 1,
        extractor_revision: 1,
        decoder_revision: 1,
        source_rom_sha256: hash(rom),
        source_rom_size: rom.len() as u32,
        profile: "synthetic".into(),
        coverage: vec![],
        resources: vec![gba_core::rom_data::Resource {
            id: "fixture".into(),
            path: "unused".into(),
            format: "test".into(),
            sha256: hash(&[]),
            dependencies: vec![],
            sources: vec![gba_core::rom_data::SourceRange {
                offset: 4,
                length: 8,
                sha256: hash(&rom[4..12]),
                role: "fixture".into(),
            }],
        }],
    };
    toml::to_string(&m).unwrap().into_bytes()
}
#[test]
fn resource_ranges_require_authentic_identity_contents_bounds_and_id() {
    let rom = vec![0; 32];
    let bytes = manifest(&rom);
    assert_eq!(
        source_ranges(&bytes, &rom, "fixture").unwrap(),
        vec![(4, 12)]
    );
    assert!(source_ranges(&bytes, &rom, "absent").is_err());
    assert!(source_ranges(&bytes, &vec![1; 32], "fixture").is_err());
    let text = String::from_utf8(bytes).unwrap();
    for edit in [
        text.replace("length = 8", "length = 0"),
        text.replace("length = 8", "length = 100"),
        text.replace(&hash(&rom[4..12]), &hash(&[1; 8])),
    ] {
        assert!(source_ranges(edit.as_bytes(), &rom, "fixture").is_err());
    }
}
#[test]
fn instruction_verification_rejects_changed_bytes_alignment_mode_and_frame_limits() {
    let rom = 0xeafffffeu32.to_le_bytes();
    assert!(instruction(&rom, 0x08000000, false, 0xeafffffe).is_ok());
    assert!(instruction(&rom, 0x08000000, false, 0xeafffffd).is_err());
    assert!(instruction(&rom, 0x08000004, false, 0).is_err());
    for condition in [
        Condition::Instruction {
            pc: 0x08000001,
            thumb: true,
            opcode: 0,
        },
        Condition::Instruction {
            pc: 0x08000002,
            thumb: false,
            opcode: 0,
        },
        Condition::Instruction {
            pc: 0x08000000,
            thumb: true,
            opcode: 0x10000,
        },
        Condition::Frame { completed: 0 },
        Condition::Frame { completed: 5 },
    ] {
        assert!(validate(
            &Settings {
                condition,
                limit_frames: 4
            },
            4
        )
        .is_err());
    }
    assert!(validate(
        &Settings {
            condition: Condition::Frame { completed: 1 },
            limit_frames: 5
        },
        4
    )
    .is_err());
    assert!(validate(
        &Settings {
            condition: Condition::Frame { completed: 1 },
            limit_frames: 0
        },
        4
    )
    .is_err());
}
#[cfg(feature = "rom-data-read-audit")]
#[test]
fn changed_manifest_is_refused_before_observation() {
    let t = tempfile::tempdir().unwrap();
    let rom = vec![0; 32];
    let bytes = manifest(&rom);
    fs::write(t.path().join("capture-plan.toml"), b"changed").unwrap();
    let s = Settings {
        condition: Condition::Resource {
            id: "fixture".into(),
            preset: "all".into(),
            manifest_sha256: hash(&bytes),
        },
        limit_frames: 4,
    };
    assert!(Observer::start(t.path(), &s, &rom).is_err());
}
#[test]
fn retained_bundles_are_bounded_without_deleting_any_existing_point() {
    let t = tempfile::tempdir().unwrap();
    assert!(capacity(t.path()).is_ok());
    let dir = t.path().join("captures");
    fs::create_dir(&dir).unwrap();
    for n in 0..8 {
        fs::create_dir(dir.join(format!("accepted-{n}"))).unwrap();
    }
    assert!(capacity(t.path()).unwrap_err().contains("Eight"));
    assert_eq!(fs::read_dir(dir).unwrap().count(), 8);
}
#[test]
fn cli_rejects_ambiguous_or_unverified_trigger_options_before_copying() {
    for args in [
        vec!["--frame", "1", "--pc", "08000000"],
        vec!["--pc", "08000000", "--mode", "thumb"],
        vec!["--pc", "08000000", "--mode", "other", "--opcode", "0"],
        vec!["--frame", "1", "--manifest", "anything"],
        vec!["--frame", "1", "--opcode", "0"],
        vec!["--frame"],
    ] {
        assert!(create_cmd(&args.into_iter().map(String::from).collect::<Vec<_>>()).is_err());
    }
}
