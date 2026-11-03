//! Focused Wine integration for parameter_records27 and its typed/raw borders.
use super::*;
use gba_core::{capi::RT_API, rom_data::parameter_records27::SOURCE, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_PARAMETER_RECORDS27_RUNTIME_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
thread_local! {static FALLBACK:Cell<[u64;2]>=const{Cell::new([0;2])};static COMPILED:Cell<[u64;2]>=const{Cell::new([0;2])};}
fn counts(m: *mut std::ffi::c_void) -> [u64; 2] {
    unsafe {
        let r = (&*(m as *const Machine)).bus.rom_resources().unwrap();
        [
            r.parameter_records27_stats().unwrap().source_bytes,
            r.event_records12_stats().unwrap().source_bytes,
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

const RUNS: [(u32, u32); 2] = [
    (SOURCE, SOURCE + 51 * 27),
    (SOURCE + 115 * 27, SOURCE + 123 * 27),
];
fn owned(at: u32) -> bool {
    RUNS.iter().any(|&(a, b)| a <= at && at < b)
}
fn poison(m: &mut Machine) {
    for (a, b) in RUNS {
        m.bus.rom[a as usize..b as usize].fill(0xcd);
    }
}
#[derive(Clone, Copy)]
struct Call {
    entry: u32,
    address: u32,
    mirror: u32,
    kind: u8,
    end: u32,
    index: u32,
}
fn family(c: Call) -> usize {
    usize::from(c.kind)
}
fn stop(c: Call) -> u32 {
    0x08000000 + c.end
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
    } else {
        let stage = usize::from(c.kind - 1);
        let base = [0, 0, 6, 4, 5, 6, 6, 0, 4, 0, 4, 4][stage];
        m.cpu.regs[base] = 0x08000000 + SOURCE + 27 * (c.index & 255);
        if stage == 11 {
            m.cpu.regs[4] = 0x02000100;
            m.bus.ewram[0x1a8] = c.index as u8;
        }
    }
    m.cpu.regs[13] = 0x03007c00;
    m.cpu.regs[14] = 0x03007f01;
    m.cpu.regs[15] = 0x08000000 + c.entry;
    m.cpu.cpsr = 0xbf;
}
fn interpreted(m: &mut Machine, c: Call, visited: &mut BTreeSet<(u32, bool)>) -> [u64; 2] {
    let mut n = 0;
    let mut observed = [0; 2];
    while m.cpu.regs[15] != stop(c) {
        assert!(n < 64);
        let pc = m.cpu.regs[15];
        assert!(m.cpu.thumb());
        let at = (pc - 0x08000000) as usize;
        let op = u16::from_le_bytes([m.bus.rom[at], m.bus.rom[at + 1]]);
        let width = match op & 0xf800 {
            0x7800 => 1,
            0x8800 => 2,
            0x6800 => 4,
            _ => 0,
        };
        let load = if width != 0 {
            Some((
                m.cpu.regs[usize::from((op >> 3) & 7)] + u32::from((op >> 6) & 31) * width,
                width,
            ))
        } else if op & 0xfe00 == 0x5600 {
            Some((
                m.cpu.regs[usize::from((op >> 3) & 7)] + m.cpu.regs[usize::from((op >> 6) & 7)],
                1,
            ))
        } else {
            None
        };
        if let Some((address, width)) = load {
            if (8..=13).contains(&(address >> 24)) {
                let physical = (address & 0x1ffffff) & !(width - 1);
                for offset in physical..physical + width {
                    if owned(offset) {
                        observed[0] += 1;
                    }
                    if (SOURCE - 3072..SOURCE).contains(&offset) {
                        observed[1] += 1;
                    }
                }
            }
        }
        visited.insert((pc, true));
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    observed
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
    let blank = Call {
        entry: 0,
        address: 0,
        mirror: 0x08000000,
        kind: 0,
        end: 0,
        index: 0,
    };
    for (i, off) in [0, 1, 7, 9, 10, 12, 13, 14, 16, 18, 21, 26]
        .into_iter()
        .enumerate()
    {
        calls.push(Call {
            entry: 0xd25bc,
            address: SOURCE + 122 * 27 + off,
            mirror: [0x08000000, 0x0a000000, 0x0c000000][i % 3],
            end: 0xd25be,
            ..blank
        });
    }
    for (entry, address) in [
        (0xd25bc, SOURCE - 1),
        (0xd25bc, RUNS[0].1),
        (0xd25bc, RUNS[1].0 - 1),
        (0xd25bc, RUNS[1].1),
        (0xd25bc, SOURCE + 255 * 27),
        (0xd08a2, SOURCE - 1),
        (0xd08a2, RUNS[0].1 - 2),
        (0xd08a2, RUNS[1].0 - 2),
    ] {
        calls.push(Call {
            entry,
            address,
            end: entry + 2,
            ..blank
        });
    }
    for (stage, (entry, mut end)) in [
        (0xd25bc, 0xd25be),
        (0x47216, 0x47226),
        (0x34d74, 0x34d82),
        (0x405b2, 0x405cc),
        (0x406da, 0x406e6),
        (0x40d22, 0x40d2a),
        (0x40d50, 0x40d58),
        (0x40fb0, 0x40fb8),
        (0x40b5a, 0x40b66),
        (0x409de, 0x409ea),
        (0x4a1ee, 0x4a202),
        (0x34d40, 0x34d52),
    ]
    .into_iter()
    .enumerate()
    {
        let index = 122;
        if stage == 10 {
            let at = SOURCE as usize + 27 * index as usize + 18;
            if rom[at..at + 2] == [0, 0] {
                end = 0x4a206;
            }
        }
        calls.push(Call {
            entry,
            end,
            index,
            kind: stage as u8 + 1,
            ..blank
        });
    }
    calls.push(Call {
        entry: 0x34d40,
        end: 0x34d52,
        index: 0x100 + 51,
        kind: 12,
        ..blank
    });
    assert_eq!(calls.len(), 33);
    let remaining = std::env::var_os("FFTA_PARAMETER_RECORDS27_RUNTIME_REMAINING").is_some();
    if remaining {
        calls.drain(..17);
    }
    let mut raw = crate::make_machine(rom.to_vec(), None);
    let initial = raw.save_state().unwrap();
    let mut visited: [_; 13] = std::array::from_fn(|_| BTreeSet::new());
    let mut expected = Vec::new();
    let mut expected_counts = Vec::new();
    for &c in &calls {
        raw.load_state(&initial).unwrap();
        setup(&mut raw, c);
        expected_counts.push(interpreted(&mut raw, c, &mut visited[family(c)]));
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
        assert!(!visited.is_empty());
        assert!(calls
            .iter()
            .filter(|c| family(**c) == group)
            .all(|&c| !visited.iter().any(|&(pc, _)| pc == stop(c))));
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
        let object = root().join(format!("parameter_records27-family{group}.o"));
        if !remaining {
            recomp_core::tcc_memory::build(
                &Analysis { blocks },
                &view,
                &recomp_core::tcc_memory::library_path().unwrap(),
                &object,
                &mut |_| {},
            )
            .unwrap();
        }
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
        interp_one,
        read8,
        read16,
        read32,
        ..RT_API
    };
    COMPILED.with(|c| c.set([0; 2]));
    FALLBACK.with(|c| c.set([0; 2]));
    let mut dispatch = 0;
    let mut observations = Vec::new();
    for (i, &c) in calls.iter().enumerate() {
        for m in [&mut typed, &mut native] {
            m.load_state(&initial).unwrap();
            setup(m, c);
            poison(m);
            m.bus.rom[SOURCE as usize - 2..SOURCE as usize].fill(0xab);
        }
        let typed_before = counts(&mut typed as *mut Machine as *mut std::ffi::c_void);
        assert_eq!(
            interpreted(&mut typed, c, &mut BTreeSet::new()),
            expected_counts[i]
        );
        let typed_after = counts(&mut typed as *mut Machine as *mut std::ffi::c_void);
        let typed_delta = std::array::from_fn::<_, 2, _>(|j| typed_after[j] - typed_before[j]);
        let native_before = counts(&mut native as *mut Machine as *mut std::ffi::c_void);
        let before = COMPILED.with(Cell::get);
        while native.cpu.regs[15] != stop(c) {
            assert!(dispatch < 30000);
            native.bus.frame_ready = false;
            let actual_before = counts(&mut native as *mut Machine as *mut std::ffi::c_void);
            let api_before = COMPILED.with(Cell::get);
            tables[family(c)]
                .get(native.cpu.regs[15] | u32::from(native.cpu.thumb()))
                .unwrap_or_else(|| panic!("uncompiled case{i} PC{:08x}", native.cpu.regs[15]))(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            let actual_after = counts(&mut native as *mut Machine as *mut std::ffi::c_void);
            let api_after = COMPILED.with(Cell::get);
            assert_eq!(
                std::array::from_fn::<_, 2, _>(|j| actual_after[j] - actual_before[j]),
                std::array::from_fn::<_, 2, _>(|j| api_after[j] - api_before[j]),
                "all callbacks case{i}"
            );
            dispatch += 1;
        }
        let after = COMPILED.with(Cell::get);
        let native_after = counts(&mut native as *mut Machine as *mut std::ffi::c_void);
        let native_delta = std::array::from_fn::<_, 2, _>(|j| native_after[j] - native_before[j]);
        let api_delta = std::array::from_fn::<_, 2, _>(|j| after[j] - before[j]);
        observations.push(serde_json::json!({"case":i,"entry":c.entry,"kind":c.kind,"expected":expected_counts[i],"typed":typed_delta,"native":native_delta,"callbacks":api_delta}));
        fs::write(
            root().join(if remaining {
                "native-observations-remaining.json"
            } else {
                "native-observations.json"
            }),
            serde_json::to_vec_pretty(&observations).unwrap(),
        )
        .unwrap();
        assert_eq!(typed_delta, expected_counts[i], "typed case{i}");
        assert_eq!(native_delta, expected_counts[i], "native case{i}");
        assert_eq!(api_delta, expected_counts[i], "callbacks case{i}");
        for (kind, m) in [("typed", &mut typed), ("native", &mut native)] {
            m.bus.rom.copy_from_slice(rom);
            check_state(m, &expected[i], &format!("{kind}-case{i}"));
        }
    }
    let observed = COMPILED.with(Cell::get);
    assert_eq!(
        observed,
        expected_counts
            .iter()
            .fold([0; 2], |a, b| [a[0] + b[0], a[1] + b[1]])
    );
    assert!(observed[0] > 0 && observed[1] > 0);
    assert_eq!(libraries.len(), 13);
    serde_json::json!({"full_state_clock_cases":calls.len(),"single_load_cases":if remaining {3} else {20},"original_prefix_cases":13,"reused_guarded_original_objects":remaining,
        "compiled_new_source_bytes":observed[0],"compiled_prior_event_bytes":observed[1],"fallback_source_bytes":FALLBACK.with(Cell::get),
        "compiled_instructions":compiled_instructions,"native_case_families":13,"each_family_excludes_stop_boundaries":true,
        "dispatch_coverage_only":dispatch,"all1593_raw_source_bytes_and_prior2_poisoned_after_Load_setup":true,"observations":observations})
}
fn bus_lifecycle(rom: &[u8]) -> serde_json::Value {
    let mut m = crate::make_machine(rom.to_vec(), None);
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    let initial = m.save_state().unwrap();
    poison(&mut m);
    for (a, b) in RUNS {
        for at in a..b {
            assert_eq!(m.bus.read8(0x0c000000 + at), rom[at as usize]);
        }
    }
    let s = owner.parameter_records27_stats().unwrap();
    assert_eq!(
        [
            s.bus_reads,
            s.source_bytes,
            s.header_bytes,
            s.packed_bytes,
            s.value07_bytes,
            s.byte09_bytes,
            s.value0a_bytes,
            s.signed0c_bytes,
            s.flags0d_bytes,
            s.operands0e_bytes,
            s.operands10_bytes,
            s.value12_bytes,
            s.signed15_bytes,
            s.opaque_bytes,
            s.records_used
        ],
        [1593, 1593, 59, 354, 118, 59, 118, 59, 59, 118, 118, 118, 59, 354, 59]
    );
    let pointer = &mut m as *mut Machine as *mut std::ffi::c_void;
    for edge in [RUNS[0].0, RUNS[0].1, RUNS[1].0, RUNS[1].1] {
        for delta in [-3, -1, 0, 1] {
            let at = (edge as i32 + delta) as usize;
            assert_eq!(
                (RT_API.read16)(pointer, 0x0a000000 + at as u32),
                u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap()))
            );
            assert_eq!(
                (RT_API.read32)(pointer, 0x08000000 + at as u32),
                u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
            );
        }
    }
    for index in (0..=50).chain(115..=122) {
        let at = SOURCE + (index + 1) * 27 - 1;
        assert_eq!(
            (RT_API.read32)(pointer, 0x0a000000 + at),
            u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
        );
    }
    // Prior event bytes remain typed. Unclassified bytes retain their actual raw values.
    m.bus.rom[SOURCE as usize - 2..SOURCE as usize].fill(0xab);
    m.bus.rom[RUNS[0].1 as usize..RUNS[1].0 as usize].fill(0xef);
    m.bus.rom[RUNS[1].1 as usize..0x71e0e4].fill(0xef);
    for at in [SOURCE - 1, RUNS[0].1 - 1, RUNS[1].0 - 1, RUNS[1].1 - 1] {
        let bytes = std::array::from_fn(|i| {
            let a = at + i as u32;
            if owned(a) || (SOURCE - 2..SOURCE).contains(&a) {
                rom[a as usize]
            } else {
                m.bus.rom[a as usize]
            }
        });
        assert_eq!(
            (RT_API.read32)(pointer, 0x08000000 + at),
            u32::from_le_bytes(bytes)
        );
    }
    assert!(owner.event_records12_stats().unwrap().source_bytes > 0);
    for at in [RUNS[0].1, RUNS[1].0 - 1, RUNS[1].1, 0x71e0e0] {
        let before = owner.parameter_records27_stats().unwrap();
        assert_eq!(m.bus.read8(0x08000000 + at), 0xef);
        assert_eq!(owner.parameter_records27_stats().unwrap(), before);
    }
    let before = owner.parameter_records27_stats().unwrap();
    let old = gba_core::rom_data::task_cycle_deltas::SOURCE as usize;
    m.bus.rom[old] = 0xab;
    assert_eq!(m.bus.read8(0x08000000 + old as u32), rom[old]);
    assert_eq!(owner.parameter_records27_stats().unwrap(), before);
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
    poison(&mut m);
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
                m.bus.write16(0x040000dc, 1664);
            } else {
                m.cpu.regs[..3].copy_from_slice(&[
                    0x0a000000 + SOURCE,
                    0x02010000,
                    if kind == 2 { 832 } else { 1664 },
                ]);
                m.cpu.regs[15] = 0x02000000;
                m.cpu.cpsr = 0xbf;
                m.bus
                    .write16(0x02000000, if kind == 2 { 0xdf0c } else { 0xdf0b });
            }
        }
        poison(&mut b);
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
            "fullstate-clock sparse27 transfer{kind}"
        );
        assert!(
            b.bus
                .rom_resources()
                .unwrap()
                .parameter_records27_stats()
                .unwrap()
                .source_bytes
                >= 1593
        );
    }
    serde_json::json!({"all1593_new_typed_bytes":true,"all12classes59records":true,"all59recordborders_mirrors_widths_prior_event_and_rawholes":true,
        "unavailable_files_Load_slot_Undo_owner_retained":true,"DMA_CpuSet_CpuFastSet_fullstate_clock":3,"DMA_CpuSet_halfwords":1664,"CpuFastSet_words":832})
}
#[test]
#[ignore = "private EUR parameter_records27 managed/native integration; Wine only, Linux/USA pending"]
fn european_parameter_records27_managed_native_and_lifecycle() {
    let root = root();
    if std::env::var_os("FFTA_PARAMETER_RECORDS27_RUNTIME_REMAINING").is_some() {
        assert!(root.is_dir());
        let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
        let native = native_prefixes(&rom);
        eprintln!("ROM_DATA_PARAMETER_RECORDS27_STAGE remaining-native16/fullstate-clock PASS");
        let bus = bus_lifecycle(&rom);
        let result = serde_json::json!({"status":"PASS-remaining","native":native,"bus_lifecycle":bus,"publication_attach_first17_native":"retained prior acceptance; not replayed","performance_measurements":false});
        fs::write(
            root.join("result-remaining.json"),
            serde_json::to_vec_pretty(&result).unwrap(),
        )
        .unwrap();
        eprintln!("ROM_DATA_PARAMETER_RECORDS27_RUNTIME remaining16/bus1593/LoadUndo/DMA3 PASS");
        return;
    }
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
        (88, 87, 80)
    );
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision,
            source.manifest.resources.len()
        ),
        (89, 88, 81)
    );
    extract(&dataset(), &mut source).unwrap();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 82);
    assert_eq!(&source.manifest.resources[..80], prior.resources.as_slice());
    assert_eq!(&source.manifest.coverage[..80], prior.coverage.as_slice());
    for entry in &prior.resources {
        assert_eq!(
            fs::read(dataset().join(&entry.path)).unwrap(),
            fs::read(old.join(&entry.path)).unwrap()
        );
    }
    let entry = &source.manifest.resources[80];
    assert_eq!(entry.id, PARAMETER_RECORDS27_ID);
    assert!(entry.dependencies.is_empty());
    assert_eq!(entry.sources.len(), 2);
    assert_eq!(entry.sources[0].offset, SOURCE);
    assert_eq!(entry.sources[0].length, 1377);
    assert_eq!(
        (entry.sources[1].offset, entry.sources[1].length),
        (RUNS[1].0, 216)
    );
    let bytes = fs::read(dataset().join(PARAMETER_RECORDS27_FILE)).unwrap();
    let manifest = fs::read(dataset().join(MANIFEST_FILE)).unwrap();
    assert_eq!(
        parse_parameter_records27(&bytes).unwrap(),
        source.parameter_records27.as_ref().unwrap().decoded
    );
    let interrupted = root.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    let mut reached = false;
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| {
        if p.file_name().unwrap() == PARAMETER_RECORDS27_FILE {
            reached = true;
            Err("injected parameter_records27 file failure".into())
        } else {
            write_synced(p, b)
        }
    })
    .is_err());
    assert!(reached);
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    assert!(verify(&interrupted, &source).is_err());
    eprintln!("ROM_DATA_PARAMETER_RECORDS27_STAGE publication82/prior80exact/interruption PASS");
    let mut m = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    let state = m.save_state().unwrap();
    for case in 0..14 {
        let mut bad = source.manifest.clone();
        let mut data = bytes.clone();
        match case {
            0..=4 => {
                let mut v = parse_parameter_records27(&data).unwrap();
                match case {
                    0 => v.records[0].signed0c ^= 1,
                    1 => v.records[0].index ^= 1,
                    2 => v.records[0].opaque16[1] ^= 0x80,
                    3 => {
                        v.records.pop();
                    }
                    _ => v.source_offset += 1,
                };
                data = encode_parameter_records27(&v).unwrap();
            }
            5 => bad.resources[80].sources[0].offset += 1,
            6 => bad.resources[80].sources[1].length -= 1,
            7 => bad.resources[80].sources[0].sha256 = "00".repeat(32),
            8 => bad.resources[80].sources[0].role = "other".into(),
            9 => {
                let mut v: serde_json::Value = serde_json::from_slice(&data).unwrap();
                v["unexpected"] = true.into();
                data = serde_json::to_vec(&v).unwrap();
            }
            10 => bad.resources[80]
                .dependencies
                .push(ENCOUNTER_HEADERS_ID.into()),
            11 => bad.resources[80]
                .sources
                .push(gba_core::rom_data::SourceRange {
                    offset: RUNS[0].1,
                    length: 2,
                    sha256: gba_core::ffta_rom::digest(
                        &rom[RUNS[0].1 as usize..RUNS[0].1 as usize + 2],
                    ),
                    role: "counterfeit-shared".into(),
                }),
            12 => bad.resources[80].format = "counterfeit".into(),
            _ => {
                bad.coverage.pop();
            }
        }
        bad.resources[80].sha256 = gba_core::ffta_rom::digest(&data);
        fs::write(dataset().join(PARAMETER_RECORDS27_FILE), data).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&bad).unwrap(),
        )
        .unwrap();
        assert!(
            attach_from_directory(&mut m, &dataset()).is_err(),
            "parameter_records27 attach guard{case}"
        );
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), state);
    }
    fs::write(dataset().join(PARAMETER_RECORDS27_FILE), &bytes).unwrap();
    fs::write(dataset().join(MANIFEST_FILE), &manifest).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let saved = source.parameter_records27.take();
    assert!(source.validate_files(files.files()).is_err());
    source.parameter_records27 = saved;
    let mut missing = files.files();
    missing.parameter_records27 = None;
    assert!(source.validate_files(missing).is_err());
    eprintln!("ROM_DATA_PARAMETER_RECORDS27_STAGE attach14/fullset2/failedownerfullstate PASS");
    let native = native_prefixes(&rom);
    eprintln!(
        "ROM_DATA_PARAMETER_RECORDS27_STAGE native33/fullstate-clock/source1593poisoned PASS"
    );
    let bus = bus_lifecycle(&rom);
    let result = serde_json::json!({"status":"PASS","revision":[1,89,88],"files":82,"resources":81,"prior80_records_files_exact":true,"source_bytes":1593,"attach_guards":14,"fullset_guards":2,"newfile_interruption":true,"native":native,"bus_lifecycle":bus,"dataset_sha256":owner.dataset_sha256(),"manifest_sha256":gba_core::ffta_rom::digest(&manifest),"parameter_records27_JSON_sha256":gba_core::ffta_rom::digest(&bytes),"actualCLI":"pending","ordinary_positive":"pending","Linux":"pending","USA":"pending","nativeWindows":"pending","new32_9":"pending","performance_measurements":false});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_PARAMETER_RECORDS27_RUNTIME e89-d88/files82/resources81/attach14/fullset2/native33/DMA3 PASS");
}
