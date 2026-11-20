//! EUR DMA register directory publication, original readers and hardware writes.
use super::*;
use gba_core::{capi::RT_API, rom_data::dma_register_directories as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};

#[test]
#[ignore = "exact EUR ROM, E188 predecessor and Wine; Linux/USA pending"]
fn european_dma_register_directories_and_native_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_DMA_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_DMA_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (188, 187, 120));
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (189, 188, 121));
    for prior in &old.resources {
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    let data = &source.dma_register_directories.as_ref().unwrap().decoded;
    for (offset, bytes, _) in data.source_spans().unwrap() {
        assert_eq!(bytes, rom[offset as usize..offset as usize + model::DIRECTORY_LENGTH]);
    }
    assert_eq!(parse_dma_register_directories(&fs::read(prepared.directory.join(DMA_REGISTER_DIRECTORIES_FILE)).unwrap()).unwrap(), *data);
    let mut bad = data.clone(); bad.write_registers[0] += 4; assert!(bad.validate().is_err());
    let mut bad = data.clone(); bad.poll_registers[3] = 0x08000000; assert!(bad.validate().is_err());
    let bad_bytes = encode_dma_register_directories(&bad).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest.resources.iter_mut().find(|entry| entry.id == DMA_REGISTER_DIRECTORIES_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bad_bytes);
    let mut files = dataset.files(); files.manifest = &altered_manifest; files.dma_register_directories = Some((&bad, &bad_bytes));
    assert!(source.validate_files(files).is_err(), "coherent file/hash edits must be rejected");
    drop(dataset);

    let cases = [
        (0x08000846, 0usize, 2usize, model::WRITE_ROOT),
        (0x080009b2, 6, 1, model::WRITE_ROOT),
        (0x08000d56, 5, 0, model::WRITE_ROOT),
        (0x08003a7e, 0, 1, model::POLL_ROOT),
    ];
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = cases.iter().map(|&(pc, _, _, _)| pc).chain([0x080009b4]).map(|addr|
        Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("dma-register-readers-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let start = model::WRITE_ROOT as usize; let end = start + model::DIRECTORY_LENGTH * 2;
    for at in start..end { managed.bus.rom[at] = !rom[at]; }
    for at in start..end { assert_eq!(managed.bus.read8(0x0c000000 + at as u32), rom[at]); }
    for edge in [start, start + model::DIRECTORY_LENGTH, end] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap(); let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
            assert_eq!((RT_API.read16)(pointer, 0x0a000000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
            assert_eq!((RT_API.read32)(pointer, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
        }
    }
    let before = owner.dma_register_directories_stats().unwrap().directory_bytes;
    let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
    assert_eq!((RT_API.read32)(pointer, 0x08000000 + (start + model::DIRECTORY_LENGTH - 2) as u32),
        u32::from_le_bytes(rom[start + model::DIRECTORY_LENGTH - 2..start + model::DIRECTORY_LENGTH + 2].try_into().unwrap()));
    let after = owner.dma_register_directories_stats().unwrap().directory_bytes;
    assert_eq!((after[0] - before[0], after[1] - before[1]), (2, 2));
    managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
    let initial = raw.save_state().unwrap();
    for (mode, &(reader, base, target, table_root)) in cases.iter().enumerate() {
        for index in 0..4 {
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
                machine.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
                if mode == 1 {
                    machine.bus.write32(0x040000b8 + index as u32 * 12, 0x10001234);
                    machine.cpu.regs[2] = 0;
                }
                machine.cpu.regs[base] = 0x08000000 + table_root + index as u32 * 4;
                machine.cpu.regs[13] = 0x03007c00; machine.cpu.regs[15] = reader; machine.cpu.cpsr = 0xbf;
            }
            for at in start..end { managed.bus.rom[at] = !rom[at]; }
            let table_index = usize::from(table_root == model::POLL_ROOT);
            let before = owner.dma_register_directories_stats().unwrap().directory_bytes[table_index];
            raw.bus.frame_ready = false; raw.step(); managed.bus.frame_ready = false;
            let native_exit = table.get(reader | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void);
            assert_eq!(owner.dma_register_directories_stats().unwrap().directory_bytes[table_index] - before, 4);
            assert_eq!(raw.cpu.regs[target], 0x040000b0 + index as u32 * 12);
            if mode == 1 {
                let offset = 0xb8 + index * 12;
                assert_eq!(&raw.bus.io[offset..offset + 4], &[0x34, 0x12, 0, 0x10]);
                // The adjacent native LDR and STR blocks chain into one call.
                assert_eq!(native_exit, 0x080009b7);
                raw.bus.frame_ready = false; raw.step();
                assert_eq!(&raw.bus.io[offset..offset + 4], &[0, 0, 0, 0]);
                assert_eq!(&managed.bus.io[offset..offset + 4], &[0, 0, 0, 0]);
            } else {
                assert_eq!(native_exit, (reader + 2) | 1);
            }
            managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
            assert!(managed.save_state().unwrap() == raw.save_state().unwrap(),
                "state at reader {reader:#x}, index {index}, clocks {} / {}", managed.bus.clock, raw.bus.clock);
        }
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[189,188], "resources":121, "preserved_other_resources":120,
        "directories":2, "new_source_bytes":32, "poisoned_mirrored_and_mixed_boundary_reads":true,
        "coherent_edit_rejected":true, "original_interpreter_native_reader_cases":16,
        "controlled_dma_register_write_cases":4, "full_state_clock_equal":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
