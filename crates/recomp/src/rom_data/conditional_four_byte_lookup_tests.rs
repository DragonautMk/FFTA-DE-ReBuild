//! EUR source model and original bounded lookup prefixes, not a guest selector cap.
use super::*;
use gba_core::rom_data::conditional_four_byte_lookup::{Catalog, Lookup, SOURCE_OFFSET, RECORD_COUNT};

#[test]
#[ignore = "private EUR source and original reader check; Wine only, Linux/USA pending"]
fn european_conditional_lookup_model_matches_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_CONDITIONAL_LOOKUP_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let model = &catalog.decoded;
    model.verify_source(&rom).unwrap();
    assert_eq!(catalog.sources.len(), 1);
    assert_eq!((catalog.sources[0].offset, catalog.sources[0].length), (SOURCE_OFFSET, 296));
    assert_eq!(model.source_bytes().unwrap(), rom[SOURCE_OFFSET as usize..SOURCE_OFFSET as usize + 296]);
    for at in SOURCE_OFFSET..SOURCE_OFFSET + 296 {
        assert_eq!(model.byte_at(at), Some(rom[at as usize]));
    }
    assert_eq!(model.byte_at(SOURCE_OFFSET - 1), None);
    assert_eq!(model.byte_at(SOURCE_OFFSET + 296), None);
    assert_eq!(model.records.len(), RECORD_COUNT);
    let json = serde_json::to_vec(model).unwrap();
    assert_eq!(serde_json::from_slice::<Lookup>(&json).unwrap(), *model);
    let mut malformed = model.clone();
    malformed.records.pop();
    assert!(malformed.validate().is_err());
    malformed = model.clone();
    malformed.source_offset += 4;
    assert!(malformed.validate().is_err());
    malformed = model.clone();
    malformed.records[0].first ^= 1;
    assert!(malformed.validate().is_ok());
    assert!(malformed.verify_source(&rom).is_err());
    assert!(serde_json::from_str::<Lookup>("{\"source_offset\":7237052,\"records\":[],\"unexpected\":true}").is_err());
    for index in [0u32, 1, 73, 74, 255, u32::MAX] {
        let addr = Lookup::encoded_view(index);
        assert_eq!(addr, 0x086e6dbcu32.wrapping_add(index.wrapping_mul(4)));
        let pos = (addr & 0x01ff_ffff) as usize;
        let expected = [u16::from_le_bytes(rom[pos..pos + 2].try_into().unwrap()), u16::from_le_bytes(rom[pos + 2..pos + 4].try_into().unwrap())];
        if let Some(record) = model.select(index) {
            assert_eq!([record.first, record.second], expected);
        } else {
            assert!(index >= 74);
        }
        for field in 0..2 {
            let mut machine = crate::make_machine(rom.clone(), None);
            machine.bus.ffta_sprites.enabled = false;
            machine.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
            machine.cpu.regs[13] = 0x03007c00;
            machine.cpu.cpsr = 0xbf;
            if field == 0 {
                machine.cpu.regs[0] = 0x086e6dbc;
                machine.cpu.regs[5] = index;
                machine.cpu.regs[15] = 0x08080302;
            } else {
                machine.cpu.regs[4] = addr;
                machine.cpu.regs[15] = 0x08080316;
            }
            let end = if field == 0 { 0x0808030e } else { 0x0808031e };
            let ewram = machine.bus.ewram.clone();
            let iwram = machine.bus.iwram.clone();
            let mut steps = 0;
            while machine.cpu.regs[15] != end {
                assert!(steps < 8);
                machine.bus.frame_ready = false;
                machine.step();
                steps += 1;
            }
            assert_eq!(machine.cpu.regs[0], u32::from(expected[field]));
            assert_eq!(machine.bus.ewram, ewram);
            assert_eq!(machine.bus.iwram, iwram);
        }
        let mut machine = crate::make_machine(rom.clone(), None);
        machine.bus.ffta_sprites.enabled = false;
        machine.cpu.regs[0] = addr;
        machine.cpu.regs[15] = 0x0807710e;
        machine.cpu.cpsr = 0xbf;
        machine.step();
        assert_eq!(machine.cpu.regs[0], u32::from(rom[pos]));
    }
    // Original metadata52 selector 12, row77, returns the high observed index.
    let mut machine = crate::make_machine(rom, None);
    machine.bus.ffta_sprites.enabled = false;
    machine.cpu.regs[..3].copy_from_slice(&[77, 77, 12]);
    machine.cpu.regs[13] = 0x03007c00;
    machine.cpu.regs[14] = 0x03007f01;
    machine.cpu.regs[15] = 0x080cbec8;
    machine.cpu.cpsr = 0xbf;
    let mut steps = 0;
    while machine.cpu.regs[15] != 0x03007f00 {
        assert!(steps < 1000);
        machine.bus.frame_ready = false;
        machine.step();
        steps += 1;
    }
    assert_eq!(machine.cpu.regs[0], 73);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","records":74,"source_bytes":296,"original_indices":[0,1,73,74,255,4294967295u32],"producer_row77":73,"Linux":"pending","USA":"pending","performance_measurements":false})).unwrap()).unwrap();
}
