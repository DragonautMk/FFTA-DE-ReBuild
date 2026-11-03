//! Isolated original-code probes, never ordinary gameplay or save fixtures.
use super::*;
use crate::{Bus, Machine};
use serde_json::{json, Value};
use std::{collections::BTreeSet, fs, path::PathBuf};

const GLOBAL: u32 = 0x0200f3f0;
const INSTANCE: u32 = 0x02018000;
const INLINE: u32 = 0x02010000;
const STACK: u32 = 0x03007d00;
const RETURN: u32 = 0x08000300;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Probe {
    kind: String,
    index: u8,
    expected: [u8; 4],
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Variant {
    label: String,
    record: Record,
    probes: Vec<Probe>,
}

fn cpu(m: &mut Machine, entry: u32) {
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.regs[15] = 0x08000000 + entry;
    m.cpu.cpsr = 0xbf;
}
fn step_to(m: &mut Machine, stop: u32) -> usize {
    let mut instructions = 0;
    while m.cpu.regs[15] != stop {
        m.bus.frame_ready = false;
        m.step();
        instructions += 1;
        assert!(
            instructions < 24,
            "unexpected original PC {:x}",
            m.cpu.regs[15]
        );
    }
    instructions
}
fn callee(m: &Machine, before: &[u32; 16]) {
    assert_eq!(&m.cpu.regs[4..14], &before[4..14]);
    assert_eq!(m.cpu.regs[14], RETURN | 1);
}

fn original_field(
    m: &mut Machine,
    data: &Records4,
    p: &Probe,
    covered: &mut BTreeSet<u32>,
) -> usize {
    let address = source_address(p.index);
    assert_eq!(data.record(p.index).unwrap().source_bytes(), p.expected);
    m.bus.write32(GLOBAL + 0x30, address);
    m.bus.write32(INSTANCE + 0x30, address);
    let (entry, stop, fields): (u32, u32, &[usize]) = match p.kind.as_str() {
        "global00" => (0x134cb8, RETURN, &[0]),
        "instance00" => (0x134cd4, RETURN, &[0]),
        "field01" => (0x134dae, 0x08134db2, &[1]),
        "global02" => (0x135108, RETURN, &[2]),
        "fields03_02" => (0x134e22, 0x08134e28, &[3, 2]),
        _ => panic!("unknown original reader {}", p.kind),
    };
    cpu(m, entry);
    m.cpu.regs[0] = INSTANCE;
    let before = m.cpu.regs;
    let ewram = m.bus.ewram.clone();
    // Observe actual effective source addresses at the original LDRB instructions.
    let mut observed = Vec::new();
    let mut instructions = 0;
    while m.cpu.regs[15] != stop {
        let pc = m.cpu.regs[15] - 0x08000000;
        let field = match pc {
            0x134cbc | 0x134cd6 => Some(0),
            0x134db0 => Some(1),
            0x13510c | 0x134e26 => Some(2),
            0x134e24 => Some(3),
            _ => None,
        };
        if let Some(field) = field {
            assert_eq!(m.cpu.regs[0], address);
            let at = address - 0x08000000 + field as u32;
            assert_eq!(m.bus.rom[at as usize], p.expected[field]);
            assert!(data.byte_at(at).is_some());
            covered.insert(at);
            observed.push(field);
        }
        m.bus.frame_ready = false;
        m.step();
        instructions += 1;
        assert!(instructions < 12);
    }
    assert_eq!(observed, fields);
    match p.kind.as_str() {
        "field01" => assert_eq!(m.cpu.regs[2], u32::from(p.expected[1])),
        "fields03_02" => {
            assert_eq!(m.cpu.regs[2], u32::from(p.expected[3]));
            assert_eq!(m.cpu.regs[3], u32::from(p.expected[2]));
        }
        "global02" => assert_eq!(m.cpu.regs[0], u32::from(p.expected[2])),
        _ => assert_eq!(m.cpu.regs[0], u32::from(p.expected[0])),
    }
    callee(m, &before);
    assert_eq!(m.cpu.cpsr, 0xbf);
    assert_eq!(m.bus.ewram, ewram);
    instructions
}

fn original_setter(m: &mut Machine, index: u8, instance: bool) -> usize {
    let context = if instance { INSTANCE } else { GLOBAL };
    m.bus.write32(context + 0x2c, INLINE);
    m.bus.write32(context + 0x30, 0xdeadbeef);
    m.bus.write8(INLINE + u32::from(index), index);
    cpu(m, if instance { 0x133038 } else { 0x132fb0 });
    if instance {
        m.cpu.regs[0] = INSTANCE;
        m.cpu.regs[1] = 0xfedc0000 | u32::from(index);
    } else {
        m.cpu.regs[0] = 0x12340000 | u32::from(index);
    }
    let before = m.cpu.regs;
    let mut expected_ram = m.bus.ewram.clone();
    let base = (context - 0x02000000) as usize;
    expected_ram[base + 0x28] = index;
    expected_ram[base + 0x30..base + 0x34].copy_from_slice(&source_address(index).to_le_bytes());
    let instructions = step_to(m, RETURN);
    callee(m, &before);
    assert_eq!(m.bus.ewram, expected_ram);
    assert_eq!(m.bus.read32(context + 0x30), source_address(index));
    assert_eq!(m.bus.read8(context + 0x28), index);
    instructions
}

#[test]
#[ignore = "private EUR records4 original readers; Wine only, Linux/USA pending"]
fn european_records4_matches_source_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_RECORDS4_MODEL_ROOT").unwrap());
    let out = root.join("windows-model-europe");
    assert!(!out.exists());
    fs::create_dir(&out).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let independent: Records4 =
        serde_json::from_slice(&fs::read(root.join("independent-model.json")).unwrap()).unwrap();
    assert_eq!(data, &independent);
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
    for index in 0..=u8::MAX {
        assert_eq!(source_address(index), 0x087133f8 + u32::from(index) * 4);
        assert_eq!(
            data.record(index).is_some(),
            usize::from(index) < RECORD_COUNT
        );
    }
    for index in [0, 1, 209, 210] {
        assert_eq!(data.record(index).unwrap().source_bytes(), [0; 4]);
        assert_eq!(data.record(index).unwrap().index, index);
    }
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(serde_json::from_slice::<Records4>(&encoded).unwrap(), *data);
    fs::write(out.join("decoded.json"), &encoded).unwrap();
    let mut guards = 0;
    for case in 0..5 {
        let mut bad = data.clone();
        match case {
            0 => bad.source_offset += 4,
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
            2 => bad["records"][0]["index"] = json!(-1),
            3 => bad["records"][0]["index"] = json!(256),
            4 => bad["records"][0]["field00"] = json!(256),
            5 => bad["records"][0]["field01"] = json!(-1),
            6 => bad["records"][0]["field02"] = json!(256),
            7 => bad["records"][0]["field03"] = json!(256),
            8 => bad["records"][0]["field01"] = json!("1"),
            _ => {
                bad["records"][0].as_object_mut().unwrap().remove("field00");
            }
        }
        assert!(serde_json::from_value::<Records4>(bad).is_err());
        guards += 1;
    }
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    guards += 1;
    let mut wrong = rom.clone();
    wrong[SOURCE as usize] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    guards += 1;
    wrong.copy_from_slice(&rom);
    wrong[0x134cb8] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    guards += 1;
    let probes: Vec<Probe> =
        serde_json::from_slice(&fs::read(root.join("independent-probes.json")).unwrap()).unwrap();
    let variants: Vec<Variant> =
        serde_json::from_slice(&fs::read(root.join("synthetic-variants.json")).unwrap()).unwrap();
    let mut m = Machine::new(rom.clone());
    m.bus.ffta_sprites.enabled = false;
    m.bus.ewram.fill(0x5a);
    m.bus.iwram.fill(0x3c);
    m.bus.vram.fill(0xa5);
    m.bus.oam.fill(0xc3);
    m.bus.palette.fill(0x24);
    let (iw, vr, oam, palette) = (
        m.bus.iwram.clone(),
        m.bus.vram.clone(),
        m.bus.oam.clone(),
        m.bus.palette.clone(),
    );
    let mut covered = BTreeSet::new();
    let mut instructions = 0;
    for p in &probes {
        instructions += original_field(&mut m, data, p, &mut covered);
    }
    assert_eq!(covered, (SOURCE..SOURCE_END).collect());
    let mut setter_calls = 0;
    for index in 0..=u8::MAX {
        for instance in [false, true] {
            instructions += original_setter(&mut m, index, instance);
            setter_calls += 1;
        }
    }
    let mut synthetic_prefixes = 0;
    for variant in &variants {
        let mut changed = data.clone();
        changed.records[0] = variant.record.clone();
        changed.validate().unwrap();
        assert_ne!(changed.records[0], data.records[0], "{}", variant.label);
        assert!(changed.verify_source(&rom).is_err());
        guards += 1;
        m.bus.rom[SOURCE as usize..SOURCE as usize + 4]
            .copy_from_slice(&changed.records[0].source_bytes());
        for p in &variant.probes {
            instructions += original_field(&mut m, &changed, p, &mut BTreeSet::new());
            synthetic_prefixes += 1;
        }
        m.bus.rom[SOURCE as usize..SOURCE as usize + 4]
            .copy_from_slice(&rom[SOURCE as usize..SOURCE as usize + 4]);
    }
    assert_eq!(m.bus.iwram, iw);
    assert_eq!(m.bus.vram, vr);
    assert_eq!(m.bus.oam, oam);
    assert_eq!(m.bus.palette, palette);
    assert_eq!(m.bus.rom, rom);
    let report = json!({"status":"PASS-source-model-original-readers","physical_slots":RECORD_COUNT,"source_bytes":SOURCE_BYTES,"actual_source_bytes_original_observed":covered.len(),"source_spans":1,"field_prefixes":probes.len(),"setter_calls":setter_calls,"original_prefixes":probes.len()+setter_calls,"synthetic_variants":variants.len(),"synthetic_prefixes":synthetic_prefixes,"guards":guards,"instructions_coverage_only":instructions,"RAM_callee_stack_VRAM_OAM_palette_ROM_checked":true,"scope":"four unsigned field readers and two full low8 selector/address setters; not fullcallerABI/nativeclock/ordinary/runtime","schema_extractor_decoder_unchanged":[1,83,82],"Linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false});
    fs::write(
        out.join("result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_RECORDS4_MODEL slots211/source844B/actual844B guards{guards} original{} synthetic{synthetic_prefixes} PASS", probes.len() + setter_calls);
}
