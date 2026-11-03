//! Reduced private source/model proof; actual original load/bit prefixes, not gameplay.
use super::*;
use crate::Machine;
use serde_json::{json, Value};
use std::{collections::BTreeSet, fs, path::PathBuf};

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
    bit_index: Option<u32>,
}
fn load_address(m: &Machine) -> (u32, u32, usize) {
    let at = (m.cpu.regs[15] - 0x08000000) as usize;
    let op = u16::from_le_bytes(m.bus.rom[at..at + 2].try_into().unwrap());
    let width = match op & 0xf800 {
        0x7800 => 1,
        0x8800 => 2,
        _ => panic!("unexpected original load {op:04x}"),
    };
    (
        m.cpu.regs[usize::from((op >> 3) & 7)] + u32::from((op >> 6) & 31) * width,
        width,
        usize::from(op & 7),
    )
}
fn original(m: &mut Machine, p: &Probe, covered: &mut BTreeSet<u32>) -> usize {
    let source = SOURCE + p.index * RECORD_BYTES as u32;
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    m.cpu.regs[13] = 0x03007e00;
    m.cpu.regs[14] = 0x08000301;
    if let Some(i) = p.bit_index {
        m.cpu.regs[8] = 0x08000000 + source;
        m.cpu.regs[6] = i << 16;
    } else {
        m.cpu.regs[p.base_register] = 0x08000000 + source + p.base_bias;
    }
    m.cpu.regs[15] = 0x08000000 + p.entry;
    m.cpu.cpsr = 0xbf;
    let before = m.cpu.regs;
    let mut instructions = 0;
    let mut loads = 0;
    loop {
        if p.bit_index.is_none() || m.cpu.regs[15] == 0x080cda72 {
            let (address, width, destination) = load_address(m);
            assert_eq!(address, 0x08000000 + source + p.field as u32);
            assert_eq!(width, p.width);
            if p.bit_index.is_none() {
                assert_eq!(destination, p.destination);
            }
            for byte in source + p.field as u32..source + p.field as u32 + width {
                covered.insert(byte);
            }
            loads += 1;
        }
        m.bus.frame_ready = false;
        m.step();
        instructions += 1;
        assert!(instructions < 24);
        if p.bit_index.is_none() || m.cpu.regs[15] == 0x080cda80 {
            break;
        }
    }
    assert_eq!(loads, 1);
    assert_eq!(
        m.cpu.regs[p.destination], p.expected,
        "record{} field{}",
        p.index, p.field
    );
    for register in 0..15 {
        if p.bit_index.is_some() && register <= 2 {
            continue;
        }
        if p.bit_index.is_none() && register == p.destination {
            continue;
        }
        assert_eq!(m.cpu.regs[register], before[register]);
    }
    if p.bit_index.is_none() {
        assert_eq!(m.cpu.regs[15], before[15] + 2);
        assert_eq!(m.cpu.cpsr, 0xbf);
    } else {
        assert_eq!(m.cpu.cpsr & 0x0fff_ffff, 0xbf);
    }
    instructions
}

#[test]
#[ignore = "private EUR unit-template model/original-reader proof; Wine only, Linux/USA pending"]
fn european_encounter_unit_templates_match_source_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_ENCOUNTER_UNIT_TEMPLATE_MODEL_ROOT").unwrap());
    let out = root.join("windows-model-europe");
    assert!(!out.exists());
    fs::create_dir(&out).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let expected: EncounterUnitTemplates =
        serde_json::from_slice(&fs::read(root.join("independent-model.json")).unwrap()).unwrap();
    assert_eq!(data, &expected);
    data.verify_source(&rom).unwrap();
    let headers = encounter_headers::Catalog::from_rom(&rom).unwrap().decoded;
    let old = constructor_alternates::Catalog::from_rom(&rom)
        .unwrap()
        .decoded;
    let item_owner = items::Catalog::from_rom(&rom).unwrap().decoded;
    let descriptors = metadata14::Catalog::from_rom(&rom).unwrap().decoded;
    let jobs = metadata52::Catalog::from_rom(&rom).unwrap().decoded;
    data.validate_headers(&headers).unwrap();
    data.validate_constructor_alternates(&old).unwrap();
    data.validate_references(&item_owner, &descriptors, &jobs)
        .unwrap();
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
    let full: Vec<_> = data.records.iter().flat_map(Record::source_bytes).collect();
    assert_eq!(full, rom[SOURCE as usize..SOURCE_END as usize]);
    assert_eq!(ffta_rom::digest(&full), SOURCE_SHA256);
    for offset in constructor_alternates::SOURCE_OFFSET..constructor_alternates::END_OFFSET {
        assert_eq!(data.byte_at(offset), None);
    }
    for at in [0, SOURCE - 1, SOURCE_END, u32::MAX] {
        assert_eq!(data.byte_at(at), None);
    }
    assert!(data.record(RECORD_COUNT as u32).is_none());
    assert!(data.record(u32::MAX).is_none());
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<EncounterUnitTemplates>(&encoded).unwrap(),
        *data
    );
    fs::write(out.join("decoded.json"), &encoded).unwrap();
    let mut guards = 0;
    for case in 0..6 {
        let mut bad = data.clone();
        match case {
            0 => bad.source_offset -= 48,
            1 => {
                bad.records.pop();
            }
            2 => bad.records.push(bad.records[0].clone()),
            3 => bad.records.swap(0, 1),
            4 => bad.records[0].index = 1,
            _ => bad.records[0].source_offset += 1,
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
            2 => bad["records"][0]["arguments05_07"] = json!([0, 0]),
            3 => bad["records"][0]["equipment08_10"] = json!([0, 0, 0, 0]),
            4 => bad["records"][0]["ability_bits14_27"] = json!([0, 0]),
            5 => bad["records"][0]["opaque2e_2f"] = json!([0, 0, 0]),
            6 => bad["records"][0]["byte00"] = json!(-1),
            7 => bad["records"][0]["halfword12"] = json!(65536),
            8 => bad["records"][0]["byte2c"] = json!(true),
            _ => bad["records"][0]["byte29"] = json!(256),
        }
        assert!(serde_json::from_value::<EncounterUnitTemplates>(bad).is_err());
        guards += 1;
    }
    for case in 0..4 {
        let mut bad = headers.clone();
        match case {
            0 => bad.headers[0].unit_count += 1,
            1 => bad.headers[0].unit_templates_pointer += 48,
            2 => bad.headers[441].unit_count = 0,
            _ => bad.headers[0].unit_count = u16::MAX,
        }
        assert!(data.validate_headers(&bad).is_err());
        guards += 1;
    }
    for case in 0..2 {
        let mut bad = data.clone();
        if case == 0 {
            bad.records[0].header_index = 0;
        } else {
            bad.records[0].unit_index = 1;
        }
        bad.validate().unwrap();
        assert!(bad.validate_headers(&headers).is_err());
        guards += 1;
    }
    let mut shared = data.clone();
    shared.records[225].opaque2e_2f[1] = 255;
    shared.validate().unwrap();
    shared.validate_headers(&headers).unwrap();
    assert_eq!(shared.source_spans().unwrap(), spans);
    assert!(shared.validate_constructor_alternates(&old).is_err());
    assert!(shared.verify_source(&rom).is_err());
    guards += 1;
    let mut bad_old = old.clone();
    bad_old.records[0].arguments05_07[0] ^= 1;
    assert!(data.validate_constructor_alternates(&bad_old).is_err());
    guards += 1;
    for case in 0..3 {
        let mut bad = data.clone();
        match case {
            0 => bad.records[0].equipment08_10[0] = item_owner.records.len() as u16,
            1 => bad.records[0].byte00 = 0,
            _ => bad.records[0].byte01 = jobs.records.len() as u8,
        }
        bad.validate().unwrap();
        assert!(bad
            .validate_references(&item_owner, &descriptors, &jobs)
            .is_err());
        guards += 1;
    }
    let mut fallback = data.clone();
    fallback.records[0].byte00 = 2;
    fallback.records[0].byte01 = 0;
    fallback
        .validate_references(&item_owner, &descriptors, &jobs)
        .unwrap();
    fallback.records[0].byte00 = 1;
    fallback
        .validate_references(&item_owner, &descriptors, &jobs)
        .unwrap();
    guards += 1;
    for case in 0..7 {
        let mut changed = data.clone();
        let r = &mut changed.records[0];
        match case {
            0 => {
                r.byte03 = 255;
                r.byte04 = 255;
            }
            1 => r.arguments05_07 = [255, 128, 127],
            2 => r.halfword12 = u16::MAX,
            3 => r.ability_bits14_27 = [255; 20],
            4 => {
                r.byte28 = 255;
                r.byte29 = 255;
            }
            5 => {
                r.flags2a = u16::MAX;
                r.byte2c = 255;
                r.byte2d = 255;
            }
            _ => r.opaque2e_2f = [255, 128],
        }
        changed.validate().unwrap();
        changed.validate_headers(&headers).unwrap();
        changed.validate_constructor_alternates(&old).unwrap();
        changed
            .validate_references(&item_owner, &descriptors, &jobs)
            .unwrap();
        assert!(changed.verify_source(&rom).is_err());
        assert_ne!(changed.source_spans().unwrap(), spans);
        guards += 1;
    }
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    let mut wrong = rom.clone();
    wrong[SOURCE as usize] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    wrong.copy_from_slice(&rom);
    wrong[0xcd418] ^= 1;
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
    let (ew, iw, vr, oa, pa) = (
        m.bus.ewram.clone(),
        m.bus.iwram.clone(),
        m.bus.vram.clone(),
        m.bus.oam.clone(),
        m.bus.palette.clone(),
    );
    let mut covered = BTreeSet::new();
    let mut instructions = 0;
    for p in &probes {
        let offset = SOURCE + p.index * 48 + p.field as u32;
        let raw = m.bus.rom[offset as usize..offset as usize + p.width as usize]
            .iter()
            .enumerate()
            .fold(0u32, |v, (i, &byte)| v | (u32::from(byte) << (8 * i)));
        if let Some(bit) = p.bit_index {
            assert_eq!(p.expected, u32::from(raw & (1 << (bit & 7)) != 0));
        } else {
            assert_eq!(raw, p.expected);
        }
        for i in 0..p.width {
            let at = offset + i;
            if (SOURCE..SOURCE_END).contains(&at)
                && !(constructor_alternates::SOURCE_OFFSET..constructor_alternates::END_OFFSET)
                    .contains(&at)
            {
                assert_eq!(data.byte_at(at), Some(m.bus.rom[at as usize]));
            }
        }
        instructions += original(&mut m, p, &mut covered);
    }
    assert_eq!(covered, expected_coverage);
    assert_eq!(m.bus.ewram, ew);
    assert_eq!(m.bus.iwram, iw);
    assert_eq!(m.bus.vram, vr);
    assert_eq!(m.bus.oam, oa);
    assert_eq!(m.bus.palette, pa);
    assert_eq!(ffta_rom::digest(&m.bus.rom), ffta_rom::EUROPE_SHA256);
    let bits = probes.iter().filter(|p| p.bit_index.is_some()).count();
    let outside = probes
        .iter()
        .filter(|p| p.bit_index.is_some_and(|i| i >= 160))
        .count();
    let report = json!({"status":"PASS-source-model-original-readers","records":2850,"full_bytes":136800,"owned_bytes":136608,"shared_bytes":192,"source_spans":2,"guards":guards,"original_probes":probes.len(),"load_prefixes":probes.len()-bits,"bit_prefixes":bits,"bit_outside_physical20B":outside,"distinct_original_bytes":covered.len(),"instructions_coverage_only":instructions,"all_RAM_VRAM_OAM_palette_ROM_checked":true,"scope":"selected actual single-load and dynamic-bit prefixes; not fullimporter/ABI/native-state-clock/ordinary/runtime proof","schema_extractor_decoder_unchanged":[1,80,79],"Linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false});
    fs::write(
        out.join("result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_ENCOUNTER_UNIT_TEMPLATE_MODEL records2850/full136800B/new136608B/shared192B guards{guards} probes{} bitprefixes{bits} PASS",probes.len());
}
