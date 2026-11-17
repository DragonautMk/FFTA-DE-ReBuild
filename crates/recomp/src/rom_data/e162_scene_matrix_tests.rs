//! Bounded ordinary EUR scene comparisons for the current managed dataset.
use super::*;
use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};

#[test]
#[ignore = "EUR ROM, e162 dataset and retained checkpoints; Wine only"]
fn european_e162_scene_matrix_interpreter_pairs() {
    const CALLBACK_BASE: u32 = gba_core::rom_data::text_catalogs::SUBSTITUTION_ROOT;
    const RANGE_START: u32 = gba_core::rom_data::range_lookup::SOURCE;
    const GEOMETRY_START: u32 = gba_core::rom_data::tilemap_geometry::SOURCE;
    const GEOMETRY_END: u32 = GEOMETRY_START + gba_core::rom_data::tilemap_geometry::SOURCE_BYTES as u32;
    let root = PathBuf::from(std::env::var_os("FFTA_SCENE_MATRIX_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_SCENE_MATRIX_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision, manifest.decoder_revision), (162, 161));
    let texts: gba_core::rom_data::text_catalogs::TextCatalogs =
        serde_json::from_slice(&fs::read(dataset.join(TEXT_CATALOGS_FILE)).unwrap()).unwrap();
    texts.validate().unwrap();
    assert_eq!(texts.substitution_callbacks.len(), 75);
    let mut ranges: Vec<(u32, u32)> = texts.substitution_callbacks.iter()
        .map(|callback| (CALLBACK_BASE + u32::from(callback.index) * 4, CALLBACK_BASE + u32::from(callback.index) * 4 + 4)).collect();
    ranges.push((RANGE_START, GEOMETRY_END));
    let cases: [(&str, &str, &str, [u16; 3]); 4] = [
        ("opening", "FFTA_SCENE_OPENING", "0a439a15a0de6ccda64a8010d50da37b0ba5a4435b683adf45fb7cf68dec56e3", [0x3ff, 0x3ff, 0x3ff]),
        ("world_menu", "FFTA_SCENE_WORLD_MENU", "b379038c2a3b6e1740fd70fe78f9f5eaf764d4d2f5eac7fa316d4640e0efb808", [0x3fe, 0x3fe, 0x3ff]),
        ("language_selector", "FFTA_SCENE_LANGUAGE", "ce54d2693068161aa1452835109c7a44082fa9bd4e2777ab6976cddd88e192e1", [0x37f, 0x37f, 0x3ff]),
        ("battle_menu", "FFTA_SCENE_BATTLE", "31b837594b1ede7f2c0d521d3db939a4cc96a1880946bd631e9dc9c027c4f409", [0x3fe, 0x3fe, 0x3ff]),
    ];
    let mut observations = Vec::new();
    for (name, env, expected, keys) in cases {
        let state = fs::read(std::env::var_os(env).unwrap()).unwrap();
        assert_eq!(gba_core::ffta_rom::digest(&state), expected, "{name} checkpoint");
        let mut raw = crate::make_machine(rom.clone(), None);
        let mut managed = crate::make_machine(rom.clone(), None);
        attach_from_directory(&mut managed, &dataset).unwrap();
        raw.load_state(&state).unwrap();
        managed.load_state(&state).unwrap();
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "{name} initial state");
        let owner = managed.bus.rom_resources().unwrap().clone();
        let before_callback = owner.text_catalog_stats().unwrap().substitution_callback_bytes;
        let before_range = owner.range_lookup_stats().unwrap().source_bytes;
        let before_geometry = owner.tilemap_geometry_stats().unwrap().source_bytes;
        let mut typed_requests = 0u64;
        let mut raw_requests = 0u64;
        let mut unclassified_requests = 0u64;
        for (frame, key) in keys.into_iter().enumerate() {
            raw.bus.keys = key;
            managed.bus.keys = key;
            raw.bus.frame_ready = false;
            for _ in 0..5_000_000 { if raw.bus.frame_ready { break; } raw.step(); }
            assert!(raw.bus.frame_ready, "{name} raw frame {frame}");
            for &(lo, hi) in &ranges { managed.bus.rom[lo as usize..hi as usize].fill(0xcd); }
            // The trace API accepts at most 32 ranges. Capture the enclosing
            // interval, then count only events overlapping an owned range.
            let capture = TraceCapture::start(TraceFilter { readers: vec![], ranges: vec![(CALLBACK_BASE, GEOMETRY_END)] }, 10_000).unwrap();
            capture.frame(Some(managed.bus.frames));
            managed.bus.frame_ready = false;
            for _ in 0..5_000_000 { if managed.bus.frame_ready { break; } managed.step(); }
            assert!(managed.bus.frame_ready, "{name} managed frame {frame}");
            let trace = capture.finish();
            assert!(!trace.truncated, "{name} trace frame {frame}");
            for event in trace.events {
                let read = event.key.read;
                let end = u64::from(read.physical_offset) + read.length as u64;
                if !ranges.iter().any(|&(lo, hi)| u64::from(lo) < end && hi > read.physical_offset) {
                    continue;
                }
                match event.key.service {
                    Service::Typed => typed_requests += event.requests,
                    Service::Raw => raw_requests += event.requests,
                    Service::Unclassified => unclassified_requests += event.requests,
                }
            }
            for &(lo, hi) in &ranges { managed.bus.rom[lo as usize..hi as usize].copy_from_slice(&rom[lo as usize..hi as usize]); }
            assert_eq!(raw.bus.framebuffer, managed.bus.framebuffer, "{name} image frame {frame}");
            assert_eq!(raw.bus.wide_framebuffer, managed.bus.wide_framebuffer, "{name} wide image frame {frame}");
            assert_eq!(raw.bus.audio_buf, managed.bus.audio_buf, "{name} audio frame {frame}");
            assert_eq!(raw.bus.oam, managed.bus.oam, "{name} OAM frame {frame}");
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "{name} full state/clock frame {frame}");
        }
        assert_eq!(raw_requests, 0, "{name} read from a covered range without a typed owner");
        observations.push(serde_json::json!({
            "scene": name, "frames": keys.len(), "typed_requests": typed_requests,
            "raw_requests": raw_requests, "unclassified_requests": unclassified_requests,
            "substitution_callback_bytes": owner.text_catalog_stats().unwrap().substitution_callback_bytes - before_callback,
            "range_source_bytes": owner.range_lookup_stats().unwrap().source_bytes - before_range,
            "geometry_source_bytes": owner.tilemap_geometry_stats().unwrap().source_bytes - before_geometry,
        }));
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"four retained EUR scene checkpoints, three input frames each; raw/managed interpreter",
        "revision":[162,161], "scenes":observations, "poisoned_source":true,
        "comparisons":"full serialized machine and guest clock, framebuffer, wide framebuffer, OAM and audio at each frame",
        "limits":"Short bounded windows only; native dispatch, full scenes, other inputs and complete source domains remain open.",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
