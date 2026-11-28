//! Verify all original E632 queue producers and their DMA output.
use super::{
    attach_dataset,
    direct_byte_copy_tests::{flip, input, same_state},
    indexed_reader_bank_tests::{execute_until, initialize},
    map_height_digits_tests::{drain_transfer_queue, install_transfer_queue_code},
    read_dataset,
    screen_update_table_tests::check_boundaries,
    QUEUED_UI_MAPS_ID,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::SourceRange,
    Bus, Machine,
};
use serde::Deserialize;
use std::{fs, sync::Arc};

const BASE: u32 = 0x0800_0000;
const TASK: u32 = 0x0203_0000;
const QUEUE: u32 = 0x0300_2bcc;
const SCREEN_STATE: u32 = 0x0300_08b0;
const AUX_STATE: u32 = 0x0300_0990;
const DESTINATION: u32 = 0x0300_4000;
const DESTINATION_INDEX: usize = 0x4000;
const DESTINATION_STRIDE: usize = 64;
const BUFFER_BYTES: usize = 16 * DESTINATION_STRIDE + 2;
const SENTINEL: u8 = 0x5a;
const PROOF_SHA: &str = "46481134b1dd7d42643470d759c2a76e126dda1e70bcd0339ab0e6a174ed3bd7";

#[derive(Deserialize)]
struct Case {
    pc: u32,
    call_pc: u32,
    source: u32,
}

#[derive(Deserialize)]
struct Map {
    offset: u32,
    width: usize,
    rows: usize,
    source_stride: usize,
    destination_stride: usize,
    live_rows: bool,
}

#[derive(Deserialize)]
struct Proof {
    cases: [Case; 9],
    maps: [Map; 5],
    physical_cells: usize,
    new_ranges: Vec<SourceRange>,
}

fn queue_case(machine: &mut Machine, case: &Case, map: &Map, rows: usize) -> u32 {
    let entry = if case.pc == 0x12d746 {
        0x12d740
    } else {
        case.pc
    };
    initialize(machine, BASE + entry);
    machine.cpu.regs[0] = TASK;
    let (state, displacement, node) = match case.pc {
        0x0951be => (SCREEN_STATE, 652, TASK),
        0x12d4e0 | 0x12d654 => {
            machine.cpu.regs[5] = map.destination_stride as u32;
            machine.cpu.regs[6] = map.source_stride as u32;
            machine.cpu.regs[7] = TASK;
            machine.cpu.regs[10] = AUX_STATE;
            machine.bus.write16(TASK + 16, rows as u16);
            (AUX_STATE, 140, TASK)
        }
        0x12d740 | 0x12d746 => {
            machine.cpu.regs[0] = u32::from(case.pc == 0x12d746);
            machine.cpu.regs[7] = TASK;
            machine.bus.write8(TASK + 932, 0);
            (AUX_STATE, 716, TASK + 708)
        }
        _ => (SCREEN_STATE, 512, TASK),
    };
    machine.bus.write32(state + 20, DESTINATION - displacement);
    execute_until(machine, BASE + case.call_pc, 96);
    assert_eq!(machine.cpu.regs[0], node);
    assert_eq!(machine.cpu.regs[1], BASE + map.offset);
    assert_eq!(machine.cpu.regs[2], DESTINATION);
    assert_eq!(machine.cpu.regs[3], map.width as u32);
    execute_until(machine, BASE + case.call_pc + 4, 128);
    node
}

fn compare_dma(machine: &mut Machine, rom: &[u8], map: &Map, rows: usize, node: u32) {
    assert_eq!(machine.bus.read32(QUEUE), node);
    assert_eq!(machine.bus.read32(node + 4), BASE + map.offset);
    assert_eq!(machine.bus.read32(node + 8), DESTINATION);
    assert_eq!(
        machine.bus.read32(node + 12),
        0x8000_0000 | map.width as u32
    );
    assert_eq!(machine.bus.read16(node + 16), rows as u16);
    assert_eq!(machine.bus.read16(node + 18), map.source_stride as u16);
    assert_eq!(machine.bus.read16(node + 20), map.destination_stride as u16);
    let start = DESTINATION_INDEX - 1;
    let end = start + BUFFER_BYTES;
    machine.bus.iwram[start..end].fill(SENTINEL);
    drain_transfer_queue(machine);

    let mut expected = [SENTINEL; BUFFER_BYTES];
    for row in 0..rows {
        let from = map.offset as usize + row * map.source_stride;
        let to = 1 + row * map.destination_stride;
        expected[to..to + 2 * map.width].copy_from_slice(&rom[from..from + 2 * map.width]);
    }
    assert_eq!(&machine.bus.iwram[start..end], &expected);
}

fn original_copies(machine: &mut Machine, rom: &[u8], proof: &Proof) -> usize {
    install_transfer_queue_code(machine, rom);
    let mut copied = 0;
    for case in &proof.cases {
        let map = proof
            .maps
            .iter()
            .find(|map| map.offset == case.source)
            .unwrap();
        assert_eq!(map.destination_stride, DESTINATION_STRIDE);
        let heights = [1, map.rows];
        for &rows in &heights[usize::from(!map.live_rows)..] {
            machine.bus.write32(QUEUE, 0);
            let node = queue_case(machine, case, map, rows);
            compare_dma(machine, rom, map, rows, node);
            copied += rows * map.width;
        }
    }
    copied
}

#[test]
#[ignore = "Exact EUR ROM, E632 source proof and e368/e369 datasets"]
fn european_queued_ui_maps_feed_original_dma() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let proof_bytes = fs::read(input("FFTA_QUEUED_UI_MAP_PROOF")).unwrap();
    assert_eq!(digest(&proof_bytes), PROOF_SHA);
    let proof: Proof = serde_json::from_slice(&proof_bytes).unwrap();
    assert_eq!(proof.physical_cells, 624);

    let mut previous = read_dataset(&input("FFTA_QUEUED_UI_MAP_PREVIOUS")).unwrap();
    let mut dataset = read_dataset(&input("FFTA_QUEUED_UI_MAP_DATASET")).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (368, 367)
    );
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (369, 368)
    );

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_dataset(&mut managed, &previous).unwrap();
    assert!(managed
        .bus
        .rom_resources()
        .unwrap()
        .queued_ui_maps_bus_bytes()
        .is_none());
    attach_dataset(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    previous.manifest.extractor_revision = 369;
    previous.manifest.decoder_revision = 368;
    assert!(attach_dataset(&mut managed, &previous).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|item| item.id == QUEUED_UI_MAPS_ID)
        .unwrap();
    assert_eq!(resource.sources, proof.new_ranges);
    let packed = dataset.queued_ui_maps_bytes.as_mut().unwrap();
    assert_eq!(packed.len(), 1284);
    packed[0] ^= 1;
    resource.sha256 = digest(packed);
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    for span in &proof.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    let boundaries = check_boundaries(&mut raw.bus, &mut managed.bus, &proof.new_ranges);
    let before = owner.queued_ui_maps_bus_bytes().unwrap();
    let copied = original_copies(&mut raw, &rom, &proof);
    assert_eq!(copied, 1326);
    assert_eq!(original_copies(&mut managed, &rom, &proof), copied);
    let managed_bytes = owner.queued_ui_maps_bus_bytes().unwrap() - before;
    assert!(managed_bytes >= 2700);
    for span in &proof.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    same_state(&raw, &managed, 0x0300_7f00);

    let result = serde_json::json!({
        "status": "PASS",
        "queue_paths": proof.cases.len(),
        "dma_cases": 11,
        "physical_cells": proof.physical_cells,
        "copied_cells": copied,
        "source_bytes": 1284,
        "mixed_mirror_boundary_checks": boundaries,
        "original_read_bytes": managed_bytes,
        "predecessor_attached": true,
        "invalid_candidates_rejected": 2,
        "full_state_and_clock_equal": true,
    });
    fs::write(
        input("FFTA_QUEUED_UI_MAP_RESULT"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
