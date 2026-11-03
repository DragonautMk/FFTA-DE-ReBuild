//! New sources only; earlier custom/indexed graphics acceptance stays closed.
use super::*;
use crate::Machine;
use serde_json::{Value, json};
use std::{fs, path::PathBuf};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SelectorCase {
    reader: u32,
    pointer_load: u32,
}

#[test]
#[ignore = "private EUR table graphics model/original decoder check; Wine only, Linux/USA pending"]
fn european_table_graphics_match_new_sources_and_original_decoder() {
    let root = PathBuf::from(std::env::var_os("FFTA_TABLE_GRAPHICS_MODEL_ROOT").unwrap());
    let out = root.join("windows-model-europe");
    assert!(!out.exists());
    fs::create_dir(&out).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let independent: TableGraphics =
        serde_json::from_slice(&fs::read(root.join("independent-model.json")).unwrap()).unwrap();
    let sources: Vec<SourceRange> =
        serde_json::from_slice(&fs::read(root.join("independent-sources.json")).unwrap()).unwrap();
    assert_eq!(*data, independent);
    assert_eq!(catalog.sources, sources);
    data.verify_source(&rom).unwrap();
    let spans = data.source_spans().unwrap();
    assert_eq!(spans.len(), 48);
    assert_eq!(spans.iter().map(|v| v.1.len()).sum::<usize>(), 53667);
    for (at, bytes, _) in &spans {
        assert_eq!(bytes, &rom[*at as usize..*at as usize + bytes.len()]);
        assert_eq!(data.byte_at(*at), bytes.first().copied());
        assert_eq!(
            data.byte_at(*at + bytes.len() as u32 - 1),
            bytes.last().copied()
        );
    }
    for at in [0, 0x71e57c, 0x71e61c, 0x71e658, 0x71e5e0, u32::MAX] {
        assert_eq!(data.byte_at(at), None);
    }
    let decoded: Vec<_> = data.banks.iter().flat_map(Graphic::decoded_bytes).collect();
    assert_eq!(decoded.len(), 114880);
    assert_eq!(
        ffta_rom::digest(&decoded),
        "11ba607260a1ae16c6eced9c1855ba3f98bd41b11d033b5950f3abb66f16c8fd"
    );
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<TableGraphics>(&encoded).unwrap(),
        *data
    );
    fs::write(out.join("decoded.json"), &encoded).unwrap();
    let mut guards = 0;
    for kind in 0..12 {
        let mut bad = data.clone();
        match kind {
            0 => {
                bad.tables.pop();
            }
            1 => bad.tables.swap(0, 1),
            2 => bad.tables[0].source_offset += 1,
            3 => bad.tables[0].pointers[0].source_offset += 4,
            4 => bad.tables[0].pointers[0].bank_offset += 4,
            5 => {
                bad.banks.pop();
            }
            6 => bad.banks.swap(0, 1),
            7 => bad.banks[0].source_offset += 4,
            8 => bad.banks[0].words[0] ^= 1,
            9 => {
                bad.banks[0].encoded_source.pop();
            }
            10 => bad.tables[0].pointers[0] = bad.tables[0].pointers[1].clone(),
            _ => bad.banks.push(bad.banks[0].clone()),
        }
        assert!(bad.validate().is_err(), "shape{kind}");
        guards += 1;
    }
    for kind in 0..7 {
        let mut bad: Value = serde_json::from_slice(&encoded).unwrap();
        match kind {
            0 => bad["extra"] = json!(0),
            1 => bad["tables"][0]["extra"] = json!(0),
            2 => bad["tables"][0]["pointers"][0]["extra"] = json!(0),
            3 => bad["banks"][0]["extra"] = json!(0),
            4 => bad["tables"][0]["pointers"] = json!([]),
            5 => bad["banks"][0]["words"][0] = json!(-1),
            _ => {
                bad["banks"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("encoded_source");
            }
        }
        assert!(serde_json::from_value::<TableGraphics>(bad).is_err());
        guards += 1;
    }
    // Coherent literal edits remain valid editable shape, but exact EUR authentication rejects them.
    let mut changed = data.clone();
    let bank = &mut changed.banks[0];
    let decoded =
        crate::rom_data::custom_lz::decode(&bank.encoded_source, rom.len(), rom.len()).unwrap();
    let literal = &decoded.literal_spans[0];
    bank.encoded_source[literal.encoded_offset] ^= 1;
    let new =
        crate::rom_data::custom_lz::decode(&bank.encoded_source, rom.len(), rom.len()).unwrap();
    bank.words = new
        .bytes
        .chunks_exact(4)
        .map(|v| u32::from_le_bytes(v.try_into().unwrap()))
        .collect();
    changed.validate().unwrap();
    assert!(changed.verify_source(&rom).is_err());
    guards += 1;
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    guards += 1;
    let mut wrong = rom.clone();
    wrong[0x30658] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    guards += 1;
    wrong.copy_from_slice(&rom);
    wrong[data.banks[0].source_offset as usize] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    guards += 1;
    let mut m = Machine::new(rom.clone());
    m.bus.ffta_sprites.enabled = false;
    let mut instructions = 0;
    for bank in &data.banks {
        m.bus.ewram.fill(0x5a);
        m.bus.iwram.fill(0x3c);
        m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
        m.cpu.regs[0] = 0x02010000;
        m.cpu.regs[1] = 0x08000000 + bank.source_offset;
        m.cpu.regs[13] = 0x03007d00;
        m.cpu.regs[14] = 0x03007f01;
        m.cpu.regs[15] = 0x080054a0;
        m.cpu.cpsr = 0xbf;
        let preserved = m.cpu.regs[4..12].to_vec();
        let (mut ewram, mut iwram, vram, oam, palette) = (
            m.bus.ewram.clone(),
            m.bus.iwram.clone(),
            m.bus.vram.clone(),
            m.bus.oam.clone(),
            m.bus.palette.clone(),
        );
        let decoded = bank.decoded_bytes();
        ewram[0x10000..0x10000 + decoded.len()].copy_from_slice(&decoded);
        let mut n = 0;
        let mut terminal = false;
        while m.cpu.regs[15] != 0x03007f00 {
            if m.cpu.regs[15] == 0x080058de {
                assert_eq!(
                    m.cpu.regs[7],
                    0x08000000 + bank.source_offset + bank.encoded_source.len() as u32
                );
                assert_eq!(m.cpu.regs[9], decoded.len() as u32);
                terminal = true;
            }
            assert!(n < 2_000_000);
            m.bus.frame_ready = false;
            m.step();
            n += 1;
        }
        instructions += n;
        assert!(terminal);
        assert_eq!(m.cpu.regs[0], decoded.len() as u32);
        assert_eq!(m.cpu.regs[13], 0x03007d00);
        assert_eq!(&m.cpu.regs[4..12], preserved);
        assert_eq!(m.bus.ewram, ewram);
        iwram[0x7cd8..0x7d00].copy_from_slice(&m.bus.iwram[0x7cd8..0x7d00]);
        assert_eq!(m.bus.iwram, iwram);
        assert_eq!(m.bus.vram, vram);
        assert_eq!(m.bus.oam, oam);
        assert_eq!(m.bus.palette, palette);
    }
    let cases: Vec<SelectorCase> =
        serde_json::from_slice(&fs::read(root.join("selector-cases.json")).unwrap()).unwrap();
    assert_eq!(cases.len(), 8);
    let mut selectors = 0;
    for (table, c) in data.tables.iter().zip(cases) {
        for language in [0u8, 1, 2, 3, 4, 5, 255] {
            m.bus.ewram.fill(0x5a);
            m.bus.ewram[0x1f6e] = language;
            m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
            m.cpu.regs[15] = 0x08000000 + c.reader;
            m.cpu.cpsr = 0xbf;
            let mut n = 0;
            while m.cpu.regs[15] != 0x08000000 + c.pointer_load + 2 {
                if m.cpu.regs[15] == 0x08000000 + c.pointer_load {
                    let at = c.pointer_load as usize;
                    let op = u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
                    assert_eq!(
                        m.cpu.regs[usize::from((op >> 3) & 7)] + u32::from((op >> 6) & 31) * 4,
                        table.encoded_view(language)
                    );
                }
                assert!(n < 12);
                m.bus.frame_ready = false;
                m.step();
                n += 1;
            }
            instructions += n;
            let at = c.pointer_load as usize;
            let reg = usize::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap()) & 7);
            let off = (table.encoded_view(language) - 0x08000000) as usize;
            let expected = u32::from_le_bytes(rom[off..off + 4].try_into().unwrap());
            assert_eq!(m.cpu.regs[reg], expected);
            assert_eq!(
                table.select(language).map(|p| p.bank_offset),
                if language < 5 {
                    Some(expected - 0x08000000)
                } else {
                    None
                }
            );
            selectors += 1;
        }
    }
    assert_eq!(m.bus.rom, rom);
    let result = json!({"status":"PASS-model-new-original-decoders","guards":guards,"banks":40,"tables":8,"source_spans":48,"source_bytes":53667,"decoded_bytes":114880,"original_decoder_calls":40,"original_selector_prefixes":selectors,"instructions_coverage_only":instructions,"source_cursor_produced_returned_exact":true,"complete_output_guards_EWRAM_exact":true,"SP_R4_R11_exact":true,"IWRAM_outside40B_original_stack_and_VRAM_OAM_palette_exact":true,"final_ROM_exact":true,"old_UI_indexed_proofs_replayed":false,"managed_catalog_unchanged":[1,91,90],"fulltransferhelperABI":"pending","ordinary":"pending","Linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false});
    fs::write(
        out.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    eprintln!(
        "ROM_DATA_TABLE_GRAPHICS_MODEL banks40 tables8 source53667 decoded114880 guards{guards} original40 selectors{selectors} PASS"
    );
}
