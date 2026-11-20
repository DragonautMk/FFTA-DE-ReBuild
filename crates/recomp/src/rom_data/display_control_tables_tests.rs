//! EUR display-control table publication and original runtime consumption.
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::display_control_tables::{Catalog, DisplayControlTables, SOURCE_OFFSET},
    Bus, Machine,
};
use recomp_core::analyze::{Analysis, Block, View};
use std::sync::Arc;

fn clear_audio(machine: &mut Machine) {
    machine.bus.audio_buf.clear();
    machine.bus.psg_tap.clear();
    machine.bus.fifo_tap.iter_mut().for_each(Vec::clear);
    machine.bus.hle_tap.clear();
}

fn configure(machine: &mut Machine, case: usize) -> u32 {
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.cpsr = 0xbf;
    match case {
        0 | 1 => {
            machine.cpu.regs[1] = u32::MAX;
            machine.cpu.regs[2] = 7;
            machine.cpu.regs[3] = 0x0800_0000 + SOURCE_OFFSET + 8 * case as u32;
            machine.cpu.regs[4] = 0x0201_0000;
            machine.cpu.regs[15] = 0x0814_7e0c;
            0x0814_7e1a
        }
        2 | 3 => {
            machine.cpu.regs[1] = 0x0201_0000;
            machine.cpu.regs[15] = if case == 2 { 0x0801_f478 } else { 0x0801_f52a };
            if case == 2 {
                0x0801_f482
            } else {
                0x0801_f534
            }
        }
        _ => unreachable!(),
    }
}

#[test]
#[ignore = "exact EUR ROM, E170 predecessor, retained ordinary state/input and guarded translation; Wine only"]
fn european_display_control_tables_publication_and_ordinary_consumption() {
    crate::configure_headless_widescreen();
    let root = PathBuf::from(std::env::var_os("FFTA_DISPLAY_CONTROL_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_DISPLAY_CONTROL_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let state = fs::read(std::env::var_os("FFTA_STATE_TEST_STATE").unwrap()).unwrap();
    let input_path = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_INPUT").unwrap());
    let input = crate::InputScript::load(input_path.to_str().unwrap()).unwrap();
    let guarded = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap());
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    assert_eq!(
        gba_core::ffta_rom::digest(&state),
        "0e35923cf10c3ea4fe5eef23f367255a721057305c5d2ae6b8114bf0ee6f032e"
    );
    assert!((0..12).all(|f| input.keys_at(f) == 0x3ff && input.keys_at(971 + f) == 0x3ff));

    let old: Manifest =
        toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (
            old.extractor_revision,
            old.decoder_revision,
            old.resources.len()
        ),
        (170, 169, 113)
    );
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision,
            source.manifest.resources.len()
        ),
        (171, 170, 114)
    );
    let prepared =
        preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(
        preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    for prior in &old.resources {
        let current = source
            .manifest
            .resources
            .iter()
            .find(|r| r.id == prior.id)
            .unwrap();
        assert_eq!(current, prior);
        assert_eq!(
            fs::read(prepared.directory.join(&current.path)).unwrap(),
            fs::read(previous.join(&prior.path)).unwrap()
        );
    }
    let catalog = source.display_control_tables.as_ref().unwrap();
    assert_eq!(catalog.sources.len(), 3);
    let model: DisplayControlTables =
        serde_json::from_slice(&serde_json::to_vec(&catalog.decoded).unwrap()).unwrap();
    assert_eq!(
        model
            .source_spans()
            .unwrap()
            .iter()
            .map(|(_, bytes, _)| bytes.len())
            .sum::<usize>(),
        32
    );
    let mut invalid = model.clone();
    invalid.iwram_pointers[0] = 0x0200_0000;
    assert!(invalid.validate().is_err());
    let mut overlap = old.resources.clone();
    let mut partial = catalog.sources[0].clone();
    partial.length = 1;
    overlap[0].sources.push(partial);
    assert!(Catalog::from_rom(&rom, &overlap).is_err());
    let mut decoded = read_dataset(&prepared.directory).unwrap();
    source.validate_files(decoded.files()).unwrap();
    decoded
        .display_control_tables
        .as_mut()
        .unwrap()
        .first_halfwords[0] ^= 1;
    let changed =
        encode_display_control_tables(decoded.display_control_tables.as_ref().unwrap()).unwrap();
    decoded
        .manifest
        .resources
        .iter_mut()
        .find(|r| r.id == DISPLAY_CONTROL_TABLES_ID)
        .unwrap()
        .sha256 = gba_core::ffta_rom::digest(&changed);
    decoded.display_control_tables_bytes = Some(changed);
    assert!(source.validate_files(decoded.files()).is_err());

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let file = prepared.directory.join(DISPLAY_CONTROL_TABLES_FILE);
    let saved_file = fs::read(&file).unwrap();
    fs::write(&file, b"{}").unwrap();
    assert!(attach_from_directory(&mut managed, &prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    fs::write(&file, saved_file).unwrap();
    let start = SOURCE_OFFSET as usize;
    let poison = |m: &mut Machine| {
        for i in start..start + 32 {
            m.bus.rom[i] = rom[i] ^ 0xff;
        }
    };
    let restore =
        |m: &mut Machine| m.bus.rom[start..start + 32].copy_from_slice(&rom[start..start + 32]);
    poison(&mut managed);
    let before = owner.display_control_tables_stats().unwrap();
    for i in 0..32 {
        assert_eq!(
            managed.bus.read8(0x0800_0000 + SOURCE_OFFSET + i),
            rom[start + i as usize]
        );
    }
    let after = owner.display_control_tables_stats().unwrap();
    assert_eq!(
        (
            after.halfword_bytes - before.halfword_bytes,
            after.pointer_bytes - before.pointer_bytes
        ),
        (16, 16)
    );
    let raw_ptr = &mut raw as *mut Machine as *mut std::ffi::c_void;
    let managed_ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for mirror in [0x0800_0000u32, 0x0a00_0000, 0x0c00_0000] {
        for offset in [
            SOURCE_OFFSET - 1,
            SOURCE_OFFSET,
            SOURCE_OFFSET + 7,
            SOURCE_OFFSET + 15,
            SOURCE_OFFSET + 31,
        ] {
            assert_eq!(
                (RT_API.read8)(raw_ptr, mirror + offset),
                (RT_API.read8)(managed_ptr, mirror + offset)
            );
            assert_eq!(
                (RT_API.read16)(raw_ptr, mirror + offset),
                (RT_API.read16)(managed_ptr, mirror + offset)
            );
            assert_eq!(
                (RT_API.read32)(raw_ptr, mirror + offset),
                (RT_API.read32)(managed_ptr, mirror + offset)
            );
        }
    }
    restore(&mut managed);
    let initial = raw.save_state().unwrap();
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: Some(&raw.bus.iwram),
        bios: None,
    };
    let blocks = [
        (0x0814_7e0c, 0x0814_7e1a),
        (0x0801_f478, 0x0801_f482),
        (0x0801_f52a, 0x0801_f534),
    ]
    .into_iter()
    .flat_map(|(a, b)| (a..b).step_by(2))
    .map(|addr| Block {
        addr,
        thumb: true,
        instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
    })
    .collect();
    let object = root.join("display-control-native.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_compiled, table) = crate::load_native(&object).unwrap();
    let mut prefix_cases = 0;
    for native in [false, true] {
        for case in 0..4 {
            raw.load_state(&initial).unwrap();
            managed.load_state(&initial).unwrap();
            let stop = configure(&mut raw, case);
            assert_eq!(configure(&mut managed, case), stop);
            poison(&mut managed);
            let before = owner.display_control_tables_stats().unwrap();
            for _ in 0..128 {
                if raw.cpu.regs[15] == stop {
                    break;
                }
                raw.bus.frame_ready = false;
                raw.step();
            }
            assert_eq!(raw.cpu.regs[15], stop);
            for _ in 0..128 {
                if managed.cpu.regs[15] == stop {
                    break;
                }
                managed.bus.frame_ready = false;
                if native {
                    table.get(managed.cpu.regs[15] | 1).unwrap()(
                        &RT_API,
                        &mut managed as *mut Machine as *mut std::ffi::c_void,
                    );
                } else {
                    managed.step();
                }
            }
            assert_eq!(managed.cpu.regs[15], stop);
            assert_eq!(
                owner.display_control_tables_stats().unwrap().source_bytes - before.source_bytes,
                if case < 2 { 8 } else { 16 }
            );
            restore(&mut managed);
            assert_eq!(
                raw.save_state().unwrap(),
                managed.save_state().unwrap(),
                "case {case}, compiled {native}"
            );
            prefix_cases += 1;
        }
    }

    let (_guarded_library, guarded_table) = crate::load_native(&guarded).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    for machine in [&mut raw, &mut managed] {
        machine.load_state(&state).unwrap();
        crate::arm_audio_hle(machine, None);
        machine.bus.tap_channels = true;
        clear_audio(machine);
    }
    assert_eq!(raw.bus.frames, 1151);
    assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
    let owner = managed.bus.rom_resources().unwrap().clone();
    let mut audio_nonempty = false;
    let mut dispatches = (0u64, 0u64);
    for frame in 0..12u64 {
        poison(&mut managed);
        let mut counts = [(0, 0); 2];
        for (arm, machine) in [&mut raw, &mut managed].into_iter().enumerate() {
            machine.bus.keys = input.keys_at(frame);
            input.apply_mouse(&mut machine.bus, frame).unwrap();
            let pointer = machine as *mut Machine as *mut std::ffi::c_void;
            counts[arm] =
                crate::run_frame_native(machine, &guarded_table, pointer, crate::FRAME_STEP_GUARD);
            assert_eq!(machine.bus.frames, 1152 + frame);
        }
        assert_eq!(counts[0], counts[1]);
        dispatches.0 += counts[0].0;
        dispatches.1 += counts[0].1;
        assert_eq!(raw.bus.clock, managed.bus.clock);
        assert_eq!(raw.bus.framebuffer, managed.bus.framebuffer);
        assert_eq!(raw.bus.wide_framebuffer, managed.bus.wide_framebuffer);
        assert_eq!(raw.bus.oam, managed.bus.oam);
        assert_eq!(raw.bus.audio_buf, managed.bus.audio_buf);
        assert_eq!(raw.bus.psg_tap, managed.bus.psg_tap);
        assert_eq!(raw.bus.fifo_tap, managed.bus.fifo_tap);
        assert!(raw.bus.hle_tap.iter().map(|x| x.to_bits()).eq(managed
            .bus
            .hle_tap
            .iter()
            .map(|x| x.to_bits())));
        audio_nonempty |= !raw.bus.audio_buf.is_empty();
        restore(&mut managed);
        assert_eq!(
            raw.save_state().unwrap(),
            managed.save_state().unwrap(),
            "ordinary frame {frame}"
        );
        clear_audio(&mut raw);
        clear_audio(&mut managed);
    }
    let usage = owner.display_control_tables_stats().unwrap();
    assert!(usage.halfword_bytes >= 16 && usage.pointer_bytes >= 16);
    assert!(audio_nonempty && dispatches.0 > 0);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[171,170], "resources":114,
        "prior_resources_preserved":old.resources.len(), "created_and_reused":true,
        "all_source_bytes_and_boundaries_checked":true, "coherent_edit_overlap_and_failed_attach_rejected":true,
        "original_prefix_cases":prefix_cases, "original_prefix_full_state_clock_equal":true,
        "ordinary_frames":12, "ordinary_halfword_bytes":usage.halfword_bytes, "ordinary_pointer_bytes":usage.pointer_bytes,
        "ordinary_state_clock_video_oam_dispatch_audio_equal_each_frame":true, "mixed_audio_nonempty":true,
        "managed_sources_inverted_during_execution":true,
        "limits":"Generated dispatch can fall back to interpretation; aggregate usage does not attribute each read. Live IWRAM structures, other callers and selector domains remain unchanged and unproved.",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
