//! Validate the complete E481 transfer projection with the EUR ROM under Wine.
use super::{
    attach_dataset, attach_from_directory,
    bank_ownership::Plan,
    direct_byte_copy_tests::{capture, flip, same_state},
    indexed_reader_bank_tests::{execute_until, initialize},
    parse, preparation, read_dataset,
    tilemap_transfer_directory_tests::{check_stream_bus_reads, replay_pair},
    Extraction, Manifest, MANIFEST_FILE, TILEMAP_TYPE3_FILE, TILEMAP_TYPE3_ID,
};
use gba_core::{
    ffta_rom::digest,
    rom_data::{
        read_audit::{Service, TraceCapture, TraceFilter},
        SourceRange,
    },
    Bus, Machine,
};
use serde::Deserialize;
use serde_json::Value;
use std::{fs, ops::Range, path::PathBuf, sync::Arc};

const MIRRORS: [u32; 3] = [0x0800_0000, 0x0a00_0000, 0x0c00_0000];
const ALIAS: Range<usize> = 0x3abd78..0x3abd7c;

#[derive(Deserialize)]
struct RecoveredTransferPlan {
    list_roots: Vec<RecoveredList>,
    required_spans: Vec<RecoveredSpan>,
    new_rows: Vec<RecoveredRow>,
}

#[derive(Deserialize)]
struct RecoveredList {
    reader: u32,
    base: u32,
    count: u32,
}

#[derive(Deserialize)]
struct RecoveredSpan {
    offset: u32,
    length: u32,
    role: String,
}

#[derive(Deserialize)]
struct RecoveredRow {
    row: u32,
    descriptor: u32,
    wrapper: u32,
    caller: u32,
    stream: RecoveredStream,
}

#[derive(Deserialize)]
struct RecoveredStream {
    encoded_length: u64,
    decoded_length: usize,
    decoded_sha256: String,
}

#[test]
#[ignore = "requires the exact EUR ROM, E597 source plan and e352/e353 generations"]
fn european_counted_type3_lists_feed_original_loaders() {
    const REVISION: (u32, u32) = (353, 352);
    const BASE: u32 = 0x0800_0000;
    const CONTEXT: u32 = 0x0200_0100;
    const MESSAGE: u32 = 0x0200_0200;
    const DIRECT_DESTINATION: u32 = 0x0200_2000;
    const STAGED_DESTINATION: u32 = 0x0200_4000;

    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let plan = fs::read(input("FFTA_COUNTED_TYPE3_PLAN")).unwrap();
    assert_eq!(
        digest(&plan),
        "3bb9b890fa4cc1c1c7e00da55e0813df7ab35d4bb99bdf4c423544d8a37b9994"
    );
    let plan: RecoveredTransferPlan = serde_json::from_slice(&plan).unwrap();
    assert_eq!(
        (
            plan.list_roots.len(),
            plan.new_rows.len(),
            plan.required_spans.len()
        ),
        (6, 3, 11)
    );

    let previous_dir = input("FFTA_STATE_TEST_DATASET");
    assert_eq!(
        digest(&fs::read(previous_dir.join("manifest.toml")).unwrap()),
        "3698b35187950c4b3df75a5d122e62db7f5c60741e7b2c9e193bc6348a0b5aca"
    );
    let mut previous = read_dataset(&previous_dir).unwrap();
    assert_eq!(
        (
            previous.manifest.extractor_revision,
            previous.manifest.decoder_revision
        ),
        (352, 351)
    );
    let current = read_dataset(&input("FFTA_COUNTED_TYPE3_DATASET")).unwrap();
    assert_eq!(
        (
            current.manifest.extractor_revision,
            current.manifest.decoder_revision
        ),
        REVISION
    );
    let mut managed = Machine::new(rom.clone());
    attach_dataset(&mut managed, &previous).unwrap();
    let prior = managed.bus.rom_resources().unwrap().clone();
    let new_stream = 0xb34148;
    flip(&mut managed, new_stream, 1);
    assert_eq!(
        managed.bus.read8(BASE + new_stream),
        !rom[new_stream as usize]
    );
    flip(&mut managed, new_stream, 1);
    (
        previous.manifest.extractor_revision,
        previous.manifest.decoder_revision,
    ) = REVISION;
    assert!(attach_dataset(&mut managed, &previous).is_err());
    assert!(Arc::ptr_eq(&prior, managed.bus.rom_resources().unwrap()));
    attach_dataset(&mut managed, &current).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let mut raw = Machine::new(rom);
    let initial = raw.save_state().unwrap();
    let source_bytes: u32 = plan.required_spans.iter().map(|span| span.length).sum();
    assert_eq!(source_bytes, 322);

    let typed_ranges: Vec<_> = plan
        .required_spans
        .iter()
        .filter(|span| span.role != "lz77-input")
        .map(|span| (span.offset, span.offset + span.length))
        .collect();
    let mut list_setups = 0;
    let mut typed_directory_loads = 0;
    for list in &plan.list_roots {
        let variant = list.reader >= 0x0fdefc;
        let entry = BASE + if variant { 0x0fdecc } else { list.reader };
        let stop = BASE + if variant { 0x0fdf14 } else { 0x0e6168 };
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap();
            initialize(machine, entry);
            if variant {
                machine.cpu.regs[4] = CONTEXT;
                machine.cpu.regs[5] = MESSAGE;
                machine
                    .bus
                    .write16(MESSAGE + 0x14, if list.count == 6 { 0x13d } else { 0 });
            } else {
                machine.cpu.regs[5] = CONTEXT;
            }
        }
        for span in &plan.required_spans {
            flip(&mut managed, span.offset, span.length);
        }
        execute_until(&mut raw, stop, 64);
        let trace = capture(&managed, typed_ranges.clone(), 8);
        execute_until(&mut managed, stop, 64);
        let report = trace.finish();
        assert_eq!(report.dropped_requests, 0);
        for event in &report.events {
            assert_eq!(event.key.service, Service::Typed);
            assert_eq!(event.key.instruction.unwrap().pc, BASE + list.reader);
        }
        typed_directory_loads += report.matched_requests;
        for span in &plan.required_spans {
            flip(&mut managed, span.offset, span.length);
        }
        assert_eq!(&managed.cpu.regs[1..3], &[BASE + list.base, list.count]);
        same_state(&raw, &managed, entry);
        list_setups += 1;
    }
    assert_eq!(typed_directory_loads, 3);

    for machine in [&mut raw, &mut managed] {
        machine.load_state(&initial).unwrap();
        initialize(machine, BASE + 0x0fdecc);
        machine.cpu.regs[4] = CONTEXT;
        machine.cpu.regs[5] = MESSAGE;
        machine.bus.write16(MESSAGE + 0x14, 0x13d);
        machine.bus.write16(MESSAGE + 0x16, 1);
        execute_until(machine, BASE + 0x0fdf14, 64);
        assert_eq!(&machine.cpu.regs[1..3], &[BASE + 0x3a16a0, 0]);
    }
    same_state(&raw, &managed, BASE + 0x0fdecc);

    let mut decoded_output_bytes = 0;
    for row in &plan.new_rows {
        let list = plan
            .list_roots
            .iter()
            .find(|list| row.row >= list.base && row.row < list.base + 12 * list.count)
            .unwrap();
        let index = (row.row - list.base) / 12;
        assert_eq!(list.base + 12 * index, row.row);
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap();
            initialize(machine, BASE + 0x0db09c);
            machine.cpu.regs[0] = CONTEXT;
            machine.bus.write32(CONTEXT, BASE + list.base);
            machine.bus.write8(CONTEXT + 7, index as u8);
            machine.bus.write32(CONTEXT + 0x20, STAGED_DESTINATION);
            machine.bus.write32(CONTEXT + 0x24, DIRECT_DESTINATION);
        }
        for span in &plan.required_spans {
            flip(&mut managed, span.offset, span.length);
        }
        execute_until(&mut raw, BASE + 0x0db17c, 128);
        let before = owner.tilemap_type3_stats().unwrap();
        let trace = capture(&managed, typed_ranges.clone(), 32);
        execute_until(&mut managed, BASE + 0x0db17c, 128);
        let report = trace.finish();
        assert_eq!(report.dropped_requests, 0);
        assert!(report
            .events
            .iter()
            .all(|event| event.key.service == Service::Typed));
        for offset in [row.row, row.descriptor] {
            assert!(report
                .events
                .iter()
                .any(|event| event.key.read.physical_offset == offset));
        }
        if row.row == 0x3a16dc {
            assert!(report
                .events
                .iter()
                .any(|event| event.key.read.physical_offset == row.wrapper));
        }
        let after = owner.tilemap_type3_stats().unwrap();
        assert_eq!(after.lz77_calls - before.lz77_calls, 1);
        assert_eq!(
            after.lz77_input_bytes - before.lz77_input_bytes,
            row.stream.encoded_length
        );
        let destination = match row.caller {
            0x0db160 => STAGED_DESTINATION,
            0x0db16e => DIRECT_DESTINATION,
            caller => panic!("Unexpected type-3 producer {caller:#x}"),
        };
        let first = (destination - 0x0200_0000) as usize;
        assert_eq!(
            digest(&managed.bus.ewram[first..first + row.stream.decoded_length]),
            row.stream.decoded_sha256
        );
        decoded_output_bytes += row.stream.decoded_length;
        for span in &plan.required_spans {
            flip(&mut managed, span.offset, span.length);
        }
        same_state(&raw, &managed, BASE + 0x0db09c);
    }

    for span in &plan.required_spans {
        flip(&mut managed, span.offset, span.length);
    }
    let mut mirror_checks = 0;
    for mirror in MIRRORS {
        for span in &plan.required_spans {
            for offset in [span.offset - 1, span.offset + span.length - 1] {
                let address = mirror + offset;
                assert_eq!(managed.bus.read16(address), raw.bus.read16(address));
                assert_eq!(managed.bus.read32(address), raw.bus.read32(address));
                mirror_checks += 2;
            }
        }
    }
    fs::write(
        input("FFTA_COUNTED_TYPE3_WORK").join("test-result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "status": "PASS", "revision": REVISION, "source_bytes": source_bytes,
            "list_setups": list_setups, "zero_count_cases": 1, "typed_directory_loads": typed_directory_loads,
            "original_swi_cases": plan.new_rows.len(), "decoded_output_bytes": decoded_output_bytes,
            "mixed_mirror_checks": mirror_checks, "full_state_and_clock_equal": true,
            "predecessor_attach": true, "missing_spans_rejected": true,
        })).unwrap(),
    ).unwrap();
}

fn input(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os(name).unwrap_or_else(|| panic!("Missing {name}")))
}

fn read_report(name: &str) -> Value {
    serde_json::from_slice(&fs::read(input(name)).unwrap()).unwrap()
}

fn verify_sources(rom: &[u8], file: &[u8], sources: &[SourceRange], manifest: &Manifest) {
    let plan: Plan = serde_json::from_slice(&fs::read(input("FFTA_TYPE3_PLAN")).unwrap()).unwrap();
    assert_eq!(plan.profile, manifest.profile);
    assert_eq!(plan.source_rom_sha256, manifest.source_rom_sha256);
    assert_eq!(plan.source_rom_size, manifest.source_rom_size);
    let mut cursor = file;
    for source in sources {
        let start = source.offset as usize;
        let (encoded, rest) = cursor.split_at(source.length as usize);
        assert!(
            encoded == &rom[start..start + encoded.len()],
            "File differs at {start:#x}"
        );
        assert_eq!(digest(encoded), source.sha256);
        cursor = rest;
    }
    assert!(cursor.is_empty());
    for (kind, role) in [
        ("rows", "transfer-row"),
        ("descriptors", "transfer-descriptor"),
        ("wrappers", "stream-wrapper"),
        ("lz77-inputs", "lz77-input"),
        ("raw-inputs", "raw-input"),
    ] {
        let mut actual: Vec<(u32, u32)> = Vec::new();
        for span in sources.iter().filter(|source| source.role == role) {
            let end = span.offset + span.length;
            if let Some(last) = actual.last_mut().filter(|last| last.1 == span.offset) {
                last.1 = end;
            } else {
                actual.push((span.offset, end));
            }
        }
        let expected = plan.banks.iter().find(|bank| bank.id == kind).unwrap();
        assert!(
            actual.into_iter().eq(expected
                .ranges
                .iter()
                .map(|span| { (span.offset, span.offset + span.length) })),
            "E481 range mismatch for {kind}",
        );
    }
}

fn write_backing(machine: &mut Machine, rom: &[u8], sources: &[SourceRange], invert: bool) {
    for span in sources {
        let first = span.offset as usize;
        for at in first..first + span.length as usize {
            machine.bus.rom[at] = if invert { !rom[at] } else { rom[at] };
        }
    }
}

fn check_controls_and_inputs(
    raw: &mut Machine,
    managed: &mut Machine,
    spans: &[SourceRange],
) -> Value {
    let owner = managed.bus.rom_resources().unwrap().clone();
    let before = owner.tilemap_type3_stats().unwrap();
    check_stream_bus_reads(raw, managed, 0x3abd74, ALIAS.end as u32);
    let mut swis = 0;
    let mut raw_copies = 0;
    for source in spans {
        let start = source.offset;
        let end = start + source.length;
        if source.role == "lz77-input" {
            check_stream_bus_reads(raw, managed, start, start + 1);
            check_stream_bus_reads(raw, managed, end - 1, end);
            let address = MIRRORS[swis % MIRRORS.len()] + start;
            let (swi, dst) = if swis % 2 == 0 {
                (0x11, 0x0200_0020)
            } else {
                (0x12, 0x0600_0020)
            };
            let prior = owner.tilemap_type3_stats().unwrap();
            for machine in [&mut *raw, &mut *managed] {
                machine.bus.ewram.fill(0x63);
                machine.bus.vram.fill(0x63);
                machine.cpu.regs[0] = address;
                machine.cpu.regs[1] = dst;
                assert!(gba_core::hle::bios_call(
                    &mut machine.cpu,
                    &mut machine.bus,
                    swi
                ));
            }
            assert!(
                raw.bus.ewram == managed.bus.ewram && raw.bus.vram == managed.bus.vram,
                "SWI output differs for {start:#x}",
            );
            assert_eq!(raw.bus.clock, managed.bus.clock);
            assert_eq!(raw.cpu.regs, managed.cpu.regs);
            let after = owner.tilemap_type3_stats().unwrap();
            assert_eq!(after.lz77_calls - prior.lz77_calls, 1);
            assert_eq!(
                after.lz77_input_bytes - prior.lz77_input_bytes,
                u64::from(source.length)
            );
            swis += 1;
        } else {
            check_stream_bus_reads(raw, managed, start, end);
        }
        if source.role == "raw-input" {
            let count = u16::try_from(source.length / 2).unwrap();
            let address = MIRRORS[raw_copies % MIRRORS.len()] + start;
            let before_dma = owner.tilemap_type3_stats().unwrap().bus_bytes;
            for machine in [&mut *raw, &mut *managed] {
                machine.bus.ewram[0x100..0x100 + source.length as usize].fill(0);
                machine.bus.write32(0x0400_00d4, address);
                machine.bus.write32(0x0400_00d8, 0x0200_0100);
                machine.bus.write16(0x0400_00dc, count);
                machine.bus.write16(0x0400_00de, 0x8000);
            }
            assert!(
                raw.bus.ewram == managed.bus.ewram,
                "DMA output differs for {start:#x}"
            );
            assert!(
                managed.bus.ewram[0x100..0x100 + source.length as usize]
                    == raw.bus.rom[start as usize..end as usize]
            );
            assert_eq!(raw.bus.clock, managed.bus.clock);
            assert_eq!(raw.bus.read16(0x0400_00de), managed.bus.read16(0x0400_00de));
            let after_dma = owner.tilemap_type3_stats().unwrap().bus_bytes;
            assert_eq!(after_dma - before_dma, u64::from(source.length));
            raw_copies += 1;
        }
    }
    assert_eq!((swis, raw_copies), (323, 14));
    let after = owner.tilemap_type3_stats().unwrap();
    assert_eq!(after.lz77_input_bytes - before.lz77_input_bytes, 304_054);
    serde_json::json!({
        "swi_checks": swis,
        "raw_dma_checks": raw_copies,
        "bus_bytes": after.bus_bytes - before.bus_bytes,
        "lz77_input_bytes": after.lz77_input_bytes - before.lz77_input_bytes,
        "preserved_partial_descriptor": true,
    })
}

fn check_shell(
    rom: &[u8],
    raw: &mut Machine,
    managed: &mut Machine,
    spans: &[SourceRange],
) -> Value {
    let evidence = read_report("FFTA_TYPE3_E477");
    let expected = &evidence["cases"]["shell"];
    let snapshot = fs::read(input("FFTA_TYPE3_SHELL")).unwrap();
    assert_eq!(expected["snapshot_sha256"], digest(&snapshot));
    for machine in [&mut *raw, &mut *managed] {
        machine.load_state(&snapshot).unwrap();
        assert_eq!(expected["first_frame"], machine.bus.frames);
    }
    let owner = managed.bus.rom_resources().unwrap().clone();
    let before = owner.tilemap_type3_stats().unwrap();
    write_backing(managed, rom, spans, true);
    let rows = [(0x39bc3c, 0x39bc48), (0x39bc48, 0x39bc54)];
    let capture = TraceCapture::start(
        TraceFilter {
            readers: Vec::new(),
            ranges: rows.to_vec(),
        },
        4096,
    )
    .unwrap();
    let steps = [(8, 0x3fe), (24, 0x3ff), (8, 0x3fe), (385, 0x3ff)];
    let samples = replay_pair(raw, managed, &steps, |frame| capture.frame(frame));
    let reads = capture.finish();
    assert!(!reads.truncated && reads.dropped_requests == 0);
    for (start, end) in rows {
        assert!(reads.events.iter().any(|event| {
            let read = event.key.read;
            (start..end).contains(&read.physical_offset) && event.key.service == Service::Typed
        }));
    }
    assert!(reads
        .events
        .iter()
        .all(|event| event.key.service == Service::Typed));
    write_backing(managed, rom, spans, false);
    assert_eq!(expected["last_frame"], raw.bus.frames);
    assert_eq!(expected["audio_samples"], samples);
    assert!(raw.bus.framebuffer == managed.bus.framebuffer && raw.bus.oam == managed.bus.oam);
    let state = raw.save_state().unwrap();
    assert!(state == managed.save_state().unwrap());
    let state_sha256 = digest(&state);
    assert_eq!(expected["endpoint_sha256"], state_sha256);
    let after = owner.tilemap_type3_stats().unwrap();
    assert!(after.bus_bytes > before.bus_bytes);
    let calls = after.lz77_calls - before.lz77_calls;
    let bytes = after.lz77_input_bytes - before.lz77_input_bytes;
    assert!(calls >= 2 && bytes >= 255);
    serde_json::json!({
        "first_frame": expected["first_frame"],
        "last_frame": raw.bus.frames,
        "audio_samples": samples,
        "bus_bytes": after.bus_bytes - before.bus_bytes,
        "lz77_calls": calls,
        "lz77_input_bytes": bytes,
        "state_sha256": state_sha256,
        "typed_row_reads": true,
    })
}

#[test]
#[ignore = "exact EUR ROM, verified E305 manifest, E481 residual and E477 Shell witness"]
fn european_type3_bank_files_and_original_readers() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    let work = input("FFTA_TYPE3_WORK");
    let root = tempfile::tempdir_in(&work).unwrap();
    let prior_bytes = fs::read(input("FFTA_TYPE3_PREDECESSOR")).unwrap();
    let prior: Manifest = parse(&prior_bytes, "E305 manifest").unwrap();
    let mut revision = prior.clone();
    (revision.extractor_revision, revision.decoder_revision) = (306, 305);
    let mut source = Extraction::from_rom_for_manifest(&rom, &revision).unwrap();
    let prepared = preparation::prepare(root.path(), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    let dataset = &prepared.directory;
    let manifest_path = dataset.join(MANIFEST_FILE);
    let manifest_bytes = fs::read(&manifest_path).unwrap();
    let manifest: Manifest = parse(&manifest_bytes, "E306 manifest").unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (306, 305)
    );
    assert_eq!(manifest.resources.len(), 153);
    let resource = manifest.resources.last().unwrap();
    assert_eq!(resource.id, TILEMAP_TYPE3_ID);
    let path = dataset.join(TILEMAP_TYPE3_FILE);
    let mut file = fs::read(&path).unwrap();
    assert_eq!(file.len(), 325_850);
    assert_eq!(digest(&file), resource.sha256);
    verify_sources(&rom, &file, &resource.sources, &manifest);
    let reused = preparation::prepare(root.path(), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(reused.action, "reused");
    assert_eq!(&reused.directory, dataset);
    drop(source);

    assert!((305, 304) == (prior.extractor_revision, prior.decoder_revision));
    assert!(prior.resources == manifest.resources[..manifest.resources.len() - 1]);
    assert!(prior.coverage == manifest.coverage[..manifest.coverage.len() - 1]);
    let mut managed = Machine::new(rom.clone());
    fs::write(&manifest_path, prior_bytes).unwrap();
    attach_from_directory(&mut managed, dataset).unwrap();
    assert!(managed
        .bus
        .rom_resources()
        .unwrap()
        .tilemap_type3_stats()
        .is_none());
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    attach_from_directory(&mut managed, dataset).unwrap();

    let active = managed.bus.rom_resources().unwrap().clone();
    *file.last_mut().unwrap() ^= 1;
    fs::write(&path, &file).unwrap();
    let mut edited = manifest.clone();
    edited.resources.last_mut().unwrap().sha256 = digest(&file);
    fs::write(&manifest_path, toml::to_string_pretty(&edited).unwrap()).unwrap();
    assert_eq!(
        attach_from_directory(&mut managed, dataset).unwrap_err(),
        "Type-3 tilemap transfer file differs from authenticated source",
    );
    assert!(std::sync::Arc::ptr_eq(
        &active,
        managed.bus.rom_resources().unwrap()
    ));

    let mut raw = Machine::new(rom.clone());
    write_backing(&mut managed, &rom, &resource.sources, true);
    for at in ALIAS {
        managed.bus.rom[at] = !rom[at];
    }
    let bank = check_controls_and_inputs(&mut raw, &mut managed, &resource.sources);
    managed.bus.rom[ALIAS].copy_from_slice(&rom[ALIAS]);
    write_backing(&mut managed, &rom, &resource.sources, false);
    let shell = check_shell(&rom, &mut raw, &mut managed, &resource.sources);
    let result = serde_json::json!({
        "status": "PASS",
        "revision": [306, 305],
        "published_bytes": file.len(),
        "new_dataset_sha256": active.dataset_sha256(),
        "published_manifest_sha256": digest(&manifest_bytes),
        "source_file_sha256": resource.sha256,
        "bank": bank,
        "shell": shell,
        "default_writer_publication_and_reuse": true,
        "e305_attach": true,
        "coherent_edit_rejected": true,
    });
    root.close().unwrap();
    fs::write(work.join(MANIFEST_FILE), manifest_bytes).unwrap();
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
