//! Two lookup words observed in an ordinary EUR Llednar battle.
use super::*;
use gba_core::{Bus, Machine};

fn lookup_original(machine: &mut Machine, index: i32) {
    machine.cpu.regs[0] = index as u32;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = 0x080a_3070;
    machine.cpu.cpsr = 0xbf;
    for _ in 0..16 {
        machine.step();
        if machine.cpu.regs[15] == 0x0800_0300 { return; }
    }
    panic!("original actor lookup did not return");
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "exact EUR ROM, E152 predecessor and authenticated battle state; Wine only"]
fn european_llednar_actor_lookup_publication_and_ordinary_pairs() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    const START: usize = 0x398df0;
    const END: usize = START + 4;
    let root = PathBuf::from(std::env::var_os("FFTA_LLEDNAR_LOOKUP_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_LLEDNAR_LOOKUP_PREVIOUS").unwrap());
    let state = fs::read(std::env::var_os("FFTA_LLEDNAR_LOOKUP_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "31b837594b1ede7f2c0d521d3db939a4cc96a1880946bd631e9dc9c027c4f409");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (152, 151));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision), (153, 152));
    assert_eq!(source.manifest.coverage, old.coverage);
    let prepared = preparation::prepare(&root.join("e153-resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("e153-resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    for old_resource in &old.resources {
        let new = source.manifest.resources.iter().find(|r| r.id == old_resource.id).unwrap();
        let old_bytes = fs::read(previous.join(&old_resource.path)).unwrap();
        let new_bytes = fs::read(prepared.directory.join(&new.path)).unwrap();
        if old_resource.id != OBJ_STATIC_LISTS_ID { assert_eq!(new, old_resource); assert_eq!(new_bytes, old_bytes); continue; }
        let prior = parse_obj_lists(&old_bytes).unwrap();
        let mut current = parse_obj_lists(&new_bytes).unwrap();
        let words = current.actor_lookup.take().unwrap();
        assert_eq!(words, [u16::from_le_bytes(rom[START..START+2].try_into().unwrap()), u16::from_le_bytes(rom[START+2..END].try_into().unwrap())]);
        assert_eq!(current, prior);
        let mut metadata = new.clone();
        metadata.sources.retain(|s| s.role != "actor-lookup");
        metadata.sha256 = old_resource.sha256.clone();
        assert_eq!(&metadata, old_resource);
    }
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let before = owner.obj_list_stats().unwrap();
    managed.bus.rom[START..END].fill(0xcd);
    for (i, &expected) in rom[START..END].iter().enumerate() { assert_eq!(managed.bus.read8(0x0800_0000 + (START + i) as u32), expected); }
    assert_eq!(owner.obj_list_stats().unwrap().actor_lookup_bytes - before.actor_lookup_bytes, 4);
    let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for edge in [START, END] { for delta in [-3isize, -1, 0, 1] {
        let at = edge.wrapping_add_signed(delta);
        assert_eq!((gba_core::capi::RT_API.read16)(ptr, 0x0a00_0000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
        assert_eq!((gba_core::capi::RT_API.read32)(ptr, 0x0800_0000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
    }}
    managed.bus.rom[START..END].copy_from_slice(&rom[START..END]);
    for index in [-1i32, 0, 1, 2] {
        let mut raw = crate::make_machine(rom.clone(), None);
        let initial = raw.save_state().unwrap(); managed.load_state(&initial).unwrap();
        managed.bus.rom[START..END].fill(0xcd);
        lookup_original(&mut raw, index); lookup_original(&mut managed, index);
        let at = START.wrapping_add_signed(index as isize * 2);
        assert_eq!(raw.cpu.regs[0], u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
        managed.bus.rom[START..END].copy_from_slice(&rom[START..END]);
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "lookup index={index}");
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
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","revision":[153,152],"new_source_bytes":4,"e152_projection_preserved":true,"source_rom_poisoned":true,"original_lookup_cases":4,"ordinary_interpreter_and_native_dispatch_pairs":results,"full_state_clock_frames_each":3,"global_index_domain":"pending","per_read_native_attribution":"pending","Linux":"pending","USA":"pending","native_windows":"pending","performance_measurements":false})).unwrap()).unwrap();
}
