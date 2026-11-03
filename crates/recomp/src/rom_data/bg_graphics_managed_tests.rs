//! Managed graphics acceptance only; reuse the completed EUR decoder oracle.
use super::*;
use gba_core::rom_data::{bg_graphics, Resource, SourceRange, BG_MAIN_GRAPHICS_FORMAT};
use std::sync::Arc;

pub(super) fn fixture() -> Extraction {
    let mut source = super::tests::fixture();
    let mut encoded = (32u32 << 8 | 0x20).to_le_bytes().to_vec();
    encoded.extend_from_slice(&32u32.to_be_bytes());
    encoded.extend_from_slice(&[1, 29]);
    let graphic = bg_graphics::Graphic::decode(&encoded, 0x4000).unwrap().0;
    let range = SourceRange {
        offset: 0x4100,
        length: encoded.len() as u32,
        sha256: gba_core::ffta_rom::digest(&encoded),
        role: "bg-main-graphics/0".into(),
    };
    source.main_graphics = Some(bg_graphics::Catalog {
        decoded: Graphics {
            descriptor_graphics: vec![0; 162],
            graphics: vec![graphic],
        },
        sources: vec![range.clone()],
    });
    source.manifest.coverage.push(BG_MAIN_GRAPHICS_ID.into());
    source.manifest.resources.push(Resource {
        id: BG_MAIN_GRAPHICS_ID.into(),
        path: BG_MAIN_GRAPHICS_FILE.into(),
        format: BG_MAIN_GRAPHICS_FORMAT.into(),
        sha256: String::new(),
        dependencies: vec![BG_DESCRIPTORS_ID.into()],
        sources: vec![range],
    });
    source.manifest.source_rom_sha256 = "0".repeat(64);
    source
}

#[test]
fn graphics_seventh_file_publication_and_no_write_reuse_are_deterministic() {
    let root = tempfile::tempdir().unwrap();
    let mut source = fixture();
    let first = root.path().join("first");
    let second = root.path().join("second");
    assert_eq!(extract(&first, &mut source).unwrap(), "created");
    assert_eq!(extract(&second, &mut source).unwrap(), "created");
    assert_eq!(fs::read_dir(&first).unwrap().count(), 7);
    for item in fs::read_dir(&first).unwrap() {
        let name = item.unwrap().file_name();
        assert_eq!(
            fs::read(first.join(&name)).unwrap(),
            fs::read(second.join(name)).unwrap()
        );
    }
    let prepared = preparation::prepare(root.path(), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    let reused = preparation::prepare(root.path(), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(reused.action, "reused");
    assert_eq!(reused.directory, prepared.directory);
    verify(&first, &source).unwrap();
}

#[test]
#[ignore = "requires EUR ROM, retained e8 dataset/typed graphics JSON and absent FFTA_STATE_TEST_GRAPHICS_MANAGED_ROOT"]
fn european_graphics_dataset_authenticates_repairs_relocates_and_retains_e8() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_GRAPHICS_MANAGED_ROOT").unwrap());
    assert!(!root.exists(), "preserve previous managed evidence");
    let old = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_OLD_E8_DATASET").unwrap());
    let typed = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_GRAPHICS_TYPED_JSON").unwrap());
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
    ];
    let old_files: Vec<_> = names[..6]
        .iter()
        .map(|n| fs::read(old.join(n)).unwrap())
        .collect();
    let prior: Manifest = parse(&old_files[0], "e8 manifest").unwrap();
    assert_eq!(
        (
            prior.schema,
            prior.extractor_revision,
            prior.decoder_revision
        ),
        (1, 8, 7)
    );
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.schema,
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (1, 9, 8)
    );
    assert_eq!(source.manifest.resources.len(), 6);
    assert_eq!(source.manifest.resources[5].sources.len(), 50);
    let offline = root.join("offline dataset");
    extract(&offline, &mut source).unwrap();
    assert_eq!(fs::read_dir(&offline).unwrap().count(), 7);
    let files: Vec<_> = names
        .iter()
        .map(|n| fs::read(offline.join(n)).unwrap())
        .collect();
    assert_eq!(files[6], fs::read(typed).unwrap());
    for index in 1..6 {
        assert_eq!(files[index], old_files[index]);
    }
    let managed = root.join("resources");
    let retained = managed
        .join(&prior.source_rom_sha256)
        .join("v1/e8-d7/generation-retained");
    fs::create_dir_all(&retained).unwrap();
    for (name, bytes) in names[..6].iter().zip(&old_files) {
        fs::write(retained.join(name), bytes).unwrap();
    }
    fs::write(
        retained.parent().unwrap().join("current"),
        b"generation-retained\n",
    )
    .unwrap();
    let first = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(first.action, "created");
    assert!(first.directory.parent().unwrap().ends_with("v1/e9-d8"));
    let reused = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(reused.action, "reused");
    assert_eq!(reused.directory, first.directory);
    fs::rename(
        first.directory.join(BG_MAIN_GRAPHICS_FILE),
        first.directory.join("missing-graphics.copy"),
    )
    .unwrap();
    let repaired = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(repaired.action, "repaired");
    assert_ne!(repaired.directory, first.directory);
    fs::write(repaired.directory.join(BG_MAIN_GRAPHICS_FILE), b"partial").unwrap();
    let repaired2 = preparation::prepare(&managed, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(repaired2.action, "repaired");
    assert_ne!(repaired2.directory, repaired.directory);
    assert_eq!(
        fs::read(repaired.directory.join(BG_MAIN_GRAPHICS_FILE)).unwrap(),
        b"partial"
    );
    let mut machine = crate::make_machine(rom, None);
    let state = machine.save_state().unwrap();
    attach_from_directory(&mut machine, &offline).unwrap();
    let owner = machine.bus.rom_resources().unwrap().clone();
    let graphics = parse_main_graphics(&files[6]).unwrap();
    let mut forged = graphics.clone();
    let original = &graphics.graphics[0];
    forged.graphics[0] = (8..original.encoded_source.len())
        .find_map(|i| {
            let mut source = original.encoded_source.clone();
            source[i] ^= 0x80;
            let (candidate, consumed) =
                bg_graphics::Graphic::decode(&source, original.relative_source).ok()?;
            (consumed == source.len() && candidate.words != original.words).then_some(candidate)
        })
        .unwrap();
    let mut manifest = source.manifest.clone();
    for invalid in 0..5 {
        let mut bytes = files[6].clone();
        manifest.clone_from(&source.manifest);
        match invalid {
            0 => {
                bytes = encode_main_graphics(&forged).unwrap();
                manifest.resources[5].sha256 = gba_core::ffta_rom::digest(&bytes);
            }
            1 => manifest.resources[5].path = "../outside.json".into(),
            2 => {
                manifest.resources.pop();
                manifest.coverage.pop();
            }
            3 => manifest.resources[5].sources[0].offset += 4,
            _ => {
                bytes.truncate(bytes.len() - 1);
                manifest.resources[5].sha256 = gba_core::ffta_rom::digest(&bytes);
            }
        }
        fs::write(offline.join(BG_MAIN_GRAPHICS_FILE), bytes).unwrap();
        fs::write(
            offline.join(MANIFEST_FILE),
            toml::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert!(attach_from_directory(&mut machine, &offline).is_err());
        assert!(Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
        assert_eq!(machine.save_state().unwrap(), state);
    }
    let mut annotated = files[6].clone();
    annotated.extend_from_slice(b"\n \n");
    manifest.clone_from(&source.manifest);
    manifest.resources[5].sha256 = gba_core::ffta_rom::digest(&annotated);
    fs::write(offline.join(BG_MAIN_GRAPHICS_FILE), annotated).unwrap();
    fs::write(
        offline.join(MANIFEST_FILE),
        toml::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
    attach_from_directory(&mut machine, &offline).unwrap();
    let current = machine.bus.rom_resources().unwrap().clone();
    assert_ne!(current.dataset_sha256(), owner.dataset_sha256());
    assert_eq!(current.packet_file_sha256(), owner.packet_file_sha256());
    assert_eq!(machine.save_state().unwrap(), state);
    fs::File::options()
        .write(true)
        .open(offline.join(BG_MAIN_GRAPHICS_FILE))
        .unwrap()
        .set_len(MAX_BG_MAIN_GRAPHICS_FILE_BYTES + 1)
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
    let retained_relative = retained.strip_prefix(&managed).unwrap().to_path_buf();
    let moved = root.join("moved resources");
    fs::rename(managed, &moved).unwrap();
    let relocated = preparation::prepare(&moved, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(relocated.action, "reused");
    assert_eq!(relocated.directory, moved.join(relative));
    for (name, bytes) in names.iter().zip(&files) {
        assert_eq!(fs::read(relocated.directory.join(name)).unwrap(), *bytes);
    }
    for (name, bytes) in names[..6].iter().zip(&old_files) {
        assert_eq!(
            fs::read(moved.join(&retained_relative).join(name)).unwrap(),
            *bytes
        );
        assert_eq!(fs::read(old.join(name)).unwrap(), *bytes);
    }
    let report = serde_json::json!({"region":"europe","files":7,"resources":6,"graphics_sources":50,
        "graphics_json_sha256":gba_core::ffta_rom::digest(&files[6]),"retained_e8_unchanged":true,
        "prior_resource_files_unchanged":true,"create_reuse_missing_corrupt_repair_move":true,
        "whole_set_identity":true,"activation_transactional":true,"coherent_edit_rejected":true,
        "fixed_paths_and_file_guards":true,"graphics_runtime":"verified-only","linux":"pending","usa":"pending"});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("BG_GRAPHICS_MANAGED EUR files=7 sources=50 retained_e8=true publication/repair/authentication/identity/move=pass");
}
