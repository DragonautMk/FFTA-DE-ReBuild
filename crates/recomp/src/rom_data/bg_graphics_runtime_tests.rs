//! Reduced EUR proof that original/native graphics consumers use decoded files.
use super::*;
use gba_core::{capi::RT_API, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
const RETURN: u32 = 0x08000300;
const STACK: u32 = 0x03007c00;
const TASK: u32 = 0x03002000;
const ENTRY: u32 = 0x0801b688;
const DEST: usize = 0x91a0;
thread_local! { static NATIVE_TYPED_BYTES: Cell<u64> = const { Cell::new(0) }; }
fn typed(machine: *mut std::ffi::c_void) -> u64 {
    unsafe {
        (&*(machine as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .graphics_stats()
            .unwrap()
            .decoded_literal_bytes
    }
}
extern "C" fn read8(machine: *mut std::ffi::c_void, address: u32) -> u32 {
    let before = typed(machine);
    let result = (RT_API.read8)(machine, address);
    NATIVE_TYPED_BYTES.with(|n| n.set(n.get() + typed(machine) - before));
    result
}
extern "C" fn read16(machine: *mut std::ffi::c_void, address: u32) -> u32 {
    let before = typed(machine);
    let result = (RT_API.read16)(machine, address);
    NATIVE_TYPED_BYTES.with(|n| n.set(n.get() + typed(machine) - before));
    result
}
extern "C" fn read32(machine: *mut std::ffi::c_void, address: u32) -> u32 {
    let before = typed(machine);
    let result = (RT_API.read32)(machine, address);
    NATIVE_TYPED_BYTES.with(|n| n.set(n.get() + typed(machine) - before));
    result
}

fn setup(machine: &mut Machine, id: u16) {
    machine.bus.ewram.fill(0);
    machine.bus.ewram[DEST - 16..DEST + gba_core::rom_data::bg_graphics::DESTINATION_BYTES + 16]
        .fill(0x5a);
    machine.bus.write16(0x02007f10, id);
    machine.bus.write32(0x02007f50, 0x06000000);
    machine.bus.write32(0x02007f54, 0x06006000);
    machine.bus.write32(0x02007f58, 0x06008000);
    machine.bus.write8(TASK, 1);
    machine.cpu.regs[1] = TASK;
    machine.cpu.regs[13] = STACK;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = ENTRY;
    machine.cpu.cpsr = 0xbf;
}
fn ranges(
    machine: &mut Machine,
    sources: &[gba_core::rom_data::SourceRange],
    original: Option<&[u8]>,
) {
    for source in sources {
        let at = source.offset as usize;
        let end = at + source.length as usize;
        if let Some(rom) = original {
            machine.bus.rom[at..end].copy_from_slice(&rom[at..end]);
        } else {
            machine.bus.rom[at..end].fill(0xcd);
        }
    }
}

#[test]
#[ignore = "requires exact EUR ROM, e9 dataset, TinyCC and absent FFTA_STATE_TEST_GRAPHICS_RUNTIME_ROOT"]
fn european_original_and_native_graphics_use_decoded_words_with_exact_states() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_GRAPHICS_RUNTIME_ROOT").unwrap());
    assert!(!root.exists(), "preserve original/native evidence");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let files = read_dataset(&dataset).unwrap();
    let graphics = files.main_graphics.as_ref().unwrap();
    let sources = &files.manifest.resources[5].sources;
    // First/new pages, both wrapper forms, maximum output and last source.
    let cases = [0u16, 19, 38, 161];
    let mut tracer = crate::make_machine(rom.clone(), None);
    let mut visited = BTreeSet::new();
    let mut instructions = 0;
    for id in cases {
        setup(&mut tracer, id);
        let mut steps = 0;
        while tracer.cpu.regs[15] != RETURN {
            assert!(steps < 2_000_000 && tracer.cpu.thumb());
            visited.insert(tracer.cpu.regs[15]);
            tracer.bus.frame_ready = false;
            tracer.step();
            steps += 1;
        }
        instructions += steps;
    }
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let blocks = visited
        .iter()
        .map(|&address| {
            let instruction = armv4t::decode_thumb(view.read16(address), address);
            let next = armv4t::decode_thumb(view.read16(address + 2), address + 2);
            let fused = armv4t::fuse_thumb_bl(&instruction, &next).is_some();
            let mut instrs = vec![instruction];
            if fused {
                instrs.push(next);
            }
            Block {
                addr: address,
                thumb: true,
                instrs,
            }
        })
        .collect();
    fs::create_dir_all(&root).unwrap();
    let object = root.join("graphics-consumers.o");
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
    for machine in [&mut decoded, &mut native] {
        attach_from_directory(machine, &dataset).unwrap();
    }
    let api = gba_core::capi::RtApi {
        read8,
        read16,
        read32,
        ..RT_API
    };
    NATIVE_TYPED_BYTES.with(|n| n.set(0));
    let mut reports = Vec::new();
    let mut dispatches = 0;
    for id in cases {
        let graphic = graphics.graphic(id as usize).unwrap();
        let expected = graphic.source_bytes();
        for machine in [&mut legacy, &mut decoded, &mut native] {
            setup(machine, id);
        }
        for machine in [&mut decoded, &mut native] {
            ranges(machine, sources, None);
        }
        for machine in [&mut legacy, &mut decoded] {
            let mut steps = 0;
            while machine.cpu.regs[15] != RETURN {
                assert!(steps < 2_000_000);
                machine.bus.frame_ready = false;
                machine.step();
                steps += 1;
            }
        }
        let before = NATIVE_TYPED_BYTES.with(Cell::get);
        let mut blocks = 0;
        while native.cpu.regs[15] != RETURN {
            assert!(blocks < 2_000_000);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | 1)
                .expect("original graphics instruction compiled")(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            blocks += 1;
        }
        let compiled_bytes = NATIVE_TYPED_BYTES.with(Cell::get) - before;
        assert!(
            compiled_bytes > 0,
            "case{id} must consume words through compiled loads"
        );
        dispatches += blocks;
        for machine in [&mut decoded, &mut native] {
            assert_eq!(machine.cpu.regs[13], STACK);
            assert_eq!(machine.bus.read8(TASK), 2);
            assert_eq!(&machine.bus.ewram[DEST..DEST + expected.len()], &expected);
            assert_eq!(&machine.bus.ewram[DEST - 16..DEST], &[0x5a; 16]);
            assert!(machine.bus.ewram[DEST + expected.len()
                ..DEST + gba_core::rom_data::bg_graphics::DESTINATION_BYTES + 16]
                .iter()
                .all(|&b| b == 0x5a));
            ranges(machine, sources, Some(&rom));
            assert!(
                legacy.save_state().unwrap() == machine.save_state().unwrap(),
                "case{id} full state/clock"
            );
        }
        reports.push(
            serde_json::json!({"descriptor":id,"tag":graphic.encoded_source[0],
            "destination_offset":graphic.destination_offset,"output_bytes":expected.len(),
            "compiled_typed_bytes":compiled_bytes,"full_states_equal":true}),
        );
    }
    let stats = native
        .bus
        .rom_resources()
        .unwrap()
        .graphics_stats()
        .unwrap();
    assert!(stats.decoded_literal_bytes > 0 && stats.bus_reads > 0);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "region":"europe","cases":reports,"poisoned_sources":sources.len(),"original_instructions":instructions,
        "native_dispatches":dispatches,"compiled_typed_bytes":NATIVE_TYPED_BYTES.with(Cell::get),
        "decoded_literal_bytes":stats.decoded_literal_bytes,"full_states_equal":true,
        "linux":"pending","usa":"pending","performance_measurements":false,
    })).unwrap()).unwrap();
    eprintln!("BG_GRAPHICS_RUNTIME cases=4 poisoned_sources=50 full_states=true compiled_typed_bytes={} original_instructions={instructions} native_dispatches={dispatches}", NATIVE_TYPED_BYTES.with(Cell::get));
}

#[test]
#[ignore = "requires exact EUR ROM and e9 dataset; only temporary copies are renamed"]
fn european_graphics_owner_survives_files_slots_undo_backup_and_restart() {
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
    ] {
        fs::copy(original.join(name), dataset.join(name)).unwrap();
    }
    let files = read_dataset(&dataset).unwrap();
    let source = &files.manifest.resources[5].sources[0];
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let mut machine = crate::make_machine(rom, None);
    let boot = machine.save_state().unwrap();
    attach_from_directory(&mut machine, &dataset).unwrap();
    let owner = machine.bus.rom_resources().unwrap().clone();
    assert_eq!(
        owner.graphics_file_sha256(),
        Some(files.manifest.resources[5].sha256.as_str())
    );
    assert_eq!(machine.save_state().unwrap(), boot);
    for at in source.offset..source.offset + source.length {
        machine.bus.read8(0x08000000 + at);
    }
    let stats = owner.graphics_stats().unwrap();
    assert!(stats.decoded_literal_bytes > 0);
    fs::rename(&dataset, temporary.path().join("retained-files")).unwrap();
    assert!(attach_from_directory(&mut machine, &dataset).is_err());
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
    assert_eq!(owner.graphics_stats().unwrap(), stats);
    let before = machine.bus.read8(0x08000000 + source.offset + 9);
    machine.bus.rom[(source.offset + 9) as usize] ^= 0xff;
    assert_eq!(machine.bus.read8(0x08000000 + source.offset + 9), before);
    machine.bus.rom[(source.offset + 9) as usize] ^= 0xff;
    eprintln!("BG_GRAPHICS_LIFECYCLE copied_files_unavailable=true failed_activation_transactional=true slots_undo_backup_restart=true owner_retained=true");
}
