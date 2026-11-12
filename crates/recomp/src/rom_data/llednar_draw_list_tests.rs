//! One OBJ list observed in an ordinary EUR Llednar battle.
use super::*;
use gba_core::{Bus, Machine};

fn draw_original(machine: &mut Machine) {
    const RETURN: u32 = 0x0800_0300;
    const STACK: u32 = 0x0300_7c00;
    const BUFFER: u32 = 0x0203_0010;
    machine.cpu.regs[..4].copy_from_slice(&[0x0839_5014, 100, 70, 0]);
    machine.cpu.regs[13] = STACK; machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = 0x0800_1500; machine.cpu.cpsr = 0xbf;
    for (i, value) in [7u32, 2, 3, BUFFER, 128, 0].into_iter().enumerate() { machine.bus.write32(STACK + i as u32 * 4, value); }
    for offset in 0..128 * 8 { machine.bus.write8(BUFFER + offset, 0x5a); }
    for _ in 0..20_000 {
        machine.step();
        if machine.cpu.regs[15] == RETURN { assert_eq!(machine.cpu.regs[13], STACK); return; }
    }
    panic!("original OBJ draw did not return");
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "exact EUR ROM, E151 predecessor and authenticated battle state; Wine only"]
fn european_llednar_draw_list_publication_and_ordinary_pairs() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    const START: usize = 0x395014;
    const END: usize = START + 8;
    let root = PathBuf::from(std::env::var_os("FFTA_LLEDNAR_LIST_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_LLEDNAR_LIST_PREVIOUS").unwrap());
    let state = fs::read(std::env::var_os("FFTA_LLEDNAR_LIST_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "31b837594b1ede7f2c0d521d3db939a4cc96a1880946bd631e9dc9c027c4f409");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (151, 150));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision), (152, 151));
    assert_eq!(source.manifest.coverage, old.coverage);
    let prepared = preparation::prepare(&root.join("e152-resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("e152-resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    for old_resource in &old.resources {
        let new = source.manifest.resources.iter().find(|r| r.id == old_resource.id).unwrap();
        let old_bytes = fs::read(previous.join(&old_resource.path)).unwrap();
        let new_bytes = fs::read(prepared.directory.join(&new.path)).unwrap();
        if old_resource.id != OBJ_STATIC_LISTS_ID { assert_eq!(new, old_resource); assert_eq!(new_bytes, old_bytes); continue; }
        let prior = parse_obj_lists(&old_bytes).unwrap();
        let mut current = parse_obj_lists(&new_bytes).unwrap();
        let selected = current.lists.iter().find(|l| l.source_offset == START as u32).unwrap();
        assert_eq!(selected.objects.len(), 1);
        assert_eq!(selected.source_bytes().unwrap(), rom[START..END]);
        current.lists.retain(|l| l.source_offset != START as u32);
        assert_eq!(current, prior);
        let mut metadata = new.clone();
        metadata.sources.retain(|s| s.offset != START as u32);
        metadata.sha256 = old_resource.sha256.clone();
        assert_eq!(&metadata, old_resource);
    }
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let before = owner.obj_list_stats().unwrap();
    managed.bus.rom[START..END].fill(0xcd);
    for (i, &expected) in rom[START..END].iter().enumerate() { assert_eq!(managed.bus.read8(0x0800_0000 + (START + i) as u32), expected); }
    assert_eq!(owner.obj_list_stats().unwrap().source_bytes - before.source_bytes, 8);
    let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for edge in [START, END] { for delta in [-3isize, -1, 0, 1] {
        let at = edge.wrapping_add_signed(delta);
        assert_eq!((gba_core::capi::RT_API.read16)(ptr, 0x0a00_0000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap())));
        assert_eq!((gba_core::capi::RT_API.read32)(ptr, 0x0800_0000 + at as u32), u32::from_le_bytes(rom[at..at + 4].try_into().unwrap()));
    }}
    managed.bus.rom[START..END].copy_from_slice(&rom[START..END]);
    let mut raw = crate::make_machine(rom.clone(), None);
    let initial = raw.save_state().unwrap(); managed.load_state(&initial).unwrap();
    managed.bus.rom[START..END].fill(0xcd);
    draw_original(&mut raw); draw_original(&mut managed);
    managed.bus.rom[START..END].copy_from_slice(&rom[START..END]);
    assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
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
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","revision":[152,151],"new_source_bytes":8,"e151_projection_preserved":true,"source_rom_poisoned":true,"original_draw_case":1,"ordinary_interpreter_and_native_dispatch_pairs":results,"full_state_clock_frames_each":3,"producer":"pending","per_read_native_attribution":"pending","Linux":"pending","USA":"pending","native_windows":"pending","performance_measurements":false})).unwrap()).unwrap();
}
