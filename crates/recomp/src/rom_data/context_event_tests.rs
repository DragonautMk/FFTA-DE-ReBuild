//! EUR timed context events and their original ordinary consumer.
use super::*;
use gba_core::{capi::RT_API, rom_data::layered_obj_animations::CONTEXT_EVENT_ROOT, Bus, Machine};

const LENGTH: usize = 80;

#[test]
#[ignore = "exact EUR ROM, E138 predecessor and ordinary startup checkpoint; Wine only"]
fn european_context_timed_events_preserve_and_serve_ordinary() {
    let root = PathBuf::from(std::env::var_os("FFTA_CONTEXT_EVENT_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_CONTEXT_EVENT_PREVIOUS").unwrap());
    let state = fs::read(std::env::var_os("FFTA_CONTEXT_EVENT_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "d8f174e8f886ac0edbbd060b6a0509c4f0c333b8bdd6f8645c536be35df6e541");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (138, 137));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision), (139, 138));
    assert_eq!(source.manifest.coverage, old.coverage);
    assert_eq!(source.manifest.resources.len(), old.resources.len());
    let prepared = preparation::prepare(&root.join("e139-resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("e139-resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    for old_resource in &old.resources {
        let new_resource = source.manifest.resources.iter().find(|r| r.id == old_resource.id).unwrap();
        let old_bytes = fs::read(previous.join(&old_resource.path)).unwrap();
        let new_bytes = fs::read(prepared.directory.join(&new_resource.path)).unwrap();
        assert_eq!(gba_core::ffta_rom::digest(&old_bytes), old_resource.sha256);
        if old_resource.id != LAYERED_OBJ_ANIMATIONS_ID {
            assert_eq!(new_resource, old_resource);
            assert_eq!(new_bytes, old_bytes);
            continue;
        }
        let old_model = parse_layered_obj_animations(&old_bytes).unwrap();
        let mut model = parse_layered_obj_animations(&new_bytes).unwrap();
        assert!(old_model.context_timed_events.is_none());
        let records = model.context_timed_events.take().unwrap();
        assert_eq!(records.len(), 5);
        assert!(records[..4].iter().all(|r| r.tick != 0));
        assert_eq!(records[4].tick, 0);
        for (index, record) in records.iter().enumerate() {
            let at = CONTEXT_EVENT_ROOT as usize + index * 16;
            assert_eq!(record.source_bytes(), rom[at..at + 16]);
        }
        assert_eq!(model, old_model);
        assert_eq!(encode_layered_obj_animations(&model).unwrap(), old_bytes);
        let mut metadata = new_resource.clone();
        metadata.sources.retain(|s| s.role != "context-timed-event-records");
        metadata.sha256 = old_resource.sha256.clone();
        assert_eq!(&metadata, old_resource);
    }
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let at = CONTEXT_EVENT_ROOT as usize;
    let poison = |machine: &mut Machine| machine.bus.rom[at..at + LENGTH].fill(0xcd);
    let restore = |machine: &mut Machine| machine.bus.rom[at..at + LENGTH].copy_from_slice(&rom[at..at + LENGTH]);
    let initial = owner.layered_obj_animations_stats().unwrap();
    poison(&mut managed);
    let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for i in 0..LENGTH { assert_eq!(managed.bus.read8(0x0800_0000 + CONTEXT_EVENT_ROOT + i as u32), rom[at + i]); }
    for edge in [at, at + LENGTH] {
        for delta in [-3isize, -1, 0, 1] {
            let p = edge.wrapping_add_signed(delta);
            assert_eq!((RT_API.read16)(pointer, 0x0a00_0000 + p as u32), u32::from(u16::from_le_bytes(rom[p..p + 2].try_into().unwrap())));
            assert_eq!((RT_API.read32)(pointer, 0x0800_0000 + p as u32), u32::from_le_bytes(rom[p..p + 4].try_into().unwrap()));
        }
    }
    assert!(owner.layered_obj_animations_stats().unwrap().event_bytes - initial.event_bytes >= LENGTH as u64);
    restore(&mut managed);
    let library = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap());
    let (_library, table) = crate::load_native(&library).unwrap();
    let mut observations = Vec::new();
    for native in [false, true] {
        let mut raw = crate::make_machine(rom.clone(), None);
        raw.load_state(&state).unwrap();
        managed.load_state(&state).unwrap();
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
        assert_eq!((raw.cpu.regs[15], raw.cpu.regs[1]), (0x0814_2ae6, 0x083a_dfe8));
        let before = owner.layered_obj_animations_stats().unwrap();
        let mut native_blocks = 0u64;
        let mut native_reader_entry = false;
        poison(&mut managed);
        for frame in 0..3 {
            if native {
                let raw_ptr = &mut raw as *mut Machine as *mut std::ffi::c_void;
                let managed_ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
                crate::run_frame_native(&mut raw, &table, raw_ptr, crate::FRAME_STEP_GUARD);
                let (count, _) = crate::run_frame_native_traced(&mut managed, &table, managed_ptr, crate::FRAME_STEP_GUARD,
                    |machine, kind| {
                        if kind == crate::NativeDispatch::Native && (0x0814_2acc..0x0814_2b48).contains(&machine.cpu.regs[15]) { native_reader_entry = true; }
                    }, |_, _, _| {});
                native_blocks += count;
            } else {
                raw.bus.frame_ready = false;
                managed.bus.frame_ready = false;
                for _ in 0..5_000_000 {
                    assert_eq!(raw.bus.frame_ready, managed.bus.frame_ready);
                    if raw.bus.frame_ready { break; }
                    assert_eq!((raw.cpu.regs[15], raw.cpu.thumb()), (managed.cpu.regs[15], managed.cpu.thumb()));
                    let event = raw.step();
                    assert_eq!(matches!(event, gba_core::StepEvent::Instr(_)), matches!(managed.step(), gba_core::StepEvent::Instr(_)));
                }
            }
            assert!(raw.bus.frame_ready && managed.bus.frame_ready, "frame {frame} exceeded step limit");
            restore(&mut managed);
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "ordinary event state and clock, native={native}, frame={frame}");
            poison(&mut managed);
        }
        let used = owner.layered_obj_animations_stats().unwrap().event_bytes - before.event_bytes;
        assert!(used >= 16);
        if native { assert!(native_blocks > 0); }
        restore(&mut managed);
        observations.push((used, native_reader_entry));
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[139,138], "records":4, "terminator":true, "source_bytes":80,
        "previous_projection_preserved":true, "ordinary_pairs":true, "full_state_clock_frames_each":3,
        "source_rom_poisoned":true, "ordinary_event_bytes_and_native_reader_entry":observations,
        "other_rows_ordinary":"pending", "CLI":"pending", "other_paths":"pending",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "ordinary EUR startup checkpoint, E139 dataset and translation; Wine only"]
fn european_context_timed_events_all_ordinary_rows() {
    let root = PathBuf::from(std::env::var_os("FFTA_CONTEXT_EVENT_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_CONTEXT_EVENT_DATASET").unwrap());
    let state = fs::read(std::env::var_os("FFTA_CONTEXT_EVENT_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "d8f174e8f886ac0edbbd060b6a0509c4f0c333b8bdd6f8645c536be35df6e541");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let library = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap());
    let (_library, table) = crate::load_native(&library).unwrap();
    let at = CONTEXT_EVENT_ROOT as usize;
    let mut seen = [false; 5];
    let mut payloads = Vec::new();
    for native in [false, true] {
        let mut raw = crate::make_machine(rom.clone(), None);
        let mut managed = crate::make_machine(rom.clone(), None);
        attach_from_directory(&mut managed, &dataset).unwrap();
        raw.load_state(&state).unwrap();
        managed.load_state(&state).unwrap();
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
        assert_eq!((raw.cpu.regs[15], raw.cpu.regs[1]), (0x0814_2ae6, 0x083a_dfe8));
        let owner = managed.bus.rom_resources().unwrap().clone();
        let initial = owner.layered_obj_animations_stats().unwrap().event_bytes;
        let mut native_blocks = 0u64;
        managed.bus.rom[at..at + LENGTH].fill(0xcd);
        for frame in 0..421 {
            raw.bus.keys = 0x3ff;
            managed.bus.keys = 0x3ff;
            if native {
                let raw_ptr = &mut raw as *mut Machine as *mut std::ffi::c_void;
                let managed_ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
                let raw_counts = crate::run_frame_native(&mut raw, &table, raw_ptr, crate::FRAME_STEP_GUARD);
                let managed_counts = crate::run_frame_native(&mut managed, &table, managed_ptr, crate::FRAME_STEP_GUARD);
                assert_eq!(raw_counts, managed_counts, "dispatch at frame {frame}");
                native_blocks += managed_counts.0;
            } else {
                raw.bus.frame_ready = false;
                managed.bus.frame_ready = false;
                for _ in 0..5_000_000 {
                    assert_eq!(raw.bus.frame_ready, managed.bus.frame_ready);
                    if raw.bus.frame_ready { break; }
                    assert_eq!((raw.cpu.regs[15], raw.cpu.thumb()), (managed.cpu.regs[15], managed.cpu.thumb()));
                    let pc = raw.cpu.regs[15];
                    let source = raw.cpu.regs[1];
                    let event = raw.step();
                    let executed = matches!(event, gba_core::StepEvent::Instr(_));
                    assert_eq!(executed, matches!(managed.step(), gba_core::StepEvent::Instr(_)));
                    if executed && pc == 0x0814_2ae6 {
                        if let Some(offset) = source.checked_sub(0x0800_0000 + CONTEXT_EVENT_ROOT) {
                            if offset < LENGTH as u32 && offset % 16 == 0 { seen[(offset / 16) as usize] = true; }
                        }
                    }
                }
            }
            assert!(raw.bus.frame_ready && managed.bus.frame_ready, "frame {frame} exceeded step limit");
            assert_eq!(raw.bus.framebuffer, managed.bus.framebuffer, "framebuffer at {frame}");
            assert_eq!(raw.bus.wide_framebuffer, managed.bus.wide_framebuffer, "wide framebuffer at {frame}");
            assert_eq!(raw.bus.oam, managed.bus.oam, "OAM at {frame}");
            assert_eq!(raw.bus.audio_buf, managed.bus.audio_buf, "mixed audio at {frame}");
            assert_eq!(raw.bus.psg_tap, managed.bus.psg_tap, "PSG audio at {frame}");
            assert_eq!(raw.bus.fifo_tap, managed.bus.fifo_tap, "FIFO audio at {frame}");
            assert!(raw.bus.hle_tap.iter().map(|x| x.to_bits()).eq(managed.bus.hle_tap.iter().map(|x| x.to_bits())), "HLE audio at {frame}");
            managed.bus.rom[at..at + LENGTH].copy_from_slice(&rom[at..at + LENGTH]);
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "state and clock, native={native}, frame={frame}");
            managed.bus.rom[at..at + LENGTH].fill(0xcd);
            for machine in [&mut raw, &mut managed] {
                machine.bus.audio_buf.clear();
                machine.bus.psg_tap.clear();
                machine.bus.hle_tap.clear();
                machine.bus.fifo_tap.iter_mut().for_each(Vec::clear);
            }
        }
        if native { assert!(native_blocks > 0); } else { assert!(seen.iter().all(|&v| v)); }
        let used = owner.layered_obj_animations_stats().unwrap().event_bytes - initial;
        assert!(used > 0);
        managed.bus.rom[at..at + LENGTH].copy_from_slice(&rom[at..at + LENGTH]);
        payloads.push(used);
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[139,138], "ordinary_event_rows_seen":seen,
        "ordinary_interpreter_pair":true, "ordinary_native_dispatch_pair":true,
        "full_state_clock_image_oam_audio_frames_each":421, "source_rom_poisoned":true,
        "managed_event_bytes":payloads, "native_per_read_attribution":"pending",
        "other_paths":"pending", "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "ordinary EUR checkpoint and E139 dataset; bounded Wine inventory"]
fn european_e139_ordinary_high_rom_read_inventory() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    use std::collections::BTreeSet;

    let root = PathBuf::from(std::env::var_os("FFTA_CONTEXT_EVENT_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_CONTEXT_EVENT_DATASET").unwrap());
    let state = fs::read(std::env::var_os("FFTA_CONTEXT_EVENT_STATE").unwrap()).unwrap();
    let expected_state = std::env::var("FFTA_ROM_INVENTORY_STATE_SHA").unwrap_or_else(|_| "d8f174e8f886ac0edbbd060b6a0509c4f0c333b8bdd6f8645c536be35df6e541".into());
    assert_eq!(gba_core::ffta_rom::digest(&state), expected_state);
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    let expected_extractor = std::env::var("FFTA_ROM_INVENTORY_EXTRACTOR").ok().map(|v| v.parse().unwrap()).unwrap_or(139);
    let expected_decoder = std::env::var("FFTA_ROM_INVENTORY_DECODER").ok().map(|v| v.parse().unwrap()).unwrap_or(138);
    assert_eq!((manifest.extractor_revision, manifest.decoder_revision), (expected_extractor, expected_decoder));
    let mut intervals: Vec<(u32, u32)> = manifest.resources.iter().flat_map(|resource| resource.sources.iter())
        .filter(|source| source.length > 0).map(|source| (source.offset, source.offset + source.length)).collect();
    intervals.sort_unstable();
    let mut merged: Vec<(u32, u32)> = Vec::new();
    for (lo, hi) in intervals {
        if let Some(last) = merged.last_mut() {
            if lo <= last.1 { last.1 = last.1.max(hi); continue; }
        }
        merged.push((lo, hi));
    }
    let owned = |at: u32| {
        let index = merged.partition_point(|&(_, hi)| hi <= at);
        index < merged.len() && merged[index].0 <= at
    };
    let mut machine = crate::make_machine(rom, None);
    attach_from_directory(&mut machine, &dataset).unwrap();
    machine.load_state(&state).unwrap();
    let target_tick = std::env::var("FFTA_ROM_INVENTORY_TARGET_TICK").ok().map(|v| v.parse::<u32>().unwrap());
    let mut prelude_frames = 0u32;
    if let Some(target) = target_tick {
        machine.bus.keys = 0x3ff;
        machine.bus.frame_ready = false;
        let mut steps_in_frame = 0u32;
        loop {
            let heap = u32::from_le_bytes(machine.bus.ewram[0xf3b0..0xf3b4].try_into().unwrap());
            let offset = heap.checked_sub(0x0200_0000).expect("live context") as usize;
            let counter = u32::from_le_bytes(machine.bus.ewram[offset + 0x8fd8..offset + 0x8fdc].try_into().unwrap());
            if machine.cpu.regs[15] == 0x0814_0e6c && counter == target { break; }
            machine.step();
            steps_in_frame += 1;
            assert!(steps_in_frame < 5_000_000, "inventory prelude frame limit");
            if machine.bus.frame_ready {
                prelude_frames += 1;
                assert!(prelude_frames < 40, "inventory target tick not reached");
                steps_in_frame = 0;
                machine.bus.frame_ready = false;
            }
        }
    }
    let capture = TraceCapture::start(TraceFilter { readers: vec![], ranges: vec![(0x0030_0000, 0x0100_0000)] }, 50_000).unwrap();
    for _ in 0..3 {
        capture.frame(Some(machine.bus.frames));
        machine.bus.frame_ready = false;
        for _ in 0..5_000_000 {
            if machine.bus.frame_ready { break; }
            machine.step();
        }
        assert!(machine.bus.frame_ready);
    }
    let report = capture.finish();
    assert!(!report.truncated, "inventory capture dropped requests");
    let mut unowned_bytes = BTreeSet::new();
    let mut unowned_events = Vec::new();
    let mut owned_raw_events = Vec::new();
    for event in report.events {
        let read = event.key.read;
        let mut unowned = false;
        let mut known = false;
        for offset in read.physical_offset..read.physical_offset.saturating_add(read.length as u32) {
            if !(0x0030_0000..0x0100_0000).contains(&offset) { continue; }
            if owned(offset) { known = true; } else { unowned = true; unowned_bytes.insert(offset); }
        }
        if unowned { unowned_events.push(event.clone()); }
        if known && event.key.service == Service::Raw { owned_raw_events.push(event); }
    }
    let mut ranges: Vec<(u32, u32)> = Vec::new();
    for at in &unowned_bytes {
        if let Some(last) = ranges.last_mut() {
            if last.1 == *at { last.1 += 1; continue; }
        }
        ranges.push((*at, *at + 1));
    }
    let unowned_total = unowned_events.len();
    let owned_raw_total = owned_raw_events.len();
    unowned_events.truncate(200);
    owned_raw_events.truncate(200);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"three no-input interpreter frames from authenticated ordinary EUR checkpoint; high ROM 300000..1000000",
        "state_sha256":expected_state, "revision":[expected_extractor, expected_decoder],
        "prelude_target_tick":target_tick, "prelude_frames":prelude_frames,
        "matched_requests":report.matched_requests, "dropped_requests":report.dropped_requests,
        "instruction_fetch_requests_excluded":report.fetch_requests_excluded,
        "manifest_merged_source_ranges":merged.len(), "unowned_byte_count":unowned_bytes.len(),
        "unowned_ranges":ranges, "unowned_event_count":unowned_total, "unowned_events_first_200":unowned_events,
        "owned_raw_event_count":owned_raw_total, "owned_raw_events_first_200":owned_raw_events,
        "limitations":"Read scopes may include nested HLE or DMA; unowned bytes are observed source leads, not asset boundaries. Static manifest ownership does not guarantee runtime service.",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
