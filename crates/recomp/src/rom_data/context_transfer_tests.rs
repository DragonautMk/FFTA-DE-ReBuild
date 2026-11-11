//! EUR conditional context transfer sources and original halfword DMA queue.
use super::*;
use gba_core::{capi::RT_API, Bus, Machine};

const SOURCES: [(u32, usize); 3] = [
    (0x00b8_8c18, 0x2de0),
    (0x00b8_dcb4, 0x800),
    (0x00b8_e4b4, 0x800),
];
// Values written by the original initializer; later live writes remain possible.
const CONDITIONAL_DESTINATIONS: [u32; 3] = [0x0600_8000, 0x0600_7000, 0x0600_7800];
const NODE: u32 = 0x0201_0000;
const STACK: u32 = 0x0300_7c00;
const RETURN: u32 = 0x0300_7f00;

fn original(machine: &mut Machine, pc: u32) {
    machine.cpu.regs[13] = STACK;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = pc;
    machine.cpu.cpsr = 0xbf;
    for _ in 0..100_000 {
        machine.bus.frame_ready = false;
        machine.step();
        if machine.cpu.regs[15] == RETURN {
            return;
        }
    }
    panic!("original transfer routine did not return: {pc:x}");
}

#[test]
#[ignore = "exact EUR ROM and E135 predecessor; Windows executable under Wine"]
fn european_context_transfers_preserve_dataset_and_original_queue() {
    let root = PathBuf::from(std::env::var_os("FFTA_CONTEXT_TRANSFERS_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_CONTEXT_TRANSFERS_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old_manifest: Manifest =
        toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (
            old_manifest.extractor_revision,
            old_manifest.decoder_revision
        ),
        (135, 134)
    );
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (136, 135)
    );
    assert_eq!(source.manifest.coverage, old_manifest.coverage);
    assert_eq!(
        source.manifest.resources.len(),
        old_manifest.resources.len()
    );
    let resources = root.join("e136-resources");
    let prepared = preparation::prepare(&resources, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(
        preparation::prepare(&resources, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
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
        if old.id != UI_RAW_TRANSFERS_ID {
            assert_eq!(new, old);
            assert_eq!(new_bytes, old_bytes);
            continue;
        }
        let prior = parse_ui_raw_transfers(&old_bytes).unwrap();
        let mut current = parse_ui_raw_transfers(&new_bytes).unwrap();
        assert_eq!((prior.fixed_banks.len(), current.fixed_banks.len()), (1, 4));
        for (bank, &(at, length)) in current.fixed_banks[1..].iter().zip(&SOURCES) {
            assert_eq!(bank.bank_offset, at);
            assert_eq!(bank.source_bytes(), rom[at as usize..at as usize + length]);
        }
        current.fixed_banks.truncate(1);
        assert_eq!(current, prior);
        let mut metadata = new.clone();
        metadata
            .sources
            .retain(|s| s.role != "fixed-context-transfer");
        metadata.sha256 = old.sha256.clone();
        assert_eq!(&metadata, old);
    }
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut decoded = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut decoded, &prepared.directory).unwrap();
    let owner = decoded.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    for &(at, length) in &SOURCES {
        decoded.bus.rom[at as usize..at as usize + length].fill(0xcd);
    }
    let before = owner.ui_raw_transfers_stats().unwrap();
    for &(at, length) in &SOURCES {
        for i in 0..length {
            assert_eq!(
                decoded.bus.read8(0x0800_0000 + at + i as u32),
                rom[at as usize + i]
            );
        }
    }
    let after = owner.ui_raw_transfers_stats().unwrap();
    assert_eq!(after.payload_bytes - before.payload_bytes, 15840);
    assert_eq!(
        (after.selector_bytes, after.banks_used, after.groups_used),
        (before.selector_bytes, before.banks_used, before.groups_used)
    );
    let pointer = &mut decoded as *mut Machine as *mut std::ffi::c_void;
    for &(start, length) in &SOURCES {
        for edge in [start, start + length as u32] {
            for delta in [-3i32, -1, 0, 1] {
                let at = (edge as i32 + delta) as usize;
                assert_eq!(
                    (RT_API.read16)(pointer, 0x0a00_0000 + at as u32),
                    u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap()))
                );
                assert_eq!(
                    (RT_API.read32)(pointer, 0x0800_0000 + at as u32),
                    u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
                );
            }
        }
    }
    for &(at, length) in &SOURCES {
        decoded.bus.rom[at as usize..at as usize + length]
            .copy_from_slice(&rom[at as usize..at as usize + length]);
    }
    for (&(at, length), destination) in SOURCES.iter().zip(CONDITIONAL_DESTINATIONS) {
        raw.load_state(&initial).unwrap();
        decoded.load_state(&initial).unwrap();
        for &(offset, len) in &SOURCES {
            decoded.bus.rom[offset as usize..offset as usize + len].fill(0xcd);
        }
        let before_dma = owner.ui_raw_transfers_stats().unwrap();
        for machine in [&mut raw, &mut decoded] {
            machine.bus.ffta_sprites.enabled = false;
            machine.bus.iwram.fill(0);
            machine.bus.write32(0x0300_2bcc, 0);
            machine.cpu.regs[..4].copy_from_slice(&[
                NODE,
                0x0800_0000 + at,
                destination,
                (length / 2) as u32,
            ]);
            original(machine, 0x0800_0ad0);
            assert_eq!(machine.bus.read32(NODE + 4), 0x0800_0000 + at);
            assert_eq!(machine.bus.read32(NODE + 8), destination);
            assert_eq!(
                machine.bus.read32(NODE + 12),
                0x8000_0000 | (length / 2) as u32
            );
            original(machine, 0x0800_0800);
        }
        let after_dma = owner.ui_raw_transfers_stats().unwrap();
        assert!(after_dma.payload_bytes - before_dma.payload_bytes >= length as u64);
        assert_eq!(
            (
                after_dma.selector_bytes,
                after_dma.banks_used,
                after_dma.groups_used
            ),
            (
                before_dma.selector_bytes,
                before_dma.banks_used,
                before_dma.groups_used
            )
        );
        let destination_offset = (destination - 0x0600_0000) as usize;
        assert_eq!(
            &decoded.bus.vram[destination_offset..destination_offset + length],
            &rom[at as usize..at as usize + length]
        );
        for &(offset, len) in &SOURCES {
            decoded.bus.rom[offset as usize..offset as usize + len]
                .copy_from_slice(&rom[offset as usize..offset as usize + len]);
        }
        assert_eq!(
            raw.save_state().unwrap(),
            decoded.save_state().unwrap(),
            "full original queue/DMA state and clock {at:x}"
        );
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS", "revision":[136,135], "new_source_bytes":15840, "source_ranges":3, "prior_resources_preserved":true, "original_queue_and_dma_full_state_clock_cases":3, "conditional_vram_destinations_injected":true, "localized_stats_unchanged":true, "live_destination_order":"pending", "ordinary_scene":"pending", "translated_native":"pending", "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false})).unwrap()).unwrap();
}

#[test]
#[ignore = "ordinary EUR startup pre-queue state and E136 dataset; Windows executable under Wine"]
fn european_context_transfers_ordinary_queue_dma_pair() {
    let root = PathBuf::from(std::env::var_os("FFTA_CONTEXT_ORDINARY_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_CONTEXT_ORDINARY_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let state = fs::read(std::env::var_os("FFTA_CONTEXT_ORDINARY_STATE").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&state),
        "9d68a0e85fecf7c8d9f314d92f21773cf5d3d5648ea051988185a2e28801e74d"
    );
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &dataset).unwrap();
    raw.load_state(&state).unwrap();
    managed.load_state(&state).unwrap();
    assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
    assert_eq!(raw.cpu.regs[15], 0x0814_4b3e);
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial_payload = owner.ui_raw_transfers_stats().unwrap().payload_bytes;
    let poison = |machine: &mut Machine| {
        for &(at, length) in &SOURCES {
            machine.bus.rom[at as usize..at as usize + length].fill(0xcd);
        }
    };
    let restore = |machine: &mut Machine| {
        for &(at, length) in &SOURCES {
            machine.bus.rom[at as usize..at as usize + length]
                .copy_from_slice(&rom[at as usize..at as usize + length]);
        }
    };
    let expected = [
        (0x0814_4b48, 0x08b8_8c18, 0x0600_0000, 0x16f0),
        (0x0814_4b5a, 0x08b8_dcb4, 0x0600_9000, 0x0400),
        (0x0814_4b6a, 0x08b8_e4b4, 0x0600_8000, 0x0400),
    ];
    let mut calls = 0usize;
    let mut vram_verified = false;
    let mut last_payload = initial_payload;
    poison(&mut managed);
    for frame in 0..3 {
        raw.bus.frame_ready = false;
        managed.bus.frame_ready = false;
        for _ in 0..5_000_000 {
            assert_eq!(raw.bus.frame_ready, managed.bus.frame_ready);
            if raw.bus.frame_ready { break; }
            assert_eq!((raw.cpu.regs[15], raw.cpu.thumb()), (managed.cpu.regs[15], managed.cpu.thumb()));
            let pc = raw.cpu.regs[15];
            let args = [raw.cpu.regs[1], raw.cpu.regs[2], raw.cpu.regs[3]];
            let raw_event = raw.step();
            let managed_event = managed.step();
            let raw_executed = matches!(raw_event, gba_core::StepEvent::Instr(_));
            assert_eq!(raw_executed, matches!(managed_event, gba_core::StepEvent::Instr(_)));
            if raw_executed && expected.iter().any(|v| v.0 == pc) {
                let &(want_pc, source, destination, count) = expected.get(calls).expect("unexpected extra transfer call");
                assert_eq!((pc, args), (want_pc, [source, destination, count]));
                calls += 1;
            }
            let payload = owner.ui_raw_transfers_stats().unwrap().payload_bytes;
            if calls == 3 && !vram_verified && payload != last_payload && payload - initial_payload >= 15_840 {
                vram_verified = SOURCES.iter().zip([0usize, 0x9000, 0x8000]).all(|(&(at, length), dest)| {
                    managed.bus.vram[dest..dest + length] == rom[at as usize..at as usize + length]
                });
            }
            last_payload = payload;
        }
        assert!(raw.bus.frame_ready && managed.bus.frame_ready, "frame {frame} exceeded step limit");
        restore(&mut managed);
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "full state and guest clock at frame {frame}");
        poison(&mut managed);
    }
    restore(&mut managed);
    let stats = owner.ui_raw_transfers_stats().unwrap();
    assert_eq!(calls, 3);
    assert!(vram_verified, "ordinary DMA did not produce all three exact VRAM ranges");
    assert!(stats.payload_bytes - initial_payload >= 15_840);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[136,135], "ordinary_queue_calls":calls,
        "live_destinations":["06000000","06009000","06008000"],
        "exact_vram_after_dma":true, "source_rom_poisoned":true,
        "full_state_clock_frames":3, "managed_payload_bytes":stats.payload_bytes-initial_payload,
        "ordinary_translated_pair":"pending", "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "ordinary EUR startup pre-queue state, E136 dataset and translation; Windows executable under Wine"]
fn european_context_transfers_ordinary_translated_pair() {
    let root = PathBuf::from(std::env::var_os("FFTA_CONTEXT_ORDINARY_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_CONTEXT_ORDINARY_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let state = fs::read(std::env::var_os("FFTA_CONTEXT_ORDINARY_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "9d68a0e85fecf7c8d9f314d92f21773cf5d3d5648ea051988185a2e28801e74d");
    let library = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap());
    let (_library, table) = crate::load_native(&library).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &dataset).unwrap();
    raw.load_state(&state).unwrap();
    managed.load_state(&state).unwrap();
    assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
    assert_eq!(raw.cpu.regs[15], 0x0814_4b3e);
    let owner = managed.bus.rom_resources().unwrap().clone();
    let initial_payload = owner.ui_raw_transfers_stats().unwrap().payload_bytes;
    let poison = |machine: &mut Machine| {
        for &(at, length) in &SOURCES {
            machine.bus.rom[at as usize..at as usize + length].fill(0xcd);
        }
    };
    let restore = |machine: &mut Machine| {
        for &(at, length) in &SOURCES {
            machine.bus.rom[at as usize..at as usize + length]
                .copy_from_slice(&rom[at as usize..at as usize + length]);
        }
    };
    let mut vram_verified = false;
    let mut translated_queue_or_dma = false;
    poison(&mut managed);
    for frame in 0..3 {
        let raw_ptr = &mut raw as *mut Machine as *mut std::ffi::c_void;
        let managed_ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
        crate::run_frame_native(&mut raw, &table, raw_ptr, crate::FRAME_STEP_GUARD);
        crate::run_frame_native_traced(&mut managed, &table, managed_ptr, crate::FRAME_STEP_GUARD,
            |machine, kind| {
                let pc = machine.cpu.regs[15];
                if kind == crate::NativeDispatch::Native && (0x0800_0800..0x0800_0c00).contains(&pc) {
                    translated_queue_or_dma = true;
                }
                if owner.ui_raw_transfers_stats().unwrap().payload_bytes - initial_payload >= 15_840 {
                    vram_verified |= SOURCES.iter().zip([0usize, 0x9000, 0x8000]).all(|(&(at, length), dest)| {
                        machine.bus.vram[dest..dest + length] == rom[at as usize..at as usize + length]
                    });
                }
            }, |_, _, _| {});
        assert!(raw.bus.frame_ready && managed.bus.frame_ready, "frame {frame} exceeded step limit");
        restore(&mut managed);
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "translated full state and guest clock at frame {frame}");
        poison(&mut managed);
    }
    restore(&mut managed);
    let payload = owner.ui_raw_transfers_stats().unwrap().payload_bytes - initial_payload;
    assert!(payload >= 15_840);
    assert!(vram_verified, "ordinary translated DMA did not produce all three exact VRAM ranges");
    assert!(translated_queue_or_dma, "no original queue/DMA code was translated on this route");
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[136,135], "ordinary_translated_pair":true,
        "translated_queue_or_dma":true, "exact_vram_after_dma":true, "source_rom_poisoned":true,
        "full_state_clock_frames":3, "managed_payload_bytes":payload,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "ordinary EUR startup pre-queue state, E136 dataset and translation; Windows executable under Wine"]
fn european_fixed_obj_transfer_ordinary_pairs() {
    const SOURCE: usize = 0x00b8_0b5c;
    const LENGTH: usize = 0x8000;
    let root = PathBuf::from(std::env::var_os("FFTA_CONTEXT_ORDINARY_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_CONTEXT_ORDINARY_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let state = fs::read(std::env::var_os("FFTA_CONTEXT_ORDINARY_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "ca90c12b4564c21fd8cea20e65bcc2c017d0d3a1ee61e89623920dd1f9a00305");
    let library = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap());
    let (_library, table) = crate::load_native(&library).unwrap();
    let mut payloads = Vec::new();
    for translated in [false, true] {
        let mut raw = crate::make_machine(rom.clone(), None);
        let mut managed = crate::make_machine(rom.clone(), None);
        attach_from_directory(&mut managed, &dataset).unwrap();
        raw.load_state(&state).unwrap();
        managed.load_state(&state).unwrap();
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
        assert_eq!(raw.cpu.regs[15], 0x0814_4aa2);
        let owner = managed.bus.rom_resources().unwrap().clone();
        let initial_payload = owner.ui_raw_transfers_stats().unwrap().payload_bytes;
        let mut queue_verified = false;
        let mut native_queue_or_dma = false;
        let mut vram_verified = false;
        managed.bus.rom[SOURCE..SOURCE + LENGTH].fill(0xcd);
        for frame in 0..3 {
            if translated {
                let raw_ptr = &mut raw as *mut Machine as *mut std::ffi::c_void;
                let managed_ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
                crate::run_frame_native(&mut raw, &table, raw_ptr, crate::FRAME_STEP_GUARD);
                crate::run_frame_native_traced(&mut managed, &table, managed_ptr, crate::FRAME_STEP_GUARD,
                    |machine, kind| {
                        let pc = machine.cpu.regs[15];
                        if kind == crate::NativeDispatch::Native && (0x0800_0800..0x0800_0c00).contains(&pc) {
                            native_queue_or_dma = true;
                        }
                        if owner.ui_raw_transfers_stats().unwrap().payload_bytes - initial_payload >= LENGTH as u64 {
                            vram_verified |= machine.bus.vram[0x10000..0x18000] == rom[SOURCE..SOURCE + LENGTH];
                        }
                    }, |_, _, _| {});
            } else {
                raw.bus.frame_ready = false;
                managed.bus.frame_ready = false;
                for _ in 0..5_000_000 {
                    assert_eq!(raw.bus.frame_ready, managed.bus.frame_ready);
                    if raw.bus.frame_ready { break; }
                    assert_eq!((raw.cpu.regs[15], raw.cpu.thumb()), (managed.cpu.regs[15], managed.cpu.thumb()));
                    let pc = raw.cpu.regs[15];
                    let args = [raw.cpu.regs[1], raw.cpu.regs[2], raw.cpu.regs[3]];
                    let raw_executed = matches!(raw.step(), gba_core::StepEvent::Instr(_));
                    assert_eq!(raw_executed, matches!(managed.step(), gba_core::StepEvent::Instr(_)));
                    if raw_executed && pc == 0x0814_4aa2 {
                        assert_eq!(args, [0x08b8_0b5c, 0x0601_0000, 0x4000]);
                        queue_verified = true;
                    }
                    if owner.ui_raw_transfers_stats().unwrap().payload_bytes - initial_payload >= LENGTH as u64 {
                        vram_verified |= managed.bus.vram[0x10000..0x18000] == rom[SOURCE..SOURCE + LENGTH];
                    }
                }
            }
            assert!(raw.bus.frame_ready && managed.bus.frame_ready, "frame {frame} exceeded step limit");
            managed.bus.rom[SOURCE..SOURCE + LENGTH].copy_from_slice(&rom[SOURCE..SOURCE + LENGTH]);
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "ordinary full state and guest clock, translated={translated}, frame={frame}");
            managed.bus.rom[SOURCE..SOURCE + LENGTH].fill(0xcd);
        }
        let payload = owner.ui_raw_transfers_stats().unwrap().payload_bytes - initial_payload;
        assert!(payload >= LENGTH as u64);
        assert!(vram_verified, "ordinary DMA did not produce the exact VRAM range, translated={translated}");
        assert!(if translated { native_queue_or_dma } else { queue_verified });
        payloads.push(payload);
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "ordinary_interpreter_pair":true, "ordinary_translated_pair":true,
        "translated_queue_or_dma":true, "exact_vram_after_dma":true, "source_rom_poisoned":true,
        "full_state_clock_frames_each":3, "managed_payload_bytes":payloads,
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "exact EUR ROM and E136 dataset; isolated copies under Windows Wine"]
fn european_context_resources_survive_slot_undo_backup_restart() {
    let root = PathBuf::from(std::env::var_os("FFTA_CONTEXT_ORDINARY_ROOT").unwrap());
    let original = PathBuf::from(std::env::var_os("FFTA_CONTEXT_ORDINARY_DATASET").unwrap());
    let temporary = tempfile::Builder::new().prefix("context-lifecycle-").tempdir_in(&root).unwrap();
    let dataset = temporary.path().join("dataset");
    fs::create_dir(&dataset).unwrap();
    for entry in fs::read_dir(&original).unwrap() {
        let entry = entry.unwrap();
        assert!(entry.file_type().unwrap().is_file());
        fs::copy(entry.path(), dataset.join(entry.file_name())).unwrap();
    }
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision, manifest.decoder_revision), (136, 135));
    let resource = manifest.resources.iter().find(|r| r.id == OBJ_ANIMATION_BANKS_ID).unwrap();
    let banks = parse_obj_animations(&fs::read(dataset.join(&resource.path)).unwrap()).unwrap();
    let bank = banks.banks.iter().find(|b| b.object_root == 0x00b9_3ff0 && b.sequence_root == 0x00b9_3f50).unwrap();
    let spans = bank.spans();
    assert_eq!(spans.len(), 24);
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let mut machine = crate::make_machine(rom.clone(), None);
    let boot = machine.save_state().unwrap();
    attach_from_directory(&mut machine, &dataset).unwrap();
    let owner = machine.bus.rom_resources().unwrap().clone();
    let fixed = [(0x00b8_0b5c, 0x8000), SOURCES[0], SOURCES[1], SOURCES[2]];
    let verify = |machine: &mut Machine| {
        let before_fixed = owner.ui_raw_transfers_stats().unwrap().payload_bytes;
        let before_obj = owner.obj_animation_stats().unwrap().source_bytes;
        for &(at, len) in &fixed { machine.bus.rom[at as usize..at as usize + len].fill(0xcd); }
        for &(at, len, _) in &spans { machine.bus.rom[at as usize..at as usize + len].fill(0xcd); }
        for &(at, len) in &fixed {
            for i in 0..len { assert_eq!(machine.bus.read8(0x0800_0000 + at + i as u32), rom[at as usize + i]); }
        }
        for &(at, len, _) in &spans {
            for i in 0..len { assert_eq!(machine.bus.read8(0x0800_0000 + at + i as u32), rom[at as usize + i]); }
        }
        assert!(owner.ui_raw_transfers_stats().unwrap().payload_bytes - before_fixed >= 48_608);
        assert!(owner.obj_animation_stats().unwrap().source_bytes > before_obj);
        for &(at, len) in &fixed { machine.bus.rom[at as usize..at as usize + len].copy_from_slice(&rom[at as usize..at as usize + len]); }
        for &(at, len, _) in &spans { machine.bus.rom[at as usize..at as usize + len].copy_from_slice(&rom[at as usize..at as usize + len]); }
    };
    verify(&mut machine);
    fs::rename(&dataset, temporary.path().join("retained-files")).unwrap();
    let state = machine.save_state().unwrap();
    assert!(attach_from_directory(&mut machine, &dataset).is_err());
    assert_eq!(machine.save_state().unwrap(), state);
    assert!(std::sync::Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
    let save = temporary.path().join("isolated.sav");
    let mut slots = crate::save_state::Slots::new(&save, &machine.bus.rom);
    slots.save(&machine, 1, &vec![[0; 4]; 240 * 160], false).unwrap();
    let original_r4 = machine.cpu.regs[4];
    machine.cpu.regs[4] = original_r4.wrapping_add(99);
    let changed = machine.save_state().unwrap();
    slots.load(&mut machine, 1).unwrap();
    assert_eq!(machine.cpu.regs[4], original_r4);
    verify(&mut machine);
    slots.undo(&mut machine).unwrap();
    assert_eq!(machine.save_state().unwrap(), changed);
    verify(&mut machine);
    let length = machine.bus.save_data().unwrap().len();
    fs::write(&save, vec![7; length]).unwrap();
    let backups = crate::save_backups::Backups::new(&save);
    backups.capture_startup().unwrap();
    backups.restore(backups.choices(length)[0].generation.unwrap(), &mut machine, &boot).unwrap();
    assert_eq!(machine.bus.save_data().unwrap(), vec![7; length]);
    verify(&mut machine);
    machine.bus.load_save_data(&vec![9; length]);
    crate::save_backups::restart(&mut machine, &boot, &save).unwrap();
    assert_eq!(machine.bus.save_data().unwrap(), vec![9; length]);
    assert_eq!(fs::read(&save).unwrap(), vec![9; length]);
    assert!(std::sync::Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
    verify(&mut machine);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[136,135], "copied_files_unavailable":true,
        "transactional_failed_reattach":true, "isolated_slots_load_undo_backup_restart":true,
        "owner_retained":true, "fixed_transfer_bytes_each":48608, "obj_bank_spans":24,
        "source_rom_poisoned":true, "frontend_ui":"pending", "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "exact EUR ROM and E139 dataset; isolated copies under Windows Wine"]
fn european_context_e139_sources_survive_slot_undo_backup_restart() {
    let root = PathBuf::from(std::env::var_os("FFTA_CONTEXT_ORDINARY_ROOT").unwrap());
    let original = PathBuf::from(std::env::var_os("FFTA_CONTEXT_ORDINARY_DATASET").unwrap());
    let temporary = tempfile::Builder::new().prefix("context-e139-lifecycle-").tempdir_in(&root).unwrap();
    let dataset = temporary.path().join("dataset");
    fs::create_dir(&dataset).unwrap();
    for entry in fs::read_dir(&original).unwrap() {
        let entry = entry.unwrap();
        assert!(entry.file_type().unwrap().is_file());
        fs::copy(entry.path(), dataset.join(entry.file_name())).unwrap();
    }
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision, manifest.decoder_revision), (139, 138));
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let mut machine = crate::make_machine(rom.clone(), None);
    let boot = machine.save_state().unwrap();
    attach_from_directory(&mut machine, &dataset).unwrap();
    let owner = machine.bus.rom_resources().unwrap().clone();
    let spans: [(u32, usize); 9] = [
        (0x003a_e12c, 8), (0x003a_e134, 8), (0x003a_e13c, 8),
        (0x003a_e144, 8), (0x003a_e14c, 8), (0x003a_e164, 8),
        (0x003a_e038, 100), (0x00b8_c318, 64), (0x003a_dfe8, 80),
    ];
    let verify = |machine: &mut gba_core::Machine| {
        assert!(std::sync::Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
        let before = owner.layered_obj_animations_stats().unwrap();
        for &(at, len) in &spans { machine.bus.rom[at as usize..at as usize + len].fill(0xcd); }
        for &(at, len) in &spans {
            for i in 0..len { assert_eq!(machine.bus.read8(0x0800_0000 + at + i as u32), rom[at as usize + i]); }
        }
        let after = owner.layered_obj_animations_stats().unwrap();
        assert_eq!(after.coordinate_bytes - before.coordinate_bytes, 48);
        assert_eq!(after.timeline_table_bytes - before.timeline_table_bytes, 100);
        assert_eq!(after.timeline_source_bytes - before.timeline_source_bytes, 64);
        assert_eq!(after.event_bytes - before.event_bytes, 80);
        for &(at, len) in &spans { machine.bus.rom[at as usize..at as usize + len].copy_from_slice(&rom[at as usize..at as usize + len]); }
    };
    verify(&mut machine);
    fs::rename(&dataset, temporary.path().join("retained-files")).unwrap();
    let state = machine.save_state().unwrap();
    assert!(attach_from_directory(&mut machine, &dataset).is_err());
    assert!(machine.save_state().unwrap() == state);
    verify(&mut machine);
    let save = temporary.path().join("isolated.sav");
    let mut slots = crate::save_state::Slots::new(&save, &machine.bus.rom);
    slots.save(&machine, 1, &vec![[0; 4]; 240 * 160], false).unwrap();
    let original_r4 = machine.cpu.regs[4];
    machine.cpu.regs[4] = original_r4.wrapping_add(99);
    let changed = machine.save_state().unwrap();
    slots.load(&mut machine, 1).unwrap();
    assert_eq!(machine.cpu.regs[4], original_r4);
    verify(&mut machine);
    slots.undo(&mut machine).unwrap();
    assert!(machine.save_state().unwrap() == changed);
    verify(&mut machine);
    let length = machine.bus.save_data().unwrap().len();
    fs::write(&save, vec![7; length]).unwrap();
    let backups = crate::save_backups::Backups::new(&save);
    backups.capture_startup().unwrap();
    backups.restore(backups.choices(length)[0].generation.unwrap(), &mut machine, &boot).unwrap();
    assert_eq!(machine.bus.save_data().unwrap(), vec![7; length]);
    verify(&mut machine);
    machine.bus.load_save_data(&vec![9; length]);
    crate::save_backups::restart(&mut machine, &boot, &save).unwrap();
    assert_eq!(machine.bus.save_data().unwrap(), vec![9; length]);
    assert_eq!(fs::read(&save).unwrap(), vec![9; length]);
    verify(&mut machine);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[139,138], "copied_files_unavailable":true,
        "transactional_failed_reattach":true, "isolated_slots_load_undo_backup_restart":true,
        "owner_retained":true, "coordinate_bytes_each":48, "timeline_table_bytes_each":100,
        "timeline_source_bytes_each":64, "event_bytes_each":80, "source_rom_poisoned":true,
        "frontend_ui":"pending", "Linux":"pending", "USA":"pending", "native_windows":"pending",
        "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "exact EUR ROM and E141 dataset; isolated copies under Windows Wine"]
fn european_language_positions_survive_slot_undo_backup_restart() {
    use gba_core::rom_data::layered_obj_animations::{CONTEXT_LANGUAGE_POSITION_A, CONTEXT_LANGUAGE_POSITION_B};

    let root = PathBuf::from(std::env::var_os("FFTA_CONTEXT_ORDINARY_ROOT").unwrap());
    let original = PathBuf::from(std::env::var_os("FFTA_CONTEXT_ORDINARY_DATASET").unwrap());
    let temporary = tempfile::Builder::new().prefix("language-position-lifecycle-").tempdir_in(&root).unwrap();
    let dataset = temporary.path().join("dataset");
    fs::create_dir(&dataset).unwrap();
    for entry in fs::read_dir(&original).unwrap() {
        let entry = entry.unwrap();
        assert!(entry.file_type().unwrap().is_file());
        fs::copy(entry.path(), dataset.join(entry.file_name())).unwrap();
    }
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision, manifest.decoder_revision), (141, 140));
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let mut machine = crate::make_machine(rom.clone(), None);
    let boot = machine.save_state().unwrap();
    attach_from_directory(&mut machine, &dataset).unwrap();
    let owner = machine.bus.rom_resources().unwrap().clone();
    let verify = |machine: &mut Machine| {
        assert!(std::sync::Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
        let before = owner.layered_obj_animations_stats().unwrap();
        for at in CONTEXT_LANGUAGE_POSITION_A..CONTEXT_LANGUAGE_POSITION_B + 10 { machine.bus.rom[at as usize] = 0xcd; }
        for at in CONTEXT_LANGUAGE_POSITION_A..CONTEXT_LANGUAGE_POSITION_B + 10 {
            assert_eq!(machine.bus.read8(0x0800_0000 + at), rom[at as usize]);
        }
        let after = owner.layered_obj_animations_stats().unwrap();
        assert_eq!(after.language_position_a_bytes - before.language_position_a_bytes, 10);
        assert_eq!(after.language_position_b_bytes - before.language_position_b_bytes, 10);
        for at in CONTEXT_LANGUAGE_POSITION_A..CONTEXT_LANGUAGE_POSITION_B + 10 { machine.bus.rom[at as usize] = rom[at as usize]; }
    };
    verify(&mut machine);
    fs::rename(&dataset, temporary.path().join("retained-files")).unwrap();
    let state = machine.save_state().unwrap();
    assert!(attach_from_directory(&mut machine, &dataset).is_err());
    assert!(machine.save_state().unwrap() == state);
    verify(&mut machine);
    let save = temporary.path().join("isolated.sav");
    let mut slots = crate::save_state::Slots::new(&save, &machine.bus.rom);
    slots.save(&machine, 1, &vec![[0; 4]; 240 * 160], false).unwrap();
    let original_r4 = machine.cpu.regs[4];
    machine.cpu.regs[4] = machine.cpu.regs[4].wrapping_add(99);
    let changed = machine.save_state().unwrap();
    slots.load(&mut machine, 1).unwrap();
    assert_eq!(machine.cpu.regs[4], original_r4);
    verify(&mut machine);
    slots.undo(&mut machine).unwrap();
    assert!(machine.save_state().unwrap() == changed);
    verify(&mut machine);
    let length = machine.bus.save_data().unwrap().len();
    fs::write(&save, vec![7; length]).unwrap();
    let backups = crate::save_backups::Backups::new(&save);
    backups.capture_startup().unwrap();
    backups.restore(backups.choices(length)[0].generation.unwrap(), &mut machine, &boot).unwrap();
    assert_eq!(machine.bus.save_data().unwrap(), vec![7; length]);
    verify(&mut machine);
    machine.bus.load_save_data(&vec![9; length]);
    crate::save_backups::restart(&mut machine, &boot, &save).unwrap();
    assert_eq!(machine.bus.save_data().unwrap(), vec![9; length]);
    assert_eq!(fs::read(&save).unwrap(), vec![9; length]);
    verify(&mut machine);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[141,140], "copied_files_unavailable":true,
        "transactional_failed_reattach":true, "isolated_slots_load_undo_backup_restart":true,
        "owner_retained":true, "language_position_bytes_each":[10,10], "source_rom_poisoned":true,
        "frontend_ui":"pending", "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "exact EUR ROM and E142 dataset; isolated copies under Windows Wine"]
fn european_activation_sources_survive_slot_undo_backup_restart() {
    use gba_core::rom_data::layered_obj_animations::{CONTEXT_ACTIVATION_GATES, CONTEXT_ACTIVATION_TICKS};

    let root = PathBuf::from(std::env::var_os("FFTA_CONTEXT_ORDINARY_ROOT").unwrap());
    let original = PathBuf::from(std::env::var_os("FFTA_CONTEXT_ORDINARY_DATASET").unwrap());
    let temporary = tempfile::Builder::new().prefix("activation-lifecycle-").tempdir_in(&root).unwrap();
    let dataset = temporary.path().join("dataset");
    fs::create_dir(&dataset).unwrap();
    for entry in fs::read_dir(&original).unwrap() {
        let entry = entry.unwrap();
        assert!(entry.file_type().unwrap().is_file());
        fs::copy(entry.path(), dataset.join(entry.file_name())).unwrap();
    }
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision, manifest.decoder_revision), (142, 141));
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let mut machine = crate::make_machine(rom.clone(), None);
    let boot = machine.save_state().unwrap();
    attach_from_directory(&mut machine, &dataset).unwrap();
    let owner = machine.bus.rom_resources().unwrap().clone();
    let verify = |machine: &mut Machine| {
        assert!(std::sync::Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
        let before = owner.layered_obj_animations_stats().unwrap();
        for at in CONTEXT_ACTIVATION_GATES..CONTEXT_ACTIVATION_TICKS + 160 { machine.bus.rom[at as usize] = 0xcd; }
        for at in CONTEXT_ACTIVATION_GATES..CONTEXT_ACTIVATION_TICKS + 160 {
            assert_eq!(machine.bus.read8(0x0800_0000 + at), rom[at as usize]);
        }
        let after = owner.layered_obj_animations_stats().unwrap();
        assert_eq!(after.activation_gate_bytes - before.activation_gate_bytes, 40);
        assert_eq!(after.activation_tick_bytes - before.activation_tick_bytes, 160);
        for at in CONTEXT_ACTIVATION_GATES..CONTEXT_ACTIVATION_TICKS + 160 { machine.bus.rom[at as usize] = rom[at as usize]; }
    };
    verify(&mut machine);
    fs::rename(&dataset, temporary.path().join("retained-files")).unwrap();
    let state = machine.save_state().unwrap();
    assert!(attach_from_directory(&mut machine, &dataset).is_err());
    assert!(machine.save_state().unwrap() == state);
    verify(&mut machine);
    let save = temporary.path().join("isolated.sav");
    let mut slots = crate::save_state::Slots::new(&save, &machine.bus.rom);
    slots.save(&machine, 1, &vec![[0; 4]; 240 * 160], false).unwrap();
    let original_r4 = machine.cpu.regs[4];
    machine.cpu.regs[4] = original_r4.wrapping_add(99);
    let changed = machine.save_state().unwrap();
    slots.load(&mut machine, 1).unwrap();
    assert_eq!(machine.cpu.regs[4], original_r4);
    verify(&mut machine);
    slots.undo(&mut machine).unwrap();
    assert!(machine.save_state().unwrap() == changed);
    verify(&mut machine);
    let length = machine.bus.save_data().unwrap().len();
    fs::write(&save, vec![7; length]).unwrap();
    let backups = crate::save_backups::Backups::new(&save);
    backups.capture_startup().unwrap();
    backups.restore(backups.choices(length)[0].generation.unwrap(), &mut machine, &boot).unwrap();
    assert_eq!(machine.bus.save_data().unwrap(), vec![7; length]);
    verify(&mut machine);
    machine.bus.load_save_data(&vec![9; length]);
    crate::save_backups::restart(&mut machine, &boot, &save).unwrap();
    assert_eq!(machine.bus.save_data().unwrap(), vec![9; length]);
    assert_eq!(fs::read(&save).unwrap(), vec![9; length]);
    verify(&mut machine);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[142,141], "copied_files_unavailable":true,
        "transactional_failed_reattach":true, "isolated_slots_load_undo_backup_restart":true,
        "owner_retained":true, "activation_gate_bytes_each":40, "activation_tick_bytes_each":160,
        "source_rom_poisoned":true, "frontend_ui":"pending", "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "exact EUR ROM and E145 dataset; isolated copies under Windows Wine"]
fn european_context_e145_geometry_survives_slot_undo_backup_restart() {
    let root = PathBuf::from(std::env::var_os("FFTA_CONTEXT_ORDINARY_ROOT").unwrap());
    let original = PathBuf::from(std::env::var_os("FFTA_CONTEXT_ORDINARY_DATASET").unwrap());
    let temporary = tempfile::Builder::new().prefix("context-e145-geometry-lifecycle-").tempdir_in(&root).unwrap();
    let dataset = temporary.path().join("dataset");
    fs::create_dir(&dataset).unwrap();
    for entry in fs::read_dir(&original).unwrap() {
        let entry = entry.unwrap();
        assert!(entry.file_type().unwrap().is_file());
        fs::copy(entry.path(), dataset.join(entry.file_name())).unwrap();
    }
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision, manifest.decoder_revision), (145, 144));
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let mut machine = crate::make_machine(rom.clone(), None);
    let boot = machine.save_state().unwrap();
    attach_from_directory(&mut machine, &dataset).unwrap();
    let owner = machine.bus.rom_resources().unwrap().clone();
    let spans: [(u32, usize); 2] = [(0x003a_e154, 16), (0x003a_e16c, 56)];
    let verify = |machine: &mut gba_core::Machine| {
        assert!(std::sync::Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
        let before = owner.layered_obj_animations_stats().unwrap();
        for &(at, len) in &spans { machine.bus.rom[at as usize..at as usize + len].fill(0xcd); }
        for &(at, len) in &spans {
            for i in 0..len { assert_eq!(machine.bus.read8(0x0800_0000 + at + i as u32), rom[at as usize + i]); }
        }
        let after = owner.layered_obj_animations_stats().unwrap();
        assert_eq!(after.target_coordinate_bytes - before.target_coordinate_bytes, 8);
        assert_eq!(after.additional_geometry_bytes - before.additional_geometry_bytes, 64);
        for &(at, len) in &spans { machine.bus.rom[at as usize..at as usize + len].copy_from_slice(&rom[at as usize..at as usize + len]); }
    };
    verify(&mut machine);
    fs::rename(&dataset, temporary.path().join("retained-files")).unwrap();
    let state = machine.save_state().unwrap();
    assert!(attach_from_directory(&mut machine, &dataset).is_err());
    assert!(machine.save_state().unwrap() == state);
    verify(&mut machine);
    let save = temporary.path().join("isolated.sav");
    let mut slots = crate::save_state::Slots::new(&save, &machine.bus.rom);
    slots.save(&machine, 1, &vec![[0; 4]; 240 * 160], false).unwrap();
    let original_r4 = machine.cpu.regs[4];
    machine.cpu.regs[4] = original_r4.wrapping_add(99);
    let changed = machine.save_state().unwrap();
    slots.load(&mut machine, 1).unwrap();
    assert_eq!(machine.cpu.regs[4], original_r4);
    verify(&mut machine);
    slots.undo(&mut machine).unwrap();
    assert!(machine.save_state().unwrap() == changed);
    verify(&mut machine);
    let length = machine.bus.save_data().unwrap().len();
    fs::write(&save, vec![7; length]).unwrap();
    let backups = crate::save_backups::Backups::new(&save);
    backups.capture_startup().unwrap();
    backups.restore(backups.choices(length)[0].generation.unwrap(), &mut machine, &boot).unwrap();
    assert_eq!(machine.bus.save_data().unwrap(), vec![7; length]);
    verify(&mut machine);
    machine.bus.load_save_data(&vec![9; length]);
    crate::save_backups::restart(&mut machine, &boot, &save).unwrap();
    assert_eq!(machine.bus.save_data().unwrap(), vec![9; length]);
    assert_eq!(fs::read(&save).unwrap(), vec![9; length]);
    verify(&mut machine);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[145,144], "copied_files_unavailable":true,
        "transactional_failed_reattach":true, "isolated_slots_load_undo_backup_restart":true,
        "owner_retained":true, "target_coordinate_bytes_each":8, "additional_geometry_bytes_each":64, "source_rom_poisoned":true,
        "frontend_ui":"pending", "Linux":"pending", "USA":"pending", "native_windows":"pending",
        "performance_measurements":false
    })).unwrap()).unwrap();
}
