//! EUR Firaga copied matrices: publication, original readers and transfer paths.
use super::*;
use gba_core::{capi::RT_API, rom_data::firaga_copied_matrices as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

fn source_spans() -> Vec<(usize, usize)> {
    model::LITERALS.into_iter().map(|(offset, _)| (offset as usize, 4))
        .chain(model::MATRICES.into_iter().map(|(offset, w, h)| (offset as usize, usize::from(w) * usize::from(h) * 2)))
        .collect()
}
fn set_backing(machine: &mut Machine, rom: &[u8], poison: bool) {
    for (start, len) in source_spans() {
        for at in start..start + len { machine.bus.rom[at] = if poison { !rom[at] } else { rom[at] }; }
    }
}

#[test]
#[ignore = "exact EUR ROM and E250 predecessor; Wine only, Linux/USA pending"]
fn european_firaga_copied_matrices_and_original_copy() {
    let root = PathBuf::from(std::env::var_os("FFTA_FIRAGA_COPIED_MATRICES_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_FIRAGA_COPIED_MATRICES_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (250, 249, 136));
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
            super::soil_sprite_lists_tests::assert_previous_obj_lists_preserved(&previous, &prepared.directory, prior, current);
            continue;
        }
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    let data = &source.firaga_copied_matrices.as_ref().unwrap().decoded;
    assert_eq!(data.matrices.iter().map(|m| (m.width, m.height, m.cells.len())).collect::<Vec<_>>(), vec![(18,7,126),(17,6,102)]);
    for (offset, bytes, _) in data.source_spans().unwrap() {
        assert_eq!(bytes, rom[offset as usize..offset as usize + bytes.len()]);
    }
    assert_eq!(parse_firaga_copied_matrices(&fs::read(prepared.directory.join(FIRAGA_COPIED_MATRICES_FILE)).unwrap()).unwrap(), *data);
    let mut invalid = data.clone(); invalid.matrices[0].width += 1; assert!(invalid.validate().is_err());
    let mut forged = data.clone(); forged.matrices[0].cells[0] ^= 1; forged.validate().unwrap(); assert!(forged.verify_source(&rom).is_err());
    let bytes = encode_firaga_copied_matrices(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut changed_manifest = dataset.manifest.clone();
    changed_manifest.resources.iter_mut().find(|r| r.id == FIRAGA_COPIED_MATRICES_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files(); files.manifest = &changed_manifest; files.firaga_copied_matrices = Some((&forged, &bytes));
    assert!(source.validate_files(files).is_err(), "coherent file/hash edit must be rejected");
    drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None);
    raw.bus.iwram[0x5ee8..0x5f28].copy_from_slice(&rom[0xd968a8..0xd968e8]);
    let pcs = [0x0802_dde0,0x0802_dde2,0x0802_dfa8,0x0802_dfc0];
    let mut visited: BTreeSet<u32> = pcs.into_iter().collect();
    visited.extend((0x0300_5ee8..0x0300_5f28).step_by(2));
    let view = View { rom: &rom, ewram: None, iwram: Some(&raw.bus.iwram), bios: None };
    let blocks = visited.into_iter().map(|addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("firaga-copy-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    set_backing(&mut managed, &rom, true);
    for (start, len) in source_spans() { for at in start..start + len {
        for mirror in [0x0800_0000,0x0a00_0000,0x0c00_0000] { assert_eq!(managed.bus.read8(mirror + at as u32), rom[at]); }
    } }
    let owned = |at: usize| source.manifest.resources.iter().flat_map(|r| &r.sources)
        .any(|s| (s.offset as usize..s.offset as usize+s.length as usize).contains(&at));
    let mut changed = BTreeSet::new();
    for (start,len) in source_spans() { for edge in [start,start+len] { for at in edge.saturating_sub(3)..edge+3 {
        if !owned(at) { changed.insert(at); managed.bus.rom[at] = rom[at] ^ 0x5a; }
    } } }
    for (start,len) in source_spans() { for edge in [start,start+len] { for delta in [-3isize,-1,0,1] {
        let at = edge.checked_add_signed(delta).unwrap();
        let byte = |i: usize| if changed.contains(&(at+i)) { rom[at+i] ^ 0x5a } else { rom[at+i] };
        let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
        for mirror in [0x0800_0000,0x0a00_0000,0x0c00_0000] {
            assert_eq!((RT_API.read16)(ptr,mirror+at as u32),u32::from(u16::from_le_bytes([byte(0),byte(1)])));
            assert_eq!((RT_API.read32)(ptr,mirror+at as u32),u32::from_le_bytes([byte(0),byte(1),byte(2),byte(3)]));
        }
    } } }
    for at in changed { managed.bus.rom[at] = rom[at]; }
    set_backing(&mut managed,&rom,false);
    let mut roots = 0;
    for native in [false,true] { for pc in pcs {
        for machine in [&mut raw,&mut managed] {
            machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
            machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000+i as u32);
            machine.cpu.regs[13] = 0x0300_7c00; machine.cpu.regs[15] = pc; machine.cpu.cpsr = 0xbf;
        }
        set_backing(&mut managed,&rom,true);
        // The generated block may combine these adjacent literal loads.
        let stop = if pc == 0x0802_dde0 { pc + 4 } else { pc + 2 };
        for (machine,use_native) in [(&mut raw,false),(&mut managed,native)] {
            let mut steps = 0;
            while machine.cpu.regs[15] != stop {
                assert!(steps < 8, "literal reader escaped at {:08x}", machine.cpu.regs[15]);
                machine.bus.frame_ready = false;
                if use_native { table.get(machine.cpu.regs[15]|1).unwrap()(&RT_API,machine as *mut Machine as *mut std::ffi::c_void); }
                else { machine.step(); }
                steps += 1;
            }
        }
        set_backing(&mut managed,&rom,false);
        assert!(raw.save_state().unwrap() == managed.save_state().unwrap(),"literal pc={pc:08x} native={native}"); roots += 1;
    } }
    let mut copies = 0;
    for (group,(offset,width,height)) in model::MATRICES.into_iter().enumerate() {
        let len = usize::from(width)*usize::from(height)*2;
        for machine in [&mut raw,&mut managed] {
            machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
            machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000+i as u32);
            machine.cpu.regs[0]=0x0201_0000; machine.cpu.regs[1]=0x0800_0000+offset;
            machine.cpu.regs[2]=len as u32; machine.cpu.regs[13]=0x0300_7c00;
            machine.cpu.regs[14]=0x0800_0301; machine.cpu.regs[15]=0x0300_5ee8; machine.cpu.cpsr=0xbf;
        }
        set_backing(&mut managed,&rom,true);
        let before=owner.firaga_copied_matrices_stats().unwrap().matrix_bytes[group];
        for (machine,native) in [(&mut raw,false),(&mut managed,true)] {
            let mut steps=0;
            while machine.cpu.regs[15]!=0x0800_0300 {
                assert!(steps<3000,"copy did not return"); machine.bus.frame_ready=false;
                if native { table.get(machine.cpu.regs[15]|1).unwrap()(&RT_API,machine as *mut Machine as *mut std::ffi::c_void); }
                else { machine.step(); }
                steps+=1;
            }
            assert_eq!(&machine.bus.ewram[0x10000..0x10000+len],&rom[offset as usize..offset as usize+len]);
        }
        assert_eq!(owner.firaga_copied_matrices_stats().unwrap().matrix_bytes[group]-before,len as u64);
        set_backing(&mut managed,&rom,false);
        assert!(raw.save_state().unwrap()==managed.save_state().unwrap(),"original copy group={group}"); copies+=1;
    }
    let mut transfer_raw=crate::make_machine(rom.clone(),None);
    let mut transfer_managed=crate::make_machine(rom.clone(),None);
    attach_from_directory(&mut transfer_managed,&prepared.directory).unwrap();
    let transfer_initial=transfer_raw.save_state().unwrap(); let mut transfers=0;
    for (group,(offset,width,height)) in model::MATRICES.into_iter().enumerate() { for kind in 0..2 {
        let count=(usize::from(width)*usize::from(height)) as u32;
        for machine in [&mut transfer_raw,&mut transfer_managed] {
            machine.load_state(&transfer_initial).unwrap();
            if kind==0 { machine.bus.write32(0x0400_00d4,0x0800_0000+offset); machine.bus.write32(0x0400_00d8,0x0203_2000); machine.bus.write16(0x0400_00dc,count as u16); }
            else { machine.cpu.regs[..3].copy_from_slice(&[0x0a00_0000+offset,0x0203_2400,count]); machine.cpu.regs[15]=0x0200_0000; machine.cpu.cpsr=0xbf; machine.bus.write16(0x0200_0000,0xdf0b); }
        }
        set_backing(&mut transfer_managed,&rom,true);
        let transfer_owner=transfer_managed.bus.rom_resources().unwrap().clone();
        let before=transfer_owner.firaga_copied_matrices_stats().unwrap().matrix_bytes[group];
        for machine in [&mut transfer_raw,&mut transfer_managed] { if kind==0 { machine.bus.write16(0x0400_00de,0x8000); } else { machine.step(); } }
        assert!(transfer_owner.firaga_copied_matrices_stats().unwrap().matrix_bytes[group]>before);
        set_backing(&mut transfer_managed,&rom,false);
        assert!(transfer_raw.save_state().unwrap()==transfer_managed.save_state().unwrap(),"group={group} kind={kind}"); transfers+=1;
    } }
    let state=managed.save_state().unwrap(); let unavailable=root.join("temporarily unavailable dataset");
    fs::rename(&prepared.directory,&unavailable).unwrap(); assert!(attach_from_directory(&mut managed,&prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(),&owner)); managed.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(),&owner)); fs::rename(&unavailable,&prepared.directory).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS","revision":[257,256],"resources":142,"previous_other_resources_unchanged":135,"previous_obj_lists_preserved":true,
        "matrix_source_bytes":456,"literal_source_bytes":16,"poisoned_mirrors_and_mixed_boundaries":true,
        "original_interpreter_generated_literal_cases":roots,"original_interpreter_generated_copy_cases":copies,
        "dma_cpuset_cases":transfers,"full_state_clock_equal":true,"coherent_edit_rejected":true,
        "missing_dataset_and_snapshot_owner_retained":true,"Linux":"pending","USA":"pending",
        "native_windows":"pending","performance_measurements":false
    })).unwrap()).unwrap();
}
