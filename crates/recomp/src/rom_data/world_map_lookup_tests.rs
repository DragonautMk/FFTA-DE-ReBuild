//! Run the original EUR label loads and calendar transfer from poisoned ROM copies.

use super::{
    attach_dataset, attach_from_directory, cmd, encode_indexed_graphics,
    indexed_reader_bank_tests::{execute_until, initialize},
    parse, parse_indexed_graphics, read_dataset, Manifest, INDEXED_GRAPHICS_FILE, INDEXED_GRAPHICS_ID,
    MANIFEST_FILE,
};
use gba_core::{
    ffta_rom::{digest, EUROPE_SHA256},
    rom_data::{
        indexed_graphics::{
            world_map_lookups::{CALENDAR_TABLE, LABEL_TABLE},
            Archive,
        },
        read_audit::{Service, TraceCapture, TraceFilter},
        SourceRange,
    },
    Bus, Machine,
};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
    sync::Arc,
};

const BASE: u32 = 0x0800_0000;
const STACK: u32 = 0x0300_7c00;
const LABEL_CONTEXT: u32 = 0x0201_0000;
const LABEL_RECORD: u32 = 0x0201_0300;
const LANGUAGE: u32 = 0x0200_1f6e;
const MONTH: u32 = 0x0200_2fba;
const SCRATCH: usize = 0x3cb0;
const VRAM_DESTINATION: usize = 0x10b80;
const COPY_BYTES: usize = 512;
const REQUESTED_PROGRAMS: u32 = 12;
const SCRATCH_BYTES: usize = REQUESTED_PROGRAMS as usize * 64;
type Reads = BTreeMap<(u32, usize, u32), u64>;

#[derive(Deserialize)]
struct Plan {
    readers: [Reader; 3],
    label_ids: [u16; 31],
    calendar_offsets: [u8; 26],
    archive_roots: [u32; 5],
    archive_sources: Vec<SourceRange>,
    spans: Vec<SourceRange>,
}

#[derive(Deserialize)]
struct Reader {
    entry: u32,
    stop: u32,
    literal_reader: u32,
    literal: u32,
    lookup: u32,
    width: usize,
    code_sha256: String,
}

fn input(name: &str) -> PathBuf {
    std::env::var_os(name).unwrap().into()
}

fn record_lookup(expected: &mut Reads, reader: &Reader, offset: u32) {
    *expected
        .entry((reader.literal, 4, BASE + reader.literal_reader))
        .or_default() += 1;
    *expected
        .entry((offset, reader.width, BASE + reader.lookup))
        .or_default() += 1;
}

fn run_lookups(
    machine: &mut Machine,
    plan: &Plan,
    archives: &[&Archive; 5],
    service: Service,
) -> Reads {
    let capture = TraceCapture::start(
        TraceFilter {
            readers: vec![],
            ranges: plan
                .spans
                .iter()
                .map(|span| (span.offset, span.offset + span.length))
                .collect(),
        },
        250,
    )
    .unwrap();
    capture.frame(Some(machine.bus.frames));
    let mut expected = Reads::new();
    let mut labels = plan.label_ids.to_vec();
    labels.push(u16::from_le_bytes(
        plan.calendar_offsets[..2].try_into().unwrap(),
    ));
    let [title, record, calendar] = &plan.readers;
    for (slot, &id) in labels.iter().enumerate() {
        let offset = LABEL_TABLE + 2 * slot as u32;
        initialize(machine, BASE + title.entry);
        machine.cpu.regs[4] = LABEL_CONTEXT;
        machine.bus.write32(LABEL_CONTEXT, LABEL_CONTEXT + 0x100);
        machine
            .bus
            .write32(LABEL_CONTEXT + 0x104, LABEL_CONTEXT + 0x200);
        machine.bus.write8(LABEL_CONTEXT + 0x21b, slot as u8);
        execute_until(machine, BASE + title.stop, 128);
        assert_eq!(machine.cpu.regs[1], u32::from(id), "Label {slot}");
        record_lookup(&mut expected, title, offset);

        // The original record loop looks up slots 1 through 31 when enabled.
        if slot != 0 {
            initialize(machine, BASE + record.entry);
            machine.cpu.regs[4] = LABEL_RECORD;
            machine.bus.write32(STACK, slot as u32);
            machine.bus.write16(LABEL_RECORD + 0xc, !id);
            execute_until(machine, BASE + record.stop, 128);
            assert_eq!(machine.bus.read16(LABEL_RECORD + 0xc), id, "Record {slot}");
            record_lookup(&mut expected, record, offset);
        }
    }
    let scratch = SCRATCH..SCRATCH + SCRATCH_BYTES;
    let destination = VRAM_DESTINATION..VRAM_DESTINATION + COPY_BYTES;
    let seed: [u8; SCRATCH_BYTES] = std::array::from_fn(|i| 0x80 | (i as u8 & 0x7f));
    for (language, archive) in archives.iter().enumerate() {
        for month in 0..=6u8 {
            // ARM signed remainder exposes the source byte preceding a row
            // at month 0. Month 6 reaches the first byte after that row.
            let index = language as i32 * 5 + (i32::from(month) - 1) % 6;
            let selector = if index == -1 {
                plan.label_ids.last().unwrap().to_le_bytes()[1]
            } else {
                plan.calendar_offsets[index as usize]
            };
            let decoded = archive
                .selected_bytes(u32::from(selector >> 1), REQUESTED_PROGRAMS)
                .unwrap();
            let mut expected_scratch = seed;
            expected_scratch[..decoded.len()].copy_from_slice(&decoded);
            let expected_vram = &expected_scratch[..COPY_BYTES];
            initialize(machine, BASE + calendar.entry);
            machine.bus.write8(LANGUAGE, language as u8);
            machine.bus.write8(MONTH, month);
            machine.bus.ewram[scratch.clone()].copy_from_slice(&seed);
            for (byte, &expected) in machine.bus.vram[destination.clone()]
                .iter_mut()
                .zip(expected_vram)
            {
                *byte = !expected;
            }
            execute_until(machine, BASE + calendar.stop, 2_000_000);
            assert_eq!(
                machine.bus.ewram[scratch.clone()],
                expected_scratch,
                "Calendar scratch language {language} month {month}",
            );
            assert_eq!(
                &machine.bus.vram[destination.clone()],
                expected_vram,
                "Calendar VRAM language {language} month {month}",
            );
            record_lookup(
                &mut expected,
                calendar,
                CALENDAR_TABLE.checked_add_signed(index).unwrap(),
            );
        }
    }
    let report = capture.finish();
    assert!(!report.truncated && report.dropped_requests == 0);
    let mut actual = Reads::new();
    for event in report.events {
        let read = event.key.read;
        let site = event.key.instruction.expect("Original map lookup");
        assert!(site.thumb);
        assert_eq!(event.key.service, service, "{event:?}");
        assert!(actual
            .insert((read.physical_offset, read.length, site.pc), event.requests)
            .is_none());
    }
    assert_eq!(actual, expected);
    actual
}

#[test]
#[ignore = "exact EUR ROM, E537 map lookup plan and e331-d330 predecessor manifest"]
fn european_world_map_lookups_use_extracted_tables() {
    let rom_path = input("FFTA_STATE_TEST_ROM");
    let rom = fs::read(&rom_path).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let work = input("FFTA_WORLD_MAP_WORK");
    let plan: Plan =
        serde_json::from_slice(&fs::read(input("FFTA_WORLD_MAP_PLAN")).unwrap()).unwrap();
    assert_eq!(plan.spans.len(), 5);
    let source_bytes = plan.spans.iter().map(|span| span.length).sum::<u32>();
    assert_eq!(source_bytes, 100);
    for span in plan.spans.iter().chain(&plan.archive_sources) {
        let start = span.offset as usize;
        assert_eq!(
            digest(&rom[start..start + span.length as usize]),
            span.sha256
        );
    }
    for reader in &plan.readers {
        assert_eq!(
            digest(&rom[reader.entry as usize..reader.stop as usize]),
            reader.code_sha256
        );
    }
    let previous_bytes = fs::read(input("FFTA_WORLD_MAP_PREDECESSOR")).unwrap();
    let previous: Manifest = parse(&previous_bytes, "E536 manifest").unwrap();
    assert_eq!(
        (previous.extractor_revision, previous.decoder_revision),
        (331, 330)
    );
    let resources = tempfile::tempdir_in(&work).unwrap();
    cmd(&[
        "prepare".into(),
        rom_path.to_str().unwrap().into(),
        resources.path().to_str().unwrap().into(),
    ])
    .unwrap();
    let revision = resources.path().join(EUROPE_SHA256).join("v1/e332-d331");
    let dataset = revision.join(fs::read_to_string(revision.join("current")).unwrap().trim());
    let manifest_path = dataset.join(MANIFEST_FILE);
    let packet_path = dataset.join(INDEXED_GRAPHICS_FILE);
    let manifest_bytes = fs::read(&manifest_path).unwrap();
    let packet_bytes = fs::read(&packet_path).unwrap();
    let manifest: Manifest = parse(&manifest_bytes, "E537 manifest").unwrap();
    let index = previous
        .resources
        .iter()
        .position(|entry| entry.id == INDEXED_GRAPHICS_ID)
        .unwrap();
    let mut expected = previous.clone();
    expected.extractor_revision = 332;
    expected.decoder_revision = 331;
    let resource = &mut expected.resources[index];
    resource.sha256 = digest(&packet_bytes);
    resource.sources.extend(plan.spans.iter().cloned());
    resource.sources.sort_unstable_by_key(|span| span.offset);
    assert_eq!(manifest, expected);

    let mut packet = parse_indexed_graphics(&packet_bytes).unwrap();
    let lookups = packet.world_map_lookups.take().unwrap();
    assert_eq!(lookups.label_ids, plan.label_ids);
    assert_eq!(lookups.calendar_offsets, plan.calendar_offsets);
    let additional = packet.additional.as_ref().unwrap();
    let archives = plan.archive_roots.map(|root| {
        let archive = additional
            .archives
            .iter()
            .find(|archive| archive.source_offset == root)
            .unwrap();
        assert_eq!(
            (archive.programs.len(), archive.fixed_decoded_bytes),
            (30, 64)
        );
        archive
    });

    let old_packet = encode_indexed_graphics(&packet).unwrap();
    assert_eq!(digest(&old_packet), previous.resources[index].sha256);
    let mut managed = Machine::new(rom.clone());
    fs::write(&packet_path, old_packet).unwrap();
    fs::write(&manifest_path, previous_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let mut incomplete = previous;
    incomplete.extractor_revision = 332;
    incomplete.decoder_revision = 331;
    fs::write(&manifest_path, toml::to_string_pretty(&incomplete).unwrap()).unwrap();
    assert!(attach_from_directory(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(&owner, managed.bus.rom_resources().unwrap()));
    fs::write(&packet_path, packet_bytes).unwrap();
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    drop(owner);

    let mut raw = Machine::new(rom);
    let origin = raw.save_state().unwrap();
    raw.load_state(&origin).unwrap();
    managed.load_state(&origin).unwrap();
    let flip_backing = |machine: &mut Machine| {
        for span in plan.spans.iter().chain(&plan.archive_sources) {
            let start = span.offset as usize;
            for byte in &mut machine.bus.rom[start..start + span.length as usize] {
                *byte ^= 0xff;
            }
        }
    };
    flip_backing(&mut managed);
    let reads = run_lookups(&mut raw, &plan, &archives, Service::Raw);
    assert_eq!(
        reads,
        run_lookups(&mut managed, &plan, &archives, Service::Typed)
    );
    let lookup_reads = reads.values().sum::<u64>();
    assert_eq!(lookup_reads, 196);
    let stats = managed
        .bus
        .rom_resources()
        .unwrap()
        .indexed_graphics_stats()
        .unwrap();
    assert_eq!((stats.archives_used, stats.programs_used), (5, 150));
    flip_backing(&mut managed);
    assert_eq!(raw.bus.clock, managed.bus.clock);
    assert_eq!(
        digest(&raw.save_state().unwrap()),
        digest(&managed.save_state().unwrap())
    );

    fs::write(work.join("manifest.toml"), &manifest_bytes).unwrap();
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "status": "PASS", "revision": [332, 331],
            "manifest_sha256": digest(&manifest_bytes),
            "source_bytes": source_bytes, "lookup_reads": lookup_reads,
            "archives_used": stats.archives_used, "programs_used": stats.programs_used,
            "predecessor_attach": true, "missing_group_rejected": true,
            "boundary_aliases_equal": true, "vram_and_full_state_equal": true,
        }))
        .unwrap(),
    )
    .unwrap();
}

struct LookupLoad {
    pc: u32,
    source: u32,
    width: usize,
    address_register: Option<usize>,
}

#[test]
#[ignore = "requires the private EUR ROM, an attached dataset with E537 banks and TinyCC"]
fn european_world_map_lookup_loads_use_generated_bus_reads() {
    use gba_core::{capi::RT_API, rom_data::read_audit::Reader as AuditReader};
    use recomp_core::analyze::{Analysis, Block, View};

    let rom = fs::read(input("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(digest(&rom), EUROPE_SHA256);
    let work = input("FFTA_WORLD_MAP_WORK");
    let dataset = read_dataset(&input("FFTA_STATE_TEST_DATASET")).unwrap();
    let owner = dataset
        .manifest
        .resources
        .iter()
        .find(|resource| resource.id == INDEXED_GRAPHICS_ID)
        .expect("indexed archives in the authenticated dataset");
    let spans: Vec<_> = owner
        .sources
        .iter()
        .filter(|source| {
            matches!(
                source.role.as_str(),
                "world-map-label-ids" | "world-map-calendar-offsets" | "world-map-lookup-literal"
            )
        })
        .map(|source| (source.offset, source.offset + source.length))
        .collect();
    assert_eq!(spans.len(), 5);
    assert_eq!(
        spans.iter().map(|&(start, end)| end - start).sum::<u32>(),
        100
    );

    let literals = [
        (0x033cae, 0x033cdc),
        (0x03eecc, 0x03ef10),
        (0x03719a, 0x037264),
    ];
    let label_loads = [(0x033cba, 1, 0), (0x03eed4, 0, 1)];
    let calendar_pc = 0x0371b2;
    let cases: Vec<_> = literals
        .iter()
        .map(|&(pc, source)| LookupLoad {
            pc,
            source,
            width: 4,
            address_register: None,
        })
        .chain(label_loads.iter().flat_map(|&(pc, register, first_slot)| {
            (first_slot..=31).map(move |slot| LookupLoad {
                pc,
                source: LABEL_TABLE + 2 * slot,
                width: 2,
                address_register: Some(register),
            })
        }))
        .chain(
            (CALENDAR_TABLE - 1..CALENDAR_TABLE + 26).map(|source| LookupLoad {
                pc: calendar_pc,
                source,
                width: 1,
                address_register: Some(0),
            }),
        )
        .collect();
    assert_eq!(cases.len(), 93);

    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let pcs = cases
        .iter()
        .map(|case| BASE + case.pc)
        .collect::<BTreeSet<_>>();
    let blocks = pcs
        .iter()
        .map(|&addr| Block {
            addr,
            thumb: true,
            instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
        })
        .collect();
    let scratch = tempfile::tempdir_in(&work).unwrap();
    let object = scratch.path().join("map-lookup-loads.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();

    let mut raw = Machine::new(rom.clone());
    let mut generated = Machine::new(rom.clone());
    attach_dataset(&mut generated, &dataset).unwrap();
    let initial = raw.save_state().unwrap();
    let mut covered = BTreeSet::new();
    for load in &cases {
        let pc = BASE + load.pc;
        let address = BASE + load.source;
        for machine in [&mut raw, &mut generated] {
            machine.load_state(&initial).unwrap();
            initialize(machine, pc);
            if let Some(register) = load.address_register {
                machine.cpu.regs[register] = address;
            }
        }
        assert!(
            matches!(raw.step(), gba_core::StepEvent::Instr(instruction) if instruction.addr == pc)
        );

        // A lookup must come through the typed view even when its raw bytes
        // differ. Restore the backing before comparing snapshot identity.
        for &(start, end) in &spans {
            for byte in &mut generated.bus.rom[start as usize..end as usize] {
                *byte ^= 0xff;
            }
        }
        let capture = TraceCapture::start(
            TraceFilter {
                readers: vec![AuditReader::Bus],
                ranges: spans.clone(),
            },
            4,
        )
        .unwrap();
        capture.frame(Some(generated.bus.frames));
        let entry = table
            .get(pc | 1)
            .expect("generated single-instruction entry");
        let next = entry(
            &RT_API,
            &mut generated as *mut Machine as *mut std::ffi::c_void,
        );
        assert_eq!(next, (pc + 2) | 1);
        let trace = capture.finish();
        assert_eq!((trace.matched_requests, trace.dropped_requests), (1, 0));
        let event = &trace.events[0];
        assert_eq!(event.key.service, Service::Typed);
        assert_eq!(event.key.read.guest_address, address);
        assert_eq!(event.key.read.physical_offset, load.source);
        assert_eq!(event.key.read.length, load.width);
        assert!(
            event.key.instruction.is_none(),
            "compiled load used interpreter at {pc:08x}"
        );
        covered.extend(load.source..load.source + load.width as u32);
        for &(start, end) in &spans {
            let span = start as usize..end as usize;
            generated.bus.rom[span.clone()].copy_from_slice(&rom[span]);
        }
        assert_eq!(generated.bus.clock, raw.bus.clock, "load at {pc:08x}");
        assert!(
            generated.save_state().unwrap() == raw.save_state().unwrap(),
            "state mismatch at {pc:08x} reading {address:08x}"
        );
    }
    let source_bytes: BTreeSet<_> = spans.iter().flat_map(|&(start, end)| start..end).collect();
    assert_eq!(covered, source_bytes);
    fs::write(
        work.join("native-result.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "status": "PASS",
            "revision": [dataset.manifest.extractor_revision, dataset.manifest.decoder_revision],
            "compiled_load_pcs": pcs,
            "generated_typed_requests": cases.len(),
            "covered_source_bytes": covered.len(),
            "all_backing_spans_altered": true,
            "state_and_guest_clock_equal_each_case": true,
        }))
        .unwrap(),
    )
    .unwrap();
}
