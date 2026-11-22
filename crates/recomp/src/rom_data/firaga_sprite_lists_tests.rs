//! EUR Firaga typed OBJ lists, guest reads and host consumption.
use super::*;
use gba_core::{capi::RT_API, rom_data::obj_lists as model, Bus, Machine};
use std::{collections::BTreeSet, sync::Arc};

const STACK: u32 = 0x0300_7c00;
const BUFFER: u32 = 0x0203_0010;
const RETURN: u32 = 0x0800_0300;

fn setup(machine: &mut Machine, offset: u32) {
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs[..4].copy_from_slice(&[0x0800_0000 + offset, 100, 70, 0]);
    machine.cpu.regs[13] = STACK;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = 0x0800_1500;
    machine.cpu.cpsr = 0xbf;
    for (i, value) in [7, 2, 3, BUFFER, 128, 0].into_iter().enumerate() {
        machine.bus.write32(STACK + i as u32 * 4, value);
    }
    for at in 0..1024 { machine.bus.write8(BUFFER + at, 0x5a); }
}

#[test]
#[ignore = "exact EUR ROM and E249 predecessor; Wine only, Linux/USA pending"]
fn european_firaga_sprite_lists_guest_and_host_consumption() {
    let root = PathBuf::from(std::env::var_os("FFTA_FIRAGA_SPRITE_LISTS_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_FIRAGA_SPRITE_LISTS_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (249, 248, 136));
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (257, 256, 142));
    for prior in &old.resources {
        if prior.id == THUMB_PC_LITERALS_ID {
            let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
            super::thumb_pc_literals_home_tests::assert_previous_thumb_literals_preserved(
                &previous, &prepared.directory, prior, current);
            continue;
        }
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        if prior.id == OBJ_STATIC_LISTS_ID {
            super::soil_sprite_lists_tests::assert_previous_obj_lists_preserved(
                &previous, &prepared.directory, prior, current);
            assert_eq!(current.sources.len() - prior.sources.len(), 50);
        } else {
            assert_eq!(current, prior);
            assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
        }
    }
    let data = &source.obj_lists.as_ref().unwrap().decoded;
    assert_eq!(data.lists.len(), 274);
    let mut total = 0;
    for (offset, count, _) in model::FIRAGA_SPRITE_LISTS {
        let list = data.lists.iter().find(|list| list.source_offset == offset).unwrap();
        assert_eq!(list.objects.len(), count);
        let bytes = list.source_bytes().unwrap();
        assert_eq!(bytes, rom[offset as usize..offset as usize + bytes.len()]);
        total += bytes.len();
    }
    assert_eq!(total, 2170);
    assert_eq!(parse_obj_lists(&fs::read(prepared.directory.join(OBJ_STATIC_LISTS_FILE)).unwrap()).unwrap(), *data);
    let mut bad = data.clone(); bad.lists.reverse(); assert!(bad.validate().is_err());
    let mut bad = serde_json::to_value(data).unwrap(); bad["lists"][0]["unexpected"] = 1.into();
    assert!(serde_json::from_value::<model::Lists>(bad).is_err());
    let mut forged = data.clone();
    let list = forged.lists.iter_mut().find(|list| list.source_offset == model::FIRAGA_SPRITE_LISTS[0].0).unwrap();
    list.objects[0].tile ^= 1;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());
    let bytes = encode_obj_lists(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest.resources.iter_mut().find(|entry| entry.id == OBJ_STATIC_LISTS_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files(); files.manifest = &altered_manifest; files.obj_lists = Some((&forged, &bytes));
    assert!(source.validate_files(files).is_err(), "coherent file/hash edits must be rejected");
    drop(dataset);

    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    for (offset, count, _) in model::FIRAGA_SPRITE_LISTS { for at in offset as usize..offset as usize + 2 + count * 6 { managed.bus.rom[at] = !rom[at]; } }
    let before = owner.obj_list_stats().unwrap().source_bytes;
    for (offset, count, _) in model::FIRAGA_SPRITE_LISTS {
        for at in offset as usize..offset as usize + 2 + count * 6 { assert_eq!(managed.bus.read8(0x0c00_0000 + at as u32), rom[at]); }
        assert_eq!(owner.obj_list_source_used(offset), Some((true, false)));
    }
    assert_eq!(owner.obj_list_stats().unwrap().source_bytes - before, 2170);
    let owned = |at: usize| source.manifest.resources.iter().flat_map(|entry| &entry.sources)
        .any(|span| (span.offset as usize..span.offset as usize + span.length as usize).contains(&at));
    let mut changed = BTreeSet::new();
    for (offset, count, _) in model::FIRAGA_SPRITE_LISTS {
        let start = offset as usize; let end = start + 2 + count * 6;
        for at in (start - 3..start).chain(end..end + 4) {
            if !owned(at) { changed.insert(at); managed.bus.rom[at] = rom[at] ^ 0x5a; }
        }
    }
    for (offset, count, _) in model::FIRAGA_SPRITE_LISTS {
        let start = offset as usize; let end = start + 2 + count * 6;
        for edge in [start, end] { for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap();
            let expected = |n: usize| if changed.contains(&(at + n)) { rom[at + n] ^ 0x5a } else { rom[at + n] };
            let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
            for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
                assert_eq!((RT_API.read16)(ptr, mirror + at as u32), u32::from(u16::from_le_bytes([expected(0), expected(1)])));
                assert_eq!((RT_API.read32)(ptr, mirror + at as u32), u32::from_le_bytes([expected(0), expected(1), expected(2), expected(3)]));
            }
        } }
    }
    for (offset, count, _) in model::FIRAGA_SPRITE_LISTS { let at = offset as usize; let end = at + 2 + count * 6; managed.bus.rom[at..end].copy_from_slice(&rom[at..end]); }
    for at in changed { managed.bus.rom[at] = rom[at]; }

    // Compare actual host descriptor decoding with original interpreter output
    // for representative one-, three- and twenty-two-object lists.
    gba_core::mem::set_extra(120);
    for index in [0usize, 4, 31] {
        let offset = model::FIRAGA_SPRITE_LISTS[index].0;
        let mut reference = crate::make_machine(rom.clone(), None);
        setup(&mut reference, offset);
        for _ in 0..20_000 {
            if reference.cpu.regs[15] == RETURN { break; }
            reference.bus.frame_ready = false;
            reference.step();
        }
        assert_eq!(reference.cpu.regs[15], RETURN);
        let oam: Vec<_> = (0..1024).map(|at| reference.bus.read8(BUFFER + at)).collect();
        let mut raw_host = crate::make_machine(rom.clone(), None);
        let mut typed_host = crate::make_machine(rom.clone(), None);
        attach_from_directory(&mut typed_host, &prepared.directory).unwrap();
        for (at, count, _) in model::FIRAGA_SPRITE_LISTS { for x in at as usize..at as usize + 2 + count * 6 { typed_host.bus.rom[x] = !rom[x]; } }
        for machine in [&mut raw_host, &mut typed_host] {
            setup(machine, offset);
            machine.bus.ffta_sprites.enabled = true;
            machine.ffta_sprite_entry();
            machine.bus.ffta_sprites.publish(BUFFER, 0x0700_0000, 1024, true, &oam);
        }
        assert!(raw_host.bus.ffta_sprites.active.is_some());
        assert_eq!(format!("{:?}", raw_host.bus.ffta_sprites.active), format!("{:?}", typed_host.bus.ffta_sprites.active));
        assert_eq!(typed_host.bus.rom_resources().unwrap().obj_list_source_used(offset), Some((false, true)));
    }
    gba_core::mem::set_extra(0);

    let mut transfer_raw = crate::make_machine(rom.clone(), None);
    let mut transfer_managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut transfer_managed, &prepared.directory).unwrap();
    let transfer_initial = transfer_raw.save_state().unwrap();
    let offset = model::FIRAGA_SPRITE_LISTS[0].0;
    for kind in 0..3 {
        for machine in [&mut transfer_raw, &mut transfer_managed] {
            machine.load_state(&transfer_initial).unwrap();
            if kind == 0 {
                machine.bus.write32(0x0400_00d4, 0x0800_0000 + offset);
                machine.bus.write32(0x0400_00d8, 0x0203_2000);
                machine.bus.write16(0x0400_00dc, 4);
            } else {
                machine.cpu.regs[..3].copy_from_slice(&[0x0a00_0000 + offset, 0x0203_2400, if kind == 2 { 8 } else { 4 }]);
                machine.cpu.regs[15] = 0x0200_0000; machine.cpu.cpsr = 0xbf;
                machine.bus.write16(0x0200_0000, if kind == 2 { 0xdf0c } else { 0xdf0b });
            }
        }
        for at in offset as usize..offset as usize + 8 { transfer_managed.bus.rom[at] = !rom[at]; }
        let transfer_owner = transfer_managed.bus.rom_resources().unwrap().clone();
        let before = transfer_owner.obj_list_stats().unwrap().source_bytes;
        for machine in [&mut transfer_raw, &mut transfer_managed] { if kind == 0 { machine.bus.write16(0x0400_00de, 0x8000); } else { machine.step(); } }
        assert!(transfer_owner.obj_list_stats().unwrap().source_bytes > before);
        transfer_managed.bus.rom[offset as usize..offset as usize + 8].copy_from_slice(&rom[offset as usize..offset as usize + 8]);
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
        "status":"PASS", "revision":[257,256], "resources":142, "previous_obj_lists_preserved":224,
        "previous_other_resources_unchanged":135, "firaga_lists":44, "firaga_objects":347, "firaga_source_bytes":2170,
        "poisoned_mixed_boundaries_and_mirrors":true, "original_interpreter_host_comparisons":3,
        "dma_cpuset_cpufastset_state_and_clock_equal":true, "coherent_edit_rejected":true,
        "missing_dataset_and_snapshot_owner_retained":true, "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
