//! Three complete EUR indexed families and their original readers.
use super::*;
use gba_core::{capi::RT_API, rom_data::indexed_graphics::{self as model, additional as extra}, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

const RETURN: u32 = 0x0800_0300;
const DESTINATION: u32 = 0x0201_0000;
const DECODED_SHA256: [&str; 15] = [
    "90f4490a2a8183d6235d54d2d62e38e50925e649ce7ba304dbe17c7cdeaffa40",
    "16120136cbb19bb0060e9f689155056637a912fe70d326d602b57ea3b0d0c088",
    "98782578618324757a98425e6164166d94b19739065d708c7ec54c7dccfe04ef",
    "df6813cfb41b8b1f6cb86a1239e5b4ddc2a4978ded3aac1c656e8238c73c81ef",
    "a136483b27b71cf98fa611292468816d9dc11314ad44f14ac88c6e0dae72c18e",
    "821723e0db4543f543433d0d203c86b51f0be943127c37c5a8ddfba5015f8bec",
    "7226f4c424bf43f85a5c3c40034ea470455cdc28c7dfffa55a6a74e480bbecd0",
    "619fb40ad4ef327b78d62b76816c2c05dea3ea0700535eb8fbe5668c9f94ce10",
    "0fe05004f6c72c8fcc21336e61bc62bc52b6a42ae775cef3aa7f543f006cea9d",
    "f023398867c436d66fad158f49ca2e528174792c71583a5db652994bea4d0c03",
    "44e6c439a20427b61b45cf072592883a9d600253804fa88a3a7741ec48d04e28",
    "af65f8b18b861e657fea01a60b1b0b813bd19c88547fbe7962ac89f6ebd38dad",
    "7fed444a1b3421af4e9ce3f6dafa631115c9e90977d866b6f8b5caa179940bbb",
    "80b64b294486c631f46e8c3bca057efc78fb9b60d1d47c04d8e98a102011b189",
    "d6758dab4f5a9c8a54fa11c6847c6e9d5a0487302768e4d12badddc247205700",
];

fn spans() -> Vec<(u32, usize)> {
    extra::GROUPS.into_iter().map(|at| (at, 20))
        .chain(extra::CALLER_LITERALS.into_iter().map(|(at, _)| (at, 4)))
        .chain(extra::POINTERS[2].into_iter().map(|at| (at, extra::EMBEDDED_PREFIX_LENGTH)))
        .chain(extra::ARCHIVES.into_iter().map(|spec| (spec.offset, spec.length)))
        .collect()
}

fn backing(machine: &mut Machine, rom: &[u8], poison: bool) {
    for (start, length) in spans() {
        let at = start as usize;
        for i in at..at + length { machine.bus.rom[i] = if poison { !rom[i] } else { rom[i] }; }
    }
}

#[derive(Clone, Copy)]
enum Case {
    Archive(usize),
    Selector { group: u8, language: u8, metadata_index: u8 },
}

fn setup(machine: &mut Machine, case: Case) -> u32 {
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[13] = 0x0300_7c00; machine.cpu.regs[14] = RETURN | 1; machine.cpu.cpsr = 0xbf;
    match case {
        Case::Archive(index) => {
            machine.cpu.regs[0] = 0x0800_0000 + extra::ARCHIVES[index].offset;
            machine.cpu.regs[1] = DESTINATION; machine.cpu.regs[2] = 0; machine.cpu.regs[3] = 0;
            machine.cpu.regs[15] = 0x0800_537c; RETURN
        }
        Case::Selector { group, language, metadata_index } => {
            machine.bus.write8(0x0200_1f6e, language);
            match group {
                0 => {
                    machine.cpu.regs[5] = DESTINATION; machine.cpu.regs[6] = 0x0200_1f6e;
                    machine.cpu.regs[15] = 0x0805_9d42; 0x0805_9d52
                }
                1 => {
                    machine.cpu.regs[4] = 0x0200_1f6e; machine.cpu.regs[5] = DESTINATION;
                    machine.cpu.regs[6] = 0; machine.cpu.regs[7] = 0;
                    machine.cpu.regs[15] = 0x0803_7bb6; 0x0803_7bd6
                }
                2 => {
                    machine.cpu.regs[2] = 2; machine.cpu.regs[3] = DESTINATION;
                    machine.cpu.regs[15] = 0x080c_f42a; 0x080c_f446
                }
                3 => {
                    machine.cpu.regs[3] = u32::from(metadata_index);
                    machine.cpu.regs[15] = 0x080c_f470; 0x080c_f48a
                }
                _ => unreachable!(),
            }
        }
    }
}

fn interpret(machine: &mut Machine, stop: u32, mut visited: Option<&mut BTreeSet<(u32, bool)>>) {
    for _ in 0..10_000_000 {
        if machine.cpu.regs[15] == stop { return; }
        if let Some(visited) = visited.as_deref_mut() { visited.insert((machine.cpu.regs[15], machine.cpu.thumb())); }
        machine.bus.frame_ready = false; machine.step();
    }
    panic!("additional indexed reader escaped at {:08x}", machine.cpu.regs[15]);
}

#[test]
#[ignore = "exact EUR ROM and E300 predecessor; Wine only, Linux/USA pending"]
fn european_fifteen_additional_indexed_archives_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_ADDITIONAL_INDEXED_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_ADDITIONAL_INDEXED_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old_manifest: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old_manifest.extractor_revision, old_manifest.decoder_revision, old_manifest.resources.len()), (300, 299, 148));
    let old = parse_indexed_graphics(&fs::read(previous.join(INDEXED_GRAPHICS_FILE)).unwrap()).unwrap();
    assert!(old.additional.is_none());
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (301, 300, 148));
    for prior in &old_manifest.resources {
        if prior.id == INDEXED_GRAPHICS_ID { continue; }
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(previous.join(&prior.path)).unwrap(), fs::read(prepared.directory.join(&current.path)).unwrap());
    }
    let graphics = &source.indexed_graphics.as_ref().unwrap().decoded;
    let mut without = graphics.clone(); let additional = without.additional.take().unwrap(); assert_eq!(without, old);
    let current = source.manifest.resources.iter().find(|r| r.id == INDEXED_GRAPHICS_ID).unwrap();
    let prior = old_manifest.resources.iter().find(|r| r.id == INDEXED_GRAPHICS_ID).unwrap();
    assert_eq!(current.sources.len(), prior.sources.len() + 29);
    assert!(prior.sources.iter().all(|item| current.sources.contains(item)));
    assert_eq!(current.dependencies, prior.dependencies);
    assert_eq!(additional.archives.iter().map(|a| a.programs.len()).sum::<usize>(), 1588);
    for (i, archive) in additional.archives.iter().enumerate() {
        assert_eq!(archive.source_bytes().unwrap(), rom[archive.source_offset as usize..archive.source_offset as usize + extra::ARCHIVES[i].length]);
        assert_eq!(gba_core::ffta_rom::digest(&archive.selected_bytes(0, 0).unwrap()), DECODED_SHA256[i]);
    }
    let mut invalid = graphics.clone(); invalid.additional.as_mut().unwrap().archives[0].programs.pop(); assert!(invalid.validate().is_err());
    let mut forged = graphics.clone(); forged.additional.as_mut().unwrap().archives[0].opaque_header[0] ^= 1;
    forged.validate().unwrap(); assert!(forged.verify_source(&rom).is_err());
    let bytes = encode_indexed_graphics(&forged).unwrap(); let dataset = read_dataset(&prepared.directory).unwrap();
    let mut changed = dataset.manifest.clone(); changed.resources.iter_mut().find(|entry| entry.id == INDEXED_GRAPHICS_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&bytes);
    let mut files = dataset.files(); files.manifest = &changed; files.indexed_graphics = Some((&forged, &bytes));
    assert!(source.validate_files(files).is_err()); drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None); let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap(); let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap(); backing(&mut managed, &rom, true);
    let before = owner.indexed_graphics_stats().unwrap();
    for (start, length) in spans() { for at in start as usize..start as usize + length {
        assert_eq!(managed.bus.read8(0x0800_0000 + at as u32), rom[at], "source byte {at:x}");
    } }
    let after = owner.indexed_graphics_stats().unwrap();
    assert_eq!(after.source_bytes - before.source_bytes, 92304);
    assert_eq!(after.additional_literal_bytes - before.additional_literal_bytes, 24);
    assert_eq!(after.additional_prefix_bytes - before.additional_prefix_bytes, 900);
    for (i, spec) in extra::ARCHIVES.iter().enumerate() {
        assert_eq!(after.additional_archive_bytes[i] - before.additional_archive_bytes[i], spec.length as u64);
    }
    let owned = |at: usize| source.manifest.resources.iter().flat_map(|entry| &entry.sources)
        .any(|span| (span.offset as usize..span.offset as usize + span.length as usize).contains(&at));
    let mut changed = BTreeSet::new(); let mut edges = BTreeSet::new();
    for (start, length) in spans() {
        let start = start as usize; let end = start + length; edges.insert(start); edges.insert(end);
        for page in ((start + 4095) & !4095..end).step_by(4096) { edges.insert(page); }
    }
    for &edge in &edges { for at in edge.saturating_sub(3)..edge + 3 {
        if !owned(at) { changed.insert(at); managed.bus.rom[at] = rom[at] ^ 0x5a; }
    } }
    for edge in edges { for delta in [-3isize, -1, 0, 1] {
        let at = edge.checked_add_signed(delta).unwrap();
        let expected = |i: usize| if changed.contains(&(at + i)) { rom[at + i] ^ 0x5a } else { rom[at + i] };
        let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
        for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
            assert_eq!((RT_API.read16)(ptr, mirror + at as u32), u32::from(u16::from_le_bytes([expected(0), expected(1)])));
            assert_eq!((RT_API.read32)(ptr, mirror + at as u32), u32::from_le_bytes([expected(0), expected(1), expected(2), expected(3)]));
        }
    } }
    for at in changed { managed.bus.rom[at] = rom[at]; } backing(&mut managed, &rom, false);

    let mut visited = BTreeSet::new(); let mut native_cases: Vec<(Case, u32, Vec<u8>)> = Vec::new();
    for (i, archive) in additional.archives.iter().enumerate() {
        let case = Case::Archive(i);
        for machine in [&mut raw, &mut managed] { machine.load_state(&initial).unwrap(); setup(machine, case); }
        backing(&mut managed, &rom, true);
        interpret(&mut raw, RETURN, [0, 5, 10].contains(&i).then_some(&mut visited));
        interpret(&mut managed, RETURN, None);
        let expected = archive.selected_bytes(0, 0).unwrap(); let at = (DESTINATION - 0x0200_0000) as usize;
        assert_eq!(&raw.bus.ewram[at..at + expected.len()], expected);
        backing(&mut managed, &rom, false); let state = raw.save_state().unwrap();
        assert_eq!(managed.save_state().unwrap(), state, "original archive {i}");
        if [0, 5, 10].contains(&i) { native_cases.push((case, RETURN, state)); }
    }
    for language in 0..5u8 { for (group, metadata_index) in [(0, 2), (1, 2), (2, 2), (3, 2), (3, 45)] {
        let case = Case::Selector { group, language, metadata_index };
        for machine in [&mut raw, &mut managed] { machine.load_state(&initial).unwrap(); }
        let stop = setup(&mut raw, case); assert_eq!(setup(&mut managed, case), stop); backing(&mut managed, &rom, true);
        interpret(&mut raw, stop, Some(&mut visited)); interpret(&mut managed, stop, None);
        if group < 3 {
            let mut expected = extra::POINTERS[group as usize][language as usize];
            if group == 2 { expected += extra::EMBEDDED_PREFIX_LENGTH as u32; }
            assert_eq!(raw.cpu.regs[0], 0x0800_0000 + expected);
        } else {
            let at = extra::POINTERS[2][language as usize] as usize + 4 + usize::from(metadata_index - 2) * 4;
            assert_eq!(raw.cpu.regs[0], u32::from(rom[at] >> 4));
        }
        backing(&mut managed, &rom, false); let state = raw.save_state().unwrap();
        assert_eq!(managed.save_state().unwrap(), state, "selector group {group} language {language}");
        native_cases.push((case, stop, state));
    } }
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = visited.iter().map(|&(addr, thumb)| {
        let mut instrs = vec![if thumb { armv4t::decode_thumb(view.read16(addr), addr) } else { armv4t::decode_arm(view.read32(addr), addr) }];
        if thumb {
            let next = armv4t::decode_thumb(view.read16(addr + 2), addr + 2);
            if armv4t::fuse_thumb_bl(&instrs[0], &next).is_some() { instrs.push(next); }
        }
        Block { addr, thumb, instrs }
    }).collect();
    let object = root.join("additional-indexed-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    for (case, stop, expected) in &native_cases {
        managed.load_state(&initial).unwrap(); setup(&mut managed, *case); backing(&mut managed, &rom, true);
        for _ in 0..10_000_000 {
            if managed.cpu.regs[15] == *stop { break; }
            managed.bus.frame_ready = false;
            table.get(managed.cpu.regs[15] | u32::from(managed.cpu.thumb())).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void);
        }
        assert_eq!(managed.cpu.regs[15], *stop);
        backing(&mut managed, &rom, false); assert_eq!(&managed.save_state().unwrap(), expected);
    }
    let state = managed.save_state().unwrap(); let unavailable = root.join("temporarily unavailable dataset");
    fs::rename(&prepared.directory, &unavailable).unwrap(); assert!(attach_from_directory(&mut managed, &prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner)); managed.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner)); fs::rename(&unavailable, &prepared.directory).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[301,300], "resources":148, "new_source_bytes":92304,
        "groups":3, "archives":15, "programs":1588, "decoded_bytes":143872,
        "prior_other_resources_unchanged":147, "prior_representation_preserved":true,
        "poisoned_sources_and_mixed_boundaries":true, "original_full_decoder_cases":15,
        "original_native_decoder_cases":3, "original_interpreter_native_selector_cases":25,
        "full_state_clock_equal":true, "coherent_edit_rejected":true, "snapshot_owner_retained":true,
        "ordinary_gameplay":"pending", "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
