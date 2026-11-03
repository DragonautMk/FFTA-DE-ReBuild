//! Reduced EUR offline catalog and original animation/DMA contract proof.
use super::*;
use gba_core::{
    rom_data::bg_transfers::{self, Catalog, Transfers},
    Bus, Machine,
};

#[test]
fn transfer_json_is_strict_and_preserves_typed_frame_order_and_words() {
    let control = bg_transfers::Control::decode(&[32, 0, 2, 0x81, 10, 0, 1, 0, 6, 0, 0, 0], 0x1000)
        .unwrap()
        .0;
    let value = Transfers {
        descriptor_links: vec![[
            Some(bg_transfers::Link {
                control: 0,
                tiles: 0,
                destination: 0x06000020,
            }),
            None,
            None,
            None,
            None,
        ]],
        variant_sources: vec![None],
        controls: vec![control],
        tiles: vec![bg_transfers::Tiles {
            relative_source: 0x2000,
            words: vec![0x81234567; 16],
        }],
    };
    value.validate().unwrap();
    let bytes = serde_json::to_vec_pretty(&value).unwrap();
    let parsed: Transfers = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed, value);
    assert_eq!(serde_json::to_vec_pretty(&parsed).unwrap(), bytes);
    let mut unknown = serde_json::to_value(value).unwrap();
    unknown["controls"][0]["unknown"] = true.into();
    assert!(serde_json::from_value::<Transfers>(unknown).is_err());
}

fn invoke(machine: &mut Machine, entry: u32, args: [u32; 3]) -> usize {
    machine.cpu.regs[..3].copy_from_slice(&args);
    machine.cpu.regs[13] = 0x03007c00;
    machine.cpu.regs[14] = 0x08000301;
    machine.cpu.regs[15] = entry;
    machine.cpu.cpsr = 0xbf;
    for steps in 1..=500_000 {
        machine.bus.frame_ready = false;
        machine.step();
        if machine.cpu.regs[15] == 0x08000300 {
            assert_eq!(machine.cpu.regs[13], 0x03007c00);
            return steps;
        }
    }
    panic!(
        "original transfer producer {entry:08x} failed to return at {:08x}",
        machine.cpu.regs[15]
    );
}

#[test]
#[ignore = "requires exact EUR ROM, retained initialized snapshot/static inventory, absent FFTA_STATE_TEST_TRANSFER_ROOT; Wine-only new checks"]
fn european_transfer_catalog_matches_original_ticks_and_split_dma() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_TRANSFER_ROOT").unwrap());
    assert!(!root.exists(), "preserve original animation evidence");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let catalog = Catalog::from_rom(&rom).unwrap();
    let value = &catalog.decoded;
    assert_eq!(value.descriptor_links.len(), 162);
    assert_eq!(value.controls.len(), 42);
    assert_eq!(value.tiles.len(), 34);
    assert_eq!(
        value.descriptor_links.iter().flatten().flatten().count(),
        97
    );
    assert!(value.variant_sources.iter().all(Option::is_none));
    let bytes = serde_json::to_vec_pretty(value).unwrap();
    let parsed: Transfers = serde_json::from_slice(&bytes).unwrap();
    parsed.verify_source(&rom).unwrap();
    assert_eq!(serde_json::to_vec_pretty(&parsed).unwrap(), bytes);
    let mut forged = parsed.clone();
    forged.tiles[0].words[0] ^= 0x80000000;
    forged.validate().unwrap();
    assert!(forged.verify_source(&rom).is_err());
    // Independent retained static classifier, not a second run of the old
    // graphics decoder or a new recording. Compare every source and alias.
    let inventory: serde_json::Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_TRANSFER_INVENTORY").unwrap()).unwrap(),
    )
    .unwrap();
    let base = 0x7662a8u32;
    for (id, links) in value.descriptor_links.iter().enumerate() {
        for (slot, link) in links.iter().enumerate() {
            let row = inventory["transfers"][slot]["rows"]
                .as_array()
                .unwrap()
                .iter()
                .find(|row| row["descriptor"].as_u64() == Some(id as u64));
            assert_eq!(row.is_some(), link.is_some());
            if let (Some(row), Some(link)) = (row, link) {
                assert_eq!(
                    row["control"].as_u64().unwrap(),
                    (base + value.controls[link.control as usize].relative_source) as u64
                );
                assert_eq!(
                    row["data"].as_u64().unwrap(),
                    (base + value.tiles[link.tiles as usize].relative_source) as u64
                );
                assert_eq!(
                    row["destination"].as_u64().unwrap(),
                    link.destination as u64
                );
            }
        }
    }
    let control_bytes: usize = value
        .controls
        .iter()
        .map(|c| c.source_bytes().unwrap().len())
        .sum();
    let tile_bytes: usize = value.tiles.iter().map(|t| t.words.len() * 4).sum();
    assert_eq!(control_bytes, 968);
    assert_eq!(tile_bytes, 199296);
    for source in &catalog.sources {
        let data = &rom[source.offset as usize..(source.offset + source.length) as usize];
        assert_eq!(gba_core::ffta_rom::digest(data), source.sha256);
    }
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("bg-transfer-programs.json"), &bytes).unwrap();
    let snapshot = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_STATE").unwrap());
    let mut machine = crate::make_machine(rom, None);
    crate::save_state::load(&mut machine, &snapshot).unwrap();
    let start = machine.save_state().unwrap();
    let first_with_count = |count| {
        value
            .descriptor_links
            .iter()
            .enumerate()
            .find_map(|(id, slots)| {
                slots.iter().enumerate().find_map(|(slot, link)| {
                    link.as_ref()
                        .filter(|link| value.controls[link.control as usize].frames.len() == count)
                        .map(|_| (id, slot))
                })
            })
            .unwrap()
    };
    let cases = [(19, 0), (4, 1), first_with_count(8), first_with_count(12)];
    let mut calls = 0;
    let mut instructions = 0;
    let mut reports = Vec::new();
    let mut split_calls = 0;
    const MANAGER: u32 = 0x02030000;
    const OBJECT: u32 = MANAGER + 4;
    for (descriptor, slot) in cases {
        machine.load_state(&start).unwrap();
        let link = value.descriptor_links[descriptor][slot].as_ref().unwrap();
        let control = &value.controls[link.control as usize];
        let tiles = &value.tiles[link.tiles as usize];
        let bank = tiles.source_bytes();
        instructions += invoke(&mut machine, 0x080220a4, [MANAGER, 0, 0]);
        calls += 1;
        instructions += invoke(
            &mut machine,
            0x08021f10,
            [
                0x08000000 + base + control.relative_source,
                link.destination,
                0x08000000 + base + tiles.relative_source,
            ],
        );
        calls += 1;
        assert_eq!(machine.cpu.regs[0], OBJECT);
        assert_eq!(machine.bus.read16(OBJECT + 2), 1);
        assert_eq!(machine.bus.read8(OBJECT + 4), 255);
        assert_eq!(machine.bus.read16(OBJECT + 8), control.transfer_bytes);
        assert_eq!(machine.bus.read16(OBJECT + 10), 1024);
        assert_eq!(
            machine.bus.read32(OBJECT + 16),
            0x08000000 + base + control.relative_source
        );
        assert_eq!(machine.bus.read32(OBJECT + 20), link.destination);
        assert_eq!(
            machine.bus.read32(OBJECT + 24),
            0x08000000 + base + tiles.relative_source
        );
        assert_eq!(
            machine.bus.read32(OBJECT + 32),
            0x08000000 + base + tiles.relative_source + control.frames[0].source_tile as u32 * 32
        );
        machine.bus.vram.fill(0x5a);
        let mut expected = machine.bus.vram.clone();
        let mut index = 255u8;
        let mut remaining = 1u16;
        let ticks = control
            .frames
            .iter()
            .map(|f| usize::from(f.duration_ticks))
            .sum::<usize>()
            * 2
            + 1;
        let mut changes = 0;
        let mut case_splits = 0;
        for _ in 0..ticks {
            instructions += invoke(&mut machine, 0x08021bd8, [0, 0, 0]);
            calls += 1;
            remaining = remaining.wrapping_sub(1);
            if remaining == 0 {
                index = index.wrapping_add(1);
                if index as usize == control.frames.len() {
                    index = 0;
                }
                remaining = control.frames[index as usize].duration_ticks;
                changes += 1;
                assert_eq!(machine.bus.read8(OBJECT + 6), 1);
                let source = control.frames[index as usize].source_tile as usize * 32;
                let destination = (link.destination - 0x06000000) as usize;
                let mut copied = 0;
                while copied < control.transfer_bytes as usize {
                    instructions += invoke(&mut machine, 0x08021d10, [0, 0, 0]);
                    calls += 1;
                    let length = (control.transfer_bytes as usize - copied).min(1024);
                    assert_eq!(machine.cpu.regs[0], length as u32);
                    expected[destination + copied..destination + copied + length]
                        .copy_from_slice(&bank[source + copied..source + copied + length]);
                    copied += length;
                    assert_eq!(machine.bus.read16(OBJECT + 12), copied as u16);
                    assert_eq!(
                        machine.bus.vram, expected,
                        "descriptor{descriptor} slot{slot} ordered DMA bytes"
                    );
                    if copied < control.transfer_bytes as usize {
                        case_splits += 1;
                    }
                }
                assert_eq!(machine.bus.read8(OBJECT + 6), 0);
            }
            assert_eq!(machine.bus.read8(OBJECT + 4), index);
            assert_eq!(machine.bus.read16(OBJECT + 2), remaining);
            assert_eq!(machine.bus.vram, expected, "unchanged tick VRAM");
        }
        split_calls += case_splits;
        reports.push(serde_json::json!({"descriptor":descriptor,"slot":slot,"frames":control.frames.len(),
            "transfer_bytes":control.transfer_bytes,"ticks":ticks,"frame_changes":changes,"split_chunks":case_splits,
            "original_countdown_and_frames":true,"ordered_full_vram_and_guards":true}));
    }
    assert!(split_calls > 0);
    let report = serde_json::json!({"region":"europe","descriptors":162,"active_links":97,"controls":42,"tile_banks":34,
        "absent_variants":162,"control_bytes":control_bytes,"tile_bytes":tile_bytes,"sources":catalog.sources,
        "json_sha256":gba_core::ffta_rom::digest(&bytes),"original_calls":calls,"original_instructions":instructions,"cases":reports,
        "snapshot_initializes_original_iwram_helpers":true,"no_new_recording":true,"linux":"pending","usa":"pending","performance_measurements":false});
    fs::write(
        root.join("result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("BG_TRANSFERS_EUR controls=42 tile_banks=34 links=97 variants=0 original_cases=4 calls={calls} instructions={instructions} ticks/frame_order/split_DMA/guards=pass");
}
