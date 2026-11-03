//! New alternate-record source proof; closed default-template acceptance is not rerun.
use super::default48_tests::{run, Case};
use super::*;
use gba_core::rom_data::{
    constructor_alternates::{
        Alternates, Catalog, Record, END_OFFSET, POINTER_OFFSET, SOURCE_OFFSET,
    },
    default48,
    items::Items,
};
use std::collections::BTreeSet;
pub(super) fn cases() -> Vec<Case> {
    let mut cases = vec![];
    for id in 20..24 {
        for (flag, selector, mode) in [
            (0, 0, 1),
            (1, 1, 2),
            (2, 2, 1),
            (3, 255, 2),
            (0x1ff, 0x101, 0x101),
            (0x103, 0x102, 0x102),
        ] {
            cases.push(Case {
                args: [0xffff0000 | id, 0x100ab, 0xffff00cd, 0x100ef],
                flag,
                selector,
                mode,
            });
        }
    }
    cases
}
#[test]
#[ignore = "private EUR constructor-alternate source/original prefix proof; reduced Wine only, Linux/USA pending"]
fn european_constructor_alternates_match_original_selected_copy_and_live_patches() {
    let root = PathBuf::from(
        std::env::var_os("FFTA_STATE_TEST_CONSTRUCTOR_ALTERNATES_ORIGINAL_ROOT").unwrap(),
    );
    assert!(!root.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let independent: Vec<Record> = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_CONSTRUCTOR_ALTERNATES_RECORDS").unwrap())
            .unwrap(),
    )
    .unwrap();
    let inv: serde_json::Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_CONSTRUCTOR_ALTERNATES_INVENTORY").unwrap())
            .unwrap(),
    )
    .unwrap();
    let owner: Items = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_ITEMS_SOURCE_FILE").unwrap()).unwrap(),
    )
    .unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    data.validate_items(&owner).unwrap();
    assert_eq!(data.records, independent);
    assert_eq!(
        data.source_bytes(),
        rom[SOURCE_OFFSET as usize..END_OFFSET as usize]
    );
    assert_eq!(
        data.pointer_bytes(),
        rom[POINTER_OFFSET as usize..POINTER_OFFSET as usize + 4]
    );
    assert_eq!(catalog.sources.iter().map(|s| s.length).sum::<u32>(), 196);
    assert_eq!(catalog.sources.len(), 2);
    for (i, s) in catalog.sources.iter().enumerate() {
        assert_eq!(inv["ranges"][i]["sha256"], s.sha256);
    }
    let bytes = serde_json::to_vec_pretty(data).unwrap();
    let parsed: Alternates = serde_json::from_slice(&bytes).unwrap();
    parsed.verify_source(&rom).unwrap();
    assert_eq!(serde_json::to_vec_pretty(&parsed).unwrap(), bytes);
    for kind in 0..6 {
        let mut bad = serde_json::to_value(data).unwrap();
        match kind {
            0 => bad["unknown"] = 0.into(),
            1 => bad["records"][0]["unknown"] = 0.into(),
            2 => bad["records"][0]["byte01"] = 256.into(),
            3 => bad["records"][0]["flags2a"] = 65536.into(),
            4 => {
                bad["records"][0]["halfwords08_28"]
                    .as_array_mut()
                    .unwrap()
                    .pop();
            }
            _ => bad["records"][0]["bytes02_04"][0] = (-1).into(),
        };
        assert!(serde_json::from_value::<Alternates>(bad).is_err());
    }
    for kind in 0..7 {
        let mut bad = data.clone();
        match kind {
            0 => bad.pointer_source_offset += 4,
            1 => bad.source_offset += 48,
            2 => {
                bad.records.pop();
            }
            3 => bad.records.push(bad.records[0].clone()),
            4 => bad.records[0].id = 19,
            5 => bad.records[0].source_offset += 48,
            _ => bad.records.swap(1, 2),
        };
        assert!(bad.validate().is_err());
    }
    for slot in 0..5 {
        let mut bad = data.clone();
        bad.records[3].halfwords08_28[slot] = owner.records.len() as u16;
        assert!(bad.validate_items(&owner).is_err());
    }
    for kind in 0..4 {
        let mut bad = data.clone();
        match kind {
            0 => bad.records[0].byte00 ^= 1,
            1 => bad.records[1].bytes02_04[1] ^= 1,
            2 => bad.records[2].flags2a ^= 1,
            _ => bad.records[3].halfwords2c_2e[1] ^= 1,
        };
        bad.validate_items(&owner).unwrap();
        assert!(bad.verify_source(&rom).is_err());
    }
    assert!(data.record(19).is_none());
    assert!(data.record(24).is_none());
    assert_eq!(data.record(0x10014).unwrap().id, 20);
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    let default = default48::Catalog::from_rom(&rom).unwrap().decoded;
    let mut m = crate::make_machine(rom, None);
    let mut visited = BTreeSet::new();
    let initialization = super::ui_palette_tests::initialize(&mut m, &mut visited);
    let ewram = m.bus.ewram.clone();
    let iwram = m.bus.iwram.clone();
    let cases = cases();
    let mut instructions = 0;
    for &case in &cases {
        instructions += run(&mut m, case, &default, &mut visited);
    }
    let mut synthetic = data.clone();
    for (i, r) in synthetic.records.iter_mut().enumerate() {
        r.byte00 = 0xf0 + i as u8;
        r.byte01 = 0x80 + i as u8;
        r.bytes02_04 = [0xfe, 0x80, 0xff];
        r.arguments05_07 = [1, 2, 3];
        r.halfwords08_28 = std::array::from_fn(|j| {
            if j < 5 {
                (owner.records.len() - 1 - j) as u16
            } else {
                0x8000 + j as u16 * 257
            }
        });
        r.flags2a = 0x0555;
        r.halfwords2c_2e = [0xdead, 0xbeef];
    }
    synthetic.validate_items(&owner).unwrap();
    assert!(synthetic.verify_source(&m.bus.rom).is_err());
    m.bus.rom[SOURCE_OFFSET as usize..END_OFFSET as usize]
        .copy_from_slice(&synthetic.source_bytes());
    let mut synthetic_cases = 0;
    for id in 20..24 {
        for flag in [1, 3] {
            let case = Case {
                args: [id, 0x100, 0x1ff, 0x180],
                flag,
                selector: if flag == 1 { 1 } else { 0x102 },
                mode: 2,
            };
            instructions += run(&mut m, case, &default, &mut visited);
            synthetic_cases += 1;
        }
    }
    m.bus.rom[SOURCE_OFFSET as usize..END_OFFSET as usize].copy_from_slice(&data.source_bytes());
    assert_eq!(m.bus.ewram, ewram);
    for (i, (&actual, &old)) in m.bus.iwram.iter().zip(&iwram).enumerate() {
        if !(0x7d00..0x7e20).contains(&i) {
            assert_eq!(actual, old, "IWRAM{i:x}");
        }
    }
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("constructor-alternates.json"), bytes).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"records":4,"pointer_bytes":4,"record_source_bytes":192,"source_bytes":196,"original_cases":cases.len(),"synthetic_cases":synthetic_cases,"instructions":instructions,"initialization_instructions":initialization,"distinct_instructions":visited.len(),"selected_ids20_23_u8truncation_original_pointer_index_copy_and_live_patches":true,"strict_json_fixed_array_ids_order_extent_source_parsed_equipment_owner_guards":true,"all20_equipment_references_have_parsed_item_owners":true,"synthetic_high_bits_unused_fields_valid_high_item_ids_flag_or_preserved":true,"scratch_frame_incoming_args_outer_ram_guards":true,"default271_plus30_native16_scene397":"closed/not repeated","allocation_live_unit_tail":"unclaimed, original code remains","managed_runtime":"pending; e29-d28 unchanged","linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
}
