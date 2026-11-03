//! One focused private Wine oracle; synthetic calls, no ordinary game fixture.
use super::*;
use crate::Machine;
use serde_json::{json, Value};
use std::{collections::BTreeSet, fs, path::PathBuf};

const STACK: u32 = 0x03007e00;
const RETURN: u32 = 0x08000300;
const OUTPUT: u32 = 0x02005000;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Probe {
    entry: u32,
    args: [u32; 3],
    expected: u32,
    loads: Vec<(u32, u32)>,
    kind: String,
}

fn original(m: &mut Machine, p: &Probe, covered: &mut BTreeSet<u32>) -> usize {
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    if p.entry == 0x4dd0c {
        m.cpu.regs[0] = OUTPUT;
        m.cpu.regs[1..4].copy_from_slice(&p.args);
    } else {
        m.cpu.regs[..3].copy_from_slice(&p.args);
    }
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.regs[15] = 0x08000000 + p.entry;
    m.cpu.cpsr = 0xbf;
    let callee = m.cpu.regs[4..12].to_vec();
    m.bus.ewram[0x5000..0x5004].fill(0xa5);
    let ew = m.bus.ewram.clone();
    let iw = m.bus.iwram.clone();
    let vram = m.bus.vram.clone();
    let oam = m.bus.oam.clone();
    let palette = m.bus.palette.clone();
    let mut loads = Vec::new();
    let mut instructions = 0;
    while m.cpu.regs[15] != RETURN {
        assert!(instructions < 80, "PC{:08x}", m.cpu.regs[15]);
        let load = match m.cpu.regs[15] {
            0x0804dd26 => Some((m.cpu.regs[0], 1)),
            0x0804dd50 => Some((m.cpu.regs[0], 4)),
            0x0804dd56 => Some((m.cpu.regs[0] + m.cpu.regs[1], 1)),
            0x0804dd5a => Some((m.cpu.regs[0] + 1, 1)),
            _ => None,
        };
        if let Some((at, size)) = load {
            let offset = at - 0x08000000;
            loads.push((offset, size));
            for b in offset..offset + size {
                if (POINTS_START..SOURCE_END).contains(&b) {
                    covered.insert(b);
                }
            }
        }
        m.bus.frame_ready = false;
        m.step();
        instructions += 1;
        assert!(m.cpu.regs[13] >= STACK - 20);
    }
    assert_eq!(loads, p.loads, "{:x}/{:?}", p.entry, p.args);
    if p.entry == 0x4dd0c {
        assert_eq!(m.cpu.regs[0], OUTPUT);
        let expected = p.expected.to_le_bytes();
        assert_eq!(&m.bus.ewram[0x5000..0x5004], &expected);
        let mut after = m.bus.ewram.clone();
        after[0x5000..0x5004].copy_from_slice(&ew[0x5000..0x5004]);
        assert_eq!(after, ew);
    } else {
        assert_eq!(m.cpu.regs[0], p.expected);
        assert_eq!(m.bus.ewram, ew);
    }
    assert_eq!(m.cpu.regs[13], STACK);
    assert_eq!(m.cpu.regs[14], RETURN | 1);
    assert_eq!(&m.cpu.regs[4..12], callee.as_slice());
    let stack_bytes = if p.entry == 0x4dd0c { 20 } else { 4 };
    let mut before = iw;
    let mut after = m.bus.iwram.clone();
    before[0x7e00 - stack_bytes..0x7e00].fill(0);
    after[0x7e00 - stack_bytes..0x7e00].fill(0);
    assert_eq!(before, after);
    assert_eq!(m.bus.vram, vram);
    assert_eq!(m.bus.oam, oam);
    assert_eq!(m.bus.palette, palette);
    instructions
}

#[test]
#[ignore = "private EUR road model/original-reader oracle; Wine only, Linux/USA pending"]
fn european_world_roads_match_source_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_WORLD_ROAD_MODEL_ROOT").unwrap());
    let out = root.join("windows-model-europe");
    assert!(!out.exists());
    fs::create_dir(&out).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let expected: WorldRoads =
        serde_json::from_slice(&fs::read(root.join("independent-model.json")).unwrap()).unwrap();
    assert_eq!(data, &expected);
    data.verify_source(&rom).unwrap();
    let expected_sources: Vec<SourceRange> =
        serde_json::from_slice(&fs::read(root.join("independent-sources.json")).unwrap()).unwrap();
    assert_eq!(catalog.sources, expected_sources);
    let spans = data.source_spans().unwrap();
    assert_eq!(spans.len(), 46);
    let mut next = POINTS_START;
    let mut reconstructed = Vec::new();
    for ((at, bytes, role), source) in spans.iter().zip(&catalog.sources) {
        assert_eq!(*at, next);
        assert_eq!(*at, source.offset);
        assert_eq!(bytes.len() as u32, source.length);
        assert_eq!(role, &source.role);
        assert_eq!(ffta_rom::digest(bytes), source.sha256);
        assert_eq!(*bytes, rom[*at as usize..*at as usize + bytes.len()]);
        for (i, &b) in bytes.iter().enumerate() {
            assert_eq!(data.byte_at(*at + i as u32), Some(b));
        }
        next += bytes.len() as u32;
        reconstructed.extend_from_slice(bytes);
    }
    assert_eq!(next, SOURCE_END);
    assert_eq!(reconstructed.len(), SOURCE_BYTES);
    assert_eq!(ffta_rom::digest(&reconstructed), SOURCE_SHA256);
    for at in [0, POINTS_START - 1, SOURCE_END, u32::MAX] {
        assert_eq!(data.byte_at(at), None);
    }
    for index in [45, 46, 255] {
        assert_eq!(data.packed_point(0, index, 255), None);
    }
    // Preserve the five one-pixel-short endpoints; never force the destination.
    for (index, end) in [
        (3, [55, -16]),
        (5, [-31, 24]),
        (7, [47, -24]),
        (9, [-16, 47]),
        (11, [47, 24]),
    ] {
        let p = data.paths[index].points.last().unwrap();
        assert_eq!([p.x, p.y], end);
    }
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<WorldRoads>(&encoded).unwrap(),
        *data
    );
    fs::write(out.join("decoded.json"), &encoded).unwrap();
    let mut guards = 0;
    for case in 0..10 {
        let mut bad = data.clone();
        match case {
            0 => bad.headers_offset += 1,
            1 => {
                bad.paths.pop();
            }
            2 => bad.paths.push(bad.paths[0].clone()),
            3 => bad.paths.swap(0, 1),
            4 => bad.paths[1].index = 0,
            5 => bad.paths[0].point_count -= 1,
            6 => {
                bad.paths[0].points.pop();
            }
            7 => bad.paths[0].encoded_pointer = 0,
            8 => bad.paths[0].encoded_pointer += 2,
            _ => bad.paths[1].encoded_pointer = bad.paths[0].encoded_pointer,
        }
        assert!(bad.validate().is_err(), "shape{case}");
        assert!(bad.source_spans().is_err());
        guards += 1;
    }
    for case in 0..10 {
        let mut bad: Value = serde_json::from_slice(&encoded).unwrap();
        match case {
            0 => bad["extra"] = json!(1),
            1 => bad["paths"][0]["extra"] = json!(1),
            2 => bad["paths"][0]["points"][0]["extra"] = json!(1),
            3 => bad["paths"][0]["opaque"] = json!([0, 0]),
            4 => bad["paths"][0]["opaque"] = json!([0, 0, 0, 0]),
            5 => bad["paths"][0]["index"] = json!(256),
            6 => bad["paths"][0]["point_count"] = json!(256),
            7 => bad["paths"][0]["points"][0]["x"] = json!(128),
            8 => bad["paths"][0]["points"][0]["y"] = json!(-129),
            _ => bad["paths"][0]["encoded_pointer"] = json!(-1),
        }
        assert!(
            serde_json::from_value::<WorldRoads>(bad).is_err(),
            "json{case}"
        );
        guards += 1;
    }
    for case in 0..4 {
        let mut changed = data.clone();
        match case {
            0 => changed.paths[0].opaque = [1, 128, 255],
            1 => changed.paths[0].points[0] = Point { x: -128, y: 127 },
            2 => changed.paths[0].points[0] = Point { x: 0, y: 0 },
            _ => changed.paths[3].points.last_mut().unwrap().x = 56,
        }
        changed.validate().unwrap();
        assert!(changed.verify_source(&rom).is_err());
        let bytes: Vec<u8> = changed
            .source_spans()
            .unwrap()
            .into_iter()
            .flat_map(|s| s.1)
            .collect();
        assert_ne!(ffta_rom::digest(&bytes), SOURCE_SHA256);
        guards += 1;
    }
    // Count0 and count255 are structurally possible within a coherent source
    // partition. They are not facts about this exact image or guest caps.
    for count in [0, 255] {
        let mut changed = data.clone();
        let mut points: Vec<Point> = changed
            .paths
            .iter()
            .flat_map(|p| p.points.clone())
            .collect();
        let first: Vec<_> = points.drain(..count).collect();
        let mut at = POINTS_START;
        for (i, path) in changed.paths.iter_mut().enumerate() {
            path.points = if i == 0 {
                first.clone()
            } else {
                let n = if i == PATH_COUNT - 1 {
                    points.len()
                } else {
                    // Spread the remainder across the remaining full-byte counts.
                    points.len().div_ceil(PATH_COUNT - i).min(255)
                };
                points.drain(..n).collect()
            };
            path.point_count = path.points.len().try_into().unwrap();
            path.encoded_pointer = 0x08000000 + at;
            at += path.points.len() as u32 * 2;
        }
        assert!(points.is_empty());
        changed.validate().unwrap();
        assert!(changed.verify_source(&rom).is_err());
        assert_eq!(changed.packed_point(0, 0, 255), Some(0));
        guards += 1;
    }
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    let mut wrong = rom.clone();
    wrong[POINTS_START as usize] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    wrong.copy_from_slice(&rom);
    wrong[0x4dd0c] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    guards += 3;
    let probes: Vec<Probe> =
        serde_json::from_slice(&fs::read(root.join("independent-probes.json")).unwrap()).unwrap();
    let expected_coverage: BTreeSet<u32> =
        serde_json::from_slice(&fs::read(root.join("independent-coverage.json")).unwrap()).unwrap();
    let mut m = Machine::new(rom);
    m.bus.ewram.fill(0x5a);
    m.bus.iwram.fill(0x3c);
    m.bus.vram.fill(0xa5);
    m.bus.oam.fill(0xc3);
    m.bus.palette.fill(0x24);
    let mut covered = BTreeSet::new();
    let mut instructions = 0;
    for probe in &probes {
        let [a, b, c] = probe.args;
        match probe.entry {
            0x4dd0c if usize::from(b as u8) < PATH_COUNT => {
                assert_eq!(data.packed_point(a, b, c), Some(probe.expected))
            }
            0x4dd0c => assert_eq!(data.packed_point(a, b, c), None),
            0x4dd70 => assert_eq!(u32::from(reverse_direction(a, b)), probe.expected),
            0x4dd84 => assert_eq!(u32::from(base_node(a, b)), probe.expected),
            _ => unreachable!(),
        }
        instructions += original(&mut m, probe, &mut covered);
    }
    assert_eq!(covered, expected_coverage);
    assert_eq!(covered.len(), 4601);
    assert_eq!(ffta_rom::digest(&m.bus.rom), ffta_rom::EUROPE_SHA256);
    let report = json!({
        "status":"PASS-source-model-original-readers", "paths":45,"points":2188,
        "source_spans":46,"source_bytes":SOURCE_BYTES,"semantic_bytes":4601,"opaque_bytes":135,
        "guards":guards,"original_calls":probes.len(),"original_declared_bytes":covered.len(),
        "original_opaque_bytes":0,"instructions_coverage_only":instructions,
        "byte_alias_calls":probes.iter().filter(|p|p.kind=="byte-alias").count(),
        "reverse_calls":probes.iter().filter(|p|p.kind=="reverse").count(),
        "out_of_range_calls":probes.iter().filter(|p|p.kind=="out-of-range").count(),
        "outside_projection_calls":probes.iter().filter(|p|p.kind=="outside-projection").count(),
        "RAM_callee_stack_VRAM_OAM_palette_checked":true,
        "scope":"synthetic original-reader inputs only, not gameplay/native/state-clock/full ABI/facing/pathfinding/runtime",
        "schema_extractor_decoder_unchanged":[1,78,77],
        "Linux":"pending","USA":"pending","nativeWindows":"pending",
        "performance_measurements":false
    });
    fs::write(
        out.join("result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_WORLD_ROAD_MODEL paths45/points2188/spans46/source4736B/original4601semanticB/opaque135B guards{guards} original{} PASS", probes.len());
}
