//! One combined EUR source/field/dispatch/default oracle; no host sequencing.
use super::*;
use gba_core::{rom_data::effect_event_directories::*, Machine};
use serde_json::Value;
use std::collections::BTreeSet;
const CONTEXT: u32 = 0x02010000;
fn reset(m: &mut Machine, pc: u32) {
    m.bus.ewram.fill(0x5a);
    m.bus.iwram.fill(0xa5);
    m.cpu.regs = [0; 16];
    for n in 1..13 {
        m.cpu.regs[n] = 0x11110000 + n as u32;
    }
    m.cpu.regs[7] = CONTEXT;
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
        let pc = m.cpu.regs[15];
        assert!(n < 40, "pc{pc:08x} end{end:08x}");
        pcs.insert(pc);
        let access = match pc {
            0x08124ec0 => Some((m.cpu.regs[0] + 2, 2)),
            0x08124eca => Some((m.cpu.regs[0], 1)),
            0x08124ed4 => Some((m.cpu.regs[0], 4)),
            0x08124edc => Some((m.cpu.regs[1], 4)),
            0x081253d6 => Some((m.cpu.regs[0] + 4, 1)),
            0x08124eee | 0x08128b3a => Some((m.cpu.regs[0], 4)),
            0x08128b2c => Some((m.cpu.regs[5], 1)),
            0x08128b6e => Some((m.cpu.regs[5] + 1, 1)),
            0x08128bc8 => Some((m.cpu.regs[5] + 2, 1)),
            0x08128e70 => Some((m.cpu.regs[5] + 3, 1)),
            _ => None,
        };
        if let Some((a, width)) = access {
            for i in 0..width {
                reads.insert(a - 0x08000000 + i);
            }
        }
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    n
}
fn half(r: &[u8], at: u32) -> u16 {
    u16::from_le_bytes(r[at as usize..at as usize + 2].try_into().unwrap())
}
fn word(r: &[u8], at: u32) -> u32 {
    u32::from_le_bytes(r[at as usize..at as usize + 4].try_into().unwrap())
}
#[test]
#[ignore = "private EUR effect event directory source/original-field/even-dispatch/default oracle; Wine only, Linux/USA pending"]
fn european_effect_event_directories_match_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_EFFECT_EVENT_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let evidence: Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_EFFECT_EVENT_EVIDENCE").unwrap()).unwrap(),
    )
    .unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let spans = data.source_spans().unwrap();
    assert_eq!(spans.len(), 4);
    for ((at, bytes, role), e) in spans
        .iter()
        .zip(evidence["source_ranges"].as_array().unwrap())
    {
        assert_eq!(*at, e["offset"].as_u64().unwrap() as u32);
        assert_eq!(bytes.len(), e["length"].as_u64().unwrap() as usize);
        assert_eq!(role, e["role"].as_str().unwrap());
        assert_eq!(
            gba_core::ffta_rom::digest(bytes),
            e["sha256"].as_str().unwrap()
        );
        assert_eq!(bytes, &rom[*at as usize..*at as usize + bytes.len()]);
    }
    assert_eq!(spans.iter().map(|s| s.1.len()).sum::<usize>(), SOURCE_BYTES);
    for (e, raw) in data
        .events
        .iter()
        .zip(evidence["events"].as_array().unwrap())
    {
        assert_eq!(e.index, raw["index"].as_u64().unwrap() as u16);
        assert_eq!(e.opcode, raw["opcode"].as_u64().unwrap() as u8);
        assert_eq!(e.byte1_raw, raw["byte1_raw"].as_u64().unwrap() as u8);
        assert_eq!(e.id_raw, raw["id_raw"].as_u64().unwrap() as u16);
        assert_eq!(e.post_delay, raw["post_delay"].as_u64().unwrap() as u8);
        assert_eq!(e.byte5_raw, raw["byte5_raw"].as_u64().unwrap() as u8);
        assert_eq!(e.half6_raw, raw["half6_raw"].as_u64().unwrap() as u16);
        assert_eq!(e.arg8_raw, raw["arg8_raw"].as_u64().unwrap() as u32);
        assert_eq!(e.arg12_raw, raw["arg12_raw"].as_u64().unwrap() as u32);
        assert_eq!(e.byte(16), None);
    }
    for (s, raw) in data
        .selectors
        .iter()
        .zip(evidence["selectors"].as_array().unwrap())
    {
        assert_eq!(s.index, raw["index"].as_u64().unwrap() as u16);
        assert_eq!(s.kind, raw["kind"].as_u64().unwrap() as u8);
        assert_eq!(
            s.parameters,
            [
                raw["first_raw"].as_u64().unwrap() as u8,
                raw["second_raw"].as_u64().unwrap() as u8,
                raw["third_raw"].as_u64().unwrap() as u8
            ]
        );
        assert_eq!(s.script_descriptor, s.index);
        assert_eq!(s.byte(4), None);
    }
    for (d, e) in [&data.event_dispatch, &data.selector_dispatch]
        .into_iter()
        .zip(evidence["dispatch_tables"].as_array().unwrap())
    {
        assert_eq!(d.source_offset, e["source_offset"].as_u64().unwrap() as u32);
        for (slot, entry) in d.slots.iter().zip(e["entries"].as_array().unwrap()) {
            assert_eq!(
                slot.target.rom_offset,
                entry["rom_offset"].as_u64().unwrap() as u32
            );
            assert_eq!(slot.selector, entry["index"].as_u64().unwrap() as u8);
        }
    }
    let tasks = gba_core::rom_data::task_descriptors::Catalog::from_rom(&rom)
        .unwrap()
        .decoded;
    let mut refs = 0;
    for e in &data.events {
        if let Some(i) = e.script_descriptor {
            assert_eq!(e.id_raw, 363 + i);
            assert!(tasks.script.entries.get(usize::from(i)).is_some());
            refs += 1;
        }
    }
    assert_eq!(refs, 19);
    for s in &data.selectors {
        assert!(tasks
            .script
            .entries
            .get(usize::from(s.script_descriptor))
            .is_some());
    }
    for (a, bytes, _) in &spans {
        assert!(data.byte_at(a - 1).is_none());
        assert!(data.byte_at(a + bytes.len() as u32).is_none());
    }
    assert_eq!(data.byte_at(u32::MAX), None);
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<EffectEventDirectories>(&encoded).unwrap(),
        *data
    );
    for case in 0..12 {
        let mut bad = data.clone();
        match case {
            0 => bad.event_source_offset += 16,
            1 => bad.selector_source_offset += 4,
            2 => {
                bad.events.pop();
            }
            3 => bad.selectors.push(bad.selectors[0].clone()),
            4 => bad.events.swap(0, 1),
            5 => bad.events[0].index = 1,
            6 => {
                bad.events
                    .iter_mut()
                    .find(|e| e.opcode == 0)
                    .unwrap()
                    .script_descriptor = Some(0)
            }
            7 => bad.selectors[0].script_descriptor = 1,
            8 => bad.event_dispatch.source_offset -= 4,
            9 => {
                bad.selector_dispatch.slots.pop();
            }
            10 => bad.event_dispatch.slots[0].target.rom_offset |= 1,
            _ => bad.selector_dispatch.slots[0].target.rom_offset = 0x1000000,
        }
        assert!(bad.validate().is_err(), "shape{case}");
    }
    for case in 0..4 {
        let mut bad = data.clone();
        match case {
            0 => bad.events[0].arg8_raw ^= 1,
            1 => bad.events[0].byte1_raw = 1,
            2 => bad.selectors[0].kind = 6,
            _ => bad.selector_dispatch.slots[0].target.rom_offset += 2,
        }
        bad.validate().unwrap();
        assert!(bad.verify_source(&rom).is_err(), "source{case}");
    }
    for path in ["top", "event", "selector", "dispatch", "slot", "target"] {
        let mut bad = serde_json::to_value(data).unwrap();
        let node = match path {
            "top" => &mut bad,
            "event" => &mut bad["events"][0],
            "selector" => &mut bad["selectors"][0],
            "dispatch" => &mut bad["event_dispatch"],
            "slot" => &mut bad["event_dispatch"]["slots"][0],
            _ => &mut bad["event_dispatch"]["slots"][0]["target"],
        };
        node["unknown"] = true.into();
        assert!(serde_json::from_value::<EffectEventDirectories>(bad).is_err());
    }
    assert!(
        serde_json::from_slice::<EffectEventDirectories>(&encoded[..encoded.len() - 1]).is_err()
    );
    let mut bad = serde_json::to_value(data).unwrap();
    bad["selectors"][0]["parameters"] = serde_json::json!([0, 0, 0, 0]);
    assert!(serde_json::from_value::<EffectEventDirectories>(bad).is_err());
    let mut bad = serde_json::to_value(data).unwrap();
    bad["events"][0]["id_raw"] = 65536.into();
    assert!(serde_json::from_value::<EffectEventDirectories>(bad).is_err());
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    let mut wrong = rom.clone();
    wrong[0] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    let mut m = Machine::new(rom.clone());
    let mut pcs = BTreeSet::new();
    let mut reads = BTreeSet::new();
    let mut instructions = 0;
    for index in (0u16..75).chain([75, 65535]) {
        let at = event_source_offset(index);
        reset(&mut m, 0x08124eb8);
        m.bus.ewram[0x10000..0x10002].copy_from_slice(&index.to_le_bytes());
        let mut ew = m.bus.ewram.clone();
        let iw = m.bus.iwram.clone();
        let regs = m.cpu.regs;
        ew[0x10008..0x1000a].copy_from_slice(&half(&rom, at + 2).to_le_bytes());
        instructions += until(&mut m, 0x08124ee0, &mut pcs, &mut reads);
        assert_eq!(m.cpu.regs[5], word(&rom, at + 8));
        assert_eq!(m.cpu.regs[6], word(&rom, at + 12));
        assert_eq!(m.cpu.regs[12], u32::from(rom[at as usize]));
        assert_eq!(m.cpu.regs[7..12], regs[7..12]);
        assert_eq!(m.cpu.regs[13..15], regs[13..15]);
        assert_eq!(m.bus.ewram, ew);
        assert_eq!(m.bus.iwram, iw);
        reset(&mut m, 0x081253ce);
        m.bus.ewram[0x10000..0x10002].copy_from_slice(&index.to_le_bytes());
        let mut ew = m.bus.ewram.clone();
        let iw = m.bus.iwram.clone();
        let regs = m.cpu.regs;
        ew[0x10002..0x10004].copy_from_slice(&u16::from(rom[at as usize + 4]).to_le_bytes());
        instructions += until(&mut m, 0x081253da, &mut pcs, &mut reads);
        assert_eq!(m.cpu.regs[0], u32::from(rom[at as usize + 4]));
        assert_eq!(m.cpu.regs[2..15], regs[2..15]);
        assert_eq!(m.bus.ewram, ew);
        assert_eq!(m.bus.iwram, iw);
        if index >= 75 {
            assert_eq!(data.byte_at(at), None);
        }
    }
    let mut parameter_probes = 0;
    for index in (0u32..51).chain([51, 65535]) {
        let at = selector_source_offset(index);
        reset(&mut m, 0x08128b08);
        m.cpu.regs[0] = index;
        let ew = m.bus.ewram.clone();
        let iw = m.bus.iwram.clone();
        let regs = m.cpu.regs;
        instructions += until(&mut m, 0x08128b0e, &mut pcs, &mut reads);
        assert_eq!(m.cpu.regs[5], 0x08000000 + at);
        assert_eq!(m.cpu.regs[2..5], regs[2..5]);
        assert_eq!(m.cpu.regs[6..15], regs[6..15]);
        assert_eq!(m.bus.ewram, ew);
        assert_eq!(m.bus.iwram, iw);
        m.cpu.regs[15] = 0x08128b2c;
        instructions += until(&mut m, 0x08128b2e, &mut pcs, &mut reads);
        assert_eq!(m.cpu.regs[0], u32::from(rom[at as usize]));
        if index < 51 {
            let s = &data.selectors[index as usize];
            let count = match s.kind {
                1 | 3 => 2,
                2 => 1,
                4 => 3,
                _ => 0,
            };
            for n in 0..count {
                let (pc, dest) = if n == 0 {
                    (0x08128b6e, 0)
                } else if n == 1 {
                    (0x08128bc8, 0)
                } else {
                    (0x08128e70, 1)
                };
                m.cpu.regs[15] = pc;
                instructions += until(&mut m, pc + 2, &mut pcs, &mut reads);
                assert_eq!(m.cpu.regs[dest], u32::from(s.parameters[n]));
                parameter_probes += 1;
            }
        } else {
            assert_eq!(data.byte_at(at), None);
        }
        assert_eq!(m.bus.ewram, ew);
        assert_eq!(m.bus.iwram, iw);
    }
    for opcode in (0u32..22).chain([22, 255]) {
        reset(&mut m, 0x08124ee0);
        m.cpu.regs[12] = opcode;
        let ew = m.bus.ewram.clone();
        let iw = m.bus.iwram.clone();
        let end = if opcode < 22 {
            data.event_dispatch.slots[opcode as usize]
                .target
                .guest_word()
                .unwrap()
        } else {
            0x08125398
        };
        instructions += until(&mut m, end, &mut pcs, &mut reads);
        assert_eq!(m.cpu.cpsr & 0x20, 0x20);
        assert_eq!(m.bus.ewram, ew);
        assert_eq!(m.bus.iwram, iw);
    }
    for kind in (0u32..6).chain([6, 255]) {
        reset(&mut m, 0x08128b2e);
        m.cpu.regs[0] = kind;
        let ew = m.bus.ewram.clone();
        let iw = m.bus.iwram.clone();
        let end = if kind < 6 {
            data.selector_dispatch.slots[kind as usize]
                .target
                .guest_word()
                .unwrap()
        } else {
            0x08128eda
        };
        instructions += until(&mut m, end, &mut pcs, &mut reads);
        assert_eq!(m.cpu.cpsr & 0x20, 0x20);
        assert_eq!(m.bus.ewram, ew);
        assert_eq!(m.bus.iwram, iw);
    }
    for q in [0u8, 1, 51, 52, 255] {
        reset(&mut m, 0x0812746e);
        m.cpu.regs[1] = CONTEXT;
        m.bus.ewram[0x10001] = q;
        let ew = m.bus.ewram.clone();
        let iw = m.bus.iwram.clone();
        instructions += until(&mut m, 0x08127476, &mut pcs, &mut reads);
        assert_eq!(m.cpu.regs[0], u32::from(u16::from(q).wrapping_sub(1)));
        assert_eq!(m.bus.ewram, ew);
        assert_eq!(m.bus.iwram, iw);
    }
    for raw in [0u32, 50, 51, 65535, 0xbeef0001] {
        reset(&mut m, 0x08128b00);
        m.cpu.regs[0] = raw;
        let mut iw = m.bus.iwram.clone();
        iw[0x7e00..0x7e04].copy_from_slice(&u32::from(raw as u16).to_le_bytes());
        let ew = m.bus.ewram.clone();
        instructions += until(&mut m, 0x08128b06, &mut pcs, &mut reads);
        assert_eq!(m.cpu.regs[0], u32::from(raw as u16));
        assert_eq!(m.bus.ewram, ew);
        assert_eq!(m.bus.iwram, iw);
        reset(&mut m, 0x08128b20);
        m.bus.iwram[0x7e00..0x7e04].copy_from_slice(&u32::from(raw as u16).to_le_bytes());
        let mut iw = m.bus.iwram.clone();
        iw[0x7e00..0x7e04].copy_from_slice(&u32::from(secondary_task_id(raw)).to_le_bytes());
        let ew = m.bus.ewram.clone();
        instructions += until(&mut m, 0x08128b2c, &mut pcs, &mut reads);
        assert_eq!(m.cpu.regs[0], u32::from(secondary_task_id(raw)));
        assert_eq!(m.bus.ewram, ew);
        assert_eq!(m.bus.iwram, iw);
    }
    for index in [0u16, 73, 74, 65535] {
        reset(&mut m, 0x081253da);
        m.bus.ewram[0x10000..0x10002].copy_from_slice(&index.to_le_bytes());
        let next = index.wrapping_add(1);
        let end = if next <= 74 { 0x081253e8 } else { 0x081253f4 };
        let mut ew = m.bus.ewram.clone();
        ew[0x10000..0x10002].copy_from_slice(&next.to_le_bytes());
        let iw = m.bus.iwram.clone();
        instructions += until(&mut m, end, &mut pcs, &mut reads);
        assert_eq!(m.cpu.regs[0], u32::from(next));
        assert_eq!(m.bus.ewram, ew);
        assert_eq!(m.bus.iwram, iw);
    }
    let owned: BTreeSet<_> = reads
        .iter()
        .copied()
        .filter(|a| data.byte_at(*a).is_some())
        .collect();
    let mut expected = BTreeSet::new();
    for e in &data.events {
        let at = event_source_offset(e.index);
        for n in [0, 2, 3, 4, 8, 9, 10, 11, 12, 13, 14, 15] {
            expected.insert(at + n);
        }
    }
    for s in &data.selectors {
        let at = selector_source_offset(u32::from(s.index));
        expected.insert(at);
        let count = match s.kind {
            1 | 3 => 2,
            2 => 1,
            4 => 3,
            _ => 0,
        };
        for n in 0..count {
            expected.insert(at + 1 + n);
        }
    }
    for d in [&data.event_dispatch, &data.selector_dispatch] {
        expected.extend(d.source_offset..d.source_offset + 4 * d.slots.len() as u32);
    }
    assert_eq!(owned, expected);
    assert_eq!(owned.len(), 1106);
    assert_eq!(parameter_probes, 43);
    fs::write(root.join("effect-event-directories.json"), encoded).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"events":75,"selectors":51,"dispatch_words":28,"source_bytes":1516,"source_ranges":4,"source_hashes_roundtrip_fields_references":true,"original_event_header_prefixes":77,"original_event_delay_prefixes":77,"original_selector_address_kind_prefixes":53,"original_kind_gated_parameter_load_prefixes":parameter_probes,"original_dispatch_cases":32,"original_frontdoor_low16_taskID_and_increment_prefixes":19,"owned_distinct_bytes_at_original_loadsites":owned.len(),"event_reserved_bytes_preserved_unread":300,"selector_parameter_bytes_preserved_unread":110,"guards":{"shape":12,"source_edit":4,"json":9,"rom_identity":2},"instructions_coverage":instructions,"pcs":pcs.len(),"raw_event75_65535_selector51_65535_q0_and_wrap":true,"even_dispatch_preserves_thumb_and_unknown_default":true,"scope":"controlled reader/field/dispatch/index prefixes; not whole event interpreter/allocator/native/stateclock/ordinary ABI or event function reachability","managed":"e48-d47 unchanged; USAe8-d7","linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
}
