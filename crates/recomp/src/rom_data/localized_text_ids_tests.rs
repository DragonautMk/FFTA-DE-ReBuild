//! Verify the original European localized text ID readers and range resolver.

use super::{
    RANGE_LOOKUP_ID, attach_dataset,
    direct_byte_copy_tests::{flip, input, same_state},
    encode_range_lookup,
    indexed_reader_bank_tests::{execute_until, initialize},
    read_dataset,
    screen_update_table_tests::check_boundaries,
};
use gba_core::{
    Bus, Machine,
    ffta_rom::{EUROPE_SHA256, digest},
    rom_data::{SourceRange, range_lookup::SOURCE},
};
use serde::Deserialize;
use std::{fs, sync::Arc};

const BASE: u32 = 0x0800_0000;
const RETURN: u32 = BASE + 0x300;
const STATE: u32 = 0x0202_0000;
const HANDLE: u32 = 0x0203_0000;
const INDEX: u32 = HANDLE + 4;
const SELECTOR: u32 = HANDLE + 8;
const CATALOG_INDEX: u32 = HANDLE + 10;
const SOURCE_START: u32 = 0x6e9d8a;
const SLOT_COUNT: u32 = 351;
const PROOF_SHA: &str = "d85d1b1106e9bedcfee11fafb837da2a6cbb6059ad281389da2585154f662b14";

#[derive(Deserialize)]
struct Proof {
    entries: u32,
    new_ranges: Vec<SourceRange>,
}

fn read_original_ids(machine: &mut Machine, rom: &[u8]) {
    machine.bus.write32(HANDLE, STATE);
    machine.bus.write32(STATE + 0x94, INDEX);
    machine.bus.write8(STATE + 0x1d46, 0);
    machine.bus.write8(STATE + 0x1d5c, 0);
    for index in 0..SLOT_COUNT {
        let at = (SOURCE_START + 2 * index) as usize;
        let id = u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
        let plus_one = u32::from(id.wrapping_add(1));
        machine.bus.write32(INDEX, index);
        machine.bus.write32(STATE + 0x424, index);
        machine.bus.write16(0x0200_1f3c, index as u16);

        initialize(machine, BASE + 0x029b6c);
        machine.cpu.regs[4] = STATE;
        execute_until(machine, BASE + 0x029b82, 16);
        assert_eq!(machine.cpu.regs[0], u32::from(id));

        initialize(machine, BASE + 0x04d3de);
        machine.cpu.regs[0] = index;
        execute_until(machine, BASE + 0x04d3e6, 16);
        assert_eq!(machine.cpu.regs[4], u32::from(id));

        initialize(machine, BASE + 0x04da4a);
        machine.cpu.regs[2] = index;
        execute_until(machine, BASE + 0x04da52, 16);
        assert_eq!(machine.cpu.regs[6], u32::from(id));

        initialize(machine, BASE + 0x051044);
        machine.cpu.regs[0] = HANDLE;
        execute_until(machine, BASE + 0x051060, 20);
        assert_eq!(machine.cpu.regs[0], plus_one);

        if index <= u8::MAX.into() {
            machine.bus.write8(0x0200_2e64, index as u8);
            initialize(machine, BASE + 0x0540d2);
            machine.cpu.regs[0] = STATE;
            execute_until(machine, BASE + 0x0540ea, 20);
            assert_eq!(machine.cpu.regs[1], u32::from(id));
        }

        initialize(machine, BASE + 0x054c28);
        machine.cpu.regs[0] = STATE;
        execute_until(machine, BASE + 0x054c42, 20);
        assert_eq!(machine.cpu.regs[1], u32::from(id));

        initialize(machine, BASE + 0x068ad4);
        machine.cpu.regs[3] = HANDLE;
        execute_until(machine, BASE + 0x068af0, 20);
        assert_eq!(machine.cpu.regs[2], plus_one);

        initialize(machine, BASE + 0x01abc0);
        machine.cpu.regs[..3].copy_from_slice(&[u32::from(id), SELECTOR, CATALOG_INDEX]);
        execute_until(machine, RETURN, 128);
        let (expected_selector, expected_index) = if id == 0 { (16, 0) } else { (23, id - 1209) };
        assert_eq!(machine.bus.read8(SELECTOR), expected_selector);
        assert_eq!(machine.bus.read16(CATALOG_INDEX), expected_index);
    }
}

#[test]
#[ignore = "Exact EUR ROM, E662 source proof and e382/e383 datasets"]
fn european_localized_text_ids_feed_original_readers() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let proof_bytes = fs::read(input("FFTA_LOCALIZED_TEXT_IDS_PROOF")).unwrap();
    assert_eq!(digest(&proof_bytes), PROOF_SHA);
    let proof: Proof = serde_json::from_slice(&proof_bytes).unwrap();
    assert_eq!(proof.entries, SLOT_COUNT);

    let mut previous = read_dataset(&input("FFTA_LOCALIZED_TEXT_IDS_PREVIOUS")).unwrap();
    let mut dataset = read_dataset(&input("FFTA_LOCALIZED_TEXT_IDS_DATASET")).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (382, 381)
    );
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (383, 382)
    );
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    assert!(
        previous
            .range_lookup
            .as_ref()
            .unwrap()
            .localized_text_ids
            .is_none()
    );
    attach_dataset(&mut managed, &previous).unwrap();
    attach_dataset(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    previous.manifest.extractor_revision = 383;
    previous.manifest.decoder_revision = 382;
    assert!(attach_dataset(&mut managed, &previous).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|item| item.id == RANGE_LOOKUP_ID)
        .unwrap();
    assert_eq!(resource.sources[0].offset, SOURCE);
    assert_eq!(resource.sources[1..], proof.new_ranges);
    let range_lookup = dataset.range_lookup.as_mut().unwrap();
    let table = range_lookup.localized_text_ids.as_mut().unwrap();
    table.ids[0] = 1;
    let digest_index = table
        .ids
        .iter()
        .flat_map(|id| id.to_le_bytes())
        .collect::<Vec<_>>();
    resource.sources.last_mut().unwrap().sha256 = digest(&digest_index);
    let encoded = encode_range_lookup(range_lookup).unwrap();
    resource.sha256 = digest(&encoded);
    dataset.range_lookup_bytes = Some(encoded);
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    for span in &proof.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    flip(&mut managed, SOURCE, 88);
    let boundaries = check_boundaries(&mut raw.bus, &mut managed.bus, &proof.new_ranges);
    flip(&mut managed, SOURCE_START - 2, 2);
    assert_ne!(
        managed.bus.read16(BASE + SOURCE_START - 2),
        raw.bus.read16(BASE + SOURCE_START - 2)
    );
    flip(&mut managed, SOURCE_START - 2, 2);

    let before = owner.range_lookup_stats().unwrap();
    read_original_ids(&mut raw, &rom);
    read_original_ids(&mut managed, &rom);
    let after = owner.range_lookup_stats().unwrap();
    let original_reads = 6 * SLOT_COUNT + 256;
    assert_eq!(
        after.text_id_bytes - before.text_id_bytes,
        6 * u64::from(original_reads)
    );
    assert!(after.source_bytes > before.source_bytes);
    flip(&mut managed, SOURCE, 88);
    for span in &proof.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    same_state(&raw, &managed, RETURN);

    let result = serde_json::json!({
        "status": "PASS",
        "source_bytes": 730,
        "original_reader_cases": original_reads,
        "catalog_resolutions": SLOT_COUNT,
        "mixed_mirror_boundary_checks": boundaries,
        "predecessor_attached": true,
        "invalid_candidates_rejected": 2,
        "left_raw_neighbor_preserved": true,
        "full_state_and_clock_equal": true,
    });
    fs::write(
        input("FFTA_LOCALIZED_TEXT_IDS_RESULT"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
