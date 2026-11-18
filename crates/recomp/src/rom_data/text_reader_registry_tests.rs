//! EUR publication and original prefixes for the conditional text-reader registry.
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::text_reader_registry::{
        Catalog, TextReaderRegistry, LOOKUP_LITERAL, PRODUCER_LITERAL, TABLE_OFFSET,
    },
    Bus, Machine,
};

fn original_prefix(machine: &mut Machine, producer: bool, index: u8) -> u32 {
    machine.cpu.regs[1] = u32::from(index);
    machine.cpu.regs[4] = u32::from(index);
    machine.cpu.regs[3] = 0x0200_1000;
    machine.cpu.regs[15] = if producer { 0x0801_9172 } else { 0x0801_9424 };
    machine.cpu.cpsr = 0xbf;
    let stop = if producer { 0x0801_9182 } else { 0x0801_9430 };
    for _ in 0..32 {
        machine.bus.frame_ready = false;
        machine.step();
        if machine.cpu.regs[15] == stop {
            return machine.cpu.regs[0];
        }
    }
    panic!("original text-reader registry prefix did not finish");
}

#[test]
#[ignore = "exact EUR ROM and E168 dataset; Wine only, Linux/USA pending"]
fn european_text_reader_registry_publication_and_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_TEXT_READER_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_TEXT_READER_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest =
        toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (168, 167));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (169, 168)
    );
    let prepared =
        preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(
        preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    assert_eq!(source.manifest.resources.len(), old.resources.len() + 1);
    for prior in &old.resources {
        let current = source
            .manifest
            .resources
            .iter()
            .find(|r| r.id == prior.id)
            .unwrap();
        assert_eq!(current, prior);
        assert_eq!(
            fs::read(prepared.directory.join(&current.path)).unwrap(),
            fs::read(previous.join(&prior.path)).unwrap()
        );
    }
    let catalog = source.text_reader_registry.as_ref().unwrap();
    let model: TextReaderRegistry =
        serde_json::from_slice(&serde_json::to_vec(&catalog.decoded).unwrap()).unwrap();
    assert_eq!(model.table_offset, TABLE_OFFSET);
    assert_eq!(model.callbacks.len(), 27);
    assert_eq!(
        [model.callbacks[0], model.callbacks[3], model.callbacks[5]],
        [0; 3]
    );
    assert_eq!(model.callbacks[11], 0x0801_6c01);
    assert_eq!(
        catalog.sources.iter().map(|s| s.length).collect::<Vec<_>>(),
        [4, 4, 108]
    );
    let mut invalid = model.clone();
    invalid.table_offset += 1;
    assert!(invalid.validate().is_err());
    let mut overlap = old.resources.clone();
    let mut partial = catalog.sources[2].clone();
    partial.length = 1;
    overlap[0].sources.push(partial);
    assert!(Catalog::from_rom(&rom, &overlap).is_err());
    let mut decoded = read_dataset(&prepared.directory).unwrap();
    source.validate_files(decoded.files()).unwrap();
    decoded.text_reader_registry.as_mut().unwrap().callbacks[11] = 0;
    let changed =
        encode_text_reader_registry(decoded.text_reader_registry.as_ref().unwrap()).unwrap();
    decoded
        .manifest
        .resources
        .iter_mut()
        .find(|r| r.id == TEXT_READER_REGISTRY_ID)
        .unwrap()
        .sha256 = gba_core::ffta_rom::digest(&changed);
    decoded.text_reader_registry_bytes = Some(changed);
    assert!(
        source.validate_files(decoded.files()).is_err(),
        "edited callback must be rejected even with a matching file hash"
    );

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let spans = [
        (PRODUCER_LITERAL as usize, 4usize),
        (LOOKUP_LITERAL as usize, 4),
        (TABLE_OFFSET as usize, 108),
    ];
    for (at, len) in spans {
        managed.bus.rom[at..at + len].fill(0xcd);
    }
    let before = owner.text_reader_registry_stats().unwrap();
    for (at, len) in spans {
        for address in at..at + len {
            assert_eq!(
                managed.bus.read8(0x0800_0000 + address as u32),
                rom[address]
            );
        }
    }
    let after = owner.text_reader_registry_stats().unwrap();
    assert_eq!(after.source_bytes - before.source_bytes, 116);
    assert_eq!(after.literal_bytes - before.literal_bytes, 8);
    assert_eq!(after.callback_bytes - before.callback_bytes, 108);
    let boundary = TABLE_OFFSET as usize;
    // The byte before this projection is raw; the following context bytes have an earlier owner.
    managed.bus.rom[boundary + 108..boundary + 112].fill(0xcd);
    let pointer = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for at in [boundary - 1, boundary, boundary + 105, boundary + 107] {
        assert_eq!(
            (RT_API.read16)(pointer, 0x0a00_0000 + at as u32),
            u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap()))
        );
        assert_eq!(
            (RT_API.read32)(pointer, 0x0800_0000 + at as u32),
            u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
        );
    }
    managed.bus.rom[boundary + 108..boundary + 112]
        .copy_from_slice(&rom[boundary + 108..boundary + 112]);
    for (at, len) in spans {
        managed.bus.rom[at..at + len].copy_from_slice(&rom[at..at + len]);
    }
    let initial = raw.save_state().unwrap();
    let before = owner.text_reader_registry_stats().unwrap();
    let mut comparisons = 0;
    for producer in [true, false] {
        for index in [0u8, 3, 5, 11, 26, 27] {
            raw.load_state(&initial).unwrap();
            managed.load_state(&initial).unwrap();
            for (at, len) in spans {
                managed.bus.rom[at..at + len].fill(0xcd);
            }
            assert_eq!(
                original_prefix(&mut raw, producer, index),
                original_prefix(&mut managed, producer, index)
            );
            for (at, len) in spans {
                managed.bus.rom[at..at + len].copy_from_slice(&rom[at..at + len]);
            }
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
            comparisons += 1;
        }
    }
    let after = owner.text_reader_registry_stats().unwrap();
    assert_eq!(after.literal_bytes - before.literal_bytes, 12 * 4);
    assert_eq!(after.callback_bytes - before.callback_bytes, 10 * 4);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[169,168], "source_bytes":116,
        "previous_resources_preserved":old.resources.len(), "created_reused_verified":true,
        "modified_callback_rejected":true, "partial_overlap_rejected":true,
        "original_prefix_comparisons":comparisons, "indices":[0,3,5,11,26,27],
        "full_state_clock_equal":true, "mixed_width_adjacent_owner_checked":true,
        "managed_sources_poisoned":true, "guest_index_bound":"unchanged",
        "ordinary_use":"unproved", "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "EUR home checkpoint, recorded input, E170 dataset and native library; Wine only"]
fn european_text_reader_registry_ordinary_audio_poisoned_pair() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    use gba_core::rom_data::text_reader_registry::ENTRY_COUNT;
    use std::collections::BTreeSet;
    crate::configure_headless_widescreen();
    let root = PathBuf::from(std::env::var_os("FFTA_TEXT_READER_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let state_path = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_STATE").unwrap());
    let state = fs::read(&state_path).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&state),
        "cad8547dcbbdc28b284229c2328bf1fcf414e1d61ee86665206eb99ba9cca9da"
    );
    let input_path = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_INPUT").unwrap());
    assert_eq!(
        gba_core::ffta_rom::digest(&fs::read(&input_path).unwrap()),
        "3b91750c1465e04e8ed8c68b73ff81c9731c7d1c545245f06749c205fbc89c45"
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
        crate::save_state::load(m, &state_path).unwrap();
        crate::arm_audio_hle(m, None);
        m.bus.tap_channels = true;
        m.bus.audio_buf.clear();
        m.bus.psg_tap.clear();
        m.bus.fifo_tap[0].clear();
        m.bus.fifo_tap[1].clear();
        m.bus.hle_tap.clear();
    }
    assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
    let spans = [
        (PRODUCER_LITERAL as usize, 4usize),
        (LOOKUP_LITERAL as usize, 4usize),
        (TABLE_OFFSET as usize, ENTRY_COUNT * 4),
    ];
    let owner = managed.bus.rom_resources().unwrap().clone();
    let before = owner.text_reader_registry_stats().unwrap();
    let trace = TraceCapture::start(
        TraceFilter {
            readers: vec![],
            ranges: vec![(TABLE_OFFSET, TABLE_OFFSET + (ENTRY_COUNT * 4) as u32)],
        },
        10000,
    )
    .unwrap();
    let start = raw.bus.frames;
    let mut audio = [0u64; 5];
    let mut dispatch_totals = (0u64, 0u64);
    for frame in 0..1800u64 {
        for (at, len) in spans {
            managed.bus.rom[at..at + len].fill(0xcd);
        }
        let mut counts = [(0, 0); 2];
        for (arm, m) in [&mut raw, &mut managed].into_iter().enumerate() {
            m.bus.keys = input.keys_at(frame);
            input.apply_mouse(&mut m.bus, frame).unwrap();
            trace.frame(if arm == 1 { Some(m.bus.frames) } else { None });
            let pointer = m as *mut Machine as *mut std::ffi::c_void;
            counts[arm] = crate::run_frame_native(m, &table, pointer, crate::FRAME_STEP_GUARD);
            assert_eq!(m.bus.frames, start + frame + 1);
        }
        trace.frame(None);
        assert_eq!(counts[0], counts[1], "dispatch at {frame}");
        dispatch_totals.0 += counts[0].0;
        dispatch_totals.1 += counts[0].1;
        assert_eq!(raw.bus.clock, managed.bus.clock, "clock at {frame}");
        assert_eq!(
            raw.bus.framebuffer, managed.bus.framebuffer,
            "framebuffer at {frame}"
        );
        assert_eq!(
            raw.bus.wide_framebuffer, managed.bus.wide_framebuffer,
            "wide framebuffer at {frame}"
        );
        assert_eq!(raw.bus.oam, managed.bus.oam, "OAM at {frame}");
        assert_eq!(raw.bus.audio_buf, managed.bus.audio_buf, "audio at {frame}");
        assert_eq!(raw.bus.psg_tap, managed.bus.psg_tap, "PSG at {frame}");
        assert_eq!(raw.bus.fifo_tap, managed.bus.fifo_tap, "FIFO at {frame}");
        assert_eq!(raw.bus.hle_tap.len(), managed.bus.hle_tap.len());
        assert!(
            raw.bus
                .hle_tap
                .iter()
                .zip(&managed.bus.hle_tap)
                .all(|(a, b)| a.to_bits() == b.to_bits()),
            "HLE at {frame}"
        );
        for (count, len) in audio.iter_mut().zip([
            managed.bus.audio_buf.len(),
            managed.bus.psg_tap.len(),
            managed.bus.fifo_tap[0].len(),
            managed.bus.fifo_tap[1].len(),
            managed.bus.hle_tap.len(),
        ]) {
            *count += len as u64;
        }
        for (at, len) in spans {
            managed.bus.rom[at..at + len].copy_from_slice(&rom[at..at + len]);
        }
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
        if frame % 300 == 299 {
            eprintln!("REGISTRY_ORDINARY frames={}", frame + 1);
        }
    }
    let trace = trace.finish();
    assert!(!trace.truncated && trace.dropped_requests == 0);
    let mut slots = BTreeSet::new();
    let mut typed = 0u64;
    for event in &trace.events {
        assert_eq!(
            event.key.service,
            Service::Typed,
            "registry table read must use the typed source"
        );
        typed += event.requests;
        let read = event.key.read;
        let start = read.physical_offset.max(TABLE_OFFSET);
        let end = (read.physical_offset + read.length as u32)
            .min(TABLE_OFFSET + (ENTRY_COUNT * 4) as u32);
        for offset in start..end {
            slots.insert((offset - TABLE_OFFSET) / 4);
        }
    }
    let after = owner.text_reader_registry_stats().unwrap();
    let callback_bytes = after.callback_bytes - before.callback_bytes;
    assert!(
        callback_bytes > 0 && typed > 0 && !slots.is_empty(),
        "ordinary registry reads required"
    );
    assert!(
        audio.iter().all(|&n| n > 0),
        "audio streams must have samples: {audio:?}"
    );
    assert!(dispatch_totals.0 > 0, "generated dispatch required");
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"1800 recorded home frames from authenticated EUR checkpoint; Wine",
        "revision":[170,169], "frames":1800, "callback_bytes":callback_bytes, "registry_table_typed_requests":typed,
        "registry_slots_addressed":slots, "trace_dropped":trace.dropped_requests, "poisoned_bytes":116,
        "full_state_clock_video_oam_audio_equal_each_frame":true, "audio_sample_counts":audio,
        "generated_dispatches":dispatch_totals.0, "fallback_dispatches":dispatch_totals.1,
        "limits":"Source addresses, not proof of callback execution; generated dispatch may fall back to interpreter. Other scenes/consumers and Linux/USA/native Windows remain open.",
        "performance_measurements":false
    })).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "EUR home checkpoint, recorded input, E170 dataset and native library; Wine only"]
fn european_geometry_special_ordinary_audio_poisoned_pair() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    use gba_core::rom_data::{script39_special_poses::Script39SpecialPoses, tilemap_geometry};
    use std::collections::BTreeSet;
    crate::configure_headless_widescreen();
    let root = PathBuf::from(std::env::var_os("FFTA_TEXT_READER_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let state_path = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_STATE").unwrap());
    let state = fs::read(&state_path).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&state),
        "cad8547dcbbdc28b284229c2328bf1fcf414e1d61ee86665206eb99ba9cca9da"
    );
    let input_path = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_INPUT").unwrap());
    assert_eq!(
        gba_core::ffta_rom::digest(&fs::read(&input_path).unwrap()),
        "3b91750c1465e04e8ed8c68b73ff81c9731c7d1c545245f06749c205fbc89c45"
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
        crate::save_state::load(m, &state_path).unwrap();
        crate::arm_audio_hle(m, None);
        m.bus.tap_channels = true;
        m.bus.audio_buf.clear();
        m.bus.psg_tap.clear();
        m.bus.fifo_tap[0].clear();
        m.bus.fifo_tap[1].clear();
        m.bus.hle_tap.clear();
    }
    assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
    let special_resource = manifest
        .resources
        .iter()
        .find(|r| r.id == "graphics.script39-special-poses")
        .unwrap();
    let special: Script39SpecialPoses =
        serde_json::from_slice(&fs::read(dataset.join(&special_resource.path)).unwrap()).unwrap();
    special.validate().unwrap();
    let mut spans: Vec<(usize, usize)> = special_resource
        .sources
        .iter()
        .filter(|s| s.role == "index")
        .map(|s| (s.offset as usize, s.length as usize))
        .collect();
    assert_eq!(spans.iter().map(|s| s.1).sum::<usize>(), 964 * 4);
    let index_min = spans.iter().map(|s| s.0).min().unwrap();
    let index_end = spans.iter().map(|s| s.0 + s.1).max().unwrap();
    spans.push((
        tilemap_geometry::SOURCE as usize,
        tilemap_geometry::SOURCE_BYTES,
    ));
    let owner = managed.bus.rom_resources().unwrap().clone();
    let before_geometry = owner.tilemap_geometry_stats().unwrap().source_bytes;
    let before_special = owner.script39_special_pose_stats().unwrap().index_bytes;
    let trace = TraceCapture::start(
        TraceFilter {
            readers: vec![],
            ranges: vec![
                (
                    tilemap_geometry::SOURCE,
                    tilemap_geometry::SOURCE + tilemap_geometry::SOURCE_BYTES as u32,
                ),
                (index_min as u32, index_end as u32),
            ],
        },
        50000,
    )
    .unwrap();
    let start = raw.bus.frames;
    let mut audio = [0u64; 5];
    let mut dispatch_totals = (0u64, 0u64);
    for frame in 0..1800u64 {
        for &(at, len) in &spans {
            managed.bus.rom[at..at + len].fill(0xcd);
        }
        let mut counts = [(0, 0); 2];
        for (arm, m) in [&mut raw, &mut managed].into_iter().enumerate() {
            m.bus.keys = input.keys_at(frame);
            input.apply_mouse(&mut m.bus, frame).unwrap();
            trace.frame(if arm == 1 { Some(m.bus.frames) } else { None });
            let pointer = m as *mut Machine as *mut std::ffi::c_void;
            counts[arm] = crate::run_frame_native(m, &table, pointer, crate::FRAME_STEP_GUARD);
            assert_eq!(m.bus.frames, start + frame + 1);
        }
        trace.frame(None);
        assert_eq!(counts[0], counts[1], "dispatch at {frame}");
        dispatch_totals.0 += counts[0].0;
        dispatch_totals.1 += counts[0].1;
        assert_eq!(raw.bus.clock, managed.bus.clock, "clock at {frame}");
        assert_eq!(
            raw.bus.framebuffer, managed.bus.framebuffer,
            "framebuffer at {frame}"
        );
        assert_eq!(
            raw.bus.wide_framebuffer, managed.bus.wide_framebuffer,
            "wide framebuffer at {frame}"
        );
        assert_eq!(raw.bus.oam, managed.bus.oam, "OAM at {frame}");
        assert_eq!(raw.bus.audio_buf, managed.bus.audio_buf, "audio at {frame}");
        assert_eq!(raw.bus.psg_tap, managed.bus.psg_tap, "PSG at {frame}");
        assert_eq!(raw.bus.fifo_tap, managed.bus.fifo_tap, "FIFO at {frame}");
        assert_eq!(raw.bus.hle_tap.len(), managed.bus.hle_tap.len());
        assert!(
            raw.bus
                .hle_tap
                .iter()
                .zip(&managed.bus.hle_tap)
                .all(|(a, b)| a.to_bits() == b.to_bits()),
            "HLE at {frame}"
        );
        for (count, len) in audio.iter_mut().zip([
            managed.bus.audio_buf.len(),
            managed.bus.psg_tap.len(),
            managed.bus.fifo_tap[0].len(),
            managed.bus.fifo_tap[1].len(),
            managed.bus.hle_tap.len(),
        ]) {
            *count += len as u64;
        }
        for &(at, len) in &spans {
            managed.bus.rom[at..at + len].copy_from_slice(&rom[at..at + len]);
        }
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
        if frame % 300 == 299 {
            eprintln!("REGISTRY_ORDINARY frames={}", frame + 1);
        }
    }
    let trace = trace.finish();
    assert!(!trace.truncated && trace.dropped_requests == 0);
    let mut groups = BTreeSet::new();
    let mut cells = BTreeSet::new();
    let mut geometry_requests = 0u64;
    let mut special_requests = 0u64;
    for event in &trace.events {
        let read = event.key.read;
        let start = read.physical_offset;
        let end = start + read.length as u32;
        let geom_start = tilemap_geometry::SOURCE;
        let geom_end = geom_start + tilemap_geometry::SOURCE_BYTES as u32;
        if start < geom_end && end > geom_start {
            assert_eq!(
                event.key.service,
                Service::Typed,
                "geometry source must be typed"
            );
            geometry_requests += event.requests;
            for offset in start.max(geom_start)..end.min(geom_end) {
                groups.insert((offset - geom_start) / 8);
            }
        }
        for cell in &special.cells {
            let at = cell.source_offset;
            if start < at + 4 && end > at {
                assert_eq!(
                    event.key.service,
                    Service::Typed,
                    "special pose index must be typed"
                );
                special_requests += event.requests;
                cells.insert(at);
            }
        }
    }
    let geometry_bytes = owner.tilemap_geometry_stats().unwrap().source_bytes - before_geometry;
    let special_index_bytes =
        owner.script39_special_pose_stats().unwrap().index_bytes - before_special;
    assert!(geometry_bytes > 0 && geometry_requests > 0 && !groups.is_empty());
    assert!(special_index_bytes > 0 && special_requests > 0 && !cells.is_empty());
    let cell_details: Vec<_> = special
        .cells
        .iter()
        .filter(|c| cells.contains(&c.source_offset))
        .collect();
    assert!(
        audio.iter().all(|&n| n > 0),
        "audio streams must have samples: {audio:?}"
    );
    assert!(dispatch_totals.0 > 0, "generated dispatch required");
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"1800 recorded home frames from authenticated EUR checkpoint; Wine",
        "revision":[170,169], "frames":1800,
        "geometry_source_bytes":geometry_bytes, "geometry_typed_requests":geometry_requests, "geometry_groups":groups,
        "special_index_bytes":special_index_bytes, "special_typed_requests":special_requests, "special_cells":cell_details,
        "trace_dropped":trace.dropped_requests, "poisoned_geometry_bytes":tilemap_geometry::SOURCE_BYTES, "poisoned_special_index_bytes":964*4,
        "full_state_clock_video_oam_audio_equal_each_frame":true, "audio_sample_counts":audio,
        "generated_dispatches":dispatch_totals.0, "fallback_dispatches":dispatch_totals.1,
        "limits":"Address reads do not prove index producer identities or pose animation; generated dispatch may fall back to interpretation. Other scenes/consumers and Linux/USA/native Windows remain open.",
        "performance_measurements":false
    })).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "EUR home checkpoint, recorded input, E170 dataset and native library; Wine only"]
fn european_special_pose_ordinary_reader_attribution() {
    use gba_core::rom_data::read_audit::{
        TraceCapture, TraceFilter, TriggerCapture, TriggerCondition,
    };
    crate::configure_headless_widescreen();
    let root = PathBuf::from(std::env::var_os("FFTA_TEXT_READER_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let state_path = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_STATE").unwrap());
    assert_eq!(
        gba_core::ffta_rom::digest(&fs::read(&state_path).unwrap()),
        "cad8547dcbbdc28b284229c2328bf1fcf414e1d61ee86665206eb99ba9cca9da"
    );
    let input_path = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_INPUT").unwrap());
    assert_eq!(
        gba_core::ffta_rom::digest(&fs::read(&input_path).unwrap()),
        "3b91750c1465e04e8ed8c68b73ff81c9731c7d1c545245f06749c205fbc89c45"
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
    let ranges = vec![
        (0x37c580, 0x37c588),
        (0x37c790, 0x37c798),
        (0x37c8b0, 0x37c8b8),
        (0x37cf04, 0x37cf0c),
        (0x37cff4, 0x37cffc),
        (0x383c78, 0x383c80),
    ];
    let mut native = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut native, &dataset).unwrap();
    crate::save_state::load(&mut native, &state_path).unwrap();
    crate::arm_audio_hle(&mut native, None);
    native.bus.tap_channels = true;
    let clear = |m: &mut Machine| {
        m.bus.audio_buf.clear();
        m.bus.psg_tap.clear();
        m.bus.fifo_tap[0].clear();
        m.bus.fifo_tap[1].clear();
        m.bus.hle_tap.clear();
    };
    clear(&mut native);
    let start = native.bus.frames;
    let trigger = TriggerCapture::start(TriggerCondition::Read {
        ranges: ranges.clone(),
        readers: vec![],
    })
    .unwrap();
    let mut selected = None;
    for relative in 0..1800u64 {
        let before = native.save_state().unwrap();
        native.bus.keys = input.keys_at(relative);
        input.apply_mouse(&mut native.bus, relative).unwrap();
        trigger.frame(Some(native.bus.frames));
        let pointer = &mut native as *mut Machine as *mut std::ffi::c_void;
        let dispatch =
            crate::run_frame_native(&mut native, &table, pointer, crate::FRAME_STEP_GUARD);
        trigger.frame(None);
        assert_eq!(native.bus.frames, start + relative + 1);
        if let Some(hit) = trigger.hit() {
            selected = Some((relative, before, hit, dispatch));
            break;
        }
        clear(&mut native);
        if relative % 300 == 299 {
            eprintln!("SPECIAL_READER_LOCATOR frames={}", relative + 1);
        }
    }
    let (relative, before, hit, dispatch) =
        selected.expect("recorded scene must reach an observed index cell");
    drop(trigger);
    let mut interpreted = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut interpreted, &dataset).unwrap();
    interpreted.load_state(&before).unwrap();
    interpreted.bus.keys = input.keys_at(relative);
    input.apply_mouse(&mut interpreted.bus, relative).unwrap();
    let trace = TraceCapture::start(
        TraceFilter {
            readers: vec![],
            ranges,
        },
        10000,
    )
    .unwrap();
    trace.frame(Some(interpreted.bus.frames));
    let mut routine_entries = Vec::new();
    let mut target_loads = Vec::new();
    let mut upstream_entries = Vec::new();
    let mut last_entry = None;
    crate::run_frame_interpreter_observed(&mut interpreted, crate::FRAME_STEP_GUARD, |m| {
        let pc = m.cpu.regs[15];
        if matches!(pc, 0x0812_a8c4 | 0x0809_d9c4 | 0x0809_acf8 | 0x0809_ae1c) {
            assert!(
                upstream_entries.len() < 4096,
                "bounded upstream observations"
            );
            let task_fields = if pc == 0x0812_a8c4 {
                let address = m.cpu.regs[1];
                let (bytes, offset) = match address >> 24 {
                    2 => (&m.bus.ewram, (address & 0x3ffff) as usize),
                    3 => (&m.bus.iwram, (address & 0x7fff) as usize),
                    _ => panic!("task fields not in RAM"),
                };
                let fields = bytes
                    .get(offset..offset + 0x1a)
                    .expect("bounded task fields");
                Some(serde_json::json!({
                    "state":u16::from_le_bytes(fields[0..2].try_into().unwrap()),
                    "selector":u16::from_le_bytes(fields[0x14..0x16].try_into().unwrap()),
                    "group":u16::from_le_bytes(fields[0x16..0x18].try_into().unwrap()),
                    "flag":u16::from_le_bytes(fields[0x18..0x1a].try_into().unwrap())
                }))
            } else {
                None
            };
            upstream_entries.push(serde_json::json!({"regs":m.cpu.regs,"task_fields":task_fields}));
        }
        if pc == 0x0802_2788 {
            let sp = m.cpu.regs[13];
            let stack: Vec<Option<u32>> = (0..8)
                .map(|i| {
                    let at = sp.wrapping_add(i * 4);
                    let (bytes, offset) = match at >> 24 {
                        2 => (&m.bus.ewram, (at & 0x3ffff) as usize),
                        3 => (&m.bus.iwram, (at & 0x7fff) as usize),
                        _ => return None,
                    };
                    bytes
                        .get(offset..offset + 4)
                        .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
                })
                .collect();
            assert!(
                routine_entries.len() < 4096,
                "bounded routine entry observations"
            );
            routine_entries.push(serde_json::json!({"regs":m.cpu.regs,"stack_words":stack,"last_upstream":upstream_entries.len().checked_sub(1)}));
            last_entry = Some(routine_entries.len() - 1);
        }
        if pc == 0x0802_27a8 && matches!(m.cpu.regs[0], 0x0837_cff4 | 0x0837_cff8) {
            target_loads
                .push(serde_json::json!({"regs":m.cpu.regs,"last_routine_entry":last_entry}));
        }
    });
    assert_eq!(
        interpreted.bus.frames,
        start + relative + 1,
        "interpreter completed frame"
    );
    let trace = trace.finish();
    assert!(!trace.truncated && trace.dropped_requests == 0);
    let state_equal = native.save_state().unwrap() == interpreted.save_state().unwrap();
    let framebuffer_equal = native.bus.framebuffer == interpreted.bus.framebuffer;
    let oam_equal = native.bus.oam == interpreted.bus.oam;
    let events: Vec<_> = trace
        .events
        .iter()
        .map(|e| serde_json::to_value(e).unwrap())
        .collect();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"locate first recorded home scene read and replay its frame through interpreter",
        "relative_frame":relative, "native_hit":hit, "native_dispatches":dispatch.0, "native_fallback":dispatch.1,
        "interpreter_trace":events, "trace_dropped":trace.dropped_requests,
        "routine_entries":routine_entries, "upstream_entries":upstream_entries, "target_loads":target_loads,
        "frame_state_equal":state_equal, "framebuffer_equal":framebuffer_equal, "oam_equal":oam_equal,
        "limits":"A read's inherited interpreter scope need not be the load instruction; native dispatch may differ. Reader observation alone does not establish the original opcode-39 producer or animation. Other scenes and platforms remain open.", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "EUR home checkpoint, recorded input, E170 dataset and native library; Wine only"]
fn european_special_pose_task_creator_candidate() {
    use gba_core::rom_data::read_audit::{
        TraceCapture, TraceFilter, TriggerCapture, TriggerCondition,
    };
    crate::configure_headless_widescreen();
    let root = PathBuf::from(std::env::var_os("FFTA_TEXT_READER_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    assert_eq!(&rom[0xb95917..0xb9591c], &[39, 17, 16, 0, 0]);
    let state_path = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_STATE").unwrap());
    assert_eq!(
        gba_core::ffta_rom::digest(&fs::read(&state_path).unwrap()),
        "cad8547dcbbdc28b284229c2328bf1fcf414e1d61ee86665206eb99ba9cca9da"
    );
    let input_path = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_INPUT").unwrap());
    assert_eq!(
        gba_core::ffta_rom::digest(&fs::read(&input_path).unwrap()),
        "3b91750c1465e04e8ed8c68b73ff81c9731c7d1c545245f06749c205fbc89c45"
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
    let mut native = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut native, &dataset).unwrap();
    crate::save_state::load(&mut native, &state_path).unwrap();
    crate::arm_audio_hle(&mut native, None);
    native.bus.tap_channels = true;
    let clear = |m: &mut Machine| {
        m.bus.audio_buf.clear();
        m.bus.psg_tap.clear();
        m.bus.fifo_tap[0].clear();
        m.bus.fifo_tap[1].clear();
        m.bus.hle_tap.clear();
    };
    clear(&mut native);
    let start = native.bus.frames;
    let trigger = TriggerCapture::start(TriggerCondition::Read {
        ranges: vec![(0xb95917, 0xb9591c)],
        readers: vec![],
    })
    .unwrap();
    let mut selected = None;
    for frame in 0..346u64 {
        let before = native.save_state().unwrap();
        native.bus.keys = input.keys_at(frame);
        input.apply_mouse(&mut native.bus, frame).unwrap();
        trigger.frame(Some(native.bus.frames));
        let ptr = &mut native as *mut Machine as *mut std::ffi::c_void;
        let dispatch = crate::run_frame_native(&mut native, &table, ptr, crate::FRAME_STEP_GUARD);
        trigger.frame(None);
        assert_eq!(native.bus.frames, start + frame + 1);
        if let Some(hit) = trigger.hit() {
            selected = Some((frame, before, hit, dispatch));
            break;
        }
        clear(&mut native);
    }
    drop(trigger);
    let mut observation = serde_json::json!({"candidate_read_observed":false});
    if let Some((frame, before, hit, dispatch)) = selected {
        let mut interpreted = crate::make_machine(rom.clone(), None);
        attach_from_directory(&mut interpreted, &dataset).unwrap();
        interpreted.load_state(&before).unwrap();
        interpreted.bus.keys = input.keys_at(frame);
        input.apply_mouse(&mut interpreted.bus, frame).unwrap();
        let trace = TraceCapture::start(
            TraceFilter {
                readers: vec![],
                ranges: vec![(0xb95917, 0xb9591c)],
            },
            10000,
        )
        .unwrap();
        trace.frame(Some(interpreted.bus.frames));
        let mut entries = Vec::new();
        crate::run_frame_interpreter_observed(&mut interpreted, crate::FRAME_STEP_GUARD, |m| {
            let pc = m.cpu.regs[15];
            if matches!(
                pc,
                0x0812_6b54 | 0x0812_aa34 | 0x0812_569c | 0x0812_aa5a | 0x0812_a8c4
            ) {
                assert!(entries.len() < 4096);
                entries.push(serde_json::json!({"regs":m.cpu.regs}));
            }
        });
        assert_eq!(interpreted.bus.frames, start + frame + 1);
        let trace = trace.finish();
        assert!(!trace.truncated && trace.dropped_requests == 0);
        observation = serde_json::json!({"candidate_read_observed":true,"relative_frame":frame,"native_hit":hit,"native_dispatches":dispatch.0,"native_fallback":dispatch.1,"interpreter_trace":trace.events,"entries":entries,"trace_dropped":trace.dropped_requests,"frame_state_equal":native.save_state().unwrap()==interpreted.save_state().unwrap(),"framebuffer_equal":native.bus.framebuffer==interpreted.bus.framebuffer,"oam_equal":native.bus.oam==interpreted.bus.oam});
    }
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","scope":"bounded 346-frame recorded home candidate locator and interpreter replay if read","observation":observation,"limits":"Candidate byte reads do not by themselves prove opcode execution, task allocation identity or ancestry. Interpreter scopes can include nested work. Negative evidence is limited to this window.","performance_measurements":false})).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "EUR home checkpoint, recorded input, E170 dataset and native library; Wine only"]
fn european_special_pose_ordinary_body_pair() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    crate::configure_headless_widescreen();
    let root = PathBuf::from(std::env::var_os("FFTA_TEXT_READER_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let state_path = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_STATE").unwrap());
    let state = fs::read(&state_path).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&state),
        "cad8547dcbbdc28b284229c2328bf1fcf414e1d61ee86665206eb99ba9cca9da"
    );
    let input_path = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_INPUT").unwrap());
    assert_eq!(
        gba_core::ffta_rom::digest(&fs::read(&input_path).unwrap()),
        "3b91750c1465e04e8ed8c68b73ff81c9731c7d1c545245f06749c205fbc89c45"
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
        crate::save_state::load(m, &state_path).unwrap();
        crate::arm_audio_hle(m, None);
        m.bus.tap_channels = true;
        m.bus.audio_buf.clear();
        m.bus.psg_tap.clear();
        m.bus.fifo_tap[0].clear();
        m.bus.fifo_tap[1].clear();
        m.bus.hle_tap.clear();
    }
    assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
    // The authenticated sprite-48/animation-65 body index points at a
    // single-frame sequence whose OBJ list and pixel block are prior owners.
    let spans = [
        (0x37cff0usize, 4usize),
        (0xa9c660, 24),
        (0xa50046, 26),
        (0x974240, 256),
    ];
    assert_eq!(
        u32::from_le_bytes(rom[0x37cff0..0x37cff4].try_into().unwrap()),
        0x08a9c660
    );
    let resource = manifest
        .resources
        .iter()
        .find(|r| r.id == "graphics.unit-obj-normal-battle")
        .unwrap();
    for &(at, len) in &spans {
        assert!(
            (at..at + len).all(|b| resource
                .sources
                .iter()
                .any(|s| s.offset as usize <= b && b < (s.offset + s.length) as usize)),
            "each body byte must belong to the existing owner"
        );
    }
    let trace = TraceCapture::start(
        TraceFilter {
            readers: vec![],
            ranges: spans
                .iter()
                .map(|&(at, len)| (at as u32, (at + len) as u32))
                .collect(),
        },
        50000,
    )
    .unwrap();
    let start = raw.bus.frames;
    let mut audio = [0u64; 5];
    let mut dispatch_totals = (0u64, 0u64);
    for frame in 0..1800u64 {
        for &(at, len) in &spans {
            managed.bus.rom[at..at + len].fill(0xcd);
        }
        let mut counts = [(0, 0); 2];
        for (arm, m) in [&mut raw, &mut managed].into_iter().enumerate() {
            m.bus.keys = input.keys_at(frame);
            input.apply_mouse(&mut m.bus, frame).unwrap();
            trace.frame(if arm == 1 { Some(m.bus.frames) } else { None });
            let pointer = m as *mut Machine as *mut std::ffi::c_void;
            counts[arm] = crate::run_frame_native(m, &table, pointer, crate::FRAME_STEP_GUARD);
            assert_eq!(m.bus.frames, start + frame + 1);
        }
        trace.frame(None);
        assert_eq!(counts[0], counts[1], "dispatch at {frame}");
        dispatch_totals.0 += counts[0].0;
        dispatch_totals.1 += counts[0].1;
        assert_eq!(raw.bus.clock, managed.bus.clock, "clock at {frame}");
        assert_eq!(
            raw.bus.framebuffer, managed.bus.framebuffer,
            "framebuffer at {frame}"
        );
        assert_eq!(
            raw.bus.wide_framebuffer, managed.bus.wide_framebuffer,
            "wide framebuffer at {frame}"
        );
        assert_eq!(raw.bus.oam, managed.bus.oam, "OAM at {frame}");
        assert_eq!(raw.bus.audio_buf, managed.bus.audio_buf, "audio at {frame}");
        assert_eq!(raw.bus.psg_tap, managed.bus.psg_tap, "PSG at {frame}");
        assert_eq!(raw.bus.fifo_tap, managed.bus.fifo_tap, "FIFO at {frame}");
        assert_eq!(raw.bus.hle_tap.len(), managed.bus.hle_tap.len());
        assert!(
            raw.bus
                .hle_tap
                .iter()
                .zip(&managed.bus.hle_tap)
                .all(|(a, b)| a.to_bits() == b.to_bits()),
            "HLE at {frame}"
        );
        for (count, len) in audio.iter_mut().zip([
            managed.bus.audio_buf.len(),
            managed.bus.psg_tap.len(),
            managed.bus.fifo_tap[0].len(),
            managed.bus.fifo_tap[1].len(),
            managed.bus.hle_tap.len(),
        ]) {
            *count += len as u64;
        }
        for &(at, len) in &spans {
            managed.bus.rom[at..at + len].copy_from_slice(&rom[at..at + len]);
        }
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
        if frame % 300 == 299 {
            eprintln!("REGISTRY_ORDINARY frames={}", frame + 1);
        }
    }
    let trace = trace.finish();
    assert!(!trace.truncated && trace.dropped_requests == 0);
    let mut counts = [0u64; 4];
    let mut typed = [0u64; 4];
    let mut raw_reads = [0u64; 4];
    let mut unknown = [0u64; 4];
    for event in &trace.events {
        let start = event.key.read.physical_offset as usize;
        let end = start + event.key.read.length;
        for (i, &(at, len)) in spans.iter().enumerate() {
            if start < at + len && end > at {
                counts[i] += event.requests;
                match event.key.service {
                    Service::Typed => typed[i] += event.requests,
                    Service::Raw => raw_reads[i] += event.requests,
                    Service::Unclassified => unknown[i] += event.requests,
                }
            }
        }
    }
    assert!(counts[0] > 0, "the selected body index must be read");
    assert!(
        audio.iter().all(|&n| n > 0),
        "audio streams must have samples: {audio:?}"
    );
    assert!(dispatch_totals.0 > 0, "generated dispatch required");
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({
      "status":"PASS", "scope":"1800 recorded home frames with selected body sources altered; EUR Wine",
      "revision":[170,169], "frames":1800, "roles":["index","sequence","objects","pixels"],
      "requests":counts,"typed_requests":typed,"raw_requests":raw_reads,"unclassified_requests":unknown,
      "trace_dropped":trace.dropped_requests,"poisoned_bytes":[4,24,26,256],
      "full_state_clock_video_oam_audio_equal_each_frame":true,"audio_sample_counts":audio,
      "generated_dispatches":dispatch_totals.0,"fallback_dispatches":dispatch_totals.1,
      "limits":"Address observation alone does not establish a full animation lifecycle or exact instruction attribution; generated dispatch may fall back to interpretation.","performance_measurements":false
    })).unwrap()).unwrap();
}
