//! Ordinary EUR opening: observe the independent title animation's OBJ draw.
use super::*;
use gba_core::Bus;

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
