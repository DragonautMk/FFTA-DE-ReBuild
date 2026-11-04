//! New table-driven banks and their typed source fields; no old case replay.
use super::*;
use super::compact_obj_catalog_tests::prefix;
use gba_core::{Bus, Machine, capi::RT_API, rom_data::{RuntimeResources, compact_obj_catalog as model}};
use model::selector_tables::{MOTION_OFFSET, POSITION_OFFSET, HORIZONTAL_OFFSET};

fn new_spans() -> Vec<(u32, u32)> {
    let roots = [0x6c58b8, 0x6c6e6c, 0x6cbaf8];
    let mut pieces: Vec<_> = model::BANK_SPECS.iter().filter(|spec| roots.contains(&spec.root))
        .flat_map(|spec| spec.spans.iter().copied()).collect();
    for index in 0..14 { let at = MOTION_OFFSET + index * 8; pieces.extend([(at, at + 3), (at + 4, at + 8)]); }
    for index in 0..10 { let at = POSITION_OFFSET + index * 8; pieces.push((at, at + 6)); }
    pieces.push((HORIZONTAL_OFFSET, HORIZONTAL_OFFSET + 15 * 4));
    pieces.sort_unstable();
    let mut merged: Vec<(u32, u32)> = Vec::new();
    for (a, z) in pieces {
        if let Some(last) = merged.last_mut() {
            assert!(a >= last.1, "overlapping new source fields");
            if a == last.1 { last.1 = z; continue; }
        }
        merged.push((a, z));
    }
    merged
}

#[test]
#[ignore = "private EUR/e112 current/e111 baseline/TinyCC/native count prefix; Wine only, Linux/USA pending"]
fn european_table_compact_catalog_matches_new_fields_and_native_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_ROOT").unwrap());
    assert!(!root.exists()); fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let prior = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_PRIOR").unwrap());
    let catalog = model::Catalog::from_rom(&rom).unwrap(); let data = &catalog.decoded;
    let roots = [0x6c58b8, 0x6c6e6c, 0x6cbaf8];
    let banks: Vec<_> = data.banks.iter().filter(|bank| roots.contains(&bank.root)).collect();
    let fields = data.selector_tables.as_ref().unwrap();
    assert_eq!(banks.len(), 3);
    assert_eq!(banks.iter().map(|bank| bank.sequences.len()).sum::<usize>(), 22);
    assert_eq!(banks.iter().flat_map(|bank| &bank.sequences).map(|entry| entry.sequence.frames.len()).sum::<usize>(), 96);
    assert_eq!(banks.iter().map(|bank| bank.lists.len()).sum::<usize>(), 90);
    assert_eq!(banks.iter().flat_map(|bank| &bank.lists).map(|entry| entry.list.objects.len()).sum::<usize>(), 119);
    let spans = new_spans(); assert_eq!(spans.iter().map(|(a, z)| z - a).sum::<u32>(), 1584);
    let encoded = encode_compact_obj_catalog(data).unwrap(); assert_eq!(parse_compact_obj_catalog(&encoded).unwrap(), *data);
    let mut missing = data.clone(); missing.selector_tables = None; assert!(missing.validate().is_err());
    let mut shifted = data.clone(); shifted.selector_tables.as_mut().unwrap().positions.source_offset += 2;
    assert!(shifted.validate().is_err());
    let old_manifest: Manifest = toml::from_str(&fs::read_to_string(prior.join(MANIFEST_FILE)).unwrap()).unwrap();
    let current = read_dataset(&dataset).unwrap();
    assert_eq!((old_manifest.extractor_revision, old_manifest.decoder_revision, old_manifest.resources.len()), (111, 110, 100));
    assert_eq!((current.manifest.extractor_revision, current.manifest.decoder_revision, current.manifest.resources.len()), (127, 126, 100));
    assert_eq!(current.manifest.resources[..99], old_manifest.resources[..99]);
    assert_eq!(current.manifest.resources[99].format, "compact-obj-catalog-v11-json");
    assert_eq!(old_manifest.resources[99].format, "compact-obj-catalog-v1-json");
    let old: model::CompactObjCatalog = serde_json::from_slice(&fs::read(prior.join(COMPACT_OBJ_CATALOG_FILE)).unwrap()).unwrap();
    assert_eq!(old.banks.len(), 34); assert!(old.selector_tables.is_none());
    for bank in &old.banks { assert_eq!(data.banks.iter().find(|new| new.root == bank.root), Some(bank)); }
    assert_eq!(current.compact_obj_catalog.as_ref().unwrap(), data);

    let mut machine = crate::make_machine(rom.clone(), None); attach_from_directory(&mut machine, &dataset).unwrap();
    let initial = machine.save_state().unwrap(); let owner = machine.bus.rom_resources().unwrap().clone();
    for &(a, z) in &spans { machine.bus.rom[a as usize..z as usize].fill(0xcd); }
    let before = owner.compact_obj_catalog_stats().unwrap().source_bytes;
    for &(a, z) in &spans { for at in a..z {
        assert_eq!(data.byte_at(at), Some(rom[at as usize]));
        assert_eq!(machine.bus.read8(0x0c000000 + at), rom[at as usize]);
    } }
    assert_eq!(owner.compact_obj_catalog_stats().unwrap().source_bytes - before, 1584);
    let unknown = (0..14).map(|i| MOTION_OFFSET + i * 8 + 3)
        .chain((0..10).flat_map(|i| [POSITION_OFFSET + i * 8 + 6, POSITION_OFFSET + i * 8 + 7]));
    let before = owner.compact_obj_catalog_stats().unwrap().source_bytes;
    for at in unknown {
        assert_eq!(data.byte_at(at), None);
        assert_eq!(machine.bus.read8(0x08000000 + at), rom[at as usize]);
    }
    assert_eq!(owner.compact_obj_catalog_stats().unwrap().source_bytes, before);
    for &(a, z) in &spans { for edge in [a, z] { for delta in [-3i32, -1, 0, 1] {
        let at = edge.wrapping_add_signed(delta) as usize; let ptr = &mut machine as *mut Machine as *mut std::ffi::c_void;
        assert_eq!((RT_API.read16)(ptr, 0x0a000000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
        assert_eq!((RT_API.read32)(ptr, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
    } } }
    machine.bus.rom.copy_from_slice(&rom); assert_eq!(machine.save_state().unwrap(), initial);
    let mut bad = read_dataset(&dataset).unwrap();
    bad.compact_obj_catalog.as_mut().unwrap().selector_tables.as_mut().unwrap().motion.rows[0].target_x ^= 1;
    let bytes = encode_compact_obj_catalog(bad.compact_obj_catalog.as_ref().unwrap()).unwrap();
    bad.manifest.resources[99].sha256 = gba_core::ffta_rom::digest(&bytes); bad.compact_obj_catalog_bytes = Some(bytes);
    assert!(RuntimeResources::from_base_files(&rom, bad.files()).is_err());
    assert!(std::sync::Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
    assert_eq!(machine.save_state().unwrap(), initial); drop(bad); drop(current); drop(machine);

    let mut raw = crate::make_machine(rom.clone(), None); raw.bus.ffta_sprites.enabled = false;
    let state = 0x02007f00;
    for bank in &banks {
        raw.bus.write32(state + 4, 0x08000000 + bank.list_root()); raw.bus.write32(state + 8, 0x08000000 + bank.sequence_root());
        for entry in &bank.sequences {
            let sequence = &entry.sequence;
            raw.cpu.regs = [0; 16]; raw.cpu.regs[3] = u32::from(sequence.selector); raw.cpu.regs[4] = state;
            prefix(&mut raw, 0x08010870, 0x08010884); assert_eq!(raw.cpu.regs[3] as usize, sequence.frames.len());
            for (phase, frame) in sequence.frames.iter().enumerate() {
                raw.cpu.regs = [0; 16]; raw.cpu.regs[5] = u32::from(sequence.selector); raw.cpu.regs[3] = state; raw.cpu.regs[2] = phase as u32;
                prefix(&mut raw, 0x08010e5a, 0x08010e70); assert_eq!(raw.cpu.regs[0], u32::from(frame.list));
                raw.cpu.regs = [0; 16]; raw.cpu.regs[7] = u32::from(sequence.selector); raw.cpu.regs[4] = state; raw.cpu.regs[3] = phase as u32;
                prefix(&mut raw, 0x08010cb2, 0x08010cc8); assert_eq!(raw.cpu.regs[1], u32::from(frame.duration));
            }
        }
        for entry in &bank.lists {
            raw.cpu.regs = [0; 16]; raw.cpu.regs[2] = u32::from(entry.index); raw.cpu.regs[3] = state;
            prefix(&mut raw, 0x08010e9a, 0x08010eb2); assert_eq!(raw.cpu.regs[1], 0x08000000 + entry.list.source_offset);
        }
    }
    let object = PathBuf::from(std::env::var_os("FFTA_COMPACT_CATALOG_NATIVE").unwrap());
    let (_count_library, count_table) = crate::load_native(&object).unwrap();
    let mut native = crate::make_machine(rom.clone(), None); attach_from_directory(&mut native, &dataset).unwrap();
    let owner = native.bus.rom_resources().unwrap().clone(); let initial = raw.save_state().unwrap();
    let mut count_bytes = 0; let mut count_cases = 0;
    for bank in &banks { for selector in bank.sequences.iter().map(|entry| entry.sequence.selector).chain([0x100]) {
        for m in [&mut raw, &mut native] {
            m.load_state(&initial).unwrap(); m.bus.ffta_sprites.enabled = false; m.bus.write32(state + 8, 0x08000000 + bank.sequence_root());
            m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32); m.cpu.regs[3] = u32::from(selector); m.cpu.regs[4] = state;
            m.cpu.regs[13] = 0x03007c00; m.cpu.regs[15] = 0x08010870; m.cpu.cpsr = 0xbf;
        }
        for &(a, z) in &spans { native.bus.rom[a as usize..z as usize].fill(0xcd); }
        let directory = bank.sequence_root() + u32::from(selector) * 2;
        let relative = u16::from_le_bytes(rom[directory as usize..directory as usize + 2].try_into().unwrap());
        let count_at = bank.sequence_root() + u32::from(relative);
        let expected: u64 = [directory, count_at].into_iter().flat_map(|at| at..at + 2).map(|at| u64::from(data.byte_at(at).is_some())).sum();
        prefix(&mut raw, 0x08010870, 0x08010884); let before = owner.compact_obj_catalog_stats().unwrap().source_bytes;
        let mut steps = 0;
        while native.cpu.regs[15] != 0x08010884 {
            assert!(steps < 32); native.bus.frame_ready = false;
            count_table.get(native.cpu.regs[15] | 1).unwrap()(&RT_API, &mut native as *mut Machine as *mut std::ffi::c_void); steps += 1;
        }
        let used = owner.compact_obj_catalog_stats().unwrap().source_bytes - before; assert_eq!(used, expected);
        if selector != 0x100 { assert_eq!(used, 4); }
        count_bytes += used; count_cases += 1; native.bus.rom.copy_from_slice(&rom);
        assert_eq!(native.save_state().unwrap(), raw.save_state().unwrap());
    } }
    assert_eq!(count_cases, 25);

    // Only new source-reading prefixes are compiled. They stop before the
    // constructor or interpolation calls; skipped paths stop at original exits.
    let ranges = [(0x08114134, 0x0811414c), (0x0811415a, 0x08114164), (0x0811416a, 0x08114174), (0x080f62b6, 0x080f62f6), (0x0810dcb8, 0x0810dcee)];
    let visited: std::collections::BTreeSet<u32> = ranges.into_iter().flat_map(|(a, z)| (a..z).step_by(2)).collect();
    let view = recomp_core::analyze::View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = visited.into_iter().map(|addr| recomp_core::analyze::Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let field_object = root.join("selector-fields-native.o");
    recomp_core::tcc_memory::build(&recomp_core::analyze::Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &field_object, &mut |_| {}).unwrap();
    let (_field_library, field_table) = crate::load_native(&field_object).unwrap();
    let mut cases: Vec<(u8, usize)> = (0..3).flat_map(|kind| (0..14).map(move |row| (kind, row))).collect();
    cases.extend((0..10).map(|row| (3, row))); cases.extend((0..15).map(|row| (4, row)));
    cases.extend([(5, 0), (6, 0), (7, 0), (8, 15)]);
    let mut table_bytes = 0; let mut table_cases = 0;
    for (kind, row) in cases {
        let (start, end, expected) = match kind {
            0 => (0x08114134, 0x0811414c, 3), 1 => (0x0811415a, 0x08114164, 3), 2 => (0x0811416a, 0x08114174, 3),
            3 => (0x080f62b6, 0x080f62f6, 6), 4 => (0x0810dcb8, 0x0810dcee, 4),
            5 => (0x080f62b6, 0x080f6300, 1), 6 => (0x080f62b6, 0x080f6300, 0),
            7 => (0x0810dcb8, 0x0810dcfe, 1), 8 => (0x0810dcb8, 0x0810dd08, 0), _ => unreachable!(),
        };
        for m in [&mut raw, &mut native] {
            m.load_state(&initial).unwrap(); m.bus.ffta_sprites.enabled = false;
            m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
            m.cpu.regs[13] = 0x03007c00; m.cpu.regs[15] = start; m.cpu.cpsr = 0xbf;
            if kind <= 2 {
                m.cpu.regs[5] = 0x08000000 + MOTION_OFFSET + row as u32 * 8;
                m.cpu.regs[4] = 0x0201a000; m.cpu.regs[6] = 0x0201b000;
            } else if matches!(kind, 3 | 5 | 6) {
                m.cpu.regs[4] = 0x02018000; m.cpu.regs[7] = row as u32; m.cpu.regs[8] = 0; m.cpu.regs[10] = 0x02019000;
                let key = fields.positions.rows[row].match_value;
                m.bus.write8(0x02018004, if kind == 5 { key ^ 0xff } else { key });
                m.bus.write32(0x02019000 + row as u32 * 4, if kind == 6 { 0x0201b000 } else { 0 });
            } else {
                m.cpu.regs[7] = 0x02018000; m.bus.write8(0x02018006, row as u8);
                let key = if kind == 8 { 0 } else { u16::from(fields.horizontal.rows[row].match_value) };
                m.bus.write16(0x02018004, if kind == 7 { key | 0x100 } else { key });
            }
        }
        for &(a, z) in &spans { native.bus.rom[a as usize..z as usize].fill(0xcd); }
        prefix(&mut raw, start, end); let before = owner.compact_obj_catalog_stats().unwrap().source_bytes;
        let mut steps = 0;
        while native.cpu.regs[15] != end {
            assert!(steps < 64, "native table prefix did not return"); native.bus.frame_ready = false;
            field_table.get(native.cpu.regs[15] | 1).unwrap()(&RT_API, &mut native as *mut Machine as *mut std::ffi::c_void); steps += 1;
        }
        let used = owner.compact_obj_catalog_stats().unwrap().source_bytes - before;
        assert_eq!(used, expected, "kind {kind}, row {row}"); table_bytes += used; table_cases += 1;
        native.bus.rom.copy_from_slice(&rom); assert_eq!(native.save_state().unwrap(), raw.save_state().unwrap(), "kind {kind}, row {row}");
        let signed = |value: i16| value as i32 as u32;
        let (r1, r2, r3) = (raw.cpu.regs[1], raw.cpu.regs[2], raw.cpu.regs[3]);
        match kind {
            0 => { let v = &fields.motion.rows[row]; assert_eq!((r1, r2, r3), (u32::from(v.selector), u32::from(v.x), u32::from(v.y))); }
            1 => { let v = &fields.motion.rows[row]; assert_eq!((r1, r2, r3), (u32::from(v.x), signed(v.target_x), 60)); }
            2 => { let v = &fields.motion.rows[row]; assert_eq!((r1, r2, r3), (u32::from(v.y), signed(v.target_y), 60)); }
            3 => { let v = &fields.positions.rows[row]; assert_eq!((r1, r2, r3), (u32::from(v.selector), signed(v.x), signed(v.y))); }
            4 => { let v = &fields.horizontal.rows[row]; assert_eq!((r1, r2, r3), (u32::from(v.selector), signed(v.x), 8)); }
            6 => assert_eq!(raw.cpu.regs[8], 1), _ => {},
        }
    }
    assert_eq!((table_cases, table_bytes), (71, 248));
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "resources":100, "format":"compact-obj-catalog-v2-json", "previous_resources_unchanged":99, "previous_banks_unchanged":34,
        "new_banks":3, "new_selectors":22, "new_bank_bytes":1366, "new_table_bytes":218, "new_source_bytes":1584, "unknown_padding_raw":34,
        "native_count_cases":count_cases, "typed_count_bytes":count_bytes, "native_table_cases":table_cases, "typed_table_bytes":table_bytes,
        "table_skip_cases":4, "full_state_clock":true, "coherent_rejection_in_memory":true, "dataset_copies_created":0,
        "old_cases_repeated":false, "ordinary_use":"not attempted", "Linux":"pending", "USA":"pending", "nativeWindows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
