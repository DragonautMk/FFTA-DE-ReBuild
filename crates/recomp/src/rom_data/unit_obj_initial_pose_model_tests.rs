//! One new source-model/original-reader Wine check; publication remains separate.
use super::*;
use gba_core::{rom_data::unit_obj_initial_pose::*, Machine};
use serde_json::{json, Value};
use std::collections::BTreeSet;

const TASK: usize = 0x10000;
const POOL: usize = 0x11000;
const STACK: u32 = 0x03007e00;
#[derive(Default)]
struct Coverage {
    reads: BTreeSet<u32>,
    pcs: BTreeSet<u32>,
    instructions: usize,
}
fn put16(b: &mut [u8], at: usize, n: u16) {
    b[at..at + 2].copy_from_slice(&n.to_le_bytes());
}
fn put32(b: &mut [u8], at: usize, n: u32) {
    b[at..at + 4].copy_from_slice(&n.to_le_bytes());
}
fn get16(b: &[u8], at: usize) -> u16 {
    u16::from_le_bytes(b[at..at + 2].try_into().unwrap())
}
fn get32(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(b[at..at + 4].try_into().unwrap())
}
fn fresh(m: &mut Machine) {
    m.bus.ewram.fill(0x5a);
    m.bus.iwram.fill(0);
    m.bus.vram.fill(0xa5);
    m.bus.reg_ie = 0;
    m.bus.ime = false;
}
fn cpu(m: &mut Machine, pc: u32, args: &[u32]) {
    m.cpu.regs = [0; 16];
    for n in 4..12 {
        m.cpu.regs[n] = 0x41414000 + n as u32 * 0x101;
    }
    m.cpu.regs[..args.len()].copy_from_slice(args);
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = 0x08000301;
    m.cpu.regs[15] = pc;
    m.cpu.cpsr = 0xbf;
}
fn until(m: &mut Machine, end: u32, c: &mut Coverage) {
    let mut n = 0;
    while m.cpu.regs[15] != end {
        let pc = m.cpu.regs[15];
        assert!(
            n < 20000,
            "unit OBJ original {pc:08x} did not reach {end:08x}"
        );
        c.pcs.insert(pc);
        let read = match pc {
            0x0802217e => Some((m.cpu.regs[0], 4)),
            0x08022e36 => Some((m.cpu.regs[4], 4)),
            0x080227a8 => Some((m.cpu.regs[0], 4)),
            0x08022e4c => Some((m.cpu.regs[4], 2)),
            0x080224b0 => Some((m.cpu.regs[0] + 8, 1)),
            0x0802225c => Some((m.cpu.regs[0] + 9, 1)),
            0x080222b0 => Some((m.cpu.regs[1] + 4, 4)),
            0x080222c0 | 0x080222ce => Some((m.cpu.regs[3], 4)),
            0x080221de | 0x08022240 => Some((m.cpu.regs[4], 1)),
            0x080221e0 | 0x08022242 => Some((m.cpu.regs[4] + 1, 1)),
            0x080221ea => Some((m.cpu.regs[3] + 1, 1)),
            0x080221f2 | 0x08022216 => Some((m.cpu.regs[3] + 3, 1)),
            0x080223ae => Some((m.cpu.regs[2] + 11, 1)),
            0x080223b4 => Some((m.cpu.regs[2] + 17, 1)),
            0x080223c0 => Some((m.cpu.regs[2] + 18, 1)),
            0x080223ce => Some((m.cpu.regs[0] + 12, 2)),
            0x080223d2 => Some((m.cpu.regs[0] + 14, 2)),
            _ => None,
        };
        if let Some((at, len)) = read {
            if (0x08000000..0x09000000).contains(&at) {
                c.reads.extend((at..at + len).map(|a| a - 0x08000000));
            }
        }
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    c.instructions += n;
}
fn call(m: &mut Machine, pc: u32, args: &[u32], c: &mut Coverage) {
    cpu(m, pc, args);
    let before = m.cpu.regs;
    until(m, 0x08000300, c);
    assert_eq!(m.cpu.regs[4..12], before[4..12]);
    assert_eq!(m.cpu.regs[13], STACK);
}
fn ram_guards(
    m: &Machine,
    ew: &[u8],
    iw: &[u8],
    ew_ranges: &[(usize, usize)],
    iw_ranges: &[(usize, usize)],
) {
    let mut actual = m.bus.ewram.clone();
    let mut expected = ew.to_vec();
    for &(at, len) in ew_ranges {
        actual[at..at + len].fill(0);
        expected[at..at + len].fill(0);
    }
    assert_eq!(actual, expected);
    let mut actual = m.bus.iwram.clone();
    let mut expected = iw.to_vec();
    for &(at, len) in iw_ranges {
        actual[at..at + len].fill(0);
        expected[at..at + len].fill(0);
    }
    actual[0x7d00..0x7e00].fill(0);
    expected[0x7d00..0x7e00].fill(0);
    assert_eq!(actual, expected);
}

#[test]
#[ignore = "private new EUR unit initial-pose source oracle; Wine only, Linux/USA pending"]
fn european_unit_initial_pose_matches_original_readers_and_uploads() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_UNIT_OBJ_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let plan: Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_UNIT_OBJ_PLAN").unwrap()).unwrap(),
    )
    .unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    data.verify_source(&rom).unwrap();
    let spans = data.source_spans().unwrap();
    assert_eq!(spans.len(), 415);
    assert_eq!(spans.iter().map(|s| s.1.len()).sum::<usize>(), 146358);
    for (((at, bytes, role), source), independent) in spans
        .iter()
        .zip(&catalog.sources)
        .zip(plan["source_ranges"].as_array().unwrap())
    {
        assert_eq!(u64::from(*at), independent["offset"].as_u64().unwrap());
        assert_eq!(bytes.len() as u64, independent["length"].as_u64().unwrap());
        assert_eq!(role, independent["role"].as_str().unwrap());
        assert_eq!(source.sha256, independent["sha256"].as_str().unwrap());
        assert_eq!(
            (source.offset, source.length as usize, &source.role),
            (*at, bytes.len(), role)
        );
        assert_eq!(bytes, &rom[*at as usize..*at as usize + bytes.len()]);
    }
    for (binding, independent) in data
        .bindings
        .iter()
        .zip(plan["sequences"].as_array().unwrap())
    {
        assert_eq!(
            u64::from(binding.sprite_id),
            independent["sprite_id"].as_u64().unwrap()
        );
        assert_eq!(
            u64::from(binding.index_source),
            independent["index_source"].as_u64().unwrap()
        );
        assert_eq!(
            u64::from(binding.sequence_source),
            independent["sequence_source"].as_u64().unwrap()
        );
        let sequence = data
            .sequences
            .iter()
            .find(|s| s.source_offset == binding.sequence_source)
            .unwrap();
        assert_eq!(
            sequence.opaque_header as u64,
            independent["opaque_header"].as_u64().unwrap()
        );
        for (frame, expected) in sequence
            .frames
            .iter()
            .zip(independent["frames"].as_array().unwrap())
        {
            assert_eq!(
                frame.pixel_offset as u64,
                expected["pixel_offset"].as_u64().unwrap()
            );
            assert_eq!(
                frame.object_offset as u64,
                expected["object_offset"].as_u64().unwrap()
            );
            assert_eq!(
                frame.duration as u64,
                expected["duration"].as_u64().unwrap()
            );
            assert_eq!(frame.operation as u64, expected["kind"].as_u64().unwrap());
            assert_eq!(
                serde_json::to_value(frame.bytes_0a_13).unwrap(),
                expected["bytes_0a_13"]
            );
        }
    }
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<UnitObjInitialPose>(&encoded).unwrap(),
        *data
    );
    fs::write(root.join("unit-obj-initial-pose.json"), &encoded).unwrap();
    let mut guards = 0;
    for variant in 0..9 {
        let mut v = serde_json::to_value(data).unwrap();
        match variant {
            0 => v["unknown"] = true.into(),
            1 => v["bindings"][0]["unknown"] = true.into(),
            2 => v["pixels"][0]["tiles"][0]["indices"][0][0] = 16.into(),
            3 => {
                v["bindings"].as_array_mut().unwrap().pop();
            }
            4 => v["bindings"][1]["sprite_id"] = v["bindings"][0]["sprite_id"].clone(),
            5 => v["sequences"][0]["frames"][0]["object_offset"] = 0xffffffffu32.into(),
            6 => v["sequences"][0]["frames"][0]["pixel_offset"] = 0xffffffffu32.into(),
            7 => v["sequences"][0]["frames"][0]["operation"] = 2.into(),
            _ => v["pixels"][0]["source_offset"] = 1.into(),
        }
        assert!(serde_json::from_value::<UnitObjInitialPose>(v)
            .map_or(true, |x| x.source_spans().is_err()));
        guards += 1;
    }
    let mut edited = data.clone();
    edited.sequences[0].opaque_header ^= 0x8000;
    edited.validate().unwrap();
    assert!(edited.verify_source(&rom).is_err());
    guards += 1;
    let mut edited = data.clone();
    edited.pixels[0].tiles[0].indices[0][0] ^= 15;
    edited.validate().unwrap();
    assert!(edited.verify_source(&rom).is_err());
    guards += 1;
    let mut m = Machine::new(rom.clone());
    let mut c = Coverage::default();
    // Full original reset path reads directory, first cell and count, while
    // independently predicted task bytes/untouched RAM and ABI are checked.
    for binding in &data.bindings {
        fresh(&mut m);
        let mut ew = m.bus.ewram.clone();
        let iw = m.bus.iwram.clone();
        let sequence = data
            .sequences
            .iter()
            .find(|s| s.source_offset == binding.sequence_source)
            .unwrap();
        put32(&mut ew, TASK, 0x45);
        put16(&mut ew, TASK + 6, binding.sprite_id);
        put16(&mut ew, TASK + 8, 0);
        put16(&mut ew, TASK + 10, 0);
        put16(&mut ew, TASK + 12, 0);
        put16(&mut ew, TASK + 14, sequence.frames.len() as u16);
        ew[TASK + 0x10] = 0;
        put32(&mut ew, TASK + 0x20, u32::MAX);
        put32(&mut ew, TASK + 0x34, 0x08000004 + binding.sequence_source);
        put32(&mut ew, TASK + 0x38, 0x08000004 + binding.sequence_source);
        call(
            &mut m,
            0x08022e20,
            &[0x02000000 + TASK as u32, u32::from(binding.sprite_id), 0, 0],
            &mut c,
        );
        assert_eq!(m.cpu.regs[0], 1);
        assert_eq!(m.bus.ewram, ew);
        ram_guards(&m, &ew, &iw, &[], &[]);
    }
    // The actual heap constructor's rooted prefix, before allocator/live state.
    fresh(&mut m);
    let b = &data.bindings[0];
    put32(&mut m.bus.iwram, 0x7e00, u32::from(b.sprite_id));
    let ew = m.bus.ewram.clone();
    let iw = m.bus.iwram.clone();
    cpu(
        &mut m,
        0x08022788,
        &[0x02000000 + POOL as u32, 0x02000000 + TASK as u32, 0, 20],
    );
    until(&mut m, 0x080227aa, &mut c);
    assert_eq!(m.cpu.regs[0], 0x08000000 + b.sequence_source);
    assert_eq!(m.cpu.regs[9], 0x08000000 + b.index_source);
    assert_eq!(m.cpu.regs[13], STACK - 28);
    ram_guards(&m, &ew, &iw, &[], &[]);
    let mut transfers = 0;
    let mut dma_sources = BTreeSet::new();
    for sequence in &data.sequences {
        for (i, frame) in sequence.frames.iter().enumerate() {
            fresh(&mut m);
            m.bus.ewram[TASK..TASK + 0x48].fill(0);
            m.bus.ewram[POOL..POOL + 0x1d4].fill(0);
            put32(&mut m.bus.ewram, TASK, 0x45);
            put16(&mut m.bus.ewram, TASK + 0xe, 1);
            put16(&mut m.bus.ewram, TASK + 0x12, 0x10);
            put32(&mut m.bus.ewram, TASK + 0x20, u32::MAX);
            put32(
                &mut m.bus.ewram,
                TASK + 0x38,
                0x08000004 + sequence.source_offset + i as u32 * 20,
            );
            put16(&mut m.bus.ewram, POOL + 0x1cc, 4);
            let ew = m.bus.ewram.clone();
            let iw = m.bus.iwram.clone();
            let object = data
                .objects
                .iter()
                .find(|o| o.source_offset == OBJECT_BASE + frame.object_offset)
                .unwrap();
            let tile_count = transfer_tiles(object);
            let length = transfer_bytes(tile_count);
            let block = data
                .pixels
                .iter()
                .find(|p| p.source_offset == PIXEL_BASE + frame.pixel_offset)
                .unwrap();
            call(
                &mut m,
                0x08022400,
                &[0x02000000 + TASK as u32, 0x02000000 + POOL as u32],
                &mut c,
            );
            assert_eq!(
                get32(&m.bus.ewram, POOL + 4),
                0x08000000 + block.source_offset
            );
            assert_eq!(get32(&m.bus.ewram, POOL + 8), 0x06010200);
            assert_eq!(get32(&m.bus.ewram, POOL + 12), 0x84000000 | length / 4);
            assert_eq!(get16(&m.bus.ewram, TASK + 0x24), tile_count as u16);
            assert_eq!(get16(&m.bus.ewram, TASK + 10), u16::from(frame.duration));
            assert_eq!(get32(&m.bus.iwram, 0x2bcc), 0x02000000 + POOL as u32);
            ram_guards(&m, &ew, &iw, &[(TASK, 0x48), (POOL, 0x1d4)], &[(0x2bcc, 4)]);
            // Original queue executor selects DMA3, programs its real registers
            // and waits for completion. Compare all VRAM, not a prefix/hash.
            call(&mut m, 0x08000800, &[], &mut c);
            let mut expected = vec![0xa5; m.bus.vram.len()];
            expected[0x10200..0x10200 + length as usize].copy_from_slice(&block.bytes().unwrap());
            assert_eq!(m.bus.vram, expected);
            assert_eq!(get32(&m.bus.iwram, 0x2bcc), 0);
            ram_guards(&m, &ew, &iw, &[(TASK, 0x48), (POOL, 0x1d4)], &[(0x2bcc, 4)]);
            dma_sources.extend(block.source_offset..block.source_offset + length);
            transfers += 1;
        }
    }
    assert_eq!(transfers, 477);
    assert_eq!(dma_sources.len(), 135168);
    // All original shape/size branches, including raw reserved shape3.
    for shape in 0..4 {
        for size in 0..4 {
            fresh(&mut m);
            put16(&mut m.bus.ewram, TASK, 1);
            put16(&mut m.bus.ewram, TASK + 2, shape << 14);
            put16(&mut m.bus.ewram, TASK + 4, size << 14);
            let list = gba_core::rom_data::obj_lists::List::decode(
                TASK as u32,
                &m.bus.ewram[TASK..TASK + 8],
            )
            .unwrap();
            let ew = m.bus.ewram.clone();
            let iw = m.bus.iwram.clone();
            call(&mut m, 0x080221d4, &[0x02000000 + TASK as u32], &mut c);
            assert_eq!(m.cpu.regs[0], transfer_tiles(&list));
            ram_guards(&m, &ew, &iw, &[], &[]);
        }
    }
    let result = json!({"source_ranges":415,"source_bytes":146358,"json_guards":guards,
        "reset_paths":121,"constructor_prefixes":1,"frame_uploads":transfers,
        "distinct_dma_source_bytes":dma_sources.len(),"shape_size_probes":16,
        "original_observed_load_bytes_without_dma":c.reads.len(),"original_pcs":c.pcs.len(),
        "instructions_coverage_only":c.instructions,"linux":"pending","usa":"pending",
        "runtime_publication":"not implemented","performance_measurements":false});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    println!("unit initial pose: {result}");
}
