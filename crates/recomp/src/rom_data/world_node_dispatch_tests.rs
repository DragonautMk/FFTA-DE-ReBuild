//! Compare the original European node renderer reads with altered ROM backing.

use super::{
    attach_dataset, encode_thumb_pc_literals, encode_thumb_switch_tables,
    indexed_reader_bank_tests::{execute_until, initialize},
    parse_thumb_pc_literals, parse_thumb_switch_tables, read_dataset, THUMB_PC_LITERALS_ID,
    THUMB_SWITCH_TABLES_ID,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::{
        read_audit::{Reader, Service, TraceCapture, TraceFilter},
        thumb_switch_tables::ADDITIONAL_TABLE_COUNT,
        Extraction,
    },
    Bus, Machine,
};
use std::{fs, path::PathBuf, sync::Arc};

const BASE: u32 = 0x0800_0000;
const DISPATCH: u32 = 0x038674;
const SWITCH_ROOT: u32 = 0x0386b0;
const FIRST_TARGET: u32 = 0x0386b4;
const DRAW: u32 = 0x03879e;
const SKIP: u32 = 0x03880e;
const MANAGER_PTR: u32 = 0x0300_2810;
const MANAGER: u32 = 0x0200_3000;
const REGION: u8 = 7;
const INITIAL_Y: u32 = 0x1234;
const WORLD_NODE_REVISION: (u32, u32) = (339, 338);

// Include the previously owned mode literal and the first signed delta.
const ALTERED_SPANS: [(u32, u32); 4] = [
    (0x0386a8, 0x0386c8),
    (0x038704, 0x038708),
    (0x0387f0, 0x038808),
    (0x3954a4, 0x3954a5),
];

fn input(name: &str) -> PathBuf {
    std::env::var_os(name).expect(name).into()
}

struct ReaderPair<'a> {
    rom: &'a [u8],
    initial: Vec<u8>,
    raw: Machine,
    managed: Machine,
}

impl ReaderPair<'_> {
    fn check_window(
        &mut self,
        start: u32,
        end: u32,
        setup: impl Fn(&mut Machine),
        expected: &[(u32, usize, u32)],
    ) -> u64 {
        for machine in [&mut self.raw, &mut self.managed] {
            machine.load_state(&self.initial).unwrap();
            initialize(machine, BASE + start);
            setup(machine);
        }
        for (start, end) in ALTERED_SPANS {
            for byte in &mut self.managed.bus.rom[start as usize..end as usize] {
                *byte = !*byte;
            }
        }
        execute_until(&mut self.raw, BASE + end, 64);
        let filter = TraceFilter {
            readers: vec![Reader::Bus],
            ranges: ALTERED_SPANS.to_vec(),
        };
        let capture = TraceCapture::start(filter, 12).unwrap();
        capture.frame(Some(self.managed.bus.frames));
        execute_until(&mut self.managed, BASE + end, 64);
        let trace = capture.finish();
        assert_eq!(trace.dropped_requests, 0);
        assert_eq!(trace.matched_requests, expected.len() as u64);
        let mut actual: Vec<_> = trace
            .events
            .iter()
            .map(|event| {
                assert_eq!(event.requests, 1);
                assert_eq!(event.key.service, Service::Typed);
                let site = event.key.instruction.unwrap();
                assert!(site.thumb);
                (
                    event.key.read.physical_offset,
                    event.key.read.length,
                    site.pc - BASE,
                )
            })
            .collect();
        let mut expected = expected.to_vec();
        actual.sort_unstable();
        expected.sort_unstable();
        assert_eq!(actual, expected, "reader at {start:06x}");
        for (start, end) in ALTERED_SPANS {
            let range = start as usize..end as usize;
            self.managed.bus.rom[range.clone()].copy_from_slice(&self.rom[range]);
        }
        assert_eq!(self.raw.bus.clock, self.managed.bus.clock);
        assert!(
            self.raw.save_state().unwrap() == self.managed.save_state().unwrap(),
            "full state at {start:06x} to {end:06x}"
        );
        trace.matched_requests
    }
}

fn initialize_node(machine: &mut Machine, mode: u8, selected: u8) {
    machine.cpu.regs[6] = INITIAL_Y;
    machine.cpu.regs[9] = MANAGER_PTR;
    machine.cpu.regs[10] = u32::from(REGION);
    machine.bus.write32(MANAGER_PTR, MANAGER);
    machine.bus.write8(MANAGER + 0x685, mode);
    machine.bus.write8(MANAGER + 0x686, selected);
    machine.bus.write8(MANAGER + 0x687, 9);
    machine.bus.write8(MANAGER + 0x688, 0);
}

#[test]
#[ignore = "requires the exact EUR ROM and the protected e338-d337 dataset"]
fn european_world_node_renderer_reads_its_complete_dispatch_bank() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let mut dataset = read_dataset(&input("FFTA_STATE_TEST_DATASET")).unwrap();
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (338, 337)
    );
    let mut current_manifest = dataset.manifest.clone();
    (
        current_manifest.extractor_revision,
        current_manifest.decoder_revision,
    ) = WORLD_NODE_REVISION;
    let source = Extraction::from_rom_for_manifest(&rom, &current_manifest).unwrap();
    let switches = source.thumb_switch_tables.unwrap();
    let literals = source.thumb_pc_literals.unwrap();

    let mut old_switches = switches.decoded.clone();
    let added_table = old_switches.additional_tables.pop().unwrap();
    assert_eq!(
        (added_table.root_offset, added_table.destinations.len()),
        (SWITCH_ROOT, 5)
    );
    let mut old_literals = literals.decoded.clone();
    assert_eq!(old_literals.world_node_renderer_literals.len(), 8);
    old_literals.world_node_renderer_literals.clear();
    assert!(
        Some(
            encode_thumb_switch_tables(&old_switches)
                .unwrap()
                .as_slice()
        ) == dataset.thumb_switch_tables_bytes.as_deref(),
        "predecessor switch packet"
    );
    assert!(
        Some(encode_thumb_pc_literals(&old_literals).unwrap().as_slice())
            == dataset.thumb_pc_literals_bytes.as_deref(),
        "predecessor literal packet"
    );

    let mut managed = Machine::new(rom.clone());
    attach_dataset(&mut managed, &dataset).unwrap();
    let old_owner = managed.bus.rom_resources().unwrap().clone();
    managed.bus.rom[FIRST_TARGET as usize] ^= 0xff;
    assert_eq!(
        managed.bus.read8(BASE + FIRST_TARGET),
        !rom[FIRST_TARGET as usize]
    );
    managed.bus.rom[FIRST_TARGET as usize] = rom[FIRST_TARGET as usize];

    dataset.manifest = current_manifest;
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(
        managed.bus.rom_resources().unwrap(),
        &old_owner
    ));

    let switch_bytes = encode_thumb_switch_tables(&switches.decoded).unwrap();
    let literal_bytes = encode_thumb_pc_literals(&literals.decoded).unwrap();
    for (id, new_sources, bytes) in [
        (THUMB_SWITCH_TABLES_ID, switches.sources, &switch_bytes),
        (THUMB_PC_LITERALS_ID, literals.sources, &literal_bytes),
    ] {
        let resource = dataset
            .manifest
            .resources
            .iter_mut()
            .find(|entry| entry.id == id)
            .unwrap();
        assert!(new_sources.starts_with(&resource.sources));
        resource.sources = new_sources;
        resource.sha256 = digest(bytes);
    }
    dataset.thumb_switch_tables = Some(parse_thumb_switch_tables(&switch_bytes).unwrap());
    dataset.thumb_pc_literals = Some(parse_thumb_pc_literals(&literal_bytes).unwrap());
    dataset.thumb_switch_tables_bytes = Some(switch_bytes);
    dataset.thumb_pc_literals_bytes = Some(literal_bytes);
    attach_dataset(&mut managed, &dataset).unwrap();

    let raw = Machine::new(rom.clone());
    let mut pair = ReaderPair {
        rom: &rom,
        initial: raw.save_state().unwrap(),
        raw,
        managed,
    };
    let mode_read = (0x0386a8, 4, 0x038678);
    let selected_read = (0x0386ac, 4, 0x038684);
    let mut total_requests = 0;
    for (index, stop) in [DRAW, 0x038730, 0x038766, DRAW, 0x038708]
        .into_iter()
        .enumerate()
    {
        let mut expected = vec![
            mode_read,
            selected_read,
            (SWITCH_ROOT, 4, 0x0386a0),
            (FIRST_TARGET + 4 * index as u32, 4, 0x0386a4),
        ];
        if index == 0 {
            expected.extend([(0x038704, 4, 0x0386f0), (0x3954a4, 1, 0x0386f8)]);
        }
        total_requests += pair.check_window(
            DISPATCH,
            stop,
            |machine| initialize_node(machine, index as u8 + 1, REGION),
            &expected,
        );
        if index == 0 {
            let delta = rom[0x3954a4] as i8 as i32;
            assert_eq!(
                pair.managed.cpu.regs[6],
                INITIAL_Y.wrapping_add_signed(delta)
            );
            assert_eq!(pair.managed.bus.read8(MANAGER + 0x688), 1);
        }
    }
    for (mode, selected, stop, expected) in [
        (0, REGION, SKIP, &[mode_read][..]),
        (6, REGION, DRAW, &[mode_read, selected_read][..]),
        (255, REGION, DRAW, &[mode_read, selected_read][..]),
        (1, REGION + 1, SKIP, &[mode_read, selected_read][..]),
    ] {
        total_requests += pair.check_window(
            DISPATCH,
            stop,
            |machine| initialize_node(machine, mode, selected),
            expected,
        );
    }

    total_requests += pair.check_window(
        DRAW,
        0x0387cc,
        |machine| initialize_node(machine, 3, REGION),
        &[
            (0x0387f0, 4, 0x03879e),
            (0x0387f4, 4, 0x0387be),
            (0x0387f8, 4, 0x0387c2),
        ],
    );
    total_requests += pair.check_window(
        0x0387d0,
        0x038840,
        |machine| initialize_node(machine, 3, REGION),
        &[
            (0x0387fc, 4, 0x0387d0),
            (0x038800, 4, 0x0387d4),
            (0x038804, 4, 0x0387de),
        ],
    );
    assert_eq!(pair.managed.bus.read8(MANAGER + 0x687), 8);
    assert_eq!(total_requests, 35);

    let stats = pair
        .managed
        .bus
        .rom_resources()
        .unwrap()
        .thumb_switch_tables_stats()
        .unwrap();
    assert_eq!(stats.additional_source_bytes, 20);
    assert_eq!(
        stats.additional_destinations_mask,
        1 << (ADDITIONAL_TABLE_COUNT - 1)
    );
    for (offset, typed) in [
        (0x032e10, true),
        (0x032e14, true),
        (0x0386c8, false),
        (0x0387ef, false),
        (0x038808, false),
    ] {
        let raw = rom[offset as usize];
        pair.managed.bus.rom[offset as usize] = !raw;
        assert_eq!(
            pair.managed.bus.read8(BASE + offset),
            if typed { raw } else { !raw }
        );
        pair.managed.bus.rom[offset as usize] = raw;
    }

    let work = input("FFTA_WORLD_NODE_DISPATCH_WORK");
    let manifest = toml::to_string_pretty(&dataset.manifest).unwrap();
    fs::write(work.join("manifest.toml"), &manifest).unwrap();
    let result = serde_json::json!({
        "status": "PASS", "revision": WORLD_NODE_REVISION,
        "manifest_sha256": digest(manifest.as_bytes()), "new_source_bytes": 52,
        "valid_modes": 5, "bypasses": 4, "draw_windows": 2,
        "matched_requests": total_requests, "switch_source_bytes_read": stats.additional_source_bytes,
        "original_state_and_clock_equal": true, "predecessor_attach": true,
        "incomplete_revision_rejected": true, "outer_owners_and_adjacent_raw_preserved": true,
    });
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
