//! Conditional language-indexed position pairs and original EUR readers.
use super::*;
use gba_core::{rom_data::layered_obj_animations::{CONTEXT_LANGUAGE_POSITION_A, CONTEXT_LANGUAGE_POSITION_B}, Bus, Machine};

#[cfg(feature = "rom-data-read-audit")]
thread_local! {
    static CONTEXT_LANGUAGE_DIRECT: std::cell::Cell<[u64; 20]> = const { std::cell::Cell::new([0; 20]) };
}

#[cfg(feature = "rom-data-read-audit")]
fn count_context_language_read(pointer: *mut std::ffi::c_void, address: u32, read: extern "C" fn(*mut std::ffi::c_void, u32) -> u32) -> u32 {
    let offset = address & 0x01ff_ffff;
    if (0x08..=0x0d).contains(&(address >> 24)) && (CONTEXT_LANGUAGE_POSITION_A..CONTEXT_LANGUAGE_POSITION_B + 10).contains(&offset) {
        CONTEXT_LANGUAGE_DIRECT.with(|counts| {
            let mut values = counts.get();
            values[(offset - CONTEXT_LANGUAGE_POSITION_A) as usize] += 1;
            counts.set(values);
        });
    }
    read(pointer, address)
}

#[cfg(feature = "rom-data-read-audit")]
extern "C" fn context_language_read8(pointer: *mut std::ffi::c_void, address: u32) -> u32 {
    count_context_language_read(pointer, address, gba_core::capi::RT_API.read8)
}
#[cfg(feature = "rom-data-read-audit")]
extern "C" fn context_language_read16(pointer: *mut std::ffi::c_void, address: u32) -> u32 {
    count_context_language_read(pointer, address, gba_core::capi::RT_API.read16)
}
#[cfg(feature = "rom-data-read-audit")]
extern "C" fn context_language_read32(pointer: *mut std::ffi::c_void, address: u32) -> u32 {
    count_context_language_read(pointer, address, gba_core::capi::RT_API.read32)
}

#[cfg(feature = "rom-data-read-audit")]
static CONTEXT_LANGUAGE_API: gba_core::capi::RtApi = gba_core::capi::RtApi {
    read8: context_language_read8, read16: context_language_read16, read32: context_language_read32,
    ..gba_core::capi::RT_API
};

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "EUR pre-selection checkpoint, E170 dataset and native library; Wine only"]
fn european_language_positions_all_direct_native_pairs() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    use std::collections::BTreeSet;

    crate::configure_headless_widescreen();
    let root = PathBuf::from(std::env::var_os("FFTA_LANGUAGE_POSITION_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_LANGUAGE_POSITION_DATASET").unwrap());
    let state = fs::read(std::env::var_os("FFTA_LANGUAGE_PRESELECTION_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "4db5a60bb293bb319e685fb317b9a31a85df8d0d872d859bcedb07c0ecc4ea22");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision, manifest.decoder_revision), (170, 169));
    assert_eq!(CONTEXT_LANGUAGE_POSITION_B, CONTEXT_LANGUAGE_POSITION_A + 10);
    let (_library, table) = crate::load_native(Path::new(&std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap())).unwrap();
    let mut observations = Vec::new();
    let mut audio_totals = [0u64; 5];
    for language in 0u8..=4 {
        let mut raw = crate::make_machine(rom.clone(), None);
        let mut managed = crate::make_machine(rom.clone(), None);
        attach_from_directory(&mut managed, &dataset).unwrap();
        for machine in [&mut raw, &mut managed] {
            machine.load_state(&state).unwrap();
            crate::arm_audio_hle(machine, None);
            machine.bus.tap_channels = true;
            machine.bus.audio_buf.clear(); machine.bus.psg_tap.clear();
            machine.bus.fifo_tap[0].clear(); machine.bus.fifo_tap[1].clear(); machine.bus.hle_tap.clear();
        }
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
        let start = raw.bus.frames;
        let confirm = 4 * u32::from(language);
        let selected = [2 * usize::from(language), 2 * usize::from(language) + 1,
            10 + 2 * usize::from(language), 11 + 2 * usize::from(language)];
        CONTEXT_LANGUAGE_DIRECT.with(|counts| counts.set([0; 20]));
        let trace = TraceCapture::start(TraceFilter { readers: vec![], ranges: vec![(CONTEXT_LANGUAGE_POSITION_A, CONTEXT_LANGUAGE_POSITION_B + 10)] }, 20_000).unwrap();
        let mut dispatch = (0u64, 0u64);
        let mut completed = None;
        for frame in 0..1000u32 {
            let keys = if frame < confirm && frame % 4 < 2 { 0x37f }
                else if (confirm..confirm + 2).contains(&frame) { 0x3fe } else { 0x3ff };
            raw.bus.keys = keys;
            trace.frame(None);
            let raw_pointer = &mut raw as *mut Machine as *mut std::ffi::c_void;
            let raw_counts = crate::run_frame_native(&mut raw, &table, raw_pointer, crate::FRAME_STEP_GUARD);
            for at in CONTEXT_LANGUAGE_POSITION_A..CONTEXT_LANGUAGE_POSITION_B + 10 { managed.bus.rom[at as usize] = 0xcd; }
            managed.bus.keys = keys;
            trace.frame(Some(managed.bus.frames));
            let managed_pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
            let managed_counts = crate::with_test_native_api(&CONTEXT_LANGUAGE_API, || crate::run_frame_native(&mut managed, &table, managed_pointer, crate::FRAME_STEP_GUARD));
            trace.frame(None);
            assert_eq!(raw_counts, managed_counts, "language {language}, dispatch frame {frame}");
            assert_eq!((raw.bus.frames, managed.bus.frames), (start + u64::from(frame) + 1, start + u64::from(frame) + 1));
            dispatch.0 += raw_counts.0; dispatch.1 += raw_counts.1;
            assert_eq!(raw.bus.clock, managed.bus.clock);
            assert_eq!(raw.bus.framebuffer, managed.bus.framebuffer);
            assert_eq!(raw.bus.wide_framebuffer, managed.bus.wide_framebuffer);
            assert_eq!(raw.bus.oam, managed.bus.oam);
            assert_eq!(raw.bus.audio_buf, managed.bus.audio_buf);
            assert_eq!(raw.bus.psg_tap, managed.bus.psg_tap);
            assert_eq!(raw.bus.fifo_tap, managed.bus.fifo_tap);
            assert!(raw.bus.hle_tap.iter().map(|v| v.to_bits()).eq(managed.bus.hle_tap.iter().map(|v| v.to_bits())));
            for (sum, len) in audio_totals.iter_mut().zip([managed.bus.audio_buf.len(), managed.bus.psg_tap.len(), managed.bus.fifo_tap[0].len(), managed.bus.fifo_tap[1].len(), managed.bus.hle_tap.len()]) { *sum += len as u64; }
            for at in CONTEXT_LANGUAGE_POSITION_A..CONTEXT_LANGUAGE_POSITION_B + 10 { managed.bus.rom[at as usize] = rom[at as usize]; }
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "language {language}, state frame {frame}");
            for machine in [&mut raw, &mut managed] {
                machine.bus.audio_buf.clear(); machine.bus.psg_tap.clear();
                machine.bus.fifo_tap[0].clear(); machine.bus.fifo_tap[1].clear(); machine.bus.hle_tap.clear();
            }
            let direct = CONTEXT_LANGUAGE_DIRECT.with(|counts| counts.get());
            if frame >= confirm && raw.bus.ewram[0x1f6e] == language && selected.iter().all(|&index| direct[index] > 0) {
                completed = Some(frame);
                break;
            }
        }
        let frame = completed.expect("ordinary selection and direct reads of both position pairs not observed");
        let report = trace.finish();
        assert!(!report.truncated && report.dropped_requests == 0);
        let mut typed_offsets = BTreeSet::new();
        let mut raw_requests = 0u64;
        let mut unclassified = 0u64;
        for event in &report.events {
            match event.key.service {
                Service::Typed => { typed_offsets.insert(event.key.read.physical_offset); }
                Service::Raw => raw_requests += event.requests,
                Service::Unclassified => unclassified += event.requests,
            }
        }
        for &index in &selected { assert!(typed_offsets.contains(&(CONTEXT_LANGUAGE_POSITION_A + index as u32))); }
        assert_eq!((raw_requests, unclassified), (0, 0));
        assert!(dispatch.0 > 0);
        let direct = CONTEXT_LANGUAGE_DIRECT.with(|counts| counts.get());
        observations.push(serde_json::json!({"language":language, "last_frame":frame,
            "direct_selected_byte_requests":selected.map(|index| direct[index]),
            "native_dispatches":dispatch.0, "fallback_dispatches":dispatch.1, "raw_requests":raw_requests}));
    }
    assert!(audio_totals.iter().all(|&samples| samples > 0));
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[170,169], "scope":"EUR startup preselection, five ordinary languages and both position tables",
        "observations":observations, "audio_sample_counts":audio_totals, "source_rom_poisoned":true,
        "full_state_clock_video_oam_dispatch_audio_equal_each_frame":true,
        "limits":"Direct generated API requests identify selected bytes, not exact guest instruction PCs; this covers one startup route, not other writers, byte values, scenes or platforms.",
        "performance_measurements":false
    })).unwrap()).unwrap();
}

fn original_pair(machine: &mut Machine, first: bool, language: u8) -> (u32, u32) {
    machine.bus.write8(0x0200_1f6e, language);
    machine.cpu.regs[1] = 0x0200_1f6e;
    machine.cpu.regs[3] = 0x0800_0000 + if first { CONTEXT_LANGUAGE_POSITION_A } else { CONTEXT_LANGUAGE_POSITION_B };
    machine.cpu.regs[15] = if first { 0x0814_2064 } else { 0x0814_20a0 };
    machine.cpu.cpsr = 0xbf;
    let stop = if first { 0x0814_2072 } else { 0x0814_20ae };
    for _ in 0..32 {
        machine.bus.frame_ready = false;
        machine.step();
        if machine.cpu.regs[15] == stop { return (machine.cpu.regs[1], machine.cpu.regs[2]); }
    }
    panic!("original language-position reader did not finish");
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "exact EUR ROM, E140 predecessor, ordinary state and translation; Wine only"]
fn european_language_positions_publication_and_ordinary_pairs() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};

    let root = PathBuf::from(std::env::var_os("FFTA_LANGUAGE_POSITION_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_LANGUAGE_POSITION_PREVIOUS").unwrap());
    let state = fs::read(std::env::var_os("FFTA_LANGUAGE_POSITION_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "d8f174e8f886ac0edbbd060b6a0509c4f0c333b8bdd6f8645c536be35df6e541");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (140, 139));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision), (141, 140));
    assert_eq!(source.manifest.coverage, old.coverage);
    assert_eq!(source.manifest.resources.len(), old.resources.len());
    let prepared = preparation::prepare(&root.join("e141-resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("e141-resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    for old_resource in &old.resources {
        let new = source.manifest.resources.iter().find(|r| r.id == old_resource.id).unwrap();
        let old_bytes = fs::read(previous.join(&old_resource.path)).unwrap();
        let new_bytes = fs::read(prepared.directory.join(&new.path)).unwrap();
        assert_eq!(gba_core::ffta_rom::digest(&old_bytes), old_resource.sha256);
        if old_resource.id != LAYERED_OBJ_ANIMATIONS_ID {
            assert_eq!(new, old_resource);
            assert!(new_bytes == old_bytes, "unrelated resource changed");
            continue;
        }
        let prior = parse_layered_obj_animations(&old_bytes).unwrap();
        let mut current = parse_layered_obj_animations(&new_bytes).unwrap();
        assert!(prior.context_language_positions.is_none());
        let positions = current.context_language_positions.take().unwrap();
        for (base, rows) in [(CONTEXT_LANGUAGE_POSITION_A, &positions.first), (CONTEXT_LANGUAGE_POSITION_B, &positions.second)] {
            for (index, row) in rows.iter().enumerate() {
                let at = base as usize + 2 * index;
                assert_eq!(row.source_bytes(), rom[at..at + 2]);
            }
        }
        assert_eq!(current, prior);
        let mut metadata = new.clone();
        metadata.sources.retain(|s| s.role != "context-language-position-a" && s.role != "context-language-position-b");
        metadata.sha256 = old_resource.sha256.clone();
        assert_eq!(&metadata, old_resource);
    }
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let before = owner.layered_obj_animations_stats().unwrap();
    for at in CONTEXT_LANGUAGE_POSITION_A..CONTEXT_LANGUAGE_POSITION_B + 10 { managed.bus.rom[at as usize] = 0xcd; }
    for at in CONTEXT_LANGUAGE_POSITION_A..CONTEXT_LANGUAGE_POSITION_B + 10 {
        assert_eq!(managed.bus.read8(0x0800_0000 + at), rom[at as usize]);
    }
    let after = owner.layered_obj_animations_stats().unwrap();
    assert_eq!(after.language_position_a_bytes - before.language_position_a_bytes, 10);
    assert_eq!(after.language_position_b_bytes - before.language_position_b_bytes, 10);
    for at in [CONTEXT_LANGUAGE_POSITION_A - 1, CONTEXT_LANGUAGE_POSITION_B + 10] {
        assert_eq!(managed.bus.read8(0x0800_0000 + at), rom[at as usize]);
    }
    let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for edge in [CONTEXT_LANGUAGE_POSITION_A, CONTEXT_LANGUAGE_POSITION_B, CONTEXT_LANGUAGE_POSITION_B + 10] {
        for delta in [-3i32, -1, 0, 1] {
            let at = edge.wrapping_add_signed(delta) as usize;
            assert_eq!((gba_core::capi::RT_API.read16)(ptr, 0x0a00_0000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
            assert_eq!((gba_core::capi::RT_API.read32)(ptr, 0x0800_0000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
        }
    }
    for at in CONTEXT_LANGUAGE_POSITION_A..CONTEXT_LANGUAGE_POSITION_B + 10 { managed.bus.rom[at as usize] = rom[at as usize]; }
    let initial = raw.save_state().unwrap();
    for first in [true, false] {
        for language in [0u8, 4, 5, 255] {
            raw.load_state(&initial).unwrap();
            managed.load_state(&initial).unwrap();
            for at in CONTEXT_LANGUAGE_POSITION_A..CONTEXT_LANGUAGE_POSITION_B + 10 { managed.bus.rom[at as usize] = 0xcd; }
            let left = original_pair(&mut raw, first, language);
            let right = original_pair(&mut managed, first, language);
            assert_eq!(left, right, "original address calculation: first={first}, language={language}");
            for at in CONTEXT_LANGUAGE_POSITION_A..CONTEXT_LANGUAGE_POSITION_B + 10 { managed.bus.rom[at as usize] = rom[at as usize]; }
            assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "original reader state and clock");
        }
    }

    let (_library, table) = crate::load_native(&PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap())).unwrap();
    let mut counts = Vec::new();
    for native in [false, true] {
        let mut raw = crate::make_machine(rom.clone(), None);
        let mut managed = crate::make_machine(rom.clone(), None);
        attach_from_directory(&mut managed, &prepared.directory).unwrap();
        raw.load_state(&state).unwrap();
        managed.load_state(&state).unwrap();
        let owner = managed.bus.rom_resources().unwrap().clone();
        let before = owner.layered_obj_animations_stats().unwrap();
        let mut typed = 0usize;
        let mut raw_reads = 0usize;
        let mut native_dispatches = 0u64;
        for frame in 0..3 {
            for at in CONTEXT_LANGUAGE_POSITION_A..CONTEXT_LANGUAGE_POSITION_B + 10 { managed.bus.rom[at as usize] = 0xcd; }
            if native {
                let ptr = &mut raw as *mut Machine as *mut std::ffi::c_void;
                crate::run_frame_native(&mut raw, &table, ptr, crate::FRAME_STEP_GUARD);
            } else {
                raw.bus.frame_ready = false;
                for _ in 0..5_000_000 { if raw.bus.frame_ready { break; } raw.step(); }
            }
            assert!(raw.bus.frame_ready);
            let capture = TraceCapture::start(TraceFilter { readers: vec![], ranges: vec![(CONTEXT_LANGUAGE_POSITION_A, CONTEXT_LANGUAGE_POSITION_B + 10)] }, 10000).unwrap();
            capture.frame(Some(managed.bus.frames));
            if native {
                let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
                native_dispatches += crate::run_frame_native(&mut managed, &table, ptr, crate::FRAME_STEP_GUARD).0;
            } else {
                managed.bus.frame_ready = false;
                for _ in 0..5_000_000 { if managed.bus.frame_ready { break; } managed.step(); }
            }
            assert!(managed.bus.frame_ready);
            let report = capture.finish();
            assert!(!report.truncated);
            for event in report.events {
                match event.key.service {
                    Service::Typed => typed += event.requests as usize,
                    Service::Raw => raw_reads += event.requests as usize,
                    _ => {}
                }
            }
            for at in CONTEXT_LANGUAGE_POSITION_A..CONTEXT_LANGUAGE_POSITION_B + 10 { managed.bus.rom[at as usize] = rom[at as usize]; }
            assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "full state and clock, native={native}, frame={frame}");
        }
        let after = owner.layered_obj_animations_stats().unwrap();
        let first_bytes = after.language_position_a_bytes - before.language_position_a_bytes;
        let second_bytes = after.language_position_b_bytes - before.language_position_b_bytes;
        assert!(first_bytes > 0 && typed > 0 && raw_reads == 0);
        if native { assert!(native_dispatches > 0); }
        counts.push((typed, raw_reads, first_bytes, second_bytes, native_dispatches));
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[141,140], "tables":2, "source_bytes":20,
        "prior_resources_preserved":true, "original_reader_cases":8, "out_of_domain_index":"original addressing preserved",
        "source_rom_poisoned":true, "ordinary_interpreter_and_native_dispatch_pairs":counts,
        "full_state_clock_frames_each":3, "ordinary_other_languages":"pending", "ordinary_second_table":"pending",
        "per_read_native_attribution":"pending", "frontend_ui":"pending", "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "ordinary EUR event checkpoint, E141 dataset and translation; Wine only"]
fn european_language_second_position_ordinary_pairs() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};

    let root = PathBuf::from(std::env::var_os("FFTA_LANGUAGE_POSITION_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_LANGUAGE_POSITION_DATASET").unwrap());
    let input = fs::read(std::env::var_os("FFTA_LANGUAGE_POSITION_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&input), "d8f174e8f886ac0edbbd060b6a0509c4f0c333b8bdd6f8645c536be35df6e541");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision, manifest.decoder_revision), (141, 140));

    // Continue the observed startup unchanged until the second-table read.
    let mut observer = crate::make_machine(rom.clone(), None);
    observer.load_state(&input).unwrap();
    let mut frames = 0u32;
    let mut steps_in_frame = 0u32;
    observer.bus.frame_ready = false;
    while observer.cpu.regs[15] != 0x0814_20a6 && frames < 80 {
        observer.step();
        steps_in_frame += 1;
        assert!(steps_in_frame < 5_000_000, "ordinary observation frame limit");
        if observer.bus.frame_ready {
            frames += 1;
            steps_in_frame = 0;
            observer.bus.frame_ready = false;
        }
    }
    assert_eq!(observer.cpu.regs[15], 0x0814_20a6, "ordinary second-table reader was not reached");
    let language = observer.bus.read8(0x0200_1f6e);
    assert_eq!(language, 0, "checkpoint was reached with the observed English selection");
    assert_eq!(observer.cpu.regs[1], 0x0800_0000 + CONTEXT_LANGUAGE_POSITION_B + 2 * u32::from(language));
    let state = observer.save_state().unwrap();
    let expected_at = CONTEXT_LANGUAGE_POSITION_B as usize + 2 * language as usize;
    let expected = (u32::from(rom[expected_at]), u32::from(rom[expected_at + 1]));
    let (_library, table) = crate::load_native(&PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap())).unwrap();
    let mut counts = Vec::new();
    let mut original_arguments = false;
    for native in [false, true] {
        let mut raw = crate::make_machine(rom.clone(), None);
        let mut managed = crate::make_machine(rom.clone(), None);
        attach_from_directory(&mut managed, &dataset).unwrap();
        raw.load_state(&state).unwrap();
        managed.load_state(&state).unwrap();
        let owner = managed.bus.rom_resources().unwrap().clone();
        let before = owner.layered_obj_animations_stats().unwrap();
        let mut typed = 0usize;
        let mut raw_reads = 0usize;
        let mut native_dispatches = 0u64;
        for frame in 0..3 {
            for at in CONTEXT_LANGUAGE_POSITION_A..CONTEXT_LANGUAGE_POSITION_B + 10 { managed.bus.rom[at as usize] = 0xcd; }
            if native {
                let ptr = &mut raw as *mut Machine as *mut std::ffi::c_void;
                crate::run_frame_native(&mut raw, &table, ptr, crate::FRAME_STEP_GUARD);
            } else {
                raw.bus.frame_ready = false;
                for _ in 0..5_000_000 {
                    if raw.bus.frame_ready { break; }
                    if raw.cpu.regs[15] == 0x0814_20c2 {
                        assert_eq!((raw.cpu.regs[1], raw.cpu.regs[2]), expected);
                        original_arguments = true;
                    }
                    raw.step();
                }
            }
            assert!(raw.bus.frame_ready);
            let capture = TraceCapture::start(TraceFilter { readers: vec![], ranges: vec![(CONTEXT_LANGUAGE_POSITION_B, CONTEXT_LANGUAGE_POSITION_B + 10)] }, 10000).unwrap();
            capture.frame(Some(managed.bus.frames));
            if native {
                let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
                native_dispatches += crate::run_frame_native(&mut managed, &table, ptr, crate::FRAME_STEP_GUARD).0;
            } else {
                managed.bus.frame_ready = false;
                for _ in 0..5_000_000 { if managed.bus.frame_ready { break; } managed.step(); }
            }
            assert!(managed.bus.frame_ready);
            let report = capture.finish();
            assert!(!report.truncated);
            for event in report.events {
                match event.key.service {
                    Service::Typed => typed += event.requests as usize,
                    Service::Raw => raw_reads += event.requests as usize,
                    _ => {}
                }
            }
            for at in CONTEXT_LANGUAGE_POSITION_A..CONTEXT_LANGUAGE_POSITION_B + 10 { managed.bus.rom[at as usize] = rom[at as usize]; }
            assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "ordinary state and clock, native={native}, frame={frame}");
        }
        let after = owner.layered_obj_animations_stats().unwrap();
        let second_bytes = after.language_position_b_bytes - before.language_position_b_bytes;
        assert!(second_bytes >= 2 && typed >= 2 && raw_reads == 0);
        if native { assert!(native_dispatches > 0); }
        counts.push((typed, raw_reads, second_bytes, native_dispatches));
    }
    assert!(original_arguments, "ordinary original draw-wrapper arguments were not observed");
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"no-input EUR startup continuation from retained checkpoint",
        "frames_to_second_table":frames, "language":language, "ordinary_original_wrapper_arguments_match":true,
        "source_rom_poisoned":true, "ordinary_interpreter_and_native_dispatch_pairs":counts,
        "full_state_clock_frames_each":3, "other_languages":"pending", "per_read_native_attribution":"pending",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "cold EUR startup, E141 dataset and translation; Wine only"]
fn european_language_one_position_ordinary_pairs() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};

    let root = PathBuf::from(std::env::var_os("FFTA_LANGUAGE_POSITION_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_LANGUAGE_POSITION_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision, manifest.decoder_revision), (141, 140));
    let mut observer = crate::make_machine(rom.clone(), None);
    let mut selection_changed = false;
    let mut language_published = false;
    let mut preselection_sha = String::new();
    let mut checkpoints: [Option<Vec<u8>>; 2] = [None, None];
    let mut observed_frames = [0u32; 2];
    'startup: for frame in 0..1600u32 {
        // Existing no-input observation establishes the chooser by frame 180.
        if frame == 180 {
            let saved = observer.save_state().unwrap();
            preselection_sha = gba_core::ffta_rom::digest(&saved);
            fs::write(root.join("pre-language.ffstate"), saved).unwrap();
        }
        // Down for two frames, release, then confirm with A for two frames.
        observer.bus.keys = if (180..182).contains(&frame) { 0x37f }
            else if (184..186).contains(&frame) { 0x3fe } else { 0x3ff };
        observer.bus.frame_ready = false;
        for _ in 0..5_000_000 {
            if observer.bus.frame_ready { break; }
            let pc = observer.cpu.regs[15];
            let selection = if pc == 0x0813_ffca {
                let address = observer.cpu.regs[0].wrapping_add(2);
                let index = address.checked_sub(0x0200_0000).map(|v| v as usize);
                index.and_then(|index| observer.bus.ewram.get(index).copied().map(|old| (index, old, observer.cpu.regs[5] as u8)))
            } else { None };
            let publication = pc == 0x0813_ff64 && observer.cpu.regs[4] == 0x0200_1f6e && observer.cpu.regs[5] as u8 == 1;
            let target = if language_published && pc == 0x0814_206a && checkpoints[0].is_none() { Some(0) }
                else if language_published && pc == 0x0814_20a6 && checkpoints[1].is_none() { Some(1) }
                else { None };
            let candidate = target.map(|which| (which, observer.cpu.regs[1], observer.save_state().unwrap()));
            let event = observer.step();
            if !matches!(event, gba_core::StepEvent::Instr(ref instruction) if instruction.addr == pc) { continue; }
            if let Some((index, old, requested)) = selection {
                if old == 0 && requested == 1 && observer.bus.ewram[index] == 1 { selection_changed = true; }
            }
            if publication && observer.bus.ewram[0x1f6e] == 1 { language_published = true; }
            if let Some((which, address, saved)) = candidate {
                let base = if which == 0 { CONTEXT_LANGUAGE_POSITION_A } else { CONTEXT_LANGUAGE_POSITION_B };
                assert_eq!(address, 0x0800_0000 + base + 2, "language 1 source address");
                observed_frames[which] = frame;
                checkpoints[which] = Some(saved);
                if checkpoints.iter().all(Option::is_some) { break 'startup; }
            }
        }
        assert!(observer.bus.frame_ready, "ordinary startup frame limit at {frame}");
        if frame == 230 { assert!(selection_changed && language_published, "Down/A did not publish language 1"); }
    }
    assert!(selection_changed && language_published, "ordinary language 1 selection not established");
    assert!(checkpoints.iter().all(Option::is_some), "ordinary language 1 did not reach both tables");
    let (_library, table) = crate::load_native(&PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap())).unwrap();
    let mut counts = Vec::new();
    for which in 0..2 {
        let base = if which == 0 { CONTEXT_LANGUAGE_POSITION_A } else { CONTEXT_LANGUAGE_POSITION_B };
        let expected_at = base as usize + 2;
        let expected = (u32::from(rom[expected_at]), u32::from(rom[expected_at + 1]));
        let call = if which == 0 { 0x0814_2086 } else { 0x0814_20c2 };
        let mut original_arguments = false;
        for native in [false, true] {
            let mut raw = crate::make_machine(rom.clone(), None);
            let mut managed = crate::make_machine(rom.clone(), None);
            attach_from_directory(&mut managed, &dataset).unwrap();
            raw.load_state(checkpoints[which].as_ref().unwrap()).unwrap();
            managed.load_state(checkpoints[which].as_ref().unwrap()).unwrap();
            let owner = managed.bus.rom_resources().unwrap().clone();
            let before = owner.layered_obj_animations_stats().unwrap();
            let mut typed = 0usize;
            let mut raw_reads = 0usize;
            let mut native_dispatches = 0u64;
            for frame in 0..3 {
                for at in CONTEXT_LANGUAGE_POSITION_A..CONTEXT_LANGUAGE_POSITION_B + 10 { managed.bus.rom[at as usize] = 0xcd; }
                if native {
                    let ptr = &mut raw as *mut Machine as *mut std::ffi::c_void;
                    crate::run_frame_native(&mut raw, &table, ptr, crate::FRAME_STEP_GUARD);
                } else {
                    raw.bus.frame_ready = false;
                    for _ in 0..5_000_000 {
                        if raw.bus.frame_ready { break; }
                        if raw.cpu.regs[15] == call {
                            assert_eq!((raw.cpu.regs[1], raw.cpu.regs[2]), expected);
                            original_arguments = true;
                        }
                        raw.step();
                    }
                }
                assert!(raw.bus.frame_ready);
                let capture = TraceCapture::start(TraceFilter { readers: vec![], ranges: vec![(base, base + 10)] }, 10000).unwrap();
                capture.frame(Some(managed.bus.frames));
                if native {
                    let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
                    native_dispatches += crate::run_frame_native(&mut managed, &table, ptr, crate::FRAME_STEP_GUARD).0;
                } else {
                    managed.bus.frame_ready = false;
                    for _ in 0..5_000_000 { if managed.bus.frame_ready { break; } managed.step(); }
                }
                assert!(managed.bus.frame_ready);
                let report = capture.finish();
                assert!(!report.truncated);
                for event in report.events {
                    match event.key.service {
                        Service::Typed => typed += event.requests as usize,
                        Service::Raw => raw_reads += event.requests as usize,
                        _ => {}
                    }
                }
                for at in CONTEXT_LANGUAGE_POSITION_A..CONTEXT_LANGUAGE_POSITION_B + 10 { managed.bus.rom[at as usize] = rom[at as usize]; }
                assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "language 1 state and clock: table={which}, native={native}, frame={frame}");
            }
            let after = owner.layered_obj_animations_stats().unwrap();
            let used = if which == 0 { after.language_position_a_bytes - before.language_position_a_bytes }
                else { after.language_position_b_bytes - before.language_position_b_bytes };
            assert!(used >= 2 && typed >= 2 && raw_reads == 0);
            if native { assert!(native_dispatches > 0); }
            counts.push((which, native, typed, raw_reads, used, native_dispatches));
        }
        assert!(original_arguments, "language 1 original wrapper arguments were not observed");
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"cold EUR startup, observed Down/release/A selection of language 1",
        "selection_changed_zero_to_one":selection_changed, "language_published":language_published,
        "preselection_sha256":preselection_sha, "source_reader_frames":observed_frames,
        "ordinary_original_wrapper_arguments_match":true, "source_rom_poisoned":true,
        "ordinary_interpreter_and_native_dispatch_pairs":counts, "full_state_clock_frames_each":3,
        "other_languages":"pending", "per_read_native_attribution":"pending", "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "EUR pre-selection checkpoint, E141 dataset and translation; Wine only"]
fn european_languages_two_to_four_position_ordinary_pairs() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};

    let root = PathBuf::from(std::env::var_os("FFTA_LANGUAGE_POSITION_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_LANGUAGE_POSITION_DATASET").unwrap());
    let preselection = fs::read(std::env::var_os("FFTA_LANGUAGE_PRESELECTION_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&preselection), "4db5a60bb293bb319e685fb317b9a31a85df8d0d872d859bcedb07c0ecc4ea22");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision, manifest.decoder_revision), (141, 140));
    let (_library, table) = crate::load_native(&PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap())).unwrap();
    let mut results = Vec::new();
    for language in [2u8, 3, 4] {
        let mut observer = crate::make_machine(rom.clone(), None);
        observer.load_state(&preselection).unwrap();
        let mut selection_mask = 0u8;
        let mut published = false;
        let mut checkpoints: [Option<Vec<u8>>; 2] = [None, None];
        let mut observed_frames = [0u32; 2];
        let confirm_frame = 4 * u32::from(language);
        'startup: for frame in 0..1000u32 {
            // Each two-frame Down pulse is followed by two neutral frames.
            observer.bus.keys = if frame < confirm_frame && frame % 4 < 2 { 0x37f }
                else if (confirm_frame..confirm_frame + 2).contains(&frame) { 0x3fe } else { 0x3ff };
            observer.bus.frame_ready = false;
            for _ in 0..5_000_000 {
                if observer.bus.frame_ready { break; }
                let pc = observer.cpu.regs[15];
                let selection = if pc == 0x0813_ffca {
                    let address = observer.cpu.regs[0].wrapping_add(2);
                    let index = address.checked_sub(0x0200_0000).map(|v| v as usize);
                    index.and_then(|index| observer.bus.ewram.get(index).copied().map(|old| (index, old, observer.cpu.regs[5] as u8)))
                } else { None };
                let publication = pc == 0x0813_ff64 && observer.cpu.regs[4] == 0x0200_1f6e && observer.cpu.regs[5] as u8 == language;
                let target = if published && pc == 0x0814_206a && checkpoints[0].is_none() { Some(0) }
                    else if published && pc == 0x0814_20a6 && checkpoints[1].is_none() { Some(1) }
                    else { None };
                let candidate = target.map(|which| (which, observer.cpu.regs[1], observer.save_state().unwrap()));
                let event = observer.step();
                if !matches!(event, gba_core::StepEvent::Instr(ref instruction) if instruction.addr == pc) { continue; }
                if let Some((index, old, requested)) = selection {
                    if old < language && requested == old + 1 && observer.bus.ewram[index] == requested {
                        selection_mask |= 1 << requested;
                    }
                }
                if publication && observer.bus.ewram[0x1f6e] == language { published = true; }
                if let Some((which, address, saved)) = candidate {
                    let base = if which == 0 { CONTEXT_LANGUAGE_POSITION_A } else { CONTEXT_LANGUAGE_POSITION_B };
                    assert_eq!(address, 0x0800_0000 + base + 2 * u32::from(language), "language {language} source address");
                    observed_frames[which] = frame;
                    checkpoints[which] = Some(saved);
                    if checkpoints.iter().all(Option::is_some) { break 'startup; }
                }
            }
            assert!(observer.bus.frame_ready, "ordinary startup frame limit, language={language}, frame={frame}");
            if frame == 120 { assert!(published, "input did not publish language {language}, selection mask {selection_mask:x}"); }
        }
        let expected_mask = (1u8 << (language + 1)) - 2;
        assert_eq!(selection_mask, expected_mask, "ordinary language {language} selection steps");
        assert!(published && checkpoints.iter().all(Option::is_some), "ordinary language {language} did not reach both tables");
        let mut counts = Vec::new();
        for which in 0..2 {
            let base = if which == 0 { CONTEXT_LANGUAGE_POSITION_A } else { CONTEXT_LANGUAGE_POSITION_B };
            let expected_at = base as usize + 2 * language as usize;
            let expected = (u32::from(rom[expected_at]), u32::from(rom[expected_at + 1]));
            let call = if which == 0 { 0x0814_2086 } else { 0x0814_20c2 };
            let mut original_arguments = false;
            for native in [false, true] {
                let mut raw = crate::make_machine(rom.clone(), None);
                let mut managed = crate::make_machine(rom.clone(), None);
                attach_from_directory(&mut managed, &dataset).unwrap();
                raw.load_state(checkpoints[which].as_ref().unwrap()).unwrap();
                managed.load_state(checkpoints[which].as_ref().unwrap()).unwrap();
                let owner = managed.bus.rom_resources().unwrap().clone();
                let before = owner.layered_obj_animations_stats().unwrap();
                let mut typed = 0usize;
                let mut raw_reads = 0usize;
                let mut native_dispatches = 0u64;
                for frame in 0..3 {
                    for at in CONTEXT_LANGUAGE_POSITION_A..CONTEXT_LANGUAGE_POSITION_B + 10 { managed.bus.rom[at as usize] = 0xcd; }
                    if native {
                        let ptr = &mut raw as *mut Machine as *mut std::ffi::c_void;
                        crate::run_frame_native(&mut raw, &table, ptr, crate::FRAME_STEP_GUARD);
                    } else {
                        raw.bus.frame_ready = false;
                        for _ in 0..5_000_000 {
                            if raw.bus.frame_ready { break; }
                            if raw.cpu.regs[15] == call {
                                assert_eq!((raw.cpu.regs[1], raw.cpu.regs[2]), expected);
                                original_arguments = true;
                            }
                            raw.step();
                        }
                    }
                    assert!(raw.bus.frame_ready);
                    let capture = TraceCapture::start(TraceFilter { readers: vec![], ranges: vec![(base, base + 10)] }, 10000).unwrap();
                    capture.frame(Some(managed.bus.frames));
                    if native {
                        let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
                        native_dispatches += crate::run_frame_native(&mut managed, &table, ptr, crate::FRAME_STEP_GUARD).0;
                    } else {
                        managed.bus.frame_ready = false;
                        for _ in 0..5_000_000 { if managed.bus.frame_ready { break; } managed.step(); }
                    }
                    assert!(managed.bus.frame_ready);
                    let report = capture.finish();
                    assert!(!report.truncated);
                    for event in report.events {
                        match event.key.service {
                            Service::Typed => typed += event.requests as usize,
                            Service::Raw => raw_reads += event.requests as usize,
                            _ => {}
                        }
                    }
                    for at in CONTEXT_LANGUAGE_POSITION_A..CONTEXT_LANGUAGE_POSITION_B + 10 { managed.bus.rom[at as usize] = rom[at as usize]; }
                    assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "language {language} state and clock: table={which}, native={native}, frame={frame}");
                }
                let after = owner.layered_obj_animations_stats().unwrap();
                let used = if which == 0 { after.language_position_a_bytes - before.language_position_a_bytes }
                    else { after.language_position_b_bytes - before.language_position_b_bytes };
                assert!(used >= 2 && typed >= 2 && raw_reads == 0);
                if native { assert!(native_dispatches > 0); }
                counts.push((which, native, typed, raw_reads, used, native_dispatches));
            }
            assert!(original_arguments, "language {language} original wrapper arguments were not observed");
        }
        results.push(serde_json::json!({"language":language, "selection_mask":selection_mask,
            "source_reader_frames_after_preselection":observed_frames, "pairs":counts}));
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"EUR startup from retained pre-selection checkpoint with observed Down/release/A inputs",
        "languages":results, "ordinary_original_wrapper_arguments_match":true, "source_rom_poisoned":true,
        "full_state_clock_frames_each":3, "per_read_native_attribution":"pending", "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
