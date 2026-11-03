use super::*;
use crate::{capi::RT_API, Bus, Machine};
fn fixture() -> (model::TextCatalogs, Vec<SourceRange>, Vec<u8>) {
    let mut value = model::TextCatalogs {
        pointer_catalogs: Vec::new(),
        strings: Vec::new(),
        archive_catalogs: Vec::new(),
        glyphs: vec![text_short::Glyph {
            index: 1,
            width: 7,
            tile_words: vec![0x12345678; 32],
        }],
    };
    for category in 0..9 {
        let mut languages = Vec::new();
        for language in 0..5 {
            let id = value.strings.len() as u16;
            value.strings.push(text_short::Text {
                source_offset: model::POINTER_STARTS[category][language],
                encoding: if category == 0 && language == 0 {
                    text_short::Encoding::Extended
                } else {
                    text_short::Encoding::Compact
                },
                units: vec![text_short::Unit::Glyph {
                    index: if category == 0 && language == 0 {
                        486
                    } else {
                        1
                    },
                }],
            });
            languages.push(model::PointerLanguage {
                source_offset: model::POINTER_ROOTS[category][language],
                entries: vec![id; model::POINTER_COUNTS[category]],
            });
        }
        value.pointer_catalogs.push(model::PointerCatalog {
            reader: model::POINTER_READERS[category],
            languages,
        });
    }
    for category in 0..29 {
        let languages = (0..5)
            .map(|language| {
                let root = model::ARCHIVE_ROOTS[category][language];
                model::Archive {
                    source_offset: root,
                    offsets: vec![6, 14, 65535],
                    programs: vec![
                        Program {
                            source_offset: root + 6,
                            header: 0,
                            units: vec![
                                ProgramUnit::Glyph { index: 1 },
                                ProgramUnit::Command {
                                    opcode: 0x3e,
                                    operands: vec![0],
                                },
                            ],
                            decoded_padding: Vec::new(),
                            encoded: None,
                        },
                        Program {
                            source_offset: root + 14,
                            header: 2,
                            units: vec![ProgramUnit::Glyph { index: 1 }],
                            decoded_padding: vec![0],
                            encoded: Some(vec![0, 0, 0, 4, 0x43, 128, 1, 0, 0]),
                        },
                    ],
                }
            })
            .collect();
        value.archive_catalogs.push(model::ArchiveCatalog {
            reader: model::ARCHIVE_READERS[category],
            languages,
        });
    }
    value.glyphs.push(text_short::Glyph {
        index: 486,
        width: 16,
        tile_words: vec![0x87654321; 32],
    });
    value.validate().unwrap();
    let mut rom = vec![0x53; 0x1000000];
    let mut ranges = Vec::new();
    let mut add = |offset: u32, bytes: Vec<u8>, role| {
        rom[offset as usize..offset as usize + bytes.len()].copy_from_slice(&bytes);
        ranges.push(SourceRange {
            offset,
            length: bytes.len() as u32,
            sha256: ffta_rom::digest(&bytes),
            role,
        })
    };
    for (id, c) in value.pointer_catalogs.iter().enumerate() {
        for (language, t) in c.languages.iter().enumerate() {
            add(
                t.source_offset,
                value.table_bytes(t).unwrap(),
                format!("text-pointer-table/{id}/{language}"),
            );
        }
    }
    for (id, t) in value.strings.iter().enumerate() {
        add(
            t.source_offset,
            t.source_bytes().unwrap(),
            format!("text-pointer-string/{id}"),
        );
    }
    for (id, c) in value.archive_catalogs.iter().enumerate() {
        for (language, a) in c.languages.iter().enumerate() {
            add(
                a.source_offset,
                a.offsets.iter().flat_map(|v| v.to_le_bytes()).collect(),
                format!("text-archive-index/{id}/{language}"),
            );
            for (index, p) in a.programs.iter().enumerate() {
                add(
                    p.source_offset,
                    p.source_bytes().unwrap(),
                    format!("text-archive-program/{id}/{language}/{index}"),
                );
            }
        }
    }
    for g in &value.glyphs {
        add(
            g.source_offset(),
            g.source_bytes(),
            format!("text-font-glyph/{}", g.index),
        );
        add(
            g.width_offset(),
            vec![g.width],
            format!("text-font-width/{}", g.index),
        );
    }
    (value, ranges, rom)
}

#[test]
fn typed_full_text_sources_and_compressed_literals_cover_edges_mirrors_swi_dma_load() {
    let (value, sources, rom) = fixture();
    let mut typed = TextCatalogResources::new(value, &sources, "text-catalogs".into()).unwrap();
    // Duplicate encoded literal bytes are deliberately unusable: values must
    // come from typed decoded glyph/command/terminator/padding fields.
    for map in &typed.programs {
        let p = &mut typed.texts.archive_catalogs[map.category].languages[map.language].programs
            [map.item];
        if let Some(encoded) = &mut p.encoded {
            for span in &map.map.literals {
                encoded[span.encoded_offset..span.encoded_offset + span.length].fill(0xcd);
            }
        }
    }
    let mut resources = RuntimeResources::new(
        ffta_rom::digest(&rom),
        "synthetic".into(),
        "obj".into(),
        Version::Europe,
        super::super::tests::dimensions(),
    );
    resources.text_catalogs = Some(Box::new(typed));
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
    let stats = resources.text_catalog_stats().unwrap();
    assert!([
        stats.pointer_table_bytes,
        stats.string_bytes,
        stats.archive_index_bytes,
        stats.header_bytes,
        stats.encoded_control_bytes,
        stats.decoded_program_bytes,
        stats.glyph_bytes,
        stats.width_bytes
    ]
    .iter()
    .all(|&v| v > 0));
    let at = sources[0].offset as usize - 1;
    let edge = resources
        .text_catalogs
        .as_ref()
        .unwrap()
        .guest_read(at, &decoded.bus.rom, Some([0xa5; 4]))
        .unwrap();
    assert_eq!(edge, [0xa5, rom[at + 1], rom[at + 2], rom[at + 3]]);
    decoded.bus.real_bios = false;
    let mut legacy = Machine::new(rom.clone());
    for t in [&mut legacy, &mut decoded] {
        t.bus
            .write32(0x040000d4, 0x08000000 + text_short::GLYPH_ROOT + 128);
        t.bus.write32(0x040000d8, 0x06000020);
        t.bus.write16(0x040000dc, 32);
        t.bus.write16(0x040000de, 0x8400);
    }
    for s in &sources {
        decoded.bus.rom[s.offset as usize..(s.offset + s.length) as usize]
            .copy_from_slice(&rom[s.offset as usize..(s.offset + s.length) as usize]);
    }
    assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    for t in [&mut legacy, &mut decoded] {
        t.cpu.cpsr = 0xbf;
        t.cpu.regs[15] = 0x02000000;
        t.cpu.regs[0] = 0x08000000 + model::POINTER_ROOTS[0][0];
        t.cpu.regs[1] = 0x02015000;
        t.cpu.regs[2] = 4;
        t.bus.write16(0x02000000, 0xdf0b);
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
    let stats = resources.text_catalog_stats().unwrap();
    decoded.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(
        decoded.bus.rom_resources().unwrap(),
        &resources
    ));
    assert_eq!(resources.text_catalog_stats().unwrap(), stats);
}
