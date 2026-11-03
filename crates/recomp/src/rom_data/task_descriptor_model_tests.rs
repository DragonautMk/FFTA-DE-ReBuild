//! One combined source-model/original-reader check; Wine only.
use super::*;
use gba_core::{rom_data::task_descriptors::*, Machine};
use serde_json::Value;
use std::collections::BTreeSet;

const MESSAGE: u32 = 0x02010000;
fn reset(m: &mut Machine, pc: u32) {
    m.bus.ewram.fill(0x5a);
    m.bus.iwram.fill(0xa5);
    m.cpu.regs = [0; 16];
    for n in 1..13 {
        m.cpu.regs[n] = 0x11110000 + n as u32;
    }
    m.cpu.regs[13] = 0x03007e00;
    m.cpu.regs[14] = 0x08000301;
    m.cpu.regs[15] = pc;
    m.cpu.cpsr = 0xbf;
    m.bus.reg_ie = 0;
    m.bus.ime = false;
}
fn until(m: &mut Machine, end: u32, pcs: &mut BTreeSet<u32>, reads: &mut BTreeSet<u32>) -> usize {
    let mut n = 0;
    while m.cpu.regs[15] != end {
        let pc = m.cpu.regs[15];
        assert!(n < 30, "pc={pc:08x} end={end:08x}");
        pcs.insert(pc);
        let access = match pc {
            0x080d7062 => Some((m.cpu.regs[0], 4)),
            0x080d7064 => Some((m.cpu.regs[0] + 4, 2)),
            0x080d707a => Some((m.cpu.regs[1], 4)),
            0x080d707c => Some((m.cpu.regs[1] + 4, 2)),
            0x080d7346 => Some((m.cpu.regs[0], 4)),
            0x080d7348 => Some((m.cpu.regs[0] + 4, 2)),
            0x080d70a2 => Some((m.cpu.regs[0] + 6, 2)),
            0x080d76b6 => Some((m.cpu.regs[1] + 6, 1)),
            _ => None,
        };
        if let Some((at, width)) = access {
            for i in 0..width {
                reads.insert(at - 0x08000000 + i);
            }
        }
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    n
}
fn word(rom: &[u8], at: u32) -> u32 {
    u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
}
fn half(rom: &[u8], at: u32) -> u16 {
    u16::from_le_bytes(rom[at as usize..at as usize + 2].try_into().unwrap())
}

#[test]
#[ignore = "private EUR task descriptor source/alias/metadata oracle; Wine only, Linux/USA pending"]
fn european_task_descriptor_model_matches_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_TASK_DESCRIPTOR_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let evidence: Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_TASK_DESCRIPTOR_EVIDENCE").unwrap()).unwrap(),
    )
    .unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let spans = data.source_spans().unwrap();
    assert_eq!(spans.len(), 3);
    let mut nulls = 0;
    for ((table, e), (offset, bytes, _)) in data
        .tables()
        .into_iter()
        .zip(evidence["tables"].as_array().unwrap())
        .zip(&spans)
    {
        assert_eq!(table.source_offset, e["offset"].as_u64().unwrap() as u32);
        let rows = e["rows"].as_array().unwrap();
        assert_eq!(table.entries.len(), rows.len());
        for (entry, row) in table.entries.iter().zip(rows) {
            assert_eq!(entry.ordinal, row["ordinal"].as_u64().unwrap() as u16);
            assert_eq!(
                entry.callback_word(),
                Some(row["callback_word"].as_u64().unwrap() as u32)
            );
            assert_eq!(
                entry.context_size,
                row["context_size"].as_u64().unwrap() as u16
            );
            assert_eq!(
                entry.metadata_raw,
                row["metadata_raw"].as_u64().unwrap() as u16
            );
            nulls += usize::from(entry.callback.is_none());
            assert_eq!(entry.byte(8), None);
            assert_eq!(entry.byte(usize::MAX), None);
        }
        assert_eq!(
            bytes,
            &rom[*offset as usize..*offset as usize + bytes.len()]
        );
        assert_eq!(
            gba_core::ffta_rom::digest(bytes),
            e["sha256"].as_str().unwrap()
        );
    }
    assert_eq!(nulls, 3);
    assert_eq!(spans.iter().map(|s| s.1.len()).sum::<usize>(), SOURCE_BYTES);
    assert_eq!(catalog.sources.len(), 3);
    for (source, (offset, bytes, role)) in catalog.sources.iter().zip(&spans) {
        assert_eq!(
            (source.offset, source.length),
            (*offset, bytes.len() as u32)
        );
        assert_eq!(source.role, *role);
        assert_eq!(source.sha256, gba_core::ffta_rom::digest(bytes));
    }
    for at in [0x3997a8, 0x39a490, 0, u32::MAX] {
        assert_eq!(data.descriptor_at(at), None);
        assert_eq!(data.byte_at(at), None);
    }
    assert_eq!(data.descriptor_at(0x3997b1), None);
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<TaskDescriptors>(&encoded).unwrap(),
        *data
    );
    for case in 0..10 {
        let mut bad = data.clone();
        match case {
            0 => bad.primary.source_offset += 8,
            1 => bad.auxiliary.source_offset += 8,
            2 => bad.script.source_offset += 8,
            3 => {
                bad.primary.entries.pop();
            }
            4 => bad.auxiliary.entries.push(bad.auxiliary.entries[0].clone()),
            5 => bad.script.entries.clear(),
            6 => bad.primary.entries[0].ordinal = 1,
            7 => bad.primary.entries[0].callback.as_mut().unwrap().rom_offset |= 1,
            8 => bad.primary.entries[0].callback.as_mut().unwrap().rom_offset = 0x1000000,
            _ => bad.script.entries.swap(0, 1),
        }
        assert!(bad.validate().is_err(), "shape guard{case}");
    }
    for case in 0..4 {
        let mut bad = data.clone();
        match case {
            0 => bad.primary.entries[0].callback.as_mut().unwrap().rom_offset += 2,
            1 => bad.auxiliary.entries[0].context_size ^= 1,
            2 => bad.script.entries[0].metadata_raw ^= 0x8000,
            _ => bad.primary.entries[0].callback = None,
        }
        bad.validate().unwrap();
        assert!(bad.verify_source(&rom).is_err(), "source guard{case}");
    }
    for path in ["top", "table", "entry", "thumb"] {
        let mut unknown = serde_json::to_value(data).unwrap();
        let value = match path {
            "top" => &mut unknown,
            "table" => &mut unknown["primary"],
            "entry" => &mut unknown["primary"]["entries"][0],
            _ => &mut unknown["primary"]["entries"][0]["callback"],
        };
        value["unknown"] = true.into();
        assert!(serde_json::from_value::<TaskDescriptors>(unknown).is_err());
    }
    assert!(serde_json::from_slice::<TaskDescriptors>(&encoded[..encoded.len() - 1]).is_err());
    for field in ["context_size", "metadata_raw"] {
        let mut bad = serde_json::to_value(data).unwrap();
        bad["primary"]["entries"][0][field] = 65536.into();
        assert!(serde_json::from_value::<TaskDescriptors>(bad).is_err());
    }
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    let mut wrong = rom.clone();
    wrong[0] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());

    let mut m = Machine::new(rom.clone());
    let mut pcs = BTreeSet::new();
    let mut reads = BTreeSet::new();
    let mut instructions = 0;
    // No allocator/callback invocation: exercise each original table load prefix.
    for id in (0u32..=412).chain([413, 65535]) {
        let at = primary_source_offset(id);
        let (start, end, scratch) = if id <= 341 {
            (0x080d705a, 0x080d7066, 0)
        } else {
            (0x080d7070, 0x080d707e, 1)
        };
        reset(&mut m, start);
        m.cpu.regs[4] = id;
        let regs = m.cpu.regs;
        let ew = m.bus.ewram.clone();
        let iw = m.bus.iwram.clone();
        instructions += until(&mut m, end, &mut pcs, &mut reads);
        assert_eq!(m.cpu.regs[scratch], 0x08000000 + at, "primary ID{id}");
        assert_eq!(m.cpu.regs[3], word(&rom, at));
        assert_eq!(m.cpu.regs[2], u32::from(half(&rom, at + 4)));
        assert_eq!(m.cpu.regs[4..15], regs[4..15]);
        assert_eq!(m.bus.ewram, ew);
        assert_eq!(m.bus.iwram, iw);
        if (1..=412).contains(&id) {
            let entry = data.descriptor_at(at).unwrap();
            assert_eq!(entry.callback_word(), Some(m.cpu.regs[3]));
            assert_eq!(u32::from(entry.context_size), m.cpu.regs[2]);
        } else {
            assert_eq!(data.descriptor_at(at), None);
        }
    }
    // Script catalog plus physical alias endpoints and raw indices outside it.
    for id in (363u32..=413).chain([0, 1, 2, 342, 343, 362, 414, 65535]) {
        let at = secondary_source_offset(id);
        reset(&mut m, 0x080d733c);
        m.cpu.regs[0] = id;
        let regs = m.cpu.regs;
        let ew = m.bus.ewram.clone();
        let iw = m.bus.iwram.clone();
        instructions += until(&mut m, 0x080d734a, &mut pcs, &mut reads);
        assert_eq!(m.cpu.regs[0], 0x08000000 + at, "secondary ID{id}");
        assert_eq!(m.cpu.regs[1], word(&rom, at));
        assert_eq!(m.cpu.regs[2], u32::from(half(&rom, at + 4)));
        assert_eq!(m.cpu.regs[3..15], regs[3..15]);
        assert_eq!(m.bus.ewram, ew);
        assert_eq!(m.bus.iwram, iw);
        if (2..=413).contains(&id) {
            let entry = data.descriptor_at(at).unwrap();
            assert_eq!(entry.callback_word(), Some(m.cpu.regs[1]));
            assert_eq!(u32::from(entry.context_size), m.cpu.regs[2]);
        } else {
            assert_eq!(data.descriptor_at(at), None);
        }
    }
    for id in (1u32..=341).chain([0, 342, 65535, 0xbeef0001, 0xffff0155]) {
        let at = 0x3997b0u32.wrapping_add(u32::from(id as u16).wrapping_sub(1).wrapping_mul(8));
        reset(&mut m, 0x080d709a);
        m.cpu.regs[2] = MESSAGE;
        m.cpu.regs[6] = u32::from(id as u16);
        let mut ew = m.bus.ewram.clone();
        let iw = m.bus.iwram.clone();
        let regs = m.cpu.regs;
        let metadata = half(&rom, at + 6);
        ew[0x10016..0x10018].copy_from_slice(&metadata.to_le_bytes());
        instructions += until(&mut m, 0x080d70a6, &mut pcs, &mut reads);
        assert_eq!(m.cpu.regs[0], u32::from(metadata));
        assert_eq!(m.cpu.regs[1], 0x083997b0);
        assert_eq!(m.cpu.regs[2..15], regs[2..15]);
        assert_eq!(m.bus.ewram, ew);
        assert_eq!(m.bus.iwram, iw);
        if let Some(entry) = data.descriptor_at(at) {
            assert_eq!(entry.metadata_raw, metadata);
        }
        reset(&mut m, 0x080d76ac);
        m.cpu.regs[1] = id;
        let regs = m.cpu.regs;
        let ew = m.bus.ewram.clone();
        let iw = m.bus.iwram.clone();
        instructions += until(&mut m, 0x080d76b8, &mut pcs, &mut reads);
        assert_eq!(m.cpu.regs[0], u32::from(metadata as u8));
        assert_eq!(m.cpu.regs[1], 0x08000000 + at);
        assert_eq!(m.cpu.regs[2..15], regs[2..15]);
        assert_eq!(m.bus.ewram, ew);
        assert_eq!(m.bus.iwram, iw);
    }
    for raw_id in [0u32, 341, 342, 345, 412, 413, 65535, 0xbeef0001, 0xffff0155] {
        for (start, end, dest) in [(0x080d7020, 0x080d7024, 4), (0x080d732c, 0x080d7330, 0)] {
            reset(&mut m, start);
            m.cpu.regs[0] = raw_id;
            instructions += until(&mut m, end, &mut pcs, &mut reads);
            assert_eq!(m.cpu.regs[dest], u32::from(raw_id as u16));
        }
        reset(&mut m, 0x080d7030);
        m.cpu.regs[4] = u32::from(raw_id as u16);
        let end = if raw_id as u16 <= 341 {
            0x080d7036
        } else {
            0x080d7070
        };
        instructions += until(&mut m, end, &mut pcs, &mut reads);
    }
    let owned: BTreeSet<_> = reads
        .iter()
        .copied()
        .filter(|a| (0x3997b0..0x39a490).contains(a))
        .collect();
    let expected: BTreeSet<_> = (0x3997b0..0x39a490)
        .filter(|a| *a < 0x39a258 || a % 8 < 6)
        .collect();
    // Raw metadata ID342 aliases the first suffix row: two extra bytes are read.
    let mut expected = expected;
    expected.extend([0x39a25e, 0x39a25f]);
    assert_eq!(owned, expected);
    fs::write(root.join("task-descriptors.json"), encoded).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "records":412,"tables":[341,20,51],"source_bytes":SOURCE_BYTES,"source_ranges":3,
        "source_roundtrip_and_independent_hashes":true,"null_records":nulls,
        "original_primary_load_prefixes":415,"original_secondary_load_prefixes":59,
        "original_metadata_halfword_prefixes":346,"original_metadata_byte_prefixes":346,
        "original_low16_prefixes":18,"original_branch_prefixes":9,
        "owned_distinct_bytes_at_original_loadsites":owned.len(),
        "suffix_metadata_bytes_preserved_unread_in_declared_probes":140,
        "raw_underflow_outside_and_cross_catalog_aliases":true,
        "guards":{"shape":10,"source_edit":4,"json":7,"rom_identity":2},
        "instructions_coverage":instructions,"pcs":pcs.len(),
        "scope":"field/index/metadata prefixes only; not whole allocator/callback/scheduler/native/stateclock/ordinary ABI",
        "managed":"e47-d46 unchanged; USAe8-d7","linux":"pending","usa":"pending","performance_measurements":false
    })).unwrap()).unwrap();
}
