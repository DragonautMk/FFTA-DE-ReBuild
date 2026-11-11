use super::*;
use crate::{Bus, Machine};

const TASK: u32 = 0x02010000;
const SP: u32 = 0x03007c00;
const RETURN: u32 = 0x08000300;

fn run_initializer(rom: &[u8], descriptor: &Descriptor, bank: &obj_animations::Bank) {
    let mut machine = Machine::new(rom.to_vec());
    machine.bus.ffta_sprites.enabled = false;
    machine.bus.ewram.fill(0xa5);
    machine.bus.iwram.fill(0xc3);
    machine.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    machine.cpu.regs[13] = SP;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.cpsr = 0xbf;
    let initial_time = u32::from_le_bytes(descriptor.opaque[4..8].try_into().unwrap());
    let mode = descriptor.opaque[14];
    machine.cpu.regs[..4].copy_from_slice(&[
        TASK,
        0x08000000 + bank.object_root,
        0x08000000 + bank.sequence_root,
        initial_time,
    ]);
    machine.bus.write32(SP, u32::from(mode));
    machine.cpu.regs[15] = 0x08142590;
    let regs = machine.cpu.regs;
    let mut expected = machine.bus.ewram.clone();
    let at = (TASK - 0x02000000) as usize;
    let write16 = |memory: &mut [u8], offset: usize, value: u16| {
        memory[at + offset..at + offset + 2].copy_from_slice(&value.to_le_bytes());
    };
    let write32 = |memory: &mut [u8], offset: usize, value: u32| {
        memory[at + offset..at + offset + 4].copy_from_slice(&value.to_le_bytes());
    };
    write32(&mut expected, 0, 0x08000000 + bank.object_root);
    write32(&mut expected, 4, 0x08000000 + bank.sequence_root);
    write16(&mut expected, 0x58, bank.sequences.len() as u16);
    for (index, sequence) in bank.sequences.iter().enumerate() {
        write16(&mut expected, 8 + index * 8, sequence.frames.len() as u16);
        write32(
            &mut expected,
            12 + index * 8,
            0x08000000 + sequence.source_offset + 2,
        );
        write16(&mut expected, 0x5a + index * 2, 0);
        write16(&mut expected, 0x6e + index * 2, 0);
        expected[at + 0x82 + index] = 0;
        let mut remaining = initial_time as i32;
        let mut selected = 0;
        for (phase, frame) in sequence.frames.iter().enumerate() {
            remaining -= i32::from(frame.duration);
            if remaining < 0 {
                write16(&mut expected, 0x5a + index * 2, (-remaining) as u16);
                selected = phase;
                break;
            }
            selected = (phase + 1).min(sequence.frames.len() - 1);
        }
        write16(&mut expected, 0x6e + index * 2, selected as u16);
    }
    expected[at + 0x8c] = mode;
    for _ in 0..100_000 {
        if machine.cpu.regs[15] == RETURN {
            assert_eq!(&machine.cpu.regs[4..12], &regs[4..12]);
            assert_eq!(
                machine.cpu.regs[12], TASK,
                "original initializer uses r12 as scratch"
            );
            assert_eq!(&machine.cpu.regs[13..15], &regs[13..15]);
            assert_eq!(
                machine.bus.ewram, expected,
                "initializer task and RAM guards"
            );
            return;
        }
        assert!(machine.cpu.thumb());
        machine.bus.frame_ready = false;
        machine.step();
    }
    panic!("layered OBJ initializer did not return");
}

#[test]
#[ignore = "private EUR source and original-code proof; Wine only, Linux/USA pending"]
fn european_layered_obj_model_and_original_initializers() {
    let rom = std::fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let model = &catalog.decoded;
    model.verify_source(&rom).unwrap();
    let spans = model.source_spans().unwrap();
    assert_eq!(
        (spans.len(), spans.iter().map(|s| s.1.len()).sum::<usize>()),
        (656, 21058)
    );
    assert_eq!(catalog.sources.len(), spans.len());
    for ((at, raw, role), source) in spans.iter().zip(&catalog.sources) {
        assert_eq!(
            (source.offset, source.length, source.role.as_str()),
            (*at, raw.len() as u32, role.as_str())
        );
        assert_eq!(source.sha256, ffta_rom::digest(raw));
        assert_eq!(raw, &rom[*at as usize..*at as usize + raw.len()]);
    }
    assert_eq!(model.bank(16), None);
    assert_eq!(
        model.byte_at(DESCRIPTOR_ROOT),
        Some(rom[DESCRIPTOR_ROOT as usize])
    );
    assert_eq!(model.byte_at(DESCRIPTOR_ROOT - 1), None);
    let encoded = serde_json::to_vec_pretty(model).unwrap();
    let parsed: LayeredObjAnimations = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(&parsed, model);
    for (descriptor, bank) in model.descriptors.iter().zip(&model.banks) {
        run_initializer(&rom, descriptor, bank);
    }
    for case in 0..6 {
        let mut bad = model.clone();
        match case {
            0 => {
                bad.descriptors.pop();
            }
            1 => bad.descriptors.swap(0, 1),
            2 => bad.banks[0].object_offsets[1] += 1,
            3 => bad.banks[0].sequences[0].frames[0].object_list = u16::MAX,
            4 => {
                bad.banks[0].sequences[0].frames.pop();
            }
            _ => {
                bad.banks[0]
                    .lists
                    .iter_mut()
                    .find(|list| !list.objects.is_empty())
                    .unwrap()
                    .objects[0]
                    .tile = 1024
            }
        }
        assert!(bad.validate().is_err(), "malformed case {case}");
    }
    let mut coherent_edit = model.clone();
    coherent_edit.descriptors[0].opaque[0] ^= 1;
    coherent_edit.validate().unwrap();
    assert!(coherent_edit.verify_source(&rom).is_err());
    let mut wrong_rom = rom.clone();
    wrong_rom[DESCRIPTOR_ROOT as usize] ^= 1;
    assert!(Catalog::from_rom(&wrong_rom).is_err());
}

#[test]
#[ignore = "private exact EUR source and original-code proof; Wine only"]
fn european_adjacent_constructor_coordinate_prefix() {
    let rom = std::fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(crate::ffta_rom::digest(&rom), crate::ffta_rom::EUROPE_SHA256);
    let offsets = [0x3ae12c, 0x3ae134, 0x3ae13c, 0x3ae144, 0x3ae14c, 0x3ae164];
    let seeds = [0u32, 1, 0x1234_5678, 0xffff_ffff];
    let read_half = |at: usize| i16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
    let next = |seed: u32| seed.wrapping_mul(0x41c6_4e6d).wrapping_add(0x3039);
    for &at in &offsets {
        let xbase = read_half(at);
        let ybase = read_half(at + 2);
        let xmod = read_half(at + 4);
        let ymod = read_half(at + 6);
        assert!(xmod > 0 && ymod > 0);
        for &seed in &seeds {
            let first = next(seed);
            let second = next(first);
            let x = i32::from(xbase) + (((first >> 16) & 0x7fff) as i32 % i32::from(xmod));
            let y = i32::from(ybase) + (((second >> 16) & 0x7fff) as i32 % i32::from(ymod));
            let mut machine = Machine::new(rom.clone());
            machine.bus.ffta_sprites.enabled = false;
            machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
            machine.cpu.regs[..4].copy_from_slice(&[TASK, 0x0800_0000 + at as u32, 0, 0]);
            machine.cpu.regs[13] = SP;
            machine.cpu.regs[14] = RETURN | 1;
            machine.cpu.regs[15] = 0x0814_36b4;
            machine.cpu.cpsr = 0xbf;
            machine.bus.write32(0x0300_34b0, seed);
            for offset in [0, 4, 8] { machine.bus.write32(SP + offset, 0); }
            let mut returned = false;
            for _ in 0..100_000 {
                if machine.cpu.regs[15] == 0x0814_372a { returned = true; break; }
                machine.bus.frame_ready = false;
                machine.step();
            }
            assert!(returned, "constructor prefix did not reach the coordinate stores");
            assert_eq!(machine.bus.read16(TASK + 2), x as u16, "x source {at:x} seed {seed:x}");
            assert_eq!(machine.bus.read16(TASK + 4), y as u16, "y source {at:x} seed {seed:x}");
            assert_eq!(machine.bus.read32(0x0300_34b0), second, "two generator advances");
        }
    }
}
