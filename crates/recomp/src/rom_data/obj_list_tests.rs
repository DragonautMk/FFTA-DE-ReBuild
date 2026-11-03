//! Exact EUR static-list source/JSON and original draw contract; no boot replay.
use super::*;
use gba_core::{
    rom_data::{
        obj_lists::{Catalog, Lists},
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

#[test]
#[ignore = "exact EUR ROM and absent FFTA_STATE_TEST_OBJ_LIST_ROOT; new Linux/USA pending"]
fn european_static_obj_lists_match_source_json_and_both_original_draw_producers() {
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_OBJ_LIST_ROOT").unwrap());
    assert!(!root.exists(), "preserve evidence");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let catalog = Catalog::from_rom(&rom).unwrap();
    assert_eq!(catalog.decoded.lists.len(), 66);
    assert_eq!(gba_core::rom_data::obj_lists::REFERENCES.len(), 107);
    let encoded = serde_json::to_vec_pretty(&catalog.decoded).unwrap();
    let decoded: Lists = serde_json::from_slice(&encoded).unwrap();
    decoded.verify_source(&rom).unwrap();
    assert_eq!(serde_json::to_vec_pretty(&decoded).unwrap(), encoded);
    let mut edited = decoded.clone();
    edited.lists[0].objects[0].tile ^= 1;
    edited.validate().unwrap();
    assert!(edited.verify_source(&rom).is_err());
    edited = decoded.clone();
    edited.lists.remove(0);
    assert!(edited.verify_source(&rom).is_err());
    let mut unknown: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    unknown["lists"][0]["objects"][0]["unrecognized"] = serde_json::json!(1);
    assert!(serde_json::from_value::<Lists>(unknown).is_err());
    assert!(serde_json::from_slice::<Lists>(&encoded[..encoded.len() - 1]).is_err());
    let dimensions = ObjDimensions::decode(&rom[0x14ced8..0x14cf08]).unwrap();
    let mut m = crate::make_machine(rom.clone(), None);
    m.bus.ffta_sprites.enabled = false;
    let mut calls = 0;
    let mut instructions = 0;
    let mut source_bytes = 0;
    let mut object_count = 0;
    for (list, source) in decoded.lists.iter().zip(&catalog.sources) {
        let bytes = list.source_bytes().unwrap();
        assert_eq!(
            bytes,
            rom[source.offset as usize..(source.offset + source.length) as usize]
        );
        assert_eq!(gba_core::ffta_rom::digest(&bytes), source.sha256);
        source_bytes += bytes.len();
        object_count += list.objects.len();
        for first in [true, false] {
            for (x, y, flags, initial, limit) in [
                (100, 70, 0, 0, 128),
                (-20, -10, 0, 1, 3),
                (250, 170, 0x10000000, 0, 128),
                (100, 70, 0x20000000, 2, 3),
                (100, 70, 0x30000400, 0, 128),
                (32760, -32760, 0, 0, 128),
                (i32::MAX, i32::MIN, 0x30000000, 0, 128),
                (100, 70, 0, 3, 3),
            ] {
                for at in 0..128 * 8 + 32 {
                    m.bus.write8(BUFFER - 16 + at, 0x5a);
                }
                let (expected, result) =
                    expected(list, &dimensions, first, x, y, flags, initial, limit);
                instructions += invoke(
                    &mut m,
                    first,
                    0x08000000 + list.source_offset,
                    x,
                    y,
                    flags,
                    initial,
                    limit,
                );
                assert_eq!(
                    m.cpu.regs[0], result,
                    "list{:x} first{first}",
                    list.source_offset
                );
                assert!(
                    (0..expected.len()).all(|at| m.bus.read8(BUFFER + at as u32) == expected[at]),
                    "full OAM buffer/matrices list{:x} first{first} x{x} y{y} flags{flags:x}",
                    list.source_offset
                );
                assert!((0..16).all(|at| m.bus.read8(BUFFER - 16 + at) == 0x5a
                    && m.bus.read8(BUFFER + 128 * 8 + at) == 0x5a));
                calls += 1;
            }
        }
    }
    assert_eq!(
        gba_core::ffta_rom::digest(&m.bus.rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("obj-static-lists.json"), &encoded).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "region":"europe", "lists":66, "direct_literal_references":107, "objects":object_count,
        "source_bytes":source_bytes, "sources":catalog.sources, "original_calls":calls,
        "original_instructions":instructions, "json_sha256":gba_core::ffta_rom::digest(&encoded),
        "full_oam_buffers_matrices_and_guards":true, "signed_wrap_clip_flip_limit_and_early_return":true,
        "runtime":"offline-only", "linux":"pending", "usa":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
    eprintln!("OBJ_LISTS_EUR lists=66 refs=107 objects={object_count} source_bytes={source_bytes} calls={calls} instructions={instructions} source/JSON/original-OAM/guards=pass");
}
