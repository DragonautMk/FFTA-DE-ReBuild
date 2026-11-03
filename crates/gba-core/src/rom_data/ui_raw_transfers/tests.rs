//! Isolated unit inputs only; no ordinary checkpoint or guest state is altered.
use super::*;
use crate::{Machine, bus::Bus};
use serde_json::{Value, json};
use std::{fs, path::PathBuf};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SelectorCase {
    root: u32,
    reader: u32,
    pointer_load: u32,
}
fn registers(m: &mut Machine, pc: u32) {
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    m.cpu.regs[13] = 0x03007d00;
    m.cpu.regs[14] = 0x03007f01;
    m.cpu.regs[15] = pc;
    m.cpu.cpsr = 0xbf;
}
fn run(m: &mut Machine) -> usize {
    let mut n = 0;
    while m.cpu.regs[15] != 0x03007f00 {
        assert!(n < 100_000, "pc={:08x}", m.cpu.regs[15]);
        m.bus.frame_ready = false;
        m.step();
        n += 1;
    }
    assert_eq!(m.cpu.regs[13], 0x03007d00);
    n
}
fn transfer(rom: &[u8], helper: u32, src: u32, dest: u32, count: u32, length: usize) -> usize {
    let mut m = Machine::new(rom.to_vec());
    m.bus.ffta_sprites.enabled = false;
    m.bus.ewram.fill(0x5a);
    m.bus.iwram.fill(0);
    m.bus.vram.fill(0xa5);
    // Use the authenticated original IWRAM callback code in an isolated unit machine.
    m.bus.iwram[0x5e78..0x5f28].copy_from_slice(&rom[0xd96838..0xd968e8]);
    let (mut ewram, mut iwram, mut vram, oam, palette) = (
        m.bus.ewram.clone(),
        m.bus.iwram.clone(),
        m.bus.vram.clone(),
        m.bus.oam.clone(),
        m.bus.palette.clone(),
    );
    let bytes = &rom[(src - 0x08000000) as usize..(src - 0x08000000) as usize + length];
    let queued = helper == 0xad0 || helper == 0xb14;
    registers(&mut m, 0x08000000 + helper);
    if queued {
        m.cpu.regs[0] = 0x02030000;
        m.cpu.regs[1] = src;
        m.cpu.regs[2] = dest;
        m.cpu.regs[3] = count;
    } else if helper == 0x145c14 {
        m.cpu.regs[0] = dest;
        m.cpu.regs[1] = src;
        m.cpu.regs[2] = count;
        m.cpu.regs[3] = 0x03005ee9;
    } else {
        m.cpu.regs[0] = src;
        m.cpu.regs[1] = dest;
        m.cpu.regs[2] = count;
    }
    let preserved = m.cpu.regs[4..12].to_vec();
    let mut n = run(&mut m);
    assert_eq!(&m.cpu.regs[4..12], preserved);
    if queued {
        let control = (if helper == 0xb14 {
            0x84000000
        } else {
            0x80000000
        }) | count;
        assert_eq!(m.bus.read32(0x03002bcc), 0x02030000);
        assert_eq!(m.bus.read32(0x02030004), src);
        assert_eq!(m.bus.read32(0x02030008), dest);
        assert_eq!(m.bus.read32(0x0203000c), control);
        assert!(m.bus.vram == vram, "complete vram guards"); // producer only queues, never performs the transfer.
        ewram[0x30000..0x3001c].copy_from_slice(&m.bus.ewram[0x30000..0x3001c]);
        assert_eq!(&ewram[0x30000..0x30004], &[0, 1, 0x5a, 0x5a]);
        registers(&mut m, 0x08000800);
        let preserved = m.cpu.regs[4..12].to_vec();
        n += run(&mut m);
        assert_eq!(&m.cpu.regs[4..12], preserved);
        assert_eq!(m.bus.read32(0x03002bcc), 0);
        assert_eq!(m.bus.ewram[0x30001], 0x81);
        ewram[0x30001] = 0x81;
    }
    if dest >> 24 == 2 {
        let at = (dest - 0x02000000) as usize;
        ewram[at..at + length].copy_from_slice(bytes);
    } else {
        let at = (dest - 0x06000000) as usize;
        vram[at..at + length].copy_from_slice(bytes);
    }
    assert!(m.bus.ewram == ewram, "complete ewram guards");
    // Only the actual helper/executor stack scratch may differ, including the16-byte DMA wait frame.
    let stack = if queued {
        36
    } else if helper == 0x145c14 {
        8
    } else {
        0
    };
    iwram[0x7d00 - stack..0x7d00].copy_from_slice(&m.bus.iwram[0x7d00 - stack..0x7d00]);
    assert!(m.bus.iwram == iwram, "complete iwram guards");
    assert!(m.bus.vram == vram, "complete vram guards");
    assert!(m.bus.oam == oam, "complete oam guards");
    assert!(m.bus.palette == palette, "complete palette guards");
    assert_eq!(m.bus.rom, rom);
    n
}

#[test]
#[ignore = "private EUR raw transfer model/original unit check; Wine only, Linux/USA pending"]
fn european_raw_transfer_views_match_sources_and_original_helpers() {
    let root = PathBuf::from(std::env::var_os("FFTA_RAW_TRANSFERS_MODEL_ROOT").unwrap());
    let out = root.join("windows-model-europe");
    assert!(!out.exists());
    fs::create_dir(&out).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let independent: RawTransfers =
        serde_json::from_slice(&fs::read(root.join("independent-model.json")).unwrap()).unwrap();
    let sources: Vec<SourceRange> =
        serde_json::from_slice(&fs::read(root.join("independent-sources.json")).unwrap()).unwrap();
    assert_eq!(*data, independent);
    assert_eq!(catalog.sources, sources);
    data.verify_source(&rom).unwrap();
    let spans = data.source_spans().unwrap();
    assert_eq!(spans.len(), 42);
    assert_eq!(spans.iter().map(|v| v.1.len()).sum::<usize>(), 38220);
    for (at, bytes, _) in &spans {
        assert_eq!(bytes, &rom[*at as usize..*at as usize + bytes.len()]);
        for (i, b) in bytes.iter().enumerate() {
            assert_eq!(data.byte_at(*at + i as u32), Some(*b));
        }
        for edge in [*at - 1, *at + bytes.len() as u32] {
            let expected = spans.iter().find_map(|(base, bytes, _)| {
                bytes.get(edge.checked_sub(*base)? as usize).copied()
            });
            assert_eq!(data.byte_at(edge), expected);
        }
    }
    for at in [0, 0x71e5e0, u32::MAX] {
        assert_eq!(data.byte_at(at), None);
    }
    for bank in data.banks.iter().filter(|b| b.relative_offset != 0) {
        for offset in [0, 0x1ff, 0x380, 0x400] {
            assert_eq!(data.byte_at(bank.bank_offset + offset), None);
        }
    }
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    assert_eq!(
        serde_json::from_slice::<RawTransfers>(&encoded).unwrap(),
        *data
    );
    fs::write(out.join("decoded.json"), &encoded).unwrap();
    let mut guards = 0;
    for kind in 0..11 {
        let mut bad = data.clone();
        match kind {
            0 => {
                bad.tables.pop();
            }
            1 => bad.tables.swap(0, 1),
            2 => bad.tables[0].source_offset += 4,
            3 => bad.tables[0].pointers[0].source_offset += 4,
            4 => bad.tables[0].pointers[0].bank_offset += 4,
            5 => {
                bad.banks.pop();
            }
            6 => bad.banks.swap(0, 1),
            7 => bad.banks[0].bank_offset += 4,
            8 => bad.banks[0].relative_offset += 4,
            9 => {
                bad.banks[0].words.pop();
            }
            _ => bad.banks.push(bad.banks[0].clone()),
        }
        assert!(bad.validate().is_err(), "shape{kind}");
        guards += 1;
    }
    for kind in 0..6 {
        let mut bad: Value = serde_json::from_slice(&encoded).unwrap();
        match kind {
            0 => bad["extra"] = json!(0),
            1 => bad["tables"][0]["extra"] = json!(0),
            2 => bad["banks"][0]["extra"] = json!(0),
            3 => bad["banks"][0]["words"][0] = json!(-1),
            4 => bad["tables"][0]["pointers"] = json!([]),
            _ => {
                bad["banks"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("relative_offset");
            }
        }
        assert!(serde_json::from_value::<RawTransfers>(bad).is_err());
        guards += 1;
    }
    let mut changed = data.clone();
    changed.banks[0].words[0] ^= 1;
    changed.validate().unwrap();
    assert!(changed.verify_source(&rom).is_err());
    guards += 1;
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    guards += 1;
    let mut wrong = rom.clone();
    wrong[0x12ee24] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    guards += 1;
    wrong.copy_from_slice(&rom);
    wrong[spans[0].0 as usize] ^= 1;
    assert!(Catalog::from_rom(&wrong).is_err());
    guards += 1;
    let cases: Vec<SelectorCase> =
        serde_json::from_slice(&fs::read(root.join("selector-cases.json")).unwrap()).unwrap();
    assert_eq!(cases.len(), 13);
    let mut m = Machine::new(rom.clone());
    m.bus.ffta_sprites.enabled = false;
    let mut selectors = 0;
    let mut instructions = 0;
    for c in cases {
        let table = data
            .tables
            .iter()
            .find(|v| v.source_offset == c.root)
            .unwrap();
        for language in [0u8, 1, 2, 3, 4, 5, 255] {
            m.bus.ewram.fill(0x5a);
            m.bus.ewram[0x1f6e] = language;
            registers(&mut m, 0x08000000 + c.reader);
            m.cpu.regs[4] = if c.reader == 0x3e616 {
                0x020020d0
            } else if c.reader == 0x23c10 || c.reader == 0x2cba6 {
                0x02001f6e
            } else {
                0
            };
            m.cpu.regs[8] = 0x02001f6e;
            let mut n = 0;
            while m.cpu.regs[15] != 0x08000000 + c.pointer_load + 2 {
                if m.cpu.regs[15] == 0x08000000 + c.pointer_load {
                    let op = u16::from_le_bytes(
                        rom[c.pointer_load as usize..c.pointer_load as usize + 2]
                            .try_into()
                            .unwrap(),
                    );
                    assert_eq!(
                        m.cpu.regs[usize::from((op >> 3) & 7)] + u32::from((op >> 6) & 31) * 4,
                        table.encoded_view(language)
                    );
                }
                assert!(n < 32);
                m.bus.frame_ready = false;
                m.step();
                n += 1;
            }
            let op = u16::from_le_bytes(
                rom[c.pointer_load as usize..c.pointer_load as usize + 2]
                    .try_into()
                    .unwrap(),
            );
            let at = (table.encoded_view(language) - 0x08000000) as usize;
            let expected = u32::from_le_bytes(rom[at..at + 4].try_into().unwrap());
            assert_eq!(m.cpu.regs[usize::from(op & 7)], expected);
            assert_eq!(
                table.select(language).map(|p| p.bank_offset),
                if language < 5 {
                    Some(expected - 0x08000000)
                } else {
                    None
                }
            );
            instructions += n;
            selectors += 1;
        }
    }
    assert_eq!(m.bus.rom, rom);
    let mut copies = 0;
    let mut queued = 0;
    for (i, table) in data.tables.iter().enumerate() {
        for pointer in &table.pointers {
            let base = 0x08000000 + pointer.bank_offset;
            let cases: Vec<(u32, u32, u32, u32, usize)> = match i {
                0 => vec![(0x145c14, base, 0x02010100, 224, 224)],
                1 => vec![(0x145c14, base, 0x020100e0, 160, 160)],
                2 => vec![(0x145230, base, 0x06016080, 0x180, 768)],
                3 => vec![
                    (0x14522c, base, 0x06016580, 0xd0, 832),
                    (0x14522c, base + 0x340, 0x06016580, 0xc0, 768),
                ],
                4 => vec![
                    (0xb14, base, 0x06013900, 0x100, 1024),
                    (0xb14, base + 0x400, 0x06013d00, 0x100, 1024),
                    (0x14522c, base, 0x06013900, 0x200, 2048),
                ],
                5 => vec![(0x14522c, base, 0x06015c00, 0x260, 2432)],
                _ => vec![
                    (0xad0, base + 0x200, 0x06013800, 0x40, 128),
                    (0xad0, base + 0x280, 0x06013800, 0x40, 128),
                    (0xad0, base + 0x300, 0x06013800, 0x40, 128),
                ],
            };
            for (helper, src, dest, count, len) in cases {
                instructions += transfer(&rom, helper, src, dest, count, len);
                copies += 1;
                if helper == 0xb14 || helper == 0xad0 {
                    queued += 1;
                }
            }
        }
    }
    assert_eq!(guards, 21);
    assert_eq!(copies, 60);
    assert_eq!(queued, 25);
    assert_eq!(selectors, 91);
    let result = json!({"status":"PASS-model-original-unit-transfers","guards":guards,"groups":7,"banks":35,"source_spans":42,"source_bytes":38220,"original_selector_prefixes":selectors,"original_helper_copies":copies,"original_queue_executor_copies":queued,"instructions_coverage_only":instructions,"all_source_words_exact":true,"all_memory_output_guards_exact":true,"SP_callee_exact":true,"final_ROM_exact":true,"isolated_unit_inputs_only":true,"fullcaller_ABI":"pending","dynamic_index_domain":"pending","ordinary":"pending","managed_catalog_unchanged":[1,92,91],"Linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false});
    fs::write(
        out.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    eprintln!(
        "ROM_DATA_RAW_TRANSFERS_MODEL banks35 groups7 source38220 guards{guards} selectors{selectors} copies{copies} queued{queued} PASS"
    );
}
