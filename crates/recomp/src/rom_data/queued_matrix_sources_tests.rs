//! Compare the original EUR queued matrix paths with altered backing.
use super::{
    attach_dataset,
    direct_byte_copy_tests::{flip, input, same_state},
    indexed_reader_bank_tests::{execute_until, initialize},
    map_height_digits_tests::{drain_transfer_queue, install_transfer_queue_code},
    read_dataset,
    screen_update_table_tests::check_boundaries,
    QUEUED_MATRIX_SOURCES_ID,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::SourceRange,
    Bus, Machine,
};
use serde::Deserialize;
use std::{fs, ops::Range, sync::Arc};

const BASE: u32 = 0x0800_0000;
const LANGUAGE: u32 = 0x0200_1f6e;
const TASK: u32 = 0x0203_0000;
const NODE: u32 = TASK + 0x1000;
const QUEUE: u32 = 0x0300_2bcc;
const SCREEN: u32 = 0x0300_08b0;
const DESTINATION: u32 = 0x0300_4000;
const OUTPUT: Range<usize> = 0x3fff..0x41c1;
const SENTINEL: u8 = 0x5a;
const PITCH: usize = 64;
const RETURN: u32 = BASE + 0x300;
const PROOF_SHA: &str = "ce81038f7df560e187098905ebc20c79aa47914f166efebb89c559efe3f8ad75";

#[derive(Deserialize)]
struct Map {
    offset: u32,
    width: usize,
    rows: usize,
}

impl Map {
    fn cells<'a>(&self, rom: &'a [u8]) -> &'a [u8] {
        let start = self.offset as usize;
        &rom[start..start + 2 * self.width * self.rows]
    }
}

#[derive(Deserialize)]
struct Selector {
    entry: u32,
    stop: u32,
    output_register: Option<usize>,
    stack_offset: Option<u32>,
}

#[derive(Deserialize)]
struct Proof {
    maps: [Map; 3],
    selectors: [Selector; 6],
    new_ranges: Vec<SourceRange>,
}

fn check_dma(machine: &mut Machine, source: u32, cells: &[u8], width: usize) {
    let rows = cells.len() / (width * 2);
    assert_eq!(machine.bus.read32(QUEUE), NODE);
    assert_eq!(machine.bus.read32(NODE + 4), source);
    assert_eq!(machine.bus.read32(NODE + 8), DESTINATION);
    assert_eq!(machine.bus.read32(NODE + 12), 0x8000_0000 | width as u32);
    assert_eq!(machine.bus.read16(NODE + 16), rows as u16);
    assert_eq!(machine.bus.read16(NODE + 18), (width * 2) as u16);
    assert_eq!(machine.bus.read16(NODE + 20), PITCH as u16);

    machine.bus.iwram[OUTPUT].fill(SENTINEL);
    drain_transfer_queue(machine);
    let mut expected = vec![SENTINEL; OUTPUT.len()];
    for (row, bytes) in cells.chunks_exact(width * 2).enumerate() {
        let at = 1 + row * PITCH;
        expected[at..at + bytes.len()].copy_from_slice(bytes);
    }
    assert_eq!(&machine.bus.iwram[OUTPUT], expected.as_slice());
}

fn original_language_and_copies(machine: &mut Machine, rom: &[u8], proof: &Proof) {
    for (site, selector) in proof.selectors.iter().enumerate() {
        for language in 0..5 {
            initialize(machine, BASE + selector.entry);
            machine.bus.write8(LANGUAGE, language);
            execute_until(machine, BASE + selector.stop, 32);
            let map = &proof.maps[if language == 1 { 2 } else { 1 }];
            let source = match (selector.output_register, selector.stack_offset) {
                (Some(register), None) => machine.cpu.regs[register],
                (None, Some(offset)) => machine.bus.read32(machine.cpu.regs[13] + offset),
                _ => panic!("Unsupported selection at {:06x}", selector.entry),
            };
            assert_eq!(source, BASE + map.offset);

            // Replay the first source selection and its queue-argument
            // suffix around the intervening task and destination setup.
            if site == 0 && language <= 1 {
                machine.bus.write32(QUEUE, 0);
                initialize(machine, BASE + 0x02f316);
                machine.cpu.regs[2] = DESTINATION;
                machine.cpu.regs[5] = NODE;
                machine.cpu.regs[6] = source;
                execute_until(machine, BASE + 0x02f32c, 128);
                check_dma(machine, source, &map.cells(rom)[..104], map.width);
            }
        }
    }

    for map in &proof.maps[1..] {
        assert_eq!((map.rows, map.width), (7, 26));
        let source = BASE + map.offset;
        let cells = map.cells(rom);
        machine.bus.iwram[OUTPUT].fill(SENTINEL);
        initialize(machine, BASE + 0x03012a);
        machine.cpu.regs[1] = DESTINATION;
        machine.cpu.regs[3] = source;
        execute_until(machine, BASE + 0x030142, 512);
        let mut expected = vec![SENTINEL; OUTPUT.len()];
        expected[1..105].copy_from_slice(&cells[260..]);
        assert_eq!(&machine.bus.iwram[OUTPUT], expected.as_slice());

        // Supply the full physical bounds to the original queue and DMA
        // helpers so that every cell of both sources is checked.
        machine.bus.write32(QUEUE, 0);
        initialize(machine, BASE + 0x000be4);
        machine.cpu.regs[..4].copy_from_slice(&[NODE, source, DESTINATION, map.width as u32]);
        let stack = machine.cpu.regs[13];
        machine.bus.write32(stack, map.rows as u32);
        machine.bus.write32(stack + 4, 2 * map.width as u32);
        machine.bus.write32(stack + 8, PITCH as u32);
        execute_until(machine, RETURN, 128);
        check_dma(machine, source, cells, map.width);
    }
}

fn original_initializer(machine: &mut Machine, rom: &[u8], map: &Map) {
    assert_eq!((map.width, map.rows), (18, 4));
    let staging = TASK + 808;
    let initializer = staging + 108;
    for offset in 0..252 {
        machine.bus.write8(staging + offset, 0);
    }
    machine.bus.write8(staging - 1, SENTINEL);
    machine.bus.write8(staging + 252, SENTINEL);
    initialize(machine, BASE + 0x02defa);
    machine.cpu.regs[5] = BASE + 0x370eb4;
    machine.cpu.regs[7] = initializer;
    execute_until(machine, BASE + 0x02df06, 1024);

    let mut expected = vec![0; 252];
    expected[108..].copy_from_slice(map.cells(rom));
    for (offset, &byte) in expected.iter().enumerate() {
        assert_eq!(machine.bus.read8(staging + offset as u32), byte);
    }
    assert_eq!(machine.bus.read8(staging - 1), SENTINEL);
    assert_eq!(machine.bus.read8(staging + 252), SENTINEL);

    machine.bus.write32(SCREEN + 20, DESTINATION);
    machine.bus.write32(QUEUE, 0);
    machine.bus.write16(TASK + 6, 0);
    initialize(machine, BASE + 0x02e080);
    machine.cpu.regs[0] = NODE;
    machine.cpu.regs[6] = TASK;
    execute_until(machine, BASE + 0x02e0a4, 128);
    check_dma(machine, staging, &expected, map.width);
}

#[test]
#[ignore = "Exact EUR ROM, E653 source proof and e378/e379 datasets"]
fn european_queued_matrix_sources_feed_original_readers() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let proof_bytes = fs::read(input("FFTA_QUEUED_MATRIX_SOURCES_PROOF")).unwrap();
    assert_eq!(digest(&proof_bytes), PROOF_SHA);
    let proof: Proof = serde_json::from_slice(&proof_bytes).unwrap();
    let cells = proof
        .maps
        .iter()
        .map(|map| map.width * map.rows)
        .sum::<usize>();
    assert_eq!(cells, 436);

    let mut previous = read_dataset(&input("FFTA_QUEUED_MATRIX_SOURCES_PREVIOUS")).unwrap();
    let mut dataset = read_dataset(&input("FFTA_QUEUED_MATRIX_SOURCES_DATASET")).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (378, 377)
    );
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (379, 378)
    );
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_dataset(&mut managed, &previous).unwrap();
    assert!(managed
        .bus
        .rom_resources()
        .unwrap()
        .queued_matrix_sources_bus_bytes()
        .is_none());
    attach_dataset(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    previous.manifest.extractor_revision = 379;
    previous.manifest.decoder_revision = 378;
    assert!(attach_dataset(&mut managed, &previous).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|item| item.id == QUEUED_MATRIX_SOURCES_ID)
        .unwrap();
    assert_eq!(resource.sources, proof.new_ranges);
    let packed = dataset.queued_matrix_sources_bytes.as_mut().unwrap();
    let source_bytes = packed.len();
    assert_eq!(source_bytes, 924);
    packed[0] ^= 1;
    resource.sha256 = digest(packed);
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    for span in &proof.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    let boundaries = check_boundaries(&mut raw.bus, &mut managed.bus, &proof.new_ranges);
    let before = owner.queued_matrix_sources_bus_bytes().unwrap();
    for machine in [&mut raw, &mut managed] {
        install_transfer_queue_code(machine, &rom);
        original_language_and_copies(machine, &rom, &proof);
        original_initializer(machine, &rom, &proof.maps[0]);
    }
    let original_reads = owner.queued_matrix_sources_bus_bytes().unwrap() - before;
    assert!(original_reads >= source_bytes as u64);
    for span in &proof.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    same_state(&raw, &managed, 0x0300_7f00);

    let result = serde_json::json!({
        "status": "PASS",
        "source_bytes": source_bytes,
        "physical_cells": cells,
        "language_cases": proof.selectors.len() * 5,
        "original_queue_suffixes": 3,
        "complete_map_dmas": 2,
        "tail_copy_cells": 104,
        "initializer_bytes": 144,
        "staged_map_cells": 126,
        "mixed_mirror_boundary_checks": boundaries,
        "original_read_bytes": original_reads,
        "predecessor_attached": true,
        "invalid_candidates_rejected": 2,
        "full_state_and_clock_equal": true,
    });
    fs::write(
        input("FFTA_QUEUED_MATRIX_SOURCES_RESULT"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
