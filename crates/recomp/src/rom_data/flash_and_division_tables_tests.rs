//! Verify the original European Flash and signed division table readers.

use super::{
    FLASH_AND_DIVISION_TABLES_ID, attach_dataset,
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
use std::{collections::BTreeMap, fs, sync::Arc};

const BASE: u32 = 0x0800_0000;
const ENTRY: u32 = BASE + 0x145760;
const RETURN: u32 = BASE + 0x300;
const SETTINGS_SLOT: u32 = 0x0200_f370;
const TIMER_TABLE_SLOT: u32 = 0x0200_f380;
const TIMER_COUNTDOWN: u32 = 0x0300_5e42;
const TIMER_ADDRESS: u32 = 0x0400_0100;
const INTERRUPT_HANDLER: u32 = 0x0203_0000;
const IO_TIMER_ENABLE: u32 = TIMER_ADDRESS + 2;
const AUDIO_COMMAND: u32 = 0x370a90;
const RAW_GAP: u32 = 0x3708d8;
const PROOF_SHA: &str = "16ebc8550f2295c993a31912aa03908bd6774d2eb0245329037661a7f38522f2";

#[derive(Deserialize)]
struct Chip {
    offset: u32,
    device_id: u16,
    timeout_table: u32,
    callbacks: [u32; 4],
}

#[derive(Deserialize)]
struct Proof {
    chip_descriptors: [Chip; 4],
    timer_tables: [u32; 3],
    new_ranges: BTreeMap<String, Vec<SourceRange>>,
}

fn halfword(rom: &[u8], offset: u32) -> u16 {
    let at = offset as usize;
    u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())
}

fn check_chip(machine: &mut Machine, chip: &Chip, status: u32) {
    assert_eq!(machine.cpu.regs[0], status);
    for (slot, callback) in [0x0200_f36c, 0x0200_f378, 0x0200_f37c, 0x0200_f368]
        .into_iter()
        .zip(chip.callbacks)
    {
        assert_eq!(machine.bus.read32(slot), BASE + callback);
    }
    assert_eq!(machine.bus.read32(SETTINGS_SLOT), BASE + chip.offset + 20);
    assert_eq!(
        machine.bus.read32(TIMER_TABLE_SLOT),
        BASE + chip.timeout_table
    );
}

fn original_flash(machine: &mut Machine, rom: &[u8], proof: &Proof) {
    let backup = machine.bus.save_data().unwrap().to_vec();
    assert_eq!(backup.len(), 0x10000);

    // Use the normal Flash ID read on the emulator's in-memory device.
    initialize(machine, ENTRY);
    execute_until(machine, RETURN, 350_000);
    check_chip(machine, &proof.chip_descriptors[2], 0);
    assert_eq!(machine.bus.save_data().unwrap(), backup.as_slice());

    // Start after the ID read to exercise every device and the fallback.
    for chip in &proof.chip_descriptors {
        initialize(machine, BASE + 0x145778);
        machine.cpu.regs[3] = if chip.device_id == 0 {
            0xffff
        } else {
            chip.device_id.into()
        };
        let stack = machine.cpu.regs[13];
        machine.bus.write32(stack, 0);
        machine.bus.write32(stack + 4, RETURN | 1);
        execute_until(machine, RETURN, 256);
        check_chip(machine, chip, u32::from(chip.device_id == 0));
    }

    initialize(machine, BASE + 0x145348);
    machine.cpu.regs[1] = INTERRUPT_HANDLER;
    execute_until(machine, RETURN, 96);
    assert_eq!(machine.cpu.regs[0], 0);
    assert_eq!(machine.bus.read32(INTERRUPT_HANDLER), BASE + 0x145321);
    for table in proof.timer_tables {
        machine.bus.write32(TIMER_TABLE_SLOT, BASE + table);
        for phase in 0..4 {
            initialize(machine, BASE + 0x145384);
            machine.cpu.regs[0] = phase;
            execute_until(machine, RETURN, 128);
            let row = table + 6 * phase;
            assert_eq!(machine.bus.read16(TIMER_COUNTDOWN), halfword(rom, row));
            assert_eq!(machine.bus.read16(IO_TIMER_ENABLE), halfword(rom, row + 4));
            // A reload write seeds the active count before the timer starts.
            let count = machine.bus.read16(TIMER_ADDRESS);
            assert_eq!(count, halfword(rom, row + 2));
            machine.bus.write16(IO_TIMER_ENABLE, 0);
        }
    }
    assert_eq!(machine.bus.save_data().unwrap(), backup.as_slice());
}

fn original_division(machine: &mut Machine) {
    let readers = [
        (0x145cc2, 0x145cd0, 2, 1),
        (0x145da6, 0x145db4, 1, 2),
        (0x145f1a, 0x145f28, 1, 2),
    ];
    for (pc, stop, input, shift_reg) in readers {
        for shift in [0, 8, 16, 24] {
            for index in u32::from(shift != 0)..=255 {
                initialize(machine, BASE + pc);
                let value = index << shift;
                machine.cpu.regs[input] = value;
                machine.cpu.regs[shift_reg] = shift;
                execute_until(machine, BASE + stop, 12);
                assert_eq!(machine.cpu.regs[2], value.leading_zeros());
            }
        }
    }

    // Cover all three normalization branches and both signs through the
    // original signed 64-bit division routine.
    for (numerator, denominator) in [
        (0x0000_0001_0000_000a_i64, 7_i64),
        (0x0000_0008_0000_002d, 7),
        (0x3860_0000_0000_0001, 0x3000_0000_0000_0010),
        (i64::MAX, 0x8000_0000),
    ] {
        for (a, b) in [
            (numerator, denominator),
            (-numerator, denominator),
            (numerator, -denominator),
        ] {
            initialize(machine, BASE + 0x145c44);
            machine.cpu.regs[..4].copy_from_slice(&[
                a as u32,
                (a >> 32) as u32,
                b as u32,
                (b >> 32) as u32,
            ]);
            execute_until(machine, RETURN, 20_000);
            let actual = i64::from(machine.cpu.regs[0]) | (i64::from(machine.cpu.regs[1]) << 32);
            assert_eq!(actual, a / b, "{a} / {b}");
        }
    }
}

#[test]
#[ignore = "Exact EUR ROM, E660 source proof and e381/e382 datasets"]
fn european_flash_and_division_tables_feed_original_readers() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let proof_bytes = fs::read(input("FFTA_FLASH_AND_DIVISION_TABLES_PROOF")).unwrap();
    assert_eq!(digest(&proof_bytes), PROOF_SHA);
    let proof: Proof = serde_json::from_slice(&proof_bytes).unwrap();
    let mut ranges: Vec<_> = proof.new_ranges.values().flatten().cloned().collect();
    ranges.sort_by_key(|span| span.offset);

    let mut previous = read_dataset(&input("FFTA_FLASH_AND_DIVISION_TABLES_PREVIOUS")).unwrap();
    let mut dataset = read_dataset(&input("FFTA_FLASH_AND_DIVISION_TABLES_DATASET")).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (381, 380)
    );
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (382, 381)
    );
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_dataset(&mut managed, &previous).unwrap();
    assert!(
        managed
            .bus
            .rom_resources()
            .unwrap()
            .flash_and_division_tables_bus_bytes()
            .is_none()
    );
    attach_dataset(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    previous.manifest.extractor_revision = 382;
    previous.manifest.decoder_revision = 381;
    assert!(attach_dataset(&mut managed, &previous).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|item| item.id == FLASH_AND_DIVISION_TABLES_ID)
        .unwrap();
    assert_eq!(resource.sources, ranges);
    let packed = dataset.flash_and_division_tables_bytes.as_mut().unwrap();
    assert_eq!(packed.len(), 536);
    packed[0] ^= 1;
    resource.sha256 = digest(packed);
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    for span in &ranges {
        flip(&mut managed, span.offset, span.length);
    }
    let boundaries = check_boundaries(&mut raw.bus, &mut managed.bus, &ranges);
    flip(&mut managed, AUDIO_COMMAND, 4);
    assert_eq!(
        managed.bus.read32(BASE + AUDIO_COMMAND),
        raw.bus.read32(BASE + AUDIO_COMMAND)
    );
    flip(&mut managed, AUDIO_COMMAND, 4);
    flip(&mut managed, RAW_GAP, 4);
    assert_ne!(
        managed.bus.read32(BASE + RAW_GAP),
        raw.bus.read32(BASE + RAW_GAP)
    );
    flip(&mut managed, RAW_GAP, 4);

    let before = owner.flash_and_division_tables_bus_bytes().unwrap();
    for machine in [&mut raw, &mut managed] {
        original_flash(machine, &rom, &proof);
        original_division(machine);
    }
    let read_bytes = owner.flash_and_division_tables_bus_bytes().unwrap() - before;
    assert!(read_bytes > 0);
    for span in &ranges {
        flip(&mut managed, span.offset, span.length);
    }
    same_state(&raw, &managed, RETURN);
    let result = serde_json::json!({
        "status": "PASS",
        "source_bytes": 536,
        "flash_id_queries": 1,
        "descriptor_cases": proof.chip_descriptors.len(),
        "timer_cases": proof.timer_tables.len() * 4,
        "normalization_cases": 3 * (256 + 3 * 255),
        "signed_divisions": 12,
        "managed_reads": read_bytes,
        "mixed_mirror_boundary_checks": boundaries,
        "backup_unchanged": true,
        "audio_and_raw_neighbors_preserved": true,
        "predecessor_attached": true,
        "invalid_candidates_rejected": 2,
        "full_state_and_clock_equal": true,
    });
    fs::write(
        input("FFTA_FLASH_AND_DIVISION_TABLES_RESULT"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
