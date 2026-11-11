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
