//! Bounded semantic diffs; each changed property is its own capped event.
use serde_json::{json,Value};
use std::collections::{BTreeMap,BTreeSet};

#[derive(Default)]
pub(crate) struct Selections { previous: Option<BTreeMap<String,Value>> }
fn bounded(value:&str)->Value {
    let count=value.chars().count();
    json!({"text":value.chars().take(256).collect::<String>(),"chars":count,"truncated":count>256,
        "sha256":(count>256).then(||gba_core::ffta_rom::digest(value.as_bytes()))})
}
impl Selections {
    pub fn reset(&mut self){self.previous=None;}
    pub fn changes(&mut self,properties:Vec<(String,String)>,visible:bool,cause:&str)->Vec<(&'static str,Value)>{
        let mut properties:BTreeMap<_,_>=properties.into_iter().collect();
        properties.insert("visible".into(),visible.to_string());
        // Current callers have <64 properties. Report an explicit gap if a
        // future form exceeds this budget; never silently claim full coverage.
        let omitted=properties.len().saturating_sub(96);
        let mut now:BTreeMap<_,_>=properties.iter().take(96).map(|(k,v)|(k.clone(),bounded(v))).collect();
        if omitted>0{now.insert("omitted_properties".into(),bounded(&omitted.to_string()));}
        let initial=self.previous.is_none();
        let old=self.previous.take().unwrap_or_default();
        let keys:BTreeSet<_>=old.keys().chain(now.keys()).cloned().collect();
        let rows=keys.into_iter().filter_map(|key|{
            let before=old.get(&key);let after=now.get(&key);
            (before!=after).then(||(if initial{"host-menu-baseline"}else{"host-menu-change"},json!({"cause":cause,"property":key,"previous":before,"current":after,
                "previous_item":{"id":old.get("item_id"),"label":old.get("label")},"current_item":{"id":now.get("item_id"),"label":now.get("label")},
                "initial":initial,"scope":"Effective semantic property at an existing frontend input/handler boundary; metadata only"})))
        }).collect();
        self.previous=Some(now);rows
    }
}

#[cfg(test)] mod tests {
    use super::*;
    #[test]
    fn changed_properties_are_exact_bounded_and_long_equal_prefixes_stay_distinct(){
        let mut tracker=Selections::default();
        let state=|suffix:&str|vec![("item_id".into(),format!("{}{}","x".repeat(5000),suffix)),("label".into(),"same label".into())];
        assert!(tracker.changes(state("a"),true,"start").iter().all(|(stage,_)|*stage=="host-menu-baseline"));
        assert!(tracker.changes(state("a"),true,"hover").is_empty());
        let delta=tracker.changes(state("b"),true,"hover");assert_eq!(delta.len(),1);assert_eq!(delta[0].0,"host-menu-change");
        assert_eq!(delta[0].1["property"],"item_id");assert_eq!(delta[0].1["current"]["truncated"],true);
        assert_ne!(delta[0].1["previous"]["sha256"],delta[0].1["current"]["sha256"]);
        assert!(serde_json::to_vec(&delta[0].1).unwrap().len()<4096);
        assert_eq!(tracker.changes(state("b"),false,"close")[0].1["property"],"visible");
        tracker.reset();assert!(tracker.changes(state("b"),false,"restart").iter().all(|(stage,_)|*stage=="host-menu-baseline"));
    }
}
