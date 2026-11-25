//! Verify the complete five-language custom graphics group through the original caller.
use super::{attach_from_directory, cmd, encode_ui_graphics, parse, parse_ui_graphics, Manifest};
use super::{Extraction, MANIFEST_FILE, UI_CUSTOM_GRAPHICS_FILE, UI_CUSTOM_GRAPHICS_ID};
use gba_core::{
    ffta_rom::digest,
    rom_data::read_audit::{Service, TraceCapture, TraceFilter, TraceReport},
    Bus, Machine,
};
use serde::Deserialize;
use std::{collections::BTreeSet, fs, path::PathBuf, sync::Arc};

const BASE: u32 = 0x08000000;
const TABLE: u32 = 0x3adfb0;
const TABLE_LITERAL: u32 = 0x13c014;
const ENTRY: u32 = BASE + 0x13bf70;
const TRANSFER_END: u32 = BASE + 0x13bf8c;
const LANGUAGE: u32 = 0x02001f6e;
const SCRATCH: u32 = super::ui_graphics_tests::SCRATCH;
const DESTINATION: usize = 0xb180;
const OUTPUT_BYTES: usize = 1664;

#[derive(Deserialize)]
struct Plan {
    source_ranges: Vec<Source>,
    banks: Vec<Stream>,
}

#[derive(Deserialize)]
struct Source {
    offset: u32,
    length: u32,
    sha256: String,
}

#[derive(Deserialize)]
struct Stream {
    language_slot: u8,
    source_offset: u32,
    source_end: u32,
    decoded_sha256: String,
}

fn input(name: &str) -> PathBuf {
    PathBuf::from(std::env::var_os(name).unwrap())
}

fn configure(machine: &mut Machine, language: u8) {
    machine.bus.ffta_sprites.enabled = false;
    machine.bus.ewram.fill(0x5a);
    machine.bus.iwram.fill(0x3c);
    machine.bus.vram.fill(0x58);
    machine.bus.write8(LANGUAGE, language);
    machine.cpu.regs.fill(0);
    machine.cpu.regs[4] = SCRATCH;
    machine.cpu.regs[5] = LANGUAGE;
    machine.cpu.regs[13] = 0x03007c00;
    machine.cpu.regs[15] = ENTRY;
    machine.cpu.cpsr = 0xbf;
    machine.cpu.branch = None;
}

fn execute(machine: &mut Machine, ranges: &[(u32, u32)], stream: &Stream) -> TraceReport {
    configure(machine, stream.language_slot);
    let capture = TraceCapture::start(
        TraceFilter {
            readers: Vec::new(),
            ranges: ranges.to_vec(),
        },
        4096,
    )
    .unwrap();
    capture.frame(Some(0));
    let mut decoded = false;
    for _ in 0..200_000 {
        match machine.cpu.regs[15] {
            0x080054a0 => assert_eq!(
                &machine.cpu.regs[..2],
                &[SCRATCH, BASE + stream.source_offset],
            ),
            0x080058de => {
                assert_eq!(machine.cpu.regs[7], BASE + stream.source_end);
                assert_eq!(machine.cpu.regs[9], OUTPUT_BYTES as u32);
                decoded = true;
            }
            0x0813bf80 => assert_eq!(machine.cpu.regs[0], OUTPUT_BYTES as u32),
            TRANSFER_END => break,
            _ => {}
        }
        machine.step();
    }
    assert!(decoded);
    assert_eq!(machine.cpu.regs[15], TRANSFER_END);
    assert_eq!(machine.cpu.regs[13], 0x03007c00);
    let scratch = (SCRATCH - 0x02000000) as usize;
    let output = &machine.bus.ewram[scratch..scratch + OUTPUT_BYTES];
    assert_eq!(digest(output), stream.decoded_sha256);
    assert_eq!(
        output,
        &machine.bus.vram[DESTINATION..DESTINATION + OUTPUT_BYTES]
    );
    assert_eq!(machine.bus.ewram[scratch - 1], 0x5a);
    assert_eq!(machine.bus.ewram[scratch + OUTPUT_BYTES], 0x5a);
    assert_eq!(machine.bus.vram[DESTINATION - 1], 0x58);
    assert_eq!(machine.bus.vram[DESTINATION + OUTPUT_BYTES], 0x58);
    let report = capture.finish();
    assert!(!report.truncated && report.dropped_requests == 0);
    report
}

#[test]
#[ignore = "exact EUR ROM, E438 source plan and E491 predecessor manifest"]
fn european_additional_ui_table_publication_and_original_reads() {
    let rom_path = input("FFTA_STATE_TEST_ROM");
    let rom = fs::read(&rom_path).unwrap();
    let work = input("FFTA_ADDITIONAL_UI_TABLE_WORK");
    let plan: Plan =
        serde_json::from_slice(&fs::read(input("FFTA_ADDITIONAL_UI_TABLE_PLAN")).unwrap()).unwrap();
    let previous_bytes = fs::read(input("FFTA_ADDITIONAL_UI_TABLE_PREDECESSOR")).unwrap();
    let previous: Manifest = parse(&previous_bytes, "E491 manifest").unwrap();
    assert_eq!(
        (previous.extractor_revision, previous.decoder_revision),
        (311, 310)
    );
    assert_eq!((plan.source_ranges.len(), plan.banks.len()), (7, 5));

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
        .join("v1/e312-d311");
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
    let manifest: Manifest = parse(&manifest_bytes, "E312 manifest").unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (312, 311)
    );
    assert_eq!(manifest.coverage, previous.coverage);
    assert_eq!(manifest.resources.len(), previous.resources.len());
    let index = manifest
        .resources
        .iter()
        .position(|entry| entry.id == UI_CUSTOM_GRAPHICS_ID)
        .unwrap();
    for (i, (new, old)) in manifest
        .resources
        .iter()
        .zip(&previous.resources)
        .enumerate()
    {
        if i != index {
            assert_eq!(new, old);
        }
    }
    let old = &previous.resources[index];
    let new = &manifest.resources[index];
    assert_eq!(
        (&new.path, &new.format, &new.dependencies),
        (&old.path, &old.format, &old.dependencies)
    );
    assert!(old
        .sources
        .iter()
        .all(|source| new.sources.contains(source)));
    let added: Vec<_> = new
        .sources
        .iter()
        .filter(|entry| !old.sources.contains(entry))
        .collect();
    assert_eq!(added.len(), 11); // Five banks, five pointer words, and the caller literal.
    assert_eq!(added.iter().map(|entry| entry.length).sum::<u32>(), 3061);
    let ranges: Vec<_> = plan
        .source_ranges
        .iter()
        .map(|plan| {
            let end = plan.offset + plan.length;
            let mut offset = plan.offset;
            while offset < end {
                let source = added.iter().find(|source| source.offset == offset).unwrap();
                offset += source.length;
                assert!(offset <= end);
            }
            assert_eq!(
                digest(&rom[plan.offset as usize..end as usize]),
                plan.sha256
            );
            (plan.offset, end)
        })
        .collect();
    assert!(added.iter().all(|entry| ranges
        .iter()
        .any(|&(lo, hi)| { lo <= entry.offset && entry.offset + entry.length <= hi })));

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
    let selector = edited
        .selectors
        .iter_mut()
        .find(|selector| selector.table_literal.is_some())
        .unwrap();
    selector.table_literal = Some(TABLE_LITERAL - 4);
    edited.validate().unwrap();
    let edited = encode_ui_graphics(&edited).unwrap();
    let mut forged = manifest.clone();
    forged.resources[index].sha256 = digest(&edited);
    fs::write(&manifest_path, toml::to_string_pretty(&forged).unwrap()).unwrap();
    fs::write(&ui_path, edited).unwrap();
    assert!(attach_from_directory(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(&owner, managed.bus.rom_resources().unwrap()));
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    fs::write(&ui_path, &ui_bytes).unwrap();

    for &(lo, hi) in &ranges {
        for byte in &mut managed.bus.rom[lo as usize..hi as usize] {
            *byte ^= 0xff;
        }
    }
    let mut counts = Vec::new();
    for (language, stream) in plan.banks.iter().enumerate() {
        assert_eq!(usize::from(stream.language_slot), language);
        let mut raw = Machine::new(rom.clone());
        let mut trial = Machine::new(rom.clone());
        trial.bus.set_rom_resources(Some(owner.clone())).unwrap();
        trial.bus.rom.copy_from_slice(&managed.bus.rom);
        let source_report = execute(&mut raw, &ranges, stream);
        let loaded_report = execute(&mut trial, &ranges, stream);
        assert!(source_report.matched_requests > 0);
        assert_eq!(
            loaded_report.matched_requests,
            source_report.matched_requests
        );
        assert_eq!(loaded_report.events.len(), source_report.events.len());
        for (source, loaded) in source_report.events.iter().zip(&loaded_report.events) {
            assert_eq!(source.key.service, Service::Raw);
            assert_eq!(loaded.key.service, Service::Typed);
            assert_eq!(source.key.read, loaded.key.read);
            assert_eq!(source.key.instruction, loaded.key.instruction);
            assert_eq!(source.requests, loaded.requests);
        }
        let read = |pc, offset| {
            loaded_report.events.iter().any(|event| {
                event.key.instruction.is_some_and(|site| site.pc == pc)
                    && event.key.read.physical_offset == offset
                    && event.key.read.length == 4
            })
        };
        assert!(read(ENTRY, TABLE_LITERAL));
        let pointer = TABLE + language as u32 * 4;
        assert!(read(BASE + 0x13bf78, pointer));
        let covered: BTreeSet<_> = loaded_report
            .events
            .iter()
            .flat_map(|event| {
                let read = event.key.read;
                read.physical_offset..read.physical_offset + u32::try_from(read.length).unwrap()
            })
            .collect();
        let expected: BTreeSet<_> = (TABLE_LITERAL..TABLE_LITERAL + 4)
            .chain(pointer..pointer + 4)
            .chain(stream.source_offset..stream.source_end)
            .collect();
        assert_eq!(covered, expected);
        assert_eq!(raw.cpu.regs, trial.cpu.regs);
        assert_eq!(raw.bus.clock, trial.bus.clock);
        assert!(raw.bus.ewram == trial.bus.ewram);
        assert!(raw.bus.iwram == trial.bus.iwram);
        assert!(raw.bus.vram == trial.bus.vram);
        assert!(raw.bus.oam == trial.bus.oam);
        assert!(raw.bus.palette == trial.bus.palette);
        counts.push(loaded_report.matched_requests);
    }
    let mut raw = Machine::new(rom.clone());
    for language in [5, 255] {
        let address = TABLE + 4 * u32::from(language);
        for machine in [&mut raw, &mut managed] {
            configure(machine, language);
            for _ in 0..5 {
                machine.step();
            }
            assert_eq!(machine.cpu.regs[15], BASE + 0x13bf7a);
            assert_eq!(machine.cpu.regs[0], BASE + address);
            let field = u32::from_le_bytes(
                rom[address as usize..address as usize + 4]
                    .try_into()
                    .unwrap(),
            );
            assert_eq!(machine.cpu.regs[1], field);
        }
    }

    let result = serde_json::json!({
        "status": "PASS", "revision": [312, 311], "source_bytes": 3061,
        "decoded_bytes": 5 * OUTPUT_BYTES, "request_counts": counts,
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
