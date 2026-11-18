//! EUR range lookup publication and original scan.
use super::*;
use gba_core::{Bus, Machine, capi::RT_API};
use gba_core::rom_data::range_lookup::{RECORD_COUNT, SOURCE, SOURCE_SHA256};
use recomp_core::analyze::{Analysis, Block, View};

#[test]
#[ignore = "exact EUR ROM, e160 dataset and TinyCC; Wine only, Linux/USA pending"]
fn european_range_lookup_publication_and_original_scan() {
    let root = PathBuf::from(std::env::var_os("FFTA_RANGE_LOOKUP_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_RANGE_LOOKUP_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (160, 159));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision), (161, 160));
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    for prior in &old.resources {
        let current = source.manifest.resources.iter().find(|r| r.id == prior.id).unwrap();
        assert_eq!(current, prior);
        assert_eq!(fs::read(prepared.directory.join(&current.path)).unwrap(), fs::read(previous.join(&prior.path)).unwrap());
    }
    assert_eq!(source.manifest.resources.len(), old.resources.len() + 1);
    let entry = source.manifest.resources.iter().find(|r| r.id == RANGE_LOOKUP_ID).unwrap();
    assert_eq!((entry.sources.len(), entry.sources[0].offset, entry.sources[0].length, entry.sources[0].sha256.as_str()), (1, SOURCE, 88, SOURCE_SHA256));
    let model = &source.range_lookup.as_ref().unwrap().decoded;
    assert_eq!(model.records.len(), RECORD_COUNT);
    assert_eq!(model.source_bytes().unwrap(), rom[SOURCE as usize..SOURCE as usize + 88]);
    let mut bad = model.clone(); bad.records.pop(); assert!(bad.validate().is_err());
    let mut edited = model.clone(); edited.records[0].opaque_01 ^= 1; edited.validate().unwrap(); assert!(edited.verify_source(&rom).is_err());
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let start = SOURCE as usize; let end = start + 88;
    managed.bus.rom[start..end].fill(0xcd);
    let before = owner.range_lookup_stats().unwrap().source_bytes;
    for at in start..end { assert_eq!(managed.bus.read8(0x0c00_0000 + at as u32), rom[at]); }
    assert_eq!(owner.range_lookup_stats().unwrap().source_bytes - before, 88);
    let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for edge in [start, end] { for delta in [-3isize, -1, 0, 1] {
        let at = edge.wrapping_add_signed(delta);
        assert_eq!((RT_API.read16)(ptr, 0x0a00_0000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at+2].try_into().unwrap())));
        assert_eq!((RT_API.read32)(ptr, 0x0800_0000 + at as u32), u32::from_le_bytes(rom[at..at+4].try_into().unwrap()));
    }}
    managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = (0x0801_abc0..0x0801_abfa).step_by(2)
        .map(|addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("range-lookup.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let initial = raw.save_state().unwrap();
    let mut values: Vec<u16> = model.records.iter().flat_map(|r| [r.lower, r.upper]).collect();
    if let Some(value) = (0..=u16::MAX).find(|v| model.select(*v).is_none()) { values.push(value); }
    let mut cases = 0;
    for native in [false, true] { for &value in &values {
        for m in [&mut raw, &mut managed] {
            m.load_state(&initial).unwrap();
            m.bus.ffta_sprites.enabled = false;
            m.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
            m.cpu.regs[0] = u32::from(value);
            m.cpu.regs[1] = 0x0201_0000;
            m.cpu.regs[2] = 0x0201_0002;
            m.cpu.regs[13] = 0x0300_7c00;
            m.cpu.regs[14] = 0x0800_0301;
            m.cpu.regs[15] = 0x0801_abc0;
            m.cpu.cpsr = 0xbf;
            m.bus.write8(0x0201_0000, 0xa6);
            m.bus.write16(0x0201_0002, 0xb7c8);
        }
        managed.bus.rom[start..end].fill(0xcd);
        let before = owner.range_lookup_stats().unwrap().source_bytes;
        let mut steps = 0;
        while raw.cpu.regs[15] != 0x0800_0300 { assert!(steps < 300); raw.step(); steps += 1; }
        steps = 0;
        while managed.cpu.regs[15] != 0x0800_0300 {
            assert!(steps < 300, "scan pc {:08x}", managed.cpu.regs[15]);
            if native { table.get(managed.cpu.regs[15] | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void); }
            else { managed.step(); }
            steps += 1;
        }
        assert!(owner.range_lookup_stats().unwrap().source_bytes > before);
        if let Some((selector, offset)) = model.select(value) {
            assert_eq!(raw.bus.read8(0x0201_0000), selector);
            assert_eq!(raw.bus.read16(0x0201_0002), offset);
        } else {
            assert_eq!(raw.bus.read8(0x0201_0000), 0xa6);
            assert_eq!(raw.bus.read16(0x0201_0002), 0xb7c8);
        }
        managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
        assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap(), "value {value}, native {native}");
        cases += 1;
    }}
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[161,160], "previous_resources_preserved":old.resources.len(),
        "records":11, "owned_bytes":88, "source_sha256":SOURCE_SHA256, "poisoned_raw_rom":true,
        "original_scan_cases":cases, "compiled_dispatch":"may use interpreter fallback", "full_state_clock_equal":true,
        "ordinary_use":"pending", "production_cli":"pending", "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "authenticated EUR battle state/input and current dataset; Wine only"]
fn european_encounter_special_descriptor_ordinary_observation() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};

    const ROW1: usize = 0x006e_6f5e;
    const READER: u32 = 0x080c_d43c;
    let root = PathBuf::from(std::env::var_os("FFTA_RANGE_LOOKUP_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_RANGE_LOOKUP_DATASET").unwrap());
    let state = fs::read(std::env::var_os("FFTA_STATE_TEST_STATE").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&state),
        "d18f2e3d80071a8a99414089e96dcceff08634d23e171d1caa4e064293012635"
    );
    let input_path = std::env::var("FFTA_STATE_TEST_INPUT").unwrap();
    let input_bytes = fs::read(&input_path).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&input_bytes),
        "51c3e491ac87b6017fc06310e2ba4d1b3b03f12690f7c32c0ffd029991452da1"
    );
    let input = crate::InputScript::load(&input_path).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &dataset).unwrap();
    managed.load_state(&state).unwrap();
    let first_frame = managed.bus.frames;
    let owner = managed.bus.rom_resources().unwrap().clone();
    let mut outcome = None;
    let mut typed_total = 0u64;

    for frame in 0..700u64 {
        managed.bus.keys = input.keys_at(frame);
        input.apply_mouse(&mut managed.bus, frame).unwrap();
        managed.bus.audio_buf.clear();
        managed.bus.psg_tap.clear();
        for channel in &mut managed.bus.fifo_tap {
            channel.clear();
        }
        managed.bus.hle_tap.clear();
        let before = managed.save_state().unwrap();
        managed.bus.rom[ROW1..ROW1 + 14].fill(0xa7);
        let capture = TraceCapture::start(
            TraceFilter {
                readers: vec![],
                ranges: vec![(ROW1 as u32, ROW1 as u32 + 14)],
            },
            100,
        )
        .unwrap();
        capture.frame(Some(managed.bus.frames));
        managed.bus.frame_ready = false;
        let mut reader_seen = false;
        for _ in 0..5_000_000 {
            if managed.bus.frame_ready {
                break;
            }
            if managed.cpu.regs[15] == READER && managed.cpu.regs[1] == 0x0800_0000 + ROW1 as u32 {
                reader_seen = true;
            }
            managed.step();
        }
        assert!(managed.bus.frame_ready, "ordinary alias frame limit");
        let trace = capture.finish();
        assert!(!trace.truncated);
        let typed: u64 = trace
            .events
            .iter()
            .filter(|event| event.key.service == Service::Typed)
            .map(|event| event.requests)
            .sum();
        let raw_reads: u64 = trace
            .events
            .iter()
            .filter(|event| event.key.service == Service::Raw)
            .map(|event| event.requests)
            .sum();
        typed_total += typed;
        managed.bus.rom[ROW1..ROW1 + 14].copy_from_slice(&rom[ROW1..ROW1 + 14]);
        assert_eq!(raw_reads, 0, "raw fallback within selected row");
        if reader_seen {
            assert!(
                trace
                    .events
                    .iter()
                    .any(|event| event.key.service == Service::Typed
                        && event.key.read.physical_offset == ROW1 as u32
                        && event.key.instruction.is_some_and(|site| site.pc == READER)),
                "original reader was reached without its typed row trace"
            );
            let mut raw = crate::make_machine(rom.clone(), None);
            raw.load_state(&before).unwrap();
            raw.bus.frame_ready = false;
            for _ in 0..5_000_000 {
                if raw.bus.frame_ready {
                    break;
                }
                raw.step();
            }
            assert!(raw.bus.frame_ready);
            assert!(
                raw.save_state().unwrap() == managed.save_state().unwrap(),
                "ordinary alias full state/clock"
            );
            assert!(
                raw.bus.framebuffer == managed.bus.framebuffer
                    && raw.bus.wide_framebuffer == managed.bus.wide_framebuffer
                    && raw.bus.oam == managed.bus.oam
                    && raw.bus.audio_buf == managed.bus.audio_buf,
                "ordinary alias image/OAM/mixed audio"
            );
            outcome = Some(
                serde_json::json!({"relative_frame":frame,"guest_frame":first_frame + frame,
                "typed_row_requests_in_frame":typed,"raw_row_requests_in_frame":raw_reads,
                "paired_full_state_clock_image_oam_mixed_audio_frames":1}),
            );
            break;
        }
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"bounded EUR Giza-to-White-Flowers input, interpreter, up to 700 frames",
        "reader_hit":outcome, "typed_row_requests_total":typed_total,
        "packed_source_bytes_total":owner.range_lookup_stats().unwrap().source_bytes,
        "source_rom_poisoned":true, "negative_is_not_global_absence":true,
        "limitations":"One retained route; no all-consumer, all-scene or translated-dispatch claim.",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "authenticated EUR Bank state/input, E170 dataset and native library; Wine only"]
fn european_range_lookup_bank_select_ordinary_pair() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    crate::configure_headless_widescreen();
    const ROW: usize = SOURCE as usize;
    const LEN: usize = RECORD_COUNT * 8;
    let root = PathBuf::from(std::env::var_os("FFTA_RANGE_LOOKUP_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_RANGE_LOOKUP_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let state = fs::read(std::env::var_os("FFTA_STATE_TEST_STATE").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&state),
        "d18f2e3d80071a8a99414089e96dcceff08634d23e171d1caa4e064293012635"
    );
    let input_path = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_INPUT").unwrap());
    assert_eq!(
        gba_core::ffta_rom::digest(&fs::read(&input_path).unwrap()),
        "51c3e491ac87b6017fc06310e2ba4d1b3b03f12690f7c32c0ffd029991452da1"
    );
    let input = crate::InputScript::load(input_path.to_str().unwrap()).unwrap();
    let manifest: Manifest =
        toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (170, 169)
    );
    let (_library, table) = crate::load_native(Path::new(
        &std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap(),
    ))
    .unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &dataset).unwrap();
    for m in [&mut raw, &mut managed] {
        m.load_state(&state).unwrap();
        crate::arm_audio_hle(m, None);
        m.bus.tap_channels = true;
        m.bus.audio_buf.clear();
        m.bus.psg_tap.clear();
        m.bus.fifo_tap[0].clear();
        m.bus.fifo_tap[1].clear();
        m.bus.hle_tap.clear();
    }
    assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
    let owner = managed.bus.rom_resources().unwrap().clone();
    let before = owner.range_lookup_stats().unwrap().source_bytes;
    let trace = TraceCapture::start(
        TraceFilter {
            readers: vec![],
            ranges: vec![(ROW as u32, (ROW + LEN) as u32)],
        },
        10000,
    )
    .unwrap();
    let start = raw.bus.frames;
    let mut audio = [0u64; 5];
    let mut dispatch = (0u64, 0u64);
    for frame in 0..120u64 {
        managed.bus.rom[ROW..ROW + LEN].fill(0xa7);
        let mut counts = [(0, 0); 2];
        for (arm, m) in [&mut raw, &mut managed].into_iter().enumerate() {
            m.bus.keys = input.keys_at(frame);
            input.apply_mouse(&mut m.bus, frame).unwrap();
            trace.frame(if arm == 1 { Some(m.bus.frames) } else { None });
            let ptr = m as *mut Machine as *mut std::ffi::c_void;
            counts[arm] = crate::run_frame_native(m, &table, ptr, crate::FRAME_STEP_GUARD);
            assert_eq!(m.bus.frames, start + frame + 1);
        }
        trace.frame(None);
        assert_eq!(counts[0], counts[1], "dispatch at {frame}");
        dispatch.0 += counts[0].0;
        dispatch.1 += counts[0].1;
        assert_eq!(raw.bus.clock, managed.bus.clock);
        assert_eq!(raw.bus.framebuffer, managed.bus.framebuffer);
        assert_eq!(raw.bus.wide_framebuffer, managed.bus.wide_framebuffer);
        assert_eq!(raw.bus.oam, managed.bus.oam);
        assert_eq!(raw.bus.audio_buf, managed.bus.audio_buf);
        assert_eq!(raw.bus.psg_tap, managed.bus.psg_tap);
        assert_eq!(raw.bus.fifo_tap, managed.bus.fifo_tap);
        assert_eq!(raw.bus.hle_tap.len(), managed.bus.hle_tap.len());
        assert!(raw
            .bus
            .hle_tap
            .iter()
            .zip(&managed.bus.hle_tap)
            .all(|(a, b)| a.to_bits() == b.to_bits()));
        for (sum, len) in audio.iter_mut().zip([
            managed.bus.audio_buf.len(),
            managed.bus.psg_tap.len(),
            managed.bus.fifo_tap[0].len(),
            managed.bus.fifo_tap[1].len(),
            managed.bus.hle_tap.len(),
        ]) {
            *sum += len as u64;
        }
        managed.bus.rom[ROW..ROW + LEN].copy_from_slice(&rom[ROW..ROW + LEN]);
        assert_eq!(
            raw.save_state().unwrap(),
            managed.save_state().unwrap(),
            "full state at {frame}"
        );
        for m in [&mut raw, &mut managed] {
            m.bus.audio_buf.clear();
            m.bus.psg_tap.clear();
            m.bus.fifo_tap[0].clear();
            m.bus.fifo_tap[1].clear();
            m.bus.hle_tap.clear();
        }
    }
    let trace = trace.finish();
    assert!(!trace.truncated && trace.dropped_requests == 0);
    let typed: u64 = trace
        .events
        .iter()
        .filter(|e| e.key.service == Service::Typed)
        .map(|e| e.requests)
        .sum();
    let raw_reads: u64 = trace
        .events
        .iter()
        .filter(|e| e.key.service == Service::Raw)
        .map(|e| e.requests)
        .sum();
    let unclassified: u64 = trace
        .events
        .iter()
        .filter(|e| e.key.service == Service::Unclassified)
        .map(|e| e.requests)
        .sum();
    let bytes = owner.range_lookup_stats().unwrap().source_bytes - before;
    let records = owner.range_lookup_stats().unwrap().records_used;
    assert!(
        typed > 0 && bytes > 0 && records > 0,
        "ordinary range lookup must be consumed"
    );
    assert_eq!((raw_reads, unclassified), (0, 0));
    assert!(audio.iter().all(|&v| v > 0), "audio counts: {audio:?}");
    assert!(dispatch.0 > 0);
    fs::write(
        root.join("range-trace.json"),
        serde_json::to_vec_pretty(&trace).unwrap(),
    )
    .unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"120-frame source-guided EUR Bank Select raw/managed translated pair",
        "revision":[170,169], "frames":120, "typed_requests":typed,
        "raw_requests":raw_reads, "unclassified_requests":unclassified,
        "source_bytes":bytes, "records_used":records, "trace_dropped":trace.dropped_requests,
        "all_88_source_bytes_poisoned":true, "full_state_clock_video_oam_audio_equal_each_frame":true,
        "audio_sample_counts":audio, "generated_dispatches":dispatch.0, "fallback_dispatches":dispatch.1,
        "limits":"Generated blocks can fall back to interpretation; aggregate attribution and other reader contexts remain open.",
        "performance_measurements":false
    })).unwrap()).unwrap();
}
#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "authenticated EUR Bank state/input, E170 dataset and native library; Wine only"]
fn european_range_lookup_bank_select_interpreter_attribution() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    crate::configure_headless_widescreen();
    const ROW: usize = SOURCE as usize;
    const LEN: usize = RECORD_COUNT * 8;
    let root = PathBuf::from(std::env::var_os("FFTA_RANGE_LOOKUP_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_RANGE_LOOKUP_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let state = fs::read(std::env::var_os("FFTA_STATE_TEST_STATE").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&state),
        "d18f2e3d80071a8a99414089e96dcceff08634d23e171d1caa4e064293012635"
    );
    let input_path = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_INPUT").unwrap());
    assert_eq!(
        gba_core::ffta_rom::digest(&fs::read(&input_path).unwrap()),
        "51c3e491ac87b6017fc06310e2ba4d1b3b03f12690f7c32c0ffd029991452da1"
    );
    let input = crate::InputScript::load(input_path.to_str().unwrap()).unwrap();
    let manifest: Manifest =
        toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (170, 169)
    );
    let (_library, table) = crate::load_native(Path::new(
        &std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap(),
    ))
    .unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &dataset).unwrap();
    for m in [&mut raw, &mut managed] {
        m.load_state(&state).unwrap();
        crate::arm_audio_hle(m, None);
        m.bus.tap_channels = true;
        m.bus.audio_buf.clear();
        m.bus.psg_tap.clear();
        m.bus.fifo_tap[0].clear();
        m.bus.fifo_tap[1].clear();
        m.bus.hle_tap.clear();
    }
    let first = raw.bus.frames;
    assert_eq!(first, 3660);
    for frame in 0..9u64 {
        for m in [&mut raw, &mut managed] {
            m.bus.keys = input.keys_at(frame);
            input.apply_mouse(&mut m.bus, frame).unwrap();
            let ptr = m as *mut Machine as *mut std::ffi::c_void;
            crate::run_frame_native(m, &table, ptr, crate::FRAME_STEP_GUARD);
            assert_eq!(m.bus.frames, first + frame + 1);
        }
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
        for m in [&mut raw, &mut managed] {
            m.bus.audio_buf.clear();
            m.bus.psg_tap.clear();
            m.bus.fifo_tap[0].clear();
            m.bus.fifo_tap[1].clear();
            m.bus.hle_tap.clear();
        }
    }
    for m in [&mut raw, &mut managed] {
        m.bus.keys = input.keys_at(9);
        input.apply_mouse(&mut m.bus, 9).unwrap();
    }
    raw.bus.frame_ready = false;
    for _ in 0..5_000_000 {
        if raw.bus.frame_ready {
            break;
        }
        raw.step();
    }
    assert!(raw.bus.frame_ready);
    managed.bus.rom[ROW..ROW + LEN].fill(0xa7);
    managed.bus.frame_ready = false;
    let owner = managed.bus.rom_resources().unwrap().clone();
    let before = owner.range_lookup_stats().unwrap().source_bytes;
    let trace = TraceCapture::start(
        TraceFilter {
            readers: vec![],
            ranges: vec![(SOURCE, SOURCE + LEN as u32)],
        },
        10000,
    )
    .unwrap();
    trace.frame(Some(managed.bus.frames));
    let mut visited = std::collections::BTreeSet::new();
    for _ in 0..5_000_000 {
        if managed.bus.frame_ready {
            break;
        }
        let pc = managed.cpu.regs[15];
        if [
            0x0805_be34,
            0x0805_be8a,
            0x0801_abfc,
            0x0801_ac54,
            0x0801_abc0,
        ]
        .contains(&pc)
        {
            visited.insert(pc);
        }
        managed.step();
    }
    assert!(managed.bus.frame_ready);
    trace.frame(None);
    let trace = trace.finish();
    assert!(!trace.truncated && trace.dropped_requests == 0);
    assert_eq!(managed.bus.frames, first + 10);
    assert_eq!(raw.bus.frames, managed.bus.frames);
    let typed: u64 = trace
        .events
        .iter()
        .filter(|e| e.key.service == Service::Typed)
        .map(|e| e.requests)
        .sum();
    let other: u64 = trace
        .events
        .iter()
        .filter(|e| e.key.service != Service::Typed)
        .map(|e| e.requests)
        .sum();
    let sites: std::collections::BTreeSet<_> = trace
        .events
        .iter()
        .filter_map(|e| e.key.instruction.map(|s| s.pc))
        .collect();
    assert!(typed > 0 && owner.range_lookup_stats().unwrap().source_bytes > before);
    assert_eq!(other, 0);
    assert!(visited.contains(&0x0801_abc0));
    assert!(
        visited.contains(&0x0805_be34) || visited.contains(&0x0805_be8a),
        "caller PCs: {visited:x?}"
    );
    assert!(
        sites.contains(&0x0801_abce) || sites.contains(&0x0801_abd4),
        "reader PCs: {sites:x?}"
    );
    assert_eq!(raw.bus.clock, managed.bus.clock);
    assert_eq!(raw.bus.framebuffer, managed.bus.framebuffer);
    assert_eq!(raw.bus.wide_framebuffer, managed.bus.wide_framebuffer);
    assert_eq!(raw.bus.oam, managed.bus.oam);
    assert_eq!(raw.bus.audio_buf, managed.bus.audio_buf);
    assert_eq!(raw.bus.psg_tap, managed.bus.psg_tap);
    assert_eq!(raw.bus.fifo_tap, managed.bus.fifo_tap);
    assert_eq!(raw.bus.hle_tap.len(), managed.bus.hle_tap.len());
    assert!(raw
        .bus
        .hle_tap
        .iter()
        .zip(&managed.bus.hle_tap)
        .all(|(a, b)| a.to_bits() == b.to_bits()));
    managed.bus.rom[ROW..ROW + LEN].copy_from_slice(&rom[ROW..ROW + LEN]);
    assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
    fs::write(
        root.join("range-site-trace.json"),
        serde_json::to_vec_pretty(&trace).unwrap(),
    )
    .unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"EUR Bank Select frame 3669 raw/managed interpreter reader/caller attribution after native prefix",
        "revision":[170,169], "guest_frame":3669, "typed_requests":typed,
        "other_requests":other, "trace_dropped":trace.dropped_requests,
        "visited_pcs":visited, "reader_sites":sites,
        "managed_source_poisoned":true, "full_state_clock_video_oam_audio_equal":true,
        "limits":"One interpreter frame after a generated-dispatch prefix from a retained checkpoint. Other readers, scenes and checkpoint ancestry remain unproved; cross-mode frame-boundary clocks are not compared here.",
        "performance_measurements":false
    })).unwrap()).unwrap();
}
#[test]
#[ignore = "authenticated EUR Bank state/input and selected native library; Wine only"]
fn european_range_lookup_raw_frame_boundary_alignment() {
    crate::configure_headless_widescreen();
    let root = PathBuf::from(std::env::var_os("FFTA_RANGE_LOOKUP_ROOT").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let state = fs::read(std::env::var_os("FFTA_STATE_TEST_STATE").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&state),
        "d18f2e3d80071a8a99414089e96dcceff08634d23e171d1caa4e064293012635"
    );
    let input_path = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_INPUT").unwrap());
    assert_eq!(
        gba_core::ffta_rom::digest(&fs::read(&input_path).unwrap()),
        "51c3e491ac87b6017fc06310e2ba4d1b3b03f12690f7c32c0ffd029991452da1"
    );
    let input = crate::InputScript::load(input_path.to_str().unwrap()).unwrap();
    let (_library, table) = crate::load_native(Path::new(
        &std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap(),
    ))
    .unwrap();
    let mut native = crate::make_machine(rom.clone(), None);
    let mut interpreted = crate::make_machine(rom, None);
    for m in [&mut native, &mut interpreted] {
        m.load_state(&state).unwrap();
        crate::arm_audio_hle(m, None);
        m.bus.tap_channels = true;
        m.bus.audio_buf.clear();
        m.bus.psg_tap.clear();
        m.bus.fifo_tap[0].clear();
        m.bus.fifo_tap[1].clear();
        m.bus.hle_tap.clear();
    }
    let first = native.bus.frames;
    assert_eq!(first, 3660);
    for frame in 0..9u64 {
        for m in [&mut native, &mut interpreted] {
            m.bus.keys = input.keys_at(frame);
            input.apply_mouse(&mut m.bus, frame).unwrap();
            let ptr = m as *mut Machine as *mut std::ffi::c_void;
            crate::run_frame_native(m, &table, ptr, crate::FRAME_STEP_GUARD);
            assert_eq!(m.bus.frames, first + frame + 1);
        }
        assert_eq!(
            native.save_state().unwrap(),
            interpreted.save_state().unwrap()
        );
        for m in [&mut native, &mut interpreted] {
            m.bus.audio_buf.clear();
            m.bus.psg_tap.clear();
            m.bus.fifo_tap[0].clear();
            m.bus.fifo_tap[1].clear();
            m.bus.hle_tap.clear();
        }
    }
    for m in [&mut native, &mut interpreted] {
        m.bus.keys = input.keys_at(9);
        input.apply_mouse(&mut m.bus, 9).unwrap();
    }
    assert_eq!(
        native.save_state().unwrap(),
        interpreted.save_state().unwrap()
    );
    let describe = |m: &Machine| serde_json::json!({
        "pc":m.cpu.regs[15], "cpsr":m.cpu.cpsr, "registers":m.cpu.regs,
        "clock":m.bus.clock, "frames":m.bus.frames, "frame_ready":m.bus.frame_ready,
        "halted":m.bus.halted, "ie":m.bus.reg_ie, "if":m.bus.reg_if,
        "ime":m.bus.ime, "irq_pending":m.bus.irq_pending(),
        "irq_masked":m.cpu.flag(gba_core::cpu::FLAG_I)
    });
    let ptr = &mut native as *mut Machine as *mut std::ffi::c_void;
    let mut native_tail = std::collections::VecDeque::new();
    let dispatch = crate::run_frame_native_traced(
        &mut native, &table, ptr, crate::FRAME_STEP_GUARD,
        |m, kind| {
            if native_tail.len() == 8 { native_tail.pop_front(); }
            native_tail.push_back(serde_json::json!({"kind":format!("{kind:?}"),"state":describe(m)}));
        }, |_,_,_| {},
    );
    interpreted.bus.frame_ready = false;
    for _ in 0..5_000_000 {
        if interpreted.bus.frame_ready {
            break;
        }
        interpreted.step();
    }
    assert!(interpreted.bus.frame_ready);
    assert_eq!(native.bus.frames, first + 10);
    assert_eq!(interpreted.bus.frames, native.bus.frames);
    let native_clock = native.bus.clock;
    let first_interpreter_clock = interpreted.bus.clock;
    let native_endpoint = describe(&native);
    let interpreter_endpoint = describe(&interpreted);
    fs::write(root.join("native-end.ffstate"), native.save_state().unwrap()).unwrap();
    fs::write(root.join("interpreter-end.ffstate"), interpreted.save_state().unwrap()).unwrap();
    let mut extra_steps = 0u32;
    let mut extra_events = Vec::new();
    while interpreted.bus.clock < native_clock && extra_steps < 1000 {
        let event = interpreted.step();
        extra_steps += 1;
        extra_events.push(serde_json::json!({"idle":matches!(event, gba_core::machine::StepEvent::Idle),"state":describe(&interpreted)}));
    }
    let aligned = interpreted.bus.clock == native_clock;
    let native_state = native.save_state().unwrap();
    let interpreted_state = interpreted.save_state().unwrap();
    let state_equal = aligned && native_state == interpreted_state;
    let audio_equal = native.bus.audio_buf == interpreted.bus.audio_buf
        && native.bus.psg_tap == interpreted.bus.psg_tap
        && native.bus.fifo_tap == interpreted.bus.fifo_tap
        && native.bus.hle_tap.len() == interpreted.bus.hle_tap.len()
        && native
            .bus
            .hle_tap
            .iter()
            .zip(&interpreted.bus.hle_tap)
            .all(|(a, b)| a.to_bits() == b.to_bits());
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"one EUR Bank Select raw/raw selected-native versus interpreter frame-boundary observation",
        "guest_frame":3669, "native_clock":native_clock, "interpreter_first_frame_clock":first_interpreter_clock,
        "interpreter_final_clock":interpreted.bus.clock, "extra_interpreter_steps":extra_steps,
        "aligned":aligned, "complete_state_equal_at_aligned_clock":state_equal,
        "audio_buffers_equal_at_observed_endpoints":audio_equal, "native_pc":native.cpu.regs[15],
        "interpreter_pc":interpreted.cpu.regs[15], "native_state_sha256":gba_core::ffta_rom::digest(&native_state),
        "interpreter_state_sha256":gba_core::ffta_rom::digest(&interpreted_state),
        "native_dispatches":dispatch.0, "fallback_dispatches":dispatch.1,
        "native_endpoint":native_endpoint, "interpreter_frame_endpoint":interpreter_endpoint,
        "native_last_dispatches":native_tail, "interpreter_extra_events":extra_events,
        "limits":"A bounded raw/raw observation using the selected native object; state comparison is reported, not assumed. Other scenes and production cache selection remain untested.",
        "performance_measurements":false
    })).unwrap()).unwrap();
}
#[test]
#[ignore = "authenticated EUR ROM, E170 dataset and TinyCC; Wine only"]
fn european_range_lookup_both_original_wrapper_prefixes() {
    let root = PathBuf::from(std::env::var_os("FFTA_RANGE_LOOKUP_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_RANGE_LOOKUP_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let manifest: Manifest =
        toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (170, 169)
    );
    let entry = manifest
        .resources
        .iter()
        .find(|r| r.id == RANGE_LOOKUP_ID)
        .unwrap();
    let model = parse_range_lookup(&fs::read(dataset.join(&entry.path)).unwrap()).unwrap();
    assert_eq!(
        model.source_bytes().unwrap(),
        rom[SOURCE as usize..SOURCE as usize + 88]
    );
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let addresses = (0x0801_abc0..0x0801_abfa)
        .step_by(2)
        .chain((0x0801_abfc..0x0801_ac1c).step_by(2))
        .chain((0x0801_ac54..0x0801_ac7c).step_by(2));
    let blocks = addresses
        .map(|addr| Block {
            addr,
            thumb: true,
            instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
        })
        .collect();
    let object = root.join("range-wrappers.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &dataset).unwrap();
    let initial = raw.save_state().unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let mut values: Vec<u16> = model
        .records
        .iter()
        .flat_map(|r| [r.lower, r.upper])
        .collect();
    if let Some(v) = (0..=u16::MAX).find(|v| model.select(*v).is_none()) {
        values.push(v);
    }
    let start = SOURCE as usize;
    let end = start + RECORD_COUNT * 8;
    let mut cases = 0;
    for native in [false, true] {
        for (entry_pc, stop_pc, stack_bytes) in [
            (0x0801_abfc, 0x0801_ac1c, 48u32),
            (0x0801_ac54, 0x0801_ac7c, 52u32),
        ] {
            for &value in &values {
                for m in [&mut raw, &mut managed] {
                    m.load_state(&initial).unwrap();
                    m.bus.ffta_sprites.enabled = false;
                    m.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
                    m.cpu.regs[0] = 0x0201_0000;
                    m.cpu.regs[1] = u32::from(value);
                    m.cpu.regs[2] = 0x0300_08b0;
                    m.cpu.regs[3] = 0x0300_0990;
                    m.cpu.regs[13] = 0x0300_7c00;
                    m.cpu.regs[14] = 0x0800_0301;
                    m.cpu.regs[15] = entry_pc;
                    m.cpu.cpsr = 0xbf;
                    m.bus.write8(0x0300_7c00 - stack_bytes + 0x18, 0xa6);
                    m.bus.write16(0x0300_7c00 - stack_bytes + 0x1a, 0xb7c8);
                    m.bus.write32(0x0300_7c00, 2);
                }
                managed.bus.rom[start..end].fill(0xcd);
                let before = owner.range_lookup_stats().unwrap().source_bytes;
                let mut steps = 0;
                while raw.cpu.regs[15] != stop_pc {
                    assert!(steps < 500, "raw pc {:08x}", raw.cpu.regs[15]);
                    raw.step();
                    steps += 1;
                }
                steps = 0;
                while managed.cpu.regs[15] != stop_pc {
                    assert!(steps < 500, "managed pc {:08x}", managed.cpu.regs[15]);
                    if native {
                        table.get(managed.cpu.regs[15] | 1).unwrap()(
                            &RT_API,
                            &mut managed as *mut Machine as *mut std::ffi::c_void,
                        );
                    } else {
                        managed.step();
                    }
                    steps += 1;
                }
                assert!(owner.range_lookup_stats().unwrap().source_bytes > before);
                let selector_addr = 0x0300_7c00 - stack_bytes + 0x18;
                let offset_addr = 0x0300_7c00 - stack_bytes + 0x1a;
                if let Some((selector, offset)) = model.select(value) {
                    assert_eq!(raw.bus.read8(selector_addr), selector);
                    assert_eq!(raw.bus.read16(offset_addr), offset);
                } else {
                    assert_eq!(raw.bus.read8(selector_addr), 0xa6);
                    assert_eq!(raw.bus.read16(offset_addr), 0xb7c8);
                }
                managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
                assert_eq!(
                    managed.save_state().unwrap(),
                    raw.save_state().unwrap(),
                    "wrapper {entry_pc:08x}, value {value}, generated {native}"
                );
                cases += 1;
            }
        }
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"EUR original range-reader wrapper prefixes through helper return",
        "revision":[170,169], "wrapper_entries":["0801abfc","0801ac54"],
        "value_count":values.len(), "cases":cases, "interpreter_and_generated_block_dispatch":true,
        "all_88_source_bytes_poisoned":true, "full_state_clock_equal":true,
        "limits":"Controlled wrapper prefixes stop before later rendering calls; generated blocks can fall back to interpretation. Upstream callers, ordinary scenes and production cache selection remain open.",
        "performance_measurements":false
    })).unwrap()).unwrap();
}
#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "authenticated EUR ROM and E170 dataset; Wine only"]
fn european_range_lookup_e170_adjacent_owner_boundaries() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    let root = PathBuf::from(std::env::var_os("FFTA_RANGE_LOOKUP_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_RANGE_LOOKUP_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let manifest: Manifest =
        toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (170, 169)
    );
    let start = SOURCE as usize;
    let end = start + RECORD_COUNT * 8;
    let lo = start - 4;
    let hi = end + 4;
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let before_sub = owner
        .conditional_substitution_words_stats()
        .unwrap()
        .source_bytes;
    let before_range = owner.range_lookup_stats().unwrap().source_bytes;
    let before_geometry = owner.tilemap_geometry_stats().unwrap().source_bytes;
    for i in lo..hi {
        managed.bus.rom[i] = rom[i] ^ 0xff;
    }
    let trace = TraceCapture::start(
        TraceFilter {
            readers: vec![],
            ranges: vec![(lo as u32, hi as u32)],
        },
        20_000,
    )
    .unwrap();
    trace.frame(Some(managed.bus.frames));
    let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
    let mut cases = [0u32; 3];
    for mirror in [0x0800_0000u32, 0x0a00_0000, 0x0c00_0000] {
        for at in lo..hi {
            assert_eq!(
                managed.bus.read8(mirror + at as u32),
                rom[at],
                "byte {at:x}, mirror {mirror:x}"
            );
            cases[0] += 1;
            if at + 2 <= hi {
                assert_eq!(
                    (RT_API.read16)(ptr, mirror + at as u32),
                    u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())),
                    "halfword {at:x}, mirror {mirror:x}"
                );
                cases[1] += 1;
            }
            if at + 4 <= hi {
                assert_eq!(
                    (RT_API.read32)(ptr, mirror + at as u32),
                    u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()),
                    "word {at:x}, mirror {mirror:x}"
                );
                cases[2] += 1;
            }
        }
    }
    trace.frame(None);
    let trace = trace.finish();
    assert!(!trace.truncated && trace.dropped_requests == 0);
    let requests = |service| {
        trace
            .events
            .iter()
            .filter(|e| e.key.service == service)
            .map(|e| e.requests)
            .sum::<u64>()
    };
    let typed = requests(Service::Typed);
    let raw = requests(Service::Raw);
    let unclassified = requests(Service::Unclassified);
    let sub = owner
        .conditional_substitution_words_stats()
        .unwrap()
        .source_bytes
        - before_sub;
    let range = owner.range_lookup_stats().unwrap().source_bytes - before_range;
    let geometry = owner.tilemap_geometry_stats().unwrap().source_bytes - before_geometry;
    assert!(
        sub > 0 && range > 0 && geometry > 0,
        "owner bytes: {sub}, {range}, {geometry}"
    );
    assert_eq!((raw, unclassified), (0, 0));
    assert_eq!(typed, cases.iter().map(|&x| u64::from(x)).sum::<u64>());
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"E170 range source and immediate substitution/geometry boundaries, 1/2/4-byte reads at all offsets and three ROM mirrors",
        "revision":[170,169], "poisoned_bytes":hi-lo, "cases_byte_halfword_word":cases,
        "typed_requests":typed, "raw_requests":raw, "unclassified_requests":unclassified,
        "trace_dropped":trace.dropped_requests,
        "substitution_source_bytes":sub, "range_source_bytes":range, "geometry_source_bytes":geometry,
        "limits":"Direct controlled bus/API reads do not prove ordinary execution, other source owners or indirect caller coverage.",
        "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "isolated portable Windows executable, authenticated EUR ROM and TinyCC; Wine only"]
fn european_range_lookup_production_cache_selection() {
    let work = fs::canonicalize(std::env::var_os("FFTA_RANGE_LOOKUP_ROOT").unwrap()).unwrap();
    let portable = fs::canonicalize(input_config::storage::portable_root().unwrap()).unwrap();
    assert!(portable.starts_with(&work), "portable root must be inside owned work");
    let rom_path = fs::canonicalize(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert!(rom_path.starts_with(&work), "ROM test copy must be inside owned work");
    let rom = fs::read(&rom_path).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    assert_eq!(crate::TRANSLATION_REV, 8);
    assert!(matches!(recomp_core::Compiler::detect().backend, recomp_core::Backend::MemoryTcc(_)));
    let cache = portable.join("Data").join("cache");
    assert!(!cache.exists(), "test requires a fresh owned cache");
    let previous = cache.join("t7");
    fs::create_dir_all(&previous).unwrap();
    fs::write(previous.join("old-translation-canary.dll"), b"not a translation").unwrap();
    let (cold_library, cold_table) = crate::ensure_native(rom_path.to_str().unwrap(), &rom, false, None, None).unwrap();
    let selected = fs::canonicalize(cold_table.source.as_ref().unwrap()).unwrap();
    assert!(selected.starts_with(fs::canonicalize(cache.join("t8")).unwrap()));
    assert!(!previous.exists(), "old revision was not swept inside owned cache");
    let bytes = fs::read(&selected).unwrap();
    let guard = gba_core::capi::EVENT_GUARD_TAG.to_le_bytes();
    let guard_count = bytes.windows(guard.len()).filter(|window| *window == guard).count();
    assert!(guard_count > 0, "selected translation has no event guard tag");
    let digest = gba_core::ffta_rom::digest(&bytes);
    let modified = fs::metadata(&selected).unwrap().modified().unwrap();
    let count = cold_table.len;
    assert!(count > 0);
    drop(cold_table);
    drop(cold_library);
    let (warm_library, warm_table) = crate::ensure_native(rom_path.to_str().unwrap(), &rom, false, None, None).unwrap();
    assert_eq!(fs::canonicalize(warm_table.source.as_ref().unwrap()).unwrap(), selected);
    assert_eq!(warm_table.len, count);
    assert_eq!(gba_core::ffta_rom::digest(&fs::read(&selected).unwrap()), digest);
    assert_eq!(fs::metadata(&selected).unwrap().modified().unwrap(), modified);
    drop(warm_table);
    drop(warm_library);
    fs::write(work.join("cache-path.txt"), selected.to_str().unwrap()).unwrap();
    fs::write(work.join("cache-result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"isolated portable production cache cold build and warm reuse",
        "revision":crate::TRANSLATION_REV, "blocks":count, "event_guard_tag_occurrences":guard_count,
        "selected_sha256":digest, "old_revision_removed_inside_owned_work":true,
        "warm_path_hash_and_mtime_equal":true,
        "limits":"Explicit portable test calls the same cache selector as play; no actual GUI play or packaged translation is exercised. Tag presence does not establish guard coverage of every block.",
        "performance_measurements":false
    })).unwrap()).unwrap();
}
