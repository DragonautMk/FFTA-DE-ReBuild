//! Bounded pre-instruction evidence for two White Flowers sampled reads.
use super::*;
use gba_core::rom_data::read_audit::SiteCapture;
use std::collections::BTreeSet;

fn clear_audio(machine: &mut gba_core::Machine) {
    machine.bus.audio_buf.clear();
    machine.bus.psg_tap.clear();
    machine.bus.fifo_tap[0].clear();
    machine.bus.fifo_tap[1].clear();
    machine.bus.hle_tap.clear();
}

#[test]
#[ignore = "exact EUR ROM and retained White Flowers scene; Wine only, Linux/USA pending"]
fn european_white_flowers_isolated_read_operands() {
    crate::configure_headless_widescreen();
    let path = |key| PathBuf::from(std::env::var_os(key).expect(key));
    let report = path("FFTA_WHITE_OPERAND_SITES_REPORT");
    assert!(!report.exists(), "preserve earlier operand reports");
    let rom = fs::read(path("FFTA_STATE_TEST_ROM")).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    assert_eq!(u16::from_le_bytes(rom[0x0cf406..0x0cf408].try_into().unwrap()), 0x7800);
    assert_eq!(u16::from_le_bytes(rom[0x02edcc..0x02edce].try_into().unwrap()), 0x8809);
    let dataset = path("FFTA_STATE_TEST_DATASET");
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision, manifest.decoder_revision), (290, 289));
    let (_library, table) = crate::load_native(&path("FFTA_STATE_TEST_NATIVE")).unwrap();
    let input = crate::InputScript::load(path("FFTA_STATE_TEST_INPUT").to_str().unwrap()).unwrap();
    let mut machine = crate::make_machine(rom, None);
    attach_from_directory(&mut machine, &dataset).unwrap();
    crate::save_state::load(&mut machine, &path("FFTA_STATE_TEST_STATE")).unwrap();
    crate::arm_audio_hle(&mut machine, None);
    machine.bus.tap_channels = true;
    clear_audio(&mut machine);
    let first = machine.bus.frames;
    let sites = SiteCapture::start([0x3c9460, 0x6e89a8]);
    let mut operands = BTreeSet::new();
    for relative in 0..600 {
        machine.bus.keys = input.keys_at(relative);
        input.apply_mouse(&mut machine.bus, relative).unwrap();
        let pointer = &mut machine as *mut gba_core::Machine as *mut std::ffi::c_void;
        crate::run_frame_native_traced(&mut machine, &table, pointer, crate::FRAME_STEP_GUARD,
            |m, kind| {
                if kind == crate::NativeDispatch::Native || !m.cpu.thumb() { return; }
                let pc = m.cpu.regs[15];
                let operand = match pc {
                    0x080c_f406 => m.cpu.regs[0],
                    0x0802_edcc => m.cpu.regs[1],
                    _ => return,
                };
                let physical = operand & 0x01ff_ffff;
                if !(0x08..=0x0d).contains(&(operand >> 24))
                    || !matches!((pc, physical), (0x080c_f406, 0x3c9460) | (0x0802_edcc, 0x6e89aa)) { return; }
                operands.insert((pc, operand, m.cpu.regs[1], m.cpu.regs[5], m.cpu.regs[8], m.cpu.regs[14], m.bus.frames, kind as u8));
            }, |_, _, _| {});
        assert_eq!(machine.bus.frames, first + relative + 1);
        clear_audio(&mut machine);
    }
    let sites = sites.finish();
    assert!(sites.iter().any(|row| row.read.physical_offset == 0x3c9460));
    assert!(sites.iter().any(|row| row.read.physical_offset == 0x6e89aa));
    let byte_matches = operands.iter().filter(|row| row.0 == 0x080c_f406).count();
    let halfword_matches = operands.iter().filter(|row| row.0 == 0x0802_edcc).count();
    let rows: Vec<_> = operands.iter().map(|&(pc, operand, r1, r5, r8, lr, frame, kind)| serde_json::json!({
        "pc":pc, "operand":operand, "r1":r1, "r5":r5, "r8":r8, "lr":lr, "frame":frame, "dispatch_kind":kind
    })).collect();
    fs::write(&report, serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[290,289], "frames":600, "first_guest_frame":first,
        "last_guest_frame":machine.bus.frames, "byte_operand_matches":byte_matches,
        "halfword_operand_matches":halfword_matches, "operands":rows, "site_reads":sites,
        "scope":"one EUR continuation; pre-step operands at two filtered interpreter sites",
        "limits":"Matching pre-step operands identify these loads; callers, full selector domains and asset extents remain open. Missing matches require further attribution.",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
    eprintln!("ROM_DATA_WHITE_FLOWERS_OPERANDS byte_matches={byte_matches} halfword_matches={halfword_matches} PASS");
}
