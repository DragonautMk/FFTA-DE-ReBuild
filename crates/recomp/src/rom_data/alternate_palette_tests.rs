//! Original alternate-palette producers against private bounded-decoder output.
use super::*;
use gba_core::{rom_data::bg_descriptors::Catalog, Bus};

#[test]
#[ignore = "requires exact ROM, alternate inventory and absent FFTA_STATE_TEST_ALT_PALETTE_ROOT"]
fn alternate_palette_catalog_matches_original_producers() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_ALT_PALETTE_ROOT").unwrap());
    assert!(!root.exists(), "preserve previous oracle evidence");
    let inventory =
        PathBuf::from(std::env::var_os("FFTA_STATE_TEST_ALT_PALETTE_INVENTORY").unwrap());
    let data: serde_json::Value =
        serde_json::from_slice(&fs::read(inventory.join("inventory.json")).unwrap()).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let hash = gba_core::ffta_rom::digest(&rom);
    assert_eq!(data["source_sha256"].as_str().unwrap(), hash);
    let catalog = Catalog::from_rom(&rom).unwrap();
    let (entry, consumer, literals, tables) = match hash.as_str() {
        gba_core::ffta_rom::SHA256 => (
            0x0801_a42c,
            0x0800_543c,
            [0x1a4f8usize, 0x1a514],
            [0x427a04u32, 0x42cc80],
        ),
        gba_core::ffta_rom::EUROPE_SHA256 => (
            0x0801_b59c,
            0x0800_54a0,
            [0x1b668usize, 0x1b684],
            [0x489bb0u32, 0x48ee2c],
        ),
        _ => panic!("exact supported ROM required"),
    };
    for (literal, table) in literals.into_iter().zip(tables) {
        assert_eq!(
            &rom[literal..literal + 4],
            &(0x0800_0000 + table).to_le_bytes()
        );
    }
    assert_eq!(data["consumer"].as_u64().unwrap(), consumer as u64);
    let keys: Vec<_> = catalog
        .decoded
        .entries
        .iter()
        .map(|e| e.alternate_palette_key)
        .collect();
    assert_eq!(keys.iter().filter(|&&key| key == 255).count(), 5);
    assert_eq!(
        keys.iter()
            .copied()
            .filter(|&key| key != 255)
            .collect::<std::collections::BTreeSet<_>>(),
        (0..157u8).collect()
    );
    let mut machine = crate::make_machine(rom, None);
    const RETURN: u32 = 0x0800_0300;
    const DEST: usize = 0x3860;
    let mut calls = 0u64;
    let mut instructions = 0u64;
    for environment in [1u8, 2, 0xfd, 0xfe] {
        machine.bus.write8(0x0200_2fc2, environment);
        let group = usize::from(environment & 3 == 1);
        for (id, &key) in keys.iter().enumerate() {
            if key == 255 {
                // The original producer has no local guard. The caller's
                // exclusion contract remains unknown; do not invoke it here.
                continue;
            }
            let row = &data["tables"][group]["sources"][key as usize];
            let offset = row["offset"].as_u64().unwrap() as usize;
            let length = row["source_length"].as_u64().unwrap() as usize;
            let table = tables[group] as usize;
            assert_eq!(
                offset,
                table
                    + u16::from_le_bytes(
                        machine.bus.rom[table + key as usize * 2..table + key as usize * 2 + 2]
                            .try_into()
                            .unwrap()
                    ) as usize
            );
            assert_eq!(
                gba_core::ffta_rom::digest(&machine.bus.rom[offset..offset + length]),
                row["source_sha256"].as_str().unwrap()
            );
            let bytes = fs::read(inventory.join(row["file"].as_str().unwrap())).unwrap();
            assert_eq!(bytes.len(), 256);
            assert_eq!(
                gba_core::ffta_rom::digest(&bytes),
                row["decoded_sha256"].as_str().unwrap()
            );
            machine.bus.write16(0x0200_7f10, id as u16);
            for argument in [id as u32, 0xffff] {
                machine.bus.iwram[DEST - 16..DEST + 512 + 16].fill(0xcd);
                let ewram = machine.bus.ewram.clone();
                machine.cpu.regs[0] = argument;
                machine.cpu.regs[1] = 0x0300_0000 + DEST as u32;
                machine.cpu.regs[13] = 0x0300_7c00;
                machine.cpu.regs[14] = RETURN | 1;
                machine.cpu.regs[15] = entry;
                machine.cpu.cpsr = 0xbf;
                let mut returned = false;
                let mut entered = false;
                for _ in 0..50_000 {
                    machine.bus.frame_ready = false;
                    if machine.cpu.regs[15] == consumer {
                        assert!(!entered);
                        entered = true;
                        assert_eq!(machine.cpu.regs[0], 0x0300_0000 + DEST as u32);
                        assert_eq!(machine.cpu.regs[1], 0x0800_0000 + offset as u32);
                    }
                    machine.step();
                    instructions += 1;
                    if machine.cpu.regs[15] == RETURN {
                        returned = true;
                        break;
                    }
                }
                assert!(
                    returned && entered,
                    "original producer must use its consumer and return"
                );
                assert_eq!(machine.cpu.regs[13], 0x0300_7c00);
                assert!(machine.bus.ewram == ewram);
                assert_eq!(
                    &machine.bus.iwram[DEST..DEST + 256],
                    bytes,
                    "id={id} environment={environment}"
                );
                assert!(machine.bus.iwram[DEST - 16..DEST]
                    .iter()
                    .all(|&b| b == 0xcd));
                assert!(machine.bus.iwram[DEST + 256..DEST + 512 + 16]
                    .iter()
                    .all(|&b| b == 0xcd));
                calls += 1;
            }
        }
    }
    assert_eq!(calls, 1256);
    assert_eq!(gba_core::ffta_rom::digest(&machine.bus.rom), hash);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "source_rom_sha256":hash,"calls":calls,"original_instructions":instructions,
        "valid_records":157,"tables":2,"source_streams":314,"output_bytes_each":256,
        "original_consumer_entered":true,"complete_output_and_guards_match":true,
        "ewram_unchanged":true,"stack_restored":true,"unavailable_records_not_invoked":5,
        "scope":"original-output inventory oracle; typed production codec/runtime migration pending"
    })).unwrap()).unwrap();
    eprintln!("ALT_PALETTE_ORIGINAL calls={calls} original_instructions={instructions} streams=314 guards=true ewram_unchanged=true");
}

#[test]
#[ignore = "requires exact ROM, retained alternate inventory and absent FFTA_STATE_TEST_ALT_PALETTE_ROOT"]
fn typed_alternate_palette_codec_matches_all_retained_sources_and_outputs() {
    use gba_core::rom_data::bg_alternate_palettes::{Catalog as AlternateCatalog, Palettes};
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_ALT_PALETTE_ROOT").unwrap());
    assert!(!root.exists(), "preserve prior codec observations");
    let inventory =
        PathBuf::from(std::env::var_os("FFTA_STATE_TEST_ALT_PALETTE_INVENTORY").unwrap());
    let data: serde_json::Value =
        serde_json::from_slice(&fs::read(inventory.join("inventory.json")).unwrap()).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let mut catalog = AlternateCatalog::from_rom(&rom).unwrap();
    let mut total = 0;
    for group in 0..2 {
        let table = &catalog.tables[group];
        let start = table.offset as usize;
        assert_eq!(table.length, 314);
        assert_eq!(
            table.offset as u64,
            data["tables"][group]["root"].as_u64().unwrap()
        );
        assert_eq!(
            table.sha256,
            data["tables"][group]["table_sha256"].as_str().unwrap()
        );
        let links: Vec<_> = catalog.decoded.relative_sources[group]
            .iter()
            .flat_map(|link| link.to_le_bytes())
            .collect();
        assert_eq!(rom[start..start + 314], links);
        for key in 0..157 {
            let expected = &data["tables"][group]["sources"][key];
            let range = &catalog.sources[group * 157 + key];
            let palette = &catalog.decoded.catalogs[group][key];
            assert_eq!(range.offset as u64, expected["offset"].as_u64().unwrap());
            assert_eq!(
                range.length as u64,
                expected["source_length"].as_u64().unwrap()
            );
            assert_eq!(range.sha256, expected["source_sha256"].as_str().unwrap());
            assert_eq!(
                palette.encoded_source,
                rom[range.offset as usize..(range.offset + range.length) as usize]
            );
            let bytes = fs::read(inventory.join(expected["file"].as_str().unwrap())).unwrap();
            assert_eq!(palette.source_bytes().as_slice(), bytes);
            palette.validate().unwrap();
            total += range.length;
        }
    }
    assert_eq!(total, 39933);
    assert_eq!(
        catalog
            .decoded
            .descriptor_keys
            .iter()
            .filter(|&&key| key == 255)
            .count(),
        5
    );
    for (id, key) in catalog.decoded.descriptor_keys.iter().enumerate() {
        for env in [0, 1, 2, 3, 0xfd, 0xfe] {
            assert_eq!(
                catalog.decoded.get(id, env).is_some(),
                *key != 255 && matches!(env & 3, 1 | 2)
            );
        }
    }
    let bytes = serde_json::to_vec_pretty(&catalog.decoded).unwrap();
    let decoded: Palettes = serde_json::from_slice(&bytes).unwrap();
    decoded.validate().unwrap();
    assert_eq!(decoded, catalog.decoded);
    assert_eq!(serde_json::to_vec_pretty(&decoded).unwrap(), bytes);
    catalog.decoded.catalogs[0][0].banks[0][0] ^= 0x8000;
    assert!(catalog.decoded.validate().is_err());
    let mut edited = rom;
    edited[catalog.sources[0].offset as usize] ^= 1;
    assert!(AlternateCatalog::from_rom(&edited).is_err());
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("palettes.json"), &bytes).unwrap();
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "streams":314,"source_bytes":total,"output_bytes":314*256,
            "retained_outputs_equal":true,"exact_extents":true,"source_authenticated":true,
            "typed_json_sha256":gba_core::ffta_rom::digest(&bytes),"typed_json_roundtrip":true,
            "runtime_consumption":false,"performance_measurements":false
        }))
        .unwrap(),
    )
    .unwrap();
}

#[test]
#[ignore = "requires exact FFTA_STATE_TEST_ROM; executes original consumer with synthetic streams"]
fn alternate_palette_synthetic_tokens_match_original_consumer() {
    use gba_core::rom_data::bg_alternate_palettes::{decode, OUTPUT_BYTES};
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let consumer = match gba_core::ffta_rom::digest(&rom).as_str() {
        gba_core::ffta_rom::SHA256 => 0x0800_543c,
        gba_core::ffta_rom::EUROPE_SHA256 => 0x0800_54a0,
        _ => panic!("exact ROM required"),
    };
    let mut cases: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
    for count in [1usize, 2, 3, 4, 63, 64] {
        let literal: Vec<_> = (0..count)
            .map(|i| (i as u8).wrapping_mul(37) | 0x80)
            .collect();
        let mut tokens = vec![0x40 | (count as u8 - 1)];
        tokens.extend_from_slice(&literal);
        cases.push((tokens, literal));
    }
    for count in [2usize, 3, 4, 32, 33] {
        cases.push((vec![0x20 | (count as u8 - 2)], vec![0; count]));
    }
    for count in [3usize, 4, 5, 255, 256] {
        for token in [1, 2] {
            cases.push((
                vec![token, (count - 3) as u8],
                vec![if token == 1 { 255 } else { 0 }; count],
            ));
        }
    }
    for distance in [1usize, 2, 3, 4, 7, 8, 63, 64] {
        let prefix: Vec<_> = (0..distance).map(|i| (i * 37 + 0x81) as u8).collect();
        let mut seed = vec![0x40 | (distance as u8 - 1)];
        seed.extend_from_slice(&prefix);
        for (token, length) in [
            (vec![0x80, (distance - 1) as u8], 3),
            (vec![0xf8, (distance - 1) as u8], 18),
            (vec![0x10, 0, (distance - 1) as u8], 4),
            (vec![0x1f, 0xc0, (distance - 1) as u8], 67),
            (vec![0, 0, 0, (distance - 1) as u8], 5),
            (
                vec![
                    0,
                    (OUTPUT_BYTES - distance - 5) as u8,
                    0,
                    (distance - 1) as u8,
                ],
                OUTPUT_BYTES - distance,
            ),
        ] {
            let mut expected = prefix.clone();
            for _ in 0..length {
                expected.push(expected[expected.len() - distance]);
            }
            let mut tokens = seed.clone();
            tokens.extend(token);
            cases.push((tokens, expected));
        }
    }
    let mut controls: Vec<_> = (3..=15u8).collect();
    controls.extend([0x40, 0xff]);
    cases.push((controls, vec![0xff]));
    let mut machine = crate::make_machine(rom, None);
    let return_pc = 0x0800_0300;
    let mut calls = 0;
    for (tokens, expected) in &cases {
        let mut source = (expected.len() as u32).to_be_bytes().to_vec();
        source.extend(tokens);
        let block = decode(&source, OUTPUT_BYTES).unwrap();
        assert_eq!(block.bytes, *expected);
        assert_eq!(block.consumed, source.len());
        for source_alignment in 0..4 {
            for destination_alignment in 0..4 {
                let start = 0x1000 + source_alignment;
                let dest = 0x3800 + destination_alignment;
                machine.bus.iwram.fill(0xcd);
                machine.bus.iwram[start..start + source.len()].copy_from_slice(&source);
                let ewram = machine.bus.ewram.clone();
                machine.cpu.regs[0] = 0x0300_0000 + dest as u32;
                machine.cpu.regs[1] = 0x0300_0000 + start as u32;
                machine.cpu.regs[13] = 0x0300_7c00;
                machine.cpu.regs[14] = return_pc | 1;
                machine.cpu.regs[15] = consumer;
                machine.cpu.cpsr = 0xbf;
                let mut finished = false;
                let mut consumed = None;
                for _ in 0..50_000 {
                    machine.bus.frame_ready = false;
                    // The end label precedes register restoration. r7 is the
                    // original consumer's live source cursor, not a guessed bound.
                    if machine.cpu.regs[15] == consumer + 0x43e {
                        consumed = Some(machine.cpu.regs[7] - (0x0300_0000 + start as u32));
                    }
                    machine.step();
                    if machine.cpu.regs[15] == return_pc {
                        finished = true;
                        break;
                    }
                }
                assert!(finished);
                assert_eq!(consumed, Some(source.len() as u32));
                assert_eq!(machine.cpu.regs[0], expected.len() as u32);
                assert_eq!(machine.cpu.regs[13], 0x0300_7c00);
                assert!(machine.bus.ewram == ewram);
                assert_eq!(&machine.bus.iwram[dest..dest + expected.len()], expected);
                assert!(machine.bus.iwram[dest - 16..dest]
                    .iter()
                    .all(|&b| b == 0xcd));
                assert!(
                    machine.bus.iwram[dest + expected.len()..dest + expected.len() + 16]
                        .iter()
                        .all(|&b| b == 0xcd)
                );
                assert_eq!(&machine.bus.iwram[start..start + source.len()], &source);
                calls += 1;
            }
        }
    }
    eprintln!("ALT_PALETTE_SYNTHETIC cases={} calls={calls} all_source_destination_alignments=true exact_cursor=true guards=true",cases.len());
}

#[test]
#[ignore = "requires exact FFTA_STATE_TEST_ROM; bounded original malformed-stream probe"]
fn alternate_palette_original_overruns_declared_size_but_offline_decoder_refuses() {
    use gba_core::rom_data::bg_alternate_palettes::decode;
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let consumer = match gba_core::ffta_rom::digest(&rom).as_str() {
        gba_core::ffta_rom::SHA256 => 0x0800_543c,
        gba_core::ffta_rom::EUROPE_SHA256 => 0x0800_54a0,
        _ => panic!("exact ROM required"),
    };
    let mut machine = crate::make_machine(rom, None);
    // Header claims one byte; the complete literal token writes two. The
    // strict offline decoder must reject it instead of silently clipping.
    let source = [0, 0, 0, 1, 0x41, 0x81, 0xff];
    assert!(decode(&source, 256).is_err());
    machine.bus.iwram[0x1000..0x1000 + source.len()].copy_from_slice(&source);
    machine.bus.iwram[0x3800..0x3820].fill(0xcd);
    machine.cpu.regs[0] = 0x0300_3800;
    machine.cpu.regs[1] = 0x0300_1000;
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = consumer;
    machine.cpu.cpsr = 0xbf;
    let mut returned = false;
    for _ in 0..5000 {
        machine.step();
        if machine.cpu.regs[15] == 0x0800_0300 {
            returned = true;
            break;
        }
    }
    assert!(returned);
    assert_eq!(machine.cpu.regs[0], 1);
    assert_eq!(&machine.bus.iwram[0x3800..0x3802], &[0x81, 0xff]);
    assert!(machine.bus.iwram[0x3802..0x3820].iter().all(|&b| b == 0xcd));
    eprintln!("ALT_PALETTE_NEGATIVE declared=1 original_written=2 offline_rejected=true");
}
