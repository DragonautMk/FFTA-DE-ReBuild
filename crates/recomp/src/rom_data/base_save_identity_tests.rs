//! EUR base-dataset compatibility with ROM-bound snapshots and cartridge saves.
use super::*;
use std::sync::Arc;

#[test]
#[ignore = "authenticated EUR ROM and E170 dataset; Windows executable under Wine"]
fn european_base_save_identity_is_bidirectionally_compatible() {
    let root = PathBuf::from(std::env::var_os("FFTA_SAVE_IDENTITY_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_SAVE_IDENTITY_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let manifest: Manifest =
        toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (
            manifest.extractor_revision,
            manifest.decoder_revision,
            manifest.resources.len()
        ),
        (170, 169, 113)
    );

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    assert!(raw.bus.rom_resources().is_none());
    let boot_raw = raw.save_state().unwrap();
    let boot_managed = managed.save_state().unwrap();
    assert_eq!(
        boot_raw, boot_managed,
        "base owner must not change snapshot bytes"
    );

    let save_len = raw.bus.save_data().unwrap().len();
    assert!(save_len > 0);
    let first: Vec<u8> = (0..save_len).map(|i| ((i * 37 + 11) & 255) as u8).collect();
    let second: Vec<u8> = (0..save_len).map(|i| ((i * 19 + 23) & 255) as u8).collect();
    for m in [&mut raw, &mut managed] {
        m.bus.load_save_data(&first);
        m.cpu.regs[4] = 0x1234_5678;
        m.bus.ewram[123] = 0x5a;
    }
    let raw_snapshot = raw.save_state().unwrap();
    let managed_snapshot = managed.save_state().unwrap();
    assert_eq!(
        raw_snapshot, managed_snapshot,
        "same state with and without base dataset"
    );
    raw.cpu.regs[4] = 1;
    managed.cpu.regs[4] = 2;
    raw.load_state(&managed_snapshot).unwrap();
    managed.load_state(&raw_snapshot).unwrap();
    assert_eq!(raw.save_state().unwrap(), managed_snapshot);
    assert_eq!(managed.save_state().unwrap(), raw_snapshot);
    assert_eq!(raw.bus.save_data().unwrap(), first);
    assert_eq!(managed.bus.save_data().unwrap(), first);
    assert!(raw.bus.rom_resources().is_none());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    // Invalid snapshot identity, version and payload are rejected before mutation.
    for m in [&mut raw, &mut managed] {
        let before = m.save_state().unwrap();
        for offset in [0usize, 8, 12, 44, 76, 111] {
            let mut invalid = raw_snapshot.clone();
            invalid[offset] ^= 1;
            assert!(
                m.load_state(&invalid).is_err(),
                "invalid snapshot byte {offset}"
            );
            assert_eq!(m.save_state().unwrap(), before);
        }
    }
    assert!(raw.bus.rom_resources().is_none());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    // A base dataset bound to the exact EUR ROM must not attach to a different ROM.
    let mut other_rom = rom.clone();
    other_rom[0x100] ^= 1;
    let mut other = crate::make_machine(other_rom, None);
    let other_before = other.save_state().unwrap();
    assert!(attach_from_directory(&mut other, &dataset).is_err());
    assert_eq!(other.save_state().unwrap(), other_before);
    assert!(other.bus.rom_resources().is_none());

    let raw_path = root.join("synthetic-raw.sav");
    let managed_path = root.join("synthetic-managed.sav");
    let sentinel = vec![0x7c; save_len];
    fs::write(&raw_path, &sentinel).unwrap();
    fs::write(&managed_path, &sentinel).unwrap();
    let mut invalid_boot = boot_raw.clone();
    invalid_boot[12] ^= 1;
    for (m, path) in [(&mut raw, &raw_path), (&mut managed, &managed_path)] {
        let before = m.save_state().unwrap();
        assert!(crate::save_backups::restart(m, &invalid_boot, path).is_err());
        assert_eq!(m.save_state().unwrap(), before);
        assert_eq!(fs::read(path).unwrap(), sentinel);
    }
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    // Write a synthetic cartridge save in one mode, consume it in the other,
    // then perform the reverse using the normal restart persistence path.
    crate::save_backups::restart(&mut raw, &boot_managed, &raw_path).unwrap();
    assert_eq!(fs::read(&raw_path).unwrap(), first);
    managed.bus.load_save_data(&fs::read(&raw_path).unwrap());
    assert_eq!(managed.bus.save_data().unwrap(), first);
    managed.bus.load_save_data(&second);
    crate::save_backups::restart(&mut managed, &boot_raw, &managed_path).unwrap();
    assert_eq!(fs::read(&managed_path).unwrap(), second);
    raw.bus.load_save_data(&fs::read(&managed_path).unwrap());
    assert_eq!(raw.bus.save_data().unwrap(), second);
    assert!(raw.bus.rom_resources().is_none());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
    fs::remove_file(&raw_path).unwrap();
    fs::remove_file(&managed_path).unwrap();

    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[170,169], "resources":113,
        "same_snapshot_bytes_with_and_without_base_dataset":true,
        "bidirectional_snapshot_load":true, "bidirectional_synthetic_cartridge_save":true,
        "invalid_header_version_rom_bios_checksum_payload_rejected_without_mutation":true,
        "wrong_rom_dataset_attach_rejected_without_mutation":true,
        "invalid_boot_rejected_before_cartridge_save_write":true,
        "owner_retained_and_raw_mode_remains_raw":true,
        "limits":"Synthetic direct APIs for the unmodified EUR E170 base. Cartridge saves remain original raw game-format bytes and have no added dataset-identity header; mod compatibility, GUI, actual game save semantics and other platforms remain open.",
        "performance_measurements":false
    })).unwrap()).unwrap();
}
