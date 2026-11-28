//! Verify the European CRC32 lookup and original save checksum call sites.
use super::{
    attach_dataset,
    direct_byte_copy_tests::{flip, input, same_state},
    indexed_reader_bank_tests::{execute_until, initialize},
    read_dataset,
    screen_update_table_tests::check_boundaries,
    SAVE_CRC32_TABLE_ID,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::SourceRange,
    Bus, Machine,
};
use serde::Deserialize;
use std::{fs, sync::Arc};

const BASE: u32 = 0x0800_0000;
const RETURN: u32 = 0x0800_0300;
const EWRAM: u32 = 0x0200_0000;
const INPUT_BUFFER: u32 = 0x0202_0000;
const SAVE_BUFFER: u32 = 0x0202_8000;
const POLYNOMIAL: u32 = 0xedb8_8320;
const PROOF_SHA: &str = "bf6ba89fd55b3bf6c0800c38c4fae84786adb89dd87373bc0b5555a35b1e4d83";
const SAVE_MODES: [(u32, usize); 2] = [(0, 0x2fc4), (2, 0x3ca8)];

#[derive(Deserialize)]
struct Proof {
    table: [u32; 2],
    words: usize,
    polynomial: u32,
    reader: u32,
    direct_calls: [u32; 2],
    new_ranges: Vec<SourceRange>,
}

fn reference_crc(bytes: &[u8], seed: u32) -> u32 {
    let mut state = !seed;
    for &byte in bytes {
        state ^= u32::from(byte);
        for _ in 0..8 {
            let feedback = if state & 1 != 0 { POLYNOMIAL } else { 0 };
            state = (state >> 1) ^ feedback;
        }
    }
    !state
}

fn original_crc(machine: &mut Machine, reader: u32, bytes: &[u8], seed: u32) -> u32 {
    let offset = (INPUT_BUFFER - EWRAM) as usize;
    machine.bus.ewram[offset..offset + bytes.len()].copy_from_slice(bytes);
    initialize(machine, BASE + reader);
    machine.cpu.regs[0] = INPUT_BUFFER;
    machine.cpu.regs[1] = bytes.len() as u32;
    machine.cpu.regs[2] = seed;
    execute_until(machine, RETURN, 20 * bytes.len() + 256);
    machine.cpu.regs[0]
}

fn write_checksum(machine: &mut Machine, mode: u32, length: usize, call: u32) {
    initialize(machine, 0x0813_f3b0);
    machine.cpu.regs[0] = mode;
    machine.cpu.regs[1] = SAVE_BUFFER;
    execute_until(machine, 0x0813_f3d0, 64);
    assert_eq!(machine.cpu.regs[8], length as u32);
    // Continue from the checksum block with the prepared scratch payload.
    machine.cpu.regs[15] = BASE + call - 6;
    execute_until(machine, BASE + call + 6, 20 * length + 256);
}

fn verify_checksum(machine: &mut Machine, mode: u32, length: usize, call: u32, matches: bool) {
    initialize(machine, 0x0813_f6a0);
    machine.cpu.regs[0] = mode;
    machine.cpu.regs[1] = SAVE_BUFFER;
    execute_until(machine, 0x0813_f6c4, 64);
    let length_slot = machine.cpu.regs[13] + 0x14;
    assert_eq!(machine.bus.read32(length_slot), length as u32);
    // Enter with the successful signature check result and the scratch payload.
    machine.cpu.regs[0] = 0;
    machine.cpu.regs[15] = BASE + call - 0x14;
    execute_until(machine, BASE + call + 6, 20 * length + 256);
    machine.step();
    if matches {
        assert_eq!(machine.cpu.regs[15], 0x0813_f6ec);
        execute_until(machine, 0x0813_f6ee, 4);
    } else {
        assert_eq!(machine.cpu.regs[15], 0x0813_f77c);
    }
}

fn original_save_paths(machine: &mut Machine, [writer, verifier]: [u32; 2]) {
    let offset = (SAVE_BUFFER - EWRAM) as usize;
    for (mode, length) in SAVE_MODES {
        let end = offset + length;
        machine.bus.ewram[offset - 2..end + 2].fill(0x5a);
        let mut frame: Vec<_> = (0..length)
            .map(|index| (index * 197 / 31 + 53) as u8)
            .collect();
        frame[0x0c..0x10].fill(0);
        frame[0x12..=0x16].fill(0);
        let expected = reference_crc(&frame, 0);
        machine.bus.ewram[offset..end].copy_from_slice(&frame);
        write_checksum(machine, mode, length, writer);
        frame[0x0c..0x10].copy_from_slice(&expected.to_le_bytes());
        assert_eq!(&machine.bus.ewram[offset..end], frame.as_slice());

        machine.bus.ewram[offset + 0x12..=offset + 0x16].fill(0x24);
        verify_checksum(machine, mode, length, verifier, true);
        assert_eq!(&machine.bus.ewram[offset..end], frame.as_slice());

        frame[0x100] ^= 0x20;
        machine.bus.ewram[offset + 0x100] = frame[0x100];
        machine.bus.ewram[offset + 0x12..=offset + 0x16].fill(0x51);
        verify_checksum(machine, mode, length, verifier, false);
        frame[0x0c..0x10].fill(0);
        assert_ne!(reference_crc(&frame, 0), expected);
        assert_eq!(&machine.bus.ewram[offset..end], frame.as_slice());
        assert_eq!(&machine.bus.ewram[offset - 2..offset], &[0x5a; 2]);
        assert_eq!(&machine.bus.ewram[end..end + 2], &[0x5a; 2]);
    }
}

#[test]
#[ignore = "Exact EUR ROM, E645 source proof and e374/e375 datasets"]
fn european_save_crc32_table_feeds_original_checksum_paths() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let proof_bytes = fs::read(input("FFTA_SAVE_CRC32_PROOF")).unwrap();
    assert_eq!(digest(&proof_bytes), PROOF_SHA);
    let proof: Proof = serde_json::from_slice(&proof_bytes).unwrap();
    assert_eq!(proof.words, 256);
    assert_eq!(proof.polynomial, POLYNOMIAL);
    assert_eq!(proof.table[1] - proof.table[0], 4 * proof.words as u32);

    let mut previous = read_dataset(&input("FFTA_SAVE_CRC32_PREVIOUS")).unwrap();
    let mut dataset = read_dataset(&input("FFTA_SAVE_CRC32_DATASET")).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (374, 373)
    );
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (375, 374)
    );
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom, None);
    attach_dataset(&mut managed, &previous).unwrap();
    assert!(managed
        .bus
        .rom_resources()
        .unwrap()
        .save_crc32_table_bus_bytes()
        .is_none());
    attach_dataset(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    previous.manifest.extractor_revision = 375;
    previous.manifest.decoder_revision = 374;
    assert!(attach_dataset(&mut managed, &previous).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|item| item.id == SAVE_CRC32_TABLE_ID)
        .unwrap();
    assert_eq!(resource.sources, proof.new_ranges);
    let packed = dataset.save_crc32_table_bytes.as_mut().unwrap();
    let source_bytes = packed.len();
    assert_eq!(source_bytes, 1032);
    packed[source_bytes - 1] ^= 0x80;
    resource.sha256 = digest(packed);
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    for span in &proof.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    let boundaries = check_boundaries(&mut raw.bus, &mut managed.bus, &proof.new_ranges);
    let before = owner.save_crc32_table_bus_bytes().unwrap();
    let seeded = 0x1020_3040;
    let check_vector = b"123456789";
    assert_eq!(reference_crc(check_vector, 0), 0xcbf4_3926);
    for machine in [&mut raw, &mut managed] {
        assert_eq!(original_crc(machine, proof.reader, &[], seeded), seeded);
        for index in 0..proof.words {
            // With seed zero the low complemented state byte is 0xff.
            let byte = [!(index as u8)];
            assert_eq!(
                original_crc(machine, proof.reader, &byte, 0),
                reference_crc(&byte, 0)
            );
        }
        for seed in [0, seeded] {
            assert_eq!(
                original_crc(machine, proof.reader, check_vector, seed),
                reference_crc(check_vector, seed)
            );
        }
        original_save_paths(machine, proof.direct_calls);
    }
    let original_reads = owner.save_crc32_table_bus_bytes().unwrap() - before;
    let check_vector_reads = 2 * (8 + 4 * check_vector.len());
    let save_reads: usize = SAVE_MODES.iter().map(|&(_, len)| 3 * (8 + 4 * len)).sum();
    assert_eq!(
        original_reads,
        (proof.words * 12 + check_vector_reads + save_reads) as u64
    );
    for span in &proof.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    same_state(&raw, &managed, 0x0813_f77c);

    let result = serde_json::json!({
        "status": "PASS",
        "source_bytes": source_bytes,
        "table_indices": proof.words,
        "crc_vector_cases": 3,
        "original_save_payload_lengths": SAVE_MODES.map(|(_, length)| length),
        "original_checksum_writes": SAVE_MODES.len(),
        "original_valid_branches": SAVE_MODES.len(),
        "original_invalid_branches": SAVE_MODES.len(),
        "mixed_mirror_boundary_checks": boundaries,
        "original_read_bytes": original_reads,
        "predecessor_attached": true,
        "invalid_candidates_rejected": 2,
        "full_state_and_clock_equal": true,
    });
    fs::write(
        input("FFTA_SAVE_CRC32_RESULT"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
