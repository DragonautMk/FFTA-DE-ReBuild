//! Compare the original descriptor and animation readers for all E613 banks.
use super::{
    attach_dataset,
    direct_byte_copy_tests::{flip, input, same_state},
    indexed_reader_bank_tests::{execute_until, initialize},
    read_dataset, PHYSICAL_COMPACT_OBJ_REGION_ID,
};
use gba_core::{ffta_rom::digest, rom_data::SourceRange, Bus, Machine};
use serde::Deserialize;
use std::{collections::BTreeSet, fs, sync::Arc};

const BASE: u32 = 0x0800_0000;
const CONTEXT: u32 = 0x0201_0000;
const RETURN: u32 = BASE + 0x300;
const PLAN_SHA: &str = "2e4a1695e773b5e0897cb1506712a650c4adc321a34e60293155f75d83d98dbc";

#[derive(Deserialize)]
struct Bank {
    offset: u32,
    flags: u16,
    sequence_index: u32,
    sequences: u16,
    frames: usize,
}

#[derive(Deserialize)]
struct DirectProducer {
    root: u32,
    count: u32,
}

#[derive(Deserialize)]
struct IndexedProducer {
    roots: [u32; 3],
    counts: [u32; 3],
}

#[derive(Deserialize)]
struct Plan {
    banks: Vec<Bank>,
    direct_producers: Vec<DirectProducer>,
    indexed_producers: Vec<IndexedProducer>,
    new_ranges: Vec<SourceRange>,
}

fn half(rom: &[u8], at: u32) -> u16 {
    u16::from_le_bytes(rom[at as usize..at as usize + 2].try_into().unwrap())
}

fn word(rom: &[u8], at: u32) -> u32 {
    u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
}

fn control_flags(header: u16) -> u16 {
    (header & 0x0c) << 11
}

fn descriptors(plan: &Plan) -> BTreeSet<u32> {
    let direct = plan.direct_producers.iter().map(|p| (p.root, p.count));
    let indexed = plan
        .indexed_producers
        .iter()
        .flat_map(|p| p.roots.into_iter().zip(p.counts));
    direct
        .chain(indexed)
        .flat_map(|(base, count)| (base..base + 24 * count).step_by(24))
        .collect()
}

fn descriptor_reader(machine: &mut Machine, rom: &[u8], at: u32) {
    initialize(machine, 0x080d_89c6);
    machine.cpu.regs[6] = BASE + at;
    execute_until(machine, 0x080d_8a14, 80);
    let bank = word(rom, at);
    let offset = bank - BASE;
    let stack = machine.cpu.regs[13];
    assert_eq!(machine.bus.read32(stack + 40), bank + 4);
    assert_eq!(
        machine.bus.read32(stack + 44),
        bank + u32::from(half(rom, offset + 2))
    );
    assert_eq!(
        machine.cpu.regs[4],
        u32::from(half(rom, at + 18) | control_flags(half(rom, offset)))
    );
    assert_eq!(machine.cpu.regs[2], u32::from(half(rom, at + 4)));
    assert_eq!(machine.cpu.regs[0], u32::from(half(rom, at + 8)));
}

fn bank_readers(machine: &mut Machine, rom: &[u8], bank: &Bank) {
    let narrow = bank.flags & 8 != 0;
    let width = if narrow { 2 } else { 4 };
    let flags = control_flags(bank.flags);
    let lists = bank.offset + 4;
    machine.bus.write32(CONTEXT + 4, BASE + lists);
    machine.bus.write32(CONTEXT + 8, BASE + bank.sequence_index);
    machine.bus.write16(CONTEXT + 20, flags);
    let mut frames = 0;
    for selector in 0..bank.sequences {
        let index = bank.sequence_index + width * u32::from(selector);
        let sequence = bank.sequence_index
            + if narrow {
                u32::from(half(rom, index))
            } else {
                word(rom, index)
            };
        let count = half(rom, sequence);
        let result = if !narrow && count > 0 && half(rom, sequence + 4 * u32::from(count)) == 0x03ff
        {
            count - 1
        } else {
            count
        };
        initialize(machine, 0x0801_0848);
        machine.cpu.regs[0] = CONTEXT;
        machine.cpu.regs[1] = u32::from(selector);
        machine.cpu.regs[2] = u32::from(flags);
        execute_until(machine, RETURN, 64);
        assert_eq!(
            machine.cpu.regs[0],
            u32::from(result),
            "sequence at {sequence:#x}"
        );

        machine.bus.write16(CONTEXT + 12, selector);
        machine.bus.write16(CONTEXT + 18, count);
        for frame in 0..count {
            let at = sequence + 2 + width * u32::from(frame);
            let (list, duration) = if narrow {
                (u16::from(rom[at as usize]), u16::from(rom[at as usize + 1]))
            } else {
                (half(rom, at), half(rom, at + 2))
            };
            initialize(machine, 0x0801_0ca8);
            machine.cpu.regs[3] = u32::from(frame);
            machine.cpu.regs[4] = CONTEXT;
            machine.cpu.regs[5] = u32::from(flags);
            machine.cpu.regs[7] = u32::from(selector);
            execute_until(machine, 0x0801_0ce4, 40);
            assert_eq!(machine.cpu.regs[1], u32::from(duration), "frame at {at:#x}");

            machine.bus.write16(CONTEXT + 14, frame);
            initialize(machine, 0x0801_0e18);
            machine.cpu.regs[0] = CONTEXT;
            execute_until(machine, RETURN, 64);
            let list_at = lists + u32::from(half(rom, lists + 2 * u32::from(list)));
            assert_eq!(
                machine.bus.read32(CONTEXT + 24),
                BASE + list_at,
                "frame at {at:#x}"
            );
            frames += 1;
        }
    }
    assert_eq!(frames, bank.frames);
}

fn verify_binding(raw: &mut Machine, managed: &mut Machine, ranges: &[SourceRange]) -> (u32, u32) {
    let mut bytes = 0;
    let mut mixed = 0;
    for span in ranges {
        let end = span.offset + span.length;
        for offset in span.offset..end {
            let address = BASE + offset;
            assert_eq!(managed.bus.read8(address), raw.bus.read8(address));
            bytes += 1;
        }
        for boundary in [span.offset, end] {
            for offset in boundary - 3..=boundary + 3 {
                for base in [BASE, 0x0a00_0000, 0x0c00_0000] {
                    let address = base + offset;
                    assert_eq!(managed.bus.read8(address), raw.bus.read8(address));
                    assert_eq!(managed.bus.read16(address), raw.bus.read16(address));
                    assert_eq!(managed.bus.read32(address), raw.bus.read32(address));
                    mixed += 3;
                }
            }
        }
    }
    (bytes, mixed)
}

#[test]
#[ignore = "Exact EUR ROM, E613 source plan and managed e359/e360 datasets"]
fn european_physical_compact_obj_region_feeds_original_readers() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    let proof = fs::read(input("FFTA_PHYSICAL_COMPACT_PLAN")).unwrap();
    assert_eq!(digest(&proof), PLAN_SHA);
    let plan: Plan = serde_json::from_slice(&proof).unwrap();
    let rows = descriptors(&plan);
    assert_eq!((plan.banks.len(), rows.len()), (199, 548));
    assert_eq!(
        plan.banks
            .iter()
            .map(|b| b.sequences as usize)
            .sum::<usize>(),
        560
    );
    assert_eq!(plan.banks.iter().map(|b| b.frames).sum::<usize>(), 5045);

    let mut previous = read_dataset(&input("FFTA_PHYSICAL_COMPACT_PREDECESSOR")).unwrap();
    let mut dataset = read_dataset(&input("FFTA_PHYSICAL_COMPACT_DATASET")).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (359, 358)
    );
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (360, 359)
    );
    let mut raw = Machine::new(rom.clone());
    let mut managed = Machine::new(rom.clone());
    attach_dataset(&mut managed, &previous).unwrap();
    assert!(managed
        .bus
        .rom_resources()
        .unwrap()
        .physical_compact_obj_bus_bytes()
        .is_none());
    attach_dataset(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    (
        previous.manifest.extractor_revision,
        previous.manifest.decoder_revision,
    ) = (360, 359);
    assert!(attach_dataset(&mut managed, &previous).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|entry| entry.id == PHYSICAL_COMPACT_OBJ_REGION_ID)
        .unwrap();
    assert_eq!(resource.sources, plan.new_ranges);
    let packed = dataset.physical_compact_obj_region_bytes.as_mut().unwrap();
    packed[0] ^= 1;
    resource.sha256 = digest(packed);
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    for span in &plan.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    let (bytes, mixed) = verify_binding(&mut raw, &mut managed, &plan.new_ranges);
    assert_eq!((bytes, mixed), (93_964, 74_592));
    let before = owner.physical_compact_obj_bus_bytes().unwrap();
    for machine in [&mut raw, &mut managed] {
        for &row in &rows {
            descriptor_reader(machine, &rom, row);
        }
        for bank in &plan.banks {
            bank_readers(machine, &rom, bank);
        }
    }
    let original_read_bytes = owner.physical_compact_obj_bus_bytes().unwrap() - before;
    assert!(original_read_bytes > 30_000);
    for span in &plan.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    same_state(&raw, &managed, RETURN);
    fs::write(
        input("FFTA_PHYSICAL_COMPACT_RESULT"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "status": "PASS",
            "banks": plan.banks.len(),
            "descriptors": rows.len(),
            "sequences": 560,
            "frames": 5045,
            "guest_payload_bytes": bytes,
            "mixed_mirror_checks": mixed,
            "original_read_bytes": original_read_bytes,
            "e359_attachment": true,
            "invalid_candidates_rejected": 2,
            "full_state_and_clock_equal": true,
        }))
        .unwrap(),
    )
    .unwrap();
}
