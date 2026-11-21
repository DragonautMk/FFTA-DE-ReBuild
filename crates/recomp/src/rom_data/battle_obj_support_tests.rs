//! EUR battle OBJ callbacks and actor lookup: source, ownership and ABI.
use super::*;
use gba_core::{capi::RT_API, rom_data::obj_lists as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

const RETURN: u32 = 0x0800_0300;
const STACK: u32 = 0x0300_7c00;
const LOOKUP: u32 = 0x080a_3070;
const CALLBACK_CASES: [(u32, usize, usize, u32); 7] = [
    (0x0809_d82e, 4, 2, 0x0814_5c10),
    (0x080a_2074, 1, 4, 0x0814_5c18),
    (0x0809_d8ec, 2, 2, 0x0814_5c10),
    (0x0809_6008, 1, 1, 0x0814_5c0c),
    (0x080a_c7c8, 4, 4, 0x0814_5c18),
    (0x0809_dd60, 5, 2, 0x0814_5c10),
    (0x080d_6cc8, 3, 3, 0x0814_5c14),
];

fn setup(machine: &mut Machine) {
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[13] = STACK;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.cpsr = 0xbf;
}

#[test]
#[ignore = "exact EUR ROM and E207 predecessor; Wine only, Linux/USA pending"]
fn european_battle_obj_support_publication_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_BATTLE_SUPPORT_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_BATTLE_SUPPORT_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (207, 206, 123));
    let prior = parse_obj_lists(&fs::read(previous.join(OBJ_STATIC_LISTS_FILE)).unwrap()).unwrap();
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (208, 207, 123));
    for old_resource in &old.resources {
        if old_resource.id == OBJ_STATIC_LISTS_ID { continue; }
        let current = source.manifest.resources.iter().find(|item| item.id == old_resource.id).unwrap();
        assert_eq!(current, old_resource);
        assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&old_resource.path)).unwrap());
    }
    let data = &source.obj_lists.as_ref().unwrap().decoded;
    let mut without_new = data.clone();
    let callbacks = without_new.iwram_additional_callbacks.take().unwrap();
    let values = without_new.actor_lookup_extension.take().unwrap();
    assert_eq!(without_new, prior);
    for (i, &at) in model::IWRAM_ADDITIONAL_CALLBACK_ROOTS.iter().enumerate() {
        assert_eq!(callbacks[i].to_le_bytes(), rom[at as usize..at as usize + 4]);
    }
    for (i, value) in values.iter().enumerate() {
        let at = model::ACTOR_LOOKUP_EXTENSION_ROOT as usize + i * 2;
        assert_eq!(value.to_le_bytes(), rom[at..at + 2]);
    }
    let current = source.manifest.resources.iter().find(|item| item.id == OBJ_STATIC_LISTS_ID).unwrap();
    let original = old.resources.iter().find(|item| item.id == OBJ_STATIC_LISTS_ID).unwrap();
    let retained: Vec<_> = current.sources.iter().filter(|s| s.offset != model::ACTOR_LOOKUP_EXTENSION_ROOT
        && !model::IWRAM_ADDITIONAL_CALLBACK_ROOTS.contains(&s.offset)).cloned().collect();
    assert_eq!(retained, original.sources);
    assert_eq!(current.sources.len(), original.sources.len() + 8);
    assert_eq!(parse_obj_lists(&fs::read(prepared.directory.join(OBJ_STATIC_LISTS_FILE)).unwrap()).unwrap(), *data);
    data.verify_source(&rom).unwrap();
    let mut invalid = data.clone(); invalid.iwram_additional_callbacks.as_mut().unwrap()[0] &= !1;
    assert!(invalid.validate().is_err());
    let mut forged = data.clone(); forged.actor_lookup_extension.as_mut().unwrap()[0] ^= 1;
    forged.validate().unwrap(); assert!(forged.verify_source(&rom).is_err());

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let original_state = raw.save_state().unwrap();
    // Old and new adjacent owners are poisoned together; gaps remain raw.
    for at in (0x397f80..0x397fb4).chain(0x397fb8..0x397fbc).chain(0x398df0..0x398dfc) { managed.bus.rom[at] = !rom[at]; }
    let before = owner.obj_list_stats().unwrap();
    for &at in &model::IWRAM_ADDITIONAL_CALLBACK_ROOTS {
        for i in 0..4 { assert_eq!(managed.bus.read8(0x0c00_0000 + at + i), rom[(at + i) as usize]); }
    }
    for at in 0x398df4..0x398dfc { assert_eq!(managed.bus.read8(0x0c00_0000 + at), rom[at as usize]); }
    let after = owner.obj_list_stats().unwrap();
    for i in 0..7 { assert_eq!(after.iwram_additional_callback_bytes[i] - before.iwram_additional_callback_bytes[i], 4); }
    assert_eq!(after.actor_lookup_extension_bytes - before.actor_lookup_extension_bytes, 8);
    let mut edges = BTreeSet::new();
    for at in model::IWRAM_ADDITIONAL_CALLBACK_ROOTS.into_iter().chain([model::ACTOR_LOOKUP_EXTENSION_ROOT]) {
        let len = if at == model::ACTOR_LOOKUP_EXTENSION_ROOT { 8 } else { 4 };
        for edge in [at, at + len] { for delta in [-3i32, -1, 0, 1] { edges.insert(edge.wrapping_add_signed(delta)); } }
    }
    for at in edges {
        let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
        for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
            let i = at as usize;
            assert_eq!((RT_API.read16)(ptr, mirror + at), u32::from(u16::from_le_bytes(rom[i..i + 2].try_into().unwrap())), "read16 {at:x}");
            assert_eq!((RT_API.read32)(ptr, mirror + at), u32::from_le_bytes(rom[i..i + 4].try_into().unwrap()), "read32 {at:x}");
        }
    }
    for (a, b) in [(0x397f80, 0x397fb4), (0x397fb8, 0x397fbc), (0x398df0, 0x398dfc)] {
        managed.bus.rom[a..b].copy_from_slice(&rom[a..b]);
    }

    let mut addresses = BTreeSet::new();
    for &(reader, _, _, bridge) in &CALLBACK_CASES { addresses.extend([reader, bridge]); }
    addresses.extend((LOOKUP..LOOKUP + 10).step_by(2));
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = addresses.into_iter().map(|addr| Block {
        addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
    }).collect();
    let object = root.join("battle-obj-support-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    for native in [false, true] {
        for (index, &(reader, base, target, bridge)) in CALLBACK_CASES.iter().enumerate() {
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&original_state).unwrap(); setup(machine);
                machine.cpu.regs[base] = 0x0800_0000 + model::IWRAM_ADDITIONAL_CALLBACK_ROOTS[index];
                machine.cpu.regs[15] = reader;
            }
            let at = model::IWRAM_ADDITIONAL_CALLBACK_ROOTS[index] as usize;
            for i in at..at + 4 { managed.bus.rom[i] = !rom[i]; }
            let before = owner.obj_list_stats().unwrap().iwram_additional_callback_bytes[index];
            raw.bus.frame_ready = false; raw.step(); managed.bus.frame_ready = false;
            if native { table.get(reader | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void); }
            else { managed.step(); }
            assert_eq!(owner.obj_list_stats().unwrap().iwram_additional_callback_bytes[index] - before, 4);
            assert_eq!(raw.cpu.regs[target], callbacks[index]);
            for machine in [&mut raw, &mut managed] { machine.cpu.regs[15] = bridge; machine.bus.frame_ready = false; }
            raw.step();
            if native { table.get(bridge | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void); }
            else { managed.step(); }
            assert_eq!(raw.cpu.regs[15], callbacks[index] & !1);
            assert_eq!(raw.cpu.regs[14], RETURN | 1);
            managed.bus.rom[at..at + 4].copy_from_slice(&rom[at..at + 4]);
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "callback {index}, native={native}");
        }
        for index in [-2i32, -1, 0, 1, 2, 3, 4, 5, 6, 7, 255] {
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&original_state).unwrap(); setup(machine);
                machine.cpu.regs[0] = index as u32; machine.cpu.regs[15] = LOOKUP;
            }
            for at in 0x398df4..0x398dfc { managed.bus.rom[at] = !rom[at]; }
            let before = owner.obj_list_stats().unwrap().actor_lookup_extension_bytes;
            for _ in 0..32 { if raw.cpu.regs[15] == RETURN { break; } raw.bus.frame_ready = false; raw.step(); }
            for _ in 0..32 {
                if managed.cpu.regs[15] == RETURN { break; }
                managed.bus.frame_ready = false;
                if native { table.get(managed.cpu.regs[15] | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void); }
                else { managed.step(); }
            }
            assert_eq!((raw.cpu.regs[15], managed.cpu.regs[15]), (RETURN, RETURN));
            assert_eq!(owner.obj_list_stats().unwrap().actor_lookup_extension_bytes - before, if (2..=5).contains(&index) { 2 } else { 0 });
            let at = (model::ACTOR_LOOKUP_ROOT as usize).wrapping_add_signed(index as isize * 2);
            assert_eq!(raw.cpu.regs[0], u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
            managed.bus.rom[0x398df4..0x398dfc].copy_from_slice(&rom[0x398df4..0x398dfc]);
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "lookup {index}, native={native}");
        }
    }
    let state = managed.save_state().unwrap();
    let unavailable = root.join("temporarily unavailable dataset");
    fs::rename(&prepared.directory, &unavailable).unwrap();
    assert!(attach_from_directory(&mut managed, &prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    managed.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    fs::rename(&unavailable, &prepared.directory).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[208,207], "prior_resources_preserved":122,
        "prior_obj_projection_preserved":true, "additional_callback_words":7, "additional_lookup_entries":4,
        "new_source_bytes":36, "poisoned_mixed_boundaries_and_mirrors":true,
        "original_interpreter_generated_callback_cases":14, "original_interpreter_generated_lookup_cases":22,
        "missing_dataset_and_snapshot_owner_retained":true, "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
