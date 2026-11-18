//! EUR publication and original prefixes for the conditional text-reader registry.
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::text_reader_registry::{
        Catalog, TextReaderRegistry, LOOKUP_LITERAL, PRODUCER_LITERAL, TABLE_OFFSET,
    },
    Bus, Machine,
};

fn original_prefix(machine: &mut Machine, producer: bool, index: u8) -> u32 {
    machine.cpu.regs[1] = u32::from(index);
    machine.cpu.regs[4] = u32::from(index);
    machine.cpu.regs[3] = 0x0200_1000;
    machine.cpu.regs[15] = if producer { 0x0801_9172 } else { 0x0801_9424 };
    machine.cpu.cpsr = 0xbf;
    let stop = if producer { 0x0801_9182 } else { 0x0801_9430 };
    for _ in 0..32 {
        machine.bus.frame_ready = false;
        machine.step();
        if machine.cpu.regs[15] == stop {
            return machine.cpu.regs[0];
        }
    }
    panic!("original text-reader registry prefix did not finish");
}

#[test]
#[ignore = "exact EUR ROM and E168 dataset; Wine only, Linux/USA pending"]
fn european_text_reader_registry_publication_and_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_TEXT_READER_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_TEXT_READER_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest =
        toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (168, 167));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (169, 168)
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
    let catalog = source.text_reader_registry.as_ref().unwrap();
    let model: TextReaderRegistry =
        serde_json::from_slice(&serde_json::to_vec(&catalog.decoded).unwrap()).unwrap();
    assert_eq!(model.table_offset, TABLE_OFFSET);
    assert_eq!(model.callbacks.len(), 27);
    assert_eq!(
        [model.callbacks[0], model.callbacks[3], model.callbacks[5]],
        [0; 3]
    );
    assert_eq!(model.callbacks[11], 0x0801_6c01);
    assert_eq!(
        catalog.sources.iter().map(|s| s.length).collect::<Vec<_>>(),
        [4, 4, 108]
    );
    let mut invalid = model.clone();
    invalid.table_offset += 1;
    assert!(invalid.validate().is_err());
    let mut overlap = old.resources.clone();
    let mut partial = catalog.sources[2].clone();
    partial.length = 1;
    overlap[0].sources.push(partial);
    assert!(Catalog::from_rom(&rom, &overlap).is_err());
    let mut decoded = read_dataset(&prepared.directory).unwrap();
    source.validate_files(decoded.files()).unwrap();
    decoded.text_reader_registry.as_mut().unwrap().callbacks[11] = 0;
    let changed =
        encode_text_reader_registry(decoded.text_reader_registry.as_ref().unwrap()).unwrap();
    decoded
        .manifest
        .resources
        .iter_mut()
        .find(|r| r.id == TEXT_READER_REGISTRY_ID)
        .unwrap()
        .sha256 = gba_core::ffta_rom::digest(&changed);
    decoded.text_reader_registry_bytes = Some(changed);
    assert!(
        source.validate_files(decoded.files()).is_err(),
        "edited callback must be rejected even with a matching file hash"
    );

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let spans = [
        (PRODUCER_LITERAL as usize, 4usize),
        (LOOKUP_LITERAL as usize, 4),
        (TABLE_OFFSET as usize, 108),
    ];
    for (at, len) in spans {
        managed.bus.rom[at..at + len].fill(0xcd);
    }
    let before = owner.text_reader_registry_stats().unwrap();
    for (at, len) in spans {
        for address in at..at + len {
            assert_eq!(
                managed.bus.read8(0x0800_0000 + address as u32),
                rom[address]
            );
        }
    }
    let after = owner.text_reader_registry_stats().unwrap();
    assert_eq!(after.source_bytes - before.source_bytes, 116);
    assert_eq!(after.literal_bytes - before.literal_bytes, 8);
    assert_eq!(after.callback_bytes - before.callback_bytes, 108);
    let boundary = TABLE_OFFSET as usize;
    // The byte before this projection is raw; the following context bytes have an earlier owner.
    managed.bus.rom[boundary + 108..boundary + 112].fill(0xcd);
    let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for at in [boundary - 1, boundary, boundary + 105, boundary + 107] {
        assert_eq!(
            (RT_API.read16)(pointer, 0x0a00_0000 + at as u32),
            u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap()))
        );
        assert_eq!(
            (RT_API.read32)(pointer, 0x0800_0000 + at as u32),
            u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
        );
    }
    managed.bus.rom[boundary + 108..boundary + 112]
        .copy_from_slice(&rom[boundary + 108..boundary + 112]);
    for (at, len) in spans {
        managed.bus.rom[at..at + len].copy_from_slice(&rom[at..at + len]);
    }
    let initial = raw.save_state().unwrap();
    let before = owner.text_reader_registry_stats().unwrap();
    let mut comparisons = 0;
    for producer in [true, false] {
        for index in [0u8, 3, 5, 11, 26, 27] {
            raw.load_state(&initial).unwrap();
            managed.load_state(&initial).unwrap();
            for (at, len) in spans {
                managed.bus.rom[at..at + len].fill(0xcd);
            }
            assert_eq!(
                original_prefix(&mut raw, producer, index),
                original_prefix(&mut managed, producer, index)
            );
            for (at, len) in spans {
                managed.bus.rom[at..at + len].copy_from_slice(&rom[at..at + len]);
            }
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
            comparisons += 1;
        }
    }
    let after = owner.text_reader_registry_stats().unwrap();
    assert_eq!(after.literal_bytes - before.literal_bytes, 12 * 4);
    assert_eq!(after.callback_bytes - before.callback_bytes, 10 * 4);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[169,168], "source_bytes":116,
        "previous_resources_preserved":old.resources.len(), "created_reused_verified":true,
        "modified_callback_rejected":true, "partial_overlap_rejected":true,
        "original_prefix_comparisons":comparisons, "indices":[0,3,5,11,26,27],
        "full_state_clock_equal":true, "mixed_width_adjacent_owner_checked":true,
        "managed_sources_poisoned":true, "guest_index_bound":"unchanged",
        "ordinary_use":"unproved", "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
