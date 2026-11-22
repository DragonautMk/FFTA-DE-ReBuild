//! EUR compact Firaga OBJ bank: publication and original guest/host readers.
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::{firaga_compact_obj_bank as model, obj_lists},
    Bus, Machine,
};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

fn spans() -> [(usize, usize); 8] {
    [
        (model::CALLER_LITERAL as usize, 4),
        (model::DESCRIPTOR_ROOT as usize, 4),
        (model::SELECTED_POINTER as usize, 4),
        (model::BANK_ROOT as usize, 24),
        (model::SEQUENCE_ROOT as usize, 24),
        (obj_lists::FIRAGA_COMPACT_EMPTY_LISTS[0].0 as usize, 2),
        (obj_lists::FIRAGA_COMPACT_EMPTY_LISTS[1].0 as usize, 2),
        (obj_lists::FIRAGA_COMPACT_EMPTY_LISTS[2].0 as usize, 2),
    ]
}
fn set_backing(machine: &mut Machine, rom: &[u8], poison: bool) {
    for (start, len) in spans() {
        for at in start..start + len {
            machine.bus.rom[at] = if poison { !rom[at] } else { rom[at] };
        }
    }
}
fn setup_host(machine: &mut Machine, offset: u32) {
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs[..4].copy_from_slice(&[0x0800_0000 + offset, 100, 70, 0]);
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = 0x0800_1500;
    machine.cpu.cpsr = 0xbf;
    for (i, value) in [7, 2, 3, 0x0203_0010, 128, 0].into_iter().enumerate() {
        machine.bus.write32(0x0300_7c00 + i as u32 * 4, value);
    }
    for at in 0..1024 {
        machine.bus.write8(0x0203_0010 + at, 0x5a);
    }
}

#[test]
#[ignore = "exact EUR ROM and E253 predecessor; Wine only, Linux/USA pending"]
fn european_firaga_compact_obj_bank_and_empty_lists() {
    let root = PathBuf::from(std::env::var_os("FFTA_FIRAGA_COMPACT_OBJ_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_FIRAGA_COMPACT_OBJ_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let old: Manifest =
        toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (
            old.extractor_revision,
            old.decoder_revision,
            old.resources.len()
        ),
        (253, 252, 139)
    );
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared =
        preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(
        preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision,
            source.manifest.resources.len()
        ),
        (257, 256, 142)
    );
    for prior in &old.resources {
        if prior.id == THUMB_PC_LITERALS_ID {
            let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
            super::thumb_pc_literals_home_tests::assert_previous_thumb_literals_preserved(
                &previous, &prepared.directory, prior, current);
            continue;
        }
        let current = source
            .manifest
            .resources
            .iter()
            .find(|entry| entry.id == prior.id)
            .unwrap();
        if prior.id == OBJ_STATIC_LISTS_ID {
            super::soil_sprite_lists_tests::assert_previous_obj_lists_preserved(
                &previous,
                &prepared.directory,
                prior,
                current,
            );
            assert_eq!(current.sources.len(), prior.sources.len() + 5);
            continue;
        }
        assert_eq!(current, prior);
        assert_eq!(
            fs::read(prepared.directory.join(&current.path)).unwrap(),
            fs::read(previous.join(&prior.path)).unwrap()
        );
    }
    let data = &source.firaga_compact_obj_bank.as_ref().unwrap().decoded;
    assert_eq!((data.frames.len(), data.list_offsets.len()), (10, 10));
    for (offset, bytes, _) in data.source_spans().unwrap() {
        assert_eq!(bytes, rom[offset as usize..offset as usize + bytes.len()]);
    }
    assert_eq!(
        parse_firaga_compact_obj_bank(
            &fs::read(prepared.directory.join(FIRAGA_COMPACT_OBJ_BANK_FILE)).unwrap()
        )
        .unwrap(),
        *data
    );
    let mut invalid = data.clone();
    invalid.list_offsets[0] += 2;
    assert!(invalid.validate().is_err());
    let mut forged = data.clone();
    forged.frames[0].duration ^= 1;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());
    let bytes = encode_firaga_compact_obj_bank(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut changed_manifest = dataset.manifest.clone();
    changed_manifest
        .resources
        .iter_mut()
        .find(|r| r.id == FIRAGA_COMPACT_OBJ_BANK_ID)
        .unwrap()
        .sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files();
    files.manifest = &changed_manifest;
    files.firaga_compact_obj_bank = Some((&forged, &bytes));
    assert!(
        source.validate_files(files).is_err(),
        "coherent file/hash edit must be rejected"
    );
    drop(dataset);

    let pcs = [
        0x080e_b8f2,
        0x080e_b8fa,
        0x080d_99c4,
        0x080d_99d0,
        0x0801_0878,
        0x0801_087c,
        0x0801_0e6e,
        0x0801_0cc6,
        0x0801_0ea2,
    ];
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let blocks = pcs
        .into_iter()
        .map(|addr| Block {
            addr,
            thumb: true,
            instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
        })
        .collect();
    let object = root.join("firaga-compact-obj-native.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    set_backing(&mut managed, &rom, true);
    for (start, len) in spans() {
        for at in start..start + len {
            for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
                assert_eq!(managed.bus.read8(mirror + at as u32), rom[at]);
            }
        }
    }
    let owned = |at: usize| {
        source
            .manifest
            .resources
            .iter()
            .flat_map(|r| &r.sources)
            .any(|s| (s.offset as usize..s.offset as usize + s.length as usize).contains(&at))
    };
    let mut changed = BTreeSet::new();
    for (start, len) in spans() {
        for edge in [start, start + len] {
            for at in edge.saturating_sub(3)..edge + 3 {
                if !owned(at) {
                    changed.insert(at);
                    managed.bus.rom[at] = rom[at] ^ 0x5a;
                }
            }
        }
    }
    for (start, len) in spans() {
        for edge in [start, start + len] {
            for delta in [-3isize, -1, 0, 1] {
                let at = edge.checked_add_signed(delta).unwrap();
                let byte = |i: usize| {
                    if changed.contains(&(at + i)) {
                        rom[at + i] ^ 0x5a
                    } else {
                        rom[at + i]
                    }
                };
                let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
                for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
                    assert_eq!(
                        (RT_API.read16)(ptr, mirror + at as u32),
                        u32::from(u16::from_le_bytes([byte(0), byte(1)]))
                    );
                    assert_eq!(
                        (RT_API.read32)(ptr, mirror + at as u32),
                        u32::from_le_bytes([byte(0), byte(1), byte(2), byte(3)])
                    );
                }
            }
        }
    }
    for at in changed {
        managed.bus.rom[at] = rom[at];
    }
    set_backing(&mut managed, &rom, false);
    let mut cases = vec![
        (0x080e_b8f2, 0usize),
        (0x080e_b8fa, 0usize),
        (0x080d_99c4, 0usize),
        (0x080d_99d0, 0usize),
        (0x0801_0878, 0usize),
        (0x0801_087c, 0usize),
    ];
    for i in [0usize, 5, 9] {
        for pc in [0x0801_0e6e, 0x0801_0cc6, 0x0801_0ea2] {
            cases.push((pc, i));
        }
    }
    let mut original_cases = 0;
    for native in [false, true] {
        for &(pc, i) in &cases {
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&initial).unwrap();
                machine.bus.ffta_sprites.enabled = false;
                machine.cpu.regs = std::array::from_fn(|j| 0x1100_0000 + j as u32);
                match pc {
                    0x080e_b8fa => machine.cpu.regs[2] = 0x0800_0000 + model::SELECTED_POINTER,
                    0x080d_99c4 | 0x080d_99d0 => {
                        machine.cpu.regs[0] = 0x0800_0000 + model::BANK_ROOT
                    }
                    0x0801_0878 => machine.cpu.regs[0] = 0x0800_0000 + model::SEQUENCE_ROOT,
                    0x0801_087c => machine.cpu.regs[1] = 0x0800_0000 + model::SEQUENCE_ROOT + 2,
                    0x0801_0e6e => {
                        machine.cpu.regs[1] = 0x0800_0000 + model::SEQUENCE_ROOT + 4 + 2 * i as u32
                    }
                    0x0801_0cc6 => {
                        machine.cpu.regs[0] = 0x0800_0000 + model::SEQUENCE_ROOT + 5 + 2 * i as u32
                    }
                    0x0801_0ea2 => {
                        machine.cpu.regs[0] = 0x0800_0000 + model::BANK_ROOT + 4 + 2 * i as u32
                    }
                    _ => {}
                }
                machine.cpu.regs[13] = 0x0300_7c00;
                machine.cpu.regs[15] = pc;
                machine.cpu.cpsr = 0xbf;
            }
            set_backing(&mut managed, &rom, true);
            raw.bus.frame_ready = false;
            raw.step();
            managed.bus.frame_ready = false;
            if native {
                table.get(pc | 1).unwrap()(
                    &RT_API,
                    &mut managed as *mut Machine as *mut std::ffi::c_void,
                );
            } else {
                managed.step();
            }
            assert_eq!(raw.cpu.regs[15], pc + 2);
            assert_eq!(managed.cpu.regs[15], pc + 2);
            set_backing(&mut managed, &rom, false);
            assert!(
                raw.save_state().unwrap() == managed.save_state().unwrap(),
                "pc={pc:08x} index={i} native={native}"
            );
            original_cases += 1;
        }
    }
    gba_core::mem::set_extra(120);
    let mut host_cases = 0;
    for &(offset, _, _) in &obj_lists::FIRAGA_COMPACT_EMPTY_LISTS {
        let mut reference = crate::make_machine(rom.clone(), None);
        setup_host(&mut reference, offset);
        for _ in 0..20_000 {
            if reference.cpu.regs[15] == 0x0800_0300 {
                break;
            }
            reference.bus.frame_ready = false;
            reference.step();
        }
        assert_eq!(reference.cpu.regs[15], 0x0800_0300);
        let reference_oam: Vec<_> = (0..1024)
            .map(|at| reference.bus.read8(0x0203_0010 + at))
            .collect();
        let mut raw_host = crate::make_machine(rom.clone(), None);
        let mut typed_host = crate::make_machine(rom.clone(), None);
        attach_from_directory(&mut typed_host, &prepared.directory).unwrap();
        typed_host.bus.rom[offset as usize] = !rom[offset as usize];
        typed_host.bus.rom[offset as usize + 1] = !rom[offset as usize + 1];
        for machine in [&mut raw_host, &mut typed_host] {
            setup_host(machine, offset);
            machine.bus.ffta_sprites.enabled = true;
            machine.ffta_sprite_entry();
            machine
                .bus
                .ffta_sprites
                .publish(0x0203_0010, 0x0700_0000, 1024, true, &reference_oam);
        }
        assert_eq!(
            format!("{:?}", raw_host.bus.ffta_sprites.active),
            format!("{:?}", typed_host.bus.ffta_sprites.active)
        );
        assert_eq!(
            typed_host
                .bus
                .rom_resources()
                .unwrap()
                .obj_list_source_used(offset),
            Some((false, true))
        );
        host_cases += 1;
    }
    gba_core::mem::set_extra(0);
    let mut transfer_raw = crate::make_machine(rom.clone(), None);
    let mut transfer_managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut transfer_managed, &prepared.directory).unwrap();
    let transfer_initial = transfer_raw.save_state().unwrap();
    let mut transfers = 0;
    for (offset, len) in [spans()[0], spans()[3], spans()[4], spans()[5]] {
        for kind in 0..2 {
            for machine in [&mut transfer_raw, &mut transfer_managed] {
                machine.load_state(&transfer_initial).unwrap();
                if kind == 0 {
                    machine
                        .bus
                        .write32(0x0400_00d4, 0x0800_0000 + offset as u32);
                    machine.bus.write32(0x0400_00d8, 0x0203_2000);
                    machine.bus.write16(0x0400_00dc, (len / 2) as u16);
                } else {
                    machine.cpu.regs[..3].copy_from_slice(&[
                        0x0a00_0000 + offset as u32,
                        0x0203_2400,
                        (len / 2) as u32,
                    ]);
                    machine.cpu.regs[15] = 0x0200_0000;
                    machine.cpu.cpsr = 0xbf;
                    machine.bus.write16(0x0200_0000, 0xdf0b);
                }
            }
            set_backing(&mut transfer_managed, &rom, true);
            for machine in [&mut transfer_raw, &mut transfer_managed] {
                if kind == 0 {
                    machine.bus.write16(0x0400_00de, 0x8000);
                } else {
                    machine.step();
                }
            }
            set_backing(&mut transfer_managed, &rom, false);
            assert!(
                transfer_raw.save_state().unwrap() == transfer_managed.save_state().unwrap(),
                "offset={offset:x} kind={kind}"
            );
            transfers += 1;
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
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS","revision":[257,256],"resources":142,"previous_other_resources_unchanged":138,
        "previous_obj_lists_preserved":true,"new_source_bytes":66,"frames":10,"empty_lists":3,
        "poisoned_mirrors_and_mixed_boundaries":true,"original_interpreter_generated_reader_cases":original_cases,
        "empty_list_host_comparisons":host_cases,"dma_cpuset_cases":transfers,"full_state_clock_equal":true,
        "coherent_edit_rejected":true,"missing_dataset_and_snapshot_owner_retained":true,
        "Linux":"pending","USA":"pending","native_windows":"pending","performance_measurements":false
    })).unwrap()).unwrap();
}
