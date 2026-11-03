use super::*;
use crate::{Bus, Machine};
use std::{collections::BTreeSet, fs, path::PathBuf};

const TASK: u32 = 0x02010000;
const SP: u32 = 0x03007c00;
const RETURN: u32 = 0x08000300;
fn machine(rom: &[u8]) -> Machine {
    let mut m = Machine::new(rom.to_vec());
    m.bus.ffta_sprites.enabled = false;
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    m.cpu.regs[13] = SP;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.cpsr = 0xbf;
    m.bus.ewram.fill(0xa5);
    m.bus.iwram.fill(0xc3);
    m
}
fn run(m: &mut Machine) -> usize {
    for steps in 0..100 {
        if m.cpu.regs[15] == RETURN {
            return steps;
        }
        assert!(m.cpu.thumb());
        m.bus.frame_ready = false;
        m.step();
    }
    panic!("word-task prefix return bound");
}
fn initial_task(ew: &mut [u8], task: u32, bank: &obj_animations::Bank, selector: u16, repeat: u8) {
    let at = (task - 0x02000000) as usize;
    let seq = &bank.sequences[usize::from(selector)];
    for (offset, value) in [
        (0, 0x08000000 + bank.object_root),
        (4, 0x08000000 + bank.sequence_root),
        (12, 0x08000000 + seq.source_offset + 2),
    ] {
        ew[at + offset..at + offset + 4].copy_from_slice(&value.to_le_bytes());
    }
    for (offset, value) in [
        (8, seq.frames.len() as u16),
        (16, selector),
        (18, 0),
        (20, 0),
        (24, 0),
    ] {
        ew[at + offset..at + offset + 2].copy_from_slice(&value.to_le_bytes());
    }
    ew[at + 22] = repeat;
}
fn constructor(rom: &[u8], bank: &obj_animations::Bank, selector: u32, repeat: u32) -> usize {
    let mut m = machine(rom);
    m.cpu.regs[..4].copy_from_slice(&[
        TASK,
        0x08000000 + bank.object_root,
        0x08000000 + bank.sequence_root,
        selector,
    ]);
    m.cpu.regs[15] = 0x0802c5e0;
    m.bus.write32(SP, repeat);
    let regs = m.cpu.regs;
    let mut ew = m.bus.ewram.clone();
    let mut iw = m.bus.iwram.clone();
    let (vram, oam, palette) = (m.bus.vram.clone(), m.bus.oam.clone(), m.bus.palette.clone());
    initial_task(&mut ew, TASK, bank, selector as u16, repeat as u8);
    iw[0x7bf8..0x7bfc].copy_from_slice(&regs[4].to_le_bytes());
    iw[0x7bfc..0x7c00].copy_from_slice(&regs[14].to_le_bytes());
    let steps = run(&mut m);
    assert_eq!(&m.cpu.regs[3..15], &regs[3..15]);
    assert_eq!(
        m.cpu.regs[0],
        RETURN | 1,
        "constructor POP r0 carries the caller return, not the task"
    );
    assert_eq!(m.cpu.cpsr & 0x0fffffff, 0xbf);
    assert_eq!(m.bus.ewram, ew);
    assert_eq!(m.bus.iwram, iw);
    assert_eq!(m.bus.vram, vram);
    assert_eq!(m.bus.oam, oam);
    assert_eq!(m.bus.palette, palette);
    steps
}
fn update(rom: &[u8], bank: &obj_animations::Bank, phase: u16, tick: u16, repeat: u8) -> usize {
    let mut m = machine(rom);
    initial_task(&mut m.bus.ewram, TASK, bank, 0, repeat);
    m.bus.write16(TASK + 18, tick);
    m.bus.write16(TASK + 20, phase);
    m.bus.write16(TASK + 24, 0x5544);
    m.cpu.regs[0] = TASK;
    m.cpu.regs[15] = 0x0802c60c;
    let regs = m.cpu.regs;
    let mut ew = m.bus.ewram.clone();
    let mut iw = m.bus.iwram.clone();
    let (vram, oam, palette) = (m.bus.vram.clone(), m.bus.oam.clone(), m.bus.palette.clone());
    let frame = &bank.sequences[0].frames[usize::from(phase)];
    let count = bank.sequences[0].frames.len() as u16;
    let increment = tick.wrapping_add(1);
    let advance = increment > frame.duration;
    let mut next = phase;
    let mut done = 0x5544u16;
    if advance {
        next = phase.wrapping_add(1);
        if next >= count {
            if repeat != 0 {
                next = 0;
            } else {
                next = count.wrapping_sub(1);
                done = 1;
            }
        }
    }
    ew[0x10012..0x10014].copy_from_slice(&(if advance { 0 } else { increment }).to_le_bytes());
    ew[0x10014..0x10016].copy_from_slice(&next.to_le_bytes());
    ew[0x10018..0x1001a].copy_from_slice(&done.to_le_bytes());
    for (at, value) in [(0x7bf4, regs[4]), (0x7bf8, regs[5]), (0x7bfc, regs[14])] {
        iw[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
    let steps = run(&mut m);
    assert_eq!(
        m.cpu.regs[0],
        0x08000000 + bank.lists[usize::from(frame.object_list)].source_offset,
        "return OLDphase list even after advance"
    );
    assert_eq!(&m.cpu.regs[4..15], &regs[4..15]);
    assert_eq!(m.cpu.cpsr & 0x0fffffff, 0xbf);
    assert_eq!(m.bus.ewram, ew, "only tick/phase/latched done may change");
    assert_eq!(m.bus.iwram, iw);
    assert_eq!(m.bus.vram, vram);
    assert_eq!(m.bus.oam, oam);
    assert_eq!(m.bus.palette, palette);
    steps
}
fn bank_producer(rom: &[u8], model: &WordTaskObjAnimations, id: u32, repeat: u32) -> usize {
    let mut m = machine(rom);
    m.bus.write32(0x0200f428, 0x02010000);
    m.cpu.regs[0] = id;
    m.cpu.regs[1] = repeat;
    m.cpu.regs[15] = 0x0805e6c4;
    let regs = m.cpu.regs;
    let mut ew = m.bus.ewram.clone();
    let iw = m.bus.iwram.clone();
    let (vram, oam, palette) = (m.bus.vram.clone(), m.bus.oam.clone(), m.bus.palette.clone());
    let bank = model
        .bank(if id as u8 <= 78 { 0x4165a0 } else { 0x41647c })
        .unwrap();
    initial_task(&mut ew, 0x020133e8, bank, 0, repeat as u8);
    let steps = run(&mut m);
    assert_eq!(&m.cpu.regs[4..14], &regs[4..14]);
    assert_eq!(m.cpu.cpsr & 0x0fffffff, 0xbf);
    assert_eq!(m.bus.ewram, ew);
    assert_eq!(&m.bus.iwram[..0x7bf0], &iw[..0x7bf0]);
    assert_eq!(&m.bus.iwram[0x7c00..], &iw[0x7c00..]);
    assert_eq!(m.bus.vram, vram);
    assert_eq!(m.bus.oam, oam);
    assert_eq!(m.bus.palette, palette);
    steps
}

#[test]
#[ignore = "private EUR u16-task bank source/model; Wine only, Linux/USA pending"]
fn european_word_task_obj_banks_match_original_prefixes() {
    let root = PathBuf::from(std::env::var_os("FFTA_WORD_OBJ_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let model = &catalog.decoded;
    model.verify_source(&rom).unwrap();
    let spans = model.source_spans().unwrap();
    assert_eq!(
        (spans.len(), spans.iter().map(|s| s.1.len()).sum::<usize>()),
        (65, 1378)
    );
    assert_eq!(catalog.sources.len(), 65);
    let mut covered = BTreeSet::new();
    for ((at, raw, role), source) in spans.iter().zip(&catalog.sources) {
        assert_eq!(
            (source.offset, source.length, source.role.as_str()),
            (*at, raw.len() as u32, role.as_str())
        );
        assert_eq!(source.sha256, ffta_rom::digest(raw));
        assert_eq!(raw, &rom[*at as usize..*at as usize + raw.len()]);
        for (i, &value) in raw.iter().enumerate() {
            covered.insert(*at + i as u32);
            assert_eq!(model.byte_at(*at + i as u32), Some(value));
        }
    }
    for (at, raw, _) in &spans {
        for edge in [at - 1, at + raw.len() as u32] {
            if !covered.contains(&edge) {
                assert_eq!(
                    model.byte_at(edge),
                    None,
                    "padding and outer edges remain outside"
                );
            }
        }
    }
    assert_eq!(model.banks.iter().map(|b| b.lists.len()).sum::<usize>(), 50);
    assert_eq!(
        model
            .banks
            .iter()
            .flat_map(|b| &b.lists)
            .map(|l| l.objects.len())
            .sum::<usize>(),
        140
    );
    assert_eq!(
        model
            .banks
            .iter()
            .flat_map(|b| &b.sequences)
            .map(|s| s.frames.len())
            .sum::<usize>(),
        49
    );
    for bank in &model.banks {
        assert!(model.list(bank.object_root, 0).is_some());
        assert!(model.list(bank.object_root, u16::MAX).is_none());
        assert!(model.sequence(bank.object_root, u16::MAX).is_none());
        assert!(model.frame(bank.object_root, 0, u16::MAX).is_none());
        if bank.sequences.len() == 2 {
            assert!(model
                .sequence(bank.object_root, 1)
                .unwrap()
                .frames
                .is_empty());
            assert!(model.frame(bank.object_root, 1, 0).is_none());
        }
    }
    let encoded = serde_json::to_vec_pretty(model).unwrap();
    let parsed: WordTaskObjAnimations = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(&parsed, model);
    let mut guards = 0;
    for case in 0..9 {
        let mut bad = model.clone();
        match case {
            0 => {
                bad.banks.pop();
            }
            1 => bad.banks.swap(0, 1),
            2 => bad.banks[0].object_root += 4,
            3 => bad.banks[0].object_offsets[1] += 1,
            4 => bad.banks[0].lists[0].source_offset += 2,
            5 => bad.banks[0].sequences[0].frames[0].object_list = u16::MAX,
            6 => {
                bad.banks[0].sequences[0].frames.pop();
            }
            7 => bad.banks[0].sequences[1]
                .frames
                .push(obj_animations::Frame {
                    object_list: 0,
                    duration: 0,
                }),
            _ => bad.banks[0].lists[1].objects[0].tile = 1024,
        };
        assert!(bad.validate().is_err());
        guards += 1;
    }
    for duration in [false, true] {
        let mut coherent = model.clone();
        if duration {
            coherent.banks[0].sequences[0].frames[0].duration = u16::MAX;
        } else {
            coherent.banks[0].lists[1].objects[0].tile ^= 1;
        }
        coherent.validate().unwrap();
        let parsed: WordTaskObjAnimations =
            serde_json::from_slice(&serde_json::to_vec(&coherent).unwrap()).unwrap();
        assert_eq!(parsed, coherent);
        assert!(coherent.verify_source(&rom).is_err());
        guards += 1;
    }
    for case in 0..8 {
        let mut v = serde_json::to_value(model).unwrap();
        match case {
            0 => v["unknown"] = true.into(),
            1 => v["banks"][0]["unknown"] = true.into(),
            2 => v["banks"][0]["sequences"][0]["unknown"] = true.into(),
            3 => v["banks"][0]["sequences"][0]["frames"][0]["unknown"] = true.into(),
            4 => v["banks"][0]["sequences"][0]["frames"][0]["duration"] = 65536.into(),
            5 => v["banks"][0]["sequences"][0]["frames"][0]["duration"] = 0.5.into(),
            6 => v["banks"][0]["lists"][0]["unknown"] = true.into(),
            _ => v["banks"][0]["lists"][1]["objects"][0]["unknown"] = true.into(),
        };
        assert!(serde_json::from_value::<WordTaskObjAnimations>(v).is_err());
        guards += 1;
    }
    assert!(
        serde_json::from_slice::<WordTaskObjAnimations>(&encoded[..encoded.len() - 1]).is_err()
    );
    guards += 1;
    for at in [0x41634c, 0x2c5e0] {
        let mut bad = rom.clone();
        bad[at] ^= 1;
        assert!(Catalog::from_rom(&bad).is_err());
        guards += 1;
    }
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    guards += 1;
    assert_eq!(guards, 23);
    let prefixes = original_prefixes(&rom, model);
    let constructors = prefixes["original_constructors"].as_u64().unwrap();
    let updates = prefixes["original_updates"].as_u64().unwrap();
    let producers = prefixes["original_producer_branches"].as_u64().unwrap();
    let steps = prefixes["original_instructions"].as_u64().unwrap();
    assert_eq!(
        (super::super::EXTRACTOR_REV, super::super::DECODER_REV),
        (76, 75)
    );
    fs::write(root.join("decoded.json"), encoded).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","banks":4,"lists":50,"objects":140,"sequences":7,"frames":49,"source_spans":65,"source_bytes":1378,"guards":guards,"original_constructors":constructors,"original_updates":updates,"original_producer_branches":producers,"original_instructions":steps,"u16_tick255_advances_65535wraps":true,"old_list_return_before_advance":true,"repeat_clamp_latched_done":true,"three_zero_headers_opaque_no_inventedframe":true,"u16_selector_u8_repeat_aliases":true,"full_RAM_callee_guards":true,"managed_runtime_revision_unchanged":true,"Linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false})).unwrap()).unwrap();
    eprintln!("ROM_DATA_WORD_OBJ_MODEL banks4 lists50 OBJ140 sequences7 frames49 source1378B guards23 ctor23 update93 producer6 PASS");
}

fn original_prefixes(rom: &[u8], model: &WordTaskObjAnimations) -> serde_json::Value {
    let (mut constructors, mut updates, mut producers, mut steps) = (0, 0, 0, 0);
    for bank in &model.banks {
        for repeat in [0, 1, 0x100, 0x101] {
            steps += constructor(&rom, bank, 0, repeat);
            constructors += 1;
        }
        steps += constructor(&rom, bank, 0x10000, 0);
        constructors += 1;
        if bank.sequences.len() == 2 {
            steps += constructor(&rom, bank, 1, 0);
            constructors += 1;
        }
        for phase in 0..bank.sequences[0].frames.len() {
            steps += update(&rom, bank, phase as u16, 0, 0);
            updates += 1;
        }
        let last = bank.sequences[0].frames.len() - 1;
        let duration = bank.sequences[0].frames[last].duration;
        for tick in [duration - 1, duration, duration + 1, 255, u16::MAX] {
            for repeat in [0, 1] {
                steps += update(&rom, bank, last as u16, tick, repeat);
                updates += 1;
            }
        }
        steps += update(&rom, bank, 0, bank.sequences[0].frames[0].duration, 0);
        updates += 1;
    }
    for (id, repeat) in [
        (0, 0),
        (78, 1),
        (79, 0),
        (255, 1),
        (0x100, 0x100),
        (0x14e, 0x101),
    ] {
        steps += bank_producer(&rom, model, id, repeat);
        producers += 1;
    }
    assert_eq!((constructors, updates, producers), (23, 93, 6));
    serde_json::json!({"original_constructors":constructors,"original_updates":updates,"original_producer_branches":producers,"original_instructions":steps,"u16_tick255_advances_65535wraps":true,"old_list_return_before_advance":true,"repeat_clamp_latched_done":true,"u16_selector_u8_repeat_aliases":true,"full_RAM_callee_guards":true})
}

#[test]
#[ignore = "remaining private EUR u16-task original prefixes after constructor-return test correction; Wine only"]
fn european_word_task_obj_banks_remaining_original_prefixes() {
    let root = PathBuf::from(std::env::var_os("FFTA_WORD_OBJ_REMAINING_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let model = Catalog::from_rom(&rom).unwrap().decoded;
    let result = original_prefixes(&rom, &model);
    fs::write(
        root.join("decoded.json"),
        serde_json::to_vec_pretty(&model).unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_WORD_OBJ_REMAINING ctor23 update93 producer6 fullRAM_callee PASS; prior model23guard stage not replayed");
}
