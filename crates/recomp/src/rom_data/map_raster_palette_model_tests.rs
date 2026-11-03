//! One EUR/Wine source oracle; original copies and live controls remain authoritative.
use super::*;
use gba_core::{rom_data::map_raster_palettes::*, Machine};
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Default)]
struct Coverage {
    pcs: BTreeSet<u32>,
    reads: BTreeSet<u32>,
    copies: Vec<(u32, u16)>,
    instructions: usize,
}
fn half(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes(bytes[at..at + 2].try_into().unwrap())
}
fn put16(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn put32(bytes: &mut [u8], at: usize, value: u32) {
    bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
}
fn fresh(m: &mut Machine) {
    m.bus.ewram.fill(0x5a);
    m.bus.iwram.fill(0xa5);
    m.bus.reg_ie = 0;
    m.bus.ime = false;
}
fn reset_cpu(m: &mut Machine, pc: u32, a: u32, b: u32) {
    m.cpu.regs = [0; 16];
    for n in 4..12 {
        m.cpu.regs[n] = 0x14104000 + n as u32 * 0x101;
    }
    m.cpu.regs[0] = a;
    m.cpu.regs[1] = b;
    m.cpu.regs[13] = 0x03007e00;
    m.cpu.regs[14] = 0x08000301;
    m.cpu.regs[15] = pc;
    m.cpu.cpsr = 0xbf;
}
fn until(m: &mut Machine, end: u32, coverage: &mut Coverage) {
    let mut n = 0;
    while m.cpu.regs[15] != end {
        let pc = m.cpu.regs[15];
        assert!(n < 2000, "original PC {pc:08x} expected {end:08x}");
        coverage.pcs.insert(pc);
        let access = match pc {
            0x0801eb18 => Some((m.cpu.regs[1], 2)),
            0x0801ee1a => Some((m.cpu.regs[0], 2)),
            0x0801eb46 => Some((m.cpu.regs[5], 2)),
            0x0801eb4a | 0x0801eb50 => Some((m.cpu.regs[5] + 4, 2)),
            0x0801eb4e => Some((m.cpu.regs[5] + 2, 2)),
            0x0801eb6a => Some((m.cpu.regs[7], 2)),
            0x0801ee42 => Some((m.cpu.regs[2], 2)),
            _ => None,
        };
        if let Some((at, len)) = access {
            coverage
                .reads
                .extend((at..at + len).map(|a| a.wrapping_sub(0x08000000)));
        }
        let copy = if pc == 0x08145230 {
            assert_eq!(m.cpu.regs[2], 0xa1);
            assert_eq!(m.cpu.regs[1], 0x02008390);
            let at = m.cpu.regs[0] - 0x08000000;
            let bytes = m.bus.rom[at as usize..at as usize + 322].to_vec();
            coverage.reads.extend(at..at + 322);
            coverage.copies.push((at + 320, half(&bytes, 320)));
            Some(bytes)
        } else {
            None
        };
        m.bus.frame_ready = false;
        m.step();
        if let Some(bytes) = copy {
            // Observe the complete original HLE copy before the caller overwrites word160.
            assert_eq!(m.bus.ewram[0x8390..0x84d2], bytes);
        }
        n += 1;
    }
    coverage.instructions += n;
}
fn call(m: &mut Machine, pc: u32, a: u32, b: u32, coverage: &mut Coverage) {
    reset_cpu(m, pc, a, b);
    let regs = m.cpu.regs;
    until(m, 0x08000300, coverage);
    assert_eq!(m.cpu.regs[4..12], regs[4..12]);
    assert_eq!(m.cpu.regs[13], regs[13]);
}
fn assert_ram(m: &Machine, ew: &[u8], iw: &[u8]) {
    assert_eq!(m.bus.ewram, ew);
    let mut actual = m.bus.iwram.clone();
    let mut expected = iw.to_vec();
    actual[0x7d00..0x7e00].fill(0);
    expected[0x7d00..0x7e00].fill(0);
    assert_eq!(actual, expected);
}
fn expect_colors(ew: &mut [u8], p: &RasterPalette) {
    for (i, &word) in p
        .colors
        .iter()
        .chain(std::iter::once(&p.colors[0]))
        .enumerate()
    {
        put16(ew, 0x8390 + 2 * i, word);
    }
}
fn expect_init(ew: &mut [u8], p: &RasterPalette, deferred: bool) {
    let pointer = 0x08000000 + p.source_offset + 6;
    if deferred {
        ew[0x7f26] = 1;
        put32(ew, 0x7f28, pointer);
    } else {
        expect_colors(ew, p);
    }
    put16(ew, 0x7f24, p.header.color_count);
    put16(ew, 0x7f2c, p.header.range_start);
    put16(
        ew,
        0x7f2e,
        p.header
            .range_length
            .wrapping_add(p.header.range_start)
            .wrapping_sub(1),
    );
    put32(ew, 0x7f30, pointer);
    put32(
        ew,
        0x7f34,
        pointer + u32::from(p.header.color_count) * 2 - 320,
    );
    put16(ew, 0x7f22, p.id);
}

#[test]
#[ignore = "private EUR raster palette source/original-reader-copy oracle; Wine only, Linux/USA pending"]
fn european_map_raster_palettes_match_original_readers_and_copies() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_RASTER_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let plan: Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_RASTER_PLAN").unwrap()).unwrap(),
    )
    .unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let descriptors = gba_core::rom_data::bg_descriptors::Catalog::from_rom(&rom)
        .unwrap()
        .decoded;
    data.validate_dependencies(&descriptors).unwrap();
    data.verify_source(&rom).unwrap();
    assert_eq!(catalog.sources.len(), 33);
    let spans = data.source_spans().unwrap();
    for ((s, (offset, bytes, role)), independent) in catalog
        .sources
        .iter()
        .zip(&spans)
        .zip(plan["sources"].as_array().unwrap())
    {
        assert_eq!(u64::from(s.offset), independent["offset"].as_u64().unwrap());
        assert_eq!(u64::from(s.length), independent["length"].as_u64().unwrap());
        assert_eq!(s.sha256, independent["sha256"].as_str().unwrap());
        assert_eq!(s.role, independent["role"].as_str().unwrap());
        assert_eq!(
            (s.offset, s.length as usize, &s.role),
            (*offset, bytes.len(), role)
        );
        assert_eq!(
            bytes,
            &rom[*offset as usize..*offset as usize + bytes.len()]
        );
        assert_eq!(gba_core::ffta_rom::digest(bytes), s.sha256);
        for (i, &byte) in bytes.iter().enumerate() {
            assert_eq!(data.byte_at(*offset + i as u32), Some(byte));
        }
    }
    assert_eq!(data.byte_at(SOURCE_OFFSET - 1), None);
    assert_eq!(data.byte_at(SOURCE_END), None);
    assert_eq!(data.byte_at(u32::MAX), None);
    for (p, independent) in data
        .programs
        .iter()
        .zip(plan["programs"].as_array().unwrap())
    {
        assert_eq!(u64::from(p.id), independent["id"].as_u64().unwrap());
        assert_eq!(
            u64::from(p.source_offset - SOURCE_OFFSET),
            independent["relative_source"].as_u64().unwrap()
        );
        assert_eq!(
            serde_json::to_value(p.header.words()).unwrap(),
            independent["header"]
        );
        assert_eq!(
            serde_json::to_value(&p.colors).unwrap(),
            independent["colors"]
        );
        assert_eq!(p.byte(RECORD_BYTES), None);
        assert_eq!(p.byte(usize::MAX), None);
    }
    for link in plan["descriptor_links"].as_array().unwrap() {
        let i = link["descriptor"].as_u64().unwrap() as usize;
        let field = link["field"].as_u64().unwrap() as usize - 84;
        let id = link["program"].as_u64().unwrap() as u16;
        assert_eq!(data.descriptor_programs[i][field], id);
        assert_eq!(
            u64::from(data.programs[usize::from(id)].source_offset),
            link["source_offset"].as_u64().unwrap()
        );
    }
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<MapRasterPalettes>(&encoded).unwrap(),
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
    invalid!("source root", |v: &mut MapRasterPalettes| v
        .source_offset +=
        2);
    invalid!("program count", |v: &mut MapRasterPalettes| {
        v.programs.pop();
    });
    invalid!("program ID", |v: &mut MapRasterPalettes| v.programs[0].id =
        1);
    invalid!("source gap", |v: &mut MapRasterPalettes| v.programs[1]
        .source_offset +=
        2);
    invalid!("source overlap", |v: &mut MapRasterPalettes| v.programs
        [1]
    .source_offset -=
        2);
    invalid!("header count", |v: &mut MapRasterPalettes| v.programs[0]
        .header
        .color_count =
        159);
    invalid!("color extent", |v: &mut MapRasterPalettes| {
        v.programs[31].colors.pop();
    });
    invalid!("extra boundary word", |v: &mut MapRasterPalettes| v
        .programs[31]
        .colors
        .push(0x13c));
    invalid!("descriptor count", |v: &mut MapRasterPalettes| {
        v.descriptor_programs.pop();
    });
    invalid!("descriptor target", |v: &mut MapRasterPalettes| v
        .descriptor_programs[0][0] =
        32);
    let mut relinked = data.clone();
    relinked.descriptor_programs[0][0] = 1;
    relinked.validate().unwrap();
    assert!(relinked.validate_dependencies(&descriptors).is_err());
    guards.push("actual descriptor dependency");
    for field in ["extra", "programs"] {
        let mut v = serde_json::to_value(data).unwrap();
        if field == "extra" {
            v["extra"] = 1.into();
        } else {
            v["programs"][0]["header"]["extra"] = 1.into();
        }
        assert!(serde_json::from_value::<MapRasterPalettes>(v).is_err());
        guards.push("unknown JSON field");
    }
    assert!(Catalog::from_rom(&[]).is_err());
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    guards.push("source identity/length");

    let mut coverage = Coverage::default();
    let mut m = Machine::new(rom.clone());
    for p in &data.programs {
        for deferred in [false, true] {
            fresh(&mut m);
            put16(&mut m.bus.ewram, 0x7f22, 0);
            let mut ew = m.bus.ewram.clone();
            let iw = m.bus.iwram.clone();
            expect_init(&mut ew, p, deferred);
            call(
                &mut m,
                0x0801eaf0,
                0x12340000 + u32::from(p.id),
                if deferred { 1 } else { 0x100 },
                &mut coverage,
            );
            assert_eq!(m.cpu.regs[0], u32::from(p.colors[0]));
            assert_ram(&m, &ew, &iw);
            if deferred && matches!(p.id, 0 | 31) {
                m.bus.ewram[0x7f20] = 1;
                let mut ew = m.bus.ewram.clone();
                let mut iw = m.bus.iwram.clone();
                expect_colors(&mut ew, p);
                ew[0x7f26] = 0;
                put16(&mut iw, 0x3860, p.colors[0]);
                call(&mut m, 0x0801eed8, 0, 0, &mut coverage);
                assert_ram(&m, &ew, &iw);
            }
        }
        fresh(&mut m);
        put16(&mut m.bus.ewram, 0x7f22, 0);
        let ew = m.bus.ewram.clone();
        let iw = m.bus.iwram.clone();
        call(&mut m, 0x0801edec, u32::from(p.id), 0, &mut coverage);
        assert_eq!(m.cpu.regs[0], 0x08000000 + p.source_offset + 6);
        assert_ram(&m, &ew, &iw);
    }
    // Full pointer getter uses signed raw16, whereas the initializer prefix uses unsigned16.
    let raw_cases = [
        (32, 0),
        (255, 0),
        (0x7fff, 31),
        (0x8000, 0),
        (0xfffe, 0),
        (0xffff, 31),
        (0xcafeffff, 0xffff),
        (0x12340001, 0),
        (0xffff001f, 0),
        (0, 0xff),
        (0xffff, 0xff),
    ];
    for (raw, current) in raw_cases {
        fresh(&mut m);
        put16(&mut m.bus.ewram, 0x7f22, current);
        let ew = m.bus.ewram.clone();
        let iw = m.bus.iwram.clone();
        let offset = getter_index_offset(raw, current);
        let expected = offset.map_or(0, |at| {
            0x08000000 + SOURCE_OFFSET + u32::from(half(&rom, at as usize)) + 6
        });
        call(&mut m, 0x0801edec, raw, 0, &mut coverage);
        assert_eq!(m.cpu.regs[0], expected, "raw {raw:x} current {current:x}");
        assert_ram(&m, &ew, &iw);
    }
    let raw_initializer = [32, 255, 0x7fff, 0x8000, 0xfffe, 0xffff, 0x12340001];
    for raw in raw_initializer {
        fresh(&mut m);
        put16(&mut m.bus.ewram, 0x7f22, 0);
        let ew = m.bus.ewram.clone();
        let iw = m.bus.iwram.clone();
        reset_cpu(&mut m, 0x0801eaf0, raw, 0);
        until(&mut m, 0x0801eb1c, &mut coverage);
        let at = initializer_index_offset(raw);
        assert_eq!(
            m.cpu.regs[5],
            0x08000000 + SOURCE_OFFSET + u32::from(half(&rom, at as usize))
        );
        assert_ram(&m, &ew, &iw);
    }
    for raw in [0, 31, 0xffff] {
        fresh(&mut m);
        put16(&mut m.bus.ewram, 0x7f22, 0xff);
        let ew = m.bus.ewram.clone();
        let iw = m.bus.iwram.clone();
        let prior = coverage.reads.clone();
        call(&mut m, 0x0801eaf0, raw, 0, &mut coverage);
        assert_eq!(m.cpu.regs[0], 0);
        assert_eq!(coverage.reads, prior);
        assert_ram(&m, &ew, &iw);
    }
    // Public explicit-source setter: immediate copy versus live deferred source/shadow.
    let p = &data.programs[31];
    for active in [0, 1] {
        fresh(&mut m);
        put16(&mut m.bus.ewram, 0x7f22, 31);
        m.bus.ewram[0x7f20] = active;
        let pointer = 0x08000000 + p.source_offset + 6;
        let mut ew = m.bus.ewram.clone();
        let mut iw = m.bus.iwram.clone();
        if active == 0 {
            expect_colors(&mut ew, p);
        } else {
            ew[0x7f26] = 1;
            put32(&mut ew, 0x7f28, pointer);
            put16(&mut iw, 0x3860, p.colors[0]);
        }
        call(&mut m, 0x0801ee28, pointer, 0, &mut coverage);
        assert_ram(&m, &ew, &iw);
    }
    // Verify original queued DMA configuration without running a raster/DMA campaign.
    fresh(&mut m);
    put16(&mut m.bus.ewram, 0x7f22, 31);
    expect_colors(&mut m.bus.ewram, p);
    put16(&mut m.bus.iwram, 0x2bc4, 0);
    let mut ew = m.bus.ewram.clone();
    let mut iw = m.bus.iwram.clone();
    ew[0x7f20] = 1;
    put16(&mut iw, 0x3860, p.colors[0]);
    put32(&mut iw, 0x2bf0, 0x02008392);
    put32(&mut iw, 0x2bf4, 0x05000000);
    put32(&mut iw, 0x2bf8, 0xa2400001);
    iw[0x2bfc] = 1;
    iw[0x2bfd] = 0x80;
    call(&mut m, 0x0801ecac, 0, 0, &mut coverage);
    assert_ram(&m, &ew, &iw);

    let mut edited = data.clone();
    edited.programs[0].header.range_length = 321;
    edited.programs[0].header.range_start = 0xfff0;
    edited.programs[0].colors[0] ^= 0x8000;
    edited.validate_dependencies(&descriptors).unwrap();
    assert!(edited.verify_source(&rom).is_err());
    guards.push("authenticated header/color content");
    let mut variant_rom = rom.clone();
    for (at, bytes, _) in edited.source_spans().unwrap() {
        variant_rom[at as usize..at as usize + bytes.len()].copy_from_slice(&bytes);
    }
    assert!(Catalog::from_rom(&variant_rom).is_err());
    let mut variant = Machine::new(variant_rom);
    fresh(&mut variant);
    put16(&mut variant.bus.ewram, 0x7f22, 0);
    let mut ew = variant.bus.ewram.clone();
    let iw = variant.bus.iwram.clone();
    expect_init(&mut ew, &edited.programs[0], false);
    call(&mut variant, 0x0801eaf0, 0, 0, &mut coverage);
    assert_eq!(variant.cpu.regs[0], u32::from(edited.programs[0].colors[0]));
    assert_ram(&variant, &ew, &iw);
    let owned: Vec<_> = coverage
        .reads
        .iter()
        .copied()
        .filter(|a| (SOURCE_OFFSET..SOURCE_END).contains(a))
        .collect();
    assert_eq!(owned, (SOURCE_OFFSET..SOURCE_END).collect::<Vec<_>>());
    assert!(coverage.copies.contains(&(SOURCE_END, 0x13c)));
    assert!(coverage
        .copies
        .iter()
        .any(|&(at, value)| at < SOURCE_END && value == 160));
    fs::write(root.join("map-raster-palettes.json"), encoded).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({
        "source_bytes":10496,"source_ranges":33,"programs":32,"color_words":5120,"descriptor_links":324,
        "shape_json_source_guards":guards,"initializers":64,"canonical_getters":32,
        "raw_getters":raw_cases.len(),"raw_initializer_prefixes":raw_initializer.len(),"disabled_initializers":3,
        "deferred_update_copies":2,"explicit_source_setters":2,"queued_DMA_setup":1,"synthetic_header_bit15_variant":1,
        "original_CpuSet_322B_observations":coverage.copies.len(),"last_extra_halfword_013C_observed_before_wrap":true,
        "owned_distinct_bytes_at_original_loadsites_or_observed_CpuSet":owned.len(),
        "instructions_coverage":coverage.instructions,"PC_coverage":coverage.pcs.len(),
        "scope":"original reader/copy/live metadata and queuedDMA component oracle; no fullstate-clock/native/ordinary or executed raster DMA claim",
        "revision":"e54-d53 unchanged","USA":"e8-d7 unchanged; new checks pending","linux":"pending","performance_measurements":false
    })).unwrap()).unwrap();
}
