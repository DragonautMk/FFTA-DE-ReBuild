//! EUR IWRAM memory-helper vector and original Thumb branch entries.
use super::*;
use gba_core::{capi::RT_API, rom_data::iwram_memory_helpers as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::collections::BTreeSet;

#[test]
#[ignore = "exact EUR ROM, E189 predecessor and Wine; Linux/USA pending"]
fn european_iwram_memory_helpers_and_native_branches() {
    let root = PathBuf::from(std::env::var_os("FFTA_IWRAM_HELPER_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_IWRAM_HELPER_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (189, 188, 121));
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (190, 189, 122));
    for prior in &old.resources {
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    let data = &source.iwram_memory_helpers.as_ref().unwrap().decoded;
    assert_eq!(data.source_bytes().unwrap(), rom[model::SOURCE_OFFSET as usize..model::SOURCE_OFFSET as usize + model::SOURCE_LENGTH]);
    assert_eq!(parse_iwram_memory_helpers(&fs::read(prepared.directory.join(IWRAM_MEMORY_HELPERS_FILE)).unwrap()).unwrap(), *data);
    let mut bad = data.clone(); bad.callbacks[0] &= !1; assert!(bad.validate().is_err());
    let mut bad = data.clone(); bad.callbacks[0] = 0x08005e79; assert!(bad.validate().is_err());
    let mut bad = data.clone(); bad.callbacks[0] += 2;
    let bad_bytes = encode_iwram_memory_helpers(&bad).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest.resources.iter_mut().find(|entry| entry.id == IWRAM_MEMORY_HELPERS_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bad_bytes);
    let mut files = dataset.files(); files.manifest = &altered_manifest; files.iwram_memory_helpers = Some((&bad, &bad_bytes));
    assert!(source.validate_files(files).is_err(), "coherent file/hash edits must be rejected");
    drop(dataset);

    let cases = [
        (0x080011da, 5usize, 2usize, 0x08145c10),
        (0x08028ff4, 4, 3, 0x08145c14),
        (0x0802903a, 2, 3, 0x08145c14),
    ];
    let visited: BTreeSet<u32> = cases.iter().flat_map(|&(reader, _, _, bridge)| [reader, bridge]).collect();
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = visited.iter().map(|&addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("iwram-memory-helpers-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let start = model::SOURCE_OFFSET as usize; let end = start + model::SOURCE_LENGTH;
    for at in start..end { managed.bus.rom[at] = !rom[at]; }
    for at in start..end { assert_eq!(managed.bus.read8(0x0c000000 + at as u32), rom[at]); }
    for edge in [start, start + 4, start + 8, end] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap(); let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
            assert_eq!((RT_API.read16)(pointer, 0x0a000000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
            assert_eq!((RT_API.read32)(pointer, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
        }
    }
    let before = owner.iwram_memory_helpers_stats().unwrap().helper_bytes;
    let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
    assert_eq!((RT_API.read32)(pointer, 0x08000000 + (start + 2) as u32), u32::from_le_bytes(rom[start + 2..start + 6].try_into().unwrap()));
    let after = owner.iwram_memory_helpers_stats().unwrap().helper_bytes;
    assert_eq!((after[0] - before[0], after[1] - before[1]), (2, 2));
    managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
    let initial = raw.save_state().unwrap();
    for (index, &(reader, base, target, bridge)) in cases.iter().enumerate() {
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
            machine.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
            machine.cpu.regs[base] = 0x08000000 + model::SOURCE_OFFSET + index as u32 * 4;
            machine.cpu.regs[13] = 0x03007c00; machine.cpu.regs[14] = 0x08000301;
            machine.cpu.regs[15] = reader; machine.cpu.cpsr = 0xbf;
        }
        for at in start..end { managed.bus.rom[at] = !rom[at]; }
        let before = owner.iwram_memory_helpers_stats().unwrap().helper_bytes[index];
        raw.bus.frame_ready = false; raw.step(); managed.bus.frame_ready = false;
        table.get(reader | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void);
        assert_eq!(owner.iwram_memory_helpers_stats().unwrap().helper_bytes[index] - before, 4);
        assert_eq!(raw.cpu.regs[target], data.callbacks[index]);
        for machine in [&mut raw, &mut managed] { machine.cpu.regs[15] = bridge; }
        raw.bus.frame_ready = false; raw.step(); managed.bus.frame_ready = false;
        table.get(bridge | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void);
        assert_eq!(raw.cpu.regs[15], data.callbacks[index] & !1);
        assert_eq!(raw.cpu.regs[14], 0x08000301);
        managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
        assert!(managed.save_state().unwrap() == raw.save_state().unwrap(), "helper branch {index}");
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[190,189], "resources":122, "preserved_other_resources":121,
        "iwram_helper_words":3, "new_source_bytes":12, "poisoned_mirrored_and_mixed_boundary_reads":true,
        "coherent_edit_rejected":true, "original_interpreter_native_branch_cases":3, "full_state_clock_equal":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
