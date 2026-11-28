//! Verify every localized OBJ directory and frame with the original readers.
use super::{
    attach_dataset,
    direct_byte_copy_tests::{flip, input, same_state},
    indexed_reader_bank_tests::{execute_until, initialize},
    read_dataset, LOCALIZED_OBJ_ANIMATION_BANKS_ID,
};
use gba_core::{
    ffta_rom::digest,
    rom_data::{obj_animations::Bank, SourceRange},
    Bus, Machine,
};
use serde::Deserialize;
use std::{collections::BTreeSet, fs, sync::Arc};

const ROM: u32 = 0x0800_0000;
const RETURN: u32 = ROM + 0x300;
const CONTEXT: u32 = 0x0201_0000;
const LANGUAGE: u32 = 0x0200_1f6e;
const PIXELS: u32 = 0x0202_0000;
const TILE_OFFSET: usize = 0x13c00;
const QUEUE: u32 = 0x0300_2bcc;
const SENTINEL: u8 = 0x5a;

#[derive(Deserialize)]
struct Plan {
    banks: Vec<Case>,
    new_ranges: Vec<SourceRange>,
}

#[derive(Deserialize)]
struct Case {
    object_root: u32,
    sequence_root: u32,
    language: Option<u8>,
    slot: Option<u32>,
    direct_language_branch: Option<String>,
    tile_index: Option<u32>,
    tile_bytes: Option<u32>,
    tile_stream: Option<u32>,
    decoded_tile_bytes: Option<usize>,
    tile_sha256: Option<String>,
}

fn check_table_selection(machine: &mut Machine, case: &Case, first_sequence: u32) {
    let slot = case.slot.unwrap();
    machine.bus.write8(LANGUAGE, case.language.unwrap());
    initialize(machine, 0x080a_c4d0);
    machine.cpu.regs[7] = CONTEXT;
    machine.cpu.regs[9] = slot;
    execute_until(machine, 0x080a_c512, 100);
    assert_eq!(machine.bus.read32(CONTEXT), ROM + case.object_root);
    assert_eq!(machine.bus.read32(CONTEXT + 4), ROM + case.sequence_root);
    assert_eq!(machine.bus.read32(CONTEXT + 24), ROM + first_sequence);

    // Provide the same working RAM inputs to the queue-setup prefix for every
    // descriptor. The original prefix computes both source and transfer size.
    let parameters = CONTEXT + 0x200;
    initialize(machine, 0x080a_c590);
    machine.cpu.regs[4] = 2 * slot;
    machine.cpu.regs[5] = parameters + 2;
    machine.cpu.regs[7] = CONTEXT;
    machine.cpu.regs[8] = parameters;
    machine.cpu.regs[9] = slot;
    machine.cpu.regs[12] = parameters + 0x100;
    machine.bus.write16(CONTEXT + 8, 0);
    machine.bus.write32(parameters + 0x100 + 76, PIXELS);
    execute_until(machine, 0x080a_c5de, 100);
    assert_eq!(machine.cpu.regs[1], PIXELS + 32 * case.tile_index.unwrap());
    assert_eq!(machine.cpu.regs[2], 0x0601_0140);
    assert_eq!(machine.cpu.regs[3], case.tile_bytes.unwrap() / 4);
}

fn check_direct_upload(machine: &mut Machine, case: &Case, first_sequence: u32) {
    let language = match case.direct_language_branch.as_deref() {
        Some("default") => 0,
        Some("2") => 2,
        Some("4") => 4,
        other => panic!("Unexpected direct language case {other:?}"),
    };
    let size = case.decoded_tile_bytes.unwrap();
    let expected = case.tile_sha256.as_deref().unwrap();
    machine.bus.write8(LANGUAGE, language);
    initialize(machine, 0x0809_5480);
    execute_until(machine, 0x0809_54e0, 100);
    assert_eq!(machine.cpu.regs[5] as usize, size);
    assert_eq!(machine.cpu.regs[6], ROM + case.sequence_root);
    assert_eq!(machine.cpu.regs[7], ROM + case.tile_stream.unwrap());
    assert_eq!(machine.cpu.regs[8], ROM + case.object_root);

    // Enter the original caller after allocation with a dedicated WRAM buffer.
    machine.cpu.regs[0] = PIXELS;
    machine.cpu.regs[15] = 0x0809_54fc;
    execute_until(machine, 0x0809_5502, 150_000);
    let start = (PIXELS - 0x0200_0000) as usize;
    assert_eq!(digest(&machine.bus.ewram[start..start + size]), expected);

    machine.cpu.regs[4] = CONTEXT;
    machine.bus.write32(CONTEXT + 88, PIXELS);
    machine.bus.vram[TILE_OFFSET - 4..TILE_OFFSET + size + 4].fill(SENTINEL);
    execute_until(machine, 0x0809_551c, 200);
    assert_eq!(machine.bus.read32(CONTEXT), ROM + case.object_root);
    assert_eq!(machine.bus.read32(CONTEXT + 4), ROM + case.sequence_root);
    assert_eq!(machine.bus.read32(CONTEXT + 24), ROM + first_sequence);
    let node = CONTEXT + 60;
    assert_eq!(machine.bus.read32(QUEUE), node);
    assert_eq!(machine.bus.read32(node + 4), PIXELS);
    assert_eq!(machine.bus.read32(node + 8), 0x0601_3c00);
    assert_eq!(
        machine.bus.read32(node + 12),
        0x8000_0000 | (size as u32 / 2)
    );
    initialize(machine, 0x0800_0800);
    execute_until(machine, RETURN, 1000);
    assert_eq!(machine.bus.read32(QUEUE), 0);
    assert_eq!(
        digest(&machine.bus.vram[TILE_OFFSET..TILE_OFFSET + size]),
        expected
    );
    assert_eq!(
        &machine.bus.vram[TILE_OFFSET - 4..TILE_OFFSET],
        &[SENTINEL; 4]
    );
    assert_eq!(
        &machine.bus.vram[TILE_OFFSET + size..TILE_OFFSET + size + 4],
        &[SENTINEL; 4]
    );
}

fn update(machine: &mut Machine, sequence: u32, index: usize) {
    machine.bus.write32(CONTEXT + 24, ROM + sequence);
    machine.bus.write16(CONTEXT + 14, index as u16);
    machine.bus.write16(CONTEXT + 16, 0);
    machine.bus.write8(CONTEXT + 29, 1);
    initialize(machine, 0x080a_ccf4);
    machine.cpu.regs[0] = CONTEXT;
    execute_until(machine, RETURN, 100);
}

pub(super) fn draw(machine: &mut Machine, at: u32, x: i16, y: i8) {
    initialize(machine, 0x0800_1f6c);
    // Position the first object on screen. Transition lists can start far
    // enough outside the viewport for a nonempty list to produce no OAM.
    let origin_x = (120i32 - i32::from(x)) as u32;
    let origin_y = (80i32 - i32::from(y)) as u32;
    machine.cpu.regs[..4].copy_from_slice(&[at, origin_x, origin_y, 0]);
    let stack = machine.cpu.regs[13];
    for word in 0..7 {
        machine.bus.write32(stack + 4 * word, 0);
    }
    machine.bus.write8(0x0300_0028, 0);
    machine.bus.write32(0x0300_0020, 0);
    execute_until(machine, RETURN, 10_000);
    assert!(
        machine.bus.read32(0x0300_0020) > 0,
        "No shadow OAM entries for {at:#x}"
    );
}

fn check_timed_reads(machine: &mut Machine, bank: &Bank, sample_draw: bool) -> (u64, u64) {
    let mut frames = 0;
    let mut drawn = 0;
    for sequence in &bank.sequences {
        for (index, frame) in sequence.frames.iter().enumerate() {
            update(machine, sequence.source_offset, index);
            let list = &bank.lists[usize::from(frame.object_list)];
            let pointer = ROM + list.source_offset;
            assert_eq!(machine.bus.read32(CONTEXT + 20), pointer);
            assert_eq!(machine.bus.read16(CONTEXT + 14), index as u16 + 1);
            assert_eq!(machine.bus.read16(CONTEXT + 16), frame.duration);
            if sample_draw && drawn == 0 {
                if let Some(first) = list.objects.first() {
                    draw(machine, pointer, first.x, first.y);
                    drawn += 1;
                }
            }
            frames += 1;
        }
        update(machine, sequence.source_offset, sequence.frames.len());
        assert_eq!(machine.bus.read32(CONTEXT + 20), 0);
        assert_eq!(machine.bus.read8(CONTEXT + 29), 0);
    }
    (frames, drawn)
}

fn source_reads(raw: &mut Machine, managed: &mut Machine, spans: &[SourceRange]) -> (u64, usize) {
    let owner = managed.bus.rom_resources().unwrap().clone();
    let before = owner.localized_obj_animation_bank_bus_bytes().unwrap();
    let mut bytes = 0;
    for span in spans {
        let start = ROM + span.offset;
        let end = start + span.length;
        let words_end = start + span.length / 4 * 4;
        for at in (start..words_end).step_by(4) {
            assert_eq!(
                raw.bus.read32(at),
                managed.bus.read32(at),
                "Word at {at:#x}"
            );
        }
        for at in words_end..end {
            assert_eq!(raw.bus.read8(at), managed.bus.read8(at), "Byte at {at:#x}");
        }
        bytes += u64::from(span.length);
    }
    assert_eq!(
        owner.localized_obj_animation_bank_bus_bytes().unwrap() - before,
        bytes
    );
    let mut roles = BTreeSet::new();
    let mut width_checks = 0;
    for span in spans {
        if !roles.insert(&span.role) {
            continue;
        }
        for boundary in [span.offset, span.offset + span.length] {
            for at in boundary - 3..=boundary + 3 {
                for base in [ROM, 0x0a00_0000, 0x0c00_0000] {
                    let address = base + at;
                    assert_eq!(raw.bus.read8(address), managed.bus.read8(address));
                    assert_eq!(raw.bus.read16(address), managed.bus.read16(address));
                    assert_eq!(raw.bus.read32(address), managed.bus.read32(address));
                    width_checks += 3;
                }
            }
        }
    }
    (bytes, width_checks)
}

#[test]
#[ignore = "Exact EUR ROM, authenticated E607 plan and managed e356/e357 datasets"]
fn european_localized_obj_animation_banks_feed_original_readers() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    let plan: Plan =
        serde_json::from_slice(&fs::read(input("FFTA_LOCALIZED_OBJ_PLAN")).unwrap()).unwrap();
    assert_eq!(plan.banks.len(), 133);
    let mut previous = read_dataset(&input("FFTA_LOCALIZED_OBJ_PREDECESSOR")).unwrap();
    let mut dataset = read_dataset(&input("FFTA_LOCALIZED_OBJ_DATASET")).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (356, 355)
    );
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (357, 356)
    );
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_dataset(&mut managed, &previous).unwrap();
    assert!(managed
        .bus
        .rom_resources()
        .unwrap()
        .localized_obj_animation_bank_bus_bytes()
        .is_none());
    attach_dataset(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    (
        previous.manifest.extractor_revision,
        previous.manifest.decoder_revision,
    ) = (357, 356);
    assert!(attach_dataset(&mut managed, &previous).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|row| row.id == LOCALIZED_OBJ_ANIMATION_BANKS_ID)
        .unwrap();
    for (published, expected) in resource.sources.iter().zip(&plan.new_ranges) {
        assert_eq!(
            (published.offset, published.length, &published.sha256),
            (expected.offset, expected.length, &expected.sha256)
        );
    }
    assert_eq!(resource.sources.len(), plan.new_ranges.len());
    let bytes = dataset
        .localized_obj_animation_banks_bytes
        .as_mut()
        .unwrap();
    bytes[0] ^= 1;
    resource.sha256 = digest(bytes);
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    for span in &plan.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    let (source_bytes, mirror_checks) = source_reads(&mut raw, &mut managed, &plan.new_ranges);
    assert_eq!(source_bytes, 113_870);
    let before = owner.localized_obj_animation_bank_bus_bytes().unwrap();
    let mut frames = 0;
    let mut draw_cases = 0;
    let mut uploads = 0;
    for case in &plan.banks {
        let bank = Bank::decode(&rom, case.object_root, case.sequence_root).unwrap();
        let first = bank.sequences[0].source_offset;
        let sample_draw = case.slot.is_none() || case.slot == Some(2);
        let results = [&mut raw, &mut managed].map(|machine| {
            if case.slot.is_some() {
                check_table_selection(machine, case, first);
            } else {
                check_direct_upload(machine, case, first);
            }
            check_timed_reads(machine, &bank, sample_draw)
        });
        assert_eq!(results[0], results[1]);
        assert_eq!(raw.bus.clock, managed.bus.clock);
        frames += results[0].0;
        draw_cases += results[0].1;
        uploads += usize::from(case.slot.is_none());
    }
    assert_eq!((frames, draw_cases, uploads), (4_435, 8, 3));
    let original_read_bytes = owner.localized_obj_animation_bank_bus_bytes().unwrap() - before;
    assert!(original_read_bytes > 40_000);
    for span in &plan.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    same_state(&raw, &managed, 0x080a_ccf4);

    fs::write(
        input("FFTA_LOCALIZED_OBJ_RESULT"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "status": "PASS",
            "registry_cases": 130,
            "direct_uploads": uploads,
            "timed_frames": frames,
            "draw_cases": draw_cases,
            "source_bytes": source_bytes,
            "mixed_mirror_checks": mirror_checks,
            "original_read_bytes": original_read_bytes,
            "e356_attachment": true,
            "invalid_candidates_rejected": 2,
            "full_state_and_clock_equal": true,
        }))
        .unwrap(),
    )
    .unwrap();
}
