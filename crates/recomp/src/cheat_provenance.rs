//! Conservative cheat ancestry and separately bounded observed application history.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const MAX_EVENTS: usize = 64;
const MAX_EXACT_EVENTS: usize = 4096;
const MAX_EXACT_VARIANTS: usize = 64;
const MAX_EXACT_BYTES: usize = 128 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExactEvent {
    kind: String,
    frame: u64,
    clock: u64,
    variant: u8,
    identity_sha256: String,
    writes: usize,
}

/// An observed suffix on one restored machine branch, not proof about its
/// earlier ancestry or surviving cartridge data. Never stores cheat code.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExactHistory {
    schema: u8,
    origin: String,
    variants: Vec<Value>,
    applications: Vec<ExactEvent>,
    truncated: bool,
    #[serde(skip)]
    reserved_bytes: usize,
}

impl ExactHistory {
    fn new()->Self {Self{schema:1,origin:"unknown-before-observed-suffix".into(),variants:Vec::new(),applications:Vec::new(),truncated:false,reserved_bytes:256}}
    fn record(&mut self,kind:&str,identity:&Value,frame:u64,clock:u64,writes:usize){
        if self.truncated{return;}
        if self.reserved_bytes==0 {self.reserved_bytes=serde_json::to_vec(self).map_or(MAX_EXACT_BYTES+1,|bytes|bytes.len()+512);}
        let valid_identity=match kind {"once"=>identity.is_object(),"maintained"=>identity.as_array().is_some_and(|items|!items.is_empty() && items.len()<=8 && items.iter().all(Value::is_object)),_=>false};
        let encoded=serde_json::to_vec(identity).unwrap_or_default();
        if !valid_identity || encoded.len()>8192 || writes>256 || self.applications.len()>=MAX_EXACT_EVENTS || self.applications.last().is_some_and(|last|frame<last.frame || clock<last.clock){self.truncated=true;return;}
        let (index,is_new)=match self.variants.iter().position(|value|value==identity){Some(index)=>(index,false),None=>(self.variants.len(),true)};
        if index>=MAX_EXACT_VARIANTS {self.truncated=true;return;}
        let event=ExactEvent{kind:kind.into(),frame,clock,variant:index as u8,identity_sha256:gba_core::ffta_rom::digest(&encoded),writes};
        let size=self.reserved_bytes+serde_json::to_vec(&event).map_or(MAX_EXACT_BYTES,|bytes|bytes.len()+1)+if is_new{encoded.len()+1}else{0};
        if size>MAX_EXACT_BYTES{self.truncated=true;return;}
        if is_new{self.variants.push(identity.clone());}self.applications.push(event);self.reserved_bytes=size;
    }
    fn validate(&self)->Result<(),String>{
        if self.schema!=1 || self.origin!="unknown-before-observed-suffix" || self.variants.len()>MAX_EXACT_VARIANTS || self.applications.len()>MAX_EXACT_EVENTS || serde_json::to_vec(self).map_err(|e|e.to_string())?.len()>MAX_EXACT_BYTES {return Err("Invalid exact cheat history bounds".into());}
        let variant_bytes=self.variants.iter().map(|identity|serde_json::to_vec(identity).map_err(|e|e.to_string())).collect::<Result<Vec<_>,_>>()?;
        if variant_bytes.iter().any(|bytes|bytes.len()>8192){return Err("Invalid exact cheat variant size".into());}
        let hashes=variant_bytes.iter().map(|bytes|gba_core::ffta_rom::digest(bytes)).collect::<Vec<_>>();
        let mut previous:Option<&ExactEvent>=None;let mut next=0usize;
        for event in &self.applications {
            let index=usize::from(event.variant);let identity=self.variants.get(index).ok_or("Invalid exact cheat variant")?;
            let valid=match event.kind.as_str(){"once"=>identity.is_object(),"maintained"=>identity.as_array().is_some_and(|items|!items.is_empty() && items.len()<=8 && items.iter().all(Value::is_object)),_=>false};
            if !valid || event.writes>256 || index>next || hashes[index]!=event.identity_sha256 || previous.is_some_and(|prior|event.frame<prior.frame || event.clock<prior.clock) {return Err("Invalid exact cheat application".into());}
            if index==next{next+=1;}previous=Some(event);
        }
        if next!=self.variants.len() || (self.applications.is_empty() && !self.truncated){return Err("Incomplete exact cheat history".into());}Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct History {
    pub(crate) schema: u8,
    pub(crate) events: Vec<Value>,
    pub(crate) truncated: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) exact: Option<ExactHistory>,
}

impl Default for History {
    fn default() -> Self { Self { schema: 1, events: Vec::new(), truncated: false, exact:None } }
}

impl History {
    pub(crate) fn is_empty(&self) -> bool { self.events.is_empty() && !self.truncated }

    fn exact_summary(&self)->Option<Value> {
        self.exact.as_ref().map(|history|{
            let bytes=serde_json::to_vec(history).expect("Exact cheat history JSON");
            json!({"origin":history.origin,"applications":history.applications.len(),"variants":history.variants.len(),"truncated":history.truncated,"sha256":gba_core::ffta_rom::digest(&bytes),"scope":"Bounded summary; full observed suffix is retained in SL3 snapshots"})
        })
    }

    /// Small report metadata for a restored starting state. The full private
    /// history is pinned separately and is never treated as a replay action.
    pub(crate) fn recording_summary(&self)->Value {
        fn collect(value:&Value,ids:&mut Vec<String>){
            if ids.len()>=8{return;}
            if let Some(items)=value.as_array(){for item in items {collect(item,ids);}return;}
            if let Some(id)=value.get("id").and_then(Value::as_str){let id:String=id.chars().take(64).collect();if !ids.contains(&id){ids.push(id);}return;}
            if let Some(inner)=value.get("identity"){collect(inner,ids);}
        }
        let mut ids=Vec::new();
        if let Some(exact)=&self.exact {for variant in &exact.variants {collect(variant,&mut ids);}}
        for event in &self.events {collect(&event["identity"],&mut ids);}
        json!({"observed_variants":ids,"conservative_events":self.events.len(),"conservative_truncated":self.truncated,"exact_suffix":self.exact_summary(),"scope":"Ancestry of the restored starting state; prior applications are not repeated"})
    }

    /// Keep the existing conservative status shape bounded independently of
    /// the full exact suffix stored in a private snapshot.
    pub(crate) fn status(&self)->Value {
        json!({"history":{"schema":1,"events":self.events,"truncated":self.truncated},"exact_suffix":self.exact_summary(),"scope":"Conservative observed ancestry; not an exact replay timeline"})
    }

    /// Retain only identities/hashes, never raw third-party codes or plans.
    pub(crate) fn record(&mut self, kind: &str, identity: Value, frame: u64, clock: u64, writes: usize) {
        if matches!(kind,"once"|"maintained") {
            self.exact.get_or_insert_with(ExactHistory::new).record(kind,&identity,frame,clock,writes);
            self.schema=2;
        }
        if let Some(last) = self.events.last_mut() {
            if last["kind"] == kind && last["identity"] == identity
                && last["last_frame"].as_u64().and_then(|v| v.checked_add(1)) == Some(frame) {
                last["last_frame"] = json!(frame);
                last["last_clock"] = json!(clock);
                last["applications"] = json!(last["applications"].as_u64().unwrap_or(0).saturating_add(1));
                last["writes"] = json!(last["writes"].as_u64().unwrap_or(0).saturating_add(writes as u64));
                return;
            }
        }
        if self.events.len() >= MAX_EVENTS { self.truncated = true; return; }
        self.events.push(json!({"kind":kind,"identity":identity,"first_frame":frame,"first_clock":clock,
            "last_frame":frame,"last_clock":clock,"applications":1,"writes":writes}));
    }

    /// Union conservatively: a clean restore cannot prove cartridge data is clean.
    pub(crate) fn include(&mut self, other: &Self) {
        for event in &other.events {
            if self.events.contains(event) { continue; }
            if self.events.len() >= MAX_EVENTS { self.truncated = true; break; }
            self.events.push(event.clone());
        }
        self.truncated |= other.truncated;
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        if !matches!((self.schema,self.exact.is_some()),(1,false)|(2,true)) || self.events.len() > MAX_EVENTS || (self.events.is_empty() && !self.truncated) {
            return Err("Invalid cheat ancestry".into());
        }
        if let Some(exact)=&self.exact {exact.validate()?;}
        Ok(())
    }
    pub(crate) fn restore_exact(&mut self,exact:Option<ExactHistory>){self.exact=exact;self.schema=if self.exact.is_some(){2}else{1};}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ancestry_is_bounded_and_survives_conservative_merges() {
        let mut source = History::default();
        for frame in 0..70 { source.record("once", json!({"id":frame}), frame, frame * 2, 1); }
        assert_eq!(source.events.len(), 64);
        assert!(source.truncated);
        let mut loaded = History::default();
        loaded.include(&source);
        loaded.include(&History::default());
        assert!(!loaded.is_empty());
        assert!(loaded.truncated);
        assert_eq!(loaded.events, source.events);
    }

    #[test]
    fn maintained_intervals_keep_order_without_raw_cheat_lines() {
        let mut history = History::default();
        for frame in 10..13 { history.record("maintained", json!([{"id":"synthetic"}]), frame, frame * 100, 2); }
        assert_eq!(history.events.len(), 1);
        assert_eq!(history.events[0]["first_frame"], 10);
        assert_eq!(history.events[0]["last_frame"], 12);
        assert_eq!(history.events[0]["applications"], 3);
        assert_eq!(history.events[0]["writes"], 6);
        history.record("once", json!({"id":"another"}), 13, 1300, 1);
        assert_eq!(history.events.len(), 2);
    }

    #[test]
    fn exact_suffix_keeps_every_application_and_reports_unknown_prefix_and_truncation() {
        let mut history=History::default();
        let identity=json!({"id":"CH-0001","plan_sha256":"a".repeat(64),"parameter":{"name":"amount","value":3}});
        history.record("once",identity.clone(),10,100,1);
        for frame in 11..14 {history.record("maintained",json!([{"id":"CH-0001","identity":identity}]),frame,frame*10,2);}
        history.record("armed-at-snapshot",json!([{"id":"CH-0001"}]),14,140,0);
        let exact=history.exact.as_ref().unwrap();assert_eq!(history.schema,2);assert_eq!(exact.applications.len(),4);assert_eq!(exact.variants.len(),2);
        assert_eq!(exact.origin,"unknown-before-observed-suffix");assert_eq!(exact.applications[2].frame,12);assert_eq!(exact.applications[2].clock,120);assert_eq!(exact.applications[2].writes,2);
        assert_eq!(exact.variants[0]["parameter"]["value"],3);history.validate().unwrap();
        let mut altered=history.clone();altered.exact.as_mut().unwrap().variants[0]["id"]=json!("CH-9999");assert!(altered.validate().is_err());
        let saved=serde_json::to_vec(&history).unwrap();let mut restored:History=serde_json::from_slice(&saved).unwrap();restored.validate().unwrap();
        restored.record("once",identity,15,150,1);restored.validate().unwrap();
        restored.record("once",json!({"id":"out-of-order"}),1,1,1);assert!(restored.exact.as_ref().unwrap().truncated);restored.validate().unwrap();
        let prior_count=restored.exact.as_ref().unwrap().applications.len();restored.record("once",json!({"id":"later"}),16,160,1);assert_eq!(restored.exact.as_ref().unwrap().applications.len(),prior_count);
    }

    #[test]
    fn exact_suffix_is_bounded_and_legacy_ancestry_stays_conservative() {
        let legacy=json!({"schema":1,"events":[{"kind":"old"}],"truncated":false});
        let mut history:History=serde_json::from_value(legacy.clone()).unwrap();history.validate().unwrap();assert!(history.exact.is_none());assert_eq!(serde_json::to_value(&history).unwrap(),legacy);
        for frame in 0..5000 {history.record("once",json!({"id":"same"}),frame,frame*10,1);}
        let exact=history.exact.as_ref().unwrap();assert!(exact.truncated);assert!(exact.applications.len()<=MAX_EXACT_EVENTS);assert!(serde_json::to_vec(exact).unwrap().len()<=MAX_EXACT_BYTES);history.validate().unwrap();
        let status=history.status();assert_eq!(status["history"]["schema"],1);assert!(status["history"].get("exact").is_none());assert_eq!(status["exact_suffix"]["truncated"],true);assert!(serde_json::to_vec(&status).unwrap().len()<4096);
        history.restore_exact(None);assert_eq!(history.schema,1);assert!(history.exact.is_none());assert!(!history.is_empty());
    }
}
