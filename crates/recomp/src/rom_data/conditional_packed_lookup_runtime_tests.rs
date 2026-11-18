//! Managed EUR packed lookup, boundaries and original native reader prefixes.
use super::*;
use gba_core::{Bus, Machine, capi::RT_API};
use gba_core::rom_data::conditional_packed_lookup::SOURCE_OFFSET;
use recomp_core::analyze::{Analysis, Block, View};
use std::collections::BTreeSet;

#[test]
#[ignore = "private EUR ROM and TinyCC; Wine only, Linux/USA pending"]
fn european_conditional_packed_managed_native_reads() {
    let root=PathBuf::from(std::env::var_os("FFTA_PACKED_RUNTIME_ROOT").unwrap());
    assert!(!root.exists());fs::create_dir_all(&root).unwrap();
    let rom=fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let old=PathBuf::from(std::env::var_os("FFTA_STATE_TEST_DATASET").unwrap());
    let prior:Manifest=toml::from_str(&fs::read_to_string(old.join(MANIFEST_FILE)).unwrap()).unwrap();
    let mut source=Extraction::from_rom(&rom).unwrap();let dataset=root.join("offline dataset");extract(&dataset,&mut source).unwrap();
    assert_eq!((source.manifest.extractor_revision,source.manifest.decoder_revision,source.manifest.resources.len()),(130, 129, 101));
    assert_eq!(prior.resources.len(),86);
    for r in &prior.resources {let current=source.manifest.resources.iter().find(|e|e.id==r.id).unwrap();super::conditional_marker_runtime_tests::assert_previous_resource_files(current,r,&dataset,&old);}
    let entry=&source.manifest.resources[86];assert_eq!(entry.id,CONDITIONAL_PACKED_LOOKUP_ID);
    assert_eq!((entry.sources.len(),entry.sources[0].offset,entry.sources[0].length),(1,SOURCE_OFFSET,132));
    let bytes=fs::read(dataset.join(CONDITIONAL_PACKED_LOOKUP_FILE)).unwrap();
    assert_eq!(parse_conditional_packed_lookup(&bytes).unwrap(),source.conditional_packed_lookup.as_ref().unwrap().decoded);
    let manifest=fs::read(dataset.join(MANIFEST_FILE)).unwrap();let start=SOURCE_OFFSET as usize;let alias=0x6e72d0usize;
    let mut machine=crate::make_machine(rom.clone(),None);attach_from_directory(&mut machine,&dataset).unwrap();
    let state=machine.save_state().unwrap();let owner=machine.bus.rom_resources().unwrap().clone();
    machine.bus.rom[start..start+132].fill(0xcd);
    machine.bus.rom[start+132..start+136].fill(0xa7);
    machine.bus.rom[alias..alias+4].fill(0xa9);
    for at in start..start+132 {assert_eq!(machine.bus.read8(0x0c000000+at as u32),rom[at],"byte {at:06x}");}
    for edge in [start,start+132] {for delta in [-3isize,-1,0,1] {
        let at=(edge as isize+delta) as usize;let ptr=&mut machine as *mut Machine as *mut std::ffi::c_void;
        assert_eq!((RT_API.read16)(ptr,0x0a000000+at as u32),u32::from(u16::from_le_bytes(rom[at..at+2].try_into().unwrap())));
        assert_eq!((RT_API.read32)(ptr,0x08000000+at as u32),u32::from_le_bytes(rom[at..at+4].try_into().unwrap()));
    }}
    for at in alias..alias+4 {assert_eq!(machine.bus.read8(0x08000000+at as u32),rom[at],"prior metadata14 alias");}
    machine.bus.rom.copy_from_slice(&rom);assert_eq!(machine.save_state().unwrap(),state);
    let mut bad=parse_conditional_packed_lookup(&bytes).unwrap();bad.records[0].byte0^=1;
    let bad_bytes=encode_conditional_packed_lookup(&bad).unwrap();let mut bad_manifest=source.manifest.clone();bad_manifest.resources[86].sha256=gba_core::ffta_rom::digest(&bad_bytes);
    fs::write(dataset.join(CONDITIONAL_PACKED_LOOKUP_FILE),bad_bytes).unwrap();fs::write(dataset.join(MANIFEST_FILE),toml::to_string_pretty(&bad_manifest).unwrap()).unwrap();
    assert!(attach_from_directory(&mut machine,&dataset).is_err());assert!(std::sync::Arc::ptr_eq(machine.bus.rom_resources().unwrap(),&owner));assert_eq!(machine.save_state().unwrap(),state);
    fs::write(dataset.join(CONDITIONAL_PACKED_LOOKUP_FILE),bytes).unwrap();fs::write(dataset.join(MANIFEST_FILE),manifest).unwrap();
    let prefixes=[(0x132046u32,0x132068u32),(0x134176,0x13418c)];let mut visited=BTreeSet::new();
    for (start,end) in prefixes {for at in (start..end).step_by(2) {visited.insert(0x08000000+at);}}
    let view=View {rom:&rom,ewram:None,iwram:None,bios:None};
    let blocks=visited.iter().map(|&addr|Block {addr,thumb:true,instrs:vec![armv4t::decode_thumb(view.read16(addr),addr)]}).collect();
    let object=root.join("packed-native.o");recomp_core::tcc_memory::build(&Analysis {blocks},&view,&recomp_core::tcc_memory::library_path().unwrap(),&object,&mut |_| {}).unwrap();
    let (_library,table)=crate::load_native(&object).unwrap();let mut raw=crate::make_machine(rom.clone(),None);let mut native=crate::make_machine(rom.clone(),None);attach_from_directory(&mut native,&dataset).unwrap();
    let initial=raw.save_state().unwrap();let native_owner=native.bus.rom_resources().unwrap().clone();let mut served=0;let cases=[(0u8,0u8,0u16,0u8),(1,11,1,5),(5,65,33,5),(0,1,251,1)];
    for (prefix,(begin,end)) in prefixes.into_iter().enumerate() {for (bank,row,index,kind) in cases {
        let ptr=0x6df4d4+usize::from(bank)*4;let descriptor=u32::from_le_bytes(rom[ptr..ptr+4].try_into().unwrap())+u32::from(row)*8;
        for m in [&mut raw,&mut native] {m.load_state(&initial).unwrap();m.bus.ffta_sprites.enabled=false;m.cpu.regs=std::array::from_fn(|i|0x11000000+i as u32);m.cpu.regs[13]=0x03007c00;m.cpu.regs[0]=descriptor;m.cpu.regs[15]=0x08000000+begin;m.cpu.cpsr=0xbf;}
        native.bus.rom[start..start+132].fill(0xcd);native.bus.rom[alias..alias+4].fill(0xa9);
        let stop=if prefix==1 && kind!=5 {0x081341cc} else {0x08000000+end};
        while raw.cpu.regs[15]!=stop {raw.bus.frame_ready=false;raw.step();}
        let before=native_owner.conditional_packed_lookup_stats().unwrap().source_bytes;
        let prior_before=native_owner.metadata14_stats().unwrap().source_bytes;
        let mut dispatches=0;while native.cpu.regs[15]!=stop {assert!(dispatches<24,"pc {:08x}",native.cpu.regs[15]);native.bus.frame_ready=false;table.get(native.cpu.regs[15]|1).unwrap()(&RT_API,&mut native as *mut Machine as *mut std::ffi::c_void);dispatches+=1;}
        let used=native_owner.conditional_packed_lookup_stats().unwrap().source_bytes-before;
        let prior_used=native_owner.metadata14_stats().unwrap().source_bytes-prior_before;
        assert_eq!(used,if kind==5 {if prefix==0 {2} else {1}} else {0},"index {index} prefix {prefix}");
        if index==251 && prefix==0 {assert_eq!(prior_used,2);}
        served+=used;native.bus.rom.copy_from_slice(&rom);
        assert_eq!(native.save_state().unwrap(),raw.save_state().unwrap(),"index {index} prefix {prefix}");
    }}
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","resources":source.manifest.resources.len(),"previous_unchanged":prior.resources.len()-1,"previous_obj_projection_preserved":true,"source_bytes":132,"native_prefixes":2,"descriptor_indices":[0,1,33,251],"typed_native_bytes":served,"metadata14_alias":true,"full_state_clock":true,"tamper_transaction":true,"Linux":"pending","USA":"pending","nativeWindows":"pending","performance_measurements":false})).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "exact EUR ROM, current dataset and TinyCC; Wine only"]
fn european_encounter_special_descriptor_packed_alias() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};

    const PREFIX: u32 = 0x080c_d42e;
    const STOP: u32 = 0x080c_d43e;
    const ROW1: u32 = 0x006e_6f5e;
    let root = PathBuf::from(std::env::var_os("FFTA_SPECIAL_ALIAS_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_SPECIAL_ALIAS_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);

    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &dataset).unwrap();
    let templates = parse_encounter_unit_templates(&fs::read(dataset.join(ENCOUNTER_UNIT_TEMPLATES_FILE)).unwrap()).unwrap();
    let special_count = templates.records.iter().filter(|record| record.byte00 == 1).count();
    let template = templates.records.iter().find(|record| record.byte00 == 1).expect("no source-backed special descriptor");
    assert_eq!(rom[template.source_offset as usize], 1);
    assert_eq!(ROW1, gba_core::rom_data::metadata14::ADDRESS_BIAS + 14);
    assert!(ROW1 >= SOURCE_OFFSET && ROW1 + 14 <= SOURCE_OFFSET + 132);

    let view = View { rom: &rom, ewram: None, iwram: None, bios: None };
    let blocks = (PREFIX..STOP).step_by(2).map(|addr| Block {
        addr, thumb: true, instrs: vec![armv4t::decode_thumb(view.read16(addr), addr)],
    }).collect();
    let object = root.join("special-descriptor-alias.o");
    recomp_core::tcc_memory::build(&Analysis { blocks }, &view,
        &recomp_core::tcc_memory::library_path().unwrap(), &object, &mut |_| {}).unwrap();
    let (_library, table) = crate::load_native(&object).unwrap();

    let mut raw = crate::make_machine(rom.clone(), None);
    let initial = raw.save_state().unwrap();
    let setup = |machine: &mut Machine| {
        machine.bus.ffta_sprites.enabled = false;
        machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
        machine.cpu.regs[8] = 0x0800_0000 + template.source_offset;
        machine.cpu.regs[13] = 0x0300_7c00;
        machine.cpu.regs[15] = PREFIX;
        machine.cpu.cpsr = 0xbf;
        machine.bus.frame_ready = false;
    };
    raw.load_state(&initial).unwrap();
    setup(&mut raw);
    while raw.cpu.regs[15] != STOP { raw.step(); }
    assert_eq!(raw.cpu.regs[1], 0x0800_0000 + ROW1);
    assert_eq!(raw.cpu.regs[0], u32::from(rom[ROW1 as usize]));
    let expected = raw.save_state().unwrap();
    let mut observations = Vec::new();

    for native in [false, true] {
        managed.load_state(&initial).unwrap();
        setup(&mut managed);
        managed.bus.rom[template.source_offset as usize] = 0xcd;
        managed.bus.rom[ROW1 as usize..ROW1 as usize + 14].fill(0xa7);
        let owner = managed.bus.rom_resources().unwrap().clone();
        let before_packed = owner.conditional_packed_lookup_stats().unwrap().source_bytes;
        let before_template = owner.encounter_unit_templates_stats().unwrap().source_bytes;
        let before_metadata14 = owner.metadata14_stats().unwrap().source_bytes;
        let capture = TraceCapture::start(TraceFilter { readers: vec![], ranges: vec![
            (template.source_offset, template.source_offset + 1), (ROW1, ROW1 + 14),
        ] }, 100).unwrap();
        capture.frame(Some(managed.bus.frames));
        let ptr = &mut managed as *mut Machine as *mut std::ffi::c_void;
        let mut steps = 0;
        while managed.cpu.regs[15] != STOP {
            assert!(steps < 16, "original prefix did not reach its continuation");
            if native { table.get(managed.cpu.regs[15] | 1).unwrap()(&RT_API, ptr); }
            else { managed.step(); }
            steps += 1;
        }
        let trace = capture.finish();
        assert!(!trace.truncated);
        let typed_row: u64 = trace.events.iter().filter(|event| event.key.service == Service::Typed
            && event.key.read.physical_offset == ROW1).map(|event| event.requests).sum();
        let raw_reads: u64 = trace.events.iter().filter(|event| event.key.service == Service::Raw)
            .map(|event| event.requests).sum();
        let packed_bytes = owner.conditional_packed_lookup_stats().unwrap().source_bytes - before_packed;
        let template_bytes = owner.encounter_unit_templates_stats().unwrap().source_bytes - before_template;
        let metadata14_bytes = owner.metadata14_stats().unwrap().source_bytes - before_metadata14;
        assert_eq!((typed_row, raw_reads, packed_bytes, template_bytes, metadata14_bytes),
            (1, 0, 1, 1, 0), "typed alias accounting, native={native}");
        managed.bus.rom[template.source_offset as usize] = rom[template.source_offset as usize];
        managed.bus.rom[ROW1 as usize..ROW1 as usize + 14].copy_from_slice(&rom[ROW1 as usize..ROW1 as usize + 14]);
        assert!(managed.save_state().unwrap() == expected, "original prefix state/clock mismatch, native={native}");
        observations.push(serde_json::json!({"native":native,"steps":steps,"typed_row_requests":typed_row,
            "raw_requests":raw_reads,"packed_bytes":packed_bytes,"template_bytes":template_bytes,
            "metadata14_bytes":metadata14_bytes,"full_state_clock_equal":true}));
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"controlled original EUR encounter-importer prefix using an authenticated descriptor-1 template",
        "special_templates":special_count, "template_index":template.index, "header_index":template.header_index,
        "row1_source_offset":ROW1, "observations":observations, "source_rom_poisoned":true,
        "limitations":"Prefix evidence only; ordinary invocation, full importer behavior and all alias consumers remain unproved.",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "authenticated EUR battle state/input and current dataset; Wine only"]
fn european_encounter_special_descriptor_ordinary_observation() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};

    const ROW1: usize = 0x006e_6f5e;
    const READER: u32 = 0x080c_d43c;
    let root = PathBuf::from(std::env::var_os("FFTA_SPECIAL_ALIAS_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_SPECIAL_ALIAS_DATASET").unwrap());
    let state = fs::read(std::env::var_os("FFTA_STATE_TEST_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "e91b8c594d3b7f10d6b815a61cc13943a7c86ea17a7c4cfc865f48eaaf37d84f");
    let input_path = std::env::var("FFTA_STATE_TEST_INPUT").unwrap();
    let input_bytes = fs::read(&input_path).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&input_bytes), "ca9b84ef9dbeb8d71c47552e9455a2c3fb636a0f7a4219196ce8d82270bf0ad9");
    let input = crate::InputScript::load(&input_path).unwrap();
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &dataset).unwrap();
    managed.load_state(&state).unwrap();
    let first_frame = managed.bus.frames;
    let owner = managed.bus.rom_resources().unwrap().clone();
    let mut outcome = None;
    let mut typed_total = 0u64;

    for frame in 0..700u64 {
        managed.bus.keys = input.keys_at(frame);
        input.apply_mouse(&mut managed.bus, frame).unwrap();
        managed.bus.audio_buf.clear();
        managed.bus.psg_tap.clear();
        for channel in &mut managed.bus.fifo_tap { channel.clear(); }
        managed.bus.hle_tap.clear();
        let before = managed.save_state().unwrap();
        managed.bus.rom[ROW1..ROW1 + 14].fill(0xa7);
        let capture = TraceCapture::start(TraceFilter { readers: vec![], ranges: vec![(ROW1 as u32, ROW1 as u32 + 14)] }, 100).unwrap();
        capture.frame(Some(managed.bus.frames));
        managed.bus.frame_ready = false;
        let mut reader_seen = false;
        for _ in 0..5_000_000 {
            if managed.bus.frame_ready { break; }
            if managed.cpu.regs[15] == READER && managed.cpu.regs[1] == 0x0800_0000 + ROW1 as u32 {
                reader_seen = true;
            }
            managed.step();
        }
        assert!(managed.bus.frame_ready, "ordinary alias frame limit");
        let trace = capture.finish();
        assert!(!trace.truncated);
        let typed: u64 = trace.events.iter().filter(|event| event.key.service == Service::Typed)
            .map(|event| event.requests).sum();
        let raw_reads: u64 = trace.events.iter().filter(|event| event.key.service == Service::Raw)
            .map(|event| event.requests).sum();
        typed_total += typed;
        managed.bus.rom[ROW1..ROW1 + 14].copy_from_slice(&rom[ROW1..ROW1 + 14]);
        assert_eq!(raw_reads, 0, "raw fallback within selected row");
        if reader_seen {
            assert!(trace.events.iter().any(|event| event.key.service == Service::Typed
                && event.key.read.physical_offset == ROW1 as u32
                && event.key.instruction.is_some_and(|site| site.pc == READER)),
                "original reader was reached without its typed row trace");
            let mut raw = crate::make_machine(rom.clone(), None);
            raw.load_state(&before).unwrap();
            raw.bus.frame_ready = false;
            for _ in 0..5_000_000 { if raw.bus.frame_ready { break; } raw.step(); }
            assert!(raw.bus.frame_ready);
            assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "ordinary alias full state/clock");
            assert!(raw.bus.framebuffer == managed.bus.framebuffer && raw.bus.wide_framebuffer == managed.bus.wide_framebuffer
                && raw.bus.oam == managed.bus.oam && raw.bus.audio_buf == managed.bus.audio_buf,
                "ordinary alias image/OAM/mixed audio");
            outcome = Some(serde_json::json!({"relative_frame":frame,"guest_frame":first_frame + frame,
                "typed_row_requests_in_frame":typed,"raw_row_requests_in_frame":raw_reads,
                "paired_full_state_clock_image_oam_mixed_audio_frames":1}));
            break;
        }
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"bounded EUR Giza-to-White-Flowers input, interpreter, up to 700 frames",
        "reader_hit":outcome, "typed_row_requests_total":typed_total,
        "packed_source_bytes_total":owner.conditional_packed_lookup_stats().unwrap().source_bytes,
        "source_rom_poisoned":true, "negative_is_not_global_absence":true,
        "limitations":"One retained route; no all-consumer, all-scene or translated-dispatch claim.",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "authenticated EUR battle state/input, current dataset and native library; Wine only"]
fn european_descriptor_alias_battle_translated_pair() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    crate::configure_headless_widescreen();
    const ROW: usize = 0x6e6f5e;
    let root = PathBuf::from(std::env::var_os("FFTA_SPECIAL_ALIAS_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_SPECIAL_ALIAS_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&rom),
        gba_core::ffta_rom::EUROPE_SHA256
    );
    let state = fs::read(std::env::var_os("FFTA_STATE_TEST_STATE").unwrap()).unwrap();
    assert_eq!(
        gba_core::ffta_rom::digest(&state),
        "e91b8c594d3b7f10d6b815a61cc13943a7c86ea17a7c4cfc865f48eaaf37d84f"
    );
    let input_path = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_INPUT").unwrap());
    assert_eq!(
        gba_core::ffta_rom::digest(&fs::read(&input_path).unwrap()),
        "ca9b84ef9dbeb8d71c47552e9455a2c3fb636a0f7a4219196ce8d82270bf0ad9"
    );
    let input = crate::InputScript::load(input_path.to_str().unwrap()).unwrap();
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
    let owner = managed.bus.rom_resources().unwrap().clone();
    let before = owner
        .conditional_packed_lookup_stats()
        .unwrap()
        .source_bytes;
    let trace = TraceCapture::start(
        TraceFilter {
            readers: vec![],
            ranges: vec![(ROW as u32, (ROW + 14) as u32)],
        },
        10000,
    )
    .unwrap();
    let start = raw.bus.frames;
    let mut audio = [0u64; 5];
    let mut dispatch = (0u64, 0u64);
    for frame in 0..240u64 {
        managed.bus.rom[ROW..ROW + 14].fill(0xa7);
        let mut counts = [(0, 0); 2];
        for (arm, m) in [&mut raw, &mut managed].into_iter().enumerate() {
            m.bus.keys = input.keys_at(frame);
            input.apply_mouse(&mut m.bus, frame).unwrap();
            trace.frame(if arm == 1 { Some(m.bus.frames) } else { None });
            let ptr = m as *mut Machine as *mut std::ffi::c_void;
            counts[arm] = crate::run_frame_native(m, &table, ptr, crate::FRAME_STEP_GUARD);
            assert_eq!(m.bus.frames, start + frame + 1);
        }
        trace.frame(None);
        assert_eq!(counts[0], counts[1], "dispatch at {frame}");
        dispatch.0 += counts[0].0;
        dispatch.1 += counts[0].1;
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
        for (sum, len) in audio.iter_mut().zip([
            managed.bus.audio_buf.len(),
            managed.bus.psg_tap.len(),
            managed.bus.fifo_tap[0].len(),
            managed.bus.fifo_tap[1].len(),
            managed.bus.hle_tap.len(),
        ]) {
            *sum += len as u64;
        }
        managed.bus.rom[ROW..ROW + 14].copy_from_slice(&rom[ROW..ROW + 14]);
        assert_eq!(
            raw.save_state().unwrap(),
            managed.save_state().unwrap(),
            "full state at {frame}"
        );
        for m in [&mut raw, &mut managed] {
            m.bus.audio_buf.clear();
            m.bus.psg_tap.clear();
            m.bus.fifo_tap[0].clear();
            m.bus.fifo_tap[1].clear();
            m.bus.hle_tap.clear();
        }
    }
    let trace = trace.finish();
    assert!(!trace.truncated && trace.dropped_requests == 0);
    let typed: u64 = trace
        .events
        .iter()
        .filter(|e| e.key.service == Service::Typed)
        .map(|e| e.requests)
        .sum();
    let raw_reads: u64 = trace
        .events
        .iter()
        .filter(|e| e.key.service == Service::Raw)
        .map(|e| e.requests)
        .sum();
    let unclassified: u64 = trace
        .events
        .iter()
        .filter(|e| e.key.service == Service::Unclassified)
        .map(|e| e.requests)
        .sum();
    let bytes = owner
        .conditional_packed_lookup_stats()
        .unwrap()
        .source_bytes
        - before;
    assert!(
        typed > 0 && bytes > 0,
        "ordinary alias row must be consumed"
    );
    assert_eq!((raw_reads, unclassified), (0, 0));
    assert!(audio.iter().all(|&v| v > 0));
    assert!(dispatch.0 > 0);
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","scope":"240-frame recorded EUR battle-entry raw/managed translated pair","revision":[170,169],"frames":240,"row_typed_requests":typed,"row_raw_requests":raw_reads,"row_unclassified_requests":unclassified,"packed_bytes_all":bytes,"trace_dropped":trace.dropped_requests,"source_row_poisoned":true,"full_state_clock_video_oam_audio_equal_each_frame":true,"audio_sample_counts":audio,"generated_dispatches":dispatch.0,"fallback_dispatches":dispatch.1,"limits":"Generated blocks may fall back to interpretation; row attribution is aggregate and full importer/other routes remain open.","performance_measurements":false})).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "authenticated EUR battle state/input and current dataset; Wine only"]
fn european_descriptor_alias_complete_ordinary_importer() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};

    const ENTRY: u32 = 0x080c_e18c;
    const ROW: usize = 0x6e6f5e;
    let root = PathBuf::from(std::env::var_os("FFTA_SPECIAL_ALIAS_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_SPECIAL_ALIAS_DATASET").unwrap());
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let state = fs::read(std::env::var_os("FFTA_STATE_TEST_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "e91b8c594d3b7f10d6b815a61cc13943a7c86ea17a7c4cfc865f48eaaf37d84f");
    let input_path = PathBuf::from(std::env::var_os("FFTA_STATE_TEST_INPUT").unwrap());
    assert_eq!(gba_core::ffta_rom::digest(&fs::read(&input_path).unwrap()), "ca9b84ef9dbeb8d71c47552e9455a2c3fb636a0f7a4219196ce8d82270bf0ad9");
    let input = crate::InputScript::load(input_path.to_str().unwrap()).unwrap();
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision, manifest.decoder_revision), (170, 169));
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    raw.load_state(&state).unwrap();
    attach_from_directory(&mut managed, &dataset).unwrap();
    let owner = managed.bus.rom_resources().unwrap().clone();
    let mut calls = 0u64;
    let mut outcome = None;

    for frame in 0..240u64 {
        raw.bus.keys = input.keys_at(frame);
        input.apply_mouse(&mut raw.bus, frame).unwrap();
        raw.bus.audio_buf.clear();
        raw.bus.frame_ready = false;
        for _ in 0..5_000_000 {
            if raw.bus.frame_ready { break; }
            if frame < 180 || raw.cpu.regs[15] != ENTRY {
                raw.step();
                continue;
            }
            calls += 1;
            let return_pc = raw.cpu.regs[14] & !1;
            let entry_sp = raw.cpu.regs[13];
            let initial = raw.save_state().unwrap();
            managed.load_state(&initial).unwrap();
            managed.bus.audio_buf.clear();
            raw.bus.audio_buf.clear();
            managed.bus.rom[ROW..ROW + 14].fill(0xa7);
            let mut raw_steps = 0u64;
            loop {
                assert!(raw_steps < 5_000_000, "raw importer did not return to {return_pc:08x}");
                raw.step();
                raw_steps += 1;
                if raw.cpu.regs[15] == return_pc && raw.cpu.regs[13] == entry_sp { break; }
            }
            let before = owner.conditional_packed_lookup_stats().unwrap().source_bytes;
            let capture = TraceCapture::start(TraceFilter { readers: vec![], ranges: vec![(ROW as u32, (ROW + 14) as u32)] }, 10000).unwrap();
            capture.frame(Some(managed.bus.frames));
            let mut managed_steps = 0u64;
            loop {
                assert!(managed_steps < 5_000_000, "managed importer did not return to {return_pc:08x}");
                managed.step();
                managed_steps += 1;
                if managed.cpu.regs[15] == return_pc && managed.cpu.regs[13] == entry_sp { break; }
            }
            let trace = capture.finish();
            assert!(!trace.truncated);
            let typed: u64 = trace.events.iter().filter(|e| e.key.service == Service::Typed).map(|e| e.requests).sum();
            let raw_reads: u64 = trace.events.iter().filter(|e| e.key.service == Service::Raw).map(|e| e.requests).sum();
            let packed_bytes = owner.conditional_packed_lookup_stats().unwrap().source_bytes - before;
            managed.bus.rom[ROW..ROW + 14].copy_from_slice(&rom[ROW..ROW + 14]);
            assert_eq!(raw_steps, managed_steps, "importer instruction count");
            assert!(raw.save_state().unwrap() == managed.save_state().unwrap(), "complete importer state/clock");
            assert!(raw.bus.framebuffer == managed.bus.framebuffer && raw.bus.wide_framebuffer == managed.bus.wide_framebuffer
                && raw.bus.oam == managed.bus.oam && raw.bus.audio_buf == managed.bus.audio_buf, "complete importer video/OAM/audio");
            assert_eq!(raw_reads, 0, "raw fallback within selected row");
            if typed > 0 {
                assert!(packed_bytes > 0);
                outcome = Some(serde_json::json!({"relative_frame":frame,"return_pc":return_pc,"instructions":raw_steps,
                    "typed_row_requests":typed,"raw_row_requests":raw_reads,"packed_bytes":packed_bytes}));
                break;
            }
        }
        if outcome.is_some() { break; }
        assert!(raw.bus.frame_ready, "ordinary route frame limit at {frame}");
    }
    let outcome = outcome.expect("no complete importer call consumed the descriptor-1 row");
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"one ordinary EUR 080ce18c importer invocation, entry through return",
        "candidate_calls":calls, "matched_call":outcome, "row_rom_poisoned":true,
        "full_state_clock_video_oam_mixed_audio_equal_at_return":true,
        "limits":"One interpreter invocation on one retained route; other calls, consumers, native attribution and platforms remain open.",
        "performance_measurements":false
    })).unwrap()).unwrap();
}
