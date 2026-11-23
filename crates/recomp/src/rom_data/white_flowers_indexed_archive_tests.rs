//! EUR White Flowers language-indexed archives and original guest readers.
use super::*;
use gba_core::{capi::RT_API, rom_data::indexed_graphics as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

const RETURN: u32 = 0x0800_0300;
const DESTINATION: u32 = 0x0202_0000;
const DECODED_SHA256: [&str; 5] = [
    "25b798c04b8742b05a1dc28f5f7e21533d4d42a5a4ef0dde64eced0de08740b7",
    "c0abbf701e805ae16fd2d9ee2425c67e98db3d5a9a93471ed114ab9eb5b427e0",
    "e40ab9c43c4684ec199862a31d7d9b37c2ad0b91d921bbf30d8e6e48c7aa535c",
    "2a7b0dc01538b475e6e8f2dc68ef3d4dec040d64609ccbfbc334ac4d6e79d5ae",
    "85ba1a1f3c531dd20a41956fb0d2f26c2fd2281cdfd878571e9e2412e258e457",
];
fn spans() -> Vec<(u32, usize)> {
    [(model::WHITE_FLOWERS_LITERALS, 16), (model::WHITE_FLOWERS_GROUP, 20)]
        .into_iter().chain(model::WHITE_FLOWERS_ROOTS.into_iter().zip(model::WHITE_FLOWERS_LENGTHS)).collect()
}
fn backing(machine: &mut Machine, rom: &[u8], altered: bool) {
    for (start, length) in spans() {
        let at = start as usize;
        if altered { for i in at..at + length { machine.bus.rom[i] = !rom[i]; } }
        else { machine.bus.rom[at..at + length].copy_from_slice(&rom[at..at + length]); }
    }
}
fn setup(machine: &mut Machine, pc: u32, slot: u8) {
    machine.bus.ffta_sprites.enabled = false;
    machine.bus.ewram[0x1f6e] = slot;
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[4] = DESTINATION;
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = pc;
    machine.cpu.cpsr = 0xbf;
    if pc == 0x0800_537c {
        machine.cpu.regs[0] = 0x0800_0000 + model::WHITE_FLOWERS_ROOTS[slot as usize];
        machine.cpu.regs[1] = DESTINATION;
        machine.cpu.regs[2] = 0;
        machine.cpu.regs[3] = 0;
    }
}
fn interpret(machine: &mut Machine, stop: u32, visited: &mut BTreeSet<(u32, bool)>) {
    for _ in 0..2_000_000 {
        if machine.cpu.regs[15] == stop { return; }
        visited.insert((machine.cpu.regs[15], machine.cpu.thumb()));
        machine.bus.frame_ready = false;
        machine.step();
    }
    panic!("original indexed reader did not stop at {:08x}", machine.cpu.regs[15]);
}

#[test]
#[ignore = "exact EUR ROM and E275 predecessor; Wine only, Linux/USA pending"]
fn european_white_flowers_indexed_archives_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_WHITE_INDEXED_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_WHITE_INDEXED_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old_manifest: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old_manifest.extractor_revision, old_manifest.decoder_revision, old_manifest.resources.len()), (275, 274, 146));
    let old = parse_indexed_graphics(&fs::read(previous.join(INDEXED_GRAPHICS_FILE)).unwrap()).unwrap();
    assert!(old.white_flowers.is_none());
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (276, 275, 146));
    for prior in &old_manifest.resources {
        if prior.id == INDEXED_GRAPHICS_ID { continue; }
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&prior.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    let graphics = &source.indexed_graphics.as_ref().unwrap().decoded;
    let mut without = graphics.clone();
    let white = without.white_flowers.take().unwrap();
    assert_eq!(without, old);
    let current = source.manifest.resources.iter().find(|r| r.id == INDEXED_GRAPHICS_ID).unwrap();
    let prior = old_manifest.resources.iter().find(|r| r.id == INDEXED_GRAPHICS_ID).unwrap();
    assert_eq!(current.sources.len(), prior.sources.len() + 7);
    assert!(prior.sources.iter().all(|item| current.sources.contains(item)));
    assert_eq!(current.dependencies, prior.dependencies);
    assert_eq!(white.group.pointers, model::WHITE_FLOWERS_ROOTS);
    for (i, archive) in white.archives.iter().enumerate() {
        let at = archive.source_offset as usize;
        let bytes = archive.source_bytes().unwrap();
        assert_eq!((archive.programs.len(), archive.fixed_decoded_bytes), (33, 64));
        assert_eq!(bytes.len(), model::WHITE_FLOWERS_LENGTHS[i]);
        assert_eq!(bytes, rom[at..at + bytes.len()]);
        assert_eq!(gba_core::ffta_rom::digest(&bytes), model::WHITE_FLOWERS_SHA256[i]);
        assert_eq!(gba_core::ffta_rom::digest(&archive.selected_bytes(0, 0).unwrap()), DECODED_SHA256[i]);
    }
    assert_eq!(parse_indexed_graphics(&fs::read(prepared.directory.join(INDEXED_GRAPHICS_FILE)).unwrap()).unwrap(), *graphics);
    let mut malformed = graphics.clone();
    malformed.white_flowers.as_mut().unwrap().archives[1].programs.pop();
    assert!(malformed.validate().is_err());
    let mut forged = graphics.clone();
    forged.white_flowers.as_mut().unwrap().archives[0].opaque_header[0] ^= 1;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());
    let bytes = encode_indexed_graphics(&forged).unwrap();
    let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered = dataset.manifest.clone();
    altered.resources.iter_mut().find(|e| e.id == INDEXED_GRAPHICS_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files();
    files.manifest = &altered;
    files.indexed_graphics = Some((&forged, &bytes));
    assert!(source.validate_files(files).is_err());
    drop(dataset);

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
    assert_eq!(after.source_bytes - before.source_bytes, 6431 * 3);
    assert_eq!(after.color_source_bytes, before.color_source_bytes);
    for (i, length) in model::WHITE_FLOWERS_LENGTHS.iter().enumerate() {
        assert_eq!(after.white_flowers_archive_bytes[i] - before.white_flowers_archive_bytes[i], (*length * 3) as u64);
    }
    let owned = |at: usize| source.manifest.resources.iter().flat_map(|e| &e.sources).any(|s| (s.offset as usize..s.offset as usize + s.length as usize).contains(&at));
    let mut changed = BTreeSet::new();
    for (start, length) in spans() {
        for edge in [start as usize, start as usize + length] {
            for at in edge.saturating_sub(3)..edge + 3 {
                if !owned(at) { changed.insert(at); managed.bus.rom[at] = rom[at] ^ 0x5a; }
            }
        }
    }
    for (start, length) in spans() {
        for edge in [start as usize, start as usize + length] {
            for delta in [-3isize, -1, 0, 1] {
                let at = edge.checked_add_signed(delta).unwrap();
                let expected = |i: usize| if changed.contains(&(at + i)) { rom[at + i] ^ 0x5a } else { rom[at + i] };
                let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
                assert_eq!((RT_API.read16)(ptr, 0x0a00_0000 + at as u32), u32::from(u16::from_le_bytes([expected(0), expected(1)])));
                assert_eq!((RT_API.read32)(ptr, 0x0800_0000 + at as u32), u32::from_le_bytes([expected(0), expected(1), expected(2), expected(3)]));
            }
        }
    }
    for at in changed { managed.bus.rom[at] = rom[at]; }
    backing(&mut managed, &rom, false);

    let mut visited = BTreeSet::new();
    let mut cases = Vec::new();
    for slot in 0..5u8 {
        for (pc, stop) in [(0x0808_9eb4, 0x0808_9eca), (0x0800_537c, RETURN), (0x0808_9ece, 0x0808_9ed0)] {
            for machine in [&mut raw, &mut managed] { machine.load_state(&initial).unwrap(); setup(machine, pc, slot); }
            backing(&mut managed, &rom, true);
            interpret(&mut raw, stop, &mut visited);
            interpret(&mut managed, stop, &mut BTreeSet::new());
            if pc == 0x0808_9eb4 { assert_eq!(raw.cpu.regs[0], 0x0800_0000 + model::WHITE_FLOWERS_ROOTS[slot as usize]); }
            if pc == 0x0800_537c {
                let at = (DESTINATION - 0x0200_0000) as usize;
                assert_eq!(&raw.bus.ewram[at..at + 2112], white.archives[slot as usize].selected_bytes(0, 0).unwrap());
            }
            backing(&mut managed, &rom, false);
            let state = raw.save_state().unwrap();
            assert_eq!(managed.save_state().unwrap(), state, "interpreter slot {slot} at {pc:08x}");
            cases.push((slot, pc, stop, state));
        }
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
    let object = root.join("white-flowers-indexed-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    for (slot, pc, stop, expected) in &cases {
        managed.load_state(&initial).unwrap(); setup(&mut managed, *pc, *slot); backing(&mut managed, &rom, true);
        for _ in 0..2_000_000 {
            if managed.cpu.regs[15] == *stop { break; }
            managed.bus.frame_ready = false;
            table.get(managed.cpu.regs[15] | u32::from(managed.cpu.thumb())).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void);
        }
        assert_eq!(managed.cpu.regs[15], *stop);
        backing(&mut managed, &rom, false);
        assert_eq!(&managed.save_state().unwrap(), expected, "native slot {slot} at {pc:08x}");
    }
    // The sixth slot has a different existing owner; preserve that owner as well.
    let adjacent = model::WHITE_FLOWERS_GROUP as usize + 20;
    assert!((adjacent..adjacent + 4).all(&owned));
    for machine in [&mut raw, &mut managed] { machine.load_state(&initial).unwrap(); setup(machine, 0x0808_9eb4, 5); }
    backing(&mut managed, &rom, true);
    for at in adjacent..adjacent + 4 { managed.bus.rom[at] = !rom[at]; }
    interpret(&mut raw, 0x0808_9eca, &mut BTreeSet::new());
    interpret(&mut managed, 0x0808_9eca, &mut BTreeSet::new());
    assert_eq!(raw.cpu.regs[0], u32::from_le_bytes(rom[adjacent..adjacent + 4].try_into().unwrap()));
    managed.bus.rom[adjacent..adjacent + 4].copy_from_slice(&rom[adjacent..adjacent + 4]);
    backing(&mut managed, &rom, false);
    assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());

    // Slot 30 is outside both this group and all other declared sources.
    let fallback = model::WHITE_FLOWERS_GROUP as usize + 30 * 4;
    assert!((fallback..fallback + 4).all(|at| !owned(at)));
    for machine in [&mut raw, &mut managed] {
        machine.load_state(&initial).unwrap(); setup(machine, 0x0808_9eb4, 30);
        for at in fallback..fallback + 4 { machine.bus.rom[at] = rom[at] ^ 0x5a; }
    }
    backing(&mut managed, &rom, true);
    interpret(&mut raw, 0x0808_9eca, &mut BTreeSet::new());
    interpret(&mut managed, 0x0808_9eca, &mut BTreeSet::new());
    let expected = u32::from_le_bytes(std::array::from_fn(|i| rom[fallback + i] ^ 0x5a));
    assert_eq!(raw.cpu.regs[0], expected); assert_eq!(managed.cpu.regs[0], expected);
    for machine in [&mut raw, &mut managed] { machine.bus.rom[fallback..fallback + 4].copy_from_slice(&rom[fallback..fallback + 4]); }
    backing(&mut managed, &rom, false);
    assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
    let state = managed.save_state().unwrap();
    let unavailable = root.join("temporarily unavailable dataset");
    fs::rename(&prepared.directory, &unavailable).unwrap();
    assert!(attach_from_directory(&mut managed, &prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    managed.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    fs::rename(&unavailable, &prepared.directory).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[276,275], "resources":146, "new_source_bytes":6431,
        "archives":5, "programs_per_archive":33, "prior_other_resources_unchanged":145,
        "prior_representation_preserved":true, "source_mirrors_mixed_boundaries":true,
        "original_interpreter_and_generated_cases":cases.len(), "all_five_decoder_outputs_verified":true,
        "unselected_language_fallback":true, "coherent_edit_rejected":true, "snapshot_owner_retained":true,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
