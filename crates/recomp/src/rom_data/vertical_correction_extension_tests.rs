//! EUR task correction indices observed during an ordinary Llednar battle.
use super::*;
use gba_core::{Bus, Machine};

fn original_prefix(machine: &mut Machine, byte: u8) {
    machine.cpu.regs = [0; 16];
    machine.cpu.regs[2] = 0x8000_0010;
    machine.cpu.regs[6] = 0x0202_0000;
    machine.cpu.regs[15] = 0x0802_64ea;
    machine.cpu.cpsr = 0xbf;
    machine.bus.write8(0x0202_0007, byte);
    for _ in 0..16 {
        if machine.cpu.regs[15] == 0x0802_64fa { return; }
        machine.step();
    }
    panic!("original correction prefix did not reach its continuation");
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "exact EUR ROM, E150 predecessor and authenticated battle state; Wine only"]
fn european_vertical_correction_extension_publication_and_ordinary_pairs() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    const START: usize = 0x394e1c;
    const END: usize = START + 4;
    let root = PathBuf::from(std::env::var_os("FFTA_VERTICAL_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_VERTICAL_PREVIOUS").unwrap());
    let state = fs::read(std::env::var_os("FFTA_VERTICAL_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "31b837594b1ede7f2c0d521d3db939a4cc96a1880946bd631e9dc9c027c4f409");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (150, 149));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision), (151, 150));
    assert_eq!(source.manifest.coverage, old.coverage);
    let prepared = preparation::prepare(&root.join("e151-resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("e151-resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
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
        assert_eq!(current.vertical_corrections.entries.len(), 6);
        for (i, row) in current.vertical_corrections.entries[4..].iter().enumerate() {
            assert_eq!(usize::from(row.index), i + 8);
            assert_eq!(row.correction, i16::from_le_bytes(rom[START+i*2..START+i*2+2].try_into().unwrap()));
        }
        current.vertical_corrections.entries.truncate(4);
        assert_eq!(current, prior);
        let mut metadata = new.clone();
        let extension = metadata.sources.iter_mut().find(|s| s.role == "vertical-correction").unwrap();
        *extension = old_resource.sources.iter().find(|s| s.role == "vertical-correction").unwrap().clone();
        metadata.sha256 = old_resource.sha256.clone();
        assert_eq!(&metadata, old_resource);
    }
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let before = owner.observed_task_parameters_stats().unwrap();
    // Battle bits must not count as correction indices, including newly added bits.
    assert_eq!(managed.bus.read8(0x0837_101c), rom[0x37101c]);
    assert_eq!(owner.observed_task_parameters_stats().unwrap().corrections_used, before.corrections_used);
    managed.bus.rom[START..END].fill(0xcd);
    for (i, &expected) in rom[START..END].iter().enumerate() { assert_eq!(managed.bus.read8(0x0800_0000 + (START + i) as u32), expected); }
    assert_eq!(owner.observed_task_parameters_stats().unwrap().corrections_used - before.corrections_used, 2);
    let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for edge in [START, END] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.wrapping_add_signed(delta);
            assert_eq!((gba_core::capi::RT_API.read16)(ptr, 0x0a00_0000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
            assert_eq!((gba_core::capi::RT_API.read32)(ptr, 0x0800_0000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
        }
    }
    managed.bus.rom[START..END].copy_from_slice(&rom[START..END]);
    for byte in 16..20 {
        let mut raw = crate::make_machine(rom.clone(), None);
        let initial = raw.save_state().unwrap();
        managed.load_state(&initial).unwrap();
        managed.bus.rom[START..END].fill(0xcd);
        original_prefix(&mut raw, byte);
        original_prefix(&mut managed, byte);
        assert_eq!(raw.cpu.regs[2], 0x8000_0010u32.wrapping_sub(i32::from(i16::from_le_bytes(rom[START + usize::from((byte-16)/2)*2..START + usize::from((byte-16)/2)*2+2].try_into().unwrap())) as u32));
        managed.bus.rom[START..END].copy_from_slice(&rom[START..END]);
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "correction prefix byte={byte}");
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
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","revision":[151,150],"new_source_bytes":4,"e150_projection_preserved":true,"source_rom_poisoned":true,"original_prefix_cases":4,"ordinary_interpreter_and_native_dispatch_pairs":results,"full_state_clock_frames_each":3,"per_read_native_attribution":"pending","Linux":"pending","USA":"pending","native_windows":"pending","performance_measurements":false})).unwrap()).unwrap();
}
