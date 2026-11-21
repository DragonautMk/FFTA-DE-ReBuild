//! European color-mode archives, physical ownership and original guest reader.
use super::*;
use gba_core::{capi::RT_API, rom_data::indexed_graphics as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

const RETURN: u32 = 0x0800_0300;
const DESTINATION: u32 = 0x0202_0000;

fn backing(machine: &mut Machine, rom: &[u8], altered: bool) {
    for (&start, &length) in model::COLOR_ARCHIVE_ROOTS.iter().zip(&model::COLOR_ARCHIVE_LENGTHS) {
        let start = start as usize;
        if altered {
            for at in start..start + length { machine.bus.rom[at] = !rom[at]; }
        } else {
            machine.bus.rom[start..start + length].copy_from_slice(&rom[start..start + length]);
        }
    }
}

fn setup(machine: &mut Machine, mode: u8, selector: u32) {
    machine.bus.ffta_sprites.enabled = false;
    machine.bus.ewram[0x2fc2] = mode;
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[0] = DESTINATION;
    machine.cpu.regs[1] = selector;
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = 0x080c_f304;
    machine.cpu.cpsr = 0xbf;
}

fn interpret(machine: &mut Machine, visited: &mut BTreeSet<(u32, bool)>) {
    for _ in 0..2_000_000 {
        if machine.cpu.regs[15] == RETURN { return; }
        visited.insert((machine.cpu.regs[15], machine.cpu.thumb()));
        machine.bus.frame_ready = false;
        machine.step();
    }
    panic!("original color-mode reader did not return at {:08x}", machine.cpu.regs[15]);
}

#[test]
#[ignore = "exact EUR ROM and E203 predecessor; Wine only, Linux/USA pending"]
fn european_color_indexed_graphics_publication_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_COLOR_INDEXED_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_COLOR_INDEXED_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old_manifest: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old_manifest.extractor_revision, old_manifest.decoder_revision, old_manifest.resources.len()), (203, 202, 123));
    let old = parse_indexed_graphics(&fs::read(previous.join(INDEXED_GRAPHICS_FILE)).unwrap()).unwrap();
    assert!(old.color_archives.is_none());
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (204, 203, 123));
    for prior in &old_manifest.resources {
        if prior.id == INDEXED_GRAPHICS_ID { continue; }
        let current = source.manifest.resources.iter().find(|item| item.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&prior.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    let graphics = &source.indexed_graphics.as_ref().unwrap().decoded;
    let mut without = graphics.clone();
    let archives = without.color_archives.take().unwrap();
    assert_eq!(without, old);
    assert_eq!(archives.len(), 3);
    let decoded_hashes = [
        "d5a0fc440152fd9dfa95b392db3f6a7f556b84307d2bf53e95fef3a721ae717b",
        "7fe7f5927ec0a7b1ea3f8bea8d402db8d427675cb7cf41be60ca80e0c6bc54b3",
        "45a3cef93b98f4c1ca2231ceed5df1136858e9ff8ea054a6c9876b4253920eef",
    ];
    for (i, archive) in archives.iter().enumerate() {
        let at = archive.source_offset as usize;
        let encoded = archive.source_bytes().unwrap();
        assert_eq!(archive.programs.len(), 165);
        assert_eq!(archive.fixed_decoded_bytes, 96);
        assert_eq!(encoded.len(), model::COLOR_ARCHIVE_LENGTHS[i]);
        assert_eq!(encoded, rom[at..at + encoded.len()]);
        assert_eq!(gba_core::ffta_rom::digest(&encoded), model::COLOR_ARCHIVE_SHA256[i]);
        let decoded: Vec<u8> = archive.programs.iter().flat_map(|p| p.decoded_bytes()).collect();
        assert_eq!(gba_core::ffta_rom::digest(&decoded), decoded_hashes[i]);
    }
    let current = source.manifest.resources.iter().find(|item| item.id == INDEXED_GRAPHICS_ID).unwrap();
    let prior = old_manifest.resources.iter().find(|item| item.id == INDEXED_GRAPHICS_ID).unwrap();
    assert_eq!(current.sources.len(), prior.sources.len() + 4);
    assert!(prior.sources.iter().all(|item| current.sources.contains(item)));
    assert_eq!(current.dependencies, vec![UNIT_OBJ_FFFF_READER_FIELDS_ID.to_string()]);
    for span in &current.sources {
        if span.role.starts_with("color-indexed-archive/") {
            assert!(span.offset + span.length <= model::COLOR_ARCHIVE_SHARED_START || span.offset >= model::COLOR_ARCHIVE_SHARED_END);
        }
    }
    assert_eq!(parse_indexed_graphics(&fs::read(prepared.directory.join(INDEXED_GRAPHICS_FILE)).unwrap()).unwrap(), *graphics);
    let mut malformed = graphics.clone();
    malformed.color_archives.as_mut().unwrap()[1].programs.pop();
    assert!(malformed.validate().is_err());
    let mut forged = graphics.clone();
    forged.color_archives.as_mut().unwrap()[0].opaque_header[0] ^= 1;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    backing(&mut managed, &rom, true);
    let before = owner.indexed_graphics_stats().unwrap();
    let prior_before = owner.unit_obj_ffff_reader_fields_stats().unwrap();
    for (&start, &length) in model::COLOR_ARCHIVE_ROOTS.iter().zip(&model::COLOR_ARCHIVE_LENGTHS) {
        for at in start as usize..start as usize + length {
            assert_eq!(managed.bus.read8(0x0800_0000 + at as u32), rom[at], "source byte {at:x}");
        }
    }
    let after = owner.indexed_graphics_stats().unwrap();
    let prior_after = owner.unit_obj_ffff_reader_fields_stats().unwrap();
    assert_eq!(after.color_source_bytes - before.color_source_bytes, 46_646);
    assert_eq!(prior_after.directory_bytes - prior_before.directory_bytes, 4);
    for at in 0x3d63d4..0x3d63d8 { managed.bus.rom[at] = !rom[at]; }
    for edge in [0x3cad94usize, 0x3ce9d1, 0x3ce9d4, 0x3d26d6, 0x3d26d8, 0x3d4830, 0x3d4834, 0x3d63d3, 0x3d63d4] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.checked_add_signed(delta).unwrap();
            let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
            for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
                assert_eq!((RT_API.read16)(ptr, mirror + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())), "read16 {at:x}");
                assert_eq!((RT_API.read32)(ptr, mirror + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()), "read32 {at:x}");
            }
        }
    }
    let before_new = owner.indexed_graphics_stats().unwrap().color_source_bytes;
    let before_old = owner.unit_obj_ffff_reader_fields_stats().unwrap().directory_bytes;
    let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for at in [0x3d482fusize, 0x3d4832] {
        assert_eq!((RT_API.read32)(ptr, 0x0800_0000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
    }
    assert_eq!(owner.indexed_graphics_stats().unwrap().color_source_bytes - before_new, 3);
    assert_eq!(owner.unit_obj_ffff_reader_fields_stats().unwrap().directory_bytes - before_old, 5);
    managed.bus.rom[0x3d63d4..0x3d63d8].copy_from_slice(&rom[0x3d63d4..0x3d63d8]);
    backing(&mut managed, &rom, false);

    let initial = raw.save_state().unwrap();
    let cases = [(0u8, 125u32, 0usize), (1, 127, 1), (2, 129, 2), (3, 125, 0), (0, 164, 0)];
    let mut states = Vec::new();
    let mut visited = BTreeSet::new();
    for &(mode, selector, bank) in &cases {
        raw.load_state(&initial).unwrap();
        managed.load_state(&initial).unwrap();
        for machine in [&mut raw, &mut managed] { setup(machine, mode, selector); }
        backing(&mut managed, &rom, true);
        interpret(&mut raw, &mut visited);
        interpret(&mut managed, &mut BTreeSet::new());
        let start = (DESTINATION - 0x0200_0000) as usize;
        assert_eq!(&raw.bus.ewram[start..start + 96], archives[bank].programs[selector as usize].decoded_bytes());
        backing(&mut managed, &rom, false);
        let state = raw.save_state().unwrap();
        assert_eq!(managed.save_state().unwrap(), state, "interpreter mode {mode} selector {selector}");
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
    let object = root.join("color-indexed-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    for (&(mode, selector, _), expected) in cases.iter().zip(&states) {
        managed.load_state(&initial).unwrap();
        setup(&mut managed, mode, selector);
        backing(&mut managed, &rom, true);
        for _ in 0..2_000_000 {
            if managed.cpu.regs[15] == RETURN { break; }
            managed.bus.frame_ready = false;
            table.get(managed.cpu.regs[15] | u32::from(managed.cpu.thumb())).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void);
        }
        assert_eq!(managed.cpu.regs[15], RETURN);
        backing(&mut managed, &rom, false);
        assert_eq!(&managed.save_state().unwrap(), expected, "native mode {mode} selector {selector}");
    }
    let snapshot = managed.save_state().unwrap();
    let unavailable = root.join("temporarily unavailable dataset");
    fs::rename(&prepared.directory, &unavailable).unwrap();
    assert!(attach_from_directory(&mut managed, &prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    managed.load_state(&snapshot).unwrap();
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    fs::rename(&unavailable, &prepared.directory).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[204,203], "archives":3, "programs":495,
        "source_bytes":46650, "shared_previous_owner_bytes":4, "prior_resources_preserved":122,
        "all_source_bytes_and_mixed_boundaries":true, "original_interpreter_native_cases":cases.len(),
        "missing_dataset_and_snapshot_owner_retained":true, "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
