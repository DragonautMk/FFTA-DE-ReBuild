//! One reduced original-source check; does not reopen audio or earlier graphics acceptance.
use super::*;
use gba_core::{rom_data::palette_effect_dispatch::*, Machine};
use serde_json::Value;
use std::collections::BTreeSet;
const OBJECT: u32 = 0x02010000;
fn reset(m: &mut Machine, pc: u32) {
    m.bus.ewram.fill(0x5a);
    m.bus.iwram.fill(0xa5);
    m.cpu.regs = [0; 16];
    for n in 1..13 {
        m.cpu.regs[n] = 0x11110000 + n as u32;
    }
    m.cpu.regs[4] = OBJECT;
    m.cpu.regs[7] = OBJECT;
    m.cpu.regs[13] = 0x03007e00;
    m.cpu.regs[14] = 0x08000301;
    m.cpu.regs[15] = pc;
    m.cpu.cpsr = 0xbf;
    m.bus.reg_ie = 0;
    m.bus.ime = false;
}
fn until(m: &mut Machine, end: u32, pcs: &mut BTreeSet<u32>, reads: &mut BTreeSet<u32>) -> usize {
    let mut n = 0;
    while m.cpu.regs[15] != end {
        assert!(n < 10, "pc{:08x}", m.cpu.regs[15]);
        pcs.insert(m.cpu.regs[15]);
        if m.cpu.regs[15] == 0x0814ac70 {
            let at = m.cpu.regs[1] - 0x08000000;
            for i in 0..4 {
                reads.insert(at + i);
            }
        }
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    n
}
#[test]
#[ignore = "private EUR palette callback projection decoder/original reader and producer prefixes; Wine only, Linux/USA pending"]
fn european_palette_effect_dispatch_model_matches_original_reader_and_producers() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_PALETTE_DISPATCH_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let evidence: Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_PALETTE_DISPATCH_EVIDENCE").unwrap()).unwrap(),
    )
    .unwrap();
    let e = &evidence["dispatch"];
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let cat = Catalog::from_rom(&rom).unwrap();
    let data = &cat.decoded;
    let words = e["words"].as_array().unwrap();
    assert_eq!(data.slots.len(), 17);
    assert_eq!(data.source_offset, e["offset"].as_u64().unwrap() as u32);
    for (kind, w) in words.iter().enumerate() {
        assert_eq!(data.slots[kind].kind, kind as u16);
        assert_eq!(
            data.callback_word(kind as u16),
            Some(w.as_u64().unwrap() as u32)
        );
    }
    assert_eq!(data.callback_word(17), None);
    assert_eq!(data.callback_word(65535), None);
    assert_eq!(data.byte(68), None);
    assert_eq!(data.byte(usize::MAX), None);
    let spans = data.source_spans().unwrap();
    assert_eq!(spans.len(), 1);
    let (at, bytes, role) = &spans[0];
    assert_eq!((*at, bytes.len()), (SOURCE_OFFSET, 68));
    assert_eq!(bytes, &rom[*at as usize..*at as usize + 68]);
    assert_eq!(
        gba_core::ffta_rom::digest(bytes),
        e["source_sha256"].as_str().unwrap()
    );
    assert_eq!(role, "graphics/palette-effect-dispatch/projection-0-16");
    assert_eq!(cat.sources.len(), 1);
    assert_eq!(cat.sources[0].sha256, e["source_sha256"].as_str().unwrap());
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<PaletteEffectDispatch>(&encoded).unwrap(),
        *data
    );
    for case in 0..8 {
        let mut bad = data.clone();
        match case {
            0 => bad.source_offset += 4,
            1 => {
                bad.slots.pop();
            }
            2 => bad.slots[1].kind = 0,
            3 => bad.slots[0].target = bad.slots[1].target.clone(),
            4 => bad.slots[1].target = None,
            5 => bad.slots[1].target.as_mut().unwrap().rom_offset |= 1,
            6 => bad.slots[1].target.as_mut().unwrap().rom_offset = 0x1000000,
            _ => bad.slots.swap(1, 2),
        }
        assert!(bad.validate().is_err(), "shape guard {case}");
    }
    let mut bad = data.clone();
    bad.slots[1].target.as_mut().unwrap().rom_offset += 2;
    bad.validate().unwrap();
    assert!(bad.verify_source(&rom).is_err());
    let mut unknown = serde_json::to_value(data).unwrap();
    unknown["unknown"] = true.into();
    assert!(serde_json::from_value::<PaletteEffectDispatch>(unknown).is_err());
    assert!(serde_json::from_str::<ThumbEntry>("{\"rom_offset\":0,\"unknown\":true}").is_err());
    assert!(serde_json::from_str::<Slot>("{\"kind\":65536,\"target\":null}").is_err());
    assert!(Catalog::from_rom(&[]).is_err());
    let mut m = Machine::new(rom.clone());
    let mut pcs = BTreeSet::new();
    let mut reads = BTreeSet::new();
    let mut instructions = 0;
    let kinds: Vec<u32> = (0..17)
        .chain([17, 31, 255, 65535, 0x12340001, 0xffff0010])
        .collect();
    for raw in &kinds {
        reset(&mut m, 0x0814ac68);
        m.bus.ewram[0x10004..0x10008].copy_from_slice(&raw.to_le_bytes());
        let ew = m.bus.ewram.clone();
        let iw = m.bus.iwram.clone();
        let regs = m.cpu.regs;
        instructions += until(&mut m, 0x0814ac72, &mut pcs, &mut reads);
        let kind = *raw as u16;
        let at = SOURCE_OFFSET as usize + usize::from(kind) * 4;
        let expected = u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        assert_eq!(m.cpu.regs[0], 0x08000000 + SOURCE_OFFSET);
        assert_eq!(m.cpu.regs[1], expected, "live u16 {raw:x}");
        if kind < 17 {
            assert_eq!(data.callback_word(kind), Some(expected));
        } else {
            assert_eq!(data.callback_word(kind), None);
        }
        assert_eq!(m.bus.ewram, ew);
        assert_eq!(m.bus.iwram, iw);
        assert_eq!(m.cpu.regs[2..15], regs[2..15]);
    }
    for state in [0u16, 2, 65535] {
        reset(&mut m, 0x0814ac62);
        m.bus.ewram[0x10000..0x10002].copy_from_slice(&state.to_le_bytes());
        let ew = m.bus.ewram.clone();
        let iw = m.bus.iwram.clone();
        let before = reads.clone();
        let regs = m.cpu.regs;
        instructions += until(&mut m, 0x0814ac88, &mut pcs, &mut reads);
        assert_eq!(reads, before);
        assert_eq!(m.cpu.regs[0], u32::from(state));
        assert_eq!(m.cpu.regs[1..15], regs[1..15]);
        assert_eq!(m.bus.ewram, ew);
        assert_eq!(m.bus.iwram, iw);
    }
    for (kind, pc, _, store, _) in PRODUCERS {
        reset(&mut m, 0x08000000 + pc);
        let mut ew = m.bus.ewram.clone();
        ew[0x10004..0x10006].copy_from_slice(&kind.to_le_bytes());
        let iw = m.bus.iwram.clone();
        let regs = m.cpu.regs;
        instructions += until(&mut m, 0x08000000 + store + 2, &mut pcs, &mut reads);
        assert_eq!(m.cpu.regs[0], u32::from(kind));
        assert_eq!(m.cpu.regs[1..15], regs[1..15]);
        assert_eq!(m.bus.ewram, ew);
        assert_eq!(m.bus.iwram, iw);
    }
    let owned: Vec<_> = reads
        .iter()
        .copied()
        .filter(|a| (SOURCE_OFFSET..SOURCE_OFFSET + 68).contains(a))
        .collect();
    assert_eq!(
        owned,
        (SOURCE_OFFSET..SOURCE_OFFSET + 68).collect::<Vec<_>>()
    );
    fs::write(root.join("palette-effect-dispatch.json"), encoded).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"source_bytes":68,"slots":17,"active_kinds":[1,2,4,8,16],"null_slots":12,"original_lookup_prefixes":kinds.len(),"original_state_gates":3,"original_constant_store_prefixes":5,"owned_distinct_bytes_at_original_loadsite":owned.len(),"raw_outside_indices":[17,31,255,65535],"low16_live_RAM_preserved":true,"ram_sp_callee_unchanged_except_declared2byte_producer_store":true,"lossless_fields_hash_json_and12shape_json_source_guards":true,"instructions_coverage":instructions,"pcs":pcs.len(),"scope":"lookup/state/constant-store prefixes, not fullcallbacks/allocators/native/stateclock/ordinary","managed":"e44-d43 unchanged; USAe8-d7","linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
}
