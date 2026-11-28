//! Exercise all decimal heights through the original map getter and DMA queue.
use super::{
    attach_dataset,
    direct_byte_copy_tests::{flip, input, same_state},
    indexed_reader_bank_tests::{execute_until, initialize},
    read_dataset,
    screen_update_table_tests::check_boundaries,
    MAP_HEIGHT_DIGITS_ID,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::{firaga_queued_transfer, SourceRange},
    Bus, Machine,
};
use serde::Deserialize;
use std::{fs, sync::Arc};

const BASE: u32 = 0x0800_0000;
const MAP: u32 = 0x0201_8000;
const MAP_STATE: u32 = 0x0200_7f10;
const TASK: u32 = 0x0203_0000;
const QUEUE: u32 = 0x0300_2bcc;
const TENS: u32 = 0x0601_3600;
const UNITS: u32 = 0x0601_3700;
const SENTINEL: u8 = 0x5a;
const PROOF_SHA: &str = "853b7df8f7d8e0b7c6e387ef2373531598a4af0e71756dd97595661d7fc03419";

#[derive(Deserialize)]
struct Glyph {
    index: usize,
    offset: u32,
    count: usize,
}

#[derive(Deserialize)]
struct Proof {
    glyphs: [Glyph; 10],
    decimal_inputs: u8,
    new_ranges: Vec<SourceRange>,
}

fn glyph_source<'a>(glyph: &Glyph, rom: &'a [u8]) -> &'a [u8] {
    let start = glyph.offset as usize;
    &rom[start..start + glyph.count * 2]
}

pub(super) fn install_transfer_queue_code(machine: &mut Machine, rom: &[u8]) {
    machine.bus.iwram[0x5e78..0x5f28].copy_from_slice(&rom[0xd96838..0xd968e8]);
}

pub(super) fn drain_transfer_queue(machine: &mut Machine) {
    initialize(machine, 0x0800_0800);
    machine.cpu.regs[13] = 0x0300_7d00;
    machine.cpu.regs[14] = 0x0300_7f01;
    execute_until(machine, 0x0300_7f00, 100_000);
}

fn original_decimal_display(machine: &mut Machine, rom: &[u8], proof: &Proof) {
    install_transfer_queue_code(machine, rom);
    machine.bus.write32(MAP_STATE + 4, MAP);
    machine.bus.write8(MAP_STATE + 8, 1);
    for height in 0..proof.decimal_inputs {
        let tens = &proof.glyphs[usize::from(height / 10)];
        let units = &proof.glyphs[usize::from(height % 10)];
        let old_slot = height & 1;
        let slot = u32::from(old_slot ^ 1);
        let first = TASK + 20 + slot * 56;
        let second = TASK + 48 + slot * 56;
        machine.bus.write8(MAP, height);
        machine.bus.write8(TASK, 255);
        machine.bus.write8(TASK + 17, old_slot);
        machine.bus.write32(QUEUE, 0);
        machine.bus.vram[0x135fe..0x13802].fill(SENTINEL);

        // The original getter and division read the map byte, then enqueue
        // the two positions. A zero quotient queues a fill in the first.
        initialize(machine, 0x080a_c8dc);
        machine.cpu.regs[4] = TASK;
        execute_until(machine, 0x080a_c95c, 5000);
        assert_eq!(machine.bus.read32(QUEUE), first);
        assert_eq!(machine.bus.read32(first + 24), second);
        assert_eq!(machine.bus.read32(second + 24), 0);
        assert_eq!(machine.bus.read32(first + 8), TENS);
        assert_eq!(machine.bus.read32(second + 8), UNITS);
        assert_eq!(machine.bus.read32(second + 4), BASE + units.offset);
        assert_eq!(machine.bus.read32(second + 12), 0x8000_0080);
        let (first_source, control) = if height < 10 {
            (0, 0x8100_0080)
        } else {
            (BASE + tens.offset, 0x8000_0080)
        };
        assert_eq!(machine.bus.read32(first + 4), first_source);
        assert_eq!(machine.bus.read32(first + 12), control);

        drain_transfer_queue(machine);
        let first_pixels = if height < 10 {
            &[0; 256][..]
        } else {
            glyph_source(tens, rom)
        };
        assert_eq!(&machine.bus.vram[0x13600..0x13700], first_pixels);
        assert_eq!(
            &machine.bus.vram[0x13700..0x13800],
            glyph_source(units, rom)
        );
        assert_eq!(&machine.bus.vram[0x135fe..0x13600], &[SENTINEL; 2]);
        assert_eq!(&machine.bus.vram[0x13800..0x13802], &[SENTINEL; 2]);
    }
}

#[test]
#[ignore = "Exact EUR ROM, E630 source proof and e367/e368 datasets"]
fn european_map_height_digits_feed_original_queues() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let proof_bytes = fs::read(input("FFTA_MAP_HEIGHT_DIGITS_PROOF")).unwrap();
    assert_eq!(digest(&proof_bytes), PROOF_SHA);
    let proof: Proof = serde_json::from_slice(&proof_bytes).unwrap();
    assert_eq!(proof.decimal_inputs, 100);
    for (index, glyph) in proof.glyphs.iter().enumerate() {
        assert_eq!((glyph.index, glyph.count), (index, 128));
    }

    let mut previous = read_dataset(&input("FFTA_MAP_HEIGHT_DIGITS_PREVIOUS")).unwrap();
    let mut dataset = read_dataset(&input("FFTA_MAP_HEIGHT_DIGITS_DATASET")).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (367, 366)
    );
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (368, 367)
    );

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_dataset(&mut managed, &previous).unwrap();
    assert!(managed
        .bus
        .rom_resources()
        .unwrap()
        .map_height_digits_bus_bytes()
        .is_none());
    attach_dataset(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    previous.manifest.extractor_revision = 368;
    previous.manifest.decoder_revision = 367;
    assert!(attach_dataset(&mut managed, &previous).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|item| item.id == MAP_HEIGHT_DIGITS_ID)
        .unwrap();
    assert_eq!(resource.sources, proof.new_ranges);
    let packed = dataset.map_height_digits_bytes.as_mut().unwrap();
    assert_eq!(packed.len(), 2308);
    packed[0] ^= 1;
    resource.sha256 = digest(packed);
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    let earlier = firaga_queued_transfer::SOURCE_OFFSETS
        .into_iter()
        .zip(firaga_queued_transfer::SOURCE_LENGTHS.map(|length| length as u32));
    let altered = proof
        .new_ranges
        .iter()
        .map(|range| (range.offset, range.length))
        .chain(earlier);
    for (offset, length) in altered.clone() {
        flip(&mut managed, offset, length);
    }
    let boundaries = check_boundaries(&mut raw.bus, &mut managed.bus, &proof.new_ranges);
    let before = owner.map_height_digits_bus_bytes().unwrap();
    let old_before = owner.firaga_queued_transfer_stats().unwrap().source_bytes;
    original_decimal_display(&mut raw, &rom, &proof);
    original_decimal_display(&mut managed, &rom, &proof);
    let managed_bytes = owner.map_height_digits_bus_bytes().unwrap() - before;
    assert!(managed_bytes >= 43_880);
    let old_after = owner.firaga_queued_transfer_stats().unwrap().source_bytes;
    let previous_counts: [u64; 3] = std::array::from_fn(|i| old_after[i] - old_before[i]);
    assert!(previous_counts[0] >= 760 && previous_counts[1] >= 400 && previous_counts[2] >= 5120);
    for (offset, length) in altered {
        flip(&mut managed, offset, length);
    }
    same_state(&raw, &managed, 0x0300_7f00);

    let result = serde_json::json!({
        "status": "PASS",
        "decimal_inputs": proof.decimal_inputs,
        "glyphs": proof.glyphs.len(),
        "leading_fills": 10,
        "glyph_transfers": 190,
        "source_bytes": 2308,
        "earlier_owner_bytes_altered": 264,
        "mixed_mirror_boundary_checks": boundaries,
        "original_read_bytes": managed_bytes,
        "earlier_owner_reads": previous_counts,
        "predecessor_attached": true,
        "invalid_candidates_rejected": 2,
        "full_state_and_clock_equal": true,
    });
    fs::write(
        input("FFTA_MAP_HEIGHT_DIGITS_RESULT"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
