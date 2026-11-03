use super::*;
struct Fixture {dir:tempfile::TempDir,rom:Vec<u8>,state:Vec<u8>,path:PathBuf}
impl Fixture {
 fn new()->Self {
    let dir=tempfile::tempdir().unwrap();let mut rom=vec![0;512];rom[3]=0xea;rom[256..262].copy_from_slice(b"SRAM_V");
    let mut machine=crate::make_machine(rom.clone(),None);machine.bus.frames=777;
    let state=machine.save_state().unwrap();std::fs::create_dir(dir.path().join("EUR")).unwrap();
    std::fs::write(dir.path().join("EUR/slot-20.ffstate"),&state).unwrap();
    let path=dir.path().join("catalog.json");
    let data=json!({"schema":1,"cases":[
        {"id":"DS-03","name":"uncovered","status":"non_coperta","checkpoints":[]},
        {"id":"DS-15","name":"uncovered alternative","status":"non_coperta","checkpoints":[]},
        {"id":"DS-20","name":"dispatch failed","status":"validata","checkpoints":[{"file":"EUR/slot-20.ffstate","sha256":gba_core::ffta_rom::digest(&state),"region":"EUR","source_case_status":"parziale","input_id":"IN-20","evidence_scope":"Recorded limited precursor"}]}],
        "inputs":[{"id":"IN-20","ready_region":"EUR","ready":{"file":"EUR/progress.sav","sha256":gba_core::ffta_rom::digest(&vec![7;65536])},"compatibility":"Booted map"}],
        "generated_standard_saves":[{"id":"GEN-30","file":"EUR/generated.sav","sha256":gba_core::ffta_rom::digest(&vec![9;65536])}],"variants":[],"variant_slots":[]});
    std::fs::write(dir.path().join("EUR/progress.sav"),vec![7;65536]).unwrap();std::fs::write(dir.path().join("EUR/generated.sav"),vec![9;65536]).unwrap();
    std::fs::write(&path,serde_json::to_vec(&data).unwrap()).unwrap();Self {dir,rom,state,path}
 }
 fn catalog(&self)->Catalog {Catalog::load(&self.path).unwrap()}
}
#[test]fn recorded_regional_status_and_independent_identifiers_are_preserved(){
 let f=Fixture::new();let original=std::fs::read(&f.path).unwrap();let c=f.catalog();
 let filter=Filter{case:Some("DS-20".into()),region:Some("EUR".into()),..Default::default()};let selected=c.selected(&filter);assert_eq!(selected.len(),1);
 let e=&c.entries[selected[0]];assert_eq!(e.status,"parziale");assert_eq!(e.source,"IN-20");assert_eq!(e.slot,Some(20));assert!(e.available);
 for id in ["DS-03","DS-15"] {let indices=c.selected(&Filter{case:Some(id.into()),region:Some("EUR".into()),..Default::default()});assert_eq!(indices.len(),1);let e=&c.entries[indices[0]];assert_eq!(e.status,"non_coperta");assert!(!e.available);}
 assert_eq!(c.selected(&Filter{kind:Some("IN".into()),..Default::default()}).len(),1);
 assert_eq!(c.selected(&Filter{kind:Some("GEN".into()),..Default::default()}).len(),1);
 assert_eq!(c.report(&filter)["schema"],1);assert_eq!(std::fs::read(f.path).unwrap(),original);
}
#[test]fn missing_and_changed_collection_files_are_visible_and_rechecked_before_import(){
 let f=Fixture::new();let c=f.catalog();let index=c.selected(&Filter{case:Some("DS-20".into()),..Default::default()})[0];
 std::fs::write(f.dir.path().join("EUR/slot-20.ffstate"),b"changed").unwrap();assert!(c.bytes(index).unwrap_err().contains("checksum"));
 let changed=f.catalog();assert_eq!(changed.entries[index].integrity,"invalid");assert!(changed.entries[index].error.is_some());
 std::fs::remove_file(f.dir.path().join("EUR/slot-20.ffstate")).unwrap();let missing=f.catalog();assert!(!missing.entries[index].available);assert_eq!(missing.report(&Filter::default())["ok"],false);
}
#[test]fn import_is_exact_and_undoable_without_loading_or_overwriting_current_state(){
 let f=Fixture::new();let c=f.catalog();let index=c.selected(&Filter{case:Some("DS-20".into()),..Default::default()})[0];
 let mut machine=crate::make_machine(f.rom.clone(),None);let before=machine.save_state().unwrap();
 let destination=f.dir.path().join("copy");std::fs::create_dir(&destination).unwrap();let mut slots=crate::save_state::Slots::new(&destination.join("ffta.sav"),&f.rom);slots.set_extended(true);
 let slot=c.import(index,&machine,&mut slots,None).unwrap();assert_eq!(slot,1);assert_eq!(std::fs::read(slots.path(slot).unwrap()).unwrap(),f.state);
 assert_eq!(machine.save_state().unwrap(),before);assert!(c.import(index,&machine,&mut slots,Some(1)).is_err());assert_eq!(std::fs::read(slots.path(1).unwrap()).unwrap(),f.state);
 slots.undo(&mut machine).unwrap();assert!(!slots.path(1).unwrap().exists());assert_eq!(machine.save_state().unwrap(),before);
 assert_eq!(std::fs::read(f.dir.path().join("EUR/slot-20.ffstate")).unwrap(),f.state);
}
#[test]fn incompatible_rom_and_standard_save_imports_cannot_create_a_snapshot(){
 let f=Fixture::new();let c=f.catalog();let index=c.selected(&Filter{case:Some("DS-20".into()),..Default::default()})[0];
 let mut wrong=f.rom.clone();wrong[400]=1;let machine=crate::make_machine(wrong.clone(),None);
 let mut slots=crate::save_state::Slots::new(&f.dir.path().join("copy/ffta.sav"),&wrong);slots.set_extended(true);
 assert!(c.import(index,&machine,&mut slots,None).is_err());assert!(!f.dir.path().join("copy").exists());
 let standard=c.selected(&Filter{kind:Some("IN".into()),..Default::default()})[0];assert!(c.import(standard,&machine,&mut slots,None).unwrap_err().contains("Only checkpoints"));
}
#[test]fn catalog_path_escapes_duplicates_and_unsupported_schemas_are_rejected(){
 let f=Fixture::new();let mut data:Value=serde_json::from_slice(&std::fs::read(&f.path).unwrap()).unwrap();
 data["cases"][2]["checkpoints"][0]["file"]=Value::String("../outside.ffstate".into());std::fs::write(&f.path,serde_json::to_vec(&data).unwrap()).unwrap();
 let c=f.catalog();assert_eq!(c.entries[2].integrity,"invalid");assert!(c.entries[2].error.as_ref().unwrap().contains("relative"));
 let duplicate=data["cases"][0].clone();data["cases"].as_array_mut().unwrap().push(duplicate);std::fs::write(&f.path,serde_json::to_vec(&data).unwrap()).unwrap();assert!(Catalog::load(&f.path).err().unwrap().contains("Duplicate"));
 data["schema"]=99.into();std::fs::write(&f.path,serde_json::to_vec(&data).unwrap()).unwrap();assert!(Catalog::load(&f.path).is_err());
}
#[test]fn envelope_preview_checksum_is_required_even_when_catalog_hash_matches(){
 let f=Fixture::new();let mut corrupt=f.state.clone();*corrupt.last_mut().unwrap()^=1;
 let mut data:Value=serde_json::from_slice(&std::fs::read(&f.path).unwrap()).unwrap();data["cases"][2]["checkpoints"][0]["sha256"]=gba_core::ffta_rom::digest(&corrupt).into();
 std::fs::write(f.dir.path().join("EUR/slot-20.ffstate"),corrupt).unwrap();std::fs::write(&f.path,serde_json::to_vec(&data).unwrap()).unwrap();
 let c=f.catalog();assert!(c.entries[2].error.as_ref().unwrap().contains("checksum"));
}

#[test]fn portable_selected_catalog_preserves_key_and_scope_after_folder_move(){
 let f=Fixture::new();let original=f.catalog();let index=original.selected(&Filter{case:Some("DS-20".into()),..Default::default()})[0];
 let entry=&original.entries[index];let folder=f.dir.path().join("selected case");std::fs::create_dir(&folder).unwrap();
 std::fs::write(folder.join("seed.ffstate"),&f.state).unwrap();std::fs::write(folder.join("debug-catalog.json"),serde_json::to_vec(&portable_view(entry,true)).unwrap()).unwrap();
 let moved=f.dir.path().join("moved selected case");std::fs::rename(folder,&moved).unwrap();let catalog=Catalog::load(&moved.join("debug-catalog.json")).unwrap();
 assert_eq!(catalog.entries.len(),1);let copy=&catalog.entries[0];assert_eq!(copy.key,entry.key);assert_eq!(copy.source,entry.source);assert_eq!(copy.status,"parziale");assert!(copy.available);assert_eq!(catalog.bytes(0).unwrap(),f.state);
}

#[test]fn equal_variant_bytes_do_not_merge_distinct_case_scopes(){
 let f=Fixture::new();let mut data:Value=serde_json::from_slice(&std::fs::read(&f.path).unwrap()).unwrap();
 let mut variants=Vec::new();
 for name in ["DS-09-area","DS-11-KO"] {let file=format!("EUR/{name}.ffstate");std::fs::write(f.dir.path().join(&file),&f.state).unwrap();variants.push(json!({"file":file,"sha256":gba_core::ffta_rom::digest(&f.state),"scope":name}));}
 data["variants"]=variants.into();std::fs::write(&f.path,serde_json::to_vec(&data).unwrap()).unwrap();let c=f.catalog();
 for id in ["DS-09","DS-11"] {let selected=c.selected(&Filter{case:Some(id.into()),kind:Some("variant".into()),..Default::default()});assert_eq!(selected.len(),1);assert!(c.entries[selected[0]].available);}
}
