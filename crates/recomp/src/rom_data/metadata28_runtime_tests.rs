//! Focused Wine integration for indexed metadata28 and its typed/raw borders.
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::metadata28::{SOURCE, SOURCE_BYTES, SOURCE_END},
    Bus, Machine,
};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_METADATA28_RUNTIME_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
thread_local! {static COMPILED:Cell<[u64;2]>=const{Cell::new([0;2])};}
fn counts(m: *mut std::ffi::c_void) -> [u64; 2] {
    unsafe {
        let r = (&*(m as *const Machine)).bus.rom_resources().unwrap();
        [
            r.metadata28_stats().unwrap().source_bytes,
            r.encounter_headers_stats().unwrap().source_bytes,
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
#[derive(Clone, Copy)]
struct Call {
    entry: u32,
    address: u32,
    mirror: u32,
    getter: Option<(u32, u32)>,
    expected: [u64; 2],
}
fn stop(c: Call) -> u32 {
    if c.getter.is_some() {
        0x08000300
    } else {
        0x08000000 + c.entry + 2
    }
}
fn setup(m: &mut Machine, c: Call) {
    m.bus.ffta_sprites.enabled = false;
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    if let Some((index, selector)) = c.getter {
        m.cpu.regs[0] = index;
        m.cpu.regs[1] = selector;
    } else {
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
        let base = usize::from((op >> 3) & 7);
        m.cpu.regs[base] = c.mirror + c.address - u32::from((op >> 6) & 31) * width;
    }
    m.cpu.regs[13] = 0x03007c00;
    m.cpu.regs[14] = 0x08000301;
    m.cpu.regs[15] = 0x08000000 + c.entry;
    m.cpu.cpsr = 0xbf;
}
fn interpreted(m: &mut Machine, c: Call, visited: &mut BTreeSet<(u32, bool)>) {
    let mut n = 0;
    while m.cpu.regs[15] != stop(c) {
        assert!(n < 128);
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
    for (n, (entry, index, field)) in [
        (0xd0878, 0, 0),
        (0xd087c, 346, 2),
        (0xd0880, 64, 4),
        (0xd0898, 128, 10),
        (0xd08aa, 192, 11),
        (0xd08a2, 255, 16),
        (0xd08a6, 320, 20),
        (0xb957a, 1, 22),
        (0xaafc0, 2, 24),
        (0xc67d0, 3, 25),
        (0x137dde, 346, 26),
        (0xd0880, 0, 3),
        (0xd0880, 346, 15),
        (0xd0880, 64, 27),
        (0xd0880, 128, 12),
    ]
    .into_iter()
    .enumerate()
    {
        let width = match u16::from_le_bytes(rom[entry..entry + 2].try_into().unwrap()) & 0xf800 {
            0x8800 => 2,
            0x6800 => 4,
            _ => 1,
        };
        calls.push(Call {
            entry: entry as u32,
            address: SOURCE + index * 28 + field,
            mirror: [0x08000000, 0x0a000000, 0x0c000000][n % 3],
            getter: None,
            expected: [width, 0],
        });
    }
    calls.push(Call {
        entry: 0xd0880,
        address: SOURCE - 1,
        mirror: 0x08000000,
        getter: None,
        expected: [0, 1],
    });
    calls.push(Call {
        entry: 0xd0880,
        address: SOURCE_END,
        mirror: 0x0a000000,
        getter: None,
        expected: [0, 0],
    });
    for (index, selector, width) in [
        (0, 9, 0),
        (346, 10, 4),
        (0x10001, 31, 4),
        (347, 0, 0),
        (65535, 34, 0),
        (1, 32, 2),
    ] {
        calls.push(Call {
            entry: 0xd07a4,
            address: 0,
            mirror: 0x08000000,
            getter: Some((index, selector)),
            expected: [width, 0],
        });
    }
    assert_eq!(calls.len(), 23);
    let mut raw = crate::make_machine(rom.to_vec(), None);
    let initial = raw.save_state().unwrap();
    let mut visited = [BTreeSet::new(), BTreeSet::new()];
    let mut expected = Vec::new();
    for &c in &calls {
        raw.load_state(&initial).unwrap();
        setup(&mut raw, c);
        interpreted(&mut raw, c, &mut visited[usize::from(c.getter.is_some())]);
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
                .filter(|c| usize::from(c.getter.is_some()) == group)
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
        let object = root().join(format!("metadata28-family{group}.o"));
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
    COMPILED.with(|c| c.set([0; 2]));
    let mut dispatch = 0;
    for (i, &c) in calls.iter().enumerate() {
        for m in [&mut typed, &mut native] {
            m.load_state(&initial).unwrap();
            setup(m, c);
            m.bus.rom[SOURCE as usize..SOURCE as usize + SOURCE_BYTES].fill(0xcd);
            m.bus.rom[SOURCE as usize - 1] = rom[SOURCE as usize - 1] ^ 0xff;
        }
        interpreted(&mut typed, c, &mut BTreeSet::new());
        let before = COMPILED.with(Cell::get);
        while native.cpu.regs[15] != stop(c) {
            assert!(dispatch < 4096);
            native.bus.frame_ready = false;
            tables[usize::from(c.getter.is_some())]
                .get(native.cpu.regs[15] | u32::from(native.cpu.thumb()))
                .unwrap_or_else(|| {
                    panic!(
                        "uncompiled original metadata28 case{i} PC{:08x} thumb{}",
                        native.cpu.regs[15],
                        native.cpu.thumb()
                    )
                })(&api, &mut native as *mut Machine as *mut std::ffi::c_void);
            dispatch += 1;
        }
        let after = COMPILED.with(Cell::get);
        assert_eq!(
            std::array::from_fn::<_, 2, _>(|j| after[j] - before[j]),
            c.expected,
            "compiled owner counts case{i}"
        );
        for (kind, m) in [("interpreted", &mut typed), ("native", &mut native)] {
            m.bus.rom[SOURCE as usize..SOURCE as usize + SOURCE_BYTES]
                .copy_from_slice(&rom[SOURCE as usize..SOURCE as usize + SOURCE_BYTES]);
            m.bus.rom[SOURCE as usize - 1] = rom[SOURCE as usize - 1];
            check_state(m, &expected[i], &format!("{kind}-case{i}"));
        }
    }
    let c = COMPILED.with(Cell::get);
    serde_json::json!({"full_state_clock_cases":calls.len(),"single_load_prefixes":17,"full_original_getter_calls":6,"compiled_new_source_bytes":c[0],"compiled_prior_header_bytes":c[1],"compiled_instructions":compiled_instructions,"native_case_families":2,"each_family_excludes_stop_boundaries":true,"dispatch_coverage_only":dispatch,"all9716_raw_source_bytes_poisoned":true,"fiveclasses_mirrors_prior_rawoutside_indexselector_aliases":true})
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
    let s = owner.metadata28_stats().unwrap();
    assert_eq!(
        [
            s.bus_reads,
            s.source_bytes,
            s.halfword_bytes,
            s.byte_bytes,
            s.inline_bytes,
            s.flag_bytes,
            s.opaque_bytes,
            s.records_used
        ],
        [9716, 9716, 2082, 4164, 1041, 1388, 1041, 347]
    );
    let pointer = &mut m as *mut Machine as *mut std::ffi::c_void;
    for delta in [
        -1,
        1,
        3,
        0x710fff - SOURCE as i32,
        0x711fff - SOURCE as i32,
        0x712fff - SOURCE as i32,
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
    // Header bytes to the left retain the prior owner; the successor on the right is raw.
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
    assert!(owner.encounter_headers_stats().unwrap().source_bytes > 0);
    let before = owner.metadata28_stats().unwrap();
    assert_eq!(m.bus.read8(0x08000000 + SOURCE_END), 0xef);
    assert_eq!(owner.metadata28_stats().unwrap(), before);
    m.bus.rom[end..end + 4].copy_from_slice(&rom[end..end + 4]);
    for index in 0..347 {
        let at = SOURCE + (index + 1) * 28 - 1;
        assert_eq!(
            (RT_API.read32)(pointer, 0x0a000000 + at),
            u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap())
        );
    }
    let before = owner.metadata28_stats().unwrap();
    let old = gba_core::rom_data::task_cycle_deltas::SOURCE as usize;
    m.bus.rom[old] = 0xab;
    assert_eq!(m.bus.read8(0x08000000 + old as u32), rom[old]);
    assert_eq!(owner.metadata28_stats().unwrap(), before);
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
                m.bus.write16(0x040000dc, 4860);
            } else {
                m.cpu.regs[..3].copy_from_slice(&[
                    0x0a000000 + SOURCE,
                    0x02010000,
                    if kind == 2 { 2432 } else { 4860 },
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
                .metadata28_stats()
                .unwrap()
                .source_bytes
                >= 9716
        );
    }
    serde_json::json!({"all9716_new_typed_bytes":true,"all5classes347records":true,"widths_mirrors_all347recordboundaries_prior_header_rawsuccessor":true,"unavailable_files_Load_slot_Undo_owner_retained":true,"DMA_CpuSet_CpuFastSet_fullstate_clock":3,"DMA_CpuSet_halfwords":4860,"CpuFastSet_words":2432})
}
#[test]
#[ignore = "private EUR metadata28 managed/native integration; Wine only, Linux/USA pending"]
fn european_metadata28_managed_native_and_lifecycle() {
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
        (82, 81, 74)
    );
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision,
            source.manifest.resources.len()
        ),
        (83, 82, 75)
    );
    extract(&dataset(), &mut source).unwrap();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 76);
    assert_eq!(&source.manifest.resources[..74], prior.resources.as_slice());
    assert_eq!(&source.manifest.coverage[..74], prior.coverage.as_slice());
    for entry in &prior.resources {
        assert_eq!(
            fs::read(dataset().join(&entry.path)).unwrap(),
            fs::read(old.join(&entry.path)).unwrap()
        );
    }
    let entry = &source.manifest.resources[74];
    assert_eq!(entry.id, METADATA28_ID);
    assert!(entry.dependencies.is_empty());
    assert_eq!(entry.sources.len(), 1);
    assert_eq!(entry.sources[0].offset, SOURCE);
    assert_eq!(entry.sources[0].length, 9716);
    let bytes = fs::read(dataset().join(METADATA28_FILE)).unwrap();
    let manifest = fs::read(dataset().join(MANIFEST_FILE)).unwrap();
    assert_eq!(
        parse_metadata28(&bytes).unwrap(),
        source.metadata28.as_ref().unwrap().decoded
    );
    let interrupted = root.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    let mut reached = false;
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| {
        if p.file_name().unwrap() == METADATA28_FILE {
            reached = true;
            Err("injected metadata28 file failure".into())
        } else {
            write_synced(p, b)
        }
    })
    .is_err());
    assert!(reached);
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    assert!(verify(&interrupted, &source).is_err());
    eprintln!("ROM_DATA_METADATA28_STAGE publication76/prior74exact/interruption PASS");
    let mut m = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    let state = m.save_state().unwrap();
    for case in 0..14 {
        let mut bad = source.manifest.clone();
        let mut data = bytes.clone();
        match case {
            0..=4 => {
                let mut v = parse_metadata28(&data).unwrap();
                match case {
                    0 => v.records[0].field00 ^= 1,
                    1 => v.records[0].index ^= 1,
                    2 => v.records[0].opaque03 = 255,
                    3 => {
                        v.records.pop();
                    }
                    _ => v.source_offset += 1,
                };
                data = encode_metadata28(&v).unwrap();
            }
            5 => bad.resources[74].sources[0].offset += 1,
            6 => bad.resources[74].sources[0].length -= 1,
            7 => bad.resources[74].sources[0].sha256 = "00".repeat(32),
            8 => bad.resources[74].sources[0].role = "other".into(),
            9 => {
                let mut v: serde_json::Value = serde_json::from_slice(&data).unwrap();
                v["unexpected"] = true.into();
                data = serde_json::to_vec(&v).unwrap();
            }
            10 => bad.resources[74]
                .dependencies
                .push(ENCOUNTER_HEADERS_ID.into()),
            11 => bad.resources[74]
                .sources
                .push(gba_core::rom_data::SourceRange {
                    offset: SOURCE_END,
                    length: 2,
                    sha256: gba_core::ffta_rom::digest(
                        &rom[SOURCE_END as usize..SOURCE_END as usize + 2],
                    ),
                    role: "counterfeit-shared".into(),
                }),
            12 => bad.resources[74].format = "counterfeit".into(),
            _ => {
                bad.coverage.pop();
            }
        }
        bad.resources[74].sha256 = gba_core::ffta_rom::digest(&data);
        fs::write(dataset().join(METADATA28_FILE), data).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&bad).unwrap(),
        )
        .unwrap();
        assert!(
            attach_from_directory(&mut m, &dataset()).is_err(),
            "metadata28 attach guard{case}"
        );
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), state);
    }
    fs::write(dataset().join(METADATA28_FILE), &bytes).unwrap();
    fs::write(dataset().join(MANIFEST_FILE), &manifest).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let saved = source.metadata28.take();
    assert!(source.validate_files(files.files()).is_err());
    source.metadata28 = saved;
    let mut missing = files.files();
    missing.metadata28 = None;
    assert!(source.validate_files(missing).is_err());
    eprintln!("ROM_DATA_METADATA28_STAGE attach14/fullset2/failedownerfullstate PASS");
    let native = native_prefixes(&rom);
    eprintln!("ROM_DATA_METADATA28_STAGE native23/fullstate-clock/source9716poisoned PASS");
    let bus = bus_lifecycle(&rom);
    let result = serde_json::json!({"status":"PASS","revision":[1,83,82],"files":76,"resources":75,"prior74_records_files_exact":true,"source_bytes":9716,"attach_guards":14,"fullset_guards":2,"newfile_interruption":true,"native":native,"bus_lifecycle":bus,"dataset_sha256":owner.dataset_sha256(),"manifest_sha256":gba_core::ffta_rom::digest(&manifest),"metadata28_JSON_sha256":gba_core::ffta_rom::digest(&bytes),"actualCLI":"pending","ordinary_positive":"pending","Linux":"pending","USA":"pending","nativeWindows":"pending","new32_9":"pending","performance_measurements":false});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_METADATA28_RUNTIME e83-d82/files76/resources75/attach14/fullset2/native23/DMA3 PASS");
}
