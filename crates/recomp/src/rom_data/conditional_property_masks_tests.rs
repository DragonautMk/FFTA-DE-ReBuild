//! EUR conditional mask model and original bounded selector/reader prefixes.
use super::*;
use gba_core::rom_data::conditional_property_masks::{Catalog, Masks, RECORD_COUNT, SOURCE_OFFSET};

#[test]
#[ignore = "private EUR source and original reader check; Wine only, Linux/USA pending"]
fn european_conditional_property_masks_match_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_PROPERTY_MASKS_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let model = &catalog.decoded;
    model.verify_source(&rom).unwrap();
    assert_eq!(catalog.sources.len(), 1);
    assert_eq!((catalog.sources[0].offset, catalog.sources[0].length), (SOURCE_OFFSET, 140));
    assert_eq!(model.source_bytes().unwrap(), rom[SOURCE_OFFSET as usize..SOURCE_OFFSET as usize + 140]);
    assert_eq!(model.words.len(), RECORD_COUNT);
    for at in SOURCE_OFFSET..SOURCE_OFFSET + 140 {
        assert_eq!(model.byte_at(at), Some(rom[at as usize]));
    }
    assert_eq!(model.byte_at(SOURCE_OFFSET - 1), None);
    assert_eq!(model.byte_at(SOURCE_OFFSET + 140), None);
    let json = serde_json::to_vec(model).unwrap();
    assert_eq!(serde_json::from_slice::<Masks>(&json).unwrap(), *model);
    let mut invalid = model.clone(); invalid.words.pop(); assert!(invalid.validate().is_err());
    invalid = model.clone(); invalid.source_offset += 4; assert!(invalid.validate().is_err());
    invalid = model.clone(); invalid.words[0] ^= 1; assert!(invalid.validate().is_ok()); assert!(invalid.verify_source(&rom).is_err());
    assert!(serde_json::from_str::<Masks>("{\"source_offset\":0,\"words\":[],\"unexpected\":true}").is_err());
    let mut machine = crate::make_machine(rom.clone(), None);
    machine.bus.ffta_sprites.enabled = false;
    for index in [0u32, 1, 34, 35, 255, u32::MAX] {
        let address = Masks::encoded_view(index);
        let physical = (address & 0x01ff_ffff) as usize;
        let expected = u32::from_le_bytes(rom[physical..physical + 4].try_into().unwrap());
        if let Some(value) = model.select(index) { assert_eq!(value, expected); } else { assert!(index >= 35); }
        machine.cpu.regs = [0; 16];
        machine.cpu.regs[0] = index;
        machine.cpu.regs[5] = 0x086e0b44;
        machine.cpu.regs[13] = 0x03007c00;
        machine.cpu.regs[15] = 0x080ceea6;
        machine.cpu.cpsr = 0xbf;
        let mut steps = 0;
        while machine.cpu.regs[15] != 0x080ceeac {
            assert!(steps < 4);
            machine.bus.frame_ready = false;
            machine.step(); steps += 1;
        }
        assert_eq!(machine.cpu.regs[5], expected, "original index {index}");
    }
    for (row, expected) in [(0u32, 0u32), (2, 1), (41, 34)] {
        machine.cpu.regs = [0; 16];
        machine.cpu.regs[..3].copy_from_slice(&[row, row, 0x24]);
        machine.cpu.regs[13] = 0x03007c00;
        machine.cpu.regs[14] = 0x03007f01;
        machine.cpu.regs[15] = 0x080cbec8;
        machine.cpu.cpsr = 0xbf;
        let mut steps = 0;
        while machine.cpu.regs[15] != 0x03007f00 {
            assert!(steps < 1000);
            machine.bus.frame_ready = false;
            machine.step(); steps += 1;
        }
        assert_eq!(machine.cpu.regs[0], expected, "original metadata row {row}");
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "words":35, "source_bytes":140,
        "original_indices":[0,1,34,35,255,4294967295u32], "producer_rows":[0,2,41],
        "Linux":"pending", "USA":"pending", "nativeWindows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
