//! Conditional EUR title-opening coordinate readers.
use super::*;
use gba_core::{Bus, Machine};

fn run_original(machine: &mut Machine, state: u8, index: u16) {
    machine.cpu.regs = [0; 16];
    machine.cpu.regs[0] = 0x0202_0000;
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = 0x0813_a1dc;
    machine.cpu.cpsr = 0xbf;
    machine.bus.write8(0x0202_0000, state);
    machine.bus.write16(0x0202_0002, 2);
    machine.bus.write16(0x0202_0004, index);
    for _ in 0..30_000 {
        machine.step();
        if machine.cpu.regs[15] == 0x0800_0300 { return; }
    }
    panic!("original title opening routine did not return");
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "EUR ROM, E157 predecessor and opening checkpoint; Wine only"]
fn european_title_opening_coordinates_publication_and_original_readers() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    let root = PathBuf::from(std::env::var_os("FFTA_OPENING_COORD_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_OPENING_COORD_PREVIOUS").unwrap());
    let state = fs::read(std::env::var_os("FFTA_OPENING_COORD_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "0a439a15a0de6ccda64a8010d50da37b0ba5a4435b683adf45fb7cf68dec56e3");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (157, 156));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision), (158, 157));
    assert_eq!(source.manifest.coverage, old.coverage);
    let prepared = preparation::prepare(&root.join("e158-resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("e158-resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    let mut ranges = vec![(0x3adeb0u32, 0x3adefcu32)];
    ranges.extend((0..19).map(|i| (0x3adefe + 4*i, 0x3adf00 + 4*i)));
    for old_resource in &old.resources {
        let new = source.manifest.resources.iter().find(|r| r.id == old_resource.id).unwrap();
        let old_bytes = fs::read(previous.join(&old_resource.path)).unwrap();
        let new_bytes = fs::read(prepared.directory.join(&new.path)).unwrap();
        if old_resource.id != LAYERED_OBJ_ANIMATIONS_ID { assert_eq!(new, old_resource); assert_eq!(new_bytes, old_bytes); continue; }
        let prior = parse_layered_obj_animations(&old_bytes).unwrap();
        let mut current = parse_layered_obj_animations(&new_bytes).unwrap();
        let selected = current.title_opening_coordinates.take().unwrap();
        for i in 0..19 { for axis in 0..2 {
            let at=0x3adeb0+4*i+2*axis; assert_eq!(selected.positions[i][axis],u16::from_le_bytes(rom[at..at+2].try_into().unwrap()));
        }
            let at=0x3adefe+4*i; assert_eq!(selected.sparse[i],u16::from_le_bytes(rom[at..at+2].try_into().unwrap()));
        }
        assert_eq!(current, prior);
        let mut metadata=new.clone();
        let added:Vec<_>=metadata.sources.iter().filter(|s|!old_resource.sources.contains(s)).cloned().collect();
        assert_eq!(added.len(),20); assert_eq!(added.iter().map(|s|s.length).sum::<u32>(),114);
        metadata.sources.retain(|s|old_resource.sources.contains(s)); metadata.sha256=old_resource.sha256.clone(); assert_eq!(&metadata,old_resource);
    }
    let mut managed=crate::make_machine(rom.clone(),None); attach_from_directory(&mut managed,&prepared.directory).unwrap();
    let owner=managed.bus.rom_resources().unwrap().clone(); let before=owner.layered_obj_animations_stats().unwrap();
    for &(lo,hi) in &ranges { managed.bus.rom[lo as usize..hi as usize].fill(0xcd); }
    for &(lo,hi) in &ranges { for at in lo..hi { assert_eq!(managed.bus.read8(0x0800_0000+at),rom[at as usize]); } }
    assert_eq!(owner.layered_obj_animations_stats().unwrap().title_opening_coordinate_bytes-before.title_opening_coordinate_bytes,114);
    let ptr=&mut managed as *mut Machine as *mut std::ffi::c_void;
    for &(lo,hi) in &ranges { for edge in [lo,hi] { for delta in [-3isize,-1,0,1] {
        let at=(edge as usize).wrapping_add_signed(delta);
        assert_eq!((gba_core::capi::RT_API.read16)(ptr,0x0a00_0000+at as u32),u32::from(u16::from_le_bytes(rom[at..at+2].try_into().unwrap())));
        assert_eq!((gba_core::capi::RT_API.read32)(ptr,0x0800_0000+at as u32),u32::from_le_bytes(rom[at..at+4].try_into().unwrap()));
    }}}
    for &(lo,hi) in &ranges { managed.bus.rom[lo as usize..hi as usize].copy_from_slice(&rom[lo as usize..hi as usize]); }
    for (state,index) in [(1u8,0u16),(1,18),(1,19),(1,255),(10,0)] {
        let mut raw=crate::make_machine(rom.clone(),None); let initial=raw.save_state().unwrap(); managed.load_state(&initial).unwrap();
        for &(lo,hi) in &ranges { managed.bus.rom[lo as usize..hi as usize].fill(0xcd); }
        run_original(&mut raw,state,index); run_original(&mut managed,state,index);
        for &(lo,hi) in &ranges { managed.bus.rom[lo as usize..hi as usize].copy_from_slice(&rom[lo as usize..hi as usize]); }
        assert_eq!(raw.save_state().unwrap(),managed.save_state().unwrap(),"state {state}, initial index {index}");
    }
    let mut raw=crate::make_machine(rom.clone(),None); let mut managed=crate::make_machine(rom.clone(),None);
    attach_from_directory(&mut managed,&prepared.directory).unwrap(); raw.load_state(&state).unwrap(); managed.load_state(&state).unwrap();
    let mut typed=0u64; let mut fallback=0u64;
    for frame in 0..3 {
        for &(lo,hi) in &ranges {managed.bus.rom[lo as usize..hi as usize].fill(0xcd);}
        raw.bus.frame_ready=false; for _ in 0..5_000_000 {if raw.bus.frame_ready {break;}raw.step();} assert!(raw.bus.frame_ready);
        let trace=TraceCapture::start(TraceFilter{readers:vec![],ranges:ranges.clone()},10_000).unwrap();trace.frame(Some(managed.bus.frames));
        managed.bus.frame_ready=false;for _ in 0..5_000_000 {if managed.bus.frame_ready {break;}managed.step();}assert!(managed.bus.frame_ready);
        let report=trace.finish();assert!(!report.truncated);
        typed+=report.events.iter().filter(|e|e.key.service==Service::Typed).map(|e|e.requests).sum::<u64>();
        fallback+=report.events.iter().filter(|e|e.key.service==Service::Raw).map(|e|e.requests).sum::<u64>();
        for &(lo,hi) in &ranges {managed.bus.rom[lo as usize..hi as usize].copy_from_slice(&rom[lo as usize..hi as usize]);}
        assert_eq!(raw.save_state().unwrap(),managed.save_state().unwrap(),"ordinary opening frame {frame}");
    }
    assert!(typed>0);assert_eq!(fallback,0);
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","revision":[158,157],"e157_projection_preserved":true,"new_spans":20,"new_bytes":114,"poisoned_source":true,"original_reader_cases":5,"ordinary_interpreter_frames":3,"ordinary_typed_requests":typed,"ordinary_raw_requests":fallback,"second_branch_ordinary":"pending","native_dispatch":"pending","Linux":"pending","USA":"pending","native_windows":"pending","performance_measurements":false})).unwrap()).unwrap();
}
