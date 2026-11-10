//! Translated EUR context updater and OBJ draw against original execution.
use super::*;
use gba_core::{capi::RT_API, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::collections::BTreeSet;

const OBJECT_ROOT: u32 = 0x00b9_3ff0;
const SEQUENCE: u32 = 0x00b9_3f54;
const CONTEXT: u32 = 0x0202_0000;
const STACK: u32 = 0x0300_7c00;
const RETURN: u32 = 0x0800_0300;
const BUFFER: u32 = 0x0203_0010;

fn setup_update(machine: &mut Machine, frame: usize) {
    machine.bus.ffta_sprites.enabled = false;
    machine.bus.write32(CONTEXT, 0x0800_0000 + OBJECT_ROOT);
    machine.bus.write32(CONTEXT + 0x18, 0x0800_0000 + SEQUENCE);
    machine.bus.write16(CONTEXT + 0x0e, frame as u16);
    machine.bus.write16(CONTEXT + 0x10, 0);
    machine.bus.write8(CONTEXT + 0x1d, 1);
    machine.cpu.regs[0] = CONTEXT;
}
fn setup_draw(machine: &mut Machine) {
    let ptr = machine.bus.read32(CONTEXT + 0x14);
    machine.cpu.regs[..4].copy_from_slice(&[ptr, 100, 70, 7]);
    for (i, value) in [2, 3, BUFFER, 128, 0].iter().enumerate() {
        machine.bus.write32(STACK + 4 * i as u32, *value);
    }
    for i in 0..128 * 8 {
        machine.bus.write8(BUFFER + i, 0x5a);
    }
}
fn entry(machine: &mut Machine, pc: u32) {
    machine.cpu.regs[13] = STACK;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = pc;
    machine.cpu.cpsr = 0xbf;
}
fn original(machine: &mut Machine, pc: u32, visited: &mut BTreeSet<(u32, bool)>) {
    entry(machine, pc);
    for _ in 0..20_000 {
        visited.insert((machine.cpu.regs[15], machine.cpu.thumb()));
        machine.bus.frame_ready = false;
        machine.step();
        if machine.cpu.regs[15] == RETURN {
            return;
        }
    }
    panic!("original consumer did not return: {pc:x}");
}
fn translated(machine: &mut Machine, pc: u32, table: &crate::BlockTable) {
    entry(machine, pc);
    let mut dispatches = 0;
    while machine.cpu.regs[15] != RETURN {
        assert!(dispatches < 20_000, "translated consumer did not return");
        machine.bus.frame_ready = false;
        table
            .get(machine.cpu.regs[15] | u32::from(machine.cpu.thumb()))
            .expect("traced instruction compiled")(
            &RT_API,
            machine as *mut Machine as *mut std::ffi::c_void,
        );
        dispatches += 1;
    }
}

#[test]
#[ignore = "exact EUR ROM, E134 dataset and TinyCC; Windows executable under Wine"]
fn european_context_bank_translated_update_and_draw() {
    let root = PathBuf::from(std::env::var_os("FFTA_CONTEXT_NATIVE_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_CONTEXT_NATIVE_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let manifest: Manifest =
        toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (134, 133)
    );
    let entry = manifest
        .resources
        .iter()
        .find(|r| r.id == OBJ_ANIMATION_BANKS_ID)
        .unwrap();
    let banks = parse_obj_animations(&fs::read(dataset.join(&entry.path)).unwrap()).unwrap();
    let bank = banks
        .banks
        .iter()
        .find(|b| b.object_root == OBJECT_ROOT)
        .unwrap();
    assert_eq!((bank.lists.len(), bank.sequences[0].frames.len()), (21, 38));
    let spans = bank.spans();
    let mut trace = crate::make_machine(rom.clone(), None);
    let initial = trace.save_state().unwrap();
    let mut visited = BTreeSet::new();
    for frame in [0, 32, 37] {
        trace.load_state(&initial).unwrap();
        setup_update(&mut trace, frame);
        original(&mut trace, 0x080a_ccf4, &mut visited);
        setup_draw(&mut trace);
        original(&mut trace, 0x0800_170c, &mut visited);
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
    let object = root.join("context-bank-native.o");
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
    for frame in [0, 32, 37] {
        raw.load_state(&initial).unwrap();
        native.load_state(&initial).unwrap();
        for &(at, len, _) in &spans {
            native.bus.rom[at as usize..at as usize + len].fill(0xcd);
        }
        setup_update(&mut raw, frame);
        setup_update(&mut native, frame);
        let mut unused = BTreeSet::new();
        let before = owner.obj_animation_stats().unwrap().source_bytes;
        original(&mut raw, 0x080a_ccf4, &mut unused);
        translated(&mut native, 0x080a_ccf4, &table);
        assert!(owner.obj_animation_stats().unwrap().source_bytes > before);
        assert_eq!(
            raw.bus.read32(CONTEXT + 0x14),
            native.bus.read32(CONTEXT + 0x14)
        );
        setup_draw(&mut raw);
        setup_draw(&mut native);
        let before_draw = owner.obj_animation_stats().unwrap().source_bytes;
        original(&mut raw, 0x0800_170c, &mut unused);
        translated(&mut native, 0x0800_170c, &table);
        assert!(owner.obj_animation_stats().unwrap().source_bytes > before_draw);
        for &(at, len, _) in &spans {
            native.bus.rom[at as usize..at as usize + len]
                .copy_from_slice(&rom[at as usize..at as usize + len]);
        }
        assert_eq!(
            raw.save_state().unwrap(),
            native.save_state().unwrap(),
            "full state and clock frame {frame}"
        );
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS", "revision":[134,133], "translated_update_draw_full_state_clock_cases":3, "managed_source_after_state_reload":true, "ordinary_scene":"pending", "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false})).unwrap()).unwrap();
}
