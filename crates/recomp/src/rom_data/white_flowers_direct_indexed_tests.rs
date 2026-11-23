//! EUR directly rooted indexed archive and original one-program wrapper.
use super::*;
use gba_core::{capi::RT_API, rom_data::indexed_graphics as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

const RETURN: u32 = 0x0800_0300;
const DESTINATION: u32 = 0x0202_0000;
fn spans() -> [(u32, usize); 2] {
    [(model::WHITE_FLOWERS_DIRECT_LITERAL, 4), (model::WHITE_FLOWERS_DIRECT_ROOT, model::WHITE_FLOWERS_DIRECT_LENGTH)]
}
fn backing(machine: &mut Machine, rom: &[u8], altered: bool) {
    for (start, length) in spans() {
        let at = start as usize;
        if altered { for i in at..at + length { machine.bus.rom[i] = !rom[i]; } }
        else { machine.bus.rom[at..at + length].copy_from_slice(&rom[at..at + length]); }
    }
}
fn setup(machine: &mut Machine, selector: u32) {
    machine.bus.ffta_sprites.enabled = false;
    let at = (DESTINATION - 0x0200_0000) as usize;
    machine.bus.ewram[at..at + 128].fill(0x5a);
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[0] = DESTINATION;
    machine.cpu.regs[1] = selector;
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = 0x080c_f3e0;
    machine.cpu.cpsr = 0xbf;
}
fn interpret(machine: &mut Machine, visited: &mut BTreeSet<(u32, bool)>) {
    for _ in 0..2_000_000 {
        if machine.cpu.regs[15] == RETURN { return; }
        visited.insert((machine.cpu.regs[15], machine.cpu.thumb()));
        machine.bus.frame_ready = false;
        machine.step();
    }
    panic!("original indexed wrapper did not return at {:08x}", machine.cpu.regs[15]);
}

#[test]
#[ignore = "exact EUR ROM and E277 predecessor; Wine only, Linux/USA pending"]
fn european_white_flowers_direct_indexed_and_original_wrapper() {
    let root = PathBuf::from(std::env::var_os("FFTA_WHITE_DIRECT_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_WHITE_DIRECT_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old_manifest: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old_manifest.extractor_revision, old_manifest.decoder_revision, old_manifest.resources.len()), (277, 276, 146));
    let old = parse_indexed_graphics(&fs::read(previous.join(INDEXED_GRAPHICS_FILE)).unwrap()).unwrap();
    assert!(old.white_flowers_direct.is_none());
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (278, 277, 146));
    for prior in &old_manifest.resources {
        if prior.id == INDEXED_GRAPHICS_ID { continue; }
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&prior.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    let graphics = &source.indexed_graphics.as_ref().unwrap().decoded;
    let mut without = graphics.clone();
    let direct = without.white_flowers_direct.take().unwrap();
    assert_eq!(without, old);
    let archive = &direct.archive;
    let bytes = archive.source_bytes().unwrap();
    assert_eq!((archive.programs.len(), archive.fixed_decoded_bytes, bytes.len()), (62, 128, model::WHITE_FLOWERS_DIRECT_LENGTH));
    assert_eq!(bytes, rom[model::WHITE_FLOWERS_DIRECT_ROOT as usize..model::WHITE_FLOWERS_DIRECT_ROOT as usize + bytes.len()]);
    assert_eq!(gba_core::ffta_rom::digest(&bytes), model::WHITE_FLOWERS_DIRECT_SHA256);
    assert_eq!(gba_core::ffta_rom::digest(&archive.selected_bytes(0, 0).unwrap()), "a34c43fb8480a20e2a130a3ff7a180745f7a034cd986e8164c4578bd5d122387");
    assert_eq!(model::WHITE_FLOWERS_DIRECT_ROOT + archive.programs[16].relative_offset, 0x3c9d94);
    assert_eq!(model::WHITE_FLOWERS_DIRECT_ROOT + archive.programs[17].relative_offset, 0x3c9df5);
    let current = source.manifest.resources.iter().find(|r| r.id == INDEXED_GRAPHICS_ID).unwrap();
    let prior = old_manifest.resources.iter().find(|r| r.id == INDEXED_GRAPHICS_ID).unwrap();
    assert_eq!(current.sources.len(), prior.sources.len() + 2);
    assert!(prior.sources.iter().all(|entry| current.sources.contains(entry)));
    assert_eq!(current.dependencies, prior.dependencies);
    assert_eq!(parse_indexed_graphics(&fs::read(prepared.directory.join(INDEXED_GRAPHICS_FILE)).unwrap()).unwrap(), *graphics);
    let mut invalid = graphics.clone(); invalid.white_flowers_direct.as_mut().unwrap().archive.programs.pop(); assert!(invalid.validate().is_err());
    let mut forged = graphics.clone(); forged.white_flowers_direct.as_mut().unwrap().archive.opaque_header[0] ^= 1;
    forged.validate().unwrap(); assert!(forged.verify_source(&rom).is_err());
    let bytes = encode_indexed_graphics(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut changed = dataset.manifest.clone(); changed.resources.iter_mut().find(|e| e.id == INDEXED_GRAPHICS_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files(); files.manifest = &changed; files.indexed_graphics = Some((&forged, &bytes));
    assert!(source.validate_files(files).is_err()); drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    backing(&mut managed, &rom, true);
    let before = owner.indexed_graphics_stats().unwrap();
    for (start, length) in spans() {
        for at in start as usize..start as usize + length {
            for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
                assert_eq!(managed.bus.read8(mirror + at as u32), rom[at], "source byte {at:x}");
            }
        }
    }
    let after = owner.indexed_graphics_stats().unwrap();
    assert_eq!(after.white_flowers_direct_literal_bytes - before.white_flowers_direct_literal_bytes, 12);
    assert_eq!(after.white_flowers_direct_archive_bytes - before.white_flowers_direct_archive_bytes, (model::WHITE_FLOWERS_DIRECT_LENGTH * 3) as u64);
    assert_eq!(after.color_source_bytes, before.color_source_bytes);
    assert_eq!(after.white_flowers_archive_bytes, before.white_flowers_archive_bytes);
    let owned = |at: usize| source.manifest.resources.iter().flat_map(|e| &e.sources).any(|s| (s.offset as usize..s.offset as usize + s.length as usize).contains(&at));
    let mut changed = BTreeSet::new();
    for (start, length) in spans() {
        for edge in [start as usize, start as usize + length] {
            for at in edge.saturating_sub(3)..edge + 3 { if !owned(at) { changed.insert(at); managed.bus.rom[at] = rom[at] ^ 0x5a; } }
        }
    }
    // The adjacent color archive retains its pre-existing owner after a three-byte gap.
    let next = model::COLOR_ARCHIVE_ROOTS[0] as usize;
    assert_eq!(model::WHITE_FLOWERS_DIRECT_ROOT as usize + model::WHITE_FLOWERS_DIRECT_LENGTH + 3, next);
    assert!((next..next + 4).all(&owned));
    for at in next..next + 4 { managed.bus.rom[at] = !rom[at]; }
    for edge in [model::WHITE_FLOWERS_DIRECT_LITERAL as usize, model::WHITE_FLOWERS_DIRECT_LITERAL as usize + 4,
                 model::WHITE_FLOWERS_DIRECT_ROOT as usize, next - 3, next] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap();
            let expected = |i: usize| if changed.contains(&(at + i)) { rom[at + i] ^ 0x5a } else { rom[at + i] };
            let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
            assert_eq!((RT_API.read16)(ptr, 0x0a00_0000 + at as u32), u32::from(u16::from_le_bytes([expected(0), expected(1)])));
            assert_eq!((RT_API.read32)(ptr, 0x0800_0000 + at as u32), u32::from_le_bytes([expected(0), expected(1), expected(2), expected(3)]));
        }
    }
    assert!(owner.indexed_graphics_stats().unwrap().color_source_bytes > after.color_source_bytes);
    managed.bus.rom[next..next + 4].copy_from_slice(&rom[next..next + 4]);
    for at in changed { managed.bus.rom[at] = rom[at]; }
    backing(&mut managed, &rom, false);

    let selectors = [0u32, 16, 61, 62, 255, 0x110];
    let mut visited = BTreeSet::new();
    let mut states = Vec::new();
    for &selector in &selectors {
        for machine in [&mut raw, &mut managed] { machine.load_state(&initial).unwrap(); setup(machine, selector); }
        backing(&mut managed, &rom, true);
        interpret(&mut raw, &mut visited); interpret(&mut managed, &mut BTreeSet::new());
        let at = (DESTINATION - 0x0200_0000) as usize;
        let program = selector as u8 as u32;
        if let Some(expected) = archive.selected_bytes(program, 1) { assert_eq!(&raw.bus.ewram[at..at + 128], expected); }
        else { assert_eq!(&raw.bus.ewram[at..at + 128], &[0x5a; 128]); }
        backing(&mut managed, &rom, false);
        let state = raw.save_state().unwrap();
        assert_eq!(managed.save_state().unwrap(), state, "interpreter selector {selector}");
        states.push(state);
    }
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = visited.iter().map(|&(address, thumb)| {
        let mut instrs = vec![if thumb { armv4t::decode_thumb(view.read16(address), address) } else { armv4t::decode_arm(view.read32(address), address) }];
        if thumb {
            let next = armv4t::decode_thumb(view.read16(address + 2), address + 2);
            if armv4t::fuse_thumb_bl(&instrs[0], &next).is_some() { instrs.push(next); }
        }
        Block { addr: address, thumb, instrs }
    }).collect();
    let object = root.join("white-flowers-direct-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    for (&selector, expected) in selectors.iter().zip(&states) {
        managed.load_state(&initial).unwrap(); setup(&mut managed, selector); backing(&mut managed, &rom, true);
        for _ in 0..2_000_000 {
            if managed.cpu.regs[15] == RETURN { break; }
            managed.bus.frame_ready = false;
            table.get(managed.cpu.regs[15] | u32::from(managed.cpu.thumb())).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void);
        }
        assert_eq!(managed.cpu.regs[15], RETURN);
        backing(&mut managed, &rom, false);
        assert_eq!(&managed.save_state().unwrap(), expected, "native selector {selector}");
    }
    let state = managed.save_state().unwrap();
    let unavailable = root.join("temporarily unavailable dataset");
    fs::rename(&prepared.directory, &unavailable).unwrap();
    assert!(attach_from_directory(&mut managed, &prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    managed.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    fs::rename(&unavailable, &prepared.directory).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[278,277], "resources":146, "new_source_bytes":6361,
        "programs":62, "decoded_bytes_per_program":128, "prior_other_resources_unchanged":145,
        "prior_representation_preserved":true, "source_mirrors_mixed_boundaries":true,
        "original_interpreter_and_generated_cases":selectors.len(), "program_16_and_boundary_selectors_verified":true,
        "existing_color_owner_and_three_byte_gap_preserved":true, "coherent_edit_rejected":true, "snapshot_owner_retained":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
