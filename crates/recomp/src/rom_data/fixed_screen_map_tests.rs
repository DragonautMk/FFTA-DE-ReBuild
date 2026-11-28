//! Exercise the original E622 map copies with altered ROM backing.
use super::{
    attach_dataset,
    direct_byte_copy_tests::{flip, input, same_state},
    indexed_reader_bank_tests::{execute_until, initialize},
    read_dataset,
    screen_update_table_tests::check_boundaries,
    FIXED_SCREEN_MAPS_ID,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::SourceRange,
    Bus, Machine,
};
use serde::Deserialize;
use std::{fs, sync::Arc};

const BASE: u32 = 0x0800_0000;
const DESTINATION: u32 = 0x0201_8000;
const STRIDE: u32 = 32;
const SENTINEL: u16 = 0x5a5a;
const PROOF_SHA: &str = "497567aca5ab4690d3628adcc991681799a504755b4ecbb360f41be9032fecc4";

#[derive(Deserialize)]
struct Reader {
    pc: u32,
    call_pc: u32,
    source: u32,
    dest_state: u32,
    count: u32,
    width: u32,
}

#[derive(Deserialize)]
struct Proof {
    readers: [Reader; 10],
    distinct_halfwords: u32,
    new_ranges: Vec<SourceRange>,
}

fn original_copy(machine: &mut Machine, row: &Reader, rom: &[u8]) -> u32 {
    let height = row.count / row.width;
    assert_eq!(height * row.width, row.count);
    initialize(machine, BASE + row.pc);
    machine.bus.write32(row.dest_state + 20, DESTINATION);
    let after = DESTINATION + 2 * STRIDE * height;
    for at in (DESTINATION - 2..=after).step_by(2) {
        machine.bus.write16(at, SENTINEL);
    }
    execute_until(machine, BASE + row.call_pc + 4, 16000);

    for y in 0..height {
        for x in 0..STRIDE {
            let expected = if x < row.width {
                let source = (row.source + 2 * (y * row.width + x)) as usize;
                u16::from_le_bytes(rom[source..source + 2].try_into().unwrap())
            } else {
                SENTINEL
            };
            assert_eq!(
                machine.bus.read16(DESTINATION + 2 * (y * STRIDE + x)),
                expected,
                "map {:06x}, column {x}, row {y}",
                row.source
            );
        }
    }
    assert_eq!(machine.bus.read16(DESTINATION - 2), SENTINEL);
    assert_eq!(machine.bus.read16(after), SENTINEL);
    (STRIDE - row.width) * height + 2
}

#[test]
#[ignore = "Exact EUR ROM, E622 source proof and e363/e364 datasets"]
fn european_fixed_screen_maps_feed_original_copies() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let proof_bytes = fs::read(input("FFTA_FIXED_SCREEN_MAP_PROOF")).unwrap();
    assert_eq!(digest(&proof_bytes), PROOF_SHA);
    let proof: Proof = serde_json::from_slice(&proof_bytes).unwrap();
    assert_eq!(proof.distinct_halfwords, 2939);
    assert_eq!(proof.readers.iter().map(|row| row.count).sum::<u32>(), 3539);

    let mut previous = read_dataset(&input("FFTA_FIXED_SCREEN_MAP_PREVIOUS")).unwrap();
    let mut dataset = read_dataset(&input("FFTA_FIXED_SCREEN_MAP_DATASET")).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (363, 362)
    );
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (364, 363)
    );

    let mut raw = Machine::new(rom.clone());
    let mut managed = Machine::new(rom.clone());
    attach_dataset(&mut managed, &previous).unwrap();
    assert!(managed
        .bus
        .rom_resources()
        .unwrap()
        .fixed_screen_map_bus_bytes()
        .is_none());
    attach_dataset(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    previous.manifest.extractor_revision = 364;
    previous.manifest.decoder_revision = 363;
    assert!(attach_dataset(&mut managed, &previous).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|item| item.id == FIXED_SCREEN_MAPS_ID)
        .unwrap();
    assert_eq!(resource.sources, proof.new_ranges);
    let packed = dataset.fixed_screen_map_bytes.as_mut().unwrap();
    assert_eq!(packed.len(), 5918);
    packed[0] ^= 1;
    resource.sha256 = digest(packed);
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    for span in &proof.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    let boundaries = check_boundaries(&mut raw.bus, &mut managed.bus, &proof.new_ranges);
    let before = owner.fixed_screen_map_bus_bytes().unwrap();
    let mut guard_cells = 0;
    for row in &proof.readers {
        guard_cells += original_copy(&mut raw, row, &rom);
        original_copy(&mut managed, row, &rom);
        assert_eq!(raw.bus.clock, managed.bus.clock, "reader {:06x}", row.pc);
    }
    let managed_bytes = owner.fixed_screen_map_bus_bytes().unwrap() - before;
    assert_eq!(managed_bytes, 7118);
    for span in &proof.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    same_state(
        &raw,
        &managed,
        BASE + proof.readers.last().unwrap().call_pc + 4,
    );

    let result = serde_json::json!({
        "status": "PASS",
        "original_copy_paths": proof.readers.len(),
        "distinct_halfwords": proof.distinct_halfwords,
        "copied_halfwords": 3539,
        "destination_guard_cells": guard_cells,
        "source_bytes": 5918,
        "mixed_mirror_boundary_checks": boundaries,
        "original_read_bytes": managed_bytes,
        "predecessor_attached": true,
        "invalid_candidates_rejected": 2,
        "full_state_and_clock_equal": true,
    });
    fs::write(
        input("FFTA_FIXED_SCREEN_MAP_RESULT"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
