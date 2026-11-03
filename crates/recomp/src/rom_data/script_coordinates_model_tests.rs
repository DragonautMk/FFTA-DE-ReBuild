//! One EUR source oracle; no replacement movement task, height sampler or guest index guard.
use super::*;
use gba_core::{rom_data::script_coordinates::*, Machine};
use serde_json::Value;
use std::collections::BTreeSet;

fn reset(m: &mut Machine, pc: u32) {
    m.cpu.regs = [0; 16];
    for n in 4..12 {
        m.cpu.regs[n] = 0x14004000 + n as u32 * 0x101;
    }
    m.cpu.regs[13] = 0x03007e00;
    m.cpu.regs[14] = 0x08000301;
    m.cpu.regs[15] = pc;
    m.cpu.cpsr = 0xbf;
    m.bus.reg_ie = 0;
    m.bus.ime = false;
}
fn until(
    m: &mut Machine,
    end: u32,
    limit: usize,
    pcs: &mut BTreeSet<u32>,
    reads: &mut BTreeSet<u32>,
) -> usize {
    let mut steps = 0;
    while m.cpu.regs[15] != end {
        let pc = m.cpu.regs[15];
        assert!(steps < limit, "pc{pc:08x} end{end:08x}");
        pcs.insert(pc);
        let access = match pc {
            0x08128928 => Some((m.cpu.regs[0] & !3, 4)),
            0x0812893a | 0x08128948 | 0x08128974 => Some((m.cpu.regs[5], 1)),
            0x0812893c | 0x0812894a | 0x08128976 => Some((m.cpu.regs[5] + 1, 1)),
            0x08128958 => Some((m.cpu.regs[6], 1)),
            0x0812895c => Some((m.cpu.regs[6] + 1, 1)),
            _ => None,
        };
        if let Some((at, n)) = access {
            for a in at..at + n {
                if (0x08000000 + SOURCE_OFFSET..0x08000000 + SOURCE_END).contains(&a) {
                    reads.insert(a - 0x08000000);
                }
            }
        }
        m.bus.frame_ready = false;
        m.step();
        steps += 1;
    }
    steps
}
fn put32(m: &mut Machine, at: usize, value: u32) {
    m.bus.ewram[at..at + 4].copy_from_slice(&value.to_le_bytes());
}

#[test]
#[ignore = "private EUR coordinate-path source/model oracle; Wine only, Linux/USA pending"]
fn european_script_coordinates_match_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_COORDINATE_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let evidence: Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_COORDINATE_EVIDENCE").unwrap()).unwrap(),
    )
    .unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let prior =
        PathBuf::from(std::env::var_os("FFTA_STATE_TEST_COORDINATE_PRIOR_DATASET").unwrap());
    for name in ["script-archive.json", "script-command-directory.json"] {
        assert_eq!(
            gba_core::ffta_rom::digest(&fs::read(prior.join(name)).unwrap()),
            evidence["dependency_file_sha256"][name].as_str().unwrap()
        );
    }
    let archive: gba_core::rom_data::script_archive::ScriptArchive =
        serde_json::from_slice(&fs::read(prior.join("script-archive.json")).unwrap()).unwrap();
    let commands: gba_core::rom_data::script_commands::ScriptCommands =
        serde_json::from_slice(&fs::read(prior.join("script-command-directory.json")).unwrap())
            .unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    data.validate_dependencies(&archive, &commands).unwrap();
    data.verify_source(&rom).unwrap();
    assert_eq!(data.paths.len(), 210);
    assert_eq!(data.script_links.len(), 68);
    assert_eq!(catalog.sources.len(), 1);
    assert_eq!(catalog.sources[0].length, 3056);
    assert_eq!(
        catalog.sources[0].sha256,
        evidence["source_ranges"][0]["sha256"].as_str().unwrap()
    );
    let spans = data.source_spans().unwrap();
    assert_eq!(spans[0].1, rom[SOURCE_OFFSET as usize..SOURCE_END as usize]);
    for at in SOURCE_OFFSET..SOURCE_END {
        assert_eq!(data.byte_at(at), Some(rom[at as usize]));
    }
    assert_eq!(data.byte_at(SOURCE_OFFSET - 1), None);
    assert_eq!(data.byte_at(SOURCE_END), None);
    for (p, e) in data.paths.iter().zip(evidence["paths"].as_array().unwrap()) {
        assert_eq!(u64::from(p.id), e["id"].as_u64().unwrap());
        assert_eq!(
            u64::from(p.source_offset),
            e["source_offset"].as_u64().unwrap()
        );
        assert_eq!(p.points.len(), e["count"].as_u64().unwrap() as usize);
        assert_eq!(serde_json::to_value(&p.points).unwrap(), e["points"]);
        assert_eq!(
            serde_json::to_value(&p.alignment_after).unwrap(),
            e["alignment_after"]
        );
    }
    assert_eq!(
        serde_json::to_value(&data.script_links).unwrap(),
        evidence["producer_references"]
    );
    for field in ["halfwords", "words"] {
        for e in evidence[field].as_array().unwrap() {
            let at = e["offset"].as_u64().unwrap() as usize;
            let actual = if field == "halfwords" {
                u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap()))
            } else {
                u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
            };
            assert_eq!(u64::from(actual), e["value"].as_u64().unwrap());
        }
    }
    assert_eq!(
        serde_json::from_slice::<ScriptCoordinates>(&serde_json::to_vec(data).unwrap()).unwrap(),
        *data
    );
    let mut guards = Vec::new();
    macro_rules! invalid {
        ($name:expr, $edit:expr) => {{
            let mut v = data.clone();
            ($edit)(&mut v);
            assert!(v.validate().is_err(), "{}", $name);
            guards.push($name);
        }};
    }
    invalid!("root", |v: &mut ScriptCoordinates| v.source_offset += 4);
    invalid!("directory count", |v: &mut ScriptCoordinates| {
        v.paths.pop();
    });
    invalid!("path ID", |v: &mut ScriptCoordinates| v.paths[0].id = 1);
    invalid!("path offset", |v: &mut ScriptCoordinates| v.paths[1]
        .source_offset +=
        4);
    invalid!("point count", |v: &mut ScriptCoordinates| {
        v.paths[0].points.pop();
    });
    invalid!("empty list", |v: &mut ScriptCoordinates| v.paths[0]
        .points
        .clear());
    invalid!("alignment value", |v: &mut ScriptCoordinates| v.paths[1]
        .alignment_after[0] =
        0);
    invalid!("alignment length", |v: &mut ScriptCoordinates| v.paths[0]
        .alignment_after
        .push(255));
    invalid!("link count", |v: &mut ScriptCoordinates| {
        v.script_links.pop();
    });
    invalid!("link opcode", |v: &mut ScriptCoordinates| v.script_links
        [0]
    .script_opcode =
        0x46);
    invalid!("link target", |v: &mut ScriptCoordinates| v.script_links
        [0]
    .path_id = 210);
    invalid!("link order", |v: &mut ScriptCoordinates| {
        v.script_links.swap(0, 1);
    });
    let mut coherent = data.clone();
    coherent.paths[0].points[0].x = 255;
    coherent.paths[0].points[0].y = 254;
    coherent.validate().unwrap();
    assert!(coherent.verify_source(&rom).is_err());
    guards.push("coherent coordinate provenance/full-u8 shape");
    let mut coherent = data.clone();
    coherent.script_links[0].path_id = (coherent.script_links[0].path_id + 1) % 210;
    coherent.validate().unwrap();
    assert!(coherent.validate_dependencies(&archive, &commands).is_err());
    guards.push("coherent producer dependency");
    let mut changed = archive.clone();
    let mut ignored = false;
    for p in &mut changed.programs {
        for i in &mut p.instructions {
            if i.command == 0x44 {
                i.parameters[2] = 255;
                ignored = true;
            }
        }
    }
    assert!(ignored);
    data.validate_dependencies(&changed, &commands).unwrap();
    let mut changed = archive.clone();
    changed
        .programs
        .iter_mut()
        .flat_map(|p| &mut p.instructions)
        .find(|i| i.command == 0x44)
        .unwrap()
        .parameters[1] = 210;
    assert!(data.validate_dependencies(&changed, &commands).is_err());
    guards.push("low8 producer dependency");
    let mut changed = archive.clone();
    changed
        .programs
        .iter_mut()
        .flat_map(|p| &mut p.instructions)
        .find(|i| i.command == 0x45)
        .unwrap()
        .parameters[1] = 1;
    assert!(data.validate_dependencies(&changed, &commands).is_err());
    guards.push("u16 producer dependency");
    for callback in [false, true] {
        let mut changed = commands.clone();
        if callback {
            changed.commands[0x44].callback.rom_offset += 2;
        } else {
            changed.commands[0x45].advance += 1;
        }
        assert!(data.validate_dependencies(&archive, &changed).is_err());
        guards.push(if callback {
            "command callback"
        } else {
            "command advance"
        });
    }
    let json = serde_json::to_value(data).unwrap();
    for at in 0..4 {
        let mut v = json.clone();
        match at {
            0 => {
                v["unknown"] = 1.into();
            }
            1 => {
                v["paths"][0]["unknown"] = 1.into();
            }
            2 => {
                v["paths"][0]["points"][0]["unknown"] = 1.into();
            }
            _ => {
                v["script_links"][0]["unknown"] = 1.into();
            }
        }
        assert!(serde_json::from_value::<ScriptCoordinates>(v).is_err());
        guards.push("strict JSON object");
    }
    assert!(Catalog::from_rom(&[]).is_err());
    guards.push("ROM length");
    let mut wrong = rom.clone();
    wrong[SOURCE_OFFSET as usize] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    guards.push("ROM identity");

    // Controlled reader arguments and a live scratch map, never a gameplay/save fixture.
    let mut m = Machine::new(rom.clone());
    put32(&mut m, 0x159d4, 0x02020000);
    put32(&mut m, 0x7f14, 0x02030000);
    m.bus.ewram[0x7f18] = 16;
    for y in 0..16usize {
        for x in 0..16usize {
            m.bus.ewram[0x30000 + 2 * (y * 16 + x)] = ((x * 7 + y * 13 + 17) & 255) as u8;
        }
    }
    let mut pcs = BTreeSet::new();
    let mut reads = BTreeSet::new();
    let mut steps = 0;
    let mut check_path = |m: &mut Machine, id: usize, raw_destination: u32| {
        let path = &data.paths[id];
        let base = 0x20000 + 0x58 + (raw_destination as u8 as usize) * 132;
        m.bus.ewram[base..base + 132].fill(0xa5);
        let mut expected = m.bus.ewram.clone();
        for (n, p) in path.points.iter().enumerate() {
            let height = m.bus.ewram[0x30000 + 2 * (usize::from(p.y) * 16 + usize::from(p.x))];
            expected[base + n * 4..base + n * 4 + 4].copy_from_slice(&[p.x, p.y, height, 0]);
        }
        expected[base + 128..base + 132].copy_from_slice(&(path.points.len() as u32).to_le_bytes());
        reset(m, 0x08128914);
        let saved = m.cpu.regs[4..12].to_vec();
        m.cpu.regs[0] = id as u32;
        m.cpu.regs[1] = raw_destination;
        steps += until(m, 0x08000300, 5000, &mut pcs, &mut reads);
        assert_eq!(m.cpu.regs[0], path.points.len() as u32 & 255);
        assert_eq!(m.cpu.regs[4..12], saved);
        assert_eq!(m.cpu.regs[13], 0x03007e00);
        assert_eq!(m.bus.ewram, expected, "path{id} dest{raw_destination}");
    };
    for id in 0..210 {
        check_path(&mut m, id, (id % 16) as u32);
    }
    // Same source pair reads a changed live map byte on both boundary lists.
    for id in [0, 209] {
        let p = &data.paths[id].points[0];
        m.bus.ewram[0x30000 + 2 * (usize::from(p.y) * 16 + usize::from(p.x))] ^= 0x80;
        check_path(&mut m, id, 0x10f);
    }
    drop(check_path);
    assert_eq!(reads.len(), 2972); // All index/count/XY bytes;84padding bytes intentionally unread.
    let mut raw_prefixes = 0;
    for raw in [210, 255, 65535, 0x10001, u32::MAX] {
        reset(&mut m, 0x08128914);
        m.cpu.regs[0] = raw;
        m.cpu.regs[1] = 1;
        let at = index_source_offset(raw) as usize;
        let relative = u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
        steps += until(&mut m, 0x0812892c, 16, &mut pcs, &mut reads);
        assert_eq!(
            m.cpu.regs[5],
            (0x08000000 + SOURCE_OFFSET).wrapping_add(relative)
        );
        raw_prefixes += 1;
    }
    for raw in [0, 1, 15, 31, 255, 256, 511, u32::MAX] {
        reset(&mut m, 0x08125768);
        m.cpu.regs[0] = raw;
        steps += until(&mut m, 0x08000300, 16, &mut pcs, &mut reads);
        assert_eq!(m.cpu.regs[0], 0x02020058 + u32::from(raw as u8) * 132);
        raw_prefixes += 1;
    }
    let mut producer_prefixes = 0;
    for (path, ignored, actor, flags) in [
        (0, 0, 0, 1),
        (72, 1, 31, 0x10),
        (209, 127, 255, 0xf1),
        (210, 255, 16, 0xff),
        (255, 128, 1, 0x20),
    ] {
        m.bus.ewram[0x11000..0x11005].copy_from_slice(&[0x44, actor, path, ignored, flags]);
        put32(&mut m, 0x10f08, 0x02018d00);
        reset(&mut m, 0x08127090);
        m.cpu.regs[0] = 0x02010f00;
        m.cpu.regs[1] = 0x02011000;
        steps += until(&mut m, 0x0812b700, 70, &mut pcs, &mut reads);
        let at = (m.cpu.regs[2] - 0x03000000) as usize;
        let half =
            |off| u16::from_le_bytes(m.bus.iwram[at + off..at + off + 2].try_into().unwrap());
        assert_eq!(half(0x14), u16::from(actor));
        assert_eq!(half(0x16), u16::from(path));
        assert_eq!(half(0x18), u16::from(flags & 15));
        assert_eq!(
            u32::from_le_bytes(m.bus.iwram[at + 8..at + 12].try_into().unwrap()),
            u32::from(flags >> 4)
        );
        assert_eq!(m.cpu.regs[0], 0x0812acb5);
        assert_eq!(m.cpu.regs[1], 16);
        assert_eq!(m.cpu.regs[3], 0x02018d00);
        producer_prefixes += 1;
    }
    for (id, destination) in [(0u16, 0u8), (209, 15), (210, 255), (65535, 1)] {
        let [lo, hi] = id.to_le_bytes();
        m.bus.ewram[0x11000..0x11004].copy_from_slice(&[0x45, lo, hi, destination]);
        reset(&mut m, 0x081270b8);
        m.cpu.regs[1] = 0x02011000;
        steps += until(&mut m, 0x08128914, 16, &mut pcs, &mut reads);
        assert_eq!(m.cpu.regs[0], u32::from(id));
        assert_eq!(m.cpu.regs[1], u32::from(destination));
        producer_prefixes += 1;
        m.bus.ewram[0x11116..0x11118].copy_from_slice(&id.to_le_bytes());
        reset(&mut m, 0x0812ad14);
        m.cpu.regs[6] = 0x02011100;
        steps += until(&mut m, 0x08128914, 6, &mut pcs, &mut reads);
        assert_eq!(m.cpu.regs[0], u32::from(id));
        assert_eq!(m.cpu.regs[1], 15);
        producer_prefixes += 1;
    }
    let source_bytes = reads.len();
    let observed_padding = data
        .paths
        .iter()
        .map(|path| {
            let first = path.source_offset + 2 + path.points.len() as u32 * 2;
            (first..first + path.alignment_after.len() as u32)
                .filter(|at| reads.contains(at))
                .count()
        })
        .sum::<usize>();
    assert_eq!(source_bytes, 2972 + observed_padding);

    fs::write(
        root.join("script-coordinates.json"),
        serde_json::to_vec_pretty(data).unwrap(),
    )
    .unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS source-model only","source_bytes":3056,"original_distinct_source_bytes":source_bytes,"original_path_distinct_source_bytes":2972,"raw_prefix_observed_padding_bytes":observed_padding,"preserved_unread_padding_bytes":84-observed_padding,"paths":210,"points":856,"script_links":68,"referenced_paths":56,"guards":guards,"original_path_calls":212,"raw_index_destination_prefixes":raw_prefixes,"producer_prefixes":producer_prefixes,"instructions":steps,"instruction_pcs":pcs,"coverage_only":true,"whole_allocator_movement_task_native_fullstate_clock_ordinary":false,"linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
}
