//! Direct EUR typed text/font reads, original/native parity and owner lifetime.
use super::*;
use gba_core::{capi::RT_API, Bus, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{cell::Cell, collections::BTreeSet, sync::Arc};
const RETURN: u32 = 0x08000300;
const STACK: u32 = 0x03007c00;
thread_local! { static NATIVE_BYTES: Cell<[u64;4]> = const { Cell::new([0;4]) }; }
fn stats(machine: *mut std::ffi::c_void) -> [u64; 4] {
    let s = unsafe {
        (&*(machine as *const Machine))
            .bus
            .rom_resources()
            .unwrap()
            .text_stats()
            .unwrap()
    };
    [s.table_bytes, s.string_bytes, s.glyph_bytes, s.width_bytes]
}
fn count(before: [u64; 4], after: [u64; 4]) {
    NATIVE_BYTES.with(|v| {
        let mut n = v.get();
        for i in 0..4 {
            n[i] += after[i] - before[i];
        }
        v.set(n);
    });
}
extern "C" fn read8(m: *mut std::ffi::c_void, a: u32) -> u32 {
    let b = stats(m);
    let r = (RT_API.read8)(m, a);
    count(b, stats(m));
    r
}
extern "C" fn read16(m: *mut std::ffi::c_void, a: u32) -> u32 {
    let b = stats(m);
    let r = (RT_API.read16)(m, a);
    count(b, stats(m));
    r
}
extern "C" fn read32(m: *mut std::ffi::c_void, a: u32) -> u32 {
    let b = stats(m);
    let r = (RT_API.read32)(m, a);
    count(b, stats(m));
    r
}
#[derive(Clone, Copy)]
struct Call {
    entry: u32,
    args: [u32; 3],
}
fn setup(machine: &mut Machine, call: Call) {
    machine.cpu.regs[..3].copy_from_slice(&call.args);
    machine.cpu.regs[13] = STACK;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = call.entry;
    machine.cpu.cpsr = 0xbf;
}
fn interpreted(machine: &mut Machine, call: Call, visited: &mut BTreeSet<(u32, bool)>) -> usize {
    setup(machine, call);
    for steps in 1..=500_000 {
        visited.insert((machine.cpu.regs[15], machine.cpu.thumb()));
        machine.bus.frame_ready = false;
        machine.step();
        if machine.cpu.regs[15] == RETURN {
            assert_eq!(machine.cpu.regs[13], STACK);
            return steps;
        }
    }
    panic!(
        "original text/font call did not return at{:08x}",
        machine.cpu.regs[15]
    );
}
fn ranges(machine: &mut Machine, sources: &[gba_core::rom_data::SourceRange], rom: Option<&[u8]>) {
    for source in sources {
        let at = source.offset as usize;
        let end = at + source.length as usize;
        if let Some(rom) = rom {
            machine.bus.rom[at..end].copy_from_slice(&rom[at..end]);
        } else {
            machine.bus.rom[at..end].fill(0xcd);
        }
    }
}

#[test]
#[ignore = "exact EUR ROM/e11 dataset/retained initialized slot/TinyCC and absent FFTA_STATE_TEST_TEXT_RUNTIME_ROOT required"]
fn european_original_and_native_texts_consume_typed_codes_glyphs_and_metrics() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_TEXT_RUNTIME_ROOT").unwrap());
    assert!(!root.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let files = read_dataset(&dataset).unwrap();
    let values = files.short_texts.as_ref().unwrap();
    let sources = &files.manifest.resources[7].sources;
    let mut tracer = crate::make_machine(rom.clone(), None);
    crate::save_state::load(
        &mut tracer,
        Path::new(&std::env::var_os("FFTA_STATE_TEST_STATE").unwrap()),
    )
    .unwrap();
    let start = tracer.save_state().unwrap();
    let iwram = tracer.bus.iwram.clone();
    let mut visited = BTreeSet::new();
    let mut scripts = Vec::new();
    let mut instructions = 0;
    // One selected reader chain per language/page, alternating source encoding
    // and fixed/variable width. Closed all535-string offline oracle is not rerun.
    for language in &values.languages {
        let encoding = if language.language_id % 2 == 0 {
            gba_core::rom_data::text_short::Encoding::Compact
        } else {
            gba_core::rom_data::text_short::Encoding::Extended
        };
        let (index, &id) = language
            .entries
            .iter()
            .enumerate()
            .find(|(_, id)| values.strings[**id as usize].encoding == encoding)
            .unwrap();
        let text = &values.strings[id as usize];
        let glyph = text
            .units
            .iter()
            .find_map(|u| {
                if let gba_core::rom_data::text_short::Unit::Glyph { index } = u {
                    Some(*index)
                } else {
                    None
                }
            })
            .unwrap();
        let calls = vec![
            Call {
                entry: 0x08016648,
                args: [index as u32, 0, 0],
            },
            Call {
                entry: 0x08019d98,
                args: [0x02030010, 0x08000000 + text.source_offset, 0],
            },
            Call {
                entry: 0x080135a4,
                args: [0x02030210, glyph as u32, 0],
            },
            Call {
                entry: 0x08013bb4,
                args: [glyph as u32, 0, 0],
            },
        ];
        tracer.load_state(&start).unwrap();
        tracer.bus.write8(0x02001f6e, language.language_id);
        tracer.bus.write8(0x020020e3, language.language_id % 2);
        for &call in &calls {
            instructions += interpreted(&mut tracer, call, &mut visited);
        }
        scripts.push((language.language_id, index, calls));
    }
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: Some(&iwram),
        bios: None,
    };
    let blocks = visited
        .iter()
        .map(|&(address, thumb)| {
            let mut instrs = vec![if thumb {
                armv4t::decode_thumb(view.read16(address), address)
            } else {
                armv4t::decode_arm(view.read32(address), address)
            }];
            if thumb {
                let next = armv4t::decode_thumb(view.read16(address + 2), address + 2);
                if armv4t::fuse_thumb_bl(&instrs[0], &next).is_some() {
                    instrs.push(next);
                }
            }
            Block {
                addr: address,
                thumb,
                instrs,
            }
        })
        .collect();
    fs::create_dir_all(&root).unwrap();
    let object = root.join("text-consumers.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut legacy = crate::make_machine(rom.clone(), None);
    let mut decoded = crate::make_machine(rom.clone(), None);
    let mut native = crate::make_machine(rom.clone(), None);
    for target in [&mut decoded, &mut native] {
        attach_from_directory(target, &dataset).unwrap();
    }
    let api = gba_core::capi::RtApi {
        read8,
        read16,
        read32,
        ..RT_API
    };
    NATIVE_BYTES.with(|v| v.set([0; 4]));
    let mut dispatches = 0;
    let mut calls = 0;
    let mut reports = Vec::new();
    for (language, index, script) in scripts {
        for target in [&mut legacy, &mut decoded, &mut native] {
            target.load_state(&start).unwrap();
            target.bus.write8(0x02001f6e, language);
            target.bus.write8(0x020020e3, language % 2);
        }
        let before = NATIVE_BYTES.with(Cell::get);
        for call in script {
            interpreted(&mut legacy, call, &mut BTreeSet::new());
            ranges(&mut decoded, sources, None);
            ranges(&mut native, sources, None);
            interpreted(&mut decoded, call, &mut BTreeSet::new());
            setup(&mut native, call);
            let mut blocks = 0;
            while native.cpu.regs[15] != RETURN {
                assert!(blocks < 500_000);
                native.bus.frame_ready = false;
                table
                    .get(native.cpu.regs[15] | u32::from(native.cpu.thumb()))
                    .expect("original text/font ROM/IWRAM instruction compiled")(
                    &api,
                    &mut native as *mut Machine as *mut std::ffi::c_void,
                );
                blocks += 1;
            }
            calls += 1;
            dispatches += blocks;
            for target in [&mut decoded, &mut native] {
                ranges(target, sources, Some(&rom));
                assert_eq!(
                    legacy.save_state().unwrap(),
                    target.save_state().unwrap(),
                    "language{language} call{:08x} full state/clock",
                    call.entry
                );
            }
        }
        let after = NATIVE_BYTES.with(Cell::get);
        let used: Vec<_> = (0..4).map(|i| after[i] - before[i]).collect();
        assert!(
            used[..3].iter().all(|&v| v > 0),
            "compiled table/string/glyph loads"
        );
        if language % 2 == 0 {
            assert!(used[3] > 0, "compiled variable-width metric loads");
        } else {
            assert_eq!(used[3], 0, "original fixed-width mode bypasses metrics");
        }
        reports.push(serde_json::json!({"language":language,"entry_index":index,"fixed_width":language%2,"compiled_table_string_glyph_width_bytes":used}));
    }
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"region":"europe","cases":reports,"poisoned_sources":sources.len(),"original_instructions":instructions,
        "calls":calls,"native_dispatches":dispatches,"compiled_table_string_glyph_width_bytes":NATIVE_BYTES.with(Cell::get),
        "all_states_and_clocks_equal":true,"linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
    eprintln!("SHORT_TEXT_RUNTIME cases=5 calls={calls} instructions={instructions} native_dispatches={dispatches} compiled_table/string/glyph/width={:?} full_states=true",NATIVE_BYTES.with(Cell::get));
}

#[test]
#[ignore = "requires exact EUR ROM/e11 dataset; only temporary copied files are renamed"]
fn european_text_owner_survives_files_slots_undo_backup_and_restart() {
    let temporary = tempfile::tempdir().unwrap();
    let dataset = temporary.path().join("dataset");
    fs::create_dir(&dataset).unwrap();
    let original = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    for name in [
        MANIFEST_FILE,
        OBJ_DIMENSIONS_FILE,
        BG_DESCRIPTORS_FILE,
        BG_BASE_PALETTES_FILE,
        BG_ALTERNATE_PALETTES_FILE,
        BG_PACKET_PROGRAMS_FILE,
        BG_MAIN_GRAPHICS_FILE,
        BG_TRANSFER_PROGRAMS_FILE,
        SHORT_TEXTS_FILE,
    ] {
        fs::copy(original.join(name), dataset.join(name)).unwrap();
    }
    let files = read_dataset(&dataset).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let mut machine = crate::make_machine(rom.clone(), None);
    let boot = machine.save_state().unwrap();
    attach_from_directory(&mut machine, &dataset).unwrap();
    let owner = machine.bus.rom_resources().unwrap().clone();
    assert_eq!(
        owner.text_file_sha256(),
        Some(files.manifest.resources[7].sha256.as_str())
    );
    let sources = &files.manifest.resources[7].sources;
    let text_values = files.short_texts.as_ref().unwrap();
    let addresses = [
        text_values.languages[0].table_offset,
        text_values.strings[0].source_offset,
        text_values.glyphs[0].source_offset(),
        text_values.glyphs[0].width_offset(),
    ];
    for address in addresses {
        machine.bus.read8(0x08000000 + address);
    }
    let stats = owner.text_stats().unwrap();
    assert!(
        stats.table_bytes > 0
            && stats.string_bytes > 0
            && stats.glyph_bytes > 0
            && stats.width_bytes > 0
    );
    fs::rename(&dataset, temporary.path().join("retained-files")).unwrap();
    let state = machine.save_state().unwrap();
    assert!(attach_from_directory(&mut machine, &dataset).is_err());
    assert_eq!(machine.save_state().unwrap(), state);
    assert!(Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
    let save = temporary.path().join("test.sav");
    let mut slots = crate::save_state::Slots::new(&save, &machine.bus.rom);
    slots
        .save(&machine, 1, &vec![[0; 4]; 240 * 160], false)
        .unwrap();
    machine.cpu.regs[4] = 99;
    let changed = machine.save_state().unwrap();
    slots.load(&mut machine, 1).unwrap();
    assert_eq!(machine.cpu.regs[4], 0);
    slots.undo(&mut machine).unwrap();
    assert_eq!(machine.save_state().unwrap(), changed);
    let length = machine.bus.save_data().unwrap().len();
    fs::write(&save, vec![7; length]).unwrap();
    let backups = crate::save_backups::Backups::new(&save);
    backups.capture_startup().unwrap();
    backups
        .restore(
            backups.choices(length)[0].generation.unwrap(),
            &mut machine,
            &boot,
        )
        .unwrap();
    assert_eq!(machine.bus.save_data().unwrap(), vec![7; length]);
    machine.bus.load_save_data(&vec![9; length]);
    crate::save_backups::restart(&mut machine, &boot, &save).unwrap();
    assert_eq!(machine.bus.save_data().unwrap(), vec![9; length]);
    assert_eq!(fs::read(&save).unwrap(), vec![9; length]);
    assert!(Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
    assert_eq!(owner.text_stats().unwrap(), stats);
    ranges(&mut machine, sources, None);
    for address in addresses {
        assert_eq!(
            machine.bus.read8(0x08000000 + address),
            rom[address as usize]
        );
    }
    ranges(&mut machine, sources, Some(&rom));
    eprintln!("SHORT_TEXT_LIFECYCLE copied_files_unavailable=true transactional_activation=true slots_undo_backup_restart=true owner_retained=true typed_tables_strings_glyphs_widths=true");
}
