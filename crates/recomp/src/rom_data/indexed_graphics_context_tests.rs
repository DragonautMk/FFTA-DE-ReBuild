//! EUR indexed-graphics contexts: publication, source boundaries and original readers.
use super::*;
use gba_core::{Bus, Machine, capi::RT_API};
use gba_core::rom_data::indexed_graphics::{CONTEXT_RECORDS, CONTEXT_SHA256, CONTEXT_SOURCE};
use recomp_core::analyze::{Analysis, Block, View};

#[test]
#[cfg(feature = "rom-data-read-audit")]
#[ignore = "exact EUR ROM, E158 dataset and TinyCC; Wine only, Linux/USA pending"]
fn european_indexed_graphics_contexts_publication_and_original_readers() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    let root = PathBuf::from(std::env::var_os("FFTA_INDEXED_CONTEXT_ROOT").unwrap());
    let previous = PathBuf::from(std::env::var_os("FFTA_INDEXED_CONTEXT_PREVIOUS").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old: Manifest = toml::from_str(&fs::read_to_string(previous.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((old.extractor_revision, old.decoder_revision), (158, 157));
    let mut source = Extraction::from_rom(&rom).unwrap();
    assert_eq!((source.manifest.extractor_revision, source.manifest.decoder_revision), (159, 158));
    let prepared = preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap();
    assert_eq!(prepared.action, "created");
    assert_eq!(preparation::prepare(&root.join("resources"), &mut source, &mut |_| Ok(())).unwrap().action, "reused");
    assert_eq!(source.manifest.coverage, old.coverage);
    for old_resource in &old.resources {
        let new = source.manifest.resources.iter().find(|r| r.id == old_resource.id).unwrap();
        let old_bytes = fs::read(previous.join(&old_resource.path)).unwrap();
        let new_bytes = fs::read(prepared.directory.join(&new.path)).unwrap();
        if old_resource.id != INDEXED_GRAPHICS_ID {
            assert_eq!(new, old_resource);
            assert_eq!(new_bytes, old_bytes);
            continue;
        }
        let old_json: serde_json::Value = serde_json::from_slice(&old_bytes).unwrap();
        let mut new_json: serde_json::Value = serde_json::from_slice(&new_bytes).unwrap();
        assert_eq!(new_json.as_object_mut().unwrap().remove("contexts").unwrap().as_array().unwrap().len(), CONTEXT_RECORDS);
        assert_eq!(new_json, old_json);
        let mut metadata = new.clone();
        let added: Vec<_> = metadata.sources.iter().filter(|s| !old_resource.sources.contains(s)).cloned().collect();
        assert_eq!(added.len(), 1);
        assert_eq!((added[0].offset, added[0].length, added[0].sha256.as_str()), (CONTEXT_SOURCE, 132, CONTEXT_SHA256));
        metadata.sources.retain(|s| old_resource.sources.contains(s));
        metadata.sha256 = old_resource.sha256.clone();
        assert_eq!(&metadata, old_resource);
    }
    let catalog = &source.indexed_graphics.as_ref().unwrap().decoded;
    let start = CONTEXT_SOURCE as usize;
    let end = start + CONTEXT_RECORDS * 4;
    for (i, row) in catalog.contexts.iter().enumerate() {
        assert_eq!(usize::from(row.index), i);
        assert_eq!([row.value, row.secondary_program, row.primary_program, row.opaque], rom[start + 4*i..start + 4*i + 4]);
    }
    let mut invalid = catalog.clone();
    invalid.contexts.pop();
    assert!(invalid.validate().is_err());
    let mut invalid = catalog.clone();
    invalid.contexts[1].index = 0;
    assert!(invalid.validate().is_err());
    let mut edited = catalog.clone();
    edited.contexts[0].opaque ^= 1;
    edited.validate().unwrap();
    assert!(edited.verify_source(&rom).is_err());

    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &prepared.directory).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let before = owner.indexed_graphics_stats().unwrap().context_bytes;
    managed.bus.rom[start..end].fill(0xcd);
    for at in start..end { assert_eq!(managed.bus.read8(0x0c00_0000 + at as u32), rom[at]); }
    assert_eq!(owner.indexed_graphics_stats().unwrap().context_bytes - before, 132);
    let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
    for edge in [start, end] { for delta in [-3isize, -1, 0, 1] {
        let at = edge.wrapping_add_signed(delta);
        assert_eq!((RT_API.read16)(ptr, 0x0a00_0000 + at as u32), u32::from(u16::from_le_bytes(rom[at..at+2].try_into().unwrap())));
        assert_eq!((RT_API.read32)(ptr, 0x0800_0000 + at as u32), u32::from_le_bytes(rom[at..at+4].try_into().unwrap()));
    }}
    managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);

    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let addresses = (0x0801_9670..0x0801_9678).step_by(2)
        .chain((0x0801_972c..0x0801_973c).step_by(2))
        .chain([0x0801_96ac, 0x0801_96c8, 0x0801_9704]);
    let blocks = addresses.map(|addr| Block { addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)] }).collect();
    let object = root.join("indexed-context.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view, &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let initial = raw.save_state().unwrap();
    let mut cases = 0;
    for native in [false, true] { for index in [0u32, 1, 30, 31, 32, 33, 127, 255, 0x120] {
        for m in [&mut raw, &mut managed] {
            m.load_state(&initial).unwrap();
            m.bus.ffta_sprites.enabled = false;
            m.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
            m.cpu.regs[0] = index;
            m.cpu.regs[13] = 0x0300_7c00;
            m.cpu.regs[14] = 0x0800_0301;
            m.cpu.regs[15] = 0x0801_972c;
            m.cpu.cpsr = 0xbf;
        }
        managed.bus.rom[start..end].fill(0xcd);
        let before = owner.indexed_graphics_stats().unwrap().context_bytes;
        let mut steps = 0;
        while raw.cpu.regs[15] != 0x0800_0300 { assert!(steps < 32); raw.step(); steps += 1; }
        steps = 0;
        while managed.cpu.regs[15] != 0x0800_0300 {
            assert!(steps < 32, "getter pc {:08x}", managed.cpu.regs[15]);
            if native { table.get(managed.cpu.regs[15] | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void); }
            else { managed.step(); }
            steps += 1;
        }
        let used = owner.indexed_graphics_stats().unwrap().context_bytes - before;
        assert_eq!(used, u64::from((index & 255) < CONTEXT_RECORDS as u32), "index {index}, native {native}");
        managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
        assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap(), "getter index {index}, native {native}");
        cases += 1;
    }}
    for native in [false, true] { for (pc, field) in [(0x0801_96ac, 2usize), (0x0801_96c8, 1), (0x0801_9704, 1)] {
        for m in [&mut raw, &mut managed] {
            m.load_state(&initial).unwrap();
            m.bus.ffta_sprites.enabled = false;
            m.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
            m.cpu.regs[7] = 0x0800_0000 + CONTEXT_SOURCE + 32 * 4;
            m.cpu.regs[15] = pc;
            m.cpu.cpsr = 0xbf;
        }
        managed.bus.rom[start..end].fill(0xcd);
        let before = owner.indexed_graphics_stats().unwrap().context_bytes;
        raw.step();
        if native { table.get(pc | 1).unwrap()(&RT_API, &mut managed as *mut Machine as *mut std::ffi::c_void); }
        else { managed.step(); }
        assert_eq!(owner.indexed_graphics_stats().unwrap().context_bytes - before, 1);
        assert_eq!(raw.cpu.regs[if field == 2 { 2 } else if pc == 0x0801_96c8 { 2 } else { 0 }], u32::from(rom[start + 32*4 + field]));
        managed.bus.rom[start..end].copy_from_slice(&rom[start..end]);
        assert_eq!(managed.save_state().unwrap(), raw.save_state().unwrap(), "field {field}, pc {pc:08x}, native {native}");
        cases += 1;
    }}
    // This retained ordinary world-map input already reaches context 3. The new
    // check observes its newly owned bytes, rather than replaying the old proof.
    let ordinary_state = fs::read(std::env::var_os("FFTA_INDEXED_CONTEXT_STATE").unwrap()).unwrap();
    let ordinary_input = PathBuf::from(std::env::var_os("FFTA_INDEXED_CONTEXT_INPUT").unwrap());
    assert_eq!(gba_core::ffta_rom::digest(&ordinary_state), "b379038c2a3b6e1740fd70fe78f9f5eaf764d4d2f5eac7fa316d4640e0efb808");
    assert_eq!(gba_core::ffta_rom::digest(&fs::read(&ordinary_input).unwrap()), "f8cbb37acf523ca0d1b2317bc1017b219ee0be0811d2ccfbb09ee32a8ec749ab");
    let input = crate::InputScript::load(&ordinary_input.to_string_lossy()).unwrap();
    let mut ordinary_raw = crate::make_machine(rom.clone(), None);
    let mut ordinary_managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut ordinary_managed, &prepared.directory).unwrap();
    ordinary_raw.load_state(&ordinary_state).unwrap();
    ordinary_managed.load_state(&ordinary_state).unwrap();
    let mut typed_record3 = 0;
    let mut raw_context = 0;
    for frame in 0..11u64 {
        for m in [&mut ordinary_raw, &mut ordinary_managed] {
            m.bus.keys = input.keys_at(frame);
            input.apply_mouse(&mut m.bus, frame).unwrap();
            m.bus.audio_buf.clear();
            m.bus.frame_ready = false;
        }
        for _ in 0..5_000_000 { if ordinary_raw.bus.frame_ready { break; } ordinary_raw.step(); }
        assert!(ordinary_raw.bus.frame_ready);
        let trace = TraceCapture::start(TraceFilter { readers: vec![], ranges: vec![(CONTEXT_SOURCE, CONTEXT_SOURCE + 132)] }, 10_000).unwrap();
        trace.frame(Some(ordinary_managed.bus.frames));
        for _ in 0..5_000_000 { if ordinary_managed.bus.frame_ready { break; } ordinary_managed.step(); }
        assert!(ordinary_managed.bus.frame_ready);
        let report = trace.finish();
        assert!(!report.truncated);
        typed_record3 += report.events.iter().filter(|e| e.key.service == Service::Typed
            && (CONTEXT_SOURCE + 12..CONTEXT_SOURCE + 16).contains(&e.key.read.physical_offset)).map(|e| e.requests).sum::<u64>();
        raw_context += report.events.iter().filter(|e| e.key.service == Service::Raw).map(|e| e.requests).sum::<u64>();
        assert_eq!(ordinary_managed.save_state().unwrap(), ordinary_raw.save_state().unwrap(), "ordinary frame {frame}");
        assert!(ordinary_managed.bus.framebuffer == ordinary_raw.bus.framebuffer
            && ordinary_managed.bus.wide_framebuffer == ordinary_raw.bus.wide_framebuffer
            && ordinary_managed.bus.oam == ordinary_raw.bus.oam
            && ordinary_managed.bus.audio_buf == ordinary_raw.bus.audio_buf, "ordinary output frame {frame}");
    }
    assert!(typed_record3 > 0, "retained ordinary menu must read context 3");
    assert_eq!(raw_context, 0);
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[159,158], "previous_resources_preserved":true,
        "context_bytes":132, "context_records":33, "source_sha256":CONTEXT_SHA256,
        "poisoned_raw_rom":true, "original_reader_cases":cases,
        "compiled_dispatch_cases":"included; may use interpreter fallback", "full_state_clock_equal":true,
        "ordinary_interpreter_frames":11, "ordinary_context3_typed_requests":typed_record3,
        "ordinary_context_raw_requests":raw_context,
        "Linux":"pending", "USA":"pending", "native_windows":"pending",
        "performance_measurements":false
    })).unwrap()).unwrap();
}
