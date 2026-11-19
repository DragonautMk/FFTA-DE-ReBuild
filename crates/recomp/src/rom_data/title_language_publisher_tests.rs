//! Ordinary EUR title-language publication from a retained title checkpoint.
use super::*;
use gba_core::Bus;

#[cfg(feature = "rom-data-read-audit")]
thread_local! {
    static TITLE_CURSOR_DIRECT: std::cell::Cell<[u64; 10]> = const { std::cell::Cell::new([0; 10]) };
}

#[cfg(feature = "rom-data-read-audit")]
fn count_title_cursor_read(pointer: *mut std::ffi::c_void, address: u32, read: extern "C" fn(*mut std::ffi::c_void, u32) -> u32) -> u32 {
    let offset = address & 0x01ff_ffff;
    if (0x08..=0x0d).contains(&(address >> 24)) && (0x3adcfc..0x3add10).contains(&offset) && (offset - 0x3adcfc) % 2 == 0 {
        TITLE_CURSOR_DIRECT.with(|counts| {
            let mut values = counts.get();
            values[((offset - 0x3adcfc) / 2) as usize] += 1;
            counts.set(values);
        });
    }
    read(pointer, address)
}

#[cfg(feature = "rom-data-read-audit")]
extern "C" fn title_cursor_read8(pointer: *mut std::ffi::c_void, address: u32) -> u32 {
    count_title_cursor_read(pointer, address, gba_core::capi::RT_API.read8)
}
#[cfg(feature = "rom-data-read-audit")]
extern "C" fn title_cursor_read16(pointer: *mut std::ffi::c_void, address: u32) -> u32 {
    count_title_cursor_read(pointer, address, gba_core::capi::RT_API.read16)
}
#[cfg(feature = "rom-data-read-audit")]
extern "C" fn title_cursor_read32(pointer: *mut std::ffi::c_void, address: u32) -> u32 {
    count_title_cursor_read(pointer, address, gba_core::capi::RT_API.read32)
}

#[cfg(feature = "rom-data-read-audit")]
static TITLE_CURSOR_API: gba_core::capi::RtApi = gba_core::capi::RtApi {
    read8: title_cursor_read8, read16: title_cursor_read16, read32: title_cursor_read32,
    ..gba_core::capi::RT_API
};

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "EUR title selector checkpoints, E170 dataset and native library; Wine only"]
fn european_title_language_all_publications_native_pairs() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    use std::collections::BTreeSet;

    const GLOBAL: usize = 0x1f6e;
    const CURSOR: u32 = 0x3adcfc;
    const CONTEXT: u32 = 0x3ae42c;
    crate::configure_headless_widescreen();
    let root = PathBuf::from(std::env::var_os("FFTA_TITLE_LANGUAGE_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_TITLE_LANGUAGE_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let selector = fs::read(std::env::var_os("FFTA_TITLE_LANGUAGE_STATE").unwrap()).unwrap();
    let french = fs::read(std::env::var_os("FFTA_TITLE_LANGUAGE_FRENCH_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&selector), "ce54d2693068161aa1452835109c7a44082fa9bd4e2777ab6976cddd88e192e1");
    assert_eq!(gba_core::ffta_rom::digest(&french), "ccece459942747b2460d85f6a459ecd34bbb107afb49d25049dc9d3c25ca9749");
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision, manifest.decoder_revision), (170, 169));
    let (_library, table) = crate::load_native(Path::new(&std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap())).unwrap();
    let spans = [(CURSOR, CURSOR + 20), (CONTEXT, CONTEXT + 20)];
    let mut observations = Vec::new();
    let mut audio_totals = [0u64; 5];

    for target in [1u8, 2, 3, 4, 0] {
        let mut raw = crate::make_machine(rom.clone(), None);
        let mut managed = crate::make_machine(rom.clone(), None);
        attach_from_directory(&mut managed, &dataset).unwrap();
        for machine in [&mut raw, &mut managed] {
            machine.load_state(if target == 0 { &french } else { &selector }).unwrap();
            crate::arm_audio_hle(machine, None);
            machine.bus.tap_channels = true;
            machine.bus.audio_buf.clear(); machine.bus.psg_tap.clear();
            machine.bus.fifo_tap[0].clear(); machine.bus.fifo_tap[1].clear(); machine.bus.hle_tap.clear();
        }
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
        assert_eq!(raw.bus.ewram[GLOBAL], if target == 0 { 1 } else { 0 });
        let start = raw.bus.frames;
        let presses = if target == 4 { 1 } else { u32::from(target) };
        let confirm = 8 * presses.max(1);
        let trace = TraceCapture::start(TraceFilter { readers: vec![], ranges: spans.to_vec() }, 20_000).unwrap();
        TITLE_CURSOR_DIRECT.with(|counts| counts.set([0; 10]));
        let mut dispatch = (0u64, 0u64);
        let mut publication_frame = None;
        for frame in 0..160u32 {
            let keys = if target == 0 {
                match frame { 0..6 | 12..18 => 0x37f, 24..30 | 102..108 => 0x3fe, 90..96 => 0x3bf, _ => 0x3ff }
            } else if frame < presses * 8 && frame % 8 < 2 {
                if target == 4 { 0x3bf } else { 0x37f }
            } else if (confirm..confirm + 2).contains(&frame) { 0x3fe } else { 0x3ff };
            raw.bus.keys = keys;
            trace.frame(None);
            let raw_pointer = &mut raw as *mut gba_core::Machine as *mut std::ffi::c_void;
            let raw_counts = crate::run_frame_native(&mut raw, &table, raw_pointer, crate::FRAME_STEP_GUARD);
            for &(lo, hi) in &spans { managed.bus.rom[lo as usize..hi as usize].fill(0xcd); }
            managed.bus.keys = keys;
            trace.frame(Some(managed.bus.frames));
            let managed_pointer = &mut managed as *mut gba_core::Machine as *mut std::ffi::c_void;
            let managed_counts = crate::with_test_native_api(&TITLE_CURSOR_API, || crate::run_frame_native(&mut managed, &table, managed_pointer, crate::FRAME_STEP_GUARD));
            trace.frame(None);
            assert_eq!(raw_counts, managed_counts, "target {target}, dispatch frame {frame}");
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
            for &(lo, hi) in &spans { managed.bus.rom[lo as usize..hi as usize].copy_from_slice(&rom[lo as usize..hi as usize]); }
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "target {target}, state frame {frame}");
            for machine in [&mut raw, &mut managed] {
                machine.bus.audio_buf.clear(); machine.bus.psg_tap.clear();
                machine.bus.fifo_tap[0].clear(); machine.bus.fifo_tap[1].clear(); machine.bus.hle_tap.clear();
            }
            let ready = if target == 0 { frame >= 102 } else { frame >= confirm };
            let direct = TITLE_CURSOR_DIRECT.with(|counts| counts.get());
            let index = usize::from(target) * 2;
            if ready && raw.bus.ewram[GLOBAL] == target && direct[index] > 0 && direct[index + 1] > 0 {
                publication_frame = Some(frame);
                break;
            }
        }
        let frame = publication_frame.expect("native language publication and both cursor reads were not observed");
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
        let selected_cursor = CURSOR + 4 * u32::from(target);
        assert!(typed_offsets.contains(&selected_cursor) && typed_offsets.contains(&(selected_cursor + 2)));
        assert_eq!((raw_requests, unclassified), (0, 0));
        assert!(dispatch.0 > 0);
        let direct = TITLE_CURSOR_DIRECT.with(|counts| counts.get());
        observations.push(serde_json::json!({"target":target, "publication_frame":frame,
            "direct_coordinate_requests":[direct[usize::from(target) * 2], direct[usize::from(target) * 2 + 1]],
            "native_dispatches":dispatch.0, "fallback_dispatches":dispatch.1, "raw_requests":raw_requests}));
    }
    assert!(audio_totals.iter().all(|&samples| samples > 0));
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[170,169], "scope":"EUR title selector, all five publications with mixed native dispatch",
        "observations":observations, "audio_sample_counts":audio_totals, "source_rom_poisoned":true,
        "full_state_clock_video_oam_dispatch_audio_equal_each_frame":true,
        "limits":"Direct API wrappers establish selected title-cursor reads, not exact guest PCs or direct native reads of the separate context-position bank. Other writers, global value domain, scenes and platforms remain open.",
        "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "authenticated EUR title language selector checkpoint; Wine only"]
fn european_title_language_publisher_ordinary_observation() {
    let root = PathBuf::from(std::env::var_os("FFTA_TITLE_LANGUAGE_ROOT").unwrap());
    let state = fs::read(std::env::var_os("FFTA_TITLE_LANGUAGE_STATE").unwrap()).unwrap();
    let expected = std::env::var("FFTA_TITLE_LANGUAGE_STATE_SHA").unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), expected);
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let mut machine = crate::make_machine(rom, None);
    machine.load_state(&state).unwrap();
    assert_eq!(machine.bus.read8(0x0200_1f6e), 0, "expected ordinary English title selector");
    let mut selection = None;
    let mut publication = None;
    for frame in 0..160u32 {
        machine.bus.keys = if frame < 2 { 0x37f } else if (8..10).contains(&frame) { 0x3fe } else { 0x3ff };
        machine.bus.frame_ready = false;
        for _ in 0..5_000_000 {
            if machine.bus.frame_ready { break; }
            let pc = machine.cpu.regs[15]; let regs = machine.cpu.regs;
            let is_selection = pc == 0x0813_aa2a;
            let is_publication = pc == 0x0813_aafe;
            let pre = if is_publication { Some(machine.save_state().unwrap()) } else { None };
            let event = machine.step();
            if !matches!(event, gba_core::StepEvent::Instr(ref i) if i.addr == pc) { continue; }
            if is_selection {
                let context = machine.bus.read32(0x0200_f3b0);
                if regs[1] == context + 0x8964 && machine.bus.read8(regs[1]) == 1 {
                    selection = Some(serde_json::json!({"frame":frame,"context":format!("{context:08x}"),"address":format!("{:08x}",regs[1]),"value":1}));
                }
            }
            if is_publication {
                let context = machine.bus.read32(0x0200_f3b0);
                let selected = machine.bus.read8(context + 0x8964);
                assert_eq!(regs[1], 0x0200_1f6e);
                assert_eq!(regs[0] as u8, selected);
                assert_eq!(selected, 1);
                assert_eq!(machine.bus.read8(0x0200_1f6e), 1);
                let checkpoints = root.join("checkpoints"); fs::create_dir_all(&checkpoints).unwrap();
                fs::write(checkpoints.join("before-publication.ffstate"), pre.unwrap()).unwrap();
                fs::write(checkpoints.join("after-publication.ffstate"), machine.save_state().unwrap()).unwrap();
                publication = Some(serde_json::json!({"frame":frame,"pc":format!("{pc:08x}"),"context":format!("{context:08x}"),"selector_address":format!("{:08x}",context+0x8964),"selected":selected,"destination":format!("{:08x}",regs[1]),"published":machine.bus.read8(0x0200_1f6e)}));
                break;
            }
        }
        if publication.is_some() { break; }
        assert!(machine.bus.frame_ready, "title language observer frame limit");
    }
    assert!(selection.is_some(), "ordinary Down did not select French");
    assert!(publication.is_some(), "ordinary title publisher was not reached");
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS","scope":"ordinary EUR title language selector: Down, release, A; stop at original publication",
        "selection":selection,"publication":publication,"other_languages_and_writers":"pending",
        "Linux":"pending","USA":"pending","native_windows":"pending","performance_measurements":false
    })).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "authenticated EUR title selector and managed dataset; Wine only"]
fn european_title_language_other_publications_ordinary_pairs() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    use std::collections::BTreeSet;

    const GLOBAL: u32 = 0x0200_1f6e;
    const CURSOR: u32 = 0x003a_dcfc;
    const CONTEXT_PAIRS: u32 = 0x003a_e42c;
    let root = PathBuf::from(std::env::var_os("FFTA_TITLE_LANGUAGE_ROOT").unwrap());
    let state = fs::read(std::env::var_os("FFTA_TITLE_LANGUAGE_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "ce54d2693068161aa1452835109c7a44082fa9bd4e2777ab6976cddd88e192e1");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let dataset = PathBuf::from(std::env::var_os("FFTA_TITLE_LANGUAGE_DATASET").unwrap());
    let spans = [(CURSOR, CURSOR + 20), (CONTEXT_PAIRS, CONTEXT_PAIRS + 20)];
    let mut observations = Vec::new();

    // ID 1 already has its own ordinary publication witness. ID 4 is reached
    // by wrapping upward from zero; IDs 2 and 3 use separate Down presses.
    for target in [0u8, 2, 3, 4] {
        let mut raw = crate::make_machine(rom.clone(), None);
        let mut managed = crate::make_machine(rom.clone(), None);
        attach_from_directory(&mut managed, &dataset).unwrap();
        raw.load_state(&state).unwrap();
        managed.load_state(&state).unwrap();
        assert!(raw.save_state().unwrap() == managed.save_state().unwrap());
        let context = raw.bus.read32(0x0200_f3b0);
        assert_eq!(raw.bus.read8(GLOBAL), 0);
        assert_eq!(raw.bus.read8(context + 0x8964), 0);
        let presses = match target { 0 => 0, 4 => 1, _ => u32::from(target) };
        let confirm_frame = 8 * presses.max(1);
        let mut selected = BTreeSet::from([0u8]);
        let mut cursor_offsets = BTreeSet::new();
        let mut typed = 0u64;
        let mut raw_reads = 0u64;
        let mut publication = None;
        let mut unchanged_branch = false;
        let mut finished_frame = None;

        for frame in 0..160u32 {
            let keys = if frame < presses * 8 && frame % 8 < 2 {
                if target == 4 { 0x3bf } else { 0x37f }
            } else if (confirm_frame..confirm_frame + 2).contains(&frame) {
                0x3fe
            } else {
                0x3ff
            };
            raw.bus.keys = keys;
            managed.bus.keys = keys;
            raw.bus.frame_ready = false;
            for _ in 0..5_000_000 {
                if raw.bus.frame_ready { break; }
                let pc = raw.cpu.regs[15];
                let regs = raw.cpu.regs;
                let event = raw.step();
                if !matches!(event, gba_core::StepEvent::Instr(ref i) if i.addr == pc) { continue; }
                if pc == 0x0813_aadc && raw.cpu.regs[15] == 0x0813_ab30 {
                    assert_eq!((regs[0] as u8, regs[3] as u8), (0, 0));
                    unchanged_branch = true;
                }
                if pc == 0x0813_aafe {
                    assert_eq!(regs[1], GLOBAL);
                    assert_eq!(regs[0] as u8, target);
                    assert_eq!(raw.bus.read8(context + 0x8964), target);
                    assert_eq!(raw.bus.read8(GLOBAL), target);
                    publication = Some(frame);
                }
            }
            assert!(raw.bus.frame_ready, "raw title frame {frame}");
            for &(lo, hi) in &spans { managed.bus.rom[lo as usize..hi as usize].fill(0xcd); }
            let capture = TraceCapture::start(TraceFilter { readers: vec![], ranges: spans.to_vec() }, 10_000).unwrap();
            capture.frame(Some(managed.bus.frames));
            managed.bus.frame_ready = false;
            for _ in 0..5_000_000 {
                if managed.bus.frame_ready { break; }
                managed.step();
            }
            assert!(managed.bus.frame_ready, "managed title frame {frame}");
            let trace = capture.finish();
            assert!(!trace.truncated);
            for event in trace.events {
                match event.key.service {
                    Service::Typed => {
                        typed += event.requests;
                        if event.key.read.length == 2 { cursor_offsets.insert(event.key.read.physical_offset); }
                    }
                    Service::Raw => raw_reads += event.requests,
                    Service::Unclassified => {}
                }
            }
            for &(lo, hi) in &spans {
                managed.bus.rom[lo as usize..hi as usize].copy_from_slice(&rom[lo as usize..hi as usize]);
            }
            assert!(raw.save_state().unwrap() == managed.save_state().unwrap(),
                    "title full state/clock target {target}, frame {frame}");
            selected.insert(raw.bus.read8(context + 0x8964));
            if publication.is_some() || unchanged_branch {
                finished_frame = Some(frame);
                break;
            }
        }
        let frame = finished_frame.expect("title confirmation did not reach the publisher or unchanged branch");
        assert_eq!(raw.bus.read8(context + 0x8964), target);
        assert_eq!(raw.bus.read8(GLOBAL), target);
        assert_eq!(publication.is_some(), target != 0);
        assert_eq!(unchanged_branch, target == 0);
        assert!(typed > 0 && raw_reads == 0);
        let cursor = CURSOR + 4 * u32::from(target);
        assert!(cursor_offsets.contains(&cursor) && cursor_offsets.contains(&(cursor + 2)),
                "both selected cursor coordinates for target {target}");
        if target == 4 { assert!(selected.contains(&4), "upward wrap was not observed"); }
        else { for value in 0..=target { assert!(selected.contains(&value)); } }
        observations.push(serde_json::json!({
            "target": target, "selection_values": selected, "frame": frame,
            "published": publication.is_some(), "unchanged_branch": unchanged_branch,
            "typed_requests": typed, "raw_requests": raw_reads,
        }));
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status": "PASS", "scope": "ordinary EUR title selector, interpreter raw/managed pairs",
        "observations": observations, "poisoned_source": true,
        "limits": "ID 0 is already selected and follows the unchanged branch; publication of zero after another selection, global writer/value domain and other scenes remain open.",
        "Linux": "pending", "USA": "pending", "native_windows": "pending", "performance_measurements": false
    })).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "EUR French title checkpoint and managed dataset; Wine only"]
fn european_title_language_return_to_english_ordinary_pair() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    use std::collections::BTreeSet;

    const GLOBAL: u32 = 0x0200_1f6e;
    const CURSOR: u32 = 0x003a_dcfc;
    const CONTEXT_PAIRS: u32 = 0x003a_e42c;
    let root = PathBuf::from(std::env::var_os("FFTA_TITLE_LANGUAGE_ROOT").unwrap());
    let state = fs::read(std::env::var_os("FFTA_TITLE_LANGUAGE_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "ccece459942747b2460d85f6a459ecd34bbb107afb49d25049dc9d3c25ca9749");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let dataset = PathBuf::from(std::env::var_os("FFTA_TITLE_LANGUAGE_DATASET").unwrap());
    let spans = [(CURSOR, CURSOR + 20), (CONTEXT_PAIRS, CONTEXT_PAIRS + 20)];
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &dataset).unwrap();
    raw.load_state(&state).unwrap();
    managed.load_state(&state).unwrap();
    assert!(raw.save_state().unwrap() == managed.save_state().unwrap());
    assert_eq!(raw.bus.read8(GLOBAL), 1, "French must still be active at the ordinary title");

    let mut selection = None;
    let mut publication = None;
    let mut typed = 0u64;
    let mut raw_reads = 0u64;
    let mut cursor_offsets = BTreeSet::new();
    for frame in 0..160u32 {
        // From the visible Saved Game entry: Down twice, A, then return from
        // French to English with Up and A. Gaps allow each key to be released.
        let keys = match frame {
            0..6 | 12..18 => 0x37f,
            24..30 | 102..108 => 0x3fe,
            90..96 => 0x3bf,
            _ => 0x3ff,
        };
        raw.bus.keys = keys;
        managed.bus.keys = keys;
        raw.bus.frame_ready = false;
        for _ in 0..5_000_000 {
            if raw.bus.frame_ready { break; }
            let pc = raw.cpu.regs[15];
            let regs = raw.cpu.regs;
            let event = raw.step();
            if !matches!(event, gba_core::StepEvent::Instr(ref i) if i.addr == pc) { continue; }
            if pc == 0x0813_aa52 && frame >= 90 {
                let context = raw.bus.read32(0x0200_f3b0);
                if regs[1] == context + 0x8964 && raw.bus.read8(regs[1]) == 0 {
                    selection = Some(frame);
                }
            }
            if pc == 0x0813_aafe {
                let context = raw.bus.read32(0x0200_f3b0);
                assert!(frame >= 102, "unexpected earlier language publication");
                assert_eq!(regs[1], GLOBAL);
                assert_eq!(regs[0] as u8, 0);
                assert_eq!(raw.bus.read8(context + 0x8964), 0);
                assert_eq!(raw.bus.read8(GLOBAL), 0);
                publication = Some(frame);
            }
        }
        assert!(raw.bus.frame_ready, "raw title frame {frame}");
        if frame == 89 {
            let context = raw.bus.read32(0x0200_f3b0);
            assert_eq!(raw.bus.read8(context + 0x8994), 2, "language selector did not open");
            assert_eq!(raw.bus.read8(context + 0x8964), 1, "selector must begin on French");
            assert_eq!(raw.bus.read8(GLOBAL), 1);
        }
        for &(lo, hi) in &spans { managed.bus.rom[lo as usize..hi as usize].fill(0xcd); }
        let capture = TraceCapture::start(TraceFilter { readers: vec![], ranges: spans.to_vec() }, 10_000).unwrap();
        capture.frame(Some(managed.bus.frames));
        managed.bus.frame_ready = false;
        for _ in 0..5_000_000 {
            if managed.bus.frame_ready { break; }
            managed.step();
        }
        assert!(managed.bus.frame_ready, "managed title frame {frame}");
        let trace = capture.finish();
        assert!(!trace.truncated);
        for event in trace.events {
            match event.key.service {
                Service::Typed => {
                    typed += event.requests;
                    if event.key.read.length == 2 { cursor_offsets.insert(event.key.read.physical_offset); }
                }
                Service::Raw => raw_reads += event.requests,
                Service::Unclassified => {}
            }
        }
        for &(lo, hi) in &spans {
            managed.bus.rom[lo as usize..hi as usize].copy_from_slice(&rom[lo as usize..hi as usize]);
        }
        assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "title full state/clock frame {frame}");
        if publication.is_some() { break; }
    }
    let selection = selection.expect("ordinary Up did not select English");
    let publication = publication.expect("ordinary English publication was not reached");
    assert!(selection < publication);
    assert!(typed > 0 && raw_reads == 0);
    assert!(cursor_offsets.contains(&CURSOR) && cursor_offsets.contains(&(CURSOR + 2)),
            "both English cursor coordinates were not read");
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status": "PASS", "scope": "EUR title: return from French to English; interpreter raw/managed pair",
        "selection_frame": selection, "publication_frame": publication,
        "typed_requests": typed, "raw_requests": raw_reads, "poisoned_source": true,
        "limits": "One ordinary title route; other writers, global value domain, native dispatch and other scenes remain open.",
        "Linux": "pending", "USA": "pending", "native_windows": "pending", "performance_measurements": false
    })).unwrap()).unwrap();
}
