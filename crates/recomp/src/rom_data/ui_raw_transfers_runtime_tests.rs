//! Focused Wine raw UI transfers integration; controlled probes, no ordinary fixture forcing.
use super::*;
use gba_core::{Bus, Machine, capi::RT_API};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_RAW_TRANSFERS_RUNTIME_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
thread_local! {static FALLBACK:Cell<[u64;2]>=const{Cell::new([0;2])};static COMPILED:Cell<[u64;2]>=const{Cell::new([0;2])};}
fn counts(m: *mut std::ffi::c_void) -> [u64; 2] {
    unsafe {
        let r = (&*(m as *const Machine)).bus.rom_resources().unwrap();
        [
            r.ui_raw_transfers_stats().unwrap().source_bytes,
            r.indexed_graphics_stats().unwrap().source_bytes,
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

extern "C" fn write32(m: *mut std::ffi::c_void, at: u32, value: u32) {
    let before = counts(m);
    (RT_API.write32)(m, at, value);
    account(before, counts(m));
}
fn spans() -> &'static Vec<(u32, u32)> {
    static SPANS: std::sync::OnceLock<Vec<(u32, u32)>> = std::sync::OnceLock::new();
    SPANS.get_or_init(|| {
        let source = fs::read(root().parent().unwrap().join("source-spans.json")).unwrap();
        let ranges: Vec<gba_core::rom_data::SourceRange> = serde_json::from_slice(&source).unwrap();
        ranges
            .into_iter()
            .map(|r| (r.offset, r.offset + r.length))
            .collect()
    })
}
fn owned(at: u32) -> bool {
    spans().iter().any(|&(a, b)| (a..b).contains(&at))
}
fn prior_owned(at: u32) -> bool {
    (0x71e668..0x71e66c).contains(&at)
}
fn poison(m: &mut Machine) {
    for &(a, b) in spans() {
        m.bus.rom[a as usize..b as usize].fill(0xcd);
    }
}
fn poison_prior(m: &mut Machine) {
    m.bus.rom[0x71e668..0x71e66c].fill(0xab);
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
    if c.kind == 1 || c.kind == 3 {
        0x03007f00
    } else {
        0x08000000 + c.end
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
    } else if c.kind == 1 {
        m.cpu.regs[..3].copy_from_slice(&[0x08000000 + c.address, 0x06016080, 0x180]);
    } else if c.kind == 3 {
        m.bus.iwram.fill(0);
        m.bus.write32(0x03002bcc, 0x02010000);
        m.bus.write32(0x02010000, 0x00000100);
        m.bus.write32(0x02010004, 0x08000000 + c.address);
        m.bus.write32(0x02010008, 0x06013800);
        m.bus.write32(0x0201000c, 0x80000040);
        m.bus.write32(0x02010018, 0);
    } else {
        m.bus.ewram[0x1f6e] = c.index as u8;
        m.cpu.regs[0] = 0x02001f6e;
        m.cpu.regs[1] = 0x08000000 + c.address;
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
        assert!(n < 100000);
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
        let mut loads = Vec::new();
        if width != 0 {
            loads.push((
                m.cpu.regs[usize::from((op >> 3) & 7)] + u32::from((op >> 6) & 31) * width,
                width,
            ));
        }
        if op == 0xdf0b || op == 0xdf0c {
            let control = m.cpu.regs[2];
            let width = if op == 0xdf0c || control & (1 << 26) != 0 {
                4
            } else {
                2
            };
            let count = if op == 0xdf0c {
                (control & 0x1fffff).div_ceil(8) * 8
            } else {
                control & 0x1fffff
            };
            assert_eq!(control & (1 << 24), 0);
            let src = m.cpu.regs[0] & !(width - 1);
            for i in 0..count {
                loads.push((src + i * width, width));
            }
        }
        if op & 0xf800 == 0x6000 {
            let dest = m.cpu.regs[usize::from((op >> 3) & 7)] + u32::from((op >> 6) & 31) * 4;
            let control = m.cpu.regs[usize::from(op & 7)];
            if dest == 0x040000dc && control & 0x80000000 != 0 {
                let width = if control & (1 << 26) != 0 { 4 } else { 2 };
                let count = control & 0xffff;
                let src = m.bus.read32(0x040000d4) & !(width - 1);
                for i in 0..count {
                    loads.push((src + i * width, width));
                }
            }
        }
        for (address, width) in loads {
            if (8..=13).contains(&(address >> 24)) {
                let physical = (address & 0x1ffffff) & !(width - 1);
                for at in physical..physical + width {
                    observed[0] += u64::from(owned(at));
                    observed[1] += u64::from(prior_owned(at));
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
        entry: 0xd34aa,
        address: 0,
        mirror: 0x08000000,
        kind: 0,
        end: 0xd34ac,
        index: 0,
    };
    // Seven representative views, mirrors, sparse/raw and prior/new boundaries.
    for (i, source) in [
        0x43ce28, 0x431470, 0x439190, 0x4386c0, 0x43cf4c, 0x4408f4, 0x441488,
    ]
    .into_iter()
    .enumerate()
    {
        calls.push(Call {
            address: source,
            mirror: [0x08000000, 0x0a000000, 0x0c000000][i % 3],
            ..blank
        });
    }
    for (entry, address) in [
        (0x4a13c, 0x71e66b),
        (0xd08a2, 0x71e6a7),
        (0xd34aa, 0x441487),
        (0xd08a2, 0x441606),
        (0xd34aa, 0x71e668),
    ] {
        calls.push(Call {
            entry,
            address,
            end: entry + 2,
            ..blank
        });
    }
    calls.push(Call {
        kind: 1,
        entry: 0x145230,
        address: 0x439190,
        ..blank
    });
    calls.push(Call {
        kind: 3,
        entry: 0x800,
        address: 0x441488,
        ..blank
    });
    for (address, index) in [
        (0x71e5f4, 4),
        (0x71e608, 4),
        (0x71e66c, 4),
        (0x71e694, 4),
        (0x71e6a8, 4),
        (0x71e6e4, 4),
        (0x71e6f8, 4),
        (0x71e6f8, 5),
        (0x71e6f8, 255),
    ] {
        calls.push(Call {
            kind: 2,
            entry: 0x23ab0,
            end: 0x23ab8,
            address,
            index,
            ..blank
        });
    }
    assert_eq!(calls.len(), 23);
    let mut raw = crate::make_machine(rom.to_vec(), None);
    let initial = raw.save_state().unwrap();
    let mut visited: [_; 4] = std::array::from_fn(|_| BTreeSet::new());
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
        assert!(
            calls
                .iter()
                .filter(|c| family(**c) == group)
                .all(|&c| !visited.iter().any(|&(pc, _)| pc == stop(c)))
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
        let object = root().join(format!("raw-transfers-family{group}.o"));
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
        write32,
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
            assert!(dispatch < 200000);
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
    assert_eq!(libraries.len(), 4);
    serde_json::json!({"full_state_clock_cases":calls.len(),"single_load_cases":12,"original_CpuSet_cases":1,"original_queue_executor_cases":1,"raw_selector_prefix_cases":9,"compiled_new_source_bytes":observed[0],"compiled_prior_indexed_bytes":observed[1],"fallback_source_bytes":FALLBACK.with(Cell::get),"compiled_instructions":compiled_instructions,"native_case_families":4,"each_family_excludes_stop_boundaries":true,"dispatch_coverage_only":dispatch,"all38220_raw_source_and_prior4_poisoned_after_Load_setup":true,"observations":observations})
}
fn bus_lifecycle(rom: &[u8]) -> serde_json::Value {
    let mut m = crate::make_machine(rom.to_vec(), None);
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    let initial = m.save_state().unwrap();
    poison(&mut m);
    let ranges = spans();
    for &(a, b) in ranges {
        for at in a..b {
            assert_eq!(m.bus.read8(0x0c000000 + at), rom[at as usize]);
        }
    }
    let s = owner.ui_raw_transfers_stats().unwrap();
    assert_eq!(
        [
            s.bus_reads,
            s.source_bytes,
            s.selector_bytes,
            s.payload_bytes,
            s.banks_used,
            s.groups_used
        ],
        [38220, 38220, 140, 38080, 35, 7]
    );
    let pointer = &mut m as *mut Machine as *mut std::ffi::c_void;
    for &(a, b) in ranges {
        for edge in [a, b] {
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
    }
    poison_prior(&mut m);
    assert_eq!(
        (RT_API.read32)(pointer, 0x0871e66b),
        u32::from_le_bytes(rom[0x71e66b..0x71e66f].try_into().unwrap())
    );
    assert!(owner.indexed_graphics_stats().unwrap().source_bytes > 0);
    let before = owner.ui_raw_transfers_stats().unwrap();
    for at in [0x441288, 0x441487] {
        m.bus.rom[at] = 0xef;
        assert_eq!(m.bus.read8(0x08000000 + at as u32), 0xef);
    }
    assert_eq!(owner.ui_raw_transfers_stats().unwrap(), before);
    let prior_before = owner.indexed_graphics_stats().unwrap().source_bytes;
    for at in [0x441608, 0x441688] {
        m.bus.rom[at] = 0xef;
        assert_eq!(m.bus.read8(0x08000000 + at as u32), rom[at]);
    }
    assert_eq!(owner.indexed_graphics_stats().unwrap().source_bytes, prior_before + 2);
    assert_eq!(owner.ui_raw_transfers_stats().unwrap(), before);
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
    assert_eq!(m.bus.read8(0x08441488), rom[0x441488]);
    m.bus.rom.copy_from_slice(rom);
    fs::rename(&unavailable, dataset()).unwrap();
    for kind in 0..4 {
        let mut a = crate::make_machine(rom.to_vec(), None);
        let mut b = crate::make_machine(rom.to_vec(), None);
        attach_from_directory(&mut b, &dataset()).unwrap();
        for m in [&mut a, &mut b] {
            if kind < 2 {
                m.bus.write32(0x040000d4, 0x0871e668);
                m.bus.write32(0x040000d8, 0x02010000);
                m.bus.write16(0x040000dc, if kind == 0 { 32 } else { 16 });
            } else {
                m.cpu.regs[..3].copy_from_slice(&[
                    0x0a71e668,
                    0x02010000,
                    if kind == 3 { 16 } else { 32 },
                ]);
                m.cpu.regs[15] = 0x02000000;
                m.cpu.cpsr = 0xbf;
                m.bus
                    .write16(0x02000000, if kind == 3 { 0xdf0c } else { 0xdf0b });
            }
        }
        poison(&mut b);
        poison_prior(&mut b);
        for m in [&mut a, &mut b] {
            if kind < 2 {
                m.bus
                    .write16(0x040000de, if kind == 0 { 0x8000 } else { 0x8400 });
            } else {
                m.step();
            }
        }
        b.bus.rom.copy_from_slice(rom);
        assert_eq!(
            b.save_state().unwrap(),
            a.save_state().unwrap(),
            "rawtransfer mixed{kind} fullstate-clock"
        );
        let owner = b.bus.rom_resources().unwrap();
        assert!(owner.ui_raw_transfers_stats().unwrap().source_bytes >= 40);
        assert!(owner.indexed_graphics_stats().unwrap().source_bytes >= 4);
    }
    serde_json::json!({"all38220_source_bytes":true,"all35views7groups140selector38080payload":true,"all42spanedges_mirrors_widths_priorindexed_adjacenttyped_and_rawsparseholes":true,"unavailable_files_Load_slot_Undo_owner_retained":true,"DMA16_DMA32_CpuSet_CpuFastSet_fullstate_clock":4,"transfer_source_bytes":64})
}
#[test]
#[ignore = "private EUR ui_raw_transfers managed/native integration; Wine only, Linux/USA pending"]
fn european_ui_raw_transfers_managed_native_and_lifecycle() {
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
        (92, 91, 83)
    );
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision,
            source.manifest.resources.len()
        ),
        (93, 92, 84)
    );
    extract(&dataset(), &mut source).unwrap();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 85);
    assert_eq!(&source.manifest.resources[..83], prior.resources.as_slice());
    assert_eq!(&source.manifest.coverage[..83], prior.coverage.as_slice());
    for entry in &prior.resources {
        assert_eq!(
            fs::read(dataset().join(&entry.path)).unwrap(),
            fs::read(old.join(&entry.path)).unwrap()
        );
    }
    let entry = &source.manifest.resources[83];
    assert_eq!(entry.id, UI_RAW_TRANSFERS_ID);
    assert!(entry.dependencies.is_empty());
    assert_eq!(entry.sources.len(), 42);
    assert_eq!(entry.sources.iter().map(|s| s.length).sum::<u32>(), 38220);
    let bytes = fs::read(dataset().join(UI_RAW_TRANSFERS_FILE)).unwrap();
    let manifest = fs::read(dataset().join(MANIFEST_FILE)).unwrap();
    assert_eq!(
        parse_ui_raw_transfers(&bytes).unwrap(),
        source.ui_raw_transfers.as_ref().unwrap().decoded
    );
    let interrupted = root.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    let mut reached = false;
    assert!(
        write_dataset(&interrupted, &mut source, &mut |p, b| {
            if p.file_name().unwrap() == UI_RAW_TRANSFERS_FILE {
                reached = true;
                Err("injected ui_raw_transfers file failure".into())
            } else {
                write_synced(p, b)
            }
        })
        .is_err()
    );
    assert!(reached);
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    assert!(verify(&interrupted, &source).is_err());
    eprintln!("ROM_DATA_UI_RAW_TRANSFERS_STAGE publication85/prior83exact/interruption PASS");
    let mut m = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    let state = m.save_state().unwrap();
    for case in 0..14 {
        let mut bad = source.manifest.clone();
        let mut data = bytes.clone();
        match case {
            0..=4 => {
                let mut v = parse_ui_raw_transfers(&data).unwrap();
                match case {
                    0 => v.banks[0].words[0] ^= 1,
                    1 => v.banks[0].relative_offset += 4,
                    2 => v.tables[0].pointers[0].bank_offset += 4,
                    3 => {
                        v.banks.pop();
                    }
                    _ => v.tables[0].source_offset += 1,
                };
                data = encode_ui_raw_transfers(&v).unwrap();
            }
            5 => bad.resources[83].sources[0].offset += 1,
            6 => bad.resources[83].sources[0].length -= 1,
            7 => bad.resources[83].sources[0].sha256 = "00".repeat(32),
            8 => bad.resources[83].sources[0].role = "other".into(),
            9 => {
                let mut v: serde_json::Value = serde_json::from_slice(&data).unwrap();
                v["unexpected"] = true.into();
                data = serde_json::to_vec(&v).unwrap();
            }
            10 => bad.resources[83]
                .dependencies
                .push(ENCOUNTER_HEADERS_ID.into()),
            11 => bad.resources[83]
                .sources
                .push(gba_core::rom_data::SourceRange {
                    offset: 0x71e57a,
                    length: 2,
                    sha256: gba_core::ffta_rom::digest(&rom[0x71e57a..0x71e57c]),
                    role: "counterfeit-shared".into(),
                }),
            12 => bad.resources[83].format = "counterfeit".into(),
            _ => {
                bad.coverage.pop();
            }
        }
        bad.resources[83].sha256 = gba_core::ffta_rom::digest(&data);
        fs::write(dataset().join(UI_RAW_TRANSFERS_FILE), data).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&bad).unwrap(),
        )
        .unwrap();
        assert!(
            attach_from_directory(&mut m, &dataset()).is_err(),
            "ui_raw_transfers attach guard{case}"
        );
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), state);
    }
    fs::write(dataset().join(UI_RAW_TRANSFERS_FILE), &bytes).unwrap();
    fs::write(dataset().join(MANIFEST_FILE), &manifest).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let saved = source.ui_raw_transfers.take();
    assert!(source.validate_files(files.files()).is_err());
    source.ui_raw_transfers = saved;
    let mut missing = files.files();
    missing.ui_raw_transfers = None;
    assert!(source.validate_files(missing).is_err());
    eprintln!("ROM_DATA_UI_RAW_TRANSFERS_STAGE attach14/fullset2/failedownerfullstate PASS");
    let native = native_prefixes(&rom);
    eprintln!("ROM_DATA_UI_RAW_TRANSFERS_STAGE native23/fullstate-clock/source38220poisoned PASS");
    let bus = bus_lifecycle(&rom);
    let result = serde_json::json!({"status":"PASS","revision":[1,93,92],"files":85,"resources":84,"prior83_records_files_exact":true,"source_bytes":38220,"attach_guards":14,"fullset_guards":2,"newfile_interruption":true,"native":native,"bus_lifecycle":bus,"dataset_sha256":owner.dataset_sha256(),"manifest_sha256":gba_core::ffta_rom::digest(&manifest),"ui_raw_transfers_JSON_sha256":gba_core::ffta_rom::digest(&bytes),"actualCLI":"pending","ordinary_positive":"pending","Linux":"pending","USA":"pending","nativeWindows":"pending","new32_9":"pending","performance_measurements":false});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    eprintln!(
        "ROM_DATA_UI_RAW_TRANSFERS_RUNTIME e93-d92/files85/resources84/attach14/fullset2/native23/DMA4 PASS"
    );
}
