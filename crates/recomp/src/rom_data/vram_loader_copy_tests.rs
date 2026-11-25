//! Check both original VRAM loaders with altered backing for their full sources.
use super::{
    attach_from_directory, cmd, encode_ui_raw_transfers, parse, parse_ui_raw_transfers, Extraction,
    Manifest, MANIFEST_FILE, UI_RAW_TRANSFERS_FILE, UI_RAW_TRANSFERS_ID,
};
use gba_core::{ffta_rom::digest, rom_data::SourceRange, Bus, Machine, StepEvent};
use serde::Deserialize;
use std::{fs, path::PathBuf, sync::Arc};

const ROM_BASE: u32 = 0x0800_0000;
const VRAM_BASE: u32 = 0x0600_0000;
const CPU_SET: u32 = 0x0814_5230;
const SEQUENCES: [(u32, usize); 2] = [(0x1399f2, 2), (0x139b3a, 4)];
const DESTINATIONS: [u32; 5] = [0x06000000, 0x06008000, 0x06010000, 0x06010000, 0x06012c00];

#[derive(Deserialize)]
struct Plan {
    sites: [u32; 5],
    extents: [Extent; 5],
    literals: Vec<Literal>,
    gap: Extent,
    owner_overlap: Extent,
}

#[derive(Deserialize)]
struct Extent {
    offset: u32,
    end: u32,
    sha256: String,
    unowned: Vec<[u32; 2]>,
}

#[derive(Deserialize)]
struct Literal {
    offset: u32,
    value: u32,
    sha256: String,
}

fn input(name: &str) -> PathBuf {
    std::env::var_os(name).unwrap().into()
}

fn expected_spans(rom: &[u8], plan: &Plan) -> Vec<SourceRange> {
    let mut spans = Vec::new();
    for (start, last) in SEQUENCES {
        for pc in (start..plan.sites[last]).step_by(2) {
            let at = pc as usize;
            let instruction = u16::from_le_bytes(rom[at..at + 2].try_into().unwrap());
            if instruction & 0xf800 == 0x4800 {
                let offset = ((pc + 4) & !3) + 4 * u32::from(instruction & 0xff);
                let witness = plan
                    .literals
                    .iter()
                    .find(|row| row.offset == offset)
                    .unwrap();
                let at = offset as usize;
                let bytes = &rom[at..at + 4];
                assert_eq!(u32::from_le_bytes(bytes.try_into().unwrap()), witness.value);
                assert_eq!(digest(bytes), witness.sha256);
                spans.push(SourceRange {
                    offset,
                    length: 4,
                    sha256: witness.sha256.clone(),
                    role: "vram-loader-literal".into(),
                });
            }
        }
    }
    assert_eq!(spans.len(), 11);
    let mut separate: Vec<_> = plan
        .extents
        .iter()
        .flat_map(|span| &span.unowned)
        .copied()
        .collect();
    separate.sort_unstable();
    let mut merged: Vec<[u32; 2]> = Vec::new();
    for [start, end] in separate {
        if let Some(prior) = merged.last_mut().filter(|extent| extent[1] >= start) {
            prior[1] = prior[1].max(end);
        } else {
            merged.push([start, end]);
        }
    }
    assert_eq!(merged.len(), 2);
    assert_eq!(
        (merged[0][1], merged[1][0]),
        (plan.gap.offset, plan.gap.end)
    );
    assert_eq!(merged[1][1], plan.owner_overlap.offset);
    spans.extend(merged.into_iter().map(|[offset, end]| SourceRange {
        offset,
        length: end - offset,
        sha256: digest(&rom[offset as usize..end as usize]),
        role: "vram-loader-source".into(),
    }));
    spans.sort_unstable_by_key(|row| row.offset);
    assert_eq!(spans.iter().map(|s| s.length).sum::<u32>(), 66_784);
    spans
}

fn run_original(machine: &mut Machine, rom: &[u8], plan: &Plan) -> usize {
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs[13] = 0x0300_7c00;
    let mut calls = 0;
    let mut copied = 0;
    for (entry, last) in SEQUENCES {
        machine.cpu.regs[15] = ROM_BASE + entry;
        machine.cpu.cpsr = 0xbf;
        machine.cpu.branch = None;
        let end = ROM_BASE + plan.sites[last] + 4;
        for _ in 0..80 {
            let pc = machine.cpu.regs[15];
            if pc == end {
                break;
            }
            let args = [
                machine.cpu.regs[0],
                machine.cpu.regs[1],
                machine.cpu.regs[2],
            ];
            let link = machine.cpu.regs[14];
            if let StepEvent::Instr(instr) = machine.step() {
                if matches!(instr.op, armv4t::Op::Swi { imm: 0x0b }) {
                    let extent = &plan.extents[calls];
                    let length = extent.end - extent.offset;
                    let address = DESTINATIONS[calls];
                    assert!(instr.thumb);
                    assert_eq!(pc, CPU_SET);
                    assert_eq!(link, ROM_BASE + plan.sites[calls] + 5);
                    assert_eq!(args, [ROM_BASE + extent.offset, address, length / 2]);
                    let destination = (address - VRAM_BASE) as usize;
                    let expected = &rom[extent.offset as usize..extent.end as usize];
                    assert_eq!(digest(expected), extent.sha256);
                    assert_eq!(
                        &machine.bus.vram[destination..destination + length as usize],
                        expected
                    );
                    copied += length as usize;
                    calls += 1;
                }
            }
        }
        assert_eq!(machine.cpu.regs[15], end);
        assert_eq!(calls, last + 1);
    }
    copied
}

#[test]
#[ignore = "exact EUR ROM, E517 source plan and e319-d318 predecessor manifest"]
fn european_vram_loader_bank_publication_and_original_copies() {
    let work = input("FFTA_VRAM_LOADER_WORK");
    let rom_path = input("FFTA_STATE_TEST_ROM");
    let rom = fs::read(&rom_path).unwrap();
    let plan: Plan =
        serde_json::from_slice(&fs::read(input("FFTA_VRAM_LOADER_PLAN")).unwrap()).unwrap();
    let prior_bytes = fs::read(input("FFTA_VRAM_LOADER_PREDECESSOR")).unwrap();
    let prior: Manifest = parse(&prior_bytes, "E515 manifest").unwrap();
    assert_eq!(
        (prior.extractor_revision, prior.decoder_revision),
        (319, 318)
    );
    let added = expected_spans(&rom, &plan);

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
        .join(&prior.source_rom_sha256)
        .join("v1/e320-d319");
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
    let manifest: Manifest = parse(&manifest_bytes, "E518 manifest").unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (320, 319)
    );
    let index = manifest
        .resources
        .iter()
        .position(|r| r.id == UI_RAW_TRANSFERS_ID)
        .unwrap();
    let old = &prior.resources[index];
    let new = &manifest.resources[index];
    let mut previous_layout = manifest.clone();
    previous_layout.resources[index] = old.clone();
    let missing = previous_layout.clone();
    previous_layout.extractor_revision = prior.extractor_revision;
    previous_layout.decoder_revision = prior.decoder_revision;
    assert_eq!(previous_layout, prior);
    assert_eq!(
        (&new.id, &new.format, &new.path, &new.dependencies),
        (&old.id, &old.format, &old.path, &old.dependencies)
    );
    let mut expected_sources = old.sources.clone();
    for source in &added {
        assert!(prior.resources.iter().flat_map(|r| &r.sources).all(|span| {
            span.offset + span.length <= source.offset
                || source.offset + source.length <= span.offset
        }));
        expected_sources.push(source.clone());
    }
    expected_sources.sort_unstable_by_key(|row| row.offset);
    assert_eq!(new.sources, expected_sources);

    let packet_path = dataset.join(UI_RAW_TRANSFERS_FILE);
    let packet_bytes = fs::read(&packet_path).unwrap();
    let mut packet = parse_ui_raw_transfers(&packet_bytes).unwrap();
    assert!(packet
        .halfword_copies
        .as_mut()
        .unwrap()
        .vram_loaders
        .take()
        .is_some());
    let previous_packet = Extraction::from_rom_for_manifest(&rom, &prior)
        .unwrap()
        .ui_raw_transfers
        .unwrap()
        .decoded;
    assert_eq!(packet, previous_packet);
    let previous_packet = encode_ui_raw_transfers(&previous_packet).unwrap();
    assert_eq!(digest(&previous_packet), old.sha256);
    let mut managed = Machine::new(rom.clone());
    fs::write(&manifest_path, prior_bytes).unwrap();
    fs::write(&packet_path, &previous_packet).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let first = plan.extents[0].offset as usize;
    managed.bus.rom[first] ^= 0xff;
    assert_eq!(managed.bus.read8(ROM_BASE + first as u32), !rom[first]);
    managed.bus.rom[first] ^= 0xff;
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    fs::write(&packet_path, &packet_bytes).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    fs::write(&manifest_path, toml::to_string_pretty(&missing).unwrap()).unwrap();
    fs::write(&packet_path, previous_packet).unwrap();
    assert!(attach_from_directory(&mut managed, &dataset).is_err());
    assert!(Arc::ptr_eq(&owner, managed.bus.rom_resources().unwrap()));
    fs::write(&manifest_path, &manifest_bytes).unwrap();
    fs::write(&packet_path, packet_bytes).unwrap();

    let tail = &plan.owner_overlap;
    let poison = |machine: &mut Machine| {
        for (offset, end) in added
            .iter()
            .map(|r| (r.offset, r.offset + r.length))
            .chain(std::iter::once((tail.offset, tail.end)))
        {
            for byte in &mut machine.bus.rom[offset as usize..end as usize] {
                *byte ^= 0xff;
            }
        }
    };
    let mut raw = Machine::new(rom.clone());
    managed.load_state(&raw.save_state().unwrap()).unwrap();
    poison(&mut managed);
    assert_eq!(run_original(&mut raw, &rom, &plan), 74_972);
    assert_eq!(run_original(&mut managed, &rom, &plan), 74_972);
    let stats = owner.ui_raw_transfers_stats().unwrap();
    assert_eq!(
        (
            stats.source_bytes,
            stats.payload_bytes,
            stats.selector_bytes
        ),
        (74_760, 74_716, 44)
    );
    assert_eq!(stats.banks_used, 2);
    let ui_bytes = owner.table_graphics_stats().unwrap().source_bytes;
    assert_eq!(ui_bytes, 256);
    poison(&mut managed);
    assert_eq!(raw.bus.clock, managed.bus.clock);
    assert_eq!(
        digest(&raw.save_state().unwrap()),
        digest(&managed.save_state().unwrap())
    );
    for offset in [plan.gap.offset, plan.gap.end - 1] {
        managed.bus.rom[offset as usize] ^= 0xff;
        assert_eq!(managed.bus.read8(ROM_BASE + offset), !rom[offset as usize]);
    }

    let result = serde_json::json!({
        "status": "PASS", "revision": [320, 319], "spans": added.len(), "new_bytes": 66784,
        "copied_bytes": 74972, "original_calls": plan.sites.len(), "loader_bytes": stats.source_bytes,
        "existing_ui_bytes": ui_bytes, "gap_preserved": true, "predecessor_attach": true,
        "missing_loaders_rejected": true, "full_state_equal": true,
        "manifest_sha256": digest(&manifest_bytes), "packet_sha256": new.sha256
    });
    fs::write(work.join("manifest.toml"), manifest_bytes).unwrap();
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
}

#[test]
#[ignore = "exact EUR ROM and E517 source plan for a bounded title startup"]
fn european_title_startup_uses_vram_loader_banks() {
    verify_title_startup(crate::demo_keys, 1200, 3);
}

#[test]
#[ignore = "exact EUR ROM and E517 source plan for the unskipped introduction"]
fn european_title_after_intro_uses_state_one_loader() {
    // The retained E136 cold route confirms the highlighted language at frame 180.
    let confirm_english = |frame| {
        if (180..182).contains(&frame) {
            0x3fe
        } else {
            0x3ff
        }
    };
    verify_title_startup(confirm_english, 7200, 1);
}

#[derive(serde::Serialize)]
struct TitleStartupEvent {
    pc: u32,
    frame: u64,
    skip_intro: u8,
    mode: u8,
    argument: u32,
}

fn verify_title_startup(keys_at: impl Fn(u64) -> u16, frame_limit: u32, expected_state: u8) {
    let work = input("FFTA_VRAM_LOADER_WORK");
    let rom_path = input("FFTA_STATE_TEST_ROM");
    let rom = fs::read(&rom_path).unwrap();
    assert_eq!(digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let plan: Plan =
        serde_json::from_slice(&fs::read(input("FFTA_VRAM_LOADER_PLAN")).unwrap()).unwrap();
    let loader_spans = expected_spans(&rom, &plan);

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut registered_at = None;
    let mut selected = None;
    let mut entry = None;
    let mut startup_events = Vec::new();
    for _ in 0..frame_limit {
        raw.bus.keys = keys_at(raw.bus.frames);
        raw.bus.frame_ready = false;
        for _ in 0..5_000_000 {
            if raw.bus.frame_ready {
                break;
            }
            let pc = raw.cpu.regs[15];
            let state = raw.cpu.regs[0] as u8;
            let handler = raw.cpu.regs[2];
            let StepEvent::Instr(instr) = raw.step() else {
                continue;
            };
            if instr.addr != pc {
                continue;
            }
            if matches!(
                pc,
                0x0813_ff64
                    | 0x0814_0504
                    | 0x0814_1884
                    | 0x0814_18c4
                    | 0x0814_4e5a
                    | 0x0814_486c
                    | 0x0814_05de
                    | 0x0813_9dce
            ) {
                assert!(startup_events.len() < 64, "Startup event limit exceeded");
                let argument = if pc == 0x0813_ff64 {
                    assert_eq!(raw.cpu.regs[4], 0x0200_1f6e);
                    raw.cpu.regs[5]
                } else {
                    raw.cpu.regs[0]
                };
                startup_events.push(TitleStartupEvent {
                    pc,
                    frame: raw.bus.frames,
                    skip_intro: raw.bus.iwram[0x280c],
                    mode: raw.bus.iwram[0x284c],
                    argument,
                });
            }
            match pc {
                0x0813_9546 if handler == 0x0813_9d29 => registered_at = Some(raw.bus.frames),
                0x0813_9dce | 0x0813_9e6c if matches!(state, 1 | 3) => {
                    selected = Some((state, pc, raw.bus.frames));
                }
                0x0813_9dee if raw.cpu.regs[15] == 0x0813_9b30 => entry = Some((1, 3..5)),
                0x0813_9e7e if raw.cpu.regs[15] == 0x0813_99e4 => entry = Some((3, 0..3)),
                _ => {}
            }
            if entry.is_some() {
                break;
            }
        }
        if entry.is_some() {
            break;
        }
        assert!(raw.bus.frame_ready, "Title startup frame did not finish");
    }
    let observation = serde_json::json!({
        "expected_state": expected_state, "frames": raw.bus.frames,
        "registered_at": registered_at, "selected": selected, "events": &startup_events,
    });
    fs::write(
        work.join("startup-observation.json"),
        serde_json::to_vec_pretty(&observation).unwrap(),
    )
    .unwrap();
    let (state, indices) = entry.expect("Expected the ordinary title loader");
    assert_eq!(state, expected_state);
    let registration_frame = registered_at.expect("Expected the ordinary title registration");
    let (selected_state, store, selected_at) = selected.expect("Expected an original state store");
    assert_eq!(selected_state, state);
    let entry_frame = raw.bus.frames;
    assert!(registration_frame <= selected_at && selected_at <= entry_frame);
    let entry_pc = raw.cpu.regs[15];
    let language = raw.bus.ewram[0x1f6e];
    if expected_state == 1 {
        let path = [
            0x0813_ff64,
            0x0814_0504,
            0x0814_18c4,
            0x0814_486c,
            0x0814_05de,
            0x0813_9dce,
        ];
        assert_eq!(
            startup_events
                .iter()
                .map(|event| event.pc)
                .collect::<Vec<_>>(),
            path
        );
        assert_eq!(startup_events[0].argument, 0);
        assert!(startup_events[1..]
            .iter()
            .all(|event| event.skip_intro == 0));
        assert_eq!(startup_events[4].argument, 5);
        assert_ne!(startup_events[5].mode, 2);
        assert_eq!(language, 0);
    }
    let origin = raw.save_state().unwrap();

    let resources = tempfile::tempdir_in(&work).unwrap();
    let prepare = [
        "prepare".into(),
        rom_path.to_str().unwrap().into(),
        resources.path().to_str().unwrap().into(),
    ];
    cmd(&prepare).unwrap();
    let revision = resources
        .path()
        .join(gba_core::ffta_rom::EUROPE_SHA256)
        .join(format!(
            "v1/e{}-d{}",
            gba_core::rom_data::EXTRACTOR_REV,
            gba_core::rom_data::DECODER_REV
        ));
    let current = fs::read_to_string(revision.join("current")).unwrap();
    let dataset = revision.join(current.trim());
    let manifest_bytes = fs::read(dataset.join(MANIFEST_FILE)).unwrap();
    let manifest: Manifest = parse(&manifest_bytes, "title loader dataset").unwrap();
    let transfer = manifest
        .resources
        .iter()
        .find(|r| r.id == UI_RAW_TRANSFERS_ID)
        .unwrap();
    assert!(loader_spans
        .iter()
        .all(|span| transfer.sources.contains(span)));

    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    raw.load_state(&origin).unwrap();
    managed.load_state(&origin).unwrap();
    let tail = &plan.owner_overlap;
    let flip_backing = |m: &mut Machine| {
        let spans = loader_spans.iter().map(|r| (r.offset, r.offset + r.length));
        for (start, end) in spans.chain(std::iter::once((tail.offset, tail.end))) {
            for byte in &mut m.bus.rom[start as usize..end as usize] {
                *byte ^= 0xff;
            }
        }
    };
    flip_backing(&mut managed);
    let mut copied = 0;
    let mut copied_sites = Vec::new();
    let mut next = indices.start;
    for _ in 0..5_000_000 {
        let pc = raw.cpu.regs[15];
        let args = [raw.cpu.regs[0], raw.cpu.regs[1], raw.cpu.regs[2]];
        let link = raw.cpu.regs[14];
        let managed_args = [
            managed.cpu.regs[0],
            managed.cpu.regs[1],
            managed.cpu.regs[2],
        ];
        let managed_link = managed.cpu.regs[14];
        let observed = raw.step();
        let candidate = managed.step();
        if let StepEvent::Instr(instruction) = observed {
            if instruction.addr == CPU_SET
                && matches!(instruction.op, armv4t::Op::Swi { imm: 0x0b })
                && next < indices.end
                && link == ROM_BASE + plan.sites[next] + 5
            {
                assert_eq!(pc, CPU_SET);
                assert!(matches!(candidate, StepEvent::Instr(ref i) if i.addr == CPU_SET));
                assert_eq!((managed_args, managed_link), (args, link));
                let extent = &plan.extents[next];
                let length = extent.end - extent.offset;
                let address = DESTINATIONS[next];
                assert_eq!(args, [ROM_BASE + extent.offset, address, length / 2]);
                let start = (address - VRAM_BASE) as usize;
                let end = start + length as usize;
                let expected = &rom[extent.offset as usize..extent.end as usize];
                assert_eq!(&raw.bus.vram[start..end], expected);
                assert_eq!(&managed.bus.vram[start..end], expected);
                copied += length;
                copied_sites.push(ROM_BASE + plan.sites[next]);
                next += 1;
                if next == indices.end {
                    break;
                }
            }
        }
    }
    assert_eq!(
        next, indices.end,
        "Ordinary title loader did not finish its CPUSet sequence"
    );
    let stats = owner.ui_raw_transfers_stats().unwrap();
    let tail_bytes = owner.table_graphics_stats().unwrap().source_bytes;
    assert!(stats.payload_bytes >= u64::from(copied) - u64::from(tail.end - tail.offset));
    assert!(tail_bytes >= u64::from(tail.end - tail.offset));
    flip_backing(&mut managed);
    assert_eq!(raw.bus.clock, managed.bus.clock);
    assert_eq!(raw.bus.audio_buf, managed.bus.audio_buf);
    assert_eq!(
        digest(&raw.save_state().unwrap()),
        digest(&managed.save_state().unwrap())
    );
    let report = serde_json::json!({
        "status": "PASS", "scope": "ordinary-startup-pair", "state": state,
        "entry_pc": entry_pc, "entry_frame": entry_frame, "registered_at": registered_at,
        "selected_at": selected_at, "store_pc": store, "stored_language": language,
        "copy_sites": copied_sites, "copied_bytes": copied, "managed_payload": stats.payload_bytes,
        "selector_bytes": stats.selector_bytes, "existing_ui_bytes": tail_bytes,
        "startup_events": startup_events,
        "full_state_and_audio_equal": true, "manifest_sha256": digest(&manifest_bytes),
    });
    fs::write(work.join("manifest.toml"), manifest_bytes).unwrap();
    fs::write(
        work.join("test-result.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}
