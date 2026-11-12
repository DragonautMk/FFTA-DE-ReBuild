//! Ordinary EUR title-language publication from a retained title checkpoint.
use super::*;
use gba_core::Bus;

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
