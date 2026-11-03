//! One reduced Wine filter for the two unpublished task-parameter fragments.
use super::*;
use gba_core::{
    rom_data::observed_task_parameters::{
        Catalog, ObservedTaskParameters, PHASE_SOURCE, VERTICAL_SOURCE,
    },
    Bus, Machine,
};

const TASK: u32 = 0x02010000;

fn pristine(rom: &[u8]) -> Machine {
    let mut m = crate::make_machine(rom.to_vec(), None);
    m.bus.ffta_sprites.enabled = false;
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    m.cpu.regs[13] = 0x03007c00;
    m.cpu.regs[14] = 0x08000301;
    m.cpu.cpsr = 0xbf;
    m
}

pub(super) fn original_phase(rom: &[u8], model: &ObservedTaskParameters, phase: u8, old: u16) {
    let threshold = model.phase_thresholds.threshold(phase).unwrap();
    let mut m = pristine(rom);
    m.cpu.regs[4] = TASK;
    m.cpu.regs[15] = 0x08018dca;
    m.bus.write16(TASK + 12, old);
    m.bus.write8(TASK + 18, phase);
    let mut expected_ew = m.bus.ewram.clone();
    let iw = m.bus.iwram.clone();
    let oam = m.bus.oam.clone();
    let vram = m.bus.vram.clone();
    let palette = m.bus.palette.clone();
    let callee = m.cpu.regs[3..15].to_vec();
    // The store precedes the comparison, which deliberately uses OLD counter.
    let advance = old >= u16::from(threshold);
    let counter = if advance { 0 } else { old.wrapping_add(1) };
    let next_phase = if advance {
        if phase == 2 {
            0
        } else {
            phase + 1
        }
    } else {
        phase
    };
    expected_ew[0x1000c..0x1000e].copy_from_slice(&counter.to_le_bytes());
    expected_ew[0x10012] = next_phase;
    let mut seen = false;
    for _ in 0..30 {
        if m.cpu.regs[15] == 0x08018e40 {
            break;
        }
        let load = m.cpu.regs[15] == 0x08018dda;
        if load {
            assert_eq!(m.cpu.regs[1], 0x08371064 + u32::from(phase));
        }
        m.bus.frame_ready = false;
        m.step();
        if load {
            assert_eq!(m.cpu.regs[1], u32::from(threshold));
            seen = true;
        }
    }
    assert!(seen);
    assert_eq!(m.cpu.regs[15], 0x08018e40);
    assert_eq!(&m.cpu.regs[3..15], callee.as_slice());
    assert_eq!(m.cpu.cpsr & 0x0fffffff, 0xbf);
    assert_eq!(m.bus.ewram, expected_ew, "only counter/phase may change");
    assert_eq!(m.bus.iwram, iw);
    assert_eq!(m.bus.oam, oam);
    assert_eq!(m.bus.vram, vram);
    assert_eq!(m.bus.palette, palette);
}

fn original_vertical(rom: &[u8], model: &ObservedTaskParameters, task_byte: u8) {
    let correction = model.vertical_corrections.correction(task_byte).unwrap();
    let projected: u32 = if task_byte & 1 == 0 { 10 } else { 0x80000010 };
    let mut m = pristine(rom);
    m.cpu.regs[2] = projected;
    m.cpu.regs[6] = TASK;
    m.cpu.regs[15] = 0x080264ea;
    m.bus.write8(TASK + 7, task_byte);
    let regs = m.cpu.regs;
    let ew = m.bus.ewram.clone();
    let iw = m.bus.iwram.clone();
    let oam = m.bus.oam.clone();
    let vram = m.bus.vram.clone();
    let palette = m.bus.palette.clone();
    let mut seen = false;
    for _ in 0..12 {
        if m.cpu.regs[15] == 0x080264fa {
            break;
        }
        let load = m.cpu.regs[15] == 0x080264f6;
        if load {
            assert_eq!(m.cpu.regs[3], 0x08394e0c + u32::from(task_byte >> 1) * 2);
            assert_eq!(m.cpu.regs[4], 0);
        }
        m.bus.frame_ready = false;
        m.step();
        if load {
            assert_eq!(m.cpu.regs[3], i32::from(correction) as u32);
            seen = true;
        }
    }
    assert!(seen);
    assert_eq!(m.cpu.regs[15], 0x080264fa);
    assert_eq!(
        m.cpu.regs[2],
        projected.wrapping_sub(i32::from(correction) as u32)
    );
    for i in [0, 1, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14] {
        assert_eq!(m.cpu.regs[i], regs[i]);
    }
    assert_eq!(m.cpu.cpsr & 0x0fffffff, 0xbf);
    assert_eq!(m.bus.ewram, ew, "correction prefix is read-only");
    assert_eq!(m.bus.iwram, iw);
    assert_eq!(m.bus.oam, oam);
    assert_eq!(m.bus.vram, vram);
    assert_eq!(m.bus.palette, palette);
}

#[test]
#[ignore = "private EUR two-fragment model/original prefixes; Wine only, Linux/USA pending"]
fn european_observed_task_parameters_model_matches_original_prefixes() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_OBSERVED_TASK_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let model = &catalog.decoded;
    model.verify_source(&rom).unwrap();
    assert_eq!(
        model
            .phase_thresholds
            .entries
            .iter()
            .map(|e| (e.phase, e.threshold))
            .collect::<Vec<_>>(),
        [(1, 8), (2, 8)]
    );
    assert_eq!(
        model
            .vertical_corrections
            .entries
            .iter()
            .map(|e| (e.index, e.correction))
            .collect::<Vec<_>>(),
        [(4, 40), (5, 40), (6, 41), (7, 41)]
    );
    assert_eq!(catalog.sources.len(), 2);
    let mut bytes = 0;
    for (source, (at, raw, role)) in catalog.sources.iter().zip(model.source_spans().unwrap()) {
        assert_eq!((source.offset, source.length), (at, raw.len() as u32));
        assert_eq!(source.role, role);
        assert_eq!(source.sha256, gba_core::ffta_rom::digest(&raw));
        assert_eq!(raw, rom[at as usize..at as usize + raw.len()]);
        for (i, &value) in raw.iter().enumerate() {
            assert_eq!(model.byte_at(at + i as u32), Some(value));
        }
        assert_eq!(model.byte_at(at - 1), None);
        assert_eq!(model.byte_at(at + raw.len() as u32), None);
        bytes += raw.len();
    }
    assert_eq!(bytes, 10);
    for phase in [0, 3, 127, 255] {
        assert_eq!(model.phase_thresholds.threshold(phase), None);
    }
    for byte in (0..8).chain(16..=255) {
        assert_eq!(model.vertical_corrections.correction(byte), None);
    }
    let encoded = serde_json::to_vec_pretty(model).unwrap();
    let parsed: ObservedTaskParameters = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(&parsed, model);
    let mut guards = 0;
    for case in 0..10 {
        let mut bad = model.clone();
        match case {
            0 => bad.phase_thresholds.source_offset += 1,
            1 => {
                bad.phase_thresholds.entries.pop();
            }
            2 => bad.phase_thresholds.entries.reverse(),
            3 => bad.phase_thresholds.entries[0].phase = 0,
            4 => bad.phase_thresholds.entries[1].phase = 1,
            5 => bad.vertical_corrections.source_offset += 2,
            6 => {
                bad.vertical_corrections.entries.pop();
            }
            7 => bad.vertical_corrections.entries.reverse(),
            8 => bad.vertical_corrections.entries[0].index = 3,
            _ => bad.vertical_corrections.entries[1].index = 4,
        }
        assert!(bad.validate().is_err());
        guards += 1;
    }
    for vertical in [false, true] {
        let mut coherent = model.clone();
        if vertical {
            coherent.vertical_corrections.entries[0].correction = i16::MIN;
        } else {
            coherent.phase_thresholds.entries[0].threshold = u8::MAX;
        }
        coherent.validate().unwrap();
        let roundtrip: ObservedTaskParameters =
            serde_json::from_slice(&serde_json::to_vec(&coherent).unwrap()).unwrap();
        assert_eq!(roundtrip, coherent);
        if vertical {
            assert_eq!(
                [
                    coherent.byte_at(VERTICAL_SOURCE).unwrap(),
                    coherent.byte_at(VERTICAL_SOURCE + 1).unwrap()
                ],
                i16::MIN.to_le_bytes()
            );
        } else {
            assert_eq!(coherent.byte_at(PHASE_SOURCE), Some(u8::MAX));
        }
        assert!(coherent.verify_source(&rom).is_err());
        guards += 1;
    }
    for case in 0..9 {
        let mut value = serde_json::to_value(model).unwrap();
        match case {
            0 => value["unknown"] = true.into(),
            1 => value["phase_thresholds"]["unknown"] = true.into(),
            2 => value["phase_thresholds"]["entries"][0]["unknown"] = true.into(),
            3 => value["vertical_corrections"]["entries"][0]["unknown"] = true.into(),
            4 => value["phase_thresholds"]["entries"][0]["threshold"] = 256.into(),
            5 => value["phase_thresholds"]["entries"][0]["phase"] = 256.into(),
            6 => value["vertical_corrections"]["entries"][0]["correction"] = 32768.into(),
            7 => value["vertical_corrections"]["entries"][0]["correction"] = (-32769).into(),
            _ => value["phase_thresholds"]["source_offset"] = 0x100000000u64.into(),
        }
        assert!(serde_json::from_value::<ObservedTaskParameters>(value).is_err());
        guards += 1;
    }
    assert!(
        serde_json::from_slice::<ObservedTaskParameters>(&encoded[..encoded.len() - 1]).is_err()
    );
    guards += 1;
    for at in [
        PHASE_SOURCE as usize,
        VERTICAL_SOURCE as usize,
        0x18dda,
        0x264f6,
    ] {
        let mut bad_rom = rom.clone();
        bad_rom[at] ^= 1;
        assert!(Catalog::from_rom(&bad_rom).is_err());
        guards += 1;
    }
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    guards += 1;
    assert_eq!(guards, 27);
    for phase in [1, 2] {
        for old in [0, 7, 8, 9, u16::MAX] {
            original_phase(&rom, model, phase, old);
        }
    }
    for task_byte in 8..16 {
        original_vertical(&rom, model, task_byte);
    }
    // Publication remains unchanged; this test never creates or attaches a dataset.
    assert_eq!(
        (
            gba_core::rom_data::EXTRACTOR_REV,
            gba_core::rom_data::DECODER_REV
        ),
        (72, 71)
    );
    fs::write(root.join("decoded.json"), encoded).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "source_bytes":10,"spans":2,"phase_rows":2,"correction_rows":4,"guards":guards,"original_prefix_paths":18,
        "old_counter_threshold_boundaries_and_wrap":true,"low_bit_alias_signed_subtraction":true,"full_RAM_callee_guards":true,
        "phase0_previous_correction_owner_other_inputs_unmodeled":true,"publication_runtime_revision_unchanged":true,
        "linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false
    })).unwrap()).unwrap();
    eprintln!("ROM_DATA_OBSERVED_TASK_MODEL source_bytes=10 spans=2 guards={guards} original_prefix_paths=18 PASS");
}
