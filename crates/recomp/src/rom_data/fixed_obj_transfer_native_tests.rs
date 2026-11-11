//! EUR fixed OBJ transfer through translated original queue and DMA code.
use super::*;
use gba_core::{capi::RT_API, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::collections::BTreeSet;

const SOURCE: u32 = 0x00b8_0b5c;
const LENGTH: usize = 0x8000;
const NODE: u32 = 0x0201_0000;
const STACK: u32 = 0x0300_7c00;
const RETURN: u32 = 0x0300_7f00;

fn setup(machine: &mut Machine) {
    machine.bus.ffta_sprites.enabled = false;
    machine.bus.iwram.fill(0);
    machine.bus.write32(0x0300_2bcc, 0);
    machine.cpu.regs[..4].copy_from_slice(&[NODE, 0x0800_0000 + SOURCE, 0x0601_0000, 0x4000]);
}

fn entry(machine: &mut Machine, pc: u32) {
    machine.cpu.regs[13] = STACK;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = pc;
    machine.cpu.cpsr = 0xbf;
}

fn original(machine: &mut Machine, pc: u32, visited: &mut BTreeSet<(u32, bool)>) {
    entry(machine, pc);
    for _ in 0..100_000 {
        visited.insert((machine.cpu.regs[15], machine.cpu.thumb()));
        machine.bus.frame_ready = false;
        machine.step();
        if machine.cpu.regs[15] == RETURN {
            return;
        }
    }
    panic!("original transfer routine did not return: {pc:x}");
}

fn translated(machine: &mut Machine, pc: u32, table: &crate::BlockTable) {
    entry(machine, pc);
    for _ in 0..100_000 {
        if machine.cpu.regs[15] == RETURN {
            return;
        }
        machine.bus.frame_ready = false;
        table
            .get(machine.cpu.regs[15] | u32::from(machine.cpu.thumb()))
            .expect("traced instruction compiled")(
            &RT_API,
            machine as *mut Machine as *mut std::ffi::c_void,
        );
    }
    panic!("translated transfer routine did not return: {pc:x}");
}

#[test]
#[ignore = "exact EUR ROM, E135 dataset and TinyCC; Windows executable under Wine"]
fn european_fixed_obj_transfer_translated_queue_and_dma() {
    let root = PathBuf::from(std::env::var_os("FFTA_FIXED_OBJ_NATIVE_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_FIXED_OBJ_NATIVE_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let manifest: Manifest =
        toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (135, 134)
    );
    let entry = manifest
        .resources
        .iter()
        .find(|r| r.id == UI_RAW_TRANSFERS_ID)
        .unwrap();
    let transfers = parse_ui_raw_transfers(&fs::read(dataset.join(&entry.path)).unwrap()).unwrap();
    assert_eq!(transfers.fixed_banks.len(), 1);
    assert_eq!(transfers.fixed_banks[0].bank_offset, SOURCE);
    assert_eq!(
        transfers.fixed_banks[0].source_bytes(),
        rom[SOURCE as usize..SOURCE as usize + LENGTH]
    );

    let mut trace = crate::make_machine(rom.clone(), None);
    let initial = trace.save_state().unwrap();
    let mut visited = BTreeSet::new();
    setup(&mut trace);
    original(&mut trace, 0x0800_0ad0, &mut visited);
    original(&mut trace, 0x0800_0800, &mut visited);
    assert_eq!(
        &trace.bus.vram[0x10000..0x18000],
        &rom[SOURCE as usize..SOURCE as usize + LENGTH]
    );
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
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
    let object = root.join("fixed-obj-native.o");
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
    raw.load_state(&initial).unwrap();
    native.load_state(&initial).unwrap();
    native.bus.rom[SOURCE as usize..SOURCE as usize + LENGTH].fill(0xcd);
    setup(&mut raw);
    setup(&mut native);
    let before = owner.ui_raw_transfers_stats().unwrap();
    let mut ignored = BTreeSet::new();
    original(&mut raw, 0x0800_0ad0, &mut ignored);
    translated(&mut native, 0x0800_0ad0, &table);
    assert_eq!(raw.bus.read32(NODE + 12), 0x8000_4000);
    assert_eq!(native.bus.read32(NODE + 12), 0x8000_4000);
    original(&mut raw, 0x0800_0800, &mut ignored);
    translated(&mut native, 0x0800_0800, &table);
    let after = owner.ui_raw_transfers_stats().unwrap();
    assert!(after.payload_bytes - before.payload_bytes >= LENGTH as u64);
    assert_eq!(
        (after.selector_bytes, after.banks_used, after.groups_used),
        (before.selector_bytes, before.banks_used, before.groups_used)
    );
    assert_eq!(
        &native.bus.vram[0x10000..0x18000],
        &rom[SOURCE as usize..SOURCE as usize + LENGTH]
    );
    native.bus.rom[SOURCE as usize..SOURCE as usize + LENGTH]
        .copy_from_slice(&rom[SOURCE as usize..SOURCE as usize + LENGTH]);
    assert_eq!(
        raw.save_state().unwrap(),
        native.save_state().unwrap(),
        "full translated original queue/DMA state and clock"
    );
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS", "revision":[135,134], "fixed_source_bytes":LENGTH, "translated_queue_dma_full_state_clock_cases":1, "managed_source_after_state_reload":true, "localized_stats_unchanged":true, "ordinary_scene":"pending", "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false})).unwrap()).unwrap();
}
