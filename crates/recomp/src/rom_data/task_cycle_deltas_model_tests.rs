//! One new Wine source/model filter; no managed set or ordinary fixture.
use super::*;
use gba_core::{
    rom_data::task_cycle_deltas::{Catalog, TaskCycleDeltas, CYCLE_LENGTH, SOURCE, SOURCE_SHA256},
    Bus, Machine,
};

const TASK: u32 = 0x02010000;
const SP: u32 = 0x03007c00;

fn pristine(rom: &[u8]) -> Machine {
    let mut m = crate::make_machine(rom.to_vec(), None);
    m.bus.ffta_sprites.enabled = false;
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    m.cpu.regs[13] = SP;
    m.cpu.regs[14] = 0x08000301;
    m.cpu.cpsr = 0xbf;
    m
}

fn original_cycle(rom: &[u8], model: &TaskCycleDeltas, phase: u8, subtract: bool) -> usize {
    // Outside phases deliberately use the original raw/prior-owner bytes.
    let raw = rom[SOURCE as usize + usize::from(phase)];
    let delta = raw as i8;
    assert_eq!(model.delta(phase), (phase < 48).then_some(delta));
    let old = match (delta.cmp(&0), subtract) {
        (std::cmp::Ordering::Greater, false) | (std::cmp::Ordering::Less, true) => u16::MAX,
        (std::cmp::Ordering::Greater, true) | (std::cmp::Ordering::Less, false) => 0,
        _ => 0x1234,
    };
    let increment = phase.wrapping_add(1);
    let mut m = pristine(rom);
    m.cpu.regs[4] = TASK;
    m.cpu.regs[5] = if subtract { 8 } else { 0 };
    m.cpu.regs[15] = 0x08018df8;
    m.bus.write16(TASK + 8, old);
    m.bus.write8(TASK + 0x12, phase);
    let regs = m.cpu.regs;
    let mut ew = m.bus.ewram.clone();
    let mut iw = m.bus.iwram.clone();
    let oam = m.bus.oam.clone();
    let vram = m.bus.vram.clone();
    let palette = m.bus.palette.clone();
    let result = if subtract {
        old.wrapping_sub(i16::from(delta) as u16)
    } else {
        old.wrapping_add(i16::from(delta) as u16)
    };
    ew[0x10008..0x1000a].copy_from_slice(&result.to_le_bytes());
    ew[0x10012] = increment % 48;
    if increment >= 48 {
        iw[0x7bfc..0x7c00].copy_from_slice(&TASK.to_le_bytes());
    }
    let mut loads = 0;
    let mut helpers = 0;
    let mut returns = 0;
    let mut steps = 0;
    while m.cpu.regs[15] != 0x08018e40 {
        assert!(steps < 160, "bounded original cycle reader");
        let pc = m.cpu.regs[15];
        assert!(m.cpu.thumb());
        if pc == 0x08018e10 || pc == 0x08018e2a {
            assert_eq!(m.cpu.regs[0], 0x08000000 + SOURCE + u32::from(phase));
            assert_eq!(
                m.bus.read8(TASK + 0x12),
                increment,
                "store BEFORE old-index lookup"
            );
            loads += 1;
        }
        if pc == 0x08018e30 {
            assert_eq!(
                m.cpu.regs[0],
                i32::from(delta) as u32,
                "plus path sign extension"
            );
        }
        if pc == 0x081464ec {
            assert_eq!(&m.cpu.regs[..2], &[u32::from(increment), 48]);
            helpers += 1;
        }
        if pc == 0x08018e3e {
            assert_eq!(m.cpu.regs[0], u32::from(increment % 48));
            returns += 1;
        }
        m.bus.frame_ready = false;
        m.step();
        if pc == 0x08018e10 {
            assert_eq!(m.cpu.regs[1], i32::from(delta) as u32, "minus LDRSB");
        } else if pc == 0x08018e2a {
            assert_eq!(
                m.cpu.regs[0],
                u32::from(raw),
                "plus unsigned load before sign extension"
            );
        }
        steps += 1;
    }
    assert_eq!((loads, helpers, returns), (1, 1, 1));
    assert_eq!(&m.cpu.regs[4..12], &regs[4..12]);
    assert_eq!(m.cpu.regs[13], SP);
    assert_eq!(m.cpu.regs[14], 0x08018e3f);
    assert_eq!(m.cpu.cpsr & 0x0fffffff, 0xbf);
    assert_eq!(m.bus.ewram, ew, "only task halfword+8 and phase may change");
    assert_eq!(m.bus.iwram, iw, "only helper push slot may change");
    assert_eq!(m.bus.oam, oam);
    assert_eq!(m.bus.vram, vram);
    assert_eq!(m.bus.palette, palette);
    steps
}

fn original_remainder(rom: &[u8], numerator: u8) -> usize {
    let mut m = pristine(rom);
    m.cpu.regs[0] = u32::from(numerator);
    m.cpu.regs[1] = 48;
    m.cpu.regs[15] = 0x081464ec;
    let regs = m.cpu.regs;
    let ew = m.bus.ewram.clone();
    let mut iw = m.bus.iwram.clone();
    if numerator >= 48 {
        iw[0x7bfc..0x7c00].copy_from_slice(&regs[4].to_le_bytes());
    }
    let mut steps = 0;
    while m.cpu.regs[15] != 0x08000300 {
        assert!(steps < 140);
        assert!(m.cpu.thumb(), "MOV PC,LR preserves Thumb");
        m.bus.frame_ready = false;
        m.step();
        steps += 1;
    }
    assert_eq!(
        m.cpu.regs[0],
        u32::from(numerator % 48),
        "remainder, not quotient"
    );
    assert_eq!(&m.cpu.regs[4..12], &regs[4..12]);
    assert_eq!(&m.cpu.regs[13..15], &regs[13..15]);
    assert!(m.cpu.thumb());
    assert_eq!(m.cpu.cpsr & 0x0fffffff, 0xbf);
    assert_eq!(m.bus.ewram, ew);
    assert_eq!(m.bus.iwram, iw);
    steps
}

#[test]
#[ignore = "private EUR signed cycle model/source prefixes; Wine only, Linux/USA pending"]
fn european_task_cycle_deltas_model_matches_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_TASK_CYCLE_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let model = &catalog.decoded;
    model.verify_source(&rom).unwrap();
    let spans = model.source_spans().unwrap();
    assert_eq!(spans.len(), 1);
    assert_eq!(catalog.sources.len(), 1);
    let (offset, bytes, role) = &spans[0];
    let source = &catalog.sources[0];
    assert_eq!((*offset, bytes.len()), (SOURCE, CYCLE_LENGTH));
    assert_eq!(bytes, &rom[SOURCE as usize..SOURCE as usize + 48]);
    assert_eq!((source.offset, source.length), (SOURCE, 48));
    assert_eq!(&source.role, role);
    assert_eq!(source.sha256, SOURCE_SHA256);
    assert_eq!(gba_core::ffta_rom::digest(bytes), SOURCE_SHA256);
    for (i, &raw) in bytes.iter().enumerate() {
        assert_eq!(model.delta(i as u8), Some(raw as i8));
        assert_eq!(model.byte_at(SOURCE + i as u32), Some(raw));
    }
    for phase in 48..=255 {
        assert_eq!(model.delta(phase), None);
    }
    for outside in [SOURCE - 1, SOURCE + 48, SOURCE + 49, SOURCE + 50, u32::MAX] {
        assert_eq!(model.byte_at(outside), None);
    }
    let encoded = serde_json::to_vec_pretty(model).unwrap();
    let parsed: TaskCycleDeltas = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(&parsed, model);
    let mut guards = 0;
    for case in 0..5 {
        let mut bad = model.clone();
        match case {
            0 => bad.source_offset += 1,
            1 => {
                bad.entries.pop();
            }
            2 => bad.entries.reverse(),
            3 => bad.entries[1].phase = 0,
            _ => bad.entries.push(bad.entries[47].clone()),
        }
        assert!(bad.validate().is_err());
        guards += 1;
    }
    for delta in [i8::MIN, i8::MAX] {
        let mut coherent = model.clone();
        coherent.entries[0].delta = delta;
        coherent.validate().unwrap();
        assert_eq!(coherent.byte_at(SOURCE), Some(delta as u8));
        let roundtrip: TaskCycleDeltas =
            serde_json::from_slice(&serde_json::to_vec(&coherent).unwrap()).unwrap();
        assert_eq!(roundtrip, coherent);
        assert!(coherent.verify_source(&rom).is_err());
        guards += 1;
    }
    for case in 0..8 {
        let mut value = serde_json::to_value(model).unwrap();
        match case {
            0 => value["unknown"] = true.into(),
            1 => value["entries"][0]["unknown"] = true.into(),
            2 => value["entries"][0]["delta"] = 128.into(),
            3 => value["entries"][0]["delta"] = (-129).into(),
            4 => value["entries"][0]["phase"] = 256.into(),
            5 => value["source_offset"] = 0x100000000u64.into(),
            6 => {
                value.as_object_mut().unwrap().remove("entries");
            }
            _ => value["entries"][0]["delta"] = 0.5.into(),
        }
        assert!(serde_json::from_value::<TaskCycleDeltas>(value).is_err());
        guards += 1;
    }
    assert!(serde_json::from_slice::<TaskCycleDeltas>(&encoded[..encoded.len() - 1]).is_err());
    guards += 1;
    // Each mutation tests exact-source refusal, including both literals/helper.
    for at in [
        SOURCE as usize,
        0x18e10,
        0x18e2a,
        0x18e18,
        0x18e58,
        0x1464ec,
    ] {
        let mut bad = rom.clone();
        bad[at] ^= 1;
        assert!(Catalog::from_rom(&bad).is_err());
        guards += 1;
    }
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    guards += 1;
    assert_eq!(guards, 23);
    let mut steps = 0;
    for phase in 0..48 {
        steps += original_cycle(&rom, model, phase, phase & 1 != 0);
    }
    // These are prelookup outside cases, not another asset or an input cap.
    for phase in [48, 49, 50, 254, 255] {
        for subtract in [false, true] {
            steps += original_cycle(&rom, model, phase, subtract);
        }
    }
    for numerator in [0, 47, 48, 95, 96, 191, 192, 255] {
        steps += original_remainder(&rom, numerator);
    }
    assert_eq!(
        (
            gba_core::rom_data::EXTRACTOR_REV,
            gba_core::rom_data::DECODER_REV
        ),
        (75, 74)
    );
    fs::write(root.join("decoded.json"), encoded).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "source_bytes":48,"spans":1,"rows":48,"guards":guards,
        "original_cycle_prefixes":58,"helper_boundaries":8,"original_instructions":steps,
        "old_index_before_normalization":true,"stored_u8_increment_wrap":true,
        "signed_add_sub_low16_wrap":true,"remainder48_not_quotient":true,
        "full_RAM_callee_guards":true,"outside_queries_preserved":true,
        "publication_runtime_revision_unchanged":true,"ordinary_fixture":false,
        "Linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false
    })).unwrap()).unwrap();
    eprintln!("ROM_DATA_TASK_CYCLE_MODEL source_bytes=48 guards={guards} prefixes=58 helper_boundaries=8 PASS");
}
