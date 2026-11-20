//! Locate the dispatch that reads an observed EUR Red Spring source.
use super::*;
use gba_core::rom_data::read_audit::{Reader, TriggerCapture, TriggerCondition};
use std::cell::Cell;

#[test]
#[ignore = "private EUR Red Spring checkpoint, input and native cache; Wine only"]
fn european_red_spring_lz_header_dispatch() {
    crate::configure_headless_widescreen();
    let path = |key| PathBuf::from(std::env::var_os(key).expect(key));
    let rom = fs::read(path("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let input = crate::InputScript::load(path("FFTA_STATE_TEST_INPUT").to_str().unwrap()).unwrap();
    let (_library, table) = crate::load_native(&path("FFTA_STATE_TEST_NATIVE")).unwrap();
    let mut machine = crate::make_machine(rom, None);
    attach_from_directory(&mut machine, &path("FFTA_STATE_TEST_DATASET")).unwrap();
    crate::save_state::load(&mut machine, &path("FFTA_STATE_TEST_STATE")).unwrap();
    crate::arm_audio_hle(&mut machine, None);
    machine.bus.tap_channels = true;
    let first = machine.bus.frames;
    assert_eq!(first, 7657);
    let offset = |key: &str, default| std::env::var(key).ok().map_or(default, |value| {
        u32::from_str_radix(value.trim_start_matches("0x"), 16).expect("hexadecimal source offset")
    });
    let start = offset("FFTA_REDSPRING_TRACE_START", 0xb0e2b8);
    let end = offset("FFTA_REDSPRING_TRACE_END", 0xb0e2bc);
    assert!(start < end && end <= 0x0100_0000);
    let trigger = TriggerCapture::start(TriggerCondition::Read {
        ranges: vec![(start, end)], readers: vec![Reader::Bus],
    }).unwrap();
    let before = Cell::new(None);
    let selected = Cell::new(None);
    for relative in 0..600 {
        machine.bus.keys = input.keys_at(relative);
        input.apply_mouse(&mut machine.bus, relative).unwrap();
        trigger.frame(Some(machine.bus.frames));
        let pointer = &mut machine as *mut gba_core::Machine as *mut std::ffi::c_void;
        crate::run_frame_native_traced(&mut machine, &table, pointer, crate::FRAME_STEP_GUARD,
            |m, kind| {
                if trigger.hit().is_none() {
                    before.set(Some((m.cpu.regs[15], m.cpu.regs[0], m.cpu.regs[1], m.cpu.regs[14], kind)));
                }
            },
            |kind, pc, _| {
                if selected.get().is_none() && trigger.hit().is_some() {
                    let state = before.get().expect("dispatch entry before read");
                    assert_eq!((state.0, state.4), (pc, kind));
                    selected.set(Some(state));
                }
            });
        assert!(machine.bus.frame_ready);
        if let (Some(hit), Some((pc, r0, r1, lr, kind))) = (trigger.hit(), selected.get()) {
            let result = serde_json::json!({
                "status":"PASS", "scope":"one observed EUR Red Spring source read; dispatch entry only",
                "first_guest_frame":first, "frame_after_hit":machine.bus.frames, "relative_frame":relative,
                "header_offset":start, "trigger":hit,
                "dispatch":{"entry_pc":pc, "kind":format!("{kind:?}"), "r0":r0, "r1":r1, "lr":lr},
                "limits":"A dispatch may contain multiple instructions or nested HLE/DMA; validate original instruction/dataflow before attributing the read.",
                "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
            });
            fs::write(path("FFTA_REDSPRING_LZ_RESULT"), serde_json::to_vec_pretty(&result).unwrap()).unwrap();
            return;
        }
        machine.bus.audio_buf.clear(); machine.bus.psg_tap.clear(); machine.bus.hle_tap.clear();
        machine.bus.fifo_tap.iter_mut().for_each(Vec::clear);
    }
    panic!("recorded Red Spring action did not read the selected source");
}
