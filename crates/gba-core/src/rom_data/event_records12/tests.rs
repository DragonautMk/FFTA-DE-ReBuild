//! Isolated original prefixes; setters, event start and callback continuation excluded.
use super::*;
use crate::Machine;
use serde_json::{json, Value};
use std::{collections::BTreeSet, fs, path::PathBuf};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    label: String,
    stage: u8,
    index: u32,
    argument: u32,
    key: u16,
    field: usize,
    patches: Vec<(u16, [u8; 12])>,
    expected: Option<Vec<u32>>,
    owned: Option<Vec<u32>>,
    reads: Vec<u32>,
}
const LOADS: [(u32, usize, u32, u32, usize); 7] = [
    (0xd40ec, 0, 0, 1, 0),
    (0x12739c, 1, 1, 1, 0),
    (0x9d28, 4, 2, 2, 0),
    (0x9d0c, 4, 4, 2, 0),
    (0xd25ae, 0, 6, 1, 1),
    (0x28e54, 4, 7, 1, 0),
    (0x24d96, 0, 8, 2, 1),
];
fn original(
    m: &mut Machine,
    data: &EventRecords12,
    c: &Case,
    covered: &mut BTreeSet<u32>,
) -> usize {
    m.bus.ewram.fill(0x5a);
    m.bus.iwram.fill(0x3c);
    m.bus.ewram[0x100..0x102].copy_from_slice(&(c.index as u16).to_le_bytes());
    m.bus.ewram[0x101..0x103].copy_from_slice(&c.key.to_le_bytes());
    if c.stage == 3 {
        m.bus.ewram[0x100..0x102].copy_from_slice(&(c.index as u16).to_le_bytes());
    }
    let (mut ewram, mut iwram, vram, oam, palette) = (
        m.bus.ewram.clone(),
        m.bus.iwram.clone(),
        m.bus.vram.clone(),
        m.bus.oam.clone(),
        m.bus.palette.clone(),
    );
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    m.cpu.regs[13] = 0x03007d00;
    m.cpu.regs[14] = 0x03007f01;
    m.cpu.cpsr = 0xbf;
    let address = 0x08000000 + SOURCE + 12 * c.index;
    let (entry, stop, stack) = match c.stage {
        0 => {
            m.cpu.regs[0] = address;
            m.cpu.regs[1] = c.argument;
            (0x9d04, 0x9d2c, vec![0x11000004, 0x03007f01])
        }
        1 => {
            m.cpu.regs[1] = 0x02000100;
            if let Some(v) = &c.expected {
                ewram[0x3c2a] = v[0] as u8;
            }
            (
                0x127388,
                if c.expected.is_some() {
                    0x1273a8
                } else {
                    0x1273c4
                },
                vec![0x11000004, 0x11000005, 0x03007f01],
            )
        }
        2 => {
            let (pc, base, _, _, _) = LOADS[c.field];
            m.cpu.regs[base] = address;
            (pc, pc + 2, vec![])
        }
        _ => {
            m.cpu.regs[0] = 0x02000100;
            (0x359f4, 0x35a00, vec![])
        }
    };
    for (i, word) in stack.iter().enumerate() {
        let at = 0x7d00 - 4 * stack.len() + 4 * i;
        iwram[at..at + 4].copy_from_slice(&u32::to_le_bytes(*word));
    }
    m.cpu.regs[15] = 0x08000000 + entry;
    let mut reads = Vec::new();
    let mut instructions = 0;
    while m.cpu.regs[15] != 0x08000000 + stop {
        let pc = m.cpu.regs[15] - 0x08000000;
        let load = if c.stage == 2 {
            let (at, base, offset, width, _) = LOADS[c.field];
            (pc == at).then(|| (m.cpu.regs[base] + offset, width))
        } else {
            match pc {
                0x9d0c => Some((m.cpu.regs[4] + 4, 2)),
                0x9d1c => Some((m.cpu.regs[0] + 2, 2)),
                0x9d1e => Some((m.cpu.regs[4] + 4, 2)),
                0x9d28 => Some((m.cpu.regs[4] + 2, 2)),
                0x12739c => Some((m.cpu.regs[1] + 1, 1)),
                0x1273a4 => Some((m.cpu.regs[1] + 2, 2)),
                _ => None,
            }
        };
        if let Some((at, width)) = load {
            let at = at - 0x08000000;
            for offset in at..at + width {
                if let Some(byte) = data.byte_at(offset) {
                    assert_eq!(byte, m.bus.rom[offset as usize]);
                    covered.insert(offset);
                }
                reads.push(offset);
            }
        }
        m.bus.frame_ready = false;
        m.step();
        instructions += 1;
        assert!(instructions < 5000, "{} PC{:08x}", c.label, m.cpu.regs[15]);
    }
    assert_eq!(reads, c.reads, "{} actual load order", c.label);
    match c.stage {
        0 => {
            let v = c.expected.as_ref().unwrap();
            assert_eq!(&m.cpu.regs[..2], v.as_slice(), "{} original pair", c.label);
        }
        1 => {
            if let Some(v) = &c.expected {
                assert_eq!(&m.cpu.regs[..2], &[v[1], 0]);
            } else {
                assert_eq!(m.cpu.regs[2], 0);
            }
        }
        2 => assert_eq!(Some(vec![m.cpu.regs[LOADS[c.field].4]]), c.expected),
        _ => assert_eq!(Some(vec![m.cpu.regs[0]]), c.expected),
    }
    assert_eq!(m.cpu.regs[13], 0x03007d00 - 4 * stack.len() as u32);
    assert_eq!(
        &m.cpu.regs[8..13],
        &(8..13).map(|i| 0x11000000 + i).collect::<Vec<_>>()
    );
    assert_eq!(m.bus.ewram, ewram);
    assert_eq!(m.bus.iwram, iwram);
    assert_eq!(m.bus.vram, vram);
    assert_eq!(m.bus.oam, oam);
    assert_eq!(m.bus.palette, palette);
    instructions
}

#[test]
#[ignore = "private EUR event-record source/original-prefix proof; Wine only, Linux/USA pending"]
fn european_event_records12_match_source_and_original_prefixes() {
    let root = PathBuf::from(std::env::var_os("FFTA_EVENT_RECORDS12_MODEL_ROOT").unwrap());
    let out = root.join("windows-model-europe");
    assert!(!out.exists());
    fs::create_dir(&out).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let independent: EventRecords12 =
        serde_json::from_slice(&fs::read(root.join("independent-model.json")).unwrap()).unwrap();
    let sources: Vec<SourceRange> =
        serde_json::from_slice(&fs::read(root.join("independent-sources.json")).unwrap()).unwrap();
    assert_eq!(*data, independent);
    assert_eq!(catalog.sources, sources);
    data.verify_source(&rom).unwrap();
    assert_eq!(
        data.source_bytes(),
        rom[SOURCE as usize..SOURCE_END as usize]
    );
    assert_eq!(ffta_rom::digest(&data.source_bytes()), SOURCE_SHA256);
    let spans = data.source_spans().unwrap();
    assert_eq!(spans.len(), 1);
    assert_eq!(
        spans[0],
        (SOURCE, data.source_bytes(), "event-records12".into())
    );
    for (i, &b) in data.source_bytes().iter().enumerate() {
        assert_eq!(data.byte_at(SOURCE + i as u32), Some(b));
    }
    for at in [
        0,
        SOURCE - 2,
        SOURCE - 1,
        SOURCE_END,
        SOURCE_END + 1,
        u32::MAX,
    ] {
        assert_eq!(data.byte_at(at), None);
    }
    for i in [256, 65535, u32::MAX] {
        assert_eq!(data.record(i), None);
    }
    assert_eq!(data.alias_record(31), None);
    assert_eq!(data.alias_record(255), None);
    assert_eq!(data.alias_record(0x100), data.record(225));
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<EventRecords12>(&encoded).unwrap(),
        *data
    );
    fs::write(out.join("decoded.json"), &encoded).unwrap();
    let mut guards = 0;
    for case in 0..4 {
        let mut bad = data.clone();
        match case {
            0 => bad.source_offset += 1,
            1 => {
                bad.records.pop();
            }
            2 => bad.records.swap(0, 1),
            _ => bad.records[255].index = 0,
        };
        assert!(bad.validate().is_err());
        assert!(bad.source_spans().is_err());
        guards += 1;
    }
    for case in 0..12 {
        let mut bad: Value = serde_json::from_slice(&encoded).unwrap();
        match case {
            0 => bad["extra"] = json!(0),
            1 => bad["records"][0]["extra"] = json!(0),
            2 => {
                bad["records"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("parameter06");
            }
            3 => bad["records"][0]["header00"] = json!(-1),
            4 => bad["records"][0]["event_key01"] = json!(256),
            5 => bad["records"][0]["primary02"] = json!(-1),
            6 => bad["records"][0]["alternate04"] = json!(65536),
            7 => bad["records"][0]["byte07"] = json!(true),
            8 => bad["records"][0]["flags08_raw"] = json!("0"),
            9 => bad["records"][0]["opaque0a"] = json!(vec![0; 1]),
            10 => bad["records"][0]["opaque0a"] = json!(vec![0; 3]),
            _ => bad["records"][0]["index"] = json!(65536),
        };
        assert!(serde_json::from_value::<EventRecords12>(bad).is_err());
        guards += 1;
    }
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    guards += 1;
    let mut wrong = rom.clone();
    wrong[SOURCE as usize] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    guards += 1;
    wrong.copy_from_slice(&rom);
    wrong[0x9d04] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    guards += 1;
    let mut changed = data.clone();
    changed.records[0].opaque0a = [255, 128];
    changed.validate().unwrap();
    assert!(changed.verify_source(&rom).is_err());
    guards += 1;
    let cases: Vec<Case> =
        serde_json::from_slice(&fs::read(root.join("independent-cases.json")).unwrap()).unwrap();
    let mut m = Machine::new(rom.clone());
    m.bus.ffta_sprites.enabled = false;
    let (mut base, mut synthetic) = (0, 0);
    let (mut base_bytes, mut synthetic_bytes) = (BTreeSet::new(), BTreeSet::new());
    let mut instructions = 0;
    for c in &cases {
        let mut current = data.clone();
        for (index, raw) in &c.patches {
            current.records[*index as usize] = Record::decode(*index, raw);
        }
        current.validate().unwrap();
        let value = match c.stage {
            0 => current
                .record(c.index)
                .and_then(|r| current.conditional_pair(r, c.argument))
                .map(|(a, b)| vec![u32::from(a), u32::from(b)]),
            1 => current
                .script_match(c.key)
                .map(|(i, v)| vec![u32::from(i), u32::from(v)]),
            2 => current.record(c.index).map(|r| {
                let (_, _, off, n, _) = LOADS[c.field];
                let raw = r.source_bytes();
                vec![if n == 1 {
                    u32::from(raw[off as usize])
                } else {
                    u32::from(u16::from_le_bytes([
                        raw[off as usize],
                        raw[off as usize + 1],
                    ]))
                }]
            }),
            _ => Some(vec![EventRecords12::encoded_view(c.index)]),
        };
        assert_eq!(value, c.owned, "{} owned projection", c.label);
        assert_eq!(
            EventRecords12::encoded_alias_view(c.argument),
            0x08000000 + ALIAS_SOURCE + 12 * u32::from(c.argument as u8)
        );
        m.bus.rom[SOURCE as usize..SOURCE_END as usize].copy_from_slice(&current.source_bytes());
        let cover = if c.patches.is_empty() {
            base += 1;
            &mut base_bytes
        } else {
            synthetic += 1;
            &mut synthetic_bytes
        };
        instructions += original(&mut m, &current, c, cover);
        m.bus.rom[SOURCE as usize..SOURCE_END as usize]
            .copy_from_slice(&rom[SOURCE as usize..SOURCE_END as usize]);
    }
    assert_eq!(m.bus.rom, rom);
    let report = json!({"status":"PASS-source-model-original-prefixes","guards":guards,"physical_records":256,"source_bytes":3072,"source_spans":1,
        "base_prefix_cases":base,"synthetic_prefix_cases":synthetic,"actual_base_source_bytes_observed":base_bytes.len(),"actual_synthetic_source_bytes_observed":synthetic_bytes.len(),
        "instructions_coverage_only":instructions,"all_EWRAM_VRAM_OAM_palette_exact_except_expected_script_index":true,"IWRAM_exact_including_prefix_pushes":true,"SP_R8_R12_final_ROM_checked":true,
        "actual_load_order_and_results_exact":true,"scope":"9D04pair before setter,127388scan before live continuation,seven unsigned loads,u16computed view; prologue/epilogue/fullABI/eventstart/liveeffects/nativeclock/ordinary/runtime excluded",
        "schema_extractor_decoder_unchanged":[1,87,86],"Linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false});
    fs::write(
        out.join("result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_EVENT_RECORDS12_MODEL bytes3072/spans1 guards{guards} base{base} synthetic{synthetic} PASS");
}
