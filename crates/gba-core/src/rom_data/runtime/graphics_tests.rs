use super::*;
use crate::{capi::RT_API, Bus, Machine};

#[test]
fn typed_graphics_cover_first_pages_merge_edges_and_preserve_dma_swis_and_owner() {
    let root = 0x7662a8usize;
    let starts = [0x769a58usize, 0x76a020, 0x76b020, 0x76c020, 0x895020];
    let mut rom = vec![0x53; 0x1000000];
    let mut graphics = bg_graphics::Graphics {
        descriptor_graphics: (0..5u16).chain(std::iter::repeat_n(0, 157)).collect(),
        graphics: Vec::new(),
    };
    let mut ranges = Vec::new();
    for (row, &start) in starts.iter().enumerate() {
        let mut encoded = Vec::new();
        if row % 2 == 1 {
            encoded.extend_from_slice(&(32u32 << 8 | 0x22).to_le_bytes());
        }
        encoded.extend_from_slice(&(32u32 << 8 | 0x20).to_le_bytes());
        encoded.extend_from_slice(&32u32.to_be_bytes());
        encoded.push(0x5f);
        encoded.extend((0..32u8).map(|byte| byte.wrapping_mul(37).wrapping_add(row as u8)));
        rom[start..start + encoded.len()].copy_from_slice(&encoded);
        ranges.push(SourceRange {
            offset: start as u32,
            length: encoded.len() as u32,
            sha256: ffta_rom::digest(&encoded),
            role: format!("bg-main-graphics/{row}"),
        });
        graphics.graphics.push(
            bg_graphics::Graphic::decode(&encoded, (start - root) as u32)
                .unwrap()
                .0,
        );
    }
    let mut main = GraphicsResources::new(graphics, &ranges, "graphics".into()).unwrap();
    // Corrupt the duplicated literal metadata as well as raw ROM after binding:
    // returning encoded_source alone must fail this provenance check.
    for (graphic, positions) in main.graphics.graphics.iter_mut().zip(&main.literals) {
        for (at, &output) in positions.iter().enumerate() {
            if output != u16::MAX {
                graphic.encoded_source[at] ^= 0xff;
            }
        }
    }
    let palette_offset = starts[3] + ranges[3].length as usize;
    let mut palette = vec![0, 32, 0, 0];
    palette.extend((0..16u16).flat_map(|word| (word | 0x8100).to_le_bytes()));
    rom[palette_offset..palette_offset + palette.len()].copy_from_slice(&palette);
    let absent = bg_descriptors::TransferLink {
        control_source: 0,
        data_source: 0,
        destination: 0,
    };
    let descriptor = bg_descriptors::Descriptor {
        graphics_source: (starts[0] - root) as u32,
        paired_planes_source: 0x5000,
        halfword_planes_source: 0x5100,
        palette_source: (palette_offset - root) as u32,
        auxiliary_patches_source: 0x5200,
        transfer: absent.clone(),
        extra_transfers: std::array::from_fn(|_| absent.clone()),
        variant_source: 0,
        selectors: [0, 255],
        alternate_palette_key: 0,
        trailing_byte: 0,
    };
    let descriptors = bg_descriptors::Descriptors {
        entries: vec![descriptor; 162],
    };
    let metadata = descriptors.source_bytes().unwrap();
    rom[root..root + metadata.len()].copy_from_slice(&metadata);
    let mut resources = RuntimeResources::new(
        ffta_rom::digest(&rom),
        "synthetic".into(),
        "obj".into(),
        Version::Europe,
        super::super::tests::dimensions(),
    );
    resources.main_graphics = Some(Box::new(main));
    resources.background = Some(Box::new(
        BackgroundResources::new(root, descriptors, "background".into()).unwrap(),
    ));
    let block = bg_palettes::decode(&palette).unwrap();
    resources.palettes = Some(Box::new(
        PaletteResources::new(
            bg_palettes::Palettes {
                entries: vec![block.palette],
            },
            &[bg_palettes::PaletteSource {
                range: SourceRange {
                    offset: palette_offset as u32,
                    length: palette.len() as u32,
                    sha256: ffta_rom::digest(&palette),
                    role: "palette".into(),
                },
                encoding: block.encoding,
            }],
            "palettes".into(),
        )
        .unwrap(),
    ));
    let resources = Arc::new(resources);
    let mut decoded = Machine::new(rom.clone());
    decoded
        .bus
        .set_rom_resources(Some(resources.clone()))
        .unwrap();
    let mut spans: Vec<_> = ranges
        .iter()
        .map(|r| (r.offset as usize, r.length as usize))
        .collect();
    spans.extend([(root, metadata.len()), (palette_offset, palette.len())]);
    for &(at, len) in &spans {
        decoded.bus.rom[at..at + len].fill(0xcd);
    }
    for real_bios in [false, true] {
        decoded.bus.real_bios = real_bios;
        for mirror in [0x08000000u32, 0x0a000000, 0x0c000000] {
            for &(at, len) in &spans[..5] {
                for index in at - 3..at + len + 3 {
                    let pointer = &mut decoded as *mut Machine as *mut std::ffi::c_void;
                    assert_eq!(decoded.bus.read8(mirror + index as u32), rom[index]);
                    assert_eq!(
                        decoded.bus.read16(mirror + index as u32),
                        u16::from_le_bytes(rom[index..index + 2].try_into().unwrap())
                    );
                    assert_eq!(
                        (RT_API.read32)(pointer, mirror + index as u32),
                        u32::from_le_bytes(rom[index..index + 4].try_into().unwrap())
                    );
                }
            }
        }
    }
    assert!(resources.graphics_stats().unwrap().decoded_literal_bytes > 0);
    assert!(resources.background_stats().unwrap().bus_reads > 0);
    assert!(resources.palette_stats().unwrap().source_bytes > 0);
    decoded.bus.real_bios = false;
    let mut legacy = Machine::new(rom.clone());
    for target in [&mut legacy, &mut decoded] {
        target.bus.write32(
            0x040000d4,
            0x08000000 + starts[3] as u32 + ranges[3].length - 2,
        );
        target.bus.write32(0x040000d8, 0x02014000);
        target.bus.write16(0x040000dc, 6);
        target.bus.write16(0x040000de, 0x8000);
    }
    for &(at, len) in &spans {
        decoded.bus.rom[at..at + len].copy_from_slice(&rom[at..at + len]);
    }
    assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    for target in [&mut legacy, &mut decoded] {
        target.cpu.cpsr = 0xbf;
        target.cpu.regs[15] = 0x02000000;
        target.cpu.regs[0] = 0x0a000000 + starts[1] as u32;
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
    let stats = resources.graphics_stats().unwrap();
    decoded.load_state(&state).unwrap();
    assert!(Arc::ptr_eq(
        decoded.bus.rom_resources().unwrap(),
        &resources
    ));
    assert_eq!(resources.graphics_stats().unwrap(), stats);
}
