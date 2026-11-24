//! Verify the complete European tilemap bank against the original guest readers.

use super::{
    attach_from_directory, parse, preparation,
    tilemap_transfer_directory_tests::{check_stream_bus_reads, replay_pair},
    Extraction, Manifest, INDEXED_TILEMAP_CONTROLS_ID, MANIFEST_FILE, TILEMAP_STREAM_BANK_FILE,
    TILEMAP_STREAM_BANK_ID,
};
use gba_core::{
    cpu::{FLAG_I, FLAG_T},
    ffta_rom::digest,
    rom_data::{
        read_audit::{Service, TraceCapture, TraceFilter},
        tilemap_stream_bank::{self as bank, Stream},
    },
    Bus, Machine,
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    ops::Range,
    path::{Path, PathBuf},
};

fn input(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os(name).unwrap_or_else(|| panic!("Missing {name}")))
}

fn read_index(name: &str) -> Value {
    serde_json::from_slice(&fs::read(input(name)).unwrap()).unwrap()
}

fn verify_extents(rom: &[u8], streams: &[Stream]) -> BTreeSet<u32> {
    let reference = read_index("FFTA_TILEMAP_STREAM_BANK_INDEX");
    let byte_streams = read_index("FFTA_TILEMAP_BYTE_EXTENTS");
    let roots = reference["roots"].as_array().unwrap();
    assert_eq!(streams.len(), roots.len());
    let mut selected = BTreeSet::new();
    for (stream, root) in streams.iter().zip(roots) {
        let Stream {
            extent,
            mode,
            byte_values,
            sources,
            ..
        } = stream;
        assert_eq!(root["start"], extent.start);
        assert_eq!(root["end"], extent.end);
        assert_eq!(root["type"], if *byte_values { 5 } else { 4 });
        assert_eq!(root["mode"], *mode);
        assert_eq!(
            root["sha256"],
            digest(&rom[extent.start as usize..extent.end as usize])
        );
        let mut expected: BTreeSet<_> = extent.clone().collect();
        if *byte_values {
            let exact = &byte_streams["streams"][extent.start.to_string()];
            for gap in exact["skipped_ranges"].as_array().unwrap() {
                let start = gap[0].as_u64().unwrap() as u32;
                let end = gap[1].as_u64().unwrap() as u32;
                if start != extent.start + 2 {
                    for at in start..end {
                        assert!(expected.remove(&at));
                    }
                }
            }
        }
        let source_bytes: BTreeSet<_> = sources.iter().cloned().flatten().collect();
        assert_eq!(source_bytes, expected);
        assert!(selected.is_disjoint(&source_bytes));
        selected.extend(source_bytes);
    }
    assert_eq!(selected.len(), 9654);
    selected
}

const CURSOR: u32 = 0x0200_0010;
const RUN_INDEX: u32 = CURSOR + 4;
const RETURN: u32 = 0x0200_0000;
const ROM_BASE: u32 = 0x0800_0000;

fn read_cell(machine: &mut Machine, entry: u32) -> u32 {
    machine.cpu.regs[..3].copy_from_slice(&[CURSOR, RUN_INDEX, 2]);
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = entry;
    for _ in 0..128 {
        machine.step();
        if machine.cpu.regs[15] == RETURN {
            return machine.cpu.regs[0];
        }
    }
    panic!("Tilemap reader {entry:#x} did not return");
}

fn prior_stream_bytes(manifest: &Manifest) -> BTreeSet<u32> {
    manifest
        .resources
        .iter()
        .flat_map(|resource| &resource.sources)
        .flat_map(|source| {
            source.offset.max(bank::START)..(source.offset + source.length).min(bank::END)
        })
        .collect()
}

fn verify_original_readers(
    rom: &[u8],
    streams: &[Stream],
    selected: &BTreeSet<u32>,
    prior: &BTreeSet<u32>,
    managed: &mut Machine,
) -> Value {
    assert_eq!(prior.len(), 844);
    assert!(prior.is_subset(selected));
    let bound = selected.len() - prior.len();

    let mut raw = Machine::new(rom.to_vec());
    for offset in bank::START..bank::END {
        let at = offset as usize;
        managed.bus.rom[at] = !rom[at];
        if !selected.contains(&offset) {
            raw.bus.rom[at] = !rom[at];
        }
    }
    let owner = managed.bus.rom_resources().unwrap().clone();
    let before = owner.tilemap_stream_bank_bytes_read().unwrap();
    check_stream_bus_reads(&mut raw, managed, bank::START, bank::END);
    let bus_bytes = owner.tilemap_stream_bank_bytes_read().unwrap() - before;
    assert_eq!(bus_bytes, bound as u64 * 24);

    let mut entries = [0, 0];
    let mut cells = [0, 0];
    let mut bound_reader_bytes = 0;
    for machine in [&mut raw, &mut *managed] {
        machine.cpu.regs[13] = 0x0300_7e00;
        machine.cpu.cpsr |= FLAG_T | FLAG_I;
    }
    for stream in streams {
        let kind = usize::from(stream.byte_values);
        let reader = [0x080d_a14c, 0x080d_a200][kind];
        for machine in [&mut raw, &mut *managed] {
            machine
                .bus
                .write32(CURSOR, ROM_BASE + stream.extent.start + 6);
            machine.bus.write16(RUN_INDEX, 0);
        }
        let before = owner.tilemap_stream_bank_bytes_read().unwrap();
        let prior_before = owner.observed_tilemap_stats().unwrap().source_bytes;
        let count = u32::from(stream.width) * u32::from(stream.height);
        for _ in 0..count {
            let reference = read_cell(&mut raw, reader);
            let candidate = read_cell(managed, reader);
            assert!(
                candidate == reference,
                "Tilemap cell differs in {:?}",
                stream.extent
            );
            assert_eq!(managed.bus.clock, raw.bus.clock);
        }
        for machine in [&mut raw, &mut *managed] {
            assert_eq!(machine.bus.read32(CURSOR), ROM_BASE + stream.extent.end);
            assert_eq!(machine.bus.read16(RUN_INDEX), 0);
        }
        let after = owner.tilemap_stream_bank_bytes_read().unwrap();
        let prior_after = owner.observed_tilemap_stats().unwrap().source_bytes;
        if prior.contains(&stream.extent.start) {
            assert_eq!(before, after);
            assert!(prior_after > prior_before);
        } else {
            assert!(after > before);
            assert_eq!(prior_after, prior_before);
        }
        bound_reader_bytes += after - before;
        entries[kind] += 1;
        cells[kind] += count;
    }
    assert_eq!(entries, [71, 7]);
    let span = bank::START as usize..bank::END as usize;
    for machine in [&mut raw, &mut *managed] {
        machine.bus.rom[span.clone()].copy_from_slice(&rom[span.clone()]);
    }
    let state = raw.save_state().unwrap();
    assert!(
        state == managed.save_state().unwrap(),
        "Different original reader endpoint"
    );
    serde_json::json!({
        "status": "PASS",
        "stream_counts": entries,
        "cell_counts": cells,
        "occupied_bytes": bank::END - bank::START,
        "data_bytes": selected.len(),
        "prior_bytes": prior.len(),
        "bound_bytes": bound,
        "padding_bytes": (bank::END - bank::START) as usize - selected.len(),
        "bound_bus_bytes": bus_bytes,
        "bound_reader_bytes": bound_reader_bytes,
        "state_sha256": digest(&state),
        "controlled_original_readers": true,
    })
}

#[test]
#[ignore = "exact EUR ROM, E302 generation and retained E450/E457 indexes"]
fn european_tilemap_stream_bank_original_readers() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    let streams = bank::from_rom(&rom).unwrap();
    let selected = verify_extents(&rom, &streams);
    let dataset = input("FFTA_TILEMAP_SLOT1_DATASET");
    let manifest: Manifest = parse(
        &fs::read(dataset.join(MANIFEST_FILE)).unwrap(),
        "E302 manifest",
    )
    .unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (302, 301)
    );
    let mut managed = Machine::new(rom.clone());
    attach_from_directory(&mut managed, &dataset).unwrap();
    let result = verify_original_readers(
        &rom,
        &streams,
        &selected,
        &prior_stream_bytes(&manifest),
        &mut managed,
    );
    fs::write(
        input("FFTA_TILEMAP_STREAM_RESULT"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}

#[test]
#[ignore = "exact EUR ROM and retained E450, E457, E475 and E478 evidence"]
fn european_tilemap_stream_bank_publishes_and_serves_original_readers() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    let streams = bank::from_rom(&rom).unwrap();
    let selected = verify_extents(&rom, &streams);
    let work = input("FFTA_TILEMAP_STREAM_WORK");
    let root = tempfile::tempdir_in(&work).unwrap();
    let mut revision = Extraction::from_rom(&rom).unwrap().manifest;
    (revision.extractor_revision, revision.decoder_revision) = (304, 303);
    let mut source = Extraction::from_rom_for_manifest(&rom, &revision).unwrap();
    let prepared = preparation::prepare(root.path(), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    let dataset = &prepared.directory;
    let manifest_path = dataset.join(MANIFEST_FILE);
    let manifest_bytes = fs::read(&manifest_path).unwrap();
    let published: Manifest = parse(&manifest_bytes, "E304 manifest").unwrap();
    assert_eq!(
        (published.extractor_revision, published.decoder_revision),
        (304, 303)
    );
    assert_eq!(published.resources.len(), 151);
    let stream_resource = published.resources.last().unwrap();
    assert_eq!(stream_resource.id, TILEMAP_STREAM_BANK_ID);
    let stream_path = dataset.join(TILEMAP_STREAM_BANK_FILE);
    let mut encoded = fs::read(&stream_path).unwrap();
    assert_eq!(encoded.len(), 8810);
    assert_eq!(digest(&encoded), stream_resource.sha256);

    // Rebuild the exact E303 manifest from the prefix to verify earlier ownership.
    let mut previous = published.clone();
    previous.resources.pop();
    assert_eq!(
        previous.coverage.pop().as_deref(),
        Some(TILEMAP_STREAM_BANK_ID)
    );
    (previous.extractor_revision, previous.decoder_revision) = (303, 302);
    let e303 = toml::to_string_pretty(&previous).unwrap();
    let precedent = read_index("FFTA_TILEMAP_STREAM_PREDECESSOR");
    assert_eq!(precedent["status"], "PASS");
    assert_eq!(
        precedent["published_manifest_sha256"],
        digest(e303.as_bytes())
    );
    let prior = prior_stream_bytes(&previous);
    let expected: BTreeSet<_> = selected.difference(&prior).copied().collect();
    let mut actual = BTreeSet::new();
    let mut serialized = encoded.as_slice();
    for span in &stream_resource.sources {
        let end = span.offset + span.length;
        let (slice, rest) = serialized.split_at(span.length as usize);
        assert!(slice == &rom[span.offset as usize..end as usize]);
        assert_eq!(digest(slice), span.sha256);
        assert!((span.offset..end).all(|at| actual.insert(at)));
        serialized = rest;
    }
    assert!(serialized.is_empty());
    assert_eq!(actual, expected);

    let reused = preparation::prepare(root.path(), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(reused.action, "reused");
    assert_eq!(&reused.directory, dataset);
    assert!(fs::read(&manifest_path).unwrap() == manifest_bytes);
    drop(source);

    let mut managed = Machine::new(rom.clone());
    fs::write(&manifest_path, e303).unwrap();
    attach_from_directory(&mut managed, dataset).unwrap();
    assert_eq!(
        previous.resources.pop().unwrap().id,
        INDEXED_TILEMAP_CONTROLS_ID
    );
    assert_eq!(
        previous.coverage.pop().as_deref(),
        Some(INDEXED_TILEMAP_CONTROLS_ID)
    );
    (previous.extractor_revision, previous.decoder_revision) = (302, 301);
    let e302 = toml::to_string_pretty(&previous).unwrap();
    assert_eq!(
        precedent["predecessor_manifest_sha256"],
        digest(e302.as_bytes())
    );
    fs::write(&manifest_path, e302).unwrap();
    attach_from_directory(&mut managed, dataset).unwrap();
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    attach_from_directory(&mut managed, dataset).unwrap();

    let active = managed.bus.rom_resources().unwrap().clone();
    let mut edited = published.clone();
    encoded[0] ^= 1;
    edited.resources.last_mut().unwrap().sha256 = digest(&encoded);
    fs::write(&stream_path, &encoded).unwrap();
    fs::write(&manifest_path, toml::to_string_pretty(&edited).unwrap()).unwrap();
    assert_eq!(
        attach_from_directory(&mut managed, dataset).unwrap_err(),
        "Tilemap stream bank differs from authenticated source",
    );
    assert!(std::sync::Arc::ptr_eq(
        &active,
        managed.bus.rom_resources().unwrap()
    ));

    // The live owner must keep serving verified bytes even when its input file
    // and the machine's cartridge backing change after attachment.
    let mut result = verify_original_readers(&rom, &streams, &selected, &prior, &mut managed);
    result["revision"] = serde_json::json!([304, 303]);
    result["published_stream_bytes"] = serde_json::json!(encoded.len());
    result["source_ranges"] = serde_json::json!(stream_resource.sources.len());
    result["source_file_sha256"] = serde_json::json!(stream_resource.sha256);
    result["published_manifest_sha256"] = serde_json::json!(digest(&manifest_bytes));
    result["default_writer_publication_and_reuse"] = serde_json::json!(true);
    result["e302_and_e303_attach"] = serde_json::json!(true);
    result["coherent_edit_rejected"] = serde_json::json!(true);
    root.close().unwrap();

    fs::write(work.join(MANIFEST_FILE), manifest_bytes).unwrap();
    fs::write(
        input("FFTA_TILEMAP_STREAM_RESULT"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}

#[derive(serde::Deserialize)]
pub(super) struct SlotEvidence {
    task_id: u32,
    slot: usize,
    pointer_cell: u32,
    transfer_row: u32,
    descriptor: u32,
    root: u32,
    end: u32,
}

impl SlotEvidence {
    pub(super) fn control_ranges(&self) -> [Range<u32>; 3] {
        [
            self.pointer_cell..self.pointer_cell + 4,
            self.transfer_row..self.transfer_row + 12,
            self.descriptor..self.descriptor + 8,
        ]
    }
}

fn alter_backing(machine: &mut Machine, rom: &[u8], ranges: &[Range<u32>], invert: bool) {
    for range in ranges {
        for offset in range.clone() {
            let at = offset as usize;
            machine.bus.rom[at] = if invert { !rom[at] } else { rom[at] };
        }
    }
}

pub(super) fn check_indexed_controls(
    rom: &[u8],
    manifest: &Manifest,
    groups: &BTreeMap<String, Vec<SlotEvidence>>,
    raw: &mut Machine,
    managed: &mut Machine,
) -> [usize; 3] {
    let prior: Vec<_> = manifest
        .resources
        .iter()
        .flat_map(|resource| &resource.sources)
        .map(|source| source.offset..source.offset + source.length)
        .collect();
    let ranges: Vec<_> = groups
        .values()
        .flatten()
        .flat_map(SlotEvidence::control_ranges)
        .collect();
    let distinct: BTreeSet<_> = ranges.iter().cloned().flatten().collect();
    assert_eq!(distinct.len(), 288);
    let raw_lane = 0x39bc60;
    assert!(!distinct.contains(&raw_lane) && prior.iter().all(|span| !span.contains(&raw_lane)));
    for machine in [&mut *raw, &mut *managed] {
        alter_backing(machine, rom, &[raw_lane..raw_lane + 1], true);
    }
    alter_backing(managed, rom, &ranges, true);
    let owner = managed.bus.rom_resources().unwrap().clone();
    let mut unowned = [0; 3];
    assert_eq!(groups.len(), 3);
    for (group, (table, slots)) in [("39bcbc", 2), ("3a1c58", 2), ("3a4d84", 8)]
        .into_iter()
        .enumerate()
    {
        let entries = &groups[table];
        assert_eq!(entries.len(), slots);
        let before = owner.tilemap_indexed_control_bytes_read().unwrap();
        for (index, slot) in entries.iter().enumerate() {
            assert_eq!(index, slot.slot);
            for span in slot.control_ranges() {
                unowned[group] += span
                    .clone()
                    .filter(|at| prior.iter().all(|old| !old.contains(at)))
                    .count();
                check_stream_bus_reads(raw, managed, span.start, span.end);
            }
        }
        let after = owner.tilemap_indexed_control_bytes_read().unwrap();
        if unowned[group] == 0 {
            assert_eq!(after, before);
        } else {
            assert!(after - before >= unowned[group] as u64 * 24);
        }
    }
    assert_eq!(unowned, [48, 48, 0]);
    alter_backing(managed, rom, &ranges, false);
    for machine in [&mut *raw, &mut *managed] {
        alter_backing(machine, rom, &[raw_lane..raw_lane + 1], false);
    }
    unowned
}

#[test]
#[ignore = "exact EUR ROM, E302 generation and both retained E476 action previews"]
fn european_indexed_tilemap_controls_and_ordinary_replays() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    let dataset = input("FFTA_TILEMAP_SLOT1_DATASET");
    let manifest: Manifest = parse(
        &fs::read(dataset.join(MANIFEST_FILE)).unwrap(),
        "E302 manifest",
    )
    .unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (302, 301)
    );
    let groups: BTreeMap<String, Vec<SlotEvidence>> =
        serde_json::from_value(read_index("FFTA_TILEMAP_INDEXED_GROUPS")).unwrap();
    let reference = read_index("FFTA_TILEMAP_INDEXED_REFERENCE");
    assert_eq!(reference["status"], "PASS");
    assert_eq!(reference["source_rom_sha256"], manifest.source_rom_sha256);
    let routes = read_index("FFTA_TILEMAP_INDEXED_ROUTES");
    assert_eq!(routes["status"], "PASS");

    let mut raw = Machine::new(rom.clone());
    let mut managed = Machine::new(rom.clone());
    attach_from_directory(&mut managed, &dataset).unwrap();
    let unowned = check_indexed_controls(&rom, &manifest, &groups, &mut raw, &mut managed);
    eprintln!("Verified managed reads for all 12 indexed slots");
    let owner = managed.bus.rom_resources().unwrap().clone();
    let mut cases = BTreeMap::new();
    // The E476 inputs reach dispatch state 36 by guest frame 424 of each replay.
    let steps = [(8, 0x3fe), (24, 0x3ff), (8, 0x3fe), (385, 0x3ff)];
    let frame_count = steps
        .iter()
        .map(|(count, _)| u64::from(*count))
        .sum::<u64>();
    const DIRECT_STATE: u32 = 0x0a95c4;
    const FINISH_STATE: u32 = 0x0a95dc;
    for (slot, (name, fixture)) in groups["39bcbc"].iter().zip([
        ("shell", "FFTA_TILEMAP_INDEXED_SHELL"),
        ("protect", "FFTA_TILEMAP_INDEXED_PROTECT"),
    ]) {
        let expected = &reference["cases"][name];
        assert_eq!(expected["task"], slot.task_id);
        assert_eq!(expected["slot"].as_u64().unwrap() as usize, slot.slot);
        assert_eq!(expected["table_word"], slot.pointer_cell);
        assert_eq!(expected["stream"], serde_json::json!([slot.root, slot.end]));
        let route = &routes["cases"][name];
        let finish_frame = route["state_table_reads"]["36"]["last"].as_u64().unwrap();
        let snapshot = fs::read(input(fixture)).unwrap();
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&snapshot).unwrap();
        }
        assert!(std::sync::Arc::ptr_eq(
            &owner,
            managed.bus.rom_resources().unwrap()
        ));
        let first = raw.bus.frames;
        assert_eq!(Some(first), route["first_frame"].as_u64());
        assert!(finish_frame < first + frame_count);
        let [pointer, row, descriptor] = slot.control_ranges();
        let ranges = [pointer, row, descriptor, slot.root..slot.end];
        alter_backing(&mut managed, &rom, &ranges, true);
        let control_before = owner.tilemap_indexed_control_bytes_read().unwrap();
        let stream_before = owner.tilemap_stream_bank_bytes_read().unwrap();
        let capture = TraceCapture::start(
            TraceFilter {
                readers: Vec::new(),
                ranges: ranges
                    .iter()
                    .map(|span| (span.start, span.end))
                    .chain([
                        (DIRECT_STATE, DIRECT_STATE + 4),
                        (FINISH_STATE, FINISH_STATE + 4),
                    ])
                    .collect(),
            },
            4096,
        )
        .unwrap();
        let samples = replay_pair(&mut raw, &mut managed, &steps, |frame| capture.frame(frame));
        let events = capture.finish();
        assert_eq!(raw.bus.frames, first + frame_count);
        assert_eq!(events.dropped_requests, 0);
        assert!(!events.truncated && samples > 0);
        let mut typed: [BTreeSet<u32>; 4] = std::array::from_fn(|_| BTreeSet::new());
        for event in &events.events {
            let read = event.key.read;
            let end = read.physical_offset + read.length as u32;
            for (index, range) in ranges.iter().enumerate() {
                let begin = read.physical_offset.max(range.start);
                let end = end.min(range.end);
                if begin < end {
                    assert_eq!(event.key.service, Service::Typed, "{event:?}");
                    typed[index].extend(begin..end);
                }
            }
        }
        let observed = typed.map(|bytes| bytes.len());
        assert_eq!(observed, [4, 12, 7, 10]);
        for (offset, pc, frame) in [
            (DIRECT_STATE, 0x080a9532, expected["direct_frame"].as_u64()),
            (
                slot.pointer_cell,
                0x080e8442,
                expected["geometry_frame"].as_u64(),
            ),
            (FINISH_STATE, 0x080a9532, Some(finish_frame)),
        ] {
            assert!(events.events.iter().any(|event| {
                event.key.read.physical_offset == offset
                    && event.key.read.length == 4
                    && event.key.instruction.is_some_and(|site| site.pc == pc)
                    && frame == Some(event.first_guest_frame)
            }));
        }
        let control_bytes = owner.tilemap_indexed_control_bytes_read().unwrap() - control_before;
        let stream_bytes = owner.tilemap_stream_bank_bytes_read().unwrap() - stream_before;
        assert!(control_bytes >= 23 && stream_bytes >= 10);
        alter_backing(&mut managed, &rom, &ranges, false);
        assert!(raw.bus.framebuffer == managed.bus.framebuffer && raw.bus.oam == managed.bus.oam);
        let endpoint = raw.save_state().unwrap();
        assert!(
            endpoint == managed.save_state().unwrap(),
            "Different {name} endpoint"
        );
        eprintln!("Matched {name} replay through dispatch state 36");
        cases.insert(
            name,
            serde_json::json!({
                "task": slot.task_id,
                "first_frame": first,
                "last_frame": raw.bus.frames,
                "direct_frame": expected["direct_frame"],
                "geometry_frame": expected["geometry_frame"],
                "finish_frame": finish_frame,
                "typed_bytes": observed,
                "control_access_bytes": control_bytes,
                "stream_access_bytes": stream_bytes,
                "audio_samples": samples,
                "endpoint_sha256": digest(&endpoint),
            }),
        );
    }
    let result = serde_json::json!({
        "status": "PASS",
        "manifest_source": manifest.source_rom_sha256,
        "indexed_slots": 12,
        "metadata_bytes_by_group": unowned,
        "previously_managed_bytes": 192,
        "cases": cases,
        "guest_replays_equal": true,
    });
    fs::write(
        input("FFTA_TILEMAP_INDEXED_RESULT"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
