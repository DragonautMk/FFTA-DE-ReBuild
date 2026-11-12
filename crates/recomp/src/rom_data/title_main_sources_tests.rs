//! Conditional EUR main-title cursor positions and independent animation bank.
use super::*;
use gba_core::{Bus, Machine};

fn run_until_return(machine: &mut Machine) {
    for _ in 0..20_000 {
        machine.step();
        if machine.cpu.regs[15] == 0x0800_0300 { return; }
    }
    panic!("original title animation routine did not return");
}

fn original_cursor(machine: &mut Machine, language: u8, row: u8) {
    machine.cpu.regs = [0; 16];
    machine.cpu.regs[..3].copy_from_slice(&[0x0202_0000, 0x0202_0002, u32::from(row)]);
    machine.cpu.regs[13] = 0x0300_7c00; machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = 0x0813_ba10; machine.cpu.cpsr = 0xbf;
    machine.bus.write8(0x0200_1f6e, language);
    machine.bus.write16(0x0202_0000, 0x1234); machine.bus.write16(0x0202_0002, 0x5678);
    run_until_return(machine);
}

fn original_animator(machine: &mut Machine, sequence: u32) {
    machine.cpu.regs = [0; 16];
    machine.cpu.regs[..4].copy_from_slice(&[0x0202_1000, 0x08b8_ed28, 0x08b8_ecb4, sequence]);
    machine.cpu.regs[13] = 0x0300_7c00; machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = 0x0813_9ca0; machine.cpu.cpsr = 0xbf;
    machine.bus.write32(0x0300_7c00, 1);
    run_until_return(machine);
    machine.cpu.regs[0] = 0x0202_1000; machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = 0x0813_9ccc;
    run_until_return(machine);
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "exact EUR ROM, E155 predecessor and retained main title; Wine only"]
fn european_title_main_cursor_and_bank_publication_and_ordinary_pairs() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    const POSITION: u32 = 0x3adca0;
    let root = PathBuf::from(std::env::var_os("FFTA_TITLE_MAIN_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_TITLE_MAIN_PREVIOUS").unwrap());
    let state = fs::read(std::env::var_os("FFTA_TITLE_MAIN_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "fefd6a26720c4574d0ccb2b93efe1c6ffc2353248e931a64fb9dc10539dd1e32");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (155, 154));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision), (156, 155));
    assert_eq!(source.manifest.coverage, old.coverage);
    let prepared = preparation::prepare(&root.join("e156-resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("e156-resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    let mut ranges = Vec::<(u32,u32)>::new();
    for old_resource in &old.resources {
        let new = source.manifest.resources.iter().find(|r| r.id == old_resource.id).unwrap();
        let old_bytes = fs::read(previous.join(&old_resource.path)).unwrap();
        let new_bytes = fs::read(prepared.directory.join(&new.path)).unwrap();
        if old_resource.id != LAYERED_OBJ_ANIMATIONS_ID { assert_eq!(new, old_resource); assert_eq!(new_bytes, old_bytes); continue; }
        let prior = parse_layered_obj_animations(&old_bytes).unwrap();
        let mut current = parse_layered_obj_animations(&new_bytes).unwrap();
        let positions = current.title_main_positions.take().unwrap();
        for (lang, rows) in positions.iter().enumerate() { for (row, pair) in rows.iter().enumerate() { for (axis, value) in pair.iter().enumerate() {
            let at=POSITION as usize+lang*16+row*4+axis*2; assert_eq!(*value,u16::from_le_bytes(rom[at..at+2].try_into().unwrap()));
        }}}
        let bank = current.title_main_bank.take().unwrap();
        assert_eq!((bank.object_root,bank.sequence_root,bank.lists.len()),(0xb8ed28,0xb8ecb4,6));
        assert_eq!(bank.sequences.iter().map(|s|s.frames.len()).collect::<Vec<_>>(),[6]);
        assert_eq!(current, prior);
        let mut metadata = new.clone();
        for span in &new.sources { if !old_resource.sources.contains(span) { ranges.push((span.offset, span.offset+span.length)); } }
        metadata.sources.retain(|span| old_resource.sources.contains(span));
        metadata.sha256 = old_resource.sha256.clone(); assert_eq!(&metadata, old_resource);
    }
    ranges.sort_unstable();
    assert_eq!(ranges.len(),10); assert_eq!(ranges.iter().map(|(a,b)|b-a).sum::<u32>(),230);
    assert!(ranges.windows(2).all(|v|v[0].1<=v[1].0));
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let before = owner.layered_obj_animations_stats().unwrap();
    for &(lo,hi) in &ranges { managed.bus.rom[lo as usize..hi as usize].fill(0xcd); }
    for &(lo,hi) in &ranges { for at in lo..hi { assert_eq!(managed.bus.read8(0x0800_0000+at),rom[at as usize]); } }
    let after=owner.layered_obj_animations_stats().unwrap();
    assert_eq!(after.title_main_position_bytes-before.title_main_position_bytes,80);
    assert_eq!(after.fields.source_bytes-before.fields.source_bytes,150);
    let ptr=&mut managed as *mut Machine as *mut std::ffi::c_void;
    for &(lo,hi) in &ranges { for edge in [lo,hi] { for delta in [-3isize,-1,0,1] {
        let at=(edge as usize).wrapping_add_signed(delta);
        assert_eq!((gba_core::capi::RT_API.read16)(ptr,0x0a00_0000+at as u32),u32::from(u16::from_le_bytes(rom[at..at+2].try_into().unwrap())));
        assert_eq!((gba_core::capi::RT_API.read32)(ptr,0x0800_0000+at as u32),u32::from_le_bytes(rom[at..at+4].try_into().unwrap()));
    }}}
    for &(lo,hi) in &ranges { managed.bus.rom[lo as usize..hi as usize].copy_from_slice(&rom[lo as usize..hi as usize]); }
    for (language,row) in [(0u8,0u8),(0,3),(1,1),(2,1),(3,1),(4,0),(4,3),(0,4),(4,255),(5,0),(255,0)] {
        let mut raw=crate::make_machine(rom.clone(),None); let initial=raw.save_state().unwrap(); managed.load_state(&initial).unwrap();
        for &(lo,hi) in &ranges { managed.bus.rom[lo as usize..hi as usize].fill(0xcd); }
        original_cursor(&mut raw,language,row); original_cursor(&mut managed,language,row);
        if language > 4 { assert_eq!(raw.bus.read16(0x0202_0000),0x1234); assert_eq!(raw.bus.read16(0x0202_0002),0x5678); }
        for &(lo,hi) in &ranges { managed.bus.rom[lo as usize..hi as usize].copy_from_slice(&rom[lo as usize..hi as usize]); }
        assert_eq!(raw.save_state().unwrap(),managed.save_state().unwrap(),"original cursor language {language}, row {row}");
    }
    {
        let mut raw=crate::make_machine(rom.clone(),None); let initial=raw.save_state().unwrap(); managed.load_state(&initial).unwrap();
        for &(lo,hi) in &ranges { managed.bus.rom[lo as usize..hi as usize].fill(0xcd); }
        original_animator(&mut raw,0); original_animator(&mut managed,0);
        let seq_root=0xb8ecb4usize; let obj_root=0xb8ed28usize;
        let seq=seq_root+u32::from_le_bytes(rom[seq_root..seq_root+4].try_into().unwrap()) as usize;
        let index=u16::from_le_bytes(rom[seq+2..seq+4].try_into().unwrap()) as usize;
        let list=obj_root+u32::from_le_bytes(rom[obj_root+index*4..obj_root+index*4+4].try_into().unwrap()) as usize;
        assert_eq!(raw.cpu.regs[0],0x0800_0000+list as u32);
        for &(lo,hi) in &ranges { managed.bus.rom[lo as usize..hi as usize].copy_from_slice(&rom[lo as usize..hi as usize]); }
        assert_eq!(raw.save_state().unwrap(),managed.save_state().unwrap(),"original main title animator");
    }
    let (_library,table)=crate::load_native(&PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap())).unwrap();
    let mut results=Vec::new();
    for native in [false,true] {
        let mut raw=crate::make_machine(rom.clone(),None); let mut managed=crate::make_machine(rom.clone(),None);
        attach_from_directory(&mut managed,&prepared.directory).unwrap(); raw.load_state(&state).unwrap(); managed.load_state(&state).unwrap();
        let mut typed_cursor=0u64;let mut typed_bank=0u64;let mut raw_reads=0u64;let mut dispatches=0u64;
        for frame in 0..3 {
            for &(lo,hi) in &ranges { managed.bus.rom[lo as usize..hi as usize].fill(0xcd); }
            if native { let p=&mut raw as *mut Machine as *mut std::ffi::c_void; crate::run_frame_native(&mut raw,&table,p,crate::FRAME_STEP_GUARD); }
            else { raw.bus.frame_ready=false;for _ in 0..5_000_000 {if raw.bus.frame_ready {break;} raw.step();} }
            assert!(raw.bus.frame_ready);
            let capture=TraceCapture::start(TraceFilter{readers:vec![],ranges:ranges.clone()},50_000).unwrap();capture.frame(Some(managed.bus.frames));
            if native {let p=&mut managed as *mut Machine as *mut std::ffi::c_void;dispatches+=crate::run_frame_native(&mut managed,&table,p,crate::FRAME_STEP_GUARD).0;}
            else {managed.bus.frame_ready=false;for _ in 0..5_000_000 {if managed.bus.frame_ready {break;} managed.step();}}
            assert!(managed.bus.frame_ready);let report=capture.finish();assert!(!report.truncated);
            for event in &report.events {if event.key.service==Service::Raw {raw_reads+=event.requests;} else if event.key.service==Service::Typed {
                if (POSITION..POSITION+80).contains(&event.key.read.physical_offset) {typed_cursor+=event.requests;} else {typed_bank+=event.requests;}
            }}
            for &(lo,hi) in &ranges { managed.bus.rom[lo as usize..hi as usize].copy_from_slice(&rom[lo as usize..hi as usize]); }
            assert_eq!(raw.save_state().unwrap(),managed.save_state().unwrap(),"ordinary title frame {frame}, native={native}");
        }
        assert!(typed_cursor>0 && raw_reads==0);if native {assert!(dispatches>0);}
        results.push((typed_cursor,typed_bank,raw_reads,dispatches));
    }
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","revision":[156,155],"new_spans":10,"new_source_bytes":230,"e155_projection_preserved":true,"source_rom_poisoned":true,"original_cursor_cases":11,"original_animator_sequences":1,"ordinary_interpreter_and_native_dispatch_pairs":results,"full_state_clock_frames_each":3,"bank_ordinary_use":"pending","per_read_native_attribution":"pending","Linux":"pending","USA":"pending","native_windows":"pending","performance_measurements":false})).unwrap()).unwrap();
}
