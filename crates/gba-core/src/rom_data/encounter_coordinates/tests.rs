//! One focused Wine proof: physical pairs, actual loads, bounded random prefix.
use super::*;
use crate::{Bus, Machine};
use serde_json::{json, Value};
use std::{collections::BTreeSet, fs, path::PathBuf};
const STACK: u32 = 0x03007d00;
const UNIT: u32 = 0x02010000;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Probe {
    address: u32,
    entry: u32,
    base: usize,
    destination: usize,
    index_register: Option<usize>,
    index_value: u32,
    signed: bool,
    expected: u32,
}
fn original_load(m: &mut Machine, p: &Probe, covered: &mut BTreeSet<u32>) -> usize {
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    m.cpu.regs[p.base] = 0x08000000 + p.address;
    if let Some(r) = p.index_register {
        m.cpu.regs[r] = p.index_value;
    }
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = 0x08000301;
    m.cpu.regs[15] = 0x08000000 + p.entry;
    m.cpu.cpsr = 0xbf;
    let before = m.cpu.regs;
    let op = u16::from_le_bytes(
        m.bus.rom[p.entry as usize..p.entry as usize + 2]
            .try_into()
            .unwrap(),
    );
    let base = usize::from((op >> 3) & 7);
    let destination = usize::from(op & 7);
    let offset = if p.signed {
        assert_eq!(op & 0xfe00, 0x5600);
        m.cpu.regs[usize::from((op >> 6) & 7)]
    } else {
        assert_eq!(op & 0xf800, 0x7800);
        u32::from((op >> 6) & 31)
    };
    assert_eq!(base, p.base);
    assert_eq!(destination, p.destination);
    assert_eq!(
        m.cpu.regs[base] + offset,
        0x08000000 + p.address + p.index_value
    );
    m.bus.frame_ready = false;
    m.step();
    assert_eq!(
        m.cpu.regs[p.destination], p.expected,
        "original{:x} source{:x}",
        p.entry, p.address
    );
    for r in 0..15 {
        if r != p.destination {
            assert_eq!(m.cpu.regs[r], before[r]);
        }
    }
    assert_eq!(m.cpu.cpsr, 0xbf);
    assert_eq!(m.cpu.regs[15], before[15] + 2);
    covered.insert(p.address + p.index_value);
    1
}
fn random_prefix(m: &mut Machine, list: &CoordinateList, seed: u32) -> usize {
    assert_eq!(list.slot, 1);
    assert_eq!(list.points.len(), 10);
    let header = encounter_headers::INDEX_ORIGIN + u32::from(list.header_index) * 40;
    m.bus.write32(0x030034b0, seed);
    m.bus.write16(STACK, 0);
    m.bus.write32(STACK + 8, 0x08000000 + header);
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    m.cpu.regs[4] = STACK;
    m.cpu.regs[5] = UNIT;
    m.cpu.regs[6] = 0;
    m.cpu.regs[7] = 10;
    m.cpu.regs[8] = UNIT + 0xf6;
    m.cpu.regs[9] = UNIT + 0xf7;
    m.cpu.regs[10] = UNIT + 0xf8;
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = 0x08000301;
    m.cpu.regs[15] = 0x08128764;
    m.cpu.cpsr = 0xbf;
    let ew = m.bus.ewram.clone();
    let iw = m.bus.iwram.clone();
    let next = seed.wrapping_mul(0x41c64e6d).wrapping_add(0x3039);
    let index = ((next >> 16) & 0x7fff) % 10;
    let point = list.points[index as usize];
    let mut calls = 0;
    let mut loads = 0;
    let mut instructions = 0;
    while m.cpu.regs[15] != 0x0812879e {
        match m.cpu.regs[15] {
            0x08128764 | 0x0812876e => calls += 1,
            0x08128768 => assert_eq!(m.cpu.regs[0], (next >> 16) & 0x7fff),
            0x08128772 => assert_eq!(m.cpu.regs[0], index),
            0x08128792 | 0x08128798 => {
                assert_eq!(m.cpu.regs[1], 0x08000000 + list.source_offset + index * 2);
                loads += 1;
            }
            _ => {}
        }
        assert!(instructions < 500);
        m.bus.frame_ready = false;
        m.step();
        instructions += 1;
    }
    assert_eq!((calls, loads), (2, 2));
    assert_eq!(m.bus.read32(0x030034b0), next);
    assert_eq!(m.bus.read16(STACK), 1u16 << index);
    assert_eq!(m.bus.read8(UNIT + 0xf6), point.x);
    assert_eq!(m.bus.read8(UNIT + 0xf7), point.y);
    assert_eq!(m.cpu.regs[13], STACK);
    assert_eq!(m.cpu.regs[14], 0x08128773);
    assert_eq!(
        &m.cpu.regs[5..12],
        &[
            UNIT,
            0,
            10,
            UNIT + 0xf6,
            UNIT + 0xf7,
            UNIT + 0xf8,
            0x1100000b
        ]
    );
    let mut ew_after = m.bus.ewram.clone();
    ew_after[0x100f6..0x100f8].copy_from_slice(&ew[0x100f6..0x100f8]);
    assert_eq!(ew_after, ew);
    let mut iw_after = m.bus.iwram.clone();
    iw_after[0x7cf8..0x7d02].copy_from_slice(&iw[0x7cf8..0x7d02]);
    iw_after[0x34b0..0x34b4].copy_from_slice(&iw[0x34b0..0x34b4]);
    assert_eq!(iw_after, iw);
    instructions
}
#[test]
#[ignore = "private EUR coordinate source/model proof; Wine only, Linux/USA pending"]
fn european_encounter_coordinates_match_source_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_ENCOUNTER_COORDINATE_MODEL_ROOT").unwrap());
    let out = root.join("windows-model-europe");
    assert!(!out.exists());
    fs::create_dir(&out).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let expected: EncounterCoordinates =
        serde_json::from_slice(&fs::read(root.join("independent-model.json")).unwrap()).unwrap();
    assert_eq!(data, &expected);
    data.verify_source(&rom).unwrap();
    let headers = encounter_headers::Catalog::from_rom(&rom).unwrap().decoded;
    data.validate_headers(&headers).unwrap();
    let spans = data.source_spans().unwrap();
    let expected_sources: Vec<SourceRange> =
        serde_json::from_slice(&fs::read(root.join("independent-sources.json")).unwrap()).unwrap();
    assert_eq!(catalog.sources, expected_sources);
    assert_eq!(spans.len(), LIST_COUNT);
    let mut full = Vec::new();
    for ((at, raw, role), source) in spans.iter().zip(&catalog.sources) {
        assert_eq!(
            (*at, raw.len() as u32, role),
            (source.offset, source.length, &source.role)
        );
        assert_eq!(ffta_rom::digest(raw), source.sha256);
        assert_eq!(*raw, rom[*at as usize..*at as usize + raw.len()]);
        for (i, &byte) in raw.iter().enumerate() {
            assert_eq!(data.byte_at(*at + i as u32), Some(byte));
        }
        full.extend(raw);
    }
    assert_eq!(full, rom[SOURCE as usize..SOURCE_END as usize]);
    assert_eq!(full.len(), SOURCE_BYTES);
    assert_eq!(ffta_rom::digest(&full), SOURCE_SHA256);
    assert_eq!(
        data.lists.iter().map(|l| l.points.len()).sum::<usize>(),
        PAIR_COUNT
    );
    for at in [0, SOURCE - 1, SOURCE_END, u32::MAX] {
        assert_eq!(data.byte_at(at), None);
    }
    assert!(data.list(0, 0).is_none());
    assert!(data.list(29, 0).is_none());
    assert!(data.list(442, 4).is_none());
    assert_eq!(data.list(442, 3).unwrap().source_offset, 0x70c8f2);
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<EncounterCoordinates>(&encoded).unwrap(),
        *data
    );
    fs::write(out.join("decoded.json"), &encoded).unwrap();
    let mut guards = 0;
    for case in 0..6 {
        let mut bad = data.clone();
        match case {
            0 => bad.source_offset -= 2,
            1 => {
                bad.lists.pop();
            }
            2 => bad.lists.push(bad.lists[0].clone()),
            3 => bad.lists.swap(0, 1),
            4 => bad.lists[0].source_offset += 1,
            _ => bad.lists[0].slot = 4,
        }
        assert!(bad.validate().is_err());
        assert!(bad.source_spans().is_err());
        guards += 1;
    }
    for case in 0..10 {
        let mut bad: Value = serde_json::from_slice(&encoded).unwrap();
        match case {
            0 => bad["extra"] = json!(1),
            1 => bad["lists"][0]["extra"] = json!(1),
            2 => bad["lists"][0]["points"][0]["extra"] = json!(1),
            3 => bad["lists"][0]["points"][0]["x"] = json!(-1),
            4 => bad["lists"][0]["points"][0]["y"] = json!(256),
            5 => bad["lists"][0]["points"][0]["x"] = json!(true),
            6 => bad["lists"][0]["index"] = json!(65536),
            7 => bad["lists"][0]["points"] = json!([1, 2]),
            8 => {
                bad["lists"][0]["points"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("y");
            }
            _ => bad["source_offset"] = json!(-1),
        }
        assert!(serde_json::from_value::<EncounterCoordinates>(bad).is_err());
        guards += 1;
    }
    for case in 0..5 {
        let mut bad = headers.clone();
        match case {
            0 => bad.headers[0].position_pointers[0] += 2,
            1 => bad.headers[0].placement_count0 += 1,
            2 => bad.headers[0].position_pointers[3] = 0,
            3 => bad.headers[0].position_pointers[1] = 0x08000000 + SOURCE,
            _ => bad.headers[441].position_pointers[3] = 0,
        }
        assert!(data.validate_headers(&bad).is_err());
        guards += 1;
    }
    for case in 0..2 {
        let mut bad = data.clone();
        if case == 0 {
            bad.lists[0].header_index = 0;
        } else {
            bad.lists[0].slot = 1;
        }
        bad.validate().unwrap();
        assert!(bad.validate_headers(&headers).is_err());
        guards += 1;
    }
    for (x, y) in [(128, 255), (127, 0), (255, 128)] {
        let mut changed = data.clone();
        changed.lists[0].points[0] = Point { x, y };
        changed.validate_headers(&headers).unwrap();
        assert_eq!(changed.lists[0].points[0].signed(), [x as i8, y as i8]);
        assert!(changed.verify_source(&rom).is_err());
        guards += 1;
    }
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    let mut wrong = rom.clone();
    wrong[SOURCE as usize] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    wrong.copy_from_slice(&rom);
    wrong[0x128792] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    guards += 3;
    assert_eq!(Point { x: 128, y: 255 }.signed(), [-128, -1]);
    guards += 1;
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
        let byte = data.byte_at(p.address + p.index_value).unwrap();
        let expected = if p.signed {
            i32::from(byte as i8) as u32
        } else {
            u32::from(byte)
        };
        assert_eq!(expected, p.expected);
        instructions += original_load(&mut m, p, &mut covered);
    }
    assert_eq!(covered, expected_coverage);
    assert_eq!(covered.len(), 4684);
    assert_eq!(m.bus.ewram, ew);
    assert_eq!(m.bus.iwram, iw);
    let first = data.lists[0].source_offset as usize;
    let pre = m.bus.rom[first..first + 2].to_vec();
    let mut synthetic = 0;
    for (x, y) in [(128, 255), (127, 0), (255, 128)] {
        m.bus.rom[first..first + 2].copy_from_slice(&[x, y]);
        for (entry, destination, index_register, index_value, signed) in [
            (0xbd00c, 0, None, 0, false),
            (0xbd012, 0, None, 1, false),
            (0xbd044, 1, Some(1), 0, true),
            (0xbd048, 2, Some(2), 1, true),
        ] {
            let byte = [x, y][index_value as usize];
            let p = Probe {
                address: first as u32,
                entry,
                base: 4,
                destination,
                index_register,
                index_value,
                signed,
                expected: if signed {
                    i32::from(byte as i8) as u32
                } else {
                    u32::from(byte)
                },
            };
            instructions += original_load(&mut m, &p, &mut BTreeSet::new());
            synthetic += 1;
        }
    }
    m.bus.rom[first..first + 2].copy_from_slice(&pre);
    let mut random_cases = 0;
    for list in data.lists.iter().filter(|l| l.slot == 1) {
        for seed in [0, 1, u32::MAX] {
            instructions += random_prefix(&mut m, list, seed);
            random_cases += 1;
        }
    }
    assert_eq!(random_cases, 102);
    assert_eq!(synthetic, 12);
    assert_eq!(m.bus.vram, vr);
    assert_eq!(m.bus.oam, oa);
    assert_eq!(m.bus.palette, pa);
    assert_eq!(ffta_rom::digest(&m.bus.rom), ffta_rom::EUROPE_SHA256);
    let report = json!({"status":"PASS-source-model-original-readers","lists":691,"pairs":2342,"source_bytes":4684,"source_spans":691,"guards":guards,"original_load_prefixes":probes.len(),"all4684sourcebytes_original_observed":true,"isolated_highbit_prefixes":synthetic,"original_random_selection_prefixes":random_cases,"random_lists":34,"seed_edges":3,"RNG_mask_effective_addresses_results_RAM_guards":true,"instructions_coverage_only":instructions,"scope":"allpair originalunsigned-signedload prefixes and bounded102random-selection prefixes; notfullplacementABI/native-state-clock/ordinary/runtime proof","schema_extractor_decoder_unchanged":[1,81,80],"Linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false});
    fs::write(
        out.join("result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_ENCOUNTER_COORDINATE_MODEL lists691/pairs2342/source4684B guards{guards} original{} highbit{synthetic} randomprefix{random_cases} PASS",probes.len());
}
