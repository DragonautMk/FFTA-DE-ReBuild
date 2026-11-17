//! EUR text substitution callback source and original dispatcher prefix.
use super::*;
use gba_core::{Bus, Machine, capi::RT_API};
use gba_core::rom_data::text_catalogs::{SUBSTITUTION_ROOT, SUBSTITUTION_SOURCE_SHA256};
use recomp_core::analyze::{Analysis, Block, View};

#[test]
#[ignore = "exact EUR ROM, e159 dataset and TinyCC; Wine only, Linux/USA pending"]
fn european_text_substitution_callbacks_publication_and_reader() {
    let root = PathBuf::from(std::env::var_os("FFTA_SUBSTITUTION_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_SUBSTITUTION_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (159, 158));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision), (160, 159));
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!(source.manifest.coverage, old.coverage);
    for old_resource in &old.resources {
        let new = source.manifest.resources.iter().find(|r| r.id == old_resource.id).unwrap();
        let old_bytes = fs::read(previous.join(&old_resource.path)).unwrap();
        let new_bytes = fs::read(prepared.directory.join(&new.path)).unwrap();
        if old_resource.id != TEXT_CATALOGS_ID {
            assert_eq!(new, old_resource);
            assert_eq!(new_bytes, old_bytes);
            continue;
        }
        let old_json: serde_json::Value = serde_json::from_slice(&old_bytes).unwrap();
        let mut new_json: serde_json::Value = serde_json::from_slice(&new_bytes).unwrap();
        assert_eq!(new_json.as_object_mut().unwrap().remove("substitution_callbacks").unwrap().as_array().unwrap().len(), 75);
        assert_eq!(new_json, old_json);
        let mut metadata = new.clone();
        let added: Vec<_> = metadata.sources.iter().filter(|s| !old_resource.sources.contains(s)).cloned().collect();
        assert_eq!(added.len(), 75);
        assert_eq!(added.iter().map(|s| s.length).sum::<u32>(), 300);
        metadata.sources.retain(|s| old_resource.sources.contains(s));
        metadata.sha256 = old_resource.sha256.clone();
        assert_eq!(&metadata, old_resource);
    }
    let catalog = &source.text_catalogs.as_ref().unwrap().decoded;
    let callbacks = &catalog.substitution_callbacks;
    assert_eq!(callbacks.len(), 75);
    assert_eq!(callbacks.iter().filter(|c| c.target == 0).count(), 4);
    let bytes: Vec<_> = callbacks.iter().flat_map(|c| c.target.to_le_bytes()).collect();
    assert_eq!(gba_core::ffta_rom::digest(&bytes), SUBSTITUTION_SOURCE_SHA256);
    for callback in callbacks {
        let at = callback.source_offset() as usize;
        assert_eq!(callback.target.to_le_bytes(), rom[at..at+4]);
    }
    let mut invalid = catalog.clone();
    invalid.substitution_callbacks[1].index = invalid.substitution_callbacks[0].index;
    assert!(invalid.validate().is_err());
    let mut edited = catalog.clone();
    edited.substitution_callbacks[0].target ^= 1;
    edited.validate().unwrap();
    assert!(edited.verify_source(&rom).is_err());

    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    for callback in callbacks {
        let at = callback.source_offset() as usize;
        managed.bus.rom[at..at+4].fill(0xcd);
    }
    let before = owner.text_catalog_stats().unwrap().substitution_callback_bytes;
    for callback in callbacks {
        for at in callback.source_offset()..callback.source_offset()+4 {
            assert_eq!(managed.bus.read8(0x0c00_0000 + at), rom[at as usize]);
        }
    }
    assert_eq!(owner.text_catalog_stats().unwrap().substitution_callback_bytes - before, 300);
    let unowned = (SUBSTITUTION_ROOT + 4) as usize;
    assert!(!callbacks.iter().any(|c| c.index == 1));
    managed.bus.rom[unowned] ^= 0x80;
    assert_eq!(managed.bus.read8(0x0800_0000 + unowned as u32), rom[unowned] ^ 0x80);
    managed.bus.rom[unowned] = rom[unowned];
    let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for at in [SUBSTITUTION_ROOT - 1, SUBSTITUTION_ROOT + 3, SUBSTITUTION_ROOT + 4] {
        let i = at as usize;
        assert_eq!((RT_API.read16)(ptr, 0x0a00_0000 + at), u32::from(u16::from_le_bytes(rom[i..i+2].try_into().unwrap())));
        assert_eq!((RT_API.read32)(ptr, 0x0800_0000 + at), u32::from_le_bytes(rom[i..i+4].try_into().unwrap()));
    }
    for callback in callbacks {
        let at = callback.source_offset() as usize;
        managed.bus.rom[at..at+4].copy_from_slice(&rom[at..at+4]);
    }
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = (0x0801_9e94..0x0801_9ea6).step_by(2)
        .map(|addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("text-substitution.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let initial = raw.save_state().unwrap();
    let mut cases = 0;
    for native in [false, true] { for index in [0u32, 19, 28, 85, 129, 136, 1, 255, 0x100] {
        for m in [&mut raw, &mut managed] {
            m.load_state(&initial).unwrap();
            m.bus.ffta_sprites.enabled = false;
            m.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
            m.cpu.regs[0] = 0x0201_0000;
            m.cpu.regs[1] = index;
            m.cpu.regs[13] = 0x0300_7c00;
            m.cpu.regs[14] = 0x0800_0301;
            m.cpu.regs[15] = 0x0801_9e94;
            m.cpu.cpsr = 0xbf;
        }
        for callback in callbacks {
            let at = callback.source_offset() as usize;
            managed.bus.rom[at..at+4].fill(0xcd);
        }
        let before = owner.text_catalog_stats().unwrap().substitution_callback_bytes;
        let mut steps = 0;
        while raw.cpu.regs[15] != 0x0801_9ea6 { assert!(steps < 24); raw.step(); steps += 1; }
        steps = 0;
        while managed.cpu.regs[15] != 0x0801_9ea6 {
            assert!(steps < 24, "dispatcher pc {:08x}", managed.cpu.regs[15]);
            if native { table.get(managed.cpu.regs[15] | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void); }
            else { managed.step(); }
            steps += 1;
        }
        let selected = callbacks.iter().any(|c| u32::from(c.index) == index & 255);
        assert_eq!(owner.text_catalog_stats().unwrap().substitution_callback_bytes - before, if selected { 4 } else { 0 });
        assert_eq!(raw.cpu.regs[2], u32::from_le_bytes(rom[(SUBSTITUTION_ROOT + (index & 255)*4) as usize..][..4].try_into().unwrap()));
        for callback in callbacks {
            let at = callback.source_offset() as usize;
            managed.bus.rom[at..at+4].copy_from_slice(&rom[at..at+4]);
        }
        assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap(), "index {index}, native {native}");
        cases += 1;
    }}
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[160,159], "e159_resources_preserved":true,
        "candidate_slots":75, "null_words":4, "owned_bytes":300,
        "source_sha256":SUBSTITUTION_SOURCE_SHA256, "poisoned_raw_rom":true,
        "original_prefix_cases":cases, "compiled_dispatch":"may use interpreter fallback",
        "full_state_clock_equal":true, "callback_invocation":"pending", "ordinary_use":"pending",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "exact EUR ROM, e162 dataset and TinyCC; Wine only, Linux/USA pending"]
fn european_text_substitution_full_copy_callbacks() {
    use std::collections::BTreeSet;
    const DEST: u32 = 0x0203_0010;
    const RETURN: u32 = 0x0800_0300;
    const CASES: [(u32, u32, &[u8], &[u8]); 4] = [
        (0, 0x0200_1f1c, &[1, 0x11, 0x12, 0], &[0x80, 0x10, 0x80, 0x11, 0]),
        (3, 0x0200_21a0, &[0x81, 0x10, 0], &[0x81, 0x10, 0]),
        (0x100, 0x0200_1f1c, &[1, 0x21, 0], &[0x80, 0x20, 0]),
        (0x103, 0x0200_21a0, &[0x82, 0x11, 0], &[0x82, 0x11, 0]),
    ];
    let root = PathBuf::from(std::env::var_os("FFTA_SUBSTITUTION_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_SUBSTITUTION_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision, manifest.decoder_revision), (162, 161));
    let texts: gba_core::rom_data::text_catalogs::TextCatalogs =
        serde_json::from_slice(&fs::read(dataset.join(TEXT_CATALOGS_FILE)).unwrap()).unwrap();
    texts.validate().unwrap();
    let callbacks = &texts.substitution_callbacks;
    assert_eq!(callbacks.len(), 75);
    assert_eq!(callbacks.iter().find(|c| c.index == 0).unwrap().target, 0x0801_9eb9);
    assert_eq!(callbacks.iter().find(|c| c.index == 3).unwrap().target, 0x0801_9ecd);

    let configure = |machine: &mut Machine, index: u32, source: u32, input: &[u8]| {
        machine.bus.ffta_sprites.enabled = false;
        machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
        machine.cpu.regs[0] = DEST;
        machine.cpu.regs[1] = index;
        machine.cpu.regs[13] = 0x0300_7c00;
        machine.cpu.regs[14] = RETURN | 1;
        machine.cpu.regs[15] = 0x0801_9e94;
        machine.cpu.cpsr = 0xbf;
        for offset in 0..32 { machine.bus.write8(DEST - 8 + offset, 0x5a); }
        for (offset, value) in input.iter().enumerate() { machine.bus.write8(source + offset as u32, *value); }
    };
    let validate = |machine: &mut Machine, output: &[u8]| {
        assert_eq!(machine.cpu.regs[0], 0, "callback return value");
        for offset in 0..8 { assert_eq!(machine.bus.read8(DEST - 8 + offset), 0x5a); }
        for (offset, value) in output.iter().enumerate() { assert_eq!(machine.bus.read8(DEST + offset as u32), *value); }
        assert_eq!(machine.bus.read8(DEST + output.len() as u32), 0x5a, "destination guard");
    };
    let mut trace = crate::make_machine(rom.clone(), None);
    let initial = trace.save_state().unwrap();
    let mut visited = BTreeSet::new();
    for &(index, source, input, output) in &CASES {
        trace.load_state(&initial).unwrap();
        configure(&mut trace, index, source, input);
        for _ in 0..1000 {
            if trace.cpu.regs[15] == RETURN { break; }
            visited.insert((trace.cpu.regs[15], trace.cpu.thumb()));
            trace.step();
        }
        assert_eq!(trace.cpu.regs[15], RETURN, "trace callback {index} did not return");
        validate(&mut trace, output);
    }
    let view = View { rom: &rom, ewram: None, iwram: Some(&trace.bus.iwram), bios: None };
    let blocks = visited.iter().map(|&(address, thumb)| {
        let first = if thumb { armv4t::decode_thumb(view.read16(address), address) }
                    else { armv4t::decode_arm(view.read32(address), address) };
        Block { addr: address, thumb, instrs: vec![first] }
    }).collect();
    let object = root.join("substitution-callbacks-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view,
        &recomp_core::tcc_memory::library_path().expect("TinyCC required"), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut results = Vec::new();
    for &(index, source, input, output) in &CASES {
        let mut raw = crate::make_machine(rom.clone(), None);
        let mut interpreted = crate::make_machine(rom.clone(), None);
        let mut translated = crate::make_machine(rom.clone(), None);
        attach_from_directory(&mut interpreted, &dataset).unwrap();
        attach_from_directory(&mut translated, &dataset).unwrap();
        for machine in [&mut raw, &mut interpreted, &mut translated] {
            machine.load_state(&initial).unwrap();
            configure(machine, index, source, input);
        }
        let owner_i = interpreted.bus.rom_resources().unwrap().clone();
        let owner_n = translated.bus.rom_resources().unwrap().clone();
        let before_i = owner_i.text_catalog_stats().unwrap().substitution_callback_bytes;
        let before_n = owner_n.text_catalog_stats().unwrap().substitution_callback_bytes;
        for machine in [&mut interpreted, &mut translated] { for callback in callbacks {
            let at = callback.source_offset() as usize;
            machine.bus.rom[at..at + 4].fill(0xcd);
        }}
        for machine in [&mut raw, &mut interpreted] {
            for _ in 0..1000 { if machine.cpu.regs[15] == RETURN { break; } machine.step(); }
            assert_eq!(machine.cpu.regs[15], RETURN, "callback {index} did not return");
            validate(machine, output);
        }
        let mut compiled_steps = 0;
        while translated.cpu.regs[15] != RETURN {
            table.get(translated.cpu.regs[15] | u32::from(translated.cpu.thumb())).expect("compiled callback instruction")(
                &RT_API, &mut translated as *mut Machine as *mut std::ffi::c_void);
            compiled_steps += 1;
            assert!(compiled_steps < 1000, "compiled callback {index} did not return");
        }
        validate(&mut translated, output);
        let typed_i = owner_i.text_catalog_stats().unwrap().substitution_callback_bytes - before_i;
        let typed_n = owner_n.text_catalog_stats().unwrap().substitution_callback_bytes - before_n;
        assert_eq!((typed_i, typed_n), (4, 4));
        for machine in [&mut interpreted, &mut translated] { for callback in callbacks {
            let at = callback.source_offset() as usize;
            machine.bus.rom[at..at + 4].copy_from_slice(&rom[at..at + 4]);
        }}
        assert_eq!(raw.save_state().unwrap(), interpreted.save_state().unwrap(), "interpreter callback {index}");
        assert_eq!(raw.save_state().unwrap(), translated.save_state().unwrap(), "compiled callback {index}");
        results.push(serde_json::json!({"index":index,"typed_interpreter_bytes":typed_i,"typed_compiled_bytes":typed_n}));
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"controlled EUR callbacks 0/3 and byte-truncated aliases; original interpreter and compiled instruction blocks",
        "revision":[162,161], "cases":results, "source_rom_poisoned":true,
        "destination_guards_and_expected_bytes":true, "full_state_clock_equal":true,
        "ordinary_reachability":"pending", "other_callbacks":"pending", "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "exact EUR ROM, e162 dataset, checkpoint and TinyCC; Wine only, Linux/USA pending"]
fn european_text_substitution_program_dispatch() {
    use std::collections::BTreeSet;
    use gba_core::rom_data::text_catalogs::ProgramUnit;

    const TASK: u32 = 0x0203_0000;
    const DEST: u32 = TASK + 0x70c;
    const PROGRAM_OFFSET: usize = 0x52177c;
    const PROGRAM: u32 = 0x0800_0000 + PROGRAM_OFFSET as u32 + 2;
    const END: u32 = 0x0801_28f6;
    let root = PathBuf::from(std::env::var_os("FFTA_SUBSTITUTION_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_SUBSTITUTION_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let initial = fs::read(std::env::var_os("FFTA_SUBSTITUTION_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    assert_eq!(gba_core::ffta_rom::digest(&initial), "0a439a15a0de6ccda64a8010d50da37b0ba5a4435b683adf45fb7cf68dec56e3");
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision, manifest.decoder_revision), (162, 161));
    let texts: gba_core::rom_data::text_catalogs::TextCatalogs =
        serde_json::from_slice(&fs::read(dataset.join(TEXT_CATALOGS_FILE)).unwrap()).unwrap();
    texts.validate().unwrap();
    let program = &texts.archive_catalogs[9].languages[0].programs[81];
    assert_eq!((program.source_offset, program.header), (PROGRAM_OFFSET as u32, 0x30));
    assert_eq!(program.units.first(), Some(&ProgramUnit::Command { opcode: 0x25, operands: vec![0] }));
    assert_eq!(&rom[PROGRAM_OFFSET..PROGRAM_OFFSET + 5], &[0x30, 0, 0x40, 0x25, 0]);
    assert_eq!(u32::from_le_bytes(rom[0x12288..0x1228c].try_into().unwrap()), 0x0801_28d4);
    let callbacks = &texts.substitution_callbacks;

    let configure = |machine: &mut Machine| {
        machine.bus.ffta_sprites.enabled = false;
        machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
        machine.cpu.regs[13] = 0x0300_7c00;
        machine.cpu.regs[14] = 0x0800_0301;
        machine.cpu.regs[15] = 0x0801_2248;
        machine.cpu.cpsr = 0xbf;
        machine.bus.write32(0x0200_918c, TASK);
        machine.bus.write32(TASK + 0x308, PROGRAM);
        for offset in 0..80 { machine.bus.write8(DEST - 8 + offset, 0x5a); }
        for (offset, value) in [1, 0x11, 0x12, 0].into_iter().enumerate() {
            machine.bus.write8(0x0200_1f1c + offset as u32, value);
        }
    };
    let validate = |machine: &mut Machine| {
        assert_eq!(machine.cpu.regs[15], END);
        for offset in 0..8 { assert_eq!(machine.bus.read8(DEST - 8 + offset), 0x5a); }
        for (offset, value) in [0x80, 0x10, 0x80, 0x11, 0].into_iter().enumerate() {
            assert_eq!(machine.bus.read8(DEST + offset as u32), value);
        }
        for offset in 0..8 { assert_eq!(machine.bus.read8(DEST + 64 + offset), 0x5a); }
    };
    let mut trace = crate::make_machine(rom.clone(), None);
    trace.load_state(&initial).unwrap();
    configure(&mut trace);
    let mut visited = BTreeSet::new();
    for _ in 0..5000 {
        if trace.cpu.regs[15] == END { break; }
        visited.insert((trace.cpu.regs[15], trace.cpu.thumb()));
        trace.step();
    }
    validate(&mut trace);
    for address in [0x0801_2248, 0x0801_28d4, 0x0801_9e94, 0x0801_9eb8, 0x0300_5e78] {
        assert!(visited.contains(&(address, true)), "missing original instruction {address:08x}");
    }
    let view = View { rom: &rom, ewram: None, iwram: Some(&trace.bus.iwram), bios: None };
    let blocks = visited.iter().map(|&(address, thumb)| {
        let first = if thumb { armv4t::decode_thumb(view.read16(address), address) }
                    else { armv4t::decode_arm(view.read32(address), address) };
        Block { addr: address, thumb, instrs: vec![first] }
    }).collect();
    let object = root.join("substitution-program-native.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view,
        &recomp_core::tcc_memory::library_path().expect("TinyCC required"), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut interpreted = crate::make_machine(rom.clone(), None);
    let mut translated = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut interpreted, &dataset).unwrap();
    attach_from_directory(&mut translated, &dataset).unwrap();
    for machine in [&mut raw, &mut interpreted, &mut translated] {
        machine.load_state(&initial).unwrap();
        configure(machine);
    }
    let owner_i = interpreted.bus.rom_resources().unwrap().clone();
    let owner_n = translated.bus.rom_resources().unwrap().clone();
    let before_i = owner_i.text_catalog_stats().unwrap();
    let before_n = owner_n.text_catalog_stats().unwrap();
    for machine in [&mut interpreted, &mut translated] {
        machine.bus.rom[PROGRAM_OFFSET + 2..PROGRAM_OFFSET + 5].fill(0xcd);
        for callback in callbacks {
            let at = callback.source_offset() as usize;
            machine.bus.rom[at..at + 4].fill(0xcd);
        }
    }
    for machine in [&mut raw, &mut interpreted] {
        for _ in 0..5000 { if machine.cpu.regs[15] == END { break; } machine.step(); }
        validate(machine);
    }
    let mut compiled_steps = 0;
    while translated.cpu.regs[15] != END {
        table.get(translated.cpu.regs[15] | u32::from(translated.cpu.thumb())).expect("compiled program instruction")(
            &RT_API, &mut translated as *mut Machine as *mut std::ffi::c_void);
        compiled_steps += 1;
        assert!(compiled_steps < 5000, "compiled program did not reach callback return");
    }
    validate(&mut translated);
    let after_i = owner_i.text_catalog_stats().unwrap();
    let after_n = owner_n.text_catalog_stats().unwrap();
    let program_i = after_i.decoded_program_bytes - before_i.decoded_program_bytes;
    let program_n = after_n.decoded_program_bytes - before_n.decoded_program_bytes;
    let callback_i = after_i.substitution_callback_bytes - before_i.substitution_callback_bytes;
    let callback_n = after_n.substitution_callback_bytes - before_n.substitution_callback_bytes;
    assert!(program_i >= 2 && program_n >= 2, "opcode and operand must use the typed program");
    assert_eq!((callback_i, callback_n), (4, 4));
    for machine in [&mut interpreted, &mut translated] {
        machine.bus.rom[PROGRAM_OFFSET + 2..PROGRAM_OFFSET + 5].copy_from_slice(&rom[PROGRAM_OFFSET + 2..PROGRAM_OFFSET + 5]);
        for callback in callbacks {
            let at = callback.source_offset() as usize;
            machine.bus.rom[at..at + 4].copy_from_slice(&rom[at..at + 4]);
        }
    }
    assert_eq!(raw.save_state().unwrap(), interpreted.save_state().unwrap());
    assert_eq!(raw.save_state().unwrap(), translated.save_state().unwrap());
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"EUR source program 52177c, controlled parser dispatch through callback 0",
        "revision":[162,161], "program_typed_bytes":[program_i,program_n],
        "callback_typed_bytes":[callback_i,callback_n], "source_rom_poisoned":true,
        "original_parser_and_callback_instructions_visited":true, "destination_guards_and_expected_bytes":true,
        "full_state_clock_equal":true, "ordinary_reachability":"pending", "other_callbacks":"pending",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
