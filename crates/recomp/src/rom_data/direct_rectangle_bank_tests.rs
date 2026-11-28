//! Run the original EUR rectangle readers with altered ROM backing.
use super::{
    attach_dataset,
    direct_byte_copy_tests::{flip, input, same_state},
    indexed_reader_bank_tests::{execute_until, initialize},
    read_dataset,
    screen_update_table_tests::check_boundaries,
    tilemap_type2_tests::install_rectangle_copy,
    DIRECT_RECTANGLE_BANK_ID,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::SourceRange,
    Bus, Machine,
};
use serde::Deserialize;
use std::{fs, ops::Range, sync::Arc};

const BASE: u32 = 0x0800_0000;
const CONTEXT: u32 = 0x0201_0000;
const COPY: u32 = BASE + 0x0d80e0;
const RETURN: u32 = BASE + 0x300;
const DESTINATION: Range<usize> = 0x385e..0x3c62;
const SENTINEL: u8 = 0x5a;
const PROOF_SHA: &str = "59b7c6586e4a4327e5628b7c5e0ceeeb0bc8128c519a5c769aedea726eb26214";

#[derive(Deserialize)]
struct Reference {
    pc: u32,
    target: u32,
    register: usize,
}

#[derive(Deserialize)]
struct Rectangle {
    offset: u32,
    height: usize,
    width: usize,
}

#[derive(Deserialize)]
struct Directory {
    lookup: u32,
    offset: usize,
    calls: Vec<u32>,
    targets: Vec<u32>,
}

#[derive(Deserialize)]
struct Proof {
    references: Vec<Reference>,
    rectangles: Vec<Rectangle>,
    residual_rectangles: Vec<Rectangle>,
    directories: Vec<Directory>,
    new_ranges: Vec<SourceRange>,
}

impl Proof {
    fn rectangle(&self, offset: u32) -> &Rectangle {
        self.rectangles
            .iter()
            .find(|rectangle| rectangle.offset == offset)
            .unwrap()
    }
}

fn map_destination(bank: usize, row: usize) -> usize {
    0x3862 + bank * 0x200 + 32 * row - DESTINATION.start
}

fn expect_rows(
    expected: &mut [u8],
    rom: &[u8],
    rectangle: &Rectangle,
    rows: Range<usize>,
    destination: usize,
) {
    assert!(rows.end <= rectangle.height);
    let width = 2 * rectangle.width;
    for (index, row) in rows.enumerate() {
        let from = rectangle.offset as usize + 2 + row * width;
        let to = destination + 32 * index;
        expected[to..to + width].copy_from_slice(&rom[from..from + width]);
    }
}

fn original_full_copies(machine: &mut Machine, rom: &[u8], proof: &Proof) {
    for (index, rectangle) in proof.residual_rectangles.iter().enumerate() {
        let bank = index % 2;
        let mut expected = vec![SENTINEL; DESTINATION.len()];
        expect_rows(
            &mut expected,
            rom,
            rectangle,
            0..rectangle.height,
            map_destination(bank, 0),
        );
        machine.bus.iwram[DESTINATION].fill(SENTINEL);
        initialize(machine, COPY);
        machine.cpu.regs[..4].copy_from_slice(&[BASE + rectangle.offset, 0, bank as u32, 0]);
        machine
            .bus
            .write32(machine.cpu.regs[13], rectangle.height as u32);
        execute_until(machine, RETURN, 5000);
        assert!(
            machine.bus.iwram[DESTINATION] == expected,
            "Full rectangle and guards at {:06x}",
            rectangle.offset
        );
    }

    let rectangle = proof.rectangle(0xae8648);
    let mut expected = vec![SENTINEL; DESTINATION.len()];
    expect_rows(
        &mut expected,
        rom,
        rectangle,
        0..rectangle.height,
        map_destination(0, 0),
    );
    machine.bus.iwram[DESTINATION].fill(SENTINEL);
    initialize(machine, BASE + 0x0dc888);
    execute_until(machine, RETURN, 5000);
    assert!(machine.bus.iwram[DESTINATION] == expected);
}

fn original_directory_copies(machine: &mut Machine, rom: &[u8], proof: &Proof) -> (usize, usize) {
    let mut slots = 0;
    let mut copies = 0;
    // The two other physical directories already have verified owners.
    for directory in &proof.directories[..4] {
        for (index, &target) in directory.targets.iter().enumerate() {
            let mut expected = vec![SENTINEL; DESTINATION.len()];
            machine.bus.iwram[DESTINATION].fill(SENTINEL);
            let index_byte = u8::try_from(index).unwrap();
            initialize(machine, BASE + directory.lookup);
            let (bank, row) = match directory.offset {
                0x39ae3c => {
                    machine.cpu.regs[4] = u32::from(index_byte);
                    machine.cpu.regs[10] = CONTEXT;
                    machine.bus.write8(CONTEXT + 2, 6);
                    (1, rom[directory.offset + index * 8 + 4] as usize)
                }
                0x39bcb4 => {
                    machine.cpu.regs[5] = CONTEXT;
                    machine.bus.write8(CONTEXT, index_byte);
                    (0, 0)
                }
                0x39fa20 => {
                    machine.cpu.regs[7] = CONTEXT;
                    machine.bus.write8(CONTEXT + 9, index_byte);
                    (1, rom[0x39fa40 + index] as usize)
                }
                0x3a4a3c => {
                    machine.cpu.regs[6] = CONTEXT;
                    machine.bus.write8(CONTEXT + 2, index_byte);
                    (0, 0)
                }
                _ => panic!("Unexpected residual directory {:06x}", directory.offset),
            };
            for (copy, &call) in directory.calls.iter().enumerate() {
                let destination_row = 6 + copy;
                execute_until(machine, BASE + call, 64);
                assert_eq!(
                    machine.cpu.regs[..4],
                    [
                        BASE + target,
                        destination_row as u32,
                        bank as u32,
                        row as u32
                    ]
                );
                assert_eq!(machine.bus.read32(machine.cpu.regs[13]), 1);
                expect_rows(
                    &mut expected,
                    rom,
                    proof.rectangle(target),
                    row..row + 1,
                    map_destination(bank, destination_row),
                );
                execute_until(machine, BASE + call + 4, 500);
                assert!(
                    machine.bus.iwram[DESTINATION] == expected,
                    "Copy and guards at {call:06x} for index {index}"
                );
                copies += 1;
            }
            slots += 1;
        }
    }
    (slots, copies)
}

fn original_alternate_copies(machine: &mut Machine, rom: &[u8], proof: &Proof) {
    for index in 0..8 {
        let offset = if index == 6 { 0xae79c0 } else { 0xae6e14 };
        let row = rom[0x3a4d5c + index] as usize;
        let mut expected = vec![SENTINEL; DESTINATION.len()];
        expect_rows(
            &mut expected,
            rom,
            proof.rectangle(offset),
            row..row + 1,
            map_destination(1, 8),
        );
        machine.bus.iwram[DESTINATION].fill(SENTINEL);
        initialize(machine, BASE + 0x10950a);
        machine.cpu.regs[4] = 1;
        machine.cpu.regs[6] = CONTEXT;
        machine.bus.write8(CONTEXT + 8, index as u8);
        execute_until(machine, BASE + 0x109530, 32);
        assert_eq!(machine.cpu.regs[..4], [BASE + offset, 8, 1, row as u32]);
        execute_until(machine, BASE + 0x109534, 500);
        assert!(machine.bus.iwram[DESTINATION] == expected);
    }
}

#[test]
#[ignore = "Exact EUR ROM, E651 source proof and e377/e378 datasets"]
fn european_direct_rectangle_bank_feeds_original_readers() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let proof_bytes = fs::read(input("FFTA_DIRECT_RECTANGLE_BANK_PROOF")).unwrap();
    assert_eq!(digest(&proof_bytes), PROOF_SHA);
    let proof: Proof = serde_json::from_slice(&proof_bytes).unwrap();
    assert_eq!(proof.references.len(), 71);
    assert_eq!(proof.rectangles.len(), 50);
    assert_eq!(proof.residual_rectangles.len(), 12);
    assert_eq!(proof.directories.len(), 6);
    let residual_cells = proof
        .residual_rectangles
        .iter()
        .map(|rectangle| rectangle.width * rectangle.height)
        .sum::<usize>();
    assert_eq!(residual_cells, 240);

    let mut previous = read_dataset(&input("FFTA_DIRECT_RECTANGLE_BANK_PREVIOUS")).unwrap();
    let mut dataset = read_dataset(&input("FFTA_DIRECT_RECTANGLE_BANK_DATASET")).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (377, 376)
    );
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (378, 377)
    );
    let mut raw = Machine::new(rom.clone());
    let mut managed = Machine::new(rom.clone());
    attach_dataset(&mut managed, &previous).unwrap();
    assert!(managed
        .bus
        .rom_resources()
        .unwrap()
        .direct_rectangle_bank_bus_bytes()
        .is_none());
    attach_dataset(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    previous.manifest.extractor_revision = 378;
    previous.manifest.decoder_revision = 377;
    assert!(attach_dataset(&mut managed, &previous).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|item| item.id == DIRECT_RECTANGLE_BANK_ID)
        .unwrap();
    assert_eq!(resource.sources, proof.new_ranges);
    let packed = dataset.direct_rectangle_bank_bytes.as_mut().unwrap();
    let source_bytes = packed.len();
    assert_eq!(source_bytes, 860);
    packed[0] ^= 1;
    resource.sha256 = digest(packed);
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    for span in &proof.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    let boundaries = check_boundaries(&mut raw.bus, &mut managed.bus, &proof.new_ranges);
    let before = owner.direct_rectangle_bank_bus_bytes().unwrap();
    for machine in [&mut raw, &mut managed] {
        install_rectangle_copy(machine, &rom);
        for reference in &proof.references {
            initialize(machine, BASE + reference.pc);
            execute_until(machine, BASE + reference.pc + 2, 2);
            assert_eq!(
                machine.cpu.regs[reference.register],
                BASE + reference.target,
                "Original literal at {:06x}",
                reference.pc
            );
        }
        original_full_copies(machine, &rom, &proof);
        assert_eq!(original_directory_copies(machine, &rom, &proof), (21, 31));
        original_alternate_copies(machine, &rom, &proof);
    }
    let original_reads = owner.direct_rectangle_bank_bus_bytes().unwrap() - before;
    assert!(original_reads >= source_bytes as u64);
    for span in &proof.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    same_state(&raw, &managed, BASE + 0x109534);

    let result = serde_json::json!({
        "status": "PASS",
        "source_bytes": source_bytes,
        "literal_readers": proof.references.len(),
        "complete_rectangle_copies": proof.residual_rectangles.len(),
        "residual_cells": residual_cells,
        "original_full_caller": true,
        "directory_slots": 21,
        "directory_copy_calls": 31,
        "alternate_copy_calls": 8,
        "mixed_mirror_boundary_checks": boundaries,
        "original_read_bytes": original_reads,
        "predecessor_attached": true,
        "invalid_candidates_rejected": 2,
        "full_state_and_clock_equal": true,
    });
    fs::write(
        input("FFTA_DIRECT_RECTANGLE_BANK_RESULT"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
