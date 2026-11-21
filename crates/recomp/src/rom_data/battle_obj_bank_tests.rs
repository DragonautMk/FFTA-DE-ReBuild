//! EUR battle OBJ bank: source identity, publication and actual readers.
use super::*;
use gba_core::{capi::RT_API, rom_data::obj_animations as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

const CONTEXT: u32 = 0x0202_0000;
const STACK: u32 = 0x0300_7c00;
const ENTRY: u32 = 0x080a_c4b4;
const STOP: u32 = 0x080a_c4bc;

fn backing(machine: &mut Machine, rom: &[u8], spans: &[(u32, usize, &'static str)], poison: bool) {
    for &(start, len, _) in spans {
        let start = start as usize;
        for at in start..start + len {
            machine.bus.rom[at] = if poison { !rom[at] } else { rom[at] };
        }
    }
}

#[test]
#[ignore = "exact EUR ROM and E206 predecessor; Wine only, Linux/USA pending"]
fn european_battle_obj_bank_publication_and_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_BATTLE_OBJ_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_BATTLE_OBJ_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old_manifest: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old_manifest.extractor_revision, old_manifest.decoder_revision, old_manifest.resources.len()), (206, 205, 123));
    let old = parse_obj_animations(&fs::read(previous.join(OBJ_ANIMATION_BANKS_FILE)).unwrap()).unwrap();
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (207, 206, 123));
    for prior in &old_manifest.resources {
        if prior.id == OBJ_ANIMATION_BANKS_ID { continue; }
        let current = source.manifest.resources.iter().find(|item| item.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&prior.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    let data = &source.obj_animations.as_ref().unwrap().decoded;
    assert_eq!(&data.banks[1..], old.banks.as_slice());
    assert_eq!(data.selector_tables, old.selector_tables);
    let bank = &data.banks[0];
    assert_eq!((bank.object_root, bank.sequence_root), (model::BATTLE_OBJECT_ROOT, model::BATTLE_SEQUENCE_ROOT));
    assert_eq!(bank.lists.len(), 17);
    assert_eq!(bank.lists.iter().map(|list| list.objects.len()).sum::<usize>(), 43);
    assert_eq!(bank.sequences.iter().map(|sequence| sequence.frames.len()).collect::<Vec<_>>(), [16, 0, 0]);
    let spans = bank.spans();
    assert_eq!((spans.len(), spans.iter().map(|(_, n, _)| n).sum::<usize>()), (22, 442));
    let current = source.manifest.resources.iter().find(|item| item.id == OBJ_ANIMATION_BANKS_ID).unwrap();
    let prior = old_manifest.resources.iter().find(|item| item.id == OBJ_ANIMATION_BANKS_ID).unwrap();
    assert_eq!(&current.sources[spans.len()..], prior.sources.as_slice());
    assert_eq!(parse_obj_animations(&fs::read(prepared.directory.join(OBJ_ANIMATION_BANKS_FILE)).unwrap()).unwrap(), *data);
    let mut forged = data.clone();
    forged.banks[0].sequences[0].frames[0].duration ^= 1;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    backing(&mut managed, &rom, &spans, true);
    let before = owner.obj_animation_stats().unwrap().battle_source_bytes;
    let mut edges = BTreeSet::new();
    for &(start, len, _) in &spans {
        for at in start..start + len as u32 {
            assert_eq!(managed.bus.read8(0x0c00_0000 + at), rom[at as usize]);
        }
        for at in [start - 1, start, start + len as u32 - 1, start + len as u32] { edges.insert(at); }
    }
    assert_eq!(owner.obj_animation_stats().unwrap().battle_source_bytes - before, 442);
    // Adjacent word-task banks must compose with the new bank and raw padding.
    for at in [0x411478usize, 0x41634c] {
        for i in at..at + 4 { managed.bus.rom[i] = !rom[i]; }
    }
    for &at in &edges {
        let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
        for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
            let i = at as usize;
            assert_eq!((RT_API.read16)(ptr, mirror + at), u32::from(u16::from_le_bytes(rom[i..i + 2].try_into().unwrap())), "read16 {at:x}");
            assert_eq!((RT_API.read32)(ptr, mirror + at), u32::from_le_bytes(rom[i..i + 4].try_into().unwrap()), "read32 {at:x}");
        }
    }
    for at in [0x411478usize, 0x41634c] { managed.bus.rom[at..at + 4].copy_from_slice(&rom[at..at + 4]); }
    backing(&mut managed, &rom, &spans, false);

    // Execute the actual sequence-zero resolution in the original context
    // initializer, through both interpreter and generated native dispatch.
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = (ENTRY..STOP).step_by(2).map(|address| Block {
        addr: address, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(address), address)],
    }).collect();
    let object = root.join("battle-obj-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let initial = raw.save_state().unwrap();
    for native in [false, true] {
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap();
            machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
            machine.cpu.regs[7] = CONTEXT;
            machine.cpu.regs[13] = STACK;
            machine.cpu.regs[15] = ENTRY;
            machine.cpu.cpsr = 0xbf;
            machine.bus.write32(CONTEXT + 4, 0x0800_0000 + bank.sequence_root);
        }
        backing(&mut managed, &rom, &spans, true);
        let before = owner.obj_animation_stats().unwrap().battle_source_bytes;
        for _ in 0..16 {
            if raw.cpu.regs[15] == STOP { break; }
            raw.bus.frame_ready = false;
            raw.step();
        }
        for _ in 0..16 {
            if managed.cpu.regs[15] == STOP { break; }
            managed.bus.frame_ready = false;
            if native { table.get(managed.cpu.regs[15] | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void); }
            else { managed.step(); }
        }
        assert_eq!((raw.cpu.regs[15], managed.cpu.regs[15]), (STOP, STOP));
        assert_eq!(owner.obj_animation_stats().unwrap().battle_source_bytes - before, 4);
        assert_eq!(raw.bus.read32(CONTEXT + 0x18), 0x0841_142c);
        backing(&mut managed, &rom, &spans, false);
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "native={native}");
    }

    // The host sprite shortcut must observe the same lists as guest memory.
    for machine in [&mut raw, &mut managed] {
        machine.bus.iwram[0x5f28..0x6d68].copy_from_slice(&rom[0xd968e8..0xd97728]);
    }
    backing(&mut managed, &rom, &spans, true);
    gba_core::mem::set_extra(120);
    let mut descriptors = 0;
    for (index, list) in bank.lists.iter().enumerate() {
        let mut oam = vec![0u8; 1024];
        for (i, object) in list.objects.iter().enumerate() {
            let [a0, a1, a2] = object.attributes();
            let attrs = [(a0 & 0xff00) | ((64 + i32::from(object.y)) as u16 & 255), (a1 & !511) | ((100 + i32::from(object.x)) as u16 & 511), a2];
            for (j, attr) in attrs.into_iter().enumerate() { oam[i * 8 + j * 2..i * 8 + j * 2 + 2].copy_from_slice(&attr.to_le_bytes()); }
        }
        let before = owner.obj_animation_stats().unwrap().battle_host_reads;
        for machine in [&mut raw, &mut managed] {
            machine.bus.frames = index as u64 + 1;
            machine.cpu.regs[..4].copy_from_slice(&[0x0800_0000 + list.source_offset, 100, 64, 0]);
            machine.cpu.regs[13] = STACK;
            machine.cpu.regs[15] = 0x0300_65a8;
            for (i, value) in [0, 0, 0, 0, 0, 0x0202_0000, 0, 0].into_iter().enumerate() { machine.bus.write32(STACK + i as u32 * 4, value); }
            machine.bus.ffta_sprites.enabled = true;
            machine.ffta_sprite_entry();
            machine.bus.ffta_sprites.publish(0x0300_0030, 0x0700_0000, 1024, true, &oam);
        }
        assert!(owner.obj_animation_stats().unwrap().battle_host_reads > before);
        assert_eq!(format!("{:?}", managed.bus.ffta_sprites.active), format!("{:?}", raw.bus.ffta_sprites.active));
        assert_eq!(managed.bus.ffta_sprites.active.as_ref().unwrap().len(), list.objects.len());
        descriptors += list.objects.len();
    }
    gba_core::mem::set_extra(0);
    assert_eq!(descriptors, 43);
    backing(&mut managed, &rom, &spans, false);
    let mut transfer_raw = crate::make_machine(rom.clone(), None);
    let mut transfer_managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut transfer_managed, &prepared.directory).unwrap();
    let transfer_initial = transfer_raw.save_state().unwrap();
    for kind in 0..3 {
        for machine in [&mut transfer_raw, &mut transfer_managed] {
            machine.load_state(&transfer_initial).unwrap();
            if kind == 0 {
                machine.bus.write32(0x0400_00d4, 0x0800_0000 + bank.object_root);
                machine.bus.write32(0x0400_00d8, 0x0203_2000);
                machine.bus.write16(0x0400_00dc, 32);
            } else {
                machine.cpu.regs[..3].copy_from_slice(&[0x0a00_0000 + bank.object_root, 0x0203_2400, if kind == 2 { 16 } else { 32 }]);
                machine.cpu.regs[15] = 0x0200_0000;
                machine.cpu.cpsr = 0xbf;
                machine.bus.write16(0x0200_0000, if kind == 2 { 0xdf0c } else { 0xdf0b });
            }
        }
        backing(&mut transfer_managed, &rom, &spans, true);
        let transfer_owner = transfer_managed.bus.rom_resources().unwrap().clone();
        let before = transfer_owner.obj_animation_stats().unwrap().battle_source_bytes;
        for machine in [&mut transfer_raw, &mut transfer_managed] {
            if kind == 0 { machine.bus.write16(0x0400_00de, 0x8000); } else { machine.step(); }
        }
        assert!(transfer_owner.obj_animation_stats().unwrap().battle_source_bytes > before);
        backing(&mut transfer_managed, &rom, &spans, false);
        assert_eq!(transfer_raw.save_state().unwrap(), transfer_managed.save_state().unwrap(), "transfer {kind}");
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
        "status":"PASS", "revision":[207,206], "prior_resources_preserved":122,
        "prior_banks_and_selector_tables_preserved":true, "bank_lists":17, "objects":43, "frames":16,
        "source_spans":22, "source_bytes":442, "source_and_mixed_boundaries":true,
        "original_interpreter_and_compiled_sequence_reader":true, "host_lists":17,
        "dma_cpuset_cpufastset_state_and_clock_equal":true,
        "missing_dataset_and_snapshot_owner_retained":true, "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
