//! Verify the complete tables through their original indexed decode and copy paths.

use super::{
    attach_from_directory, cmd, encode_indexed_graphics,
    indexed_reader_bank_tests::{execute_until, initialize},
    parse, parse_indexed_graphics, Manifest, INDEXED_GRAPHICS_FILE, INDEXED_GRAPHICS_ID,
    MANIFEST_FILE,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::{
        indexed_graphics::vram_selectors::{PROGRAM_TABLE, TILE_ORDER},
        read_audit::{Service, TraceCapture, TraceFilter},
        SourceRange,
    },
    Bus, Machine,
};
use serde::Deserialize;
use std::{collections::BTreeMap, fs, path::PathBuf, sync::Arc};

const BASE: u32 = 0x0800_0000;
const VRAM: u32 = 0x0600_0000;
const DESTINATION: u32 = 0x0601_1000;
const SCRATCH: u32 = 0x0200_3cb0;
const LANGUAGE: u32 = 0x0200_1f6e;
type Reads = BTreeMap<(u32, usize, u32), u64>;

#[derive(Deserialize)]
struct Plan {
    readers: [Reader; 5],
    program_indices: Vec<u16>,
    tile_order: Vec<u8>,
    archive_roots: [u32; 6],
    archive_sources: Vec<SourceRange>,
    spans: Vec<SourceRange>,
}

#[derive(Deserialize)]
struct Reader {
    entry: u32,
    stop: u32,
    literal_reader: u32,
    literal: u32,
    lookup: u32,
    code_sha256: String,
}

fn input(name: &str) -> PathBuf {
    std::env::var_os(name).unwrap().into()
}

fn record_lookup(expected: &mut Reads, reader: &Reader, source: u32) {
    *expected
        .entry((reader.literal, 4, BASE + reader.literal_reader))
        .or_default() += 1;
    *expected
        .entry((source, 1, BASE + reader.lookup))
        .or_default() += 1;
}

fn run_vram_paths(
    machine: &mut Machine,
    plan: &Plan,
    programs: &[Vec<u8>],
    localized_tiles: &[Vec<u8>],
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
        500,
    )
    .unwrap();
    capture.frame(Some(machine.bus.frames));
    let mut expected: Reads = BTreeMap::new();
    let all_slots: Vec<_> = (0..plan.program_indices.len()).collect();
    let boundaries = [0, all_slots.len() - 1];
    for (i, reader) in plan.readers[..3].iter().enumerate() {
        let slots: &[usize] = if i == 0 { &all_slots } else { &boundaries };
        for &slot in slots {
            initialize(machine, BASE + reader.entry);
            machine.cpu.regs[0] = slot as u32;
            machine.cpu.regs[4] = DESTINATION;
            machine.cpu.regs[5] = DESTINATION;
            machine.cpu.regs[9] = SCRATCH;
            execute_until(machine, BASE + reader.stop, 100_000);
            let start = (DESTINATION - VRAM) as usize;
            let program = usize::from(plan.program_indices[slot]);
            let bytes = &programs[program];
            assert_eq!(&machine.bus.vram[start..start + bytes.len()], bytes);
            record_lookup(&mut expected, reader, PROGRAM_TABLE + 2 * slot as u32);
        }
    }
    // The second loop exercises the same language helper and loading order.
    for (reader, language) in (0..5u8)
        .map(|language| (&plan.readers[4], language))
        .chain(std::iter::once((&plan.readers[3], 4)))
    {
        initialize(machine, BASE + reader.entry);
        machine.bus.write8(LANGUAGE, language);
        execute_until(machine, BASE + reader.stop, 2_000_000);
        let tiles = &localized_tiles[language as usize];
        assert_eq!(&machine.bus.vram[0x20..0x20 + tiles.len()], tiles);
        for index in 0..plan.tile_order.len() as u32 {
            record_lookup(&mut expected, reader, TILE_ORDER + index);
        }
    }
    let report = capture.finish();
    assert!(!report.truncated && report.dropped_requests == 0);
    let mut actual = Reads::new();
    for event in report.events {
        let read = event.key.read;
        let site = event.key.instruction.expect("Original VRAM selector load");
        assert!(site.thumb);
        assert_eq!(event.key.service, service, "{event:?}");
        assert!(actual
            .insert((read.physical_offset, read.length, site.pc), event.requests)
            .is_none());
    }
    assert_eq!(actual, expected);
    actual
}

#[test]
#[ignore = "exact EUR ROM, E536 selector plan and e330-d329 predecessor manifest"]
fn european_indexed_vram_selectors_use_extracted_tables() {
    let rom_path = input("FFTA_STATE_TEST_ROM");
    let rom = fs::read(&rom_path).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let work = input("FFTA_INDEXED_VRAM_WORK");
    let plan: Plan =
        serde_json::from_slice(&fs::read(input("FFTA_INDEXED_VRAM_PLAN")).unwrap()).unwrap();
    assert_eq!(
        (plan.program_indices.len(), plan.tile_order.len()),
        (90, 42)
    );
    assert_eq!(plan.spans.len(), 7);
    let source_bytes = plan.spans.iter().map(|span| span.length).sum::<u32>();
    assert_eq!(source_bytes, 242);
    for record in plan.spans.iter().chain(&plan.archive_sources) {
        let start = record.offset as usize;
        assert_eq!(
            digest(&rom[start..start + record.length as usize]),
            record.sha256
        );
    }
    for reader in &plan.readers {
        assert_eq!(
            digest(&rom[reader.entry as usize..reader.stop as usize]),
            reader.code_sha256
        );
    }
    let old_manifest_bytes = fs::read(input("FFTA_INDEXED_VRAM_PREDECESSOR")).unwrap();
    let previous: Manifest = parse(&old_manifest_bytes, "E535 manifest").unwrap();
    assert_eq!(
        (previous.extractor_revision, previous.decoder_revision),
        (330, 329)
    );

    let resources = tempfile::tempdir_in(&work).unwrap();
    cmd(&[
        "prepare".into(),
        rom_path.to_str().unwrap().into(),
        resources.path().to_str().unwrap().into(),
    ])
    .unwrap();
    let revision = resources.path().join(EUROPE_SHA256).join("v1/e331-d330");
    let current = fs::read_to_string(revision.join("current")).unwrap();
    let dataset = revision.join(current.trim());
    let manifest_path = dataset.join(MANIFEST_FILE);
    let packet_path = dataset.join(INDEXED_GRAPHICS_FILE);
    let manifest_bytes = fs::read(&manifest_path).unwrap();
    let packet_bytes = fs::read(&packet_path).unwrap();
    let manifest: Manifest = parse(&manifest_bytes, "E536 manifest").unwrap();
    let index = previous
        .resources
        .iter()
        .position(|entry| entry.id == INDEXED_GRAPHICS_ID)
        .unwrap();
    let mut expected = previous.clone();
    expected.extractor_revision = 331;
    expected.decoder_revision = 330;
    expected.resources[index].sha256 = digest(&packet_bytes);
    expected.resources[index]
        .sources
        .extend(plan.spans.iter().cloned());
    expected.resources[index]
        .sources
        .sort_unstable_by_key(|span| span.offset);
    assert_eq!(manifest, expected);

    let mut packet = parse_indexed_graphics(&packet_bytes).unwrap();
    let selectors = packet.vram_selectors.take().unwrap();
    assert_eq!(selectors.program_indices, plan.program_indices);
    assert_eq!(selectors.tile_order, plan.tile_order);
    let archive = packet
        .reader_banks
        .as_ref()
        .unwrap()
        .archives
        .iter()
        .find(|a| a.source_offset == plan.archive_roots[0])
        .unwrap();
    assert_eq!(
        (archive.programs.len(), archive.fixed_decoded_bytes),
        (54, 640)
    );
    let programs: Vec<_> = archive.programs.iter().map(|p| p.decoded_bytes()).collect();
    let localized_tiles: Vec<Vec<u8>> = plan.archive_roots[1..]
        .iter()
        .map(|&root| {
            let archive = packet
                .additional
                .as_ref()
                .unwrap()
                .archives
                .iter()
                .find(|a| a.source_offset == root)
                .unwrap();
            assert_eq!(
                (archive.programs.len(), archive.fixed_decoded_bytes),
                (44, 256)
            );
            plan.tile_order
                .iter()
                .flat_map(|&id| archive.programs[usize::from(id - 2)].decoded_bytes())
                .collect()
        })
        .collect();

    let old_packet = encode_indexed_graphics(&packet).unwrap();
    assert_eq!(digest(&old_packet), previous.resources[index].sha256);
    let mut managed = Machine::new(rom.clone());
    fs::write(&packet_path, old_packet).unwrap();
    fs::write(&manifest_path, old_manifest_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let mut incomplete = previous;
    incomplete.extractor_revision = 331;
    incomplete.decoder_revision = 330;
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
        for span in plan.spans.iter().chain(&plan.archive_sources) {
            let start = span.offset as usize;
            for byte in &mut machine.bus.rom[start..start + span.length as usize] {
                *byte ^= 0xff;
            }
        }
    };
    flip_backing(&mut managed);
    let reads = run_vram_paths(&mut raw, &plan, &programs, &localized_tiles, Service::Raw);
    assert_eq!(
        reads,
        run_vram_paths(
            &mut managed,
            &plan,
            &programs,
            &localized_tiles,
            Service::Typed
        )
    );
    for (slot, &value) in plan.program_indices.iter().enumerate() {
        let address = BASE + PROGRAM_TABLE + 2 * slot as u32;
        assert_eq!(managed.bus.read16(address), value);
        assert_eq!(raw.bus.read16(address), value);
    }
    let stats = managed
        .bus
        .rom_resources()
        .unwrap()
        .indexed_graphics_stats()
        .unwrap();
    assert_eq!((stats.archives_used, stats.programs_used), (6, 264));
    flip_backing(&mut managed);
    assert_eq!(raw.bus.clock, managed.bus.clock);
    assert_eq!(
        digest(&raw.save_state().unwrap()),
        digest(&managed.save_state().unwrap())
    );
    let gap = TILE_ORDER as usize + plan.tile_order.len();
    for byte in &mut managed.bus.rom[gap..gap + 2] {
        *byte ^= 0xff;
    }
    assert_eq!(
        managed.bus.read16(BASE + gap as u32),
        !raw.bus.read16(BASE + gap as u32)
    );
    managed.bus.rom[gap..gap + 2].copy_from_slice(&rom[gap..gap + 2]);

    fs::write(work.join("manifest.toml"), &manifest_bytes).unwrap();
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "status": "PASS", "revision": [331, 330],
            "manifest_sha256": digest(&manifest_bytes),
            "source_bytes": source_bytes, "selector_reads": reads.values().sum::<u64>(),
            "archives_used": stats.archives_used, "programs_used": stats.programs_used,
            "predecessor_attach": true, "missing_group_rejected": true,
            "vram_and_full_state_equal": true, "raw_gap_preserved": true
        }))
        .unwrap(),
    )
    .unwrap();
}
