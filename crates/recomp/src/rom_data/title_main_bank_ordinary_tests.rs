//! Neutral EUR opening continuation for the separate title animation bank.
use super::*;
use gba_core::Bus;

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "EUR opening checkpoint and E156 dataset; Wine only"]
fn european_title_main_bank_neutral_opening() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    let root = PathBuf::from(std::env::var_os("FFTA_TITLE_BANK_ROOT").unwrap());
    let dataset = PathBuf::from(std::env::var_os("FFTA_TITLE_BANK_DATASET").unwrap());
    let state = fs::read(std::env::var_os("FFTA_TITLE_BANK_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state), "55191340adc8558f5eb4935cc89b1a21be44da7e8cf265c4b64be12ef7b24e19");
    let rom = fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let manifest: Manifest = toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision, manifest.decoder_revision), (156, 155));
    let resource = manifest.resources.iter().find(|r| r.id == LAYERED_OBJ_ANIMATIONS_ID).unwrap();
    let data = parse_layered_obj_animations(&fs::read(dataset.join(&resource.path)).unwrap()).unwrap();
    let bank = data.title_main_bank.as_ref().unwrap();
    assert_eq!((bank.object_root, bank.sequence_root), (0xb8ed28, 0xb8ecb4));
    let mut ranges = Vec::new();
    for span in &resource.sources {
        if (0xb8ec00..0xb8f000).contains(&span.offset) {
            // Use only this bank's disjoint spans; title-language spans are separate.
            if span.offset == bank.object_root || span.offset == bank.sequence_root
                || bank.lists.iter().any(|l| l.source_offset == span.offset)
                || bank.sequences.iter().any(|s| s.source_offset == span.offset)
            { ranges.push((span.offset, span.offset + span.length)); }
        }
    }
    ranges.sort_unstable();
    assert_eq!(ranges.len(), 9);
    assert_eq!(ranges.iter().map(|(a,b)| b-a).sum::<u32>(), 150);
    let mut managed = crate::make_machine(rom.clone(), None);
    attach_from_directory(&mut managed, &dataset).unwrap();
    managed.load_state(&state).unwrap();
    let mut branch = None;
    let mut positive = None;
    let mut completed = 0;
    for frame in 0..2400u32 {
        managed.bus.keys = 0x3ff;
        let before = managed.save_state().unwrap();
        for &(lo,hi) in &ranges { managed.bus.rom[lo as usize..hi as usize].fill(0xcd); }
        let capture = TraceCapture::start(TraceFilter { readers: vec![], ranges: ranges.clone() }, 50_000).unwrap();
        capture.frame(Some(managed.bus.frames));
        managed.bus.frame_ready = false;
        for _ in 0..5_000_000 {
            if managed.bus.frame_ready { break; }
            if managed.cpu.regs[15] == 0x0813_9da4 && branch.is_none() {
                branch = Some(serde_json::json!({"frame":frame,"r0":managed.cpu.regs[0],"skip_flag":managed.bus.read8(0x0300_280c)}));
            }
            managed.step();
        }
        assert!(managed.bus.frame_ready);
        let report = capture.finish();
        assert!(!report.truncated);
        let typed: u64 = report.events.iter().filter(|e| e.key.service == Service::Typed).map(|e| e.requests).sum();
        let raw_reads: u64 = report.events.iter().filter(|e| e.key.service == Service::Raw).map(|e| e.requests).sum();
        assert_eq!(raw_reads, 0);
        for &(lo,hi) in &ranges { managed.bus.rom[lo as usize..hi as usize].copy_from_slice(&rom[lo as usize..hi as usize]); }
        completed = frame + 1;
        if typed > 0 {
            let mut raw = crate::make_machine(rom.clone(), None);
            raw.load_state(&before).unwrap(); raw.bus.keys = 0x3ff;
            raw.bus.frame_ready = false;
            for _ in 0..5_000_000 { if raw.bus.frame_ready { break; } raw.step(); }
            assert!(raw.bus.frame_ready);
            assert_eq!(raw.save_state().unwrap(), managed.save_state().unwrap(), "first ordinary bank frame");
            positive = Some(serde_json::json!({"frame":frame,"typed_requests":typed,"raw_requests":raw_reads,"paired_full_state_clock_frames":1}));
            break;
        }
    }
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "status":"PASS", "scope":"bounded neutral EUR opening continuation, interpreter",
        "frames":completed, "title_branch":branch, "ordinary_bank":positive,
        "unobserved_is_not_global_absence":true, "native_dispatch":"pending",
        "Linux":"pending", "USA":"pending", "native_windows":"pending", "performance_measurements":false
    })).unwrap()).unwrap();
}
