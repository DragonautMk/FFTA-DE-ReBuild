//! All E605 effect descriptors and both original queued transfers on EUR.
use super::{
    attach_dataset,
    direct_byte_copy_tests::{flip, input, same_state},
    indexed_reader_bank_tests::{execute_until, initialize},
    read_dataset, BACKGROUND_EFFECT_GRAPHICS_ID,
};
use gba_core::{rom_data::SourceRange, Bus, Machine};
use serde::Deserialize;
use std::{fs, ops::Range, sync::Arc};

const ROM_BASE: u32 = 0x0800_0000;
const MIRRORS: [u32; 3] = [ROM_BASE, 0x0a00_0000, 0x0c00_0000];
const TASK: u32 = 0x0201_0000;
const MESSAGE: u32 = 0x0201_1000;
const PARAMETERS: u32 = 0x0201_1100;
const QUEUE: u32 = 0x0300_2bcc;
const RETURN: u32 = 0x0300_7f00;
const VRAM_BASE: u32 = 0x0600_0000;
const TILE_DESTINATION: u32 = 0x0600_8a40;
const MAP_DESTINATION: u32 = 0x0600_7000;
const MAP_ROWS: usize = 20;
const SOURCE_STRIDE: usize = 60;
const DESTINATION_STRIDE: usize = 64;
const SENTINEL: u8 = 0x5a;

#[derive(Deserialize)]
struct Span {
    offset: u32,
    length: u32,
}

impl Span {
    fn host_range(&self) -> Range<usize> {
        self.offset as usize..(self.offset + self.length) as usize
    }
}

#[derive(Deserialize)]
struct Descriptor {
    index: u8,
    descriptor_offset: u32,
    tiles: Span,
    tilemap: Span,
}

#[derive(Deserialize)]
struct Plan {
    descriptors_rows: Vec<Descriptor>,
}

fn execute(machine: &mut Machine, start: u32, stop: u32) {
    initialize(machine, start);
    machine.cpu.regs[6] = TASK;
    machine.cpu.regs[10] = MESSAGE;
    machine.cpu.regs[14] = RETURN | 1;
    execute_until(machine, stop, 100_000);
}

fn select_descriptor(machine: &mut Machine, index: u8, expected: u32) {
    machine.bus.write32(MESSAGE + 8, PARAMETERS);
    machine.bus.write8(PARAMETERS + 4, index);
    execute(machine, 0x0812_f9dc, 0x0812_fa16);
    assert_eq!(machine.bus.read32(TASK + 4), ROM_BASE + expected);
}

fn execute_transfers(machine: &mut Machine, rom: &[u8], row: &Descriptor) {
    select_descriptor(machine, row.index, row.descriptor_offset);
    let tile_offset = (TILE_DESTINATION - VRAM_BASE) as usize;
    let tile_end = tile_offset + row.tiles.length as usize;
    machine.bus.vram[tile_offset - 4..tile_end + 4].fill(SENTINEL);
    execute(machine, 0x0812_fa56, 0x0812_fa92);
    let first = TASK + 0x2c;
    let group = row.tiles.length / 4;
    assert_eq!(machine.bus.read32(QUEUE), first);
    for index in 0..4 {
        let node = first + 0x1c * index;
        assert_eq!(
            machine.bus.read32(node + 4),
            ROM_BASE + row.tiles.offset + group * index
        );
        assert_eq!(
            machine.bus.read32(node + 8),
            TILE_DESTINATION + group * index
        );
        assert_eq!(machine.bus.read32(node + 12), 0x8400_0000 | (group / 4));
        let next = if index < 3 { node + 0x1c } else { 0 };
        assert_eq!(machine.bus.read32(node + 24), next);
    }
    execute(machine, 0x0800_0800, RETURN);
    assert_eq!(machine.bus.read32(QUEUE), 0);
    assert_eq!(
        &machine.bus.vram[tile_offset..tile_end],
        &rom[row.tiles.host_range()]
    );
    assert_eq!(
        &machine.bus.vram[tile_offset - 4..tile_offset],
        &[SENTINEL; 4]
    );
    assert_eq!(&machine.bus.vram[tile_end..tile_end + 4], &[SENTINEL; 4]);

    let map_offset = (MAP_DESTINATION - VRAM_BASE) as usize;
    let map_end = map_offset + MAP_ROWS * DESTINATION_STRIDE;
    machine.bus.vram[map_offset - 4..map_end + 4].fill(SENTINEL);
    machine.bus.write32(0x0300_09a4, MAP_DESTINATION);
    execute(machine, 0x0812_faf0, 0x0812_fb0e);
    let node = TASK + 0x9c;
    assert_eq!(machine.bus.read32(QUEUE), node);
    assert_eq!(machine.bus.read32(node + 4), ROM_BASE + row.tilemap.offset);
    assert_eq!(machine.bus.read32(node + 8), MAP_DESTINATION);
    assert_eq!(machine.bus.read32(node + 12), 0x8000_001e);
    assert_eq!(machine.bus.read16(node + 16) as usize, MAP_ROWS);
    assert_eq!(machine.bus.read16(node + 18) as usize, SOURCE_STRIDE);
    assert_eq!(machine.bus.read16(node + 20) as usize, DESTINATION_STRIDE);
    assert_eq!(machine.bus.read32(node + 24), 0);
    execute(machine, 0x0800_0800, RETURN);
    assert_eq!(machine.bus.read32(QUEUE), 0);
    let map = &rom[row.tilemap.host_range()];
    assert_eq!(map.len(), MAP_ROWS * SOURCE_STRIDE);
    for (index, expected) in map.chunks_exact(SOURCE_STRIDE).enumerate() {
        let start = map_offset + index * DESTINATION_STRIDE;
        assert_eq!(&machine.bus.vram[start..start + SOURCE_STRIDE], expected);
        assert_eq!(
            &machine.bus.vram[start + SOURCE_STRIDE..start + DESTINATION_STRIDE],
            &[SENTINEL; DESTINATION_STRIDE - SOURCE_STRIDE],
        );
    }
    assert_eq!(
        &machine.bus.vram[map_offset - 4..map_offset],
        &[SENTINEL; 4]
    );
    assert_eq!(&machine.bus.vram[map_end..map_end + 4], &[SENTINEL; 4]);
}

fn flip_bank(machine: &mut Machine, spans: &[SourceRange]) {
    for span in spans {
        flip(machine, span.offset, span.length);
    }
}

fn check_boundaries(raw: &mut Machine, managed: &mut Machine, sources: &[SourceRange]) -> u64 {
    let owner = managed.bus.rom_resources().unwrap().clone();
    let before = owner.background_effect_graphics_bus_bytes().unwrap();
    let mut checks = 0;
    let mut managed_bytes = 0;
    for span in sources {
        for end in [span.offset, span.offset + span.length] {
            for offset in end - 3..=end + 3 {
                for base in MIRRORS {
                    let address = base + offset;
                    assert_eq!(managed.bus.read8(address), raw.bus.read8(address));
                    assert_eq!(managed.bus.read16(address), raw.bus.read16(address));
                    assert_eq!(managed.bus.read32(address), raw.bus.read32(address));
                    for width in [1, 2, 4] {
                        managed_bytes += (offset..offset + width)
                            .filter(|&lane| {
                                sources
                                    .iter()
                                    .any(|s| (s.offset..s.offset + s.length).contains(&lane))
                            })
                            .count() as u64;
                    }
                    checks += 3;
                }
            }
        }
    }
    assert_eq!(
        owner.background_effect_graphics_bus_bytes().unwrap() - before,
        managed_bytes
    );
    checks
}

#[test]
#[ignore = "Exact EUR ROM, authenticated E605 plan and managed e355/e356 datasets"]
fn european_background_effect_graphics_feed_original_dma() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    let plan: Plan =
        serde_json::from_slice(&fs::read(input("FFTA_BACKGROUND_EFFECT_PLAN")).unwrap()).unwrap();
    assert_eq!(plan.descriptors_rows.len(), 25);

    let mut predecessor = read_dataset(&input("FFTA_BACKGROUND_EFFECT_PREDECESSOR")).unwrap();
    let mut dataset = read_dataset(&input("FFTA_BACKGROUND_EFFECT_DATASET")).unwrap();
    assert_eq!(
        (
            predecessor.manifest.extractor_revision,
            predecessor.manifest.decoder_revision
        ),
        (355, 354)
    );
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (356, 355)
    );
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    raw.bus.ffta_sprites.enabled = false;
    managed.bus.ffta_sprites.enabled = false;
    attach_dataset(&mut managed, &predecessor).unwrap();
    assert!(managed
        .bus
        .rom_resources()
        .unwrap()
        .background_effect_graphics_bus_bytes()
        .is_none());
    attach_dataset(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();

    (
        predecessor.manifest.extractor_revision,
        predecessor.manifest.decoder_revision,
    ) = (356, 355);
    assert!(attach_dataset(&mut managed, &predecessor).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|item| item.id == BACKGROUND_EFFECT_GRAPHICS_ID)
        .unwrap();
    let spans = resource.sources.clone();
    let bytes = dataset.background_effect_graphics_bytes.as_mut().unwrap();
    let midpoint = bytes.len() / 2;
    bytes[midpoint] ^= 1;
    resource.sha256 = gba_core::ffta_rom::digest(bytes);
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    let initial = raw.save_state().unwrap();
    flip_bank(&mut managed, &spans);
    let mixed_mirror_checks = check_boundaries(&mut raw, &mut managed, &spans);
    for index in [25, 255] {
        let expected = 0x3ad318 + 12 * u32::from(index);
        select_descriptor(&mut raw, index, expected);
        select_descriptor(&mut managed, index, expected);
        assert_eq!(managed.cpu.regs, raw.cpu.regs);
    }
    flip_bank(&mut managed, &spans);

    let before = owner.background_effect_graphics_bus_bytes().unwrap();
    let mut payload_bytes = 0;
    for row in &plan.descriptors_rows {
        raw.load_state(&initial).unwrap();
        managed.load_state(&initial).unwrap();
        flip_bank(&mut managed, &spans);
        let read_before = owner.background_effect_graphics_bus_bytes().unwrap();
        execute_transfers(&mut raw, &rom, row);
        execute_transfers(&mut managed, &rom, row);
        let payload = u64::from(row.tiles.length + row.tilemap.length);
        let read = owner.background_effect_graphics_bus_bytes().unwrap() - read_before;
        assert_eq!(
            read,
            payload + 20,
            "ROM-data reads for descriptor {}",
            row.index
        );
        flip_bank(&mut managed, &spans);
        same_state(&raw, &managed, ROM_BASE + row.descriptor_offset);
        payload_bytes += payload;
    }
    assert_eq!(payload_bytes, 177_008);
    fs::write(
        input("FFTA_BACKGROUND_EFFECT_RESULT"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "status": "PASS",
            "descriptors": 25,
            "word_dma_transfers": 100,
            "halfword_dma_rows": 500,
            "payload_bytes": payload_bytes,
            "managed_read_bytes": owner.background_effect_graphics_bus_bytes().unwrap() - before,
            "mixed_mirror_checks": mixed_mirror_checks,
            "out_of_projection_selector_prefixes": [25, 255],
            "full_state_and_clock_equal": true,
            "e355_attachment": true,
            "invalid_candidates_rejected": 2,
        }))
        .unwrap(),
    )
    .unwrap();
}
