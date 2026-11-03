use super::*;
use crate::{capi::RT_API, Bus, Machine};

#[test]
fn typed_packets_merge_palette_edges_cover_last_page_and_preserve_swis_dma_and_owner() {
    let dims = super::super::tests::dimensions();
    let mut rom = vec![0x53; 0x1000000];
    let palette_offset = 0x76c800;
    let mut palette = vec![0, 32, 0, 0];
    palette.extend((0..16u16).flat_map(|word| (word | 0x8100).to_le_bytes()));
    rom[palette_offset..palette_offset + palette.len()].copy_from_slice(&palette);
    let mut paired = vec![16, 0, 0, 128, 0, 0, 2, 0, 1, 0, 0];
    paired.resize(16, 0);
    paired.extend_from_slice(&[7, 0, 0x23, 0x81, 0x56, 0x84]);
    let halfwords = vec![0, 0, 4, 0, 0x23, 0x81, 0x67, 0x45, 0, 0];
    let offsets = [0x895020usize, palette_offset + palette.len(), 0x76d000];
    let mut ranges = Vec::new();
    let mut families = Vec::new();
    for (index, role) in Role::ALL.into_iter().enumerate() {
        let body = if role == Role::Paired {
            &paired
        } else {
            &halfwords
        };
        let mut encoded = vec![if index == 1 { 1 } else { 0x11 }, 255, 255, 255];
        if index == 1 {
            encoded.extend_from_slice(body);
        } else {
            encoded.extend_from_slice(&((body.len() as u32) << 8 | 0x10).to_le_bytes());
            for chunk in body.chunks(8) {
                encoded.push(0);
                encoded.extend_from_slice(chunk);
            }
        }
        let program =
            bg_packet_data::Program::decode(&encoded, role, 0x5000 + index as u32 * 0x100)
                .unwrap()
                .0;
        rom[offsets[index]..offsets[index] + encoded.len()].copy_from_slice(&encoded);
        ranges.push(SourceRange {
            offset: offsets[index] as u32,
            length: encoded.len() as u32,
            sha256: ffta_rom::digest(&encoded),
            role: format!("bg-packet-program/{}/0", role.name()),
        });
        families.push(bg_packet_data::Family {
            descriptor_programs: vec![0],
            programs: vec![program],
        });
    }
    let programs = Programs {
        paired: families.remove(0),
        halfwords: families.remove(0),
        auxiliary: families.remove(0),
    };
    let mut resources = RuntimeResources::new(
        ffta_rom::digest(&rom),
        "synthetic".into(),
        "obj".into(),
        Version::Europe,
        dims,
    );
    resources.packet_programs = Some(Box::new(
        PacketResources::new(programs, &ranges, "packets".into()).unwrap(),
    ));
    let block = bg_palettes::decode(&palette).unwrap();
    resources.palettes = Some(Box::new(
        super::super::PaletteResources::new(
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
    spans.push((palette_offset, palette.len()));
    for &(at, len) in &spans {
        decoded.bus.rom[at..at + len].fill(0xcd);
    }
    let clock = decoded.bus.clock;
    for real_bios in [false, true] {
        decoded.bus.real_bios = real_bios;
        for mirror in [0x08000000u32, 0x0a000000, 0x0c000000] {
            for &(at, len) in &spans {
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
    assert_eq!(decoded.bus.clock, clock);
    assert!(resources.packet_stats().unwrap().typed_value_bytes > 0);
    assert!(resources.palette_stats().unwrap().source_bytes > 0);
    decoded.bus.real_bios = false;
    for &(at, len) in &spans {
        decoded.bus.rom[at..at + len].copy_from_slice(&rom[at..at + len]);
    }
    let mut legacy = Machine::new(rom.clone());
    for swi in [0x11u16, 0x12] {
        for (index, body) in [(0, &paired), (2, &halfwords)] {
            for dest in [0x02012000u32, 0x06000020] {
                for target in [&mut legacy, &mut decoded] {
                    target.cpu.cpsr = 0xbf;
                    target.cpu.regs[15] = 0x02000000;
                    target.cpu.regs[0] = 0x0a000004 + offsets[index] as u32;
                    target.cpu.regs[1] = dest;
                    target.bus.write16(0x02000000, 0xdf00 | swi);
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
                let actual = if dest >> 24 == 2 {
                    &decoded.bus.ewram[0x12000..0x12000 + body.len()]
                } else {
                    &decoded.bus.vram[0x20..0x20 + body.len()]
                };
                assert_eq!(actual, body);
            }
        }
    }
    assert_eq!(resources.packet_stats().unwrap().lz77_calls, 8);
    for &(at, len) in &spans {
        decoded.bus.rom[at..at + len].fill(0xcd);
    }
    for target in [&mut legacy, &mut decoded] {
        target
            .bus
            .write32(0x040000d4, 0x08000000 + offsets[1] as u32 - 2);
        target.bus.write32(0x040000d8, 0x02014000);
        target.bus.write16(0x040000dc, 6);
        target.bus.write16(0x040000de, 0x8000);
    }
    for &(at, len) in &spans {
        decoded.bus.rom[at..at + len].copy_from_slice(&rom[at..at + len]);
    }
    assert_eq!(legacy.save_state().unwrap(), decoded.save_state().unwrap());
    assert!(decoded.bus.decoded_packet_body(0x02000000).is_none());
    assert!(decoded
        .bus
        .decoded_packet_body(0x08000004 + offsets[1] as u32)
        .is_none());
    let read = decoded
        .bus
        .decoded_packet_body(0x08000004 + offsets[0] as u32)
        .unwrap();
    let snapshot = decoded.save_state().unwrap();
    let stats = resources.packet_stats().unwrap();
    decoded.load_state(&snapshot).unwrap();
    assert!(Arc::ptr_eq(
        decoded.bus.rom_resources().unwrap(),
        &resources
    ));
    assert_eq!(resources.packet_stats().unwrap(), stats);
    decoded.bus.set_rom_resources(None).unwrap();
    assert_eq!(read.bytes(), paired);
}
