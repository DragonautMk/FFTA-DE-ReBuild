//! Compare the five original screen builders and DMA transfers on EUR.
use super::{
    attach_dataset,
    direct_byte_copy_tests::{flip, input, same_state},
    indexed_reader_bank_tests::{execute_until, initialize},
    read_dataset, BACKGROUND_SCREEN_BANKS_ID,
};
use gba_core::{ffta_rom::digest, rom_data::SourceRange, Bus, Machine};
use serde::Deserialize;
use std::{fs, ops::Range, sync::Arc};

const ROM_BASE: u32 = 0x0800_0000;
const TASK: u32 = 0x0201_0000;
const MESSAGE: u32 = 0x0201_2000;
const READY: u32 = MESSAGE + 0x40;
const QUEUE: u32 = 0x0300_2bcc;
const QUEUE_CONTEXT: u32 = 0x0201_8000;
const RETURN: u32 = 0x0300_7f00;
const TILES: u32 = 0x0600_0000;
const MAP: u32 = 0x0600_6000;
const MAP_BYTES: usize = 4096;
const FILL: u8 = 0x5a;
const PLAN_SHA: &str = "2c3a4dfd022fd7e10af01b6fc32437526940610dfa2833b85f5e57d77624e65c";

#[derive(Deserialize)]
struct Span {
    offset: u32,
    length: u32,
}

impl Span {
    fn range(&self) -> Range<usize> {
        self.offset as usize..(self.offset + self.length) as usize
    }
}

#[derive(Deserialize)]
struct Screen {
    index: u8,
    width: usize,
    height: usize,
    tiles: Span,
    tilemap: Span,
    cleared_cells: usize,
    flagged_03ff_cells: usize,
}

#[derive(Deserialize)]
struct Plan {
    rows: Vec<Screen>,
    new_ranges: Vec<SourceRange>,
    storage_rows_outside_dimensions: [Span; 1],
}

fn enter(machine: &mut Machine, start: u32, stop: u32) {
    initialize(machine, start);
    machine.cpu.regs[5] = TASK;
    machine.cpu.regs[8] = MESSAGE;
    machine.cpu.regs[14] = RETURN | 1;
    execute_until(machine, stop, 100_000);
}

fn drain_dma(machine: &mut Machine, source: u32, destination: u32, control: u32) {
    let node = machine.bus.read32(QUEUE);
    let start = QUEUE_CONTEXT + 0x18 + 0x918;
    assert!((start..start + 16 * 28).contains(&node) && (node - start) % 28 == 0);
    assert_eq!(machine.bus.read32(node + 4), source);
    assert_eq!(machine.bus.read32(node + 8), destination);
    assert_eq!(machine.bus.read32(node + 12), control);
    assert_eq!(machine.bus.read32(node + 24), 0);
    enter(machine, 0x0800_0800, RETURN);
    assert_eq!(machine.bus.read32(QUEUE), 0);
}

fn upload_tiles(machine: &mut Machine, rom: &[u8], screen: &Screen) -> usize {
    machine.bus.write8(READY, FILL);
    machine.bus.write32(MESSAGE + 4, READY);
    machine.bus.write16(MESSAGE + 20, u16::from(screen.index));
    enter(machine, 0x0812_34e4, 0x0812_34f0);
    assert_eq!(machine.bus.read8(TASK + 8), screen.index);
    assert_eq!(machine.bus.read8(READY), 0);
    enter(machine, 0x0812_35ea, 0x0812_3608);
    let source = ROM_BASE + screen.tiles.offset;
    let size = screen.tiles.length;
    assert_eq!(machine.bus.read32(TASK + 24), source);
    assert_eq!(machine.bus.read32(TASK + 20), TILES);
    assert_eq!(u32::from(machine.bus.read16(TASK + 16)), size);
    machine.bus.vram[..size as usize + 4].fill(FILL);
    let mut chunks = 0;
    for offset in (0..size).step_by(4096) {
        let chunk = (size - offset).min(4096);
        let end = offset + chunk;
        enter(machine, 0x0812_3620, 0x0812_3654);
        assert_eq!(machine.cpu.regs[6], u32::from(end == size));
        assert_eq!(machine.bus.read32(TASK + 24), source + end);
        assert_eq!(machine.bus.read32(TASK + 20), TILES + end);
        assert_eq!(u32::from(machine.bus.read16(TASK + 16)), size - end);
        drain_dma(
            machine,
            source + offset,
            TILES + offset,
            0x8400_0000 | (chunk / 4),
        );
        chunks += 1;
    }
    assert_eq!(
        &machine.bus.vram[..size as usize],
        &rom[screen.tiles.range()]
    );
    assert_eq!(
        &machine.bus.vram[size as usize..size as usize + 4],
        &[FILL; 4]
    );
    chunks
}

fn upload_map(machine: &mut Machine, rom: &[u8], screen: &Screen) {
    let source = &rom[screen.tilemap.range()];
    assert_eq!(source.len(), screen.width * screen.height * 2);
    let mut expected = [0; MAP_BYTES];
    let mut cleared = 0;
    let mut flagged = 0;
    for (index, bytes) in source.chunks_exact(2).enumerate() {
        let x = index % screen.width;
        let y = index / screen.width;
        let index = 2 * (32 * y + x % 32 + 1024 * (x / 32));
        let raw = u16::from_le_bytes([bytes[0], bytes[1]]);
        cleared += usize::from(raw == 0x03ff);
        flagged += usize::from(raw == 0x23ff);
        if raw != 0x03ff {
            expected[index..index + 2].copy_from_slice(bytes);
        }
    }
    assert_eq!(
        (cleared, flagged),
        (screen.cleared_cells, screen.flagged_03ff_cells)
    );

    let buffer = (TASK + 56 - 0x0200_0000) as usize;
    let destination = (MAP - TILES) as usize;
    machine.bus.ewram[buffer - 4..buffer + MAP_BYTES + 4].fill(FILL);
    machine.bus.vram[destination - 4..destination + MAP_BYTES + 4].fill(FILL);
    enter(machine, 0x0812_365c, 0x0812_371c);
    assert_eq!(&machine.bus.ewram[buffer..buffer + MAP_BYTES], &expected);
    assert_eq!(&machine.bus.ewram[buffer - 4..buffer], &[FILL; 4]);
    assert_eq!(
        &machine.bus.ewram[buffer + MAP_BYTES..buffer + MAP_BYTES + 4],
        &[FILL; 4]
    );
    drain_dma(machine, TASK + 56, MAP, 0x8000_0800);
    assert_eq!(
        &machine.bus.vram[destination..destination + MAP_BYTES],
        &expected
    );
    assert_eq!(&machine.bus.vram[destination - 4..destination], &[FILL; 4]);
    assert_eq!(
        &machine.bus.vram[destination + MAP_BYTES..destination + MAP_BYTES + 4],
        &[FILL; 4]
    );
}

fn check_boundaries(raw: &mut Machine, managed: &mut Machine, sources: &[SourceRange]) -> u64 {
    let owner = managed.bus.rom_resources().unwrap().clone();
    let before = owner.background_screen_bank_bus_bytes().unwrap();
    let mut checks = 0;
    let mut managed_bytes = 0;
    for span in sources {
        for end in [span.offset, span.offset + span.length] {
            for offset in end - 3..=end + 3 {
                for base in [ROM_BASE, 0x0a00_0000, 0x0c00_0000] {
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
        owner.background_screen_bank_bus_bytes().unwrap() - before,
        managed_bytes
    );
    checks
}

#[test]
#[ignore = "Exact EUR ROM, E609 source proof and managed e357/e358 datasets"]
fn european_background_screen_banks_feed_original_uploads() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    let bytes = fs::read(input("FFTA_BACKGROUND_SCREEN_PLAN")).unwrap();
    assert_eq!(digest(&bytes), PLAN_SHA);
    let plan: Plan = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(plan.rows.len(), 5);
    let mut previous = read_dataset(&input("FFTA_BACKGROUND_SCREEN_PREDECESSOR")).unwrap();
    let mut dataset = read_dataset(&input("FFTA_BACKGROUND_SCREEN_DATASET")).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (357, 356)
    );
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (358, 357)
    );
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_dataset(&mut managed, &previous).unwrap();
    assert!(managed
        .bus
        .rom_resources()
        .unwrap()
        .background_screen_bank_bus_bytes()
        .is_none());
    attach_dataset(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    (
        previous.manifest.extractor_revision,
        previous.manifest.decoder_revision,
    ) = (358, 357);
    assert!(attach_dataset(&mut managed, &previous).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|row| row.id == BACKGROUND_SCREEN_BANKS_ID)
        .unwrap();
    assert_eq!(resource.sources, plan.new_ranges);
    let bytes = dataset.background_screen_banks_bytes.as_mut().unwrap();
    bytes[0] ^= 1;
    resource.sha256 = digest(bytes);
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));

    for span in &plan.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    let mixed = check_boundaries(&mut raw, &mut managed, &plan.new_ranges);
    let [extra] = plan.storage_rows_outside_dimensions;
    assert_eq!(extra.length, 74);
    flip(&mut managed, extra.offset, extra.length);
    let before = owner.background_screen_bank_bus_bytes().unwrap();
    assert_eq!(managed.bus.read16(ROM_BASE + extra.offset), !0x03ff);
    assert_eq!(owner.background_screen_bank_bus_bytes().unwrap(), before);

    for machine in [&mut raw, &mut managed] {
        machine.bus.write32(0x0201_59c8, QUEUE_CONTEXT);
    }
    let mut chunks = 0;
    for screen in &plan.rows {
        let counts = [&mut raw, &mut managed].map(|machine| {
            let count = upload_tiles(machine, &rom, screen);
            upload_map(machine, &rom, screen);
            count
        });
        assert_eq!(counts[0], counts[1]);
        assert_eq!(raw.bus.clock, managed.bus.clock);
        chunks += counts[0];
    }
    assert_eq!(chunks, 18);
    let original_read_bytes = owner.background_screen_bank_bus_bytes().unwrap() - before;
    assert_eq!(original_read_bytes, 103_460);
    for span in &plan.new_ranges {
        flip(&mut managed, span.offset, span.length);
    }
    flip(&mut managed, extra.offset, extra.length);
    same_state(&raw, &managed, RETURN);
    fs::write(
        input("FFTA_BACKGROUND_SCREEN_RESULT"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "status": "PASS",
            "screens": plan.rows.len(),
            "tile_dma_chunks": chunks,
            "map_dma_uploads": plan.rows.len(),
            "original_read_bytes": original_read_bytes,
            "mixed_mirror_checks": mixed,
            "e357_attachment": true,
            "invalid_candidates_rejected": 2,
            "extra_row_uses_backing": true,
            "full_state_and_clock_equal": true
        }))
        .unwrap(),
    )
    .unwrap();
}
