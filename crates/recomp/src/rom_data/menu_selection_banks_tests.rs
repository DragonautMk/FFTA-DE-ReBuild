//! Compare the European menu readers against the original cartridge bytes.
use super::{
    MENU_SELECTION_BANKS_ID, attach_dataset,
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
const RETURN: u32 = BASE + 0x300;
const WORK: u32 = 0x0203_0010;
const OUTPUT: u32 = 0x0203_0100;
const MATCH_INPUT: u32 = 0x0203_0200;
const ADJUSTMENT_INDEX: u32 = 0x0201_f500;
const PROOF_SHA: &str = "a1e4ddfb37ad91fffa049632d19211c79025aecd141ba328f65c0363b97e2c48";

#[derive(Deserialize)]
struct Table {
    offset: u32,
    count: u32,
    stride: u32,
}

#[derive(Deserialize)]
struct Reference {
    pc: u32,
    target: u32,
    register: usize,
}

#[derive(Deserialize)]
struct Proof {
    tables: [Table; 13],
    references: [Reference; 32],
    new_ranges: Vec<SourceRange>,
}

impl Proof {
    fn table(&self, offset: u32) -> &Table {
        self.tables
            .iter()
            .find(|table| table.offset == offset)
            .unwrap()
    }
}

fn halfword(rom: &[u8], offset: u32) -> u32 {
    let at = offset as usize;
    u16::from_le_bytes(rom[at..at + 2].try_into().unwrap()) as u32
}

fn original_row_fields(machine: &mut Machine, rom: &[u8], proof: &Proof) {
    let paths = [
        (0x39576c, 0x03e02e, 0x03e93a, 0x03e948),
        (0x395820, 0x03e0e2, 0x03e908, 0x03e912),
        (0x395848, 0x03e05c, 0x03e970, 0x03e97a),
        (0x395850, 0x03e084, 0x03e9b0, 0x03e9ba),
        (0x395858, 0x03e0ac, 0x03e9ec, 0x03e9f6),
    ];
    for (offset, select, entry, stop) in paths {
        let table = proof.table(offset);
        for index in 0..table.count {
            let row = offset + index * table.stride;
            initialize(machine, BASE + select);
            machine.cpu.regs[2] = OUTPUT;
            machine.cpu.regs[4] = index;
            execute_until(machine, BASE + 0x03e0ec, 16);
            assert_eq!(machine.bus.read16(OUTPUT) as u32, halfword(rom, row + 2));

            initialize(machine, BASE + entry);
            machine.cpu.regs[4] = OUTPUT;
            machine.cpu.regs[6] = index;
            execute_until(machine, BASE + stop, 16);
            assert_eq!(
                machine.bus.read16(OUTPUT + 12) as u32,
                halfword(rom, row + 4)
            );
        }
    }

    let table = proof.table(0x395a1c);
    for index in 0..table.count {
        let row = (table.offset + index * table.stride) as usize;
        initialize(machine, BASE + 0x03e8de);
        machine.cpu.regs[9] = index + 2;
        execute_until(machine, BASE + 0x03e8ea, 16);
        assert_eq!(machine.cpu.regs[6], rom[row + 1] as u32);
        for (field, pc) in [(2, 0x03e91a), (3, 0x03e950), (4, 0x03e982)] {
            machine.cpu.regs[15] = BASE + pc;
            execute_until(machine, BASE + pc + 2, 4);
            assert_eq!(machine.cpu.regs[6], rom[row + field] as u32);
        }
        initialize(machine, BASE + 0x03e9c2);
        machine.cpu.regs[9] = index + 2;
        execute_until(machine, BASE + 0x03e9ce, 16);
        assert_eq!(machine.cpu.regs[6], rom[row + 5] as u32);
    }
}

fn original_matches(machine: &mut Machine, rom: &[u8], proof: &Proof) {
    for (offset, entry, stop) in [
        (0x3958bc, 0x03bea4, 0x03beae),
        (0x3958d0, 0x03be18, 0x03be22),
        (0x3958e8, 0x03bd7c, 0x03bd86),
        (0x395908, 0x03bf2c, 0x03bf36),
    ] {
        let table = proof.table(offset);
        for index in 0..table.count {
            initialize(machine, BASE + entry);
            machine.cpu.regs[5] = index + 1;
            execute_until(machine, BASE + stop, 16);
            assert_eq!(
                machine.cpu.regs[0],
                halfword(rom, offset + index * table.stride + 2)
            );
        }
    }

    let table = proof.table(0x395978);
    for index in 0..table.count {
        initialize(machine, BASE + 0x03bda8);
        machine.cpu.regs[0] = index << 24;
        execute_until(machine, BASE + 0x03bdb0, 16);
        assert_eq!(
            machine.cpu.regs[0],
            halfword(rom, table.offset + index * table.stride)
        );
    }

    let table = proof.table(0x395908);
    for index in 0..table.count {
        let row = (table.offset + index * table.stride) as usize;
        let (first, second) = (rom[row], rom[row + 1]);
        initialize(machine, BASE + 0x03c2bc);
        machine.bus.write8(OUTPUT, 0xff);
        machine.bus.write32(
            MATCH_INPUT,
            u32::from_le_bytes([first, first, second, second]),
        );
        machine.cpu.regs[..2].copy_from_slice(&[OUTPUT, MATCH_INPUT]);
        execute_until(machine, RETURN, 8192);
        assert_eq!(machine.cpu.regs[0], 1);
        assert_eq!(machine.bus.read8(OUTPUT), index as u8);
    }

    // No pattern matches, so the original loops visit every first byte in
    // their respective tables and preserve the result sentinel.
    for entry in [0x03c308, 0x03c390, 0x03c3f0] {
        initialize(machine, BASE + entry);
        machine.bus.write8(OUTPUT, 0x5a);
        machine.bus.write32(MATCH_INPUT, u32::MAX);
        machine.cpu.regs[..2].copy_from_slice(&[OUTPUT, MATCH_INPUT]);
        execute_until(machine, RETURN, 8192);
        assert_eq!(machine.cpu.regs[0], 0);
        assert_eq!(machine.bus.read8(OUTPUT), 0x5a);
    }
}

fn original_labels_and_adjustments(machine: &mut Machine, rom: &[u8], proof: &Proof) {
    let table = proof.table(0x395874);
    for index in 0..table.count {
        let row = (table.offset + index * table.stride) as usize;
        initialize(machine, BASE + 0x03e1c6);
        machine.bus.write8(WORK, index as u8);
        machine.cpu.regs[6] = WORK;
        execute_until(machine, BASE + 0x03e1d0, 16);
        assert_eq!(machine.cpu.regs[0], rom[row + 1] as u32);
        machine.cpu.regs[15] = BASE + 0x03e1ee;
        execute_until(machine, BASE + 0x03e1f6, 16);
        assert_eq!(machine.cpu.regs[2], rom[row] as u32);
    }

    let table = proof.table(0x395860);
    for index in 0..table.count {
        let row = (table.offset + index * table.stride) as usize;
        initialize(machine, BASE + 0x04974c);
        machine.bus.write16(ADJUSTMENT_INDEX, index as u16 + 1);
        machine.cpu.regs[1] = 17;
        machine.cpu.regs[2] = OUTPUT;
        execute_until(machine, BASE + 0x049762, 16);
        assert_eq!(
            machine.bus.read32(OUTPUT),
            2 * 17 * (100 + rom[row + 1] as u32)
        );
        machine.cpu.regs[15] = BASE + 0x0497b4;
        execute_until(machine, BASE + 0x0497be, 16);
        assert_eq!(machine.cpu.regs[0], rom[row + 2] as u32);
        initialize(machine, BASE + 0x04989c);
        execute_until(machine, BASE + 0x0498aa, 16);
        assert_eq!(machine.cpu.regs[0], rom[row + 1] as u32);
    }
}

#[test]
#[ignore = "Exact EUR ROM, E656 source proof and e379/e380 datasets"]
fn european_menu_selection_banks_feed_original_readers() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let proof_bytes = fs::read(input("FFTA_MENU_SELECTION_BANKS_PROOF")).unwrap();
    assert_eq!(digest(&proof_bytes), PROOF_SHA);
    let proof: Proof = serde_json::from_slice(&proof_bytes).unwrap();
    let mut previous = read_dataset(&input("FFTA_MENU_SELECTION_BANKS_PREVIOUS")).unwrap();
    let mut dataset = read_dataset(&input("FFTA_MENU_SELECTION_BANKS_DATASET")).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (379, 378)
    );
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (380, 379)
    );

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_dataset(&mut managed, &previous).unwrap();
    assert!(
        managed
            .bus
            .rom_resources()
            .unwrap()
            .menu_selection_banks_bus_bytes()
            .is_none()
    );
    attach_dataset(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    previous.manifest.extractor_revision = 380;
    previous.manifest.decoder_revision = 379;
    assert!(attach_dataset(&mut managed, &previous).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|item| item.id == MENU_SELECTION_BANKS_ID)
        .unwrap();
    assert_eq!(resource.sources, proof.new_ranges);
    let packed = dataset.menu_selection_banks_bytes.as_mut().unwrap();
    assert_eq!(packed.len(), 728);
    packed[0] ^= 1;
    resource.sha256 = digest(packed);
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    for span in &proof.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    // The neighboring indexed words retain their independent typed owner.
    flip(&mut managed, 0x3959a4, 120);
    let boundaries = check_boundaries(&mut raw.bus, &mut managed.bus, &proof.new_ranges);
    for offset in (0x3959a4..0x395a1c).step_by(4) {
        assert_eq!(
            managed.bus.read32(BASE + offset),
            raw.bus.read32(BASE + offset)
        );
    }

    let before = owner.menu_selection_banks_bus_bytes().unwrap();
    for machine in [&mut raw, &mut managed] {
        for reference in &proof.references {
            initialize(machine, BASE + reference.pc);
            execute_until(machine, BASE + reference.pc + 2, 4);
            assert_eq!(
                machine.cpu.regs[reference.register],
                BASE + reference.target
            );
        }
        original_row_fields(machine, &rom, &proof);
        original_matches(machine, &rom, &proof);
        original_labels_and_adjustments(machine, &rom, &proof);
    }
    let reader_bytes = owner.menu_selection_banks_bus_bytes().unwrap() - before;
    assert!(reader_bytes > 0);
    for span in &proof.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    flip(&mut managed, 0x3959a4, 120);
    same_state(&raw, &managed, BASE + 0x0498aa);
    let result = serde_json::json!({
        "status": "PASS",
        "source_bytes": 728,
        "tables": proof.tables.len(),
        "literal_readers": proof.references.len(),
        "pattern_matches": proof.table(0x395908).count,
        "original_read_bytes": reader_bytes,
        "mixed_mirror_boundary_checks": boundaries,
        "neighboring_words_preserved": 30,
        "predecessor_attached": true,
        "invalid_candidates_rejected": 2,
        "full_state_and_clock_equal": true,
    });
    fs::write(
        input("FFTA_MENU_SELECTION_BANKS_RESULT"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
