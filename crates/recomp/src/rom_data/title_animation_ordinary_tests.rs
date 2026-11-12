//! Bounded ordinary EUR title animation list coverage under Wine.
use super::*;
use gba_core::{Bus,Machine};

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "EUR title selector checkpoint and E155 dataset; Wine only"]
fn european_title_sequence_one_lists_ordinary_interpreter_pair() {
    use gba_core::rom_data::read_audit::{Service,TraceCapture,TraceFilter};
    let root=PathBuf::from(std::env::var_os("FFTA_TITLE_ANIMATION_ROOT").unwrap());
    let dataset=PathBuf::from(std::env::var_os("FFTA_TITLE_ANIMATION_DATASET").unwrap());
    let state=fs::read(std::env::var_os("FFTA_TITLE_ANIMATION_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state),"ce54d2693068161aa1452835109c7a44082fa9bd4e2777ab6976cddd88e192e1");
    let rom=fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let manifest:Manifest=toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision,manifest.decoder_revision),(155,154));
    let resource=manifest.resources.iter().find(|r|r.id==LAYERED_OBJ_ANIMATIONS_ID).unwrap();
    let data=parse_layered_obj_animations(&fs::read(dataset.join(&resource.path)).unwrap()).unwrap();
    let bank=data.title_language_bank.as_ref().unwrap();
    let indices=[9usize,10,11,12,13,14];
    let ranges:Vec<(u32,u32)>=indices.iter().map(|&i|{let l=&bank.lists[i];(l.source_offset,l.source_offset+l.source_len() as u32)}).collect();
    assert_eq!(ranges,[(0xb8ee40,0xb8ee48),(0xb8ee48,0xb8ee50),(0xb8ee50,0xb8ee58),(0xb8ee58,0xb8ee66),(0xb8ee68,0xb8ee76),(0xb8ee78,0xb8ee86)]);
    let mut raw=crate::make_machine(rom.clone(),None);let mut managed=crate::make_machine(rom.clone(),None);
    attach_from_directory(&mut managed,&dataset).unwrap();raw.load_state(&state).unwrap();managed.load_state(&state).unwrap();
    let mut hits=[0u64;6];let mut first=[None;6];let mut frames=0u32;
    for frame in 0..100u32 {
        raw.bus.keys=0x3ff;managed.bus.keys=0x3ff;
        for &(lo,hi) in &ranges {managed.bus.rom[lo as usize..hi as usize].fill(0xcd);}
        raw.bus.frame_ready=false;for _ in 0..5_000_000 {if raw.bus.frame_ready {break;}raw.step();}assert!(raw.bus.frame_ready);
        let capture=TraceCapture::start(TraceFilter{readers:vec![],ranges:ranges.clone()},30_000).unwrap();capture.frame(Some(managed.bus.frames));
        managed.bus.frame_ready=false;for _ in 0..5_000_000 {if managed.bus.frame_ready {break;}managed.step();}assert!(managed.bus.frame_ready);
        let report=capture.finish();assert!(!report.truncated);
        for event in &report.events {
            assert_ne!(event.key.service,Service::Raw,"raw read of managed title list");
            if event.key.service==Service::Typed {
                for (i,&(lo,_)) in ranges.iter().enumerate() {
                    if event.key.read.physical_offset==lo {hits[i]+=event.requests;first[i].get_or_insert(frame);}
                }
            }
        }
        for &(lo,hi) in &ranges {managed.bus.rom[lo as usize..hi as usize].copy_from_slice(&rom[lo as usize..hi as usize]);}
        assert_eq!(raw.save_state().unwrap(),managed.save_state().unwrap(),"title animation frame {frame}");
        frames=frame+1;if first.iter().all(Option::is_some){break;}
    }
    assert!(first.iter().all(Option::is_some),"not all six lists were observed in the bounded window: {first:?}");
    let observations:Vec<_>=indices.iter().enumerate().map(|(i,index)|serde_json::json!({"index":index,"source":format!("{:06x}",ranges[i].0),"first_frame":first[i],"typed_header_requests":hits[i]})).collect();
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","scope":"one neutral ordinary EUR title-selector continuation, interpreter raw/managed pairs until all sequence-1 list headers observed","revision":[155,154],"frames":frames,"lists":observations,"source_rom_poisoned":true,"raw_source_reads":0,"sequence_zero_and_other_scenes":"pending","native_dispatch":"pending","Linux":"pending","USA":"pending","native_windows":"pending","performance_measurements":false})).unwrap()).unwrap();
}
