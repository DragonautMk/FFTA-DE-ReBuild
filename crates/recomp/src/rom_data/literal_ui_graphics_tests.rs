//! Verify the complete E500 literal source projection through original guest reads.

use super::{
    attach_from_directory, cmd, encode_ui_graphics, parse, parse_ui_graphics, Extraction, Manifest,
    MANIFEST_FILE, UI_CUSTOM_GRAPHICS_FILE, UI_CUSTOM_GRAPHICS_ID,
};
use gba_core::{
    ffta_rom::digest,
    rom_data::{
        read_audit::{Service, TraceCapture, TraceFilter, TraceReport},
        unit_obj_ffff_reader_fields::PROPERTY_OFFSET,
        UNIT_OBJ_FFFF_READER_FIELDS_ID,
    },
    Bus, Machine,
};
use serde::Deserialize;
use std::{collections::BTreeSet, fs, path::PathBuf, sync::Arc};

const BASE: u32 = 0x08000000;
const DECODER: u32 = BASE + 0x54a0;
const SCRATCH: u32 = super::ui_graphics_tests::SCRATCH;

#[derive(Deserialize)]
struct Plan {
    references: Vec<Reference>,
    streams: Vec<Stream>,
    source_ranges: Vec<Source>,
}

#[derive(Deserialize)]
struct Reference {
    site: u32,
    reader: u32,
    literal: u32,
    source: u32,
    new: Vec<Source>,
}

#[derive(Deserialize)]
struct Stream {
    source: u32,
    end: u32,
    decoded_bytes: usize,
    decoded_sha256: String,
    new: Vec<Source>,
}

#[derive(Deserialize)]
struct Source {
    offset: u32,
    length: u32,
    sha256: String,
}

fn input(name: &str) -> PathBuf {
    std::env::var_os(name).unwrap().into()
}

fn run_reader(
    machine: &mut Machine,
    reference: &Reference,
    stream: Option<&Stream>,
) -> TraceReport {
    machine.bus.ffta_sprites.enabled = false;
    machine.bus.ewram.fill(0x5a);
    machine.bus.iwram.fill(0x3c);
    machine.cpu.regs.fill(SCRATCH);
    machine.cpu.regs[13] = 0x03007c00;
    machine.cpu.regs[15] = BASE + reference.reader;
    machine.cpu.cpsr = 0xbf;
    machine.cpu.branch = None;

    let mut ranges = vec![(reference.literal, reference.literal + 4)];
    if let Some(stream) = stream {
        ranges.push((stream.source, stream.end));
    }
    let capture = TraceCapture::start(
        TraceFilter {
            readers: Vec::new(),
            ranges,
        },
        32_768,
    )
    .unwrap();
    capture.frame(Some(0));
    machine.step();
    if reference.site == 0x954fe {
        assert_eq!(machine.cpu.regs[7], BASE + reference.source);
        // Resume at the final move with scratch allocated by the test.
        machine.cpu.regs[15] = BASE + reference.site - 2;
    }
    for _ in 0..4 {
        if machine.cpu.regs[15] == DECODER {
            break;
        }
        machine.step();
    }
    assert_eq!(machine.cpu.regs[15], DECODER);
    assert_eq!(&machine.cpu.regs[..2], &[SCRATCH, BASE + reference.source]);

    if let Some(stream) = stream {
        let return_pc = BASE + reference.site + 4;
        let mut cursor_checked = false;
        for _ in 0..500_000 {
            let pc = machine.cpu.regs[15];
            if pc == return_pc {
                break;
            }
            if pc == 0x080058de {
                assert_eq!(machine.cpu.regs[7], BASE + stream.end);
                cursor_checked = true;
            }
            machine.step();
        }
        assert!(cursor_checked);
        assert_eq!(machine.cpu.regs[15], return_pc);
        assert_eq!(machine.cpu.regs[0] as usize, stream.decoded_bytes);
        let start = (SCRATCH - 0x02000000) as usize;
        let end = start + stream.decoded_bytes;
        assert_eq!(
            digest(&machine.bus.ewram[start..end]),
            stream.decoded_sha256
        );
        assert_eq!(machine.bus.ewram[start - 1], 0x5a);
        assert_eq!(machine.bus.ewram[end], 0x5a);
    }
    let report = capture.finish();
    assert!(!report.truncated);
    report
}

#[test]
#[ignore = "exact EUR ROM, E500 source plan and E499 predecessor manifest"]
fn european_literal_graphics_publication_and_original_reads() {
    let rom_path = input("FFTA_STATE_TEST_ROM");
    let rom = fs::read(&rom_path).unwrap();
    let work = input("FFTA_LITERAL_UI_WORK");
    let plan: Plan =
        serde_json::from_slice(&fs::read(input("FFTA_LITERAL_UI_PLAN")).unwrap()).unwrap();
    let previous_bytes = fs::read(input("FFTA_LITERAL_UI_PREDECESSOR")).unwrap();
    let previous: Manifest = parse(&previous_bytes, "E499 manifest").unwrap();
    assert_eq!(
        (previous.extractor_revision, previous.decoder_revision),
        (312, 311)
    );

    let resources = tempfile::tempdir_in(&work).unwrap();
    let rom_arg = rom_path.to_str().unwrap().to_owned();
    let prepare = [
        "prepare".into(),
        rom_arg.clone(),
        resources.path().to_str().unwrap().into(),
    ];
    cmd(&prepare).unwrap();
    let revision = resources
        .path()
        .join(&previous.source_rom_sha256)
        .join("v1/e313-d312");
    let current = fs::read_to_string(revision.join("current")).unwrap();
    let dataset = revision.join(current.trim());
    cmd(&prepare).unwrap();
    assert_eq!(
        fs::read_to_string(revision.join("current")).unwrap(),
        current
    );
    cmd(&["verify".into(), rom_arg, dataset.to_str().unwrap().into()]).unwrap();

    let manifest_path = dataset.join(MANIFEST_FILE);
    let manifest_bytes = fs::read(&manifest_path).unwrap();
    let manifest: Manifest = parse(&manifest_bytes, "E501 manifest").unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (313, 312)
    );
    assert_eq!(manifest.coverage, previous.coverage);
    assert_eq!(manifest.resources.len(), previous.resources.len());
    let index = manifest
        .resources
        .iter()
        .position(|entry| entry.id == UI_CUSTOM_GRAPHICS_ID)
        .unwrap();
    for (row, (new, old)) in manifest
        .resources
        .iter()
        .zip(&previous.resources)
        .enumerate()
    {
        if row != index {
            assert_eq!(new, old);
        }
    }
    let old = &previous.resources[index];
    let new = &manifest.resources[index];
    assert_eq!((&new.path, &new.format), (&old.path, &old.format));
    let mut dependencies = old.dependencies.clone();
    dependencies.push(UNIT_OBJ_FFFF_READER_FIELDS_ID.into());
    assert_eq!(new.dependencies, dependencies);
    assert!(old
        .sources
        .iter()
        .all(|source| new.sources.contains(source)));
    let added: Vec<_> = new
        .sources
        .iter()
        .filter(|entry| !old.sources.contains(entry))
        .collect();
    assert_eq!(added.len(), 33);
    assert_eq!(added.len(), plan.source_ranges.len());
    assert_eq!(added.iter().map(|entry| entry.length).sum::<u32>(), 47_843);
    for range in &plan.source_ranges {
        let source = added
            .iter()
            .find(|source| source.offset == range.offset)
            .unwrap();
        assert_eq!(
            (source.length, &source.sha256),
            (range.length, &range.sha256)
        );
    }

    let ui_path = dataset.join(UI_CUSTOM_GRAPHICS_FILE);
    let ui_bytes = fs::read(&ui_path).unwrap();
    let mut legacy = Extraction::from_rom_for_manifest(&rom, &previous).unwrap();
    let prior_ui = encode_ui_graphics(&legacy.ui_graphics.take().unwrap().decoded).unwrap();
    assert_eq!(digest(&prior_ui), old.sha256);
    let mut managed = Machine::new(rom.clone());
    fs::write(&manifest_path, previous_bytes).unwrap();
    fs::write(&ui_path, prior_ui).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    assert_eq!(
        managed
            .bus
            .rom_resources()
            .unwrap()
            .ui_graphics_file_sha256(),
        Some(old.sha256.as_str())
    );
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    fs::write(&ui_path, &ui_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();

    let mut edited = parse_ui_graphics(&ui_bytes).unwrap();
    edited.direct_pointers[0].bank_offset = 0x40e28c;
    edited.validate().unwrap();
    let edited = encode_ui_graphics(&edited).unwrap();
    let mut forged = manifest.clone();
    forged.resources[index].sha256 = digest(&edited);
    fs::write(&manifest_path, toml::to_string_pretty(&forged).unwrap()).unwrap();
    fs::write(&ui_path, edited).unwrap();
    assert!(attach_from_directory(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(&owner, managed.bus.rom_resources().unwrap()));
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    fs::write(&ui_path, ui_bytes).unwrap();

    for source in &plan.source_ranges {
        let span = source.offset as usize..(source.offset + source.length) as usize;
        assert_eq!(digest(&rom[span.clone()]), source.sha256);
        for byte in &mut managed.bus.rom[span] {
            *byte ^= 0xff;
        }
    }
    for offset in PROPERTY_OFFSET..PROPERTY_OFFSET + 2 {
        managed.bus.rom[offset as usize] ^= 0xff;
    }

    let mut raw = Machine::new(rom);
    let mut verified = BTreeSet::new();
    let mut decoded_bytes = 0;
    let mut requests = Vec::new();
    for reference in plan.references.iter().filter(|row| !row.new.is_empty()) {
        let stream = plan
            .streams
            .iter()
            .find(|row| row.source == reference.source && !row.new.is_empty());
        if let Some(stream) = stream {
            decoded_bytes += stream.decoded_bytes;
            assert!(verified.insert(stream.source));
        }
        let original = run_reader(&mut raw, reference, stream);
        let loaded = run_reader(&mut managed, reference, stream);
        assert_eq!(loaded.matched_requests, original.matched_requests);
        assert_eq!(loaded.events.len(), original.events.len());
        for (source, target) in original.events.iter().zip(&loaded.events) {
            assert_eq!(source.key.service, Service::Raw);
            assert_eq!(target.key.service, Service::Typed);
            assert_eq!(source.key.read, target.key.read);
            assert_eq!(source.key.instruction, target.key.instruction);
            assert_eq!(source.requests, target.requests);
        }
        let covered: BTreeSet<_> = loaded
            .events
            .iter()
            .flat_map(|event| {
                let read = event.key.read;
                read.physical_offset..read.physical_offset + u32::try_from(read.length).unwrap()
            })
            .collect();
        let expected: BTreeSet<_> = (reference.literal..reference.literal + 4)
            .chain(stream.into_iter().flat_map(|row| row.source..row.end))
            .collect();
        assert_eq!(covered, expected);
        assert!(loaded.events.iter().any(|event| {
            event
                .key
                .instruction
                .is_some_and(|site| site.pc == BASE + reference.reader)
                && event.key.read.physical_offset == reference.literal
                && event.key.read.length == 4
        }));
        assert_eq!(raw.cpu.regs, managed.cpu.regs);
        assert_eq!(raw.cpu.cpsr, managed.cpu.cpsr);
        assert_eq!(raw.bus.clock, managed.bus.clock);
        assert!(raw.bus.ewram == managed.bus.ewram);
        assert!(raw.bus.iwram == managed.bus.iwram);
        requests.push(loaded.matched_requests);
    }
    assert_eq!(
        (requests.len(), verified.len(), decoded_bytes),
        (26, 6, 63_744)
    );

    let alias_reads = owner
        .unit_obj_ffff_reader_fields_stats()
        .unwrap()
        .property_bytes;
    assert!(alias_reads >= 2);
    // The word straddles the end of the first new span and the existing owner.
    let mixed = PROPERTY_OFFSET - 2;
    for mirror in [BASE, 0x0a000000, 0x0c000000] {
        assert_eq!(
            managed.bus.read32(mirror + mixed),
            raw.bus.read32(mirror + mixed)
        );
    }
    let alias_bytes = owner
        .unit_obj_ffff_reader_fields_stats()
        .unwrap()
        .property_bytes;
    assert_eq!(alias_bytes - alias_reads, 6);

    let result = serde_json::json!({
        "status": "PASS", "revision": [313, 312], "source_bytes": 47_843,
        "decoded_bytes": decoded_bytes, "decoded_streams": verified.len(), "request_counts": requests,
        "alias_read_bytes": alias_reads, "mixed_mirrors": 3,
        "prior_manifest_attach": true, "rehashed_edit_rejected": true,
        "manifest_sha256": digest(&manifest_bytes), "ui_file_sha256": new.sha256,
    });
    fs::write(work.join("manifest.toml"), manifest_bytes).unwrap();
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
