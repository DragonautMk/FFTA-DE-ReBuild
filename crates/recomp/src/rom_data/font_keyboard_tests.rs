//! EUR complete font and original name-keyboard archive/cell producer oracle.
use super::*;
use gba_core::{
    rom_data::{
        fonts::{self, FontBank},
        name_keyboard::{self, Keyboard},
    },
    Bus, Machine,
};
use std::collections::BTreeSet;
fn invoke(m: &mut Machine, entry: u32, args: [u32; 4]) -> usize {
    m.cpu.regs[..4].copy_from_slice(&args);
    m.cpu.regs[13] = 0x03007c00;
    m.cpu.regs[14] = 0x08000301;
    m.cpu.regs[15] = entry;
    m.cpu.cpsr = 0xbf;
    m.bus.write32(0x03007c00, 0);
    m.bus.write32(0x03007c04, 0);
    for n in 1..=100_000 {
        m.bus.frame_ready = false;
        m.step();
        if m.cpu.regs[15] == 0x08000300 {
            assert_eq!(m.cpu.regs[13], 0x03007c00);
            return n;
        }
    }
    panic!(
        "original font/keyboard {entry:08x} did not return at{:08x}",
        m.cpu.regs[15]
    );
}
#[test]
#[ignore = "exact EUR ROM/retained initialized slot/static inventory and absent FFTA_STATE_TEST_FONT_KEYBOARD_ROOT; new Linux/USA pending"]
fn european_complete_font_and_name_keyboard_match_original_slots_cells_and_selector() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_FONT_KEYBOARD_ROOT").unwrap());
    assert!(!root.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let font = fonts::Catalog::from_rom(&rom).unwrap();
    let keyboard = name_keyboard::Catalog::from_rom(&rom).unwrap();
    assert_eq!(font.decoded.glyphs.len(), 3175);
    assert_eq!(font.sources.len(), 2);
    assert_eq!(keyboard.sources.len(), 5);
    let fb = serde_json::to_vec_pretty(&font.decoded).unwrap();
    let kb = serde_json::to_vec_pretty(&keyboard.decoded).unwrap();
    let f: FontBank = serde_json::from_slice(&fb).unwrap();
    let k: Keyboard = serde_json::from_slice(&kb).unwrap();
    f.verify_source(&rom).unwrap();
    k.verify_source(&rom).unwrap();
    assert_eq!(serde_json::to_vec_pretty(&f).unwrap(), fb);
    assert_eq!(serde_json::to_vec_pretty(&k).unwrap(), kb);
    let mut forged = f.clone();
    forged.glyphs[3174].tile_words[0] ^= 1;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());
    let mut bad = k.clone();
    bad.cells[0][0] ^= 1;
    assert!(bad.validate().is_err());
    bad = k.clone();
    bad.row_widths[0][0] = 9;
    assert!(bad.validate().is_err());
    bad = k.clone();
    bad.offsets[3] = 65534;
    assert!(bad.validate().is_err());
    let inventory: serde_json::Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_FONT_KEYBOARD_INVENTORY").unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(inventory["cells"], serde_json::json!(k.cells));
    assert_eq!(inventory["row_widths"], serde_json::json!(k.row_widths));
    for source in font.sources.iter().chain(&keyboard.sources) {
        assert_eq!(
            gba_core::ffta_rom::digest(
                &rom[source.offset as usize..(source.offset + source.length) as usize]
            ),
            source.sha256
        );
    }
    let old: BTreeSet<u16> = inventory["prior_glyphs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as u16)
        .collect();
    assert_eq!(old.len(), 128);
    let mut m = crate::make_machine(rom.clone(), None);
    crate::save_state::load(
        &mut m,
        Path::new(&std::env::var_os("FFTA_STATE_TEST_STATE").unwrap()),
    )
    .unwrap();
    let start = m.save_state().unwrap();
    const DEST: u32 = 0x02030010;
    let mut calls = 0;
    let mut instructions = 0;
    let mut glyph_cases = 0;
    m.bus.write8(0x020020e3, 0);
    for glyph in &f.glyphs {
        if old.contains(&glyph.index) {
            continue;
        }
        for i in 0..160 {
            m.bus.write8(DEST - 16 + i, 0x5a);
        }
        instructions += invoke(&mut m, 0x080135a4, [DEST, glyph.index as u32, 0, 0]);
        calls += 1;
        let bytes = glyph.source_bytes();
        assert_eq!(m.cpu.regs[0], glyph.width as u32);
        assert!((0..128).all(|i| m.bus.read8(DEST + i) == bytes[i as usize]));
        assert!((0..16)
            .all(|i| m.bus.read8(DEST - 16 + i) == 0x5a && m.bus.read8(DEST + 128 + i) == 0x5a));
        glyph_cases += 1;
    }
    assert_eq!(glyph_cases, 3047);
    const TASK: u32 = 0x02028000;
    const CELLS: u32 = 0x02029010;
    const TEXT: u32 = 0x02029410;
    const SCRATCH: u32 = 0x02029810;
    let mut selector_calls = 0;
    for page in 0..3 {
        m.load_state(&start).unwrap();
        for i in 0..512 {
            m.bus.write8(CELLS - 16 + i, 0x5a);
        }
        m.bus.write32(TASK + 0x304, CELLS);
        m.bus.write32(TASK + 0x308, TEXT);
        m.bus.write32(TASK + 0x30c, SCRATCH);
        instructions += invoke(&mut m, 0x0812e900, [TASK, page as u32, 0, 0]);
        calls += 1;
        let begin = CELLS + page as u32 * 120;
        let expected: Vec<u8> = k.cells[page].iter().flat_map(|v| v.to_be_bytes()).collect();
        assert!(
            (0..120).all(|i| m.bus.read8(begin + i) == expected[i as usize]),
            "complete original keyboard page{page}"
        );
        assert!((0..16)
            .all(|i| m.bus.read8(begin - 16 + i) == 0x5a && m.bus.read8(begin + 120 + i) == 0x5a));
        let program = k.programs[page].program_bytes().unwrap();
        assert!((0..program.len()).all(|i| m.bus.read8(TEXT + i as u32) == program[i]));
        for row in 0..6 {
            for column in 0..10 {
                instructions += invoke(
                    &mut m,
                    0x0812f15c,
                    [TASK, page as u32, column as u32, row as u32],
                );
                calls += 1;
                selector_calls += 1;
                assert_eq!(
                    m.cpu.regs[0],
                    k.cells[page][row * 10 + column] as u32,
                    "BE code from original selector"
                );
            }
        }
    }
    assert_eq!(selector_calls, 180);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("font-bank.json"), &fb).unwrap();
    fs::write(root.join("name-keyboard.json"), &kb).unwrap();
    let report = serde_json::json!({"region":"europe","font_slots":3175,"font_source_bytes":fonts::BANK_BYTES,"width_source_bytes":3175,"font_sources":font.sources,
        "keyboard_pages":3,"keyboard_program_sources":keyboard.sources,"keyboard_glyphs":160,"new_keyboard_glyphs":37,"cells":180,"row_widths":k.row_widths,
        "new_original_font_slots":glyph_cases,"original_cell_pages":3,"original_selector_calls":selector_calls,"original_calls":calls,"original_instructions":instructions,
        "font_json_sha256":gba_core::ffta_rom::digest(&fb),"keyboard_json_sha256":gba_core::ffta_rom::digest(&kb),"exact_source_json_and_original_outputs":true,
        "runtime":"offline-only","full_font_bank_endpoint_verified":true,"unicode":"unverified","linux":"pending","usa":"pending","performance_measurements":false});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("FONT_KEYBOARD_EUR font_slots=3175 new_original_slots=3047 pages=3 selector_calls=180 calls={calls} instructions={instructions} source/JSON/font/cells/guards=pass");
}
