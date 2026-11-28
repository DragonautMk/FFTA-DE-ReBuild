//! Verify all five language cases in the original E624 builders.
use super::{
    attach_dataset,
    counted_effect_table_tests::literal_loads,
    direct_byte_copy_tests::{flip, input, same_state},
    indexed_reader_bank_tests::{execute_until, initialize},
    localized_obj_animation_bank_tests::draw,
    read_dataset,
    screen_update_table_tests::check_boundaries,
    LOCALIZED_LAYOUT_TABLES_ID,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::SourceRange,
    Bus, Machine,
};
use serde::Deserialize;
use std::{collections::BTreeMap, fs, sync::Arc};

const ROM: u32 = 0x0800_0000;
const EWRAM: u32 = 0x0200_0000;
const LANGUAGE: u32 = 0x0200_1f6e;
const TASK: u32 = 0x0201_8000;
const TRANSITION_STATE: u32 = 0x0201_9000;
const PARAMETER: u32 = 0x0200_f498;
const ROW_COUNT: u32 = 0x0200_f49c;
const ORIGIN_X: u32 = 0x0200_f4a0;
const ORIGIN_Y: u32 = 0x0200_f4a4;
const SHADOW_OAM: u32 = 0x0300_0030;
const PROOF_SHA: &str = "6fe639b8b18027de64d92bd6570cbc4f161800f427d89ce6736654676496727f";

#[derive(Deserialize)]
struct Selector {
    entry: u32,
}

#[derive(Deserialize)]
struct Layout {
    selectors: [Selector; 4],
    counts: [u32; 5],
    roots: [u32; 5],
}

#[derive(Deserialize)]
struct ObjectList {
    offset: u32,
    count: u32,
}

#[derive(Deserialize)]
struct Proof {
    layouts: [Layout; 4],
    record_count: u32,
    transition_count: u32,
    reader_sites: BTreeMap<u32, u32>,
    object_lists: [ObjectList; 3],
    new_ranges: Vec<SourceRange>,
}

fn half(rom: &[u8], at: u32) -> u16 {
    u16::from_le_bytes(rom[at as usize..][..2].try_into().unwrap())
}

fn word(rom: &[u8], at: u32) -> u32 {
    u32::from_le_bytes(rom[at as usize..][..4].try_into().unwrap())
}

fn layout_builders(machine: &mut Machine, rom: &[u8], proof: &Proof) {
    let builders = [
        (0x093960, 0x093974, 0x093b44, 1, 7, 4),
        (0x093ca4, 0x093cb8, 0x093e88, 3, 6, 7),
        (0x093fcc, 0x093fde, 0x0941ac, 0, 7, 4),
        (0x094310, 0x094324, 0x0944ec, 2, 7, 4),
    ];
    let mut records = 0;
    for (layout, (parameter_read, parameter_end, build_end, field, task_reg, ewram_reg)) in
        proof.layouts.iter().zip(builders)
    {
        for (language, (&base, &count)) in layout.roots.iter().zip(&layout.counts).enumerate() {
            machine.bus.write8(LANGUAGE, language as u8);
            initialize(machine, ROM + parameter_read);
            machine.cpu.regs[3] = PARAMETER;
            execute_until(machine, ROM + parameter_end, 16);
            assert_eq!(
                machine.bus.read8(PARAMETER),
                rom[0x398d90 + language * 4 + field]
            );

            // Enter before loading the stored language byte. The normal loop
            // initializes all selected rows and saves the fourth table choice.
            initialize(machine, ROM + layout.selectors[0].entry - 6);
            machine.cpu.regs[task_reg] = TASK;
            machine.cpu.regs[ewram_reg] = EWRAM;
            let last = TASK + 12 + 16 * count;
            machine.bus.write32(last, 0x5a5a_5a5a);
            execute_until(machine, ROM + build_end, 4000);
            assert_eq!(machine.bus.read32(last), 0x5a5a_5a5a);
            assert_eq!(machine.bus.read32(TASK + 8), ROM + base);
            for row in 0..count {
                let source = base + 12 * row;
                let destination = TASK + 16 * row;
                assert_eq!(machine.bus.read32(destination + 12), word(rom, source));
                assert_eq!(
                    machine.bus.read16(destination + 20),
                    half(rom, source + 4).wrapping_add(480)
                );
                assert_eq!(
                    machine.bus.read8(destination + 23),
                    half(rom, source + 6) as u8
                );
            }

            machine.bus.write8(ROW_COUNT, count as u8);
            machine.bus.write8(ORIGIN_X, 3);
            machine.bus.write8(ORIGIN_Y, 4);
            initialize(machine, 0x0809_3656);
            machine.cpu.regs[7] = TASK;
            execute_until(machine, 0x0809_3696, 700);
            for row in 0..count {
                assert_eq!(
                    machine.bus.read16(TASK + 16 + 16 * row),
                    half(rom, base + 12 * row + 8).wrapping_add(24)
                );
                assert_eq!(machine.bus.read16(TASK + 18 + 16 * row), 32);
            }
            records += count;
        }
    }
    assert_eq!(records, proof.record_count);
}

fn allocations_and_list_selection(machine: &mut Machine, rom: &[u8], proof: &Proof) {
    for language in 0..5 {
        machine.bus.write8(LANGUAGE, language as u8);
        initialize(machine, 0x0809_2746);
        execute_until(machine, 0x0809_2758, 16);
        let bytes = word(rom, 0x3980c0 + language * 4);
        assert_eq!(machine.cpu.regs[0], bytes);

        initialize(machine, 0x0809_2786);
        machine.cpu.regs[5] = LANGUAGE;
        machine.cpu.regs[8] = ROM + 0x3980c0;
        execute_until(machine, 0x0809_2792, 10);
        assert_eq!(machine.cpu.regs[3], bytes / 4);
    }
    for (kind, list) in [4, 5].into_iter().zip(&proof.object_lists[..2]) {
        initialize(machine, 0x0809_2700);
        machine.cpu.regs[7] = TASK;
        machine.cpu.regs[8] = TRANSITION_STATE;
        machine.bus.write8(TRANSITION_STATE + 34, kind);
        execute_until(machine, 0x0809_271a, 16);
        assert_eq!(machine.bus.read32(TASK + 732), ROM + list.offset);
    }
}

fn transitions(machine: &mut Machine, rom: &[u8], count: u32) {
    for index in 0..count {
        initialize(machine, 0x0809_5738);
        machine.cpu.regs[5] = TASK;
        machine.cpu.regs[6] = TRANSITION_STATE;
        machine.bus.write16(TRANSITION_STATE, 9);
        machine.bus.write8(TASK + 94, index as u8);
        machine.bus.write8(TASK + 95, 3);
        execute_until(machine, 0x0809_5794, 64);
        let source = (0x398da4 + index * 4) as usize;
        assert_eq!(machine.bus.read8(0x0300_098c), rom[source]);
        assert_eq!(machine.bus.read8(0x0300_098d), rom[source + 1]);
        assert_eq!(machine.bus.read8(TASK + 94), (index + 1) as u8);
        assert_eq!(machine.bus.read8(TASK + 95), 0);
        assert_eq!(
            machine.bus.read16(TRANSITION_STATE),
            if index == count - 1 { 10 } else { 9 }
        );
    }
}

fn object_lists(raw: &mut Machine, managed: &mut Machine, rom: &[u8], proof: &Proof) -> u32 {
    let mut drawn = 0;
    for list in &proof.object_lists {
        let x = ((half(rom, list.offset + 4) << 7) as i16) >> 7;
        let y = rom[list.offset as usize + 2] as i8;
        draw(raw, ROM + list.offset, x, y);
        draw(managed, ROM + list.offset, x, y);
        assert_eq!(raw.bus.read32(0x0300_0020), list.count);
        assert_eq!(managed.bus.read32(0x0300_0020), list.count);
        for at in (SHADOW_OAM..SHADOW_OAM + 8 * list.count).step_by(4) {
            assert_eq!(raw.bus.read32(at), managed.bus.read32(at));
        }
        drawn += list.count;
    }
    drawn
}

#[test]
#[ignore = "Exact EUR ROM, E624 source proof and e364/e365 datasets"]
fn european_localized_layout_tables_feed_original_builders() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let proof_bytes = fs::read(input("FFTA_LOCALIZED_LAYOUT_PROOF")).unwrap();
    assert_eq!(digest(&proof_bytes), PROOF_SHA);
    let proof: Proof = serde_json::from_slice(&proof_bytes).unwrap();
    assert_eq!(proof.record_count, 267);
    assert_eq!(proof.transition_count, 17);
    assert_eq!(proof.reader_sites.len(), 88);

    let mut previous = read_dataset(&input("FFTA_LOCALIZED_LAYOUT_PREVIOUS")).unwrap();
    let mut dataset = read_dataset(&input("FFTA_LOCALIZED_LAYOUT_DATASET")).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (364, 363)
    );
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (365, 364)
    );

    let mut raw = Machine::new(rom.clone());
    let mut managed = Machine::new(rom.clone());
    attach_dataset(&mut managed, &previous).unwrap();
    assert!(managed
        .bus
        .rom_resources()
        .unwrap()
        .localized_layout_table_bus_bytes()
        .is_none());
    attach_dataset(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    previous.manifest.extractor_revision = 365;
    previous.manifest.decoder_revision = 364;
    assert!(attach_dataset(&mut managed, &previous).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|item| item.id == LOCALIZED_LAYOUT_TABLES_ID)
        .unwrap();
    assert_eq!(resource.sources, proof.new_ranges);
    let packed = dataset.localized_layout_table_bytes.as_mut().unwrap();
    assert_eq!(packed.len(), 3718);
    packed[0] ^= 1;
    resource.sha256 = digest(packed);
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    for span in &proof.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    let boundaries = check_boundaries(&mut raw.bus, &mut managed.bus, &proof.new_ranges);
    let before = owner.localized_layout_table_bus_bytes().unwrap();
    for machine in [&mut raw, &mut managed] {
        literal_loads(machine, &rom, &proof.reader_sites);
        layout_builders(machine, &rom, &proof);
        allocations_and_list_selection(machine, &rom, &proof);
        transitions(machine, &rom, proof.transition_count);
    }
    let objects = object_lists(&mut raw, &mut managed, &rom, &proof);
    assert_eq!(objects, 8);
    let managed_bytes = owner.localized_layout_table_bus_bytes().unwrap() - before;
    assert!(managed_bytes >= 10 * 267 + 4 * 88);
    for span in &proof.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    same_state(&raw, &managed, raw.cpu.regs[15]);

    let result = serde_json::json!({
        "status": "PASS",
        "localized_layouts": 20,
        "layout_records": proof.record_count,
        "transition_records": proof.transition_count,
        "literal_readers": proof.reader_sites.len(),
        "objects_drawn": objects,
        "source_bytes": 3718,
        "mixed_mirror_boundary_checks": boundaries,
        "original_read_bytes": managed_bytes,
        "predecessor_attached": true,
        "invalid_candidates_rejected": 2,
        "full_state_and_clock_equal": true,
    });
    fs::write(
        input("FFTA_LOCALIZED_LAYOUT_RESULT"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
