//! Verify the original direct and stored-language tile attribute readers.
use super::{
    attach_dataset,
    direct_byte_copy_tests::{flip, input, same_state},
    indexed_reader_bank_tests::{execute_until, initialize},
    read_dataset,
    screen_update_table_tests::check_boundaries,
    INDEXED_TILE_ATTRIBUTES_ID,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::SourceRange,
    Bus, Machine,
};
use serde::Deserialize;
use std::{fs, sync::Arc};

const BASE: u32 = 0x0800_0000;
const RETURN: u32 = 0x0800_0300;
const LANGUAGE: u32 = 0x0200_1f6e;
const PROOF_SHA: &str = "0f697162673b355011edec3baf2b3fc07a1ecbbfc876f14cb929755dddb91987";

#[derive(Deserialize)]
struct Table {
    offset: u32,
    end: u32,
    programs: u32,
}

#[derive(Deserialize)]
struct Proof {
    direct: [Table; 2],
    localized: [Table; 5],
    new_ranges: Vec<SourceRange>,
}

fn attribute(machine: &mut Machine, entry: u32, input: u32) -> u32 {
    initialize(machine, entry);
    machine.cpu.regs[0] = input;
    execute_until(machine, RETURN, 64);
    machine.cpu.regs[0]
}

fn original_attributes(machine: &mut Machine, rom: &[u8], proof: &Proof) -> (u32, usize) {
    let mut direct_cases = 0;
    for (entry, table) in [0x080c_f3cc, 0x080c_f3fc].into_iter().zip(&proof.direct) {
        assert_eq!(table.end - table.offset, 4 + 2 * table.programs);
        for index in 0..table.programs {
            let source = table.offset + 4 + 2 * index;
            assert_eq!(
                attribute(machine, entry, index),
                u32::from(rom[source as usize] >> 4)
            );
            direct_cases += 1;
        }
    }
    let mut localized_cases = 0;
    for (language, table) in proof.localized.iter().enumerate() {
        machine.bus.write8(LANGUAGE, language as u8);
        assert_eq!(table.end - table.offset, 4 + 4 * table.programs);
        for (input, index) in [(2, 0), (44, table.programs - 2), (83, table.programs - 1)] {
            let source = table.offset + 4 + 4 * index;
            assert_eq!(
                attribute(machine, 0x080c_f45c, input),
                u32::from(rom[source as usize] >> 4)
            );
            localized_cases += 1;
        }
    }
    (direct_cases, localized_cases)
}

#[test]
#[ignore = "Exact EUR ROM, E639 source proof and e371/e372 datasets"]
fn european_indexed_tile_attributes_feed_original_getters() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let proof_bytes = fs::read(input("FFTA_INDEXED_TILE_ATTRIBUTES_PROOF")).unwrap();
    assert_eq!(digest(&proof_bytes), PROOF_SHA);
    let proof: Proof = serde_json::from_slice(&proof_bytes).unwrap();

    let mut previous = read_dataset(&input("FFTA_INDEXED_TILE_ATTRIBUTES_PREVIOUS")).unwrap();
    let mut dataset = read_dataset(&input("FFTA_INDEXED_TILE_ATTRIBUTES_DATASET")).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (371, 370)
    );
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (372, 371)
    );
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_dataset(&mut managed, &previous).unwrap();
    assert!(managed
        .bus
        .rom_resources()
        .unwrap()
        .indexed_tile_attributes_bus_bytes()
        .is_none());
    attach_dataset(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    previous.manifest.extractor_revision = 372;
    previous.manifest.decoder_revision = 371;
    assert!(attach_dataset(&mut managed, &previous).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|item| item.id == INDEXED_TILE_ATTRIBUTES_ID)
        .unwrap();
    assert_eq!(resource.sources, proof.new_ranges);
    let packed = dataset.indexed_tile_attributes_bytes.as_mut().unwrap();
    assert_eq!(packed.len(), 1187);
    packed[0] ^= 0x20;
    resource.sha256 = digest(packed);
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    // Cover all seven tables, the five getter literals and the original
    // language directory. This includes the existing E399 byte and archive owner.
    let altered: Vec<_> = [
        0x0cf3dc..0x0cf3e0,
        0x0cf40c..0x0cf410,
        0x0cf490..0x0cf49c,
        0x397d94..0x397da8,
    ]
    .into_iter()
    .chain(
        proof
            .direct
            .iter()
            .chain(&proof.localized)
            .map(|table| table.offset..table.end),
    )
    .collect();
    for span in &altered {
        flip(&mut managed, span.start, span.end - span.start);
        for at in span.clone() {
            assert_eq!(
                managed.bus.read8(BASE + at),
                rom[at as usize],
                "attribute byte {at:06x}"
            );
        }
    }
    let boundaries = check_boundaries(&mut raw.bus, &mut managed.bus, &proof.new_ranges);
    let before = owner.indexed_tile_attributes_bus_bytes().unwrap();
    let cases = original_attributes(&mut raw, &rom, &proof);
    assert_eq!(cases, (584, 15));
    assert_eq!(original_attributes(&mut managed, &rom, &proof), cases);
    let original_reads = owner.indexed_tile_attributes_bus_bytes().unwrap() - before;
    assert_eq!(original_reads, 2791);
    for span in &altered {
        flip(&mut managed, span.start, span.end - span.start);
    }
    same_state(&raw, &managed, RETURN);

    let result = serde_json::json!({
        "status": "PASS",
        "source_bytes": 1187,
        "direct_getter_cases": cases.0,
        "localized_getter_cases": cases.1,
        "languages": proof.localized.len(),
        "mixed_mirror_boundary_checks": boundaries,
        "original_read_bytes": original_reads,
        "predecessor_attached": true,
        "invalid_candidates_rejected": 2,
        "full_state_and_clock_equal": true,
    });
    fs::write(
        input("FFTA_INDEXED_TILE_ATTRIBUTES_RESULT"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
