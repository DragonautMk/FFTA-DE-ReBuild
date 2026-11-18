//! Publication and original-reader checks for the opaque EUR source pair.
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::{
        conditional_language_pair::{ConditionalLanguagePair, SOURCE_OFFSET},
        layered_obj_animations::{CONTEXT_LANGUAGE_POSITION_A, CONTEXT_LANGUAGE_POSITION_B},
    },
    Bus, Machine,
};

fn original_pair(machine: &mut Machine, first: bool, language: u8) -> (u32, u32) {
    machine.bus.write8(0x0200_1f6e, language);
    machine.cpu.regs[1] = 0x0200_1f6e;
    machine.cpu.regs[3] = 0x0800_0000
        + if first {
            CONTEXT_LANGUAGE_POSITION_A
        } else {
            CONTEXT_LANGUAGE_POSITION_B
        };
    machine.cpu.regs[15] = if first { 0x0814_2064 } else { 0x0814_20a0 };
    machine.cpu.cpsr = 0xbf;
    let stop = if first { 0x0814_2072 } else { 0x0814_20ae };
    for _ in 0..32 {
        machine.bus.frame_ready = false;
        machine.step();
        if machine.cpu.regs[15] == stop {
            return (machine.cpu.regs[1], machine.cpu.regs[2]);
        }
    }
    panic!("original language-position reader did not finish");
}

#[test]
#[ignore = "exact EUR ROM and E167 dataset; Wine only, Linux/USA pending"]
fn european_conditional_language_pair_publication_and_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_LANGUAGE_GAP_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_LANGUAGE_GAP_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest =
        toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (167, 166));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (168, 167)
    );
    let prepared =
        preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(
        preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    assert_eq!(source.manifest.resources.len(), old.resources.len() + 1);
    for prior in &old.resources {
        let current = source
            .manifest
            .resources
            .iter()
            .find(|r| r.id == prior.id)
            .unwrap();
        assert_eq!(current, prior);
        assert_eq!(
            fs::read(prepared.directory.join(&current.path)).unwrap(),
            fs::read(previous.join(&prior.path)).unwrap()
        );
    }
    let catalog = source.conditional_language_pair.as_ref().unwrap();
    let model: ConditionalLanguagePair =
        serde_json::from_slice(&serde_json::to_vec(&catalog.decoded).unwrap()).unwrap();
    assert_eq!(model.source_offset, SOURCE_OFFSET);
    assert_eq!(
        model.opaque_bytes,
        rom[SOURCE_OFFSET as usize..SOURCE_OFFSET as usize + 2]
    );
    assert_eq!(catalog.sources.len(), 1);
    assert_eq!(catalog.sources[0].length, 2);
    let mut invalid = model.clone();
    invalid.source_offset += 1;
    assert!(invalid.validate().is_err());
    let mut overlapping = old.resources.clone();
    let mut partial = catalog.sources[0].clone();
    partial.length = 1;
    overlapping[0].sources.push(partial);
    assert!(
        gba_core::rom_data::conditional_language_pair::Catalog::from_rom(&rom, &overlapping)
            .is_err()
    );

    let mut decoded = read_dataset(&prepared.directory).unwrap();
    source.validate_files(decoded.files()).unwrap();
    decoded
        .conditional_language_pair
        .as_mut()
        .unwrap()
        .opaque_bytes[0] ^= 1;
    let changed =
        encode_conditional_language_pair(decoded.conditional_language_pair.as_ref().unwrap())
            .unwrap();
    decoded
        .manifest
        .resources
        .iter_mut()
        .find(|r| r.id == CONDITIONAL_LANGUAGE_PAIR_ID)
        .unwrap()
        .sha256 = gba_core::ffta_rom::digest(&changed);
    decoded.conditional_language_pair_bytes = Some(changed);
    assert!(
        source.validate_files(decoded.files()).is_err(),
        "modified source bytes must not be accepted even with a matching file hash"
    );

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let at = SOURCE_OFFSET as usize;
    managed.bus.rom[at - 3..at + 5].fill(0xcd);
    let before = owner.conditional_language_pair_stats().unwrap();
    let list_before = owner.obj_list_stats().unwrap();
    let mut expected_gap_bytes = 0u64;
    for address in at - 3..at + 5 {
        assert_eq!(
            managed.bus.read8(0x0800_0000 + address as u32),
            rom[address]
        );
        expected_gap_bytes += u64::from((at..at + 2).contains(&address));
    }
    let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for address in at - 3..at + 2 {
        assert_eq!(
            (RT_API.read16)(pointer, 0x0a00_0000 + address as u32),
            u32::from(u16::from_le_bytes(
                rom[address..address + 2].try_into().unwrap()
            ))
        );
        assert_eq!(
            (RT_API.read32)(pointer, 0x0800_0000 + address as u32),
            u32::from_le_bytes(rom[address..address + 4].try_into().unwrap())
        );
        expected_gap_bytes += (0..2)
            .filter(|i| (at..at + 2).contains(&(address + i)))
            .count() as u64;
        expected_gap_bytes += (0..4)
            .filter(|i| (at..at + 2).contains(&(address + i)))
            .count() as u64;
    }
    let after = owner.conditional_language_pair_stats().unwrap();
    assert_eq!(after.source_bytes - before.source_bytes, expected_gap_bytes);
    assert!(owner.obj_list_stats().unwrap().source_bytes > list_before.source_bytes);
    managed.bus.rom[at - 3..at + 5].copy_from_slice(&rom[at - 3..at + 5]);

    let initial = raw.save_state().unwrap();
    let before = owner.conditional_language_pair_stats().unwrap();
    for (first, index) in [(true, 35), (false, 30)] {
        raw.load_state(&initial).unwrap();
        managed.load_state(&initial).unwrap();
        managed.bus.rom[at..at + 2].fill(0xcd);
        assert_eq!(
            original_pair(&mut raw, first, index),
            original_pair(&mut managed, first, index)
        );
        managed.bus.rom[at..at + 2].copy_from_slice(&rom[at..at + 2]);
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
    }
    assert_eq!(
        owner
            .conditional_language_pair_stats()
            .unwrap()
            .source_bytes
            - before.source_bytes,
        4
    );
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[168,167], "source_bytes":2,
        "previous_resources_preserved":old.resources.len(), "created_reused_verified":true,
        "modified_pair_rejected":true, "partial_overlap_rejected":true,
        "controlled_reader_indices":[35,30], "controlled_full_state_clock_equal":true,
        "mixed_width_adjacent_owners_checked":true, "managed_sources_poisoned":true,
        "ordinary_indices_30_35":"unproved", "source_meaning":"opaque",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
