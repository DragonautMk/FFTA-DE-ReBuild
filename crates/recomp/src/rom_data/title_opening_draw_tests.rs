//! Ordinary neutral EUR opening, observed until the first managed OBJ list read.
use super::*;
use gba_core::Bus;

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "EUR neutral opening checkpoint and E157 dataset; Wine only"]
fn european_title_opening_first_obj_draw() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    let root = PathBuf::from(std::env::var_os("FFTA_OPENING_DRAW_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_OPENING_DRAW_DATASET").unwrap());
    let state = fs::read(std::env::var_os("FFTA_OPENING_DRAW_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "55191340adc8558f5eb4935cc89b1a21be44da7e8cf265c4b64be12ef7b24e19");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision, manifest.decoder_revision), (157, 156));
    let ranges = [(0x3ade0au32, 0x3ade62u32), (0x3ade64, 0x3adeb0)];
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &dataset).unwrap(); managed.load_state(&state).unwrap();
    let mut directory_requests = 0u64;
    let mut positive = None;
    let mut checkpoint = Vec::new();
    let mut completed = 0u32;
    for frame in 0..2700u32 {
        managed.bus.keys = 0x3ff;
        let before = managed.save_state().unwrap();
        for &(lo,hi) in &ranges { managed.bus.rom[lo as usize..hi as usize].fill(0xcd); }
        let trace = TraceCapture::start(TraceFilter { readers: vec![], ranges: ranges.to_vec() }, 50_000).unwrap();
        trace.frame(Some(managed.bus.frames));
        managed.bus.frame_ready = false;
        for _ in 0..5_000_000 { if managed.bus.frame_ready { break; } managed.step(); }
        assert!(managed.bus.frame_ready);
        let report = trace.finish(); assert!(!report.truncated);
        let raw_requests: u64 = report.events.iter().filter(|e| e.key.service == Service::Raw).map(|e| e.requests).sum();
        assert_eq!(raw_requests, 0);
        directory_requests += report.events.iter().filter(|e| e.key.service == Service::Typed && (ranges[1].0..ranges[1].1).contains(&e.key.read.physical_offset)).map(|e| e.requests).sum::<u64>();
        let hits: Vec<_> = report.events.iter().filter(|e| e.key.service == Service::Typed && (ranges[0].0..ranges[0].1).contains(&e.key.read.physical_offset)).collect();
        for &(lo,hi) in &ranges { managed.bus.rom[lo as usize..hi as usize].copy_from_slice(&rom[lo as usize..hi as usize]); }
        completed = frame + 1;
        if !hits.is_empty() {
            let mut raw = crate::make_machine(rom.clone(), None); raw.load_state(&before).unwrap(); raw.bus.keys = 0x3ff;
            raw.bus.frame_ready = false;
            for _ in 0..5_000_000 { if raw.bus.frame_ready { break; } raw.step(); }
            assert!(raw.bus.frame_ready);
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "first opening OBJ read frame");
            let observations: Vec<_> = hits.iter().map(|e| serde_json::json!({"offset":format!("{:06x}",e.key.read.physical_offset),"requests":e.requests,"site":format!("{:?}",e.key.instruction)})).collect();
            positive = Some(serde_json::json!({"frame":frame,"reads":observations,"paired_full_state_clock_frames":1}));
            checkpoint = before;
            break;
        }
    }
    if checkpoint.is_empty() { checkpoint = managed.save_state().unwrap(); }
    fs::write(root.join("checkpoint.ffstate"), &checkpoint).unwrap();
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"bounded neutral EUR opening, interpreter",
        "frames":completed, "typed_directory_requests":directory_requests, "first_list_reads":positive,
        "checkpoint_sha256":gba_core::ffta_rom::digest(&checkpoint), "source_rom_poisoned":true,
        "raw_source_requests":0, "negative_is_not_global_absence":true, "native_dispatch":"pending",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
