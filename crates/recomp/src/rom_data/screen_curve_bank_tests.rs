//! Exercise the original screen controller rows with altered ROM backing.
use super::{
    attach_dataset,
    direct_byte_copy_tests::{flip, input, same_state},
    indexed_reader_bank_tests::{execute_until, initialize},
    map_height_digits_tests::install_transfer_queue_code,
    read_dataset,
    screen_update_table_tests::check_boundaries,
    SCREEN_CURVE_BANK_ID,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::SourceRange,
    Bus, Machine,
};
use serde::Deserialize;
use std::{collections::BTreeSet, fs, sync::Arc};

const BASE: u32 = 0x0800_0000;
const FIRST_TASK: u32 = 0x0201_0000;
const SECOND_TASK: u32 = 0x0201_1000;
const ACTOR: u32 = 0x0201_2000;
const COPY_TASK: u32 = 0x0201_3000;
const COPY_RETURN: u32 = 0x0811_93c4;
const PROOF_SHA: &str = "7a947c1797d23b0d20dd2f88415475c4a2dfda4d4e3f9ad123a1e70e9df7afa8";

#[derive(Deserialize)]
struct Record {
    offset: u32,
    axes: [Option<u32>; 2],
}

#[derive(Deserialize)]
struct Controller {
    records: Vec<Record>,
    position_base: u32,
    position_count: u32,
}

#[derive(Deserialize)]
struct Curve {
    offset: u32,
    samples: u16,
    sample_60: u32,
}

#[derive(Deserialize)]
struct Proof {
    controllers: [Controller; 2],
    curves: Vec<Curve>,
    first_last_cursor: u16,
    terminal_copy: [u32; 2],
    new_ranges: Vec<SourceRange>,
}

fn half(rom: &[u8], offset: u32) -> u16 {
    let at = offset as usize;
    u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())
}

fn check_coordinates(machine: &Machine, rom: &[u8], source: u32) {
    assert_eq!(machine.cpu.regs[2], half(rom, source) as i16 as i32 as u32);
    assert_eq!(
        machine.cpu.regs[3],
        half(rom, source + 2) as i16 as i32 as u32
    );
}

fn check_motion(machine: &mut Machine, at: u32, start: u16, end: u16) {
    let distance = end.wrapping_sub(start) as i16;
    assert_eq!(machine.bus.read16(at), start);
    assert_eq!(machine.bus.read16(at + 2), 60);
    assert_eq!(
        machine.bus.read16(at + 6),
        if distance < 0 { 0xffff } else { 1 }
    );
    assert_eq!(machine.bus.read16(at + 10), distance.unsigned_abs());
}

fn check_transform(machine: &mut Machine, rom: &[u8], row: &Record, cursor: u16, rotation: u16) {
    for (axis, at) in row.axes.into_iter().zip([ACTOR + 0x32, ACTOR + 0x34]) {
        let expected = axis.map_or(256, |base| half(rom, base + 2 * u32::from(cursor)));
        assert_eq!(
            machine.bus.read16(at),
            expected,
            "row {:06x}, cursor {cursor}",
            row.offset
        );
    }
    assert_eq!(machine.bus.read16(ACTOR + 0x36), rotation);
}

fn first_controller(machine: &mut Machine, rom: &[u8], controller: &Controller, last: u16) {
    for (index, row) in controller.records.iter().enumerate() {
        let index = index as u32;
        initialize(machine, 0x0811_409c);
        machine.cpu.regs[7] = index;
        machine.cpu.regs[8] = FIRST_TASK;
        execute_until(machine, 0x0811_40b4, 64);
        let state = FIRST_TASK + 8 + 32 * index;
        assert_eq!(machine.cpu.regs[4], BASE + row.offset);
        assert_eq!(machine.cpu.regs[5], state);
        check_coordinates(machine, rom, row.offset);

        // Supply the actor handle that the allocation segment would return.
        machine.bus.write32(state, ACTOR);
        machine.cpu.regs[6] = state + 4;
        machine.cpu.regs[15] = 0x0811_40f4;
        execute_until(machine, 0x0811_4116, 256);
        check_motion(
            machine,
            state + 4,
            half(rom, row.offset),
            half(rom, row.offset + 4),
        );
        check_motion(
            machine,
            state + 0x10,
            half(rom, row.offset + 2),
            half(rom, row.offset + 6),
        );

        machine.bus.write16(FIRST_TASK, 0);
        for cursor in 1..=last {
            initialize(machine, 0x0811_421c);
            machine.cpu.regs[8] = FIRST_TASK;
            execute_until(machine, 0x0811_422a, 64);
            machine.cpu.regs[6] = index;
            execute_until(machine, 0x0811_42ac, 4096);
            assert_eq!(machine.bus.read16(FIRST_TASK), cursor);
            assert_eq!(machine.cpu.regs[0], u32::from(cursor == last));
            check_transform(
                machine,
                rom,
                row,
                cursor,
                half(rom, row.offset + 16).wrapping_mul(cursor),
            );
        }
    }

    for index in 0..controller.position_count {
        let offset = controller.position_base + 8 * index;
        initialize(machine, 0x0811_4122);
        machine.cpu.regs[7] = index;
        machine.cpu.regs[8] = FIRST_TASK;
        execute_until(machine, 0x0811_413a, 64);
        assert_eq!(machine.cpu.regs[5], BASE + offset);
        let at = offset as usize;
        let coordinates = [rom[at], rom[at + 1], rom[at + 2]].map(u32::from);
        assert_eq!(machine.cpu.regs[1..4], coordinates);

        let state = machine.cpu.regs[4];
        machine.cpu.regs[6] = state + 4;
        machine.cpu.regs[15] = 0x0811_415a;
        execute_until(machine, 0x0811_4178, 256);
        check_motion(
            machine,
            state + 4,
            u16::from(rom[offset as usize + 1]),
            half(rom, offset + 4),
        );
        check_motion(
            machine,
            state + 0x10,
            u16::from(rom[offset as usize + 2]),
            half(rom, offset + 6),
        );
    }
}

fn second_controller(machine: &mut Machine, rom: &[u8], controller: &Controller, last: u16) {
    for (index, row) in controller.records.iter().enumerate() {
        let index = index as u32;
        initialize(machine, 0x0811_51ce);
        machine.cpu.regs[6] = index;
        machine.cpu.regs[7] = SECOND_TASK;
        execute_until(machine, 0x0811_51e0, 64);
        assert_eq!(machine.cpu.regs[5], BASE + row.offset);
        check_coordinates(machine, rom, row.offset);

        machine.bus.write32(SECOND_TASK + 0x0c + 4 * index, ACTOR);
        machine.cpu.regs[15] = 0x0811_520a;
        execute_until(machine, 0x0811_5238, 256);
        check_motion(
            machine,
            SECOND_TASK + 0x74 + 12 * index,
            half(rom, row.offset),
            half(rom, row.offset + 4),
        );
        check_motion(
            machine,
            SECOND_TASK + 0xec + 12 * index,
            half(rom, row.offset + 2),
            half(rom, row.offset + 6),
        );

        // The original callback checks the palette task after these reads.
        // Seed the cursors to verify the accesses independently of scheduling.
        for cursor in 0..=last {
            machine.bus.write16(SECOND_TASK, cursor);
            initialize(machine, 0x0811_52d8);
            machine.cpu.regs[7] = SECOND_TASK;
            execute_until(machine, 0x0811_52ee, 64);
            machine.cpu.regs[5] = index;
            execute_until(machine, 0x0811_5396, 4096);
            assert_eq!(machine.bus.read16(SECOND_TASK), cursor);
            check_transform(
                machine,
                rom,
                row,
                cursor,
                half(rom, row.offset + 16).wrapping_mul(cursor + 1),
            );
        }
    }

    for index in 0..controller.position_count {
        let offset = controller.position_base + 8 * index;
        initialize(machine, 0x0811_5244);
        machine.cpu.regs[6] = index;
        machine.cpu.regs[7] = SECOND_TASK;
        execute_until(machine, 0x0811_5252, 64);
        assert_eq!(machine.cpu.regs[4], BASE + offset);
        check_coordinates(machine, rom, offset);

        machine.cpu.regs[15] = 0x0811_5272;
        execute_until(machine, 0x0811_52a4, 256);
        check_motion(
            machine,
            SECOND_TASK + 0x268 + 12 * index,
            half(rom, offset),
            half(rom, offset + 4),
        );
        check_motion(
            machine,
            SECOND_TASK + 0x328 + 12 * index,
            half(rom, offset + 2),
            half(rom, offset + 6),
        );
    }
}

fn copy_terminal_record(machine: &mut Machine, rom: &[u8], [start, end]: [u32; 2]) {
    let destination = (COPY_TASK - 0x0200_0000 + 0x1c) as usize;
    let stop = destination + (end - start) as usize;
    machine.bus.ewram[destination - 2..stop + 2].fill(0x5a);
    install_transfer_queue_code(machine, rom);
    initialize(machine, 0x0811_93b4);
    machine.cpu.regs[6] = COPY_TASK;
    execute_until(machine, COPY_RETURN, 4096);
    assert_eq!(
        &machine.bus.ewram[destination..stop],
        &rom[start as usize..end as usize]
    );
    assert_eq!(&machine.bus.ewram[destination - 2..destination], &[0x5a; 2]);
    assert_eq!(&machine.bus.ewram[stop..stop + 2], &[0x5a; 2]);
}

#[test]
#[ignore = "Exact EUR ROM, E641 source proof and e372/e373 datasets"]
fn european_screen_curve_bank_feeds_original_controllers() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let proof_bytes = fs::read(input("FFTA_SCREEN_CURVE_PROOF")).unwrap();
    assert_eq!(digest(&proof_bytes), PROOF_SHA);
    let proof: Proof = serde_json::from_slice(&proof_bytes).unwrap();
    let [first, second] = &proof.controllers;
    let last = proof.first_last_cursor;
    assert_eq!(
        (first.records.len(), second.records.len(), last),
        (11, 10, 60)
    );
    assert_eq!((first.position_count, second.position_count), (14, 16));
    assert_eq!(proof.terminal_copy[1] - proof.terminal_copy[0], 20);
    let aliases: BTreeSet<_> = first
        .records
        .iter()
        .chain(&second.records)
        .flat_map(|row| row.axes.into_iter().flatten())
        .map(|axis| axis + 2 * u32::from(last))
        .collect();
    assert_eq!(aliases.len(), 7);
    for (index, curve) in proof.curves.iter().enumerate() {
        assert_eq!(curve.samples, last);
        let next = proof
            .curves
            .get(index + 1)
            .map_or(proof.terminal_copy[0], |next| next.offset);
        assert_eq!(curve.sample_60, next);
        assert_eq!(curve.offset + 2 * u32::from(curve.samples), curve.sample_60);
    }
    assert_eq!(
        aliases,
        proof.curves.iter().map(|curve| curve.sample_60).collect()
    );

    let mut previous = read_dataset(&input("FFTA_SCREEN_CURVE_PREVIOUS")).unwrap();
    let mut dataset = read_dataset(&input("FFTA_SCREEN_CURVE_DATASET")).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (372, 371)
    );
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (373, 372)
    );
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_dataset(&mut managed, &previous).unwrap();
    assert!(managed
        .bus
        .rom_resources()
        .unwrap()
        .screen_curve_bank_bus_bytes()
        .is_none());
    attach_dataset(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    previous.manifest.extractor_revision = 373;
    previous.manifest.decoder_revision = 372;
    assert!(attach_dataset(&mut managed, &previous).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|item| item.id == SCREEN_CURVE_BANK_ID)
        .unwrap();
    assert_eq!(resource.sources, proof.new_ranges);
    let packed = dataset.screen_curve_bank_bytes.as_mut().unwrap();
    let source_bytes = packed.len();
    assert_eq!(source_bytes, 1450);
    packed[0] ^= 0x80;
    resource.sha256 = digest(packed);
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    // Poison the complete first position array, including the earlier owner.
    let shared = first.position_base..first.position_base + 8 * first.position_count;
    let altered: Vec<_> = std::iter::once((shared.start, shared.end - shared.start))
        .chain(
            proof
                .new_ranges
                .iter()
                .filter(|span| !shared.contains(&span.offset))
                .map(|span| (span.offset, span.length)),
        )
        .collect();
    for &(offset, length) in &altered {
        flip(&mut managed, offset, length);
    }
    for at in shared {
        assert_eq!(managed.bus.read8(BASE + at), rom[at as usize]);
    }
    let boundaries = check_boundaries(&mut raw.bus, &mut managed.bus, &proof.new_ranges);
    let before = owner.screen_curve_bank_bus_bytes().unwrap();
    for machine in [&mut raw, &mut managed] {
        first_controller(machine, &rom, first, last);
        second_controller(machine, &rom, second, last);
        copy_terminal_record(machine, &rom, proof.terminal_copy);
    }
    let original_reads = owner.screen_curve_bank_bus_bytes().unwrap() - before;
    assert!(original_reads > 0);
    for (offset, length) in altered {
        flip(&mut managed, offset, length);
    }
    same_state(&raw, &managed, COPY_RETURN);

    let result = serde_json::json!({
        "status": "PASS",
        "source_bytes": source_bytes,
        "first_cursor_steps": first.records.len() * usize::from(last),
        "second_seeded_cursor_steps": second.records.len() * (usize::from(last) + 1),
        "position_rows": first.position_count + second.position_count,
        "terminal_aliases": aliases.len(),
        "terminal_copy_bytes": proof.terminal_copy[1] - proof.terminal_copy[0],
        "mixed_mirror_boundary_checks": boundaries,
        "original_read_bytes": original_reads,
        "predecessor_attached": true,
        "invalid_candidates_rejected": 2,
        "full_state_and_clock_equal": true,
    });
    fs::write(
        input("FFTA_SCREEN_CURVE_RESULT"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
