//! Normal EUR title cursor choices and typed source reads under Wine.
use super::*;
use gba_core::{Bus, Machine};

#[cfg(feature = "rom-data-read-audit")]
fn paired_frame(raw: &mut Machine, managed: &mut Machine, rom: &[u8], keys: u16) -> Vec<(u32,usize)> {
    use gba_core::rom_data::read_audit::{Service,TraceCapture,TraceFilter};
    const LO: usize=0x3adcfc; const HI:usize=LO+20;
    raw.bus.keys=keys;managed.bus.keys=keys;managed.bus.rom[LO..HI].fill(0xcd);
    raw.bus.frame_ready=false;for _ in 0..5_000_000 {if raw.bus.frame_ready {break;} raw.step();}assert!(raw.bus.frame_ready);
    let capture=TraceCapture::start(TraceFilter{readers:vec![],ranges:vec![(LO as u32,HI as u32)]},10_000).unwrap();capture.frame(Some(managed.bus.frames));
    managed.bus.frame_ready=false;for _ in 0..5_000_000 {if managed.bus.frame_ready {break;} managed.step();}assert!(managed.bus.frame_ready);
    let report=capture.finish();assert!(!report.truncated);
    assert!(report.events.iter().all(|e|e.key.service!=Service::Raw),"raw read of managed title positions");
    let reads=report.events.iter().filter(|e|e.key.service==Service::Typed).map(|e|(e.key.read.physical_offset,e.key.read.length)).collect();
    managed.bus.rom[LO..HI].copy_from_slice(&rom[LO..HI]);
    assert_eq!(raw.save_state().unwrap(),managed.save_state().unwrap(),"title selector full state/clock");
    reads
}

#[cfg(feature = "rom-data-read-audit")]
#[test]
#[ignore = "EUR title selector checkpoint, E155 dataset; Wine only"]
fn european_title_cursor_all_languages_ordinary_interpreter_pair() {
    const ROOT:u32=0x3adcfc;
    let root=PathBuf::from(std::env::var_os("FFTA_TITLE_CURSOR_ROOT").unwrap());
    let dataset=PathBuf::from(std::env::var_os("FFTA_TITLE_CURSOR_DATASET").unwrap());
    let state=fs::read(std::env::var_os("FFTA_TITLE_CURSOR_STATE").unwrap()).unwrap();
    assert_eq!(gba_core::ffta_rom::digest(&state),"ce54d2693068161aa1452835109c7a44082fa9bd4e2777ab6976cddd88e192e1");
    let rom=fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
    let manifest:Manifest=toml::from_str(&fs::read_to_string(dataset.join(MANIFEST_FILE)).unwrap()).unwrap();
    assert_eq!((manifest.extractor_revision,manifest.decoder_revision),(155,154));
    let mut raw=crate::make_machine(rom.clone(),None);let mut managed=crate::make_machine(rom.clone(),None);
    attach_from_directory(&mut managed,&dataset).unwrap();raw.load_state(&state).unwrap();managed.load_state(&state).unwrap();
    let mut observations=Vec::new();
    for selected in 0..5u8 {
        let mut offsets=std::collections::BTreeSet::new();
        let frames=if selected==0 {1} else {8};
        for frame in 0..frames {
            let keys=if selected!=0 && frame<2 {0x37f} else {0x3ff};
            for (offset,length) in paired_frame(&mut raw,&mut managed,&rom,keys) {
                if length==2 {offsets.insert(offset);}
            }
        }
        let context=managed.bus.read32(0x0200_f3b0);
        assert_eq!(managed.bus.read8(context+0x8964),selected,"normal title selection");
        let base=ROOT+4*u32::from(selected);
        assert!(offsets.contains(&base) && offsets.contains(&(base+2)),"both cursor halfwords observed for {selected}: {offsets:?}");
        observations.push(serde_json::json!({"selection":selected,"first":format!("{base:06x}"),"second":format!("{:06x}",base+2),"frames":frames}));
    }
    fs::write(root.join("result.json"),serde_json::to_vec_pretty(&serde_json::json!({"status":"PASS","scope":"ordinary EUR title selector, neutral then four Down/release steps; interpreter raw/managed paired frames","revision":[155,154],"selections":observations,"source_rom_poisoned":true,"raw_source_reads":0,"native_dispatch_for_new_choices":"pending","Linux":"pending","USA":"pending","native_windows":"pending","performance_measurements":false})).unwrap()).unwrap();
}
