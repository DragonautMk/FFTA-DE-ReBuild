//! Reduced offline EUR five-selector catalog and original string/font readers.
use super::*;
use gba_core::{
    rom_data::text_short::{self, Catalog, Encoding, ShortTexts},
    Bus, Machine,
};

fn invoke(machine: &mut Machine, entry: u32, args: [u32; 3]) -> usize {
    machine.cpu.regs[..3].copy_from_slice(&args);
    machine.cpu.regs[13] = 0x03007c00;
    machine.cpu.regs[14] = 0x08000301;
    machine.cpu.regs[15] = entry;
    machine.cpu.cpsr = 0xbf;
    for steps in 1..=100_000 {
        machine.bus.frame_ready = false;
        machine.step();
        if machine.cpu.regs[15] == 0x08000300 {
            assert_eq!(machine.cpu.regs[13], 0x03007c00);
            return steps;
        }
    }
    panic!(
        "original text/font reader {entry:08x} failed at {:08x}",
        machine.cpu.regs[15]
    );
}

#[test]
#[ignore = "exact EUR ROM/retained initialized slot/static inventory and absent FFTA_STATE_TEST_TEXT_ROOT required; new Linux/USA pending"]
fn european_short_texts_match_original_language_copy_and_font_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_TEXT_ROOT").unwrap());
    assert!(!root.exists(), "preserve text/font evidence");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let value = &catalog.decoded;
    assert_eq!(value.languages.len(), 5);
    assert_eq!(value.strings.len(), 535);
    assert_eq!(value.glyphs.len(), 65);
    assert_eq!(catalog.sources.len(), 670);
    let bytes = serde_json::to_vec_pretty(value).unwrap();
    let parsed: ShortTexts = serde_json::from_slice(&bytes).unwrap();
    parsed.verify_source(&rom).unwrap();
    assert_eq!(serde_json::to_vec_pretty(&parsed).unwrap(), bytes);
    let mut unknown = serde_json::to_value(value).unwrap();
    unknown["strings"][0]["unknown"] = true.into();
    assert!(serde_json::from_value::<ShortTexts>(unknown).is_err());
    let mut forged = parsed.clone();
    forged.glyphs[0].width ^= 1;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());
    forged.glyphs[0].tile_words.pop();
    assert!(forged.validate().is_err());
    let inventory: serde_json::Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_TEXT_INVENTORY").unwrap()).unwrap(),
    )
    .unwrap();
    for (language, row) in value
        .languages
        .iter()
        .zip(inventory["languages"].as_array().unwrap())
    {
        assert_eq!(row["table"].as_u64().unwrap(), language.table_offset as u64);
        for (&id, source) in language
            .entries
            .iter()
            .zip(row["entries"].as_array().unwrap())
        {
            assert_eq!(
                source.as_u64().unwrap(),
                value.strings[id as usize].source_offset as u64
            );
        }
    }
    for (text, row) in value
        .strings
        .iter()
        .zip(inventory["strings"].as_array().unwrap())
    {
        let source = text.source_bytes().unwrap();
        assert_eq!(row["offset"].as_u64().unwrap(), text.source_offset as u64);
        assert_eq!(row["length"].as_u64().unwrap(), source.len() as u64);
        assert_eq!(
            row["sha256"].as_str().unwrap(),
            gba_core::ffta_rom::digest(&source)
        );
    }
    assert_eq!(
        inventory["glyphs"],
        serde_json::json!(value.glyphs.iter().map(|g| g.index).collect::<Vec<_>>())
    );
    for source in &catalog.sources {
        let slice = &rom[source.offset as usize..(source.offset + source.length) as usize];
        assert_eq!(gba_core::ffta_rom::digest(slice), source.sha256);
    }
    let snapshot = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_STATE").unwrap());
    let mut machine = crate::make_machine(rom, None);
    crate::save_state::load(&mut machine, &snapshot).unwrap();
    // Original IWRAM memcpy callback used by 080135a4 is already initialized.
    assert_eq!(machine.bus.read32(0x08370eb4), 0x03005ee9);
    const DEST: u32 = 0x02030010;
    let mut instructions = 0;
    let mut calls = 0;
    let mut compact = 0;
    let mut extended = 0;
    let mut source_bytes = 0;
    for language in &value.languages {
        machine.bus.write8(0x02001f6e, language.language_id);
        for (index, &id) in language.entries.iter().enumerate() {
            let text = &value.strings[id as usize];
            let pointer = text.source_offset + 0x08000000;
            instructions += invoke(&mut machine, 0x08016648, [index as u32, 0, 0]);
            calls += 1;
            assert_eq!(machine.cpu.regs[0], pointer, "original language selector");
            let expanded = text.expanded_bytes().unwrap();
            let source = text.source_bytes().unwrap();
            source_bytes += source.len();
            if text.encoding == Encoding::Compact {
                compact += 1;
            } else {
                extended += 1;
            }
            let mut expected = vec![0x5a; expanded.len() + 32];
            for i in 0..expected.len() {
                machine.bus.write8(DEST - 16 + i as u32, 0x5a);
            }
            expected[16..16 + expanded.len()].copy_from_slice(&expanded);
            instructions += invoke(&mut machine, 0x08019d98, [DEST, pointer, 0]);
            calls += 1;
            assert_eq!(machine.cpu.regs[0], expanded.len() as u32 - 1);
            let actual: Vec<_> = (0..expected.len())
                .map(|i| machine.bus.read8(DEST - 16 + i as u32))
                .collect();
            assert_eq!(
                actual, expected,
                "language{} string{index} output and guards",
                language.language_id
            );
        }
    }
    machine.bus.write8(0x02001f6e, 255);
    instructions += invoke(&mut machine, 0x08016648, [0, 0, 0]);
    calls += 1;
    assert_eq!(
        machine.cpu.regs[0],
        0x08000000 + value.strings[0].source_offset,
        "original unknown-language fallback"
    );
    for glyph in &value.glyphs {
        for fixed in [0, 1] {
            machine.bus.write8(0x020020e3, fixed);
            for i in 0..text_short::GLYPH_BYTES + 32 {
                machine.bus.write8(DEST - 16 + i as u32, 0x5a);
            }
            instructions += invoke(&mut machine, 0x080135a4, [DEST, glyph.index as u32, 0]);
            calls += 1;
            let width = if fixed == 0 { glyph.width as u32 } else { 16 };
            assert_eq!(machine.cpu.regs[0], width, "original glyph metric");
            let expected = glyph.source_bytes();
            let actual: Vec<_> = (0..text_short::GLYPH_BYTES)
                .map(|i| machine.bus.read8(DEST + i as u32))
                .collect();
            assert_eq!(actual, expected, "complete original glyph slot");
            assert!((0..16).all(|i| machine.bus.read8(DEST - 16 + i) == 0x5a
                && machine.bus.read8(DEST + text_short::GLYPH_BYTES as u32 + i) == 0x5a));
            instructions += invoke(&mut machine, 0x08013bb4, [glyph.index as u32, 0, 0]);
            calls += 1;
            assert_eq!(
                machine.cpu.regs[0], width,
                "independent original width-only reader"
            );
        }
    }
    assert_eq!((compact, extended, source_bytes), (478, 57, 5468));
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("short-texts.json"), &bytes).unwrap();
    let report = serde_json::json!({"region":"europe","selector_language_ids":[0,1,2,3,4],"entries_per_language":107,
        "unique_strings":535,"compact_links":compact,"extended_links":extended,"string_source_bytes":source_bytes,
        "referenced_glyphs":65,"glyph_source_bytes":65*128,"width_source_bytes":65,"table_source_bytes":5*107*4,
        "sources":catalog.sources,"json_sha256":gba_core::ffta_rom::digest(&bytes),"original_calls":calls,
        "original_instructions":instructions,"all_selector_pointers":true,"all_expanded_bytes_and_guards":true,
        "all_glyph_bytes_and_variable_fixed_widths":true,"unknown_language_fallback":true,
        "full_text_font_catalog":false,"unicode_mapping":"unverified","runtime":"not_integrated",
        "linux":"pending","usa":"pending","performance_measurements":false});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("SHORT_TEXT_EUR languages=5 entries=535 compact={compact} extended={extended} glyphs=65 original_calls={calls} instructions={instructions} pointers/expansion/font/widths/guards=pass");
}
