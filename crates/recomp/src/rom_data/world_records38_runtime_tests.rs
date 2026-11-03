//! Focused Wine world-record integration; controlled probes, no ordinary fixture forcing.
use super::*;
use gba_core::{capi::RT_API, rom_data::world_records38::SOURCE, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_WORLD_RECORDS38_RUNTIME_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
thread_local! {static FALLBACK:Cell<[u64;2]>=const{Cell::new([0;2])};static COMPILED:Cell<[u64;2]>=const{Cell::new([0;2])};}
fn counts(m: *mut std::ffi::c_void) -> [u64; 2] {
    unsafe {
        let r = (&*(m as *const Machine)).bus.rom_resources().unwrap();
        [
            r.world_records38_stats().unwrap().source_bytes,
            r.parameter_records27_stats().unwrap().source_bytes,
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

const END: u32 = SOURCE + 1178;
fn owned(at: u32) -> bool {
    (SOURCE..END).contains(&at)
}
fn prior_owned(at: u32) -> bool {
    let base = gba_core::rom_data::parameter_records27::SOURCE;
    (base..base + 51 * 27).contains(&at) || (base + 115 * 27..base + 123 * 27).contains(&at)
}
fn poison(m: &mut Machine) {
    m.bus.rom[SOURCE as usize..END as usize].fill(0xcd);
}
fn poison_prior(m: &mut Machine) {
    m.bus.rom[0x71e054..0x71e059].fill(0xab);
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
        let address = 0x08000000 + SOURCE + 7 * 38;
        match c.kind {
            1 => m.cpu.regs[7] = address,
            2 => {
                m.cpu.regs[4] = address;
                m.cpu.regs[7] = 0x02000100;
            }
            3 => {
                m.cpu.regs[7] = address;
                m.cpu.regs[4] = 100;
                m.cpu.regs[0] = 0;
                m.bus.iwram[0x7c00..0x7c04].copy_from_slice(&c.index.to_le_bytes());
            }
            4 => m.cpu.regs[0] = c.index,
            5 => m.cpu.regs[5] = c.index,
            6 => m.cpu.regs[8] = c.index,
            7 => {
                m.cpu.regs[5] = 40;
                m.cpu.regs[0] = 216;
            }
            8 => {
                m.bus.iwram[0x7c00..0x7c04].copy_from_slice(&255u32.to_le_bytes());
            }
            9 => m.cpu.regs[4] = 40 << 24,
            _ => panic!("unknown prefix"),
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
                    if prior_owned(offset) {
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
    for (i, off) in [0, 1, 2, 4, 6, 7, 37].into_iter().enumerate() {
        calls.push(Call {
            entry: 0xd34aa,
            address: SOURCE + off,
            mirror: [0x08000000, 0x0a000000, 0x0c000000][i % 3],
            end: 0xd34ac,
            ..blank
        });
    }
    for (entry, address) in [
        (0xd34aa, SOURCE - 1),
        (0xd34aa, END),
        (0xd34aa, END + 2),
        (0xd34aa, 0x71e058),
        (0x4a13c, SOURCE - 1),
        (0x4a13c, SOURCE + 37),
        (0x4a13c, END - 1),
        (0xd08a2, SOURCE + 37),
        (0xd08a2, END - 1),
        (0xd08a2, 0x71e057),
    ] {
        calls.push(Call {
            entry,
            address,
            end: entry + 2,
            ..blank
        });
    }
    for (kind, entry, end, index) in [
        (1, 0xd35d8, 0xd35e0, 7),
        (2, 0xd4dfe, 0xd4e1e, 7),
        (4, 0xd347e, 0xd3496, 0x11f),
        (5, 0xd4dce, 0xd4de4, u32::MAX),
        (6, 0xd3582, 0xd358c, 29),
        (7, 0xd34c0, 0xd34cc, 7),
        (8, 0xd3566, 0xd3570, 255),
        (9, 0xd34b8, 0xd34bc, 7),
    ] {
        calls.push(Call {
            kind,
            entry,
            end,
            index,
            ..blank
        });
    }
    for count in [0, 30] {
        let threshold = rom[(SOURCE + 7 * 38 + 7 + count) as usize];
        calls.push(Call {
            kind: 3,
            entry: 0xd366c,
            end: if 0 >= 100 - i32::from(threshold) {
                0xd36a0
            } else {
                0xd3678
            },
            index: count,
            ..blank
        });
    }
    assert_eq!(calls.len(), 27);

    let mut raw = crate::make_machine(rom.to_vec(), None);
    let initial = raw.save_state().unwrap();
    let mut visited: [_; 10] = std::array::from_fn(|_| BTreeSet::new());
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
        let object = root().join(format!("world_records38-family{group}.o"));
        {
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
            poison_prior(m);
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
            root().join("native-observations.json"),
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
    assert_eq!(libraries.len(), 10);
    serde_json::json!({"full_state_clock_cases":calls.len(),"single_load_cases":17,"original_prefix_cases":10,
        "compiled_new_source_bytes":observed[0],"compiled_prior_parameter_bytes":observed[1],"fallback_source_bytes":FALLBACK.with(Cell::get),
        "compiled_instructions":compiled_instructions,"native_case_families":10,"each_family_excludes_stop_boundaries":true,
        "dispatch_coverage_only":dispatch,"all1178_raw_source_bytes_and_prior5_poisoned_after_Load_setup":true,"observations":observations})
}
fn bus_lifecycle(rom: &[u8]) -> serde_json::Value {
    let mut m = crate::make_machine(rom.to_vec(), None);
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    let initial = m.save_state().unwrap();
    poison(&mut m);
    for at in SOURCE..END {
        assert_eq!(m.bus.read8(0x0c000000 + at), rom[at as usize]);
    }
    let s = owner.world_records38_stats().unwrap();
    assert_eq!(
        [
            s.bus_reads,
            s.source_bytes,
            s.byte01_bytes,
            s.value02_bytes,
            s.value04_bytes,
            s.threshold_bytes,
            s.opaque_bytes,
            s.records_used
        ],
        [1178, 1178, 31, 62, 62, 961, 62, 31]
    );
    let pointer = &mut m as *mut Machine as *mut std::ffi::c_void;
    for edge in [SOURCE, END] {
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
    for index in 0..31 {
        let at = SOURCE + (index + 1) * 38 - 1;
        assert_eq!(
            (RT_API.read32)(pointer, 0x0a000000 + at),
            u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
        );
    }
    poison_prior(&mut m);
    m.bus.rom[0x71e059..SOURCE as usize].fill(0xef);
    m.bus.rom[END as usize..END as usize + 6].fill(0xef);
    for at in [0x71e056, SOURCE - 1, END - 1] {
        let b = std::array::from_fn(|i| {
            let a = at + i as u32;
            if owned(a) || prior_owned(a) {
                rom[a as usize]
            } else {
                m.bus.rom[a as usize]
            }
        });
        assert_eq!(
            (RT_API.read32)(pointer, 0x08000000 + at),
            u32::from_le_bytes(b)
        );
    }
    assert!(owner.parameter_records27_stats().unwrap().source_bytes > 0);
    for at in [0x71e059, SOURCE - 1, END, END + 2] {
        let before = owner.world_records38_stats().unwrap();
        assert_eq!(m.bus.read8(0x08000000 + at), 0xef);
        assert_eq!(owner.world_records38_stats().unwrap(), before);
    }
    let old = gba_core::rom_data::task_cycle_deltas::SOURCE as usize;
    let before = owner.world_records38_stats().unwrap();
    m.bus.rom[old] = 0xab;
    assert_eq!(m.bus.read8(0x08000000 + old as u32), rom[old]);
    assert_eq!(owner.world_records38_stats().unwrap(), before);
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
                m.bus.write32(0x040000d4, 0x0871e040);
                m.bus.write32(0x040000d8, 0x02010000);
                m.bus.write16(0x040000dc, 704);
            } else {
                m.cpu.regs[..3].copy_from_slice(&[
                    0x0a71e040,
                    0x02010000,
                    if kind == 2 { 352 } else { 704 },
                ]);
                m.cpu.regs[15] = 0x02000000;
                m.cpu.cpsr = 0xbf;
                m.bus
                    .write16(0x02000000, if kind == 2 { 0xdf0c } else { 0xdf0b });
            }
        }
        poison(&mut b);
        poison_prior(&mut b);
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
            "fullstate-clock world38 transfer{kind}"
        );
        assert!(
            b.bus
                .rom_resources()
                .unwrap()
                .world_records38_stats()
                .unwrap()
                .source_bytes
                >= 1178
        );
        assert!(
            b.bus
                .rom_resources()
                .unwrap()
                .parameter_records27_stats()
                .unwrap()
                .source_bytes
                >= 5
        );
    }
    serde_json::json!({"all1178_new_typed_bytes":true,"all5classes31records":true,"all31recordborders_mirrors_widths_prior_parameter_and_rawgap_nextroot":true,
        "unavailable_files_Load_slot_Undo_owner_retained":true,"DMA_CpuSet_CpuFastSet_fullstate_clock":3,"DMA_CpuSet_halfwords":704,"CpuFastSet_words":352})
}

#[test]
#[ignore = "private EUR world_records38 managed/native integration; Wine only, Linux/USA pending"]
fn european_world_records38_managed_native_and_lifecycle() {
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
        (89, 88, 81)
    );
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision,
            source.manifest.resources.len()
        ),
        (90, 89, 82)
    );
    extract(&dataset(), &mut source).unwrap();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 83);
    assert_eq!(&source.manifest.resources[..81], prior.resources.as_slice());
    assert_eq!(&source.manifest.coverage[..81], prior.coverage.as_slice());
    for entry in &prior.resources {
        assert_eq!(
            fs::read(dataset().join(&entry.path)).unwrap(),
            fs::read(old.join(&entry.path)).unwrap()
        );
    }
    let entry = &source.manifest.resources[81];
    assert_eq!(entry.id, WORLD_RECORDS38_ID);
    assert!(entry.dependencies.is_empty());
    assert_eq!(entry.sources.len(), 1);
    assert_eq!(entry.sources[0].offset, SOURCE);
    assert_eq!(entry.sources[0].length, 1178);
    let bytes = fs::read(dataset().join(WORLD_RECORDS38_FILE)).unwrap();
    let manifest = fs::read(dataset().join(MANIFEST_FILE)).unwrap();
    assert_eq!(
        parse_world_records38(&bytes).unwrap(),
        source.world_records38.as_ref().unwrap().decoded
    );
    let interrupted = root.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    let mut reached = false;
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| {
        if p.file_name().unwrap() == WORLD_RECORDS38_FILE {
            reached = true;
            Err("injected world_records38 file failure".into())
        } else {
            write_synced(p, b)
        }
    })
    .is_err());
    assert!(reached);
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    assert!(verify(&interrupted, &source).is_err());
    eprintln!("ROM_DATA_WORLD_RECORDS38_STAGE publication83/prior81exact/interruption PASS");
    let mut m = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    let state = m.save_state().unwrap();
    for case in 0..14 {
        let mut bad = source.manifest.clone();
        let mut data = bytes.clone();
        match case {
            0..=4 => {
                let mut v = parse_world_records38(&data).unwrap();
                match case {
                    0 => v.records[0].byte01 ^= 1,
                    1 => v.records[0].index ^= 1,
                    2 => v.records[0].opaque06 ^= 0x80,
                    3 => {
                        v.records.pop();
                    }
                    _ => v.source_offset += 1,
                };
                data = encode_world_records38(&v).unwrap();
            }
            5 => bad.resources[81].sources[0].offset += 1,
            6 => bad.resources[81].sources[0].length -= 1,
            7 => bad.resources[81].sources[0].sha256 = "00".repeat(32),
            8 => bad.resources[81].sources[0].role = "other".into(),
            9 => {
                let mut v: serde_json::Value = serde_json::from_slice(&data).unwrap();
                v["unexpected"] = true.into();
                data = serde_json::to_vec(&v).unwrap();
            }
            10 => bad.resources[81]
                .dependencies
                .push(ENCOUNTER_HEADERS_ID.into()),
            11 => bad.resources[81]
                .sources
                .push(gba_core::rom_data::SourceRange {
                    offset: END,
                    length: 2,
                    sha256: gba_core::ffta_rom::digest(&rom[END as usize..END as usize + 2]),
                    role: "counterfeit-shared".into(),
                }),
            12 => bad.resources[81].format = "counterfeit".into(),
            _ => {
                bad.coverage.pop();
            }
        }
        bad.resources[81].sha256 = gba_core::ffta_rom::digest(&data);
        fs::write(dataset().join(WORLD_RECORDS38_FILE), data).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&bad).unwrap(),
        )
        .unwrap();
        assert!(
            attach_from_directory(&mut m, &dataset()).is_err(),
            "world_records38 attach guard{case}"
        );
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), state);
    }
    fs::write(dataset().join(WORLD_RECORDS38_FILE), &bytes).unwrap();
    fs::write(dataset().join(MANIFEST_FILE), &manifest).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let saved = source.world_records38.take();
    assert!(source.validate_files(files.files()).is_err());
    source.world_records38 = saved;
    let mut missing = files.files();
    missing.world_records38 = None;
    assert!(source.validate_files(missing).is_err());
    eprintln!("ROM_DATA_WORLD_RECORDS38_STAGE attach14/fullset2/failedownerfullstate PASS");
    let native = native_prefixes(&rom);
    eprintln!("ROM_DATA_WORLD_RECORDS38_STAGE native27/fullstate-clock/source1178poisoned PASS");
    let bus = bus_lifecycle(&rom);
    let result = serde_json::json!({"status":"PASS","revision":[1,90,89],"files":83,"resources":82,"prior81_records_files_exact":true,"source_bytes":1178,"attach_guards":14,"fullset_guards":2,"newfile_interruption":true,"native":native,"bus_lifecycle":bus,"dataset_sha256":owner.dataset_sha256(),"manifest_sha256":gba_core::ffta_rom::digest(&manifest),"world_records38_JSON_sha256":gba_core::ffta_rom::digest(&bytes),"actualCLI":"pending","ordinary_positive":"pending","Linux":"pending","USA":"pending","nativeWindows":"pending","new32_9":"pending","performance_measurements":false});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_WORLD_RECORDS38_RUNTIME e90-d89/files83/resources82/attach14/fullset2/native27/DMA3 PASS");
}
