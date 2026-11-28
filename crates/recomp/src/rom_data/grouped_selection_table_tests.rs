//! Controlled original readers for the complete E620 source layout.
use super::{
    attach_dataset,
    counted_effect_table_tests::literal_loads,
    direct_byte_copy_tests::{flip, input, same_state},
    indexed_reader_bank_tests::{execute_until, initialize},
    read_dataset,
    screen_update_table_tests::check_boundaries,
    GROUPED_SELECTION_TABLES_ID,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::SourceRange,
    Bus, Machine,
};
use serde::Deserialize;
use std::{collections::BTreeMap, fs, sync::Arc};

const BASE: u32 = 0x0800_0000;
const CONTEXT: u32 = 0x0201_0000;
const DESTINATION: u32 = 0x0201_2000;
const PROOF_SHA: &str = "d03910125ea49b6543aad98baa0a7f97284e2b0fa59145b1fc04805b5011bbdb";
const FIRST_SHARED_ROW: (u32, u32) = (0x6e8994, 40);
const SHARED_LITERAL: (u32, u32) = (0x02ed2c, 4);

#[derive(Deserialize)]
struct Pool {
    offset: u32,
    rows: u32,
    columns: u32,
}

impl Pool {
    fn value(&self, rom: &[u8], group: u32, column: u32) -> u16 {
        half(rom, self.offset + 2 * (group * self.columns + column))
    }
}

#[derive(Deserialize)]
struct Terminal {
    normal: u32,
    doubled: u32,
}

#[derive(Deserialize)]
struct Proof {
    pools: [Pool; 4],
    weighted_offset: u32,
    weighted_rows: u32,
    weighted_groups: BTreeMap<u32, Terminal>,
    reader_sites: BTreeMap<u32, u32>,
    new_ranges: Vec<SourceRange>,
}

fn half(rom: &[u8], at: u32) -> u16 {
    u16::from_le_bytes(rom[at as usize..][..2].try_into().unwrap())
}

fn word(rom: &[u8], at: u32) -> u32 {
    u32::from_le_bytes(rom[at as usize..][..4].try_into().unwrap())
}

fn flag_scan(machine: &mut Machine, rom: &[u8], pool: &Pool) {
    initialize(machine, 0x080d_5d34);
    machine.cpu.regs[8] = pool.rows;
    execute_until(machine, 0x080d_5d76, 4000);
    let mut expected = pool.rows;
    for group in 0..pool.rows {
        for column in 0..pool.columns {
            let row = u32::from(pool.value(rom, group, column));
            let flags = half(rom, 0x6e8aac + 16 * row + 2);
            expected += (flags & 0x00c0).count_ones();
        }
    }
    assert_eq!(machine.cpu.regs[4], expected);
    assert_eq!(machine.cpu.regs[6], pool.rows);
}

fn mission_pool_reader(machine: &mut Machine, rom: &[u8], pool: &Pool, default: bool) {
    // The entry point follows the modulo-twenty operation in each path.
    let (entry, stop) = if default {
        (0x080d_3a2e, 0x080d_3a3c)
    } else {
        (0x080d_3a74, 0x080d_3a98)
    };
    for group in 0..pool.rows {
        machine.bus.write16(CONTEXT, 0xfff1 + group as u16);
        for column in 0..pool.columns {
            initialize(machine, entry);
            machine.cpu.regs[0] = column;
            machine.cpu.regs[4] = 255;
            machine.cpu.regs[6] = DESTINATION;
            machine.cpu.regs[7] = CONTEXT + 1;
            machine.cpu.regs[8] = CONTEXT;
            execute_until(machine, stop, 32);
            let expected = pool.value(rom, group, column);
            assert_eq!(machine.cpu.regs[0], u32::from(expected));
            assert_eq!(machine.bus.read8(DESTINATION + 8), expected as u8);
        }
    }
}

fn clamped_pool_reader(machine: &mut Machine, rom: &[u8], pool: &Pool) {
    for input_group in (0..pool.rows).chain([255]) {
        initialize(machine, 0x080d_6a98);
        machine.cpu.regs[5] = input_group;
        execute_until(machine, 0x080d_6aa0, 6);
        let group = input_group.min(9);
        let base = machine.cpu.regs[4];
        assert_eq!(base, BASE + pool.offset);
        assert_eq!(machine.cpu.regs[5], group);
        for column in 0..pool.columns {
            initialize(machine, 0x080d_6aae);
            machine.cpu.regs[0] = column;
            machine.cpu.regs[4] = base;
            machine.cpu.regs[5] = group;
            execute_until(machine, 0x080d_6abe, 12);
            assert_eq!(
                machine.cpu.regs[0],
                u32::from(pool.value(rom, group, column))
            );
        }
    }
}

fn original_weighted_address(machine: &mut Machine, offset: u32, index: u32) -> u32 {
    initialize(machine, 0x080d_5fa4);
    machine.cpu.regs[6] = index;
    execute_until(machine, 0x080d_5fb0, 10);
    let address = machine.cpu.regs[8];
    assert_eq!(address, BASE + offset + 12 * index);
    address
}

fn weighted_field_readers(machine: &mut Machine, rom: &[u8], proof: &Proof) {
    let readers = [
        (0x080d_64a0, 4, 0, false),
        (0x080d_6400, 1, 1, false),
        (0x080d_6350, 0, 2, false),
        (0x080d_62d8, 0, 3, false),
        (0x080d_62f4, 0, 4, true),
        (0x080d_6322, 0, 6, true),
    ];
    for row in 0..proof.weighted_rows {
        let address = original_weighted_address(machine, proof.weighted_offset, row);
        for (entry, output, field, is_halfword) in readers {
            initialize(machine, entry);
            machine.cpu.regs[8] = address;
            execute_until(machine, entry + 4, 4);
            let at = address - BASE + field;
            let expected = if is_halfword {
                half(rom, at)
            } else {
                u16::from(rom[at as usize])
            };
            assert_eq!(machine.cpu.regs[output], u32::from(expected));
        }
    }
}

fn expected_weighted_choice(
    rom: &[u8],
    offset: u32,
    rows: std::ops::RangeInclusive<u32>,
    doubled: bool,
    mut seed: u32,
) -> (u32, u32) {
    let multiplier = word(rom, 0x002880);
    let increment = word(rom, 0x002884);
    let mut remaining = 100;
    for row in rows {
        let weight = u32::from(rom[(offset + 12 * row) as usize]) * if doubled { 2 } else { 1 };
        seed = seed.wrapping_mul(multiplier).wrapping_add(increment);
        let chance = (seed >> 16) & 0x7fff;
        if chance % remaining < weight {
            return (row, seed);
        }
        remaining -= weight;
    }
    panic!("Weighted selection exceeded the authenticated group");
}

fn weighted_scans(machine: &mut Machine, rom: &[u8], proof: &Proof) {
    let rng = word(rom, 0x00287c);
    for (&start, terminal) in &proof.weighted_groups {
        for (doubled, last) in [(false, terminal.normal), (true, terminal.doubled)] {
            // Drive each original scan to its terminal row. The zero bonus
            // is the longest supported path through these source ranges.
            let (seed, final_rng) = (0..65536)
                .find_map(|seed| {
                    let (selected, final_rng) = expected_weighted_choice(
                        rom,
                        proof.weighted_offset,
                        start..=last,
                        doubled,
                        seed,
                    );
                    (selected == last).then_some((seed, final_rng))
                })
                .unwrap();
            let address = original_weighted_address(machine, proof.weighted_offset, start);
            initialize(machine, 0x080d_6470);
            let mission: u32 = if doubled { 123 } else { 0 };
            machine.cpu.regs[5] = mission;
            machine.cpu.regs[7] = 100;
            machine.cpu.regs[8] = address;
            machine.cpu.regs[9] = mission.wrapping_sub(123);
            machine.bus.write32(rng, seed);
            execute_until(machine, 0x080d_64c6, 12000);
            assert_eq!(
                machine.cpu.regs[8],
                BASE + proof.weighted_offset + 12 * last
            );
            assert_eq!(machine.bus.read32(rng), final_rng);
        }
    }
}

#[test]
#[ignore = "Exact EUR ROM, E620 source proof and e362/e363 datasets"]
fn european_grouped_selection_tables_feed_original_readers() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let proof_bytes = fs::read(input("FFTA_GROUPED_SELECTION_PROOF")).unwrap();
    assert_eq!(digest(&proof_bytes), PROOF_SHA);
    let proof: Proof = serde_json::from_slice(&proof_bytes).unwrap();
    assert_eq!(
        proof
            .pools
            .iter()
            .map(|pool| pool.rows * pool.columns)
            .sum::<u32>(),
        500
    );
    assert_eq!(proof.weighted_rows, 160);
    assert_eq!(proof.weighted_groups.len(), 21);

    let mut previous = read_dataset(&input("FFTA_GROUPED_SELECTION_PREVIOUS")).unwrap();
    let mut dataset = read_dataset(&input("FFTA_GROUPED_SELECTION_DATASET")).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (362, 361)
    );
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (363, 362)
    );

    let mut raw = Machine::new(rom.clone());
    let mut managed = Machine::new(rom.clone());
    attach_dataset(&mut managed, &previous).unwrap();
    assert!(managed
        .bus
        .rom_resources()
        .unwrap()
        .grouped_selection_table_bus_bytes()
        .is_none());
    attach_dataset(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    previous.manifest.extractor_revision = 363;
    previous.manifest.decoder_revision = 362;
    assert!(attach_dataset(&mut managed, &previous).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|entry| entry.id == GROUPED_SELECTION_TABLES_ID)
        .unwrap();
    assert_eq!(resource.sources, proof.new_ranges);
    let packed = dataset.grouped_selection_table_bytes.as_mut().unwrap();
    assert_eq!(packed.len(), 2912);
    packed[0] ^= 1;
    resource.sha256 = digest(packed);
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    let affected = proof
        .new_ranges
        .iter()
        .map(|range| (range.offset, range.length))
        .chain([FIRST_SHARED_ROW, SHARED_LITERAL]);
    for (offset, length) in affected.clone() {
        flip(&mut managed, offset, length);
    }
    let boundaries = check_boundaries(&mut raw.bus, &mut managed.bus, &proof.new_ranges);
    let before = owner.grouped_selection_table_bus_bytes().unwrap();
    for machine in [&mut raw, &mut managed] {
        literal_loads(machine, &rom, &proof.reader_sites);
        flag_scan(machine, &rom, &proof.pools[0]);
        mission_pool_reader(machine, &rom, &proof.pools[1], false);
        mission_pool_reader(machine, &rom, &proof.pools[2], true);
        clamped_pool_reader(machine, &rom, &proof.pools[3]);
        weighted_field_readers(machine, &rom, &proof);
        weighted_scans(machine, &rom, &proof);
    }
    let read_bytes = owner.grouped_selection_table_bus_bytes().unwrap() - before;
    assert!(read_bytes >= 2240);
    for (offset, length) in affected {
        flip(&mut managed, offset, length);
    }
    same_state(&raw, &managed, 0x080d_64c6);
    let result = serde_json::json!({
        "status": "PASS",
        "halfword_entries": 500,
        "weighted_rows": proof.weighted_rows,
        "weighted_terminal_paths": 2 * proof.weighted_groups.len(),
        "literal_readers": proof.reader_sites.len(),
        "source_bytes": 2912,
        "earlier_owner_bytes_altered": 44,
        "mixed_mirror_boundary_checks": boundaries,
        "original_read_bytes": read_bytes,
        "predecessor_attached": true,
        "invalid_candidates_rejected": 2,
        "full_state_clock_and_rng_equal": true,
    });
    fs::write(
        input("FFTA_GROUPED_SELECTION_RESULT"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
