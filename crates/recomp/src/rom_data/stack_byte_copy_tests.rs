//! Verify all original EUR fixed ten-byte stack-copy paths.

use super::{
    attach_dataset, encode_copied_halfword_lookups,
    indexed_reader_bank_tests::{execute_until, initialize},
    read_dataset, COPIED_HALFWORD_LOOKUPS_ID, THUMB_PC_LITERALS_ID,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::{
        copied_halfword_lookups::stack_byte_copies::{COPY_BYTES, READERS},
        read_audit::{Reader, Service, TraceCapture, TraceFilter},
        Extraction, SourceRange,
    },
    Bus, Machine,
};
use std::{collections::BTreeSet, fs, path::PathBuf, sync::Arc};

const BASE: u32 = 0x0800_0000;
const STACK: usize = 0x7c00;
const CURRENT: (u32, u32) = (341, 340);
const PREVIOUS: (u32, u32) = (340, 339);
const PREVIOUS_MANIFEST_SHA256: &str =
    "6eddac576b41f442b67e4ff9bc2ebc549fec5e30e1ff1d021a36e0b2df560cdf";
const PLAN_SHA256: &str = "85b08267eec28fcf5e559be616452f3f49a27d673f8201006020549fd78ccef7";

fn input(name: &str) -> PathBuf {
    std::env::var_os(name).expect(name).into()
}

fn word_span(offset: u32, bytes: &[u8], role: &str) -> SourceRange {
    SourceRange {
        offset,
        length: bytes.len() as u32,
        sha256: digest(bytes),
        role: role.into(),
    }
}

#[test]
#[ignore = "requires the exact EUR ROM, retained e340 files and the E565 source plan"]
fn european_stack_byte_copies_use_extracted_sources() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let plan = fs::read(input("FFTA_STACK_COPY_PLAN")).unwrap();
    assert_eq!(digest(&plan), PLAN_SHA256);
    let plan: serde_json::Value = serde_json::from_slice(&plan).unwrap();
    let rows = plan["immediate_ten_byte_copies"].as_array().unwrap();
    assert_eq!(rows.len(), READERS.len());
    for (row, reader) in rows.iter().zip(&READERS) {
        assert_eq!(row["entry"], reader.entry);
        assert_eq!(row["call"], reader.call);
        assert_eq!(row["literal"], reader.literal);
        assert_eq!(row["source"], reader.source);
        assert_eq!(row["length"], COPY_BYTES);
        assert_eq!(row["stack_offset"], reader.stack_offset);
        assert_eq!(row["reader_sha256"], reader.reader_sha256);
        assert_eq!(row["source_sha256"], reader.source_sha256);
    }
    let dataset_path = input("FFTA_STATE_TEST_DATASET");
    assert_eq!(
        digest(&fs::read(dataset_path.join("manifest.toml")).unwrap()),
        PREVIOUS_MANIFEST_SHA256
    );
    let mut dataset = read_dataset(&dataset_path).unwrap();
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        PREVIOUS
    );
    let old_literal_owner = dataset
        .manifest
        .resources
        .iter()
        .find(|resource| resource.id == THUMB_PC_LITERALS_ID)
        .unwrap();
    for reader in READERS.iter().filter(|reader| !reader.owns_literal) {
        let offset = reader.literal as usize;
        assert!(old_literal_owner.sources.contains(&word_span(
            reader.literal,
            &rom[offset..offset + 4],
            "thumb-pc-literal",
        )));
    }

    let mut managed = Machine::new(rom.clone());
    attach_dataset(&mut managed, &dataset).unwrap();
    let predecessor = managed.bus.rom_resources().unwrap().clone();
    let offset = READERS[0].source as usize;
    managed.bus.rom[offset] ^= 0xff;
    assert_eq!(managed.bus.read8(BASE + offset as u32), !rom[offset]);
    managed.bus.rom[offset] = rom[offset];

    (
        dataset.manifest.extractor_revision,
        dataset.manifest.decoder_revision,
    ) = CURRENT;
    let catalog = Extraction::from_rom_for_manifest(&rom, &dataset.manifest)
        .unwrap()
        .copied_halfword_lookups
        .unwrap();
    let mut without_bank = catalog.decoded.clone();
    let bank = without_bank.stack_byte_copies.take().unwrap();
    assert_eq!(
        encode_copied_halfword_lookups(&without_bank)
            .unwrap()
            .as_slice(),
        dataset.copied_halfword_lookups_bytes.as_deref().unwrap()
    );
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(
        &predecessor,
        managed.bus.rom_resources().unwrap()
    ));

    let new_spans: Vec<_> = bank.source_spans().collect();
    assert_eq!(
        (
            new_spans.len(),
            new_spans
                .iter()
                .map(|(_, bytes, _)| bytes.len())
                .sum::<usize>()
        ),
        (12, 90)
    );
    let additions: Vec<_> = new_spans
        .iter()
        .map(|(offset, bytes, role)| word_span(*offset, bytes, role))
        .collect();
    let packet = encode_copied_halfword_lookups(&catalog.decoded).unwrap();
    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|resource| resource.id == COPIED_HALFWORD_LOOKUPS_ID)
        .unwrap();
    assert!(catalog.sources.starts_with(&resource.sources));
    assert_eq!(catalog.sources[resource.sources.len()..], additions);
    resource.sha256 = digest(&packet);
    resource.sources = catalog.sources;
    dataset.copied_halfword_lookups = Some(catalog.decoded);
    dataset.copied_halfword_lookups_bytes = Some(packet.clone());
    attach_dataset(&mut managed, &dataset).unwrap();

    let mut raw = Machine::new(rom.clone());
    let initial = raw.save_state().unwrap();
    let flip_sources = |machine: &mut Machine| {
        for reader in &READERS {
            for byte in
                &mut machine.bus.rom[reader.source as usize..reader.source as usize + COPY_BYTES]
            {
                *byte ^= 0xff;
            }
            for byte in &mut machine.bus.rom[reader.literal as usize..reader.literal as usize + 4] {
                *byte ^= 0xff;
            }
        }
    };
    let mut requests = 0;
    for reader in &READERS {
        let stop = BASE + reader.call + 4;
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&initial).unwrap();
            initialize(machine, BASE + reader.entry);
        }
        execute_until(&mut raw, stop, 256);
        flip_sources(&mut managed);
        let source = reader.source..reader.source + COPY_BYTES as u32;
        let literal = reader.literal..reader.literal + 4;
        let trace = TraceCapture::start(
            TraceFilter {
                readers: vec![Reader::Bus],
                ranges: vec![(source.start, source.end), (literal.start, literal.end)],
            },
            24,
        )
        .unwrap();
        trace.frame(Some(managed.bus.frames));
        execute_until(&mut managed, stop, 256);
        let trace = trace.finish();
        assert_eq!(trace.dropped_requests, 0);
        let mut covered = BTreeSet::new();
        for event in &trace.events {
            let access = event.key.read;
            let at = access.physical_offset;
            let site = event.key.instruction.unwrap();
            assert_eq!(
                event.key.service,
                Service::Typed,
                "original read at {:08x}",
                site.pc
            );
            assert!(site.thumb);
            if literal.contains(&at) {
                assert_eq!(site.pc, BASE + reader.call - 6);
            } else {
                assert!((0x08147dc0..0x08147e22).contains(&site.pc));
            }
            covered.extend(at..at + access.length as u32);
        }
        assert_eq!(covered, source.clone().chain(literal).collect());
        requests += trace.matched_requests;
        let stack = STACK + usize::from(reader.stack_offset);
        assert_eq!(
            &managed.bus.iwram[stack..stack + COPY_BYTES],
            &rom[source.start as usize..source.end as usize]
        );
        flip_sources(&mut managed);
        assert_eq!(managed.bus.clock, raw.bus.clock);
        assert!(
            managed.save_state().unwrap() == raw.save_state().unwrap(),
            "state after {:08x}",
            BASE + reader.entry
        );
    }

    flip_sources(&mut managed);
    for gap in [0x14d936, 0x14d942] {
        for machine in [&mut raw, &mut managed] {
            for byte in &mut machine.bus.rom[gap..gap + 2] {
                *byte ^= 0xff;
            }
        }
        assert_eq!(
            managed.bus.read16(BASE + gap as u32),
            raw.bus.read16(BASE + gap as u32)
        );
    }
    let mut boundary_reads = 0;
    for (offset, bytes, _) in &new_spans {
        for at in [offset - 1, offset + bytes.len() as u32 - 1] {
            for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
                assert_eq!(managed.bus.read16(mirror + at), raw.bus.read16(mirror + at));
                assert_eq!(managed.bus.read32(mirror + at), raw.bus.read32(mirror + at));
                boundary_reads += 2;
            }
        }
    }

    let work = input("FFTA_STACK_COPY_WORK");
    let manifest = toml::to_string_pretty(&dataset.manifest).unwrap();
    fs::write(work.join("manifest.toml"), &manifest).unwrap();
    fs::write(work.join("copied-halfword-lookups.json"), &packet).unwrap();
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "status": "PASS", "revision": CURRENT, "manifest_sha256": digest(manifest.as_bytes()),
            "new_source_bytes": 70, "new_literal_bytes": 20, "previous_literals": 2,
            "original_copy_paths": READERS.len(), "typed_requests": requests,
            "mixed_mirror_checks": boundary_reads, "raw_gaps_preserved": true,
            "full_state_and_clock_equal_each_copy": true,
            "predecessor_attach": true, "missing_group_rejected": true,
        }))
        .unwrap(),
    )
    .unwrap();
}
