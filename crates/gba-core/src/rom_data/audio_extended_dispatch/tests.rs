//! Original dispatcher prefix only: no indirect callbacks are executed.
use super::*;
use crate::Machine;
use std::{fs, path::PathBuf};

#[test]
#[ignore = "private EUR source ROM, original dispatcher prefix; Wine only"]
fn european_extended_dispatch_fragment_matches_original_prefix() {
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let model = &catalog.decoded;
    let spans = model.source_spans().unwrap();
    assert_eq!(spans.iter().map(|s| s.1.len()).sum::<usize>(), SOURCE_BYTES);
    assert_eq!(
        catalog
            .sources
            .iter()
            .map(|s| (s.offset, s.length))
            .collect::<Vec<_>>(),
        vec![(TABLE_OFFSET, 32), (TABLE_OFFSET + 40, 8)]
    );
    for (offset, bytes, _) in &spans {
        assert_eq!(
            *bytes,
            rom[*offset as usize..*offset as usize + bytes.len()]
        );
    }
    assert_eq!(model.callback(0), model.callback(3));
    for selector in [8, 9, 12, 255] {
        assert_eq!(model.callback(selector), None);
        assert_eq!(model.byte_at(TABLE_OFFSET + u32::from(selector) * 4), None);
    }
    let json = serde_json::to_vec(model).unwrap();
    assert_eq!(
        serde_json::from_slice::<AudioExtendedDispatch>(&json).unwrap(),
        *model
    );
    let mut invalid = model.clone();
    invalid.entries.swap(0, 1);
    assert!(invalid.validate().is_err());
    invalid = model.clone();
    invalid.entries[0].callback &= !1;
    assert!(invalid.validate().is_err());
    invalid = model.clone();
    invalid.entries[0].callback ^= 2;
    assert!(invalid.validate().is_ok());
    assert!(invalid.verify_source(&rom).is_err());
    let mut json_value: serde_json::Value = serde_json::from_slice(&json).unwrap();
    json_value["unknown"] = serde_json::json!(1);
    assert!(serde_json::from_value::<AudioExtendedDispatch>(json_value).is_err());
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());

    let mut machine = Machine::new(rom.clone());
    let mut cases = 0;
    // The additional four cases demonstrate that this fragment does not limit
    // the original guest's u8 index, and that slots 8/9 remain outside it.
    for selector in SELECTORS.into_iter().chain([8, 9, 12, 255]) {
        machine.bus.ewram[0x140..0x144].copy_from_slice(&0x0200_0300u32.to_le_bytes());
        machine.bus.ewram[0x300] = selector;
        machine.cpu.regs = [0; 16];
        machine.cpu.regs[1] = 0x0200_0100;
        machine.cpu.regs[13] = 0x0300_7d00;
        machine.cpu.cpsr = 0xbf;
        machine.cpu.regs[15] = 0x0814_9e8a;
        let mut steps = 0;
        while machine.cpu.regs[15] != 0x0814_9e9a {
            machine.step();
            steps += 1;
            assert!(
                steps <= 8,
                "selector {selector} pc {:08x}",
                machine.cpu.regs[15]
            );
        }
        assert_eq!(steps, 8);
        let at = TABLE_OFFSET + u32::from(selector) * 4;
        let raw = u32::from_le_bytes(rom[at as usize..at as usize + 4].try_into().unwrap());
        assert_eq!(machine.cpu.regs[3], BASE + at);
        assert_eq!(machine.cpu.regs[2], raw);
        assert_eq!(
            u32::from_le_bytes(machine.bus.ewram[0x140..0x144].try_into().unwrap()),
            0x0200_0301
        );
        if let Some(callback) = model.callback(selector) {
            assert_eq!(callback, raw);
        }
        cases += 1;
    }
    let out = PathBuf::from(std::env::var_os("FFTA_AUDIO_EXTENDED_REPORT").unwrap());
    fs::write(out, serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"conditional extra M4A dispatch words and original interpreter prefix; no callbacks executed",
        "source_bytes":SOURCE_BYTES, "selectors":SELECTORS, "prefix_cases":cases,
        "excluded_existing_selectors":[8,9], "outside_projection_cases":[12,255],
        "linux":"pending", "usa":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
