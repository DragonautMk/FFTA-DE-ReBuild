//! Exact, bounded application events for isolated input recordings.
//!
//! This is independent of conservative save-state ancestry. The private
//! catalog and decoded plans are authenticated again before replay.
use crate::debug_cheat_decoder::Plan;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

pub(crate) const MAX_BYTES: u64 = 16 * 1024 * 1024;
const MAX_PLANS: usize = 64;
const MAX_EVENTS: usize = 65_536;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Entry { pub identity: Value, pub plan: Value }

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Event {
    pub frame: u64,
    pub clock: u64,
    pub kind: String,
    pub entries: Vec<u8>,
    pub writes: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Timeline {
    pub schema: u8,
    pub kind: String,
    pub catalog_sha256: String,
    pub rom_sha256: String,
    pub first_frame: u64,
    pub entries: Vec<Entry>,
    pub events: Vec<Event>,
    #[serde(skip)]
    reserved_bytes: u64,
}

impl Timeline {
    fn checked_entry(&self,entry:&Entry)->Result<crate::debug_cheat_decoder::Footprint,String>{
        let plan=Plan::from_recorded(&entry.plan)?;
        let hash=gba_core::ffta_rom::digest(&serde_json::to_vec(&plan).map_err(|e|e.to_string())?);
        if entry.identity["plan_sha256"].as_str()!=Some(hash.as_str()) || entry.identity["decoder"]!="codebreaker-subset-v1" || entry.identity["source_sha256"]!=self.catalog_sha256 || entry.identity["rom_sha256"]!=self.rom_sha256 || serde_json::to_vec(&entry.identity).map_err(|e|e.to_string())?.len()>8192 {return Err("Cheat timeline identity does not match its plan or inputs".into());}
        plan.footprint()
    }
    pub fn parse(bytes:&[u8])->Result<Self,String>{
        if bytes.len() as u64>MAX_BYTES{return Err("Cheat timeline exceeds its size limit".into());}
        let timeline:Self=serde_json::from_slice(bytes).map_err(|e|format!("Invalid cheat timeline: {e}"))?;
        if serde_json::to_vec(&timeline).map_err(|e|e.to_string())? != bytes{return Err("Cheat timeline is not canonical".into());}
        Ok(timeline)
    }
    pub fn new(catalog_sha256:String, rom_sha256:String, first_frame:u64)->Self {
        Self { schema:1, kind:"ffta-cheat-application-timeline".into(),catalog_sha256,rom_sha256,first_frame,entries:Vec::new(),events:Vec::new(),reserved_bytes:512 }
    }
    /// Validate the entire proposed event before changing the retained timeline.
    pub fn reserve(&mut self, frame:u64, clock:u64, kind:&str, entries:Vec<Entry>)->Result<(),String>{
        if !matches!(kind,"once"|"maintained") || entries.is_empty() || entries.len()>8 || (kind=="once" && entries.len()!=1) || self.events.len()>=MAX_EVENTS || self.events.last().is_some_and(|e|e.writes.is_none()) {return Err("Invalid or full cheat timeline".into());}
        if frame<self.first_frame || self.events.last().is_some_and(|e|frame<e.frame || clock<e.clock || (frame==e.frame && e.kind=="maintained")) {return Err("Cheat application is outside timeline order".into());}
        let mut additions:Vec<Entry>=Vec::new();let mut indices=Vec::with_capacity(entries.len());
        let mut footprints:Vec<crate::debug_cheat_decoder::Footprint>=Vec::new();let mut max_writes=0usize;
        for entry in entries {
            let footprint=self.checked_entry(&entry)?;
            max_writes+=footprint.max_writes();
            if max_writes>256 || footprints.iter().any(|prior|prior.conflicts(&footprint)){return Err("Recorded cheat set exceeds limits or has conflicting footprints".into());}
            footprints.push(footprint);
            let index=match self.entries.iter().chain(additions.iter()).position(|old|old.identity==entry.identity && old.plan==entry.plan) {Some(i)=>i,None=>{if self.entries.len()+additions.len()>=MAX_PLANS{return Err("Too many distinct recorded cheat plans".into());}additions.push(entry);self.entries.len()+additions.len()-1}};
            if indices.contains(&(index as u8)){return Err("Duplicate cheat plan in application".into());}indices.push(index as u8);
        }
        let event=Event{frame,clock,kind:kind.into(),entries:indices,writes:None};
        let added=additions.iter().try_fold(0u64,|sum,entry|Ok::<_,String>(sum+serde_json::to_vec(entry).map_err(|e|e.to_string())?.len() as u64+1))?;
        let size=self.reserved_bytes+added+serde_json::to_vec(&event).map_err(|e|e.to_string())?.len() as u64+24;
        if size>MAX_BYTES{return Err("Cheat timeline exceeds its size limit".into());}
        self.entries.extend(additions);self.events.push(event);self.reserved_bytes=size;Ok(())
    }
    pub fn finish_event(&mut self,writes:usize)->Result<(),String>{
        if writes>256{return Err("Recorded application exceeds write limit".into());}
        let event=self.events.last_mut().ok_or("Missing cheat application")?;
        if event.writes.is_some(){return Err("Cheat application already committed".into());}event.writes=Some(writes);Ok(())
    }
    pub fn validate(&self, frames:u64)->Result<(),String>{
        if self.schema!=1 || self.kind!="ffta-cheat-application-timeline" || self.entries.is_empty() || self.entries.len()>MAX_PLANS || self.events.is_empty() || self.events.len()>MAX_EVENTS || serde_json::to_vec(self).map_err(|e|e.to_string())?.len() as u64>MAX_BYTES {return Err("Invalid cheat timeline bounds".into());}
        let mut footprints=Vec::with_capacity(self.entries.len());
        for (i,entry) in self.entries.iter().enumerate(){
            if self.entries[..i].iter().any(|prior|prior.identity==entry.identity && prior.plan==entry.plan){return Err("Duplicate recorded cheat variant".into());}
            footprints.push(self.checked_entry(entry)?);
        }
        let mut next_entry=0usize;let mut previous:Option<&Event>=None;
        for event in &self.events {
            if event.frame.checked_sub(self.first_frame).is_none_or(|n|n>frames || (n==frames && event.kind=="maintained")) {return Err("Cheat application frame is outside recording".into());}
            if !matches!(event.kind.as_str(),"once"|"maintained") || event.entries.is_empty() || event.entries.len()>8 || (event.kind=="once" && event.entries.len()!=1) || event.writes.is_none_or(|n|n>256) {return Err("Invalid recorded cheat application".into());}
            if previous.is_some_and(|p|event.frame<p.frame || event.clock<p.clock || (event.frame==p.frame && p.kind=="maintained")){return Err("Cheat application is outside timeline order".into());}
            let mut selected=Vec::with_capacity(event.entries.len());let mut writes=0usize;
            for index in &event.entries {
                let index=usize::from(*index);let footprint=footprints.get(index).ok_or("Invalid cheat plan index")?;
                if index>next_entry || selected.contains(&index){return Err("Noncanonical cheat plan order".into());}
                if index==next_entry{next_entry+=1;}
                writes+=footprint.max_writes();
                if writes>256 || selected.iter().any(|prior|footprints[*prior].conflicts(footprint)){return Err("Recorded cheat set exceeds limits or has conflicting footprints".into());}
                selected.push(index);
            }
            previous=Some(event);
        }
        if next_entry!=self.entries.len(){return Err("Unused recorded cheat variant".into());}Ok(())
    }
    /// Re-decode the pinned source and master against the exact ROM. No guest
    /// writes happen here, including when any declaration has changed.
    pub fn authenticate(&self,catalog_path:&Path,rom:&[u8],frames:u64)->Result<Vec<Plan>,String>{
        self.validate(frames)?;
        let rom_sha=gba_core::ffta_rom::digest(rom);
        if rom_sha!=self.rom_sha256{return Err("Cheat timeline ROM differs".into());}
        let reference=crate::debug_inputs::Reference::explicit(catalog_path,menu::ContentSlot::CheatCatalog,&rom_sha)?;
        let catalog=crate::debug_cheats::Catalog::load(catalog_path)?;
        if catalog.source_sha256!=self.catalog_sha256{return Err("Cheat timeline catalog differs".into());}
        let snapshot=crate::debug_cheats::Snapshot::new(catalog,reference.entry.id.clone(),rom_sha)?;
        let mut plans=Vec::with_capacity(self.entries.len());
        for entry in &self.entries {
            let id=entry.identity["id"].as_str().ok_or("Missing cheat identity")?;
            let master=entry.identity["master_id"].as_str().ok_or("Missing master identity")?;
            let index=entry.identity["provenance_index"].as_u64().and_then(|n|usize::try_from(n).ok()).ok_or("Invalid provenance index")?;
            let single=entry.identity.get("parameter");let multiple=entry.identity.get("parameters");
            if single.is_some() && multiple.is_some(){return Err("Ambiguous recorded parameters".into());}
            let choices:Vec<crate::debug_cheat_parameters::Choice>=if let Some(c)=single{vec![serde_json::from_value(c.clone()).map_err(|e|format!("Invalid parameter: {e}"))?]}else if let Some(c)=multiple{serde_json::from_value(c.clone()).map_err(|e|format!("Invalid parameters: {e}"))?}else{Vec::new()};
            let (plan,info)=snapshot.prepare_once(&reference,rom,id,index,master,&snapshot.generation,&choices)?;
            for field in ["source_sha256","rom_sha256","entry_lines_sha256","master_lines_sha256"] {if entry.identity[field]!=info[field]{return Err(format!("Recorded cheat {field} changed"));}}
            if serde_json::to_value(&plan).map_err(|e|e.to_string())?!=entry.plan {return Err("Recorded cheat plan differs from source".into());}
            plans.push(plan);
        }
        Ok(plans)
    }
    pub fn report(&self)->Value {
        let variants=self.entries.iter().take(8).map(|e|serde_json::json!({"id":e.identity["id"],"plan_sha256":e.identity["plan_sha256"],"provenance_index":e.identity["provenance_index"]})).collect::<Vec<_>>();
        serde_json::json!({"timeline_sha256":gba_core::ffta_rom::digest(&serde_json::to_vec(self).expect("timeline JSON")),"catalog_sha256":self.catalog_sha256,"rom_sha256":self.rom_sha256,"applications":self.events.len(),"variants":variants,"remaining_variants":self.entries.len().saturating_sub(8),"timeline":"cheats-timeline.json","scope":"Full variants, parameters and ordered applications are in the pinned private timeline; not hardware-hook verification"})
    }
}

pub(crate) struct Playback { timeline:Timeline, plans:Vec<Plan>, next:usize }
impl Playback {
    pub fn new(timeline:Timeline,plans:Vec<Plan>)->Self{Self{timeline,plans,next:0}}
    pub fn apply<B:gba_core::Bus>(&mut self,bus:&mut B,frame:u64,clock:u64,kind:&str)->Result<(),String>{
        while let Some(event)=self.timeline.events.get(self.next) {
            if event.frame<frame || (event.frame==frame && event.kind=="once" && kind=="maintained") {return Err("Missed recorded cheat application".into());}
            if event.frame!=frame || event.kind!=kind {break;}
            if event.clock!=clock{return Err("Recorded cheat application clock differs".into());}
            let mut writes=0;for i in &event.entries {writes+=self.plans.get(usize::from(*i)).ok_or("Missing recorded plan")?.apply_once(bus)?;}
            if Some(writes)!=event.writes{return Err("Recorded cheat application write count differs".into());}
            self.next+=1;
        }
        Ok(())
    }
    pub fn done(&self)->bool{self.next==self.timeline.events.len()}
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn entry()->(Entry,Plan) {
        let lines=["BB28F300 B8C0".to_owned()];
        let master=["9ABCDEF0 1234".to_owned(),"B131BCEF 0691".to_owned(),"26960CD0 140B".to_owned()];
        let (plan,_)=crate::debug_cheat_decoder::inspect_source(&lines,&master,None).unwrap();
        let identity=json!({"id":"CH-0001","source_sha256":"a".repeat(64),"rom_sha256":"b".repeat(64),"plan_sha256":gba_core::ffta_rom::digest(&serde_json::to_vec(&plan).unwrap()),"decoder":"codebreaker-subset-v1"});
        (Entry{identity,plan:serde_json::to_value(&plan).unwrap()},plan)
    }
    #[test]
    fn exact_timeline_rejects_incomplete_out_of_order_and_unsafe_plans(){
        let (entry,plan)=entry();let mut timeline=Timeline::new("a".repeat(64),"b".repeat(64),5);
        timeline.reserve(5,90,"once",vec![entry.clone()]).unwrap();assert!(timeline.validate(1).is_err());
        timeline.finish_event(1).unwrap();assert!(timeline.finish_event(0).is_err());assert_eq!(timeline.events[0].writes,Some(1));assert!(timeline.reserve(4,90,"once",vec![entry.clone()]).is_err());
        assert!(timeline.reserve(5,89,"once",vec![entry.clone()]).is_err());
        assert!(timeline.reserve(5,90,"maintained",vec![entry.clone(),entry.clone()]).is_err());
        let mut conflicting=entry.clone();conflicting.identity["id"]=json!("CH-0002");
        assert!(timeline.reserve(5,90,"maintained",vec![entry.clone(),conflicting]).is_err());
        let mut unsafe_entry=entry.clone();unsafe_entry.plan["operations"][0]["address"]=json!(0x04000000u32);
        unsafe_entry.identity["plan_sha256"]=json!(gba_core::ffta_rom::digest(&serde_json::to_vec(&unsafe_entry.plan).unwrap()));
        assert!(timeline.reserve(5,90,"once",vec![unsafe_entry]).is_err());
        timeline.reserve(5,90,"maintained",vec![entry.clone()]).unwrap();timeline.finish_event(1).unwrap();
        assert!(timeline.reserve(5,90,"once",vec![entry]).is_err());assert!(timeline.validate(0).is_err());timeline.validate(1).unwrap();
        let encoded=serde_json::to_vec(&timeline).unwrap();Timeline::parse(&encoded).unwrap();
        let mut noncanonical=encoded;noncanonical.push(b' ');assert!(Timeline::parse(&noncanonical).is_err());
        let mut damaged=timeline.clone();damaged.events[0].entries[0]=9;assert!(damaged.validate(1).is_err());
        let mut machine=gba_core::Machine::new(vec![]);let before=machine.save_state().unwrap();
        let mut playback=Playback::new(timeline,vec![plan]);assert!(playback.apply(&mut machine.bus,5,91,"once").is_err());assert_eq!(machine.save_state().unwrap(),before);
    }
    #[test]
    fn maintained_sequence_is_bounded_and_reuses_one_source_variant(){
        let (entry,_)=entry();let mut timeline=Timeline::new("a".repeat(64),"b".repeat(64),0);
        for frame in 0..512 {timeline.reserve(frame,frame*10,"maintained",vec![entry.clone()]).unwrap();timeline.finish_event(1).unwrap();}
        assert_eq!(timeline.entries.len(),1);timeline.validate(512).unwrap();
        let bytes=serde_json::to_vec(&timeline).unwrap();let copy=Timeline::parse(&bytes).unwrap();copy.validate(512).unwrap();
        let mut damaged=copy;damaged.events[10].frame=0;assert!(damaged.validate(512).is_err());
    }
}
