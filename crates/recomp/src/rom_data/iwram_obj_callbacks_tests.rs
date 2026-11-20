//! EUR observed OBJ callback words and original Thumb branch entry ABI.
use super::*;
use gba_core::{capi::RT_API, rom_data::obj_lists as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::collections::BTreeSet;

#[test]
#[ignore = "exact EUR ROM, E187 predecessor and Wine; Linux/USA pending"]
fn european_iwram_obj_callbacks_publication_and_native_branches() {
    let root = PathBuf::from(std::env::var_os("FFTA_IWRAM_CALLBACK_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_IWRAM_CALLBACK_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (187, 186, 120));
    let old_data: serde_json::Value = serde_json::from_slice(&fs::read(previous.join(OBJ_STATIC_LISTS_FILE)).unwrap()).unwrap();
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (188, 187, 120));
    for prior in &old.resources {
        if prior.id == OBJ_STATIC_LISTS_ID { continue; }
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    let data = &source.obj_lists.as_ref().unwrap().decoded;
    let new_data = serde_json::to_value(data).unwrap();
    for (key, value) in old_data.as_object().unwrap() { assert_eq!(&new_data[key], value, "preserved field {key}"); }
    let current = source.manifest.resources.iter().find(|entry| entry.id == OBJ_STATIC_LISTS_ID).unwrap();
    let prior = old.resources.iter().find(|entry| entry.id == OBJ_STATIC_LISTS_ID).unwrap();
    let without_new: Vec<_> = current.sources.iter().filter(|entry| entry.offset != model::IWRAM_DRAW_CALLBACKS_ROOT).cloned().collect();
    assert_eq!(without_new, prior.sources);
    assert_eq!(current.sources.len(), prior.sources.len() + 1);
    let callbacks = data.iwram_draw_callbacks.unwrap();
    assert_eq!(callbacks.iter().flat_map(|word| word.to_le_bytes()).collect::<Vec<_>>(),
        rom[model::IWRAM_DRAW_CALLBACKS_ROOT as usize..model::IWRAM_DRAW_CALLBACKS_ROOT as usize + 20]);
    data.verify_source(&rom).unwrap();
    assert_eq!(parse_obj_lists(&fs::read(prepared.directory.join(OBJ_STATIC_LISTS_FILE)).unwrap()).unwrap(), *data);
    let mut bad = data.clone(); bad.iwram_draw_callbacks.as_mut().unwrap()[0] &= !1; assert!(bad.validate().is_err());
    let mut bad = data.clone(); bad.iwram_draw_callbacks.as_mut().unwrap()[0] = 0x08006171; assert!(bad.validate().is_err());
    let mut bad = data.clone(); bad.iwram_draw_callbacks.as_mut().unwrap()[0] += 2;
    let bad_bytes = encode_obj_lists(&bad).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest.resources.iter_mut().find(|entry| entry.id == OBJ_STATIC_LISTS_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bad_bytes);
    let mut files = dataset.files(); files.manifest = &altered_manifest; files.obj_lists = Some((&bad, &bad_bytes));
    assert!(source.validate_files(files).is_err(), "coherent file/hash edits must be rejected");
    drop(dataset);

    // Each original LDR is followed by its existing BX-register bridge. The
    // loaded PC/Thumb bit and caller LR remain guest-controlled.
    let cases = [
        (0x080923f4, 1usize, 1usize, 0x08145c0c),
        (0x080922fc, 6, 4, 0x08145c18),
        (0x08091f5e, 1, 2, 0x08145c10),
        (0x080925ca, 4, 4, 0x08145c18),
        (0x080920a8, 7, 4, 0x08145c18),
    ];
    let visited: BTreeSet<u32> = cases.iter().flat_map(|&(reader, _, _, bridge)| [reader, bridge]).collect();
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = visited.iter().map(|&addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("iwram-obj-callbacks-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let start = model::IWRAM_DRAW_CALLBACKS_ROOT as usize; let end = start + 4 * model::IWRAM_DRAW_CALLBACKS_COUNT;
    for at in start..end { managed.bus.rom[at] = !rom[at]; }
    for at in start..end { assert_eq!(managed.bus.read8(0x0c000000 + at as u32), rom[at]); }
    for edge in [start, end] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap(); let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
            assert_eq!((RT_API.read16)(pointer, 0x0a000000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
            assert_eq!((RT_API.read32)(pointer, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
        }
    }
    let before = owner.obj_list_stats().unwrap().iwram_callback_bytes;
    let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
    assert_eq!((RT_API.read32)(pointer, 0x08000000 + (start + 2) as u32), u32::from_le_bytes(rom[start + 2..start + 6].try_into().unwrap()));
    let after = owner.obj_list_stats().unwrap().iwram_callback_bytes;
    assert_eq!((after[0] - before[0], after[1] - before[1]), (2, 2));
    managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
    let initial = raw.save_state().unwrap();
    for (index, &(reader, base, target, bridge)) in cases.iter().enumerate() {
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
            machine.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
            machine.cpu.regs[base] = 0x08000000 + model::IWRAM_DRAW_CALLBACKS_ROOT + index as u32 * 4;
            machine.cpu.regs[13] = 0x03007c00; machine.cpu.regs[14] = 0x08000301;
            machine.cpu.regs[15] = reader; machine.cpu.cpsr = 0xbf;
        }
        for at in start..end { managed.bus.rom[at] = !rom[at]; }
        let before = owner.obj_list_stats().unwrap().iwram_callback_bytes[index];
        raw.bus.frame_ready = false; raw.step(); managed.bus.frame_ready = false;
        table.get(reader | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void);
        assert_eq!(owner.obj_list_stats().unwrap().iwram_callback_bytes[index] - before, 4);
        assert_eq!(raw.cpu.regs[target], callbacks[index]);
        for machine in [&mut raw, &mut managed] { machine.cpu.regs[15] = bridge; }
        raw.bus.frame_ready = false; raw.step(); managed.bus.frame_ready = false;
        table.get(bridge | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void);
        assert_eq!(raw.cpu.regs[15], callbacks[index] & !1);
        assert_eq!(raw.cpu.regs[14], 0x08000301);
        managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
        assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap());
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[188,187], "resources":120, "preserved_other_resources":119,
        "previous_obj_lists_preserved":true, "iwram_callback_words":5, "new_source_bytes":20,
        "poisoned_mirrored_and_mixed_boundary_reads":true, "coherent_edit_rejected":true,
        "original_interpreter_native_branch_cases":5, "full_state_clock_equal":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
