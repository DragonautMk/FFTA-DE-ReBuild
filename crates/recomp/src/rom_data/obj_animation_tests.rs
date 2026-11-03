//! EUR UI bank source/JSON and original task constructor/scheduler-body oracle.
use super::*;
use gba_core::{
    rom_data::{
        obj_animations::{Banks, Catalog},
        obj_lists::List,
        ObjDimensions,
    },
    Bus, Machine,
};
const TASK: u32 = 0x02028010;
const STACK: u32 = 0x03007c00;
const RETURN: u32 = 0x08000300;
const BUFFER: u32 = 0x03000030;

pub(super) fn invoke(m: &mut Machine, entry: u32, args: [u32; 4], stack: &[u32]) -> usize {
    m.cpu.regs[..4].copy_from_slice(&args);
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.regs[15] = entry;
    m.cpu.cpsr = 0xbf;
    for (i, &word) in stack.iter().enumerate() {
        m.bus.write32(STACK + i as u32 * 4, word);
    }
    for steps in 1..=20_000 {
        m.bus.frame_ready = false;
        m.step();
        if m.cpu.regs[15] == RETURN {
            assert_eq!(m.cpu.regs[13], STACK);
            return steps;
        }
    }
    panic!(
        "OBJ animation constructor did not return at{:08x}",
        m.cpu.regs[15]
    );
}

pub(super) fn tick(m: &mut Machine) -> usize {
    // The exact original per-object body, entered after the scheduler's mode
    // dispatch and stopped at its next-object edge. Do not invent executable
    // code or pretend the scene-specific tail was covered by this unit.
    m.cpu.regs[5] = TASK;
    m.cpu.regs[13] = STACK;
    m.cpu.regs[15] = 0x0813f814;
    m.cpu.cpsr = 0xbf;
    m.bus.write8(0x03000028, 0);
    let mode1 = m.bus.read8(TASK + 12) == 1;
    let count = if mode1 { 0x03002c50 } else { 0x03000020 };
    let buffer = if mode1 { 0x03002c58 } else { BUFFER };
    m.bus.write32(count, 0);
    for i in 0..1024 {
        m.bus.write8(buffer + i, 0x5a);
    }
    for steps in 1..=20_000 {
        m.bus.frame_ready = false;
        m.step();
        if m.cpu.regs[15] == 0x0813f9a6 {
            assert_eq!(m.cpu.regs[13], STACK);
            return steps;
        }
    }
    panic!("original OBJ scheduler body did not reach next-object edge");
}

fn output(list: &List, dimensions: &ObjDimensions, mode: u8) -> (Vec<u8>, u32) {
    let mut bytes = vec![0x5a; 1024];
    let mut next = 0usize;
    for object in &list.objects {
        let dim = dimensions.get(object.shape, object.size).unwrap();
        let x = (100 + i32::from(object.x)) as i16 as i32;
        let y = (70 + i32::from(object.y)) as i16 as i32;
        if x < 0 && x + i32::from(dim.width) < 0
            || x > 239 && x + i32::from(dim.width) > 239
            || y < 0 && y + i32::from(dim.height) < 0
            || y > 159 && y + i32::from(dim.height) > 159
        {
            continue;
        }
        if next >= 128 {
            break;
        }
        let [a0, a1, _] = object.attributes();
        let attrs = [
            (a0 & 0xff00) | (y as u16 & 255),
            (a1 & !511) | (x as u16 & 511),
            (object.tile + 7) % 1024
                | (u16::from((object.priority + 2) % 4) << 10)
                | (u16::from((object.palette + if mode == 0 { 3 } else { 0 }) % 16) << 12),
        ];
        for (i, attr) in attrs.into_iter().enumerate() {
            bytes[next * 8 + i * 2..next * 8 + i * 2 + 2].copy_from_slice(&attr.to_le_bytes());
        }
        next += 1;
    }
    (bytes, next as u32)
}

#[test]
#[ignore = "exact EUR ROM/new root/static inventory; reduced Wine, Linux/USA pending"]
fn european_ui_animation_banks_match_sources_original_constructors_and_all_frame_ticks() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_OBJ_ANIMATION_ROOT").unwrap());
    assert!(!root.exists(), "preserve prior evidence");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    assert_eq!(catalog.decoded.banks.len(), 10);
    assert_eq!(gba_core::rom_data::obj_animations::REFERENCES.len(), 30);
    assert_eq!(catalog.sources.len(), 282);
    let bytes = serde_json::to_vec_pretty(&catalog.decoded).unwrap();
    let decoded: Banks = serde_json::from_slice(&bytes).unwrap();
    decoded.verify_source(&rom).unwrap();
    assert_eq!(serde_json::to_vec_pretty(&decoded).unwrap(), bytes);
    let mut forged = decoded.clone();
    forged.banks[0].sequences[0].frames[0].duration ^= 1;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());
    forged = decoded.clone();
    forged.banks.pop();
    assert!(forged.verify_source(&rom).is_err());
    let mut unknown: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    unknown["banks"][0]["sequences"][0]["frames"][0]["unsupported"] = serde_json::json!(1);
    assert!(serde_json::from_value::<Banks>(unknown).is_err());
    assert!(serde_json::from_slice::<Banks>(&bytes[..bytes.len() - 1]).is_err());
    let inventory: serde_json::Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_OBJ_ANIMATION_INVENTORY").unwrap()).unwrap(),
    )
    .unwrap();
    let dimensions = ObjDimensions::decode(&rom[0x14ced8..0x14cf08]).unwrap();
    let mut m = crate::make_machine(rom.clone(), None);
    m.bus.ffta_sprites.enabled = false;
    let mut constructors = 0;
    let mut ticks = 0;
    let mut instructions = 0;
    let mut lists = 0;
    let mut objects = 0;
    let mut sequences = 0;
    let mut frames = 0;
    let mut empty = 0;
    for (bank_id, bank) in decoded.banks.iter().enumerate() {
        assert_eq!(inventory[bank_id]["objects"], bank.object_root);
        assert_eq!(inventory[bank_id]["frames"], bank.sequence_root);
        for (root, offsets) in [
            (bank.object_root, &bank.object_offsets),
            (bank.sequence_root, &bank.sequence_offsets),
        ] {
            let source: Vec<_> = offsets.iter().flat_map(|v| v.to_le_bytes()).collect();
            assert_eq!(source, rom[root as usize..root as usize + source.len()]);
        }
        for list in &bank.lists {
            let source = list.source_bytes().unwrap();
            assert_eq!(
                source,
                rom[list.source_offset as usize..list.source_offset as usize + source.len()]
            );
            lists += 1;
            objects += list.objects.len();
        }
        for (sequence_id, sequence) in bank.sequences.iter().enumerate() {
            let source = sequence.source_bytes().unwrap();
            assert_eq!(
                source,
                rom[sequence.source_offset as usize
                    ..sequence.source_offset as usize + source.len()]
            );
            let frame_values: Vec<_> = sequence
                .frames
                .iter()
                .map(|f| [f.object_list, f.duration])
                .collect();
            assert_eq!(
                inventory[bank_id]["frame"]["records"][sequence_id]["frames"],
                serde_json::json!(frame_values)
            );
            sequences += 1;
            frames += sequence.frames.len();
            if sequence.frames.is_empty() {
                empty += 1;
            }
            // All declared sequences get original mode0 initialization and a
            // complete loop. Sample mode1 and terminal-frame hold per bank.
            let modes: &[u8] = if sequence_id == 0 { &[0, 1, 2] } else { &[0] };
            for &mode in modes {
                for at in 0..60 {
                    m.bus.write8(TASK - 16 + at, 0x5a);
                }
                for at in 0..28 {
                    m.bus.write8(TASK + at, 0);
                }
                let repeat = u32::from(mode != 2);
                let entry = if mode == 1 { 0x0804e558 } else { 0x0804e4e4 };
                instructions += invoke(
                    &mut m,
                    entry,
                    [TASK, 0, sequence_id as u32, 100],
                    &[
                        70,
                        repeat,
                        2,
                        0x08000000 + bank.object_root,
                        0x08000000 + bank.sequence_root,
                        7,
                        3,
                    ],
                );
                constructors += 1;
                let mut expected = vec![0; 28];
                expected[..4]
                    .copy_from_slice(&(0x08000000 + sequence.source_offset + 2).to_le_bytes());
                expected[10..12].copy_from_slice(&(repeat as u16).to_le_bytes());
                expected[12] = u8::from(mode == 1);
                expected[13] = 1;
                expected[14] = sequence.frames.len() as u8;
                expected[16] = 100;
                expected[17] = 70;
                expected[18] = if mode == 1 { 0 } else { 3 };
                expected[19] = 2;
                expected[22..24].copy_from_slice(&7u16.to_le_bytes());
                expected[24..28].copy_from_slice(&(0x08000000 + bank.object_root).to_le_bytes());
                assert!(
                    (0..28).all(|at| m.bus.read8(TASK + at) == expected[at as usize]),
                    "constructor bank{bank_id} seq{sequence_id} mode{mode}"
                );
                if sequence.frames.is_empty() {
                    continue;
                }
                let mut index = 0;
                let mut elapsed = 0u8;
                let cycle: usize = sequence
                    .frames
                    .iter()
                    .map(|f| usize::from(f.duration.max(1)))
                    .sum();
                for _ in 0..cycle + 1 {
                    let frame = &sequence.frames[index];
                    let (oam, count) = output(
                        &bank.lists[frame.object_list as usize],
                        &dimensions,
                        u8::from(mode == 1),
                    );
                    instructions += tick(&mut m);
                    ticks += 1;
                    let buffer = if mode == 1 { 0x03002c58 } else { BUFFER };
                    assert!(
                        (0..1024).all(|at| m.bus.read8(buffer + at) == oam[at as usize]),
                        "OAM/matrices bank{bank_id} seq{sequence_id} frame{index} mode{mode}"
                    );
                    assert_eq!(
                        m.bus
                            .read32(if mode == 1 { 0x03002c50 } else { 0x03000020 }),
                        count
                    );
                    elapsed = elapsed.wrapping_add(1);
                    if u16::from(elapsed) >= frame.duration {
                        elapsed = 0;
                        index = if repeat == 1 {
                            (index + 1) % sequence.frames.len()
                        } else {
                            (index + 1).min(sequence.frames.len() - 1)
                        };
                    }
                    expected[8..10].copy_from_slice(&(index as u16).to_le_bytes());
                    expected[15] = elapsed;
                    assert!(
                        (0..28).all(|at| m.bus.read8(TASK + at) == expected[at as usize]),
                        "frame task bank{bank_id} seq{sequence_id} mode{mode}"
                    );
                    assert!((0..16).all(|at| m.bus.read8(TASK - 16 + at) == 0x5a
                        && m.bus.read8(TASK + 28 + at) == 0x5a));
                }
            }
        }
    }
    assert_eq!(
        (lists, objects, sequences, frames, empty),
        (167, 284, 95, 184, 3)
    );
    for source in &catalog.sources {
        assert_eq!(
            gba_core::ffta_rom::digest(
                &rom[source.offset as usize..(source.offset + source.length) as usize]
            ),
            source.sha256
        );
    }
    assert_eq!(
        gba_core::ffta_rom::digest(&m.bus.rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("obj-animation-banks.json"), &bytes).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({"region":"europe","banks":10,"literal_pairs":30,
        "lists":lists,"objects":objects,"sequences":sequences,"frames":frames,"empty_sequences":empty,
        "original_constructor_calls":constructors,"original_scheduler_body_ticks":ticks,"original_instructions":instructions,
        "task_fields_indices_durations_loop_hold_oam_matrices_guards":true,"strict_json_source_and_independent_inventory":true,
        "sources":catalog.sources,"json_sha256":gba_core::ffta_rom::digest(&bytes),"runtime":"offline-only",
        "empty_sequence_scheduling":"caller precondition pending; constructors/source only",
        "whole_scheduler_and_ordinary_scene":"pending","linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
    eprintln!("OBJ_ANIMATION_EUR banks=10 lists=167 objects=284 sequences=95 frames=184 empty=3 constructors={constructors} ticks={ticks} instructions={instructions} source/JSON/original-tasks/OAM/guards=pass");
}
