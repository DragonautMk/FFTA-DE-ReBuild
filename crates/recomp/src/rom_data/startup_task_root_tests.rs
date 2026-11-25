//! Check the EUR startup dispatcher and callbacks at their original readers.

mod title_color_mode;
mod title_frontend;

use super::{
    attach_from_directory, cmd, encode_thumb_pc_literals, parse, parse_thumb_pc_literals, Manifest,
    MANIFEST_FILE, THUMB_PC_LITERALS_FILE, THUMB_PC_LITERALS_ID,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::{
        read_audit::{Service, TraceCapture, TraceFilter, TraceReport},
        thumb_pc_literals::SOURCE_ROLE,
        SourceRange,
    },
    Bus, Machine, StepEvent,
};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

const BASE: u32 = 0x08000000;
const INSTALL_END: u32 = BASE + 0x3710;

#[derive(Deserialize)]
struct Plan {
    words: Vec<Word>,
    registry_lookup_offset: u32,
}

#[derive(Deserialize)]
struct Word {
    offset: u32,
    reader: u32,
    opcode: u16,
    value: u32,
    sha256: String,
}

fn input(name: &str) -> PathBuf {
    std::env::var_os(name).unwrap().into()
}

fn run_startup(
    machine: &mut Machine,
    end_pc: u32,
    frame_limit: u32,
    ranges: Vec<(u32, u32)>,
) -> TraceReport {
    let capture = TraceCapture::start(
        TraceFilter {
            readers: Vec::new(),
            ranges,
        },
        256,
    )
    .unwrap();
    reach_startup_pc(machine, end_pc, frame_limit, Some(&capture));
    capture.finish()
}

fn reach_startup_pc(
    machine: &mut Machine,
    end_pc: u32,
    frame_limit: u32,
    capture: Option<&TraceCapture>,
) {
    reach_guest_pc(machine, end_pc, frame_limit, capture, crate::demo_keys);
}

fn reach_guest_pc(
    machine: &mut Machine,
    end_pc: u32,
    frame_limit: u32,
    capture: Option<&TraceCapture>,
    keys_at: impl Fn(u64) -> u16,
) {
    for _ in 0..frame_limit {
        if let Some(capture) = capture {
            capture.frame(Some(machine.bus.frames));
        }
        machine.bus.keys = keys_at(machine.bus.frames);
        machine.bus.frame_ready = false;
        for _ in 0..5_000_000 {
            if machine.bus.frame_ready {
                break;
            }
            if matches!(machine.step(), StepEvent::Instr(i) if i.addr == end_pc) {
                return;
            }
        }
        assert!(machine.bus.frame_ready, "Startup frame did not finish");
    }
    panic!("Startup did not reach {end_pc:08x}");
}

fn run_installer(machine: &mut Machine) -> TraceReport {
    run_startup(
        machine,
        INSTALL_END,
        180,
        vec![(0x05c0, 0x05c4), (0x3738, 0x376c)],
    )
}

fn check_reads(trace: &TraceReport, words: &[Word], service: Service) {
    assert!(!trace.truncated);
    assert_eq!(
        trace.matched_requests,
        words.len() as u64,
        "{:?}",
        trace.events
    );
    for word in words {
        assert!(
            trace.events.iter().any(|event| {
                event.key.service == service
                    && event.key.read.physical_offset == word.offset
                    && event.key.read.length == 4
                    && event
                        .key
                        .instruction
                        .is_some_and(|site| site.thumb && site.pc == BASE + word.reader)
                    && event.requests == 1
            }),
            "Expected {service:?} startup load at {:08x}",
            BASE + word.reader
        );
    }
}

#[test]
#[ignore = "exact EUR ROM, E528 source plan and E525 predecessor manifest"]
fn european_startup_task_table_publication_and_installation() {
    let rom_path = input("FFTA_STATE_TEST_ROM");
    let rom = fs::read(&rom_path).unwrap();
    let work = input("FFTA_STARTUP_TASK_WORK");
    let plan: Plan =
        serde_json::from_slice(&fs::read(input("FFTA_STARTUP_TASK_PLAN")).unwrap()).unwrap();
    let prior_bytes = fs::read(input("FFTA_STARTUP_TASK_PREDECESSOR")).unwrap();
    let prior: Manifest = parse(&prior_bytes, "E525 manifest").unwrap();
    assert_eq!(
        (prior.extractor_revision, prior.decoder_revision),
        (323, 322)
    );
    assert_eq!(digest(&rom), EUROPE_SHA256);
    assert_eq!(plan.words.len(), 14);

    let resources = tempfile::tempdir_in(&work).unwrap();
    cmd(&[
        "prepare".into(),
        rom_path.to_str().unwrap().into(),
        resources.path().to_str().unwrap().into(),
    ])
    .unwrap();
    let revision = resources.path().join(EUROPE_SHA256).join("v1/e324-d323");
    let current = fs::read_to_string(revision.join("current")).unwrap();
    let dataset = revision.join(current.trim());
    let manifest_path = dataset.join(MANIFEST_FILE);
    let manifest_bytes = fs::read(&manifest_path).unwrap();
    let manifest: Manifest = parse(&manifest_bytes, "E528 manifest").unwrap();
    let index = prior
        .resources
        .iter()
        .position(|entry| entry.id == THUMB_PC_LITERALS_ID)
        .unwrap();
    let previous_entry = &prior.resources[index];
    assert!(previous_entry
        .sources
        .iter()
        .any(|s| s.offset == plan.registry_lookup_offset && s.length == 4));
    let literal_path = dataset.join(THUMB_PC_LITERALS_FILE);
    let literal_bytes = fs::read(&literal_path).unwrap();
    let mut expected = prior.clone();
    expected.extractor_revision = 324;
    expected.decoder_revision = 323;
    expected.resources[index].sha256 = digest(&literal_bytes);
    for word in &plan.words {
        let start = word.offset as usize;
        assert_eq!(rom[start..start + 4], word.value.to_le_bytes());
        assert_eq!(digest(&rom[start..start + 4]), word.sha256);
        expected.resources[index].sources.push(SourceRange {
            offset: word.offset,
            length: 4,
            sha256: word.sha256.clone(),
            role: SOURCE_ROLE.into(),
        });
    }
    assert_eq!(manifest, expected);

    let mut without_startup = parse_thumb_pc_literals(&literal_bytes).unwrap();
    without_startup.startup_task_roots.clear();
    let previous_packet = encode_thumb_pc_literals(&without_startup).unwrap();
    assert_eq!(digest(&previous_packet), previous_entry.sha256);
    let mut managed = crate::make_machine(rom.clone(), None);
    fs::write(&literal_path, &previous_packet).unwrap();
    fs::write(&manifest_path, &prior_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let previous_owner = managed.bus.rom_resources().unwrap().clone();
    let mut missing_group = prior;
    missing_group.extractor_revision = 324;
    missing_group.decoder_revision = 323;
    fs::write(
        &manifest_path,
        toml::to_string_pretty(&missing_group).unwrap(),
    )
    .unwrap();
    assert!(attach_from_directory(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(
        &previous_owner,
        managed.bus.rom_resources().unwrap()
    ));
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    fs::write(&literal_path, &literal_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    drop(previous_owner);

    let mut raw = crate::make_machine(rom.clone(), None);
    for word in &plan.words {
        let start = word.offset as usize;
        managed.bus.rom[start..start + 4].copy_from_slice(&(!word.value).to_le_bytes());
    }
    check_reads(&run_installer(&mut raw), &plan.words, Service::Raw);
    check_reads(&run_installer(&mut managed), &plan.words, Service::Typed);
    for machine in [&raw, &managed] {
        assert_eq!(machine.bus.iwram[0x830..0x860], rom[0x373c..0x376c]);
    }
    for word in &plan.words {
        let start = word.offset as usize;
        managed.bus.rom[start..start + 4].copy_from_slice(&word.value.to_le_bytes());
    }
    assert_eq!(raw.bus.frames, managed.bus.frames);
    assert_eq!(raw.bus.clock, managed.bus.clock);
    assert_eq!(raw.bus.audio_buf, managed.bus.audio_buf);
    assert_eq!(
        digest(&raw.save_state().unwrap()),
        digest(&managed.save_state().unwrap())
    );

    let lookup = plan.registry_lookup_offset as usize;
    managed.bus.rom[lookup..lookup + 4].fill(0xff);
    assert_eq!(
        managed.bus.read32(BASE + plan.registry_lookup_offset),
        0x03000830
    );

    let report = serde_json::json!({
        "status": "PASS", "revision": [324, 323], "slots": 12, "source_bytes": 56,
        "raw_reads": plan.words.len(), "typed_reads": plan.words.len(),
        "install_pc": INSTALL_END, "install_frame": raw.bus.frames,
        "predecessor_attach": true, "missing_group_rejected": true,
        "prior_lookup_preserved": true, "full_state_and_audio_equal": true,
        "manifest_sha256": digest(&manifest_bytes), "literal_file_sha256": digest(&literal_bytes),
    });
    fs::write(work.join("manifest.toml"), manifest_bytes).unwrap();
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}

#[derive(Deserialize)]
struct DispatcherPlan {
    words: Vec<Word>,
    owned: Vec<Word>,
}

fn run_literal_loads(
    machine: &mut Machine,
    words: &[Word],
    ranges: Vec<(u32, u32)>,
) -> TraceReport {
    let capture = TraceCapture::start(
        TraceFilter {
            readers: Vec::new(),
            ranges,
        },
        words.len(),
    )
    .unwrap();
    capture.frame(Some(machine.bus.frames));
    for word in words {
        let pc = BASE + word.reader;
        machine.cpu.regs[15] = pc;
        machine.cpu.cpsr = 0xbf;
        machine.cpu.branch = None;
        assert!(matches!(machine.step(), StepEvent::Instr(i) if i.addr == pc));
        let destination = ((word.opcode >> 8) & 7) as usize;
        assert_eq!(
            machine.cpu.regs[destination], word.value,
            "Load at {pc:08x}"
        );
        assert_eq!(machine.cpu.regs[15], pc + 2);
    }
    capture.finish()
}

fn checked_trace(
    trace: &TraceReport,
    readers: &BTreeSet<(u32, u32)>,
    service: Service,
) -> BTreeMap<(u32, u32), (u64, u64, u64)> {
    assert!(!trace.truncated);
    let mut reads = BTreeMap::new();
    for event in &trace.events {
        assert_eq!(event.key.service, service, "{event:?}");
        assert_eq!(event.key.read.length, 4, "{event:?}");
        let site = event
            .key
            .instruction
            .expect("Original callback instruction");
        assert!(site.thumb);
        let offset = event.key.read.physical_offset;
        assert!(
            readers.contains(&(offset, site.pc)),
            "Unclassified startup read {event:?}"
        );
        let frames = (
            event.requests,
            event.first_guest_frame,
            event.last_guest_frame,
        );
        assert!(reads.insert((offset, site.pc), frames).is_none());
    }
    reads
}

fn check_source_word(rom: &[u8], word: &Word) {
    let at = word.offset as usize;
    assert_eq!(rom[at..at + 4], word.value.to_le_bytes());
    assert_eq!(digest(&rom[at..at + 4]), word.sha256);
    let reader = word.reader as usize;
    assert_eq!(rom[reader..reader + 2], word.opcode.to_le_bytes());
}

fn flip_backing(machine: &mut Machine, words: &[Word]) {
    for word in words {
        let at = word.offset as usize;
        for byte in &mut machine.bus.rom[at..at + 4] {
            *byte ^= 0xff;
        }
    }
}

fn prepare_startup_extension(
    rom: &[u8],
    rom_path: &Path,
    work: &Path,
    prior: &Manifest,
    prior_bytes: &[u8],
    words: &[Word],
) -> (Machine, Vec<u8>, String) {
    let resources = tempfile::tempdir_in(work).unwrap();
    cmd(&[
        "prepare".into(),
        rom_path.to_str().unwrap().into(),
        resources.path().to_str().unwrap().into(),
    ])
    .unwrap();
    let extractor = prior.extractor_revision + 1;
    let decoder = prior.decoder_revision + 1;
    let revision = resources
        .path()
        .join(EUROPE_SHA256)
        .join(format!("v1/e{extractor}-d{decoder}"));
    let current = fs::read_to_string(revision.join("current")).unwrap();
    let dataset = revision.join(current.trim());
    let manifest_path = dataset.join(MANIFEST_FILE);
    let literal_path = dataset.join(THUMB_PC_LITERALS_FILE);
    let manifest_bytes = fs::read(&manifest_path).unwrap();
    let literal_bytes = fs::read(&literal_path).unwrap();
    let literal_sha256 = digest(&literal_bytes);
    let manifest: Manifest = parse(&manifest_bytes, "extended startup manifest").unwrap();
    let index = prior
        .resources
        .iter()
        .position(|resource| resource.id == THUMB_PC_LITERALS_ID)
        .unwrap();
    let mut expected = prior.clone();
    expected.extractor_revision = extractor;
    expected.decoder_revision = decoder;
    expected.resources[index].sha256 = literal_sha256.clone();
    for word in words {
        check_source_word(rom, word);
        expected.resources[index].sources.push(SourceRange {
            offset: word.offset,
            length: 4,
            role: SOURCE_ROLE.into(),
            sha256: word.sha256.clone(),
        });
    }
    assert_eq!(manifest, expected);

    let mut preceding_literals = parse_thumb_pc_literals(&literal_bytes).unwrap();
    let prior_count = preceding_literals.startup_task_roots.len() - words.len();
    preceding_literals.startup_task_roots.truncate(prior_count);
    let preceding_packet = encode_thumb_pc_literals(&preceding_literals).unwrap();
    assert_eq!(digest(&preceding_packet), prior.resources[index].sha256);
    let mut managed = crate::make_machine(rom.to_vec(), None);
    fs::write(&literal_path, preceding_packet).unwrap();
    fs::write(&manifest_path, prior_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let previous_owner = managed.bus.rom_resources().unwrap().clone();
    let mut missing_group = prior.clone();
    missing_group.extractor_revision = extractor;
    missing_group.decoder_revision = decoder;
    fs::write(
        &manifest_path,
        toml::to_string_pretty(&missing_group).unwrap(),
    )
    .unwrap();
    assert!(attach_from_directory(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(
        &previous_owner,
        managed.bus.rom_resources().unwrap()
    ));
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    fs::write(&literal_path, literal_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    (managed, manifest_bytes, literal_sha256)
}

#[test]
#[ignore = "exact EUR ROM, E530 source plan and E528 predecessor manifest"]
fn european_startup_dispatcher_uses_extracted_literal_pools() {
    let rom_path = input("FFTA_STATE_TEST_ROM");
    let rom = fs::read(&rom_path).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let work = input("FFTA_STARTUP_DISPATCH_WORK");
    let plan: DispatcherPlan =
        serde_json::from_slice(&fs::read(input("FFTA_STARTUP_DISPATCH_PLAN")).unwrap()).unwrap();
    let prior_bytes = fs::read(input("FFTA_STARTUP_DISPATCH_PREDECESSOR")).unwrap();
    let prior: Manifest = parse(&prior_bytes, "E528 manifest").unwrap();
    assert_eq!(
        (prior.extractor_revision, prior.decoder_revision),
        (324, 323)
    );
    assert_eq!((plan.words.len(), plan.owned.len()), (39, 19));

    let earlier = prior
        .resources
        .iter()
        .find(|r| r.id == THUMB_PC_LITERALS_ID)
        .unwrap();
    for word in &plan.owned {
        check_source_word(&rom, word);
        assert!(earlier.sources.iter().any(|source| {
            source.offset == word.offset && source.length == 4 && source.sha256 == word.sha256
        }));
    }
    let (mut managed, manifest_bytes, literal_sha256) =
        prepare_startup_extension(&rom, &rom_path, &work, &prior, &prior_bytes, &plan.words);
    flip_backing(&mut managed, &plan.words);
    let mut raw = crate::make_machine(rom, None);
    let reach_title =
        |machine: &mut Machine| run_startup(machine, 0x0813_9508, 1200, vec![(0x376c, 0x3a6c)]);
    let raw_trace = reach_title(&mut raw);
    let managed_trace = reach_title(&mut managed);
    let readers = plan
        .words
        .iter()
        .chain(&plan.owned)
        .map(|word| (word.offset, BASE + word.reader))
        .collect();
    let raw_reads = checked_trace(&raw_trace, &readers, Service::Raw);
    let managed_reads = checked_trace(&managed_trace, &readers, Service::Typed);
    assert_eq!(raw_reads, managed_reads);
    for (source, reader) in [(0x376c, 0x3712), (0x38a0, 0x3892), (0x3948, 0x3936)] {
        assert!(managed_reads.contains_key(&(source, BASE + reader)));
    }
    for machine in [&raw, &managed] {
        assert_eq!(machine.bus.iwram[0x34d4..0x34d6], [5, 0]);
    }
    assert_eq!(raw.bus.frames, managed.bus.frames);
    assert_eq!(raw.bus.clock, managed.bus.clock);
    assert_eq!(raw.bus.audio_buf, managed.bus.audio_buf);
    let title_frame = raw.bus.frames;
    // Restore the original ROM identity while comparing the states at title entry.
    flip_backing(&mut managed, &plan.words);
    assert_eq!(
        digest(&raw.save_state().unwrap()),
        digest(&managed.save_state().unwrap())
    );
    flip_backing(&mut managed, &plan.words);

    check_reads(
        &run_literal_loads(&mut raw, &plan.words, vec![(0x376c, 0x3a6c)]),
        &plan.words,
        Service::Raw,
    );
    check_reads(
        &run_literal_loads(&mut managed, &plan.words, vec![(0x376c, 0x3a6c)]),
        &plan.words,
        Service::Typed,
    );
    let ordinary_new_roots = plan
        .words
        .iter()
        .filter(|word| managed_reads.contains_key(&(word.offset, BASE + word.reader)))
        .count();
    let report = serde_json::json!({
        "status": "PASS", "revision": [325, 324], "new_words": 39, "old_words": 19,
        "source_bytes": 156, "title_slot": 5, "title_frame": title_frame,
        "ordinary_new_roots": ordinary_new_roots, "ordinary_read_sites": managed_reads.len(),
        "ordinary_typed_requests": managed_trace.matched_requests,
        "controlled_reads": plan.words.len(), "full_state_and_audio_equal": true,
        "predecessor_attach": true, "missing_group_rejected": true,
        "manifest_sha256": digest(&manifest_bytes), "literal_file_sha256": literal_sha256,
    });
    fs::write(work.join("manifest.toml"), manifest_bytes).unwrap();
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}

#[derive(Deserialize)]
struct TitlePlan {
    words: Vec<Word>,
    readers: Vec<Word>,
}

#[test]
#[ignore = "exact EUR ROM, E531 source plan and E530 predecessor manifest"]
fn european_title_callbacks_use_extracted_literal_pools() {
    const PARENT: u32 = 0x0813_9508;
    const CHILD: u32 = 0x0813_9d29;
    const LOADER: u32 = 0x0813_99e4;
    let rom_path = input("FFTA_STATE_TEST_ROM");
    let rom = fs::read(&rom_path).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let work = input("FFTA_TITLE_TASK_WORK");
    let plan: TitlePlan =
        serde_json::from_slice(&fs::read(input("FFTA_TITLE_TASK_PLAN")).unwrap()).unwrap();
    let prior_bytes = fs::read(input("FFTA_TITLE_TASK_PREDECESSOR")).unwrap();
    let prior: Manifest = parse(&prior_bytes, "E530 manifest").unwrap();
    assert_eq!(
        (prior.extractor_revision, prior.decoder_revision),
        (325, 324)
    );
    assert_eq!((plan.words.len(), plan.readers.len()), (69, 71));
    for word in &plan.readers {
        check_source_word(&rom, word);
    }
    let readers = plan
        .readers
        .iter()
        .map(|word| (word.offset, BASE + word.reader))
        .collect();
    let (mut managed, manifest_bytes, literal_sha256) =
        prepare_startup_extension(&rom, &rom_path, &work, &prior, &prior_bytes, &plan.words);

    let mut raw = crate::make_machine(rom, None);
    reach_startup_pc(&mut raw, PARENT, 1200, None);
    assert_eq!(raw.bus.iwram[0x34d4..0x34d6], [5, 0]);
    let parent_frame = raw.bus.frames;
    let origin = raw.save_state().unwrap();
    raw.load_state(&origin).unwrap();
    managed.load_state(&origin).unwrap();
    flip_backing(&mut managed, &plan.words);

    let ranges = vec![(0x139508, 0x1395fc), (0x139d28, 0x13a1dc)];
    let reach_loader = |machine: &mut Machine| run_startup(machine, LOADER, 120, ranges.clone());
    let raw_trace = reach_loader(&mut raw);
    let managed_trace = reach_loader(&mut managed);
    let raw_reads = checked_trace(&raw_trace, &readers, Service::Raw);
    let managed_reads = checked_trace(&managed_trace, &readers, Service::Typed);
    assert_eq!(raw_reads, managed_reads);
    for (source, reader) in [
        (0x13955c, 0x139540),
        (0x139dbc, 0x139d8e),
        (0x13a1cc, 0x13a1ae),
    ] {
        assert!(managed_reads.contains_key(&(source, BASE + reader)));
    }
    for machine in [&raw, &managed] {
        assert_eq!(machine.bus.ewram[0xf3bc..0xf3c0], CHILD.to_le_bytes());
    }
    assert_eq!(raw.bus.frames, managed.bus.frames);
    assert_eq!(raw.bus.clock, managed.bus.clock);
    assert_eq!(raw.bus.audio_buf, managed.bus.audio_buf);
    let loader_frame = raw.bus.frames;
    flip_backing(&mut managed, &plan.words);
    assert_eq!(
        digest(&raw.save_state().unwrap()),
        digest(&managed.save_state().unwrap())
    );
    flip_backing(&mut managed, &plan.words);

    check_reads(
        &run_literal_loads(&mut raw, &plan.readers, ranges.clone()),
        &plan.readers,
        Service::Raw,
    );
    check_reads(
        &run_literal_loads(&mut managed, &plan.readers, ranges),
        &plan.readers,
        Service::Typed,
    );
    let report = serde_json::json!({
        "status": "PASS", "revision": [326, 325], "new_words": 69, "source_bytes": 276,
        "parent_pc": PARENT, "parent_frame": parent_frame, "registered_child": CHILD,
        "loader_pc": LOADER, "loader_frame": loader_frame,
        "ordinary_read_sites": managed_reads.len(), "ordinary_typed_requests": managed_trace.matched_requests,
        "controlled_readers": plan.readers.len(), "full_state_and_audio_equal": true,
        "predecessor_attach": true, "missing_group_rejected": true,
        "manifest_sha256": digest(&manifest_bytes), "literal_file_sha256": literal_sha256,
    });
    fs::write(work.join("manifest.toml"), manifest_bytes).unwrap();
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}
