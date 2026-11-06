//! Declared original prefixes only; this does not execute the surrounding UI.
use super::*;
use crate::Machine;
use std::{fs, path::PathBuf};

fn begin(machine: &mut Machine, pc: u32) {
    machine.cpu.regs = [0; 16];
    machine.cpu.regs[13] = 0x0300_7d00;
    machine.cpu.cpsr = 0xbf;
    machine.cpu.regs[15] = pc;
}
fn run(machine: &mut Machine, stop: u32) {
    let mut steps = 0;
    while machine.cpu.regs[15] != stop {
        machine.bus.frame_ready = false;
        machine.step();
        steps += 1;
        assert!(steps <= 10, "unexpected PC {:08x}", machine.cpu.regs[15]);
    }
}

#[test]
#[ignore = "private EUR ROM; original reader prefixes on Wine"]
fn european_item_parameters_match_original_reader_prefixes() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let source = Catalog::from_rom(&rom).unwrap();
    let model = &source.decoded;
    let items = items::Catalog::from_rom(&rom).unwrap().decoded;
    assert_eq!(
        model.source_bytes().unwrap(),
        rom[SOURCE_OFFSET as usize..SOURCE_OFFSET as usize + SOURCE_BYTES]
    );
    assert_eq!(source.sources.len(), 1);
    assert_eq!(source.sources[0].sha256, SOURCE_SHA256);
    let serialized = serde_json::to_vec(model).unwrap();
    assert_eq!(
        serde_json::from_slice::<ConditionalItemParameters>(&serialized).unwrap(),
        *model
    );
    let mut bad = model.clone();
    bad.records.pop();
    assert!(bad.validate().is_err());
    bad = model.clone();
    bad.records[0].byte00 ^= 1;
    assert!(bad.validate().is_ok() && bad.verify_source(&rom).is_err());
    let mut bad_items = items.clone();
    bad_items.records[0].bytes_18_1f[0] = 31;
    assert!(model.validate_items(&bad_items).is_err());
    let mut unknown: serde_json::Value = serde_json::from_slice(&serialized).unwrap();
    unknown["unknown"] = serde_json::json!(true);
    assert!(serde_json::from_value::<ConditionalItemParameters>(unknown).is_err());
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    assert!(
        model.record(31).is_none() && model.byte_at(SOURCE_OFFSET + SOURCE_BYTES as u32).is_none()
    );

    let mut machine = Machine::new(rom.clone());
    let mut prefixes = 0;
    for index in (0u16..31).chain([256, u16::MAX]) {
        let address = SOURCE_OFFSET + u32::from(index) * 4;
        machine.bus.ewram[0x10e..0x110].copy_from_slice(&index.to_le_bytes());
        begin(&mut machine, 0x0801_618e);
        machine.cpu.regs[0] = 0x0200_0100;
        run(&mut machine, 0x0801_6198);
        assert_eq!(machine.cpu.regs[4], 0x0800_0000 + address);
        assert_eq!(machine.cpu.regs[1], u32::from(rom[address as usize]));
        prefixes += 1;
        begin(&mut machine, 0x0801_9efa);
        machine.cpu.regs[0] = 0x0200_0100;
        run(&mut machine, 0x0801_9f04);
        assert_eq!(machine.cpu.regs[0], u32::from(rom[address as usize + 1]));
        prefixes += 1;
        if index >= 31 {
            continue;
        }
        let (item_id, _) = items
            .records
            .iter()
            .enumerate()
            .find(|(_, item)| u16::from(item.bytes_18_1f[0]) == index)
            .unwrap();
        begin(&mut machine, 0x0806_fda2);
        machine.cpu.regs[0] = item_id as u32;
        machine.cpu.regs[10] = 0x086e_0bd0;
        run(&mut machine, 0x0806_fdae);
        assert_eq!(machine.cpu.regs[1], 0x0800_0000 + address);
        prefixes += 1;
        machine.bus.ewram[0x180..0x184].copy_from_slice(&(0x0800_0000 + address).to_le_bytes());
        begin(&mut machine, 0x0806_fdb8);
        machine.cpu.regs[3] = 0x0200_0180;
        run(&mut machine, 0x0806_fdbc);
        assert_eq!(
            machine.cpu.regs[2],
            u32::from(model.record(u32::from(index)).unwrap().byte01)
        );
        prefixes += 1;
        let item_address = 0x086e_0bd0 + item_id as u32 * 32;
        begin(&mut machine, 0x0807_00bc);
        machine.cpu.regs[1] = item_address;
        run(&mut machine, 0x0807_00cc);
        assert_eq!(
            machine.cpu.regs[2],
            u32::from(model.record(u32::from(index)).unwrap().value02)
        );
        prefixes += 1;
        begin(&mut machine, 0x0807_05ce);
        machine.cpu.regs[0] = item_address;
        run(&mut machine, 0x0807_05d8);
        assert_eq!(
            machine.cpu.regs[1],
            u32::from(model.record(u32::from(index)).unwrap().value02)
        );
        prefixes += 1;
    }
    let report = PathBuf::from(std::env::var_os("FFTA_ITEM_PARAMETERS_REPORT").unwrap());
    fs::write(report, serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","scope":"conditional item-derived EUR records and original reader prefixes; no full caller ABI","records":31,"source_bytes":124,"item_records":376,"original_prefixes":prefixes,"full_u16_outside_cases":[256,65535],"Linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false})).unwrap()).unwrap();
}
