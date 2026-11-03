//! Focused Wine integration for encounter headers and their existing shared owner.
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::{
        constructor_alternates::SOURCE_OFFSET as SHARED,
        encounter_unit_templates::{SOURCE, SOURCE_BYTES},
    },
    Bus, Machine,
};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("FFTA_ENCOUNTER_UNIT_TEMPLATE_RUNTIME_ROOT").unwrap())
}
fn dataset() -> PathBuf {
    root().join("offline dataset")
}
thread_local! { static COMPILED_NEW:Cell<u64>=const{Cell::new(0)}; static COMPILED_SHARED:Cell<u64>=const{Cell::new(0)}; }
fn counts(m: *mut std::ffi::c_void) -> [u64; 2] {
    unsafe {
        let r = (&*(m as *const Machine)).bus.rom_resources().unwrap();
        [
            r.encounter_unit_templates_stats().unwrap().source_bytes,
            r.constructor_alternates_stats().unwrap().source_bytes,
        ]
    }
}
fn account(before: [u64; 2], after: [u64; 2]) {
    COMPILED_NEW.with(|n| n.set(n.get() + after[0] - before[0]));
    COMPILED_SHARED.with(|n| n.set(n.get() + after[1] - before[1]));
}
extern "C" fn read8(m: *mut std::ffi::c_void, at: u32) -> u32 {
    let before = counts(m);
    let value = (RT_API.read8)(m, at);
    account(before, counts(m));
    value
}
extern "C" fn read16(m: *mut std::ffi::c_void, at: u32) -> u32 {
    let before = counts(m);
    let value = (RT_API.read16)(m, at);
    account(before, counts(m));
    value
}
extern "C" fn read32(m: *mut std::ffi::c_void, at: u32) -> u32 {
    let before = counts(m);
    let value = (RT_API.read32)(m, at);
    account(before, counts(m));
    value
}
#[derive(Clone, Copy)]
struct Call {
    entry: u32,
    index: u32,
    base: usize,
    bias: u32,
    mirror: u32,
    bit: Option<u32>,
    new_bytes: u64,
    shared_bytes: u64,
}
fn stop(c: Call) -> u32 {
    if c.bit.is_some() {
        0x080cda80
    } else {
        0x08000000 + c.entry + 2
    }
}
fn setup(m: &mut Machine, c: Call) {
    m.bus.ffta_sprites.enabled = false;
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    let source = c.mirror + SOURCE + c.index * 48;
    if let Some(i) = c.bit {
        m.cpu.regs[8] = source;
        m.cpu.regs[6] = i << 16;
    } else {
        m.cpu.regs[c.base] = source + c.bias;
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
    for (entry, index, base, bias, mirror, bit) in [
        (0xccfca, 0, 2, 0, 0x08000000, None),
        (0xcd064, 2849, 3, 0, 0x0a000000, None),
        (0xcd094, 64, 3, 0, 0x08000000, None),
        (0xcd37e, 229, 1, 0, 0x08000000, None),
        (0xcd3b0, 1376, 2, 0, 0x0c000000, None),
        (0xcd418, 1280, 1, 0, 0x08000000, None),
        (0xcd41e, 1281, 1, 0, 0x08000000, None),
        (0xcd424, 1377, 1, 0, 0x08000000, None),
        (0xcd01a, 224, 4, 8, 0x08000000, None),
        (0xcd01a, 225, 4, 10, 0x08000000, None),
        (0xcd01a, 228, 4, 12, 0x0a000000, None),
        (0xcd01a, 229, 4, 14, 0x08000000, None),
        (0xcd01a, 2849, 4, 16, 0x08000000, None),
        (0xccfe2, 1376, 1, 0, 0x08000000, None),
        (0xcdaae, 0, 1, 40, 0x08000000, None),
        (0xcdab8, 2849, 1, 41, 0x08000000, None),
        (0xccfc4, 225, 2, 0, 0x08000000, None),
        (0xcd458, 0, 1, 44, 0x08000000, None),
        (0xcd0a6, 2849, 3, 45, 0x08000000, None),
        (0xcda5e, 0, 0, 0, 0x08000000, Some(0)),
        (0xcda5e, 1376, 0, 0, 0x0a000000, Some(159)),
        (0xcda5e, 225, 0, 0, 0x08000000, Some(160)),
        (0xcda5e, 2849, 0, 0, 0x08000000, Some(223)),
        (0xcda5e, 2849, 0, 0, 0x08000000, Some(224)),
        (0xcda5e, 229, 0, 0, 0x0c000000, Some(253)),
        (0xcda5e, 224, 0, 0, 0x08000000, Some(224)),
        (0xcda5e, 228, 0, 0, 0x08000000, Some(224)),
    ] {
        let (field, width) = if let Some(i) = bit {
            (20 + (i >> 3), 1)
        } else {
            let op =
                u16::from_le_bytes(rom[entry as usize..entry as usize + 2].try_into().unwrap());
            let w = match op & 0xf800 {
                0x7800 => 1,
                0x8800 => 2,
                _ => panic!("unexpected original load"),
            };
            (bias + u32::from((op >> 6) & 31) * w, w)
        };
        let at = SOURCE + index * 48 + field;
        let mut new_bytes = 0;
        let mut shared_bytes = 0;
        for p in at..at + width {
            if (SHARED..SHARED + 192).contains(&p) {
                shared_bytes += 1;
            } else if (SOURCE..SOURCE + SOURCE_BYTES as u32).contains(&p) {
                new_bytes += 1;
            }
        }
        calls.push(Call {
            entry,
            index,
            base,
            bias,
            mirror,
            bit,
            new_bytes,
            shared_bytes,
        });
    }
    assert_eq!(calls.len(), 27);
    let mut raw = crate::make_machine(rom.to_vec(), None);
    let initial = raw.save_state().unwrap();
    let mut visited = BTreeSet::new();
    let mut expected = Vec::new();
    for &c in &calls {
        raw.load_state(&initial).unwrap();
        setup(&mut raw, c);
        interpreted(&mut raw, c, &mut visited);
        expected.push(raw.save_state().unwrap());
    }
    let view = View {
        rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let blocks = visited
        .iter()
        .map(|&(addr, thumb)| Block {
            addr,
            thumb,
            instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
        })
        .collect();
    let object = root().join("encounter-unit-template.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
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
    COMPILED_NEW.with(|n| n.set(0));
    COMPILED_SHARED.with(|n| n.set(0));
    let mut dispatch = 0;
    for (i, &c) in calls.iter().enumerate() {
        for m in [&mut typed, &mut native] {
            m.load_state(&initial).unwrap();
            setup(m, c);
            m.bus.rom[SOURCE as usize..SOURCE as usize + SOURCE_BYTES].fill(0xcd);
        }
        interpreted(&mut typed, c, &mut BTreeSet::new());
        let before = [
            COMPILED_NEW.with(Cell::get),
            COMPILED_SHARED.with(Cell::get),
        ];
        while native.cpu.regs[15] != stop(c) {
            assert!(dispatch < 4096);
            native.bus.frame_ready = false;
            table
                .get(native.cpu.regs[15] | u32::from(native.cpu.thumb()))
                .expect("original encounter instruction compiled")(
                &api,
                &mut native as *mut Machine as *mut std::ffi::c_void,
            );
            dispatch += 1;
        }
        assert_eq!(
            [
                COMPILED_NEW.with(Cell::get) - before[0],
                COMPILED_SHARED.with(Cell::get) - before[1]
            ],
            [c.new_bytes, c.shared_bytes],
            "compiled owner counts case{i}"
        );
        for (kind, m) in [("interpreted", &mut typed), ("native", &mut native)] {
            m.bus.rom[SOURCE as usize..SOURCE as usize + SOURCE_BYTES]
                .copy_from_slice(&rom[SOURCE as usize..SOURCE as usize + SOURCE_BYTES]);
            check_state(m, &expected[i], &format!("{kind}-case{i}"));
        }
    }
    serde_json::json!({"full_state_clock_prefixes":calls.len(),"compiled_new_source_bytes":COMPILED_NEW.with(Cell::get),"compiled_shared_prior_bytes":COMPILED_SHARED.with(Cell::get),"compiled_instructions":visited.len(),"dispatch_coverage_only":dispatch,"all136800_raw_template_bytes_poisoned":true,"originalLDR_dynamicbits_beyond20B_rawoutside_mirrors_priorowner":true})
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
    let s = owner.encounter_unit_templates_stats().unwrap();
    assert_eq!(
        [
            s.bus_reads,
            s.source_bytes,
            s.selector_bytes,
            s.argument_bytes,
            s.equipment_bytes,
            s.auxiliary_bytes,
            s.ability_bit_bytes,
            s.optional_bytes,
            s.flag_bytes,
            s.control_bytes,
            s.opaque_bytes,
            s.records_used,
            s.headers_used
        ],
        [136608, 136608, 14230, 8538, 28460, 5692, 56920, 5692, 5692, 5692, 5692, 2846, 441]
    );
    assert_eq!(
        owner.constructor_alternates_stats().unwrap().source_bytes,
        192
    );
    assert_eq!(
        owner.constructor_alternates_stats().unwrap().record_mask,
        15
    );
    let pointer = &mut m as *mut Machine as *mut std::ffi::c_void;
    for delta in [
        -1,
        1,
        3,
        (SHARED - SOURCE) as i32 - 1,
        (SHARED - SOURCE) as i32 + 1,
        (SHARED - SOURCE) as i32 + 190,
        (SHARED - SOURCE) as i32 + 191,
        (SHARED - SOURCE) as i32 + 192,
        0x6eafff - SOURCE as i32,
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
    m.bus.rom[SOURCE as usize - 1] ^= 0x55;
    m.bus.rom[end] ^= 0x33;
    assert_eq!(
        (RT_API.read32)(pointer, 0x08000000 + SOURCE - 1),
        u32::from_le_bytes([
            rom[SOURCE as usize - 1] ^ 0x55,
            rom[SOURCE as usize],
            rom[SOURCE as usize + 1],
            rom[SOURCE as usize + 2]
        ])
    );
    assert_eq!(
        (RT_API.read32)(pointer, 0x0c000000 + end as u32 - 1),
        u32::from_le_bytes([rom[end - 1], rom[end] ^ 0x33, rom[end + 1], rom[end + 2]])
    );
    assert!(owner.constructor_alternates_stats().unwrap().source_bytes > 192);
    let before = owner.encounter_unit_templates_stats().unwrap();
    let old = gba_core::rom_data::task_cycle_deltas::SOURCE as usize;
    m.bus.rom[old] = 0xab;
    assert_eq!(m.bus.read8(0x08000000 + old as u32), rom[old]);
    assert_eq!(owner.encounter_unit_templates_stats().unwrap(), before);
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
                m.bus.write16(0x040000dc, 34202);
            } else {
                m.cpu.regs[..3].copy_from_slice(&[
                    0x0a000000 + SOURCE,
                    0x02010000,
                    if kind == 2 { 34208 } else { 68404 },
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
                m.bus.write16(0x040000de, 0x8400);
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
                .encounter_unit_templates_stats()
                .unwrap()
                .source_bytes
                >= 136608
        );
    }
    serde_json::json!({"all136608_new_typed_bytes_and192prior":true,"all9classes2846newrecords441headers":true,"widths_mirrors_mixedraw_edges_prior_owner":true,"unavailable_files_Load_slot_Undo_owner_retained":true,"DMA_CpuSet_CpuFastSet_fullstate_clock":3})
}

#[test]
#[ignore = "private EUR encounter-unit-template managed/native integration; Wine only, Linux/USA pending"]
fn european_encounter_unit_templates_managed_native_and_lifecycle() {
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
        (80, 79, 72)
    );
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision,
            source.manifest.resources.len()
        ),
        (81, 80, 73)
    );
    extract(&dataset(), &mut source).unwrap();
    assert_eq!(fs::read_dir(dataset()).unwrap().count(), 74);
    assert_eq!(&source.manifest.resources[..72], prior.resources.as_slice());
    assert_eq!(&source.manifest.coverage[..72], prior.coverage.as_slice());
    for entry in &prior.resources {
        assert_eq!(
            fs::read(dataset().join(&entry.path)).unwrap(),
            fs::read(old.join(&entry.path)).unwrap()
        );
    }
    let entry = &source.manifest.resources[72];
    assert_eq!(entry.id, ENCOUNTER_UNIT_TEMPLATES_ID);
    assert_eq!(
        entry.dependencies,
        [
            ENCOUNTER_HEADERS_ID,
            CONSTRUCTOR_ALTERNATES_ID,
            ITEM_RECORDS_ID,
            METADATA14_ID,
            METADATA52_ID
        ]
    );
    assert_eq!(entry.sources.len(), 2);
    assert_eq!(entry.sources.iter().map(|s| s.length).sum::<u32>(), 136608);
    assert_eq!(entry.sources.first().unwrap().offset, SOURCE);
    assert_eq!(entry.sources.last().unwrap().offset, SHARED + 192);
    let bytes = fs::read(dataset().join(ENCOUNTER_UNIT_TEMPLATES_FILE)).unwrap();
    let manifest = fs::read(dataset().join(MANIFEST_FILE)).unwrap();
    assert_eq!(
        parse_encounter_unit_templates(&bytes).unwrap(),
        source.encounter_unit_templates.as_ref().unwrap().decoded
    );
    let interrupted = root.join("interrupted");
    fs::create_dir(&interrupted).unwrap();
    let mut reached = false;
    assert!(write_dataset(&interrupted, &mut source, &mut |p, b| {
        if p.file_name().unwrap() == ENCOUNTER_UNIT_TEMPLATES_FILE {
            reached = true;
            Err("injected encounter-unit-template file failure".into())
        } else {
            write_synced(p, b)
        }
    })
    .is_err());
    assert!(reached);
    assert!(!interrupted.join(MANIFEST_FILE).exists());
    assert!(verify(&interrupted, &source).is_err());
    eprintln!(
        "ROM_DATA_ENCOUNTER_UNIT_TEMPLATE_STAGE publication74/prior72exact/interruption PASS"
    );
    let mut m = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut m, &dataset()).unwrap();
    let owner = m.bus.rom_resources().unwrap().clone();
    let state = m.save_state().unwrap();
    for case in 0..14 {
        let mut bad = source.manifest.clone();
        let mut data = bytes.clone();
        match case {
            0..=4 => {
                let mut v = parse_encounter_unit_templates(&data).unwrap();
                match case {
                    0 => v.records[0].unit_index ^= 1,
                    1 => v.records[225].equipment08_10[0] ^= 1,
                    2 => v.records[0].opaque2e_2f[1] = 255,
                    3 => {
                        v.records.pop();
                    }
                    _ => v.source_offset += 1,
                };
                data = encode_encounter_unit_templates(&v).unwrap();
            }
            5 => bad.resources[72].sources[0].offset += 1,
            6 => bad.resources[72].sources[0].length -= 1,
            7 => bad.resources[72].sources[0].sha256 = "00".repeat(32),
            8 => bad.resources[72].sources[0].role = "other".into(),
            9 => {
                let mut v: serde_json::Value = serde_json::from_slice(&data).unwrap();
                v["unexpected"] = true.into();
                data = serde_json::to_vec(&v).unwrap();
            }
            10 => bad.resources[72].dependencies.clear(),
            11 => bad.resources[72]
                .sources
                .push(gba_core::rom_data::SourceRange {
                    offset: SHARED,
                    length: 192,
                    sha256: gba_core::ffta_rom::digest(
                        &rom[SHARED as usize..SHARED as usize + 192],
                    ),
                    role: "counterfeit-shared".into(),
                }),
            12 => bad.resources[72].sources.swap(0, 1),
            _ => {
                bad.coverage.pop();
            }
        }
        bad.resources[72].sha256 = gba_core::ffta_rom::digest(&data);
        fs::write(dataset().join(ENCOUNTER_UNIT_TEMPLATES_FILE), data).unwrap();
        fs::write(
            dataset().join(MANIFEST_FILE),
            toml::to_string_pretty(&bad).unwrap(),
        )
        .unwrap();
        assert!(
            attach_from_directory(&mut m, &dataset()).is_err(),
            "encounter-unit-template attach guard{case}"
        );
        assert!(Arc::ptr_eq(m.bus.rom_resources().unwrap(), &owner));
        assert_eq!(m.save_state().unwrap(), state);
    }
    fs::write(dataset().join(ENCOUNTER_UNIT_TEMPLATES_FILE), &bytes).unwrap();
    fs::write(dataset().join(MANIFEST_FILE), &manifest).unwrap();
    let files = read_dataset(&dataset()).unwrap();
    let saved = source.encounter_unit_templates.take();
    assert!(source.validate_files(files.files()).is_err());
    source.encounter_unit_templates = saved;
    let mut missing = files.files();
    missing.encounter_unit_templates = None;
    assert!(source.validate_files(missing).is_err());
    for missing_owner in 0..5 {
        let mut missing = files.files();
        match missing_owner {
            0 => missing.encounter_headers = None,
            1 => missing.constructor_alternates = None,
            2 => missing.items = None,
            3 => missing.metadata14 = None,
            _ => missing.metadata52 = None,
        }
        assert!(source.validate_files(missing).is_err());
    }
    eprintln!("ROM_DATA_ENCOUNTER_UNIT_TEMPLATE_STAGE attach14/fullset7/failedownerfullstate PASS");
    let native = native_prefixes(&rom);
    eprintln!(
        "ROM_DATA_ENCOUNTER_UNIT_TEMPLATE_STAGE native27/fullstate-clock/source136800poisoned PASS"
    );
    let bus = bus_lifecycle(&rom);
    let result = serde_json::json!({"status":"PASS","revision":[1,81,80],"files":74,"resources":73,"prior72_records_files_exact":true,"source_bytes":136608,"shared_prior_bytes":192,"attach_guards":14,"fullset_guards":7,"newfile_interruption":true,"native":native,"bus_lifecycle":bus,"dataset_sha256":owner.dataset_sha256(),"manifest_sha256":gba_core::ffta_rom::digest(&manifest),"encounter_unit_templates_JSON_sha256":gba_core::ffta_rom::digest(&bytes),"actualCLI":"pending","ordinary_positive":"pending","Linux":"pending","USA":"pending","nativeWindows":"pending","new32_9":"pending","performance_measurements":false});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_ENCOUNTER_UNIT_TEMPLATE_RUNTIME e81-d80/files74/resources73/attach14/fullset7/native27/DMA3 PASS");
}
