//! One Wine source-model filter for two newly observed indirect OBJ lists.
use super::*;
use gba_core::{
    rom_data::{
        obj_lists::{
            observed::{Catalog, ObservedLists, ROOTS},
            List,
        },
        ObjDimensions,
    },
    Bus, Machine,
};
const BUFFER: u32 = 0x02030010;
const STACK: u32 = 0x03007c00;
const RETURN: u32 = 0x08000300;

fn invoke(
    m: &mut Machine,
    first: bool,
    list: u32,
    x: i32,
    y: i32,
    flags: u32,
    initial: u32,
    limit: u32,
) -> usize {
    m.cpu.regs[..4].copy_from_slice(&[list, x as u32, y as u32, if first { flags } else { 7 }]);
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.regs[15] = if first { 0x08001500 } else { 0x0800170c };
    m.cpu.cpsr = 0xbf;
    let args = if first {
        vec![7, 2, 3, BUFFER, limit, initial]
    } else {
        vec![0x102, 3, BUFFER, limit, initial]
    };
    for (index, value) in args.into_iter().enumerate() {
        m.bus.write32(STACK + index as u32 * 4, value);
    }
    for steps in 1..=20_000 {
        m.bus.frame_ready = false;
        m.step();
        if m.cpu.regs[15] == RETURN {
            assert_eq!(m.cpu.regs[13], STACK);
            return steps;
        }
    }
    panic!("original OBJ draw did not return at{:08x}", m.cpu.regs[15]);
}

fn expected(
    list: &gba_core::rom_data::obj_lists::List,
    dimensions: &ObjDimensions,
    first: bool,
    x: i32,
    y: i32,
    flags: u32,
    initial: u32,
    limit: u32,
) -> (Vec<u8>, u32) {
    let mut bytes = vec![0x5a; 128 * 8];
    let mut next = initial as usize;
    for object in &list.objects {
        if next >= limit as usize {
            break;
        }
        let dim = dimensions.get(object.shape, object.size).unwrap();
        let x = if first && flags & 0x10000000 != 0 {
            x.wrapping_sub(i32::from(object.x))
                .wrapping_sub(i32::from(dim.width))
        } else {
            x.wrapping_add(i32::from(object.x))
        } as i16 as i32;
        let y = if first && flags & 0x20000000 != 0 {
            y.wrapping_sub(i32::from(object.y))
                .wrapping_sub(i32::from(dim.height))
        } else {
            y.wrapping_add(i32::from(object.y))
        } as i16 as i32;
        if x < 0 && x + i32::from(dim.width) < 0
            || x > 239 && x + i32::from(dim.width) > 239
            || y < 0 && y + i32::from(dim.height) < 0
            || y > 159 && y + i32::from(dim.height) > 159
        {
            continue;
        }
        let [a0, a1, _] = object.attributes();
        let a0 = (a0 & 0xff00) | (y as u16 & 255);
        let a1 = (a1 & !511) | (x as u16 & 511);
        let mask = if first { flags } else { 0 };
        let packed =
            (u32::from(a0) | (u32::from(a1) << 16) | (mask & 0xcfffffff)) ^ (mask & 0x30000000);
        let a2 = (object.tile + 7) % 1024
            | (u16::from((object.priority + 2) % 4) << 10)
            | (u16::from((object.palette + 3) % 16) << 12);
        let attrs = [packed as u16, (packed >> 16) as u16, a2];
        for (j, attr) in attrs.into_iter().enumerate() {
            bytes[next * 8 + j * 2..next * 8 + j * 2 + 2].copy_from_slice(&attr.to_le_bytes());
        }
        next += 1;
    }
    // EUR draw A loads the initial index into r0 before its capacity guard.
    let result = next as u32;
    (bytes, result)
}

fn pristine(rom: &[u8]) -> Machine {
    let mut m = crate::make_machine(rom.to_vec(), None);
    m.bus.ffta_sprites.enabled = false;
    m.cpu.regs = std::array::from_fn(|i| 0x11000000 + i as u32);
    m
}
fn check_ram(before_ew: &[u8], before_iw: &[u8], m: &Machine, ram_draw: bool) {
    let mut ew = m.bus.ewram.clone();
    let mut expected_ew = before_ew.to_vec();
    if !ram_draw {
        ew[0x30000..0x30420].fill(0);
        expected_ew[0x30000..0x30420].fill(0);
    }
    assert_eq!(ew, expected_ew, "EWRAM outside declared destination");
    let mut iw = m.bus.iwram.clone();
    let mut expected_iw = before_iw.to_vec();
    for bytes in [&mut iw, &mut expected_iw] {
        bytes[0x7b00..0x7c80].fill(0);
        if ram_draw {
            bytes[0x20..0x24].fill(0);
            bytes[0x30..0x430].fill(0);
        }
    }
    assert_eq!(iw, expected_iw, "IWRAM outside output/cursor/stack");
}
fn rom_draw(rom: &[u8], list: &List, dimensions: &ObjDimensions, first: bool) {
    let mut m = pristine(rom);
    for i in 0..128 * 8 + 32 {
        m.bus.write8(BUFFER - 16 + i, 0x5a);
    }
    let ew = m.bus.ewram.clone();
    let iw = m.bus.iwram.clone();
    let vram = m.bus.vram.clone();
    let oam = m.bus.oam.clone();
    let palette = m.bus.palette.clone();
    let callee = m.cpu.regs[4..12].to_vec();
    let (expected, result) = expected(list, dimensions, first, 100, 110, 0, 0, 128);
    invoke(
        &mut m,
        first,
        0x08000000 + list.source_offset,
        100,
        110,
        0,
        0,
        128,
    );
    assert_eq!(m.cpu.regs[0], result);
    assert_eq!(&m.cpu.regs[4..12], callee.as_slice());
    for (i, &byte) in expected.iter().enumerate() {
        assert_eq!(m.bus.read8(BUFFER + i as u32), byte, "full buffer/matrix");
    }
    for i in 0..16 {
        assert_eq!(m.bus.read8(BUFFER - 16 + i), 0x5a);
        assert_eq!(m.bus.read8(BUFFER + 1024 + i), 0x5a);
    }
    check_ram(&ew, &iw, &m, false);
    assert_eq!(m.bus.vram, vram);
    assert_eq!(m.bus.oam, oam);
    assert_eq!(m.bus.palette, palette);
}
pub(super) fn copied_ram_draw(rom: &[u8], list: &List) {
    let mut m = pristine(rom);
    // Authenticated original copier mapping, reused as test setup. Do not rerun
    // the already classified copier or turn this into a new game checkpoint.
    m.bus.iwram[0x5f28..0x6d68].copy_from_slice(&rom[0xd968e8..0xd97728]);
    let buffer = 0x03000030;
    for i in 0..1024 + 16 {
        m.bus.write8(buffer + i, 0x5a);
    }
    m.bus.write8(0x03000028, 0);
    m.bus.write32(0x03000020, 0);
    for i in 0..list.objects.len() {
        m.bus.write8(0x02020000 + i as u32, 0);
    }
    m.cpu.regs[..4].copy_from_slice(&[0x08000000 + list.source_offset, 100, 110, 0]);
    m.cpu.regs[13] = STACK;
    m.cpu.regs[14] = RETURN | 1;
    m.cpu.regs[15] = 0x030065a8;
    m.cpu.cpsr = 0xbf;
    let color = u32::from(list.objects[0].color_256);
    assert!(list
        .objects
        .iter()
        .all(|o| u32::from(o.color_256) == color && o.object_mode == 0 && !o.mosaic));
    for (i, value) in [0, color, 0, 0, 0, 0x02020000, 0, 0]
        .into_iter()
        .enumerate()
    {
        m.bus.write32(STACK + i as u32 * 4, value);
    }
    let ew = m.bus.ewram.clone();
    let iw = m.bus.iwram.clone();
    let callee = m.cpu.regs[4..12].to_vec();
    let vram = m.bus.vram.clone();
    let oam = m.bus.oam.clone();
    let palette = m.bus.palette.clone();
    let mut bytes = vec![0x5a; 1024];
    for (i, o) in list.objects.iter().enumerate() {
        let [a0, a1, a2] = o.attributes();
        let x = 100 + i32::from(o.x);
        let y = 110 + i32::from(o.y);
        assert!((0..=239).contains(&x) && (0..=159).contains(&y));
        let attrs = [
            (a0 & 0xff00) | (y as u16 & 255),
            (a1 & !511) | (x as u16 & 511),
            a2,
        ];
        for (j, value) in attrs.into_iter().enumerate() {
            bytes[i * 8 + j * 2..i * 8 + j * 2 + 2].copy_from_slice(&value.to_le_bytes());
        }
    }
    for n in 0..20_000 {
        if m.cpu.regs[15] == RETURN {
            break;
        }
        assert!(n < 19_999, "copied reader did not return");
        m.bus.frame_ready = false;
        m.step();
        assert!(m.cpu.regs[13] >= STACK - 0x100);
    }
    assert_eq!(m.cpu.regs[13], STACK);
    assert_eq!(&m.cpu.regs[4..12], callee.as_slice());
    assert_eq!(m.bus.read32(0x03000020), list.objects.len() as u32);
    for (i, &byte) in bytes.iter().enumerate() {
        assert_eq!(
            m.bus.read8(buffer + i as u32),
            byte,
            "copied reader fullbuffer/matrix"
        );
    }
    check_ram(&ew, &iw, &m, true);
    assert_eq!(m.bus.vram, vram);
    assert_eq!(m.bus.oam, oam);
    assert_eq!(m.bus.palette, palette);
}
#[test]
#[ignore = "private exact EUR observed-list model/source/original readers; Wine only, Linux/USA pending"]
fn european_observed_obj_lists_model_matches_source_and_original_readers() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_OBSERVED_OBJ_ROOT").unwrap());
    let out = root.join("windows-model-europe");
    assert!(!out.exists());
    fs::create_dir(&out).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    let data = &catalog.decoded;
    let expected_model: ObservedLists =
        serde_json::from_slice(&fs::read(root.join("independent-model.json")).unwrap()).unwrap();
    assert_eq!(data, &expected_model);
    data.verify_source(&rom).unwrap();
    let directory = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let manifest: Manifest =
        toml::from_str(&fs::read_to_string(directory.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (
            manifest.extractor_revision,
            manifest.decoder_revision,
            manifest.resources.len()
        ),
        (71, 70, 65)
    );
    assert_eq!(
        (
            gba_core::rom_data::EXTRACTOR_REV,
            gba_core::rom_data::DECODER_REV
        ),
        (71, 70)
    );
    let published: gba_core::rom_data::obj_lists::Lists =
        serde_json::from_slice(&fs::read(directory.join(OBJ_STATIC_LISTS_FILE)).unwrap()).unwrap();
    assert_eq!(published.lists.len(), 74);
    assert_eq!(
        published
            .lists
            .iter()
            .map(|l| l.objects.len())
            .sum::<usize>(),
        175
    );
    let spans = data.source_spans().unwrap();
    assert_eq!(spans.len(), 2);
    for ((list, root), (source, (offset, bytes, role))) in data
        .lists
        .iter()
        .zip(ROOTS)
        .zip(catalog.sources.iter().zip(&spans))
    {
        assert_eq!(list.source_offset, root.source_offset);
        assert_eq!(list.objects.len(), root.object_count);
        assert_eq!(*offset, source.offset);
        assert_eq!(source.length as usize, bytes.len());
        assert_eq!(source.role, *role);
        assert_eq!(
            bytes,
            &rom[*offset as usize..*offset as usize + bytes.len()]
        );
        assert_eq!(gba_core::ffta_rom::digest(bytes), root.source_sha256);
        assert_eq!(source.sha256, root.source_sha256);
        assert!(manifest
            .resources
            .iter()
            .flat_map(|r| &r.sources)
            .all(|s| s.offset + s.length <= *offset || *offset + bytes.len() as u32 <= s.offset));
        assert_eq!(data.byte_at(*offset - 1), None);
        assert_eq!(data.byte_at(*offset + bytes.len() as u32), None);
        for (i, &value) in bytes.iter().enumerate() {
            assert_eq!(data.byte_at(*offset + i as u32), Some(value));
        }
    }
    assert_eq!(spans.iter().map(|(_, b, _)| b.len()).sum::<usize>(), 64);
    let encoded = serde_json::to_vec_pretty(data).unwrap();
    let round: ObservedLists = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(&round, data);
    let mut guards = 0;
    for case in 0..9 {
        let mut invalid = data.clone();
        match case {
            0 => {
                invalid.lists.pop();
            }
            1 => invalid.lists.swap(0, 1),
            2 => invalid.lists[0].source_offset += 1,
            3 => {
                invalid.lists[0].objects.pop();
            }
            4 => invalid.lists[0].objects[0].x = 256,
            5 => invalid.lists[0].objects[0].shape = 3,
            6 => invalid.lists[0].objects[0].priority = 4,
            7 => invalid.lists[0].objects[0].transform = 32,
            _ => invalid.lists[1].source_offset = invalid.lists[0].source_offset,
        }
        assert!(invalid.validate().is_err());
        guards += 1;
    }
    let mut coherent = data.clone();
    coherent.lists[0].objects[0].tile ^= 1;
    coherent.validate().unwrap();
    assert!(coherent.verify_source(&rom).is_err());
    guards += 1;
    let mut unknown: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    unknown["unknown"] = serde_json::json!(1);
    assert!(serde_json::from_value::<ObservedLists>(unknown).is_err());
    guards += 1;
    let mut unknown: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    unknown["lists"][1]["objects"][0]["unknown"] = serde_json::json!(1);
    assert!(serde_json::from_value::<ObservedLists>(unknown).is_err());
    guards += 1;
    assert!(serde_json::from_slice::<ObservedLists>(&encoded[..encoded.len() - 1]).is_err());
    guards += 1;
    let mut modified = rom.clone();
    modified[0x394f4c] ^= 1;
    assert!(Catalog::from_rom(&modified).is_err());
    guards += 1;
    let dimensions = ObjDimensions::decode(&rom[0x14ced8..0x14cf08]).unwrap();
    for list in &data.lists {
        rom_draw(&rom, list, &dimensions, true);
        rom_draw(&rom, list, &dimensions, false);
        copied_ram_draw(&rom, list);
    }
    fs::write(out.join("decoded.json"), &encoded).unwrap();
    fs::write(out.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"lists":2,"objects":10,"source_bytes":64,"guards":guards,"original_paths":6,"readers":["ROM1500","ROM170C","copiedRAM030065A8"],"all_source_and_predicted_output_bytes_equal":true,"full_RAM_outside_output_cursor_stack_and_callee_guards":true,"old_74_catalog_and_e71_unchanged":true,"decoded_sha256":gba_core::ffta_rom::digest(&encoded),"linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false})).unwrap()).unwrap();
    eprintln!("ROM_DATA_OBSERVED_OBJ_MODEL lists=2 objects=10 source_bytes=64 guards={guards} original_paths=6 PASS");
}
