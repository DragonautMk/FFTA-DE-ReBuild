use super::*;
use crate::{capi::RT_API, Bus, Machine};

#[test]
fn typed_transfer_controls_tiles_merge_edges_cover_last_page_and_preserve_dma_load() {
    let root = 0x7662a8usize;
    let main_start = 0x778f00usize;
    let mut main_encoded = (32u32 << 8 | 0x20).to_le_bytes().to_vec();
    main_encoded.extend_from_slice(&32u32.to_be_bytes());
    main_encoded.push(0x5f);
    main_encoded.extend(0..32u8);
    let c0 = main_start + main_encoded.len();
    let control0 = bg_transfers::Control::decode(
        &[32, 0, 2, 0x81, 0, 0, 1, 0, 16, 0, 0, 0],
        (c0 - root) as u32,
    )
    .unwrap()
    .0;
    let t0 = c0 + control0.source_bytes().unwrap().len();
    let t1 = 0x897fe0usize;
    let c1 = t1 - 8;
    let control1 = bg_transfers::Control::decode(&[32, 0, 1, 0, 8, 0, 0, 0], (c1 - root) as u32)
        .unwrap()
        .0;
    let values = bg_transfers::Transfers {
        descriptor_links: vec![
            [
                Some(bg_transfers::Link {
                    control: 0,
                    tiles: 0,
                    destination: 0x06000020
                }),
                Some(bg_transfers::Link {
                    control: 1,
                    tiles: 1,
                    destination: 0x06000040
                }),
                None,
                None,
                None
            ];
            2
        ],
        variant_sources: vec![None; 2],
        controls: vec![control0, control1],
        tiles: vec![
            bg_transfers::Tiles {
                relative_source: (t0 - root) as u32,
                words: vec![0x81234567; 16],
            },
            bg_transfers::Tiles {
                relative_source: (t1 - root) as u32,
                words: vec![0xfedcba98; 8],
            },
        ],
    };
    let mut rom = vec![0x53; 0x1000000];
    let mut sources = Vec::new();
    for (kind, rows) in [
        (
            "control",
            values
                .controls
                .iter()
                .map(|c| (root + c.relative_source as usize, c.source_bytes().unwrap()))
                .collect::<Vec<_>>(),
        ),
        (
            "tiles",
            values
                .tiles
                .iter()
                .map(|t| (root + t.relative_source as usize, t.source_bytes()))
                .collect(),
        ),
    ] {
        for (id, (at, bytes)) in rows.into_iter().enumerate() {
            rom[at..at + bytes.len()].copy_from_slice(&bytes);
            sources.push(SourceRange {
                offset: at as u32,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(&bytes),
                role: format!("bg-transfer-{kind}/{id}"),
            });
        }
    }
    rom[main_start..main_start + main_encoded.len()].copy_from_slice(&main_encoded);
    let p0 = t0 + 64;
    let mut palette = vec![0, 32, 0, 0];
    palette.extend((0..16u16).flat_map(|word| (word | 0x8100).to_le_bytes()));
    rom[p0..p0 + palette.len()].copy_from_slice(&palette);
    let main = bg_graphics::Graphic::decode(&main_encoded, (main_start - root) as u32)
        .unwrap()
        .0;
    let mut resources = RuntimeResources::new(
        ffta_rom::digest(&rom),
        "synthetic".into(),
        "obj".into(),
        Version::Europe,
        super::super::tests::dimensions(),
    );
    resources.transfers = Some(Box::new(
        TransferResources::new(values, &sources, "transfers".into()).unwrap(),
    ));
    resources.main_graphics = Some(Box::new(
        GraphicsResources::new(
            bg_graphics::Graphics {
                descriptor_graphics: vec![0, 0],
                graphics: vec![main],
            },
            &[SourceRange {
                offset: main_start as u32,
                length: main_encoded.len() as u32,
                sha256: ffta_rom::digest(&main_encoded),
                role: "bg-main-graphics/0".into(),
            }],
            "graphics".into(),
        )
        .unwrap(),
    ));
    let block = bg_palettes::decode(&palette).unwrap();
    resources.palettes = Some(Box::new(
        PaletteResources::new(
            bg_palettes::Palettes {
                entries: vec![block.palette],
            },
            &[bg_palettes::PaletteSource {
                range: SourceRange {
                    offset: p0 as u32,
                    length: palette.len() as u32,
                    sha256: ffta_rom::digest(&palette),
                    role: "palette".into(),
                },
                encoding: block.encoding,
            }],
            "palette".into(),
        )
        .unwrap(),
    ));
    let resources = Arc::new(resources);
    let mut decoded = Machine::new(rom.clone());
    decoded
        .bus
        .set_rom_resources(Some(resources.clone()))
        .unwrap();
    let mut spans = sources
        .iter()
        .map(|s| (s.offset as usize, s.length as usize))
        .collect::<Vec<_>>();
    spans.extend([(main_start, main_encoded.len()), (p0, palette.len())]);
    for &(at, len) in &spans {
        decoded.bus.rom[at..at + len].fill(0xcd);
    }
    let clock = decoded.bus.clock;
    for real_bios in [false, true] {
        decoded.bus.real_bios = real_bios;
        for mirror in [0x08000000u32, 0x0a000000, 0x0c000000] {
            for &(at, len) in &spans {
                for index in at - 3..at + len + 3 {
                    let ptr = &mut decoded as *mut Machine as *mut std::ffi::c_void;
                    assert_eq!(decoded.bus.read8(mirror + index as u32), rom[index]);
                    assert_eq!(
                        decoded.bus.read16(mirror + index as u32),
                        u16::from_le_bytes(rom[index..index + 2].try_into().unwrap())
                    );
                    assert_eq!(
                        (RT_API.read32)(ptr, mirror + index as u32),
                        u32::from_le_bytes(rom[index..index + 4].try_into().unwrap())
                    );
                }
            }
        }
    }
    assert_eq!(decoded.bus.clock, clock);
    let stats = resources.transfer_stats().unwrap();
    assert!(stats.control_bytes > 0 && stats.tile_bytes > 0);
    assert_eq!(stats.source_bytes, stats.control_bytes + stats.tile_bytes);
    assert!(
        resources.graphics_stats().unwrap().decoded_literal_bytes > 0
            && resources.palette_stats().unwrap().source_bytes > 0
    );
    decoded.bus.real_bios = false;
    let mut legacy = Machine::new(rom.clone());
    for target in [&mut legacy, &mut decoded] {
        target
            .bus
            .write32(0x040000d4, 0x08000000 + (t0 as u32 & !3));
        target.bus.write32(0x040000d8, 0x06000020);
        target.bus.write16(0x040000dc, 8);
        target.bus.write16(0x040000de, 0x8400);
    }
    for &(at, len) in &spans {
        decoded.bus.rom[at..at + len].copy_from_slice(&rom[at..at + len]);
    }
    assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    for target in [&mut legacy, &mut decoded] {
        target.cpu.cpsr = 0xbf;
        target.cpu.regs[15] = 0x02000000;
        target.cpu.regs[0] = 0x0a000000 + c0 as u32;
        target.cpu.regs[1] = 0x02015000;
        target.cpu.regs[2] = 4;
        target.bus.write16(0x02000000, 0xdf0b);
    }
    for &(at, len) in &spans {
        decoded.bus.rom[at..at + len].fill(0xcd);
    }
    legacy.step();
    decoded.step();
    for &(at, len) in &spans {
        decoded.bus.rom[at..at + len].copy_from_slice(&rom[at..at + len]);
    }
    assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    let state = decoded.save_state().unwrap();
    let stats = resources.transfer_stats().unwrap();
    decoded.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(
        decoded.bus.rom_resources().unwrap(),
        &resources
    ));
    assert_eq!(resources.transfer_stats().unwrap(), stats);
}
