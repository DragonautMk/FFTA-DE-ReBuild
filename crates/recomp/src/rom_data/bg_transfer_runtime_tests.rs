//! Reduced source-provenance proof through original animation and native DMA.
use super::*;
use gba_core::{capi::RT_API, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};

const RETURN: u32 = 0x08000300;
const STACK: u32 = 0x03007c00;
const MANAGER: u32 = 0x02030000;
const OBJECT: u32 = MANAGER + 4;
thread_local! {
    static NATIVE_CONTROLS:Cell<u64>=const{Cell::new(0)};
    static NATIVE_DMA_TILES:Cell<u64>=const{Cell::new(0)};
}
fn stats(machine: *mut std::ffi::c_void) -> gba_core::rom_data::TransferStats {
    unsafe {
        (&*(machine as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .transfer_stats()
            .unwrap()
    }
}
extern "C" fn read8(machine: *mut std::ffi::c_void, address: u32) -> u32 {
    let before = stats(machine).control_bytes;
    let result = (RT_API.read8)(machine, address);
    NATIVE_CONTROLS.with(|v| v.set(v.get() + stats(machine).control_bytes - before));
    result
}
extern "C" fn read16(machine: *mut std::ffi::c_void, address: u32) -> u32 {
    let before = stats(machine).control_bytes;
    let result = (RT_API.read16)(machine, address);
    NATIVE_CONTROLS.with(|v| v.set(v.get() + stats(machine).control_bytes - before));
    result
}
extern "C" fn read32(machine: *mut std::ffi::c_void, address: u32) -> u32 {
    let before = stats(machine).control_bytes;
    let result = (RT_API.read32)(machine, address);
    NATIVE_CONTROLS.with(|v| v.set(v.get() + stats(machine).control_bytes - before));
    result
}
extern "C" fn write16(machine: *mut std::ffi::c_void, address: u32, value: u32) {
    let before = stats(machine).tile_bytes;
    (RT_API.write16)(machine, address, value);
    NATIVE_DMA_TILES.with(|v| v.set(v.get() + stats(machine).tile_bytes - before));
}
extern "C" fn write32(machine: *mut std::ffi::c_void, address: u32, value: u32) {
    let before = stats(machine).tile_bytes;
    (RT_API.write32)(machine, address, value);
    NATIVE_DMA_TILES.with(|v| v.set(v.get() + stats(machine).tile_bytes - before));
}

#[derive(Clone, Copy)]
struct Call {
    entry: u32,
    args: [u32; 3],
}
fn setup(machine: &mut Machine, call: Call) {
    machine.cpu.regs[..3].copy_from_slice(&call.args);
    machine.cpu.regs[13] = STACK;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = call.entry;
    machine.cpu.cpsr = 0xbf;
}
fn interpreted(machine: &mut Machine, call: Call, visited: &mut BTreeSet<(u32, bool)>) -> usize {
    setup(machine, call);
    for steps in 1..=500_000 {
        visited.insert((machine.cpu.regs[15], machine.cpu.thumb()));
        machine.bus.frame_ready = false;
        machine.step();
        if machine.cpu.regs[15] == RETURN {
            assert_eq!(machine.cpu.regs[13], STACK);
            return steps;
        }
    }
    panic!(
        "original transfer call did not return at{:08x}",
        machine.cpu.regs[15]
    );
}
fn ranges(machine: &mut Machine, sources: &[gba_core::rom_data::SourceRange], rom: Option<&[u8]>) {
    for source in sources {
        let at = source.offset as usize;
        let end = at + source.length as usize;
        if let Some(rom) = rom {
            machine.bus.rom[at..end].copy_from_slice(&rom[at..end]);
        } else {
            machine.bus.rom[at..end].fill(0xcd);
        }
    }
}

#[test]
#[ignore = "requires EUR ROM/e10 dataset/retained initialized snapshot/TinyCC and absent FFTA_STATE_TEST_TRANSFER_RUNTIME_ROOT"]
fn european_original_and_native_transfers_consume_controls_and_dma_words_with_exact_states() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_TRANSFER_RUNTIME_ROOT").unwrap());
    assert!(!root.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let files = read_dataset(&dataset).unwrap();
    let values = files.transfers.as_ref().unwrap();
    let sources = &files.manifest.resources[6].sources;
    let mut tracer = crate::make_machine(rom.clone(), None);
    crate::save_state::load(
        &mut tracer,
        Path::new(&std::env::var_os("FFTA_STATE_TEST_STATE").unwrap()),
    )
    .unwrap();
    let start = tracer.save_state().unwrap();
    let iwram = tracer.bus.iwram.clone();
    let mut visited = BTreeSet::new();
    let mut scripts = Vec::new();
    let mut instructions = 0;
    // New consumption criterion only: sample forward/reverse transitions and
    // live 512/1024-byte limits, rather than repeat the closed two-loop oracle.
    for (descriptor, slot, reverse, limit) in
        [(58usize, 0usize, false, 512u32), (63, 1, true, 1024)]
    {
        tracer.load_state(&start).unwrap();
        tracer.bus.vram.fill(0x5a);
        let link = values.descriptor_links[descriptor][slot].as_ref().unwrap();
        let control = &values.controls[link.control as usize];
        let tiles = &values.tiles[link.tiles as usize];
        let base = files.manifest.resources[1].sources[0].offset;
        let mut calls = vec![
            Call {
                entry: 0x080220a4,
                args: [MANAGER, 0, 0],
            },
            Call {
                entry: 0x08021f10,
                args: [
                    0x08000000 + base + control.relative_source,
                    link.destination,
                    0x08000000 + base + tiles.relative_source,
                ],
            },
            Call {
                entry: 0x08022160,
                args: [OBJECT, limit, 0],
            },
            Call {
                entry: 0x0802213c,
                args: [OBJECT, u32::from(reverse), 0],
            },
        ];
        for &call in &calls {
            instructions += interpreted(&mut tracer, call, &mut visited);
        }
        for _ in 0..17 {
            let call = Call {
                entry: 0x08021bd8,
                args: [0; 3],
            };
            instructions += interpreted(&mut tracer, call, &mut visited);
            calls.push(call);
            while tracer.bus.read8(OBJECT + 6) != 0 {
                let call = Call {
                    entry: 0x08021d10,
                    args: [0; 3],
                };
                instructions += interpreted(&mut tracer, call, &mut visited);
                calls.push(call);
                assert!(calls.len() < 200, "bounded reduced animation source probe");
            }
        }
        scripts.push((descriptor, slot, reverse, limit, calls));
    }
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: Some(&iwram),
        bios: None,
    };
    let blocks = visited
        .iter()
        .map(|&(address, thumb)| {
            let mut instrs = vec![if thumb {
                armv4t::decode_thumb(view.read16(address), address)
            } else {
                armv4t::decode_arm(view.read32(address), address)
            }];
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
    fs::create_dir_all(&root).unwrap();
    let object = root.join("transfer-consumers.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut legacy = crate::make_machine(rom.clone(), None);
    let mut decoded = crate::make_machine(rom.clone(), None);
    let mut native = crate::make_machine(rom.clone(), None);
    for target in [&mut decoded, &mut native] {
        attach_from_directory(target, &dataset).unwrap();
    }
    let api = gba_core::capi::RtApi {
        read8,
        read16,
        read32,
        write16,
        write32,
        ..RT_API
    };
    NATIVE_CONTROLS.with(|v| v.set(0));
    NATIVE_DMA_TILES.with(|v| v.set(0));
    let mut reports = Vec::new();
    let mut dispatches = 0;
    let mut calls = 0;
    for (id, slot, reverse, limit, script) in scripts {
        for target in [&mut legacy, &mut decoded, &mut native] {
            target.load_state(&start).unwrap();
            target.bus.vram.fill(0x5a);
        }
        let before_controls = NATIVE_CONTROLS.with(Cell::get);
        let before_tiles = NATIVE_DMA_TILES.with(Cell::get);
        for call in script {
            interpreted(&mut legacy, call, &mut BTreeSet::new());
            ranges(&mut decoded, sources, None);
            ranges(&mut native, sources, None);
            interpreted(&mut decoded, call, &mut BTreeSet::new());
            setup(&mut native, call);
            let mut blocks = 0;
            while native.cpu.regs[15] != RETURN {
                assert!(blocks < 500_000);
                native.bus.frame_ready = false;
                table
                    .get(native.cpu.regs[15] | u32::from(native.cpu.thumb()))
                    .expect("original ROM/IWRAM instruction compiled")(
                    &api,
                    &mut native as *mut Machine as *mut std::ffi::c_void,
                );
                blocks += 1;
            }
            calls += 1;
            dispatches += blocks;
            for target in [&mut decoded, &mut native] {
                ranges(target, sources, Some(&rom));
                assert!(
                    legacy.save_state().unwrap() == target.save_state().unwrap(),
                    "descriptor{id} call{:08x} full state/clock",
                    call.entry
                );
                assert_eq!(
                    target.bus.vram, legacy.bus.vram,
                    "ordered full VRAM and guards"
                );
            }
        }
        let controls = NATIVE_CONTROLS.with(Cell::get) - before_controls;
        let tiles = NATIVE_DMA_TILES.with(Cell::get) - before_tiles;
        assert!(
            controls > 0 && tiles > 0,
            "compiled control reads and native-initiated DMA must use typed files"
        );
        reports.push(serde_json::json!({"descriptor":id,"slot":slot,"reverse":reverse,"limit":limit,"ticks":17,
            "compiled_control_bytes":controls,"native_initiated_dma_tile_bytes":tiles,"full_states_equal":true}));
    }
    let stats = native
        .bus
        .rom_resources()
        .unwrap()
        .transfer_stats()
        .unwrap();
    let report = serde_json::json!({"region":"europe","cases":reports,"poisoned_sources":sources.len(),"original_instructions":instructions,
        "calls":calls,"native_dispatches":dispatches,"compiled_control_bytes":NATIVE_CONTROLS.with(Cell::get),
        "native_initiated_dma_tile_bytes":NATIVE_DMA_TILES.with(Cell::get),"control_bytes":stats.control_bytes,"tile_bytes":stats.tile_bytes,
        "full_states_equal_at_every_call":true,"linux":"pending","usa":"pending","performance_measurements":false});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("BG_TRANSFER_RUNTIME cases=2 calls={calls} instructions={instructions} native_dispatches={dispatches} compiled_control_bytes={} native_DMA_tile_bytes={} full_states=true",NATIVE_CONTROLS.with(Cell::get),NATIVE_DMA_TILES.with(Cell::get));
}

#[test]
#[ignore = "requires exact EUR ROM/e10 dataset; only temporary copied files are renamed"]
fn european_transfer_owner_survives_files_slots_undo_backup_and_restart() {
    let temporary = tempfile::tempdir().unwrap();
    let dataset = temporary.path().join("dataset");
    fs::create_dir(&dataset).unwrap();
    let original = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    for name in [
        MANIFEST_FILE,
        OBJ_DIMENSIONS_FILE,
        BG_DESCRIPTORS_FILE,
        BG_BASE_PALETTES_FILE,
        BG_ALTERNATE_PALETTES_FILE,
        BG_PACKET_PROGRAMS_FILE,
        BG_MAIN_GRAPHICS_FILE,
        BG_TRANSFER_PROGRAMS_FILE,
    ] {
        fs::copy(original.join(name), dataset.join(name)).unwrap();
    }
    let files = read_dataset(&dataset).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let mut machine = crate::make_machine(rom.clone(), None);
    let boot = machine.save_state().unwrap();
    attach_from_directory(&mut machine, &dataset).unwrap();
    let owner = machine.bus.rom_resources().unwrap().clone();
    assert_eq!(
        owner.transfer_file_sha256(),
        Some(files.manifest.resources[6].sha256.as_str())
    );
    let sources = &files.manifest.resources[6].sources;
    let control = &sources[0];
    let tile = &sources[files.transfers.as_ref().unwrap().controls.len()];
    machine.bus.read16(0x08000000 + control.offset);
    machine.bus.read32(0x08000000 + tile.offset);
    let stats = owner.transfer_stats().unwrap();
    assert!(stats.control_bytes > 0 && stats.tile_bytes > 0);
    fs::rename(&dataset, temporary.path().join("retained-files")).unwrap();
    let state = machine.save_state().unwrap();
    assert!(attach_from_directory(&mut machine, &dataset).is_err());
    assert_eq!(machine.save_state().unwrap(), state);
    assert!(Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
    let save = temporary.path().join("test.sav");
    let mut slots = crate::save_state::Slots::new(&save, &machine.bus.rom);
    slots
        .save(&machine, 1, &vec![[0; 4]; 240 * 160], false)
        .unwrap();
    machine.cpu.regs[4] = 99;
    let changed = machine.save_state().unwrap();
    slots.load(&mut machine, 1).unwrap();
    assert_eq!(machine.cpu.regs[4], 0);
    slots.undo(&mut machine).unwrap();
    assert_eq!(machine.save_state().unwrap(), changed);
    let length = machine.bus.save_data().unwrap().len();
    fs::write(&save, vec![7; length]).unwrap();
    let backups = crate::save_backups::Backups::new(&save);
    backups.capture_startup().unwrap();
    backups
        .restore(
            backups.choices(length)[0].generation.unwrap(),
            &mut machine,
            &boot,
        )
        .unwrap();
    assert_eq!(machine.bus.save_data().unwrap(), vec![7; length]);
    machine.bus.load_save_data(&vec![9; length]);
    crate::save_backups::restart(&mut machine, &boot, &save).unwrap();
    assert_eq!(machine.bus.save_data().unwrap(), vec![9; length]);
    assert_eq!(fs::read(&save).unwrap(), vec![9; length]);
    assert!(Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
    assert_eq!(owner.transfer_stats().unwrap(), stats);
    ranges(&mut machine, sources, None);
    assert_eq!(
        machine.bus.read16(0x08000000 + control.offset),
        u16::from_le_bytes(
            rom[control.offset as usize..control.offset as usize + 2]
                .try_into()
                .unwrap()
        )
    );
    assert_eq!(
        machine.bus.read32(0x08000000 + tile.offset),
        u32::from_le_bytes(
            rom[tile.offset as usize..tile.offset as usize + 4]
                .try_into()
                .unwrap()
        )
    );
    ranges(&mut machine, sources, Some(&rom));
    eprintln!("BG_TRANSFER_LIFECYCLE copied_files_unavailable=true transactional_activation=true slots_undo_backup_restart=true owner_retained=true typed_control_tiles=true");
}
