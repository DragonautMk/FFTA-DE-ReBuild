//! Check the original 30-by-20 copy with altered ROM backing.
use super::{
    attach_dataset,
    direct_byte_copy_tests::{flip, input, same_state},
    indexed_reader_bank_tests::{execute_until, initialize},
    read_dataset,
    screen_update_table_tests::check_boundaries,
    FULL_SCREEN_MAP_ID,
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
const CONTEXT: u32 = 0x0300_0e60;
const SHADOW: u32 = 0x0300_0940;
const STOP: u32 = 0x0804_fdc8;
const SENTINEL: u16 = 0x5a5a;
const PROOF_SHA: &str = "426cd10b844e1f5eec84f993a14dcaac3c14291987382a77b3e93c5e69d909f0";

#[derive(Deserialize)]
struct Map {
    reader: u32,
    rows: u32,
    columns: u32,
    pitch: u32,
    source: [u32; 2],
}

#[derive(Deserialize)]
struct Proof {
    fixed_map: Map,
    new_ranges: Vec<SourceRange>,
}

fn original_copy(machine: &mut Machine, rom: &[u8], map: &Map) -> u32 {
    initialize(machine, BASE + map.reader);
    machine.bus.write32(CONTEXT + 20, DESTINATION);
    const SHADOW_BEFORE: u16 = 0x0062;
    machine.bus.write16(SHADOW, SHADOW_BEFORE);
    let end = DESTINATION + map.rows * map.pitch;
    for at in (DESTINATION - 2..=end).step_by(2) {
        machine.bus.write16(at, SENTINEL);
    }
    execute_until(machine, STOP, 8192);

    let stride = map.pitch / 2;
    for row in 0..map.rows {
        for column in 0..stride {
            let expected = if column < map.columns {
                let offset = (map.source[0] + 2 * (row * map.columns + column)) as usize;
                u16::from_le_bytes(rom[offset..offset + 2].try_into().unwrap())
            } else {
                SENTINEL
            };
            let destination = DESTINATION + row * map.pitch + 2 * column;
            assert_eq!(
                machine.bus.read16(destination),
                expected,
                "row {row}, column {column}"
            );
        }
    }
    assert_eq!(machine.bus.read16(DESTINATION - 2), SENTINEL);
    assert_eq!(machine.bus.read16(end), SENTINEL);
    assert_eq!(machine.bus.read16(SHADOW), SHADOW_BEFORE | (1 << 8));
    map.rows * (stride - map.columns) + 2
}

#[test]
#[ignore = "Exact EUR ROM, E643 source proof and e373/e374 datasets"]
fn european_full_screen_map_feeds_original_copy() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let proof_bytes = fs::read(input("FFTA_FULL_SCREEN_MAP_PROOF")).unwrap();
    assert_eq!(digest(&proof_bytes), PROOF_SHA);
    let proof: Proof = serde_json::from_slice(&proof_bytes).unwrap();
    let map = &proof.fixed_map;
    assert_eq!((map.rows, map.columns, map.pitch), (20, 30, 64));
    assert_eq!(map.source[1] - map.source[0], 2 * map.rows * map.columns);

    let mut previous = read_dataset(&input("FFTA_FULL_SCREEN_MAP_PREVIOUS")).unwrap();
    let mut dataset = read_dataset(&input("FFTA_FULL_SCREEN_MAP_DATASET")).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (373, 372)
    );
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (374, 373)
    );
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_dataset(&mut managed, &previous).unwrap();
    assert!(managed
        .bus
        .rom_resources()
        .unwrap()
        .full_screen_map_bus_bytes()
        .is_none());
    attach_dataset(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    previous.manifest.extractor_revision = 374;
    previous.manifest.decoder_revision = 373;
    assert!(attach_dataset(&mut managed, &previous).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|item| item.id == FULL_SCREEN_MAP_ID)
        .unwrap();
    assert_eq!(resource.sources, proof.new_ranges);
    let packed = dataset.full_screen_map_bytes.as_mut().unwrap();
    let source_bytes = packed.len();
    assert_eq!(source_bytes, 1212);
    packed[0] ^= 0x80;
    resource.sha256 = digest(packed);
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    for span in &proof.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    let boundaries = check_boundaries(&mut raw.bus, &mut managed.bus, &proof.new_ranges);
    let before = owner.full_screen_map_bus_bytes().unwrap();
    let guards = original_copy(&mut raw, &rom, map);
    assert_eq!(original_copy(&mut managed, &rom, map), guards);
    let original_reads = owner.full_screen_map_bus_bytes().unwrap() - before;
    assert_eq!(original_reads, source_bytes as u64);
    for span in &proof.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    same_state(&raw, &managed, STOP);

    let result = serde_json::json!({
        "status": "PASS",
        "source_bytes": source_bytes,
        "copied_halfwords": map.rows * map.columns,
        "destination_guard_halfwords": guards,
        "shadow_flag_matched": true,
        "mixed_mirror_boundary_checks": boundaries,
        "original_read_bytes": original_reads,
        "predecessor_attached": true,
        "invalid_candidates_rejected": 2,
        "full_state_and_clock_equal": true,
    });
    fs::write(
        input("FFTA_FULL_SCREEN_MAP_RESULT"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
