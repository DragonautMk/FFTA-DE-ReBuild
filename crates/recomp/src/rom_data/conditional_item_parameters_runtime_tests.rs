//! Conditional item-derived records: publication and original guest prefixes.
use super::*;
use gba_core::{capi::RT_API, rom_data::conditional_item_parameters::*, Machine};
use recomp_core::analyze::{Analysis, Block, View};
use std::{collections::BTreeSet, sync::Arc};

#[derive(Clone, Copy)]
struct Case {
    stage: u8,
    index: u16,
    item_id: usize,
}
fn setup(m: &mut Machine, c: Case) -> u32 {
    m.cpu.regs = [0; 16];
    m.cpu.regs[13] = 0x0300_7d00;
    m.cpu.cpsr = 0xbf;
    let addr = 0x0800_0000 + SOURCE_OFFSET + u32::from(c.index) * 4;
    let item = 0x086e_0bd0 + c.item_id as u32 * 32;
    match c.stage {
        0 => {
            m.bus.ewram[0x10e..0x110].copy_from_slice(&c.index.to_le_bytes());
            m.cpu.regs[0] = 0x0200_0100;
            m.cpu.regs[15] = 0x0801_618e;
            0x0801_6198
        }
        1 => {
            m.bus.ewram[0x10e..0x110].copy_from_slice(&c.index.to_le_bytes());
            m.cpu.regs[0] = 0x0200_0100;
            m.cpu.regs[15] = 0x0801_9efa;
            0x0801_9f04
        }
        2 => {
            m.cpu.regs[0] = c.item_id as u32;
            m.cpu.regs[10] = 0x086e_0bd0;
            m.cpu.regs[15] = 0x0806_fda2;
            0x0806_fdae
        }
        3 => {
            m.bus.ewram[0x180..0x184].copy_from_slice(&addr.to_le_bytes());
            m.cpu.regs[3] = 0x0200_0180;
            m.cpu.regs[15] = 0x0806_fdb8;
            0x0806_fdbc
        }
        4 => {
            m.cpu.regs[1] = item;
            m.cpu.regs[15] = 0x0807_00bc;
            0x0807_00cc
        }
        5 => {
            m.cpu.regs[0] = item;
            m.cpu.regs[15] = 0x0807_05ce;
            0x0807_05d8
        }
        _ => unreachable!(),
    }
}
fn run(m: &mut Machine, stop: u32) {
    let mut steps = 0;
    while m.cpu.regs[15] != stop {
        m.bus.frame_ready = false;
        m.step();
        steps += 1;
        assert!(steps <= 10);
    }
}

#[test]
#[ignore = "private exact EUR ROM and E132 dataset; Windows executable under Wine"]
fn european_conditional_item_parameters_survive_save_lifecycle() {
    use gba_core::Bus;

    let original = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let temporary = tempfile::tempdir().unwrap();
    let dataset = temporary.path().join("dataset");
    fs::create_dir(&dataset).unwrap();
    for entry in fs::read_dir(&original).unwrap() {
        let entry = entry.unwrap();
        assert!(entry.file_type().unwrap().is_file());
        fs::copy(entry.path(), dataset.join(entry.file_name())).unwrap();
    }
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let mut machine = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut machine, &dataset).unwrap();
    let owner = machine.bus.rom_resources().unwrap().clone();
    let start = SOURCE_OFFSET as usize;
    let expected = u32::from_le_bytes(rom[start..start + 4].try_into().unwrap());
    for byte in &mut machine.bus.rom[start..start + SOURCE_BYTES] {
        *byte ^= 0xff;
    }
    assert_eq!(machine.bus.read32(0x0800_0000 + SOURCE_OFFSET), expected);
    assert!(owner.conditional_item_parameters_stats().unwrap().source_bytes > 0);
    machine.bus.rom[start..start + SOURCE_BYTES]
        .copy_from_slice(&rom[start..start + SOURCE_BYTES]);
    let boot = machine.save_state().unwrap();

    fs::rename(&dataset, temporary.path().join("unavailable-dataset")).unwrap();
    let state = machine.save_state().unwrap();
    assert!(attach_from_directory(&mut machine, &dataset).is_err());
    assert_eq!(machine.save_state().unwrap(), state);
    assert!(Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
    let save = temporary.path().join("test.sav");
    let mut slots = crate::save_state::Slots::new(&save, &rom);
    slots.save(&machine, 1, &vec![[0; 4]; 240 * 160], false).unwrap();
    machine.cpu.regs[4] = 99;
    let changed = machine.save_state().unwrap();
    slots.load(&mut machine, 1).unwrap();
    assert_eq!(machine.cpu.regs[4], 0);
    slots.undo(&mut machine).unwrap();
    assert_eq!(machine.save_state().unwrap(), changed);
    let length = machine.bus.save_data().unwrap().len();
    fs::write(&save, vec![7; length]).unwrap();
    let backups = crate::save_backups::Backups::new(&save);
    backups.capture_startup().unwrap();
    backups.restore(backups.choices(length)[0].generation.unwrap(), &mut machine, &boot).unwrap();
    assert_eq!(machine.bus.save_data().unwrap(), vec![7; length]);
    machine.bus.load_save_data(&vec![9; length]);
    crate::save_backups::restart(&mut machine, &boot, &save).unwrap();
    assert_eq!(machine.bus.save_data().unwrap(), vec![9; length]);
    assert_eq!(fs::read(&save).unwrap(), vec![9; length]);
    assert!(Arc::ptr_eq(machine.bus.rom_resources().unwrap(), &owner));
    let before = owner.conditional_item_parameters_stats().unwrap();
    for byte in &mut machine.bus.rom[start..start + SOURCE_BYTES] {
        *byte ^= 0xff;
    }
    assert_eq!(machine.bus.read32(0x0800_0000 + SOURCE_OFFSET), expected);
    let after = owner.conditional_item_parameters_stats().unwrap();
    assert!(after.source_bytes >= before.source_bytes + 4);
    let report = PathBuf::from(std::env::var_os("FFTA_ITEM_PARAMETERS_REPORT").unwrap());
    fs::write(report, serde_json::to_vec_pretty(&serde_json::json!({
        "status": "PASS", "scope": "EUR E132 owner after unavailable files, slots, Undo, backup restore and restart",
        "poisoned_rom_typed_reads": true, "owner_retained": true,
        "Linux": "pending", "USA": "pending", "nativeWindows": "pending", "performance_measurements": false
    })).unwrap()).unwrap();
}
#[test]
#[ignore = "private EUR ROM, E131 dataset and TinyCC; Windows executable under Wine"]
fn european_conditional_item_parameters_publication_and_prefixes() {
    let root = PathBuf::from(std::env::var_os("FFTA_ITEM_PARAMETERS_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old_dir = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let old: Manifest =
        toml::from_str(&fs::read_to_string(old_dir.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (
            old.extractor_revision,
            old.decoder_revision,
            old.resources.len()
        ),
        (131, 130, 102)
    );
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision,
            source.manifest.resources.len()
        ),
        (132, 131, 103)
    );
    let resources = root.join("resources");
    let created = preparation::prepare(&resources, &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(created.action, "created");
    assert!(created
        .directory
        .parent()
        .unwrap()
        .ends_with("v1/e132-d131"));
    assert_eq!(
        preparation::prepare(&resources, &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    verify(&created.directory, &source).unwrap();
    assert_eq!(fs::read_dir(&created.directory).unwrap().count(), 104);
    assert_eq!(&source.manifest.coverage[..102], old.coverage.as_slice());
    for (current, previous) in source.manifest.resources[..102].iter().zip(&old.resources) {
        assert_eq!(current, previous);
        assert_eq!(
            fs::read(created.directory.join(&current.path)).unwrap(),
            fs::read(old_dir.join(&previous.path)).unwrap()
        );
    }
    assert_eq!(
        source.manifest.resources[102].dependencies,
        vec![ITEM_RECORDS_ID]
    );
    let model = &source.conditional_item_parameters.as_ref().unwrap().decoded;
    let items = &source.items.as_ref().unwrap().decoded;
    model.validate_items(items).unwrap();
    let mut cases = Vec::new();
    for index in 0..31u16 {
        let item_id = items
            .records
            .iter()
            .position(|r| u16::from(r.bytes_18_1f[0]) == index)
            .unwrap();
        for stage in 0..6 {
            cases.push(Case {
                stage,
                index,
                item_id,
            });
        }
    }
    for index in [31, 55, 256, u16::MAX] {
        for stage in 0..2 {
            cases.push(Case {
                stage,
                index,
                item_id: 0,
            });
        }
    }
    let mut trace = crate::make_machine(rom.clone(), None);
    let initial = trace.save_state().unwrap();
    let mut visited = BTreeSet::new();
    for &case in &cases {
        trace.load_state(&initial).unwrap();
        let stop = setup(&mut trace, case);
        while trace.cpu.regs[15] != stop {
            visited.insert((trace.cpu.regs[15], trace.cpu.thumb()));
            trace.step();
            assert!(visited.len() < 64);
        }
    }
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: Some(&trace.bus.iwram),
        bios: None,
    };
    let blocks = visited
        .iter()
        .map(|&(addr, thumb)| {
            let first = if thumb {
                armv4t::decode_thumb(view.read16(addr), addr)
            } else {
                armv4t::decode_arm(view.read32(addr), addr)
            };
            Block {
                addr,
                thumb,
                instrs: vec![first],
            }
        })
        .collect();
    let object = root.join("item-parameters-native.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().expect("TinyCC required"),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &created.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let start = SOURCE_OFFSET as usize;
    let end = start + SOURCE_BYTES;
    for &case in &cases {
        for m in [&mut raw, &mut managed] {
            m.load_state(&initial).unwrap();
            setup(m, case);
        }
        managed.bus.rom[start..end].fill(0xcd);
        let item_at = 0x6e0bd0 + case.item_id * 32 + 24;
        if case.stage >= 2 {
            managed.bus.rom[item_at] ^= 0x5a;
        }
        let outside = start + usize::from(case.index) * 4 + usize::from(case.stage == 1);
        let poisoned_outside = case.index == 31 || case.index == 256;
        if poisoned_outside {
            raw.bus.rom[outside] ^= 0x5a;
            managed.bus.rom[outside] ^= 0x5a;
        }
        let before = owner
            .conditional_item_parameters_stats()
            .unwrap()
            .source_bytes;
        let stop = match case.stage {
            0 => 0x0801_6198,
            1 => 0x0801_9f04,
            2 => 0x0806_fdae,
            3 => 0x0806_fdbc,
            4 => 0x0807_00cc,
            _ => 0x0807_05d8,
        };
        run(&mut raw, stop);
        let mut steps = 0;
        while managed.cpu.regs[15] != stop {
            managed.bus.frame_ready = false;
            table
                .get(managed.cpu.regs[15] | u32::from(managed.cpu.thumb()))
                .unwrap()(
                &RT_API,
                &mut managed as *mut Machine as *mut std::ffi::c_void,
            );
            steps += 1;
            assert!(steps <= 10);
        }
        let used = owner
            .conditional_item_parameters_stats()
            .unwrap()
            .source_bytes
            - before;
        if case.index >= 31 || case.stage == 2 {
            assert_eq!(used, 0);
        } else {
            assert!(used > 0);
        }
        managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
        if case.stage >= 2 {
            managed.bus.rom[item_at] = rom[item_at];
        }
        if poisoned_outside {
            raw.bus.rom[outside] = rom[outside];
            managed.bus.rom[outside] = rom[outside];
        }
        assert_eq!(
            raw.save_state().unwrap(),
            managed.save_state().unwrap(),
            "stage {} index {}",
            case.stage,
            case.index
        );
    }
    let stats = owner.conditional_item_parameters_stats().unwrap();
    assert_eq!(stats.record_mask, (1u64 << 31) - 1);
    assert!(stats.byte00 > 0 && stats.byte01 > 0 && stats.value02_bytes > 0);
    let before = managed.save_state().unwrap();
    let original = fs::read(created.directory.join(CONDITIONAL_ITEM_PARAMETERS_FILE)).unwrap();
    let manifest = fs::read(created.directory.join(MANIFEST_FILE)).unwrap();
    let mut changed = model.clone();
    changed.records[0].byte00 ^= 1;
    let bad = encode_conditional_item_parameters(&changed).unwrap();
    let mut forged = source.manifest.clone();
    forged.resources[102].sha256 = gba_core::ffta_rom::digest(&bad);
    fs::write(
        created.directory.join(CONDITIONAL_ITEM_PARAMETERS_FILE),
        bad,
    )
    .unwrap();
    fs::write(
        created.directory.join(MANIFEST_FILE),
        toml::to_string_pretty(&forged).unwrap(),
    )
    .unwrap();
    assert!(attach_from_directory(&mut managed, &created.directory).is_err());
    assert!(Arc::ptr_eq(managed.bus.rom_resources().unwrap(), &owner));
    assert_eq!(managed.save_state().unwrap(), before);
    fs::write(
        created.directory.join(CONDITIONAL_ITEM_PARAMETERS_FILE),
        original,
    )
    .unwrap();
    fs::write(created.directory.join(MANIFEST_FILE), manifest).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","revision":[132,131],"resources":103,"previous_unchanged":102,"prefix_cases":cases.len(),"raw_and_translated_managed_full_state":true,"records_mask":stats.record_mask,"outside_raw_indices":[31,256],"outside_prior_owner_indices":[55,65535],"coherent_tamper_transaction":true,"ordinary_producer":"pending","Linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false})).unwrap()).unwrap();
}
