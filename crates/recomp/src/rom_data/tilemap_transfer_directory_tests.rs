//! Ownership and ordinary interpreter use of the European transfer bank.
use super::*;
use gba_core::{capi::RT_API, ffta_rom::digest, rom_data::SourceRange, Bus, Machine};
use std::{collections::BTreeSet, ffi::c_void};

#[cfg(feature = "rom-data-read-audit")]
use gba_core::rom_data::read_audit::TraceReport;

fn input(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os(name).unwrap_or_else(|| panic!("Missing {name}")))
}

fn set_backing(machine: &mut Machine, rom: &[u8], sources: &[SourceRange], altered: bool) {
    for source in sources {
        let start = source.offset as usize;
        let end = start + source.length as usize;
        for at in start..end {
            machine.bus.rom[at] = if altered { !rom[at] } else { rom[at] };
        }
    }
}

fn assert_bank_owners(sources: &[SourceRange], reference: &serde_json::Value) {
    let banks = reference["banks"].as_array().unwrap();
    for (role, bank, count) in [
        ("list-pointer", "list-pointers", 84),
        ("list-count", "list-counts", 21),
        ("transfer-row", "rows", 192),
        ("transfer-descriptor", "descriptors", 112),
        ("stream-wrapper", "wrappers", 52),
    ] {
        let group = banks.iter().find(|v| v["id"] == bank).unwrap();
        let expected: BTreeSet<_> = group["segments"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|segment| segment["owners"].as_array().unwrap().is_empty())
            .flat_map(|segment| {
                segment["start"].as_u64().unwrap()..segment["end"].as_u64().unwrap()
            })
            .collect();
        let actual: BTreeSet<_> = sources
            .iter()
            .filter(|source| source.role == role)
            .flat_map(|source| u64::from(source.offset)..u64::from(source.offset + source.length))
            .collect();
        assert_eq!(actual, expected, "{role}");
        assert_eq!(actual.len(), count, "{role}");
    }
    assert_eq!(sources.iter().map(|span| span.length).sum::<u32>(), 461);
}

fn check_reads(raw: &mut Machine, managed: &mut Machine, sources: &[SourceRange]) {
    let raw_ptr = (raw as *mut Machine).cast::<c_void>();
    let managed_ptr = (managed as *mut Machine).cast::<c_void>();
    for source in sources {
        for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
            for offset in source.offset..source.offset + source.length {
                let address = mirror + offset;
                assert_eq!(
                    managed.bus.read8(address),
                    raw.bus.read8(address),
                    "byte {address:#x}"
                );
            }
            for edge in [source.offset, source.offset + source.length] {
                for address in mirror + edge - 3..=mirror + edge + 1 {
                    assert_eq!(
                        (RT_API.read16)(managed_ptr, address),
                        (RT_API.read16)(raw_ptr, address),
                        "halfword {address:#x}"
                    );
                    assert_eq!(
                        (RT_API.read32)(managed_ptr, address),
                        (RT_API.read32)(raw_ptr, address),
                        "word {address:#x}"
                    );
                }
            }
        }
    }
}

pub(super) fn replay_pair(
    raw: &mut Machine,
    managed: &mut Machine,
    steps: &[(u32, u16)],
    mut trace_frame: impl FnMut(Option<u64>),
) -> usize {
    let mut samples = 0;
    for &(frames, keys) in steps {
        raw.bus.keys = keys;
        managed.bus.keys = keys;
        for _ in 0..frames {
            let frame = raw.bus.frames;
            trace_frame(None);
            raw.run_frame(5_000_000);
            trace_frame(Some(frame));
            managed.run_frame(5_000_000);
            assert_eq!(raw.bus.frames, frame + 1);
            assert_eq!(managed.bus.frames, frame + 1);
            assert_eq!(raw.bus.clock, managed.bus.clock, "guest clock at {frame}");
            assert!(
                raw.bus.audio_buf == managed.bus.audio_buf,
                "audio differs at {frame}"
            );
            samples += raw.bus.audio_buf.len();
            for machine in [&mut *raw, &mut *managed] {
                machine.bus.audio_buf.clear();
                machine.bus.psg_tap.clear();
                for fifo in &mut machine.bus.fifo_tap {
                    fifo.clear();
                }
                machine.bus.hle_tap.clear();
            }
        }
    }
    samples
}

#[test]
#[ignore = "exact EUR ROM, E301 resource generation and the retained Shell preview"]
fn european_task_345_transfer_directory_managed_shell() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    let dataset = input("FFTA_TILEMAP_BANK_DATASET");
    let previous: Manifest = parse(
        &fs::read(input("FFTA_TILEMAP_BANK_PREDECESSOR").join(MANIFEST_FILE)).unwrap(),
        "E301 manifest",
    )
    .unwrap();
    assert_eq!(
        (previous.extractor_revision, previous.decoder_revision),
        (301, 300)
    );
    assert_eq!(previous.resources.len(), 148);

    let mut revision = previous.clone();
    (revision.extractor_revision, revision.decoder_revision) = (302, 301);
    let source = Extraction::from_rom_for_manifest(&rom, &revision).unwrap();
    let catalog = source.tilemap_transfer_directory.as_ref().unwrap();
    let metadata = &catalog.decoded;
    assert_eq!(metadata.slots.len(), 23);
    assert_eq!(metadata.rows.len(), 18);
    assert_eq!(metadata.descriptors.len(), 16);
    let nulls: Vec<_> = metadata
        .slots
        .iter()
        .enumerate()
        .filter_map(|(index, slot)| (slot.row_pointer == 0).then_some(index))
        .collect();
    assert_eq!(nulls, [16, 19, 20]);
    let plan: serde_json::Value =
        serde_json::from_slice(&fs::read(input("FFTA_TILEMAP_BANK_PLAN")).unwrap()).unwrap();
    assert_eq!(
        serde_json::to_value(tilemap_bank_plan::plan(&rom).unwrap()).unwrap(),
        plan
    );
    let ownership: serde_json::Value =
        serde_json::from_slice(&fs::read(input("FFTA_TILEMAP_BANK_OWNERSHIP")).unwrap()).unwrap();
    assert_bank_owners(&catalog.sources, &ownership);

    let mut manifest = source.manifest.clone();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (302, 301)
    );
    assert_eq!(manifest.resources.len(), previous.resources.len() + 1);
    assert_eq!(
        &manifest.coverage[..previous.coverage.len()],
        &previous.coverage
    );
    for (candidate, prior) in manifest.resources.iter_mut().zip(&previous.resources) {
        candidate.sha256.clone_from(&prior.sha256);
        assert_eq!(candidate, prior);
    }
    let bytes = encode_tilemap_transfer_directory(metadata).unwrap();
    let resource = manifest.resources.last_mut().unwrap();
    assert_eq!(resource.id, TILEMAP_TRANSFER_DIRECTORY_ID);
    resource.sha256 = digest(&bytes);
    fs::write(dataset.join(TILEMAP_TRANSFER_DIRECTORY_FILE), &bytes).unwrap();
    fs::write(
        dataset.join(MANIFEST_FILE),
        toml::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();

    let mut invalid = metadata.clone();
    invalid.slots[16].row_pointer = invalid.slots[0].row_pointer;
    assert!(parse_tilemap_transfer_directory(
        &encode_tilemap_transfer_directory(&invalid).unwrap()
    )
    .is_err());
    let loaded = read_dataset(&dataset).unwrap();
    let mut forged = metadata.clone();
    forged.descriptors[0].parameter8 ^= 1;
    let forged_bytes = encode_tilemap_transfer_directory(&forged).unwrap();
    let mut forged_manifest = manifest.clone();
    forged_manifest.resources.last_mut().unwrap().sha256 = digest(&forged_bytes);
    let mut files = loaded.files();
    files.manifest = &forged_manifest;
    files.tilemap_transfer_directory = Some((&forged, &forged_bytes));
    assert_eq!(
        source.validate_files(files).unwrap_err(),
        "Decoded tilemap transfer directory differs from authenticated source",
    );
    drop(loaded);

    let mut raw = Machine::new(rom.clone());
    let mut managed = Machine::new(rom.clone());
    attach_from_directory(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    set_backing(&mut managed, &rom, &catalog.sources, true);
    check_reads(&mut raw, &mut managed, &catalog.sources);
    set_backing(&mut managed, &rom, &catalog.sources, false);

    let snapshot = fs::read(input("FFTA_TILEMAP_BANK_SHELL")).unwrap();
    raw.load_state(&snapshot).unwrap();
    managed.load_state(&snapshot).unwrap();
    assert!(std::sync::Arc::ptr_eq(
        &owner,
        managed.bus.rom_resources().unwrap()
    ));
    let first = raw.bus.frames;
    assert_eq!(first, 7737);
    set_backing(&mut managed, &rom, &catalog.sources, true);
    let before = owner.tilemap_transfer_directory_stats().unwrap();
    let steps = [(8, 0x3fe), (24, 0x3ff), (8, 0x3fe), (260, 0x3ff)];
    let samples = replay_pair(&mut raw, &mut managed, &steps, |_| {});
    assert_eq!(raw.bus.frames - first, 300);
    assert!(samples > 0);
    let after = owner.tilemap_transfer_directory_stats().unwrap();
    let reads = [
        after.pointer_bytes - before.pointer_bytes,
        after.count_bytes - before.count_bytes,
        after.row_bytes - before.row_bytes,
        after.descriptor_bytes - before.descriptor_bytes,
        after.wrapper_bytes - before.wrapper_bytes,
    ];
    assert!(
        reads
            .into_iter()
            .zip([4, 1, 12, 5, 2])
            .all(|(actual, minimum)| actual >= minimum),
        "{reads:?}"
    );
    set_backing(&mut managed, &rom, &catalog.sources, false);
    assert!(raw.bus.framebuffer == managed.bus.framebuffer);
    assert!(raw.bus.oam == managed.bus.oam);
    let end = raw.save_state().unwrap();
    assert!(
        end == managed.save_state().unwrap(),
        "Different managed endpoint"
    );

    let report = serde_json::json!({
        "status": "PASS",
        "source_rom_sha256": source.manifest.source_rom_sha256,
        "revision": [302, 301],
        "previous_resources_preserved": previous.resources.len(),
        "source_bytes": 461,
        "ordinary_reads_by_role": reads,
        "guest_first_frame": first,
        "guest_last_frame": raw.bus.frames,
        "guest_clock": raw.bus.clock,
        "audio_samples": samples,
        "end_state_sha256": digest(&end),
        "old_payloads_reused": true,
        "managed_state_equivalent": true,
    });
    fs::write(
        input("FFTA_TILEMAP_BANK_RESULT"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
pub(super) fn check_stream_bus_reads(
    raw: &mut Machine,
    managed: &mut Machine,
    start: u32,
    end: u32,
) {
    let raw_ptr = (raw as *mut Machine).cast::<c_void>();
    let managed_ptr = (managed as *mut Machine).cast::<c_void>();
    for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
        for offset in start - 3..=end {
            let address = mirror + offset;
            assert!(
                managed.bus.read8(address) == raw.bus.read8(address),
                "Bus byte differs at {address:#x}"
            );
            for (width, read) in [(1, RT_API.read8), (2, RT_API.read16), (4, RT_API.read32)] {
                assert!(
                    read(managed_ptr, address) == read(raw_ptr, address),
                    "ABI width {width} differs at {address:#x}"
                );
            }
        }
    }
}

#[cfg(feature = "rom-data-read-audit")]
fn check_stream_dma_boundary(raw: &mut Machine, managed: &mut Machine, end: usize) -> u64 {
    let last_word = end - 4..end;
    assert_eq!(end % 4, 0);
    for at in last_word.clone() {
        managed.bus.rom[at] = !raw.bus.rom[at];
    }
    let owner = managed.bus.rom_resources().unwrap().clone();
    let before = owner.tilemap_transfer_directory_stats().unwrap();
    for (count, control) in [(4, 0x8000), (2, 0x8400)] {
        for machine in [&mut *raw, &mut *managed] {
            machine.bus.ewram[0x100..0x108].fill(0);
            machine
                .bus
                .write32(0x0400_00d4, 0x0a00_0000 + end as u32 - 4);
            machine.bus.write32(0x0400_00d8, 0x0200_0100);
            machine.bus.write16(0x0400_00dc, count);
            machine.bus.write16(0x0400_00de, control);
        }
        assert!(
            managed.bus.ewram[0x100..0x108] == raw.bus.rom[end - 4..end + 4]
                && managed.bus.ewram == raw.bus.ewram,
            "DMA source bytes differ for control {control:#x}"
        );
        assert_eq!(managed.bus.clock, raw.bus.clock);
        assert_eq!(managed.bus.read16(0x0400_00de), raw.bus.read16(0x0400_00de));
    }
    managed.bus.rom[last_word.clone()].copy_from_slice(&raw.bus.rom[last_word]);
    let after = owner.tilemap_transfer_directory_stats().unwrap();
    let bytes = after.bound_lz77_bus_bytes - before.bound_lz77_bus_bytes;
    assert_eq!(bytes, 8);
    bytes
}

#[cfg(feature = "rom-data-read-audit")]
fn check_bank_lz77_inputs(
    rom: &[u8],
    catalog: &TilemapTransferDirectory,
    manifest: &Manifest,
    raw: &mut Machine,
    managed: &mut Machine,
) -> serde_json::Value {
    let plan: serde_json::Value =
        serde_json::from_slice(&fs::read(input("FFTA_TILEMAP_BANK_PLAN")).unwrap()).unwrap();
    assert_eq!(
        serde_json::to_value(tilemap_bank_plan::plan(rom).unwrap()).unwrap(),
        plan
    );
    let ownership: serde_json::Value =
        serde_json::from_slice(&fs::read(input("FFTA_TILEMAP_BANK_OWNERSHIP")).unwrap()).unwrap();
    let bank = ownership["banks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|bank| bank["id"] == "lz77-inputs")
        .unwrap();
    let segments = bank["segments"].as_array().unwrap();
    let ranges = catalog.lz77_input_ranges(rom).unwrap();
    assert_eq!(ranges.len(), 15);
    assert_eq!(ranges.len(), segments.len());
    if let Some(file) = manifest
        .resources
        .iter()
        .find(|item| item.id == TILEMAP_TRANSFER_LZ77_ID)
    {
        let uncovered = segments
            .iter()
            .filter(|segment| segment["owners"].as_array().unwrap().is_empty())
            .map(|segment| {
                (
                    segment["start"].as_u64().unwrap(),
                    segment["end"].as_u64().unwrap(),
                )
            });
        assert!(
            file.sources
                .iter()
                .map(|source| {
                    (
                        u64::from(source.offset),
                        u64::from(source.offset + source.length),
                    )
                })
                .eq(uncovered)
        );
    }
    let owner = managed.bus.rom_resources().unwrap().clone();
    let mut bound_streams = 0;
    let mut bound_bytes = 0u32;
    let mut bus_bytes = 0u64;
    let mut prior_bus_bytes = 0u64;
    let mut lz77_checks = 0;
    // This adjacent byte belongs to neither a stream nor an earlier resource.
    // Alter it in both machines to check mixed raw/managed reads at the boundary.
    let gap = ranges[0].end;
    assert!(ranges.iter().all(|range| !range.contains(&gap)));
    assert!(manifest
        .resources
        .iter()
        .flat_map(|item| &item.sources)
        .all(|source| !(source.offset..source.offset + source.length).contains(&gap)));
    let gap = gap as usize;
    raw.bus.rom[gap] = !rom[gap];
    managed.bus.rom[gap] = !rom[gap];
    for (range, segment) in ranges.iter().zip(segments) {
        assert_eq!(segment["start"], range.start);
        assert_eq!(segment["end"], range.end);
        let prior = segment["owners"].as_array().unwrap();
        let bound = prior.is_empty();
        if bound {
            bound_streams += 1;
            bound_bytes += range.end - range.start;
        } else {
            assert_eq!(prior, &[serde_json::json!(OBSERVED_LZ_ID)]);
        }
        let span = range.start as usize..range.end as usize;
        for at in span.clone() {
            managed.bus.rom[at] = !rom[at];
        }
        let before = owner.tilemap_transfer_directory_stats().unwrap();
        let old_before = owner.observed_lz_stats().unwrap();
        check_stream_bus_reads(raw, managed, range.start, range.end);
        let after = owner.tilemap_transfer_directory_stats().unwrap();
        let old_after = owner.observed_lz_stats().unwrap();
        let new_bytes = after.bound_lz77_bus_bytes - before.bound_lz77_bus_bytes;
        let old_bytes = old_after.source_bytes - old_before.source_bytes;
        // One direct byte and ABI widths 1, 2 and 4 at each address on three mirrors.
        let expected_bytes = u64::from(range.end - range.start) * (1 + 1 + 2 + 4) * 3;
        let expected = if bound {
            (expected_bytes, 0)
        } else {
            (0, expected_bytes)
        };
        assert_eq!((new_bytes, old_bytes), expected, "Stream {range:#x?}");
        bus_bytes += new_bytes;
        prior_bus_bytes += old_bytes;

        managed.bus.rom[span.clone()].fill(0);
        for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
            let source = mirror + range.start;
            assert!(managed.bus.bound_tilemap_lz77(source + 1).is_none());
            for (swi, destination) in [(0x11, 0x0200_0020), (0x12, 0x0600_0020)] {
                let before = owner.tilemap_transfer_directory_stats().unwrap();
                let old_before = owner.observed_lz_stats().unwrap();
                for machine in [&mut *raw, &mut *managed] {
                    machine.bus.ewram.fill(0x63);
                    machine.bus.vram.fill(0x63);
                    machine.cpu.regs[0] = source;
                    machine.cpu.regs[1] = destination;
                    assert!(gba_core::hle::bios_call(
                        &mut machine.cpu,
                        &mut machine.bus,
                        swi
                    ));
                }
                assert!(
                    raw.bus.ewram == managed.bus.ewram && raw.bus.vram == managed.bus.vram,
                    "SWI {swi:#x} destination differs for source {source:#x}"
                );
                assert_eq!(raw.cpu.regs, managed.cpu.regs);
                assert_eq!(raw.bus.clock, managed.bus.clock);
                let after = owner.tilemap_transfer_directory_stats().unwrap();
                let old_after = owner.observed_lz_stats().unwrap();
                assert_eq!(
                    after.bound_lz77_calls - before.bound_lz77_calls,
                    u64::from(bound)
                );
                assert_eq!(
                    after.bound_lz77_input_bytes - before.bound_lz77_input_bytes,
                    if bound {
                        u64::from(range.end - range.start)
                    } else {
                        0
                    },
                );
                assert_eq!(
                    old_after.hle_calls - old_before.hle_calls,
                    u64::from(!bound)
                );
                lz77_checks += 1;
            }
        }
        managed.bus.rom[span.clone()].copy_from_slice(&rom[span]);
    }
    let dma_bytes = check_stream_dma_boundary(raw, managed, gap);
    raw.bus.rom[gap] = rom[gap];
    managed.bus.rom[gap] = rom[gap];
    assert_eq!((bound_streams, bound_bytes, lz77_checks), (13, 3742, 90));
    serde_json::json!({
        "distinct_inputs": ranges.len(),
        "bound_streams": bound_streams,
        "bound_input_bytes": bound_bytes,
        "bound_bus_bytes": bus_bytes,
        "prior_bus_bytes": prior_bus_bytes,
        "bound_dma_bytes": dma_bytes,
        "raw_boundary_checked": true,
        "earlier_stream_owners_used": ranges.len() - bound_streams,
        "controlled_swi_checks": lz77_checks,
    })
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "exact EUR ROM, E302 generation, E466/E467 bank and E471 IN-08 action preview"]
fn european_task_345_bank_lz77_binding() {
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
    let mut managed = Machine::new(rom.clone());
    attach_from_directory(&mut managed, &dataset).unwrap();
    let (result, report) = check_task_345_lz77_replay(&rom, &dataset, &manifest, managed);
    write_lz77_result(&result, &report);
}

#[cfg(feature = "rom-data-read-audit")]
fn check_task_345_lz77_replay(
    rom: &[u8],
    dataset: &Path,
    manifest: &Manifest,
    mut managed: Machine,
) -> (serde_json::Value, TraceReport) {
    use gba_core::rom_data::{
        read_audit::{Service, TraceCapture, TraceFilter},
        tilemap_transfer_directory as model,
    };

    let catalog = parse_tilemap_transfer_directory(
        &fs::read(dataset.join(TILEMAP_TRANSFER_DIRECTORY_FILE)).unwrap(),
    )
    .unwrap();
    let offsets: Vec<_> = catalog.slots[1].row_offsets().unwrap().collect();
    let &[offset] = offsets.as_slice() else {
        panic!("Expected one row for slot 1");
    };
    let row = catalog
        .rows
        .iter()
        .find(|row| row.source_offset == offset)
        .unwrap();
    let descriptor = catalog
        .descriptors
        .iter()
        .find(|item| model::ROM_BASE + item.source_offset == row.descriptor_pointer)
        .unwrap();
    let resource = manifest
        .resources
        .iter()
        .find(|item| item.id == TILEMAP_TRANSFER_DIRECTORY_ID)
        .unwrap();
    let controls = [
        ("list-pointer", model::POINTER_DIRECTORY + 4, 4),
        ("list-count", model::COUNT_DIRECTORY + 1, 1),
        ("transfer-row", row.source_offset, model::ROW_BYTES),
        ("transfer-descriptor", descriptor.source_offset, 8),
        ("stream-wrapper", descriptor.wrapper.source_offset, 4),
    ];
    let sources: Vec<_> = controls
        .iter()
        .map(|&(role, offset, length)| {
            resource
                .sources
                .iter()
                .find(|source| {
                    source.role == role && source.offset == offset && source.length == length
                })
                .unwrap()
                .clone()
        })
        .collect();
    let reference: serde_json::Value =
        serde_json::from_slice(&fs::read(input("FFTA_TILEMAP_SLOT1_REFERENCE")).unwrap()).unwrap();
    assert_eq!(reference["status"], "PASS");
    assert_eq!(reference["verified_slot1_target"], true);
    assert_eq!(reference["source_rom_sha256"], manifest.source_rom_sha256);
    let stream_start = descriptor.wrapper.source_offset + 4;
    let stream_end = stream_start + 57;
    assert_eq!(
        reference["stream"],
        serde_json::json!([stream_start, stream_end])
    );
    assert!(manifest
        .resources
        .iter()
        .filter(|item| item.id != TILEMAP_TRANSFER_LZ77_ID)
        .flat_map(|item| &item.sources)
        .all(|span| { span.offset >= stream_end || span.offset + span.length <= stream_start }));
    let ranges = sources
        .iter()
        .map(|span| (span.offset, span.offset + span.length))
        .chain([(stream_start, stream_end)])
        .collect();

    let steps: Vec<_> = fs::read_to_string(input("FFTA_TILEMAP_SLOT1_INPUT"))
        .unwrap()
        .lines()
        .map(|line| {
            let (frames, keys) = line.split_once(' ').unwrap();
            let keys = u16::from_str_radix(keys, 16).unwrap();
            assert!(keys <= 0x03ff);
            (frames.parse::<u32>().unwrap(), keys)
        })
        .collect();
    let snapshot = fs::read(input("FFTA_TILEMAP_SLOT1_SNAPSHOT")).unwrap();
    assert_eq!(reference["preview_sha256"], digest(&snapshot));
    let mut raw = Machine::new(rom.to_vec());
    let bank = check_bank_lz77_inputs(rom, &catalog, manifest, &mut raw, &mut managed);
    let owner = managed.bus.rom_resources().unwrap().clone();
    for machine in [&mut raw, &mut managed] {
        machine.load_state(&snapshot).unwrap();
    }
    let first = raw.bus.frames;
    assert_eq!(reference["first_guest_frame"].as_u64().unwrap(), first);

    set_backing(&mut managed, rom, &sources, true);
    let stream = stream_start as usize..stream_end as usize;
    managed.bus.rom[stream.clone()].fill(0);
    let before = owner.tilemap_transfer_directory_stats().unwrap();
    let capture = TraceCapture::start(
        TraceFilter {
            readers: Vec::new(),
            ranges,
        },
        4096,
    )
    .unwrap();
    let samples = replay_pair(&mut raw, &mut managed, &steps, |frame| capture.frame(frame));
    let report = capture.finish();
    assert_eq!(raw.bus.frames - first, 300);
    assert!(samples > 0);
    assert_eq!(report.dropped_requests, 0);
    assert!(!report.truncated);
    let after = owner.tilemap_transfer_directory_stats().unwrap();
    let bound_calls = after.bound_lz77_calls - before.bound_lz77_calls;
    let bound_bytes = after.bound_lz77_input_bytes - before.bound_lz77_input_bytes;
    assert_eq!((bound_calls, bound_bytes), (1, 57));

    let mut typed: [BTreeSet<u32>; 5] = std::array::from_fn(|_| BTreeSet::new());
    let mut raw_stream = BTreeSet::new();
    for event in &report.events {
        let read = event.key.read;
        let start = read.physical_offset;
        let end = start + read.length as u32;
        for (index, source) in sources.iter().enumerate() {
            let lower = start.max(source.offset);
            let upper = end.min(source.offset + source.length);
            if lower < upper {
                assert_eq!(event.key.service, Service::Typed, "{event:?}");
                typed[index].extend(lower..upper);
            }
        }
        if start < stream_end
            && end > stream_start
            && event
                .key
                .instruction
                .is_some_and(|site| site.pc == 0x0814_5254)
        {
            assert_eq!(event.key.service, Service::Raw, "{event:?}");
            raw_stream.extend(start.max(stream_start)..end.min(stream_end));
        }
    }
    let ordinary_reads = typed.map(|bytes| bytes.len());
    assert_eq!(ordinary_reads, [4, 1, 12, 5, 2]);
    assert!(raw_stream.is_empty());
    for ((_, address, length), (pc, frame_key)) in controls[..2].iter().zip([
        (0x080e_61cc, "first_pointer_frame"),
        (0x080e_61d6, "first_pointer_frame"),
    ]) {
        assert!(report.events.iter().any(|event| {
            event.key.service == Service::Typed
                && event.key.read.physical_offset == *address
                && event.key.read.length as u32 == *length
                && event.key.instruction.is_some_and(|site| site.pc == pc)
                && reference["slot_1"][frame_key].as_u64() == Some(event.first_guest_frame)
        }));
    }
    set_backing(&mut managed, rom, &sources, false);
    managed.bus.rom[stream.clone()].copy_from_slice(&rom[stream]);
    assert!(raw.bus.framebuffer == managed.bus.framebuffer);
    assert!(raw.bus.oam == managed.bus.oam);
    let endpoint = raw.save_state().unwrap();
    assert!(
        endpoint == managed.save_state().unwrap(),
        "Different managed endpoint"
    );

    let result = serde_json::json!({
        "status": "PASS",
        "source_rom_sha256": manifest.source_rom_sha256,
        "revision": [manifest.extractor_revision, manifest.decoder_revision],
        "bank": bank,
        "slot": 1,
        "poisoned_metadata_bytes": sources.iter().map(|span| span.length).sum::<u32>(),
        "typed_metadata_bytes": ordinary_reads,
        "altered_lz77_bytes": stream_end - stream_start,
        "bound_lz77_calls": bound_calls,
        "bound_lz77_input_bytes": bound_bytes,
        "replay_bound_lz77_bus_bytes": after.bound_lz77_bus_bytes - before.bound_lz77_bus_bytes,
        "raw_swi_bytes": raw_stream.len(),
        "guest_first_frame": first,
        "guest_last_frame": raw.bus.frames,
        "guest_clock": raw.bus.clock,
        "audio_samples": samples,
        "end_state_sha256": digest(&endpoint),
        "equivalent": true,
    });
    (result, report)
}

#[cfg(feature = "rom-data-read-audit")]
fn write_lz77_result(result: &serde_json::Value, report: &TraceReport) {
    fs::write(
        input("FFTA_TILEMAP_SLOT1_RESULT"),
        serde_json::to_vec_pretty(result).unwrap(),
    )
    .unwrap();
    fs::write(
        input("FFTA_TILEMAP_SLOT1_TRACE"),
        serde_json::to_vec_pretty(report).unwrap(),
    )
    .unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "exact EUR ROM and retained E466, E467, E471, E474 and E479 evidence"]
fn european_task_345_lz77_file_and_replay() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    let work = input("FFTA_TILEMAP_LZ77_WORK");
    let root = tempfile::tempdir_in(&work).unwrap();
    let mut revision: Manifest = parse(
        &fs::read(input("FFTA_TILEMAP_LZ77_PREDECESSOR")).unwrap(),
        "E304 manifest",
    )
    .unwrap();
    (revision.extractor_revision, revision.decoder_revision) = (305, 304);
    let mut source = Extraction::from_rom_for_manifest(&rom, &revision).unwrap();
    let prepared = preparation::prepare(root.path(), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    let dataset = &prepared.directory;
    let manifest_path = dataset.join(MANIFEST_FILE);
    let manifest_bytes = fs::read(&manifest_path).unwrap();
    let manifest: Manifest = parse(&manifest_bytes, "E305 manifest").unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (305, 304)
    );
    assert_eq!(manifest.resources.len(), 152);
    let resource = manifest.resources.last().unwrap();
    assert_eq!(resource.id, TILEMAP_TRANSFER_LZ77_ID);
    assert_eq!(resource.sources.len(), 13);
    let path = dataset.join(TILEMAP_TRANSFER_LZ77_FILE);
    let mut encoded = fs::read(&path).unwrap();
    assert_eq!(encoded.len(), 3742);
    assert_eq!(digest(&encoded), resource.sha256);
    let mut packed = encoded.as_slice();
    for span in &resource.sources {
        let (bytes, rest) = packed.split_at(span.length as usize);
        let start = span.offset as usize;
        assert!(bytes == &rom[start..start + bytes.len()]);
        assert_eq!(span.role, "lz77-input");
        assert_eq!(span.sha256, digest(bytes));
        packed = rest;
    }
    assert!(packed.is_empty());
    let reused = preparation::prepare(root.path(), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(reused.action, "reused");
    assert_eq!(&reused.directory, dataset);
    assert!(fs::read(&manifest_path).unwrap() == manifest_bytes);
    drop(source);

    let mut prior = manifest.clone();
    assert_eq!(prior.resources.pop().unwrap().id, TILEMAP_TRANSFER_LZ77_ID);
    assert_eq!(
        prior.coverage.pop().as_deref(),
        Some(TILEMAP_TRANSFER_LZ77_ID)
    );
    (prior.extractor_revision, prior.decoder_revision) = (304, 303);
    let prior_bytes = toml::to_string_pretty(&prior).unwrap();
    assert!(
        prior_bytes.as_bytes() == fs::read(input("FFTA_TILEMAP_LZ77_PREDECESSOR")).unwrap(),
        "E304 manifest prefix changed"
    );
    let mut managed = Machine::new(rom.clone());
    fs::write(&manifest_path, prior_bytes).unwrap();
    attach_from_directory(&mut managed, dataset).unwrap();
    let first = &resource.sources[0];
    let legacy_input = managed
        .bus
        .bound_tilemap_lz77(0x0800_0000 + first.offset)
        .unwrap();
    assert!(legacy_input.bytes() == &encoded[..first.length as usize]);
    drop(legacy_input);
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    attach_from_directory(&mut managed, dataset).unwrap();

    // A matching file digest cannot authorize a changed base resource.
    let active = managed.bus.rom_resources().unwrap().clone();
    *encoded.last_mut().unwrap() ^= 1;
    fs::write(&path, &encoded).unwrap();
    let mut edited = manifest.clone();
    edited.resources.last_mut().unwrap().sha256 = digest(&encoded);
    fs::write(&manifest_path, toml::to_string_pretty(&edited).unwrap()).unwrap();
    assert_eq!(
        attach_from_directory(&mut managed, dataset).unwrap_err(),
        "Tilemap transfer LZ77 file differs from authenticated source",
    );
    assert!(std::sync::Arc::ptr_eq(
        &active,
        managed.bus.rom_resources().unwrap()
    ));

    let (mut result, report) = check_task_345_lz77_replay(&rom, dataset, &manifest, managed);
    result["published_lz77_bytes"] = serde_json::json!(encoded.len());
    result["lz77_source_ranges"] = serde_json::json!(resource.sources.len());
    result["lz77_file_sha256"] = serde_json::json!(resource.sha256);
    result["published_manifest_sha256"] = serde_json::json!(digest(&manifest_bytes));
    result["default_writer_publication_and_reuse"] = serde_json::json!(true);
    result["e304_attach"] = serde_json::json!(true);
    result["coherent_edit_rejected"] = serde_json::json!(true);
    root.close().unwrap();
    fs::write(work.join(MANIFEST_FILE), manifest_bytes).unwrap();
    write_lz77_result(&result, &report);
}

#[cfg(feature = "rom-data-read-audit")]
#[derive(serde::Deserialize)]
struct Task345Replay {
    slot: u32,
    snapshot: PathBuf,
    snapshot_sha256: String,
    steps: Vec<(u32, u16)>,
    first_guest_frame: u64,
    guest_frames: u64,
    action_frame: u64,
    action_task_field: u32,
    primary_selector_field: u32,
    first_pointer_frame: u64,
    first_stream_frame: u64,
    stream_bytes: u32,
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "exact EUR ROM, E310 manifest and two retained E470 replays"]
fn european_task_345_ordinary_existing_streams() {
    use gba_core::rom_data::{
        observed_lz,
        read_audit::{Service, TraceCapture, TraceFilter},
        tilemap_transfer_directory as model,
    };

    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    let work = input("FFTA_TILEMAP_EXISTING_STREAMS_WORK");
    let expected_manifest = fs::read(input("FFTA_TILEMAP_EXISTING_STREAMS_MANIFEST")).unwrap();
    let manifest: Manifest = parse(&expected_manifest, "E310 manifest").unwrap();
    let cases: [Task345Replay; 2] =
        serde_json::from_slice(&fs::read(input("FFTA_TILEMAP_EXISTING_STREAMS_REPLAYS")).unwrap())
            .unwrap();
    assert_eq!(cases.each_ref().map(|case| case.slot), [2, 4]);

    let root = tempfile::tempdir_in(&work).unwrap();
    let mut extraction = Extraction::from_rom_for_manifest(&rom, &manifest).unwrap();
    let prepared = preparation::prepare(root.path(), &mut extraction, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    let dataset = &prepared.directory;
    assert_eq!(
        fs::read(dataset.join(MANIFEST_FILE)).unwrap(),
        expected_manifest
    );
    let catalog = parse_tilemap_transfer_directory(
        &fs::read(dataset.join(TILEMAP_TRANSFER_DIRECTORY_FILE)).unwrap(),
    )
    .unwrap();
    let inputs = catalog.lz77_input_ranges(&rom).unwrap();
    let mut raw = Machine::new(rom.clone());
    let mut managed = Machine::new(rom.clone());
    attach_from_directory(&mut managed, dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let mut results = Vec::new();

    for case in cases {
        let slot = case.slot;
        let offsets: Vec<_> = catalog.slots[slot as usize]
            .row_offsets()
            .unwrap()
            .collect();
        let &[offset] = offsets.as_slice() else {
            panic!("Expected one transfer row for slot {slot}");
        };
        let row = catalog
            .rows
            .iter()
            .find(|row| row.source_offset == offset)
            .unwrap();
        let descriptor = catalog
            .descriptors
            .iter()
            .find(|descriptor| model::ROM_BASE + descriptor.source_offset == row.descriptor_pointer)
            .unwrap();
        let wrapper = descriptor.wrapper.source_offset;
        let stream = inputs
            .iter()
            .find(|range| range.start == wrapper + 4)
            .unwrap();
        assert_eq!(stream.end - stream.start, case.stream_bytes);
        // The first counter belongs to the base block. Additional specs follow it.
        let block = 1 + observed_lz::ADDITIONAL_SPECS
            .iter()
            .position(|spec| {
                spec.offset == stream.start && spec.encoded_length == case.stream_bytes as usize
            })
            .unwrap();
        let spans = [
            (
                model::POINTER_DIRECTORY + 4 * slot,
                4,
                SOIL_SCOPED_VALUES_ID,
            ),
            (model::COUNT_DIRECTORY + slot, 1, SOIL_SCOPED_VALUES_ID),
            (offset, model::ROW_BYTES, OBSERVED_TILEMAP_ROW_ID),
            (descriptor.source_offset, 8, OBSERVED_TILEMAP_DESCRIPTOR_ID),
            (wrapper, 4, OBSERVED_TILEMAP_DESCRIPTOR_ID),
            (stream.start, case.stream_bytes, OBSERVED_LZ_ID),
        ];
        let sources = spans.map(|(offset, length, id)| {
            manifest
                .resources
                .iter()
                .find(|resource| resource.id == id)
                .unwrap()
                .sources
                .iter()
                .find(|source| source.offset == offset && source.length == length)
                .unwrap()
                .clone()
        });
        let action_reads = [
            (0x0a_a158, 4, 0x080a_a140),
            (case.action_task_field, 2, 0x080d_08a6),
            (case.primary_selector_field, 2, 0x080d_70a2),
        ];
        let ranges = spans
            .iter()
            .map(|&(start, length, _)| (start, start + length))
            .chain(
                action_reads
                    .iter()
                    .map(|&(start, length, _)| (start, start + length)),
            )
            .collect();
        let snapshot = fs::read(case.snapshot).unwrap();
        assert_eq!(digest(&snapshot), case.snapshot_sha256);
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&snapshot).unwrap();
            assert_eq!(machine.bus.frames, case.first_guest_frame);
        }
        set_backing(&mut managed, &rom, &sources, true);
        let capture = TraceCapture::start(
            TraceFilter {
                readers: Vec::new(),
                ranges,
            },
            4096,
        )
        .unwrap();
        let hle_calls = || owner.observed_lz_stats().unwrap().hle_calls_by_block[block];
        let mut calls = hle_calls();
        let mut hle_frames = Vec::new();
        let mut previous_managed_frame = None;
        let mut trace_frame = |frame| {
            if let Some(previous) = previous_managed_frame {
                let after = hle_calls();
                if after != calls {
                    assert_eq!(after - calls, 1);
                    hle_frames.push(previous);
                    calls = after;
                }
            }
            previous_managed_frame = frame;
            capture.frame(frame);
        };
        let samples = replay_pair(&mut raw, &mut managed, &case.steps, &mut trace_frame);
        trace_frame(None);
        let report = capture.finish();
        assert_eq!(hle_frames, [case.first_stream_frame], "slot {slot}");
        assert_eq!(raw.bus.frames - case.first_guest_frame, case.guest_frames);
        assert!(samples > 0 && report.dropped_requests == 0 && !report.truncated);

        let mut served: [BTreeSet<u32>; 6] = std::array::from_fn(|_| BTreeSet::new());
        for event in &report.events {
            let read = event.key.read;
            let start = read.physical_offset;
            let end = start + read.length as u32;
            for (&(offset, length, _), bytes) in spans.iter().zip(&mut served) {
                let lower = start.max(offset);
                let upper = end.min(offset + length);
                if lower < upper {
                    assert_eq!(event.key.service, Service::Typed, "slot {slot}: {event:?}");
                    bytes.extend(lower..upper);
                }
            }
        }
        let metadata_bytes = served[..5].iter().map(BTreeSet::len).collect::<Vec<_>>();
        assert_eq!(metadata_bytes, [4, 1, 12, 5, 2], "slot {slot}");
        let observed_load = |offset, length, pc, frame, service: Option<Service>| {
            report.events.iter().any(|event| {
                let read = event.key.read;
                read.physical_offset == offset
                    && read.length == length
                    && event
                        .key
                        .instruction
                        .is_some_and(|site| site.thumb && site.pc == pc)
                    && event.first_guest_frame == frame
                    && service.is_none_or(|s| event.key.service == s)
            })
        };
        for (offset, length, pc) in action_reads {
            assert!(observed_load(
                offset,
                length as usize,
                pc,
                case.action_frame,
                None
            ));
        }
        for (&(offset, length, _), pc) in spans[..2].iter().zip([0x080e_61cc, 0x080e_61d6]) {
            assert!(observed_load(
                offset,
                length as usize,
                pc,
                case.first_pointer_frame,
                Some(Service::Typed)
            ));
        }
        set_backing(&mut managed, &rom, &sources, false);
        assert!(raw.bus.framebuffer == managed.bus.framebuffer && raw.bus.oam == managed.bus.oam);
        let endpoint = raw.save_state().unwrap();
        assert!(
            endpoint == managed.save_state().unwrap(),
            "slot {slot} endpoint differs"
        );
        fs::write(
            work.join(format!("slot-{slot}-trace.json")),
            serde_json::to_vec(&report).unwrap(),
        )
        .unwrap();
        results.push(serde_json::json!({
            "slot": slot, "guest_frames": case.guest_frames, "guest_first_frame": case.first_guest_frame,
            "first_pointer_frame": case.first_pointer_frame, "stream_hle_frames": hle_frames,
            "metadata_bytes": metadata_bytes, "altered_bytes": spans.iter().map(|span| span.1).sum::<u32>(),
            "stream_bytes": case.stream_bytes, "stream_bus_bytes": served[5].len(),
            "stream_owner": OBSERVED_LZ_ID, "audio_samples": samples, "end_state_sha256": digest(&endpoint),
        }));
    }
    root.close().unwrap();
    fs::write(
        work.join("result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "status": "PASS", "manifest_sha256": digest(&expected_manifest),
            "source_rom_sha256": manifest.source_rom_sha256, "replays": results,
        }))
        .unwrap(),
    )
    .unwrap();
}
