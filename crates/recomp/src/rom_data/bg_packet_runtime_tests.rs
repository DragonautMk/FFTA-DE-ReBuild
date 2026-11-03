//! Reduced EUR consumer proof; no old whole-catalog/boot or performance rerun.
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::bg_packet_data::{self, Role, Writes},
    Bus, Machine,
};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};

const RETURN: u32 = 0x08000300;
const STACK: u32 = 0x03007c00;
thread_local! { static NATIVE_TYPED_BYTES: Cell<u64> = const { Cell::new(0) }; }
fn typed(machine: *mut std::ffi::c_void) -> u64 {
    unsafe {
        (&*(machine as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .packet_stats()
            .unwrap()
            .typed_value_bytes
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
fn setup(machine: &mut Machine, source: u32, entry: u32, base: usize, size: usize, blank: u16) {
    machine.bus.ewram.fill(0);
    machine.bus.ewram[base - 16..base + size + 16].fill(0x5a);
    machine.bus.write16(0x02007f5e, blank);
    machine.cpu.regs[0] = 0x08000000 + source;
    machine.cpu.regs[13] = STACK;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = entry;
    machine.cpu.cpsr = 0xbf;
}
fn apply(program: &bg_packet_data::Program, target: &mut [u8]) {
    match &program.writes {
        Writes::Paired(value) => {
            for patch in &value.calculated_plane_writes().unwrap().packets {
                for (i, word) in patch.values.iter().enumerate() {
                    let at = patch.byte_offset as usize + i * 4;
                    target[at..at + 4].copy_from_slice(&word.to_le_bytes());
                }
            }
        }
        Writes::Halfwords(value) => {
            for patch in &value.packets {
                for (i, word) in patch.values.iter().enumerate() {
                    let at = patch.byte_offset as usize + i * 2;
                    target[at..at + 2].copy_from_slice(&word.to_le_bytes());
                }
            }
        }
    }
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
#[ignore = "requires exact EUR ROM, existing e8 dataset, TinyCC and absent FFTA_STATE_TEST_PACKET_RUNTIME_ROOT"]
fn european_original_and_native_packets_consume_files_with_exact_states() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_PACKET_RUNTIME_ROOT").unwrap());
    assert!(!root.exists(), "preserve consumer evidence");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let files = read_dataset(&dataset).unwrap();
    let sources = &files.manifest.resources[4].sources;
    let programs = &files.packet_programs;
    let descriptor_root = files.manifest.resources[1].sources[0].offset;
    let mut cases = Vec::new();
    for (role, entry, base) in [
        (Role::Paired, 0x08020318, 0x91a0),
        (Role::Halfwords, 0x0802045c, 0xd1a0),
        (Role::Auxiliary, 0x080205a8, 0x7cb0),
    ] {
        let family = programs.family(role);
        let mut ids = BTreeSet::new();
        for tag in [1, 0x11] {
            if let Some(id) = (0..family.descriptor_programs.len())
                .find(|&id| family.program(id).unwrap().encoded_source[0] == tag)
            {
                ids.insert(id);
            }
        }
        if let Some(id) = (0..family.descriptor_programs.len())
            .find(|&id| family.program(id).unwrap().parent_descriptor.is_some())
        {
            ids.insert(id);
        }
        if role == Role::Paired {
            for wide in [false, true] {
                ids.insert(
                    (0..family.descriptor_programs.len())
                        .find(|&id| match &family.program(id).unwrap().writes {
                            Writes::Paired(value) => (value.header & 0x40000000 != 0) == wide,
                            _ => false,
                        })
                        .unwrap(),
                );
            }
        }
        for id in ids {
            cases.push((role, id, entry, base, 0x6421u16));
        }
        if role == Role::Paired {
            let id = (0..family.descriptor_programs.len())
                .find(|&id| family.program(id).unwrap().parent_descriptor.is_none())
                .unwrap();
            cases.push((role, id, entry, base, 0x8327));
        }
    }
    assert!(
        cases.len() <= 12,
        "keep the declared consumer matrix reduced"
    );
    let mut tracer = crate::make_machine(rom.clone(), None);
    let mut visited = BTreeSet::new();
    let mut instructions = 0;
    for &(role, id, entry, base, blank) in &cases {
        let program = programs.family(role).program(id).unwrap();
        setup(
            &mut tracer,
            descriptor_root + program.relative_source,
            entry,
            base,
            role.destination_bytes(),
            blank,
        );
        let mut steps = 0;
        while tracer.cpu.regs[15] != RETURN {
            assert!(steps < 2_000_000, "bounded original call");
            assert!(tracer.cpu.thumb());
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
    let object = root.join("packet-consumers.o");
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
    for &(role, id, entry, base, blank) in &cases {
        let family = programs.family(role);
        let program = family.program(id).unwrap();
        for machine in [&mut legacy, &mut decoded, &mut native] {
            setup(
                machine,
                descriptor_root + program.relative_source,
                entry,
                base,
                role.destination_bytes(),
                blank,
            );
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
        let mut blocks = 0;
        while native.cpu.regs[15] != RETURN {
            assert!(blocks < 2_000_000);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | 1)
                .expect("traced original consumer must be compiled")(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            blocks += 1;
        }
        dispatches += blocks;
        let mut expected = vec![0; role.destination_bytes()];
        if role == Role::Paired {
            for word in expected.chunks_exact_mut(2) {
                word.copy_from_slice(&blank.to_le_bytes());
            }
        }
        for parent in family.ordered_programs(id).unwrap() {
            apply(parent, &mut expected);
        }
        for machine in [&mut decoded, &mut native] {
            assert_eq!(machine.cpu.regs[13], STACK);
            assert!(
                machine.bus.ewram[base..base + expected.len()] == expected,
                "{} id={id} typed destination",
                role.name()
            );
            assert_eq!(&machine.bus.ewram[base - 16..base], &[0x5a; 16]);
            assert_eq!(
                &machine.bus.ewram[base + expected.len()..base + expected.len() + 16],
                &[0x5a; 16]
            );
            if role == Role::Paired {
                assert_eq!(machine.bus.read32(0x020080a8), 0);
            }
            ranges(machine, sources, Some(&rom));
            assert!(
                legacy.save_state().unwrap() == machine.save_state().unwrap(),
                "{} id={id} full state/clock",
                role.name()
            );
        }
        reports.push(serde_json::json!({"role":role.name(),"descriptor":id,"wrapper":program.encoded_source[0],"parent":program.parent_descriptor,"blank":blank,"full_states_equal":true}));
    }
    assert!(
        NATIVE_TYPED_BYTES.with(Cell::get) > 0,
        "compiled loads must consume typed values"
    );
    let stats = native.bus.rom_resources().unwrap().packet_stats().unwrap();
    assert!(stats.typed_value_bytes > 0 && stats.typed_hle_bytes > 0 && stats.lz77_calls > 0);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "region":"europe","cases":reports,"original_instructions":instructions,"native_dispatches":dispatches,
        "compiled_typed_bytes":NATIVE_TYPED_BYTES.with(Cell::get),"typed_value_bytes":stats.typed_value_bytes,
        "lz77_calls":stats.lz77_calls,"typed_hle_bytes":stats.typed_hle_bytes,"source_poisoned":true,
        "full_states_equal":true,"dynamic_paired_fill":true,"linux":"pending","usa":"pending","performance_measurements":false,
    })).unwrap()).unwrap();
    eprintln!("BG_PACKET_RUNTIME cases={} full_states=true compiled_typed_bytes={} hle_typed_bytes={} linux=pending usa=pending", cases.len(), NATIVE_TYPED_BYTES.with(Cell::get), stats.typed_hle_bytes);
}

#[test]
#[ignore = "requires exact EUR ROM and existing e8 dataset; uses only temporary copies"]
fn european_packet_owner_survives_files_slots_undo_backup_and_restart() {
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
    let files = read_dataset(&dataset).unwrap();
    let source = &files.manifest.resources[4].sources[0];
    let program = &files.packet_programs.paired.programs[0];
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
        owner.packet_file_sha256(),
        Some(files.manifest.resources[4].sha256.as_str())
    );
    assert_eq!(machine.save_state().unwrap(), boot);
    for index in source.offset..source.offset + source.length {
        machine.bus.read8(0x08000000 + index);
    }
    let read = machine
        .bus
        .decoded_packet_body(0x08000004 + source.offset)
        .unwrap();
    assert_eq!(read.bytes(), program.packet_body);
    let stats = owner.packet_stats().unwrap();
    assert!(stats.typed_hle_bytes > 0);
    // Make only our temporary copy unavailable. Never delete or move the
    // caller-supplied dataset: it has been opened read-only and copied above.
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
    assert_eq!(owner.packet_stats().unwrap(), stats);
    assert_eq!(read.bytes(), program.packet_body);
    machine.bus.set_rom_resources(None).unwrap();
    assert_eq!(read.bytes(), program.packet_body);
    eprintln!("BG_PACKET_LIFECYCLE copied_files_unavailable=true failed_activation_transactional=true slots_undo_backup_restart=true owner_retained=true");
}
