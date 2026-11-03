//! One Wine filter for newly authenticated conditional constructor animation data.
use gba_core::{
    rom_data::obj_lists::{
        constructor_animation::{Animation, Catalog, LIST_ROOTS, TABLE_LENGTH, TABLE_OFFSET},
        Lists,
    },
    Bus, Machine,
};
use std::{collections::BTreeSet, fs, path::PathBuf};

const SLOT: u32 = 0x02010000;
const INSTANCE: u32 = 0x02012000;

fn pristine(rom: &[u8]) -> Machine {
    let mut m = crate::make_machine(rom.to_vec(), None);
    m.bus.ffta_sprites.enabled = false;
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    m.cpu.regs[13] = 0x03007c00;
    m.cpu.regs[14] = 0x08000301;
    m.cpu.cpsr = 0xbf;
    m
}

fn until(m: &mut Machine, pc: u32) {
    for _ in 0..60 {
        if m.cpu.regs[15] == pc {
            return;
        }
        m.bus.frame_ready = false;
        m.step();
    }
    panic!("new original prefix did not reach {pc:08x}");
}

fn counter_prefix(rom: &[u8], animation: &Animation, phase: u8, old: u8) {
    let threshold = animation.frame(phase).unwrap().threshold;
    let mut m = pristine(rom);
    m.cpu.regs[2] = SLOT;
    m.cpu.regs[3] = 0xa5;
    m.cpu.regs[7] = 0x08000000 + TABLE_OFFSET;
    m.cpu.regs[15] = 0x0809db56;
    m.bus.write8(SLOT + 10, 0xa5);
    m.bus.write8(SLOT + 11, old);
    m.bus.write8(SLOT + 12, phase);
    let mut ew = m.bus.ewram.clone();
    let iw = m.bus.iwram.clone();
    let oam = m.bus.oam.clone();
    let vram = m.bus.vram.clone();
    let palette = m.bus.palette.clone();
    let callee = m.cpu.regs[6..15].to_vec();
    // The original reloads the stored u8. In particular 255 increments to zero.
    let incremented = old.wrapping_add(1);
    let advance = u32::from(incremented) > threshold;
    ew[0x1000b] = if advance { 0 } else { incremented };
    ew[0x1000c] = if advance { phase + 1 } else { phase };
    if advance && phase == 3 {
        ew[0x1000a] = 0xa4;
    }
    until(&mut m, 0x0809db66);
    assert_eq!(
        m.cpu.regs[0],
        0x08000000 + TABLE_OFFSET + 8 * u32::from(phase)
    );
    m.step();
    assert_eq!(m.cpu.regs[0], threshold);
    until(&mut m, 0x0809db82);
    assert_eq!(m.bus.ewram, ew, "only counter/phase/terminal active flag");
    assert_eq!(m.bus.iwram, iw);
    assert_eq!(m.bus.oam, oam);
    assert_eq!(m.bus.vram, vram);
    assert_eq!(m.bus.palette, palette);
    assert_eq!(&m.cpu.regs[6..15], callee.as_slice());
    assert_eq!(m.cpu.cpsr & 0x0fffffff, 0xbf);
}

fn constructor_prefix(rom: &[u8], animation: &Animation, phase: u8) {
    let frame = animation.frame(phase).unwrap();
    let mut m = pristine(rom);
    // Reuse the authenticated original copier map, without executing it again.
    m.bus.iwram[0x5f28..0x6d68].copy_from_slice(&rom[0xd968e8..0xd97728]);
    m.cpu.regs[4] = SLOT;
    m.cpu.regs[8] = 0x08397f9c;
    m.cpu.regs[9] = 0x08000000 + TABLE_OFFSET;
    m.cpu.regs[15] = 0x0809dbca;
    m.bus.write32(SLOT, INSTANCE);
    m.bus.write8(SLOT + 12, phase);
    m.bus.write32(INSTANCE + 0x154, 0x12345678);
    let mut ew = m.bus.ewram.clone();
    ew[0x12158..0x1215c].copy_from_slice(&(0x08000000 + frame.list_offset).to_le_bytes());
    let iw = m.bus.iwram.clone();
    let oam = m.bus.oam.clone();
    let vram = m.bus.vram.clone();
    let palette = m.bus.palette.clone();
    let preserved = m.cpu.regs[4..7].to_vec();
    until(&mut m, 0x0809dbd6);
    assert_eq!(
        m.cpu.regs[1],
        0x08000000 + TABLE_OFFSET + 8 * u32::from(phase) + 4
    );
    m.step();
    assert_eq!(m.cpu.regs[1], 0x08000000 + frame.list_offset);
    until(&mut m, 0x03006b6c);
    assert_eq!(m.cpu.regs[0], INSTANCE);
    assert_eq!(m.cpu.regs[1], 0x08000000 + frame.list_offset);
    until(&mut m, 0x0809dbe0);
    assert_eq!(
        m.cpu.regs[0], 0x12345678,
        "constructor returns prior active field"
    );
    assert_eq!(m.bus.ewram, ew, "only pending list field+158 changes");
    assert_eq!(m.bus.iwram, iw);
    assert_eq!(m.bus.oam, oam);
    assert_eq!(m.bus.vram, vram);
    assert_eq!(m.bus.palette, palette);
    assert_eq!(&m.cpu.regs[4..7], preserved.as_slice());
    assert_eq!(m.cpu.regs[8], 0x08397f9c);
    assert_eq!(m.cpu.regs[9], 0x08000000 + TABLE_OFFSET);
    assert_eq!(m.cpu.regs[13], 0x03007c00);
    assert_eq!(m.cpu.cpsr & 0x0fffffff, 0xbf);
}

#[test]
#[ignore = "private EUR new constructor animation model/original prefixes; Wine only"]
fn european_constructor_obj_animation_matches_original_sources() {
    let root = PathBuf::from(std::env::var_os("FFTA_CONSTRUCTOR_ANIMATION_ROOT").unwrap());
    let out = root.join("windows-model");
    assert!(!out.exists());
    fs::create_dir(&out).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let animation = &catalog.decoded;
    let independent: Animation =
        serde_json::from_slice(&fs::read(root.join("independent-animation.json")).unwrap())
            .unwrap();
    let lists: Lists =
        serde_json::from_slice(&fs::read(root.join("independent-lists.json")).unwrap()).unwrap();
    assert_eq!(animation, &independent);
    assert_eq!(catalog.lists, lists);
    animation.verify_source(&rom).unwrap();
    animation.validate_lists(&lists).unwrap();
    assert_eq!(catalog.sources.len(), 5);
    let mut projected = animation.source_spans().unwrap();
    for list in &lists.lists {
        projected.push((
            list.source_offset,
            list.source_bytes().unwrap(),
            "static-obj-list",
        ));
    }
    assert_eq!(
        projected
            .iter()
            .map(|(_, bytes, _)| bytes.len())
            .sum::<usize>(),
        88
    );
    for (offset, bytes, role) in &projected {
        assert_eq!(
            bytes,
            &rom[*offset as usize..*offset as usize + bytes.len()]
        );
        let source = catalog
            .sources
            .iter()
            .find(|s| s.offset == *offset)
            .unwrap();
        assert_eq!(source.length as usize, bytes.len());
        assert_eq!(&source.role, role);
        assert_eq!(source.sha256, gba_core::ffta_rom::digest(bytes));
    }
    for at in TABLE_OFFSET..TABLE_OFFSET + TABLE_LENGTH as u32 {
        assert_eq!(animation.byte_at(at), Some(rom[at as usize]));
    }
    for at in [0, TABLE_OFFSET - 1, TABLE_OFFSET + 32, u32::MAX] {
        assert_eq!(animation.byte_at(at), None);
    }
    for phase in 4..=255 {
        assert!(animation.frame(phase).is_none());
    }
    let canonical = serde_json::to_vec(animation).unwrap();
    assert_eq!(
        serde_json::from_slice::<Animation>(&canonical).unwrap(),
        *animation
    );
    fs::write(out.join("constructor-obj-animation.json"), &canonical).unwrap();
    let mut guards = 0;
    for mutation in 0..5 {
        let mut changed = animation.clone();
        match mutation {
            0 => changed.source_offset += 4,
            1 => {
                changed.frames.pop();
            }
            2 => changed.frames.push(changed.frames[0].clone()),
            3 => changed.frames[0].list_offset += 2,
            _ => changed.frames.swap(0, 1),
        }
        assert!(changed.validate().is_err());
        guards += 1;
    }
    for mutation in 0..3 {
        let mut changed = lists.clone();
        match mutation {
            0 => {
                changed.lists.pop();
            }
            1 => changed.lists[0].objects[0].shape = 3,
            _ => changed.lists.swap(0, 1),
        }
        assert!(animation.validate_lists(&changed).is_err());
        guards += 1;
    }
    for nested in [false, true] {
        let mut value = serde_json::to_value(animation).unwrap();
        let object = if nested {
            value["frames"][0].as_object_mut().unwrap()
        } else {
            value.as_object_mut().unwrap()
        };
        object.insert("unrecognized".into(), serde_json::json!(0));
        assert!(serde_json::from_value::<Animation>(value).is_err());
        guards += 1;
    }
    // A threshold is a full unsigned word. Source verification rejects edits,
    // while lossless projection preserves values beyond the current u8 counter.
    let mut changed = animation.clone();
    changed.frames[0].threshold = 0xfedcba98;
    changed.validate().unwrap();
    assert_eq!(
        &changed.source_spans().unwrap()[0].1[..4],
        &[0x98, 0xba, 0xdc, 0xfe]
    );
    assert!(changed.verify_source(&rom).is_err());
    guards += 1;
    assert!(Catalog::from_rom(&rom[..rom.len() - 1]).is_err());
    guards += 1;
    let mut edited_rom = rom.clone();
    edited_rom[TABLE_OFFSET as usize] ^= 1;
    assert!(Catalog::from_rom(&edited_rom).is_err());
    guards += 1;
    let mut counter_cases = 0;
    for phase in 0..4 {
        let threshold = animation.frame(phase).unwrap().threshold as u8;
        for old in BTreeSet::from([0, threshold - 1, threshold, 255]) {
            counter_prefix(&rom, animation, phase, old);
            counter_cases += 1;
        }
        constructor_prefix(&rom, animation, phase);
    }
    for (list, (offset, _)) in lists.lists.iter().zip(LIST_ROOTS) {
        assert_eq!(list.source_offset, offset);
        // Reuse the existing ABI oracle on four NEW lists only.
        super::observed_obj_lists_model_tests::copied_ram_draw(&rom, list);
    }
    let result = serde_json::json!({
        "status":"PASS", "source_bytes":88,"table_bytes":32,"list_bytes":56,"objects":8,
        "guards":guards,"counter_prefixes":counter_cases,"constructor_prefixes":4,"new_list_reader_cases":4,
        "canonical_sha256":gba_core::ffta_rom::digest(&canonical),
        "scope":"conditional0..3 only; no global prelookup cap/initializer/lifetime/all-use or ordinary-scene positive use claim",
        "production":"unchanged e73-d72, no new managed resource or owner",
        "Linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false
    });
    fs::write(
        out.join("result.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    eprintln!("ROM_DATA_CONSTRUCTOR_ANIMATION source88/table32/lists56/objects8 guards={guards} counter={counter_cases} constructor4 draw4 PASS");
}
