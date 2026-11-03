//! Broad EUR text data oracle, reduced to one focused optional Wine check.
use super::*;
use gba_core::{
    rom_data::text_catalogs::{Catalog, ProgramUnit, TextCatalogs},
    Bus, Machine,
};
use std::collections::BTreeSet;
fn invoke(machine: &mut Machine, entry: u32, args: [u32; 4]) -> usize {
    machine.cpu.regs[..4].copy_from_slice(&args);
    machine.cpu.regs[13] = 0x03007c00;
    machine.cpu.regs[14] = 0x08000301;
    machine.cpu.regs[15] = entry;
    machine.cpu.cpsr = 0xbf;
    machine.bus.write32(0x03007c00, 0);
    machine.bus.write32(0x03007c04, 0);
    for steps in 1..=100_000 {
        machine.bus.frame_ready = false;
        machine.step();
        if machine.cpu.regs[15] == 0x08000300 {
            assert_eq!(machine.cpu.regs[13], 0x03007c00);
            return steps;
        }
    }
    panic!(
        "text original {entry:08x} args={args:08x?} did not return at {:08x}; regs={:08x?}",
        machine.cpu.regs[15], machine.cpu.regs
    );
}
#[test]
#[ignore = "exact EUR ROM/retained initialized slot/static catalogs and absent FFTA_STATE_TEST_TEXT_CATALOG_ROOT required; Linux/USA pending"]
fn european_pointer_and_relative_text_catalogs_match_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_TEXT_CATALOG_ROOT").unwrap());
    assert!(!root.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let value = &catalog.decoded;
    assert_eq!(value.strings.len(), 16100);
    assert_eq!(value.glyphs.len(), 128);
    assert_eq!(catalog.sources.len(), 33581);
    let bytes = serde_json::to_vec_pretty(value).unwrap();
    let parsed: TextCatalogs = serde_json::from_slice(&bytes).unwrap();
    parsed.verify_source(&rom).unwrap();
    assert_eq!(serde_json::to_vec_pretty(&parsed).unwrap(), bytes);
    let mut forged = parsed.clone();
    forged.glyphs[0].width ^= 1;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());
    let mut bad = parsed.clone();
    bad.archive_catalogs[0].languages[0].offsets.pop();
    assert!(bad.validate().is_err());
    let inventory =
        PathBuf::from(std::env::var_os("FFTA_STATE_TEST_TEXT_CATALOG_INVENTORY").unwrap());
    let pointer: serde_json::Value =
        serde_json::from_slice(&fs::read(inventory.join("pointer-catalogs.json")).unwrap())
            .unwrap();
    let archives: serde_json::Value =
        serde_json::from_slice(&fs::read(inventory.join("archives.json")).unwrap()).unwrap();
    for (c, row) in value
        .pointer_catalogs
        .iter()
        .zip(pointer["catalogs"].as_array().unwrap())
    {
        assert_eq!(c.reader as u64, row["reader"].as_u64().unwrap());
        for (l, row) in c.languages.iter().zip(row["languages"].as_array().unwrap()) {
            assert_eq!(l.source_offset as u64, row["table"].as_u64().unwrap());
            for (&id, row) in l.entries.iter().zip(row["entries"].as_array().unwrap()) {
                assert_eq!(
                    value.strings[id as usize].source_offset as u64,
                    row.as_u64().unwrap()
                );
            }
        }
    }
    let mut programs = 0;
    let mut flags = [0usize; 4];
    for (c, index) in value.archive_catalogs.iter().zip(0..) {
        for (language, a) in c.languages.iter().enumerate() {
            let row = &archives["archives"][index * 5 + language];
            assert_eq!(a.source_offset as u64, row["root"].as_u64().unwrap());
            assert_eq!(serde_json::json!(a.offsets), row["offsets"]);
            for (p, row) in a.programs.iter().zip(row["entries"].as_array().unwrap()) {
                let source = p.source_bytes().unwrap();
                assert_eq!(p.source_offset as u64, row["offset"].as_u64().unwrap());
                assert_eq!(source.len() as u64, row["length"].as_u64().unwrap());
                assert_eq!(
                    gba_core::ffta_rom::digest(&source),
                    row["sha256"].as_str().unwrap()
                );
                assert_eq!(
                    p.program_bytes().unwrap().len() as u64,
                    row["program_bytes"].as_u64().unwrap()
                );
                programs += 1;
                flags[(p.header & 3) as usize] += 1;
            }
        }
    }
    assert_eq!((programs, flags), (17035, [6138, 1999, 8898, 0]));
    for source in &catalog.sources {
        assert_eq!(
            gba_core::ffta_rom::digest(
                &rom[source.offset as usize..(source.offset + source.length) as usize]
            ),
            source.sha256
        );
    }
    let mut machine = crate::make_machine(rom.clone(), None);
    crate::save_state::load(
        &mut machine,
        Path::new(&std::env::var_os("FFTA_STATE_TEST_STATE").unwrap()),
    )
    .unwrap();
    let start = machine.save_state().unwrap();
    let mut calls = 0;
    let mut instructions = 0;
    let mut pointer_cases = 0;
    const DEST: u32 = 0x02030010;
    const SCRATCH: u32 = 0x02032000;
    const REBUILT: u32 = 0x02034000;
    for (category, c) in value.pointer_catalogs.iter().enumerate() {
        // The original107-entry all-string oracle is closed; this integration
        // checks only newly covered pointer categories at their two boundaries.
        if category == 2 {
            continue;
        }
        for (language, l) in c.languages.iter().enumerate() {
            for index in [0, l.entries.len() - 1] {
                machine.load_state(&start).unwrap();
                machine.bus.write8(0x02001f6e, language as u8);
                instructions += invoke(&mut machine, c.reader, [index as u32, 0, 0, 0]);
                calls += 1;
                let text = &value.strings[l.entries[index] as usize];
                assert_eq!(machine.cpu.regs[0], 0x08000000 + text.source_offset);
                for i in 0..4096 {
                    machine.bus.write8(DEST - 16 + i, 0x5a);
                }
                let pointer = machine.cpu.regs[0];
                instructions += invoke(&mut machine, 0x08019d98, [DEST, pointer, 0, 0]);
                calls += 1;
                let expanded = text.expanded_bytes().unwrap();
                assert_eq!(machine.cpu.regs[0] as usize, expanded.len() - 1);
                assert!(
                    (0..expanded.len()).all(|i| machine.bus.read8(DEST + i as u32) == expanded[i])
                );
                assert!((0..16).all(|i| machine.bus.read8(DEST - 16 + i) == 0x5a
                    && machine.bus.read8(DEST + expanded.len() as u32 + i) == 0x5a));
                pointer_cases += 1;
            }
        }
    }
    let mut cases = BTreeSet::new();
    let mut forms = BTreeSet::new();
    let mut opcodes = BTreeSet::new();
    for (category, c) in value.archive_catalogs.iter().enumerate() {
        for (language, a) in c.languages.iter().enumerate() {
            machine.load_state(&start).unwrap();
            machine.bus.write8(0x02001f6e, language as u8);
            instructions += invoke(&mut machine, c.reader, [0; 4]);
            calls += 1;
            assert_eq!(machine.cpu.regs[0], 0x08000000 + a.source_offset);
            cases.insert((category, language, 0));
            cases.insert((category, language, a.programs.len() - 1));
            for (index, p) in a.programs.iter().enumerate() {
                if forms.insert(p.header & 3) {
                    cases.insert((category, language, index));
                }
                for unit in &p.units {
                    if let ProgramUnit::Command { opcode, .. } = unit {
                        if opcodes.insert((p.header & 1, *opcode)) {
                            cases.insert((category, language, index));
                        }
                    }
                }
            }
        }
    }
    let mut report_cases = Vec::new();
    for (category, language, index) in cases {
        let a = &value.archive_catalogs[category].languages[language];
        let p = &a.programs[index];
        eprintln!("TEXT_ARCHIVE_CASE category={category} language={language} index={index} source={:08x} header={:04x}", p.source_offset, p.header);
        let mut outputs = Vec::new();
        for rebuilt in [false, true] {
            machine.load_state(&start).unwrap();
            machine.bus.write8(0x02001f6e, language as u8);
            // Dynamic command80uses the current unit/mission context. This
            // retained pub snapshot has no active text context; index0would
            // wrap toFFFFin original08013dd2. Use its explicit nullable-context
            // fallback with valid one-based index1for both original/typed
            // scripts. This controlled reader fixture is not a gameplay scene.
            machine.bus.write32(0x020020d0, 0);
            machine.bus.write16(0x020020dc, 1);
            for i in 0..4096 {
                machine.bus.write8(DEST - 16 + i, 0x5a);
            }
            let (root, index) = if rebuilt {
                let mut data = vec![4, 0, 255, 255];
                data.extend_from_slice(&(p.header & !2).to_le_bytes());
                data.extend(p.program_bytes().unwrap());
                for (i, &v) in data.iter().enumerate() {
                    machine.bus.write8(REBUILT + i as u32, v);
                }
                (REBUILT, 0)
            } else {
                (0x08000000 + a.source_offset, index as u32)
            };
            instructions += invoke(&mut machine, 0x08013f8c, [DEST, root, SCRATCH, index]);
            calls += 1;
            let output: Vec<_> = (0..4096)
                .map(|i| machine.bus.read8(DEST - 16 + i))
                .collect();
            outputs.push((machine.cpu.regs[0], output));
        }
        let expected_return = if p.header & 2 != 0 && outputs[0].0 > 511 {
            outputs[0].0 - 512
        } else {
            outputs[0].0
        };
        assert_eq!(
            outputs[1].0, expected_return,
            "original uncompressed return adjustment"
        );
        assert_eq!(outputs[0].1,outputs[1].1,"category{category} language{language} index{index} original/typed script output+guards");
        report_cases.push(serde_json::json!({"category":category,"language":language,"index":index,"header":p.header,"output_return":outputs[0].0}));
    }
    // Only newly referenced font slots need an original copy oracle here.
    let old: std::collections::BTreeSet<u16> = serde_json::from_slice::<serde_json::Value>(
        &fs::read(inventory.join("../rom-data-text-fonts/static-short-catalog.json")).unwrap(),
    )
    .unwrap()["glyphs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as u16)
        .collect();
    let mut glyph_cases = 0;
    for glyph in &value.glyphs {
        if !old.contains(&glyph.index) {
            machine.load_state(&start).unwrap();
            machine.bus.write8(0x020020e3, 0);
            instructions += invoke(&mut machine, 0x080135a4, [DEST, glyph.index as u32, 0, 0]);
            calls += 1;
            assert_eq!(machine.cpu.regs[0], glyph.width as u32);
            let expected = glyph.source_bytes();
            assert!((0..128).all(|i| machine.bus.read8(DEST + i) == expected[i as usize]));
            glyph_cases += 1;
        }
    }
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("text-catalogs.json"), &bytes).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"region":"europe","pointer_catalogs":9,"pointer_languages":45,"strings":16100,
        "archive_catalogs":29,"archives":145,"programs":programs,"raw_extended_compact_compressed_flags":flags,"glyphs":128,"sources":catalog.sources.len(),
        "json_sha256":gba_core::ffta_rom::digest(&bytes),"pointer_boundary_cases":pointer_cases,"archive_cases":report_cases,"archive_command_forms":opcodes.len(),
        "new_glyph_cases":glyph_cases,"calls":calls,"original_instructions":instructions,"typed_rebuilt_scripts_match_original_outputs":true,
        "runtime":"offline-only","linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
    eprintln!("TEXT_CATALOGS_EUR pointer_catalogs=9 strings=16100 archives=145 programs=17035 glyphs=128 calls={calls} instructions={instructions} source/JSON/original_outputs=pass");
}
