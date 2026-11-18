//! EUR callback descriptor publication and controlled original dispatch proof.
use super::*;
use gba_core::{
    capi::RT_API,
    rom_data::context_callback_descriptor::{
        ContextCallbackDescriptor, CONSTRUCTOR_LITERAL, DESCRIPTOR_OFFSET,
    },
    Bus, Machine,
};
use recomp_core::analyze::{Analysis, Block, View};
use std::collections::BTreeSet;

#[test]
#[ignore = "exact EUR ROM, e166 dataset and TinyCC; Wine only, Linux/USA pending"]
fn european_context_callback_descriptor_publication_and_dispatch() {
    const START: u32 = 0x0814_4d24;
    const CALL: u32 = 0x0814_4db2;
    const CALLBACK: u32 = 0x0814_49fc;
    const RETURN: u32 = 0x0300_7ff0;
    const OBJECT: u32 = 0x0201_0000;
    let root = PathBuf::from(std::env::var_os("FFTA_CONTEXT_CALLBACK_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_CONTEXT_CALLBACK_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest =
        toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (166, 165));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!(
        (
            source.manifest.extractor_revision,
            source.manifest.decoder_revision
        ),
        (167, 166)
    );
    let prepared =
        preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(
        preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(()))
            .unwrap()
            .action,
        "reused"
    );
    assert_eq!(source.manifest.resources.len(), old.resources.len() + 1);
    for prior in &old.resources {
        let current = source
            .manifest
            .resources
            .iter()
            .find(|r| r.id == prior.id)
            .unwrap();
        assert_eq!(current, prior);
        assert_eq!(
            fs::read(prepared.directory.join(&current.path)).unwrap(),
            fs::read(previous.join(&prior.path)).unwrap()
        );
    }
    let catalog = source.context_callback_descriptor.as_ref().unwrap();
    let model = &catalog.decoded;
    let entry = source
        .manifest
        .resources
        .iter()
        .find(|r| r.id == CONTEXT_CALLBACK_DESCRIPTOR_ID)
        .unwrap();
    assert_eq!(entry.sources, catalog.sources);
    let decoded: ContextCallbackDescriptor =
        serde_json::from_slice(&serde_json::to_vec(model).unwrap()).unwrap();
    assert_eq!(&decoded, model);
    let mut invalid = decoded.clone();
    invalid.callback_thumb_pointer &= !1;
    assert!(invalid.validate().is_err());
    let spans = model.source_spans().unwrap();
    assert_eq!(spans.iter().map(|s| s.1.len()).sum::<usize>(), 12);
    for (at, bytes, _) in &spans {
        assert_eq!(bytes, &rom[*at as usize..*at as usize + bytes.len()]);
    }

    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let poison = |m: &mut Machine| {
        for (at, bytes, _) in &spans {
            m.bus.rom[*at as usize..*at as usize + bytes.len()].fill(0xcd);
        }
    };
    let restore = |m: &mut Machine| {
        for (at, bytes, _) in &spans {
            m.bus.rom[*at as usize..*at as usize + bytes.len()]
                .copy_from_slice(&rom[*at as usize..*at as usize + bytes.len()]);
        }
    };
    poison(&mut managed);
    let before = owner.context_callback_descriptor_stats().unwrap();
    for (at, bytes, _) in &spans {
        for (i, expected) in bytes.iter().enumerate() {
            assert_eq!(managed.bus.read8(0x0800_0000 + *at + i as u32), *expected);
        }
    }
    let after = owner.context_callback_descriptor_stats().unwrap();
    assert_eq!(
        (
            after.source_bytes - before.source_bytes,
            after.constructor_literal_bytes - before.constructor_literal_bytes
        ),
        (12, 4)
    );
    assert_eq!(
        (
            after.callback_bytes - before.callback_bytes,
            after.delay_bytes - before.delay_bytes,
            after.flags_bytes - before.flags_bytes,
            after.opaque_bytes - before.opaque_bytes
        ),
        (4, 2, 1, 1)
    );
    let adjacent = DESCRIPTOR_OFFSET as usize + 8;
    managed.bus.rom[adjacent..adjacent + 3].fill(0xcd);
    let old_before = owner.direct_indexed_graphics_stats().unwrap().source_bytes;
    let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for edge in [
        CONSTRUCTOR_LITERAL as usize,
        DESCRIPTOR_OFFSET as usize,
        DESCRIPTOR_OFFSET as usize + 8,
    ] {
        for delta in [-3isize, -1, 0, 1] {
            let at = edge.wrapping_add_signed(delta);
            assert_eq!(
                (RT_API.read16)(ptr, 0x0a00_0000 + at as u32),
                u32::from(u16::from_le_bytes(rom[at..at + 2].try_into().unwrap()))
            );
            assert_eq!(
                (RT_API.read32)(ptr, 0x0800_0000 + at as u32),
                u32::from_le_bytes(rom[at..at + 4].try_into().unwrap())
            );
        }
    }
    assert!(owner.direct_indexed_graphics_stats().unwrap().source_bytes > old_before);
    managed.bus.rom[adjacent..adjacent + 3].copy_from_slice(&rom[adjacent..adjacent + 3]);
    restore(&mut managed);

    let configure = |m: &mut Machine| {
        m.bus.ewram.fill(0x5a);
        m.bus.ewram[0x10000..0x10400].fill(0);
        m.bus.iwram.fill(0);
        m.bus.ffta_sprites.enabled = false;
        m.bus.reg_ie = 0;
        m.bus.ime = false;
        m.bus.frame_ready = false;
        m.bus.write16(OBJECT + 0x2ac, 4);
        m.bus
            .write32(OBJECT + 0x2f8, 0x0800_0000 + DESCRIPTOR_OFFSET);
        m.bus.write16(OBJECT + 0x2fc, 1);
        m.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
        m.cpu.regs[0] = OBJECT;
        m.cpu.regs[13] = 0x0300_7c00;
        m.cpu.regs[14] = RETURN | 1;
        m.cpu.regs[15] = START;
        m.cpu.cpsr = 0xbf;
    };
    let assert_call = |m: &Machine| {
        assert_eq!(&m.cpu.regs[..3], &[OBJECT + 4, 0, 0x0814_49fd]);
    };
    let mut trace = crate::make_machine(rom.clone(), None);
    let initial = trace.save_state().unwrap();
    configure(&mut trace);
    let mut visited = BTreeSet::new();
    let mut saw_call = false;
    for _ in 0..10000 {
        if trace.cpu.regs[15] == RETURN {
            break;
        }
        if trace.cpu.regs[15] == CALL {
            assert_call(&trace);
            saw_call = true;
        }
        visited.insert((trace.cpu.regs[15], trace.cpu.thumb()));
        trace.bus.frame_ready = false;
        trace.step();
    }
    assert!(saw_call);
    assert_eq!(trace.cpu.regs[15], RETURN, "original callback dispatcher");
    assert_eq!(trace.cpu.regs[0], 0);
    assert_eq!(trace.bus.read8(OBJECT + 0x2f4), 1);
    let view = View {
        rom: &rom,
        ewram: None,
        iwram: Some(&trace.bus.iwram),
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
            Block {
                addr: address,
                thumb,
                instrs: vec![first],
            }
        })
        .collect();
    let object = root.join("context-callback-native.o");
    recomp_core::tcc_memory::build(
        &Analysis { blocks },
        &view,
        &recomp_core::tcc_memory::library_path().unwrap(),
        &object,
        &mut |_| {},
    )
    .unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut comparisons = 0;
    for native in [false, true] {
        let mut raw = crate::make_machine(rom.clone(), None);
        raw.load_state(&initial).unwrap();
        managed.load_state(&initial).unwrap();
        configure(&mut raw);
        configure(&mut managed);
        poison(&mut managed);
        let before = owner.context_callback_descriptor_stats().unwrap();
        for _ in 0..10000 {
            if raw.cpu.regs[15] == RETURN {
                break;
            }
            raw.bus.frame_ready = false;
            raw.step();
        }
        assert_eq!(raw.cpu.regs[15], RETURN);
        let mut steps = 0;
        let mut saw_call = false;
        let mut saw_callback_entry = false;
        while managed.cpu.regs[15] != RETURN {
            if managed.cpu.regs[15] == CALL {
                assert_call(&managed);
                saw_call = true;
            }
            if managed.cpu.regs[15] == CALLBACK {
                assert_call(&managed);
                assert_eq!(managed.cpu.regs[14], 0x0814_4db7);
                assert!(managed.cpu.thumb());
                saw_callback_entry = true;
            }
            managed.bus.frame_ready = false;
            if native {
                table
                    .get(managed.cpu.regs[15] | u32::from(managed.cpu.thumb()))
                    .expect("compiled callback instruction")(
                    &RT_API,
                    &mut managed as *mut Machine as *mut std::ffi::c_void,
                );
            } else {
                managed.step();
            }
            steps += 1;
            assert!(
                steps < 10000,
                "managed callback PC {:08x}",
                managed.cpu.regs[15]
            );
        }
        assert!(native || saw_call, "interpreter call boundary missing");
        assert!(
            saw_callback_entry,
            "callback entry missing; native={native}, saw_call={saw_call}"
        );
        let after = owner.context_callback_descriptor_stats().unwrap();
        assert!(after.callback_bytes - before.callback_bytes >= 4);
        assert!(after.delay_bytes - before.delay_bytes >= 2);
        assert!(after.flags_bytes - before.flags_bytes >= 1);
        assert_eq!(after.opaque_bytes - before.opaque_bytes, 0);
        assert_eq!(managed.cpu.regs[0], 0);
        assert_eq!(managed.bus.read8(OBJECT + 0x2f4), 1);
        restore(&mut managed);
        assert_eq!(
            raw.save_state().unwrap(),
            managed.save_state().unwrap(),
            "compiled {native}"
        );
        comparisons += 1;
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"EUR one-row context callback publication and controlled original dispatch",
        "revision":[167,166], "previous_resources_preserved":old.resources.len(), "source_bytes":12,
        "source_rom_poisoned":true, "boundaries_and_json_guards":true,
        "original_dispatch_comparisons":comparisons, "callback_arguments_checked":true, "status_byte":1,
        "full_state_clock_equal":true, "opaque_byte":"preserved by direct read; not read by controlled dispatch",
        "compiled_dispatch":"instruction blocks may use interpreter fallback", "ordinary_reachability":"pending",
        "remaining_halfword":"3ae472..3ae474 unclassified", "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "authenticated EUR callback checkpoint, current dataset and native library; Wine only"]
fn european_context_callback_ordinary_poisoned_audio_pair() {
    use gba_core::rom_data::context_callback_descriptor::{CONSTRUCTOR_LITERAL, DESCRIPTOR_OFFSET};
    crate::configure_headless_widescreen();
    let root = PathBuf::from(std::env::var_os("FFTA_CONTEXT_CALLBACK_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let state = fs::read(std::env::var_os("FFTA_STATE_TEST_STATE").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&state),
        "9d68a0e85fecf7c8d9f314d92f21773cf5d3d5648ea051988185a2e28801e74d"
    );
    let manifest: Manifest =
        toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!(
        (manifest.extractor_revision, manifest.decoder_revision),
        (170, 169)
    );
    let (_library, table) = crate::load_native(Path::new(
        &std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap(),
    ))
    .unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &dataset).unwrap();
    for m in [&mut raw, &mut managed] {
        m.load_state(&state).unwrap();
        crate::arm_audio_hle(m, None);
        m.bus.tap_channels = true;
        m.bus.audio_buf.clear();
        m.bus.psg_tap.clear();
        m.bus.fifo_tap[0].clear();
        m.bus.fifo_tap[1].clear();
        m.bus.hle_tap.clear();
    }
    assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
    let spans = [
        (CONSTRUCTOR_LITERAL as usize, 4usize),
        (DESCRIPTOR_OFFSET as usize, 8usize),
    ];
    let owner = managed.bus.rom_resources().unwrap().clone();
    let before = owner
        .context_callback_descriptor_stats()
        .unwrap()
        .source_bytes;
    let start = raw.bus.frames;
    let mut samples = [0usize; 5];
    for frame in 0..3 {
        for (at, len) in spans {
            managed.bus.rom[at..at + len].fill(0xcd);
        }
        let mut dispatch = [(0, 0); 2];
        for (arm, m) in [&mut raw, &mut managed].into_iter().enumerate() {
            m.bus.keys = 0x03ff;
            let pointer = m as *mut Machine as *mut std::ffi::c_void;
            dispatch[arm] = crate::run_frame_native(m, &table, pointer, crate::FRAME_STEP_GUARD);
            assert_eq!(m.bus.frames, start + frame + 1);
        }
        assert_eq!(dispatch[0], dispatch[1]);
        assert_eq!(raw.bus.clock, managed.bus.clock);
        assert_eq!(raw.bus.framebuffer, managed.bus.framebuffer);
        assert_eq!(raw.bus.wide_framebuffer, managed.bus.wide_framebuffer);
        assert_eq!(raw.bus.oam, managed.bus.oam);
        assert_eq!(raw.bus.audio_buf, managed.bus.audio_buf);
        assert_eq!(raw.bus.psg_tap, managed.bus.psg_tap);
        assert_eq!(raw.bus.fifo_tap, managed.bus.fifo_tap);
        assert_eq!(raw.bus.hle_tap.len(), managed.bus.hle_tap.len());
        assert!(raw
            .bus
            .hle_tap
            .iter()
            .zip(&managed.bus.hle_tap)
            .all(|(a, b)| a.to_bits() == b.to_bits()));
        for (count, len) in samples.iter_mut().zip([
            managed.bus.audio_buf.len(),
            managed.bus.psg_tap.len(),
            managed.bus.fifo_tap[0].len(),
            managed.bus.fifo_tap[1].len(),
            managed.bus.hle_tap.len(),
        ]) {
            *count += len;
        }
        for (at, len) in spans {
            managed.bus.rom[at..at + len].copy_from_slice(&rom[at..at + len]);
        }
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
        for m in [&mut raw, &mut managed] {
            m.bus.audio_buf.clear();
            m.bus.psg_tap.clear();
            m.bus.fifo_tap[0].clear();
            m.bus.fifo_tap[1].clear();
            m.bus.hle_tap.clear();
        }
    }
    let source_bytes = owner
        .context_callback_descriptor_stats()
        .unwrap()
        .source_bytes
        - before;
    assert!(
        source_bytes > 0,
        "ordinary scene must consume the descriptor"
    );
    assert!(samples[0] > 0, "mixed audio must contain samples");
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"three frames from authenticated EUR ordinary callback checkpoint, Wine",
        "revision":[170,169], "source_bytes":source_bytes, "poisoned_bytes":12,
        "full_state_clock_framebuffer_wide_oam_audio_equal_each_frame":true, "audio_sample_counts":samples,
        "dispatch":"generated blocks may fall back to interpretation", "Linux":"pending", "USA":"pending",
        "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
