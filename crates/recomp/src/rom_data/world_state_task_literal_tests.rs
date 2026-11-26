//! Verify all bounded world-state task literal reads on the exact EUR ROM.

use super::{
    attach_dataset, encode_thumb_pc_literals,
    indexed_reader_bank_tests::{execute_until, initialize},
    parse_thumb_pc_literals, read_dataset, THUMB_PC_LITERALS_ID,
};
use gba_core::{
    capi::RT_API,
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::{
        read_audit::{Reader, Service, TraceCapture, TraceFilter},
        thumb_pc_literals::SOURCE_ROLE,
        Extraction, SourceRange,
    },
    Bus, Machine,
};
use recomp_core::analyze::{Analysis, Block, View};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
    sync::Arc,
};

const BASE: u32 = 0x0800_0000;
const WORLD_STATE_REVISION: (u32, u32) = (340, 339);
const CENSUS_SHA256: &str = "d8274316637c8a490de63e1771a0cc5d08847d80c2a6b1333525936e425e182a";

#[derive(Deserialize)]
struct Census {
    literal_sources: Vec<Word>,
}

#[derive(Deserialize)]
struct Word {
    offset: u32,
    length: u32,
    sha256: String,
    readers: Vec<Load>,
    owners: Vec<Owner>,
}

#[derive(Deserialize)]
struct Load {
    instruction_offset: u32,
    halfword: u16,
}

#[derive(Deserialize)]
struct Owner {
    resource: String,
    role: String,
}

fn input(name: &str) -> PathBuf {
    std::env::var_os(name).expect(name).into()
}

fn read_census() -> Census {
    let bytes = fs::read(input("FFTA_WORLD_STATE_TASK_CENSUS")).unwrap();
    assert_eq!(digest(&bytes), CENSUS_SHA256);
    serde_json::from_slice(&bytes).unwrap()
}

fn run_load(machine: &mut Machine, rom: &[u8], word: &Word, load: &Load) -> ([u32; 16], u32, u64) {
    let start = BASE + load.instruction_offset;
    initialize(machine, start);
    execute_until(machine, start + 2, 4);
    let destination = usize::from((load.halfword >> 8) & 7);
    let offset = word.offset as usize;
    let expected = u32::from_le_bytes(rom[offset..offset + 4].try_into().unwrap());
    assert_eq!(
        machine.cpu.regs[destination], expected,
        "load at {start:08x}"
    );
    (machine.cpu.regs, machine.cpu.cpsr, machine.bus.clock)
}

fn check_mixed_reads(machine: &mut Machine, rom: &[u8]) -> usize {
    let mut requests = 0;
    for (offset, raw) in [
        (0x03409f, [true, false, false, false]), // raw bytes and the new page
        (0x0360ab, [false, true, true, true]),   // end of the final added word
        (0x034117, [false; 4]),                  // new word and an earlier custom-bank literal
        (0x033cdf, [false; 4]),                  // earlier world-map literal and new word
    ] {
        let range = offset..offset + 4;
        for byte in &mut machine.bus.rom[range.clone()] {
            *byte = !*byte;
        }
        let expected: [u8; 4] = std::array::from_fn(|index| {
            let original = rom[offset + index];
            if raw[index] {
                !original
            } else {
                original
            }
        });
        for mirror in [0x0800_0000, 0x0a00_0000, 0x0c00_0000] {
            let address = mirror + offset as u32;
            assert_eq!(
                machine.bus.read16(address),
                u16::from_le_bytes([expected[0], expected[1]])
            );
            assert_eq!(machine.bus.read32(address), u32::from_le_bytes(expected));
            requests += 2;
        }
        machine.bus.rom[range.clone()].copy_from_slice(&rom[range]);
    }
    requests
}

#[test]
#[ignore = "requires the exact EUR ROM, retained e339 files and the E560 census"]
fn european_world_state_task_serves_the_complete_literal_bank() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let census = read_census();
    let words = &census.literal_sources;
    assert_eq!(words.len(), 372);
    let mut dataset = read_dataset(&input("FFTA_STATE_TEST_DATASET")).unwrap();
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        (339, 338)
    );
    let mut updated_manifest = dataset.manifest.clone();
    (
        updated_manifest.extractor_revision,
        updated_manifest.decoder_revision,
    ) = WORLD_STATE_REVISION;
    let catalog = Extraction::from_rom_for_manifest(&rom, &updated_manifest)
        .unwrap()
        .thumb_pc_literals
        .unwrap();

    let mut additions = Vec::new();
    for word in words {
        let offset = word.offset;
        assert_eq!(word.length, 4);
        assert_eq!(
            digest(&rom[offset as usize..offset as usize + 4]),
            word.sha256
        );
        let mut span = SourceRange {
            offset,
            length: 4,
            sha256: word.sha256.clone(),
            role: SOURCE_ROLE.into(),
        };
        match word.owners.as_slice() {
            [] => additions.push(span),
            [owner] => {
                span.role.clone_from(&owner.role);
                let previous = dataset
                    .manifest
                    .resources
                    .iter()
                    .find(|entry| entry.id == owner.resource)
                    .unwrap();
                assert!(previous.sources.contains(&span));
            }
            _ => panic!("Multiple owners for {offset:06x}"),
        }
    }
    assert_eq!(additions.len(), 357);
    let mut predecessor_packet = catalog.decoded.clone();
    assert_eq!(
        predecessor_packet.world_state_task_literals.len(),
        additions.len()
    );
    predecessor_packet.world_state_task_literals.clear();
    assert!(
        Some(
            encode_thumb_pc_literals(&predecessor_packet)
                .unwrap()
                .as_slice()
        ) == dataset.thumb_pc_literals_bytes.as_deref(),
        "unchanged predecessor encoding"
    );

    let mut managed = Machine::new(rom.clone());
    attach_dataset(&mut managed, &dataset).unwrap();
    let predecessor = managed.bus.rom_resources().unwrap().clone();
    let sample = 0x0340a0;
    managed.bus.rom[sample] = !rom[sample];
    assert_eq!(managed.bus.read8(BASE + sample as u32), !rom[sample]);
    managed.bus.rom[sample] = rom[sample];
    dataset.manifest = updated_manifest;
    assert!(attach_dataset(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(
        managed.bus.rom_resources().unwrap(),
        &predecessor
    ));

    let bytes = encode_thumb_pc_literals(&catalog.decoded).unwrap();
    let resource = dataset
        .manifest
        .resources
        .iter_mut()
        .find(|entry| entry.id == THUMB_PC_LITERALS_ID)
        .unwrap();
    assert!(catalog.sources.starts_with(&resource.sources));
    assert_eq!(catalog.sources[resource.sources.len()..], additions);
    resource.sha256 = digest(&bytes);
    resource.sources = catalog.sources;
    dataset.thumb_pc_literals = Some(parse_thumb_pc_literals(&bytes).unwrap());
    dataset.thumb_pc_literals_bytes = Some(bytes);
    attach_dataset(&mut managed, &dataset).unwrap();

    let cases: Vec<_> = words
        .iter()
        .flat_map(|word| word.readers.iter().map(move |load| (word, load)))
        .collect();
    assert_eq!(cases.len(), 381);
    assert_eq!(
        cases
            .iter()
            .filter(|(word, _)| word.owners.is_empty())
            .count(),
        365
    );
    let mut raw = Machine::new(rom.clone());
    managed.load_state(&raw.save_state().unwrap()).unwrap();
    let raw_results: Vec<_> = cases
        .iter()
        .map(|(word, load)| run_load(&mut raw, &rom, word, load))
        .collect();
    for word in words {
        for byte in &mut managed.bus.rom[word.offset as usize..word.offset as usize + 4] {
            *byte = !*byte;
        }
    }
    let filter = TraceFilter {
        readers: vec![Reader::Bus],
        ranges: vec![(
            words.first().unwrap().offset,
            words.last().unwrap().offset + 4,
        )],
    };
    let capture = TraceCapture::start(filter, cases.len()).unwrap();
    capture.frame(Some(0));
    for ((word, load), raw_result) in cases.iter().zip(raw_results) {
        assert_eq!(run_load(&mut managed, &rom, word, load), raw_result);
    }
    let trace = capture.finish();
    assert_eq!(trace.dropped_requests, 0);
    assert_eq!(trace.matched_requests, cases.len() as u64);
    let mut expected: Vec<_> = cases
        .iter()
        .map(|(word, load)| (word.offset, BASE + load.instruction_offset))
        .collect();
    let mut observed: Vec<_> = trace
        .events
        .iter()
        .map(|event| {
            assert_eq!(event.requests, 1);
            assert_eq!(event.key.service, Service::Typed);
            assert_eq!(event.key.read.length, 4);
            let site = event.key.instruction.unwrap();
            assert!(site.thumb);
            (event.key.read.physical_offset, site.pc)
        })
        .collect();
    observed.sort_unstable();
    expected.sort_unstable();
    assert_eq!(observed, expected);
    for word in words {
        let range = word.offset as usize..word.offset as usize + 4;
        managed.bus.rom[range.clone()].copy_from_slice(&rom[range]);
    }
    assert!(managed.save_state().unwrap() == raw.save_state().unwrap());
    let mixed_reads = check_mixed_reads(&mut managed, &rom);

    let work = input("FFTA_WORLD_STATE_TASK_WORK");
    let manifest = toml::to_string_pretty(&dataset.manifest).unwrap();
    fs::write(work.join("manifest.toml"), &manifest).unwrap();
    let result = serde_json::json!({
        "status": "PASS", "revision": WORLD_STATE_REVISION,
        "manifest_sha256": digest(manifest.as_bytes()), "published_words": additions.len(),
        "original_typed_requests": trace.matched_requests, "new_reader_sites": 365,
        "mixed_mirror_requests": mixed_reads, "previous_owners_checked": words.len() - additions.len(),
        "registers_and_clock_equal_per_read": true, "final_full_state_equal": true,
        "predecessor_attach": true, "missing_bank_rejected": true,
    });
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}

#[test]
#[ignore = "requires the exact EUR ROM, the e340 dataset, the E560 census and TinyCC"]
fn european_world_state_task_generated_loads_use_the_complete_literal_bank() {
    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let census = read_census();
    let words = &census.literal_sources;
    assert_eq!(words.len(), 372);
    let cases: BTreeMap<_, _> = words
        .iter()
        .flat_map(|word| {
            word.readers
                .iter()
                .map(move |load| (BASE + load.instruction_offset, (word, load)))
        })
        .collect();
    assert_eq!(cases.len(), 381);

    let work = input("FFTA_WORLD_STATE_TASK_WORK");
    let dataset = read_dataset(&input("FFTA_STATE_TEST_DATASET")).unwrap();
    assert_eq!(
        (
            dataset.manifest.extractor_revision,
            dataset.manifest.decoder_revision
        ),
        WORLD_STATE_REVISION
    );
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let scratch = tempfile::tempdir_in(&work).unwrap();
    let compiler = recomp_core::tcc_memory::library_path().unwrap();

    let mut raw = Machine::new(rom.clone());
    let mut generated = Machine::new(rom.clone());
    attach_dataset(&mut generated, &dataset).unwrap();
    let initial = raw.save_state().unwrap();
    let flip_backing = |machine: &mut Machine| {
        for word in words {
            let start = word.offset as usize;
            for byte in &mut machine.bus.rom[start..start + word.length as usize] {
                *byte ^= 0xff;
            }
        }
    };
    let filter = TraceFilter {
        readers: vec![Reader::Bus],
        ranges: vec![(words[0].offset, words.last().unwrap().offset + 4)],
    };
    let mut covered = BTreeSet::new();
    // Keep adjacent instructions in different objects so the emitter cannot
    // chain them. Each entry must return after the single original load.
    for alignment in [0, 2] {
        let group: Vec<_> = cases
            .iter()
            .filter(|(pc, _)| **pc & 2 == alignment)
            .collect();
        let blocks = group
            .iter()
            .map(|&(&addr, &(_, load))| {
                let opcode = view.read16(addr);
                assert_eq!(opcode, load.halfword, "load at {addr:08x}");
                Block {
                    addr,
                    thumb: true,
                    instrs: vec![armv4t::decode_thumb(opcode, addr)],
                }
            })
            .collect();
        let object = scratch
            .path()
            .join(format!("world-state-task-loads-{alignment}.o"));
        recomp_core::tcc_memory::build(
            &Analysis { blocks },
            &view,
            &compiler,
            &object,
            &mut |_| {},
        )
        .unwrap();
        let (_library, table) = crate::load_native(&object).unwrap();

        for (&pc, &(word, load)) in group {
            raw.load_state(&initial).unwrap();
            generated.load_state(&initial).unwrap();
            let (regs, cpsr, clock) = run_load(&mut raw, &rom, word, load);
            initialize(&mut generated, pc);
            flip_backing(&mut generated);

            let capture = TraceCapture::start(filter.clone(), 4).unwrap();
            capture.frame(Some(generated.bus.frames));
            let entry = table.get(pc | 1).expect("generated literal load entry");
            let next = entry(&RT_API, (&mut generated as *mut Machine).cast());
            assert_eq!(next, (pc + 2) | 1, "load at {pc:08x}");
            let trace = capture.finish();
            assert_eq!(
                (trace.matched_requests, trace.dropped_requests),
                (1, 0),
                "load at {pc:08x}"
            );
            let [event] = trace.events.as_slice() else {
                panic!("Expected a single read at {pc:08x}");
            };
            assert_eq!(event.key.service, Service::Typed, "load at {pc:08x}");
            assert_eq!(event.key.read.guest_address, BASE + word.offset);
            assert_eq!(event.key.read.physical_offset, word.offset);
            assert_eq!(event.key.read.length, 4);
            assert!(
                event.key.instruction.is_none(),
                "interpreter load at {pc:08x}"
            );
            covered.insert(word.offset);

            flip_backing(&mut generated);
            assert_eq!(
                (generated.cpu.regs, generated.cpu.cpsr, generated.bus.clock),
                (regs, cpsr, clock),
                "generated state at {pc:08x}"
            );
            assert!(
                generated.save_state().unwrap() == raw.save_state().unwrap(),
                "serialized state at {pc:08x}"
            );
        }
    }
    assert_eq!(covered, words.iter().map(|word| word.offset).collect());
    let result = serde_json::json!({
        "status": "PASS",
        "revision": WORLD_STATE_REVISION,
        "generated_typed_requests": cases.len(),
        "compiled_load_pcs": cases.keys().collect::<Vec<_>>(),
        "covered_words": covered.len(),
        "previous_owners_checked": words.iter().filter(|word| !word.owners.is_empty()).count(),
        "all_backing_words_altered": true,
        "state_and_guest_clock_equal_each_load": true,
    });
    fs::write(
        work.join("generated-result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}
