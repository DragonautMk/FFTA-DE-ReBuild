//! Focused Wine indexed graphics integration; controlled probes, no ordinary fixture forcing.
use super::*;
use gba_core::{Bus, Machine, capi::RT_API};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_INDEXED_GRAPHICS_RUNTIME_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
thread_local! {static FALLBACK:Cell<[u64;2]>=const{Cell::new([0;2])};static COMPILED:Cell<[u64;2]>=const{Cell::new([0;2])};}
fn counts(m: *mut std::ffi::c_void) -> [u64; 2] {
    unsafe {
        let r = (&*(m as *const Machine)).bus.rom_resources().unwrap();
        [
            r.indexed_graphics_stats().unwrap().source_bytes,
            r.world_records38_stats().unwrap().source_bytes,
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
    (0x71e0e0..0x71e57a).contains(&at)
}
fn poison(m: &mut Machine) {
    for &(a, b) in spans() {
        m.bus.rom[a as usize..b as usize].fill(0xcd);
    }
}
fn poison_prior(m: &mut Machine) {
    m.bus.rom[0x71e0e0..0x71e57a].fill(0xab);
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
    if c.kind == 1 {
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
        m.cpu.regs[..4].copy_from_slice(&[0x08000000 + c.address, 0x02000100, 0, 1]);
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
        entry: 0xd34aa,
        address: 0,
        mirror: 0x08000000,
        kind: 0,
        end: 0xd34ac,
        index: 0,
    };
    for (i, source) in gba_core::rom_data::indexed_graphics::ARCHIVES
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
        (0x4a13c, 0x71e579),
        (0xd08a2, 0x71e579),
        (0xd34aa, 0x71e57b),
        (0xd34aa, 0x71e57c),
        (0xd08a2, 0x71e58f),
        (0x4a13c, 0x430247),
        (0xd08a2, 0x430247),
        (0xd34aa, 0x71e57a),
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
        entry: 0x537c,
        end: 0,
        address: 0x430248,
        ..blank
    });
    for (address, index) in [
        (0x71e57c, 4),
        (0x71e61c, 4),
        (0x71e630, 4),
        (0x71e658, 4),
        (0x71e57c, 5),
        (0x71e57c, 255),
    ] {
        calls.push(Call {
            kind: 2,
            entry: 0x196a0,
            end: 0x196a8,
            address,
            index,
            ..blank
        });
    }
    assert_eq!(calls.len(), 35);
    let mut raw = crate::make_machine(rom.to_vec(), None);
    let initial = raw.save_state().unwrap();
    let mut visited: [_; 3] = std::array::from_fn(|_| BTreeSet::new());
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
        let object = root().join(format!("indexed-graphics-family{group}.o"));
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
    assert_eq!(libraries.len(), 3);
    serde_json::json!({"full_state_clock_cases":calls.len(),"single_load_cases":28,"original_oneprogram_wrapper_cases":1,"raw_selector_prefix_cases":6,
        "compiled_new_source_bytes":observed[0],"compiled_prior_world_bytes":observed[1],"fallback_source_bytes":FALLBACK.with(Cell::get),"compiled_instructions":compiled_instructions,"native_case_families":3,"each_family_excludes_stop_boundaries":true,"dispatch_coverage_only":dispatch,"all74907_raw_source_and_prior1178_poisoned_after_Load_setup":true,"observations":observations})
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
    let expected: Vec<u64> = serde_json::from_slice(
        &fs::read(root().parent().unwrap().join("expected-field-counts.json")).unwrap(),
    )
    .unwrap();
    let s = owner.indexed_graphics_stats().unwrap();
    assert_eq!(
        [
            s.bus_reads,
            s.source_bytes,
            s.archives_used,
            s.programs_used,
            s.groups_used
        ],
        [74907, 74907, 20, 649, 4]
    );
    assert_eq!(
        vec![
            s.selector_bytes,
            s.archive_header_bytes,
            s.index_bytes,
            s.token_bytes,
            s.literal_bytes,
            s.dictionary_header_bytes,
            s.dictionary_bytes
        ],
        expected
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
    m.bus.rom[0x71e57a..0x71e57c].fill(0xef);
    let expected = std::array::from_fn(|i| {
        let at = 0x71e579 + i;
        if (0x71e57a..0x71e57c).contains(&at) {
            0xef
        } else {
            rom[at]
        }
    });
    assert_eq!(
        (RT_API.read32)(pointer, 0x0871e579),
        u32::from_le_bytes(expected)
    );
    let before = owner.indexed_graphics_stats().unwrap();
    for at in [0x71e57a, 0x71e57b] {
        assert_eq!(m.bus.read8(0x08000000 + at), 0xef);
    }
    assert_eq!(owner.indexed_graphics_stats().unwrap(), before);
    assert!(owner.world_records38_stats().unwrap().source_bytes > 0);
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
    assert_eq!(m.bus.read8(0x08430248), rom[0x430248]);
    m.bus.rom.copy_from_slice(rom);
    fs::rename(&unavailable, dataset()).unwrap();
    for kind in 0..3 {
        let mut a = crate::make_machine(rom.to_vec(), None);
        let mut b = crate::make_machine(rom.to_vec(), None);
        attach_from_directory(&mut b, &dataset()).unwrap();
        for m in [&mut a, &mut b] {
            if kind == 0 {
                m.bus.write32(0x040000d4, 0x0871e560);
                m.bus.write32(0x040000d8, 0x02010000);
                m.bus.write16(0x040000dc, 32);
            } else {
                m.cpu.regs[..3].copy_from_slice(&[
                    0x0a71e560,
                    0x02010000,
                    if kind == 2 { 16 } else { 32 },
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
            "indexed mixedtransfer{kind} fullstate-clock"
        );
        let owner = b.bus.rom_resources().unwrap();
        assert!(owner.indexed_graphics_stats().unwrap().source_bytes >= 20);
        assert!(owner.world_records38_stats().unwrap().source_bytes >= 26);
    }
    serde_json::json!({"all74907_source_bytes":true,"all7classes20archives649programs4groups":true,"all24spanedges_mirrors_widths_prior_world_and_rawgap":true,"unavailable_files_Load_slot_Undo_owner_retained":true,"DMA_CpuSet_CpuFastSet_fullstate_clock":3,"DMA_CpuSet_halfwords":32,"CpuFastSet_words":16})
}
#[test]
#[ignore = "private EUR indexed_graphics managed/native integration; Wine only, Linux/USA pending"]
fn european_indexed_graphics_managed_native_and_lifecycle() {
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
        (90, 89, 82)
    );
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision,
            source.manifest.resources.len()
        ),
        (91, 90, 83)
    );
    extract(&dataset(), &mut source).unwrap();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 84);
    assert_eq!(&source.manifest.resources[..82], prior.resources.as_slice());
    assert_eq!(&source.manifest.coverage[..82], prior.coverage.as_slice());
    for entry in &prior.resources {
        assert_eq!(
            fs::read(dataset().join(&entry.path)).unwrap(),
            fs::read(old.join(&entry.path)).unwrap()
        );
    }
    let entry = &source.manifest.resources[82];
    assert_eq!(entry.id, INDEXED_GRAPHICS_ID);
    assert!(entry.dependencies.is_empty());
    assert_eq!(entry.sources.len(), 24);
    assert_eq!(entry.sources.iter().map(|s| s.length).sum::<u32>(), 74907);
    let bytes = fs::read(dataset().join(INDEXED_GRAPHICS_FILE)).unwrap();
    let manifest = fs::read(dataset().join(MANIFEST_FILE)).unwrap();
    assert_eq!(
        parse_indexed_graphics(&bytes).unwrap(),
        source.indexed_graphics.as_ref().unwrap().decoded
    );
    let interrupted = root.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    let mut reached = false;
    assert!(
        write_dataset(&interrupted, &mut source, &mut |p, b| {
            if p.file_name().unwrap() == INDEXED_GRAPHICS_FILE {
                reached = true;
                Err("injected indexed_graphics file failure".into())
            } else {
                write_synced(p, b)
            }
        })
        .is_err()
    );
    assert!(reached);
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    assert!(verify(&interrupted, &source).is_err());
    eprintln!("ROM_DATA_INDEXED_GRAPHICS_STAGE publication84/prior82exact/interruption PASS");
    let mut m = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    let state = m.save_state().unwrap();
    for case in 0..14 {
        let mut bad = source.manifest.clone();
        let mut data = bytes.clone();
        match case {
            0..=4 => {
                let mut v = parse_indexed_graphics(&data).unwrap();
                match case {
                    0 => v.archives[0].opaque_header[0] ^= 1,
                    1 => v.archives[0].programs[0].index ^= 1,
                    2 => v.archives[0].programs[0].decoded_words[0] ^= 0x80,
                    3 => {
                        v.archives.pop();
                    }
                    _ => v.groups[0].source_offset += 1,
                };
                data = encode_indexed_graphics(&v).unwrap();
            }
            5 => bad.resources[82].sources[0].offset += 1,
            6 => bad.resources[82].sources[0].length -= 1,
            7 => bad.resources[82].sources[0].sha256 = "00".repeat(32),
            8 => bad.resources[82].sources[0].role = "other".into(),
            9 => {
                let mut v: serde_json::Value = serde_json::from_slice(&data).unwrap();
                v["unexpected"] = true.into();
                data = serde_json::to_vec(&v).unwrap();
            }
            10 => bad.resources[82]
                .dependencies
                .push(ENCOUNTER_HEADERS_ID.into()),
            11 => bad.resources[82]
                .sources
                .push(gba_core::rom_data::SourceRange {
                    offset: 0x71e57a,
                    length: 2,
                    sha256: gba_core::ffta_rom::digest(&rom[0x71e57a..0x71e57c]),
                    role: "counterfeit-shared".into(),
                }),
            12 => bad.resources[82].format = "counterfeit".into(),
            _ => {
                bad.coverage.pop();
            }
        }
        bad.resources[82].sha256 = gba_core::ffta_rom::digest(&data);
        fs::write(dataset().join(INDEXED_GRAPHICS_FILE), data).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&bad).unwrap(),
        )
        .unwrap();
        assert!(
            attach_from_directory(&mut m, &dataset()).is_err(),
            "indexed_graphics attach guard{case}"
        );
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), state);
    }
    fs::write(dataset().join(INDEXED_GRAPHICS_FILE), &bytes).unwrap();
    fs::write(dataset().join(MANIFEST_FILE), &manifest).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let saved = source.indexed_graphics.take();
    assert!(source.validate_files(files.files()).is_err());
    source.indexed_graphics = saved;
    let mut missing = files.files();
    missing.indexed_graphics = None;
    assert!(source.validate_files(missing).is_err());
    eprintln!("ROM_DATA_INDEXED_GRAPHICS_STAGE attach14/fullset2/failedownerfullstate PASS");
    let native = native_prefixes(&rom);
    eprintln!("ROM_DATA_INDEXED_GRAPHICS_STAGE native35/fullstate-clock/source74907poisoned PASS");
    let bus = bus_lifecycle(&rom);
    let result = serde_json::json!({"status":"PASS","revision":[1,91,90],"files":84,"resources":83,"prior82_records_files_exact":true,"source_bytes":74907,"attach_guards":14,"fullset_guards":2,"newfile_interruption":true,"native":native,"bus_lifecycle":bus,"dataset_sha256":owner.dataset_sha256(),"manifest_sha256":gba_core::ffta_rom::digest(&manifest),"indexed_graphics_JSON_sha256":gba_core::ffta_rom::digest(&bytes),"actualCLI":"pending","ordinary_positive":"pending","Linux":"pending","USA":"pending","nativeWindows":"pending","new32_9":"pending","performance_measurements":false});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    eprintln!(
        "ROM_DATA_INDEXED_GRAPHICS_RUNTIME e91-d90/files84/resources83/attach14/fullset2/native35/DMA3 PASS"
    );
}
