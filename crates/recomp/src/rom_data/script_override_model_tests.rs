//! Reduced conditional EUR script override source oracle; prior proofs stay closed.
use super::*;
use gba_core::rom_data::unit_obj_initial_pose::{
    transfer_bytes, transfer_tiles, OBJECT_BASE, PIXEL_BASE,
};
use gba_core::{rom_data::unit_obj_script_override::*, Machine};
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
            0x080223f4 => Some((m.cpu.regs[1] + 10, 1)),
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
#[ignore = "private EUR conditional forced-current script-override source oracle; Wine only, Linux/USA pending"]
fn european_script_override_poses_match_new_original_sources_and_uploads() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_SCRIPT_OVERRIDE_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let plan: Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_SCRIPT_OVERRIDE_PLAN").unwrap()).unwrap(),
    )
    .unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let normal = gba_core::rom_data::unit_obj_normal_battle::Catalog::from_rom(&rom)
        .unwrap()
        .decoded;
    let alternate = gba_core::rom_data::unit_obj_alternate_battle::Catalog::from_rom(&rom)
        .unwrap()
        .decoded;
    let spans = data.owned_spans(&normal, &alternate).unwrap();
    assert_eq!(spans.len(), 40);
    assert_eq!(spans.iter().map(|s| s.1.len()).sum::<usize>(), 1050);
    let mut expected_bindings = plan["bindings"].clone();
    for value in expected_bindings.as_array_mut().unwrap() {
        let kind = value
            .as_object_mut()
            .unwrap()
            .remove("target_view")
            .unwrap();
        assert!(kind == "null" || kind == "sequence");
    }
    assert_eq!(
        serde_json::to_value(&data.bindings).unwrap(),
        expected_bindings
    );
    assert_eq!(
        serde_json::to_value(&data.producers).unwrap(),
        plan["producer_pairs"]
    );
    assert_eq!(
        data.producers
            .iter()
            .map(|p| p.script_sources.len())
            .sum::<usize>(),
        334
    );

    let full = data.source_spans().unwrap();
    assert_eq!(full.iter().map(|s| s.1.len()).sum::<usize>(), 11722);
    for (at, bytes, _) in &full {
        assert_eq!(bytes, &rom[*at as usize..*at as usize + bytes.len()]);
    }
    assert_eq!(
        gba_core::ffta_rom::digest(
            &full
                .iter()
                .flat_map(|s| s.1.iter().copied())
                .collect::<Vec<_>>()
        ),
        "6d0850006ab9c7225eb5930af6cd5f0d9f8737c84024239ff251c04ca6d8e323"
    );
    let owned = |at: u32| {
        let i = spans.partition_point(|s| s.0 <= at);
        i > 0 && at < spans[i - 1].0 + spans[i - 1].1.len() as u32
    };
    for (((at, bytes, role), source), independent) in spans
        .iter()
        .zip(&catalog.sources)
        .zip(plan["new_spans_detail"].as_array().unwrap())
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
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<UnitObjScriptOverride>(&encoded).unwrap(),
        *data
    );
    fs::write(root.join("unit-obj-script-override.json"), &encoded).unwrap();
    let mut guards = 0;
    for variant in 0..13 {
        let mut v = serde_json::to_value(data).unwrap();
        match variant {
            0 => v["unknown"] = true.into(),
            1 => v["bindings"][0]["unknown"] = true.into(),
            2 => v["pixels"][0]["tiles"][0]["indices"][0][0] = 16.into(),
            3 => {
                v["bindings"].as_array_mut().unwrap().pop();
            }
            4 => v["bindings"][1] = v["bindings"][0].clone(),
            5 => v["pixels"][0]["source_offset"] = 1.into(),
            6 => v["bindings"][0]["sequence_source"] = 0xffffffffu32.into(),
            7 => v["producers"][0]["unknown"] = true.into(),
            8 => {
                v["producers"][0]["script_sources"]
                    .as_array_mut()
                    .unwrap()
                    .pop();
            }
            9 => v["producers"][0]["assignment_source"] = 0.into(),
            10 => v["producers"][1] = v["producers"][0].clone(),
            11 => v["producers"][0]["mode"] = "unknown".into(),
            _ => v["producers"][0]["sprite_id"] = 230.into(),
        }
        assert!(serde_json::from_value::<UnitObjScriptOverride>(v)
            .map_or(true, |x| x.source_spans().is_err()));
        guards += 1;
    }
    for variant in 0..2 {
        let mut edited = data.clone();
        if variant == 0 {
            edited
                .sequences
                .iter_mut()
                .find(|s| owned(s.source_offset))
                .unwrap()
                .opaque_header ^= 0x8000;
        } else {
            let frame = edited
                .sequences
                .iter_mut()
                .flat_map(|s| &mut s.frames)
                .find(|f| f.operation != 1)
                .unwrap();
            frame.word00 ^= 0x80000000;
        }
        edited.validate().unwrap();
        assert!(edited.verify_source(&rom).is_err());
        guards += 1;
    }
    let scripts = gba_core::rom_data::script_archive::Catalog::from_rom(&rom)
        .unwrap()
        .decoded;
    let mut changed_scripts = scripts.clone();
    let instruction = changed_scripts
        .programs
        .iter_mut()
        .flat_map(|p| &mut p.instructions)
        .find(|i| i.command == 49 && i.parameters[0] >= 128)
        .unwrap();
    instruction.parameters[1] ^= 1;
    changed_scripts.validate().unwrap();
    assert!(data.validate_producers(&changed_scripts).is_err());
    guards += 1;
    let mut changed_reference = data.clone();
    changed_reference.producers[0].script_sources[0] += 1;
    changed_reference.producers[13].script_sources[0] += 1;
    changed_reference.validate().unwrap();
    assert!(changed_reference.validate_producers(&scripts).is_err());
    guards += 1;
    // Both previous typed views are checked before subtracting their union.
    let mut changed_normal = normal.clone();
    changed_normal
        .sequences
        .iter_mut()
        .find(|s| {
            data.sequences
                .iter()
                .any(|x| x.source_offset == s.source_offset)
        })
        .unwrap()
        .opaque_header ^= 0x8000;
    changed_normal.validate().unwrap();
    assert!(data.owned_spans(&changed_normal, &alternate).is_err());
    guards += 1;
    let mut changed_alternate = alternate.clone();
    changed_alternate
        .sequences
        .iter_mut()
        .find(|s| {
            data.sequences
                .iter()
                .any(|x| x.source_offset == s.source_offset)
        })
        .unwrap()
        .opaque_header ^= 0x8000;
    changed_alternate.validate().unwrap();
    assert!(data.owned_spans(&normal, &changed_alternate).is_err());
    guards += 1;
    let mut m = Machine::new(rom.clone());
    let mut c = Coverage::default();
    let mut nulls = 0;
    let mut reset_sources = BTreeSet::new();
    // One representative per new physical cell, with both original facing
    // aliases. Skip already accepted initial/equipment cells and sequences.
    let mut cells = BTreeSet::new();
    let mut prefixes = 0;
    let mut constructor_parts = BTreeSet::new();
    for binding in &data.bindings {
        if !owned(binding.index_source) || !cells.insert(binding.index_source) {
            continue;
        }
        constructor_parts.insert(binding.part);
        let base = binding.animation & !3;
        let directions = if binding.animation & 3 == 0 {
            [0, 3]
        } else {
            [1, 2]
        };
        for direction in directions {
            prefixes += 1;
            fresh(&mut m);
            put32(&mut m.bus.iwram, 0x7e00, u32::from(binding.sprite_id));
            put32(&mut m.bus.iwram, 0x7e04, u32::from(base + direction));
            put32(&mut m.bus.iwram, 0x7e08, u32::from(binding.part));
            let ew = m.bus.ewram.clone();
            let iw = m.bus.iwram.clone();
            cpu(
                &mut m,
                0x08022788,
                &[0x02000000 + POOL as u32, 0x02000000 + TASK as u32, 0, 20],
            );
            until(&mut m, 0x080227aa, &mut c);
            assert_eq!(
                m.cpu.regs[0],
                binding.sequence_source.map_or(0, |a| 0x08000000 + a)
            );
            assert_eq!(
                m.cpu.regs[9],
                0x08000000 + binding.index_source - 4 * u32::from(binding.part)
            );
            assert_eq!(m.cpu.regs[13], STACK - 28);
            ram_guards(&m, &ew, &iw, &[], &[]);
        }
        if binding
            .sequence_source
            .is_some_and(|source| !owned(source) || !reset_sources.insert(source))
        {
            continue;
        }
        fresh(&mut m);
        let mut ew = m.bus.ewram.clone();
        let iw = m.bus.iwram.clone();
        if let Some(source) = binding.sequence_source {
            let seq = data
                .sequences
                .iter()
                .find(|s| s.source_offset == source)
                .unwrap();
            put32(&mut ew, TASK, 0x45);
            put16(&mut ew, TASK + 6, binding.sprite_id);
            put16(&mut ew, TASK + 8, binding.animation);
            put16(&mut ew, TASK + 10, 0);
            put16(&mut ew, TASK + 12, 0);
            put16(&mut ew, TASK + 14, seq.frames.len() as u16);
            ew[TASK + 0x10] = 0;
            put32(&mut ew, TASK + 0x20, u32::MAX);
            put32(&mut ew, TASK + 0x34, 0x08000004 + source);
            put32(&mut ew, TASK + 0x38, 0x08000004 + source);
        } else {
            nulls += 1;
        }
        call(
            &mut m,
            0x08022e20,
            &[
                0x02000000 + TASK as u32,
                u32::from(binding.sprite_id),
                u32::from(binding.animation),
                u32::from(binding.part),
            ],
            &mut c,
        );
        assert_eq!(m.cpu.regs[0], u32::from(binding.sequence_source.is_some()));
        assert_eq!(m.bus.ewram, ew);
        ram_guards(&m, &ew, &iw, &[], &[]);
    }
    assert_eq!(cells.len(), 96);
    assert_eq!(prefixes, 192);
    assert_eq!(constructor_parts, BTreeSet::from([0, 1, 2]));
    assert_eq!(nulls, 90);
    assert_eq!(reset_sources.len(), 6);
    let mut commands = Vec::new();
    let mut transfers = BTreeSet::new();
    let mut dma_sources = BTreeSet::new();
    for sequence in &data.sequences {
        if !owned(sequence.source_offset) {
            continue;
        }
        for (i, frame) in sequence.frames.iter().enumerate() {
            let source = sequence.source_offset + 4 + i as u32 * 20;
            if frame.operation != 1 {
                assert_eq!(
                    frame.operation, 2,
                    "only two new argument-copy commands are source coverage"
                );
                commands.push(source);
                fresh(&mut m);
                m.bus.ewram[TASK..TASK + 0x48].fill(0);
                put32(&mut m.bus.ewram, TASK, 0x45);
                put32(&mut m.bus.ewram, TASK + 0x28, 0x13579bdf);
                put32(&mut m.bus.ewram, TASK + 0x38, 0x08000000 + source);
                let mut expected = m.bus.ewram.clone();
                let iw = m.bus.iwram.clone();
                expected[TASK + 0x18..TASK + 0x1c].copy_from_slice(&frame.bytes_0a_13[2..6]);
                call(
                    &mut m,
                    0x08022254,
                    &[0x02000000 + TASK as u32, 0x02000000 + POOL as u32],
                    &mut c,
                );
                assert_eq!(m.bus.ewram, expected);
                ram_guards(&m, &expected, &iw, &[], &[]);
                continue;
            }
            let pixel_at = PIXEL_BASE + frame.word00;
            if !owned(pixel_at) || !transfers.insert(pixel_at) {
                continue;
            }
            fresh(&mut m);
            m.bus.ewram[TASK..TASK + 0x48].fill(0);
            m.bus.ewram[POOL..POOL + 0x1d4].fill(0);
            put32(&mut m.bus.ewram, TASK, 0x45);
            put16(&mut m.bus.ewram, TASK + 0xe, 1);
            put16(&mut m.bus.ewram, TASK + 0x12, 0x10);
            put32(&mut m.bus.ewram, TASK + 0x20, u32::MAX);
            put32(&mut m.bus.ewram, TASK + 0x38, 0x08000000 + source);
            put16(&mut m.bus.ewram, POOL + 0x1cc, 4);
            let ew = m.bus.ewram.clone();
            let iw = m.bus.iwram.clone();
            let object = data
                .objects
                .iter()
                .find(|o| o.source_offset == OBJECT_BASE + frame.word04)
                .unwrap();
            let tile_count = transfer_tiles(object);
            let length = transfer_bytes(tile_count);
            let block = data
                .pixels
                .iter()
                .find(|p| p.source_offset == pixel_at)
                .unwrap();
            call(
                &mut m,
                0x08022400,
                &[0x02000000 + TASK as u32, 0x02000000 + POOL as u32],
                &mut c,
            );
            assert_eq!(get32(&m.bus.ewram, POOL + 4), 0x08000000 + pixel_at);
            assert_eq!(get32(&m.bus.ewram, POOL + 8), 0x06010200);
            assert_eq!(get32(&m.bus.ewram, POOL + 12), 0x84000000 | length / 4);
            assert_eq!(get16(&m.bus.ewram, TASK + 0x24), tile_count as u16);
            assert_eq!(get16(&m.bus.ewram, TASK + 10), u16::from(frame.duration));
            ram_guards(&m, &ew, &iw, &[(TASK, 0x48), (POOL, 0x1d4)], &[(0x2bcc, 4)]);
            call(&mut m, 0x08000800, &[], &mut c);
            let mut expected = vec![0xa5; m.bus.vram.len()];
            expected[0x10200..0x10200 + length as usize].copy_from_slice(&block.bytes().unwrap());
            assert_eq!(m.bus.vram, expected);
            assert_eq!(get32(&m.bus.iwram, 0x2bcc), 0);
            ram_guards(&m, &ew, &iw, &[(TASK, 0x48), (POOL, 0x1d4)], &[(0x2bcc, 4)]);
            dma_sources.extend(pixel_at..pixel_at + length);
        }
    }
    assert_eq!(transfers.len(), 2);
    assert_eq!(dma_sources.len(), 256);
    assert_eq!(commands.len(), 2);
    let result = json!({"source_ranges":40,"source_bytes":1050,"full_graph_bytes":11722,"script_references":167,"conditional_reference_witnesses":334,"producer_pairs":26,"bindings":156,
        "json_guards":guards,"constructor_prefixes":prefixes,"physical_new_cells":cells.len(),
        "constructor_parts":[0,1,2],"null_reset_paths":nulls,"sequence_reset_paths":reset_sources.len(),
        "sequences":28,"frames":140,"command_probes":commands.len(),"frame_uploads":transfers.len(),
        "distinct_dma_source_bytes":dma_sources.len(),"original_observed_load_bytes_without_dma":c.reads.len(),
        "original_pcs":c.pcs.len(),"instructions_coverage_only":c.instructions,"linux":"pending","usa":"pending",
        "runtime_publication":"not implemented","previous_owner_original_acceptance_replayed":false,
        "performance_measurements":false});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    println!("script override OBJ: {result}");
}
