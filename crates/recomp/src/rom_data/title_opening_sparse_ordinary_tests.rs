//! Ordinary EUR opening continuation to the sparse coordinate branch.
use super::*;
use gba_core::Bus;

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "EUR opening checkpoint and E158 dataset; Wine only"]
fn european_title_opening_sparse_coordinates_ordinary() {
    use gba_core::rom_data::read_audit::{Service, TraceCapture, TraceFilter};
    use std::collections::BTreeSet;
    let root=PathBuf::from(std::env::var_os("FFTA_OPENING_SPARSE_ROOT").unwrap());
    let dataset=PathBuf::from(std::env::var_os("FFTA_OPENING_SPARSE_DATASET").unwrap());
    let state=fs::read(std::env::var_os("FFTA_OPENING_SPARSE_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state),"0a439a15a0de6ccda64a8010d50da37b0ba5a4435b683adf45fb7cf68dec56e3");
    let rom=fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let manifest:Manifest=toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision,manifest.decoder_revision),(158,157));
    let ranges:Vec<(u32,u32)>=(0..19).map(|i|(0x3adefe+4*i,0x3adf00+4*i)).collect();
    let mut managed=crate::make_machine(rom.clone(),None);attach_from_directory(&mut managed,&dataset).unwrap();managed.load_state(&state).unwrap();
    let mut positive=None;let mut checkpoint=Vec::new();let mut completed=0;
    for frame in 0..180u32 {
        managed.bus.keys=0x3ff;let before=managed.save_state().unwrap();
        for &(lo,hi) in &ranges {managed.bus.rom[lo as usize..hi as usize].fill(0xcd);}
        let trace=TraceCapture::start(TraceFilter{readers:vec![],ranges:ranges.clone()},10_000).unwrap();trace.frame(Some(managed.bus.frames));
        managed.bus.frame_ready=false;for _ in 0..5_000_000 {if managed.bus.frame_ready {break;}managed.step();}assert!(managed.bus.frame_ready);
        let report=trace.finish();assert!(!report.truncated);
        let raw_reads:u64=report.events.iter().filter(|e|e.key.service==Service::Raw).map(|e|e.requests).sum();assert_eq!(raw_reads,0);
        let observed:BTreeSet<_>=report.events.iter().filter(|e|e.key.service==Service::Typed).map(|e|e.key.read.physical_offset).collect();
        let requests:u64=report.events.iter().filter(|e|e.key.service==Service::Typed).map(|e|e.requests).sum();
        for &(lo,hi) in &ranges {managed.bus.rom[lo as usize..hi as usize].copy_from_slice(&rom[lo as usize..hi as usize]);}
        completed=frame+1;
        if !observed.is_empty() {
            assert_eq!(observed,ranges.iter().map(|range|range.0).collect());
            let mut raw=crate::make_machine(rom.clone(),None);raw.load_state(&before).unwrap();raw.bus.keys=0x3ff;
            raw.bus.frame_ready=false;for _ in 0..5_000_000 {if raw.bus.frame_ready {break;}raw.step();}assert!(raw.bus.frame_ready);
            assert_eq!(raw.save_state().unwrap(),managed.save_state().unwrap(),"ordinary sparse reader frame");
            positive=Some(serde_json::json!({"frame":frame,"typed_requests":requests,"distinct_halfwords":observed.len(),"paired_full_state_clock_frames":1}));checkpoint=before;break;
        }
    }
    if checkpoint.is_empty(){checkpoint=managed.save_state().unwrap();}
    fs::write(root.join("checkpoint.ffstate"),&checkpoint).unwrap();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","revision":[158,157],"scope":"bounded neutral EUR opening, interpreter","frames":completed,"ordinary_sparse":positive,"checkpoint_sha256":gba_core::ffta_rom::digest(&checkpoint),"poisoned_source":true,"raw_requests":0,"negative_is_not_global_absence":true,"native_dispatch":"pending","Linux":"pending","USA":"pending","native_windows":"pending","performance_measurements":false})).unwrap()).unwrap();
}
