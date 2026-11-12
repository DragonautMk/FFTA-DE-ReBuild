//! EUR conditional draw sequence observed in Llednar dialogue.
use super::*;
use gba_core::{Bus, Machine};

fn original_prefix(machine: &mut Machine, index: i8) {
    machine.cpu.regs = [0; 16];
    machine.cpu.regs[2] = 0x0202_0000;
    machine.cpu.regs[15] = 0x0801_3142;
    machine.cpu.cpsr = 0xbf;
    machine.bus.write8(0x0202_0038, index as u8);
    for _ in 0..24 {
        if machine.cpu.regs[15] == 0x0801_315a { return; }
        machine.step();
    }
    panic!("original draw-sequence prefix did not reach its continuation");
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "exact EUR ROM, E153 predecessor and authenticated battle state; Wine only"]
fn european_dialogue_draw_sequence_publication_and_ordinary_pairs() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    const START: usize = 0x370ffc;
    const END: usize = START + 32;
    let root = PathBuf::from(std::env::var_os("FFTA_DIALOGUE_SEQUENCE_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_DIALOGUE_SEQUENCE_PREVIOUS").unwrap());
    let state = fs::read(std::env::var_os("FFTA_DIALOGUE_SEQUENCE_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "d7002f4806922dea8430a1962fa9e58887e83662dc765942816e3a1c7dc832ad");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (153, 152));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision), (154, 153));
    assert_eq!(source.manifest.coverage, old.coverage);
    let prepared = preparation::prepare(&root.join("e154-resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("e154-resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    for old_resource in &old.resources {
        let new = source.manifest.resources.iter().find(|r| r.id == old_resource.id).unwrap();
        let old_bytes = fs::read(previous.join(&old_resource.path)).unwrap();
        let new_bytes = fs::read(prepared.directory.join(&new.path)).unwrap();
        if old_resource.id != OBSERVED_TASK_PARAMETERS_ID {
            assert_eq!(new, old_resource);
            assert_eq!(new_bytes, old_bytes);
            continue;
        }
        let prior = parse_observed_task_parameters(&old_bytes).unwrap();
        let mut current = parse_observed_task_parameters(&new_bytes).unwrap();
        let sequence = current.battle_draw_sequence.take().unwrap();
        assert_eq!(sequence.as_slice(), &rom[START..END]);
        assert_eq!(current, prior);
        let mut metadata = new.clone();
        metadata.sources.retain(|s| s.role != "battle-draw-sequence");
        metadata.sha256 = old_resource.sha256.clone();
        assert_eq!(&metadata, old_resource);
    }
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let before = owner.observed_task_parameters_stats().unwrap();
    managed.bus.rom[START..END].fill(0xcd);
    for (i, &expected) in rom[START..END].iter().enumerate() { assert_eq!(managed.bus.read8(0x0800_0000 + (START + i) as u32), expected); }
    assert_eq!(owner.observed_task_parameters_stats().unwrap().battle_draw_sequence_bytes - before.battle_draw_sequence_bytes, 32);
    let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for edge in [START, END] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.wrapping_add_signed(delta);
            assert_eq!((gba_core::capi::RT_API.read16)(ptr, 0x0a00_0000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
            assert_eq!((gba_core::capi::RT_API.read32)(ptr, 0x0800_0000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
        }
    }
    managed.bus.rom[START..END].copy_from_slice(&rom[START..END]);
    for index in [-1i8, 0, 12, 31, 32] {
        let mut raw = crate::make_machine(rom.clone(), None);
        let initial = raw.save_state().unwrap(); managed.load_state(&initial).unwrap();
        managed.bus.rom[START..END].fill(0xcd);
        original_prefix(&mut raw, index); original_prefix(&mut managed, index);
        let at = START.wrapping_add_signed(index as isize);
        assert_eq!(raw.cpu.regs[2], u32::from(rom[at].wrapping_add(10)));
        assert_eq!(raw.bus.read8(0x0202_0038), (index as u8).wrapping_add(1));
        managed.bus.rom[START..END].copy_from_slice(&rom[START..END]);
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "draw sequence prefix index={index}");
    }
    let (_library, table) = crate::load_native(&PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap())).unwrap();
    let mut results = Vec::new();
    for native in [false, true] {
        let mut raw = crate::make_machine(rom.clone(), None);
        let mut managed = crate::make_machine(rom.clone(), None);
        attach_from_directory(&mut managed, &prepared.directory).unwrap();
        raw.load_state(&state).unwrap(); managed.load_state(&state).unwrap();
        let mut typed=0u64; let mut raw_reads=0u64; let mut dispatches=0u64;
        for frame in 0..3 {
            managed.bus.rom[START..END].fill(0xcd);
            if native { let p=&mut raw as *mut Machine as *mut std::ffi::c_void; crate::run_frame_native(&mut raw,&table,p,crate::FRAME_STEP_GUARD); }
            else { raw.bus.frame_ready=false; for _ in 0..5_000_000 { if raw.bus.frame_ready { break; } raw.step(); } }
            assert!(raw.bus.frame_ready);
            let capture=TraceCapture::start(TraceFilter { readers:vec![], ranges:vec![(START as u32,END as u32)] },10000).unwrap();
            capture.frame(Some(managed.bus.frames));
            if native { let p=&mut managed as *mut Machine as *mut std::ffi::c_void; dispatches+=crate::run_frame_native(&mut managed,&table,p,crate::FRAME_STEP_GUARD).0; }
            else { managed.bus.frame_ready=false; for _ in 0..5_000_000 { if managed.bus.frame_ready { break; } managed.step(); } }
            assert!(managed.bus.frame_ready);
            let report=capture.finish(); assert!(!report.truncated);
            typed+=report.events.iter().filter(|e| e.key.service==Service::Typed).map(|e|e.requests).sum::<u64>();
            raw_reads+=report.events.iter().filter(|e| e.key.service==Service::Raw).map(|e|e.requests).sum::<u64>();
            managed.bus.rom[START..END].copy_from_slice(&rom[START..END]);
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "ordinary frame {frame}, native={native}");
        }
        assert!(typed>0 && raw_reads==0); if native { assert!(dispatches>0); }
        results.push((typed,raw_reads,dispatches));
    }
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","revision":[154,153],"new_source_bytes":32,"e153_projection_preserved":true,"source_rom_poisoned":true,"original_prefix_cases":5,"ordinary_interpreter_and_native_dispatch_pairs":results,"full_state_clock_frames_each":3,"per_read_native_attribution":"pending","Linux":"pending","USA":"pending","native_windows":"pending","performance_measurements":false})).unwrap()).unwrap();
}
