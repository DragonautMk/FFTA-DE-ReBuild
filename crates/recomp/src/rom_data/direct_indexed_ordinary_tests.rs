//! Bounded ordinary EUR source lead and, on a witness, full scene equivalence.
use super::*;
use gba_core::{
    rom_data::{direct_indexed_dictionary, direct_indexed_graphics},
    Machine,
};

fn clear_audio(m: &mut Machine) {
    m.bus.audio_buf.clear();
    m.bus.psg_tap.clear();
    m.bus.fifo_tap[0].clear();
    m.bus.fifo_tap[1].clear();
    m.bus.hle_tap.clear();
}

#[test]
#[ignore = "retained cold-origin EUR input and historical endpoint; Wine only"]
fn ordinary_direct_indexed_route_historical_endpoint() {
    crate::configure_headless_widescreen();
    let root = PathBuf::from(std::env::var_os("FFTA_DIRECT_ORDINARY_ROOT").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let state = fs::read(std::env::var_os("FFTA_STATE_TEST_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "4db5a60bb293bb319e685fb317b9a31a85df8d0d872d859bcedb07c0ecc4ea22");
    let historical = fs::read(std::env::var_os("FFTA_DIRECT_HISTORICAL_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&historical), "6b48c830e5da559da1233c8a41ed41f9ae1c5c67b09da28e1100c8da97b43d86");
    let input_path = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_INPUT").unwrap());
    assert_eq!(gba_core::ffta_rom::digest(&fs::read(&input_path).unwrap()), "802ec824a4d039a7721b682fd4543091b383793b1ced66812ecff4f92d1fee0b");
    let input = crate::InputScript::load(input_path.to_str().unwrap()).unwrap();
    let (_library, table) = crate::load_native(&PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap())).unwrap();
    let mut replay = crate::make_machine(rom.clone(), None);
    replay.load_state(&state).unwrap();
    crate::arm_audio_hle(&mut replay, None);
    replay.bus.tap_channels = true;
    clear_audio(&mut replay);
    assert_eq!(replay.bus.frames, 180);
    for frame in 0..8_820u64 {
        replay.bus.keys = input.keys_at(frame);
        input.apply_mouse(&mut replay.bus, frame).unwrap();
        let pointer = &mut replay as *mut Machine as *mut std::ffi::c_void;
        crate::run_frame_native(&mut replay, &table, pointer, crate::FRAME_STEP_GUARD);
        assert_eq!(replay.bus.frames, frame + 181);
        clear_audio(&mut replay);
        if (frame + 1) % 1_000 == 0 { println!("INDEXED_ENDPOINT frames={}/8820", frame + 1); }
    }
    let mut original = crate::make_machine(rom, None);
    original.load_state(&historical).unwrap();
    assert_eq!(original.bus.frames, 9_000);
    // The historical capture did not arm host-side audio shadows or taps.
    // Remove those observation-only fields from both copies before comparing.
    for machine in [&mut replay, &mut original] {
        clear_audio(machine);
        machine.bus.tap_channels = false;
        machine.bus.mp2k = None; machine.bus.gax = None; machine.bus.rdrv = None;
    }
    let actual = replay.save_state().unwrap();
    let expected = original.save_state().unwrap();
    let equal = actual == expected;
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":if equal { "PASS" } else { "FAIL" }, "frames":8_820, "endpoint_frame":9_000,
        "normalized_complete_states_equal":equal,
        "replayed_sha256":gba_core::ffta_rom::digest(&actual), "historical_sha256":gba_core::ffta_rom::digest(&expected),
        "normalized_fields":"output audio, tap flag and host-side MP2K/GAX/RDRV shadows",
        "limits":"One recorded route, normalized audio observation fields; not equality of the original undrained snapshots or other scenes.",
        "performance_measurements":false
    })).unwrap()).unwrap();
    if !equal {
        fs::write(root.join("replayed-normalized.ffstate"), actual).unwrap();
        fs::write(root.join("historical-normalized.ffstate"), expected).unwrap();
    }
    assert!(equal, "historical frame-9000 normalized state differs");
}

#[test]
#[ignore = "retained cold-origin EUR input, E170 dataset and translation; Wine only"]
fn ordinary_direct_indexed_archives_full_recorded_route() {
    crate::configure_headless_widescreen();
    let root = PathBuf::from(std::env::var_os("FFTA_DIRECT_ORDINARY_ROOT").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let state = fs::read(std::env::var_os("FFTA_STATE_TEST_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "4db5a60bb293bb319e685fb317b9a31a85df8d0d872d859bcedb07c0ecc4ea22");
    let input_path = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_INPUT").unwrap());
    let input_bytes = fs::read(&input_path).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&input_bytes), "802ec824a4d039a7721b682fd4543091b383793b1ced66812ecff4f92d1fee0b");
    let input = crate::InputScript::load(input_path.to_str().unwrap()).unwrap();
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision, manifest.decoder_revision, manifest.resources.len()), (170, 169, 113));
    let (_library, table) = crate::load_native(&PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap())).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &dataset).unwrap();
    for machine in [&mut raw, &mut managed] {
        machine.load_state(&state).unwrap();
        crate::arm_audio_hle(machine, None);
        machine.bus.tap_channels = true;
        clear_audio(machine);
    }
    assert_eq!(raw.bus.frames, 180);
    assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
    let spans = [
        (direct_indexed_graphics::CALLER_LITERAL as usize, 4usize),
        (direct_indexed_graphics::ARCHIVE_OFFSET as usize, direct_indexed_graphics::ARCHIVE_LENGTH),
        (direct_indexed_dictionary::CALLER_LITERAL as usize, 4usize),
        (direct_indexed_dictionary::ARCHIVE_OFFSET as usize, direct_indexed_dictionary::ARCHIVE_LENGTH),
    ];
    let owner = managed.bus.rom_resources().unwrap().clone();
    let mut samples = [0u64; 5];
    let mut dispatch_total = (0u64, 0u64);
    const FRAMES: u64 = 8_820;
    for frame in 0..FRAMES {
        for (at, len) in spans {
            for (target, &original) in managed.bus.rom[at..at + len].iter_mut().zip(&rom[at..at + len]) { *target = original ^ 0xff; }
        }
        let mut dispatch = [(0u64, 0u64); 2];
        for (arm, machine) in [&mut raw, &mut managed].into_iter().enumerate() {
            machine.bus.keys = input.keys_at(frame);
            input.apply_mouse(&mut machine.bus, frame).unwrap();
            let pointer = machine as *mut Machine as *mut std::ffi::c_void;
            dispatch[arm] = crate::run_frame_native(machine, &table, pointer, crate::FRAME_STEP_GUARD);
            assert_eq!(machine.bus.frames, 181 + frame);
        }
        assert_eq!(dispatch[0], dispatch[1], "dispatch frame {frame}");
        dispatch_total.0 += dispatch[0].0; dispatch_total.1 += dispatch[0].1;
        assert_eq!(raw.bus.clock, managed.bus.clock, "clock frame {frame}");
        assert_eq!(raw.bus.framebuffer, managed.bus.framebuffer, "framebuffer frame {frame}");
        assert_eq!(raw.bus.wide_framebuffer, managed.bus.wide_framebuffer, "wide framebuffer frame {frame}");
        assert_eq!(raw.bus.oam, managed.bus.oam, "OAM frame {frame}");
        assert_eq!(raw.bus.audio_buf, managed.bus.audio_buf, "mixed audio frame {frame}");
        assert_eq!(raw.bus.psg_tap, managed.bus.psg_tap, "PSG frame {frame}");
        assert_eq!(raw.bus.fifo_tap, managed.bus.fifo_tap, "FIFO frame {frame}");
        assert!(raw.bus.hle_tap.iter().map(|x| x.to_bits()).eq(managed.bus.hle_tap.iter().map(|x| x.to_bits())), "HLE frame {frame}");
        for (count, n) in samples.iter_mut().zip([managed.bus.audio_buf.len(), managed.bus.psg_tap.len(), managed.bus.fifo_tap[0].len(), managed.bus.fifo_tap[1].len(), managed.bus.hle_tap.len()]) { *count += n as u64; }
        for (at, len) in spans { managed.bus.rom[at..at + len].copy_from_slice(&rom[at..at + len]); }
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "full state frame {frame}");
        clear_audio(&mut raw); clear_audio(&mut managed);
        if (frame + 1) % 500 == 0 { println!("INDEXED_FULL_ROUTE frames={}/{}", frame + 1, FRAMES); }
    }
    let first = owner.direct_indexed_graphics_stats().unwrap();
    let second = owner.direct_indexed_dictionary_stats().unwrap();
    assert!(first.source_bytes > 0 && second.source_bytes > 0, "both archives must be read during the route");
    assert!(samples[0] > 0 && dispatch_total.0 > 0);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"recorded EUR cold-origin input from frame 180 through frame 9000",
        "revision":[170,169], "frames":FRAMES, "first_frame":180, "last_frame":raw.bus.frames,
        "first_archive_source_bytes":first.source_bytes, "second_archive_source_bytes":second.source_bytes,
        "selected_source_spans_inverted_during_guest_execution":true,
        "full_state_clock_video_oam_dispatch_five_audio_streams_equal_each_frame":true,
        "audio_sample_counts":samples, "native_dispatches":dispatch_total.0, "fallback_dispatches":dispatch_total.1,
        "limits":"One recorded route; generated dispatch may fall back. Does not prove original historical endpoint bytes, per-read native attribution, all scenes or consumers, or other platforms.",
        "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "retained genuine EUR state/input, E167 and native library; Wine only"]
fn ordinary_direct_indexed_archives_source_and_equivalence() {
    crate::configure_headless_widescreen();
    let root = PathBuf::from(std::env::var_os("FFTA_DIRECT_ORDINARY_ROOT").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let input_path = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_INPUT").unwrap());
    let input = crate::InputScript::load(input_path.to_str().unwrap()).unwrap();
    assert!(
        fs::read_to_string(&input_path)
            .unwrap()
            .lines()
            .skip(1)
            .all(|line| line.split_whitespace().count() == 2),
        "fixture uses recorded keys only"
    );
    let state = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_STATE").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let (_library, table) = crate::load_native(&PathBuf::from(
        std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap(),
    ))
    .unwrap();
    let frames: u64 = std::env::var("FFTA_STATE_TEST_FRAMES")
        .unwrap()
        .parse()
        .unwrap();
    assert!((1..=6000).contains(&frames));
    let trace_ds20 = std::env::var_os("FFTA_DIRECT_TRACE_DS20").is_some();
    if trace_ds20 {
        assert!(frames <= 480 && (0..frames).all(|frame| input.keys_at(frame) == 0x03ff));
        assert!(fs::read(&state).unwrap().starts_with(b"FFTA-SL1"));
    }
    let mut trace = Vec::new();
    let mut observed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut observed, &dataset).unwrap();
    let ancestry = crate::save_state::load_with_history(&mut observed, &state).unwrap();
    assert!(
        ancestry.is_empty(),
        "ordinary witness cannot have recorded cheat ancestry"
    );
    if trace_ds20 {
        let ewram = &observed.bus.ewram;
        assert_eq!(
            u32::from_le_bytes(ewram[0x163b8..0x163bc].try_into().unwrap()),
            0x08ba_d84c
        );
        assert_eq!(
            u16::from_le_bytes(ewram[0x163ac..0x163ae].try_into().unwrap()),
            0x0063
        );
        assert_eq!(ewram[0x1fb8] & 0x10, 0);
    }
    crate::arm_audio_hle(&mut observed, None);
    observed.bus.tap_channels = true;
    clear_audio(&mut observed);
    let first_frame = observed.bus.frames;
    let mut rolling_state = observed.save_state().unwrap();
    let mut rolling_relative = 0u64;
    let mut selected = None;
    let mut keys = Vec::new();
    let mut first_hit = None;
    let mut observed_bytes = (0, 0);
    for relative in 0..frames {
        if selected.is_none() && relative % 16 == 0 {
            rolling_state = observed.save_state().unwrap();
            rolling_relative = relative;
        }
        let key = input.keys_at(relative);
        keys.push(key);
        observed.bus.keys = key;
        input.apply_mouse(&mut observed.bus, relative).unwrap();
        if trace_ds20 {
            // Interpreter observation sees every instruction, including code
            // chained within a generated block. Keep this diagnostic distinct
            // from a proof about the legacy checkpoint's prior ancestry.
            crate::run_frame_interpreter_observed(&mut observed, crate::FRAME_STEP_GUARD, |m| {
                let pc = m.cpu.regs[15];
                let row = match pc {
                    0x0812_75bc if m.cpu.regs[1] == 0x08ba_d8bc => Some(serde_json::json!({
                        "kind":"program160_condition", "frame":relative,
                        "cursor_address":m.cpu.regs[0], "flag_byte":m.bus.ewram[0x1fb8]
                    })),
                    0x0812_75d2 if m.cpu.regs[4] == 0x08ba_d8bc => Some(serde_json::json!({
                        "kind":"program160_condition_branch", "frame":relative,
                        "zero_flag":m.cpu.flag(gba_core::cpu::FLAG_Z)
                    })),
                    0x0812_56ec => Some(serde_json::json!({
                        "kind":"request_setter", "frame":relative,
                        "value":m.cpu.regs[0], "return_pc":m.cpu.regs[14]
                    })),
                    0x0812_5558 => {
                        let ram = &m.bus.ewram;
                        let owner = u32::from_le_bytes(ram[0x159d4..0x159d8].try_into().unwrap());
                        let value = owner
                            .checked_sub(0x0200_0000)
                            .and_then(|offset| ram.get(offset as usize + 0x0c))
                            .copied();
                        Some(
                            serde_json::json!({"kind":"dispatcher", "frame":relative, "value":value}),
                        )
                    }
                    0x0802_3c98 => Some(serde_json::json!({"kind":"loader", "frame":relative})),
                    _ => None,
                };
                if let Some(row) = row {
                    assert!(trace.len() < 4096, "diagnostic trace bound");
                    trace.push(row);
                }
            });
        } else {
            let pointer = &mut observed as *mut Machine as *mut std::ffi::c_void;
            crate::run_frame_native(&mut observed, &table, pointer, crate::FRAME_STEP_GUARD);
        }
        assert_eq!(observed.bus.frames, first_frame + relative + 1);
        clear_audio(&mut observed);
        let owner = observed.bus.rom_resources().unwrap();
        let first = owner.direct_indexed_graphics_stats().unwrap();
        let second = owner.direct_indexed_dictionary_stats().unwrap();
        observed_bytes = (first.source_bytes, second.source_bytes);
        if first.source_bytes + second.source_bytes > 0 && selected.is_none() {
            selected = Some((rolling_relative, rolling_state.clone()));
            first_hit = Some(relative);
        }
        if first_hit.is_some_and(|at| relative >= at + 3) {
            break;
        }
    }
    assert_eq!(
        gba_core::ffta_rom::digest(&observed.bus.rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    // Keep an authentic endpoint for a source-guided follow-up if no archive
    // is reached. These are observations, not claims that a candidate pointer
    // names the active script task.
    let ewram = &observed.bus.ewram;
    let owner = u32::from_le_bytes(ewram[0x159d4..0x159d8].try_into().unwrap());
    let request = owner
        .checked_sub(0x0200_0000)
        .and_then(|offset| ewram.get(offset as usize + 0x0c))
        .copied();
    let mut program_pointer_candidates = Vec::new();
    for offset in (12..ewram.len() - 3).step_by(4) {
        let word = u32::from_le_bytes(ewram[offset..offset + 4].try_into().unwrap());
        if matches!(word, 0x08b9_4d07 | 0x08b9_5aa1) {
            let nearby = u16::from_le_bytes(ewram[offset - 12..offset - 10].try_into().unwrap());
            program_pointer_candidates.push(serde_json::json!({
                "pointer_location":0x0200_0000u32 + offset as u32,
                "program_pointer":word, "halfword_12_bytes_before":nearby
            }));
        }
    }
    let end_flag_40 = ewram[0x1f78] & 1 != 0;
    fs::write(root.join("end.ffstate"), observed.save_state().unwrap()).unwrap();
    crate::dump_frame(
        &observed,
        Some(root.join("end.ppm").to_string_lossy().into_owned()),
    )
    .unwrap();

    let mut compared_frames = 0usize;
    let mut pair_bytes = (0, 0);
    let mut audio_nonempty = false;
    if let Some((relative, snapshot)) = selected {
        let mut raw = crate::make_machine(rom.clone(), None);
        let mut managed = crate::make_machine(rom.clone(), None);
        attach_from_directory(&mut managed, &dataset).unwrap();
        raw.load_state(&snapshot).unwrap();
        managed.load_state(&snapshot).unwrap();
        for m in [&mut raw, &mut managed] {
            crate::arm_audio_hle(m, None);
            m.bus.tap_channels = true;
            clear_audio(m);
        }
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
        let spans = [
            (direct_indexed_graphics::CALLER_LITERAL as usize, 4usize),
            (
                direct_indexed_graphics::ARCHIVE_OFFSET as usize,
                direct_indexed_graphics::ARCHIVE_LENGTH,
            ),
            (direct_indexed_dictionary::CALLER_LITERAL as usize, 4usize),
            (
                direct_indexed_dictionary::ARCHIVE_OFFSET as usize,
                direct_indexed_dictionary::ARCHIVE_LENGTH,
            ),
        ];
        let poison = |m: &mut Machine| {
            for (at, n) in spans {
                m.bus.rom[at..at + n].fill(0xcd);
            }
        };
        let restore = |m: &mut Machine| {
            for (at, n) in spans {
                m.bus.rom[at..at + n].copy_from_slice(&rom[at..at + n]);
            }
        };
        let owner = managed.bus.rom_resources().unwrap().clone();
        let start = raw.bus.frames;
        poison(&mut managed);
        for (index, &key) in keys.iter().skip(relative as usize).enumerate() {
            let mut dispatch = [(0, 0); 2];
            for (arm, m) in [&mut raw, &mut managed].into_iter().enumerate() {
                m.bus.keys = key;
                input
                    .apply_mouse(&mut m.bus, relative + index as u64)
                    .unwrap();
                let pointer = m as *mut Machine as *mut std::ffi::c_void;
                dispatch[arm] =
                    crate::run_frame_native(m, &table, pointer, crate::FRAME_STEP_GUARD);
                assert_eq!(m.bus.frames, start + index as u64 + 1);
            }
            assert_eq!(dispatch[0], dispatch[1], "dispatch at {index}");
            assert_eq!(raw.bus.clock, managed.bus.clock, "clock at {index}");
            assert_eq!(
                raw.bus.framebuffer, managed.bus.framebuffer,
                "framebuffer at {index}"
            );
            assert_eq!(
                raw.bus.wide_framebuffer, managed.bus.wide_framebuffer,
                "wide framebuffer at {index}"
            );
            assert_eq!(raw.bus.oam, managed.bus.oam, "OAM at {index}");
            assert_eq!(raw.bus.audio_buf, managed.bus.audio_buf, "audio at {index}");
            assert_eq!(raw.bus.psg_tap, managed.bus.psg_tap, "PSG at {index}");
            assert_eq!(raw.bus.fifo_tap, managed.bus.fifo_tap, "FIFO at {index}");
            assert_eq!(raw.bus.hle_tap.len(), managed.bus.hle_tap.len());
            assert!(
                raw.bus
                    .hle_tap
                    .iter()
                    .zip(&managed.bus.hle_tap)
                    .all(|(a, b)| a.to_bits() == b.to_bits()),
                "HLE audio at {index}"
            );
            audio_nonempty |= !managed.bus.audio_buf.is_empty();
            restore(&mut managed);
            assert_eq!(
                raw.save_state().unwrap(),
                managed.save_state().unwrap(),
                "full state at {index}"
            );
            poison(&mut managed);
            clear_audio(&mut raw);
            clear_audio(&mut managed);
            compared_frames += 1;
        }
        let first = owner.direct_indexed_graphics_stats().unwrap();
        let second = owner.direct_indexed_dictionary_stats().unwrap();
        pair_bytes = (first.source_bytes, second.source_bytes);
        assert!(
            observed_bytes.0 == 0 || pair_bytes.0 > 0,
            "first archive witness must repeat"
        );
        assert!(
            observed_bytes.1 == 0 || pair_bytes.1 > 0,
            "second archive witness must repeat"
        );
        assert!(compared_frames > 0);
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"bounded retained EUR ordinary key input; conditional positive only",
        "positive":first_hit.is_some(), "first_frame":first_frame,
        "first_hit_relative":first_hit, "observed_frames":keys.len(),
        "first_archive_source_bytes":observed_bytes.0, "second_archive_source_bytes":observed_bytes.1,
        "end_request_owner":owner, "end_request_byte":request, "end_flag_40":end_flag_40,
        "end_program_pointer_candidates":program_pointer_candidates,
        "pair_first_archive_source_bytes":pair_bytes.0, "pair_second_archive_source_bytes":pair_bytes.1,
        "compared_frames":compared_frames, "pair_full_state_clock_framebuffer_oam_audio_equal":first_hit.is_some(),
        "audio_nonempty":audio_nonempty, "managed_sources_poisoned_during_pair":first_hit.is_some(),
        "locator_dispatch":if trace_ds20 { "interpreter instruction observation" } else { "generated blocks with interpreter fallback" },
        "recorded_ancestry_empty":ancestry.is_empty(), "ancestry_proven_clean":false,
        "ancestry_scope":"No recorded ancestry does not establish absence of earlier cheats", "ds20_trace":trace,
        "pair_dispatch":"generated blocks with interpreter fallback", "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
