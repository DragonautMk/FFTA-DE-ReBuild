//! Ordinary EUR opening: observe the independent title animation's OBJ draw.
use super::*;

#[cfg(feature = "rom-data-read-audit")]
const MAIN_BANK_HEADERS: [u32; 6] = [0xb8ed40, 0xb8ed44, 0xb8ed58, 0xb8ed6c, 0xb8ed80, 0xb8ed90];

#[cfg(feature = "rom-data-read-audit")]
thread_local! {
    static MAIN_BANK_DIRECT: std::cell::Cell<([u64; 6], [u64; 6])> = const { std::cell::Cell::new(([0; 6], [0; 6])) };
    static MAIN_BANK_INTERP_DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    static MAIN_BANK_API_CALLS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

#[cfg(feature = "rom-data-read-audit")]
fn count_main_bank_native_read(pointer: *mut std::ffi::c_void, address: u32, read: extern "C" fn(*mut std::ffi::c_void, u32) -> u32) -> u32 {
    MAIN_BANK_API_CALLS.with(|calls| calls.set(calls.get() + 1));
    if (0x08..=0x0d).contains(&(address >> 24)) {
        let offset = address & 0x01ff_ffff;
        if let Some(index) = MAIN_BANK_HEADERS.iter().position(|&header| header == offset) {
            let nested = MAIN_BANK_INTERP_DEPTH.with(|depth| depth.get() > 0);
            MAIN_BANK_DIRECT.with(|counts| {
                let (mut direct, mut fallback) = counts.get();
                if nested { fallback[index] += 1; } else { direct[index] += 1; }
                counts.set((direct, fallback));
            });
        }
    }
    read(pointer, address)
}

#[cfg(feature = "rom-data-read-audit")]
extern "C" fn main_bank_read8(pointer: *mut std::ffi::c_void, address: u32) -> u32 {
    count_main_bank_native_read(pointer, address, gba_core::capi::RT_API.read8)
}
#[cfg(feature = "rom-data-read-audit")]
extern "C" fn main_bank_read16(pointer: *mut std::ffi::c_void, address: u32) -> u32 {
    count_main_bank_native_read(pointer, address, gba_core::capi::RT_API.read16)
}
#[cfg(feature = "rom-data-read-audit")]
extern "C" fn main_bank_read32(pointer: *mut std::ffi::c_void, address: u32) -> u32 {
    count_main_bank_native_read(pointer, address, gba_core::capi::RT_API.read32)
}
#[cfg(feature = "rom-data-read-audit")]
extern "C" fn main_bank_interp_one(pointer: *mut std::ffi::c_void) -> u32 {
    MAIN_BANK_INTERP_DEPTH.with(|depth| depth.set(depth.get() + 1));
    let result = (gba_core::capi::RT_API.interp_one)(pointer);
    MAIN_BANK_INTERP_DEPTH.with(|depth| depth.set(depth.get() - 1));
    result
}

#[cfg(feature = "rom-data-read-audit")]
static MAIN_BANK_API: gba_core::capi::RtApi = gba_core::capi::RtApi {
    read8: main_bank_read8, read16: main_bank_read16, read32: main_bank_read32,
    interp_one: main_bank_interp_one, ..gba_core::capi::RT_API
};

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "EUR opening checkpoint or cold save, E170 dataset and native library; Wine only"]
fn european_title_main_bank_native_ordinary_pair() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};

    crate::configure_headless_widescreen();
    let root = PathBuf::from(std::env::var_os("FFTA_TITLE_BANK_DRAW_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_TITLE_BANK_DRAW_DATASET").unwrap());
    let cold_boot = std::env::var_os("FFTA_TITLE_BANK_COLD_BOOT").is_some();
    let state = if cold_boot { None } else {
        let state = fs::read(std::env::var_os("FFTA_TITLE_BANK_DRAW_STATE").unwrap()).unwrap();
        assert_eq!(gba_core::ffta_rom::digest(&state), "0a439a15a0de6ccda64a8010d50da37b0ba5a4435b683adf45fb7cf68dec56e3");
        Some(state)
    };
    let save = if cold_boot {
        assert_eq!(std::env::var("FFTA_PATCH_SPR").as_deref(), Ok("0"));
        let boot_input = fs::read_to_string(std::env::var_os("FFTA_TITLE_BANK_BOOT_INPUT").unwrap()).unwrap();
        assert_eq!(boot_input.replace("\r\n", "\n"), "900 3ff\n2 3f7\n120 3ff\n");
        Some(fs::read(std::env::var_os("FFTA_STATE_TEST_SAVE").unwrap()).unwrap())
    } else { None };
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision, manifest.decoder_revision), (170, 169));
    let resource = manifest.resources.iter().find(|r| r.id == LAYERED_OBJ_ANIMATIONS_ID).unwrap();
    let data = parse_layered_obj_animations(&fs::read(dataset.join(&resource.path)).unwrap()).unwrap();
    let bank = data.title_main_bank.as_ref().unwrap();
    assert_eq!((bank.object_root, bank.sequence_root, bank.lists.len()), (0xb8ed28, 0xb8ecb4, 6));
    let ranges: Vec<_> = bank.lists.iter().map(|list| (list.source_offset, list.source_offset + list.source_len() as u32)).collect();
    assert_eq!(ranges.iter().map(|(lo, _)| *lo).collect::<Vec<_>>(), MAIN_BANK_HEADERS);
    let (_library, table) = crate::load_native(Path::new(&std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap())).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &dataset).unwrap();
    for machine in [&mut raw, &mut managed] {
        if let Some(state) = &state { machine.load_state(state).unwrap(); }
        if let Some(save) = &save {
            assert!(!machine.bus.ffta_sprites.enabled, "cold boot must start with sprite patch disabled");
            machine.bus.load_save_data(save);
        }
        crate::arm_audio_hle(machine, None);
        machine.bus.tap_channels = true;
        machine.bus.audio_buf.clear(); machine.bus.psg_tap.clear();
        machine.bus.fifo_tap[0].clear(); machine.bus.fifo_tap[1].clear(); machine.bus.hle_tap.clear();
    }
    assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
    MAIN_BANK_DIRECT.with(|counts| counts.set(([0; 6], [0; 6])));
    MAIN_BANK_API_CALLS.with(|calls| calls.set(0));
    let trace = TraceCapture::start(TraceFilter { readers: vec![], ranges: ranges.clone() }, 50_000).unwrap();
    let start = raw.bus.frames;
    let mut first = [None; 6];
    let mut frames = 0u64;
    let mut audio = [0u64; 5];
    let mut dispatch = (0u64, 0u64);
    let max_frames = if cold_boot { 4_500 } else { 180 };
    for frame in 0..max_frames {
        for &(lo, hi) in &ranges { managed.bus.rom[lo as usize..hi as usize].fill(0xcd); }
        let keys = if cold_boot && (900..902).contains(&frame) { 0x3f7 } else { 0x3ff };
        raw.bus.keys = keys;
        trace.frame(None);
        let raw_pointer = &mut raw as *mut gba_core::Machine as *mut std::ffi::c_void;
        let raw_counts = crate::run_frame_native(&mut raw, &table, raw_pointer, crate::FRAME_STEP_GUARD);
        managed.bus.keys = keys;
        trace.frame(Some(managed.bus.frames));
        let managed_pointer = &mut managed as *mut gba_core::Machine as *mut std::ffi::c_void;
        let managed_counts = crate::with_test_native_api(&MAIN_BANK_API, || crate::run_frame_native(&mut managed, &table, managed_pointer, crate::FRAME_STEP_GUARD));
        trace.frame(None);
        assert_eq!(raw_counts, managed_counts, "dispatch at frame {frame}");
        assert_eq!((raw.bus.frames, managed.bus.frames), (start + frame + 1, start + frame + 1));
        dispatch.0 += raw_counts.0; dispatch.1 += raw_counts.1;
        assert_eq!(raw.bus.clock, managed.bus.clock);
        assert_eq!(raw.bus.framebuffer, managed.bus.framebuffer);
        assert_eq!(raw.bus.wide_framebuffer, managed.bus.wide_framebuffer);
        assert_eq!(raw.bus.oam, managed.bus.oam);
        assert_eq!(raw.bus.audio_buf, managed.bus.audio_buf);
        assert_eq!(raw.bus.psg_tap, managed.bus.psg_tap);
        assert_eq!(raw.bus.fifo_tap, managed.bus.fifo_tap);
        assert!(raw.bus.hle_tap.iter().map(|v| v.to_bits()).eq(managed.bus.hle_tap.iter().map(|v| v.to_bits())));
        for (sum, len) in audio.iter_mut().zip([managed.bus.audio_buf.len(), managed.bus.psg_tap.len(), managed.bus.fifo_tap[0].len(), managed.bus.fifo_tap[1].len(), managed.bus.hle_tap.len()]) { *sum += len as u64; }
        for &(lo, hi) in &ranges { managed.bus.rom[lo as usize..hi as usize].copy_from_slice(&rom[lo as usize..hi as usize]); }
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "state at frame {frame}");
        for machine in [&mut raw, &mut managed] {
            machine.bus.audio_buf.clear(); machine.bus.psg_tap.clear();
            machine.bus.fifo_tap[0].clear(); machine.bus.fifo_tap[1].clear(); machine.bus.hle_tap.clear();
        }
        let direct = MAIN_BANK_DIRECT.with(|counts| counts.get().0);
        for index in 1..6 { if direct[index] > 0 && first[index].is_none() { first[index] = Some(frame); } }
        frames = frame + 1;
        if cold_boot && frames % 500 == 0 { println!("TITLE_NATIVE_COLD_BOOT frames={frames}/{max_frames} direct={direct:?}"); }
        if (1..6).all(|index| first[index].is_some()) { break; }
    }
    let report = trace.finish();
    fs::write(root.join("trace-report.json"), serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    assert!(!report.truncated && report.dropped_requests == 0);
    let (direct, nested) = MAIN_BANK_DIRECT.with(|counts| counts.get());
    let api_calls = MAIN_BANK_API_CALLS.with(|calls| calls.get());
    let mut typed = [0u64; 6];
    let mut raw_requests = 0u64;
    let mut unclassified = 0u64;
    for event in &report.events {
        match event.key.service {
            Service::Raw => raw_requests += event.requests,
            Service::Unclassified => unclassified += event.requests,
            Service::Typed => if let Some(index) = MAIN_BANK_HEADERS.iter().position(|&at| at == event.key.read.physical_offset) { typed[index] += event.requests; },
        }
    }
    assert_eq!((raw_requests, unclassified), (0, 0));
    assert!(dispatch.0 > 0 && audio[0] > 0 && api_calls > 0);
    assert!((1..6).all(|index| typed[index] > 0), "typed={typed:?}");
    if cold_boot {
        assert!((1..6).all(|index| direct[index] > 0 && typed[index] >= direct[index] && nested[index] == 0), "direct={direct:?} typed={typed:?} nested={nested:?}");
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[170,169], "frames":frames, "headers":MAIN_BANK_HEADERS,
        "cold_boot":cold_boot, "sprite_patch_disabled":cold_boot,
        "input":if cold_boot { "recorded two-frame Start after 900 neutral frames, then neutral" } else { "neutral" },
        "direct_native_header_requests":direct, "nested_interpreter_header_requests":nested,
        "typed_header_requests":typed, "first_direct_frame":first, "raw_source_requests":raw_requests,
        "full_state_clock_video_oam_dispatch_audio_equal_each_frame":true, "audio_sample_counts":audio,
        "native_dispatches":dispatch.0, "fallback_dispatches":dispatch.1, "wrapped_native_api_read_calls":api_calls, "source_rom_poisoned":true,
        "limits":"One EUR opening route; direct wrapper counts do not identify exact guest instruction PCs. Nested wrapper counts do not capture direct Rust-bus interpreter loads. See trace-report for reader and interpreter-site attribution; sites can include inherited host/HLE work. Other configurations, scenes, producers and other platforms remain open.",
        "performance_measurements":false
    })).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "EUR opening checkpoint and E158 dataset; Wine only"]
fn european_title_main_bank_first_ordinary_draw() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    let root=PathBuf::from(std::env::var_os("FFTA_TITLE_BANK_DRAW_ROOT").unwrap());
    let dataset=PathBuf::from(std::env::var_os("FFTA_TITLE_BANK_DRAW_DATASET").unwrap());
    let state=fs::read(std::env::var_os("FFTA_TITLE_BANK_DRAW_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state),"18ef2f591239fced9c5b2ed59625ab5ab945298c409addcd6b98a228bc85ba17");
    let rom=fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let manifest:Manifest=toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision,manifest.decoder_revision),(158,157));
    let resource=manifest.resources.iter().find(|r|r.id==LAYERED_OBJ_ANIMATIONS_ID).unwrap();
    let data=parse_layered_obj_animations(&fs::read(dataset.join(&resource.path)).unwrap()).unwrap();
    let bank=data.title_main_bank.as_ref().unwrap();assert_eq!((bank.object_root,bank.sequence_root,bank.lists.len()),(0xb8ed28,0xb8ecb4,6));
    let ranges:Vec<(u32,u32)>=bank.lists.iter().map(|list|(list.source_offset,list.source_offset+list.source_len() as u32)).collect();
    let mut managed=crate::make_machine(rom.clone(),None);attach_from_directory(&mut managed,&dataset).unwrap();managed.load_state(&state).unwrap();
    let mut positive=None;let mut completed=0;
    for frame in 0..180u32 {
        managed.bus.keys=0x3ff;let before=managed.save_state().unwrap();
        for &(lo,hi) in &ranges {managed.bus.rom[lo as usize..hi as usize].fill(0xcd);}
        let trace=TraceCapture::start(TraceFilter{readers:vec![],ranges:ranges.clone()},10_000).unwrap();trace.frame(Some(managed.bus.frames));
        managed.bus.frame_ready=false;for _ in 0..5_000_000 {if managed.bus.frame_ready {break;}managed.step();}assert!(managed.bus.frame_ready);
        let report=trace.finish();assert!(!report.truncated);
        let raw_requests:u64=report.events.iter().filter(|e|e.key.service==Service::Raw).map(|e|e.requests).sum();assert_eq!(raw_requests,0);
        let hits:Vec<_>=report.events.iter().filter(|e|e.key.service==Service::Typed && ranges.iter().any(|(lo,_)|*lo==e.key.read.physical_offset)).collect();
        for &(lo,hi) in &ranges {managed.bus.rom[lo as usize..hi as usize].copy_from_slice(&rom[lo as usize..hi as usize]);}
        completed=frame+1;
        if !hits.is_empty() {
            let mut raw=crate::make_machine(rom.clone(),None);raw.load_state(&before).unwrap();raw.bus.keys=0x3ff;
            raw.bus.frame_ready=false;for _ in 0..5_000_000 {if raw.bus.frame_ready {break;}raw.step();}assert!(raw.bus.frame_ready);
            assert_eq!(raw.save_state().unwrap(),managed.save_state().unwrap(),"ordinary bank draw frame");
            let observations:Vec<_>=hits.iter().map(|e|serde_json::json!({"offset":format!("{:06x}",e.key.read.physical_offset),"requests":e.requests,"site":format!("{:?}",e.key.instruction)})).collect();
            positive=Some(serde_json::json!({"frame":frame,"header_reads":observations,"paired_full_state_clock_frames":1}));break;
        }
    }
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","revision":[158,157],"scope":"bounded neutral EUR opening interpreter","frames":completed,"ordinary_list_header":positive,"poisoned_source":true,"raw_source_requests":0,"negative_is_not_global_absence":true,"native_dispatch":"pending","Linux":"pending","USA":"pending","native_windows":"pending","performance_measurements":false})).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "EUR opening checkpoint and E158 dataset; Wine only"]
fn european_title_main_bank_following_lists_ordinary() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    let root=PathBuf::from(std::env::var_os("FFTA_TITLE_BANK_DRAW_ROOT").unwrap());
    let dataset=PathBuf::from(std::env::var_os("FFTA_TITLE_BANK_DRAW_DATASET").unwrap());
    let state=fs::read(std::env::var_os("FFTA_TITLE_BANK_DRAW_STATE").unwrap()).unwrap();
    let expected=std::env::var("FFTA_TITLE_BANK_DRAW_STATE_SHA").unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state),expected);
    let rom=fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let manifest:Manifest=toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision,manifest.decoder_revision),(158,157));
    let resource=manifest.resources.iter().find(|r|r.id==LAYERED_OBJ_ANIMATIONS_ID).unwrap();
    let data=parse_layered_obj_animations(&fs::read(dataset.join(&resource.path)).unwrap()).unwrap();
    let bank=data.title_main_bank.as_ref().unwrap();assert_eq!((bank.object_root,bank.sequence_root,bank.lists.len()),(0xb8ed28,0xb8ecb4,6));
    let ranges:Vec<(u32,u32)>=bank.lists.iter().map(|list|(list.source_offset,list.source_offset+list.source_len() as u32)).collect();
    let mut managed=crate::make_machine(rom.clone(),None);attach_from_directory(&mut managed,&dataset).unwrap();managed.load_state(&state).unwrap();
    let mut first=[None;6];let mut counts=[0u64;6];let mut completed=0;
    for frame in 0..180u32 {
        managed.bus.keys=0x3ff;let before=managed.save_state().unwrap();
        for &(lo,hi) in &ranges {managed.bus.rom[lo as usize..hi as usize].fill(0xcd);}
        let trace=TraceCapture::start(TraceFilter{readers:vec![],ranges:ranges.clone()},10_000).unwrap();trace.frame(Some(managed.bus.frames));
        managed.bus.frame_ready=false;for _ in 0..5_000_000 {if managed.bus.frame_ready {break;}managed.step();}assert!(managed.bus.frame_ready);
        let report=trace.finish();assert!(!report.truncated);
        let raw_requests:u64=report.events.iter().filter(|e|e.key.service==Service::Raw).map(|e|e.requests).sum();assert_eq!(raw_requests,0);
        let mut new_hit=false;
        for event in report.events.iter().filter(|e|e.key.service==Service::Typed) {
            for (i,(lo,_)) in ranges.iter().enumerate(){if event.key.read.physical_offset==*lo {counts[i]+=event.requests;if first[i].is_none(){first[i]=Some(frame);new_hit=true;}}}
        }
        for &(lo,hi) in &ranges {managed.bus.rom[lo as usize..hi as usize].copy_from_slice(&rom[lo as usize..hi as usize]);}
        completed=frame+1;
        if new_hit {
            let mut raw=crate::make_machine(rom.clone(),None);raw.load_state(&before).unwrap();raw.bus.keys=0x3ff;
            raw.bus.frame_ready=false;for _ in 0..5_000_000 {if raw.bus.frame_ready {break;}raw.step();}assert!(raw.bus.frame_ready);
            assert_eq!(raw.save_state().unwrap(),managed.save_state().unwrap(),"first frame of each observed bank list");
        }
        if (1..6).all(|i|first[i].is_some()){break;}
    }
    let observations:Vec<_>=ranges.iter().enumerate().map(|(i,(lo,_))|serde_json::json!({"index":i,"source":format!("{lo:06x}"),"first_frame":first[i],"typed_header_requests":counts[i]})).collect();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","revision":[158,157],"scope":"bounded neutral EUR opening interpreter","frames":completed,"lists":observations,"first_observed_list_frames_match":true,"poisoned_source":true,"raw_source_requests":0,"unobserved_is_not_global_absence":true,"native_dispatch":"pending","Linux":"pending","USA":"pending","native_windows":"pending","performance_measurements":false})).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "EUR opening checkpoint and E158 dataset; Wine only"]
fn european_title_opening_state11_coordinate_ordinary() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    let root=PathBuf::from(std::env::var_os("FFTA_TITLE_BANK_DRAW_ROOT").unwrap());
    let dataset=PathBuf::from(std::env::var_os("FFTA_TITLE_BANK_DRAW_DATASET").unwrap());
    let state=fs::read(std::env::var_os("FFTA_TITLE_BANK_DRAW_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state),"18ef2f591239fced9c5b2ed59625ab5ab945298c409addcd6b98a228bc85ba17");
    let rom=fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let manifest:Manifest=toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision,manifest.decoder_revision),(158,157));
    let mut managed=crate::make_machine(rom.clone(),None);attach_from_directory(&mut managed,&dataset).unwrap();managed.load_state(&state).unwrap();
    let mut positive=None;let mut completed=0;
    for frame in 0..4u32 {
        managed.bus.keys=0x3ff;let before=managed.save_state().unwrap();managed.bus.rom[0x3adefe..0x3adf00].fill(0xcd);
        let trace=TraceCapture::start(TraceFilter{readers:vec![],ranges:vec![(0x3adefe,0x3adf00)]},10_000).unwrap();trace.frame(Some(managed.bus.frames));
        managed.bus.frame_ready=false;for _ in 0..5_000_000 {if managed.bus.frame_ready {break;}managed.step();}assert!(managed.bus.frame_ready);
        let report=trace.finish();assert!(!report.truncated);
        let raw_requests:u64=report.events.iter().filter(|e|e.key.service==Service::Raw).map(|e|e.requests).sum();assert_eq!(raw_requests,0);
        let hits:Vec<_>=report.events.iter().filter(|e|e.key.service==Service::Typed && e.key.instruction.as_ref().is_some_and(|site|site.pc==0x0813_a396)).collect();
        managed.bus.rom[0x3adefe..0x3adf00].copy_from_slice(&rom[0x3adefe..0x3adf00]);completed=frame+1;
        if !hits.is_empty(){
            let mut raw=crate::make_machine(rom.clone(),None);raw.load_state(&before).unwrap();raw.bus.keys=0x3ff;
            raw.bus.frame_ready=false;for _ in 0..5_000_000 {if raw.bus.frame_ready {break;}raw.step();}assert!(raw.bus.frame_ready);
            assert_eq!(raw.save_state().unwrap(),managed.save_state().unwrap(),"ordinary state-11 coordinate reader frame");
            positive=Some(serde_json::json!({"frame":frame,"typed_requests":hits.iter().map(|e|e.requests).sum::<u64>(),"reader_pc":"0813a396","paired_full_state_clock_frames":1}));break;
        }
    }
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","revision":[158,157],"scope":"bounded neutral EUR opening interpreter","frames":completed,"ordinary_state11":positive,"poisoned_source":true,"raw_source_requests":0,"negative_is_not_global_absence":true,"native_dispatch":"pending","Linux":"pending","USA":"pending","native_windows":"pending","performance_measurements":false})).unwrap()).unwrap();
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "EUR opening checkpoint, E170 dataset and native library; Wine only"]
fn european_title_main_bank_complete_animation_pair() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};

    crate::configure_headless_widescreen();
    let root = PathBuf::from(std::env::var_os("FFTA_TITLE_BANK_DRAW_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_TITLE_BANK_DRAW_DATASET").unwrap());
    let state = fs::read(std::env::var_os("FFTA_TITLE_BANK_DRAW_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "0a439a15a0de6ccda64a8010d50da37b0ba5a4435b683adf45fb7cf68dec56e3");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&rom), gba_core::ffta_rom::EUROPE_SHA256);
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision, manifest.decoder_revision), (170, 169));
    let resource = manifest.resources.iter().find(|entry| entry.id == LAYERED_OBJ_ANIMATIONS_ID).unwrap();
    let data = parse_layered_obj_animations(&fs::read(dataset.join(&resource.path)).unwrap()).unwrap();
    let bank = data.title_main_bank.as_ref().unwrap();
    assert_eq!((bank.object_root, bank.sequence_root, bank.sequences.len()), (0xb8ed28, 0xb8ecb4, 1));
    let sequence = &bank.sequences[0].frames;
    let phases: Vec<(usize, usize)> = sequence.iter().map(|frame| (usize::from(frame.object_list), usize::from(frame.duration) + 1)).collect();
    assert_eq!(phases, [(1, 7), (4, 9), (2, 5), (3, 7), (5, 2), (3, 33)]);
    let ranges: Vec<_> = bank.lists.iter().map(|list| (list.source_offset, list.source_offset + list.source_len() as u32)).collect();
    assert_eq!(ranges.iter().map(|(lo, _)| *lo).collect::<Vec<_>>(), MAIN_BANK_HEADERS);
    let (_library, table) = crate::load_native(Path::new(&std::env::var_os("FFTA_STATE_TEST_NATIVE").unwrap())).unwrap();
    let mut raw = crate::make_machine(rom.clone(), None);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &dataset).unwrap();
    for machine in [&mut raw, &mut managed] {
        machine.load_state(&state).unwrap();
        assert!(machine.bus.ffta_sprites.enabled, "the recorded default sprite hook must remain enabled");
        crate::arm_audio_hle(machine, None);
        machine.bus.tap_channels = true;
        machine.bus.audio_buf.clear(); machine.bus.psg_tap.clear();
        machine.bus.fifo_tap[0].clear(); machine.bus.fifo_tap[1].clear(); machine.bus.hle_tap.clear();
    }
    assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap());
    let start_frame = raw.bus.frames;
    let mut masks = Vec::new();
    let mut completion_flags = Vec::new();
    let mut raw_requests = 0u64;
    let mut unclassified = 0u64;
    let mut audio = [0u64; 5];
    let mut dispatch = (0u64, 0u64);
    for frame in 0..180u64 {
        for &(lo, hi) in &ranges { managed.bus.rom[lo as usize..hi as usize].fill(0xcd); }
        raw.bus.keys = 0x3ff;
        let raw_pointer = &mut raw as *mut gba_core::Machine as *mut std::ffi::c_void;
        let raw_counts = crate::run_frame_native(&mut raw, &table, raw_pointer, crate::FRAME_STEP_GUARD);
        managed.bus.keys = 0x3ff;
        let capture = TraceCapture::start(TraceFilter { readers: vec![], ranges: ranges.clone() }, 10_000).unwrap();
        capture.frame(Some(managed.bus.frames));
        let managed_pointer = &mut managed as *mut gba_core::Machine as *mut std::ffi::c_void;
        let managed_counts = crate::run_frame_native(&mut managed, &table, managed_pointer, crate::FRAME_STEP_GUARD);
        let trace = capture.finish();
        assert!(!trace.truncated && trace.dropped_requests == 0);
        let mut mask = 0u8;
        for event in &trace.events {
            match event.key.service {
                Service::Raw => raw_requests += event.requests,
                Service::Unclassified => unclassified += event.requests,
                Service::Typed => if let Some(index) = MAIN_BANK_HEADERS.iter().position(|&at| at == event.key.read.physical_offset) { mask |= 1 << index; },
            }
        }
        masks.push(mask);
        assert_eq!(raw_counts, managed_counts, "dispatch at frame {frame}");
        assert_eq!((raw.bus.frames, managed.bus.frames), (start_frame + frame + 1, start_frame + frame + 1));
        dispatch.0 += raw_counts.0; dispatch.1 += raw_counts.1;
        assert_eq!(raw.bus.clock, managed.bus.clock);
        assert_eq!(raw.bus.framebuffer, managed.bus.framebuffer);
        assert_eq!(raw.bus.wide_framebuffer, managed.bus.wide_framebuffer);
        assert_eq!(raw.bus.oam, managed.bus.oam);
        assert_eq!(raw.bus.audio_buf, managed.bus.audio_buf);
        assert_eq!(raw.bus.psg_tap, managed.bus.psg_tap);
        assert_eq!(raw.bus.fifo_tap, managed.bus.fifo_tap);
        assert!(raw.bus.hle_tap.iter().map(|v| v.to_bits()).eq(managed.bus.hle_tap.iter().map(|v| v.to_bits())));
        for (sum, len) in audio.iter_mut().zip([managed.bus.audio_buf.len(), managed.bus.psg_tap.len(), managed.bus.fifo_tap[0].len(), managed.bus.fifo_tap[1].len(), managed.bus.hle_tap.len()]) { *sum += len as u64; }
        for &(lo, hi) in &ranges { managed.bus.rom[lo as usize..hi as usize].copy_from_slice(&rom[lo as usize..hi as usize]); }
        assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "state at frame {frame}");
        // Initializer 0813a228 derives the animator from this live EWRAM pointer.
        // Read the immutable RAM snapshot directly so observation adds no guest bus work.
        let ram = &managed.bus.ewram;
        let word = |at: usize| ram.get(at..at + 4).map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()));
        let flag = word(0xf3b0).and_then(|base| base.checked_add(0x896c))
            .filter(|&address| (0x0200_0000..0x0204_0000).contains(&address))
            .and_then(|address| {
                let at = (address - 0x0200_0000) as usize;
                if word(at) != Some(0x08b8_ed28) || word(at + 4) != Some(0x08b8_ecb4) { return None; }
                ram.get(at + 0x18..at + 0x1a).map(|bytes| u16::from_le_bytes(bytes.try_into().unwrap()))
            });
        completion_flags.push(flag);
        for machine in [&mut raw, &mut managed] {
            machine.bus.audio_buf.clear(); machine.bus.psg_tap.clear();
            machine.bus.fifo_tap[0].clear(); machine.bus.fifo_tap[1].clear(); machine.bus.hle_tap.clear();
        }
    }
    assert_eq!((raw_requests, unclassified), (0, 0));
    assert!(dispatch.0 > 0 && audio.iter().all(|&samples| samples > 0));
    let first = masks.iter().position(|mask| mask & (1 << phases[0].0) != 0).expect("first animation phase");
    let mut cursor = first;
    for &(index, duration) in &phases {
        for step in 0..duration {
            let mask = *masks.get(cursor).expect("complete animation window");
            assert_eq!(mask, 1 << index, "phase list {index}, step {step}, frame {cursor}, mask {mask:02x}");
            cursor += 1;
        }
    }
    assert!(completion_flags[first..cursor - 1].iter().all(|&flag| flag == Some(0)), "animator completed before final phase ended");
    assert_eq!(completion_flags[cursor - 1], Some(1), "animator completion flag after final phase");
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "revision":[170,169], "frames":masks.len(), "first_relative_frame":first,
        "last_relative_frame":cursor - 1, "phase_list_and_frame_counts":phases,
        "full_state_clock_video_oam_dispatch_audio_equal_each_frame":true, "audio_sample_counts":audio,
        "raw_source_requests":raw_requests, "source_rom_poisoned":true, "default_sprite_hook":true,
        "animator_completion_flag_raises_after_sixth_phase":true,
        "limits":"One EUR opening and the six recorded sequence phases; does not establish other animation banks, producers, native header attribution or other platforms.",
        "performance_measurements":false
    })).unwrap()).unwrap();
}
