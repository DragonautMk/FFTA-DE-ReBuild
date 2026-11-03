//! Five newly rooted EUR language-selected OBJ banks, no old-bank oracle repeat.
use super::obj_animation_tests::{invoke, tick};
use super::*;
use gba_core::{
    rom_data::{
        obj_animations::{Catalog, SELECTOR_REFERENCES},
        obj_lists::List,
        ObjDimensions,
    },
    Bus,
};
const TASK: u32 = 0x02028010;
const STACK: u32 = 0x03007c00;
fn output(list: &List, dimensions: &ObjDimensions, mode: u8) -> (Vec<u8>, u32) {
    let mut out = vec![0x5a; 1024];
    let mut count = 0;
    for object in &list.objects {
        let d = dimensions.get(object.shape, object.size).unwrap();
        let x = (100 + i32::from(object.x)) as i16 as i32;
        let y = (70 + i32::from(object.y)) as i16 as i32;
        if x < 0 && x + i32::from(d.width) < 0
            || x > 239 && x + i32::from(d.width) > 239
            || y < 0 && y + i32::from(d.height) < 0
            || y > 159 && y + i32::from(d.height) > 159
        {
            continue;
        }
        let [a0, a1, _] = object.attributes();
        let attrs = [
            (a0 & 0xff00) | (y as u16 & 255),
            (a1 & !511) | (x as u16 & 511),
            (object.tile + 292) % 1024
                | (u16::from((object.priority + 10) % 4) << 10)
                | (u16::from(object.palette) << 12),
        ];
        assert!(count < if mode == 1 { 48 } else { 128 });
        for (i, a) in attrs.into_iter().enumerate() {
            out[count * 8 + i * 2..count * 8 + i * 2 + 2].copy_from_slice(&a.to_le_bytes());
        }
        count += 1;
    }
    (out, count as u32)
}
#[test]
#[ignore = "exact EUR ROM/independent new-bank inventory; reduced Wine, Linux/USA pending"]
fn european_localized_obj_tables_sources_and_original_producers() {
    let root =
        PathBuf::from(std::env::var_os("FFTA_STATE_TEST_OBJ_LOCALIZED_ORIGINAL_ROOT").unwrap());
    assert!(!root.exists());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let banks = &catalog.decoded;
    assert_eq!((banks.banks.len(), catalog.sources.len()), (16, 456));
    assert_eq!(catalog.sources.iter().map(|s| s.length).sum::<u32>(), 9054);
    assert_eq!(gba_core::rom_data::obj_animations::REFERENCES.len(), 33);
    assert_eq!(SELECTOR_REFERENCES.len(), 3);
    assert_eq!(banks.selector_tables.len(), 1);
    let tables = &banks.selector_tables[0];
    assert_eq!(
        (tables.object_roots.len(), tables.sequence_roots.len()),
        (5, 5)
    );
    assert_eq!(tables.select(5), None);
    let bytes = encode_obj_animations(banks).unwrap();
    let parsed = parse_obj_animations(&bytes).unwrap();
    parsed.verify_source(&rom).unwrap();
    assert_eq!(encode_obj_animations(&parsed).unwrap(), bytes);
    for case in 0..5 {
        let mut bad = banks.clone();
        match case {
            0 => {
                bad.selector_tables[0].object_roots.pop();
            }
            1 => bad.selector_tables[0].object_table_offset += 2,
            2 => bad.selector_tables[0].object_roots[0] = 0xb4bc84,
            3 => bad.selector_tables.push(bad.selector_tables[0].clone()),
            _ => bad.selector_tables[0].selector_address += 1,
        }
        assert!(bad.validate().is_err());
    }
    let mut coherent = banks.clone();
    coherent.selector_tables[0].object_roots.swap(0, 1);
    coherent.selector_tables[0].sequence_roots.swap(0, 1);
    coherent.validate().unwrap();
    assert!(coherent.verify_source(&rom).is_err());
    let inventory: serde_json::Value = serde_json::from_slice(
        &fs::read(std::env::var_os("FFTA_STATE_TEST_OBJ_LOCALIZED_INVENTORY").unwrap()).unwrap(),
    )
    .unwrap();
    let dimensions = ObjDimensions::decode(&rom[0x14ced8..0x14cf08]).unwrap();
    let mut m = crate::make_machine(rom.clone(), None);
    m.bus.ffta_sprites.enabled = false;
    let mut selections = 0;
    let mut constructors = 0;
    let mut ticks = 0;
    let mut instructions = 0;
    let mut objects = 0;
    let mut lists = 0;
    let mut frames = 0;
    for language in 0..5u8 {
        let (object_root, sequence_root) = tables.select(language).unwrap();
        let bank = banks
            .banks
            .iter()
            .find(|b| b.object_root == object_root && b.sequence_root == sequence_root)
            .unwrap();
        let source = &inventory["banks"][language as usize];
        assert_eq!(source["object_root"], object_root);
        assert_eq!(source["sequence_root"], sequence_root);
        assert_eq!(source["lists"].as_array().unwrap().len(), bank.lists.len());
        for (i, list) in bank.lists.iter().enumerate() {
            assert_eq!(source["lists"][i]["source"], list.source_offset);
            assert_eq!(source["lists"][i]["count"], list.objects.len());
            assert_eq!(
                list.source_bytes().unwrap(),
                rom[list.source_offset as usize..list.source_offset as usize + list.source_len()]
            );
            objects += list.objects.len();
            lists += 1;
        }
        let sequence = &bank.sequences[0];
        frames += sequence.frames.len();
        assert_eq!(
            source["sequences"][0]["frames"],
            serde_json::json!(sequence
                .frames
                .iter()
                .map(|f| [f.object_list, f.duration])
                .collect::<Vec<_>>())
        );
        // Execute all three actual original table-selection chains through their
        // constructor argument setup. RAM holds only the original language ID.
        for reference in SELECTOR_REFERENCES {
            m.bus.write8(tables.selector_address, language);
            m.cpu.regs[4] = tables.selector_address;
            m.cpu.regs[13] = STACK;
            m.cpu.regs[15] = 0x08000000 + reference.object_reader;
            m.cpu.cpsr = 0xbf;
            let mut reached = false;
            for _ in 0..40 {
                if m.cpu.regs[15] == 0x08000000 + reference.call {
                    reached = true;
                    break;
                }
                m.bus.frame_ready = false;
                m.step();
                instructions += 1;
            }
            assert!(reached);
            assert_eq!(m.bus.read32(STACK + 12), 0x08000000 + object_root);
            assert_eq!(m.bus.read32(STACK + 16), 0x08000000 + sequence_root);
            assert_eq!(m.cpu.regs[1], 3);
            assert_eq!(m.cpu.regs[2], 0);
            selections += 1;
        }
        for mode in [0u8, 1] {
            for at in 0..60 {
                m.bus.write8(TASK - 16 + at, 0x5a);
            }
            for at in 0..28 {
                m.bus.write8(TASK + at, 0);
            }
            instructions += invoke(
                &mut m,
                if mode == 0 { 0x0804e494 } else { 0x0804e558 },
                [TASK, 0, 0, 100],
                &[
                    70,
                    0,
                    10,
                    0x08000000 + object_root,
                    0x08000000 + sequence_root,
                    292,
                    0,
                ],
            );
            constructors += 1;
            let mut expected = vec![0u8; 28];
            expected[..4].copy_from_slice(&(0x08000000 + sequence.source_offset + 2).to_le_bytes());
            expected[12] = mode;
            expected[13] = 1;
            expected[14] = sequence.frames.len() as u8;
            expected[16] = 100;
            expected[17] = 70;
            expected[19] = 10;
            expected[22..24].copy_from_slice(&292u16.to_le_bytes());
            expected[24..28].copy_from_slice(&(0x08000000 + object_root).to_le_bytes());
            assert!((0..28).all(|at| m.bus.read8(TASK + at) == expected[at as usize]));
            let mut index = 0usize;
            let mut elapsed = 0u8;
            let iterations = if mode == 0 {
                sequence
                    .frames
                    .iter()
                    .map(|f| usize::from(f.duration.max(1)))
                    .sum::<usize>()
                    + 1
            } else {
                1
            };
            for _ in 0..iterations {
                let frame = &sequence.frames[index];
                let (oam, count) =
                    output(&bank.lists[frame.object_list as usize], &dimensions, mode);
                instructions += tick(&mut m);
                ticks += 1;
                let buffer = if mode == 0 { 0x03000030 } else { 0x03002c58 };
                assert_eq!(
                    m.bus
                        .read32(if mode == 0 { 0x03000020 } else { 0x03002c50 }),
                    count
                );
                assert!(
                    (0..1024).all(|at| m.bus.read8(buffer + at) == oam[at as usize]),
                    "localized OAM/matrices language{language} frame{index} mode{mode}"
                );
                elapsed = elapsed.wrapping_add(1);
                if u16::from(elapsed) >= frame.duration {
                    elapsed = 0;
                    index = (index + 1).min(sequence.frames.len() - 1);
                }
                expected[8..10].copy_from_slice(&(index as u16).to_le_bytes());
                expected[15] = elapsed;
                assert!((0..28).all(|at| m.bus.read8(TASK + at) == expected[at as usize]));
                assert!((0..16)
                    .all(|at| m.bus.read8(TASK - 16 + at) == 0x5a
                        && m.bus.read8(TASK + 28 + at) == 0x5a));
            }
        }
    }
    assert_eq!(
        (lists, objects, frames, selections, constructors),
        (149, 574, 144, 15, 10)
    );
    assert_eq!(
        gba_core::ffta_rom::digest(&m.bus.rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"languages":5,"new_banks":5,"new_lists":lists,"new_objects":objects,"new_sequences":5,"new_frames":frames,"original_selector_chains":selections,"original_constructors":constructors,"original_ticks":ticks,"instructions":instructions,"source_json_guards_inventory_task_oam_matrices_clipping_and_terminal_hold":true,"linux":"pending","usa":"pending","performance_measurements":false})).unwrap()).unwrap();
    eprintln!("OBJ_LOCALIZED_EUR selectors={selections} constructors={constructors} ticks={ticks} instructions={instructions} source/JSON/guards/tasks/OAM/matrices=pass");
}
