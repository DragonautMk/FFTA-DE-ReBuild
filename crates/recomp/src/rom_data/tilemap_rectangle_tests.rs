//! Compare the original EUR rectangle draws with their extracted sources.

use super::{
    attach_from_directory, cmd, encode_ui_raw_transfers,
    indexed_reader_bank_tests::{execute_until, initialize},
    parse, parse_ui_raw_transfers, Manifest, MANIFEST_FILE, UI_RAW_TRANSFERS_FILE,
    UI_RAW_TRANSFERS_ID,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::{
        obj_graphics::Graphic,
        read_audit::{Service, TraceCapture, TraceFilter},
        ui_raw_transfers::halfword_copies::TilemapRectangle,
        SourceRange,
    },
    Bus, Machine,
};
use serde::Deserialize;
use std::{collections::BTreeMap, fs, path::PathBuf, sync::Arc};

const ROM_BASE: u32 = 0x0800_0000;
const EWRAM: u32 = 0x0200_0000;
const BG_CONTEXT: [u32; 2] = [0x0300_08b0, 0x0300_0990];
const BG_BUFFER: [u32; 2] = [0x0201_0000, 0x0201_2000];
const MANAGER: u32 = 0x0201_8000;
const TEMPLATE: [u32; 2] = [MANAGER + 0x4af4, MANAGER + 0x4b9c];
const LANGUAGE: u32 = 0x0200_1f6e;
const COLOR_MODE: u32 = 0x0200_2fc2;
const STRIDE: usize = 32;
const MAP_CELLS: usize = STRIDE * STRIDE;
const UNTOUCHED_CELL: u16 = 0xba5e;
type Reads = BTreeMap<(u32, usize, u32), u64>;

#[derive(Deserialize)]
struct Plan {
    rectangles: [Shape; 3],
    localized: [u32; 5],
    palettes: [[u32; 2]; 4],
    dependencies: Vec<SourceRange>,
    spans: Vec<SourceRange>,
    code: Vec<Window>,
}

#[derive(Deserialize)]
struct Shape {
    offset: u32,
    width: u8,
    height: u8,
}

#[derive(Deserialize)]
struct Window {
    start: usize,
    stop: usize,
    sha256: String,
}

fn input(name: &str) -> PathBuf {
    std::env::var_os(name).unwrap().into()
}

fn expect_draw(
    map: &mut [u16; MAP_CELLS],
    x: i32,
    y: usize,
    columns: usize,
    cells: &[u16],
    last_column: i32,
) {
    for (row, line) in cells.chunks_exact(columns).enumerate() {
        for (column, &cell) in line.iter().enumerate() {
            let screen_column = x + column as i32;
            if (0..=last_column).contains(&screen_column) {
                map[(y + row) * STRIDE + screen_column as usize] = cell;
            }
        }
    }
}

fn seed_maps(machine: &mut Machine) {
    for (context, buffer) in BG_CONTEXT.into_iter().zip(BG_BUFFER) {
        machine.bus.write8(context + 0xc, STRIDE as u8);
        machine.bus.write32(context + 0x14, buffer);
        for cell in 0..MAP_CELLS as u32 {
            machine.bus.write16(buffer + 2 * cell, UNTOUCHED_CELL);
        }
    }
}

fn assert_map(machine: &Machine, layer: usize, expected: &[u16; MAP_CELLS]) {
    let start = (BG_BUFFER[layer] - EWRAM) as usize;
    let actual = &machine.bus.ewram[start..start + 2 * MAP_CELLS];
    let matches = actual
        .iter()
        .copied()
        .eq(expected.iter().flat_map(|cell| cell.to_le_bytes()));
    assert!(matches, "Tilemap mismatch in layer {layer}");
}

fn return_from_path(machine: &mut Machine, step_limit: usize) {
    let stop = machine.cpu.regs[14] & !1;
    execute_until(machine, stop, step_limit);
}

fn run_direct_and_staged_draws(machine: &mut Machine, rectangles: &[TilemapRectangle; 3]) {
    seed_maps(machine);
    initialize(machine, ROM_BASE + 0x06a5c0);
    return_from_path(machine, 300_000);
    let mut foreground = [UNTOUCHED_CELL; MAP_CELLS];
    let mut mask = foreground;
    for (rectangle, x, color) in [(&rectangles[0], 1, 0xc001), (&rectangles[1], 15, 0xa001)] {
        let tiles: Vec<_> = rectangle
            .bank
            .halfwords
            .iter()
            .map(|tile| tile.wrapping_add(0x300))
            .collect();
        expect_draw(
            &mut foreground,
            x,
            11,
            usize::from(rectangle.width),
            &tiles,
            31,
        );
        expect_draw(
            &mut mask,
            x,
            11,
            14,
            &vec![color; 14 * usize::from(rectangle.height)],
            31,
        );
    }
    assert_map(machine, 0, &foreground);
    assert_map(machine, 1, &mask);

    let mut staged: [Vec<u16>; 2] = std::array::from_fn(|index| {
        rectangles[index]
            .bank
            .halfwords
            .iter()
            .map(|tile| tile.wrapping_add(0xc0))
            .collect()
    });
    staged[1] = staged[1]
        .chunks_exact(12)
        .flat_map(|row| row.iter().copied().chain([0, 0]))
        .collect();
    for (&address, cells) in TEMPLATE.iter().zip(&staged) {
        for index in 0..cells.len() as u32 {
            machine.bus.write16(address + 6 + 2 * index, 0xabcd);
        }
    }
    // The original parent builds the two RAM records at distinct offsets.
    initialize(machine, ROM_BASE + 0x06b210);
    machine.cpu.regs[7] = 0x0201_7000;
    machine.bus.write32(0x0201_7000, MANAGER);
    execute_until(machine, ROM_BASE + 0x06b224, 300_000);
    for (address, cells) in TEMPLATE.into_iter().zip(&staged) {
        for (index, &value) in cells.iter().enumerate() {
            assert_eq!(machine.bus.read16(address + 6 + 2 * index as u32), value);
        }
    }

    for positions in [[1, 15], [-2, 30]] {
        seed_maps(machine);
        foreground.fill(UNTOUCHED_CELL);
        mask.fill(UNTOUCHED_CELL);
        for (index, (entry, columns, height, color)) in
            [(0x071538, 10, 8, 0xc001), (0x071824, 14, 3, 0xa001)]
                .into_iter()
                .enumerate()
        {
            let x = positions[index];
            initialize(machine, ROM_BASE + entry);
            machine.cpu.regs[0] = TEMPLATE[index];
            machine.cpu.regs[1] = x as u32;
            machine.cpu.regs[2] = 11;
            return_from_path(machine, 300_000);
            expect_draw(&mut foreground, x, 11, columns, &staged[index], 30);
            expect_draw(&mut mask, x, 11, 14, &vec![color; 14 * height], 31);
        }
        assert_map(machine, 0, &foreground);
        assert_map(machine, 1, &mask);
    }
}

fn run_localized_loader(
    machine: &mut Machine,
    plan: &Plan,
    rectangle: &TilemapRectangle,
    rom: &[u8],
    decoded: &[Vec<u8>; 2],
) {
    let mut map = [UNTOUCHED_CELL; MAP_CELLS];
    expect_draw(
        &mut map,
        1,
        1,
        usize::from(rectangle.width),
        &rectangle.bank.halfwords,
        30,
    );
    for (language, &offset) in plan.localized.iter().enumerate() {
        let mode = language % plan.palettes.len();
        let localized = &rom[offset as usize..offset as usize + 768];
        let tile_outputs = [
            (0x15800, decoded[0].as_slice()),
            (0x16080, localized),
            (0x9c00, decoded[1].as_slice()),
        ];
        seed_maps(machine);
        machine.bus.write8(BG_CONTEXT[1] + 0xc, 29);
        machine.bus.write8(LANGUAGE, language as u8);
        machine.bus.write8(COLOR_MODE, mode as u8);
        for (start, bytes) in tile_outputs {
            for (dst, &expected) in machine.bus.vram[start..start + bytes.len()]
                .iter_mut()
                .zip(bytes)
            {
                *dst = !expected;
            }
        }
        initialize(machine, ROM_BASE + 0x12edf4);
        return_from_path(machine, 2_000_000);
        for (start, bytes) in tile_outputs {
            assert_eq!(&machine.bus.vram[start..start + bytes.len()], bytes);
        }
        for (&offset, start) in plan.palettes[mode].iter().zip([0x3b00, 0x3b80]) {
            let source = &rom[offset as usize..offset as usize + 32];
            assert_eq!(&machine.bus.iwram[start..start + 32], source);
        }
        assert_map(machine, 1, &map);
        assert_eq!(machine.bus.read8(BG_CONTEXT[1] + 0xc), 29);
    }
}

fn run_paths(
    machine: &mut Machine,
    plan: &Plan,
    rectangles: &[TilemapRectangle; 3],
    rom: &[u8],
    decoded: &[Vec<u8>; 2],
    service: Service,
) -> Reads {
    let capture = TraceCapture::start(
        TraceFilter {
            readers: vec![],
            ranges: plan
                .spans
                .iter()
                .map(|span| (span.offset, span.offset + span.length))
                .collect(),
        },
        700,
    )
    .unwrap();
    capture.frame(Some(machine.bus.frames));
    run_direct_and_staged_draws(machine, rectangles);
    run_localized_loader(machine, plan, &rectangles[2], rom, decoded);
    let report = capture.finish();
    assert!(!report.truncated && report.dropped_requests == 0);
    let mut actual = Reads::new();
    for event in report.events {
        let read = event.key.read;
        let site = event.key.instruction.expect("Original tilemap read");
        assert!(site.thumb);
        assert_eq!(event.key.service, service, "{event:?}");
        assert!(actual
            .insert((read.physical_offset, read.length, site.pc), event.requests)
            .is_none());
    }
    let mut expected = Reads::new();
    for (at, site, count) in [
        (0x06a6d4, 0x06a610, 1),
        (0x06a6dc, 0x06a68a, 1),
        (0x071534, 0x071512, 1),
        (0x071800, 0x0717ca, 1),
        (0x12ee90, 0x12ee0e, 5),
        (0x12eeac, 0x12ee44, 5),
    ] {
        expected.insert((at, 4, ROM_BASE + site), count);
    }
    for (rectangle, site, count) in [
        (&rectangles[0], 0x06a628, 1),
        (&rectangles[1], 0x06a6a4, 1),
        (&rectangles[0], 0x07151e, 1),
        (&rectangles[1], 0x0717f0, 1),
        (&rectangles[2], 0x02c74e, 5),
    ] {
        for index in 0..rectangle.bank.halfwords.len() as u32 {
            expected.insert(
                (rectangle.bank.source_offset + 2 * index, 2, ROM_BASE + site),
                count,
            );
        }
    }
    assert_eq!(actual, expected);
    actual
}

#[test]
#[ignore = "exact EUR ROM, E538 rectangle plan and e332-d331 predecessor manifest"]
fn european_tilemap_rectangles_use_extracted_banks() {
    let rom_path = input("FFTA_STATE_TEST_ROM");
    let rom = fs::read(&rom_path).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let work = input("FFTA_TILEMAP_RECTANGLES_WORK");
    let plan: Plan =
        serde_json::from_slice(&fs::read(input("FFTA_TILEMAP_RECTANGLES_PLAN")).unwrap()).unwrap();
    assert_eq!(plan.spans.len(), 9);
    let source_bytes = plan.spans.iter().map(|span| span.length).sum::<u32>();
    assert_eq!(source_bytes, 1004);
    for span in plan.spans.iter().chain(&plan.dependencies) {
        let start = span.offset as usize;
        assert_eq!(
            digest(&rom[start..start + span.length as usize]),
            span.sha256
        );
    }
    for window in &plan.code {
        assert_eq!(digest(&rom[window.start..window.stop]), window.sha256);
    }
    let previous_bytes = fs::read(input("FFTA_TILEMAP_RECTANGLES_PREDECESSOR")).unwrap();
    let previous: Manifest = parse(&previous_bytes, "E537 manifest").unwrap();
    assert_eq!(
        (previous.extractor_revision, previous.decoder_revision),
        (332, 331)
    );
    let resources = tempfile::tempdir_in(&work).unwrap();
    cmd(&[
        "prepare".into(),
        rom_path.to_str().unwrap().into(),
        resources.path().to_str().unwrap().into(),
    ])
    .unwrap();
    let revision = resources.path().join(EUROPE_SHA256).join("v1/e333-d332");
    let dataset = revision.join(fs::read_to_string(revision.join("current")).unwrap().trim());
    let manifest_path = dataset.join(MANIFEST_FILE);
    let packet_path = dataset.join(UI_RAW_TRANSFERS_FILE);
    let manifest_bytes = fs::read(&manifest_path).unwrap();
    let packet_bytes = fs::read(&packet_path).unwrap();
    let manifest: Manifest = parse(&manifest_bytes, "E538 manifest").unwrap();
    let index = previous
        .resources
        .iter()
        .position(|entry| entry.id == UI_RAW_TRANSFERS_ID)
        .unwrap();
    let mut expected = previous.clone();
    expected.extractor_revision = 333;
    expected.decoder_revision = 332;
    let resource = &mut expected.resources[index];
    resource.sha256 = digest(&packet_bytes);
    resource.sources.extend(plan.spans.iter().cloned());
    resource.sources.sort_unstable_by_key(|span| span.offset);
    assert_eq!(manifest, expected);

    let mut packet = parse_ui_raw_transfers(&packet_bytes).unwrap();
    let rectangles = packet
        .halfword_copies
        .as_mut()
        .unwrap()
        .tilemap_rectangles
        .take()
        .unwrap()
        .rectangles;
    for (rectangle, shape) in rectangles.iter().zip(&plan.rectangles) {
        assert_eq!(
            (
                rectangle.bank.source_offset,
                rectangle.width,
                rectangle.height
            ),
            (shape.offset, shape.width, shape.height)
        );
    }
    let decoded = [0x3bac48, 0x3bade0].map(|offset| {
        Graphic::decode(&rom[offset as usize..], offset)
            .unwrap()
            .decoded_bytes()
    });
    assert_eq!((decoded[0].len(), decoded[1].len()), (0x880, 0x680));

    let old_packet = encode_ui_raw_transfers(&packet).unwrap();
    assert_eq!(digest(&old_packet), previous.resources[index].sha256);
    let mut managed = Machine::new(rom.clone());
    fs::write(&packet_path, old_packet).unwrap();
    fs::write(&manifest_path, previous_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let mut incomplete = previous;
    incomplete.extractor_revision = 333;
    incomplete.decoder_revision = 332;
    fs::write(&manifest_path, toml::to_string_pretty(&incomplete).unwrap()).unwrap();
    assert!(attach_from_directory(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(&owner, managed.bus.rom_resources().unwrap()));
    fs::write(&packet_path, packet_bytes).unwrap();
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    drop(owner);

    let mut raw = Machine::new(rom.clone());
    let origin = raw.save_state().unwrap();
    raw.load_state(&origin).unwrap();
    managed.load_state(&origin).unwrap();
    let flip_backing = |machine: &mut Machine| {
        for span in plan.spans.iter().chain(&plan.dependencies) {
            let start = span.offset as usize;
            for byte in &mut machine.bus.rom[start..start + span.length as usize] {
                *byte ^= 0xff;
            }
        }
    };
    flip_backing(&mut managed);
    let reads = run_paths(&mut raw, &plan, &rectangles, &rom, &decoded, Service::Raw);
    assert_eq!(
        reads,
        run_paths(
            &mut managed,
            &plan,
            &rectangles,
            &rom,
            &decoded,
            Service::Typed
        )
    );
    let rectangle_reads = reads.values().sum::<u64>();
    assert_eq!(rectangle_reads, 2116);
    let stats = managed
        .bus
        .rom_resources()
        .unwrap()
        .ui_raw_transfers_stats()
        .unwrap();
    assert_eq!((stats.banks_used, stats.groups_used), (8, 1));

    for gap in [0x3b0e5b, 0x3ba95b] {
        managed.bus.rom[gap] ^= 0xff;
        for base in [ROM_BASE, 0x0a00_0000, 0x0c00_0000] {
            assert_eq!(managed.bus.read8(base + gap as u32), !rom[gap]);
            assert_eq!(
                managed.bus.read16(base + gap as u32 + 1),
                u16::from_le_bytes([rom[gap + 1], rom[gap + 2]])
            );
        }
        managed.bus.rom[gap] = rom[gap];
    }
    flip_backing(&mut managed);
    assert_eq!(raw.bus.clock, managed.bus.clock);
    assert_eq!(
        digest(&raw.save_state().unwrap()),
        digest(&managed.save_state().unwrap())
    );

    fs::write(work.join("manifest.toml"), &manifest_bytes).unwrap();
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "status": "PASS", "revision": [333, 332],
            "manifest_sha256": digest(&manifest_bytes), "source_bytes": source_bytes,
            "rectangle_reads": rectangle_reads, "banks_used": stats.banks_used,
            "predecessor_attach": true, "missing_group_rejected": true,
            "draws_and_full_state_equal": true, "raw_gaps_preserved": true,
        }))
        .unwrap(),
    )
    .unwrap();
}
