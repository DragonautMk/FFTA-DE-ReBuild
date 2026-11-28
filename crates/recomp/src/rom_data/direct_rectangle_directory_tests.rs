//! Compare the E626 directory, presets and original rectangle copies.
use super::{
    attach_dataset,
    direct_byte_copy_tests::{flip, input, same_state},
    indexed_reader_bank_tests::{execute_until, initialize},
    read_dataset,
    screen_update_table_tests::check_boundaries,
    tilemap_type2_tests::install_rectangle_copy,
    DIRECT_RECTANGLE_DIRECTORY_ID,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::SourceRange,
    Bus, Machine,
};
use serde::Deserialize;
use std::{fs, sync::Arc};

const BASE: u32 = 0x0800_0000;
const TASK: u32 = 0x0201_8000;
const PARAMETERS: u32 = TASK + 256;
const DESTINATION: usize = 0x3b22;
const STRIDE: usize = 32;
const BUFFER_END: usize = DESTINATION + 16 * STRIDE + 1;
const SENTINEL: u8 = 0x5a;
const PROOF_SHA: &str = "c21dac38615727c1ed1d0ff3078c9e5ba3ad14adf4759cd0c4d28cc0dc11be42";

#[derive(Deserialize)]
struct Rectangle {
    offset: u32,
    width: usize,
    height: usize,
}

#[derive(Deserialize)]
struct Proof {
    rectangles: [Rectangle; 5],
    preset_count: usize,
    physical_cells: usize,
    new_ranges: Vec<SourceRange>,
}

fn check_copy(machine: &Machine, rom: &[u8], rectangle: &Rectangle, start: usize, count: usize) {
    let mut expected = [SENTINEL; 16 * STRIDE + 2];
    let row_bytes = rectangle.width * 2;
    for row in 0..count {
        let from = rectangle.offset as usize + 2 + (start + row) * row_bytes;
        let to = 1 + row * STRIDE;
        expected[to..to + row_bytes].copy_from_slice(&rom[from..from + row_bytes]);
    }
    assert_eq!(&machine.bus.iwram[DESTINATION - 1..BUFFER_END], &expected);
}

fn original_readers(machine: &mut Machine, rom: &[u8], proof: &Proof) {
    install_rectangle_copy(machine, rom);
    for (index, rectangle) in proof.rectangles.iter().enumerate() {
        assert_eq!((rectangle.width, rectangle.height), (15, 16));
        machine.bus.write8(TASK + 8, index as u8);
        machine.bus.write8(TASK + 9, 1);
        machine.bus.iwram[DESTINATION - 1..BUFFER_END].fill(SENTINEL);
        initialize(machine, 0x0812_40ac);
        machine.cpu.regs[5] = TASK;
        execute_until(machine, 0x0812_40c6, 20);
        assert_eq!(machine.cpu.regs[0], BASE + rectangle.offset);
        execute_until(machine, 0x0812_40ca, 600);
        check_copy(machine, rom, rectangle, 0, 1);

        // The original caller requests only row zero. Enter its shared
        // copy helper with the remaining legal rows to check the full bank.
        machine.bus.iwram[DESTINATION - 1..BUFFER_END].fill(SENTINEL);
        initialize(machine, 0x080d_80e0);
        machine.cpu.regs[..4].copy_from_slice(&[BASE + rectangle.offset, 6, 1, 1]);
        let stack = machine.cpu.regs[13];
        let stop = machine.cpu.regs[14] & !1;
        machine.bus.write32(stack, (rectangle.height - 1) as u32);
        execute_until(machine, stop, 5000);
        check_copy(machine, rom, rectangle, 1, rectangle.height - 1);

        for slot in 0..4 {
            initialize(machine, 0x0812_4108);
            machine.cpu.regs[5] = TASK;
            machine.cpu.regs[6] = slot as u32;
            machine.cpu.regs[7] = PARAMETERS;
            execute_until(machine, 0x0812_4118, 20);
            let expected = u16::from(rom[0x3ac9e4 + index * 4 + slot]) + 80;
            assert_eq!(machine.bus.read16(PARAMETERS + 24), expected);
        }
    }
}

#[test]
#[ignore = "Exact EUR ROM, E626 source proof and e365/e366 datasets"]
fn european_direct_rectangle_directory_feeds_original_readers() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let proof_bytes = fs::read(input("FFTA_DIRECT_RECTANGLE_PROOF")).unwrap();
    assert_eq!(digest(&proof_bytes), PROOF_SHA);
    let proof: Proof = serde_json::from_slice(&proof_bytes).unwrap();
    assert_eq!(proof.preset_count, 20);
    assert_eq!(proof.physical_cells, 1200);

    let mut previous = read_dataset(&input("FFTA_DIRECT_RECTANGLE_PREVIOUS")).unwrap();
    let mut dataset = read_dataset(&input("FFTA_DIRECT_RECTANGLE_DATASET")).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (365, 364)
    );
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (366, 365)
    );

    let mut raw = Machine::new(rom.clone());
    let mut managed = Machine::new(rom.clone());
    attach_dataset(&mut managed, &previous).unwrap();
    assert!(managed
        .bus
        .rom_resources()
        .unwrap()
        .direct_rectangle_directory_bus_bytes()
        .is_none());
    attach_dataset(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    previous.manifest.extractor_revision = 366;
    previous.manifest.decoder_revision = 365;
    assert!(attach_dataset(&mut managed, &previous).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|item| item.id == DIRECT_RECTANGLE_DIRECTORY_ID)
        .unwrap();
    assert_eq!(resource.sources, proof.new_ranges);
    let packed = dataset.direct_rectangle_directory_bytes.as_mut().unwrap();
    assert_eq!(packed.len(), 2458);
    packed[0] ^= 1;
    resource.sha256 = digest(packed);
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    for span in &proof.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    let boundaries = check_boundaries(&mut raw.bus, &mut managed.bus, &proof.new_ranges);
    let before = owner.direct_rectangle_directory_bus_bytes().unwrap();
    original_readers(&mut raw, &rom, &proof);
    original_readers(&mut managed, &rom, &proof);
    let managed_bytes = owner.direct_rectangle_directory_bus_bytes().unwrap() - before;
    assert!(managed_bytes >= 2448);
    for span in &proof.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    same_state(&raw, &managed, 0x0812_4118);

    let result = serde_json::json!({
        "status": "PASS",
        "original_first_row_paths": proof.rectangles.len(),
        "remaining_row_paths": proof.rectangles.len(),
        "physical_cells": proof.physical_cells,
        "preset_reads": proof.preset_count,
        "source_bytes": 2458,
        "mixed_mirror_boundary_checks": boundaries,
        "original_read_bytes": managed_bytes,
        "predecessor_attached": true,
        "invalid_candidates_rejected": 2,
        "full_state_and_clock_equal": true,
    });
    fs::write(
        input("FFTA_DIRECT_RECTANGLE_RESULT"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
