//! Ordinary E134 OBJ draw through explicitly observed translated instructions.
use super::*;
use gba_core::{
    capi::{RtApi, RT_API},
    rom_data::read_audit::{Service, TraceCapture, TraceFilter},
    Machine,
};
use recomp_core::analyze::{Analysis, Block, View};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeSet,
    ffi::c_void,
};

const OBJECT_ROOT: u32 = 0xb93ff0;
const SEQUENCE_ROOT: u32 = 0xb93f50;
const DRAW_START: u32 = 0x08001500;
const DRAW_END: u32 = 0x080018d8;
const FRAMES: u64 = 160;

thread_local! {
    static HEADERS: RefCell<Vec<(u32, u64)>> = const { RefCell::new(Vec::new()) };
    static IN_DRAW: Cell<bool> = const { Cell::new(false) };
    static INTERPRETER_DEPTH: Cell<u32> = const { Cell::new(0) };
}

fn native_read(
    pointer: *mut c_void,
    address: u32,
    read: extern "C" fn(*mut c_void, u32) -> u32,
) -> u32 {
    if IN_DRAW.with(Cell::get)
        && INTERPRETER_DEPTH.with(Cell::get) == 0
        && (0x08..=0x0d).contains(&(address >> 24))
    {
        let offset = address & 0x01ff_ffff;
        HEADERS.with(|headers| {
            if let Some((_, count)) = headers
                .borrow_mut()
                .iter_mut()
                .find(|(at, _)| *at == offset)
            {
                *count += 1;
            }
        });
    }
    read(pointer, address)
}
extern "C" fn read8(pointer: *mut c_void, address: u32) -> u32 {
    native_read(pointer, address, RT_API.read8)
}
extern "C" fn read16(pointer: *mut c_void, address: u32) -> u32 {
    native_read(pointer, address, RT_API.read16)
}
extern "C" fn read32(pointer: *mut c_void, address: u32) -> u32 {
    native_read(pointer, address, RT_API.read32)
}
extern "C" fn interp_one(pointer: *mut c_void) -> u32 {
    INTERPRETER_DEPTH.with(|depth| depth.set(depth.get() + 1));
    let result = (RT_API.interp_one)(pointer);
    INTERPRETER_DEPTH.with(|depth| depth.set(depth.get() - 1));
    result
}
static API: RtApi = RtApi {
    read8,
    read16,
    read32,
    interp_one,
    ..RT_API
};

#[test]
#[ignore = "EUR ordinary checkpoint, E170 dataset and TinyCC; Windows executable under Wine"]
fn european_context_bank_ordinary_translated_draw() {
    assert_eq!(std::env::var("FFTA_PATCH_SPR").as_deref(), Ok("0"));
    crate::configure_headless_widescreen();
    let root = PathBuf::from(std::env::var_os("FFTA_CONTEXT_ORDINARY_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_CONTEXT_ORDINARY_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let state = fs::read(std::env::var_os("FFTA_CONTEXT_ORDINARY_STATE").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&state),
        "ca90c12b4564c21fd8cea20e65bcc2c017d0d3a1ee61e89623920dd1f9a00305"
    );
    let manifest: Manifest =
        toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (170, 169)
    );
    let resource = manifest
        .resources
        .iter()
        .find(|r| r.id == OBJ_ANIMATION_BANKS_ID)
        .unwrap();
    let banks = parse_obj_animations(&fs::read(dataset.join(&resource.path)).unwrap()).unwrap();
    let bank = banks
        .banks
        .iter()
        .find(|b| b.object_root == OBJECT_ROOT && b.sequence_root == SEQUENCE_ROOT)
        .unwrap();
    let selected: BTreeSet<usize> = bank.sequences[0]
        .frames
        .iter()
        .map(|f| usize::from(f.object_list))
        .collect();
    assert_eq!(
        (
            bank.lists.len(),
            selected.len(),
            bank.sequences[0].frames.len()
        ),
        (21, 19, 38)
    );
    let ranges: Vec<_> = bank
        .spans()
        .iter()
        .map(|&(at, len, _)| (at, at + len as u32))
        .collect();
    HEADERS.with(|headers| {
        *headers.borrow_mut() = bank
            .lists
            .iter()
            .map(|list| (list.source_offset, 0))
            .collect()
    });

    // Derive entry points from this exact no-input route with the sprite patch disabled.
    let mut scout = crate::make_machine(rom.clone(), None);
    scout.load_state(&state).unwrap();
    scout.bus.ffta_sprites.enabled = false;
    assert!(!scout.bus.ffta_sprites.enabled);
    crate::arm_audio_hle(&mut scout, None);
    let mut visited = BTreeSet::new();
    for frame in 0..FRAMES {
        scout.bus.keys = 0x3ff;
        scout.bus.frame_ready = false;
        for _ in 0..5_000_000 {
            if scout.bus.frame_ready {
                break;
            }
            let pc = scout.cpu.regs[15];
            if (DRAW_START..DRAW_END).contains(&pc) {
                visited.insert((pc, scout.cpu.thumb()));
            }
            scout.step();
        }
        assert!(
            scout.bus.frame_ready,
            "scout frame {frame} exceeded step limit"
        );
        if (frame + 1) % 40 == 0 {
            println!("E134_DRAW_SCOUT frames={}/{}", frame + 1, FRAMES);
        }
    }
    assert!(
        visited.iter().any(|&(pc, _)| pc == 0x0800153c),
        "ordinary list-header reader not observed"
    );
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: Some(&scout.bus.iwram),
        bios: None,
    };
    let blocks = visited
        .iter()
        .map(|&(address, thumb)| {
            let first = if thumb {
                armv4t::decode_thumb(view.read16(address), address)
            } else {
                armv4t::decode_arm(view.read32(address), address)
            };
            let mut instrs = vec![first];
            if thumb {
                let next = armv4t::decode_thumb(view.read16(address + 2), address + 2);
                if armv4t::fuse_thumb_bl(&instrs[0], &next).is_some() {
                    instrs.push(next);
                }
            }
            Block {
                addr: address,
                thumb,
                instrs,
            }
        })
        .collect();
    let object = root.join("ordinary-draw-supplement.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().expect("TinyCC required"),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_supplement_library, supplement) = crate::load_native(&object).unwrap();
    let (_library, mut table) = crate::load_native(Path::new(
        &std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap(),
    ))
    .unwrap();
    let mut added = 0;
    for &(pc, thumb) in &visited {
        let key = pc | u32::from(thumb);
        if table.get(key).is_none() {
            let function = supplement
                .get(key)
                .expect("observed draw instruction compiled");
            let index = key.wrapping_sub(0x0800_0000) as usize;
            if index >= table.rom.len() {
                table.rom.resize(index + 1, None);
            }
            table.rom[index] = Some(function);
            added += 1;
        }
    }

    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &dataset).unwrap();
    for machine in [&mut raw, &mut managed] {
        machine.load_state(&state).unwrap();
        machine.bus.ffta_sprites.enabled = false;
        assert!(!machine.bus.ffta_sprites.enabled);
        crate::arm_audio_hle(machine, None);
        machine.bus.tap_channels = true;
        machine.bus.audio_buf.clear();
        machine.bus.psg_tap.clear();
        machine.bus.fifo_tap[0].clear();
        machine.bus.fifo_tap[1].clear();
        machine.bus.hle_tap.clear();
    }
    assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
    let trace = TraceCapture::start(
        TraceFilter {
            readers: vec![],
            ranges: ranges.clone(),
        },
        50_000,
    )
    .unwrap();
    let start = raw.bus.frames;
    let mut dispatch = (0u64, 0u64);
    let mut draw_dispatches = 0u64;
    let mut audio = [0u64; 5];
    for frame in 0..FRAMES {
        for &(lo, hi) in &ranges {
            managed.bus.rom[lo as usize..hi as usize].fill(0xcd);
        }
        raw.bus.keys = 0x3ff;
        trace.frame(None);
        let raw_pointer = &mut raw as *mut Machine as *mut c_void;
        let raw_dispatch =
            crate::run_frame_native(&mut raw, &table, raw_pointer, crate::FRAME_STEP_GUARD);
        managed.bus.keys = 0x3ff;
        trace.frame(Some(managed.bus.frames));
        let managed_pointer = &mut managed as *mut Machine as *mut c_void;
        let managed_dispatch = crate::with_test_native_api(&API, || {
            crate::run_frame_native_traced(
                &mut managed,
                &table,
                managed_pointer,
                crate::FRAME_STEP_GUARD,
                |machine, kind| {
                    let active = kind == crate::NativeDispatch::Native
                        && (DRAW_START..DRAW_END).contains(&machine.cpu.regs[15]);
                    IN_DRAW.with(|v| v.set(active));
                    if active {
                        draw_dispatches += 1;
                    }
                },
                |_, _, _| IN_DRAW.with(|v| v.set(false)),
            )
        });
        trace.frame(None);
        assert_eq!(raw_dispatch, managed_dispatch, "dispatch frame {frame}");
        assert_eq!(
            (raw.bus.frames, managed.bus.frames),
            (start + frame + 1, start + frame + 1)
        );
        dispatch.0 += raw_dispatch.0;
        dispatch.1 += raw_dispatch.1;
        assert_eq!(raw.bus.clock, managed.bus.clock);
        assert_eq!(raw.bus.framebuffer, managed.bus.framebuffer);
        assert_eq!(raw.bus.wide_framebuffer, managed.bus.wide_framebuffer);
        assert_eq!(raw.bus.oam, managed.bus.oam);
        assert_eq!(raw.bus.audio_buf, managed.bus.audio_buf);
        assert_eq!(raw.bus.psg_tap, managed.bus.psg_tap);
        assert_eq!(raw.bus.fifo_tap, managed.bus.fifo_tap);
        assert!(raw.bus.hle_tap.iter().map(|v| v.to_bits()).eq(managed
            .bus
            .hle_tap
            .iter()
            .map(|v| v.to_bits())));
        for (sum, len) in audio.iter_mut().zip([
            managed.bus.audio_buf.len(),
            managed.bus.psg_tap.len(),
            managed.bus.fifo_tap[0].len(),
            managed.bus.fifo_tap[1].len(),
            managed.bus.hle_tap.len(),
        ]) {
            *sum += len as u64;
        }
        for &(lo, hi) in &ranges {
            managed.bus.rom[lo as usize..hi as usize]
                .copy_from_slice(&rom[lo as usize..hi as usize]);
        }
        assert_eq!(
            raw.save_state().unwrap(),
            managed.save_state().unwrap(),
            "state frame {frame}"
        );
        for machine in [&mut raw, &mut managed] {
            machine.bus.audio_buf.clear();
            machine.bus.psg_tap.clear();
            machine.bus.fifo_tap[0].clear();
            machine.bus.fifo_tap[1].clear();
            machine.bus.hle_tap.clear();
        }
        if (frame + 1) % 40 == 0 {
            println!("E134_DRAW_PAIR frames={}/{}", frame + 1, FRAMES);
        }
    }
    let report = trace.finish();
    assert_eq!(report.dropped_requests, 0);
    let mut typed = 0u64;
    let mut unexpected = 0u64;
    for event in &report.events {
        match event.key.service {
            Service::Typed => typed += event.requests,
            _ => unexpected += event.requests,
        }
    }
    assert_eq!(unexpected, 0, "raw or unclassified selected-bank read");
    assert!(typed > 0 && draw_dispatches > 0 && audio.iter().all(|&n| n > 0));
    let headers = HEADERS.with(|headers| headers.borrow().clone());
    let direct: Vec<_> = selected
        .iter()
        .map(|&i| (i, headers[i].0, headers[i].1))
        .collect();
    assert!(
        direct.iter().all(|(_, _, count)| *count > 0),
        "missing direct header reads: {direct:?}"
    );
    fs::write(
        root.join("trace-report.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"ordinary EUR neutral continuation, sprite patch disabled and observed draw entries supplemented",
        "revision":[170,169], "frames":FRAMES, "observed_draw_instructions":visited.len(), "supplemented_entries":added,
        "selected_headers_and_direct_reads":direct, "native_draw_dispatches":draw_dispatches,
        "native_dispatches":dispatch.0, "fallback_dispatches":dispatch.1,
        "typed_requests":typed, "raw_or_unclassified_requests":unexpected, "trace_dropped_requests":report.dropped_requests,
        "full_state_clock_video_oam_dispatch_audio_equal_each_frame":true, "audio_sample_counts":audio,
        "limits":"Temporary translation supplements observed draw instructions; this is not default production-cache coverage or an exact native load-PC attribution. The final animation hold and other consumers remain open.",
        "performance_measurements":false
    })).unwrap()).unwrap();
}
