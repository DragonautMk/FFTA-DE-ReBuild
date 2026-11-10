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
