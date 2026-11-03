//! Private normal-input navigation and passive new-owner activation observation.
//! No manual guest RAM/ROM/task mutation; checkpoints are normal gameplay copies.
use gba_core::{rom_data::obj_lists::constructor_animation::LIST_ROOTS, Machine};
use std::{fs, path::PathBuf};

fn clear_audio(m: &mut Machine) {
    m.bus.audio_buf.clear();
    m.bus.psg_tap.clear();
    m.bus.fifo_tap[0].clear();
    m.bus.fifo_tap[1].clear();
    m.bus.hle_tap.clear();
}
fn frame_image(m: &Machine, path: PathBuf) {
    let mut ppm = b"P6\n240 160\n255\n".to_vec();
    for &pixel in &m.bus.framebuffer {
        ppm.extend_from_slice(&[
            ((pixel & 31) * 255 / 31) as u8,
            (((pixel >> 5) & 31) * 255 / 31) as u8,
            (((pixel >> 10) & 31) * 255 / 31) as u8,
        ]);
    }
    fs::write(path, ppm).unwrap();
}

#[test]
#[ignore = "authorized private EUR copies/normal game inputs, passive positive fixture discovery; Wine only"]
fn normal_gameplay_observes_constructor_animation() {
    crate::configure_headless_widescreen();
    let root = PathBuf::from(std::env::var_os("FFTA_CONSTRUCTOR_SCENE_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let (_library, table) = crate::load_native(&PathBuf::from(
        std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap(),
    ))
    .unwrap();
    let input_path = std::env::var("FFTA_STATE_TEST_INPUT").unwrap();
    let input = crate::InputScript::load(&input_path).unwrap();
    let input_text = fs::read_to_string(&input_path).unwrap();
    assert!(
        input_text
            .lines()
            .skip(1)
            .all(|line| line.split_whitespace().count() == 2),
        "this normal-key recorder has no mouse events"
    );
    let frames: u64 = std::env::var("FFTA_STATE_TEST_FRAMES")
        .unwrap()
        .parse()
        .unwrap();
    assert!(frames > 0);
    let mut m = crate::make_machine(rom, None);
    super::attach_from_directory(
        &mut m,
        &PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap()),
    )
    .unwrap();
    crate::save_state::load(
        &mut m,
        &PathBuf::from(std::env::var_os("FFTA_STATE_TEST_STATE").unwrap()),
    )
    .unwrap();
    crate::arm_audio_hle(&mut m, None);
    m.bus.tap_channels = true;
    clear_audio(&mut m);
    let first = m.bus.frames;
    let mut rolling = m.save_state().unwrap();
    let mut rolling_relative = 0;
    let mut selected = None;
    let mut keys = Vec::new();
    let mut observations = Vec::new();
    let mut positive = false;
    for relative in 0..frames {
        if selected.is_none() && relative % 16 == 0 {
            rolling = m.save_state().unwrap();
            rolling_relative = relative;
        }
        let key = input.keys_at(relative);
        keys.push(key);
        m.bus.keys = key;
        input.apply_mouse(&mut m.bus, relative).unwrap();
        let pointer = &mut m as *mut Machine as *mut std::ffi::c_void;
        crate::run_frame_native(&mut m, &table, pointer, crate::FRAME_STEP_GUARD);
        assert_eq!(m.bus.frames, first + relative + 1);
        clear_audio(&mut m);
        let owner = m.bus.rom_resources().unwrap();
        let stats = owner.constructor_obj_animation_stats().unwrap();
        let used: Vec<_> = LIST_ROOTS
            .into_iter()
            .map(|(offset, _)| (offset, owner.obj_list_source_used(offset)))
            .collect();
        if stats.source_bytes > 0 && selected.is_none() {
            selected = Some((rolling_relative, rolling.clone()));
        }
        positive = stats.threshold_bytes > 0
            && stats.pointer_bytes > 0
            && stats.phases_used == 4
            && used.iter().all(|(_, used)| *used == Some((true, true)));
        if relative % 16 == 0 || stats.source_bytes > 0 {
            observations.push(serde_json::json!({"relative":relative,"guest_frame":m.bus.frames,"bus_reads":stats.bus_reads,"source_bytes":stats.source_bytes,"threshold_bytes":stats.threshold_bytes,"pointer_bytes":stats.pointer_bytes,"phases_used":stats.phases_used,"lists_guest_host":used}));
        }
        if positive {
            break;
        }
    }
    assert_eq!(
        gba_core::ffta_rom::digest(&m.bus.rom),
        gba_core::ffta_rom::EUROPE_SHA256,
        "original ROM remains unmodified"
    );
    fs::write(root.join("navigation-end.ffstate"), m.save_state().unwrap()).unwrap();
    frame_image(&m, root.join("navigation-end.ppm"));
    let mut suffix_frames = None;
    if let Some((relative, state)) = selected {
        fs::write(root.join("before-activation.ffstate"), state).unwrap();
        let mut suffix = String::from("gba-input v1\n");
        for (frame, key) in keys.iter().skip(relative as usize).enumerate() {
            suffix.push_str(&format!("{frame} {key:03x}\n"));
        }
        fs::write(root.join("activation.input"), suffix).unwrap();
        suffix_frames = Some(keys.len() as u64 - relative);
    }
    let owner = m.bus.rom_resources().unwrap();
    let stats = owner.constructor_obj_animation_stats().unwrap();
    let report = serde_json::json!({
        "status":if positive {"POSITIVE/normalgameplay"}else{"NEGATIVE/selectednormalinputbound"},
        "first_guest_frame":first,"last_guest_frame":m.bus.frames,"executed_frames":keys.len(),
        "positive":positive,"suffix_frames":suffix_frames,"keys":keys,"observations":observations,
        "new_table":{"bus_reads":stats.bus_reads,"source_bytes":stats.source_bytes,"threshold_bytes":stats.threshold_bytes,"pointer_bytes":stats.pointer_bytes,"phases_used":stats.phases_used},
        "scope":"one ordinary decoded arm only, normalinput controls and passive source stats, original ROM/state/input preserved; no equivalence or positive claim unless all four phases/lists guest-host", "Linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false
    });
    fs::write(
        root.join("observation.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_CONSTRUCTOR_NAV positive={positive} first={first} last={} source_bytes={} phases_used={} suffix_frames={suffix_frames:?}",m.bus.frames,stats.source_bytes,stats.phases_used);
}
