//! Newly rooted raw custom OBJ-related graphics: source/original producer proof.
use super::*;
use gba_core::{
    rom_data::obj_graphics::{Catalog, Graphic, Graphics, DIRECT_REFERENCES, TABLE_REFERENCES},
    Bus, Machine,
};
const SCRATCH: u32 = 0x02020010;
const STACK: u32 = 0x03007c00;
const RETURN: u32 = 0x08000300;
fn invoke(m: &mut Machine, entry: u32, args: [u32; 4], expected_end: Option<u32>) -> usize {
    m.cpu.regs[..4].copy_from_slice(&args);
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.regs[15] = entry;
    m.cpu.cpsr = 0xbf;
    let mut terminal = false;
    for n in 1..=2_000_000 {
        if m.cpu.regs[15] == 0x080058de {
            if let Some(end) = expected_end {
                assert_eq!(m.cpu.regs[7], end, "original exact encoded extent");
            }
            terminal = true;
        }
        m.bus.frame_ready = false;
        m.step();
        if m.cpu.regs[15] == RETURN {
            assert_eq!(m.cpu.regs[13], STACK);
            assert!(expected_end.is_none() || terminal);
            return n;
        }
    }
    panic!(
        "OBJ graphics original producer did not return at{:08x}",
        m.cpu.regs[15]
    );
}
#[test]
#[ignore = "exact EUR ROM/new independent graphics inventory; Wine, Linux/USA pending"]
fn european_obj_custom_graphics_match_sources_selectors_and_original_decode_copy() {
    let root =
        PathBuf::from(std::env::var_os("FFTA_STATE_TEST_OBJ_GRAPHICS_ORIGINAL_ROOT").unwrap());
    assert!(!root.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let graphics = &catalog.decoded;
    assert_eq!(
        (
            graphics.banks.len(),
            graphics.pointer_tables.len(),
            catalog.sources.len()
        ),
        (39, 6, 45)
    );
    assert_eq!(catalog.sources.iter().map(|s| s.length).sum::<u32>(), 62674);
    assert_eq!(
        graphics
            .banks
            .iter()
            .map(|b| b.words.len() * 4)
            .sum::<usize>(),
        131872
    );
    assert_eq!((DIRECT_REFERENCES.len(), TABLE_REFERENCES.len()), (13, 6));
    let encoded = serde_json::to_vec_pretty(graphics).unwrap();
    let parsed: Graphics = serde_json::from_slice(&encoded).unwrap();
    parsed.verify_source(&rom).unwrap();
    assert_eq!(serde_json::to_vec_pretty(&parsed).unwrap(), encoded);
    let inventory: serde_json::Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_OBJ_GRAPHICS_INVENTORY").unwrap()).unwrap(),
    )
    .unwrap();
    for case in 0..7 {
        let mut bad = graphics.clone();
        match case {
            0 => bad.banks[0].words[0] ^= 1,
            1 => bad.banks[0].encoded_source.push(0),
            2 => {
                bad.banks[0].encoded_source.pop();
            }
            3 => bad.banks[0].source_offset = u32::MAX,
            4 => bad.pointer_tables[0].bank_offsets[0] = 0,
            5 => bad.pointer_tables[0].source_offset += 2,
            _ => bad.banks.push(bad.banks[0].clone()),
        }
        assert!(bad.validate().is_err(), "malformed custom bank {case}");
    }
    let mut coherent = graphics.clone();
    let bank = &graphics.banks[0];
    let mut found = false;
    for at in 4..bank.encoded_source.len().min(128) {
        let mut bytes = bank.encoded_source.clone();
        bytes[at] ^= 1;
        if let Ok(changed) = Graphic::decode(&bytes, bank.source_offset) {
            if changed != *bank {
                coherent.banks[0] = changed;
                found = true;
                break;
            }
        }
    }
    assert!(found);
    coherent.validate().unwrap();
    assert!(coherent.verify_source(&rom).is_err());
    let mut swapped = graphics.clone();
    swapped.pointer_tables[0].bank_offsets.swap(0, 1);
    swapped.validate().unwrap();
    assert!(swapped.verify_source(&rom).is_err());
    let mut unknown: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    unknown["banks"][0]["unknown"] = serde_json::json!(0);
    assert!(serde_json::from_value::<Graphics>(unknown).is_err());
    let mut m = crate::make_machine(rom.clone(), None);
    m.bus.ffta_sprites.enabled = false;
    let mut instructions = 0;
    let mut decodes = 0;
    let mut selections = 0;
    let mut copies = 0;
    for (i, bank) in graphics.banks.iter().enumerate() {
        let decoded = bank.decoded_bytes();
        let inv = &inventory["banks"][i];
        assert_eq!(inv["source_offset"], bank.source_offset);
        assert_eq!(inv["source_length"], bank.encoded_source.len());
        assert_eq!(inv["decoded_sha256"], gba_core::ffta_rom::digest(&decoded));
        assert_eq!(
            bank.encoded_source,
            rom[bank.source_offset as usize
                ..bank.source_offset as usize + bank.encoded_source.len()]
        );
        for at in 0..32768 + 32 {
            m.bus.write8(SCRATCH - 16 + at, 0x5a);
        }
        instructions += invoke(
            &mut m,
            0x080054a0,
            [SCRATCH, 0x08000000 + bank.source_offset, 0, 0],
            Some(0x08000000 + bank.source_offset + bank.encoded_source.len() as u32),
        );
        assert_eq!(m.cpu.regs[0], decoded.len() as u32);
        assert!((0..decoded.len()).all(|i| m.bus.read8(SCRATCH + i as u32) == decoded[i]));
        assert!((0..16).all(|i| m.bus.read8(SCRATCH - 16 + i) == 0x5a));
        assert!(
            (decoded.len()..32768 + 16).all(|i| m.bus.read8(SCRATCH + i as u32) == 0x5a),
            "original scratch tail remains live/unowned"
        );
        decodes += 1;
    }
    // Actual canonical indexed source-argument chains, prepared language state.
    for reference in TABLE_REFERENCES {
        let table = graphics
            .pointer_tables
            .iter()
            .find(|t| t.source_offset == reference.table_offset)
            .unwrap();
        for language in 0..5u8 {
            m.bus.write8(0x02001f6e, language);
            m.cpu.regs[4] = SCRATCH;
            m.cpu.regs[5] = SCRATCH;
            m.cpu.regs[7] = SCRATCH;
            m.cpu.regs[8] = 0x02001f6e;
            match reference.table_offset {
                0x395588 => m.cpu.regs[4] = 0x02000000,
                0x39559c => m.cpu.regs[4] = 0x02001f6e,
                0x397d30 | 0x397d80 => m.cpu.regs[5] = 0x02001f6e,
                _ => {}
            }
            m.cpu.regs[13] = STACK;
            m.cpu.regs[15] = 0x08000000 + reference.reader;
            m.cpu.cpsr = 0xbf;
            let mut reached = false;
            for _ in 0..40 {
                if m.cpu.regs[15] == 0x08000000 + reference.call {
                    reached = true;
                    break;
                }
                m.bus.frame_ready = false;
                m.step();
                instructions += 1;
            }
            assert!(reached);
            assert_eq!(m.cpu.regs[0], SCRATCH);
            assert_eq!(m.cpu.regs[1], 0x08000000 + table.select(language).unwrap());
            selections += 1;
        }
    }
    // Isolated original upload protocols: exact full, partial prefix, offset and
    // longer-than-decoded copy. Other caller scratch transforms are not bypassed.
    let mut cases: Vec<_> = [0x340e8, 0x143d44, 0x3289a, 0x3e608]
        .into_iter()
        .map(|call| {
            let r = DIRECT_REFERENCES.iter().find(|r| r.call == call).unwrap();
            (
                r.source_offset,
                r.decoded_offset,
                r.destination,
                r.control,
                r.transfer_target,
            )
        })
        .collect();
    cases.push((0x431e18, 0, 0x06012480, 0x600, 0x145230));
    cases.push((0x43c7bc, 0x300, 0x06012700, 0xa0, 0x145230));
    for (source, offset, destination, control, entry) in cases {
        let bank = graphics
            .banks
            .iter()
            .find(|b| b.source_offset == source)
            .unwrap();
        let decoded = bank.decoded_bytes();
        let mut scratch = vec![0x5a; 32768];
        scratch[..decoded.len()].copy_from_slice(&decoded);
        for (i, &v) in scratch.iter().enumerate() {
            m.bus.write8(SCRATCH + i as u32, v);
        }
        m.bus.vram.fill(0x5a);
        let count = (control & 0x1fffff) as usize;
        let length = if entry == 0x14522c {
            ((count + 7) & !7) * 4
        } else {
            count * 2
        };
        let mut expected = vec![0x5a; 0x18000];
        let at = destination as usize - 0x06000000;
        expected[at..at + length]
            .copy_from_slice(&scratch[offset as usize..offset as usize + length]);
        instructions += invoke(
            &mut m,
            0x08000000 + entry,
            [SCRATCH + offset, destination, control, 0],
            None,
        );
        assert_eq!(m.bus.vram.as_slice(), expected);
        assert!((0..32768).all(|i| m.bus.read8(SCRATCH + i) == scratch[i as usize]));
        copies += 1;
    }
    assert_eq!((decodes, selections, copies), (39, 30, 6));
    assert_eq!(
        gba_core::ffta_rom::digest(&m.bus.rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("obj-graphics.json"), encoded).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"region":"europe","banks":39,"pointer_tables":6,"source_ranges":45,"source_bytes":62674,"decoded_bytes":131872,"original_decodes":decodes,"original_selector_chains":selections,"sampled_original_copies":copies,"instructions":instructions,"exact_source_cursor_output_guards_and_live_ram_tail":true,"strict_json_source_and_coherent_edit_guards":true,"runtime":"offline-only; managed/consumption pending","whole_upload_contexts_and_ordinary_scene":"pending","linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
    eprintln!("OBJ_GRAPHICS_EUR decodes={decodes} selectors={selections} copies={copies} instructions={instructions} source/JSON/original-scratch/VRAM/live-tail=pass");
}
