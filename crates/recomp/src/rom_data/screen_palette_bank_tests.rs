//! Verify the two screen palette copies and the staged 256-color loader.
use super::{
    attach_dataset,
    direct_byte_copy_tests::{flip, input, same_state},
    indexed_reader_bank_tests::{execute_until, initialize},
    map_height_digits_tests::install_transfer_queue_code,
    read_dataset,
    screen_update_table_tests::check_boundaries,
    SCREEN_PALETTE_BANK_ID,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::SourceRange,
    Bus, Machine,
};
use serde::Deserialize;
use std::{fs, sync::Arc};

const BASE: u32 = 0x0800_0000;
const PROOF_SHA: &str = "5695b945d14d51c6465aee720ca62e9c9fefed8ebfdac84134eb902135e18c56";
const CONTEXT: u32 = 0x0202_0000;
const CONTEXT_SLOT: u32 = 0x0300_08b0;
const COPY_START: usize = 0x28bd8;
const COPY_LEN: usize = 480;
const STAGED_START: usize = 0x4a68;
const STAGED_COLORS: usize = 256;
const STRIDE: usize = 12;
const SENTINEL: u8 = 0x5a;

#[derive(Deserialize)]
struct Proof {
    colors: usize,
    new_ranges: Vec<SourceRange>,
}

fn copy_screen_palettes(machine: &mut Machine, rom: &[u8]) {
    install_transfer_queue_code(machine, rom);
    machine.bus.write32(0x0200_f3b0, CONTEXT);
    machine.bus.write32(CONTEXT_SLOT, CONTEXT);
    let end = COPY_START + COPY_LEN;
    for (entry, stop, source) in [
        (0x0814_3c38, 0x0814_3c52, 0xb8be58),
        (0x0814_433e, 0x0814_4352, 0xb8c038),
    ] {
        machine.bus.ewram[COPY_START - 2..end + 2].fill(SENTINEL);
        initialize(machine, entry);
        machine.cpu.regs[4] = CONTEXT_SLOT;
        execute_until(machine, stop, 4096);
        assert_eq!(
            &machine.bus.ewram[COPY_START..end],
            &rom[source..source + COPY_LEN]
        );
        assert_eq!(
            &machine.bus.ewram[COPY_START - 2..COPY_START],
            &[SENTINEL; 2]
        );
        assert_eq!(&machine.bus.ewram[end..end + 2], &[SENTINEL; 2]);
    }
}

fn load_staged_palette(machine: &mut Machine, rom: &[u8]) {
    let end = STAGED_START + STAGED_COLORS * STRIDE;
    machine.bus.iwram[STAGED_START - 2..end + 2].fill(SENTINEL);
    initialize(machine, 0x0814_4a80);
    // Stop at the end of the original color loop before it schedules updates.
    execute_until(machine, 0x0814_b73c, 8192);
    for (index, slot) in machine.bus.iwram[STAGED_START..end]
        .chunks_exact(STRIDE)
        .enumerate()
    {
        let offset = 0xb8c378 + 2 * index;
        assert_eq!(&slot[..2], &rom[offset..offset + 2]);
        assert_eq!(&slot[2..], &[SENTINEL; STRIDE - 2]);
    }
    assert_eq!(
        &machine.bus.iwram[STAGED_START - 2..STAGED_START],
        &[SENTINEL; 2]
    );
    assert_eq!(&machine.bus.iwram[end..end + 2], &[SENTINEL; 2]);
}

#[test]
#[ignore = "Exact EUR ROM, E637 source proof and e370/e371 datasets"]
fn european_screen_palette_bank_feeds_original_copy_paths() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let proof_bytes = fs::read(input("FFTA_SCREEN_PALETTE_PROOF")).unwrap();
    assert_eq!(digest(&proof_bytes), PROOF_SHA);
    let proof: Proof = serde_json::from_slice(&proof_bytes).unwrap();
    assert_eq!(proof.colors, 736);

    let mut previous = read_dataset(&input("FFTA_SCREEN_PALETTE_PREVIOUS")).unwrap();
    let mut dataset = read_dataset(&input("FFTA_SCREEN_PALETTE_DATASET")).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (370, 369)
    );
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (371, 370)
    );
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_dataset(&mut managed, &previous).unwrap();
    assert!(managed
        .bus
        .rom_resources()
        .unwrap()
        .screen_palette_bank_bus_bytes()
        .is_none());
    attach_dataset(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    previous.manifest.extractor_revision = 371;
    previous.manifest.decoder_revision = 370;
    assert!(attach_dataset(&mut managed, &previous).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|item| item.id == SCREEN_PALETTE_BANK_ID)
        .unwrap();
    assert_eq!(resource.sources, proof.new_ranges);
    let packed = dataset.screen_palette_bank_bytes.as_mut().unwrap();
    assert_eq!(packed.len(), 1484);
    packed[12] ^= 0x80;
    resource.sha256 = digest(packed);
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    // Alter the complete palette allocation, including its earlier owners.
    flip(&mut managed, 0xb8b9f8, 0xd80);
    for span in &proof.new_ranges {
        if span.offset < 0xb8b9f8 {
            flip(&mut managed, span.offset, span.length);
        }
    }
    for at in 0xb8b9f8..0xb8c778 {
        assert_eq!(
            managed.bus.read8(BASE + at),
            rom[at as usize],
            "color byte {at:06x}"
        );
    }
    let boundaries = check_boundaries(&mut raw.bus, &mut managed.bus, &proof.new_ranges);
    let before = owner.screen_palette_bank_bus_bytes().unwrap();
    for machine in [&mut raw, &mut managed] {
        copy_screen_palettes(machine, &rom);
        load_staged_palette(machine, &rom);
    }
    let original_reads = owner.screen_palette_bank_bus_bytes().unwrap() - before;
    assert_eq!(original_reads, 1484);
    flip(&mut managed, 0xb8b9f8, 0xd80);
    for span in &proof.new_ranges {
        if span.offset < 0xb8b9f8 {
            flip(&mut managed, span.offset, span.length);
        }
    }
    same_state(&raw, &managed, 0x0814_b73c);

    let result = serde_json::json!({
        "status": "PASS",
        "source_bytes": 1484,
        "screen_copies": 2,
        "staged_copy_paths": 1,
        "colors": proof.colors,
        "mixed_mirror_boundary_checks": boundaries,
        "original_read_bytes": original_reads,
        "predecessor_attached": true,
        "invalid_candidates_rejected": 2,
        "full_state_and_clock_equal": true,
    });
    fs::write(
        input("FFTA_SCREEN_PALETTE_RESULT"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
