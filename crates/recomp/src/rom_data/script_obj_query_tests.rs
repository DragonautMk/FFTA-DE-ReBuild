//! Functional source-query census on an existing unmodified EUR recording.
//! Entry probes preserve existing native functions and normal missing-entry steps.
use super::*;
use crate::{BlockFn, InputScript};
use gba_core::{capi::RtApi, Machine};
use serde::Serialize;
use std::{cell::RefCell, collections::BTreeMap, ffi::c_void};

#[derive(Clone, Copy)]
struct Pending {
    source: u32,
    selector: u8,
    group: u16,
    stack: u32,
    return_pc: u32,
    context: Option<u32>,
}
#[derive(Serialize)]
struct Query {
    frame: u64,
    script_source: u32,
    actor_selector: u8,
    group: u16,
    base: u16,
    actor_context: u32,
    actor_flags: u32,
    sprite: u32,
    animation: u32,
    part: u32,
    index_source: Option<u32>,
    target_guest_word: Option<u32>,
}
#[derive(Default)]
struct Observation {
    originals: [Option<BlockFn>; 3],
    expected: BTreeMap<u32, (u8, u16)>,
    pending: Option<Pending>,
    handlers: usize,
    bodies: usize,
    queries: Vec<Query>,
}
thread_local! {
    static OBSERVATION: RefCell<Observation> = RefCell::new(Observation::default());
}
fn byte(m: &Machine, address: u32) -> Option<u8> {
    match address >> 24 {
        2 => m.bus.ewram.get((address & 0x3ffff) as usize).copied(),
        3 => m.bus.iwram.get((address & 0x7fff) as usize).copied(),
        8 => m.bus.rom.get((address - 0x08000000) as usize).copied(),
        _ => None,
    }
}
fn half(m: &Machine, address: u32) -> Option<u16> {
    Some(u16::from_le_bytes([
        byte(m, address)?,
        byte(m, address + 1)?,
    ]))
}
fn word(m: &Machine, address: u32) -> Option<u32> {
    Some(u32::from_le_bytes([
        byte(m, address)?,
        byte(m, address + 1)?,
        byte(m, address + 2)?,
        byte(m, address + 3)?,
    ]))
}
fn record(index: usize, m: &Machine) -> Option<BlockFn> {
    OBSERVATION.with(|cell| {
        let mut o = cell.borrow_mut();
        let original = o.originals[index];
        if index == 0 {
            let cursor = m.cpu.regs[1];
            let source = cursor.checked_sub(0x08000000).unwrap();
            let &(selector, group) = o.expected.get(&source).expect("authenticated ROM script49");
            assert_eq!(byte(m, cursor), Some(49));
            assert_eq!(byte(m, cursor + 1), Some(selector));
            assert_eq!(half(m, cursor + 2), Some(group));
            o.handlers += 1;
            o.pending = Some(Pending {
                source,
                selector,
                group,
                stack: m.cpu.regs[13],
                return_pc: m.cpu.regs[14] & !1,
                context: None,
            });
        } else if let Some(mut p) = o.pending {
            if index == 1
                && m.cpu.regs[13] == p.stack.wrapping_sub(32)
                && m.cpu.regs[14] & !1 == 0x0809d9e0
            {
                assert_eq!(m.cpu.regs[1], u32::from(p.group.wrapping_mul(4)));
                assert_eq!(m.cpu.regs[3], 0, "script body disables status substitution");
                p.context = Some(m.cpu.regs[0]);
                o.pending = Some(p);
                o.bodies += 1;
            } else if index == 2
                && m.cpu.regs[13] == p.stack.wrapping_sub(68)
                && matches!(m.cpu.regs[14] & !1, 0x0809ada6 | 0x0809ae74)
            {
                let context = p.context.expect("resolved body before constructor");
                let stack = m.cpu.regs[13];
                let sprite = word(m, stack).unwrap();
                let animation = word(m, stack + 4).unwrap();
                let part = word(m, stack + 8).unwrap();
                assert_eq!(sprite, u32::from(half(m, context + 0x34).unwrap()));
                assert!(part <= 2);
                assert_eq!(part == 0, m.cpu.regs[14] & !1 == 0x0809ada6);
                let directory = 0x08394834u32.wrapping_add(sprite.wrapping_mul(4));
                let index_address = word(m, directory).map(|root| {
                    root.wrapping_add((animation & !3u32).wrapping_mul(6))
                        .wrapping_add(if matches!(animation & 3, 1 | 2) {
                            12
                        } else {
                            0
                        })
                        .wrapping_add(part.wrapping_mul(4))
                });
                let index_source = index_address.and_then(|a| a.checked_sub(0x08000000));
                let target_guest_word = index_address.and_then(|a| word(m, a));
                o.queries.push(Query {
                    frame: m.bus.frames,
                    script_source: p.source,
                    actor_selector: p.selector,
                    group: p.group,
                    base: p.group.wrapping_mul(4),
                    actor_context: context,
                    actor_flags: word(m, context + 0x30).unwrap(),
                    sprite,
                    animation,
                    part,
                    index_source,
                    target_guest_word,
                });
            }
        }
        original
    })
}
fn forward(index: usize, api: *const RtApi, pointer: *mut c_void) -> u32 {
    // The immutable borrow ends before the original native function executes.
    let original = record(index, unsafe { &*pointer.cast::<Machine>() });
    if let Some(original) = original {
        original(api, pointer)
    } else {
        // This entry was already missing from the retained cache. Execute the
        // same single original instruction as the normal missing-entry path.
        unsafe { &mut *pointer.cast::<Machine>() }.step();
        0 // The outer dispatcher uses the machine PC, not this return value.
    }
}
extern "C" fn handler(api: *const RtApi, pointer: *mut c_void) -> u32 {
    forward(0, api, pointer)
}
extern "C" fn body(api: *const RtApi, pointer: *mut c_void) -> u32 {
    forward(1, api, pointer)
}
extern "C" fn constructor(api: *const RtApi, pointer: *mut c_void) -> u32 {
    forward(2, api, pointer)
}

#[test]
#[ignore = "private immutable EUR script recording, functional query census; Wine only, Linux/USA pending"]
fn european_script_animation_resolved_queries_from_existing_recording() {
    crate::configure_headless_widescreen();
    let root = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_SCRIPT_OBJ_QUERY_ROOT").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(&root).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let library = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap());
    let (_library, mut table) = crate::load_native(&library).unwrap();
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let script_bytes = fs::read(dataset.join("script-archive.json")).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&script_bytes),
        "29e61d5ddec422d21b92ff795ef931e51b85d5a129316bb8f1873264f7d3330f"
    );
    let archive: gba_core::rom_data::script_archive::ScriptArchive =
        serde_json::from_slice(&script_bytes).unwrap();
    OBSERVATION.with(|cell| {
        let mut o = cell.borrow_mut();
        *o = Observation::default();
        for instruction in archive.programs.iter().flat_map(|p| &p.instructions) {
            if instruction.command == 49 {
                let p = &instruction.parameters;
                o.expected.insert(
                    instruction.source_offset,
                    (p[0], u16::from_le_bytes([p[1], p[2]])),
                );
            }
        }
        for (index, (key, probe)) in [
            (0x08126cc1, handler as BlockFn),
            (0x0809acf9, body as BlockFn),
            (0x08022789, constructor as BlockFn),
        ]
        .into_iter()
        .enumerate()
        {
            o.originals[index] = table.get(key);
            table.rom[(key - crate::ROM_BASE) as usize] = Some(probe);
        }
    });
    let mut machine = crate::make_machine(rom, None);
    let state = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_STATE").unwrap());
    crate::save_state::load(&mut machine, &state).unwrap();
    crate::arm_audio_hle(&mut machine, None);
    let input = InputScript::load(&std::env::var("FFTA_STATE_TEST_INPUT").unwrap()).unwrap();
    let frames: u64 = std::env::var("FFTA_STATE_TEST_FRAMES")
        .unwrap()
        .parse()
        .unwrap();
    assert!(frames > 0);
    let first = machine.bus.frames;
    for relative in 0..frames {
        machine.bus.keys = input.keys_at(relative);
        input.apply_mouse(&mut machine.bus, relative).unwrap();
        let pointer = &mut machine as *mut Machine as *mut c_void;
        crate::run_frame_native_observed(
            &mut machine,
            &table,
            pointer,
            crate::FRAME_STEP_GUARD,
            |_, pc, _| {
                OBSERVATION.with(|cell| {
                    let mut o = cell.borrow_mut();
                    if o.pending.is_some_and(|p| pc == p.return_pc) {
                        o.pending = None;
                    }
                });
            },
        );
        assert_eq!(machine.bus.frames, first + relative + 1, "complete frame");
        assert!(machine.bus.frame_ready);
        machine.bus.audio_buf.clear();
    }
    OBSERVATION.with(|cell| {
        let o = cell.borrow();
        let distinct: std::collections::BTreeSet<_> = o.queries.iter()
            .map(|q| [q.sprite, q.animation, q.part]).collect();
        fs::write(root.join("resolved-queries.json"), serde_json::to_vec_pretty(&o.queries).unwrap()).unwrap();
        let result = serde_json::json!({
            "frames":frames,"first_frame":first,"end_frame":machine.bus.frames,
            "handlers":o.handlers,"resolved_bodies":o.bodies,"constructor_queries":o.queries.len(),
            "existing_native_entries":o.originals.map(|f| f.is_some()),
            "distinct_queries":distinct,"scope":"functional script49-to-live-actor-to-constructor source census, original retained native functions or normal missing-entry instruction; no equivalence/performance claim",
            "guest_or_private_payload_edits":false,"linux":"pending","usa":"pending","performance_measurements":false
        });
        fs::write(root.join("result.json"),serde_json::to_vec_pretty(&result).unwrap()).unwrap();
        eprintln!("ROM_DATA_SCRIPT_OBJ_QUERIES {result}");
        // A negative source census is terminal for its unchanged recording.
        // It is reported in result.json without inventing a positive query.
        assert_eq!(o.handlers, o.bodies, "each observed49resolves a body");
        assert_eq!(o.queries.len(), o.bodies * 3, "body and two real extra queries");
    });
}
