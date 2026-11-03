//! Actual original interpreter/native palette producers with poisoned ROM data.
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::{bg_alternate_palettes::Catalog, bg_descriptors},
    Bus, Machine,
};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, sync::Arc};

thread_local! {
    static API_READS: Cell<u64> = const { Cell::new(0) };
    static API_LITERAL_BYTES: Cell<u64> = const { Cell::new(0) };
    static INTERPRETER_SHELLS: Cell<u64> = const { Cell::new(0) };
}
fn counted(address: u32) {
    if matches!((address & 0x01ff_ffff) >> 12, 0x427..=0x431 | 0x489..=0x493) {
        API_READS.with(|count| count.set(count.get() + 1));
    }
}
extern "C" fn read8(machine: *mut std::ffi::c_void, address: u32) -> u32 {
    counted(address);
    let before = literal_count(machine);
    let result = (RT_API.read8)(machine, address);
    literal_delta(machine, before);
    result
}
extern "C" fn read16(machine: *mut std::ffi::c_void, address: u32) -> u32 {
    counted(address);
    let before = literal_count(machine);
    let result = (RT_API.read16)(machine, address);
    literal_delta(machine, before);
    result
}
extern "C" fn read32(machine: *mut std::ffi::c_void, address: u32) -> u32 {
    counted(address);
    let before = literal_count(machine);
    let result = (RT_API.read32)(machine, address);
    literal_delta(machine, before);
    result
}
fn literal_count(machine: *mut std::ffi::c_void) -> u64 {
    // Test-only callback inspection. Do not hold a shared borrow while the
    // original API performs its mutable bus read.
    unsafe {
        (&*(machine as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .alternate_palette_stats()
            .unwrap()
            .decoded_literal_bytes
    }
}
fn literal_delta(machine: *mut std::ffi::c_void, before: u64) {
    API_LITERAL_BYTES.with(|count| count.set(count.get() + literal_count(machine) - before));
}
extern "C" fn interp(machine: *mut std::ffi::c_void) -> u32 {
    INTERPRETER_SHELLS.with(|count| count.set(count.get() + 1));
    (RT_API.interp_one)(machine)
}

#[test]
#[ignore = "requires exact ROM, current dataset and FFTA_STATE_TEST_ALT_PALETTE_NATIVE from the completed original/native probe"]
fn native_api_reads_decoded_literal_banks_using_existing_component() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let entry = if gba_core::ffta_rom::digest(&rom) == gba_core::ffta_rom::SHA256 {
        0x0801_a42c
    } else {
        0x0801_b59c
    };
    let mut machine = crate::make_machine(rom, None);
    attach_from_directory(
        &mut machine,
        &PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap()),
    )
    .unwrap();
    for source in catalog.tables.iter().chain(&catalog.sources) {
        machine.bus.rom[source.offset as usize..(source.offset + source.length) as usize]
            .fill(0xcd);
    }
    let (_library, table) = crate::load_native(&PathBuf::from(
        std::env::var_os("FFTA_STATE_TEST_ALT_PALETTE_NATIVE").unwrap(),
    ))
    .unwrap();
    let api = gba_core::capi::RtApi {
        read8,
        read16,
        read32,
        interp_one: interp,
        ..RT_API
    };
    machine.bus.write8(0x0200_2fc2, 1);
    machine.cpu.regs[0] = 0;
    machine.cpu.regs[1] = 0x0300_3860;
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = entry;
    machine.cpu.cpsr = 0xbf;
    API_LITERAL_BYTES.with(|count| count.set(0));
    API_READS.with(|count| count.set(0));
    let mut returned = false;
    for _ in 0..50_000 {
        let function = table.get(machine.cpu.regs[15] | 1).unwrap();
        function(&api, &mut machine as *mut Machine as *mut std::ffi::c_void);
        if machine.cpu.regs[15] == 0x0800_0300 {
            returned = true;
            break;
        }
    }
    assert!(returned);
    assert!(API_LITERAL_BYTES.with(Cell::get) > 0);
    assert_eq!(
        &machine.bus.iwram[0x3860..0x3960],
        catalog.decoded.get(0, 1).unwrap().source_bytes().as_slice()
    );
    eprintln!(
        "ALT_PALETTE_NATIVE_BANKS api_reads={} native_literal_bytes={} reused_component=true",
        API_READS.with(Cell::get),
        API_LITERAL_BYTES.with(Cell::get)
    );
}

#[test]
#[ignore = "requires exact ROM and existing current FFTA_STATE_TEST_DATASET"]
fn loaded_alternates_survive_deleted_files_slots_undo_backup_and_restart() {
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
    ] {
        fs::copy(original.join(name), dataset.join(name)).unwrap();
    }
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let mut machine = crate::make_machine(rom, None);
    let boot = machine.save_state().unwrap();
    attach_from_directory(&mut machine, &dataset).unwrap();
    let owner = machine.bus.rom_resources().unwrap().clone();
    assert!(owner.alternate_palette_file_sha256().is_some());
    let first = &catalog.sources[0];
    for index in first.offset..first.offset + first.length {
        machine.bus.read8(0x0800_0000 + index);
    }
    let count = owner.alternate_palette_stats().unwrap();
    assert!(count.decoded_literal_bytes > 0);
    fs::remove_dir_all(dataset).unwrap();
    let save = temporary.path().join("test.sav");
    let mut slots = crate::save_state::Slots::new(&save, &machine.bus.rom);
    slots
        .save(&machine, 1, &vec![[0; 4]; 240 * 160], false)
        .unwrap();
    machine.cpu.regs[4] = 99;
    let modified = machine.save_state().unwrap();
    slots.load(&mut machine, 1).unwrap();
    assert_eq!(machine.cpu.regs[4], 0);
    slots.undo(&mut machine).unwrap();
    assert_eq!(machine.save_state().unwrap(), modified);
    assert!(Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
    assert_eq!(owner.alternate_palette_stats().unwrap(), count);
    let length = machine.bus.save_data().unwrap().len();
    fs::write(&save, vec![7; length]).unwrap();
    let backups = crate::save_backups::Backups::new(&save);
    backups.capture_startup().unwrap();
    let generation = backups.choices(length)[0].generation.unwrap();
    backups.restore(generation, &mut machine, &boot).unwrap();
    assert_eq!(machine.bus.save_data().unwrap(), vec![7; length]);
    machine.bus.load_save_data(&vec![9; length]);
    crate::save_backups::restart(&mut machine, &boot, &save).unwrap();
    assert_eq!(machine.bus.save_data().unwrap(), vec![9; length]);
    assert_eq!(fs::read(&save).unwrap(), vec![9; length]);
    assert!(Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
    assert_eq!(owner.alternate_palette_stats().unwrap(), count);
    assert_eq!(machine.bus.read8(0x0800_0000 + first.offset), 0);
    assert!(owner.alternate_palette_stats().unwrap().bus_reads > count.bus_reads);
}

fn component(rom: &[u8], entries: [u32; 2], output: &Path) {
    let view = View {
        rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let mut blocks = Vec::new();
    for start in entries {
        let mut returned = false;
        for address in (start..start + 0x480).step_by(2) {
            let instruction = armv4t::decode_thumb(view.read16(address), address);
            let end = matches!(instruction.op, armv4t::Op::Bx { .. });
            let next = armv4t::decode_thumb(view.read16(address + 2), address + 2);
            let fused = armv4t::fuse_thumb_bl(&instruction, &next).is_some();
            let mut instrs = vec![instruction];
            if fused {
                instrs.push(next);
            }
            blocks.push(Block {
                addr: address,
                thumb: true,
                instrs,
            });
            if end {
                returned = true;
                break;
            }
        }
        assert!(
            returned,
            "bounded producer/consumer must reach original return"
        );
    }
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        output,
        &mut |_| {},
    )
    .unwrap();
}

fn equal(left: &Machine, right: &Machine) {
    assert_eq!(left.cpu.regs, right.cpu.regs);
    assert_eq!(left.cpu.cpsr, right.cpu.cpsr);
    assert_eq!(left.bus.clock, right.bus.clock);
    assert!(left.bus.ewram == right.bus.ewram);
    assert!(left.bus.iwram == right.bus.iwram);
}

#[test]
#[ignore = "requires exact ROM, existing current dataset, TinyCC and absent FFTA_STATE_TEST_ALT_PALETTE_RUNTIME_ROOT"]
fn original_and_native_alternate_producers_read_decoded_banks_with_legacy_states() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_ALT_PALETTE_RUNTIME_ROOT").unwrap());
    assert!(!root.exists(), "preserve previous runtime observations");
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let bg = bg_descriptors::Catalog::from_rom(&rom).unwrap();
    let entries = match gba_core::ffta_rom::digest(&rom).as_str() {
        gba_core::ffta_rom::SHA256 => [0x0801_a42c, 0x0800_543c],
        gba_core::ffta_rom::EUROPE_SHA256 => [0x0801_b59c, 0x0800_54a0],
        _ => panic!("exact ROM required"),
    };
    fs::create_dir_all(&root).unwrap();
    let object = root.join("alternate-producer.o");
    component(&rom, entries, &object);
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut legacy = crate::make_machine(rom.clone(), None);
    let mut decoded = crate::make_machine(rom.clone(), None);
    let mut native = crate::make_machine(rom.clone(), None);
    let boot = legacy.save_state().unwrap();
    for machine in [&mut decoded, &mut native] {
        attach_from_directory(machine, &dataset).unwrap();
        assert_eq!(machine.save_state().unwrap(), boot);
        for source in catalog
            .tables
            .iter()
            .chain(&catalog.sources)
            .chain(std::iter::once(&bg.source))
        {
            machine.bus.rom[source.offset as usize..(source.offset + source.length) as usize]
                .fill(0xcd);
        }
    }
    let api = gba_core::capi::RtApi {
        read8,
        read16,
        read32,
        interp_one: interp,
        ..RT_API
    };
    API_READS.with(|count| count.set(0));
    INTERPRETER_SHELLS.with(|count| count.set(0));
    let (mut calls, mut instructions, mut dispatches) = (0u64, 0u64, 0u64);
    const RETURN: u32 = 0x0800_0300;
    for environment in [1u8, 2, 0xfd, 0xfe] {
        for (id, &key) in catalog.decoded.descriptor_keys.iter().enumerate() {
            if key == 255 {
                continue;
            } // Caller preconditions remain separate.
            let expected = catalog.decoded.get(id, environment).unwrap().source_bytes();
            for argument in [id as u32, 0xffff] {
                for machine in [&mut legacy, &mut decoded, &mut native] {
                    machine.bus.iwram[0x3850..0x3970].fill(0xcd);
                    machine.bus.write8(0x0200_2fc2, environment);
                    machine.bus.write16(0x0200_7f10, id as u16);
                    machine.cpu.regs[0] = argument;
                    machine.cpu.regs[1] = 0x0300_3860;
                    machine.cpu.regs[13] = 0x0300_7c00;
                    machine.cpu.regs[14] = RETURN | 1;
                    machine.cpu.regs[15] = entries[0];
                    machine.cpu.cpsr = 0xbf;
                }
                let before = native
                    .bus
                    .rom_resources()
                    .unwrap()
                    .alternate_palette_stats()
                    .unwrap();
                let mut blocks = 0;
                while native.cpu.regs[15] != RETURN {
                    blocks += 1;
                    assert!(blocks < 50_000, "original producer must return");
                    if native.bus.frame_ready {
                        assert!(legacy.bus.frame_ready && decoded.bus.frame_ready);
                        for machine in [&mut legacy, &mut decoded, &mut native] {
                            machine.bus.frame_ready = false;
                        }
                    }
                    let function = table
                        .get(native.cpu.regs[15] | 1)
                        .expect("compiled original producer/consumer instruction");
                    function(&api, &mut native as *mut Machine as *mut std::ffi::c_void);
                    dispatches += 1;
                    while legacy.bus.clock < native.bus.clock {
                        legacy.step();
                        decoded.step();
                        equal(&legacy, &decoded);
                        instructions += 1;
                    }
                    equal(&legacy, &native);
                }
                assert_eq!(&native.bus.iwram[0x3860..0x3960], expected.as_slice());
                assert!(native.bus.iwram[0x3850..0x3860].iter().all(|&b| b == 0xcd));
                assert!(native.bus.iwram[0x3960..0x3970].iter().all(|&b| b == 0xcd));
                assert_eq!(native.cpu.regs[13], 0x0300_7c00);
                let after = native
                    .bus
                    .rom_resources()
                    .unwrap()
                    .alternate_palette_stats()
                    .unwrap();
                assert!(
                    after.table_bytes > before.table_bytes
                        && after.decoded_literal_bytes > before.decoded_literal_bytes
                );
                calls += 1;
            }
        }
    }
    assert_eq!(calls, 1256);
    for machine in [&mut decoded, &mut native] {
        for source in catalog
            .tables
            .iter()
            .chain(&catalog.sources)
            .chain(std::iter::once(&bg.source))
        {
            machine.bus.rom[source.offset as usize..(source.offset + source.length) as usize]
                .copy_from_slice(
                    &rom[source.offset as usize..(source.offset + source.length) as usize],
                );
        }
        assert_eq!(machine.save_state().unwrap(), legacy.save_state().unwrap());
    }
    assert!(
        API_READS.with(Cell::get) > 0,
        "native loads must actually call the data bus API"
    );
    let resources = decoded.bus.rom_resources().unwrap().clone();
    let stats = resources.alternate_palette_stats().unwrap();
    decoded.load_state(&boot).unwrap();
    assert!(Arc::ptr_eq(
        decoded.bus.rom_resources().unwrap(),
        &resources
    ));
    assert_eq!(resources.alternate_palette_stats().unwrap(), stats);
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"calls":calls,"original_instructions":instructions,"native_dispatches":dispatches,"native_api_reads":API_READS.with(Cell::get),"interpreter_shells":INTERPRETER_SHELLS.with(Cell::get),"source_poisoned":true,"full_states_equal":true,"decoded_literal_bytes":stats.decoded_literal_bytes,"table_bytes":stats.table_bytes,"bus_reads":stats.bus_reads,"all_outputs_guards":true,"load_retains_owner":true,"performance_measurements":false})).unwrap()).unwrap();
    eprintln!("ALT_PALETTE_RUNTIME calls={calls} instructions={instructions} native_dispatches={dispatches} native_api_reads={} decoded_literal_bytes={} full_states=true",API_READS.with(Cell::get),stats.decoded_literal_bytes);
}
