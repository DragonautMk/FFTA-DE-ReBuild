//! EUR title-opening pointer directory and its conditional OBJ lists.
use super::*;
use gba_core::{Bus, Machine};

fn run_until_return(machine: &mut Machine) {
    for _ in 0..30_000 {
        machine.step();
        if machine.cpu.regs[15] == 0x0800_0300 { return; }
    }
    panic!("original title routine did not return");
}

fn initialize_original(machine: &mut Machine) {
    machine.cpu.regs = [0; 16];
    machine.cpu.regs[0] = 0x0202_0000;
    machine.cpu.regs[13] = 0x0300_7c00;
    machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = 0x0813_a1dc;
    machine.cpu.cpsr = 0xbf;
    machine.bus.write32(0x0200_f3b0, 0x0201_0000);
    machine.bus.write8(0x0202_0000, 0);
    run_until_return(machine);
}

fn draw_original(machine: &mut Machine, source: u32) {
    const STACK: u32 = 0x0300_7c00;
    const BUFFER: u32 = 0x0203_0010;
    machine.cpu.regs = [0; 16];
    machine.cpu.regs[..4].copy_from_slice(&[0x0800_0000 + source, 100, 70, 0]);
    machine.cpu.regs[13] = STACK; machine.cpu.regs[14] = 0x0800_0301;
    machine.cpu.regs[15] = 0x0800_1500; machine.cpu.cpsr = 0xbf;
    for (i, value) in [7u32, 2, 3, BUFFER, 128, 0].into_iter().enumerate() {
        machine.bus.write32(STACK + i as u32 * 4, value);
    }
    for offset in 0..128 * 8 { machine.bus.write8(BUFFER + offset, 0x5a); }
    run_until_return(machine);
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "exact EUR ROM and E156 predecessor; Wine only"]
fn european_title_opening_obj_publication_and_original_readers() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    let root = PathBuf::from(std::env::var_os("FFTA_TITLE_OBJ_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_TITLE_OBJ_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (156, 155));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision), (157, 156));
    assert_eq!(source.manifest.coverage, old.coverage);
    let prepared = preparation::prepare(&root.join("e157-resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("e157-resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    let ranges = [(0x3ade0au32, 0x3ade62u32), (0x3ade64, 0x3adeb0)];
    for old_resource in &old.resources {
        let new = source.manifest.resources.iter().find(|r| r.id == old_resource.id).unwrap();
        let old_bytes = fs::read(previous.join(&old_resource.path)).unwrap();
        let new_bytes = fs::read(prepared.directory.join(&new.path)).unwrap();
        if old_resource.id != OBJ_STATIC_LISTS_ID { assert_eq!(new, old_resource); assert_eq!(new_bytes, old_bytes); continue; }
        let prior = parse_obj_lists(&old_bytes).unwrap();
        let mut current = parse_obj_lists(&new_bytes).unwrap();
        let directory = current.title_opening_directory.take().unwrap();
        assert_eq!(directory.len(), 19);
        for (i, value) in directory.iter().enumerate() {
            assert_eq!(*value, u32::from_le_bytes(rom[0x3ade64 + i*4..0x3ade68 + i*4].try_into().unwrap()));
        }
        let selected: Vec<_> = current.lists.iter().filter(|list| (ranges[0].0..ranges[0].1).contains(&list.source_offset)).collect();
        assert_eq!(selected.len(), 11);
        for list in selected { assert_eq!(list.objects.len(), 1); assert_eq!(list.source_bytes().unwrap(), rom[list.source_offset as usize..list.source_offset as usize + 8]); }
        current.lists.retain(|list| !(ranges[0].0..ranges[0].1).contains(&list.source_offset));
        assert_eq!(current, prior);
        let mut metadata = new.clone();
        let added: Vec<_> = metadata.sources.iter().filter(|s| !old_resource.sources.contains(s)).cloned().collect();
        assert_eq!(added.len(), 12); assert_eq!(added.iter().map(|s| s.length).sum::<u32>(), 164);
        metadata.sources.retain(|span| old_resource.sources.contains(span)); metadata.sha256 = old_resource.sha256.clone();
        assert_eq!(&metadata, old_resource);
    }
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let before = owner.obj_list_stats().unwrap();
    for &(lo,hi) in &ranges { managed.bus.rom[lo as usize..hi as usize].fill(0xcd); }
    for &(lo,hi) in &ranges { for at in lo..hi { assert_eq!(managed.bus.read8(0x0800_0000 + at), rom[at as usize]); } }
    assert_eq!(owner.obj_list_stats().unwrap().source_bytes - before.source_bytes, 164);
    let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for &(lo,hi) in &ranges { for edge in [lo,hi] { for delta in [-3isize,-1,0,1] {
        let at = (edge as usize).wrapping_add_signed(delta);
        assert_eq!((gba_core::capi::RT_API.read16)(ptr,0x0a00_0000 + at as u32),u32::from(u16::from_le_bytes(rom[at..at+2].try_into().unwrap())));
        assert_eq!((gba_core::capi::RT_API.read32)(ptr,0x0800_0000 + at as u32),u32::from_le_bytes(rom[at..at+4].try_into().unwrap()));
    }}}
    for &(lo,hi) in &ranges { managed.bus.rom[lo as usize..hi as usize].copy_from_slice(&rom[lo as usize..hi as usize]); }
    let mut counts = Vec::new();
    for case in 0..3 {
        let mut raw = crate::make_machine(rom.clone(), None);
        let initial = raw.save_state().unwrap(); managed.load_state(&initial).unwrap();
        for &(lo,hi) in &ranges { managed.bus.rom[lo as usize..hi as usize].fill(0xcd); }
        if case == 0 { initialize_original(&mut raw); }
        else { draw_original(&mut raw, if case == 1 { 0x3ade0a } else { 0x3ade5a }); }
        let trace = TraceCapture::start(TraceFilter { readers: vec![], ranges: ranges.to_vec() }, 10_000).unwrap();
        trace.frame(Some(managed.bus.frames));
        if case == 0 { initialize_original(&mut managed); }
        else { draw_original(&mut managed, if case == 1 { 0x3ade0a } else { 0x3ade5a }); }
        let report = trace.finish(); assert!(!report.truncated);
        let typed: u64 = report.events.iter().filter(|event| event.key.service == Service::Typed).map(|event| event.requests).sum();
        let fallback: u64 = report.events.iter().filter(|event| event.key.service == Service::Raw).map(|event| event.requests).sum();
        assert!(typed > 0, "no typed reads for original reader case {case}"); assert_eq!(fallback,0); counts.push(typed);
        for &(lo,hi) in &ranges { managed.bus.rom[lo as usize..hi as usize].copy_from_slice(&rom[lo as usize..hi as usize]); }
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "original reader case {case}");
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[157,156], "e156_projection_preserved":true,
        "new_spans":12, "new_bytes":164, "poisoned_source":true,
        "original_initializer_and_two_draw_cases_typed_requests":counts,
        "ordinary_draw":"pending", "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
