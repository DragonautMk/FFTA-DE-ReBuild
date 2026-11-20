//! EUR adjacent offset tables and original Thumb byte readers.
use super::*;
use gba_core::{capi::RT_API, rom_data::adjacent_offset_tables as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};

#[test]
#[ignore = "exact EUR ROM, E193 predecessor and Wine; Linux/USA pending"]
fn european_adjacent_offset_tables_publication_and_native_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_OFFSET_TABLE_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_OFFSET_TABLE_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (193, 192, 122));
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (194, 193, 123));
    for prior in &old.resources {
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    let data = &source.adjacent_offset_tables.as_ref().unwrap().decoded;
    assert_eq!(data.source_bytes().unwrap(), rom[model::SOURCE_OFFSET as usize..model::SOURCE_OFFSET as usize + model::SOURCE_LENGTH]);
    assert_eq!(parse_adjacent_offset_tables(&fs::read(prepared.directory.join(ADJACENT_OFFSET_TABLES_FILE)).unwrap()).unwrap(), *data);
    let mut malformed = serde_json::to_value(data).unwrap(); malformed["vectors"][0].as_array_mut().unwrap().pop();
    assert!(serde_json::from_value::<model::AdjacentOffsetTables>(malformed).is_err());
    let mut bad = data.clone(); bad.rows[0][0] ^= 1;
    let bad_bytes = encode_adjacent_offset_tables(&bad).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered_manifest = dataset.manifest.clone();
    altered_manifest.resources.iter_mut().find(|entry| entry.id == ADJACENT_OFFSET_TABLES_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bad_bytes);
    let mut files = dataset.files(); files.manifest = &altered_manifest; files.adjacent_offset_tables = Some((&bad, &bad_bytes));
    assert!(source.validate_files(files).is_err(), "coherent file/hash edits must be rejected");
    drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None); let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let start = model::SOURCE_OFFSET as usize; let end = start + model::SOURCE_LENGTH;
    for at in start..end { managed.bus.rom[at] = !rom[at]; }
    for at in start..end { assert_eq!(managed.bus.read8(0x0c000000 + at as u32), rom[at]); }
    for edge in [start, start + 16, start + 20, start + 24, start + 28, end] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap(); let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
            assert_eq!((RT_API.read16)(pointer, 0x0a000000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
            assert_eq!((RT_API.read32)(pointer, 0x08000000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
        }
    }
    managed.bus.rom[start..end].copy_from_slice(&rom[start..end]); let initial = raw.save_state().unwrap();
    // Compile each single reader separately so adjacent entries cannot chain.
    // base, target, source offset, optional zero index register, signed result, owner table.
    let cases = [
        (0x08130392, 4usize, 1usize, 4usize, None, false, 0usize),
        (0x081303bc, 4, 0, 4, Some(0usize), true, 0),
        (0x08130372, 0, 0, 16, None, false, 1),
        (0x08130394, 0, 3, 20, None, false, 2),
        (0x081304e6, 0, 1, 24, Some(1), true, 3),
        (0x08130f36, 0, 1, 29, Some(1), true, 4),
    ];
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    for (case, &(reader, base, target, relative, index_reg, signed, owner_table)) in cases.iter().enumerate() {
        let block = Block { addr: reader, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(reader), reader)] };
        let object = root.join(format!("offset-reader-{case}.o"));
        recomp_core::tcc_memory::build(&Analysis { blocks: vec![block] }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
        let (_library, table) = crate::load_native(&object).unwrap();
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap(); machine.bus.ffta_sprites.enabled = false;
            machine.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
            if let Some(index_reg) = index_reg { machine.cpu.regs[index_reg] = 0; }
            machine.cpu.regs[base] = 0x08000000 + model::SOURCE_OFFSET + relative as u32;
            machine.cpu.regs[13] = 0x03007c00; machine.cpu.regs[14] = 0x08000301;
            machine.cpu.regs[15] = reader; machine.cpu.cpsr = 0xbf;
        }
        for at in start..end { managed.bus.rom[at] = !rom[at]; }
        let before = owner.adjacent_offset_tables_stats().unwrap().table_bytes[owner_table];
        raw.bus.frame_ready = false; raw.step(); managed.bus.frame_ready = false;
        table.get(reader | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void);
        assert_eq!(owner.adjacent_offset_tables_stats().unwrap().table_bytes[owner_table] - before, 1);
        let byte = rom[start + relative]; let expected = if signed { (byte as i8 as i32) as u32 } else { u32::from(byte) };
        assert_eq!(raw.cpu.regs[target], expected);
        managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
        assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap(), "reader case {case}");
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[194,193], "resources":123, "preserved_other_resources":122,
        "tables":5, "new_source_bytes":32, "poisoned_mirrored_and_mixed_boundary_reads":true,
        "coherent_edit_rejected":true, "original_interpreter_native_reader_cases":6, "full_state_clock_equal":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
