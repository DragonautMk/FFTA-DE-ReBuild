//! One private Wine source/model oracle. Synthetic prefixes are not ordinary gameplay.
use super::*;
use crate::Machine;
use serde_json::{json, Value};
use std::{collections::BTreeSet, fs, path::PathBuf};

const STACK: u32 = 0x03007e00;
const RETURN: u32 = 0x08000300;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Probe {
    index: u32,
    entry: u32,
    field: usize,
    width: u32,
    base_register: usize,
    base_bias: u32,
    destination: usize,
    expected: u32,
    full_gate: bool,
    kind: String,
}
fn load_address(m: &Machine, entry: u32) -> (u32, u32, usize) {
    let at = entry as usize;
    let op = u16::from_le_bytes(m.bus.rom[at..at + 2].try_into().unwrap());
    let width = match op & 0xf800 {
        0x6800 => 4,
        0x7800 => 1,
        0x8800 => 2,
        _ => panic!("unexpected original load {op:04x} at{entry:x}"),
    };
    let base = usize::from((op >> 3) & 7);
    let offset = u32::from((op >> 6) & 31) * width;
    (m.cpu.regs[base] + offset, width, usize::from(op & 7))
}
fn original(m: &mut Machine, p: &Probe, covered: &mut BTreeSet<u32>) -> usize {
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    let source = INDEX_ORIGIN + u32::from(p.index as u16) * HEADER_BYTES as u32;
    if p.full_gate {
        m.cpu.regs[0] = p.index;
    } else {
        m.cpu.regs[p.base_register] = 0x08000000 + source + p.base_bias;
    }
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.regs[15] = 0x08000000 + p.entry;
    m.cpu.cpsr = 0xbf;
    let before = m.cpu.regs;
    let mut loads = Vec::new();
    let mut instructions = 0;
    loop {
        if !p.full_gate || m.cpu.regs[15] == 0x080c0046 {
            let (address, width, destination) = load_address(m, m.cpu.regs[15] - 0x08000000);
            assert_eq!(address, 0x08000000 + source + p.field as u32);
            assert_eq!(width, p.width);
            if !p.full_gate {
                assert_eq!(destination, p.destination);
            }
            loads.push((address, width));
            for byte in address - 0x08000000..address - 0x08000000 + width {
                if (SOURCE..SOURCE_END).contains(&byte) {
                    covered.insert(byte);
                }
            }
        }
        m.bus.frame_ready = false;
        m.step();
        instructions += 1;
        assert!(instructions < 40);
        if !p.full_gate || m.cpu.regs[15] == RETURN {
            break;
        }
        assert!(m.cpu.regs[13] >= STACK - 4);
    }
    assert_eq!(loads, vec![(0x08000000 + source + p.field as u32, p.width)]);
    if p.full_gate {
        assert_eq!(m.cpu.regs[0], p.expected);
        assert_eq!(&m.cpu.regs[4..12], &before[4..12]);
        assert_eq!(m.cpu.regs[13], STACK);
        assert_eq!(m.cpu.regs[14], RETURN | 1);
    } else {
        assert_eq!(
            m.cpu.regs[p.destination], p.expected,
            "field{}/index{}",
            p.field, p.index
        );
        for register in 0..15 {
            if register != p.destination {
                assert_eq!(m.cpu.regs[register], before[register]);
            }
        }
        assert_eq!(m.cpu.cpsr, 0xbf);
        assert_eq!(m.cpu.regs[15], before[15] + 2);
    }
    instructions
}

#[test]
#[ignore = "private EUR encounter-header source/model oracle; Wine only, Linux/USA pending"]
fn european_encounter_headers_match_source_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_ENCOUNTER_HEADER_MODEL_ROOT").unwrap());
    let out = root.join("windows-model-europe");
    assert!(!out.exists());
    fs::create_dir(&out).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let expected: EncounterHeaders =
        serde_json::from_slice(&fs::read(root.join("independent-model.json")).unwrap()).unwrap();
    assert_eq!(data, &expected);
    data.verify_source(&rom).unwrap();
    let owner = constructor_alternates::Catalog::from_rom(&rom)
        .unwrap()
        .decoded;
    data.validate_constructor_alternates(&owner).unwrap();
    let spans = data.source_spans().unwrap();
    let expected_sources: Vec<SourceRange> =
        serde_json::from_slice(&fs::read(root.join("independent-sources.json")).unwrap()).unwrap();
    assert_eq!(catalog.sources, expected_sources);
    assert_eq!(spans.len(), 2);
    let mut owned = 0;
    for ((at, raw, role), source) in spans.iter().zip(&catalog.sources) {
        assert_eq!(*at, source.offset);
        assert_eq!(raw.len() as u32, source.length);
        assert_eq!(role, &source.role);
        assert_eq!(ffta_rom::digest(raw), source.sha256);
        assert_eq!(*raw, rom[*at as usize..*at as usize + raw.len()]);
        for (i, &byte) in raw.iter().enumerate() {
            assert_eq!(data.byte_at(*at + i as u32), Some(byte));
        }
        owned += raw.len();
    }
    assert_eq!(owned, OWNED_BYTES);
    let full: Vec<_> = data.headers.iter().flat_map(Header::source_bytes).collect();
    assert_eq!(full, rom[SOURCE as usize..SOURCE_END as usize]);
    assert_eq!(full.len(), SOURCE_BYTES);
    assert_eq!(ffta_rom::digest(&full), SOURCE_SHA256);
    for at in constructor_alternates::POINTER_OFFSET..constructor_alternates::POINTER_OFFSET + 4 {
        assert_eq!(data.byte_at(at), None);
    }
    for at in [0, INDEX_ORIGIN, SOURCE - 1, SOURCE_END, u32::MAX] {
        assert_eq!(data.byte_at(at), None);
    }
    for index in [0, 443, 65535, 0x10000, 0x101bb, u32::MAX] {
        assert!(data.header(index).is_none());
        assert_eq!(data.nonzero_gate(index), None);
    }
    assert_eq!(data.header(0x10001).unwrap().index, 1);
    assert_eq!(data.header(0x800001ba).unwrap().index, 442);
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<EncounterHeaders>(&encoded).unwrap(),
        *data
    );
    fs::write(out.join("decoded.json"), &encoded).unwrap();
    let mut guards = 0;
    for case in 0..6 {
        let mut bad = data.clone();
        match case {
            0 => bad.source_offset -= 40,
            1 => {
                bad.headers.pop();
            }
            2 => bad.headers.push(bad.headers[0].clone()),
            3 => bad.headers.swap(0, 1),
            4 => bad.headers[0].index = 0,
            _ => bad.headers[1].index = 1,
        }
        assert!(bad.validate().is_err());
        assert!(bad.source_spans().is_err());
        guards += 1;
    }
    for case in 0..10 {
        let mut bad: Value = serde_json::from_slice(&encoded).unwrap();
        match case {
            0 => bad["extra"] = json!(1),
            1 => bad["headers"][0]["extra"] = json!(1),
            2 => bad["headers"][0]["position_pointers"] = json!([0, 0, 0]),
            3 => bad["headers"][0]["opaque"] = json!([0, 0, 0, 0, 0, 0, 0, 0]),
            4 => bad["headers"][0]["unit_count"] = json!(65536),
            5 => bad["headers"][0]["map_argument"] = json!(-1),
            6 => bad["headers"][0]["unit_templates_pointer"] = json!(4294967296u64),
            7 => bad["headers"][0]["opaque"][0] = json!(256),
            8 => bad["headers"][0]["nonzero_gate"] = json!(true),
            _ => bad["headers"][0]["index"] = json!(-1),
        }
        assert!(serde_json::from_value::<EncounterHeaders>(bad).is_err());
        guards += 1;
    }
    for case in 0..8 {
        let mut changed = data.clone();
        let h = &mut changed.headers[0];
        match case {
            0 => h.unit_count = u16::MAX,
            1 => h.map_argument = 255,
            2 => h.unit_templates_pointer = 0,
            3 => h.position_pointers = [0, u32::MAX, 0x02000000, 1],
            4 => h.nonzero_gate = 255,
            5 => h.switch_selector = 255,
            6 => h.opaque = [255; 7],
            _ => h.random_position_range = 255,
        }
        changed.validate().unwrap();
        changed.validate_constructor_alternates(&owner).unwrap();
        assert!(changed.verify_source(&rom).is_err());
        assert_ne!(changed.source_spans().unwrap(), spans);
        guards += 1;
    }
    let mut changed = data.clone();
    changed.headers[28].unit_templates_pointer ^= 1;
    changed.validate().unwrap();
    assert_eq!(changed.source_spans().unwrap(), spans);
    assert!(changed.validate_constructor_alternates(&owner).is_err());
    assert!(changed.verify_source(&rom).is_err());
    guards += 1;
    let mut bad_owner = owner.clone();
    bad_owner.source_offset += 1;
    assert!(data.validate_constructor_alternates(&bad_owner).is_err());
    guards += 1;
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    let mut wrong = rom.clone();
    wrong[SOURCE as usize] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    wrong.copy_from_slice(&rom);
    wrong[0xc0034] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    guards += 3;
    let probes: Vec<Probe> =
        serde_json::from_slice(&fs::read(root.join("independent-probes.json")).unwrap()).unwrap();
    let expected_coverage: BTreeSet<u32> =
        serde_json::from_slice(&fs::read(root.join("independent-coverage.json")).unwrap()).unwrap();
    let mut m = Machine::new(rom);
    m.bus.ffta_sprites.enabled = false;
    m.bus.ewram.fill(0x5a);
    m.bus.iwram.fill(0x3c);
    m.bus.vram.fill(0xa5);
    m.bus.oam.fill(0xc3);
    m.bus.palette.fill(0x24);
    let ew = m.bus.ewram.clone();
    let iw = m.bus.iwram.clone();
    let vr = m.bus.vram.clone();
    let oam = m.bus.oam.clone();
    let palette = m.bus.palette.clone();
    let mut covered = BTreeSet::new();
    let mut instructions = 0;
    for p in &probes {
        if let Some(header) = data.header(p.index) {
            if p.full_gate {
                assert_eq!(data.nonzero_gate(p.index), Some(p.expected != 0));
            } else {
                let bytes = header.source_bytes();
                let mut word = [0; 4];
                word[..p.width as usize]
                    .copy_from_slice(&bytes[p.field..p.field + p.width as usize]);
                assert_eq!(u32::from_le_bytes(word), p.expected);
            }
        } else {
            assert!(p.full_gate);
        }
        instructions += original(&mut m, p, &mut covered);
    }
    assert_eq!(covered, expected_coverage);
    assert_eq!(covered.len(), 14586);
    assert_eq!(m.bus.ewram, ew);
    let mut iw_after = m.bus.iwram.clone();
    iw_after[0x7dfc..0x7e00].copy_from_slice(&iw[0x7dfc..0x7e00]);
    assert_eq!(iw_after, iw);
    assert_eq!(m.bus.vram, vr);
    assert_eq!(m.bus.oam, oam);
    assert_eq!(m.bus.palette, palette);
    assert_eq!(ffta_rom::digest(&m.bus.rom), ffta_rom::EUROPE_SHA256);
    let report = json!({"status":"PASS-source-model-original-readers","headers":442,"full_bytes":17680,"owned_bytes":17676,"shared_bytes":4,"source_spans":2,"semantic_original_bytes":14586,"new_owned_semantic_original_bytes":14582,"opaque_bytes":3094,"original_calls":probes.len(),"field_load_prefixes":probes.iter().filter(|p|!p.full_gate).count(),"full_gate_calls":probes.iter().filter(|p|p.full_gate).count(),"byte_alias_calls":probes.iter().filter(|p|p.kind=="u16-alias").count(),"outside_projection_calls":probes.iter().filter(|p|p.kind=="outside").count(),"guards":guards,"instructions_coverage_only":instructions,"RAM_callee_stack_VRAM_OAM_palette_checked":true,"scope":"synthetic original one-load prefixes and fullC0034 returns only; not fullABI/native-clock/liveimporter/ordinary/runtime/payload proof","schema_extractor_decoder_unchanged":[1,79,78],"Linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false});
    fs::write(
        out.join("result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_ENCOUNTER_HEADER_MODEL headers442/full17680B/owned17676B/shared4B/original14586semanticB/opaque3094B guards{guards} original{} PASS",probes.len());
}
