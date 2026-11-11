//! Conditional EUR context-updater OBJ bank, publication and original consumers.
use super::*;
use gba_core::{Bus, Machine};

const OBJECT_ROOT: u32 = 0x00b9_3ff0;
const SEQUENCE_ROOT: u32 = 0x00b9_3f50;
const SEQUENCE: u32 = 0x00b9_3f54;
const CONTEXT: u32 = 0x0202_0000;
const STACK: u32 = 0x0300_7c00;
const RETURN: u32 = 0x0800_0300;
const BUFFER: u32 = 0x0203_0010;

fn call(machine: &mut Machine, pc: u32) {
    machine.cpu.regs[13] = STACK;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = pc;
    machine.cpu.cpsr = 0xbf;
    for _ in 0..20_000 {
        machine.bus.frame_ready = false;
        machine.step();
        if machine.cpu.regs[15] == RETURN {
            assert_eq!(machine.cpu.regs[13], STACK);
            return;
        }
    }
    panic!("original consumer did not return: {pc:x}");
}

#[test]
#[ignore = "exact EUR ROM and E133 predecessor; Windows executable under Wine"]
fn european_context_bank_preserves_dataset_and_original_consumers() {
    let root = PathBuf::from(std::env::var_os("FFTA_CONTEXT_BANK_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_CONTEXT_BANK_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old_manifest: Manifest =
        toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (
            old_manifest.extractor_revision,
            old_manifest.decoder_revision
        ),
        (133, 132)
    );
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (134, 133)
    );
    assert_eq!(source.manifest.coverage, old_manifest.coverage);
    assert_eq!(
        source.manifest.resources.len(),
        old_manifest.resources.len()
    );
    let resources = root.join("e134-resources");
    let prepared = preparation::prepare(&resources, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(
        preparation::prepare(&resources, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );

    let mut spans = Vec::new();
    let mut frames = Vec::new();
    for old in &old_manifest.resources {
        let new = source
            .manifest
            .resources
            .iter()
            .find(|r| r.id == old.id)
            .unwrap();
        let old_bytes = fs::read(previous.join(&old.path)).unwrap();
        let new_bytes = fs::read(prepared.directory.join(&new.path)).unwrap();
        assert_eq!(gba_core::ffta_rom::digest(&old_bytes), old.sha256);
        if old.id != OBJ_ANIMATION_BANKS_ID {
            assert_eq!(new, old);
            assert_eq!(new_bytes, old_bytes);
            continue;
        }
        let prior = parse_obj_animations(&old_bytes).unwrap();
        let current = parse_obj_animations(&new_bytes).unwrap();
        let bank = current
            .banks
            .iter()
            .find(|b| b.object_root == OBJECT_ROOT && b.sequence_root == SEQUENCE_ROOT)
            .unwrap();
        assert_eq!(
            (
                bank.lists.len(),
                bank.sequences.len(),
                bank.sequences[0].frames.len()
            ),
            (21, 1, 38)
        );
        assert_eq!(
            bank.lists.iter().map(|l| l.objects.len()).sum::<usize>(),
            258
        );
        assert_eq!(bank.sequences[0].source_offset, SEQUENCE);
        spans = bank.spans();
        frames = bank.sequences[0].frames.clone();
        let mut projection = current.clone();
        projection.banks.retain(|b| b.object_root != OBJECT_ROOT);
        assert_eq!(projection, prior);
        let mut metadata = new.clone();
        metadata
            .sources
            .retain(|s| !spans.iter().any(|(at, _, _)| *at == s.offset));
        metadata.sha256 = old.sha256.clone();
        assert_eq!(&metadata, old);
    }
    assert_eq!(spans.len(), 24);
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut decoded = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut decoded, &prepared.directory).unwrap();
    let owner = decoded.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    for &(at, len, _) in &spans {
        decoded.bus.rom[at as usize..at as usize + len].fill(0xcd);
    }
    let before = owner.obj_animation_stats().unwrap().source_bytes;
    for &(at, len, _) in &spans {
        for i in 0..len {
            assert_eq!(
                decoded.bus.read8(0x0800_0000 + at + i as u32),
                rom[at as usize + i]
            );
        }
    }
    assert!(owner.obj_animation_stats().unwrap().source_bytes > before);
    for &(at, len, _) in &spans {
        decoded.bus.rom[at as usize..at as usize + len]
            .copy_from_slice(&rom[at as usize..at as usize + len]);
    }
    for frame_index in [0usize, 32, 37] {
        raw.load_state(&initial).unwrap();
        decoded.load_state(&initial).unwrap();
        for &(at, len, _) in &spans {
            decoded.bus.rom[at as usize..at as usize + len].fill(0xcd);
        }
        for machine in [&mut raw, &mut decoded] {
            machine.bus.ffta_sprites.enabled = false;
            machine.bus.write32(CONTEXT, 0x0800_0000 + OBJECT_ROOT);
            machine.bus.write32(CONTEXT + 0x18, 0x0800_0000 + SEQUENCE);
            machine.bus.write16(CONTEXT + 0x0e, frame_index as u16);
            machine.bus.write16(CONTEXT + 0x10, 0);
            machine.bus.write8(CONTEXT + 0x1d, 1);
            machine.cpu.regs[0] = CONTEXT;
            call(machine, 0x080a_ccf4);
        }
        let ptr = raw.bus.read32(CONTEXT + 0x14);
        assert_eq!(ptr, decoded.bus.read32(CONTEXT + 0x14));
        assert_eq!(raw.bus.read16(CONTEXT + 0x0e), frame_index as u16 + 1);
        assert_eq!(raw.bus.read16(CONTEXT + 0x10), frames[frame_index].duration);
        let before_draw = owner.obj_animation_stats().unwrap().source_bytes;
        for machine in [&mut raw, &mut decoded] {
            machine.cpu.regs[..4].copy_from_slice(&[ptr, 100, 70, 7]);
            for (i, value) in [2, 3, BUFFER, 128, 0].iter().enumerate() {
                machine.bus.write32(STACK + 4 * i as u32, *value);
            }
            for i in 0..128 * 8 {
                machine.bus.write8(BUFFER + i, 0x5a);
            }
            call(machine, 0x0800_170c);
        }
        assert!(owner.obj_animation_stats().unwrap().source_bytes > before_draw);
        for &(at, len, _) in &spans {
            decoded.bus.rom[at as usize..at as usize + len]
                .copy_from_slice(&rom[at as usize..at as usize + len]);
        }
        assert_eq!(
            raw.save_state().unwrap(),
            decoded.save_state().unwrap(),
            "full state/clock frame {frame_index}"
        );
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[134,133], "lists":21, "objects":258, "frames":38,
        "prior_resources_preserved":true, "poisoned_full_source_bytes_checked":true,
        "original_update_draw_full_state_clock_cases":3, "ordinary_scene":"pending",
        "translated_native":"pending", "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "ordinary EUR startup state, E136 dataset and translation; Windows executable under Wine"]
fn european_context_bank_ordinary_pairs() {
    let root = PathBuf::from(std::env::var_os("FFTA_CONTEXT_ORDINARY_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_CONTEXT_ORDINARY_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let state = fs::read(std::env::var_os("FFTA_CONTEXT_ORDINARY_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "ca90c12b4564c21fd8cea20e65bcc2c017d0d3a1ee61e89623920dd1f9a00305");
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    let resource = manifest.resources.iter().find(|r| r.id == OBJ_ANIMATION_BANKS_ID).unwrap();
    let banks = parse_obj_animations(&fs::read(dataset.join(&resource.path)).unwrap()).unwrap();
    let bank = banks.banks.iter().find(|b| b.object_root == OBJECT_ROOT && b.sequence_root == SEQUENCE_ROOT).unwrap();
    let spans = bank.spans();
    assert_eq!(spans.len(), 24);
    let library = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap());
    let (_library, table) = crate::load_native(&library).unwrap();
    let mut counts = Vec::new();
    for translated in [false, true] {
        let mut raw = crate::make_machine(rom.clone(), None);
        let mut managed = crate::make_machine(rom.clone(), None);
        attach_from_directory(&mut managed, &dataset).unwrap();
        raw.load_state(&state).unwrap();
        managed.load_state(&state).unwrap();
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
        assert_eq!(raw.cpu.regs[15], 0x0814_4aa2);
        let owner = managed.bus.rom_resources().unwrap().clone();
        let initial = owner.obj_animation_stats().unwrap().source_bytes;
        let poison = |machine: &mut Machine| {
            for &(at, len, _) in &spans { machine.bus.rom[at as usize..at as usize + len].fill(0xcd); }
        };
        let restore = |machine: &mut Machine| {
            for &(at, len, _) in &spans { machine.bus.rom[at as usize..at as usize + len].copy_from_slice(&rom[at as usize..at as usize + len]); }
        };
        poison(&mut managed);
        let mut target_reads = 0usize;
        let mut raw_reads = 0usize;
        let mut unscoped_typed_reads = 0usize;
        let mut native_dispatches = 0u64;
        let mut updater_call = false;
        for frame in 0..3 {
            let raw_ptr = &mut raw as *mut Machine as *mut std::ffi::c_void;
            let managed_ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
            if translated {
                crate::run_frame_native(&mut raw, &table, raw_ptr, crate::FRAME_STEP_GUARD);
            } else {
                raw.bus.frame_ready = false;
                for _ in 0..5_000_000 {
                    if raw.bus.frame_ready { break; }
                    let pc = raw.cpu.regs[15];
                    if matches!(raw.step(), gba_core::StepEvent::Instr(_)) && pc == 0x0814_4ac2 { updater_call = true; }
                }
            }
            let ranges = spans.iter().map(|&(at, len, _)| (at, at + len as u32)).collect();
            let capture = gba_core::rom_data::read_audit::TraceCapture::start(
                gba_core::rom_data::read_audit::TraceFilter { readers: vec![], ranges }, 10000
            ).unwrap();
            capture.frame(Some(managed.bus.frames));
            if translated {
                let (native, _) = crate::run_frame_native_traced(&mut managed, &table, managed_ptr, crate::FRAME_STEP_GUARD,
                    |machine, kind| {
                        if kind == crate::NativeDispatch::Native && (0x080a_c640..0x080a_cd00).contains(&machine.cpu.regs[15]) { updater_call = true; }
                    }, |_, _, _| {});
                native_dispatches += native;
            } else {
                managed.bus.frame_ready = false;
                for _ in 0..5_000_000 {
                    if managed.bus.frame_ready { break; }
                    managed.step();
                }
            }
            let trace = capture.finish();
            assert!(!trace.truncated);
            for event in trace.events {
                match event.key.service {
                    gba_core::rom_data::read_audit::Service::Typed => {
                        target_reads += event.requests as usize;
                        if event.key.instruction.is_none() { unscoped_typed_reads += event.requests as usize; }
                    }
                    gba_core::rom_data::read_audit::Service::Raw => raw_reads += event.requests as usize,
                    gba_core::rom_data::read_audit::Service::Unclassified => {}
                }
            }
            assert!(raw.bus.frame_ready && managed.bus.frame_ready, "frame {frame} exceeded step limit");
            restore(&mut managed);
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "ordinary full state and clock, translated={translated}, frame={frame}");
            poison(&mut managed);
        }
        let served = owner.obj_animation_stats().unwrap().source_bytes - initial;
        assert!(target_reads > 0, "no typed ordinary read of the selected E134 bank, translated={translated}");
        assert_eq!(raw_reads, 0, "raw fallback read in the selected E134 bank, translated={translated}");
        assert!(served > 0, "no managed animation source bytes, translated={translated}");
        if translated { assert!(native_dispatches > 0); } else { assert!(updater_call); }
        counts.push((target_reads, served, unscoped_typed_reads, native_dispatches, updater_call));
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "ordinary_interpreter_pair":true, "ordinary_translated_pair":true,
        "bank_reads_and_managed_animation_bytes":counts, "source_rom_poisoned":true,
        "full_state_clock_frames_each":3, "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "ordinary EUR checkpoint, E139 dataset and TinyCC; Windows executable under Wine"]
fn european_context_bank_ordinary_augmented_updater() {
    use gba_core::rom_data::read_audit::{Reader, Service, TraceCapture, TraceFilter, TraceReport};
    use recomp_core::analyze::{Analysis, Block, View};
    use std::collections::BTreeSet;

    const UPDATER: std::ops::Range<u32> = 0x080a_c640..0x080a_cd00;
    let root = PathBuf::from(std::env::var_os("FFTA_CONTEXT_ORDINARY_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_CONTEXT_ORDINARY_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let state = fs::read(std::env::var_os("FFTA_CONTEXT_ORDINARY_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "ca90c12b4564c21fd8cea20e65bcc2c017d0d3a1ee61e89623920dd1f9a00305");
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    let resource = manifest.resources.iter().find(|r| r.id == OBJ_ANIMATION_BANKS_ID).unwrap();
    let banks = parse_obj_animations(&fs::read(dataset.join(&resource.path)).unwrap()).unwrap();
    let bank = banks.banks.iter().find(|b| b.object_root == OBJECT_ROOT && b.sequence_root == SEQUENCE_ROOT).unwrap();
    let spans = bank.spans();
    assert_eq!(spans.len(), 24);

    // Capture the updater instructions from the unchanged ordinary checkpoint.
    let mut trace = crate::make_machine(rom.clone(), None);
    trace.load_state(&state).unwrap();
    let mut visited = BTreeSet::new();
    for _ in 0..3 {
        trace.bus.frame_ready = false;
        for _ in 0..5_000_000 {
            if trace.bus.frame_ready { break; }
            let pc = trace.cpu.regs[15];
            if UPDATER.contains(&pc) { visited.insert((pc, trace.cpu.thumb())); }
            trace.step();
        }
        assert!(trace.bus.frame_ready);
    }
    assert!(!visited.is_empty(), "ordinary checkpoint did not reach updater");
    let view = View { rom: &rom, ewram: None, iwram: Some(&trace.bus.iwram), bios: None };
    let blocks = visited.iter().map(|&(address, thumb)| {
        let first = if thumb { armv4t::decode_thumb(view.read16(address), address) }
            else { armv4t::decode_arm(view.read32(address), address) };
        let mut instrs = vec![first];
        if thumb {
            let next = armv4t::decode_thumb(view.read16(address + 2), address + 2);
            if armv4t::fuse_thumb_bl(&instrs[0], &next).is_some() { instrs.push(next); }
        }
        Block { addr: address, thumb, instrs }
    }).collect();
    let object = root.join("ordinary-updater-supplement.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view,
        &recomp_core::tcc_memory::library_path().expect("TinyCC required"), &object, &mut |_| {}).unwrap();
    let (_supplement_library, supplement) = crate::load_native(&object).unwrap();
    let (_production_library, mut table) = crate::load_native(&PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap())).unwrap();
    for &(pc, thumb) in &visited {
        let key = pc | u32::from(thumb);
        assert!(table.get(key).is_none(), "production already has updater entry {key:08x}");
        let function = supplement.get(key).expect("compiled observed updater instruction");
        let index = key.wrapping_sub(0x0800_0000) as usize;
        if index >= table.rom.len() { table.rom.resize(index + 1, None); }
        table.rom[index] = Some(function);
    }

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &dataset).unwrap();
    raw.load_state(&state).unwrap();
    managed.load_state(&state).unwrap();
    fn inspect(report: TraceReport, unscoped: &mut usize, raw_reads: &mut usize) -> bool {
        assert!(!report.truncated);
        let before = *unscoped;
        for event in report.events {
            match event.key.service {
                Service::Typed if event.key.instruction.is_none() => *unscoped += event.requests as usize,
                Service::Raw => *raw_reads += event.requests as usize,
                _ => {}
            }
        }
        *unscoped > before
    }
    let mut unscoped = 0usize;
    let mut raw_reads = 0usize;
    let mut updater_dispatches = 0usize;
    let mut updater_source_dispatches = 0usize;
    for frame in 0..3 {
        for &(at, len, _) in &spans { managed.bus.rom[at as usize..at as usize + len].fill(0xcd); }
        let raw_ptr = &mut raw as *mut Machine as *mut std::ffi::c_void;
        crate::run_frame_native(&mut raw, &table, raw_ptr, crate::FRAME_STEP_GUARD);
        assert!(raw.bus.frame_ready);
        let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
        let mut active_capture: Option<TraceCapture> = None;
        crate::run_frame_native_traced(&mut managed, &table, ptr, crate::FRAME_STEP_GUARD,
            |machine, kind| {
                if let Some(capture) = active_capture.take() {
                    if inspect(capture.finish(), &mut unscoped, &mut raw_reads) { updater_source_dispatches += 1; }
                }
                let native_updater = kind == crate::NativeDispatch::Native && UPDATER.contains(&machine.cpu.regs[15]);
                if native_updater {
                    updater_dispatches += 1;
                    let ranges = spans.iter().map(|&(at, len, _)| (at, at + len as u32)).collect();
                    let capture = TraceCapture::start(TraceFilter { readers: vec![Reader::Bus], ranges }, 10000).unwrap();
                    capture.frame(Some(machine.bus.frames));
                    active_capture = Some(capture);
                }
            }, |_, _, _| {});
        assert!(managed.bus.frame_ready);
        if let Some(capture) = active_capture.take() {
            if inspect(capture.finish(), &mut unscoped, &mut raw_reads) { updater_source_dispatches += 1; }
        }
        for &(at, len, _) in &spans { managed.bus.rom[at as usize..at as usize + len].copy_from_slice(&rom[at as usize..at as usize + len]); }
        assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "full state and clock frame {frame}");
    }
    assert_eq!(raw_reads, 0);
    assert!(updater_dispatches > 0, "no augmented updater dispatch");
    assert!(updater_source_dispatches > 0, "no managed animation bytes served during augmented updater dispatch");
    assert!(unscoped > 0, "no selected-bank typed reads outside interpreter scope");
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"ordinary EUR checkpoint, temporary augmented translation table",
        "observed_updater_instructions":visited.len(), "native_updater_dispatches":updater_dispatches,
        "native_updater_dispatches_serving_animation_bytes":updater_source_dispatches,
        "unscoped_selected_bank_typed_reads":unscoped, "raw_selected_bank_reads":raw_reads,
        "full_state_clock_frames":3, "source_rom_poisoned":true,
        "production_translation_coverage":"not established", "ordinary_translated_draw":"pending",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
