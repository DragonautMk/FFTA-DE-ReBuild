//! EUR indexed-graphics contexts: publication, source boundaries and original readers.
use super::*;
use gba_core::{Bus, Machine, capi::RT_API};
use gba_core::rom_data::indexed_graphics::{CONTEXT_RECORDS, CONTEXT_SHA256, CONTEXT_SOURCE};
use recomp_core::analyze::{Analysis, Block, View};

#[test]
#[cfg(feature = "rom-data-read-audit")]
#[ignore = "exact EUR ROM, E158 dataset and TinyCC; Wine only, Linux/USA pending"]
fn european_indexed_graphics_contexts_publication_and_original_readers() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    let root = PathBuf::from(std::env::var_os("FFTA_INDEXED_CONTEXT_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_INDEXED_CONTEXT_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (158, 157));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision), (159, 158));
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!(source.manifest.coverage, old.coverage);
    for old_resource in &old.resources {
        let new = source.manifest.resources.iter().find(|r| r.id == old_resource.id).unwrap();
        let old_bytes = fs::read(previous.join(&old_resource.path)).unwrap();
        let new_bytes = fs::read(prepared.directory.join(&new.path)).unwrap();
        if old_resource.id != INDEXED_GRAPHICS_ID {
            assert_eq!(new, old_resource);
            assert_eq!(new_bytes, old_bytes);
            continue;
        }
        let old_json: serde_json::Value = serde_json::from_slice(&old_bytes).unwrap();
        let mut new_json: serde_json::Value = serde_json::from_slice(&new_bytes).unwrap();
        assert_eq!(new_json.as_object_mut().unwrap().remove("contexts").unwrap().as_array().unwrap().len(), CONTEXT_RECORDS);
        assert_eq!(new_json, old_json);
        let mut metadata = new.clone();
        let added: Vec<_> = metadata.sources.iter().filter(|s| !old_resource.sources.contains(s)).cloned().collect();
        assert_eq!(added.len(), 1);
        assert_eq!((added[0].offset, added[0].length, added[0].sha256.as_str()), (CONTEXT_SOURCE, 132, CONTEXT_SHA256));
        metadata.sources.retain(|s| old_resource.sources.contains(s));
        metadata.sha256 = old_resource.sha256.clone();
        assert_eq!(&metadata, old_resource);
    }
    let catalog = &source.indexed_graphics.as_ref().unwrap().decoded;
    let start = CONTEXT_SOURCE as usize;
    let end = start + CONTEXT_RECORDS * 4;
    for (i, row) in catalog.contexts.iter().enumerate() {
        assert_eq!(usize::from(row.index), i);
        assert_eq!([row.value, row.secondary_program, row.primary_program, row.opaque], rom[start + 4*i..start + 4*i + 4]);
    }
    let mut invalid = catalog.clone();
    invalid.contexts.pop();
    assert!(invalid.validate().is_err());
    let mut invalid = catalog.clone();
    invalid.contexts[1].index = 0;
    assert!(invalid.validate().is_err());
    let mut edited = catalog.clone();
    edited.contexts[0].opaque ^= 1;
    edited.validate().unwrap();
    assert!(edited.verify_source(&rom).is_err());

    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let before = owner.indexed_graphics_stats().unwrap().context_bytes;
    managed.bus.rom[start..end].fill(0xcd);
    for at in start..end { assert_eq!(managed.bus.read8(0x0c00_0000 + at as u32), rom[at]); }
    assert_eq!(owner.indexed_graphics_stats().unwrap().context_bytes - before, 132);
    let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for edge in [start, end] { for delta in [-3isize, -1, 0, 1] {
        let at = edge.wrapping_add_signed(delta);
        assert_eq!((RT_API.read16)(ptr, 0x0a00_0000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at+2].try_into().unwrap())));
        assert_eq!((RT_API.read32)(ptr, 0x0800_0000 + at as u32), u32::from_le_bytes(rom[at..at+4].try_into().unwrap()));
    }}
    managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);

    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let addresses = (0x0801_9670..0x0801_9678).step_by(2)
        .chain((0x0801_972c..0x0801_973c).step_by(2))
        .chain([0x0801_96ac, 0x0801_96c8, 0x0801_9704]);
    let blocks = addresses.map(|addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("indexed-context.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let initial = raw.save_state().unwrap();
    let mut cases = 0;
    for native in [false, true] { for index in [0u32, 1, 30, 31, 32, 33, 127, 255, 0x120] {
        for m in [&mut raw, &mut managed] {
            m.load_state(&initial).unwrap();
            m.bus.ffta_sprites.enabled = false;
            m.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
            m.cpu.regs[0] = index;
            m.cpu.regs[13] = 0x0300_7c00;
            m.cpu.regs[14] = 0x0800_0301;
            m.cpu.regs[15] = 0x0801_972c;
            m.cpu.cpsr = 0xbf;
        }
        managed.bus.rom[start..end].fill(0xcd);
        let before = owner.indexed_graphics_stats().unwrap().context_bytes;
        let mut steps = 0;
        while raw.cpu.regs[15] != 0x0800_0300 { assert!(steps < 32); raw.step(); steps += 1; }
        steps = 0;
        while managed.cpu.regs[15] != 0x0800_0300 {
            assert!(steps < 32, "getter pc {:08x}", managed.cpu.regs[15]);
            if native { table.get(managed.cpu.regs[15] | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void); }
            else { managed.step(); }
            steps += 1;
        }
        let used = owner.indexed_graphics_stats().unwrap().context_bytes - before;
        assert_eq!(used, u64::from((index & 255) < CONTEXT_RECORDS as u32), "index {index}, native {native}");
        managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
        assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap(), "getter index {index}, native {native}");
        cases += 1;
    }}
    for native in [false, true] { for (pc, field) in [(0x0801_96ac, 2usize), (0x0801_96c8, 1), (0x0801_9704, 1)] {
        for m in [&mut raw, &mut managed] {
            m.load_state(&initial).unwrap();
            m.bus.ffta_sprites.enabled = false;
            m.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
            m.cpu.regs[7] = 0x0800_0000 + CONTEXT_SOURCE + 32 * 4;
            m.cpu.regs[15] = pc;
            m.cpu.cpsr = 0xbf;
        }
        managed.bus.rom[start..end].fill(0xcd);
        let before = owner.indexed_graphics_stats().unwrap().context_bytes;
        raw.step();
        if native { table.get(pc | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void); }
        else { managed.step(); }
        assert_eq!(owner.indexed_graphics_stats().unwrap().context_bytes - before, 1);
        assert_eq!(raw.cpu.regs[if field == 2 { 2 } else if pc == 0x0801_96c8 { 2 } else { 0 }], u32::from(rom[start + 32*4 + field]));
        managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
        assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap(), "field {field}, pc {pc:08x}, native {native}");
        cases += 1;
    }}
    // This retained ordinary world-map input already reaches context 3. The new
    // check observes its newly owned bytes, rather than replaying the old proof.
    let ordinary_state = fs::read(std::env::var_os("FFTA_INDEXED_CONTEXT_STATE").unwrap()).unwrap();
    let ordinary_input = PathBuf::from(std::env::var_os("FFTA_INDEXED_CONTEXT_INPUT").unwrap());
    assert_eq!(gba_core::ffta_rom::digest(&ordinary_state), "b379038c2a3b6e1740fd70fe78f9f5eaf764d4d2f5eac7fa316d4640e0efb808");
    assert_eq!(gba_core::ffta_rom::digest(&fs::read(&ordinary_input).unwrap()), "f8cbb37acf523ca0d1b2317bc1017b219ee0be0811d2ccfbb09ee32a8ec749ab");
    let input = crate::InputScript::load(&ordinary_input.to_string_lossy()).unwrap();
    let mut ordinary_raw = crate::make_machine(rom.clone(), None);
    let mut ordinary_managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut ordinary_managed, &prepared.directory).unwrap();
    ordinary_raw.load_state(&ordinary_state).unwrap();
    ordinary_managed.load_state(&ordinary_state).unwrap();
    let mut typed_record3 = 0;
    let mut raw_context = 0;
    for frame in 0..11u64 {
        for m in [&mut ordinary_raw, &mut ordinary_managed] {
            m.bus.keys = input.keys_at(frame);
            input.apply_mouse(&mut m.bus, frame).unwrap();
            m.bus.audio_buf.clear();
            m.bus.frame_ready = false;
        }
        for _ in 0..5_000_000 { if ordinary_raw.bus.frame_ready { break; } ordinary_raw.step(); }
        assert!(ordinary_raw.bus.frame_ready);
        let trace = TraceCapture::start(TraceFilter { readers: vec![], ranges: vec![(CONTEXT_SOURCE, CONTEXT_SOURCE + 132)] }, 10_000).unwrap();
        trace.frame(Some(ordinary_managed.bus.frames));
        for _ in 0..5_000_000 { if ordinary_managed.bus.frame_ready { break; } ordinary_managed.step(); }
        assert!(ordinary_managed.bus.frame_ready);
        let report = trace.finish();
        assert!(!report.truncated);
        typed_record3 += report.events.iter().filter(|e| e.key.service == Service::Typed
            && (CONTEXT_SOURCE + 12..CONTEXT_SOURCE + 16).contains(&e.key.read.physical_offset)).map(|e| e.requests).sum::<u64>();
        raw_context += report.events.iter().filter(|e| e.key.service == Service::Raw).map(|e| e.requests).sum::<u64>();
        assert_eq!(ordinary_managed.save_state().unwrap(), ordinary_raw.save_state().unwrap(), "ordinary frame {frame}");
        assert!(ordinary_managed.bus.framebuffer == ordinary_raw.bus.framebuffer
            && ordinary_managed.bus.wide_framebuffer == ordinary_raw.bus.wide_framebuffer
            && ordinary_managed.bus.oam == ordinary_raw.bus.oam
            && ordinary_managed.bus.audio_buf == ordinary_raw.bus.audio_buf, "ordinary output frame {frame}");
    }
    assert!(typed_record3 > 0, "retained ordinary menu must read context 3");
    assert_eq!(raw_context, 0);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[159,158], "previous_resources_preserved":true,
        "context_bytes":132, "context_records":33, "source_sha256":CONTEXT_SHA256,
        "poisoned_raw_rom":true, "original_reader_cases":cases,
        "compiled_dispatch_cases":"included; may use interpreter fallback", "full_state_clock_equal":true,
        "ordinary_interpreter_frames":11, "ordinary_context3_typed_requests":typed_record3,
        "ordinary_context_raw_requests":raw_context,
        "Linux":"pending", "USA":"pending", "native_windows":"pending",
        "performance_measurements":false
    })).unwrap()).unwrap();
}
#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "authenticated EUR battle state/input and current dataset; Wine only"]
fn european_encounter_special_descriptor_ordinary_observation() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};

    const ROW1: usize = 0x006e_6f5e;
    const READER: u32 = 0x080c_d43c;
    let root = PathBuf::from(std::env::var_os("FFTA_INDEXED_CONTEXT_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_INDEXED_CONTEXT_DATASET").unwrap());
    let state = fs::read(std::env::var_os("FFTA_STATE_TEST_STATE").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&state),
        "b379038c2a3b6e1740fd70fe78f9f5eaf764d4d2f5eac7fa316d4640e0efb808"
    );
    let input_path = std::env::var("FFTA_STATE_TEST_INPUT").unwrap();
    let input_bytes = fs::read(&input_path).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&input_bytes),
        "f8cbb37acf523ca0d1b2317bc1017b219ee0be0811d2ccfbb09ee32a8ec749ab"
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
        "packed_source_bytes_total":owner.indexed_graphics_stats().unwrap().context_bytes,
        "source_rom_poisoned":true, "negative_is_not_global_absence":true,
        "limitations":"One retained route; no all-consumer, all-scene or translated-dispatch claim.",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "authenticated EUR world-map state/input, current dataset and native library; Wine only"]
fn european_indexed_context_world_map_translated_pair() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    crate::configure_headless_widescreen();
    const ROW: usize = CONTEXT_SOURCE as usize;
    const LEN: usize = CONTEXT_RECORDS * 4;
    let root = PathBuf::from(std::env::var_os("FFTA_INDEXED_CONTEXT_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_INDEXED_CONTEXT_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let state = fs::read(std::env::var_os("FFTA_STATE_TEST_STATE").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&state),
        "b379038c2a3b6e1740fd70fe78f9f5eaf764d4d2f5eac7fa316d4640e0efb808"
    );
    let input_path = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_INPUT").unwrap());
    assert_eq!(
        gba_core::ffta_rom::digest(&fs::read(&input_path).unwrap()),
        "f8cbb37acf523ca0d1b2317bc1017b219ee0be0811d2ccfbb09ee32a8ec749ab"
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
    let before = owner.indexed_graphics_stats().unwrap().context_bytes;
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
    for frame in 0..41u64 {
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
    let bytes = owner.indexed_graphics_stats().unwrap().context_bytes - before;
    let records: std::collections::BTreeSet<_> = trace
        .events
        .iter()
        .filter(|e| e.key.service == Service::Typed)
        .flat_map(|e| {
            let start = e.key.read.physical_offset.max(CONTEXT_SOURCE);
            let end = (e.key.read.physical_offset + e.key.read.length as u32)
                .min(CONTEXT_SOURCE + LEN as u32);
            (start..end).map(|offset| (offset - CONTEXT_SOURCE) / 4)
        })
        .collect();
    assert!(
        records.contains(&3) && typed > 0 && bytes > 0,
        "ordinary context 3 must be consumed"
    );
    assert_eq!((raw_reads, unclassified), (0, 0));
    assert!(audio.iter().all(|&v| v > 0), "audio counts: {audio:?}");
    assert!(dispatch.0 > 0);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"41-frame recorded EUR world-map raw/managed translated pair",
        "revision":[170,169], "frames":41, "context_typed_requests":typed,
        "context_raw_requests":raw_reads, "context_unclassified_requests":unclassified,
        "context_records_touched":records, "context_bytes":bytes, "trace_dropped":trace.dropped_requests,
        "all_context_sources_poisoned":true, "full_state_clock_video_oam_audio_equal_each_frame":true,
        "audio_sample_counts":audio, "generated_dispatches":dispatch.0, "fallback_dispatches":dispatch.1,
        "limits":"Generated blocks may fall back to interpretation; read attribution is aggregate and other contexts and the full reader ABI remain open.",
        "performance_measurements":false
    })).unwrap()).unwrap();
}
#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "authenticated EUR ROM, E170 dataset and TinyCC; Wine only"]
fn european_indexed_context_full_byte_domain_and_fields() {
    let root = PathBuf::from(std::env::var_os("FFTA_INDEXED_CONTEXT_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_INDEXED_CONTEXT_DATASET").unwrap());
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
    let start = CONTEXT_SOURCE as usize;
    let end = start + CONTEXT_RECORDS * 4;
    assert_eq!(gba_core::ffta_rom::digest(&rom[start..end]), CONTEXT_SHA256);

    let view = View {
        rom: &rom,
        ewram: None,
        iwram: None,
        bios: None,
    };
    let addresses = (0x0801_9670..0x0801_9678)
        .step_by(2)
        .chain((0x0801_972c..0x0801_973c).step_by(2))
        .chain([0x0801_96ac, 0x0801_96c8, 0x0801_9704]);
    let blocks = addresses
        .map(|addr| Block {
            addr,
            thumb: true,
            instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
        })
        .collect();
    let object = root.join("indexed-context-domain.o");
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
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    let mut getter_cases = 0u32;
    let mut field_cases = 0u32;
    for generated in [false, true] {
        for high in [0u32, 0x100] {
            for low in 0..=255u32 {
                let index = high | low;
                for m in [&mut raw, &mut managed] {
                    m.load_state(&initial).unwrap();
                    m.bus.ffta_sprites.enabled = false;
                    m.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
                    m.cpu.regs[0] = index;
                    m.cpu.regs[13] = 0x0300_7c00;
                    m.cpu.regs[14] = 0x0800_0301;
                    m.cpu.regs[15] = 0x0801_972c;
                    m.cpu.cpsr = 0xbf;
                }
                managed.bus.rom[start..end].fill(0xcd);
                let before = owner.indexed_graphics_stats().unwrap().context_bytes;
                let mut steps = 0;
                while raw.cpu.regs[15] != 0x0800_0300 {
                    assert!(steps < 32);
                    raw.step();
                    steps += 1;
                }
                steps = 0;
                while managed.cpu.regs[15] != 0x0800_0300 {
                    assert!(steps < 32, "getter pc {:08x}", managed.cpu.regs[15]);
                    if generated {
                        table.get(managed.cpu.regs[15] | 1).unwrap()(
                            &RT_API,
                            &mut managed as *mut Machine as *mut std::ffi::c_void,
                        );
                    } else {
                        managed.step();
                    }
                    steps += 1;
                }
                assert_eq!(
                    owner.indexed_graphics_stats().unwrap().context_bytes - before,
                    u64::from(low < CONTEXT_RECORDS as u32),
                    "index {index}, generated {generated}"
                );
                assert_eq!(raw.cpu.regs[0], u32::from(rom[start + low as usize * 4]));
                managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
                assert_eq!(
                    managed.save_state().unwrap(),
                    raw.save_state().unwrap(),
                    "getter index {index}, generated {generated}"
                );
                getter_cases += 1;
            }
        }
        for record in 0..CONTEXT_RECORDS {
            for (pc, field, reg) in [
                (0x0801_96ac, 2usize, 2usize),
                (0x0801_96c8, 1, 2),
                (0x0801_9704, 1, 0),
            ] {
                for m in [&mut raw, &mut managed] {
                    m.load_state(&initial).unwrap();
                    m.bus.ffta_sprites.enabled = false;
                    m.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
                    m.cpu.regs[7] = 0x0800_0000 + CONTEXT_SOURCE + record as u32 * 4;
                    m.cpu.regs[15] = pc;
                    m.cpu.cpsr = 0xbf;
                }
                managed.bus.rom[start..end].fill(0xcd);
                let before = owner.indexed_graphics_stats().unwrap().context_bytes;
                raw.step();
                if generated {
                    table.get(pc | 1).unwrap()(
                        &RT_API,
                        &mut managed as *mut Machine as *mut std::ffi::c_void,
                    );
                } else {
                    managed.step();
                }
                assert_eq!(
                    owner.indexed_graphics_stats().unwrap().context_bytes - before,
                    1
                );
                assert_eq!(
                    raw.cpu.regs[reg],
                    u32::from(rom[start + record * 4 + field])
                );
                managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
                assert_eq!(
                    managed.save_state().unwrap(),
                    raw.save_state().unwrap(),
                    "record {record}, pc {pc:08x}, generated {generated}"
                );
                field_cases += 1;
            }
        }
    }
    assert_eq!((getter_cases, field_cases), (1024, 198));
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[170,169],
        "scope":"controlled original context getter full byte domain, one high-byte alias and three field instructions for all records",
        "getter_cases":getter_cases, "field_instruction_cases":field_cases,
        "getter_inputs":"0..255 and 0x100..0x1ff", "records":CONTEXT_RECORDS,
        "interpreter_and_generated_block_dispatch":true, "full_state_clock_equal":true,
        "source_context_bytes_poisoned":true,
        "limits":"Controlled registers do not prove ordinary reachability, other high-byte aliases, callers or complete reader ABI; generated blocks can use interpreter fallback.",
        "performance_measurements":false
    })).unwrap()).unwrap();
}
