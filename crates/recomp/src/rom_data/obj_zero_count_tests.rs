//! New EUR zero-count bank: original read contract before typed migration.
use super::*;
use gba_core::{
    rom_data::{obj_animations::Bank, obj_lists::List, ObjDimensions},
    Bus, Machine,
};
use std::collections::BTreeSet;

const TASK: u32 = 0x02028050;
const STACK: u32 = 0x03007c00;
const RETURN: u32 = 0x08000300;
const BUFFER: u32 = 0x03000030;
const OBJECT_ROOT: u32 = 0x00b4d8e8;
const SEQUENCE_ROOT: u32 = 0x00b4f4e8;

fn invoke(m: &mut Machine, args: [u32; 4], stack: &[u32]) -> usize {
    m.cpu.regs[..4].copy_from_slice(&args);
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.regs[15] = 0x0804e494;
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
    panic!("new zero-count constructor failed to return");
}

fn tick(m: &mut Machine, body: u32, reads: &mut BTreeSet<(u32, u32, u16)>) -> usize {
    let (selector_pc, duration_pc, end) = match body {
        0x0804e058 => (0x0804e060, 0x0804e0ca, 0x0804e1e8),
        0x0813f814 => (0x0813f81c, 0x0813f888, 0x0813f9a6),
        _ => unreachable!(),
    };
    m.cpu.regs[5] = TASK;
    m.cpu.regs[13] = STACK;
    m.cpu.regs[15] = body;
    m.cpu.cpsr = 0xbf;
    m.bus.write8(0x03000028, 0);
    m.bus.write32(0x03000020, 0);
    for at in 0..1024 {
        m.bus.write8(BUFFER + at, 0x5a);
    }
    for steps in 1..=20_000 {
        let pc = m.cpu.regs[15];
        let address = if pc == selector_pc {
            Some(m.cpu.regs[0])
        } else if pc == duration_pc {
            Some(m.cpu.regs[1] + 2)
        } else {
            None
        };
        if let Some(address) = address {
            reads.insert((pc, address, m.bus.read16(address)));
        }
        m.bus.frame_ready = false;
        m.step();
        if m.cpu.regs[15] == end {
            assert_eq!(m.cpu.regs[13], STACK);
            return steps;
        }
    }
    panic!("new zero-count scheduler failed to reach next-object edge");
}

fn expected_oam(
    list: &List,
    dimensions: &ObjDimensions,
    x: u32,
    y: u32,
    tile: u16,
) -> (Vec<u8>, u32) {
    let mut out = vec![0x5a; 1024];
    let mut count = 0;
    for object in &list.objects {
        let dim = dimensions.get(object.shape, object.size).unwrap();
        let px = (x as i32 + i32::from(object.x)) as i16 as i32;
        let py = (y as i32 + i32::from(object.y)) as i16 as i32;
        if (px < 0 && px + i32::from(dim.width) < 0)
            || (px > 239 && px + i32::from(dim.width) > 239)
            || (py < 0 && py + i32::from(dim.height) < 0)
            || (py > 159 && py + i32::from(dim.height) > 159)
        {
            continue;
        }
        let [a0, a1, _] = object.attributes();
        let attrs = [
            (a0 & 0xff00) | (py as u16 & 255),
            (a1 & !511) | (px as u16 & 511),
            ((object.tile + tile) % 1024)
                | (u16::from(object.priority) << 10)
                | (u16::from(object.palette) << 12),
        ];
        for (n, attr) in attrs.into_iter().enumerate() {
            out[count * 8 + n * 2..count * 8 + n * 2 + 2].copy_from_slice(&attr.to_le_bytes());
        }
        count += 1;
    }
    (out, count as u32)
}

#[test]
#[ignore = "exact new EUR zero-count bank; Wine only, Linux/USA pending"]
fn european_zero_count_pair_original_nonrepeat_constructors_and_scheduler_reads() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_OBJ_ZERO_ROOT").unwrap());
    assert!(!root.exists(), "preserve existing original-code evidence");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let bank = Bank::decode(&rom, OBJECT_ROOT, SEQUENCE_ROOT).unwrap();
    assert_eq!(
        (
            bank.lists.len(),
            bank.lists[0].objects.len(),
            bank.sequences.len()
        ),
        (1, 2, 5)
    );
    assert!(bank.sequences.iter().all(|s| s.frames.is_empty()));
    let dimensions = ObjDimensions::decode(&rom[0x14ced8..0x14cf08]).unwrap();
    let mut cases = vec![(1u32, 0usize, 4u32, 24u32, 0x34u16), (2, 0, 212, 24, 0x3c)];
    cases.extend((0..5).map(|sequence| (0, sequence, 100, 70, 7)));
    let mut reports = Vec::new();
    let mut constructors = 0;
    let mut ticks = 0;
    let mut instructions = 0;
    for (task_index, sequence_id, x, y, tile) in cases {
        let sequence = &bank.sequences[sequence_id];
        let selector_at = 0x08000000 + sequence.source_offset + 2;
        let duration_at = selector_at + 2;
        let expected_duration = if sequence_id == 4 { 32 } else { 0 };
        let (oam, count) = expected_oam(&bank.lists[0], &dimensions, x, y, tile);
        for body in [0x0804e058, 0x0813f814] {
            let mut m = crate::make_machine(rom.clone(), None);
            m.bus.ffta_sprites.enabled = false;
            for at in 0..60 {
                m.bus.write8(TASK - 16 + at, 0x5a);
            }
            for at in 0..28 {
                m.bus.write8(TASK + at, 0);
            }
            instructions += invoke(
                &mut m,
                [TASK - task_index * 28, task_index, sequence_id as u32, x],
                &[
                    y,
                    0,
                    0,
                    0x08000000 + OBJECT_ROOT,
                    0x08000000 + SEQUENCE_ROOT,
                    u32::from(tile),
                ],
            );
            constructors += 1;
            let mut expected = vec![0u8; 28];
            expected[..4].copy_from_slice(&selector_at.to_le_bytes());
            expected[13] = 1;
            expected[16] = x as u8;
            expected[17] = y as u8;
            expected[22..24].copy_from_slice(&tile.to_le_bytes());
            expected[24..].copy_from_slice(&(0x08000000 + OBJECT_ROOT).to_le_bytes());
            assert!(
                (0..28).all(|at| m.bus.read8(TASK + at) == expected[at as usize]),
                "zero-count constructor task fields"
            );
            let mut reads = BTreeSet::new();
            let mut elapsed = 0u8;
            let iterations = u32::from(expected_duration.max(1)) + 2;
            for _ in 0..iterations {
                instructions += tick(&mut m, body, &mut reads);
                ticks += 1;
                elapsed = elapsed.wrapping_add(1);
                if u16::from(elapsed) >= expected_duration {
                    elapsed = 0;
                }
                expected[15] = elapsed;
                assert!(
                    (0..28).all(|at| m.bus.read8(TASK + at) == expected[at as usize]),
                    "nonrepeat zero count holds index zero, including after duration"
                );
                assert_eq!(m.bus.read32(0x03000020), count);
                assert!(
                    (0..1024).all(|at| m.bus.read8(BUFFER + at) == oam[at as usize]),
                    "complete OAM/matrices and guards"
                );
                assert!((0..16)
                    .all(|at| m.bus.read8(TASK - 16 + at) == 0x5a
                        && m.bus.read8(TASK + 28 + at) == 0x5a));
            }
            assert_eq!(reads.len(), 2);
            assert!(reads
                .iter()
                .any(|&(_, address, value)| address == selector_at && value == 0));
            assert!(reads
                .iter()
                .any(|&(_, address, value)| address == duration_at && value == expected_duration));
            reports.push(serde_json::json!({"task_index":task_index,"sequence_id":sequence_id,"body":body,"ticks":iterations,
                "reads":reads,"zero_count_nonrepeat_holds_first_selector":true,"oam_matrices_task_guards":true}));
            assert_eq!(
                gba_core::ffta_rom::digest(&m.bus.rom),
                gba_core::ffta_rom::EUROPE_SHA256
            );
        }
    }
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"region":"europe","constructor_calls":constructors,
        "scheduler_body_ticks":ticks,"instructions":instructions,"cases":reports,"runtime":"source contract only; no migration yet",
        "actual_callers":"080332ca/080332f0 select sequence zero and repeat zero; other sequence IDs are bounded source/read probes",
        "duration_alias":"next sequence count, or first halfword of the already rooted next bank's index for sequence4",
        "repeat_one":"not selected by these two callers; unverified","whole_scheduler_and_ordinary_scene":"pending",
        "linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
    eprintln!("OBJ_ZERO_EUR constructors={constructors} ticks={ticks} instructions={instructions} source/selectors/aliased-duration/OAM/task/guards=pass");
}
