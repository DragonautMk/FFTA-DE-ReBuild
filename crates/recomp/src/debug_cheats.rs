//! Readonly local reference catalog and explicitly assumed plaintext inspection.
use serde_json::{json, Value};
use std::{collections::BTreeSet, path::{Path, PathBuf}};

const MAX_BYTES:u64=4*1024*1024;
const MAX_ENTRIES:usize=2048;
const MAX_PAGE:usize=64;
const MAX_CAPABILITY_SOURCES:usize=8;
const MAX_CAPABILITY_LINES:usize=64;
const MAX_CATALOG_CAPABILITY_LINES:usize=16384;

pub(crate) struct Catalog { pub source_sha256:String, metadata:Value, entries:Vec<Value>, capabilities:Vec<Value> }

/// Only the two project-known exact digests identify a ROM region. An entry's
/// region remains a source declaration, not proof that its codes work.
struct Target<'a>{sha256:Option<&'a str>,region:Option<&'static str>}
impl<'a> Target<'a>{
    fn new(sha256:Option<&'a str>)->Self{
        let region=match sha256 {Some(s) if s.eq_ignore_ascii_case(gba_core::ffta_rom::EUROPE_SHA256)=>Some("EUR"),Some(s) if s.eq_ignore_ascii_case(gba_core::ffta_rom::SHA256)=>Some("USA"),_=>None};
        Self{sha256,region}
    }
    fn declaration(&self,entry_region:&str)->&'static str{
        match (self.region,entry_region){(Some(target),"EUR"|"USA"|"JPN") if target==entry_region=>"declared-match",(Some(_),"EUR"|"USA"|"JPN")=>"declared-mismatch",_=>"unknown"}
    }
    fn explanation(&self,entry_region:&str)->&'static str{
        match self.declaration(entry_region){
            "declared-match"=>"The source region matches the region recognized from the provided SHA-256; neither the actual ROM bytes nor code behavior are verified here.",
            "declared-mismatch"=>"The source region differs from the region recognized from the provided SHA-256; this is metadata, not a runtime test.",
            _=>"The region of the provided SHA-256 or the source region is unknown; no regional compatibility is inferred.",
        }
    }
}

fn text<'a>(entry:&'a Value,key:&str)->Result<&'a str,String>{
    entry.get(key).and_then(Value::as_str).filter(|s|!s.is_empty() && s.len()<=4096 && !s.chars().any(char::is_control))
        .ok_or_else(||format!("Invalid catalog field {key}"))
}
fn strings(entry:&Value,key:&str,max:usize)->Result<(),String>{
    let values=entry.get(key).and_then(Value::as_array).ok_or_else(||format!("Invalid catalog array {key}"))?;
    if values.len()>max || values.iter().any(|v|v.as_str().is_none_or(|s|s.len()>4096)) {return Err(format!("Invalid catalog array {key}"));}Ok(())
}

fn has_undeclared_placeholders(source:&Value)->bool {
    if source.get("parameter").is_some() || source.get("parameters").is_some(){return false;}
    source.get("raw_code_lines").and_then(Value::as_array).is_some_and(|lines|lines.iter().any(|line|{
        let Some(line)=line.as_str() else{return false;};
        let parts:Vec<_>=line.split_ascii_whitespace().collect();
        if parts.len()!=2 || parts[0].len()!=8 || parts[1].len()!=4 || !line.is_ascii(){return false;}
        let mut placeholder=false;
        for byte in parts[0].bytes().chain(parts[1].bytes()) {
            if matches!(byte.to_ascii_lowercase(),b'x'|b'y'|b'z'){placeholder=true;}
            else if !byte.is_ascii_hexdigit(){return false;}
        }
        placeholder
    }))
}
impl Catalog {
    fn parse(bytes:&[u8])->Result<Self,String>{
        if bytes.len() as u64>MAX_BYTES{return Err("Catalog exceeds 4 MiB".into());}
        let mut value:Value=serde_json::from_slice(bytes).map_err(|e|format!("Catalog JSON: {e}"))?;
        let object=value.as_object_mut().ok_or("Catalog root must be an object")?;
        if object.get("schema").and_then(Value::as_u64)!=Some(1){return Err("Unsupported cheat catalog schema".into());}
        let entries=object.remove("entries").and_then(|v|v.as_array().cloned()).ok_or("Catalog entries must be an array")?;
        if entries.len()>MAX_ENTRIES{return Err("Catalog exceeds 2048 entries".into());}
        let mut ids=BTreeSet::new();
        for entry in &entries {
            let id=text(entry,"id")?;
            if id.len()!=7 || !id.starts_with("CH-") || !id[3..].bytes().all(|b|b.is_ascii_digit()) || !ids.insert(id.to_owned()) {return Err("Invalid or duplicate CH identifier".into());}
            for key in ["format","region","section","title","status"]{text(entry,key)?;}
            for (key,max) in [("codes",1024),("notes",256),("aliases",256)]{strings(entry,key,max)?;}
            let provenance=entry.get("provenance").and_then(Value::as_array).ok_or("Invalid provenance array")?;
            if provenance.len()>256 || provenance.iter().any(|p|!p.is_object()){return Err("Invalid provenance array".into());}
        }
        let mut catalog=Self{source_sha256:gba_core::ffta_rom::digest(bytes),metadata:value,entries,capabilities:Vec::new()};
        let mut budget=MAX_CATALOG_CAPABILITY_LINES;
        catalog.capabilities=catalog.entries.iter().map(|entry|catalog.assess(entry,&mut budget)).collect();
        Ok(catalog)
    }
    pub(crate) fn load(path:&Path)->Result<Self,String>{
        crate::debug_content::ordinary(path)?;
        let bytes=crate::diagnostics::read(path,MAX_BYTES)?;
        Self::parse(&bytes)
    }
    fn summary(entry:&Value,rom:Option<&str>)->Value {
        let format=entry["format"].as_str().unwrap();let region=entry["region"].as_str().unwrap();let target=Target::new(rom);
        json!({"id":entry["id"],"title":entry["title"],"section":entry["section"],"format":format,"region":region,
            "enabled":false,"executable":false,"compatibility":{
                "rom":{"state":target.declaration(region),"sha256":target.sha256,"recognized_region":target.region,"reason":target.explanation(region)},
                "device_version":{"state":"unknown","reason":"Source device version or encryption has not been verified"},
                "decoder":{"state":"unresolved","reason":"Source capability has not been assessed"},
                "verification":{"state":"unverified","reason":"Catalog preservation does not establish runtime behavior"}},
            "source_status":entry["status"]})
    }
    fn entry(&self,id:&str)->Option<&Value>{self.entries.iter().find(|e|e["id"]==id)}
    /// Evaluate only explicitly linked source masters, without ROM bytes or
    /// guest access. A decoding result is an assumption, not verification.
    fn assess(&self,entry:&Value,budget:&mut usize)->Value {
        if !entry["format"].as_str().unwrap().eq_ignore_ascii_case("codebreaker") {
            return json!({"state":"unsupported-format","sources":[],"unexamined":0});
        }
        let provenance=entry["provenance"].as_array().unwrap();let mut sources=Vec::new();
        for (index,p) in provenance.iter().take(MAX_CAPABILITY_SOURCES).enumerate(){
            let master=p.get("master_id").and_then(Value::as_str);
            let lines=p.get("raw_code_lines").and_then(Value::as_array);
            let n=lines.map_or(0,Vec::len);
            let outcome=if master.is_none() || lines.is_none() || n==0 {json!({"state":"unresolved","reason":"Source lines or master reference are missing"})}
            else if n>MAX_CAPABILITY_LINES || n>*budget {json!({"state":"not-evaluated","reason":"Capability assessment limit reached"})}
            else if self.entry(master.unwrap()).and_then(|m|m.get("provenance")).and_then(Value::as_array).is_none_or(|items|{
                let matches=items.iter().filter(|m|m.get("source").and_then(Value::as_str)==p.get("source").and_then(Value::as_str)).collect::<Vec<_>>();
                matches.len()!=1 || matches[0].get("raw_code_lines").and_then(Value::as_array).is_none_or(|lines|!matches!(lines.len(),2|3))
            }) {json!({"state":"unresolved","reason":"Source master is missing, ambiguous or not a supported two- or three-line block"})}
            else {
                *budget-=n;
                match self.source_bound(entry,index,master.unwrap()) {
                    Err(error)=>json!({"state":"unresolved","reason":error}),
                    Ok((_,_lines,master_lines)) if crate::debug_cheat_decoder::validate_source_master(&master_lines).is_err()=>json!({"state":"unresolved","reason":"Source master is malformed or unsupported"}),
                    Ok((_,lines,master_lines))=>match Self::parameter_specs(p,&lines) {
                        Err(error)=>json!({"state":"unresolved","reason":error}),
                        Ok(specs) if specs.len()==1=>json!({"state":"parameter-required","parameter":specs[0],"scope":"Declared values are source metadata, not verified game or device behavior"}),
                        Ok(specs) if !specs.is_empty()=>json!({"state":"parameter-required","parameters":specs,"scope":"Declared values are source metadata, not verified game or device behavior"}),
                        Ok(_)=>match crate::debug_cheat_decoder::inspect_source(&lines,&master_lines,None){
                        Err(error)=>json!({"state":"not-decoded","reason":error}),
                        Ok((plan,master_info))=>match plan.footprint(){
                            Ok(footprint)=>json!({"state":"manual-once-subset","plan":plan,"footprint":footprint,"checksum_required":master_info["game_id"]["checksum_enabled"]}),
                            Err(error)=>json!({"state":"decoded-outside-once-limits","reason":error,"plan":plan}),
                        }
                    }}
                }
            };
            sources.push(json!({"provenance_index":index,"master_id":master,"assessment":outcome}));
        }
        let accepted=sources.iter().filter(|s|s["assessment"]["state"]=="manual-once-subset").count();
        let decoded=sources.iter().filter(|s|s["assessment"]["state"]=="decoded-outside-once-limits").count();
        let unexamined=provenance.len().saturating_sub(sources.len());
        let state=if accepted>0 {"manual-once-subset"} else if sources.iter().any(|s|s["assessment"]["state"]=="parameter-required") {"parameter-required"} else if decoded>0 {"decoded-outside-once-limits"} else if unexamined>0 || sources.iter().any(|s|s["assessment"]["state"]=="not-evaluated") {"not-evaluated"} else {"unresolved"};
        json!({"state":state,"sources":sources,"unexamined":unexamined,"scope":"Source assumptions only; device, ROM bytes, checksum and behavior require separate verification"})
    }
    fn indexed_summary(&self,index:usize,rom:Option<&str>)->Value {
        let entry=&self.entries[index];let mut value=Self::summary(entry,rom);let c=&self.capabilities[index];
        let count=c["sources"].as_array().unwrap().iter().filter(|s|s["assessment"]["state"]=="manual-once-subset").count();
        let checksum_sources=c["sources"].as_array().unwrap().iter().filter(|s|s["assessment"]["state"]=="manual-once-subset" && s["assessment"]["checksum_required"]==true).count();
        let pending=c["sources"].as_array().unwrap().iter().filter(|s|s["assessment"]["state"]=="not-evaluated").count();
        let unexamined=c["unexamined"].as_u64().unwrap_or(0) as usize+pending;
        let relation=Target::new(rom).declaration(entry["region"].as_str().unwrap());
        let once=if count==0 && unexamined>0 {"not-evaluated"} else if count==0 {match c["state"].as_str().unwrap(){"not-evaluated"=>"not-evaluated","parameter-required"=>"parameter-required","unresolved"=>"not-established",_=>"unavailable"}} else if checksum_sources==0 {"master-checksum-required"} else if relation=="declared-mismatch" {"rom-region-mismatch"} else if relation=="unknown" {"rom-context-required"} else {"requires-runtime-preflight"};
        value["compatibility"]["decoder"]=json!({"state":c["state"],"reason":"Source-bound assessment only; inspect the selected provenance for its full plan"});
        value["compatibility"]["manual_once"]=json!({"state":once,"assessed_sources":c["sources"].as_array().unwrap().len()-pending,"supported_sources":count,"unexamined_sources":unexamined,"scope":"Requires explicit source and master, isolated paused interpreter, unchanged snapshot and actual ROM checksum; not a device or gameplay verification"});
        value
    }
    fn source_lines(provenance:&Value)->Result<Vec<String>,String>{
        let lines=provenance.get("raw_code_lines").and_then(Value::as_array).ok_or("Source record has no raw code lines")?;
        if lines.is_empty() || lines.len()>1024{return Err("Source needs 1..1024 raw code lines".into());}
        lines.iter().map(|v|v.as_str().filter(|s|s.len()<=4096).map(str::to_owned).ok_or_else(||"Invalid source code line".to_string())).collect()
    }
    fn parameter_specs(provenance:&Value,lines:&[String])->Result<Vec<crate::debug_cheat_parameters::Specification>,String>{
        let single=provenance.get("parameter");let multiple=provenance.get("parameters");
        if single.is_some() && multiple.is_some(){return Err("Use either parameter or parameters, not both".into());}
        let specs=if let Some(value)=single{vec![serde_json::from_value(value.clone()).map_err(|e|format!("Invalid declared parameter: {e}"))?]}
            else if let Some(value)=multiple{serde_json::from_value(value.clone()).map_err(|e|format!("Invalid declared parameters: {e}"))?}else{return Ok(Vec::new())};
        crate::debug_cheat_parameters::validate_many_source(lines,&specs)?;Ok(specs)
    }
    fn parameterized_source(provenance:&Value,lines:&[String],parameters:&[crate::debug_cheat_parameters::Choice])->Result<(Vec<String>,Vec<crate::debug_cheat_parameters::Specification>),String>{
        let declared=Self::parameter_specs(provenance,lines)?;
        let expanded=if declared.is_empty(){if !parameters.is_empty(){return Err("This source does not declare parameters".into());}lines.to_vec()}
            else{crate::debug_cheat_parameters::expand_many(lines,&declared,parameters)?};
        Ok((expanded,declared))
    }
    fn source_bound(&self,entry:&Value,index:usize,master_id:&str)->Result<(String,Vec<String>,Vec<String>),String>{
        let p=entry["provenance"].as_array().unwrap().get(index).ok_or("Unknown provenance index")?;
        let source=p.get("source").and_then(Value::as_str).filter(|s|!s.is_empty() && s.len()<=256 && !s.chars().any(char::is_control)).ok_or("Invalid source identifier")?;
        if p.get("master_id").and_then(Value::as_str)!=Some(master_id) || entry["id"].as_str()==Some(master_id){return Err("Selected master does not match this source reference".into());}
        let master=self.entry(master_id).ok_or("Unknown master identifier")?;
        if !master["format"].as_str().unwrap().eq_ignore_ascii_case(entry["format"].as_str().unwrap()) || master["region"]!=entry["region"]{return Err("Master format or region differs from selected entry".into());}
        let matches=master["provenance"].as_array().unwrap().iter().filter(|m|m.get("source").and_then(Value::as_str)==Some(source)).collect::<Vec<_>>();
        if matches.len()!=1{return Err("Master source reference is missing or ambiguous".into());}
        Ok((source.to_owned(),Self::source_lines(p)?,Self::source_lines(matches[0])?))
    }
    /// Report source links independently; a resolved reference never verifies
    /// encryption, operation semantics, ROM compatibility, or enablement.
    fn master_links(&self,entry:&Value)->Value{
        let links=entry["provenance"].as_array().unwrap().iter().enumerate().map(|(index,p)|{
            let source=p.get("source").and_then(Value::as_str).filter(|s|!s.is_empty() && s.len()<=256 && !s.chars().any(char::is_control));
            let raw=p.get("master_id");
            let id=raw.and_then(Value::as_str).filter(|s|s.len()==7 && s.starts_with("CH-") && s[3..].bytes().all(|b|b.is_ascii_digit()));
            let target=id.and_then(|id|self.entry(id));
            let matches=match (source,target){(Some(source),Some(target))=>Some(target["provenance"].as_array().unwrap().iter().filter(|p|p.get("source").and_then(Value::as_str)==Some(source)).count()),_=>None};
            let state=if raw.is_none(){"missing"}else if raw==Some(&Value::Null){"null"}else if id.is_none(){"invalid-id"}else if source.is_none(){"invalid-source"}else if target.is_none(){"unknown-id"}else if id==entry["id"].as_str(){"self-reference"}else if matches==Some(0){"source-mismatch"}else{"same-source-reference"};
            json!({"provenance_index":index,"source":source,"master_id":id,"state":state,"matching_master_source_records":matches,
                "master_format_matches":target.map(|t|t["format"]==entry["format"]),"master_region_matches":target.map(|t|t["region"]==entry["region"])})
        }).collect::<Vec<_>>();
        json!({"links":links,"scope":"Source metadata only; no dependency selection, encryption detection, verification or activation"})
    }
    /// A separate bounded host view; original entry data remains in the catalog.
    pub fn menu_entries(&self,rom:Option<&str>,language:input_config::Language)->Vec<menu::ContentEntry>{
        let target=Target::new(rom);
        self.entries.iter().enumerate().map(|(index,entry)|{
            let full=serde_json::to_string_pretty(entry).unwrap_or_default();
            let truncated=full.chars().count()>8192;let mut details:String=full.chars().take(8192).collect();
            if truncated{details.push_str(&format!("\n[{}]",language.text("Details abbreviated; use debug-cheats show for the complete record")));}
            let region=entry["region"].as_str().unwrap();let relation=language.text(match target.declaration(region){"declared-match"=>"Declared source region matches","declared-mismatch"=>"Declared source region differs",_=>"Source region match unknown"});
            let summary=self.indexed_summary(index,rom);
            let capability=language.text(match summary["compatibility"]["manual_once"]["state"].as_str().unwrap(){
                "requires-runtime-preflight"=>"Source plan supports a guarded one-shot preview",
                "rom-context-required"=>"Source plan needs a recognized matching ROM",
                "rom-region-mismatch"=>"Source region differs from the selected ROM",
                "master-checksum-required"=>"Source master needs a supported checksum",
                "not-evaluated"=>"Source capability was not evaluated",
                "parameter-required"=>"Source requires an explicit declared parameter",
                _=>match self.capabilities[index]["state"].as_str().unwrap(){"decoded-outside-once-limits"=>"Source plan exceeds one-shot limits","unsupported-format"=>"Source format is not supported",_=>"Source capability is unresolved"}});
            let mut reason=menu::i18n::format(language,"{relation}; {capability}. Source declaration only, not code verification. Device and behavior unverified; runtime preflight required.",&[("relation",relation.into()),("capability",capability.into())]);
            if let (Some(reports),Some(provenance))=(self.capabilities[index]["sources"].as_array(),entry["provenance"].as_array()) {
                if provenance.is_empty(){reason.push('\n');reason.push_str(language.text("No source provenance was provided"));}
                for report in reports {
                    let Some(source_index)=report["provenance_index"].as_u64().and_then(|n|usize::try_from(n).ok()) else{continue;};
                    let Some(source)=provenance.get(source_index) else{continue;};
                    let explanation=report["assessment"]["reason"].as_str().filter(|s|!s.is_empty());
                    if let Some(explanation)=explanation {
                        let clean:String=language.text(explanation).chars().filter(|c|!c.is_control()).take(200).collect();
                        reason.push('\n');reason.push_str(&menu::i18n::format(language,"Source {index}: {reason}",&[("index",source_index.to_string()),("reason",clean)]));
                    }
                    if has_undeclared_placeholders(source) {
                        reason.push('\n');reason.push_str(&menu::i18n::format(language,"Source {index}: {reason}",&[("index",source_index.to_string()),("reason",language.text("Placeholder values need a declared domain").into())]));
                    }
                }
            }
            if self.capabilities[index]["unexamined"].as_u64().unwrap_or(0)>0 {
                reason.push('\n');reason.push_str(language.text("Additional sources were not assessed"));
            }
            menu::ContentEntry{id:entry["id"].as_str().unwrap().into(),name:format!("{} · {}",entry["id"].as_str().unwrap(),entry["title"].as_str().unwrap()),
                kind:format!("{} / {}",entry["format"].as_str().unwrap(),entry["region"].as_str().unwrap()),description:details,
                location:language.text("Local catalog · read only").into(),identity:menu::i18n::format(language,"Catalog SHA256 {catalog}\nROM SHA256 {rom} · recognized region {region}",&[("catalog",self.source_sha256.clone()),("rom",target.sha256.unwrap_or_else(||language.text("Unknown ROM hash")).into()),("region",target.region.unwrap_or_else(||language.text("Unknown recognized region")).into())]),status:language.text("Cheat disabled").into(),compatibility:language.text("Cheat unverified").into(),
                reason,available:true,
                cheat_filter:Some(menu::CheatFilter{armed:false,declared_match:target.declaration(region)=="declared-match",plan_candidate:summary["compatibility"]["manual_once"]["state"]=="requires-runtime-preflight",
                    values_required:self.capabilities[index]["sources"].as_array().is_some_and(|sources|sources.iter().any(|source|source["assessment"]["state"]=="parameter-required"))})}
        }).collect()
    }
    pub fn menu_sources(&self,id:&str)->Result<Vec<menu::CheatSource>,String>{
        let index=self.entries.iter().position(|e|e["id"]==id).ok_or("Unknown CH identifier")?;
        let entry=&self.entries[index];let sources=self.capabilities[index]["sources"].as_array().ok_or("Missing source assessment")?;
        Ok(sources.iter().filter_map(|s|{
            let index=s["provenance_index"].as_u64()?;let provenance=entry["provenance"].as_array()?.get(index as usize)?;
            let assessment=&s["assessment"];let specs=if let Some(value)=assessment.get("parameter"){serde_json::from_value::<crate::debug_cheat_parameters::Specification>(value.clone()).ok().into_iter().collect()}else{assessment.get("parameters").cloned().and_then(|v|serde_json::from_value::<Vec<crate::debug_cheat_parameters::Specification>>(v).ok()).unwrap_or_default()};
            let parameters=specs.into_iter().map(|spec|menu::CheatParameter{name:spec.name,labels:spec.labels.into_iter().map(|item|(item.value,item.label)).collect(),domain:match spec.allowed{crate::debug_cheat_parameters::Domain::List{values}=>menu::CheatParameterDomain::List(values),crate::debug_cheat_parameters::Domain::Range{min,max,step}=>menu::CheatParameterDomain::Range{min,max,step},crate::debug_cheat_parameters::Domain::DependentList{on,cases}=>menu::CheatParameterDomain::DependentList{on,cases:cases.into_iter().map(|case|(case.when,case.values)).collect()}}}).collect();
            Some(menu::CheatSource{index:u16::try_from(index).ok()?,name:provenance["source"].as_str().unwrap_or("unknown").chars().take(80).collect(),master:s["master_id"].as_str().unwrap_or("unknown").chars().take(16).collect(),assessment:assessment["state"].as_str().unwrap_or("unresolved").into(),parameters})
        }).collect())
    }
}

/// Immutable, explicitly refreshed view. Queries never reopen the source: the
/// returned identity denotes the snapshot, not a claim that the file is still
/// unchanged. Any action capable of applying data must recheck its source.
pub(crate) struct Snapshot {
    catalog:Catalog,
    encoded:Vec<String>,
    encoded_sha256:Vec<String>,
    capabilities_encoded:Vec<String>,
    capabilities_sha256:Vec<String>,
    pub selection_id:String,
    pub source_sha256:String,
    pub rom_sha256:String,
    pub generation:String,
}
impl Snapshot {
    pub(crate) fn new(catalog:Catalog,selection_id:String,rom_sha256:String)->Result<Self,String>{
        let source_sha256=catalog.source_sha256.clone();
        let generation=gba_core::ffta_rom::digest(format!("{source_sha256}\n{rom_sha256}").as_bytes());
        let encoded=catalog.entries.iter().map(|v|serde_json::to_string(v).map_err(|e|e.to_string())).collect::<Result<Vec<_>,_>>()?;
        let encoded_sha256=encoded.iter().map(|s|gba_core::ffta_rom::digest(s.as_bytes())).collect();
        let capabilities_encoded=catalog.capabilities.iter().map(|v|serde_json::to_string(v).map_err(|e|e.to_string())).collect::<Result<Vec<_>,_>>()?;
        let capabilities_sha256=capabilities_encoded.iter().map(|s|gba_core::ffta_rom::digest(s.as_bytes())).collect();
        Ok(Self{catalog,encoded,encoded_sha256,capabilities_encoded,capabilities_sha256,selection_id,source_sha256,rom_sha256,generation})
    }
    pub fn menu_sources(&self,id:&str)->Result<Vec<menu::CheatSource>,String>{self.catalog.menu_sources(id)}
    pub fn matches(&self,reference:Option<&crate::debug_inputs::Reference>,rom:&str)->bool{
        rom==self.rom_sha256 && reference.is_some_and(|r|r.entry.id==self.selection_id && r.entry.sha256.as_deref()==Some(&self.source_sha256))
    }
    pub fn prepare_once(&self,reference:&crate::debug_inputs::Reference,rom:&[u8],id:&str,index:usize,master_id:&str,generation:&str,parameters:&[crate::debug_cheat_parameters::Choice])->Result<(crate::debug_cheat_decoder::Plan,Value),String>{
        self.check_generation(Some(generation))?;
        let sha=gba_core::ffta_rom::digest(rom);
        if !self.matches(Some(reference),&sha){return Err("Cheat snapshot or ROM identity changed".into());}
        let path=reference.recheck(menu::ContentSlot::CheatCatalog,&sha)?;let current=Catalog::load(&path)?;
        if current.source_sha256!=self.source_sha256{return Err("Cheat catalog changed after refresh".into());}
        let entry=current.entry(id).ok_or("Unknown CH identifier")?;
        if !entry["format"].as_str().unwrap().eq_ignore_ascii_case("codebreaker") || Target::new(Some(&sha)).declaration(entry["region"].as_str().unwrap())!="declared-match" {return Err("One-shot requires a recognized exact ROM and matching declared CodeBreaker region".into());}
        let (source,lines,master)=current.source_bound(entry,index,master_id)?;
        let provenance=&entry["provenance"][index];let (expanded,declared)=Catalog::parameterized_source(provenance,&lines,parameters)?;
        let (plan,master_info)=crate::debug_cheat_decoder::inspect_source(&expanded,&master,Some(rom))?;
        if master_info["rom"]["checksum_state"]!="match"{return Err("Source master checksum does not match the loaded ROM".into());}
        let footprint=plan.footprint()?;
        let mut info=json!({"id":id,"provenance_index":index,"source":source,"master_id":master_id,"generation":generation,"selection_id":self.selection_id,
            "source_sha256":current.source_sha256,"rom_sha256":sha,"entry_lines_sha256":gba_core::ffta_rom::digest(&serde_json::to_vec(&lines).unwrap()),
            "master_lines_sha256":gba_core::ffta_rom::digest(&serde_json::to_vec(&master).unwrap()),"master":master_info,"plan":plan,"footprint":footprint,
            "scope":"Manual paused one-shot; source/device behavior and hardware hook timing remain unverified"});
        if parameters.len()==1{info["parameter"]=json!({"choice":parameters[0],"declaration":declared[0],"scope":"Source-declared domain; not verified device or gameplay behavior"});}
        else if !parameters.is_empty(){info["parameters"]=json!({"choices":parameters,"declarations":declared,"scope":"Source-declared domains; not verified device or gameplay behavior"});}
        Ok((plan,info))
    }
    fn check_generation(&self,generation:Option<&str>)->Result<(),String>{
        if generation.is_some_and(|g|g!=self.generation){return Err("Cheat snapshot generation changed; refresh the query".into());}Ok(())
    }
    fn identity(&self)->Value {json!({"generation":self.generation,"source_sha256":self.source_sha256,"rom_sha256":self.rom_sha256,"selection_id":self.selection_id,"scope":"Immutable snapshot; source verified at refresh, not re-read by this query"})}
    pub fn page(&self,offset:u32,limit:u8,generation:Option<&str>)->Result<Value,String>{
        if !(1..=16).contains(&limit){return Err("Cheat page limit needs 1..16".into());}self.check_generation(generation)?;
        let start=offset as usize;if start>self.catalog.entries.len(){return Err("Cheat page offset exceeds entry count".into());}let end=start.saturating_add(limit as usize).min(self.catalog.entries.len());
        let entries=if start>=end{Vec::new()}else{(start..end).map(|index|{
            let mut v=self.catalog.indexed_summary(index,Some(&self.rom_sha256));let mut truncated=false;
            for key in ["title","section","format","region","source_status"] {if let Some(text)=v[key].as_str(){truncated|=text.chars().count()>128;let shortened:String=text.chars().take(128).collect();v[key]=json!(shortened);}}
            v["display_truncated"]=json!(truncated);
            v
        }).collect::<Vec<_>>()};
        Ok(json!({"snapshot":self.identity(),"offset":offset,"total":self.catalog.entries.len(),"next_offset":if end<self.catalog.entries.len(){Some(end)}else{None},"entries":entries,"display_fields_capped":128}))
    }
    pub fn show(&self,id:&str,offset:u32,generation:Option<&str>)->Result<Value,String>{
        self.check_generation(generation)?;let index=self.catalog.entries.iter().position(|e|e["id"]==id).ok_or("Unknown CH identifier")?;
        let value=&self.encoded[index];let start=offset as usize;if start>value.len() || !value.is_char_boundary(start){return Err("Detail offset is not a valid UTF-8 boundary".into());}
        let mut end=start.saturating_add(4096).min(value.len());while !value.is_char_boundary(end){end-=1;}
        Ok(json!({"snapshot":self.identity(),"id":id,"offset_bytes":start,"total_bytes":value.len(),"entry_sha256":self.encoded_sha256[index],"json_part":&value[start..end],"next_offset_bytes":if end<value.len(){Some(end)}else{None},"scope":"Concatenate UTF-8 parts for the complete original entry JSON"}))
    }
    pub fn capabilities(&self,id:&str,offset:u32,generation:Option<&str>)->Result<Value,String>{
        self.check_generation(generation)?;let index=self.catalog.entries.iter().position(|e|e["id"]==id).ok_or("Unknown CH identifier")?;
        let value=&self.capabilities_encoded[index];let start=offset as usize;if start>value.len() || !value.is_char_boundary(start){return Err("Capability offset is not a valid UTF-8 boundary".into());}
        let mut end=start.saturating_add(4096).min(value.len());while !value.is_char_boundary(end){end-=1;}
        Ok(json!({"snapshot":self.identity(),"id":id,"offset_bytes":start,"total_bytes":value.len(),"capability_sha256":self.capabilities_sha256[index],"json_part":&value[start..end],"next_offset_bytes":if end<value.len(){Some(end)}else{None},"scope":"Concatenate UTF-8 parts for the source-assumed capability report; not ROM, device or gameplay verification"}))
    }
}

pub(crate) struct Refresh {
    handle:std::thread::JoinHandle<Result<Snapshot,String>>,
    cancelled:bool,
}
impl Refresh {
    pub fn start(reference:crate::debug_inputs::Reference,rom:String)->Result<Self,String>{
        let id=reference.entry.id.clone();let expected=reference.entry.sha256.clone().ok_or("Selected catalog has no source hash")?;
        Ok(Self{cancelled:false,handle:std::thread::spawn(move||{
            let path=reference.recheck(menu::ContentSlot::CheatCatalog,&rom)?;let catalog=Catalog::load(&path)?;
            if catalog.source_sha256!=expected{return Err("Catalog changed during refresh; select it again".into());}
            Snapshot::new(catalog,id,rom)
        })})
    }
    pub fn cancel(&mut self){self.cancelled=true;}
    pub fn is_finished(&self)->bool{self.handle.is_finished()}
    pub fn finish(self)->Result<Snapshot,String>{
        let result=self.handle.join().unwrap_or_else(|_|Err("Cheat refresh worker failed".into()));
        if self.cancelled{Err("Cheat refresh control-cancelled".into())}else{result}
    }
}

fn decimal(value:&str)->Result<usize,String>{
    if value.is_empty()||!value.bytes().all(|v|v.is_ascii_digit())||(value.len()>1&&value.starts_with('0')){return Err("Use unambiguous decimal numbers".into());}
    value.parse().map_err(|_|"Number out of range".into())
}
fn execute(args:&[String],default_root:&Path)->Result<Value,String>{
    let operation=args.first().map(String::as_str).ok_or("Use debug-cheats list | show ID | inspect ID | info")?;
    let mut pos=1;let id=if operation=="show" || operation=="inspect"{let id=args.get(pos).ok_or("Show or inspect needs a CH ID")?.clone();pos+=1;Some(id)}else{None};
    if !["list","show","inspect","info"].contains(&operation){return Err("Use debug-cheats list | show ID | inspect ID | info".into());}
    let(mut path,mut rom,mut offset,mut limit,mut assumed,mut seed,mut source_index,mut master_id,mut rom_file)=(None,None,0,MAX_PAGE,false,None,None,None,None);let mut seen=BTreeSet::new();
    while pos<args.len(){let key=&args[pos];if !seen.insert(key.clone()){return Err("Duplicate option".into());}pos+=1;
        if key=="--assume-plaintext-codebreaker" && operation=="inspect" {assumed=true;continue;}
        let value=args.get(pos).ok_or("Option needs a value")?;pos+=1;
        match key.as_str(){
            "--catalog"=>path=Some(PathBuf::from(value)),
            "--rom-sha256"=>{if value.len()!=64||!value.bytes().all(|b|b.is_ascii_hexdigit()){return Err("ROM identity needs a SHA-256".into());}rom=Some(value.to_ascii_lowercase());},
            "--codebreaker-seed" if operation=="inspect"=>seed=Some(value.clone()),
            "--provenance-index" if operation=="inspect"=>source_index=Some(decimal(value)?),
            "--master-id" if operation=="inspect"=>master_id=Some(value.clone()),
            "--rom-file" if operation=="inspect"=>rom_file=Some(PathBuf::from(value)),
            "--offset" if operation=="list"=>offset=decimal(value)?,
            "--limit" if operation=="list"=>{limit=decimal(value)?;if !(1..=MAX_PAGE).contains(&limit){return Err("Page limit needs 1..64".into());}},
            _=>return Err("Unknown option".into()),
        }
    }
    if operation=="inspect" {
        if source_index.is_some()!=master_id.is_some(){return Err("Source inspection requires both --provenance-index and --master-id".into());}
        if usize::from(assumed)+usize::from(seed.is_some())+usize::from(source_index.is_some())!=1{return Err("Inspect requires exactly one plaintext, explicit-seed or source-bound mode".into());}
        if rom_file.is_some() && (source_index.is_none() || rom.is_some()){return Err("--rom-file requires source-bound mode and cannot be combined with --rom-sha256".into());}
    }
    let explicit=path.is_some();let path=path.unwrap_or_else(||default_root.join("cheats").join("catalogo.json"));
    if !explicit && !path.exists(){
        return Ok(json!({"schema":1,"kind":"ffta-debug-cheat-catalog","outcome":"empty","entries":[],"total":0,"message":"No local catalog. Import explicitly with --catalog FILE.","enabled_count":0}));
    }
    let catalog=Catalog::load(&path)?;let total=catalog.entries.len();let identity=rom.as_deref();
    let payload=match operation{
        "list"=>{let end=offset.saturating_add(limit).min(total);let entries=if offset>=total{Vec::new()}else{(offset..end).map(|i|catalog.indexed_summary(i,identity)).collect()};
            json!({"entries":entries,"offset":offset,"limit":limit,"next_offset":if end<total{Some(end)}else{None}})},
        "show"=>{let index=catalog.entries.iter().position(|e|e["id"]==id.as_deref().unwrap()).ok_or("Unknown CH identifier")?;let entry=&catalog.entries[index];json!({"summary":catalog.indexed_summary(index,identity),"entry":entry,"source_capabilities":catalog.capabilities[index],"master_dependencies":catalog.master_links(entry)})},
        "inspect"=>{let entry=catalog.entry(id.as_deref().unwrap()).ok_or("Unknown CH identifier")?;
            if !entry["format"].as_str().unwrap().eq_ignore_ascii_case("codebreaker"){return Err("Inspection supports only entries declared CodeBreaker".into());}
            let entry_index=catalog.entries.iter().position(|e|e["id"]==id.as_deref().unwrap()).unwrap();
            if let Some(index)=source_index {
                let master_id=master_id.as_deref().unwrap();let (source,lines,master)=catalog.source_bound(entry,index,master_id)?;
                let rom_bytes=if let Some(path)=rom_file.as_ref(){crate::debug_content::ordinary(path)?;Some(crate::diagnostics::read(path,32<<20)?)}else{None};
                let (plan,master_info)=crate::debug_cheat_decoder::inspect_source(&lines,&master,rom_bytes.as_deref())?;
                let actual_sha=rom_bytes.as_deref().map(gba_core::ffta_rom::digest);let target_sha=actual_sha.as_deref().or(identity);
                let canonical=entry["codes"].as_array().unwrap().iter().map(|v|v.as_str().unwrap()).eq(lines.iter().map(String::as_str));
                json!({"summary":catalog.indexed_summary(entry_index,target_sha),"plan":plan,"master":master_info,"source":{"provenance_index":index,"source":source,"master_id":master_id,
                    "entry_lines_sha256":gba_core::ffta_rom::digest(&serde_json::to_vec(&lines).unwrap()),"master_lines_sha256":gba_core::ffta_rom::digest(&serde_json::to_vec(&master).unwrap()),"canonical_lines_equal":canonical},"master_dependencies":catalog.master_links(entry)})
            }else{
            let lines=entry["codes"].as_array().unwrap().iter().map(|v|v.as_str().unwrap().to_owned()).collect::<Vec<_>>();
            let plan=if let Some(seed)=seed.as_deref(){crate::debug_cheat_decoder::inspect_seeded(&lines,seed)?}else{crate::debug_cheat_decoder::inspect(&lines)?};
            json!({"summary":catalog.indexed_summary(entry_index,identity),"plan":plan,"supplied_seed":seed,"master_dependencies":catalog.master_links(entry)})}},
        _=>json!({"metadata":catalog.metadata}),
    };
    Ok(json!({"schema":1,"kind":"ffta-debug-cheat-catalog","outcome":"loaded","source_sha256":catalog.source_sha256,"total":total,"enabled_count":0,"result":payload,"scope":"Readonly reference and explicitly assumed inspection only; no guest writes"}))
}
pub(crate) fn cmd(args:&[String])->u8{
    let Some(root)=input_config::exe_dir() else {eprintln!("Cannot locate executable directory");return 1;};
    match execute(args,&root){Ok(value)=>{println!("{value}");0},Err(error)=>{println!("{}",json!({"schema":1,"kind":"ffta-debug-cheat-error","outcome":"invalid-request","error":error}));1}}
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture()->Value{json!({"schema":1,"sources":[{"id":"synthetic","sha256":"a".repeat(64)}],"entries":[
        {"id":"CH-0001","format":"codebreaker","region":"EUR","section":"synthetic","title":"Example","codes":["FIRST","SECOND","FIRST"],"notes":["note"],"aliases":["alias"],"status":"not-runtime-tested","provenance":[{"source":"synthetic","master_id":"CH-0002","raw_code_lines":["FIRST","SECOND","FIRST"]}]},
        {"id":"CH-0002","format":"unknown-format","region":"unknown","section":"synthetic","title":"Master","codes":[],"notes":[],"aliases":[],"status":"not-runtime-tested","provenance":[]}]})}
    #[test]
    fn imports_order_provenance_and_inert_separate_states(){
        let v=fixture();let bytes=serde_json::to_vec(&v).unwrap();let catalog=Catalog::parse(&bytes).unwrap();assert_eq!(catalog.entries,v["entries"].as_array().unwrap().clone());assert_eq!(catalog.metadata["sources"],v["sources"]);
        assert_eq!(catalog.entry("CH-0001").unwrap()["codes"],json!(["FIRST","SECOND","FIRST"]));assert_eq!(catalog.entry("CH-0001").unwrap()["provenance"][0]["master_id"],"CH-0002");
        let result=catalog.indexed_summary(0,Some(&"a".repeat(64)));assert_eq!(result["enabled"],false);assert_eq!(result["executable"],false);assert_eq!(result["compatibility"]["rom"]["state"],"unknown");assert_eq!(result["compatibility"]["decoder"]["state"],"unresolved");assert_eq!(result["compatibility"]["verification"]["state"],"unverified");
    }
    #[test]
    fn menu_explains_missing_source_dependencies_and_placeholder_domains(){
        let mut v=fixture();
        v["entries"][0]["codes"]=json!(["82000084 XXYY"]);
        v["entries"][0]["provenance"]=json!([
            {"source":"missing","raw_code_lines":["82000084 XXYY"]},
            {"source":"other","master_id":"CH-0002","raw_code_lines":["82000084 XXYY"]}
        ]);
        let catalog=Catalog::parse(&serde_json::to_vec(&v).unwrap()).unwrap();
        let en=catalog.menu_entries(None,input_config::Language::default());
        let it=catalog.menu_entries(None,input_config::Language::parse("it"));
        assert!(en[0].reason.contains("Source 0: Source lines or master reference are missing"));
        assert!(en[0].reason.contains("Source 0: Placeholder values need a declared domain"));
        assert!(en[0].reason.contains("Source 1: Source master is missing"));
        assert!(it[0].reason.contains("Fonte 0: Mancano le righe della fonte o il riferimento al master"));
        assert!(it[0].reason.contains("Fonte 0: I valori segnaposto richiedono un dominio dichiarato"));
        assert_eq!(catalog.entries,v["entries"].as_array().unwrap().clone());
        assert_eq!(catalog.indexed_summary(0,None)["executable"],false);
        let mut declared=v["entries"][0]["provenance"][0].clone();declared["parameter"]=json!({});
        assert!(!has_undeclared_placeholders(&declared));
        assert!(!has_undeclared_placeholders(&json!({"raw_code_lines":["not a code x"]})));
        let mut missing=v.clone();missing["entries"][0]["provenance"]=json!([]);
        let empty=Catalog::parse(&serde_json::to_vec(&missing).unwrap()).unwrap();
        assert!(empty.menu_entries(None,input_config::Language::default())[0].reason.contains("No source provenance was provided"));
        let mut noisy=catalog;noisy.capabilities[0]["sources"][0]["assessment"]["reason"]=json!(format!("bad\r{}","x".repeat(1000)));
        let reason=&noisy.menu_entries(None,input_config::Language::default())[0].reason;
        assert!(!reason.contains('\r'));assert!(reason.len()<1000);
    }
    #[test]
    fn inspection_requires_explicit_assumption_and_does_not_enable_a_catalog_entry(){
        let dir=tempfile::tempdir().unwrap();let path=dir.path().join("catalog.json");let mut v=fixture();
        v["entries"][0]["codes"]=json!(["32000001 00FF","82000002 1234"]);
        std::fs::write(&path,serde_json::to_vec(&v).unwrap()).unwrap();let path=path.to_str().unwrap();
        let args=|s:&[&str]|s.iter().map(|v|v.to_string()).collect::<Vec<_>>();
        assert!(execute(&args(&["inspect","CH-0001","--catalog",path]),dir.path()).is_err());
        let result=execute(&args(&["inspect","CH-0001","--assume-plaintext-codebreaker","--catalog",path]),dir.path()).unwrap();
        assert_eq!(result["result"]["plan"]["executable"],false);assert_eq!(result["result"]["plan"]["operations"].as_array().unwrap().len(),2);
        assert_eq!(result["result"]["summary"]["compatibility"]["device_version"]["state"],"unknown");
        assert!(execute(&args(&["inspect","CH-0002","--assume-plaintext-codebreaker","--catalog",path]),dir.path()).is_err());
        v["entries"][0]["codes"]=json!(["32000001 00FF","92000000 0000"]);std::fs::write(&path,serde_json::to_vec(&v).unwrap()).unwrap();
        assert!(execute(&args(&["inspect","CH-0001","--assume-plaintext-codebreaker","--catalog",path]),dir.path()).is_err());
    }
    #[test]
    fn seeded_cli_requires_an_explicit_unambiguous_mode(){
        let dir=tempfile::tempdir().unwrap();let path=dir.path().join("catalog.json");let mut v=fixture();
        v["entries"][0]["codes"]=json!(["BB28F300 B8C0","8C8303D6 5E13","2C940CDC 920B"]);
        std::fs::write(&path,serde_json::to_vec(&v).unwrap()).unwrap();let p=path.to_str().unwrap();
        let args=|s:&[&str]|s.iter().map(|v|v.to_string()).collect::<Vec<_>>();
        let result=execute(&args(&["inspect","CH-0001","--codebreaker-seed","9ABCDEF0:1234","--catalog",p]),dir.path()).unwrap();
        assert_eq!(result["result"]["plan"]["format"],"codebreaker-advance-seeded-subset");assert_eq!(result["result"]["plan"]["executable"],false);
        assert_eq!(result["result"]["plan"]["operations"][0]["value"],255);assert_eq!(result["result"]["plan"]["operations"][1]["count"],4);
        assert_eq!(result["result"]["supplied_seed"],"9ABCDEF0:1234");
        assert!(execute(&args(&["inspect","CH-0001","--codebreaker-seed","9ABCDEF0:1234","--assume-plaintext-codebreaker","--catalog",p]),dir.path()).is_err());
        assert!(execute(&args(&["show","CH-0001","--codebreaker-seed","9ABCDEF0:1234","--catalog",p]),dir.path()).is_err());
        assert!(execute(&args(&["inspect","CH-0001","--codebreaker-seed","not-a-seed","--catalog",p]),dir.path()).is_err());
    }
    #[test]
    fn source_bound_cli_requires_exact_unambiguous_provenance_and_checks_rom(){
        let dir=tempfile::tempdir().unwrap();let path=dir.path().join("catalog.json");let rom=dir.path().join("synthetic.gba");let mut v=fixture();
        v["entries"][0]["codes"]=json!(["BB28F300 B8C0"]);v["entries"][0]["provenance"][0]["raw_code_lines"]=v["entries"][0]["codes"].clone();
        v["entries"][1]["format"]=json!("codebreaker");v["entries"][1]["region"]=json!("EUR");
        v["entries"][1]["provenance"]=json!([{"source":"synthetic","raw_code_lines":["9ABCDEF0 1234","B131BCEF 0691","26960CD0 140B"]}]);
        std::fs::write(&rom,(0..65536).map(|n|n as u8).collect::<Vec<_>>()).unwrap();std::fs::write(&path,serde_json::to_vec(&v).unwrap()).unwrap();
        let p=path.to_str().unwrap();let rp=rom.to_str().unwrap();let args=|s:&[&str]|s.iter().map(|v|v.to_string()).collect::<Vec<_>>();
        let a=["inspect","CH-0001","--provenance-index","0","--master-id","CH-0002","--rom-file",rp,"--catalog",p];
        let result=execute(&args(&a),dir.path()).unwrap();assert_eq!(result["result"]["master"]["rom"]["checksum_state"],"match");
        assert_eq!(result["result"]["source"]["canonical_lines_equal"],true);assert_eq!(result["result"]["plan"]["executable"],false);
        assert!(execute(&args(&["inspect","CH-0001","--provenance-index","0","--catalog",p]),dir.path()).is_err());
        assert!(execute(&args(&["inspect","CH-0001","--provenance-index","0","--master-id","CH-9999","--catalog",p]),dir.path()).is_err());
        assert!(execute(&args(&["inspect","CH-0001","--provenance-index","0","--master-id","CH-0002","--assume-plaintext-codebreaker","--catalog",p]),dir.path()).is_err());
        v["entries"][1]["provenance"][0]["source"]=json!("other");std::fs::write(&path,serde_json::to_vec(&v).unwrap()).unwrap();assert!(execute(&args(&a),dir.path()).is_err());
        v["entries"][1]["provenance"]=json!([{"source":"synthetic","raw_code_lines":["9ABCDEF0 1234","B131BCEF 0691","26960CD0 140B"]},{"source":"synthetic","raw_code_lines":["9ABCDEF0 1234","B131BCEF 0691","26960CD0 140B"]}]);std::fs::write(&path,serde_json::to_vec(&v).unwrap()).unwrap();assert!(execute(&args(&a),dir.path()).is_err());
    }
    #[test]
    fn per_source_capabilities_distinguish_decoding_context_and_limits(){
        let mut v=fixture();v["entries"][0]["codes"]=json!(["BB28F300 B8C0"]);
        v["entries"][0]["provenance"]=json!([
            {"source":"synthetic","master_id":"CH-0002","raw_code_lines":["BB28F300 B8C0"]},
            {"source":"synthetic","master_id":"CH-0002","raw_code_lines":["4CD38AFF 7753"]},
            {"source":"synthetic","master_id":"CH-0002","raw_code_lines":vec!["BB28F300 B8C0";65]},
            {"source":"other","master_id":"CH-0002","raw_code_lines":["BB28F300 B8C0"]}]);
        v["entries"][1]["format"]=json!("codebreaker");v["entries"][1]["region"]=json!("EUR");
        v["entries"][1]["provenance"]=json!([{"source":"synthetic","raw_code_lines":["9ABCDEF0 1234","B131BCEF 0691","26960CD0 140B"]}]);
        let catalog=Catalog::parse(&serde_json::to_vec(&v).unwrap()).unwrap();let report=&catalog.capabilities[0];
        assert_eq!(report["state"],"manual-once-subset");assert_eq!(report["sources"][0]["assessment"]["state"],"manual-once-subset");
        assert_eq!(report["sources"][0]["assessment"]["plan"]["operations"][0]["kind"],"write");
        assert_eq!(report["sources"][1]["assessment"]["state"],"not-decoded");assert_eq!(report["sources"][2]["assessment"]["state"],"not-evaluated");assert_eq!(report["sources"][3]["assessment"]["state"],"unresolved");
        for (sha,state) in [(Some(gba_core::ffta_rom::EUROPE_SHA256),"requires-runtime-preflight"),(Some(gba_core::ffta_rom::SHA256),"rom-region-mismatch"),(None,"rom-context-required")]{
            let summary=catalog.indexed_summary(0,sha);assert_eq!(summary["compatibility"]["manual_once"]["state"],state);assert_eq!(summary["compatibility"]["manual_once"]["unexamined_sources"],1);assert_eq!(summary["enabled"],false);assert_eq!(summary["executable"],false);assert_eq!(summary["compatibility"]["verification"]["state"],"unverified");
        }
        assert_eq!(catalog.indexed_summary(1,None)["compatibility"]["decoder"]["state"],"unresolved");
        let menu=catalog.menu_entries(Some(gba_core::ffta_rom::EUROPE_SHA256),input_config::Language::default());assert!(menu[0].reason.contains("guarded one-shot preview"));assert!(menu[0].reason.contains("behavior unverified"));
        assert_eq!(catalog.entries,v["entries"].as_array().unwrap().clone());
        let mut none=0;let bounded=catalog.assess(&catalog.entries[0],&mut none);assert_eq!(bounded["state"],"not-evaluated");
        assert_eq!(bounded["sources"][0]["assessment"]["state"],"not-evaluated");
        let mut limited=v.clone();limited["entries"][0]["provenance"]=json!([{"source":"synthetic","master_id":"CH-0002","raw_code_lines":vec!["BB28F300 B8C0";65]}]);
        let limited=Catalog::parse(&serde_json::to_vec(&limited).unwrap()).unwrap();assert_eq!(limited.indexed_summary(0,None)["compatibility"]["manual_once"]["state"],"not-evaluated");assert!(limited.menu_entries(None,input_config::Language::default())[0].reason.contains("was not evaluated"));
        let mut mixed=limited;mixed.capabilities[0]=json!({"state":"decoded-outside-once-limits","unexamined":0,"sources":[{"assessment":{"state":"decoded-outside-once-limits"}},{"assessment":{"state":"not-evaluated"}}]});
        assert_eq!(mixed.indexed_summary(0,None)["compatibility"]["manual_once"]["state"],"not-evaluated");assert!(mixed.menu_entries(None,input_config::Language::default())[0].reason.contains("was not evaluated"));
        let mut more=v.clone();more["entries"][0]["provenance"]=json!(vec![json!({"source":"synthetic","master_id":"CH-0002","raw_code_lines":["BB28F300 B8C0"]});10]);
        let c=Catalog::parse(&serde_json::to_vec(&more).unwrap()).unwrap();assert_eq!(c.capabilities[0]["unexamined"],2);
        assert_eq!(c.indexed_summary(0,None)["compatibility"]["manual_once"]["unexamined_sources"],2);
    }
    #[test]
    fn declared_parameter_requires_choice_and_preserves_source_before_decode(){
        let mut v=fixture();v["entries"][0]["codes"]=json!(["BB28F30X B8C0"]);
        v["entries"][0]["provenance"]=json!([{"source":"synthetic","master_id":"CH-0002","raw_code_lines":["BB28F30X B8C0"],
            "parameter":{"name":"amount","allowed":{"kind":"list","values":[0]},"slots":[{"line":0,"field":"word","offset":7,"digits":1}]}}]);
        v["entries"][1]["format"]=json!("codebreaker");v["entries"][1]["region"]=json!("EUR");
        v["entries"][1]["provenance"]=json!([{"source":"synthetic","raw_code_lines":["9ABCDEF0 1234","B131BCEF 0691","26960CD0 140B"]}]);
        let c=Catalog::parse(&serde_json::to_vec(&v).unwrap()).unwrap();let report=&c.capabilities[0];
        assert_eq!(report["state"],"parameter-required");assert_eq!(report["sources"][0]["assessment"]["parameter"]["name"],"amount");
        for sha in [Some(gba_core::ffta_rom::EUROPE_SHA256), Some(gba_core::ffta_rom::SHA256), None] {
            let menu=c.menu_entries(sha,input_config::Language::default());
            let filter=menu[0].cheat_filter.as_ref().unwrap();
            assert!(filter.values_required);
            assert!(!filter.plan_candidate);
            assert!(!menu[1].cheat_filter.as_ref().unwrap().values_required);
        }
        let mut mixed=v.clone();
        mixed["entries"][0]["provenance"].as_array_mut().unwrap().push(json!({"source":"synthetic","master_id":"CH-0002","raw_code_lines":["BB28F300 B8C0"]}));
        let mixed=Catalog::parse(&serde_json::to_vec(&mixed).unwrap()).unwrap();
        assert_eq!(mixed.capabilities[0]["state"],"manual-once-subset");
        let filter=mixed.menu_entries(Some(gba_core::ffta_rom::EUROPE_SHA256),input_config::Language::default())[0].cheat_filter.clone().unwrap();
        assert!(filter.plan_candidate && filter.values_required);
        let mut invalid=v.clone();invalid["entries"][0]["provenance"][0]["parameter"]["allowed"]["values"]=json!([]);
        let invalid=Catalog::parse(&serde_json::to_vec(&invalid).unwrap()).unwrap();
        assert!(!invalid.menu_entries(None,input_config::Language::default())[0].cheat_filter.as_ref().unwrap().values_required);
        let sources=c.menu_sources("CH-0001").unwrap();assert_eq!(sources.len(),1);
        assert_eq!(sources[0].parameters,vec![menu::CheatParameter{name:"amount".into(),domain:menu::CheatParameterDomain::List(vec![0]),labels:vec![]}]);
        let source=&c.entries[0]["provenance"][0];let (name,raw,master)=c.source_bound(&c.entries[0],0,"CH-0002").unwrap();assert_eq!(name,"synthetic");
        assert!(Catalog::parameterized_source(source,&raw,&[]).is_err());
        let good=crate::debug_cheat_parameters::Choice{name:"amount".into(),value:0};
        let (expanded,_)=Catalog::parameterized_source(source,&raw,std::slice::from_ref(&good)).unwrap();
        let (plan,_)=crate::debug_cheat_decoder::inspect_source(&expanded,&master,None).unwrap();assert!(plan.footprint().is_ok());
        assert!(Catalog::parameterized_source(source,&raw,&[crate::debug_cheat_parameters::Choice{name:"amount".into(),value:1}]).is_err());
        assert_eq!(c.entries,v["entries"].as_array().unwrap().clone());
    }
    #[test]
    fn plaintext_master_capability_keeps_unknown_region_unverified() {
        let mut v=fixture();
        v["entries"][0]["region"]=json!("unknown");
        v["entries"][0]["provenance"]=json!([{"source":"synthetic","master_id":"CH-0002","raw_code_lines":["3200000X 0001"],
            "parameter":{"name":"offset","allowed":{"kind":"list","values":[1]},"slots":[{"line":0,"field":"word","offset":7,"digits":1}]}}]);
        v["entries"][1]["format"]=json!("codebreaker");v["entries"][1]["region"]=json!("unknown");
        v["entries"][1]["provenance"]=json!([{"source":"synthetic","raw_code_lines":["0000FFE7 000A","10000100 0007"]}]);
        let catalog=Catalog::parse(&serde_json::to_vec(&v).unwrap()).unwrap();
        assert_eq!(catalog.capabilities[0]["state"],"parameter-required");
        assert_eq!(catalog.indexed_summary(0,Some(gba_core::ffta_rom::SHA256))["compatibility"]["rom"]["state"],"unknown");
        assert_eq!(catalog.entries,v["entries"].as_array().unwrap().clone());
        v["entries"][1]["provenance"][0]["raw_code_lines"][1]=json!("10000101 0007");
        let malformed=Catalog::parse(&serde_json::to_vec(&v).unwrap()).unwrap();
        assert_eq!(malformed.capabilities[0]["state"],"unresolved");
    }
    #[test]
    #[ignore = "Requires protected known USA and EUR ROMs and validation work"]
    fn plaintext_master_checks_actual_rom_before_synthetic_write() {
        use gba_core::Bus;
        let work=std::path::PathBuf::from(std::env::var_os("FFTA_VALIDATION_WORK").unwrap());
        let dir=tempfile::tempdir_in(work).unwrap();
        for (region,var,expected) in [("USA","FFTA_PLAIN_TEST_ROM_USA",true),("EUR","FFTA_PLAIN_TEST_ROM_EUR",false)] {
            let rom=std::fs::read(std::env::var_os(var).unwrap()).unwrap();
            let mut value=fixture();
            value["entries"][0]["region"]=json!(region);
            value["entries"][0]["provenance"]=json!([{"source":"synthetic","master_id":"CH-0002","raw_code_lines":["3200000X 0001"],
                "parameter":{"name":"offset","allowed":{"kind":"list","values":[1]},"slots":[{"line":0,"field":"word","offset":7,"digits":1}]}}]);
            value["entries"][1]["format"]=json!("codebreaker");value["entries"][1]["region"]=json!(region);
            value["entries"][1]["provenance"]=json!([{"source":"synthetic","raw_code_lines":["000037AE 000A","10000488 0007"]}]);
            let folder=dir.path().join(region);std::fs::create_dir(&folder).unwrap();
            let path=folder.join("catalogo.json");std::fs::write(&path,serde_json::to_vec(&value).unwrap()).unwrap();
            let sha=gba_core::ffta_rom::digest(&rom);let reference=crate::debug_inputs::Reference::explicit(&path,menu::ContentSlot::CheatCatalog,&sha).unwrap();
            let snapshot=Snapshot::new(Catalog::load(&path).unwrap(),reference.entry.id.clone(),sha).unwrap();
            let choice=crate::debug_cheat_parameters::Choice{name:"offset".into(),value:1};
            let result=snapshot.prepare_once(&reference,&rom,"CH-0001",0,"CH-0002",&snapshot.generation,&[choice]);
            if expected {
                let (plan,info)=result.unwrap();assert_eq!(info["master"]["rom"]["checksum_state"],"match");
                let mut machine=gba_core::Machine::new(vec![]);assert_eq!(plan.apply_once(&mut machine.bus).unwrap(),1);assert_eq!(machine.bus.read8(0x02000001),1);
            } else { assert!(result.unwrap_err().contains("checksum")); }
        }
    }
    #[test]
    #[ignore = "Requires a protected derived catalog and recognized USA and EUR ROMs"]
    fn derived_usa_ability_catalog_obeys_source_and_rom_guards() {
        use gba_core::Bus;
        let path=std::path::PathBuf::from(std::env::var_os("FFTA_ABILITY_PROJECTION").unwrap());
        let usa=std::fs::read(std::env::var_os("FFTA_PLAIN_TEST_ROM_USA").unwrap()).unwrap();
        let eur=std::fs::read(std::env::var_os("FFTA_PLAIN_TEST_ROM_EUR").unwrap()).unwrap();
        let catalog_bytes=std::fs::read(&path).unwrap();
        let choice=crate::debug_cheat_parameters::Choice{name:"ability_set_label".into(),value:1};
        for (rom,should_pass) in [(&usa,true),(&eur,false)] {
            let sha=gba_core::ffta_rom::digest(rom);let reference=crate::debug_inputs::Reference::explicit(&path,menu::ContentSlot::CheatCatalog,&sha).unwrap();
            let snapshot=Snapshot::new(Catalog::load(&path).unwrap(),reference.entry.id.clone(),sha).unwrap();
            let result=snapshot.prepare_once(&reference,rom,"CH-0226",0,"CH-0225",&snapshot.generation,std::slice::from_ref(&choice));
            if should_pass {
                let (plan,info)=result.unwrap();assert_eq!(info["master"]["rom"]["checksum_state"],"match");
                assert_eq!(info["parameter"]["choice"]["value"],1);
                let address=info["plan"]["operations"][0]["address"].as_u64().unwrap() as u32;
                let mut machine=gba_core::Machine::new(vec![]);assert_eq!(plan.apply_once(&mut machine.bus).unwrap(),1);assert_eq!(machine.bus.read8(address),1);
                let invalid=crate::debug_cheat_parameters::Choice{name:"ability_set_label".into(),value:35};
                assert!(snapshot.prepare_once(&reference,rom,"CH-0226",0,"CH-0225",&snapshot.generation,&[invalid]).is_err());
            } else { assert!(result.is_err()); }
        }
        assert_eq!(std::fs::read(path).unwrap(),catalog_bytes);
    }
    #[test]
    fn multiple_declared_parameters_map_to_menu_and_decode_together(){
        let mut v=fixture();v["entries"][0]["codes"]=json!(["BB28F3XY B8CZ"]);
        let spec=|name:&str,field:&str,offset:u8|json!({"name":name,"allowed":{"kind":"list","values":[0]},"slots":[{"line":0,"field":field,"offset":offset,"digits":1}]});
        v["entries"][0]["provenance"]=json!([{"source":"synthetic","master_id":"CH-0002","raw_code_lines":v["entries"][0]["codes"],"parameters":[spec("race","word",6),spec("job","word",7),spec("look","value",3)]}]);
        v["entries"][1]["format"]=json!("codebreaker");v["entries"][1]["region"]=json!("EUR");v["entries"][1]["provenance"]=json!([{"source":"synthetic","raw_code_lines":["9ABCDEF0 1234","B131BCEF 0691","26960CD0 140B"]}]);
        let c=Catalog::parse(&serde_json::to_vec(&v).unwrap()).unwrap();assert_eq!(c.capabilities[0]["state"],"parameter-required");let sources=c.menu_sources("CH-0001").unwrap();assert_eq!(sources[0].parameters.len(),3);
        let (_,raw,master)=c.source_bound(&c.entries[0],0,"CH-0002").unwrap();let source=&c.entries[0]["provenance"][0];
        let choices=[crate::debug_cheat_parameters::Choice{name:"race".into(),value:0},crate::debug_cheat_parameters::Choice{name:"job".into(),value:0},crate::debug_cheat_parameters::Choice{name:"look".into(),value:0}];
        assert!(Catalog::parameterized_source(source,&raw,&choices[..2]).is_err());let (expanded,declared)=Catalog::parameterized_source(source,&raw,&choices).unwrap();assert_eq!(declared.len(),3);assert_eq!(expanded[0],"BB28F300 B8C0");
        assert!(crate::debug_cheat_decoder::inspect_source(&expanded,&master,None).unwrap().0.footprint().is_ok());
    }
    #[test]
    fn dependent_parameter_catalog_maps_menu_and_rejects_cross_combinations(){
        let mut v=fixture();v["entries"][0]["codes"]=json!(["820000XX 00YY"]);
        v["entries"][0]["provenance"]=json!([{"source":"synthetic","master_id":"CH-0002","raw_code_lines":v["entries"][0]["codes"],"parameters":[
            {"name":"race","allowed":{"kind":"list","values":[1,2]},"slots":[{"line":0,"field":"word","offset":6,"digits":2}]},
            {"name":"job","allowed":{"kind":"dependent-list","on":"race","cases":[{"when":1,"values":[10,11]},{"when":2,"values":[20]}]},"slots":[{"line":0,"field":"value","offset":2,"digits":2}]}
        ]}]);
        v["entries"][1]["format"]=json!("codebreaker");v["entries"][1]["region"]=json!("EUR");v["entries"][1]["provenance"]=json!([{"source":"synthetic","raw_code_lines":["0000FFE7 000A","10000100 0007"]}]);
        let c=Catalog::parse(&serde_json::to_vec(&v).unwrap()).unwrap();assert_eq!(c.capabilities[0]["state"],"parameter-required");
        let source=c.menu_sources("CH-0001").unwrap().remove(0);
        assert_eq!(source.parameters[1].domain,menu::CheatParameterDomain::DependentList{on:"race".into(),cases:vec![(1,vec![10,11]),(2,vec![20])]});
        let (_,lines,_)=c.source_bound(&c.entries[0],0,"CH-0002").unwrap();let p=&c.entries[0]["provenance"][0];
        let choice=|name:&str,value|crate::debug_cheat_parameters::Choice{name:name.into(),value};
        assert!(Catalog::parameterized_source(p,&lines,&[choice("race",1),choice("job",20)]).is_err());
        assert_eq!(Catalog::parameterized_source(p,&lines,&[choice("race",2),choice("job",20)]).unwrap().0,["82000002 0014"]);
    }
    #[test]
    #[ignore = "Requires an explicitly protected EUR ROM fixture and validation work directory"]
    fn declared_parameter_uses_real_rom_preflight_with_a_synthetic_catalog(){
        let rom=std::fs::read(std::env::var_os("FFTA_PARAMETER_TEST_ROM").expect("protected ROM fixture")).unwrap();
        let work=std::path::PathBuf::from(std::env::var_os("FFTA_VALIDATION_WORK").expect("validation work"));
        let dir=tempfile::tempdir_in(work).unwrap();let path=dir.path().join("catalogo.json");
        let mut value=fixture();value["entries"][0]["codes"]=json!(["AE1934BX 70E6","524D3BD0 7FEE","473AF42B 20FC","07A5CB01 381C"]);
        value["entries"][0]["provenance"]=json!([{"source":"synthetic","master_id":"CH-0002","raw_code_lines":value["entries"][0]["codes"],
            "parameter":{"name":"amount","allowed":{"kind":"list","values":[15]},"slots":[{"line":0,"field":"word","offset":7,"digits":1}]}}]);
        value["entries"][1]["format"]=json!("codebreaker");value["entries"][1]["region"]=json!("EUR");
        value["entries"][1]["provenance"]=json!([{"source":"synthetic","raw_code_lines":["9ABCDEF0 1234","19A10BC4 AA65","26960CD0 140B"]}]);
        std::fs::write(&path,serde_json::to_vec(&value).unwrap()).unwrap();let source=std::fs::read(&path).unwrap();
        let sha=gba_core::ffta_rom::digest(&rom);let reference=crate::debug_inputs::Reference::explicit(&path,menu::ContentSlot::CheatCatalog,&sha).unwrap();
        let catalog=Catalog::load(&path).unwrap();let snapshot=Snapshot::new(catalog,reference.entry.id.clone(),sha).unwrap();let generation=snapshot.generation.clone();
        let invalid=crate::debug_cheat_parameters::Choice{name:"amount".into(),value:14};
        assert!(snapshot.prepare_once(&reference,&rom,"CH-0001",0,"CH-0002",&generation,&[]).is_err());
        assert!(snapshot.prepare_once(&reference,&rom,"CH-0001",0,"CH-0002",&generation,std::slice::from_ref(&invalid)).is_err());
        let choice=crate::debug_cheat_parameters::Choice{name:"amount".into(),value:15};
        let (plan,info)=snapshot.prepare_once(&reference,&rom,"CH-0001",0,"CH-0002",&generation,std::slice::from_ref(&choice)).unwrap();
        assert_eq!(info["parameter"]["choice"]["value"],15);assert_eq!(info["master"]["rom"]["checksum_state"],"match");
        let mut machine=gba_core::Machine::new(vec![]);assert_eq!(plan.apply_once(&mut machine.bus).unwrap(),3);
        assert_eq!(std::fs::read(&path).unwrap(),source);
        value["entries"][0]["codes"][0]=json!("AE1934BX 70EY");value["entries"][0]["provenance"][0]["raw_code_lines"]=value["entries"][0]["codes"].clone();
        let provenance=&mut value["entries"][0]["provenance"][0];provenance.as_object_mut().unwrap().remove("parameter");
        provenance["parameters"]=json!([{ "name":"first","allowed":{"kind":"list","values":[15]},"slots":[{"line":0,"field":"word","offset":7,"digits":1}] },{ "name":"second","allowed":{"kind":"range","min":6,"max":6,"step":1},"slots":[{"line":0,"field":"value","offset":3,"digits":1}] }]);
        std::fs::write(&path,serde_json::to_vec(&value).unwrap()).unwrap();assert!(snapshot.prepare_once(&reference,&rom,"CH-0001",0,"CH-0002",&generation,std::slice::from_ref(&choice)).is_err());
        let reference=crate::debug_inputs::Reference::explicit(&path,menu::ContentSlot::CheatCatalog,&gba_core::ffta_rom::digest(&rom)).unwrap();let snapshot=Snapshot::new(Catalog::load(&path).unwrap(),reference.entry.id.clone(),gba_core::ffta_rom::digest(&rom)).unwrap();
        let choices=[crate::debug_cheat_parameters::Choice{name:"first".into(),value:15},crate::debug_cheat_parameters::Choice{name:"second".into(),value:6}];
        assert!(snapshot.prepare_once(&reference,&rom,"CH-0001",0,"CH-0002",&snapshot.generation,&choices[..1]).is_err());
        let (multiple,info)=snapshot.prepare_once(&reference,&rom,"CH-0001",0,"CH-0002",&snapshot.generation,&choices).unwrap();assert_eq!(info["parameters"]["choices"].as_array().unwrap().len(),2);
        let mut other=gba_core::Machine::new(vec![]);assert_eq!(multiple.apply_once(&mut other.bus).unwrap(),3);
    }
    #[test]
    fn master_links_distinguish_source_records_without_selecting_dependencies(){
        let mut v=fixture();v["entries"][1]["format"]=json!("codebreaker");v["entries"][1]["region"]=json!("EUR");
        v["entries"][1]["provenance"]=json!([{"source":"one"},{"source":"one"}]);
        v["entries"][0]["provenance"]=json!([
            {"source":"one","master_id":"CH-0002"},{"source":"two","master_id":"CH-0002"},
            {"source":"one","master_id":"CH-0001"},{"source":"one","master_id":"CH-9999"},
            {"source":"one","master_id":null},{"source":"one"},{"source":"one","master_id":{}},
            {"master_id":"CH-0002"}]);
        let c=Catalog::parse(&serde_json::to_vec(&v).unwrap()).unwrap();let entry=c.entry("CH-0001").unwrap();let report=c.master_links(entry);
        let links=report["links"].as_array().unwrap();let states=links.iter().map(|l|l["state"].as_str().unwrap()).collect::<Vec<_>>();
        assert_eq!(states,["same-source-reference","source-mismatch","self-reference","unknown-id","null","missing","invalid-id","invalid-source"]);
        assert_eq!(links[0]["matching_master_source_records"],2);assert_eq!(links[0]["master_format_matches"],true);assert_eq!(links[0]["master_region_matches"],true);
        assert_eq!(links[1]["matching_master_source_records"],0);assert_eq!(links[2]["matching_master_source_records"],6);
        assert_eq!(c.entry("CH-0001").unwrap(),&v["entries"][0]);assert_eq!(Catalog::summary(entry,None)["enabled"],false);
        let dir=tempfile::tempdir().unwrap();let path=dir.path().join("synthetic.json");std::fs::write(&path,serde_json::to_vec(&v).unwrap()).unwrap();
        let args=vec!["show".into(),"CH-0001".into(),"--catalog".into(),path.to_str().unwrap().into()];
        assert_eq!(execute(&args,dir.path()).unwrap()["result"]["master_dependencies"],report);
    }
    #[test]
    fn exact_rom_context_distinguishes_declarations_from_verification(){
        let v=fixture();let catalog=Catalog::parse(&serde_json::to_vec(&v).unwrap()).unwrap();let entry=&catalog.entries[0];
        for (sha,state,region) in [(gba_core::ffta_rom::EUROPE_SHA256,"declared-match",Some("EUR")),(gba_core::ffta_rom::SHA256,"declared-mismatch",Some("USA")),("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","unknown",None)]{
            let summary=Catalog::summary(entry,Some(sha));assert_eq!(summary["compatibility"]["rom"]["state"],state);assert_eq!(summary["compatibility"]["rom"]["recognized_region"],json!(region));
            assert_eq!(summary["enabled"],false);assert_eq!(summary["executable"],false);assert_eq!(summary["compatibility"]["verification"]["state"],"unverified");assert_eq!(summary["compatibility"]["device_version"]["state"],"unknown");
            let view=catalog.menu_entries(Some(sha),input_config::Language::default());assert!(view[0].identity.contains(sha));assert!(view[0].reason.contains("not code verification"));assert!(view[0].reason.contains("capability is unresolved"));
        }
        let unknown=Catalog::summary(&catalog.entries[1],Some(gba_core::ffta_rom::EUROPE_SHA256));assert_eq!(unknown["compatibility"]["rom"]["state"],"unknown");
    }
    #[test]
    fn localized_cheat_rows_keep_imported_records_and_identity() {
        let input=fixture();let catalog=Catalog::parse(&serde_json::to_vec(&input).unwrap()).unwrap();
        let before=Catalog::summary(&catalog.entries[0],Some(gba_core::ffta_rom::EUROPE_SHA256));
        let en=catalog.menu_entries(Some(gba_core::ffta_rom::EUROPE_SHA256),input_config::Language::default());
        let it=catalog.menu_entries(Some(gba_core::ffta_rom::EUROPE_SHA256),input_config::Language::parse("it"));
        for (index,(english,italian)) in en.iter().zip(&it).enumerate() {
            assert_eq!(english.id,italian.id);assert_eq!(english.name,italian.name);assert_eq!(english.kind,italian.kind);
            assert_eq!(english.description,serde_json::to_string_pretty(&input["entries"][index]).unwrap());
            assert_eq!(english.description,italian.description);assert_eq!(english.available,italian.available);
            assert!(english.identity.contains(gba_core::ffta_rom::EUROPE_SHA256));
            assert!(italian.identity.contains(gba_core::ffta_rom::EUROPE_SHA256));
            if index==0 {assert!(english.reason.contains("capability is unresolved"));assert!(italian.reason.contains("capacità della fonte non è determinata"));}
            else {assert!(english.reason.contains("format is not supported"));assert!(italian.reason.contains("formato della fonte non è supportato"));}
        }
        assert_eq!(en[0].status,"Disabled");assert_eq!(it[0].status,"Disattivato");
        assert_eq!(Catalog::summary(&catalog.entries[0],Some(gba_core::ffta_rom::EUROPE_SHA256)),before);
    }
    #[test]
    fn bounded_snapshot_pages_and_utf8_details_preserve_identity(){
        let mut v=fixture();v["entries"][0]["notes"]=json!(["é".repeat(2000),"é".repeat(2000),"é".repeat(2000)]);
        let c=Catalog::parse(&serde_json::to_vec(&v).unwrap()).unwrap();let s=Snapshot::new(c,"content-id".into(),"a".repeat(64)).unwrap();
        let first=s.page(0,1,Some(&s.generation)).unwrap();assert_eq!(first["total"],2);assert_eq!(first["next_offset"],1);assert_eq!(first["entries"][0]["id"],"CH-0001");assert!(s.page(0,17,None).is_err());assert!(s.page(0,1,Some(&"0".repeat(64))).is_err());
        let mut offset=0u32;let mut joined=String::new();loop{let chunk=s.show("CH-0001",offset,Some(&s.generation)).unwrap();let text=chunk["json_part"].as_str().unwrap();assert!(text.len()<=4096);joined.push_str(text);match chunk["next_offset_bytes"].as_u64(){Some(next)=>offset=next as u32,None=>break,}}
        assert_eq!(serde_json::from_str::<Value>(&joined).unwrap(),v["entries"][0]);assert!(s.show("CH-9999",0,None).is_err());
        let pos=joined.find('é').unwrap();assert!(s.show("CH-0001",(pos+1) as u32,None).is_err());
    }
    #[test]
    fn capability_snapshot_chunks_preserve_report_generation_and_utf8(){
        let mut c=Catalog::parse(&serde_json::to_vec(&fixture()).unwrap()).unwrap();
        c.capabilities[0]["test_long_reason"]=json!("é".repeat(5000));
        let expected=c.capabilities[0].clone();let s=Snapshot::new(c,"content-id".into(),"a".repeat(64)).unwrap();
        let mut offset=0;let mut joined=String::new();let mut expected_hash=None;
        loop{let chunk=s.capabilities("CH-0001",offset,Some(&s.generation)).unwrap();let text=chunk["json_part"].as_str().unwrap();assert!(text.len()<=4096);joined.push_str(text);let hash=chunk["capability_sha256"].as_str().unwrap();if let Some(previous)=&expected_hash{assert_eq!(previous,hash)}else{expected_hash=Some(hash.to_owned());}match chunk["next_offset_bytes"].as_u64(){Some(next)=>offset=next as u32,None=>break,}}
        assert_eq!(serde_json::from_str::<Value>(&joined).unwrap(),expected);assert_eq!(expected_hash.unwrap(),gba_core::ffta_rom::digest(joined.as_bytes()));
        let invalid=joined.find('é').unwrap()+1;assert!(s.capabilities("CH-0001",invalid as u32,None).is_err());assert!(s.capabilities("CH-0001",0,Some(&"0".repeat(64))).is_err());assert!(s.capabilities("CH-9999",0,None).is_err());
    }
    #[test]
    fn cancelled_refresh_never_commits_a_snapshot(){
        let t=tempfile::tempdir().unwrap();let p=t.path().join("catalogo.json");std::fs::write(&p,serde_json::to_vec(&fixture()).unwrap()).unwrap();let rom="a".repeat(64);
        let reference=crate::debug_inputs::Reference::explicit(&p,menu::ContentSlot::CheatCatalog,&rom).unwrap();let mut job=Refresh::start(reference,rom).unwrap();job.cancel();
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(10);while !job.is_finished(){assert!(std::time::Instant::now()<deadline);std::thread::yield_now();}
        assert!(job.finish().err().unwrap().contains("control-cancelled"));
    }
    #[test]
    fn malformed_duplicate_schema_and_bounds_are_rejected_atomically(){
        let mut v=fixture();v["entries"][1]["id"]=json!("CH-0001");assert!(Catalog::parse(&serde_json::to_vec(&v).unwrap()).is_err());
        let mut v=fixture();v["entries"][0]["codes"]=json!([1]);assert!(Catalog::parse(&serde_json::to_vec(&v).unwrap()).is_err());
        let mut v=fixture();v["schema"]=json!(2);assert!(Catalog::parse(&serde_json::to_vec(&v).unwrap()).is_err());assert!(Catalog::parse(&vec![b' ';MAX_BYTES as usize+1]).is_err());
    }
    #[test]
    fn empty_explicit_import_paging_and_readonly_source(){
        let dir=tempfile::tempdir().unwrap();let root=dir.path();let args=|s:&[&str]|s.iter().map(|v|v.to_string()).collect::<Vec<_>>();
        assert_eq!(execute(&args(&["list"]),root).unwrap()["outcome"],"empty");let path=root.join("fixture.json");let original=serde_json::to_vec(&fixture()).unwrap();std::fs::write(&path,&original).unwrap();let p=path.to_str().unwrap();
        let list=execute(&args(&["list","--catalog",p,"--limit","1"]),root).unwrap();assert_eq!(list["total"],2);assert_eq!(list["result"]["entries"][0]["id"],"CH-0001");assert_eq!(list["result"]["next_offset"],1);
        let show=execute(&args(&["show","CH-0001","--catalog",p]),root).unwrap();assert_eq!(show["result"]["entry"],fixture()["entries"][0]);assert!(execute(&args(&["show","CH-9999","--catalog",p]),root).is_err());
        for bad in [["list","--limit","0"],["list","--offset","01"],["list","--rom-sha256","invalid"]]{assert!(execute(&args(&bad),root).is_err());}
        assert_eq!(std::fs::read(path).unwrap(),original);
    }
    #[test]
    #[ignore = "explicit readonly check against the user's local reference catalog; no payload is printed"]
    fn local_catalog_menu_mapping_keeps_every_original_entry(){
        let path=PathBuf::from(std::env::var_os("FFTA_CHEAT_REFERENCE").unwrap());let before=std::fs::read(&path).unwrap();let catalog=Catalog::load(&path).unwrap();
        let rows=catalog.menu_entries(None,input_config::Language::default());assert_eq!(rows.len(),catalog.entries.len());assert_eq!(rows.len(),341);
        for (row,entry) in rows.iter().zip(&catalog.entries){assert_eq!(row.id,entry["id"].as_str().unwrap());assert_eq!(row.description,serde_json::to_string_pretty(entry).unwrap());assert!(row.reason.contains("behavior unverified"));}
        assert_eq!(std::fs::read(path).unwrap(),before);
    }
}
