//! One focused private Wine source oracle; synthetic call inputs, no game fixture.
use super::*;
use crate::Machine;
use serde_json::{json, Value};
use std::{collections::BTreeSet, fs, path::PathBuf};

const STACK: u32 = 0x03007e00;
const RETURN: u32 = 0x08000300;
const PARENT: u32 = 0x02004000;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Probe {
    entry: u32,
    first: u32,
    second: u32,
    expected: u32,
    loads: Vec<u32>,
    kind: String,
}

fn original(m: &mut Machine, p: &Probe, covered: &mut BTreeSet<u32>) -> usize {
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    m.cpu.regs[0] = p.first;
    m.cpu.regs[1] = p.second;
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.regs[15] = 0x08000000 + p.entry;
    m.cpu.cpsr = 0xbf;
    let preserved = m.cpu.regs[4..12].to_vec();
    let ew = m.bus.ewram.clone();
    let iw = m.bus.iwram.clone();
    let vram = m.bus.vram.clone();
    let oam = m.bus.oam.clone();
    let palette = m.bus.palette.clone();
    let mut loads = Vec::new();
    let mut instructions = 0;
    while m.cpu.regs[15] != RETURN {
        assert!(
            instructions < 128,
            "entry{:x} PC{:08x}",
            p.entry,
            m.cpu.regs[15]
        );
        let location = match m.cpu.regs[15] {
            0x08036ea6 => Some((1, 0)),
            0x08036eca => Some((1, 1)),
            0x0804dae6 => Some((1, 3)),
            0x0804daec => Some((1, 2)),
            0x0804daf4 => Some((1, 7)),
            0x0804dafa => Some((1, 6)),
            0x0804dafe => Some((1, 11)),
            0x0804db04 => Some((1, 10)),
            0x0804db08 => Some((1, 15)),
            0x0804db12 => Some((1, 14)),
            0x08046ed0 => Some((1, 2)),
            0x08046ed6 => Some((1, 3)),
            0x08046eea => Some((1, 6)),
            0x08046ef0 => Some((1, 7)),
            0x08046f04 => Some((1, 10)),
            0x08046f0a => Some((1, 11)),
            0x08046f1e => Some((2, 14)),
            0x08046f24 => Some((2, 15)),
            _ => None,
        };
        if let Some((reg, add)) = location {
            let at = m.cpu.regs[reg] + add - 0x08000000;
            loads.push(at);
            if (SOURCE..SOURCE + SOURCE_BYTES as u32).contains(&at) {
                covered.insert(at);
            }
        }
        m.bus.frame_ready = false;
        m.step();
        instructions += 1;
        assert!(m.cpu.regs[13] >= STACK - 12);
    }
    assert_eq!(
        loads, p.loads,
        "entry{:x}/{}->{}",
        p.entry, p.first, p.second
    );
    assert_eq!(
        m.cpu.regs[0], p.expected,
        "entry{:x}/{}->{}",
        p.entry, p.first, p.second
    );
    assert_eq!(m.cpu.regs[13], STACK);
    assert_eq!(&m.cpu.regs[4..12], preserved.as_slice());
    assert_eq!(m.bus.ewram, ew);
    let stack_bytes = match p.entry {
        0x4dad0 => 8,
        0x46eb0 => 12,
        _ => 0,
    };
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
#[ignore = "private EUR world-node model/original source readers; Wine only, Linux/USA pending"]
fn european_world_nodes_match_source_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_WORLD_NODE_MODEL_ROOT").unwrap());
    let out = root.join("windows-model-europe");
    assert!(!out.exists());
    fs::create_dir(&out).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let expected: WorldNodes =
        serde_json::from_slice(&fs::read(root.join("independent-model.json")).unwrap()).unwrap();
    assert_eq!(data, &expected);
    data.verify_source(&rom).unwrap();
    let spans = data.source_spans().unwrap();
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].0, SOURCE);
    assert_eq!(
        spans[0].1,
        rom[SOURCE as usize..SOURCE as usize + SOURCE_BYTES]
    );
    assert_eq!(spans[0].2, "world-node-records");
    assert_eq!(catalog.sources.len(), 1);
    let source = &catalog.sources[0];
    assert_eq!(
        (source.offset, source.length),
        (SOURCE, SOURCE_BYTES as u32)
    );
    assert_eq!(source.sha256, SOURCE_SHA256);
    assert_eq!(source.role, spans[0].2);
    assert_eq!(ffta_rom::digest(&spans[0].1), SOURCE_SHA256);
    for (i, &byte) in spans[0].1.iter().enumerate() {
        assert_eq!(data.byte_at(SOURCE + i as u32), Some(byte));
    }
    for at in [0, SOURCE - 1, SOURCE + SOURCE_BYTES as u32, u32::MAX] {
        assert_eq!(data.byte_at(at), None);
    }
    for index in [30, 31, 255] {
        assert_eq!(data.pixel_position(index), None);
        assert_eq!(data.path_to(index, index), None);
        assert_eq!(data.adjacent(index, index), None);
    }
    assert_eq!(data.path_to(1, 0), Some(1));
    assert_eq!(data.adjacent(1, 0), Some(true));
    assert_eq!(data.path_to(3, 0), Some(0));
    assert_eq!(data.adjacent(3, 0), Some(false));
    assert_eq!(data.path_to(3, 3), Some(255));
    assert_eq!(data.adjacent(3, 3), Some(true));
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<WorldNodes>(&encoded).unwrap(),
        *data
    );
    fs::write(out.join("decoded.json"), &encoded).unwrap();
    let mut guards = 0;
    for case in 0..5 {
        let mut bad = data.clone();
        match case {
            0 => bad.source_offset += 1,
            1 => {
                bad.nodes.pop();
            }
            2 => bad.nodes.push(bad.nodes[0].clone()),
            3 => bad.nodes.swap(0, 1),
            _ => bad.nodes[1].index = 0,
        }
        assert!(bad.validate().is_err());
        guards += 1;
    }
    for case in 0..10 {
        let mut bad: Value = serde_json::from_slice(&encoded).unwrap();
        match case {
            0 => bad["extra"] = json!(1),
            1 => bad["nodes"][0]["extra"] = json!(1),
            2 => bad["nodes"][0]["links"][0]["extra"] = json!(1),
            3 => {
                bad["nodes"][0]["links"].as_array_mut().unwrap().pop();
            }
            4 => bad["nodes"][0]["links"][0]["opaque"] = json!([0]),
            5 => bad["nodes"][0]["opaque_tail"] = json!([0, 0, 0]),
            6 => bad["nodes"][0]["x_tile"] = json!(256),
            7 => bad["nodes"][0]["links"][0]["path_id"] = json!(256),
            8 => bad["nodes"][0]["links"][0]["neighbor"] = json!(-1),
            _ => bad["nodes"][0]["index"] = json!(256),
        }
        assert!(serde_json::from_value::<WorldNodes>(bad).is_err());
        guards += 1;
    }
    // Structural values remain full bytes. Authenticity is a separate check;
    // do not invent max45/max29/opaque-zero policies from this exact image.
    for case in 0..5 {
        let mut changed = data.clone();
        match case {
            0 => changed.nodes[0].x_tile = 255,
            1 => changed.nodes[0].links[0].path_id = 255,
            2 => changed.nodes[0].links[0].neighbor = 255,
            3 => changed.nodes[0].links[0].opaque[1] = 255,
            _ => changed.nodes[0].opaque_tail[0] = 255,
        }
        changed.validate().unwrap();
        assert!(changed.verify_source(&rom).is_err());
        let spans = changed.source_spans().unwrap();
        assert_eq!(spans[0].1.len(), SOURCE_BYTES);
        assert_ne!(ffta_rom::digest(&spans[0].1), SOURCE_SHA256);
        guards += 1;
    }
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    let mut wrong = rom.clone();
    wrong[SOURCE as usize] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    wrong.copy_from_slice(&rom);
    wrong[0x36e90] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    guards += 3;
    let probes: Vec<Probe> =
        serde_json::from_slice(&fs::read(root.join("independent-probes.json")).unwrap()).unwrap();
    let expected_covered: BTreeSet<u32> =
        serde_json::from_slice(&fs::read(root.join("independent-coverage.json")).unwrap()).unwrap();
    let mut m = Machine::new(rom);
    m.bus.ewram.fill(0x5a);
    m.bus.iwram.fill(0x3c);
    m.bus.vram.fill(0xa5);
    m.bus.oam.fill(0xc3);
    m.bus.palette.fill(0x24);
    m.bus.iwram[0x2810..0x2814].copy_from_slice(&PARENT.to_le_bytes());
    m.bus.ewram[0x467c..0x4680].copy_from_slice(&(0x08000000 + SOURCE).to_le_bytes());
    let mut covered = BTreeSet::new();
    let mut instructions = 0;
    for p in &probes {
        if (p.first as u8 as usize) < NODE_COUNT {
            let expected = match p.entry {
                0x36e90 => u32::from(data.pixel_position(p.first as u8).unwrap()[0]),
                0x36eb4 => u32::from(data.pixel_position(p.first as u8).unwrap()[1]),
                0x4dad0 => u32::from(data.path_to(p.first as u8, p.second as u8).unwrap()),
                0x46eb0 => u32::from(data.adjacent(p.first as u8, p.second as u8).unwrap()),
                _ => unreachable!(),
            };
            assert_eq!(expected, p.expected);
        }
        instructions += original(&mut m, p, &mut covered);
    }
    assert_eq!(covered, expected_covered);
    assert_eq!(covered.len(), 300);
    assert_eq!(ffta_rom::digest(&m.bus.rom), ffta_rom::EUROPE_SHA256);
    let report = json!({
        "status":"PASS-source-model-original-readers", "nodes":30,"links":120,
        "source_spans":1,"source_bytes":600,"semantic_bytes":300,"opaque_bytes":300,
        "original_calls":probes.len(),"original_declared_bytes":covered.len(),
        "original_opaque_bytes":0,"byte_alias_calls":probes.iter().filter(|p|p.kind=="byte-alias").count(),
        "outside_projection_calls":probes.iter().filter(|p|p.kind=="outside-projection").count(),
        "guards":guards,"instructions_coverage_only":instructions,
        "RAM_callee_stack_VRAM_OAM_palette_checked":true,
        "scope":"synthetic original-reader call inputs only, not gameplay/state-clock/native/runtime/BFS/ordinary acceptance",
        "schema_extractor_decoder_unchanged":[1,77,76],"Linux":"pending","USA":"pending","nativeWindows":"pending",
        "performance_measurements":false
    });
    fs::write(
        out.join("result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_WORLD_NODE_MODEL nodes30/links120/source600B/original300semanticB/opaque300B guards{guards} original{} PASS", probes.len());
}
