//! Verify the original reset and transfer paths for the newly served unit OBJ data.
use super::{
    attach_dataset,
    direct_byte_copy_tests::{flip, input, same_state},
    indexed_reader_bank_tests::{execute_until, initialize},
    map_height_digits_tests::{drain_transfer_queue, install_transfer_queue_code},
    read_dataset,
    screen_update_table_tests::check_boundaries,
    PHYSICAL_UNIT_OBJ_BANK_ID,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::{
        obj_lists::List,
        unit_obj_equipment::Frame,
        unit_obj_initial_pose::{
            transfer_bytes, transfer_tiles, DIRECTORY, OBJECT_BASE, PIXEL_BASE,
        },
        SourceRange,
    },
    Bus, Machine,
};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    ops::Range,
    sync::Arc,
};

const BASE: u32 = 0x0800_0000;
const RETURN: u32 = 0x0800_0300;
const INDEX: u32 = 0x371494;
const DIRECTORY_END: u32 = 0x394c14;
const OBJECT_END: u32 = 0xa5129c;
const TASK: u32 = 0x0201_0000;
const POOL: u32 = 0x0201_1000;
const QUEUE: u32 = 0x0300_2bcc;
const DESTINATION: u32 = 0x0601_0200;
const PROOF_SHA: &str = "4f1a28f6803033b0b8d55f45be8c4922d098325bdfcb78403bb3892ffea78794";

#[derive(Deserialize)]
struct IndexRow {
    offset: u32,
    end: u32,
    sprite_ids: Vec<u32>,
}

#[derive(Deserialize)]
struct Proof {
    index_rows: Vec<IndexRow>,
    new_ranges: Vec<SourceRange>,
}

fn bytes(rom: &[u8], at: u32, len: u32) -> &[u8] {
    &rom[at as usize..(at + len) as usize]
}

fn half(rom: &[u8], at: u32) -> u16 {
    u16::from_le_bytes(bytes(rom, at, 2).try_into().unwrap())
}

fn word(rom: &[u8], at: u32) -> u32 {
    u32::from_le_bytes(bytes(rom, at, 4).try_into().unwrap())
}

fn new_bytes(spans: &[SourceRange], range: Range<u32>) -> u32 {
    let first = spans.partition_point(|span| span.offset + span.length <= range.start);
    spans[first..]
        .iter()
        .take_while(|span| span.offset < range.end)
        .map(|span| (span.offset + span.length).min(range.end) - span.offset.max(range.start))
        .sum()
}

fn reset_pose(
    machine: &mut Machine,
    sprite: u32,
    animation: u32,
    part: u32,
    sequence: u32,
    rom: &[u8],
) {
    machine.bus.ewram[0x10000..0x10048].fill(0x5a);
    initialize(machine, 0x0802_2e20);
    machine.cpu.regs[..4].copy_from_slice(&[TASK, sprite, animation, part]);
    execute_until(machine, RETURN, 96);
    if sequence == 0 {
        assert_eq!(machine.cpu.regs[0], 0);
        assert_eq!(machine.bus.ewram[0x10000..0x10048], [0x5a; 0x48]);
    } else {
        assert_eq!(machine.cpu.regs[0], 1);
        assert_eq!(machine.bus.read16(TASK + 0x0e), half(rom, sequence - BASE));
        assert_eq!(machine.bus.read32(TASK + 0x34), sequence + 4);
        assert_eq!(machine.bus.read32(TASK + 0x38), sequence + 4);
        assert_eq!(machine.bus.read16(TASK + 6), sprite as u16);
        assert_eq!(machine.bus.read16(TASK + 8), animation as u16);
    }
}

fn begin_frame(machine: &mut Machine, frame: u32, entry: u32) {
    machine.bus.ewram[0x10000..0x10048].fill(0);
    machine.bus.ewram[0x11000..0x111d4].fill(0);
    machine.bus.write32(TASK, 0x45);
    machine.bus.write16(TASK + 0x0e, 1);
    machine.bus.write16(TASK + 0x12, 0x10);
    machine.bus.write32(TASK + 0x20, u32::MAX);
    machine.bus.write32(TASK + 0x28, 0x1357_9bdf);
    machine.bus.write32(TASK + 0x38, BASE + frame);
    machine.bus.write16(POOL + 0x1cc, 4);
    machine.bus.write32(QUEUE, 0);
    initialize(machine, entry);
    machine.cpu.regs[..2].copy_from_slice(&[TASK, POOL]);
}

fn upload(machine: &mut Machine, rom: &[u8], frame: u32, source: u32, len: u32) {
    let start = (DESTINATION - 0x0600_0000) as usize;
    let end = start + len as usize;
    machine.bus.vram[start - 2..end + 2].fill(0x5a);
    begin_frame(machine, frame, 0x0802_2400);
    execute_until(machine, RETURN, 4096);
    assert_eq!(machine.bus.read32(QUEUE), POOL);
    assert_eq!(machine.bus.read32(POOL + 4), BASE + source);
    assert_eq!(machine.bus.read32(POOL + 8), DESTINATION);
    assert_eq!(machine.bus.read32(POOL + 12), 0x8400_0000 | len / 4);
    drain_transfer_queue(machine);
    assert_eq!(&machine.bus.vram[start..end], bytes(rom, source, len));
    assert_eq!(&machine.bus.vram[start - 2..start], &[0x5a; 2]);
    assert_eq!(&machine.bus.vram[end..end + 2], &[0x5a; 2]);
    assert_eq!(machine.bus.read32(QUEUE), 0);
}

#[test]
#[ignore = "Exact EUR ROM, E635 source proof and e369/e370 datasets"]
fn european_physical_unit_obj_bank_feeds_original_pose_and_dma_readers() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let proof_bytes = fs::read(input("FFTA_UNIT_OBJ_BANK_PROOF")).unwrap();
    assert_eq!(digest(&proof_bytes), PROOF_SHA);
    let proof: Proof = serde_json::from_slice(&proof_bytes).unwrap();

    let mut previous = read_dataset(&input("FFTA_UNIT_OBJ_BANK_PREVIOUS")).unwrap();
    let mut dataset = read_dataset(&input("FFTA_UNIT_OBJ_BANK_DATASET")).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (369, 368)
    );
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (370, 369)
    );
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_dataset(&mut managed, &previous).unwrap();
    assert!(managed
        .bus
        .rom_resources()
        .unwrap()
        .physical_unit_obj_bank_bus_bytes()
        .is_none());
    attach_dataset(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    previous.manifest.extractor_revision = 370;
    previous.manifest.decoder_revision = 369;
    assert!(attach_dataset(&mut managed, &previous).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|item| item.id == PHYSICAL_UNIT_OBJ_BANK_ID)
        .unwrap();
    assert_eq!(resource.sources, proof.new_ranges);
    assert_eq!(resource.sources.len(), 4751);
    let packed = dataset.physical_unit_obj_bank_bytes.as_mut().unwrap();
    assert_eq!(packed.len(), 154032);
    packed[0] ^= 1;
    resource.sha256 = digest(packed);
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    let sequences: BTreeSet<_> = (INDEX..DIRECTORY)
        .step_by(4)
        .filter_map(|cell| word(&rom, cell).checked_sub(BASE))
        .collect();
    assert_eq!(sequences.len(), 4934);
    // Alter all file-backed graph bytes, including the earlier owners.
    // Alignment and neighboring data stay available through the raw bus.
    let altered: Vec<_> = [
        INDEX..DIRECTORY_END,
        PIXEL_BASE..OBJECT_END,
        0x02219c..0x0221a0,
        0x0221b8..0x0221bc,
        0x0221c0..0x0221c4,
        0x0221d0..0x0221d4,
    ]
    .into_iter()
    .chain(
        sequences
            .iter()
            .map(|&at| at..at + 4 + 20 * u32::from(half(&rom, at))),
    )
    .collect();
    for span in &altered {
        flip(&mut managed, span.start, span.end - span.start);
    }
    for span in &proof.new_ranges {
        for at in span.offset..span.offset + span.length {
            assert_eq!(
                managed.bus.read8(BASE + at),
                rom[at as usize],
                "source byte {at:06x}"
            );
        }
    }
    let mut edges = BTreeMap::new();
    for span in &proof.new_ranges {
        edges
            .entry(span.role.as_str())
            .and_modify(|(_, last)| *last = span)
            .or_insert((span, span));
    }
    let mut boundary_spans = Vec::new();
    for (first, last) in edges.into_values() {
        boundary_spans.push(first.clone());
        if first != last {
            boundary_spans.push(last.clone());
        }
    }
    let boundaries = check_boundaries(&mut raw.bus, &mut managed.bus, &boundary_spans);

    let before = owner.physical_unit_obj_bank_bus_bytes().unwrap();
    let mut reset_cells = 0;
    let mut null_cells = 0;
    let mut aliases = 0;
    let mut reached = BTreeSet::new();
    for row in &proof.index_rows {
        for cell in (row.offset..row.end).step_by(4) {
            if new_bytes(&proof.new_ranges, cell..cell + 4) == 0 {
                continue;
            }
            let displacement = cell - row.offset;
            let animation = 4 * (displacement / 24) + (displacement % 24) / 12;
            let part = displacement % 12 / 4;
            let sequence = word(&rom, cell);
            for machine in [&mut raw, &mut managed] {
                reset_pose(machine, row.sprite_ids[0], animation, part, sequence, &rom);
            }
            reset_cells += 1;
            if sequence == 0 {
                null_cells += 1;
            } else {
                reached.insert(sequence - BASE);
            }
        }
        if row.sprite_ids.len() > 1 {
            let cell = (row.offset..row.end)
                .step_by(4)
                .find(|&at| word(&rom, at) != 0)
                .unwrap();
            let displacement = cell - row.offset;
            let animation = 4 * (displacement / 24) + 3 - (displacement % 24) / 12;
            let part = displacement % 12 / 4;
            for &sprite in &row.sprite_ids {
                for machine in [&mut raw, &mut managed] {
                    reset_pose(machine, sprite, animation, part, word(&rom, cell), &rom);
                }
                aliases += 1;
            }
        }
    }
    assert_eq!((reset_cells, null_cells, aliases), (11596, 10717, 6));
    assert!(owner.physical_unit_obj_bank_bus_bytes().unwrap() - before >= 46384);

    let mut affected_sequences = 0;
    let mut commands = BTreeMap::new();
    let mut uploads = BTreeMap::new();
    for sequence in sequences {
        let end = sequence + 4 + 20 * u32::from(half(&rom, sequence));
        if new_bytes(&proof.new_ranges, sequence..end) != 0 {
            assert!(reached.contains(&sequence), "new sequence {sequence:06x}");
            affected_sequences += 1;
        }
        for at in (sequence + 4..end).step_by(20) {
            let frame = Frame::from_bytes(bytes(&rom, at, 20).try_into().unwrap());
            if frame.operation != 1 {
                if new_bytes(&proof.new_ranges, at..at + 20) != 0 {
                    commands.entry(frame.operation).or_insert(at);
                }
                continue;
            }
            let object = OBJECT_BASE + frame.word04;
            let size = 2 + 6 * u32::from(half(&rom, object));
            let list = List::decode(object, bytes(&rom, object, size)).unwrap();
            let len = transfer_bytes(transfer_tiles(&list));
            let pixel = PIXEL_BASE + frame.word00;
            let novel = new_bytes(&proof.new_ranges, pixel..pixel + len);
            if novel != 0 || new_bytes(&proof.new_ranges, object..object + size) != 0 {
                uploads.entry(pixel).or_insert((at, len, novel));
            }
        }
    }
    assert_eq!(affected_sequences, 817);
    assert_eq!(commands.keys().copied().collect::<Vec<_>>(), [2, 3, 6, 8]);
    for &frame in commands.values() {
        for machine in [&mut raw, &mut managed] {
            begin_frame(machine, frame, 0x0802_2254);
            execute_until(machine, RETURN, 4096);
        }
        assert_eq!(managed.cpu.regs, raw.cpu.regs);
        assert_eq!(
            managed.bus.ewram[0x10000..0x10048],
            raw.bus.ewram[0x10000..0x10048]
        );
        assert_eq!(managed.bus.clock, raw.bus.clock);
    }

    install_transfer_queue_code(&mut raw, &rom);
    install_transfer_queue_code(&mut managed, &rom);
    let before_uploads = owner.physical_unit_obj_bank_bus_bytes().unwrap();
    let mut novel_pixels = 0;
    for (&pixel, &(frame, len, novel)) in &uploads {
        for machine in [&mut raw, &mut managed] {
            upload(machine, &rom, frame, pixel, len);
        }
        novel_pixels += novel;
    }
    assert_eq!((uploads.len(), novel_pixels), (138, 45120));
    assert!(
        owner.physical_unit_obj_bank_bus_bytes().unwrap() - before_uploads
            >= u64::from(novel_pixels)
    );
    for span in &altered {
        flip(&mut managed, span.start, span.end - span.start);
    }
    same_state(&raw, &managed, 0x0300_7f00);
    let result = serde_json::json!({
        "status": "PASS",
        "source_bytes": 154032,
        "reset_cells": reset_cells,
        "null_cells": null_cells,
        "directory_alias_cases": aliases,
        "affected_sequences": affected_sequences,
        "command_operations": commands.len(),
        "dma_cases": uploads.len(),
        "new_dma_bytes": novel_pixels,
        "mixed_mirror_boundary_checks": boundaries,
        "original_read_bytes": owner.physical_unit_obj_bank_bus_bytes().unwrap() - before,
        "predecessor_attached": true,
        "invalid_candidates_rejected": 2,
        "full_state_and_clock_equal": true,
    });
    fs::write(
        input("FFTA_UNIT_OBJ_BANK_RESULT"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
