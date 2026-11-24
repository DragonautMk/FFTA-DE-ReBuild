//! Validate the complete E481 transfer projection with the EUR ROM under Wine.
use super::{
    attach_from_directory,
    bank_ownership::Plan,
    parse, preparation,
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
use serde_json::Value;
use std::{fs, ops::Range, path::PathBuf};

const MIRRORS: [u32; 3] = [0x0800_0000, 0x0a00_0000, 0x0c00_0000];
const ALIAS: Range<usize> = 0x3abd78..0x3abd7c;

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
