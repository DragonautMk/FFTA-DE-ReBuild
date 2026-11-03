//! Focused Wine integration for indexed conditional_records8 and its typed/raw borders.
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::conditional_records8::{SOURCE, SOURCE_BYTES, SOURCE_END},
    Bus, Machine,
};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_CONDITIONAL_RECORDS8_RUNTIME_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
thread_local! {static FALLBACK:Cell<[u64;2]>=const{Cell::new([0;2])};static COMPILED:Cell<[u64;2]>=const{Cell::new([0;2])};}
fn counts(m: *mut std::ffi::c_void) -> [u64; 2] {
    unsafe {
        let r = (&*(m as *const Machine)).bus.rom_resources().unwrap();
        [
            r.conditional_records8_stats().unwrap().source_bytes,
            r.records4_stats().unwrap().source_bytes,
        ]
    }
}
fn account(before: [u64; 2], after: [u64; 2]) {
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
// A generated fallback shell executes memory reads through the real interpreter.
// Account that callback too; direct read8/16/32 wrappers alone miss those bytes.
extern "C" fn interp_one(m: *mut std::ffi::c_void) -> u32 {
    let before = counts(m);
    let value = (RT_API.interp_one)(m);
    let after = counts(m);
    FALLBACK.with(|cell| {
        let old = cell.get();
        cell.set(std::array::from_fn(|j| old[j] + after[j] - before[j]));
    });
    account(before, after);
    value
}
#[derive(Clone, Copy)]
struct Call {
    entry: u32,
    address: u32,
    mirror: u32,
    kind: u8,
    expected: [u64; 2],
}
fn family(c: Call) -> usize {
    if c.kind == 0 {
        ((c.entry >> 1) & 1) as usize
    } else {
        c.kind as usize + 1
    }
}
fn stop(c: Call) -> u32 {
    match c.kind {
        0 => 0x08000000 + c.entry + 2,
        1 => 0x080d56e8,
        2 => 0x080d5746,
        _ => 0x080d57a4,
    }
}
fn setup(m: &mut Machine, c: Call) {
    m.bus.ffta_sprites.enabled = false;
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    if c.kind == 0 {
        let op = u16::from_le_bytes(
            m.bus.rom[c.entry as usize..c.entry as usize + 2]
                .try_into()
                .unwrap(),
        );
        let width = match op & 0xf800 {
            0x8800 => 2,
            0x6800 => 4,
            _ => {
                assert_eq!(op & 0xf800, 0x7800);
                1
            }
        };
        m.cpu.regs[usize::from((op >> 3) & 7)] =
            c.mirror + c.address - u32::from((op >> 6) & 31) * width;
    } else if c.kind == 1 || c.kind == 2 {
        m.cpu.regs[5] = c.mirror + c.address;
    } else {
        m.bus.ewram.fill(0x5a);
        m.bus.ewram[0x1f70..0x2030].fill(255);
        m.bus.ewram[0x2030..0x2030 + 0xfa00].fill(1);
        m.bus.write32(0x03007c00, 0x12340000);
        m.cpu.regs[5] = c.mirror + SOURCE;
        m.cpu.regs[6] = 64;
        m.cpu.regs[7] = 0x02020000;
        m.cpu.regs[8] = 1;
        m.cpu.regs[9] = 0;
        m.cpu.regs[10] = 0x05ff;
    }
    m.cpu.regs[13] = 0x03007c00;
    m.cpu.regs[14] = 0x08000301;
    m.cpu.regs[15] = 0x08000000 + c.entry;
    m.cpu.cpsr = 0xbf;
}
fn interpreted(m: &mut Machine, c: Call, visited: &mut BTreeSet<(u32, bool)>) -> u64 {
    let mut n = 0;
    let mut actual_loop_source_bytes = 0;
    while m.cpu.regs[15] != stop(c) {
        assert!(n < 20000);
        let pc = m.cpu.regs[15];
        if c.kind == 3
            && matches!(
                pc,
                0x080d56ce
                    | 0x080d56d4
                    | 0x080d56e0
                    | 0x080d56e2
                    | 0x080d56fe
                    | 0x080d5726
                    | 0x080d5738
                    | 0x080d573e
                    | 0x080d5740
                    | 0x080d575c
                    | 0x080d577a
                    | 0x080d5784
            )
        {
            actual_loop_source_bytes += 1;
        }
        visited.insert((pc, m.cpu.thumb()));
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    actual_loop_source_bytes
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
    for (n, (entry, index, field)) in [
        (0xd56ce, 0, 0),
        (0xd56d4, 126, 1),
        (0xd56e0, 64, 2),
        (0xd56e2, 32, 3),
        (0xd56fe, 55, 4),
        (0xd573e, 96, 5),
        (0xd5740, 125, 6),
        (0xd575c, 55, 7),
    ]
    .into_iter()
    .enumerate()
    {
        calls.push(Call {
            entry,
            address: SOURCE + index * 8 + field,
            mirror: [0x08000000, 0x0a000000, 0x0c000000][n % 3],
            kind: 0,
            expected: [1, 0],
        });
    }
    for (entry, address, expected) in [
        (0xd0878, SOURCE + 2, [2, 0]),
        (0xd08a2, SOURCE + 4, [4, 0]),
        (0xd56ce, SOURCE - 1, [0, 1]),
        (0xd56ce, SOURCE_END, [0, 0]),
        // Guest LDR word aligns before rotating; no 2+2 expectation for these.
        (0xd08a2, SOURCE - 2, [0, 4]),
        (0xd08a2, SOURCE_END - 2, [4, 0]),
    ] {
        calls.push(Call {
            entry,
            address,
            mirror: 0x0c000000,
            kind: 0,
            expected,
        });
    }
    calls.push(Call {
        entry: 0xd56e0,
        address: SOURCE + 55 * 8,
        mirror: 0x0a000000,
        kind: 1,
        expected: [2, 0],
    });
    calls.push(Call {
        entry: 0xd573e,
        address: SOURCE + 55 * 8,
        mirror: 0x0c000000,
        kind: 2,
        expected: [2, 0],
    });
    calls.push(Call {
        entry: 0xd56ce,
        address: 0,
        mirror: 0x08000000,
        kind: 3,
        expected: [0, 0],
    });
    assert_eq!(calls.len(), 17);
    let remaining = std::env::var_os("FFTA_CONDITIONAL_RECORDS8_REMAINING").is_some();
    let diagnostic =
        remaining && std::env::var_os("FFTA_CONDITIONAL_RECORDS8_FINAL_OBSERVER").is_none();
    if remaining {
        calls.retain(|c| c.kind == 3);
    }
    let mut read_differences = Vec::new();
    let mut case_observations = Vec::new();
    let mut raw = crate::make_machine(rom.to_vec(), None);
    let initial = raw.save_state().unwrap();
    let mut visited: [_; 5] = std::array::from_fn(|_| BTreeSet::new());
    let mut expected = Vec::new();
    let mut expected_counts = Vec::new();
    for &c in &calls {
        raw.load_state(&initial).unwrap();
        setup(&mut raw, c);
        let original_loop_source_bytes = interpreted(&mut raw, c, &mut visited[family(c)]);
        expected_counts.push(if c.kind == 3 {
            assert!(original_loop_source_bytes > 500);
            [original_loop_source_bytes, 0]
        } else {
            c.expected
        });
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
        if visited.is_empty() {
            tables.push(None);
            continue;
        }
        assert!(
            calls
                .iter()
                .filter(|c| family(**c) == group)
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
        let object = root().join(format!("conditional_records8-family{group}.o"));
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
        tables.push(Some(table));
    }
    let mut typed = crate::make_machine(rom.to_vec(), None);
    let mut native = crate::make_machine(rom.to_vec(), None);
    for m in [&mut typed, &mut native] {
        attach_from_directory(m, &dataset()).unwrap();
    }
    let api = gba_core::capi::RtApi {
        interp_one,
        read8,
        read16,
        read32,
        ..RT_API
    };
    COMPILED.with(|c| c.set([0; 2]));
    FALLBACK.with(|c| c.set([0; 2]));
    let mut dispatch = 0;
    for (i, &c) in calls.iter().enumerate() {
        for m in [&mut typed, &mut native] {
            m.load_state(&initial).unwrap();
            setup(m, c);
            m.bus.rom[SOURCE as usize..SOURCE as usize + SOURCE_BYTES].fill(0xcd);
            m.bus.rom[SOURCE as usize - 4..SOURCE as usize].fill(0xab);
        }
        let typed_before = counts(&mut typed as *mut Machine as *mut std::ffi::c_void);
        let typed_observer = interpreted(&mut typed, c, &mut BTreeSet::new());
        let typed_after = counts(&mut typed as *mut Machine as *mut std::ffi::c_void);
        let typed_delta = std::array::from_fn::<_, 2, _>(|j| typed_after[j] - typed_before[j]);
        let native_before = counts(&mut native as *mut Machine as *mut std::ffi::c_void);
        let before = COMPILED.with(Cell::get);
        while native.cpu.regs[15] != stop(c) {
            assert!(dispatch < 20000);
            native.bus.frame_ready = false;
            let pc = native.cpu.regs[15];
            let actual_before = counts(&mut native as *mut Machine as *mut std::ffi::c_void);
            let api_before = COMPILED.with(Cell::get);
            tables[family(c)]
                .as_ref()
                .unwrap()
                .get(native.cpu.regs[15] | u32::from(native.cpu.thumb()))
                .unwrap_or_else(|| {
                    panic!(
                        "uncompiled original conditional_records8 case{i} PC{:08x} thumb{}",
                        native.cpu.regs[15],
                        native.cpu.thumb()
                    )
                })(&api, &mut native as *mut Machine as *mut std::ffi::c_void);
            let actual_after = counts(&mut native as *mut Machine as *mut std::ffi::c_void);
            let api_after = COMPILED.with(Cell::get);
            let actual = std::array::from_fn::<_, 2, _>(|j| actual_after[j] - actual_before[j]);
            let api = std::array::from_fn::<_, 2, _>(|j| api_after[j] - api_before[j]);
            if actual != api {
                read_differences.push(serde_json::json!({"pc":pc,"actual":actual,"api":api}));
            }
            dispatch += 1;
        }
        let after = COMPILED.with(Cell::get);
        let native_after = counts(&mut native as *mut Machine as *mut std::ffi::c_void);
        let native_delta = std::array::from_fn::<_, 2, _>(|j| native_after[j] - native_before[j]);
        let api_delta = std::array::from_fn::<_, 2, _>(|j| after[j] - before[j]);
        case_observations.push(serde_json::json!({"case":i,"raw_pc_observer":expected_counts[i],"typed_pc_observer":typed_observer,"typed_actual":typed_delta,"native_actual":native_delta,"api":api_delta,"read_differences":read_differences}));
        fs::write(
            root().join(if diagnostic {"remaining-native-observations.json"}else{"final-native-observations.json"}),
            serde_json::to_vec_pretty(&case_observations).unwrap(),
        )
        .unwrap();
        if !diagnostic {
            assert_eq!(
                api_delta, expected_counts[i],
                "all native callbacks case{i}"
            );
            assert_eq!(
                typed_delta, expected_counts[i],
                "actual typed reads case{i}"
            );
            assert_eq!(
                native_delta, expected_counts[i],
                "actual native reads case{i}"
            );
            assert!(
                read_differences.is_empty(),
                "all native callback reads must be attributed"
            );
        }
        for (kind, m) in [("interpreted", &mut typed), ("native", &mut native)] {
            m.bus.rom[SOURCE as usize..SOURCE as usize + SOURCE_BYTES]
                .copy_from_slice(&rom[SOURCE as usize..SOURCE as usize + SOURCE_BYTES]);
            m.bus.rom[SOURCE as usize - 4..SOURCE as usize]
                .copy_from_slice(&rom[SOURCE as usize - 4..SOURCE as usize]);
            check_state(m, &expected[i], &format!("{kind}-case{i}"));
        }
    }
    let c = COMPILED.with(Cell::get);
    if !diagnostic {
        assert_eq!(
            c,
            expected_counts
                .iter()
                .fold([0; 2], |a, b| [a[0] + b[0], a[1] + b[1]])
        );
    }
    assert_eq!(c[1], if remaining { 0 } else { 5 });
    serde_json::json!({"full_state_clock_cases":calls.len(),"diagnostic_only":diagnostic,"case_observations":case_observations,"single_load_prefixes":if remaining {0}else{14},
        "encoded_key_prefixes":if remaining {0}else{2},"original_selection_loops":1,
        "compiled_new_source_bytes":c[0],"fallback_source_bytes":FALLBACK.with(Cell::get),"compiled_prior_records4_bytes":c[1],
        "compiled_selection_source_bytes":expected_counts.last().unwrap()[0],
        "compiled_instructions":compiled_instructions,"native_case_families":if remaining {1}else{5},
        "each_family_excludes_stop_boundaries":true,"dispatch_coverage_only":dispatch,
        "all1016_raw_source_bytes_poisoned":true,"eight_unsigned_bytes_LEkeys_mirrors_prior_rawoutside":true})
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
    let s = owner.conditional_records8_stats().unwrap();
    assert_eq!(
        [
            s.bus_reads,
            s.source_bytes,
            s.gate_bytes,
            s.filter_bytes,
            s.first_key_bytes,
            s.first_operand_bytes,
            s.second_key_bytes,
            s.second_operand_bytes,
            s.records_used
        ],
        [1016, 1016, 127, 127, 254, 127, 254, 127, 127]
    );
    let pointer = &mut m as *mut Machine as *mut std::ffi::c_void;
    for delta in [
        -2,
        -1,
        1,
        3,
        SOURCE_BYTES as i32 - 2,
        SOURCE_BYTES as i32 - 1,
    ] {
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
    // Records4 bytes to the left retain the prior owner; the successor on the right is raw.
    m.bus.rom[SOURCE as usize - 4..SOURCE as usize].fill(0xab);
    m.bus.rom[end..end + 4].fill(0xef);
    assert_eq!(
        (RT_API.read32)(pointer, 0x08000000 + SOURCE - 1),
        u32::from_le_bytes(
            rom[SOURCE as usize - 1..SOURCE as usize + 3]
                .try_into()
                .unwrap()
        )
    );
    assert_eq!(
        (RT_API.read32)(pointer, 0x08000000 + SOURCE_END - 1),
        u32::from_le_bytes([rom[end - 1], 0xef, 0xef, 0xef])
    );
    assert!(owner.records4_stats().unwrap().source_bytes > 0);
    let before = owner.conditional_records8_stats().unwrap();
    assert_eq!(m.bus.read8(0x08000000 + SOURCE_END), 0xef);
    assert_eq!(owner.conditional_records8_stats().unwrap(), before);
    m.bus.rom[end..end + 4].copy_from_slice(&rom[end..end + 4]);
    for index in 0..127 {
        let at = SOURCE + (index + 1) * 8 - 1;
        assert_eq!(
            (RT_API.read32)(pointer, 0x0a000000 + at),
            u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
        );
    }
    let before = owner.conditional_records8_stats().unwrap();
    let old = gba_core::rom_data::task_cycle_deltas::SOURCE as usize;
    m.bus.rom[old] = 0xab;
    assert_eq!(m.bus.read8(0x08000000 + old as u32), rom[old]);
    assert_eq!(owner.conditional_records8_stats().unwrap(), before);
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
                m.bus.write16(0x040000dc, 510);
            } else {
                m.cpu.regs[..3].copy_from_slice(&[
                    0x0a000000 + SOURCE,
                    0x02010000,
                    if kind == 2 { 256 } else { 510 },
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
            "full state/clock conditional_records8 transfer{kind}"
        );
        assert!(
            b.bus
                .rom_resources()
                .unwrap()
                .conditional_records8_stats()
                .unwrap()
                .source_bytes
                >= 1016
        );
    }
    serde_json::json!({"all1016_new_typed_bytes":true,"all6fields127slots":true,
        "widths_mirrors_all127recordboundaries_prior_records4_rawsuccessor":true,
        "unavailable_files_Load_slot_Undo_owner_retained":true,"DMA_CpuSet_CpuFastSet_fullstate_clock":3,
        "DMA_CpuSet_halfwords":510,"CpuFastSet_words":256})
}
#[test]
#[ignore = "private EUR conditional_records8 managed/native integration; Wine only, Linux/USA pending"]
fn european_conditional_records8_managed_native_and_lifecycle() {
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
        (84, 83, 76)
    );
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision,
            source.manifest.resources.len()
        ),
        (85, 84, 77)
    );
    extract(&dataset(), &mut source).unwrap();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 78);
    assert_eq!(&source.manifest.resources[..76], prior.resources.as_slice());
    assert_eq!(&source.manifest.coverage[..76], prior.coverage.as_slice());
    for entry in &prior.resources {
        assert_eq!(
            fs::read(dataset().join(&entry.path)).unwrap(),
            fs::read(old.join(&entry.path)).unwrap()
        );
    }
    let entry = &source.manifest.resources[76];
    assert_eq!(entry.id, CONDITIONAL_RECORDS8_ID);
    assert!(entry.dependencies.is_empty());
    assert_eq!(entry.sources.len(), 1);
    assert_eq!(entry.sources[0].offset, SOURCE);
    assert_eq!(entry.sources[0].length, 1016);
    let bytes = fs::read(dataset().join(CONDITIONAL_RECORDS8_FILE)).unwrap();
    let manifest = fs::read(dataset().join(MANIFEST_FILE)).unwrap();
    assert_eq!(
        parse_conditional_records8(&bytes).unwrap(),
        source.conditional_records8.as_ref().unwrap().decoded
    );
    let interrupted = root.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    let mut reached = false;
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| {
        if p.file_name().unwrap() == CONDITIONAL_RECORDS8_FILE {
            reached = true;
            Err("injected conditional_records8 file failure".into())
        } else {
            write_synced(p, b)
        }
    })
    .is_err());
    assert!(reached);
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    assert!(verify(&interrupted, &source).is_err());
    eprintln!("ROM_DATA_CONDITIONAL_RECORDS8_STAGE publication78/prior76exact/interruption PASS");
    let mut m = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    let state = m.save_state().unwrap();
    for case in 0..14 {
        let mut bad = source.manifest.clone();
        let mut data = bytes.clone();
        match case {
            0..=4 => {
                let mut v = parse_conditional_records8(&data).unwrap();
                match case {
                    0 => v.records[0].field00 ^= 1,
                    1 => v.records[0].index ^= 1,
                    2 => v.records[0].operand07 = 255,
                    3 => {
                        v.records.pop();
                    }
                    _ => v.source_offset += 1,
                };
                data = encode_conditional_records8(&v).unwrap();
            }
            5 => bad.resources[76].sources[0].offset += 1,
            6 => bad.resources[76].sources[0].length -= 1,
            7 => bad.resources[76].sources[0].sha256 = "00".repeat(32),
            8 => bad.resources[76].sources[0].role = "other".into(),
            9 => {
                let mut v: serde_json::Value = serde_json::from_slice(&data).unwrap();
                v["unexpected"] = true.into();
                data = serde_json::to_vec(&v).unwrap();
            }
            10 => bad.resources[76]
                .dependencies
                .push(ENCOUNTER_HEADERS_ID.into()),
            11 => bad.resources[76]
                .sources
                .push(gba_core::rom_data::SourceRange {
                    offset: SOURCE_END,
                    length: 2,
                    sha256: gba_core::ffta_rom::digest(
                        &rom[SOURCE_END as usize..SOURCE_END as usize + 2],
                    ),
                    role: "counterfeit-shared".into(),
                }),
            12 => bad.resources[76].format = "counterfeit".into(),
            _ => {
                bad.coverage.pop();
            }
        }
        bad.resources[76].sha256 = gba_core::ffta_rom::digest(&data);
        fs::write(dataset().join(CONDITIONAL_RECORDS8_FILE), data).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&bad).unwrap(),
        )
        .unwrap();
        assert!(
            attach_from_directory(&mut m, &dataset()).is_err(),
            "conditional_records8 attach guard{case}"
        );
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), state);
    }
    fs::write(dataset().join(CONDITIONAL_RECORDS8_FILE), &bytes).unwrap();
    fs::write(dataset().join(MANIFEST_FILE), &manifest).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let saved = source.conditional_records8.take();
    assert!(source.validate_files(files.files()).is_err());
    source.conditional_records8 = saved;
    let mut missing = files.files();
    missing.conditional_records8 = None;
    assert!(source.validate_files(missing).is_err());
    eprintln!("ROM_DATA_CONDITIONAL_RECORDS8_STAGE attach14/fullset2/failedownerfullstate PASS");
    let native = native_prefixes(&rom);
    eprintln!(
        "ROM_DATA_CONDITIONAL_RECORDS8_STAGE native17/fullstate-clock/source1016poisoned PASS"
    );
    let bus = bus_lifecycle(&rom);
    let result = serde_json::json!({"status":"PASS","revision":[1,85,84],"files":78,"resources":77,"prior76_records_files_exact":true,"source_bytes":1016,"attach_guards":14,"fullset_guards":2,"newfile_interruption":true,"native":native,"bus_lifecycle":bus,"dataset_sha256":owner.dataset_sha256(),"manifest_sha256":gba_core::ffta_rom::digest(&manifest),"conditional_records8_JSON_sha256":gba_core::ffta_rom::digest(&bytes),"actualCLI":"pending","ordinary_positive":"pending","Linux":"pending","USA":"pending","nativeWindows":"pending","new32_9":"pending","performance_measurements":false});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_CONDITIONAL_RECORDS8_RUNTIME e85-d84/files78/resources77/attach14/fullset2/native17/DMA3 PASS");
}

#[test]
#[ignore = "remaining EUR native full-selection diagnosis and lifecycle only; Wine"]
fn european_conditional_records8_remaining_native_and_lifecycle() {
    assert!(std::env::var_os("FFTA_CONDITIONAL_RECORDS8_REMAINING").is_some());
    assert!(dataset().join(MANIFEST_FILE).is_file());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let native = native_prefixes(&rom);
    let bus = bus_lifecycle(&rom);
    let report = serde_json::json!({"status":"PASS-remaining-diagnostics","native":native,"bus_lifecycle":bus,"publication_attach_first16_not_repeated":true});
    fs::write(
        root().join("remaining-result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_CONDITIONAL_RECORDS8_REMAINING fullstate-clock1/bus-lifecycle PASS; callback-gap requires diagnosis");
}

#[test]
#[ignore = "remaining EUR final complete native read observer only; Wine"]
fn european_conditional_records8_final_native_observer() {
    assert!(std::env::var_os("FFTA_CONDITIONAL_RECORDS8_REMAINING").is_some());
    assert!(std::env::var_os("FFTA_CONDITIONAL_RECORDS8_FINAL_OBSERVER").is_some());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let native = native_prefixes(&rom);
    assert_eq!(native["full_state_clock_cases"], 1);
    assert_eq!(native["compiled_new_source_bytes"], 609);
    assert_eq!(native["fallback_source_bytes"], serde_json::json!([2, 0]));
    assert_eq!(native["diagnostic_only"], false);
    let report = serde_json::json!({"status":"PASS-final-native-observer","native":native,
        "publication_attach_first16_bus_not_repeated":true});
    fs::write(
        root().join("final-native-result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_CONDITIONAL_RECORDS8_FINAL nativeactual609/direct607/fallback2/fullstate-clock PASS");
}
