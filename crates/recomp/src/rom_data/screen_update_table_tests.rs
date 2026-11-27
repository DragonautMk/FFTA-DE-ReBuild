//! Verify the EUR screen-update table binding through the original Thumb readers.
use super::{
    attach_dataset,
    direct_byte_copy_tests::{flip, input, same_state},
    indexed_reader_bank_tests::{execute_until, initialize},
    read_dataset, SCREEN_UPDATE_TABLES_ID,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::SourceRange,
    Bus, Machine, MemMap,
};
use serde::Deserialize;
use std::{fs, sync::Arc};

const ROM_BASE: u32 = 0x0800_0000;
const CONTEXT: u32 = 0x0201_0000;
const SOURCE_PROOF: &str = "79dc4271347449856536a28d6bf02d059e6d0e644066a45e429376851872cddc";

#[derive(Deserialize)]
struct Bank {
    grid: u32,
    directory: u32,
    byte_curves: Vec<u32>,
    row_byte_curves: Vec<u32>,
}

#[derive(Deserialize)]
struct Proof {
    banks: [Bank; 2],
    new_ranges: Vec<SourceRange>,
}

struct Reader {
    store_pointer: u32,
    initial_scale: u32,
    step: u32,
    after_step: u32,
    reads_global_curve: bool,
    y_offset: u32,
    scale: u32,
    continue_test: u32,
    continue_to: u32,
}

const READERS: [Reader; 2] = [
    Reader {
        store_pointer: 0x0811_a0f4,
        initial_scale: 0x0811_a1b6,
        step: 0x0811_a28e,
        after_step: 0x0811_a2a0,
        reads_global_curve: true,
        y_offset: 0x0811_a2da,
        scale: 0x0811_a316,
        continue_test: 0x0811_a376,
        continue_to: 0x0811_a3b2,
    },
    Reader {
        store_pointer: 0x0811_b7ec,
        initial_scale: 0x0811_b8ae,
        step: 0x0811_b9aa,
        after_step: 0x0811_b9b2,
        reads_global_curve: false,
        y_offset: 0x0811_b9e6,
        scale: 0x0811_ba22,
        continue_test: 0x0811_ba82,
        continue_to: 0x0811_bb26,
    },
];

fn half(rom: &[u8], at: u32) -> u16 {
    u16::from_le_bytes(rom[at as usize..][..2].try_into().unwrap())
}

fn word(rom: &[u8], at: u32) -> u32 {
    u32::from_le_bytes(rom[at as usize..][..4].try_into().unwrap())
}

fn curve_value(rom: &[u8], bank: &Bank, slot: u32, step: u32) -> Option<u16> {
    let pointer = word(rom, bank.directory + slot * 4);
    (pointer != 0).then(|| half(rom, pointer - ROM_BASE + 2 * step))
}

fn initialize_bank(machine: &mut Machine, rom: &[u8], bank: &Bank, reader: &Reader) {
    assert_eq!(bank.row_byte_curves.len(), 24);
    for (row, &curve) in bank.row_byte_curves.iter().enumerate() {
        let row = row as u32;
        let dest = CONTEXT + 268 + 4 * row;
        initialize(machine, reader.store_pointer);
        machine.cpu.regs[1] = dest;
        machine.cpu.regs[10] = 16 * row;
        execute_until(machine, reader.store_pointer + 10, 8);
        assert_eq!(machine.bus.read32(dest), ROM_BASE + curve);
        machine.bus.write16(
            CONTEXT + 122 + 2 * row,
            half(rom, bank.grid + 16 * row + 8).wrapping_add(80),
        );
    }

    let initial =
        |slot| curve_value(rom, bank, slot, 0).unwrap_or(if slot < 6 { 0x0100 } else { 0xff00 });
    for slot in 0..12 {
        initialize(machine, reader.initial_scale);
        let other_slot = 11 - slot;
        machine.cpu.regs[6] = slot;
        machine.cpu.regs[7] = other_slot;
        machine.cpu.regs[10] = 16 * slot;
        execute_until(machine, reader.initial_scale + 0x64, 40);
        assert_eq!(machine.cpu.regs[5], u32::from(initial(slot)));
        assert_eq!(machine.cpu.regs[2], u32::from(initial(other_slot)));
        let flag = match rom[(bank.grid + 16 * slot + 3) as usize] {
            1 => 0xc000,
            2 => 0x4000,
            3 => 0x8000,
            _ => 0,
        };
        assert_eq!(machine.cpu.regs[3], flag);
    }
}

fn update_bank(machine: &mut Machine, rom: &[u8], bank: &Bank, reader: &Reader) {
    machine.bus.write16(CONTEXT, 0);
    for step in 1..=150 {
        initialize(machine, reader.step);
        machine.cpu.regs[8] = CONTEXT;
        execute_until(machine, reader.after_step, 12);
        assert_eq!(machine.bus.read16(CONTEXT), step as u16);
        if reader.reads_global_curve {
            assert_eq!(
                machine.cpu.regs[3],
                u32::from(rom[(bank.byte_curves[0] + step) as usize])
            );
        }

        for (row, &curve) in bank.row_byte_curves.iter().enumerate() {
            let row = row as u32;
            initialize(machine, reader.y_offset);
            machine.cpu.regs[3] = 4 * row;
            machine.cpu.regs[4] = CONTEXT + 122 + 2 * row;
            machine.cpu.regs[8] = CONTEXT;
            execute_until(machine, reader.y_offset + 28, 18);
            let base = half(rom, bank.grid + 16 * row + 8).wrapping_add(80);
            let delta = u16::from(rom[(curve + step) as usize]);
            assert_eq!(
                machine.cpu.regs[2],
                base.wrapping_add(delta) as i16 as i32 as u32
            );
        }

        for slot in 0..12 {
            let other_slot = 11 - slot;
            machine.bus.write8(CONTEXT + 26, slot as u8);
            machine.bus.write8(CONTEXT + 50, other_slot as u8);
            initialize(machine, reader.scale);
            let first = machine.cpu.regs[13] + 16;
            machine.cpu.regs[4] = first + 2;
            machine.cpu.regs[8] = CONTEXT;
            machine.cpu.regs[9] = first;
            machine.bus.write16(first, 0x5012);
            machine.bus.write16(first + 2, 0x6023);
            execute_until(machine, reader.scale + 0x40, 40);
            assert_eq!(
                machine.bus.read16(first),
                curve_value(rom, bank, slot, step).unwrap_or(0x5012)
            );
            assert_eq!(
                machine.bus.read16(first + 2),
                curve_value(rom, bank, other_slot, step).unwrap_or(0x6023)
            );
        }

        initialize(machine, reader.continue_test);
        machine.cpu.regs[0] = step;
        let expected = if step < 150 {
            reader.continue_to
        } else {
            reader.continue_test + 4
        };
        execute_until(machine, expected, 3);
    }
}

pub(super) fn check_boundaries(
    raw: &mut MemMap,
    managed: &mut MemMap,
    ranges: &[SourceRange],
) -> usize {
    for span in ranges {
        let end = span.offset + span.length;
        for at in span.offset..end {
            assert_eq!(managed.read8(ROM_BASE + at), raw.read8(ROM_BASE + at));
        }
        for base in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
            for at in (span.offset - 4..span.offset + 4).chain(end - 4..end + 4) {
                assert_eq!(managed.read16(base + at), raw.read16(base + at));
                assert_eq!(managed.read32(base + at), raw.read32(base + at));
            }
        }
    }
    ranges.len() * 3 * 16 * 2
}

#[test]
#[ignore = "Exact EUR ROM, E615 source proof and e360/e361 datasets"]
fn european_screen_update_tables_feed_original_readers() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let proof_bytes = fs::read(input("FFTA_SCREEN_UPDATE_PROOF")).unwrap();
    assert_eq!(digest(&proof_bytes), SOURCE_PROOF);
    let proof: Proof = serde_json::from_slice(&proof_bytes).unwrap();
    let mut previous = read_dataset(&input("FFTA_SCREEN_UPDATE_PREVIOUS")).unwrap();
    let mut dataset = read_dataset(&input("FFTA_SCREEN_UPDATE_DATASET")).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (360, 359)
    );
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (361, 360)
    );

    let mut raw = Machine::new(rom.clone());
    let mut managed = Machine::new(rom.clone());
    attach_dataset(&mut managed, &previous).unwrap();
    assert!(managed
        .bus
        .rom_resources()
        .unwrap()
        .screen_update_table_bus_bytes()
        .is_none());
    attach_dataset(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    previous.manifest.extractor_revision = 361;
    previous.manifest.decoder_revision = 360;
    assert!(attach_dataset(&mut managed, &previous).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|item| item.id == SCREEN_UPDATE_TABLES_ID)
        .unwrap();
    assert_eq!(resource.sources, proof.new_ranges);
    let packed = dataset.screen_update_table_bytes.as_mut().unwrap();
    assert_eq!(packed.len(), 6434);
    packed[0] ^= 1;
    resource.sha256 = digest(packed);
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    for span in &proof.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    let boundary_checks = check_boundaries(&mut raw.bus, &mut managed.bus, &proof.new_ranges);
    let before = owner.screen_update_table_bus_bytes().unwrap();
    for machine in [&mut raw, &mut managed] {
        for (bank, reader) in proof.banks.iter().zip(&READERS) {
            initialize_bank(machine, &rom, bank, reader);
            update_bank(machine, &rom, bank, reader);
        }
    }
    let read_bytes = owner.screen_update_table_bus_bytes().unwrap() - before;
    assert!(read_bytes > 55_000);
    for span in &proof.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    same_state(&raw, &managed, 0x0811_ba86);
    fs::write(
        input("FFTA_SCREEN_UPDATE_RESULT"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "status": "PASS",
            "banks": 2,
            "update_steps_per_bank": 150,
            "row_pointers": 48,
            "directory_slots_per_bank": 12,
            "source_bytes": proof.new_ranges.iter().map(|span| span.length).sum::<u32>(),
            "mixed_mirror_boundary_checks": boundary_checks,
            "original_read_bytes": read_bytes,
            "predecessor_attached": true,
            "invalid_candidates_rejected": 2,
            "full_state_and_clock_equal": true,
        }))
        .unwrap(),
    )
    .unwrap();
}
