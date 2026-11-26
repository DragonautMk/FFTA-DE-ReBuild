//! Verify extracted byte-copy producer literals against original Thumb reads.

use super::{
    attach_dataset,
    direct_byte_copy_tests::{capture, flip, input, same_state},
    encode_copied_halfword_lookups,
    indexed_reader_bank_tests::{execute_until, initialize},
    read_dataset, COPIED_HALFWORD_LOOKUPS_ID,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::{
        copied_halfword_lookups::fixed_byte_copies::{
            PRODUCER_LITERAL_ROLE, SHARED_SLOT_LITERAL_ROLE,
        },
        read_audit::Service,
        Extraction, SourceRange,
    },
    Bus, Machine,
};
use serde::Deserialize;
use std::{fs, sync::Arc};

const BASE: u32 = 0x0800_0000;

struct LiteralBankCase {
    revision: (u32, u32),
    predecessor: (u32, u32),
    predecessor_manifest_sha256: &'static str,
    plan_sha256: &'static str,
    role: &'static str,
    raw_gap: u32,
    source_words: usize,
    added_words: usize,
    readers: usize,
}

#[derive(Deserialize)]
struct Plan {
    #[serde(alias = "literal_spans")]
    literal_ranges: Vec<LiteralRange>,
}

#[derive(Deserialize)]
struct LiteralRange {
    offset: u32,
    length: u32,
    sha256: String,
    readers: Vec<u32>,
    residual_bytes: u32,
}

#[test]
#[ignore = "requires the exact EUR ROM, retained e343 files and the E574 producer plan"]
fn european_byte_copy_producer_literals_use_extracted_sources() {
    verify_bank(LiteralBankCase {
        revision: (344, 343),
        predecessor: (343, 342),
        predecessor_manifest_sha256:
            "bf97bffda16641ffb150e401258f42cbafa2638db91da65278a842d31b4d72e4",
        plan_sha256: "d63b0112009762308b98b904b85058747abc1691b8c7d9ef3d5366af1aa1ef9a",
        role: PRODUCER_LITERAL_ROLE,
        raw_gap: 0x006910,
        source_words: 124,
        added_words: 84,
        readers: 85,
    });
}

#[test]
#[ignore = "requires the exact EUR ROM, retained e344 files and the E577 producer plan"]
fn european_byte_copy_shared_slot_literals_use_extracted_sources() {
    verify_bank(LiteralBankCase {
        revision: (345, 344),
        predecessor: (344, 343),
        predecessor_manifest_sha256:
            "6c0288afbd3f30c7a1284a77fd376868e68c98d202cd1d3214755f479d319bcd",
        plan_sha256: "feb89e8579fe9bb3fe2bef50995e7e94adbdc4f127cbb32dd26bdf79e6c89f49",
        role: SHARED_SLOT_LITERAL_ROLE,
        raw_gap: 0x0270b0,
        source_words: 51,
        added_words: 27,
        readers: 27,
    });
}

fn verify_bank(case: LiteralBankCase) {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let plan = fs::read(input("FFTA_COPY_PRODUCER_PLAN")).unwrap();
    assert_eq!(digest(&plan), case.plan_sha256);
    let plan: Plan = serde_json::from_slice(&plan).unwrap();
    assert_eq!(plan.literal_ranges.len(), case.source_words);
    let residuals: Vec<_> = plan
        .literal_ranges
        .iter()
        .filter(|span| span.residual_bytes != 0)
        .collect();
    assert_eq!(residuals.len(), case.added_words);
    let additions: Vec<_> = residuals
        .iter()
        .map(|span| {
            assert_eq!(
                (span.length, span.residual_bytes, span.offset % 4),
                (4, 4, 0)
            );
            SourceRange {
                offset: span.offset,
                length: span.length,
                role: case.role.into(),
                sha256: span.sha256.clone(),
            }
        })
        .collect();

    let dataset_path = input("FFTA_STATE_TEST_DATASET");
    assert_eq!(
        digest(&fs::read(dataset_path.join("manifest.toml")).unwrap()),
        case.predecessor_manifest_sha256
    );
    let mut dataset = read_dataset(&dataset_path).unwrap();
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        case.predecessor
    );
    let mut managed = Machine::new(rom.clone());
    attach_dataset(&mut managed, &dataset).unwrap();
    let predecessor = managed.bus.rom_resources().unwrap().clone();
    let first = additions[0].offset;
    flip(&mut managed, first, 4);
    assert_eq!(managed.bus.read8(BASE + first), !rom[first as usize]);
    flip(&mut managed, first, 4);
    (
        dataset.manifest.extractor_revision,
        dataset.manifest.decoder_revision,
    ) = case.revision;
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(
        &predecessor,
        managed.bus.rom_resources().unwrap()
    ));

    let catalog = Extraction::from_rom_for_manifest(&rom, &dataset.manifest)
        .unwrap()
        .copied_halfword_lookups
        .unwrap();
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
    let mut loads = 0;
    for span in &residuals {
        let start = span.offset as usize;
        let expected = u32::from_le_bytes(rom[start..start + 4].try_into().unwrap());
        for &reader in &span.readers {
            let pc = BASE + reader;
            let at = reader as usize;
            let opcode = u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
            assert_eq!(opcode & 0xf800, 0x4800);
            assert_eq!(
                ((reader + 4) & !3) + 4 * u32::from(opcode & 0xff),
                span.offset
            );
            let register = ((opcode >> 8) & 7) as usize;
            for machine in [&mut raw, &mut managed] {
                machine.load_state(&initial).unwrap();
                initialize(machine, pc);
            }
            execute_until(&mut raw, pc + 2, 2);
            flip(&mut managed, span.offset, 4);
            let trace = capture(&managed, vec![(span.offset, span.offset + 4)], 2);
            execute_until(&mut managed, pc + 2, 2);
            let report = trace.finish();
            assert_eq!((report.matched_requests, report.dropped_requests), (1, 0));
            let [event] = report.events.as_slice() else {
                panic!("Expected one producer read at {pc:08x}");
            };
            let site = event.key.instruction.unwrap();
            assert_eq!((site.pc, site.thumb), (pc, true));
            assert_eq!(
                event.key.service,
                Service::Typed,
                "producer load at {pc:08x}"
            );
            assert_eq!(
                (event.key.read.physical_offset, event.key.read.length),
                (span.offset, 4)
            );
            assert_eq!(managed.cpu.regs[register], expected);
            flip(&mut managed, span.offset, 4);
            same_state(&raw, &managed, pc);
            loads += 1;
        }
    }
    assert_eq!(loads, case.readers);

    let raw_gap = case.raw_gap;
    assert!(dataset.manifest.resources.iter().all(|resource| {
        resource
            .sources
            .iter()
            .all(|span| span.offset >= raw_gap + 4 || span.offset + span.length <= raw_gap)
    }));
    let gap = raw_gap as usize;
    let original = u32::from_le_bytes(rom[gap..gap + 4].try_into().unwrap());
    for machine in [&mut raw, &mut managed] {
        flip(machine, raw_gap, 4);
        assert_eq!(machine.bus.read32(BASE + raw_gap), !original);
    }
    for span in &residuals {
        flip(&mut managed, span.offset, 4);
    }
    let mut boundaries = 0;
    for span in residuals {
        for at in [span.offset - 1, span.offset + 3] {
            for mirror in [BASE, 0x0a00_0000, 0x0c00_0000] {
                let address = mirror + at;
                assert_eq!(
                    managed.bus.read16(address),
                    raw.bus.read16(address),
                    "halfword at {address:08x}"
                );
                assert_eq!(
                    managed.bus.read32(address),
                    raw.bus.read32(address),
                    "word at {address:08x}"
                );
                boundaries += 2;
            }
        }
    }

    let work = input("FFTA_COPY_PRODUCER_WORK");
    let manifest = toml::to_string_pretty(&dataset.manifest).unwrap();
    fs::write(work.join("manifest.toml"), &manifest).unwrap();
    fs::write(work.join("copied-halfword-lookups.json"), &packet).unwrap();
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "status": "PASS", "revision": case.revision, "manifest_sha256": digest(manifest.as_bytes()),
            "new_words": additions.len(), "new_literal_bytes": 4 * additions.len(),
            "original_readers": loads, "typed_requests": loads, "mixed_mirror_checks": boundaries,
            "full_state_and_clock_equal": true, "raw_gap_preserved": true,
            "predecessor_attach": true, "missing_group_rejected": true,
        }))
        .unwrap(),
    )
    .unwrap();
}
