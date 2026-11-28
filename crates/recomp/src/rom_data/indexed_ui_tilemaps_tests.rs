//! Verify every original E628 map copier and directory slot.
use super::{
    attach_dataset,
    direct_byte_copy_tests::{flip, input, same_state},
    indexed_reader_bank_tests::{execute_until, initialize},
    read_dataset,
    screen_update_table_tests::check_boundaries,
    INDEXED_UI_TILEMAPS_ID,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::SourceRange,
    Bus, Machine,
};
use serde::Deserialize;
use std::{collections::BTreeMap, fs, sync::Arc};

const BASE: u32 = 0x0800_0000;
const DESTINATION: u32 = 0x0201_8000;
const TASK_ROOT: u32 = 0x0300_2810;
const TASK: u32 = 0x0201_a000;
const SELECTOR: u32 = TASK + 0x620;
const STRIDE: u32 = 27;
const TILE_ADDEND: u16 = 87;
const SENTINEL: u16 = 0x5a5a;
const PROOF_SHA: &str = "d72e37815c41e945bfa1c245ba560cd8fd6b8a8d8c2ba1fba4bd9a2ec40cacfe";

#[derive(Deserialize)]
struct Direct {
    pc: u32,
    call_pc: u32,
    source: u32,
    rows: u32,
    cols: u32,
}

#[derive(Deserialize)]
struct Indexed {
    pc: u32,
    call_pc: u32,
    directory: u32,
    rows: u32,
    cols: u32,
}

#[derive(Deserialize)]
struct Proof {
    direct: [Direct; 3],
    indexed: [Indexed; 5],
    directories: BTreeMap<u32, [u32; 10]>,
    physical_cells: u32,
    new_ranges: Vec<SourceRange>,
}

fn check_copy(machine: &mut Machine, rom: &[u8], call: u32, source: u32, rows: u32, cols: u32) {
    let end = DESTINATION + 4 * STRIDE;
    for at in (DESTINATION - 2..=end).step_by(2) {
        machine.bus.write16(at, SENTINEL);
    }
    machine.cpu.regs[1] = DESTINATION;
    execute_until(machine, BASE + call, 64);
    assert_eq!(machine.cpu.regs[0], BASE + source);
    assert_eq!(machine.cpu.regs[2], rows);
    assert_eq!(machine.cpu.regs[3], cols);
    assert_eq!(machine.bus.read32(machine.cpu.regs[13]), STRIDE);
    assert_eq!(
        machine.bus.read32(machine.cpu.regs[13] + 4),
        u32::from(TILE_ADDEND)
    );
    execute_until(machine, BASE + call + 4, 1500);

    for y in 0..2 {
        for x in 0..STRIDE {
            let expected = if y < rows && x < cols {
                let at = (source + 2 * (y * cols + x)) as usize;
                u16::from_le_bytes(rom[at..at + 2].try_into().unwrap()).wrapping_add(TILE_ADDEND)
            } else {
                SENTINEL
            };
            assert_eq!(
                machine.bus.read16(DESTINATION + 2 * (y * STRIDE + x)),
                expected
            );
        }
    }
    assert_eq!(machine.bus.read16(DESTINATION - 2), SENTINEL);
    assert_eq!(machine.bus.read16(end), SENTINEL);
}

fn original_copies(machine: &mut Machine, rom: &[u8], proof: &Proof) -> u32 {
    let mut copied_cells = 0;
    for row in &proof.direct {
        initialize(machine, BASE + row.pc);
        machine.cpu.regs[7] = STRIDE;
        check_copy(machine, rom, row.call_pc, row.source, row.rows, row.cols);
        copied_cells += row.rows * row.cols;
    }

    machine.bus.write32(TASK_ROOT, TASK);
    for row in &proof.indexed {
        for (slot, &source) in proof.directories[&row.directory].iter().enumerate() {
            machine.bus.write8(SELECTOR, slot as u8);
            let entry = match row.pc {
                0x03a816 => 0x03a812,
                0x03aa34 => 0x03aa30,
                _ => row.pc,
            };
            initialize(machine, BASE + entry);
            match row.pc {
                0x03a816 => machine.cpu.regs[2] = SELECTOR,
                0x03a858 | 0x03a8a2 => {
                    machine.cpu.regs[8] = STRIDE;
                    machine.cpu.regs[9] = u32::from(TILE_ADDEND);
                }
                0x03aa34 => {
                    machine.cpu.regs[0] = SELECTOR;
                    machine.cpu.regs[5] = STRIDE;
                }
                0x03aab2 => machine.cpu.regs[5] = TASK_ROOT,
                _ => panic!("Unexpected directory selector at {:06x}", row.pc),
            }
            check_copy(machine, rom, row.call_pc, source, row.rows, row.cols);
            copied_cells += row.rows * row.cols;
        }
    }
    copied_cells
}

#[test]
#[ignore = "Exact EUR ROM, E628 source proof and e366/e367 datasets"]
fn european_indexed_ui_tilemaps_feed_original_copies() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let proof_bytes = fs::read(input("FFTA_INDEXED_UI_TILEMAP_PROOF")).unwrap();
    assert_eq!(digest(&proof_bytes), PROOF_SHA);
    let proof: Proof = serde_json::from_slice(&proof_bytes).unwrap();
    assert_eq!(proof.physical_cells, 855);
    assert_eq!(proof.directories.len(), 4);

    let mut previous = read_dataset(&input("FFTA_INDEXED_UI_TILEMAP_PREVIOUS")).unwrap();
    let mut dataset = read_dataset(&input("FFTA_INDEXED_UI_TILEMAP_DATASET")).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (366, 365)
    );
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (367, 366)
    );

    let mut raw = Machine::new(rom.clone());
    let mut managed = Machine::new(rom.clone());
    attach_dataset(&mut managed, &previous).unwrap();
    assert!(managed
        .bus
        .rom_resources()
        .unwrap()
        .indexed_ui_tilemaps_bus_bytes()
        .is_none());
    attach_dataset(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    previous.manifest.extractor_revision = 367;
    previous.manifest.decoder_revision = 366;
    assert!(attach_dataset(&mut managed, &previous).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|item| item.id == INDEXED_UI_TILEMAPS_ID)
        .unwrap();
    assert_eq!(resource.sources, proof.new_ranges);
    let packed = dataset.indexed_ui_tilemaps_bytes.as_mut().unwrap();
    assert_eq!(packed.len(), 1902);
    packed[0] ^= 1;
    resource.sha256 = digest(packed);
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    for span in &proof.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    let boundaries = check_boundaries(&mut raw.bus, &mut managed.bus, &proof.new_ranges);
    let before = owner.indexed_ui_tilemaps_bus_bytes().unwrap();
    let copies = original_copies(&mut raw, &rom, &proof);
    assert_eq!(copies, 1485);
    assert_eq!(original_copies(&mut managed, &rom, &proof), copies);
    let managed_bytes = owner.indexed_ui_tilemaps_bus_bytes().unwrap() - before;
    assert_eq!(managed_bytes, 3382);
    for span in &proof.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    same_state(&raw, &managed, 0x0803_aada);

    let result = serde_json::json!({
        "status": "PASS",
        "direct_paths": proof.direct.len(),
        "indexed_paths": proof.indexed.len() * 10,
        "physical_cells": proof.physical_cells,
        "copied_cells": copies,
        "source_bytes": 1902,
        "mixed_mirror_boundary_checks": boundaries,
        "original_read_bytes": managed_bytes,
        "predecessor_attached": true,
        "invalid_candidates_rejected": 2,
        "full_state_and_clock_equal": true,
    });
    fs::write(
        input("FFTA_INDEXED_UI_TILEMAP_RESULT"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
