//! Whole EUR localized queued catalog and the original selector/DMA paths.
use super::*;
use gba_core::{capi::RT_API, rom_data::ui_raw_transfers::localized_queued as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

const NODE: u32 = 0x0203_0000;
const DESTINATION: u32 = 0x0601_0306;
const STOP: u32 = 0x0300_7f00;

fn spans() -> Vec<(u32, usize)> {
    [(model::POINTERS, 120), (model::LENGTHS, 120), (model::POINTER_LITERAL, 4), (model::LENGTH_LITERAL, 4)]
        .into_iter().chain(model::BANKS.into_iter().map(|at| (at, model::BANK_BYTES))).collect()
}

fn backing(machine: &mut Machine, rom: &[u8], poison: bool) {
    for (start, length) in spans() {
        let at = start as usize;
        for i in at..at + length { machine.bus.rom[i] = if poison { !rom[i] } else { rom[i] }; }
    }
}

fn setup(machine: &mut Machine, language: u8, selector: u8) {
    machine.bus.ffta_sprites.enabled = false;
    machine.bus.ewram.fill(0); machine.bus.iwram.fill(0); machine.bus.vram.fill(0xa5);
    machine.bus.write8(0x0200_1f6e, language);
    machine.bus.write32(0x0300_08c4, DESTINATION - 0x306);
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[0] = NODE;
    machine.cpu.regs[5] = 0x0200_1f6e;
    machine.cpu.regs[6] = u32::from(selector);
    machine.cpu.regs[13] = 0x0300_7d00;
    machine.cpu.regs[15] = 0x0809_279c;
    machine.cpu.cpsr = 0xbf;
}

fn interpret(machine: &mut Machine, stop: u32) {
    for _ in 0..100_000 {
        if machine.cpu.regs[15] == stop { return; }
        machine.bus.frame_ready = false; machine.step();
    }
    panic!("localized queue path escaped at {:08x}", machine.cpu.regs[15]);
}

fn source(language: u8, selector: u8) -> u32 {
    0x0800_0000 + model::BANKS[usize::from(language)] + u32::from(selector) * model::ENTRY_BYTES as u32
}

fn check_queue(machine: &mut Machine, language: u8, selector: u8) {
    assert_eq!(machine.bus.read32(0x0300_2bcc), NODE);
    assert_eq!(machine.bus.read32(NODE + 4), source(language, selector));
    assert_eq!(machine.bus.read32(NODE + 8), DESTINATION);
    assert_eq!(machine.bus.read32(NODE + 12), 0x8000_0018);
    assert_eq!(machine.bus.read16(NODE + 16), 4);
    assert_eq!(machine.bus.read16(NODE + 18), 48);
    assert_eq!(machine.bus.read16(NODE + 20), 64);
}

fn execute_queue(machine: &mut Machine, rom: &[u8], language: u8, selector: u8) {
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[13] = 0x0300_7d00; machine.cpu.regs[14] = STOP | 1;
    machine.cpu.regs[15] = 0x0800_0800; machine.cpu.cpsr = 0xbf;
    interpret(machine, STOP);
    assert_eq!(machine.bus.read32(0x0300_2bcc), 0);
    let at = (source(language, selector) - 0x0800_0000) as usize;
    for row in 0..4 {
        let dest = (DESTINATION - 0x0600_0000) as usize + row * 64;
        assert_eq!(&machine.bus.vram[dest..dest + 48], &rom[at + row * 48..at + row * 48 + 48]);
    }
}

#[test]
#[ignore = "exact EUR ROM and E299 predecessor; Wine only, Linux/USA pending"]
fn european_thirty_localized_queued_resources_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_LOCALIZED_QUEUED_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_LOCALIZED_QUEUED_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (299, 298, 148));
    let mut extraction = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut extraction, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut extraction, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((extraction.manifest.extractor_revision, extraction.manifest.decoder_revision, extraction.manifest.resources.len()), (300, 299, 148));
    for prior in &old.resources {
        let current = extraction.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        if prior.id != DYNAMIC_RAW_FRAGMENTS_ID {
            assert_eq!(current, prior);
            assert_eq!(fs::read(previous.join(&prior.path)).unwrap(), fs::read(prepared.directory.join(&current.path)).unwrap());
        } else {
            let old_data = parse_dynamic_raw_fragments(&fs::read(previous.join(&prior.path)).unwrap()).unwrap();
            let mut new_data = parse_dynamic_raw_fragments(&fs::read(prepared.directory.join(&current.path)).unwrap()).unwrap();
            assert!(old_data.localized_queued.is_none()); assert!(new_data.localized_queued.take().is_some());
            assert_eq!(new_data, old_data);
            assert_eq!(current.sources.len(), prior.sources.len() + 9);
            assert_eq!(&current.sources[..prior.sources.len()], &prior.sources);
            assert_eq!(current.sources[prior.sources.len()..].iter().map(|s| s.length).sum::<u32>(), 6008);
            let mut metadata = current.clone(); metadata.sources.truncate(prior.sources.len());
            metadata.format = prior.format.clone(); metadata.sha256 = prior.sha256.clone(); assert_eq!(&metadata, prior);
        }
    }
    let data = &extraction.dynamic_raw_fragments.as_ref().unwrap().decoded;
    let catalog = data.localized_queued.as_ref().unwrap();
    assert_eq!((catalog.pointers.len(), catalog.lengths.len(), catalog.banks.len()), (30, 30, 5));
    let mut invalid = data.clone(); invalid.localized_queued.as_mut().unwrap().lengths[0] = 191; assert!(invalid.validate().is_err());
    let mut forged = data.clone(); forged.localized_queued.as_mut().unwrap().banks[0].words[0] ^= 1;
    forged.validate().unwrap(); assert!(forged.verify_source(&rom).is_err());
    let forged_bytes = encode_dynamic_raw_fragments(&forged).unwrap(); let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered = dataset.manifest.clone(); altered.resources.iter_mut().find(|e| e.id == DYNAMIC_RAW_FRAGMENTS_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&forged_bytes);
    let mut files = dataset.files(); files.manifest = &altered; files.dynamic_raw_fragments = Some((&forged, &forged_bytes));
    assert!(extraction.validate_files(files).is_err()); drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None); let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap(); let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap(); backing(&mut managed, &rom, true);
    for (start, length) in spans() { for at in start as usize..start as usize + length {
        for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] { assert_eq!(managed.bus.read8(mirror + at as u32), rom[at]); }
    } }
    assert_eq!(owner.dynamic_raw_fragments_stats().unwrap().localized_queued_bank_bytes, [3456; 5]);
    let owned = |at: usize| extraction.manifest.resources.iter().flat_map(|entry| &entry.sources)
        .any(|span| (span.offset as usize..span.offset as usize + span.length as usize).contains(&at));
    let mut changed = BTreeSet::new();
    let edges: Vec<_> = spans().into_iter().flat_map(|(start, length)| [start as usize, start as usize + length])
        .chain([0x398000, 0x439000, 0x44a000, 0x46d000]).collect();
    for &edge in &edges { for at in edge.saturating_sub(3)..edge + 3 {
        if !owned(at) { changed.insert(at); managed.bus.rom[at] = rom[at] ^ 0x5a; }
    } }
    for edge in edges { for delta in [-3isize, -1, 0, 1] {
        let at = edge.checked_add_signed(delta).unwrap();
        let expected = |i: usize| if changed.contains(&(at + i)) { rom[at + i] ^ 0x5a } else { rom[at + i] };
        let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
        assert_eq!((RT_API.read16)(ptr, 0x0a00_0000 + at as u32), u32::from(u16::from_le_bytes([expected(0), expected(1)])));
        assert_eq!((RT_API.read32)(ptr, 0x0800_0000 + at as u32), u32::from_le_bytes([expected(0), expected(1), expected(2), expected(3)]));
    } }
    for at in changed { managed.bus.rom[at] = rom[at]; } backing(&mut managed, &rom, false);

    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = (0x0809_279c..0x0809_27de).step_by(2).map(|addr| Block {
        addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
    }).collect();
    let object = root.join("localized-queued-selector-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let before = owner.dynamic_raw_fragments_stats().unwrap().localized_queued_bank_bytes;
    for language in 0..5u8 { for selector in 0..6u8 {
        for native in [false, true] {
            for machine in [&mut raw, &mut managed] { machine.load_state(&initial).unwrap(); setup(machine, language, selector); }
            backing(&mut managed, &rom, true); interpret(&mut raw, 0x0809_27de);
            if native {
                for _ in 0..100 {
                    if managed.cpu.regs[15] == 0x0809_27de { break; }
                    managed.bus.frame_ready = false;
                    table.get(managed.cpu.regs[15] | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void);
                }
                assert_eq!(managed.cpu.regs[15], 0x0809_27de);
            } else { interpret(&mut managed, 0x0809_27de); }
            assert_eq!(raw.cpu.regs[1], source(language, selector));
            backing(&mut managed, &rom, false);
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "selector {language}/{selector}, native={native}");
        }
        for machine in [&mut raw, &mut managed] { machine.load_state(&initial).unwrap(); setup(machine, language, selector); }
        backing(&mut managed, &rom, true);
        for machine in [&mut raw, &mut managed] {
            interpret(machine, 0x0809_27e2); check_queue(machine, language, selector);
            execute_queue(machine, &rom, language, selector);
        }
        backing(&mut managed, &rom, false);
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "queued transfer {language}/{selector}");
    } }
    let after = owner.dynamic_raw_fragments_stats().unwrap().localized_queued_bank_bytes;
    assert!((0..5).all(|i| after[i] >= before[i] + 6 * model::ENTRY_BYTES as u64));
    let state = managed.save_state().unwrap(); let unavailable = root.join("temporarily unavailable dataset");
    fs::rename(&prepared.directory, &unavailable).unwrap(); assert!(attach_from_directory(&mut managed, &prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner)); managed.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner)); fs::rename(&unavailable, &prepared.directory).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[300,299], "resources":148, "new_source_bytes":6008,
        "languages":5, "entries_per_language":6, "entry_bytes":192, "prior_other_resources_unchanged":147,
        "prior_representation_preserved":true, "poisoned_mirrors_and_mixed_boundaries":true,
        "original_interpreter_generated_selector_cases":30, "original_selector_queue_dma_cases":30,
        "full_state_clock_equal":true, "coherent_edit_rejected":true, "missing_dataset_and_snapshot_owner_retained":true,
        "ordinary_gameplay":"pending", "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
