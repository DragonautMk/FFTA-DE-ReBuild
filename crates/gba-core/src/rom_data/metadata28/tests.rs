//! Original-code oracle on private inputs; synthetic calls are not gameplay fixtures.
use super::*;
use crate::{Bus, Machine};
use serde_json::{json, Value};
use std::{collections::BTreeSet, fs, path::PathBuf};
const STACK: u32 = 0x03007d00;
const RETURN: u32 = 0x08000300;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Probe {
    kind: String,
    index: u32,
    selector: u32,
    entry: u32,
    field: u32,
    width: u32,
    expected: u32,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Variant {
    label: String,
    record: Record,
    probes: Vec<Probe>,
}
fn value(raw: &[u8], field: u32, width: u32) -> u32 {
    let mut word = [0; 4];
    word[..width as usize].copy_from_slice(&raw[field as usize..(field + width) as usize]);
    u32::from_le_bytes(word)
}
fn original(m: &mut Machine, data: &Metadata28, p: &Probe, covered: &mut BTreeSet<u32>) -> usize {
    let source = SOURCE + u32::from(p.index as u16) * RECORD_BYTES as u32;
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.regs[15] = 0x08000000 + p.entry;
    m.cpu.cpsr = 0xbf;
    let mut destination = 0;
    if p.kind == "getter" {
        m.cpu.regs[0] = p.index;
        m.cpu.regs[1] = p.selector;
        if let Some(projected) = data.lookup(p.index, p.selector) {
            assert_eq!(projected.value(), p.expected);
            assert_eq!(
                matches!(projected, Lookup::SourceAddress(_)),
                p.selector as u8 == 9
            );
        } else {
            assert!(data.record(p.index).is_none());
            assert!((p.selector as u8) < 34 && p.selector as u8 != 9);
        }
    } else if p.kind == "tail" {
        let op = u16::from_le_bytes(
            m.bus.rom[p.entry as usize..p.entry as usize + 2]
                .try_into()
                .unwrap(),
        );
        let width = if op & 0xf800 == 0x8800 {
            2
        } else {
            assert_eq!(op & 0xf800, 0x7800);
            1
        };
        let base = usize::from((op >> 3) & 7);
        destination = usize::from(op & 7);
        assert_eq!(width, p.width);
        assert_eq!(u32::from((op >> 6) & 31) * width, p.field);
        m.cpu.regs[base] = 0x08000000 + source;
        assert_eq!(
            value(
                &data.record(p.index).unwrap().source_bytes(),
                p.field,
                width
            ),
            p.expected
        );
    } else {
        assert_eq!(p.kind, "inline");
        assert!(p.field < 3);
        m.cpu.regs[1] = p.field;
        m.cpu.regs[2] = data.lookup(p.index, 9).unwrap().value();
        m.cpu.regs[3] = 0x08000000 + SOURCE_END;
        assert_eq!(
            data.byte_at(source + 12 + p.field),
            Some(m.bus.rom[(source + 12 + p.field) as usize])
        );
    }
    let before = m.cpu.regs;
    let mut instructions = 0;
    let mut source_loads = Vec::new();
    let stop = if p.kind == "getter" {
        RETURN
    } else if p.kind == "inline" {
        0x080b854a
    } else {
        0x08000000 + p.entry + 2
    };
    while m.cpu.regs[15] != stop {
        let pc = m.cpu.regs[15] - 0x08000000;
        let load = if p.kind == "getter" {
            match pc {
                0xd07ca => Some((m.cpu.regs[2] + 16, 4)),
                0xd0878 => Some((m.cpu.regs[2], 2)),
                0xd087c => Some((m.cpu.regs[2] + 2, 1)),
                0xd0880..=0xd0898 if (pc - 0xd0880) % 4 == 0 => {
                    Some((m.cpu.regs[2] + 4 + (pc - 0xd0880) / 4, 1))
                }
                0xd08a2 => Some((m.cpu.regs[2] + 16, 4)),
                0xd08a6 => Some((m.cpu.regs[2] + 20, 2)),
                0xd08aa => Some((m.cpu.regs[2] + 11, 1)),
                _ => None,
            }
        } else if p.kind == "tail" {
            Some((0x08000000 + source + p.field, p.width))
        } else if pc == 0xb8542 {
            Some((m.cpu.regs[0], 1))
        } else {
            None
        };
        if let Some((address, width)) = load {
            let expected_address = source
                + if p.kind == "inline" {
                    12 + p.field
                } else {
                    p.field
                };
            assert_eq!(address, 0x08000000 + expected_address);
            assert_eq!(width, p.width);
            source_loads.push((address, width));
            for at in address - 0x08000000..address - 0x08000000 + width {
                if (SOURCE..SOURCE_END).contains(&at) {
                    covered.insert(at);
                }
            }
        }
        if p.kind == "inline" && pc == 0xb8548 {
            let index = u32::from(m.bus.rom[(source + 12 + p.field) as usize]);
            assert_eq!(m.cpu.regs[0], 0x08000000 + SOURCE_END + index * 4);
        }
        m.bus.frame_ready = false;
        m.step();
        instructions += 1;
        assert!(
            instructions < 80,
            "{} index{} selector{} pc{:x}",
            p.kind,
            p.index,
            p.selector,
            m.cpu.regs[15]
        );
    }
    assert_eq!(source_loads.len(), usize::from(p.width != 0));
    assert_eq!(m.cpu.regs[destination], p.expected);
    assert_eq!(&m.cpu.regs[4..12], &before[4..12]);
    assert_eq!(m.cpu.regs[13], STACK);
    assert_eq!(m.cpu.regs[14], RETURN | 1);
    if p.kind == "tail" {
        for i in 0..15 {
            if i != destination {
                assert_eq!(m.cpu.regs[i], before[i]);
            }
        }
        assert_eq!(m.cpu.cpsr, 0xbf);
    }
    instructions
}

#[test]
#[ignore = "private EUR metadata28 source/model original-reader oracle; Wine only, Linux/USA pending"]
fn european_metadata28_matches_source_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_METADATA28_MODEL_ROOT").unwrap());
    let out = root.join("windows-model-europe");
    assert!(!out.exists());
    fs::create_dir(&out).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let expected: Metadata28 =
        serde_json::from_slice(&fs::read(root.join("independent-model.json")).unwrap()).unwrap();
    assert_eq!(data, &expected);
    data.verify_source(&rom).unwrap();
    assert_eq!(
        data.source_bytes(),
        rom[SOURCE as usize..SOURCE_END as usize]
    );
    let sources: Vec<SourceRange> =
        serde_json::from_slice(&fs::read(root.join("independent-sources.json")).unwrap()).unwrap();
    assert_eq!(catalog.sources, sources);
    let spans = data.source_spans().unwrap();
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].0, SOURCE);
    assert_eq!(spans[0].1, data.source_bytes());
    assert_eq!(spans[0].2, catalog.sources[0].role);
    assert_eq!(ffta_rom::digest(&spans[0].1), SOURCE_SHA256);
    for (i, &byte) in spans[0].1.iter().enumerate() {
        assert_eq!(data.byte_at(SOURCE + i as u32), Some(byte));
    }
    for at in [0, SOURCE - 1, SOURCE_END, SOURCE_END + 1, u32::MAX] {
        assert_eq!(data.byte_at(at), None);
    }
    for i in [347, 65535, u32::MAX] {
        assert!(data.record(i).is_none());
        assert_eq!(data.lookup(i, 0), None);
        assert!(matches!(data.lookup(i, 9), Some(Lookup::SourceAddress(_))));
        assert_eq!(data.lookup(i, 255), Some(Lookup::Scalar(0)));
    }
    assert_eq!(data.record(0x10000).unwrap().index, 0);
    assert_eq!(data.record(0x8000015a).unwrap().index, 346);
    assert_eq!(data.records[0].opaque03, 1);
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<Metadata28>(&encoded).unwrap(),
        *data
    );
    fs::write(out.join("decoded.json"), &encoded).unwrap();
    let mut guards = 0;
    for case in 0..5 {
        let mut bad = data.clone();
        match case {
            0 => bad.source_offset += 28,
            1 => {
                bad.records.pop();
            }
            2 => bad.records.push(bad.records[0].clone()),
            3 => bad.records.swap(0, 1),
            _ => bad.records[1].index = 0,
        }
        assert!(bad.validate().is_err());
        assert!(bad.source_spans().is_err());
        guards += 1;
    }
    for case in 0..10 {
        let mut bad: Value = serde_json::from_slice(&encoded).unwrap();
        match case {
            0 => bad["extra"] = json!(1),
            1 => bad["records"][0]["extra"] = json!(1),
            2 => bad["records"][0]["fields04_0b"] = json!([0, 0, 0, 0, 0, 0, 0]),
            3 => bad["records"][0]["inline0c_0f"] = json!([0, 0, 0]),
            4 => bad["records"][0]["field00"] = json!(65536),
            5 => bad["records"][0]["field02"] = json!(256),
            6 => bad["records"][0]["flags10"] = json!(4294967296u64),
            7 => bad["records"][0]["index"] = json!(-1),
            8 => {
                bad["records"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("opaque03");
            }
            _ => bad["records"][0]["field18"] = json!("1"),
        }
        assert!(serde_json::from_value::<Metadata28>(bad).is_err());
        guards += 1;
    }
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    guards += 1;
    let mut wrong = rom.clone();
    wrong[SOURCE as usize] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    guards += 1;
    wrong.copy_from_slice(&rom);
    wrong[0xd07a4] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    guards += 1;
    let probes: Vec<Probe> =
        serde_json::from_slice(&fs::read(root.join("independent-probes.json")).unwrap()).unwrap();
    let coverage: BTreeSet<u32> =
        serde_json::from_slice(&fs::read(root.join("independent-coverage.json")).unwrap()).unwrap();
    let variants: Vec<Variant> =
        serde_json::from_slice(&fs::read(root.join("synthetic-variants.json")).unwrap()).unwrap();
    let mut m = Machine::new(rom.clone());
    m.bus.ffta_sprites.enabled = false;
    m.bus.ewram.fill(0x5a);
    m.bus.iwram.fill(0x3c);
    m.bus.vram.fill(0xa5);
    m.bus.oam.fill(0xc3);
    m.bus.palette.fill(0x24);
    m.bus.write32(STACK - 4, RETURN | 1);
    let (ew, iw, vr, oam, palette) = (
        m.bus.ewram.clone(),
        m.bus.iwram.clone(),
        m.bus.vram.clone(),
        m.bus.oam.clone(),
        m.bus.palette.clone(),
    );
    let mut observed = BTreeSet::new();
    let mut instructions = 0;
    for p in &probes {
        instructions += original(&mut m, data, p, &mut observed);
    }
    assert_eq!(observed, coverage);
    assert_eq!(observed.len(), 8675);
    let mut synthetic_prefixes = 0;
    for variant in &variants {
        let mut changed = data.clone();
        changed.records[0] = variant.record.clone();
        changed.validate().unwrap();
        assert_ne!(changed.records[0], data.records[0], "{}", variant.label);
        assert!(changed.verify_source(&rom).is_err());
        guards += 1;
        m.bus.rom[SOURCE as usize..SOURCE as usize + 28]
            .copy_from_slice(&changed.records[0].source_bytes());
        for p in &variant.probes {
            instructions += original(&mut m, &changed, p, &mut BTreeSet::new());
            synthetic_prefixes += 1;
        }
        m.bus.rom[SOURCE as usize..SOURCE as usize + 28]
            .copy_from_slice(&rom[SOURCE as usize..SOURCE as usize + 28]);
    }
    assert_eq!(m.bus.ewram, ew);
    assert_eq!(m.bus.iwram, iw);
    assert_eq!(m.bus.vram, vr);
    assert_eq!(m.bus.oam, oam);
    assert_eq!(m.bus.palette, palette);
    assert_eq!(m.bus.rom, rom);
    let report = json!({"status":"PASS-source-model-original-readers","records":347,"source_bytes":9716,"source_spans":1,"original_prefixes":probes.len(),"getter_calls":probes.iter().filter(|p|p.kind=="getter").count(),"tail_loads":probes.iter().filter(|p|p.kind=="tail").count(),"inline_successor_prefixes":probes.iter().filter(|p|p.kind=="inline").count(),"actual_source_bytes_original_observed":observed.len(),"opaque_bytes_roundtrip_only":1041,"synthetic_variants":variants.len(),"synthetic_prefixes":synthetic_prefixes,"guards":guards,"instructions_coverage_only":instructions,"RAM_callee_stack_VRAM_OAM_palette_ROM_checked":true,"scope":"fullD07A4getter plusdirecttail/inline-successorprefixes; not fullcallerABI/nativeclock/ordinary/runtime","schema_extractor_decoder_unchanged":[1,82,81],"Linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false});
    fs::write(
        out.join("result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_METADATA28_MODEL records347/full9716B/original8675B guards{guards} original{} synthetic{synthetic_prefixes} PASS",probes.len());
}
