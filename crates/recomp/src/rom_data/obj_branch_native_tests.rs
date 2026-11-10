//! Translated-original draw and state reload checks for EUR branch OBJ lists.
use super::*;
use gba_core::{capi::RT_API, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::collections::BTreeSet;

const ROOTS: [(u32, usize); 3] = [(0x397e22, 14), (0x399724, 8), (0x3add94, 14)];
const STACK: u32 = 0x0300_7c00;
const RETURN: u32 = 0x0800_0300;
const BUFFER: u32 = 0x0203_0010;

fn setup(machine: &mut Machine, offset: u32, first: bool) {
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs[..4].copy_from_slice(&[
        0x0800_0000 + offset,
        100,
        70,
        if first { 0x3000_0400 } else { 7 },
    ]);
    machine.cpu.regs[13] = STACK;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = if first { 0x0800_1500 } else { 0x0800_170c };
    machine.cpu.cpsr = 0xbf;
    let args: &[u32] = if first {
        &[7, 2, 3, BUFFER, 128, 0]
    } else {
        &[2, 3, BUFFER, 128, 0]
    };
    for (i, value) in args.iter().enumerate() {
        machine.bus.write32(STACK + 4 * i as u32, *value);
    }
    for at in 0..128 * 8 {
        machine.bus.write8(BUFFER + at, 0x5a);
    }
}

#[test]
#[ignore = "exact EUR ROM, E133 dataset and TinyCC; Windows executable under Wine"]
fn european_branch_lists_translated_draw_and_state_reload() {
    let root = PathBuf::from(std::env::var_os("FFTA_BRANCH_NATIVE_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_BRANCH_NATIVE_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let manifest: Manifest =
        toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (133, 132)
    );
    let mut trace = crate::make_machine(rom.clone(), None);
    let initial = trace.save_state().unwrap();
    let mut visited = BTreeSet::new();
    for (offset, _) in ROOTS {
        for first in [true, false] {
            trace.load_state(&initial).unwrap();
            setup(&mut trace, offset, first);
            for _ in 0..20_000 {
                visited.insert((trace.cpu.regs[15], trace.cpu.thumb()));
                trace.bus.frame_ready = false;
                trace.step();
                if trace.cpu.regs[15] == RETURN {
                    break;
                }
            }
            assert_eq!(trace.cpu.regs[15], RETURN);
        }
    }
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: Some(&trace.bus.iwram),
        bios: None,
    };
    let blocks = visited
        .iter()
        .map(|&(address, thumb)| {
            let first = if thumb {
                armv4t::decode_thumb(view.read16(address), address)
            } else {
                armv4t::decode_arm(view.read32(address), address)
            };
            let mut instrs = vec![first];
            if thumb {
                let next = armv4t::decode_thumb(view.read16(address + 2), address + 2);
                if armv4t::fuse_thumb_bl(&instrs[0], &next).is_some() {
                    instrs.push(next);
                }
            }
            Block {
                addr: address,
                thumb,
                instrs,
            }
        })
        .collect();
    let object = root.join("branch-lists-native.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().expect("TinyCC required"),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut native = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut native, &dataset).unwrap();
    let owner = native.bus.rom_resources().unwrap().clone();
    for (offset, length) in ROOTS {
        for first in [true, false] {
            raw.load_state(&initial).unwrap();
            native.load_state(&initial).unwrap();
            setup(&mut raw, offset, first);
            setup(&mut native, offset, first);
            native.bus.rom[offset as usize..offset as usize + length].fill(0xcd);
            let before = owner.obj_list_stats().unwrap().source_bytes;
            for _ in 0..20_000 {
                raw.bus.frame_ready = false;
                raw.step();
                if raw.cpu.regs[15] == RETURN {
                    break;
                }
            }
            assert_eq!(raw.cpu.regs[15], RETURN);
            let mut dispatches = 0;
            while native.cpu.regs[15] != RETURN {
                assert!(dispatches < 20_000, "translated draw did not return");
                native.bus.frame_ready = false;
                table
                    .get(native.cpu.regs[15] | u32::from(native.cpu.thumb()))
                    .expect("traced instruction compiled")(
                    &RT_API,
                    &mut native as *mut Machine as *mut std::ffi::c_void,
                );
                dispatches += 1;
            }
            assert!(owner.obj_list_stats().unwrap().source_bytes > before);
            assert_eq!(owner.obj_list_source_used(offset), Some((true, false)));
            native.bus.rom[offset as usize..offset as usize + length]
                .copy_from_slice(&rom[offset as usize..offset as usize + length]);
            assert_eq!(
                raw.save_state().unwrap(),
                native.save_state().unwrap(),
                "full state and clock for {offset:x}"
            );
        }
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status": "PASS", "revision": [133, 132], "translated_original_full_state_clock_cases": 6,
        "managed_source_after_state_reload": true, "ordinary_scene": "pending", "Linux": "pending",
        "USA": "pending", "native_windows": "pending", "performance_measurements": false
    })).unwrap()).unwrap();
}
