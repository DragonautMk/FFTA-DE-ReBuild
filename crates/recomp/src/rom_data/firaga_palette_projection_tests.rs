//! EUR caller-bounded palette projection and original Thumb reader.
use super::*;
use gba_core::{capi::RT_API, rom_data::ui_palettes as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::sync::Arc;

fn set_backing(machine: &mut Machine, rom: &[u8], poison: bool) {
    let start = model::FIRAGA_PALETTE_PROJECTION_OFFSET as usize;
    for at in start..start + model::FIRAGA_PALETTE_PROJECTION_BYTES {
        machine.bus.rom[at] = if poison { !rom[at] } else { rom[at] };
    }
}

#[test]
#[ignore = "exact EUR ROM and E273 predecessor; Wine only, Linux/USA pending"]
fn european_firaga_palette_projection_and_original_reader() {
    let root = PathBuf::from(std::env::var_os("FFTA_FIRAGA_PALETTE_PROJECTION_ROOT").unwrap());
    let previous =
        PathBuf::from(std::env::var_os("FFTA_FIRAGA_PALETTE_PROJECTION_PREVIOUS").unwrap());
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
        (273, 272, 146)
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
        (274, 273, 146)
    );
    for prior in &old.resources {
        let current = source
            .manifest
            .resources
            .iter()
            .find(|entry| entry.id == prior.id)
            .unwrap();
        if prior.id != UI_RAW_PALETTES_ID {
            assert_eq!(current, prior);
            assert_eq!(
                fs::read(previous.join(&prior.path)).unwrap(),
                fs::read(prepared.directory.join(&current.path)).unwrap()
            );
        } else {
            let old_data =
                parse_ui_palettes(&fs::read(previous.join(&prior.path)).unwrap()).unwrap();
            let mut new_data =
                parse_ui_palettes(&fs::read(prepared.directory.join(&current.path)).unwrap())
                    .unwrap();
            let index = new_data
                .banks
                .iter()
                .position(|bank| bank.source_offset == model::FIRAGA_PALETTE_PROJECTION_OFFSET)
                .unwrap();
            let added = new_data.banks.remove(index);
            assert_eq!(added.colors.len(), 16);
            assert_eq!(new_data, old_data);
            let mut metadata = current.clone();
            let added = metadata.sources.remove(index);
            assert_eq!(
                (added.offset, added.length),
                (model::FIRAGA_PALETTE_PROJECTION_OFFSET, 32)
            );
            metadata.sha256 = prior.sha256.clone();
            assert_eq!(&metadata, prior);
        }
    }
    let palettes = &source.ui_palettes.as_ref().unwrap().decoded;
    let bank = palettes
        .banks
        .iter()
        .find(|bank| bank.source_offset == model::FIRAGA_PALETTE_PROJECTION_OFFSET)
        .unwrap();
    let start = model::FIRAGA_PALETTE_PROJECTION_OFFSET as usize;
    assert_eq!(
        bank.source_bytes(),
        rom[start..start + model::FIRAGA_PALETTE_PROJECTION_BYTES]
    );
    let literal = source
        .manifest
        .resources
        .iter()
        .find(|entry| entry.id == THUMB_PC_LITERALS_ID)
        .unwrap();
    assert!(literal
        .sources
        .iter()
        .any(|span| span.offset == 0x1390c && span.length == 4));
    let mut forged = palettes.clone();
    forged
        .banks
        .iter_mut()
        .find(|bank| bank.source_offset == model::FIRAGA_PALETTE_PROJECTION_OFFSET)
        .unwrap()
        .colors[0] ^= 1;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());
    let forged_bytes = encode_ui_palettes(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered = dataset.manifest.clone();
    altered
        .resources
        .iter_mut()
        .find(|entry| entry.id == UI_RAW_PALETTES_ID)
        .unwrap()
        .sha256 = gba_core::ffta_rom::digest(&forged_bytes);
    let mut files = dataset.files();
    files.manifest = &altered;
    files.ui_palettes = Some((&forged, &forged_bytes));
    assert!(source.validate_files(files).is_err());
    drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    set_backing(&mut managed, &rom, true);
    for at in start..start + model::FIRAGA_PALETTE_PROJECTION_BYTES {
        for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
            assert_eq!(managed.bus.read8(mirror + at as u32), rom[at]);
        }
    }
    assert_eq!(
        owner.ui_palette_source_bytes(model::FIRAGA_PALETTE_PROJECTION_OFFSET),
        Some(96)
    );
    for edge in [start, start + model::FIRAGA_PALETTE_PROJECTION_BYTES] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap();
            let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
            assert_eq!(
                (RT_API.read16)(ptr, 0x0a00_0000 + at as u32),
                u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap()))
            );
            assert_eq!(
                (RT_API.read32)(ptr, 0x0800_0000 + at as u32),
                u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
            );
        }
    }
    set_backing(&mut managed, &rom, false);

    let (pc, stop) = (0x0801_38ec, 0x0801_390a);
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let blocks = (pc..stop)
        .step_by(2)
        .map(|addr| Block {
            addr,
            thumb: true,
            instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
        })
        .collect();
    let object = root.join("firaga-palette-projection-native.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut cases = 0;
    for native in [false, true] {
        for selector in 0..4u8 {
            for argument in 1..=4u32 {
                for machine in [&mut raw, &mut managed] {
                    machine.load_state(&initial).unwrap();
                    machine.bus.ffta_sprites.enabled = false;
                    machine.bus.write8(0x0200_2fc2, selector);
                    machine.cpu.regs = std::array::from_fn(|n| 0x1100_0000 + n as u32);
                    machine.cpu.regs[0] = argument;
                    machine.cpu.regs[13] = 0x0300_7c00;
                    machine.cpu.regs[15] = pc;
                    machine.cpu.cpsr = 0xbf;
                }
                set_backing(&mut managed, &rom, true);
                for (machine, use_native) in [(&mut raw, false), (&mut managed, native)] {
                    let mut steps = 0;
                    while machine.cpu.regs[15] != stop {
                        assert!(steps < 32);
                        machine.bus.frame_ready = false;
                        if use_native {
                            table.get(machine.cpu.regs[15] | 1).unwrap()(
                                &RT_API,
                                machine as *mut Machine as *mut std::ffi::c_void,
                            );
                        } else {
                            machine.step();
                        }
                        steps += 1;
                    }
                    let at = start + (selector as usize * 4 + argument as usize - 1) * 2;
                    assert_eq!(
                        machine.cpu.regs[0],
                        u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap()))
                    );
                }
                set_backing(&mut managed, &rom, false);
                assert_eq!(
                    raw.save_state().unwrap(),
                    managed.save_state().unwrap(),
                    "selector={selector} argument={argument} native={native}"
                );
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 32);
    // Outside the selected caller, zero wraps to a distant halfword; keep guest fallback.
    let fallback = start + 0x1fffe;
    assert!(source
        .manifest
        .resources
        .iter()
        .flat_map(|entry| &entry.sources)
        .all(|span| fallback + 2 <= span.offset as usize
            || fallback >= span.offset as usize + span.length as usize));
    for native in [false, true] {
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap();
            machine.bus.ffta_sprites.enabled = false;
            machine.bus.write8(0x0200_2fc2, 0);
            machine.cpu.regs = std::array::from_fn(|n| 0x1100_0000 + n as u32);
            machine.cpu.regs[0] = 0;
            machine.cpu.regs[13] = 0x0300_7c00;
            machine.cpu.regs[15] = pc;
            machine.cpu.cpsr = 0xbf;
            for at in fallback..fallback + 2 {
                machine.bus.rom[at] = rom[at] ^ 0x5a;
            }
        }
        set_backing(&mut managed, &rom, true);
        for (machine, use_native) in [(&mut raw, false), (&mut managed, native)] {
            let mut steps = 0;
            while machine.cpu.regs[15] != stop {
                assert!(steps < 32);
                machine.bus.frame_ready = false;
                if use_native {
                    table.get(machine.cpu.regs[15] | 1).unwrap()(
                        &RT_API,
                        machine as *mut Machine as *mut std::ffi::c_void,
                    );
                } else {
                    machine.step();
                }
                steps += 1;
            }
            assert_eq!(
                machine.cpu.regs[0],
                u32::from(u16::from_le_bytes([
                    rom[fallback] ^ 0x5a,
                    rom[fallback + 1] ^ 0x5a
                ]))
            );
        }
        for machine in [&mut raw, &mut managed] {
            machine.bus.rom[fallback..fallback + 2].copy_from_slice(&rom[fallback..fallback + 2]);
        }
        set_backing(&mut managed, &rom, false);
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
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
        "status":"PASS", "revision":[274,273], "resources":146, "new_source_bytes":32,
        "prior_other_resources_unchanged":145, "prior_representation_preserved":true, "root_remains_in_existing_owner":true,
        "poisoned_mirrors_and_mixed_boundaries":true, "original_interpreter_generated_cases":cases,
        "outside_selected_caller_falls_back":true, "full_state_clock_equal":true, "coherent_edit_rejected":true,
        "missing_dataset_and_snapshot_owner_retained":true, "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
