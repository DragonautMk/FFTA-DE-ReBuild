//! Exercise both original European world-map label paths.

use super::{
    INDEXED_GRAPHICS_ID, IndexedGraphics, attach_dataset,
    direct_byte_copy_tests::{flip, input, same_state},
    encode_indexed_graphics,
    indexed_reader_bank_tests::{execute_until, initialize},
    localized_obj_animation_bank_tests::draw,
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
const LANGUAGE: u32 = 0x0200_1f6e;
const CONTEXT: u32 = 0x0300_2810;
const PARENT: u32 = 0x0202_f000;
const VIEW: u32 = 0x0203_1000;
const DESTINATION: u32 = 0x0200_3cb0;
const OBJ_DIRECTORY: u32 = 0x39573c;
const RAW_ALIGNMENT: u32 = 0x395ca6;
const PROOF_SHA: &str = "390af148ec21ea0c9403dcd15216ae3fc2c2a5819dda4424cb3704e9334f6c79";

#[derive(Deserialize)]
struct LanguageBank {
    language: u8,
    archive: u32,
    ranges: [[u32; 2]; 30],
}

#[derive(Deserialize)]
struct ObjList {
    offset: u32,
    entries: u32,
}

#[derive(Deserialize)]
struct Proof {
    languages: [LanguageBank; 5],
    obj_lists: [ObjList; 7],
    obj_slots: [u32; 8],
    new_ranges: Vec<SourceRange>,
}

fn original_label_paths(machine: &mut Machine, graphics: &IndexedGraphics, proof: &Proof) {
    machine.bus.write32(CONTEXT, PARENT);
    machine.bus.write32(PARENT + 4, VIEW);
    let additional = graphics.additional.as_ref().unwrap();
    for bank in &proof.languages {
        machine.bus.write8(LANGUAGE, bank.language);
        let archive = additional
            .archives
            .iter()
            .find(|a| a.source_offset == bank.archive)
            .unwrap();
        for (node, [start, end]) in bank.ranges.into_iter().enumerate() {
            let count = end - start;
            let variant = (count - 5) as usize;
            for (entry, decoder, state, destination) in [
                (0x037b7c, 0x037bd6, PARENT, CONTEXT),
                (0x037ddc, 0x037e42, VIEW, DESTINATION),
            ] {
                initialize(machine, BASE + entry);
                machine.bus.write8(PARENT + 0x674, 0);
                machine.bus.write8(VIEW + 0x13, 3);
                machine.bus.write8(VIEW + 0x1b, node as u8 + 1);
                machine.cpu.regs[3] = state;
                machine.cpu.regs[6] = CONTEXT;
                machine.cpu.regs[7] = node as u32;
                machine.cpu.regs[8] = destination;
                execute_until(machine, BASE + decoder, 96);
                assert_eq!(
                    machine.cpu.regs[..4],
                    [BASE + bank.archive, DESTINATION, start, count]
                );
                assert_eq!(
                    machine.bus.read32(PARENT + 0x678),
                    BASE + proof.obj_slots[variant]
                );
                if entry == 0x037ddc {
                    assert_eq!(machine.bus.read8(VIEW + 0x13), 4 * variant as u8 | 3);
                    if node == 29 {
                        let expected = archive.selected_bytes(start, count).unwrap();
                        assert!(!expected.is_empty());
                        execute_until(machine, BASE + decoder + 4, 150_000);
                        assert_eq!(machine.cpu.regs[0], 0);
                        let at = (DESTINATION - 0x0200_0000) as usize;
                        assert_eq!(
                            &machine.bus.ewram[at..at + expected.len()],
                            expected.as_slice()
                        );
                    }
                }
            }
        }
    }
    for (slot, offset) in proof.obj_slots.into_iter().enumerate() {
        assert_eq!(
            machine.bus.read32(BASE + OBJ_DIRECTORY + 4 * slot as u32),
            BASE + offset
        );
    }
    for list in &proof.obj_lists {
        draw(machine, BASE + list.offset, 0, 0);
        assert_eq!(machine.bus.read32(0x0300_0020), list.entries);
    }
}

#[test]
#[ignore = "Exact EUR ROM, E658 source proof and e380/e381 datasets"]
fn european_world_map_labels_feed_original_readers() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let proof_bytes = fs::read(input("FFTA_WORLD_MAP_LABELS_PROOF")).unwrap();
    assert_eq!(digest(&proof_bytes), PROOF_SHA);
    let proof: Proof = serde_json::from_slice(&proof_bytes).unwrap();
    let mut previous = read_dataset(&input("FFTA_WORLD_MAP_LABELS_PREVIOUS")).unwrap();
    let mut dataset = read_dataset(&input("FFTA_WORLD_MAP_LABELS_DATASET")).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (380, 379)
    );
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (381, 380)
    );
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    assert!(
        previous
            .indexed_graphics
            .as_ref()
            .unwrap()
            .world_map_labels
            .is_none()
    );
    attach_dataset(&mut managed, &previous).unwrap();
    attach_dataset(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    previous.manifest.extractor_revision = 381;
    previous.manifest.decoder_revision = 380;
    assert!(attach_dataset(&mut managed, &previous).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    let graphics = dataset.indexed_graphics.clone().unwrap();
    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|item| item.id == INDEXED_GRAPHICS_ID)
        .unwrap();
    let declared: Vec<_> = resource
        .sources
        .iter()
        .filter(|span| {
            proof
                .new_ranges
                .iter()
                .any(|expected| expected.offset == span.offset)
        })
        .cloned()
        .collect();
    assert_eq!(declared, proof.new_ranges);
    let edited = dataset.indexed_graphics.as_mut().unwrap();
    edited.world_map_labels.as_mut().unwrap().obj_lists[0][0][0] ^= 1;
    let source = edited
        .source_spans()
        .unwrap()
        .into_iter()
        .find(|span| span.0 == proof.obj_lists[0].offset)
        .unwrap();
    resource
        .sources
        .iter_mut()
        .find(|span| span.offset == source.0)
        .unwrap()
        .sha256 = digest(&source.1);
    let bytes = encode_indexed_graphics(edited).unwrap();
    resource.sha256 = digest(&bytes);
    dataset.indexed_graphics_bytes = Some(bytes);
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    for span in &proof.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    let boundaries = check_boundaries(&mut raw.bus, &mut managed.bus, &proof.new_ranges);
    flip(&mut managed, RAW_ALIGNMENT, 2);
    assert_ne!(
        managed.bus.read16(BASE + RAW_ALIGNMENT),
        raw.bus.read16(BASE + RAW_ALIGNMENT)
    );
    flip(&mut managed, RAW_ALIGNMENT, 2);
    let before = owner.indexed_graphics_stats().unwrap();
    original_label_paths(&mut raw, &graphics, &proof);
    original_label_paths(&mut managed, &graphics, &proof);
    let after = owner.indexed_graphics_stats().unwrap();
    for span in &proof.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    same_state(&raw, &managed, 0x0800_1f6c);
    assert!(after.selector_bytes > before.selector_bytes);
    for i in 5..10 {
        assert!(after.additional_archive_bytes[i] > before.additional_archive_bytes[i]);
    }

    let result = serde_json::json!({
        "status": "PASS",
        "source_bytes": 628,
        "selector_cases": 300,
        "original_archive_decodes": 5,
        "variant_slots": proof.obj_slots.len(),
        "obj_lists": proof.obj_lists.len(),
        "drawn_objects": proof.obj_lists.iter().map(|list| list.entries).sum::<u32>(),
        "mixed_mirror_boundary_checks": boundaries,
        "predecessor_attached": true,
        "invalid_candidates_rejected": 2,
        "raw_alignment_preserved": true,
        "full_state_and_clock_equal": true,
    });
    fs::write(
        input("FFTA_WORLD_MAP_LABELS_RESULT"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
