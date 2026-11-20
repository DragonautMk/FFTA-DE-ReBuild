//! EUR action-callback records and original Thumb reader/branch ABI.
use super::*;
use gba_core::{capi::RT_API, rom_data::records4 as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::collections::BTreeSet;

#[test]
#[ignore = "exact EUR ROM, E192 predecessor and Wine; Linux/USA pending"]
fn european_action_callbacks_publication_and_native_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_ACTION_CALLBACK_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_ACTION_CALLBACK_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (192, 191, 122));
    let old_data: serde_json::Value = serde_json::from_slice(&fs::read(previous.join(RECORDS4_FILE)).unwrap()).unwrap();
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (193, 192, 122));
    for prior in &old.resources {
        if prior.id == RECORDS4_ID { continue; }
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    let data = &source.records4.as_ref().unwrap().decoded;
    let new_data = serde_json::to_value(data).unwrap();
    for (key, value) in old_data.as_object().unwrap() { assert_eq!(&new_data[key], value, "preserved field {key}"); }
    let current = source.manifest.resources.iter().find(|entry| entry.id == RECORDS4_ID).unwrap();
    let prior = old.resources.iter().find(|entry| entry.id == RECORDS4_ID).unwrap();
    let without_new: Vec<_> = current.sources.iter().filter(|entry| entry.offset != model::ACTION_ROOT).cloned().collect();
    assert_eq!(without_new, prior.sources);
    assert_eq!(current.sources.len(), prior.sources.len() + 1);
    let actions = data.action_callbacks.as_ref().unwrap();
    assert_eq!(actions.len(), model::ACTION_COUNT);
    assert_eq!(actions.iter().flat_map(model::ActionCallback::source_bytes).collect::<Vec<_>>(),
        rom[model::ACTION_ROOT as usize..model::ACTION_END as usize]);
    data.verify_source(&rom).unwrap();
    assert_eq!(parse_records4(&fs::read(prepared.directory.join(RECORDS4_FILE)).unwrap()).unwrap(), *data);
    let mut bad = data.clone(); bad.action_callbacks.as_mut().unwrap()[0].callback &= !1; assert!(bad.validate().is_err());
    let mut bad = data.clone(); bad.action_callbacks.as_mut().unwrap().pop(); assert!(bad.validate().is_err());
    let mut bad = data.clone(); bad.action_callbacks.as_mut().unwrap()[0].opaque_first ^= 1;
    let bad_bytes = encode_records4(&bad).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest.resources.iter_mut().find(|entry| entry.id == RECORDS4_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bad_bytes);
    let mut files = dataset.files(); files.manifest = &altered_manifest; files.records4 = Some((&bad, &bad_bytes));
    assert!(source.validate_files(files).is_err(), "coherent file/hash edits must be rejected");
    drop(dataset);

    // Both readers use the original record address; the pointer feeds BX r1.
    let cases = [(0x0813752e, true), (0x08137610, false)]; let bridge = 0x08145c0c;
    let visited: BTreeSet<u32> = cases.iter().map(|&(reader, _)| reader).chain([bridge]).collect();
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = visited.iter().map(|&addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("action-callbacks-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None); let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let start = model::ACTION_ROOT as usize; let end = model::ACTION_END as usize;
    for at in start..end { managed.bus.rom[at] = !rom[at]; }
    for at in start..end { assert_eq!(managed.bus.read8(0x0c000000 + at as u32), rom[at]); }
    for edge in [start, start + model::ACTION_RECORD_BYTES, end] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap(); let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
            assert_eq!((RT_API.read16)(pointer, 0x0a000000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
            assert_eq!((RT_API.read32)(pointer, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
        }
    }
    managed.bus.rom[start..end].copy_from_slice(&rom[start..end]); let initial = raw.save_state().unwrap();
    for &(reader, is_callback) in &cases {
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
            machine.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
            machine.cpu.regs[0] = 0x08000000 + model::ACTION_ROOT + 52 * model::ACTION_RECORD_BYTES as u32;
            machine.cpu.regs[13] = 0x03007c00; machine.cpu.regs[14] = 0x08000301;
            machine.cpu.regs[15] = reader; machine.cpu.cpsr = 0xbf;
        }
        for at in start..end { managed.bus.rom[at] = !rom[at]; }
        let field = if is_callback { 0 } else { 2 }; let before = owner.records4_stats().unwrap().action_field_bytes[field];
        raw.bus.frame_ready = false; raw.step(); managed.bus.frame_ready = false;
        table.get(reader | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void);
        assert_eq!(owner.records4_stats().unwrap().action_field_bytes[field] - before, if is_callback { 4 } else { 1 });
        if is_callback {
            assert_eq!(raw.cpu.regs[1], actions[52].callback);
            for machine in [&mut raw, &mut managed] { machine.cpu.regs[15] = bridge; }
            raw.bus.frame_ready = false; raw.step(); managed.bus.frame_ready = false;
            table.get(bridge | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void);
            assert_eq!(raw.cpu.regs[15], actions[52].callback & !1); assert_eq!(raw.cpu.regs[14], 0x08000301);
        } else { assert_eq!(raw.cpu.regs[0], actions[52].opaque_second & 0xff); }
        managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
        assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap());
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[193,192], "resources":122, "preserved_other_resources":121,
        "previous_records4_preserved":true, "action_records":93, "new_source_bytes":1116,
        "poisoned_mirrored_and_mixed_boundary_reads":true, "coherent_edit_rejected":true,
        "original_interpreter_native_reader_cases":2, "full_state_clock_equal":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
