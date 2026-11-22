//! EUR Soil Evidence copied halfwords and their original readers.
use super::*;
use gba_core::{capi::RT_API, rom_data::copied_halfword_lookups as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

fn setup(machine: &mut Machine, pc: u32) {
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[13] = 0x0300_7a00;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = 0x0800_0000 + pc;
    machine.cpu.cpsr = 0xbf;
}

#[test]
#[ignore = "exact EUR ROM and E239 predecessor; Wine only, Linux/USA pending"]
fn european_soil_copied_halfwords_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_SOIL_COPIED_HALFWORDS_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_SOIL_COPIED_HALFWORDS_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (239, 238, 130));
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (248, 247, 136));
    for prior in &old.resources {
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        if prior.id == COPIED_HALFWORD_LOOKUPS_ID {
            super::copied_halfword_lookups_tests::assert_previous_copied_halfwords_preserved(&previous, &prepared.directory, prior, current);
            assert_eq!(current.sources.len() - prior.sources.len(), 1);
        } else if prior.id == OBSERVED_TILEMAP_ROW_ID || prior.id == OBSERVED_TILEMAP_DESCRIPTOR_ID || prior.id == OBSERVED_TILEMAP_ID {
            super::soil_transfer_records_tests::assert_previous_soil_transfer_resource_preserved(
                &previous, &prepared.directory, prior, current);
        } else {
            assert_eq!(current, prior);
            assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
        }
    }
    let data = &source.copied_halfword_lookups.as_ref().unwrap().decoded;
    let selected = data.soil_evidence_table.as_ref().unwrap();
    assert_eq!(selected.source_offset, model::SOIL_EVIDENCE_OFFSET);
    assert_eq!(selected.values.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>(), rom[model::SOIL_EVIDENCE_OFFSET as usize..model::SOIL_EVIDENCE_OFFSET as usize + 8]);
    let spans = data.source_spans().unwrap();
    assert_eq!(spans.last().unwrap().0, model::SOIL_EVIDENCE_OFFSET);
    assert_eq!(parse_copied_halfword_lookups(&fs::read(prepared.directory.join(COPIED_HALFWORD_LOOKUPS_FILE)).unwrap()).unwrap(), *data);
    let mut invalid = data.clone(); invalid.soil_evidence_table = None; assert!(invalid.validate().is_err());
    let mut invalid = serde_json::to_value(data).unwrap(); invalid["soil_evidence_table"]["unexpected"] = 1.into();
    assert!(serde_json::from_value::<model::CopiedHalfwordLookups>(invalid).is_err());
    let mut forged = data.clone(); forged.soil_evidence_table.as_mut().unwrap().values[0] ^= 1;
    forged.validate().unwrap(); assert!(forged.verify_source(&rom).is_err());
    let bytes = encode_copied_halfword_lookups(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest.resources.iter_mut().find(|entry| entry.id == COPIED_HALFWORD_LOOKUPS_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files(); files.manifest = &altered_manifest; files.copied_halfword_lookups = Some((&forged, &bytes));
    assert!(source.validate_files(files).is_err(), "coherent file/hash edits must be rejected"); drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    let start = model::SOIL_EVIDENCE_OFFSET as usize;
    for at in start..start + 8 { managed.bus.rom[at] = !rom[at]; }
    let before = owner.copied_halfword_lookups_stats().unwrap();
    for at in start..start + 8 { assert_eq!(managed.bus.read8(0x0c00_0000 + at as u32), rom[at]); }
    let after = owner.copied_halfword_lookups_stats().unwrap();
    assert_eq!(after.table_bytes - before.table_bytes, 8); assert_eq!(after.soil_source_mask, 0xff);
    let owned = |at: usize| source.manifest.resources.iter().flat_map(|entry| &entry.sources)
        .any(|span| (span.offset as usize..span.offset as usize + span.length as usize).contains(&at));
    let mut changed = BTreeSet::new();
    for at in (start - 3..start).chain(start + 8..start + 12) {
        if !owned(at) { changed.insert(at); managed.bus.rom[at] = rom[at] ^ 0x5a; }
    }
    for edge in [start, start + 8] { for delta in [-3isize, -1, 0, 1] {
        let at = edge.checked_add_signed(delta).unwrap();
        let expected = |n: usize| if changed.contains(&(at + n)) { rom[at + n] ^ 0x5a } else { rom[at + n] };
        let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
        for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
            assert_eq!((RT_API.read16)(ptr, mirror + at as u32), u32::from(u16::from_le_bytes([expected(0), expected(1)])));
            assert_eq!((RT_API.read32)(ptr, mirror + at as u32), u32::from_le_bytes([expected(0), expected(1), expected(2), expected(3)]));
        }
    } }
    managed.bus.rom[start..start + 8].copy_from_slice(&rom[start..start + 8]);
    for at in changed { managed.bus.rom[at] = rom[at]; }

    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let mut addresses = BTreeSet::from([0x080f_042c]);
    addresses.extend((0x0814_7e0c..0x0814_7e1a).step_by(2));
    addresses.extend((0x080f_04d2..0x080f_04dc).step_by(2));
    let blocks = addresses.into_iter().map(|addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("soil-copied-halfwords-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let execute = |machine: &mut Machine, native: bool| {
        machine.bus.frame_ready = false;
        if native {
            let pc = machine.cpu.regs[15];
            table.get(pc | 1).unwrap_or_else(|| panic!("Missing generated reader {pc:08x}"))(
                &RT_API, machine as *mut Machine as *mut std::ffi::c_void);
        }
        else { machine.step(); }
    };
    let mut copy_cases = 0;
    for native in [false, true] {
        for machine in [&mut raw, &mut managed] { machine.load_state(&initial).unwrap(); setup(machine, 0x0f042c); }
        for at in model::SOIL_EVIDENCE_ROOT as usize..model::SOIL_EVIDENCE_ROOT as usize + 4 { managed.bus.rom[at] = !rom[at]; }
        let before = owner.thumb_pc_literals_stats().unwrap().source_bytes;
        execute(&mut raw, false); execute(&mut managed, native);
        assert_eq!(owner.thumb_pc_literals_stats().unwrap().source_bytes - before, 4);
        assert_eq!((raw.cpu.regs[1], managed.cpu.regs[1]), (0x0800_0000 + model::SOIL_EVIDENCE_OFFSET, 0x0800_0000 + model::SOIL_EVIDENCE_OFFSET));
        managed.bus.rom[model::SOIL_EVIDENCE_ROOT as usize..model::SOIL_EVIDENCE_ROOT as usize + 4].copy_from_slice(&rom[model::SOIL_EVIDENCE_ROOT as usize..model::SOIL_EVIDENCE_ROOT as usize + 4]);
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
        for selector in [0u32, 3, 4, 255] {
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&initial).unwrap(); setup(machine, 0x147e0c);
                machine.cpu.regs[1] = u32::MAX; machine.cpu.regs[2] = 7;
                machine.cpu.regs[3] = 0x0800_0000 + model::SOIL_EVIDENCE_OFFSET;
                machine.cpu.regs[4] = 0x0300_7a04; machine.cpu.regs[5] = 0x0203_3000;
                machine.bus.write8(0x0203_3000, selector as u8);
            }
            for at in start..start + 8 { managed.bus.rom[at] = !rom[at]; }
            let before = owner.copied_halfword_lookups_stats().unwrap().table_bytes;
            for _ in 0..128 { if raw.cpu.regs[15] == 0x0814_7e1a { break; } execute(&mut raw, false); }
            for _ in 0..128 { if managed.cpu.regs[15] == 0x0814_7e1a { break; } execute(&mut managed, native); }
            assert_eq!((raw.cpu.regs[15], managed.cpu.regs[15]), (0x0814_7e1a, 0x0814_7e1a));
            assert_eq!(owner.copied_halfword_lookups_stats().unwrap().table_bytes - before, 8);
            raw.cpu.regs[15] = 0x080f_04d2; managed.cpu.regs[15] = 0x080f_04d2;
            for _ in 0..8 { if raw.cpu.regs[15] == 0x080f_04dc { break; } execute(&mut raw, false); }
            for _ in 0..8 { if managed.cpu.regs[15] == 0x080f_04dc { break; } execute(&mut managed, native); }
            assert_eq!((raw.cpu.regs[15], managed.cpu.regs[15]), (0x080f_04dc, 0x080f_04dc));
            if selector < 4 { assert_eq!(raw.cpu.regs[0], u32::from(selected.values[selector as usize])); }
            managed.bus.rom[start..start + 8].copy_from_slice(&rom[start..start + 8]);
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "selector={selector}, native={native}");
            copy_cases += 1;
        }
    }
    assert_eq!(copy_cases, 8);

    let mut transfer_raw = crate::make_machine(rom.clone(), None);
    let mut transfer_managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut transfer_managed, &prepared.directory).unwrap();
    let transfer_initial = transfer_raw.save_state().unwrap();
    for kind in 0..3 {
        for machine in [&mut transfer_raw, &mut transfer_managed] {
            machine.load_state(&transfer_initial).unwrap();
            if kind == 0 {
                machine.bus.write32(0x0400_00d4, 0x0800_0000 + model::SOIL_EVIDENCE_OFFSET);
                machine.bus.write32(0x0400_00d8, 0x0203_2000); machine.bus.write16(0x0400_00dc, 4);
            } else {
                machine.cpu.regs[..3].copy_from_slice(&[0x0a00_0000 + model::SOIL_EVIDENCE_OFFSET, 0x0203_2400, if kind == 2 { 8 } else { 4 }]);
                machine.cpu.regs[15] = 0x0200_0000; machine.cpu.cpsr = 0xbf;
                machine.bus.write16(0x0200_0000, if kind == 2 { 0xdf0c } else { 0xdf0b });
            }
        }
        for at in start..start + 8 { transfer_managed.bus.rom[at] = !rom[at]; }
        let transfer_owner = transfer_managed.bus.rom_resources().unwrap().clone();
        let before = transfer_owner.copied_halfword_lookups_stats().unwrap().table_bytes;
        for machine in [&mut transfer_raw, &mut transfer_managed] { if kind == 0 { machine.bus.write16(0x0400_00de, 0x8000); } else { machine.step(); } }
        assert!(transfer_owner.copied_halfword_lookups_stats().unwrap().table_bytes > before);
        transfer_managed.bus.rom[start..start + 8].copy_from_slice(&rom[start..start + 8]);
        assert_eq!(transfer_raw.save_state().unwrap(), transfer_managed.save_state().unwrap(), "transfer={kind}");
    }
    let state = managed.save_state().unwrap(); let unavailable = root.join("temporarily unavailable dataset");
    fs::rename(&prepared.directory, &unavailable).unwrap(); assert!(attach_from_directory(&mut managed, &prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner)); managed.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner)); fs::rename(&unavailable, &prepared.directory).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[248,247], "soil_tables":1, "halfwords":4, "source_bytes":8,
        "previous_tables_preserved":3, "previous_other_resources_unchanged":126, "existing_root_retained":true,
        "poisoned_mixed_boundaries_and_mirrors":true, "original_interpreter_generated_copy_index_cases":copy_cases,
        "outside_indices_preserve_original_stack":true, "dma_cpuset_cpufastset_state_and_clock_equal":true,
        "coherent_edit_rejected":true, "missing_dataset_and_snapshot_owner_retained":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
