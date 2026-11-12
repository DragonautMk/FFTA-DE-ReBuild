//! Bounded ordinary EUR lookup and conditional OBJ-writer observation.
use super::*;
use gba_core::Bus;

#[test]
#[ignore = "authenticated ordinary EUR battle checkpoint; Wine only"]
fn european_llednar_lookup_caller_and_writer_observation() {
    let root = PathBuf::from(std::env::var_os("FFTA_LLEDNAR_OBSERVER_ROOT").unwrap());
    let state = fs::read(std::env::var_os("FFTA_LLEDNAR_OBSERVER_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "31b837594b1ede7f2c0d521d3db939a4cc96a1880946bd631e9dc9c027c4f409");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let mut machine = crate::make_machine(rom, None);
    machine.load_state(&state).unwrap();
    let mut lookups = Vec::new(); let mut writes = Vec::new(); let mut draws = Vec::new();
    for frame in 0..3 {
        machine.bus.keys = 0x3ff; machine.bus.frame_ready = false;
        for _ in 0..5_000_000 {
            if machine.bus.frame_ready { break; }
            let pc = machine.cpu.regs[15];
            let regs = machine.cpu.regs;
            let write = pc == 0x0802_2f12 && regs[1] == 0x0839_5014;
            let old = if write { Some(machine.bus.read32(regs[0] + 0x28)) } else { None };
            let event = machine.step();
            if !matches!(event, gba_core::StepEvent::Instr(ref i) if i.addr == pc) { continue; }
            if pc == 0x080a_3076 && (0x0839_8df0..0x0839_8df4).contains(&regs[0]) {
                lookups.push(serde_json::json!({"frame":frame,"address":format!("{:08x}",regs[0]),"lr":format!("{:08x}",regs[14]),"value":machine.cpu.regs[0]}));
            }
            if write {
                let after = machine.bus.read32(regs[0] + 0x28);
                writes.push(serde_json::json!({"frame":frame,"object":format!("{:08x}",regs[0]),"lr":format!("{:08x}",regs[14]),"before":format!("{:08x}",old.unwrap()),"after":format!("{after:08x}")}));
                assert_eq!(after, 0x0839_5014);
            }
            if pc == 0x0800_1500 && regs[0] == 0x0839_5014 {
                draws.push(serde_json::json!({"frame":frame,"lr":format!("{:08x}",regs[14])}));
            }
            assert!(lookups.len() <= 1000 && writes.len() <= 1000 && draws.len() <= 1000);
        }
        assert!(machine.bus.frame_ready, "observer frame limit {frame}");
    }
    assert!(!lookups.is_empty(), "expected observed lookup was not reached");
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"three neutral interpreter frames from authenticated ordinary EUR Llednar checkpoint",
        "lookup_reads":lookups,"writer_hits":writes,"original_draw_entries":draws,
        "limitations":"bounded observer; absent PCs do not prove global absence; no dynamic object-to-draw association is inferred",
        "Linux":"pending","USA":"pending","native_windows":"pending","performance_measurements":false
    })).unwrap()).unwrap();
}
