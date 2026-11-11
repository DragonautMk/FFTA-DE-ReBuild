//! EUR context-selected OBJ lists, publication and ordinary source use.
use super::*;
use gba_core::{rom_data::obj_lists::context_selector::{self, ROOTS}, Bus, Machine};

const CONTEXT: u32 = 0x0202_0000;
const RETURN: u32 = 0x0800_0300;
const STACK: u32 = 0x0300_7c00;

fn original_select(machine: &mut Machine, selector: u32) -> u32 {
    machine.bus.write32(CONTEXT + 0x10, 0x1234_5678);
    machine.cpu.regs[0] = CONTEXT;
    machine.cpu.regs[1] = selector;
    machine.cpu.regs[13] = STACK;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = 0x0814_380c;
    machine.cpu.cpsr = 0xbf;
    for _ in 0..1000 {
        machine.bus.frame_ready = false;
        machine.step();
        if machine.cpu.regs[15] == RETURN { return machine.bus.read32(CONTEXT + 0x10); }
    }
    panic!("original OBJ selector did not return");
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "exact EUR ROM, E139 predecessor, ordinary state and translation; Wine only"]
fn european_context_obj_selector_publication_and_ordinary_pairs() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};

    let root = PathBuf::from(std::env::var_os("FFTA_CONTEXT_SELECTOR_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_CONTEXT_SELECTOR_PREVIOUS").unwrap());
    let state = fs::read(std::env::var_os("FFTA_CONTEXT_SELECTOR_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "d8f174e8f886ac0edbbd060b6a0509c4f0c333b8bdd6f8645c536be35df6e541");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (139, 138));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision), (140, 139));
    assert_eq!(source.manifest.coverage, old.coverage);
    assert_eq!(source.manifest.resources.len(), old.resources.len());
    let prepared = preparation::prepare(&root.join("e140-resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("e140-resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    for old_resource in &old.resources {
        let new = source.manifest.resources.iter().find(|r| r.id == old_resource.id).unwrap();
        let old_bytes = fs::read(previous.join(&old_resource.path)).unwrap();
        let new_bytes = fs::read(prepared.directory.join(&new.path)).unwrap();
        assert_eq!(gba_core::ffta_rom::digest(&old_bytes), old_resource.sha256);
        if old_resource.id != OBJ_STATIC_LISTS_ID {
            assert_eq!(new, old_resource);
            assert!(new_bytes == old_bytes, "unrelated resource changed");
            continue;
        }
        let prior = parse_obj_lists(&old_bytes).unwrap();
        let mut current = parse_obj_lists(&new_bytes).unwrap();
        for (offset, count, _) in ROOTS {
            assert_eq!(current.lists.iter().find(|l| l.source_offset == offset).unwrap().objects.len(), count);
        }
        current.lists.retain(|list| !ROOTS.iter().any(|root| root.0 == list.source_offset));
        assert_eq!(current, prior);
        let mut metadata = new.clone();
        metadata.sources.retain(|s| !ROOTS.iter().any(|root| root.0 == s.offset));
        metadata.sha256 = old_resource.sha256.clone();
        assert_eq!(&metadata, old_resource);
    }
    let catalog = context_selector::Catalog::from_rom(&rom).unwrap();
    assert_eq!(catalog.sources.iter().map(|s| s.length).sum::<u32>(), 50);
    let mut selector_machine = crate::make_machine(rom.clone(), None);
    for selector in [0, 1, 2, 3, 4, 5, 255, 0x100, 0x105] {
        let expected = context_selector::select(selector).map_or(0x1234_5678, |p| 0x0800_0000 + p);
        assert_eq!(original_select(&mut selector_machine, selector), expected, "selector {selector:x}");
    }
    let mut byte_machine = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut byte_machine, &prepared.directory).unwrap();
    let owner = byte_machine.bus.rom_resources().unwrap().clone();
    for &(at, count, _) in &ROOTS { byte_machine.bus.rom[at as usize..at as usize + 2 + count * 6].fill(0xcd); }
    let before = owner.obj_list_stats().unwrap().source_bytes;
    for &(at, count, _) in &ROOTS {
        for i in 0..2 + count * 6 { assert_eq!(byte_machine.bus.read8(0x0800_0000 + at + i as u32), rom[at as usize + i]); }
    }
    assert_eq!(owner.obj_list_stats().unwrap().source_bytes - before, 50);
    for at in [0x003a_e43fu32, 0x003a_e472] { assert_eq!(byte_machine.bus.read8(0x0800_0000 + at), rom[at as usize]); }

    let (_library, table) = crate::load_native(&PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap())).unwrap();
    let mut counts = Vec::new();
    for native in [false, true] {
        let mut raw = crate::make_machine(rom.clone(), None);
        let mut managed = crate::make_machine(rom.clone(), None);
        attach_from_directory(&mut managed, &prepared.directory).unwrap();
        raw.load_state(&state).unwrap();
        managed.load_state(&state).unwrap();
        let owner = managed.bus.rom_resources().unwrap().clone();
        let mut typed = 0usize;
        let mut raw_reads = 0usize;
        let mut native_dispatches = 0u64;
        for frame in 0..3 {
            for &(at, count, _) in &ROOTS { managed.bus.rom[at as usize..at as usize + 2 + count * 6].fill(0xcd); }
            if native {
                let ptr = &mut raw as *mut Machine as *mut std::ffi::c_void;
                crate::run_frame_native(&mut raw, &table, ptr, crate::FRAME_STEP_GUARD);
            } else {
                raw.bus.frame_ready = false;
                for _ in 0..5_000_000 { if raw.bus.frame_ready { break; } raw.step(); }
            }
            assert!(raw.bus.frame_ready);
            let ranges = ROOTS.iter().map(|&(at, count, _)| (at, at + 2 + (count * 6) as u32)).collect();
            let capture = TraceCapture::start(TraceFilter { readers: vec![], ranges }, 10000).unwrap();
            capture.frame(Some(managed.bus.frames));
            if native {
                let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
                native_dispatches += crate::run_frame_native(&mut managed, &table, ptr, crate::FRAME_STEP_GUARD).0;
            } else {
                managed.bus.frame_ready = false;
                for _ in 0..5_000_000 { if managed.bus.frame_ready { break; } managed.step(); }
            }
            assert!(managed.bus.frame_ready);
            let report = capture.finish();
            assert!(!report.truncated);
            for event in report.events {
                match event.key.service {
                    Service::Typed => typed += event.requests as usize,
                    Service::Raw => raw_reads += event.requests as usize,
                    _ => {}
                }
            }
            for &(at, count, _) in &ROOTS { managed.bus.rom[at as usize..at as usize + 2 + count * 6].copy_from_slice(&rom[at as usize..at as usize + 2 + count * 6]); }
            assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "full state and clock, native={native}, frame={frame}");
        }
        assert!(owner.obj_list_source_used(ROOTS[0].0).unwrap().0, "ordinary selected list not used");
        assert!(typed > 0 && raw_reads == 0);
        if native { assert!(native_dispatches > 0); }
        counts.push((typed, raw_reads, native_dispatches));
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[140,139], "source_lists":4, "source_bytes":50,
        "prior_resources_preserved":true, "original_selector_cases":9,
        "source_rom_poisoned":true, "ordinary_interpreter_and_native_dispatch_pairs":counts,
        "full_state_clock_frames_each":3, "ordinary_other_selectors":"pending", "per_read_native_attribution":"pending",
        "frontend_ui":"pending", "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
