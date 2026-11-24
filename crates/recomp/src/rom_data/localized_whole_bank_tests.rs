//! Complete bounded EUR language-selected raw banks and original transfer paths.
use super::*;
use gba_core::{capi::RT_API, rom_data::ui_raw_transfers::dynamic_fragments as model, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

const STOP: u32 = 0x0300_7f00;
const TASK: u32 = 0x0201_0000;

fn backing(machine: &mut Machine, rom: &[u8], poison: bool) {
    for start in model::BANKS.into_iter().chain([0x71e6f8]) {
        let length = if start == 0x71e6f8 { 20 } else { model::BANK_LENGTH };
        for at in start as usize..start as usize + length {
            machine.bus.rom[at] = if poison { !rom[at] } else { rom[at] };
        }
    }
}

fn registers(machine: &mut Machine, pc: u32) {
    machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = STOP | 1;
    machine.cpu.regs[15] = pc;
    machine.cpu.cpsr = 0xbf;
}

fn setup(machine: &mut Machine, selector: u8, flag: bool, language: u8) {
    machine.bus.ffta_sprites.enabled = false;
    machine.bus.ewram.fill(0); machine.bus.iwram.fill(0); machine.bus.vram.fill(0xa5);
    machine.bus.write8(TASK + 2, selector);
    machine.bus.write16(TASK + 0x48, 1);
    machine.bus.write8(0x0300_28a0, if flag { 8 } else { 0 });
    machine.bus.write8(0x0200_1f6e, language);
    registers(machine, 0x080c_9060); machine.cpu.regs[0] = TASK;
}

fn expected_sources(rom: &[u8], selector: u8, flag: bool, language: u8) -> [u32; 2] {
    let table = if flag { 0x3996f4 } else { 0x39970c };
    let at = table + (usize::from(selector) - 4) * 4;
    let root = 0x71e6f8 + usize::from(language) * 4;
    let bank = u32::from_le_bytes(rom[root..root + 4].try_into().unwrap());
    std::array::from_fn(|i| bank + ((u16::from_le_bytes(rom[at + i * 2..at + i * 2 + 2].try_into().unwrap()) as u32) << 5))
}

fn check_queue(machine: &mut Machine, sources: [u32; 2]) {
    assert_eq!(machine.bus.read32(0x0300_2bcc), TASK + 0x10);
    for (i, src) in sources.into_iter().enumerate() {
        let node = TASK + 0x10 + 0x1c * i as u32;
        assert_eq!(machine.bus.read32(node + 4), src);
        assert_eq!(machine.bus.read32(node + 8), 0x0601_3800 + i as u32 * 0x40);
        assert_eq!(machine.bus.read32(node + 12), 0x8000_0020);
    }
    assert_eq!(machine.bus.read16(TASK + 0x48), 2);
}

fn check_transfer(machine: &Machine, rom: &[u8], sources: [u32; 2]) {
    for (i, src) in sources.into_iter().enumerate() {
        let at = (src - 0x0800_0000) as usize;
        assert_eq!(&machine.bus.vram[0x13800 + 64 * i..0x13840 + 64 * i], &rom[at..at + 64]);
    }
}

fn interpret(machine: &mut Machine, visited: &mut BTreeSet<(u32, bool)>) {
    let mut steps = 0;
    while machine.cpu.regs[15] != STOP {
        assert!(steps < 100_000, "interpreter pc {:08x}", machine.cpu.regs[15]);
        assert!(machine.cpu.thumb()); assert_eq!(machine.cpu.regs[15] >> 24, 8);
        visited.insert((machine.cpu.regs[15], true)); machine.bus.frame_ready = false;
        machine.step(); steps += 1;
    }
}

#[test]
#[ignore = "exact EUR ROM and E298 predecessor; Wine only, Linux/USA pending"]
fn european_five_complete_raw_banks_and_original_transfers() {
    let root = PathBuf::from(std::env::var_os("FFTA_WHOLE_UI_BANK_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_WHOLE_UI_BANK_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision, old.resources.len()), (298, 297, 148));
    let mut source = Extraction::from_rom(&rom).unwrap();
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision, source.manifest.resources.len()), (299, 298, 148));
    for prior in &old.resources {
        let current = source.manifest.resources.iter().find(|entry| entry.id == prior.id).unwrap();
        if prior.id != DYNAMIC_RAW_FRAGMENTS_ID {
            assert_eq!(current, prior);
            assert_eq!(fs::read(previous.join(&prior.path)).unwrap(), fs::read(prepared.directory.join(&current.path)).unwrap());
        } else {
            let old_data = parse_dynamic_raw_fragments(&fs::read(previous.join(&prior.path)).unwrap()).unwrap();
            let mut new_data = parse_dynamic_raw_fragments(&fs::read(prepared.directory.join(&current.path)).unwrap()).unwrap();
            assert!(old_data.bank_starts.is_empty()); assert_eq!(new_data.bank_starts.len(), 5); new_data.bank_starts.clear();
            assert_eq!(new_data, old_data);
            assert_eq!(current.sources.len(), prior.sources.len() + 5);
            assert_eq!(&current.sources[..prior.sources.len()], &prior.sources);
            assert_eq!(current.sources[prior.sources.len()..].iter().map(|s| s.length).sum::<u32>(), 1280);
            let mut metadata = current.clone(); metadata.sources.truncate(prior.sources.len());
            metadata.format = prior.format.clone(); metadata.sha256 = prior.sha256.clone(); assert_eq!(&metadata, prior);
        }
    }
    let data = &source.dynamic_raw_fragments.as_ref().unwrap().decoded;
    data.verify_source(&rom).unwrap();
    for (bank, &expected) in data.bank_starts.iter().zip(&model::BANKS) {
        assert_eq!((bank.bank_offset, bank.relative_offset, bank.words.len()), (expected, 0, 64));
        assert_eq!(bank.source_bytes(), &rom[expected as usize..expected as usize + model::BANK_START_LENGTH]);
    }
    let mut bad = data.clone(); bad.bank_starts[0].relative_offset = 4; assert!(bad.validate().is_err());
    let mut forged = data.clone(); forged.bank_starts[0].words[0] ^= 1; forged.validate().unwrap(); assert!(forged.verify_source(&rom).is_err());
    let forged_bytes = encode_dynamic_raw_fragments(&forged).unwrap(); let dataset = read_dataset(&prepared.directory).unwrap();
    let mut altered = dataset.manifest.clone(); altered.resources.iter_mut().find(|e| e.id == DYNAMIC_RAW_FRAGMENTS_ID).unwrap().sha256 = gba_core::ffta_rom::digest(&forged_bytes);
    let mut files = dataset.files(); files.manifest = &altered; files.dynamic_raw_fragments = Some((&forged, &forged_bytes));
    assert!(source.validate_files(files).is_err(), "coherent modified prefix and hash must be rejected"); drop(dataset);

    let mut raw = crate::make_machine(rom.clone(), None); let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap(); let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap(); backing(&mut managed, &rom, true);
    for start in model::BANKS { for at in start as usize..start as usize + model::BANK_LENGTH {
        for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] { assert_eq!(managed.bus.read8(mirror + at as u32), rom[at]); }
    } }
    assert_eq!(owner.dynamic_raw_fragments_stats().unwrap().bank_start_bytes, [768; 5]);
    let owned = |at: usize| source.manifest.resources.iter().flat_map(|entry| &entry.sources)
        .any(|span| (span.offset as usize..span.offset as usize + span.length as usize).contains(&at));
    let mut changed = BTreeSet::new();
    let edges: Vec<_> = model::BANKS.iter().flat_map(|&start| [start as usize, start as usize + 0x100, start as usize + 0x200, start as usize + model::BANK_LENGTH]).chain([0x487000]).collect();
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

    let cases: Vec<_> = (4..=6u8).flat_map(|selector| [false, true].into_iter().flat_map(move |flag| (0..5u8).map(move |language| (selector, flag, language)))).collect();
    let mut visited = BTreeSet::new(); let mut expected_states = Vec::new(); let mut selected_offsets = BTreeSet::new();
    for &(selector, flag, language) in &cases {
        raw.load_state(&initial).unwrap(); setup(&mut raw, selector, flag, language);
        interpret(&mut raw, &mut visited); let sources = expected_sources(&rom, selector, flag, language);
        check_queue(&mut raw, sources);
        for src in sources { let offset = src - (0x0800_0000 + model::BANKS[language as usize]); assert!(offset < 0x100); selected_offsets.insert(offset); }
        registers(&mut raw, 0x0800_0800); interpret(&mut raw, &mut visited); check_transfer(&raw, &rom, sources);
        expected_states.push(raw.save_state().unwrap());
    }
    assert_eq!(selected_offsets, BTreeSet::from([0, 0x40, 0x80, 0xc0]));
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = visited.iter().map(|&(addr, thumb)| {
        let mut instrs = vec![armv4t::decode_thumb(view.read16(addr), addr)];
        let next = armv4t::decode_thumb(view.read16(addr + 2), addr + 2);
        if armv4t::fuse_thumb_bl(&instrs[0], &next).is_some() { instrs.push(next); }
        Block { addr, thumb, instrs }
    }).collect();
    let object = root.join("localized-whole-bank-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut translated = crate::make_machine(rom.clone(), None); attach_from_directory(&mut translated, &prepared.directory).unwrap();
    let before_typed = owner.dynamic_raw_fragments_stats().unwrap().bank_start_bytes;
    let before_native = translated.bus.rom_resources().unwrap().dynamic_raw_fragments_stats().unwrap().bank_start_bytes;
    for (i, &(selector, flag, language)) in cases.iter().enumerate() {
        let sources = expected_sources(&rom, selector, flag, language);
        for (native, machine) in [(false, &mut managed), (true, &mut translated)] {
            machine.load_state(&initial).unwrap(); setup(machine, selector, flag, language); backing(machine, &rom, true);
            for phase in 0..2 {
                if phase == 1 { check_queue(machine, sources); registers(machine, 0x0800_0800); }
                if native {
                    let mut dispatches = 0;
                    while machine.cpu.regs[15] != STOP {
                        assert!(dispatches < 100_000, "native case {i} pc {:08x}", machine.cpu.regs[15]); machine.bus.frame_ready = false;
                        table.get(machine.cpu.regs[15] | u32::from(machine.cpu.thumb())).unwrap_or_else(|| panic!("uncompiled pc {:08x}", machine.cpu.regs[15]))(&RT_API, machine as *mut Machine as *mut std::ffi::c_void);
                        dispatches += 1;
                    }
                } else { interpret(machine, &mut BTreeSet::new()); }
            }
            check_transfer(machine, &rom, sources); backing(machine, &rom, false);
            assert_eq!(machine.save_state().unwrap(), expected_states[i], "state/clock case {i}, native={native}");
        }
    }
    let after_typed = owner.dynamic_raw_fragments_stats().unwrap().bank_start_bytes;
    let after_native = translated.bus.rom_resources().unwrap().dynamic_raw_fragments_stats().unwrap().bank_start_bytes;
    assert!((0..5).all(|i| after_typed[i] > before_typed[i] && after_native[i] > before_native[i]));
    let state = managed.save_state().unwrap(); let unavailable = root.join("temporarily unavailable dataset");
    fs::rename(&prepared.directory, &unavailable).unwrap(); assert!(attach_from_directory(&mut managed, &prepared.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner)); managed.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner)); fs::rename(&unavailable, &prepared.directory).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[299,298], "resources":148, "new_source_bytes":1280,
        "whole_banks":5, "whole_bank_bytes":4480, "prior_other_resources_unchanged":147,
        "prior_representation_preserved":true, "poisoned_mirrors_and_mixed_boundaries":true,
        "original_selector_values":[4,5,6], "original_flag_branches":2, "languages":5,
        "original_interpreter_generated_cases":cases.len(), "all_prefix_quarters_used":true,
        "full_state_clock_equal":true, "coherent_edit_rejected":true, "missing_dataset_and_snapshot_owner_retained":true,
        "ordinary_gameplay":"pending", "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
