//! EUR branch-selected static OBJ lists and managed-reader equivalence.
use super::*;
use gba_core::{Bus, Machine};

const ROOTS: [(u32, usize); 3] = [(0x397e22, 14), (0x399724, 8), (0x3add94, 14)];
const STACK: u32 = 0x0300_7c00;
const RETURN: u32 = 0x0800_0300;
const BUFFER: u32 = 0x0203_0010;

fn draw(machine: &mut Machine, offset: u32, first: bool) {
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs[..4].copy_from_slice(&[
        0x0800_0000 + offset,
        100,
        70,
        if first { 0x3000_0400 } else { 7 },
    ]);
    machine.cpu.regs[13] = STACK;
    machine.cpu.regs[14] = RETURN | 1;
    machine.cpu.regs[15] = if first { 0x0800_1500 } else { 0x0800_170c };
    machine.cpu.cpsr = 0xbf;
    let args: &[u32] = if first {
        &[7, 2, 3, BUFFER, 128, 0]
    } else {
        &[2, 3, BUFFER, 128, 0]
    };
    for (i, value) in args.iter().enumerate() {
        machine.bus.write32(STACK + 4 * i as u32, *value);
    }
    for at in 0..128 * 8 {
        machine.bus.write8(BUFFER + at, 0x5a);
    }
    for _ in 0..20_000 {
        machine.bus.frame_ready = false;
        machine.step();
        if machine.cpu.regs[15] == RETURN {
            assert_eq!(machine.cpu.regs[13], STACK);
            return;
        }
    }
    panic!("original OBJ draw did not return for {offset:x}");
}

#[test]
#[ignore = "exact EUR ROM and E132 predecessor; Windows executable under Wine"]
fn european_branch_lists_preserve_prior_dataset_and_original_draws() {
    let root = PathBuf::from(std::env::var_os("FFTA_BRANCH_OBJ_ROOT").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let previous = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let previous_manifest: Manifest =
        toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((previous_manifest.extractor_revision, previous_manifest.decoder_revision), (132, 131));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision), (133, 132));
    assert_eq!(source.manifest.coverage, previous_manifest.coverage);
    let resources = root.join("e133-resources");
    let prepared = preparation::prepare(&resources, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&resources, &mut source, &mut |_| Ok(())).unwrap().action, "reused");

    for old in &previous_manifest.resources {
        let new = source.manifest.resources.iter().find(|r| r.id == old.id).unwrap();
        let old_bytes = fs::read(previous.join(&old.path)).unwrap();
        let new_bytes = fs::read(prepared.directory.join(&new.path)).unwrap();
        assert_eq!(gba_core::ffta_rom::digest(&old_bytes), old.sha256);
        if old.id != OBJ_STATIC_LISTS_ID {
            assert_eq!(new, old);
            assert_eq!(new_bytes, old_bytes);
            continue;
        }
        let old_lists = parse_obj_lists(&old_bytes).unwrap();
        let new_lists = parse_obj_lists(&new_bytes).unwrap();
        let mut prior_projection = new_lists.clone();
        prior_projection.lists.retain(|list| !ROOTS.iter().any(|(at, _)| *at == list.source_offset));
        assert_eq!(prior_projection, old_lists);
        let mut metadata = new.clone();
        metadata.sources.retain(|s| !ROOTS.iter().any(|(at, _)| *at == s.offset));
        metadata.sha256 = old.sha256.clone();
        assert_eq!(&metadata, old);
        for (offset, length) in ROOTS {
            let list = new_lists.lists.iter().find(|l| l.source_offset == offset).unwrap();
            assert_eq!(list.source_len(), length);
            assert_eq!(list.source_bytes().unwrap(), rom[offset as usize..offset as usize + length]);
        }
    }

    for (offset, length) in ROOTS {
        for first in [true, false] {
            let mut raw = crate::make_machine(rom.clone(), None);
            let mut decoded = crate::make_machine(rom.clone(), None);
            attach_from_directory(&mut decoded, &prepared.directory).unwrap();
            let owner = decoded.bus.rom_resources().unwrap().clone();
            decoded.bus.rom[offset as usize..offset as usize + length].fill(0xcd);
            for i in 0..length {
                assert_eq!(decoded.bus.read8(0x0800_0000 + offset + i as u32), rom[offset as usize + i]);
            }
            let before = owner.obj_list_stats().unwrap().source_bytes;
            draw(&mut raw, offset, first);
            draw(&mut decoded, offset, first);
            assert!(owner.obj_list_stats().unwrap().source_bytes > before);
            assert_eq!(owner.obj_list_source_used(offset), Some((true, false)));
            decoded.bus.rom[offset as usize..offset as usize + length]
                .copy_from_slice(&rom[offset as usize..offset as usize + length]);
            assert_eq!(raw.save_state().unwrap(), decoded.save_state().unwrap());
        }
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status": "PASS", "revision": [133, 132], "new_lists": 3,
        "new_bytes": 36, "original_draw_full_state_clock_cases": 6,
        "previous_resources_preserved": true, "Linux": "pending", "USA": "pending",
        "ordinary_scene": "pending", "native_windows": "pending", "performance_measurements": false
    })).unwrap()).unwrap();
}
