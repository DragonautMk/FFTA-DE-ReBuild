//! Verify the original European indexed compact OBJ and parameter readers.

use super::{
    PHYSICAL_COMPACT_OBJ_REGION_ID, attach_dataset,
    direct_byte_copy_tests::{flip, input, same_state},
    indexed_reader_bank_tests::{execute_until, initialize},
    physical_compact_obj_region_tests::descriptor_reader,
    read_dataset,
    screen_update_table_tests::check_boundaries,
};
use gba_core::{
    Bus, Machine,
    ffta_rom::{EUROPE_SHA256, digest},
    rom_data::SourceRange,
};
use serde::Deserialize;
use std::{fs, sync::Arc};

const BASE: u32 = 0x0800_0000;
const CONTEXT: u32 = 0x0202_0000;
const CALLBACK_ARGUMENT: u32 = 0x0202_1000;
const CALLBACK_TASK: u32 = 0x0202_2000;
const REVISION: (u32, u32) = (385, 384);
const PROOF_SHA: &str = "98e53feb08523ff1a8891d0719e5ff8018bddd71c483d815912e932681a1d90a";

#[derive(Deserialize)]
struct Child {
    root: u32,
}

#[derive(Deserialize)]
struct Row {
    offset: u32,
}

#[derive(Deserialize)]
struct Proof {
    children: Vec<Child>,
    rows: Vec<Row>,
    new_ranges: Vec<SourceRange>,
}

fn word(rom: &[u8], offset: u32) -> u32 {
    let at = offset as usize;
    u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
}

fn read_original_bank(machine: &mut Machine, rom: &[u8], proof: &Proof) {
    for index in 0..3 {
        machine.bus.write8(CONTEXT, index);
        machine.bus.write8(CONTEXT + 1, index);

        initialize(machine, BASE + 0x0e7632);
        machine.cpu.regs[5] = CONTEXT;
        machine.cpu.regs[8] = CALLBACK_TASK;
        execute_until(machine, BASE + 0x0e7656, 32);
        let stack = machine.cpu.regs[13];
        assert_eq!(
            machine.bus.read32(stack),
            u32::from(rom[0x39b7b3 + index as usize])
        );
        assert_eq!(machine.bus.read32(stack + 4), CONTEXT + 0x10);
        assert_eq!(
            machine.bus.read32(stack + 8),
            word(rom, 0x39b7a4 + 4 * u32::from(index))
        );
        assert_eq!(machine.bus.read32(stack + 12), CALLBACK_TASK);

        initialize(machine, BASE + 0x0e77ec);
        machine.cpu.regs[5] = CONTEXT;
        machine.cpu.regs[7] = CALLBACK_TASK;
        execute_until(machine, BASE + 0x0e7812, 32);
        let stack = machine.cpu.regs[13];
        assert_eq!(
            machine.bus.read32(stack),
            u32::from(rom[0x39b7b0 + index as usize])
        );
        assert_eq!(machine.bus.read32(stack + 4), CONTEXT + 0x10);
        assert_eq!(
            machine.bus.read32(stack + 8),
            word(rom, 0x39b798 + 4 * u32::from(index))
        );
        assert_eq!(machine.bus.read32(stack + 12), CALLBACK_TASK);
        assert_eq!(machine.cpu.regs[..4], [120, 96, 0, 1]);

        for variant in 0..2 {
            initialize(machine, BASE + 0x0e7684);
            machine.cpu.regs[1] = 20 * variant;
            machine.cpu.regs[5] = CONTEXT;
            machine.cpu.regs[6] = 4;
            machine.cpu.regs[8] = CALLBACK_TASK;
            execute_until(machine, BASE + 0x0e76a0, 32);
            let stack = machine.cpu.regs[13];
            let parameter = u32::from(rom[0x39b7f8 + index as usize]);
            assert_eq!(machine.cpu.regs[1], BASE + 0x39b7d0 + 20 * variant);
            assert_eq!(machine.cpu.regs[2], 12);
            assert_eq!(machine.cpu.regs[3], parameter);
            assert_eq!(machine.bus.read32(stack), 4);
            assert_eq!(machine.bus.read32(stack + 4), CALLBACK_TASK);
            assert_eq!(machine.bus.read32(stack + 8), parameter);
        }

        initialize(machine, BASE + 0x0e7818);
        machine.cpu.regs[5] = CONTEXT;
        machine.cpu.regs[7] = CALLBACK_TASK;
        execute_until(machine, BASE + 0x0e7824, 12);
        assert_eq!(
            machine.cpu.regs[0],
            word(rom, 0x39b7bc + 4 * u32::from(index))
        );
        assert_eq!(machine.cpu.regs[1], CALLBACK_TASK);

        initialize(machine, BASE + 0x0e782a);
        machine.cpu.regs[5] = CONTEXT;
        machine.cpu.regs[7] = CALLBACK_TASK;
        execute_until(machine, BASE + 0x0e7836, 12);
        assert_eq!(machine.cpu.regs[0], BASE + 0xb47d44);
        assert_eq!(machine.cpu.regs[1], CALLBACK_TASK);
        assert_eq!(
            machine.cpu.regs[2],
            u32::from(rom[0x39b7cb + index as usize])
        );

        initialize(machine, BASE + 0x0e783c);
        machine.cpu.regs[5] = CONTEXT;
        machine.cpu.regs[7] = CALLBACK_TASK;
        execute_until(machine, BASE + 0x0e784c, 16);
        assert_eq!(machine.cpu.regs[0], BASE + 0x39b7fc + 8 * u32::from(index));

        initialize(machine, BASE + 0x0e7852);
        machine.cpu.regs[5] = CONTEXT;
        execute_until(machine, BASE + 0x0e785c, 12);
        assert_eq!(
            machine.bus.read8(CONTEXT + 3),
            rom[0x39b7c8 + index as usize]
        );

        initialize(machine, BASE + 0x0e7924);
        machine.cpu.regs[5] = CONTEXT;
        execute_until(machine, BASE + 0x0e792e, 12);
        assert_eq!(
            machine.bus.read8(CONTEXT + 4),
            rom[0x39b7b6 + index as usize]
        );
    }

    for row in &proof.rows {
        descriptor_reader(machine, rom, row.offset);
    }
    for variant in 0..2 {
        let source = 0x39b7d0 + 20 * variant;
        machine.bus.write32(CALLBACK_ARGUMENT + 8, BASE + source);
        initialize(machine, BASE + 0x0e00b2);
        machine.cpu.regs[1] = CALLBACK_ARGUMENT;
        machine.cpu.regs[5] = CALLBACK_TASK;
        execute_until(machine, BASE + 0x0e00c2, 1000);
        for byte in 0..20 {
            assert_eq!(
                machine.bus.read8(CALLBACK_TASK + 0x10 + byte),
                rom[(source + byte) as usize]
            );
        }
    }
}

#[test]
#[ignore = "Exact EUR ROM, E666 source proof and e384/e385 datasets"]
fn european_indexed_compact_obj_bank_feeds_original_readers() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let proof_bytes = fs::read(input("FFTA_INDEXED_COMPACT_OBJ_PROOF")).unwrap();
    assert_eq!(digest(&proof_bytes), PROOF_SHA);
    let proof: Proof = serde_json::from_slice(&proof_bytes).unwrap();
    assert_eq!(
        (
            proof.children.len(),
            proof.rows.len(),
            proof.new_ranges.len()
        ),
        (6, 20, 7)
    );
    assert_eq!(
        proof.new_ranges.iter().map(|span| span.length).sum::<u32>(),
        606
    );

    let mut previous = read_dataset(&input("FFTA_INDEXED_COMPACT_OBJ_PREVIOUS")).unwrap();
    let mut dataset = read_dataset(&input("FFTA_INDEXED_COMPACT_OBJ_DATASET")).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (384, 383)
    );
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        REVISION
    );
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_dataset(&mut managed, &previous).unwrap();
    attach_dataset(&mut managed, &dataset).unwrap();
    let active = managed.bus.rom_resources().unwrap().clone();
    (
        previous.manifest.extractor_revision,
        previous.manifest.decoder_revision,
    ) = REVISION;
    assert!(attach_dataset(&mut managed, &previous).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &active));

    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|item| item.id == PHYSICAL_COMPACT_OBJ_REGION_ID)
        .unwrap();
    let additions = resource
        .sources
        .iter()
        .filter(|item| item.role.starts_with("indexed-obj-"));
    assert!(additions.eq(&proof.new_ranges));
    let first = resource
        .sources
        .iter()
        .position(|item| item == &proof.new_ranges[0])
        .unwrap();
    let offset = resource.sources[..first]
        .iter()
        .map(|span| span.length as usize)
        .sum::<usize>();
    let encoded = dataset.physical_compact_obj_region_bytes.as_mut().unwrap();
    encoded[offset] ^= 1;
    let span = &mut resource.sources[first];
    span.sha256 = digest(&encoded[offset..offset + span.length as usize]);
    resource.sha256 = digest(encoded);
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &active));

    let owned = proof
        .new_ranges
        .iter()
        .map(|span| (span.offset, span.length))
        .chain(proof.children.iter().map(|child| (child.root, 4)))
        .chain([(0x0e76e4, 4), (0x0e787c, 4), (0x39b7bc, 12), (0x39b7fc, 48)])
        .collect::<Vec<_>>();
    for &(offset, len) in &owned {
        flip(&mut managed, offset, len);
    }
    let boundaries = check_boundaries(&mut raw.bus, &mut managed.bus, &proof.new_ranges);
    let raw_neighbor = 0x39b5b7;
    flip(&mut managed, raw_neighbor, 1);
    assert_eq!(
        managed.bus.read8(BASE + raw_neighbor),
        !raw.bus.read8(BASE + raw_neighbor)
    );
    flip(&mut managed, raw_neighbor, 1);

    let before = active.physical_compact_obj_bus_bytes().unwrap();
    read_original_bank(&mut raw, &rom, &proof);
    read_original_bank(&mut managed, &rom, &proof);
    let original_reads = active.physical_compact_obj_bus_bytes().unwrap() - before;
    assert!(original_reads > 0);
    for (offset, len) in owned {
        flip(&mut managed, offset, len);
    }
    same_state(&raw, &managed, BASE + 0x0e00c2);
    let result = serde_json::json!({
        "status": "PASS",
        "source_bytes": 606,
        "selector_cases": 6,
        "descriptor_cases": proof.rows.len(),
        "parameter_variants": 2,
        "physical_owner_bytes_read": original_reads,
        "mixed_mirror_boundary_checks": boundaries,
        "raw_neighbor_preserved": true,
        "predecessor_attached": true,
        "invalid_candidates_rejected": 2,
        "full_state_and_clock_equal": true,
    });
    fs::write(
        input("FFTA_INDEXED_COMPACT_OBJ_RESULT"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
