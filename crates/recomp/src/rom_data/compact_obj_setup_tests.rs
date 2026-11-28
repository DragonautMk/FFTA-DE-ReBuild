//! Verify the original European setup readers and compact OBJ constructor inputs.

use super::{
    PHYSICAL_COMPACT_OBJ_REGION_ID, attach_dataset,
    direct_byte_copy_tests::{flip, input, same_state},
    indexed_reader_bank_tests::{execute_until, initialize},
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
const SETUP: u32 = 0x39ac9c;
const RECTANGLES: u32 = 0x39ae3c;
const CONTEXT: u32 = 0x0202_0000;
const AGE: u8 = 4;
const X: i16 = -18;
const Y: i16 = 43;
const NEW_REVISION: (u32, u32) = (384, 383);
const PROOF_SHA: &str = "fc02fbbcdfa663786970943bfd9d827a8d097249a8698a0fc727fd36fb2dbd92";

#[derive(Deserialize)]
struct Child {
    root: u32,
}

#[derive(Deserialize)]
struct Proof {
    children: Vec<Child>,
    new_ranges: Vec<SourceRange>,
}

fn half(rom: &[u8], offset: u32) -> u16 {
    let at = offset as usize;
    u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())
}

fn word(rom: &[u8], offset: u32) -> u32 {
    let at = offset as usize;
    u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
}

fn read_original_setups(machine: &mut Machine, rom: &[u8]) -> u32 {
    machine.bus.write8(CONTEXT + 2, AGE);
    for index in 0..6 {
        initialize(machine, BASE + 0x0e651a);
        machine.cpu.regs[4] = index;
        machine.cpu.regs[10] = CONTEXT;
        execute_until(machine, BASE + 0x0e652c, 12);
        let rectangle = RECTANGLES + 8 * index;
        assert_eq!(
            machine.cpu.regs[..4],
            [
                word(rom, rectangle),
                AGE.into(),
                1,
                rom[(rectangle + 4) as usize].into()
            ]
        );
    }

    let mut active = 0;
    for index in 0..8 {
        let row_offset = 52 * index;
        let row = SETUP + row_offset;
        let count = u32::from(rom[row as usize]);
        for (start, end) in [(0x0e6534, 0x0e6540), (0x0e66c0, 0x0e66c8)] {
            initialize(machine, BASE + start);
            let stack = machine.cpu.regs[13];
            machine.bus.write32(stack + 0x1c, index);
            machine.cpu.regs[9] = row_offset;
            execute_until(machine, BASE + end, 12);
            assert_eq!(machine.cpu.regs[0], count);
        }
        for child in 0..count {
            let record = row + 4 + 12 * child;
            let bank = word(rom, record + 8);
            let x = X.wrapping_add(half(rom, record + 2) as i16);
            let y = Y.wrapping_add(half(rom, record + 4) as i16);
            initialize(machine, BASE + 0x0e6648);
            let stack = machine.cpu.regs[13];
            machine.bus.write32(stack + 0x1c, index);
            machine.bus.write32(stack + 0x28, u32::from(X as u16) << 16);
            machine.bus.write32(stack + 0x2c, u32::from(Y as u16) << 16);
            machine.bus.write32(stack + 0x30, child * 2);
            machine.cpu.regs[8] = child;
            machine.cpu.regs[10] = CONTEXT;
            execute_until(machine, BASE + 0x0e668c, 48);
            assert_eq!(
                machine.cpu.regs[..4],
                [
                    bank,
                    rom[record as usize].into(),
                    x as i32 as u32,
                    y as i32 as u32
                ],
            );
            assert_eq!(machine.bus.read32(stack), u32::from(half(rom, record + 6)));
            assert_eq!(machine.bus.read32(stack + 4), u32::from(AGE));
            assert_eq!(machine.bus.read32(stack + 8), 0);
            assert_eq!(machine.bus.read32(stack + 12), 1);
            execute_until(machine, BASE + 0x0d9828, 64);
            assert_eq!(machine.cpu.regs[8], bank + 4);
            assert_eq!(
                machine.cpu.regs[6],
                bank + u32::from(half(rom, bank - BASE + 2))
            );

            initialize(machine, BASE + 0x0e669c);
            machine.cpu.regs[5] = record + BASE;
            execute_until(machine, BASE + 0x0e66a4, 12);
            assert_eq!(
                machine.cpu.regs[0],
                u32::from(rom[(record + 1) as usize] & 1)
            );
            active += 1;
        }
    }
    active
}

#[test]
#[ignore = "Exact EUR ROM, E664 source proof and e383/e384 datasets"]
fn european_compact_obj_setups_feed_original_readers() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let proof_bytes = fs::read(input("FFTA_COMPACT_OBJ_SETUP_PROOF")).unwrap();
    assert_eq!(digest(&proof_bytes), PROOF_SHA);
    let proof: Proof = serde_json::from_slice(&proof_bytes).unwrap();
    assert_eq!(proof.children.len(), 6);
    assert_eq!(proof.new_ranges.len(), 3);
    assert_eq!(
        proof.new_ranges.iter().map(|span| span.length).sum::<u32>(),
        424
    );

    let mut previous = read_dataset(&input("FFTA_COMPACT_OBJ_SETUP_PREVIOUS")).unwrap();
    let mut dataset = read_dataset(&input("FFTA_COMPACT_OBJ_SETUP_DATASET")).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (383, 382)
    );
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        NEW_REVISION
    );
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_dataset(&mut managed, &previous).unwrap();
    attach_dataset(&mut managed, &dataset).unwrap();
    let active = managed.bus.rom_resources().unwrap().clone();
    (
        previous.manifest.extractor_revision,
        previous.manifest.decoder_revision,
    ) = NEW_REVISION;
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
        .filter(|item| item.role.starts_with("compact-obj-setup-"));
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
        .chain([(RECTANGLES, 48), (0x0e65d8, 4)])
        .collect::<Vec<_>>();
    for &(offset, len) in &owned {
        flip(&mut managed, offset, len);
    }
    let boundaries = check_boundaries(&mut raw.bus, &mut managed.bus, &proof.new_ranges);
    let raw_neighbor = 0x0e66ec;
    flip(&mut managed, raw_neighbor, 2);
    assert_eq!(
        managed.bus.read16(BASE + raw_neighbor),
        !raw.bus.read16(BASE + raw_neighbor)
    );
    flip(&mut managed, raw_neighbor, 2);

    let before = active.physical_compact_obj_bus_bytes().unwrap();
    assert_eq!(read_original_setups(&mut raw, &rom), 11);
    assert_eq!(read_original_setups(&mut managed, &rom), 11);
    let read_bytes = active.physical_compact_obj_bus_bytes().unwrap() - before;
    assert!(
        read_bytes >= 256,
        "compact OBJ source bytes read {read_bytes}"
    );
    for (offset, len) in owned {
        flip(&mut managed, offset, len);
    }
    same_state(&raw, &managed, BASE + 0x0e66a4);

    let result = serde_json::json!({
        "status": "PASS",
        "source_bytes": 424,
        "setup_count_readers": 16,
        "original_child_arguments_and_headers": 11,
        "companion_arguments": 6,
        "physical_owner_bytes_read": read_bytes,
        "mixed_mirror_boundary_checks": boundaries,
        "raw_neighbor_preserved": true,
        "predecessor_attached": true,
        "invalid_candidates_rejected": 2,
        "full_state_and_clock_equal": true,
    });
    fs::write(
        input("FFTA_COMPACT_OBJ_SETUP_RESULT"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
