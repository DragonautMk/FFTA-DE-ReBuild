//! Projection of the private schema-1 save collection. Never rewrites its evidence.
use serde::Serialize;
use serde_json::{json,Value};
use std::{collections::{BTreeMap,BTreeSet}, path::{Component,Path,PathBuf}};
const MAX_STATE: u64 = gba_core::save_state::MAX_STATE_BYTES as u64 + 240*160*4 + 48;

#[derive(Clone,Serialize)]
pub(crate) struct Entry {
    pub key:String, pub id:String, pub name:String, pub kind:String, pub case:String,
    pub region:String, pub status:String, pub slot:Option<u8>, pub source:String,
    pub evidence:String, pub file:Option<String>, pub sha256:Option<String>,
    pub integrity:String, pub error:Option<String>, pub available:bool,
}
pub(crate) struct Catalog { pub path:PathBuf, root:PathBuf, pub entries:Vec<Entry> }
#[derive(Default)]
pub(crate) struct Filter { pub case:Option<String>, pub region:Option<String>, pub status:Option<String>, pub kind:Option<String>, pub entry:Option<String> }
fn string(v:&Value,key:&str)->String { v[key].as_str().unwrap_or("").to_owned() }
fn slot(file:&str)->Option<u8> {
    Path::new(file).file_stem()?.to_str()?.rsplit_once('-')?.1.parse::<u8>().ok().filter(|n|(1..=99).contains(n))
}
fn ds(name:&str)->String {
    let prefix=name.get(..5).unwrap_or("");
    if prefix.starts_with("DS-") && prefix[3..].bytes().all(|b|b.is_ascii_digit()) {prefix.to_owned()} else {String::new()}
}
impl Filter {
    pub fn matches(&self,e:&Entry)->bool {
        self.case.as_ref().is_none_or(|x|x==&e.case)
        && self.region.as_ref().is_none_or(|x|e.region.is_empty()||x==&e.region)
        && self.status.as_ref().is_none_or(|x|x==&e.status)
        && self.kind.as_ref().is_none_or(|x|x==&e.kind)
        && self.entry.as_ref().is_none_or(|x|x==&e.key)
    }
}
fn relative(root:&Path,name:&str)->Result<PathBuf,String> {
    if name.is_empty() || !Path::new(name).components().all(|c|matches!(c,Component::Normal(_))) {
        return Err("Catalog file paths must be relative and stay inside the collection".into());
    }
    let path=std::fs::canonicalize(root.join(name)).map_err(|e|e.to_string())?;
    if !path.starts_with(root) { return Err("Catalog file resolves outside the collection".into()); }
    Ok(path)
}
impl Catalog {
    pub fn load(path:&Path)->Result<Self,String> {
        let path=std::fs::canonicalize(path).map_err(|e|format!("Cannot open catalog: {e}"))?;
        let root=path.parent().ok_or("Catalog needs a parent directory")?.to_owned();
        let data:Value=serde_json::from_slice(&crate::diagnostics::read(&path,4*1024*1024)?).map_err(|e|e.to_string())?;
        if data["schema"]!=1 || !data["cases"].is_array() { return Err("Expected a schema-1 debug-save catalog with cases".into()); }
        let mut entries=Vec::new();let mut keys=BTreeSet::new();
        let mut add=|item:&Value,id:String,name:String,kind:&str,case:String,status:String,region:String,evidence:String|->Result<(),String>{
            if !id.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'-'||b==b'_') || !["","EUR","USA"].contains(&region.as_str()) {
                return Err("Invalid catalog identifier or region".into());
            }
            let file=item["file"].as_str().map(String::from);
            let slot=file.as_deref().and_then(slot).or_else(||item["slot"].as_u64().and_then(|n|u8::try_from(n).ok()));
            let key=if kind=="case" {format!("{id}/{region}/{}",slot.map_or_else(||"none".into(),|s|s.to_string()))}
                else if kind=="variant" {format!("VAR/{}",file.as_deref().unwrap_or(&name))}
                else {format!("{id}/{region}")};
            let key=item["catalog_key"].as_str().unwrap_or(&key).to_owned();
            if key.len()>512 || key.chars().any(char::is_control) {return Err("Invalid catalog entry key".into());}
            if !keys.insert(key.clone()) {return Err(format!("Duplicate catalog key: {key}"));}
            entries.push(Entry {key,id,name,kind:kind.into(),case,region,status,slot,
                source:item["input_id"].as_str().or(item["origin"].as_str()).or(item["source"].as_str()).unwrap_or("").into(),
                evidence,file,sha256:item["sha256"].as_str().map(String::from),
                integrity:"no-checkpoint".into(),error:None,available:false});
            if entries.len()>1024 {return Err("Catalog exceeds 1024 entries".into());} Ok(())
        };
        for case in data["cases"].as_array().unwrap() {
            let id=string(case,"id");
            if ds(&id)!=id || id.len()!=5 { return Err("Case IDs must be DS-NN".into()); }
            let checkpoints=case["checkpoints"].as_array().ok_or("Case needs checkpoint list")?;
            if checkpoints.is_empty() {add(&Value::Null,id.clone(),string(case,"name"),"case",id.clone(),string(case,"status"),String::new(),string(case,"scope"))?;}
            for cp in checkpoints {
                add(cp,id.clone(),string(case,"name"),"case",id.clone(),
                    cp["source_case_status"].as_str().unwrap_or(case["status"].as_str().unwrap_or("parziale")).into(),
                    string(cp,"region"),cp["evidence_scope"].as_str().unwrap_or(case["scope"].as_str().unwrap_or("")).into())?;
            }
        }
        let mut variant_hashes=BTreeSet::new();
        for item in data["variant_slots"].as_array().into_iter().flatten().chain(data["variants"].as_array().into_iter().flatten()) {
            let file=string(item,"file");let region=item["region"].as_str().unwrap_or(file.split('/').next().unwrap_or("")).to_owned();
            let name=item["name"].as_str().map(String::from).unwrap_or_else(||Path::new(&file).file_stem().unwrap_or_default().to_string_lossy().into_owned());
            // Equal bytes can describe different scene scopes. Collapse only a named alias.
            if !variant_hashes.insert((string(item,"sha256"),name.clone())) {continue;}
            let case=ds(&name);
            add(item,case.clone(),name,"variant",case,"variante".into(),region,string(item,"scope"))?;
        }
        for item in data["inputs"].as_array().into_iter().flatten() {
            let mut ready=item["ready"].clone();
            if ready.is_object() { ready["input_id"]=item["id"].clone(); }
            add(&ready,string(item,"id"),string(item,"id"),"IN",String::new(),"origine".into(),string(item,"ready_region"),string(item,"compatibility"))?;
        }
        for section in ["generated_standard_saves","factory_standard_saves"] {
            for item in data[section].as_array().into_iter().flatten() {
                let id=string(item,"id");let kind=if id.starts_with("NEW-"){"NEW"}else{"GEN"};
                add(item,id.clone(),id,kind,String::new(),"origine".into(),item["region"].as_str().unwrap_or(item["file"].as_str().unwrap_or("").split('/').next().unwrap_or("")).into(),string(item,"scope"))?;
            }
        }
        let mut catalog=Self {path,root,entries};
        // Hash each physical collection file once for the initial projection.
        let mut cache=BTreeMap::<String,Result<String,String>>::new();
        for entry in &mut catalog.entries {
            let Some(name)=&entry.file else {continue;};
            let result=cache.entry(name.clone()).or_insert_with(||{
                let path=relative(&catalog.root,name)?;
                let bytes=crate::diagnostics::read(&path,MAX_STATE)?;
                // Per-entry hashes are checked below, including duplicate paths.
                if matches!(entry.kind.as_str(),"case"|"variant") {crate::save_state::preview(&bytes)?;}
                Ok(gba_core::ffta_rom::digest(&bytes))
            }).clone();
            let result=result.and_then(|observed| if entry.sha256.as_deref()==Some(observed.as_str()) {Ok(())} else {Err("Catalog checksum mismatch".into())});
            match result {Ok(())=>{entry.available=true;entry.integrity="verified".into();},Err(e)=>{entry.integrity="invalid".into();entry.error=Some(e);}}
        }
        Ok(catalog)
    }
    fn verify_bytes(root:&Path,e:&Entry)->Result<Vec<u8>,String> {
        let path=relative(root,e.file.as_deref().ok_or("This entry has no checkpoint")?)?;
        let bytes=crate::diagnostics::read(&path,MAX_STATE)?;
        let expected=e.sha256.as_deref().ok_or("Catalog checksum missing")?;
        if expected.len()!=64 || gba_core::ffta_rom::digest(&bytes)!=expected {return Err("Catalog checksum mismatch".into());}
        Ok(bytes)
    }
    pub fn bytes(&self,index:usize)->Result<Vec<u8>,String> {Self::verify_bytes(&self.root,self.entries.get(index).ok_or("Unknown catalog entry")?)}
    pub fn selected(&self,filter:&Filter)->Vec<usize> {self.entries.iter().enumerate().filter(|(_,e)|filter.matches(e)).map(|(i,_)|i).collect()}
    pub fn report(&self,filter:&Filter)->Value {
        let entries:Vec<_>=self.entries.iter().filter(|e|filter.matches(e)).collect();
        json!({"schema":1,"kind":"ffta-debug-catalog","catalog":self.path,"ok":entries.iter().all(|e|e.integrity!="invalid"),"entries":entries,
            "limits":["Recorded validation status is not new runtime acceptance.","Variant/IN/GEN identifiers and slot numbers are independent."]})
    }
    pub fn preview(&self,index:usize)->Result<Vec<[u8;4]>,String> {
        if !matches!(self.entries[index].kind.as_str(),"case"|"variant") {return Ok(Vec::new());}
        crate::save_state::preview(&self.bytes(index)?)
    }
    pub fn import(&self,index:usize,machine:&gba_core::Machine,slots:&mut crate::save_state::Slots,slot:Option<u8>)->Result<u8,String> {
        let e=self.entries.get(index).ok_or("Unknown catalog entry")?;
        if !matches!(e.kind.as_str(),"case"|"variant") {return Err("Only checkpoints can be imported into snapshot slots".into());}
        let slot=slot.or_else(||(1..=slots.count()).find(|&n|slots.path(n).is_ok_and(|p|!p.exists()))).ok_or("No empty snapshot slot")?;
        slots.import(machine,slot,&self.bytes(index)?)?;Ok(slot)
    }
    pub fn create(&self,index:usize,destination:&Path,rom:&[u8],bios:Option<&[u8]>,interp:bool,av:input_config::AvConfig,input:input_config::InputConfig,dataset:Option<&Path>)->Result<PathBuf,String> {
        let entry=self.entries.get(index).ok_or("Unknown catalog entry")?;let bytes=self.bytes(index)?;
        let mut machine=crate::make_machine(rom.to_vec(),bios);
        let snapshot=matches!(entry.kind.as_str(),"case"|"variant");
        if snapshot {crate::save_state::load_bytes(&mut machine,&bytes)?;} else {
            if machine.bus.save_data().is_none_or(|s|s.len()!=bytes.len()) {return Err("Cartridge save size incompatible with selected ROM".into());}
            machine.bus.load_save_data(&bytes);
        }
        let region=match gba_core::ffta_rom::version(rom) {Some(gba_core::ffta_rom::Version::Europe)=>"EUR",Some(gba_core::ffta_rom::Version::Usa)=>"USA",None=>""};
        if entry.region!=region {return Err("Catalog region does not match selected ROM".into());}
        let root=crate::debug_session::create(destination,crate::debug_session::Source{rom,bios,save:machine.bus.save_data(),state:snapshot.then_some(bytes.as_slice()),
            av,input,interp,dataset,executable:&std::env::current_exe().map_err(|e|e.to_string())?})?;
        std::fs::write(root.join("catalog-entry.json"),serde_json::to_vec_pretty(entry).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
        // A portable, one-entry view reuses the existing seed/save: no third payload copy.
        std::fs::write(root.join("debug-catalog.json"),serde_json::to_vec_pretty(&portable_view(entry,snapshot)).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
        Ok(root)
    }
}

fn portable_view(entry:&Entry,snapshot:bool)->Value {
        let mut item=json!({"catalog_key":entry.key,"file":if snapshot{"seed.ffstate"}else{"Data/saves/ffta.sav"},"sha256":entry.sha256,"slot":entry.slot,"region":entry.region,"input_id":entry.source,"source_case_status":entry.status,"evidence_scope":entry.evidence,"scope":entry.evidence,"name":entry.name});
        let mut view=json!({"schema":1,"cases":[],"variants":[],"inputs":[],"generated_standard_saves":[],"factory_standard_saves":[]});
        match entry.kind.as_str() {
            "case"=>view["cases"]=json!([{"id":entry.id,"name":entry.name,"status":entry.status,"scope":entry.evidence,"checkpoints":[item]}]),
            "variant"=>view["variants"]=json!([item]),
            "IN"=>view["inputs"]=json!([{"id":entry.id,"ready_region":entry.region,"compatibility":entry.evidence,"ready":item}]),
            kind=>{item["id"]=entry.id.clone().into();view[if kind=="NEW"{"factory_standard_saves"}else{"generated_standard_saves"}]=json!([item]);},
        }
        view
}

pub(crate) fn default_path()->Option<PathBuf> {
    if crate::debug_session::present(){let path=input_config::exe_dir()?.join("debug-catalog.json");if path.is_file(){return Some(path);}}
    Some(input_config::config_root()?.join("debug-catalog/catalog.json"))
}

pub(crate) fn cmd(args:&[String])->Result<(),String> {
    let mode=args.first().ok_or("Use debug-catalog list/open/import CATALOG [options]")?;
    let path=Path::new(args.get(1).ok_or("Catalog path required")?);
    let mut filter=Filter::default();let mut session=None;let mut destination=None;let mut dataset=None;let mut rom=None;let mut bios=None;let mut no_bios=false;let mut interp=false;let mut json=false;let mut launch=false;let mut slot=None;
    let mut it=args[2..].iter();
    while let Some(option)=it.next() {
        match option.as_str(){
            "--json"=>json=true,"--no-bios"=>no_bios=true,"--interp"=>interp=true,"--launch"=>launch=true,
            "--case"|"--region"|"--status"|"--kind"|"--entry"|"--session"|"--dir"|"--dataset"|"--rom"|"--bios"|"--slot"=>{
                let value=it.next().ok_or_else(||format!("{option} needs a value"))?.clone();
                match option.as_str(){"--case"=>filter.case=Some(value),"--region"=>filter.region=Some(value),"--status"=>filter.status=Some(value),"--kind"=>filter.kind=Some(value),"--entry"=>filter.entry=Some(value),
                    "--session"=>session=Some(PathBuf::from(value)),"--dir"=>destination=Some(PathBuf::from(value)),"--dataset"=>dataset=Some(PathBuf::from(value)),"--rom"=>rom=Some(PathBuf::from(value)),"--bios"=>bios=Some(PathBuf::from(value)),
                    "--slot"=>{let n=value.parse::<u8>().map_err(|_|"Slot must be 1..99")?;if !(1..=99).contains(&n){return Err("Slot must be 1..99".into());}slot=Some(n);},_=>unreachable!()}
            },_=>return Err(format!("Unknown catalog option {option}"))
        }
    }
    if no_bios&&bios.is_some(){return Err("--bios and --no-bios are mutually exclusive".into());}
    if filter.region.as_ref().is_some_and(|r| !["EUR","USA"].contains(&r.as_str())) {return Err("Region must be EUR or USA".into());}
    if filter.case.as_ref().is_some_and(|id|ds(id)!=*id||id.len()!=5) {return Err("DS case must be DS-NN; IN/GEN identifiers use --kind or --entry".into());}
    if filter.kind.as_ref().is_some_and(|k|!["case","variant","IN","GEN","NEW"].contains(&k.as_str())) {return Err("Unknown catalog kind".into());}
    if filter.status.as_ref().is_some_and(|s|!["validata","parziale","non_coperta","variante","origine"].contains(&s.as_str())) {return Err("Unknown recorded catalog status".into());}
    if mode=="import"&&(rom.is_some()||bios.is_some()||no_bios||destination.is_some()||dataset.is_some()||interp||launch) {
        return Err("Import uses the closed session's pinned ROM/BIOS and does not accept launch or build overrides".into());
    }
    if mode=="open"&&crate::debug_session::present(){return Err("Close this debug copy before opening another isolated case".into());}
    let catalog=Catalog::load(path)?;let selected=catalog.selected(&filter);
    match mode.as_str(){
        "list"=>{let report=catalog.report(&filter);if json {println!("{}",serde_json::to_string_pretty(&report).map_err(|e|e.to_string())?);}else{for i in selected {let e=&catalog.entries[i];println!("{} | {} | {} | {} | {} | {}",e.key,e.kind,e.name,e.region,e.status,e.error.as_deref().unwrap_or(&e.evidence));}}if report["ok"]!=true{return Err("Catalog contains missing, changed or invalid files".into());}},
        "open"|"import"=>{
            if selected.len()!=1 {return Err("Select exactly one entry with --entry KEY (or a unique case/region filter)".into());}let index=selected[0];
            if mode=="import" {
                let session=crate::debug_session::open(&session.ok_or("Import needs --session DIR for a closed isolated copy")?)?;
                let rom=crate::diagnostics::read(&session.rom(),32*1024*1024)?;let bios=session.bios().map(|p|crate::diagnostics::read(&p,16384)).transpose()?;
                let machine=crate::make_machine(rom,bios.as_deref());let mut slots=crate::save_state::Slots::new(&session.root.join("Data/saves/ffta.sav"),&machine.bus.rom);slots.set_extended(true);
                let slot=catalog.import(index,&machine,&mut slots,slot)?;
                println!("{}",json!({"schema":1,"kind":"ffta-catalog-import","entry":catalog.entries[index].key,"slot":slot,"requires_extended_saves":slot>9,"session":session.root}));
            }else{
                let rom=crate::diagnostics::read(&rom.ok_or("Open needs --rom FILE")?,32*1024*1024)?;
                let bios=if no_bios{None}else{bios.or_else(input_config::find_bios)}.map(|p|crate::diagnostics::read(&p,16384)).transpose()?;
                let root=catalog.create(index,&destination.ok_or("Open needs a new --dir DIR")?,&rom,bios.as_deref(),interp,input_config::AvConfig::load(),input_config::InputConfig::load(),dataset.as_deref())?;
                println!("{}",json!({"schema":1,"kind":"ffta-catalog-session","entry":catalog.entries[index].key,"root":root}));
                if launch {let status=crate::debug_session::launch(&root)?.wait().map_err(|e|e.to_string())?;if !status.success(){return Err(format!("Catalog session exited {status}"));}}
            }
        },_=>return Err("Use debug-catalog list, open or import".into())
    }Ok(())
}
#[cfg(test)]mod tests;
