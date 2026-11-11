//! EUR fixed OBJ transfer publication and original queue/DMA consumption.
use super::*;
use gba_core::{Bus, Machine, capi::RT_API};

const SOURCE: u32 = 0x00b8_0b5c;
const LENGTH: usize = 0x8000;
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
#[ignore = "exact EUR ROM and E134 predecessor; Windows executable under Wine"]
fn european_fixed_obj_transfer_preserves_dataset_and_original_queue() {
    let root = PathBuf::from(std::env::var_os("FFTA_FIXED_OBJ_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_FIXED_OBJ_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old_manifest: Manifest =
        toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (
            old_manifest.extractor_revision,
            old_manifest.decoder_revision
        ),
        (134, 133)
    );
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (135, 134)
    );
    assert_eq!(source.manifest.coverage, old_manifest.coverage);
    assert_eq!(
        source.manifest.resources.len(),
        old_manifest.resources.len()
    );
    let resources = root.join("e135-resources");
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
        assert!(prior.fixed_banks.is_empty());
        assert_eq!(current.fixed_banks.len(), 1);
        assert_eq!(current.fixed_banks[0].bank_offset, SOURCE);
        assert_eq!(
            current.fixed_banks[0].source_bytes(),
            rom[SOURCE as usize..SOURCE as usize + LENGTH]
        );
        current.fixed_banks.clear();
        assert_eq!(current, prior);
        let mut metadata = new.clone();
        metadata.sources.retain(|s| s.role != "fixed-obj-transfer");
        metadata.sha256 = old.sha256.clone();
        assert_eq!(&metadata, old);
    }
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut decoded = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut decoded, &prepared.directory).unwrap();
    let owner = decoded.bus.rom_resources().unwrap().clone();
    let initial = raw.save_state().unwrap();
    decoded.bus.rom[SOURCE as usize..SOURCE as usize + LENGTH].fill(0xcd);
    let before = owner.ui_raw_transfers_stats().unwrap();
    for i in 0..LENGTH {
        assert_eq!(
            decoded.bus.read8(0x0800_0000 + SOURCE + i as u32),
            rom[SOURCE as usize + i]
        );
    }
    let after = owner.ui_raw_transfers_stats().unwrap();
    assert_eq!(after.payload_bytes - before.payload_bytes, LENGTH as u64);
    assert_eq!(
        (after.selector_bytes, after.banks_used, after.groups_used),
        (before.selector_bytes, before.banks_used, before.groups_used)
    );
    let pointer = &mut decoded as *mut Machine as *mut std::ffi::c_void;
    for edge in [SOURCE, SOURCE + LENGTH as u32] {
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
    decoded.bus.rom[SOURCE as usize..SOURCE as usize + LENGTH]
        .copy_from_slice(&rom[SOURCE as usize..SOURCE as usize + LENGTH]);
    raw.load_state(&initial).unwrap();
    decoded.load_state(&initial).unwrap();
    decoded.bus.rom[SOURCE as usize..SOURCE as usize + LENGTH].fill(0xcd);
    let before_dma = owner.ui_raw_transfers_stats().unwrap();
    for machine in [&mut raw, &mut decoded] {
        machine.bus.ffta_sprites.enabled = false;
        machine.bus.iwram.fill(0);
        machine.bus.write32(0x0300_2bcc, 0);
        machine.cpu.regs[..4].copy_from_slice(&[NODE, 0x0800_0000 + SOURCE, 0x0601_0000, 0x4000]);
        original(machine, 0x0800_0ad0);
        assert_eq!(machine.bus.read32(NODE + 4), 0x0800_0000 + SOURCE);
        assert_eq!(machine.bus.read32(NODE + 8), 0x0601_0000);
        assert_eq!(machine.bus.read32(NODE + 12), 0x8000_4000);
        original(machine, 0x0800_0800);
    }
    let after_dma = owner.ui_raw_transfers_stats().unwrap();
    assert!(after_dma.payload_bytes - before_dma.payload_bytes >= LENGTH as u64);
    assert_eq!(
        &decoded.bus.vram[0x10000..0x18000],
        &rom[SOURCE as usize..SOURCE as usize + LENGTH]
    );
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
    decoded.bus.rom[SOURCE as usize..SOURCE as usize + LENGTH]
        .copy_from_slice(&rom[SOURCE as usize..SOURCE as usize + LENGTH]);
    assert_eq!(
        raw.save_state().unwrap(),
        decoded.save_state().unwrap(),
        "full original queue/DMA state and clock"
    );
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS", "revision":[135,134], "fixed_source_bytes":LENGTH, "prior_resources_preserved":true, "original_queue_and_dma_full_state_clock_cases":1, "localized_stats_unchanged":true, "ordinary_scene":"pending", "translated_native":"pending", "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false})).unwrap()).unwrap();
}
