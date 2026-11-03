use super::*;
use crate::{capi::RT_API, Bus, Machine};
#[test]
fn typed_texts_cover_encodings_widths_mirrors_edges_swi_dma_and_load() {
    let texts = text_short::ShortTexts {
        languages: text_short::TABLES
            .iter()
            .enumerate()
            .map(|(id, &table_offset)| text_short::Language {
                language_id: id as u8,
                table_offset,
                entries: vec![id as u16; 107],
            })
            .collect(),
        strings: text_short::STRING_STARTS
            .iter()
            .enumerate()
            .map(|(id, &source_offset)| text_short::Text {
                source_offset,
                encoding: if id % 2 == 0 {
                    text_short::Encoding::Compact
                } else {
                    text_short::Encoding::Extended
                },
                units: if id % 2 == 0 {
                    vec![text_short::Unit::Glyph { index: 254 }]
                } else {
                    vec![
                        text_short::Unit::Glyph { index: 0 },
                        text_short::Unit::ControlByte { value: 64 },
                        text_short::Unit::ControlByte { value: 115 },
                    ]
                },
            })
            .collect(),
        glyphs: [0, 254]
            .into_iter()
            .map(|index| text_short::Glyph {
                index,
                width: 7,
                tile_words: vec![0x81234567; 32],
            })
            .collect(),
    };
    texts.validate().unwrap();
    let mut sources = Vec::new();
    let mut rom = vec![0x53; 0x1000000];
    let mut add = |offset: u32, bytes: Vec<u8>, role| {
        rom[offset as usize..offset as usize + bytes.len()].copy_from_slice(&bytes);
        sources.push(SourceRange {
            offset,
            length: bytes.len() as u32,
            sha256: ffta_rom::digest(&bytes),
            role,
        });
    };
    for (id, l) in texts.languages.iter().enumerate() {
        add(
            l.table_offset,
            texts.table_bytes(l).unwrap(),
            format!("short-text-table/{id}"),
        );
    }
    for (id, t) in texts.strings.iter().enumerate() {
        add(
            t.source_offset,
            t.source_bytes().unwrap(),
            format!("short-text-string/{id}"),
        );
    }
    for g in &texts.glyphs {
        add(
            g.source_offset(),
            g.source_bytes(),
            format!("short-text-glyph/{}", g.index),
        );
        add(
            g.width_offset(),
            vec![g.width],
            format!("short-text-width/{}", g.index),
        );
    }
    let mut resources = RuntimeResources::new(
        ffta_rom::digest(&rom),
        "synthetic".into(),
        "obj".into(),
        Version::Europe,
        super::super::tests::dimensions(),
    );
    resources.short_texts = Some(Box::new(
        TextResources::new(texts.clone(), &sources, "text".into()).unwrap(),
    ));
    let resources = Arc::new(resources);
    let mut decoded = Machine::new(rom.clone());
    decoded
        .bus
        .set_rom_resources(Some(resources.clone()))
        .unwrap();
    for s in &sources {
        decoded.bus.rom[s.offset as usize..(s.offset + s.length) as usize].fill(0xcd);
    }
    let clock = decoded.bus.clock;
    for real in [false, true] {
        decoded.bus.real_bios = real;
        for mirror in [0x08000000, 0x0a000000, 0x0c000000] {
            for s in &sources {
                for at in s.offset as usize - 3..(s.offset + s.length) as usize + 3 {
                    let ptr = &mut decoded as *mut Machine as *mut std::ffi::c_void;
                    assert_eq!(decoded.bus.read8(mirror + at as u32), rom[at]);
                    assert_eq!(
                        decoded.bus.read16(mirror + at as u32),
                        u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())
                    );
                    assert_eq!(
                        (RT_API.read32)(ptr, mirror + at as u32),
                        u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
                    );
                }
            }
        }
    }
    assert_eq!(decoded.bus.clock, clock);
    let stats = resources.text_stats().unwrap();
    assert!(
        stats.table_bytes > 0
            && stats.string_bytes > 0
            && stats.glyph_bytes > 0
            && stats.width_bytes > 0
    );
    let g = &texts.glyphs[1];
    let at = g.source_offset() as usize - 1;
    let edge = resources
        .short_texts
        .as_ref()
        .unwrap()
        .guest_read(at, &decoded.bus.rom, Some([0xa5; 4]))
        .unwrap();
    assert_eq!(edge, [0xa5, 0x67, 0x45, 0x23]);
    decoded.bus.real_bios = false;
    let mut legacy = Machine::new(rom.clone());
    for target in [&mut legacy, &mut decoded] {
        target
            .bus
            .write32(0x040000d4, 0x08000000 + g.source_offset());
        target.bus.write32(0x040000d8, 0x06000020);
        target.bus.write16(0x040000dc, 32);
        target.bus.write16(0x040000de, 0x8400);
    }
    for s in &sources {
        decoded.bus.rom[s.offset as usize..(s.offset + s.length) as usize]
            .copy_from_slice(&rom[s.offset as usize..(s.offset + s.length) as usize]);
    }
    assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    for target in [&mut legacy, &mut decoded] {
        target.cpu.cpsr = 0xbf;
        target.cpu.regs[15] = 0x02000000;
        target.cpu.regs[0] = 0x0a000000 + text_short::TABLES[0];
        target.cpu.regs[1] = 0x02015000;
        target.cpu.regs[2] = 4;
        target.bus.write16(0x02000000, 0xdf0b);
    }
    for s in &sources {
        decoded.bus.rom[s.offset as usize..(s.offset + s.length) as usize].fill(0xcd);
    }
    legacy.step();
    decoded.step();
    for s in &sources {
        decoded.bus.rom[s.offset as usize..(s.offset + s.length) as usize]
            .copy_from_slice(&rom[s.offset as usize..(s.offset + s.length) as usize]);
    }
    assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    let state = decoded.save_state().unwrap();
    let stats = resources.text_stats().unwrap();
    decoded.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(
        decoded.bus.rom_resources().unwrap(),
        &resources
    ));
    assert_eq!(resources.text_stats().unwrap(), stats);
}
