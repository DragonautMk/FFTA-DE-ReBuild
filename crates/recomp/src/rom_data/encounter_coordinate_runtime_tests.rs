//! Reduced Wine integration for coordinate payloads and the two bordering owners.
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::{
        encounter_coordinates::{SOURCE, SOURCE_BYTES, SOURCE_END},
        encounter_headers,
    },
    Bus, Machine,
};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_ENCOUNTER_COORDINATE_RUNTIME_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
thread_local! {static COMPILED:Cell<[u64;3]>=const{Cell::new([0;3])};}
fn counts(m: *mut std::ffi::c_void) -> [u64; 3] {
    unsafe {
        let r = (&*(m as *const Machine)).bus.rom_resources().unwrap();
        [
            r.encounter_coordinates_stats().unwrap().source_bytes,
            r.encounter_unit_templates_stats().unwrap().source_bytes,
            r.encounter_headers_stats().unwrap().source_bytes,
        ]
    }
}
fn account(before: [u64; 3], after: [u64; 3]) {
    COMPILED.with(|c| {
        let old = c.get();
        c.set(std::array::from_fn(|i| old[i] + after[i] - before[i]));
    });
}
extern "C" fn read8(m: *mut std::ffi::c_void, at: u32) -> u32 {
    let before = counts(m);
    let v = (RT_API.read8)(m, at);
    account(before, counts(m));
    v
}
extern "C" fn read16(m: *mut std::ffi::c_void, at: u32) -> u32 {
    let before = counts(m);
    let v = (RT_API.read16)(m, at);
    account(before, counts(m));
    v
}
extern "C" fn read32(m: *mut std::ffi::c_void, at: u32) -> u32 {
    let before = counts(m);
    let v = (RT_API.read32)(m, at);
    account(before, counts(m));
    v
}
#[derive(Clone, Copy)]
struct Call {
    entry: u32,
    address: u32,
    base: usize,
    index: Option<(usize, u32)>,
    mirror: u32,
    random_header: Option<u16>,
    seed: u32,
    gate: bool,
    expected: [u64; 3],
}
fn stop(c: Call) -> u32 {
    if c.gate {
        0x08000300
    } else if c.random_header.is_some() {
        0x0812879e
    } else {
        0x08000000 + c.entry + 2
    }
}
fn setup(m: &mut Machine, c: Call) {
    m.bus.ffta_sprites.enabled = false;
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    if c.gate {
        m.cpu.regs[0] = 0;
    } else if let Some(header) = c.random_header {
        m.bus.write32(0x030034b0, c.seed);
        m.bus.write16(0x03007c00, 0);
        m.bus.write32(
            0x03007c08,
            0x08000000 + encounter_headers::INDEX_ORIGIN + u32::from(header) * 40,
        );
        m.cpu.regs[4] = 0x03007c00;
        m.cpu.regs[5] = 0x02010000;
        m.cpu.regs[6] = 0;
        m.cpu.regs[7] = 10;
        m.cpu.regs[8] = 0x020100f6;
        m.cpu.regs[9] = 0x020100f7;
        m.cpu.regs[10] = 0x020100f8;
    } else {
        m.cpu.regs[c.base] = c.mirror + c.address;
        if let Some((register, value)) = c.index {
            m.cpu.regs[register] = value;
        }
    }
    m.cpu.regs[13] = 0x03007c00;
    m.cpu.regs[14] = 0x08000301;
    m.cpu.regs[15] = 0x08000000 + c.entry;
    m.cpu.cpsr = 0xbf;
}
fn interpreted(m: &mut Machine, c: Call, visited: &mut BTreeSet<(u32, bool)>) {
    let mut n = 0;
    while m.cpu.regs[15] != stop(c) {
        assert!(n < 512);
        visited.insert((m.cpu.regs[15], m.cpu.thumb()));
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
}
fn check_state(m: &Machine, expected: &[u8], label: &str) {
    let actual = m.save_state().unwrap();
    if actual != expected {
        fs::write(root().join(format!("{label}-actual.ffstate")), &actual).unwrap();
        fs::write(root().join(format!("{label}-expected.ffstate")), expected).unwrap();
        let differences: Vec<_> = actual
            .iter()
            .zip(expected)
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .map(|(i, _)| i)
            .take(20)
            .collect();
        panic!(
            "fullstate-clock mismatch {label}, firstoffsets{differences:?}, lengths{}/{}",
            actual.len(),
            expected.len()
        );
    }
}
fn native_prefixes(rom: &[u8]) -> serde_json::Value {
    let mut calls = Vec::new();
    calls.push(Call {
        entry: 0xbd00c,
        address: 0x70b6a8,
        base: 4,
        index: None,
        mirror: 0x8000000,
        random_header: None,
        seed: 0,
        gate: false,
        expected: [1, 0, 0],
    });
    calls.push(Call {
        entry: 0xbd012,
        address: 0x70b6a8,
        base: 4,
        index: None,
        mirror: 0x8000000,
        random_header: None,
        seed: 0,
        gate: false,
        expected: [1, 0, 0],
    });
    calls.push(Call {
        entry: 0xbd044,
        address: 0x70b6a8,
        base: 4,
        index: Some((1, 0)),
        mirror: 0x8000000,
        random_header: None,
        seed: 0,
        gate: false,
        expected: [1, 0, 0],
    });
    calls.push(Call {
        entry: 0xbd048,
        address: 0x70b6a8,
        base: 4,
        index: Some((2, 1)),
        mirror: 0x8000000,
        random_header: None,
        seed: 0,
        gate: false,
        expected: [1, 0, 0],
    });
    calls.push(Call {
        entry: 0x128792,
        address: 0x70bbc0,
        base: 1,
        index: None,
        mirror: 0x8000000,
        random_header: None,
        seed: 0,
        gate: false,
        expected: [1, 0, 0],
    });
    calls.push(Call {
        entry: 0x128798,
        address: 0x70bbc0,
        base: 1,
        index: None,
        mirror: 0xa000000,
        random_header: None,
        seed: 0,
        gate: false,
        expected: [1, 0, 0],
    });
    calls.push(Call {
        entry: 0xcbe7c,
        address: 0x70b6b4,
        base: 0,
        index: None,
        mirror: 0x8000000,
        random_header: None,
        seed: 0,
        gate: false,
        expected: [1, 0, 0],
    });
    calls.push(Call {
        entry: 0xcbeaa,
        address: 0x70b6b4,
        base: 0,
        index: None,
        mirror: 0x8000000,
        random_header: None,
        seed: 0,
        gate: false,
        expected: [1, 0, 0],
    });
    calls.push(Call {
        entry: 0xcbe7c,
        address: 0x70c8f2,
        base: 0,
        index: None,
        mirror: 0xa000000,
        random_header: None,
        seed: 0,
        gate: false,
        expected: [1, 0, 0],
    });
    calls.push(Call {
        entry: 0xcbeaa,
        address: 0x70c8f2,
        base: 0,
        index: None,
        mirror: 0xc000000,
        random_header: None,
        seed: 0,
        gate: false,
        expected: [1, 0, 0],
    });
    calls.push(Call {
        entry: 0xbd33a,
        address: 0x70bba6,
        base: 2,
        index: None,
        mirror: 0x8000000,
        random_header: None,
        seed: 0,
        gate: false,
        expected: [1, 0, 0],
    });
    calls.push(Call {
        entry: 0xbd340,
        address: 0x70bba6,
        base: 2,
        index: None,
        mirror: 0x8000000,
        random_header: None,
        seed: 0,
        gate: false,
        expected: [1, 0, 0],
    });
    calls.push(Call {
        entry: 0xbd044,
        address: 0x70bba6,
        base: 4,
        index: Some((1, 0)),
        mirror: 0x8000000,
        random_header: None,
        seed: 0,
        gate: false,
        expected: [1, 0, 0],
    });
    calls.push(Call {
        entry: 0xbd048,
        address: 0x70bba6,
        base: 4,
        index: Some((2, 1)),
        mirror: 0x8000000,
        random_header: None,
        seed: 0,
        gate: false,
        expected: [1, 0, 0],
    });
    calls.push(Call {
        entry: 0xbd00c,
        address: 0x70b6a7,
        base: 4,
        index: None,
        mirror: 0x8000000,
        random_header: None,
        seed: 0,
        gate: false,
        expected: [0, 1, 0],
    });
    calls.push(Call {
        entry: 0xbd044,
        address: 0x70b6a7,
        base: 4,
        index: Some((1, 0)),
        mirror: 0x8000000,
        random_header: None,
        seed: 0,
        gate: false,
        expected: [0, 1, 0],
    });
    calls.push(Call {
        entry: 0xbd00c,
        address: 0x70c8f4,
        base: 4,
        index: None,
        mirror: 0x8000000,
        random_header: None,
        seed: 0,
        gate: false,
        expected: [0, 0, 1],
    });
    calls.push(Call {
        entry: 0xcbe7c,
        address: 0x710e04,
        base: 0,
        index: None,
        mirror: 0x8000000,
        random_header: None,
        seed: 0,
        gate: false,
        expected: [0, 0, 0],
    });
    calls.push(Call {
        entry: 0xc0034,
        address: 0,
        base: 0,
        index: None,
        mirror: 0x08000000,
        random_header: None,
        seed: 0,
        gate: true,
        expected: [1, 0, 0],
    });
    calls.push(Call {
        entry: 0x128764,
        address: 0,
        base: 0,
        index: None,
        mirror: 0x08000000,
        random_header: Some(171),
        seed: 0,
        gate: false,
        expected: [2, 0, 4],
    });
    calls.push(Call {
        entry: 0x128764,
        address: 0,
        base: 0,
        index: None,
        mirror: 0x08000000,
        random_header: Some(201),
        seed: 1,
        gate: false,
        expected: [2, 0, 4],
    });
    calls.push(Call {
        entry: 0x128764,
        address: 0,
        base: 0,
        index: None,
        mirror: 0x08000000,
        random_header: Some(233),
        seed: 4294967295,
        gate: false,
        expected: [2, 0, 4],
    });
    assert_eq!(calls.len(), 22);
    let mut raw = crate::make_machine(rom.to_vec(), None);
    let initial = raw.save_state().unwrap();
    let mut visited = [BTreeSet::new(), BTreeSet::new()];
    let mut expected = Vec::new();
    for &c in &calls {
        raw.load_state(&initial).unwrap();
        setup(&mut raw, c);
        interpreted(
            &mut raw,
            c,
            &mut visited[usize::from(c.random_header.is_some())],
        );
        expected.push(raw.save_state().unwrap());
    }
    let view = View {
        rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let compiled_instructions: usize = visited.iter().map(BTreeSet::len).sum();
    let mut libraries = Vec::new();
    let mut tables = Vec::new();
    for (group, visited) in visited.iter().enumerate() {
        assert!(
            calls
                .iter()
                .filter(|c| usize::from(c.random_header.is_some()) == group)
                .all(|&c| !visited.iter().any(|&(pc, _)| pc == stop(c))),
            "case family must not compile its stop boundary"
        );
        let blocks = visited
            .iter()
            .map(|&(addr, thumb)| {
                let mut instrs = vec![if thumb {
                    armv4t::decode_thumb(view.read16(addr), addr)
                } else {
                    armv4t::decode_arm(view.read32(addr), addr)
                }];
                if thumb {
                    let next = armv4t::decode_thumb(view.read16(addr + 2), addr + 2);
                    if armv4t::fuse_thumb_bl(&instrs[0], &next).is_some() {
                        instrs.push(next);
                    }
                }
                Block {
                    addr,
                    thumb,
                    instrs,
                }
            })
            .collect();
        let object = root().join(format!("encounter-coordinate-family{group}.o"));
        recomp_core::tcc_memory::build(
            &Analysis { blocks },
            &view,
            &recomp_core::tcc_memory::library_path().unwrap(),
            &object,
            &mut |_| {},
        )
        .unwrap();
        let (library, table) = crate::load_native(&object).unwrap();
        libraries.push(library);
        tables.push(table);
    }
    let mut typed = crate::make_machine(rom.to_vec(), None);
    let mut native = crate::make_machine(rom.to_vec(), None);
    for m in [&mut typed, &mut native] {
        attach_from_directory(m, &dataset()).unwrap();
    }
    let api = gba_core::capi::RtApi {
        read8,
        read16,
        read32,
        ..RT_API
    };
    COMPILED.with(|c| c.set([0; 3]));
    let mut dispatch = 0;
    for (i, &c) in calls.iter().enumerate() {
        for m in [&mut typed, &mut native] {
            m.load_state(&initial).unwrap();
            setup(m, c);
            m.bus.rom[SOURCE as usize..SOURCE as usize + SOURCE_BYTES].fill(0xcd);
        }
        interpreted(&mut typed, c, &mut BTreeSet::new());
        let before = COMPILED.with(Cell::get);
        while native.cpu.regs[15] != stop(c) {
            assert!(dispatch < 4096);
            native.bus.frame_ready = false;
            tables[usize::from(c.random_header.is_some())]
                .get(native.cpu.regs[15] | u32::from(native.cpu.thumb()))
                .unwrap_or_else(|| {
                    panic!(
                        "uncompiled original coordinate case{i} PC{:08x} thumb{}",
                        native.cpu.regs[15],
                        native.cpu.thumb()
                    )
                })(&api, &mut native as *mut Machine as *mut std::ffi::c_void);
            dispatch += 1;
        }
        let after = COMPILED.with(Cell::get);
        assert_eq!(
            std::array::from_fn::<_, 3, _>(|j| after[j] - before[j]),
            c.expected,
            "compiled owner counts case{i}"
        );
        for (kind, m) in [("interpreted", &mut typed), ("native", &mut native)] {
            m.bus.rom[SOURCE as usize..SOURCE as usize + SOURCE_BYTES]
                .copy_from_slice(&rom[SOURCE as usize..SOURCE as usize + SOURCE_BYTES]);
            check_state(m, &expected[i], &format!("{kind}-case{i}"));
        }
    }
    let c = COMPILED.with(Cell::get);
    serde_json::json!({"full_state_clock_cases":calls.len(),"single_load_prefixes":18,"full_index0_gate_calls":1,"original_random_prefixes":3,"compiled_new_source_bytes":c[0],"compiled_prior_template_bytes":c[1],"compiled_prior_header_bytes":c[2],"compiled_instructions":compiled_instructions,"native_case_families":2,"each_family_excludes_stop_boundaries":true,"dispatch_coverage_only":dispatch,"all4684_raw_coordinate_bytes_poisoned":true,"signed_unsigned_all4roles_mirrors_prior_rawoutside_index0prefix":true})
}
fn bus_lifecycle(rom: &[u8]) -> serde_json::Value {
    let mut m = crate::make_machine(rom.to_vec(), None);
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    let initial = m.save_state().unwrap();
    m.bus.rom[SOURCE as usize..SOURCE as usize + SOURCE_BYTES].fill(0xcd);
    for i in 0..SOURCE_BYTES {
        assert_eq!(
            m.bus.read8(0x0c000000 + SOURCE + i as u32),
            rom[SOURCE as usize + i]
        );
    }
    let s = owner.encounter_coordinates_stats().unwrap();
    assert_eq!(
        [
            s.bus_reads,
            s.source_bytes,
            s.placement0_bytes,
            s.random_bytes,
            s.placement2_bytes,
            s.default_bytes,
            s.x_bytes,
            s.y_bytes,
            s.lists_used,
            s.headers_used
        ],
        [4684, 4684, 3272, 680, 88, 644, 2342, 2342, 691, 323]
    );
    let pointer = &mut m as *mut Machine as *mut std::ffi::c_void;
    for delta in [-1, 1, 3, 0x70bfff - SOURCE as i32, SOURCE_BYTES as i32 - 1] {
        let at = (SOURCE as i32 + delta) as usize;
        assert_eq!(
            (RT_API.read16)(pointer, 0x0a000000 + at as u32),
            u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap()))
        );
        assert_eq!(
            (RT_API.read32)(pointer, 0x08000000 + at as u32),
            u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
        );
    }
    let end = SOURCE as usize + SOURCE_BYTES;
    // The immediate borders are existing typed owners, not raw sentinels.
    m.bus.rom[SOURCE as usize - 4..SOURCE as usize].fill(0xab);
    m.bus.rom[end..end + 4].fill(0xef);
    for at in [SOURCE as usize - 1, end - 1] {
        assert_eq!(
            (RT_API.read32)(pointer, 0x08000000 + at as u32),
            u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
        );
    }
    assert!(owner.encounter_unit_templates_stats().unwrap().source_bytes > 0);
    assert!(owner.encounter_headers_stats().unwrap().source_bytes > 0);
    let catalog = gba_core::rom_data::encounter_coordinates::Catalog::from_rom(rom).unwrap();
    for list in &catalog.decoded.lists {
        let at = list.source_offset + list.points.len() as u32 * 2 - 1;
        assert_eq!(
            (RT_API.read32)(pointer, 0x0a000000 + at),
            u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
        );
    }
    let before = owner.encounter_coordinates_stats().unwrap();
    let old = gba_core::rom_data::task_cycle_deltas::SOURCE as usize;
    m.bus.rom[old] = 0xab;
    assert_eq!(m.bus.read8(0x08000000 + old as u32), rom[old]);
    assert_eq!(owner.encounter_coordinates_stats().unwrap(), before);
    m.bus.rom.copy_from_slice(rom);
    let unavailable = root().join("unavailable dataset");
    fs::rename(dataset(), &unavailable).unwrap();
    assert!(attach_from_directory(&mut m, &dataset()).is_err());
    assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
    m.load_state(&initial).unwrap();
    let mut slots = crate::save_state::Slots::new(&root().join("private.sav"), rom);
    slots.save(&m, 1, &vec![[0; 4]; 240 * 160], false).unwrap();
    m.cpu.regs[4] = 123;
    let changed = m.save_state().unwrap();
    slots.load(&mut m, 1).unwrap();
    assert_eq!(m.save_state().unwrap(), initial);
    slots.undo(&mut m).unwrap();
    assert_eq!(m.save_state().unwrap(), changed);
    assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
    m.bus.rom[SOURCE as usize..end].fill(0xcd);
    assert_eq!(m.bus.read8(0x08000000 + SOURCE), rom[SOURCE as usize]);
    m.bus.rom.copy_from_slice(rom);
    fs::rename(&unavailable, dataset()).unwrap();
    for kind in 0..3 {
        let mut a = crate::make_machine(rom.to_vec(), None);
        let mut b = crate::make_machine(rom.to_vec(), None);
        attach_from_directory(&mut b, &dataset()).unwrap();
        for m in [&mut a, &mut b] {
            if kind == 0 {
                m.bus.write32(0x040000d4, 0x08000000 + SOURCE);
                m.bus.write32(0x040000d8, 0x02010000);
                m.bus.write16(0x040000dc, 2344);
            } else {
                m.cpu.regs[..3].copy_from_slice(&[
                    0x0a000000 + SOURCE,
                    0x02010000,
                    if kind == 2 { 1184 } else { 2344 },
                ]);
                m.cpu.regs[15] = 0x02000000;
                m.cpu.cpsr = 0xbf;
                m.bus
                    .write16(0x02000000, if kind == 2 { 0xdf0c } else { 0xdf0b });
            }
        }
        b.bus.rom[SOURCE as usize..end].fill(0xcd);
        for m in [&mut a, &mut b] {
            if kind == 0 {
                m.bus.write16(0x040000de, 0x8000);
            } else {
                m.step();
            }
        }
        b.bus.rom.copy_from_slice(rom);
        assert_eq!(
            b.save_state().unwrap(),
            a.save_state().unwrap(),
            "full state/clock node transfer{kind}"
        );
        assert!(
            b.bus
                .rom_resources()
                .unwrap()
                .encounter_coordinates_stats()
                .unwrap()
                .source_bytes
                >= 4684
        );
    }
    serde_json::json!({"all4684_new_typed_bytes":true,"all4rolesXY691lists323headers":true,"widths_mirrors_all691listboundaries_prior_owners_index0prefix":true,"unavailable_files_Load_slot_Undo_owner_retained":true,"DMA_CpuSet_CpuFastSet_fullstate_clock":3})
}

#[test]
#[ignore = "private EUR encounter-coordinate managed/native integration; Wine only, Linux/USA pending"]
fn european_encounter_coordinates_managed_native_and_lifecycle() {
    let root = root();
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let prior: Manifest =
        toml::from_str(&fs::read_to_string(old.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (
            prior.extractor_revision,
            prior.decoder_revision,
            prior.resources.len()
        ),
        (81, 80, 73)
    );
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision,
            source.manifest.resources.len()
        ),
        (82, 81, 74)
    );
    extract(&dataset(), &mut source).unwrap();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 75);
    assert_eq!(&source.manifest.resources[..73], prior.resources.as_slice());
    assert_eq!(&source.manifest.coverage[..73], prior.coverage.as_slice());
    for entry in &prior.resources {
        assert_eq!(
            fs::read(dataset().join(&entry.path)).unwrap(),
            fs::read(old.join(&entry.path)).unwrap()
        );
    }
    let entry = &source.manifest.resources[73];
    assert_eq!(entry.id, ENCOUNTER_COORDINATES_ID);
    assert_eq!(entry.dependencies, [ENCOUNTER_HEADERS_ID]);
    assert_eq!(entry.sources.len(), 691);
    assert_eq!(entry.sources.iter().map(|s| s.length).sum::<u32>(), 4684);
    assert_eq!(entry.sources.first().unwrap().offset, SOURCE);
    assert_eq!(entry.sources.last().unwrap().offset, SOURCE_END - 2);
    let bytes = fs::read(dataset().join(ENCOUNTER_COORDINATES_FILE)).unwrap();
    let manifest = fs::read(dataset().join(MANIFEST_FILE)).unwrap();
    assert_eq!(
        parse_encounter_coordinates(&bytes).unwrap(),
        source.encounter_coordinates.as_ref().unwrap().decoded
    );
    let interrupted = root.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    let mut reached = false;
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| {
        if p.file_name().unwrap() == ENCOUNTER_COORDINATES_FILE {
            reached = true;
            Err("injected encounter-coordinate file failure".into())
        } else {
            write_synced(p, b)
        }
    })
    .is_err());
    assert!(reached);
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    assert!(verify(&interrupted, &source).is_err());
    eprintln!("ROM_DATA_ENCOUNTER_COORDINATE_STAGE publication75/prior73exact/interruption PASS");
    let mut m = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    let state = m.save_state().unwrap();
    for case in 0..14 {
        let mut bad = source.manifest.clone();
        let mut data = bytes.clone();
        match case {
            0..=4 => {
                let mut v = parse_encounter_coordinates(&data).unwrap();
                match case {
                    0 => v.lists[0].points[0].x ^= 1,
                    1 => v.lists[0].header_index ^= 1,
                    2 => v.lists[0].points[0].y = 255,
                    3 => {
                        v.lists.pop();
                    }
                    _ => v.source_offset += 1,
                };
                data = encode_encounter_coordinates(&v).unwrap();
            }
            5 => bad.resources[73].sources[0].offset += 1,
            6 => bad.resources[73].sources[0].length -= 1,
            7 => bad.resources[73].sources[0].sha256 = "00".repeat(32),
            8 => bad.resources[73].sources[0].role = "other".into(),
            9 => {
                let mut v: serde_json::Value = serde_json::from_slice(&data).unwrap();
                v["unexpected"] = true.into();
                data = serde_json::to_vec(&v).unwrap();
            }
            10 => bad.resources[73].dependencies.clear(),
            11 => bad.resources[73]
                .sources
                .push(gba_core::rom_data::SourceRange {
                    offset: SOURCE_END,
                    length: 2,
                    sha256: gba_core::ffta_rom::digest(
                        &rom[SOURCE_END as usize..SOURCE_END as usize + 2],
                    ),
                    role: "counterfeit-shared".into(),
                }),
            12 => bad.resources[73].sources.swap(0, 1),
            _ => {
                bad.coverage.pop();
            }
        }
        bad.resources[73].sha256 = gba_core::ffta_rom::digest(&data);
        fs::write(dataset().join(ENCOUNTER_COORDINATES_FILE), data).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&bad).unwrap(),
        )
        .unwrap();
        assert!(
            attach_from_directory(&mut m, &dataset()).is_err(),
            "encounter-coordinate attach guard{case}"
        );
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), state);
    }
    fs::write(dataset().join(ENCOUNTER_COORDINATES_FILE), &bytes).unwrap();
    fs::write(dataset().join(MANIFEST_FILE), &manifest).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let saved = source.encounter_coordinates.take();
    assert!(source.validate_files(files.files()).is_err());
    source.encounter_coordinates = saved;
    let mut missing = files.files();
    missing.encounter_coordinates = None;
    assert!(source.validate_files(missing).is_err());
    let mut missing = files.files();
    missing.encounter_headers = None;
    assert!(source.validate_files(missing).is_err());
    eprintln!("ROM_DATA_ENCOUNTER_COORDINATE_STAGE attach14/fullset3/failedownerfullstate PASS");
    let native = native_prefixes(&rom);
    eprintln!(
        "ROM_DATA_ENCOUNTER_COORDINATE_STAGE native22/fullstate-clock/source4684poisoned PASS"
    );
    let bus = bus_lifecycle(&rom);
    let result = serde_json::json!({"status":"PASS","revision":[1,82,81],"files":75,"resources":74,"prior73_records_files_exact":true,"source_bytes":4684,"attach_guards":14,"fullset_guards":3,"newfile_interruption":true,"native":native,"bus_lifecycle":bus,"dataset_sha256":owner.dataset_sha256(),"manifest_sha256":gba_core::ffta_rom::digest(&manifest),"encounter_coordinates_JSON_sha256":gba_core::ffta_rom::digest(&bytes),"actualCLI":"pending","ordinary_positive":"pending","Linux":"pending","USA":"pending","nativeWindows":"pending","new32_9":"pending","performance_measurements":false});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_ENCOUNTER_COORDINATE_RUNTIME e82-d81/files75/resources74/attach14/fullset3/native22/DMA3 PASS");
}

#[test]
#[ignore = "private EUR remaining coordinate native/lifecycle proof; Wine only"]
fn european_encounter_coordinates_remaining_native_and_lifecycle() {
    let root = root();
    assert!(root.is_dir());
    assert!(!root.join("result.json").exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let manifest = fs::read(dataset().join(MANIFEST_FILE)).unwrap();
    let bytes = fs::read(dataset().join(ENCOUNTER_COORDINATES_FILE)).unwrap();
    let mut m = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    let native = native_prefixes(&rom);
    eprintln!(
        "ROM_DATA_ENCOUNTER_COORDINATE_STAGE native22/fullstate-clock/source4684poisoned PASS"
    );
    let bus = bus_lifecycle(&rom);
    let result = serde_json::json!({"status":"PASS","revision":[1,82,81],"files":75,"resources":74,"prior73_records_files_exact":true,"source_bytes":4684,"attach_guards":14,"fullset_guards":3,"newfile_interruption":true,"native":native,"bus_lifecycle":bus,"dataset_sha256":owner.dataset_sha256(),"manifest_sha256":gba_core::ffta_rom::digest(&manifest),"encounter_coordinates_JSON_sha256":gba_core::ffta_rom::digest(&bytes),"publication_attach_fullset":"cached first filter PASS, not rerun","actualCLI":"pending","ordinary_positive":"pending","Linux":"pending","USA":"pending","nativeWindows":"pending","new32_9":"pending","performance_measurements":false});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_ENCOUNTER_COORDINATE_RUNTIME e82-d81/files75/resources74/attach14/fullset3/native22/DMA3 PASS");
}
