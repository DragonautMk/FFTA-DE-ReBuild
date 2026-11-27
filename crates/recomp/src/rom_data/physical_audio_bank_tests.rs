//! Check the physical E611 bank projection through original readers and MP2K.
use super::{
    attach_dataset,
    direct_byte_copy_tests::{flip, input, same_state},
    indexed_reader_bank_tests::{execute_until, initialize},
    read_dataset, PHYSICAL_AUDIO_BANKS_ID,
};
use gba_core::{ffta_rom::digest, mp2k::MemView, rom_data::SourceRange, Bus, Machine};
use serde::Deserialize;
use std::{collections::BTreeMap, fs, sync::Arc};

const BASE: u32 = 0x0800_0000;
const PLAYER: u32 = 0x0201_0000;
const TRACK: u32 = 0x0201_0100;
const SELECTOR: u32 = 0x0201_0200;
const RETURN: u32 = BASE + 0x300;
const PLAN_SHA: &str = "5d09d6aee19838ccde9c88e0777bc2a7f0d7d50933767e0b4be4d1d817524f4f";

#[derive(Deserialize)]
struct Bank {
    offset: u32,
    records: usize,
}

#[derive(Deserialize)]
struct Wave {
    offset: u32,
    size: u32,
    interpolated: bool,
}

#[derive(Deserialize)]
struct Plan {
    banks: Vec<Bank>,
    pcm_waves: Vec<Wave>,
    cgb_waves: BTreeMap<u32, Vec<u32>>,
    new_ranges: Vec<SourceRange>,
}

fn verify_programs(machine: &mut Machine, rom: &[u8], banks: &[Bank]) -> usize {
    let mut programs = 0;
    for bank in banks {
        machine.bus.write32(PLAYER + 0x30, BASE + bank.offset);
        for selector in 0..bank.records {
            machine.bus.write32(TRACK + 0x40, SELECTOR);
            machine.bus.write8(SELECTOR, selector as u8);
            initialize(machine, 0x0814_83e4);
            machine.cpu.regs[0] = PLAYER;
            machine.cpu.regs[1] = TRACK;
            execute_until(machine, RETURN, 100);
            let source = bank.offset as usize + 12 * selector;
            let track = (TRACK - 0x0200_0000) as usize;
            assert_eq!(
                &machine.bus.ewram[track + 0x24..track + 0x30],
                &rom[source..source + 12],
                "program {selector} from bank {:#x}",
                bank.offset,
            );
            assert_eq!(machine.bus.read32(TRACK + 0x40), SELECTOR + 1);
            programs += 1;
        }
    }
    programs
}

fn verify_new_pcm(machine: &mut Machine, rom: &[u8], wave: &Wave) {
    let address = BASE + wave.offset;
    initialize(machine, 0x0814_8006);
    machine.cpu.regs[3] = address;
    machine.cpu.regs[4] = PLAYER;
    machine.cpu.regs[6] = 3;
    execute_until(machine, 0x0814_8018, 20);
    assert_eq!(machine.bus.read32(PLAYER + 0x28), address + 16);
    assert_eq!(machine.bus.read32(PLAYER + 0x18), wave.size);
    assert_eq!(
        machine.cpu.regs[2],
        u32::from(rom[wave.offset as usize + 3])
    );

    let last_sample = wave.offset + 16 + wave.size - 1;
    assert!(wave.interpolated);
    initialize(machine, 0x0814_81e0);
    machine.cpu.cpsr = 0x9f;
    machine.cpu.regs[3] = BASE + last_sample;
    execute_until(machine, 0x0814_81e8, 2 + 1);
    for (offset, byte) in rom[last_sample as usize..][..2].iter().enumerate() {
        assert_eq!(machine.cpu.regs[offset], *byte as i8 as i32 as u32);
    }
    assert_eq!(machine.cpu.regs[3], BASE + last_sample + 1);
}

fn verify_cgb(machine: &mut Machine, rom: &[u8], wave: u32) {
    machine.bus.io[0x90..0xa0].fill(0);
    machine.bus.write32(PLAYER + 0x24, BASE + wave);
    initialize(machine, 0x0814_97b0);
    machine.cpu.regs[4] = PLAYER;
    execute_until(machine, 0x0814_97cc, 32);
    assert_eq!(
        &machine.bus.io[0x90..0xa0],
        &rom[wave as usize..wave as usize + 16]
    );
    assert_eq!(machine.bus.read32(PLAYER + 0x28), BASE + wave);
}

fn host_view(machine: &Machine) -> MemView<'_> {
    MemView {
        rom: &machine.bus.rom,
        ewram: &machine.bus.ewram,
        iwram: &machine.bus.iwram,
        resources: machine.bus.rom_resources().map(Arc::as_ref),
    }
}

fn verify_views(raw: &mut Machine, managed: &mut Machine, spans: &[SourceRange]) -> (u32, u32) {
    let mut bytes = 0;
    let mut mixed = 0;
    for span in spans {
        let end = span.offset + span.length;
        for offset in span.offset..end {
            let address = BASE + offset;
            let expected = raw.bus.read8(address);
            assert_eq!(managed.bus.read8(address), expected);
            assert_eq!(host_view(managed).u8(address), Some(expected));
            bytes += 1;
        }
        for boundary in [span.offset, end] {
            for offset in boundary - 3..=boundary + 3 {
                for base in [BASE, 0x0a00_0000, 0x0c00_0000] {
                    let address = base + offset;
                    assert_eq!(managed.bus.read16(address), raw.bus.read16(address));
                    assert_eq!(managed.bus.read32(address), raw.bus.read32(address));
                    assert_eq!(
                        host_view(managed).bytes::<4>(address),
                        host_view(raw).bytes::<4>(address)
                    );
                    mixed += 3;
                }
            }
        }
    }
    (bytes, mixed)
}

#[test]
#[ignore = "Exact EUR ROM, E611 source plan and managed e358/e359 datasets"]
fn european_physical_audio_banks_feed_original_and_host_readers() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    let proof = fs::read(input("FFTA_PHYSICAL_AUDIO_PLAN")).unwrap();
    assert_eq!(digest(&proof), PLAN_SHA);
    let plan: Plan = serde_json::from_slice(&proof).unwrap();
    assert_eq!(
        (plan.banks.len(), plan.pcm_waves.len(), plan.cgb_waves.len()),
        (4, 285, 8)
    );
    let mut previous = read_dataset(&input("FFTA_PHYSICAL_AUDIO_PREDECESSOR")).unwrap();
    let mut dataset = read_dataset(&input("FFTA_PHYSICAL_AUDIO_DATASET")).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (358, 357)
    );
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (359, 358)
    );
    let mut raw = Machine::new(rom.clone());
    let mut managed = Machine::new(rom.clone());
    attach_dataset(&mut managed, &previous).unwrap();
    assert!(managed
        .bus
        .rom_resources()
        .unwrap()
        .physical_audio_bank_bus_bytes()
        .is_none());
    attach_dataset(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    (
        previous.manifest.extractor_revision,
        previous.manifest.decoder_revision,
    ) = (359, 358);
    assert!(attach_dataset(&mut managed, &previous).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|entry| entry.id == PHYSICAL_AUDIO_BANKS_ID)
        .unwrap();
    assert_eq!(resource.sources, plan.new_ranges);
    let packed = dataset.physical_audio_banks_bytes.as_mut().unwrap();
    packed[0] ^= 1;
    resource.sha256 = digest(packed);
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    for span in &plan.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    let (bytes, mixed) = verify_views(&mut raw, &mut managed, &plan.new_ranges);
    assert_eq!((bytes, mixed), (57_131, 5_922));
    let pcm_sources: Vec<_> = plan
        .pcm_waves
        .iter()
        .filter(|wave| {
            plan.new_ranges
                .iter()
                .any(|span| span.offset == wave.offset && span.role == "direct-pcm-waves")
        })
        .collect();
    assert_eq!(pcm_sources.len(), 8);

    let before = owner.physical_audio_bank_bus_bytes().unwrap();
    for machine in [&mut raw, &mut managed] {
        assert_eq!(verify_programs(machine, &rom, &plan.banks), 499);
        for wave in &pcm_sources {
            verify_new_pcm(machine, &rom, wave);
        }
        for &wave in plan.cgb_waves.keys() {
            verify_cgb(machine, &rom, wave);
        }
    }
    let original_read_bytes = owner.physical_audio_bank_bus_bytes().unwrap() - before;
    assert_eq!(original_read_bytes, 992);
    for span in &plan.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    same_state(&raw, &managed, 0x0814_97cc);

    fs::write(
        input("FFTA_PHYSICAL_AUDIO_RESULT"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "status": "PASS",
            "programs": 499,
            "pcm_headers_and_endpoint_pairs": pcm_sources.len(),
            "cgb_copies": plan.cgb_waves.len(),
            "guest_and_host_payload_bytes": bytes,
            "mixed_mirror_checks": mixed,
            "original_read_bytes": original_read_bytes,
            "e358_attachment": true,
            "invalid_candidates_rejected": 2,
            "full_state_and_clock_equal": true,
        }))
        .unwrap(),
    )
    .unwrap();
}
