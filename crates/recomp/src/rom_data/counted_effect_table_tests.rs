//! Controlled original reads of the complete E618 effect table bank.
use super::{
    attach_dataset,
    direct_byte_copy_tests::{flip, input, same_state},
    indexed_reader_bank_tests::{execute_until, initialize},
    read_dataset,
    screen_update_table_tests::check_boundaries,
    COUNTED_EFFECT_TABLES_ID,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::SourceRange,
    Bus, Machine,
};
use serde::Deserialize;
use std::{collections::BTreeMap, fs, sync::Arc};

const BASE: u32 = 0x0800_0000;
const RETURN: u32 = BASE + 0x300;
const CONTEXT: u32 = 0x0201_0000;
const DESTINATION: u32 = 0x0201_2000;
const PROOF_SHA: &str = "e832bfb6625b6460ac239e3f62d62e520b16146c4c70f5b3d6ba1c7b03098a24";

#[derive(Deserialize)]
struct Table {
    offset: u32,
    samples: u16,
}

impl Table {
    fn sample(&self, rom: &[u8], index: u16) -> [u8; 2] {
        let at = (self.offset + 2 + 2 * u32::from(index)) as usize;
        rom[at..at + 2].try_into().unwrap()
    }
}

#[derive(Deserialize)]
struct Proof {
    tables: [Table; 13],
    reader_sites: BTreeMap<u32, u32>,
    new_ranges: Vec<SourceRange>,
}

fn literal_loads(machine: &mut Machine, rom: &[u8], sites: &BTreeMap<u32, u32>) {
    for (&pc, &source) in sites {
        let instruction = u16::from_le_bytes(rom[pc as usize..][..2].try_into().unwrap());
        let destination = usize::from((instruction >> 8) & 7);
        initialize(machine, BASE + pc);
        execute_until(machine, BASE + pc + 2, 4);
        assert_eq!(machine.cpu.regs[destination], BASE + source);
    }
}

fn edge_reader(machine: &mut Machine, rom: &[u8], table: &Table) {
    initialize(machine, 0x080e_9e06);
    machine.cpu.regs[7] = CONTEXT;
    execute_until(machine, 0x080e_9e20, 60);
    assert_eq!(machine.bus.read16(CONTEXT + 8), table.samples);
    machine.bus.write8(CONTEXT, 20);

    for _ in 0..40 {
        let previous = machine.bus.read16(CONTEXT + 8);
        initialize(machine, 0x080e_415c);
        machine.cpu.regs[0] = CONTEXT + 8;
        execute_until(machine, RETURN, 64);
        let current = machine.bus.read16(CONTEXT + 8);
        assert!(current < previous && previous <= table.samples);

        initialize(machine, 0x080e_9f14);
        machine.cpu.regs[4] = BASE + table.offset;
        machine.cpu.regs[7] = CONTEXT;
        execute_until(machine, 0x080e_9f30, 20);
        let [lo, hi] = table.sample(rom, current);
        let difference = (i32::from(hi) - i32::from(lo)) as u32;
        assert_eq!(machine.cpu.regs[0], (difference << 15) >> 16);

        initialize(machine, 0x080e_9f46);
        machine.cpu.regs[3] = u32::from(current);
        machine.cpu.regs[5] = DESTINATION + 2 * u32::from(current);
        machine.cpu.regs[7] = CONTEXT;
        machine.cpu.regs[8] = BASE + table.offset + 2;
        machine.cpu.regs[10] = BASE + table.offset + 3;
        machine
            .bus
            .write32(machine.cpu.regs[13] + 24, u32::from(previous));
        execute_until(machine, 0x080e_9f78, 60);
    }
    assert_eq!(machine.bus.read16(CONTEXT + 8), 0);
    for row in 0..table.samples {
        assert_eq!(
            machine.bus.read16(DESTINATION + 2 * u32::from(row)),
            u16::from_le_bytes(table.sample(rom, row))
        );
    }
}

fn check_reversed_pairs(machine: &mut Machine, rom: &[u8], table: &Table, start: u32, bias: u8) {
    for index in 0..table.samples {
        let [a, b] = table.sample(rom, index);
        assert_eq!(
            machine.bus.read16(start + 2 * u32::from(index)),
            u16::from_be_bytes([a.wrapping_add(bias), b.wrapping_add(bias)])
        );
    }
}

fn indexed_readers(machine: &mut Machine, rom: &[u8], tables: &[Table]) {
    for (slot, table) in tables.iter().enumerate() {
        initialize(machine, 0x0811_3332);
        machine.cpu.regs[0] = slot as u32;
        execute_until(machine, 0x0811_333a, 8);
        let source = machine.cpu.regs[4];
        assert_eq!(source, BASE + table.offset);

        initialize(machine, 0x0811_3342);
        machine.cpu.regs[4] = source;
        machine.cpu.regs[9] = DESTINATION;
        execute_until(machine, 0x0811_337e, 3500);
        assert_eq!(machine.cpu.regs[5], u32::from(table.samples));
        check_reversed_pairs(machine, rom, table, DESTINATION + 64, 0);
    }
}

fn direct_pair_readers(machine: &mut Machine, rom: &[u8], tables: &[Table]) {
    let copies = [
        (0x0811_da44, 0x0811_da8e, 88),
        (0x0811_da9a, 0x0811_dadc, 120),
    ];
    for (table, (start, stop, bias)) in tables.iter().zip(copies) {
        initialize(machine, start);
        machine.cpu.regs[5] = BASE + table.offset;
        machine.cpu.regs[7] = CONTEXT;
        machine.cpu.regs[12] = DESTINATION;
        execute_until(machine, stop, 2000);
        assert_eq!(machine.cpu.regs[4], u32::from(table.samples));
        check_reversed_pairs(machine, rom, table, DESTINATION + 64, bias);
    }
}

fn direct_halfword_readers(machine: &mut Machine, rom: &[u8], tables: &[Table]) {
    // Original paths write the halfword at byte 12 of a 20-byte destination row.
    let copies = [
        (0x080e_f816, 0x080e_f854, 5, 0),
        (0x0811_45da, 0x0811_463c, 6, 32 * 20),
        (0x080f_d150, 0x080f_d192, 8, 800),
        (0x080f_65ce, 0x080f_6614, 12, 1120),
    ];
    for (table, (start, stop, destination_reg, first_row)) in tables.iter().zip(copies) {
        initialize(machine, start);
        machine.cpu.regs[7] = CONTEXT;
        machine.cpu.regs[destination_reg] = DESTINATION;
        execute_until(machine, stop, 6000);
        for row in 0..table.samples {
            assert_eq!(
                machine
                    .bus
                    .read16(DESTINATION + first_row + 20 * u32::from(row) + 12),
                u16::from_le_bytes(table.sample(rom, row))
            );
        }
    }
}

#[test]
#[ignore = "Exact EUR ROM, E618 source proof and e361/e362 datasets"]
fn european_counted_effect_tables_feed_original_readers() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let proof_bytes = fs::read(input("FFTA_COUNTED_EFFECT_PROOF")).unwrap();
    assert_eq!(digest(&proof_bytes), PROOF_SHA);
    let proof: Proof = serde_json::from_slice(&proof_bytes).unwrap();
    let mut previous = read_dataset(&input("FFTA_COUNTED_EFFECT_PREVIOUS")).unwrap();
    let mut dataset = read_dataset(&input("FFTA_COUNTED_EFFECT_DATASET")).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (361, 360)
    );
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (362, 361)
    );

    let mut raw = Machine::new(rom.clone());
    let mut managed = Machine::new(rom.clone());
    attach_dataset(&mut managed, &previous).unwrap();
    assert!(managed
        .bus
        .rom_resources()
        .unwrap()
        .counted_effect_table_bus_bytes()
        .is_none());
    attach_dataset(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    previous.manifest.extractor_revision = 362;
    previous.manifest.decoder_revision = 361;
    assert!(attach_dataset(&mut managed, &previous).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|entry| entry.id == COUNTED_EFFECT_TABLES_ID)
        .unwrap();
    assert_eq!(resource.sources, proof.new_ranges);
    let packed = dataset.counted_effect_table_bytes.as_mut().unwrap();
    assert_eq!(packed.len(), 3016);
    packed[0] ^= 1;
    resource.sha256 = digest(packed);
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    for span in &proof.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    let boundaries = check_boundaries(&mut raw.bus, &mut managed.bus, &proof.new_ranges);
    let before = owner.counted_effect_table_bus_bytes().unwrap();
    for machine in [&mut raw, &mut managed] {
        literal_loads(machine, &rom, &proof.reader_sites);
        edge_reader(machine, &rom, &proof.tables[0]);
        indexed_readers(machine, &rom, &proof.tables[1..7]);
        direct_pair_readers(machine, &rom, &proof.tables[7..9]);
        direct_halfword_readers(machine, &rom, &proof.tables[9..]);
    }
    let read_bytes = owner.counted_effect_table_bus_bytes().unwrap() - before;
    assert!(read_bytes >= 3016);
    for span in &proof.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    same_state(&raw, &managed, 0x080f_6614);
    let result = serde_json::json!({
        "status": "PASS",
        "tables": proof.tables.len(),
        "samples": proof.tables.iter().map(|table| u32::from(table.samples)).sum::<u32>(),
        "directory_slots": 6,
        "literal_readers": proof.reader_sites.len(),
        "source_bytes": 3016,
        "mixed_mirror_boundary_checks": boundaries,
        "original_read_bytes": read_bytes,
        "predecessor_attached": true,
        "invalid_candidates_rejected": 2,
        "full_state_and_clock_equal": true,
    });
    fs::write(
        input("FFTA_COUNTED_EFFECT_RESULT"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
