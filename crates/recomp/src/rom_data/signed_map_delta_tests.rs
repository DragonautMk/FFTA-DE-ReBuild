//! Verify the complete signed map delta table through the original reader.
use super::{
    attach_dataset, encode_ui_raw_transfers,
    indexed_reader_bank_tests::{execute_until, initialize},
    parse_ui_raw_transfers, read_dataset, UI_RAW_TRANSFERS_ID,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::{
        read_audit::{Reader, Service, TraceCapture, TraceFilter},
        ui_raw_transfers::{
            signed_map_deltas::{ENTRY_COUNT, LITERAL_OFFSET, TABLE_OFFSET},
            Catalog,
        },
    },
    Bus, Machine,
};
use std::{fs, path::PathBuf, sync::Arc};

const BASE: u32 = 0x0800_0000;
const ENTRY_PC: u32 = 0x0803_86ea;
const ACCEPTED_END: u32 = 0x0803_8700;
const REJECTED_END: u32 = 0x0803_879e;
const COUNTER: u32 = 0x0200_2000;
const INITIAL_R6: u32 = 0x1234;
const SPANS: [(u32, u32); 2] = [(LITERAL_OFFSET, 4), (TABLE_OFFSET, ENTRY_COUNT as u32)];
const SIGNED_DELTA_REVISION: (u32, u32) = (338, 337);

fn input(name: &str) -> PathBuf {
    std::env::var_os(name).expect(name).into()
}

fn set_source_bytes(machine: &mut Machine, rom: &[u8], altered: bool) {
    for (offset, size) in SPANS {
        let range = offset as usize..(offset + size) as usize;
        for (byte, &raw) in machine.bus.rom[range.clone()].iter_mut().zip(&rom[range]) {
            *byte = if altered { !raw } else { raw };
        }
    }
}

#[test]
#[ignore = "requires the exact EUR ROM and the protected e337-d336 dataset"]
fn european_signed_map_deltas_serve_the_original_guarded_reader() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let mut dataset = read_dataset(&input("FFTA_STATE_TEST_DATASET")).unwrap();
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (337, 336)
    );
    let catalog = Catalog::from_rom(&rom).unwrap();
    let mut old_packet = catalog.decoded.clone();
    old_packet.signed_map_deltas.take().expect("new table");
    assert_eq!(Some(&old_packet), dataset.ui_raw_transfers.as_ref());
    assert_eq!(
        Some(encode_ui_raw_transfers(&old_packet).unwrap().as_slice()),
        dataset.ui_raw_transfers_bytes.as_deref()
    );

    let mut managed = Machine::new(rom.clone());
    attach_dataset(&mut managed, &dataset).unwrap();
    let previous_owner = managed.bus.rom_resources().unwrap().clone();
    set_source_bytes(&mut managed, &rom, true);
    assert_eq!(
        managed.bus.read8(BASE + TABLE_OFFSET),
        !rom[TABLE_OFFSET as usize],
        "the previous revision still uses the original backing for this table"
    );
    set_source_bytes(&mut managed, &rom, false);

    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|resource| resource.id == UI_RAW_TRANSFERS_ID)
        .unwrap();
    let old_sources: Vec<_> = catalog
        .sources
        .iter()
        .filter(|source| !SPANS.iter().any(|&(start, _)| source.offset == start))
        .cloned()
        .collect();
    assert_eq!(resource.sources, old_sources);
    let added: Vec<_> = catalog
        .sources
        .iter()
        .filter(|source| !old_sources.contains(source))
        .map(|source| (source.offset, source.length))
        .collect();
    assert_eq!(added, SPANS);
    let bytes = encode_ui_raw_transfers(&catalog.decoded).unwrap();
    resource.sources = catalog.sources;
    resource.sha256 = digest(&bytes);
    (
        dataset.manifest.extractor_revision,
        dataset.manifest.decoder_revision,
    ) = SIGNED_DELTA_REVISION;

    // The current manifest cannot attach the predecessor's packet in place
    // of its new table. A failed replacement leaves the live owner intact.
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(
        managed.bus.rom_resources().unwrap(),
        &previous_owner
    ));
    dataset.ui_raw_transfers = Some(parse_ui_raw_transfers(&bytes).unwrap());
    dataset.ui_raw_transfers_bytes = Some(bytes);
    attach_dataset(&mut managed, &dataset).unwrap();

    let mut raw = Machine::new(rom.clone());
    let initial = raw.save_state().unwrap();
    let filter = TraceFilter {
        readers: vec![Reader::Bus],
        ranges: SPANS
            .iter()
            .map(|&(start, length)| (start, start + length))
            .collect(),
    };
    let mut typed_requests = 0;
    for index in (0..=ENTRY_COUNT as u8).chain([u8::MAX]) {
        let accepted = usize::from(index) < ENTRY_COUNT;
        let stop = if accepted { ACCEPTED_END } else { REJECTED_END };
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap();
            initialize(machine, ENTRY_PC);
            machine.cpu.regs[1] = u32::from(index);
            machine.cpu.regs[2] = COUNTER;
            machine.cpu.regs[6] = INITIAL_R6;
            machine.bus.write8(COUNTER, index);
        }
        execute_until(&mut raw, stop, 16);
        set_source_bytes(&mut managed, &rom, true);
        let capture = TraceCapture::start(filter.clone(), 4).unwrap();
        capture.frame(Some(managed.bus.frames));
        execute_until(&mut managed, stop, 16);
        let trace = capture.finish();
        assert_eq!(trace.dropped_requests, 0);
        if accepted {
            let expected_read = [
                (LITERAL_OFFSET, 4, 0x0803_86f0),
                (TABLE_OFFSET + u32::from(index), 1, 0x0803_86f8),
            ];
            assert_eq!(trace.matched_requests, 2);
            assert_eq!(trace.events.len(), 2);
            for (event, (offset, size, pc)) in trace.events.iter().zip(expected_read) {
                assert_eq!(event.requests, 1);
                assert_eq!(event.key.service, Service::Typed);
                assert_eq!(event.key.read.physical_offset, offset);
                assert_eq!(event.key.read.length, size);
                let site = event.key.instruction.unwrap();
                assert!(site.thumb);
                assert_eq!(site.pc, pc);
            }
            let delta = rom[TABLE_OFFSET as usize + usize::from(index)] as i8;
            assert_eq!(
                managed.cpu.regs[6],
                INITIAL_R6.wrapping_add_signed(i32::from(delta))
            );
            assert_eq!(managed.bus.read8(COUNTER), index + 1);
        } else {
            assert_eq!(trace.matched_requests, 0);
            assert_eq!(managed.cpu.regs[6], INITIAL_R6);
            assert_eq!(managed.bus.read8(COUNTER), index);
        }
        typed_requests += trace.matched_requests;
        set_source_bytes(&mut managed, &rom, false);
        assert_eq!(raw.bus.clock, managed.bus.clock, "index {index}");
        assert!(
            raw.save_state().unwrap() == managed.save_state().unwrap(),
            "full state at index {index}"
        );
    }
    let stats = managed
        .bus
        .rom_resources()
        .unwrap()
        .ui_raw_transfers_stats()
        .unwrap();
    assert_eq!(stats.source_bytes, 240);
    assert_eq!((stats.selector_bytes, stats.payload_bytes), (192, 48));
    let peer = TABLE_OFFSET + ENTRY_COUNT as u32;
    managed.bus.rom[peer as usize] ^= 0xff;
    assert_eq!(managed.bus.read8(BASE + peer), !rom[peer as usize]);
    managed.bus.rom[peer as usize] = rom[peer as usize];

    let work = input("FFTA_SIGNED_MAP_DELTAS_WORK");
    let manifest = toml::to_string_pretty(&dataset.manifest).unwrap();
    fs::write(work.join("manifest.toml"), &manifest).unwrap();
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "status": "PASS",
            "revision": SIGNED_DELTA_REVISION,
            "manifest_sha256": digest(manifest.as_bytes()),
            "accepted_indices": ENTRY_COUNT,
            "rejected_indices": [48, 255],
            "original_typed_requests": typed_requests,
            "source_bytes": 52,
            "each_state_and_clock_equal": true,
            "predecessor_attach": true,
            "missing_table_rejected": true,
            "adjacent_peer_uses_raw": true,
        }))
        .unwrap(),
    )
    .unwrap();
}
