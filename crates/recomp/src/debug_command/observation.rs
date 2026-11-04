//! Bounded presentation of already-owned frontend state. Never dispatches jobs.
use serde_json::Value;
use input_config::Language;
use menu::i18n::format as tr;

fn clean(value:&str,limit:usize)->String {
    let mut chars=value.chars().filter(|c|!c.is_control());
    let mut text:String=chars.by_ref().take(limit).collect();
    if chars.next().is_some(){text.push_str("...");}text
}

pub(crate) fn target(manifest:&Value,run:&str,language:Language)->String {
    let label=clean(manifest["label"].as_str().unwrap_or("?"),48);
    let hash=manifest["pins"]["ffta.gba"].as_str().filter(|h|h.len()==64&&h.bytes().all(|c|c.is_ascii_hexdigit())).unwrap_or("");
    let rom=if hash.is_empty(){"?".into()}else{hash.to_owned()};
    tr(language,"Session {label} | ROM {rom} | run {run}",&[("label",label),("rom",rom),("run",clean(run,100))])
}
pub(crate) fn verified_target(bytes:&[u8],run:&str,expected:&str,language:Language)->Result<String,String>{
    if gba_core::ffta_rom::digest(bytes)!=expected{return Err("Session manifest changed; reopen the debug copy".into());}
    let manifest:Value=serde_json::from_slice(bytes).map_err(|e|e.to_string())?;Ok(target(&manifest,run,language))
}

pub(crate) fn pending(value:&Value,remaining:u32,search_remaining:Option<u64>,language:Language)->(String,String) {
    if value.is_null(){return (language.text("No pending job").into(),language.text("Parameters: -").into());}
    let id=clean(value["request_id"].as_str().unwrap_or("?"),100);
    let command=&value["command"];
    let progress=match command["action"].as_str(){
        Some("step")=>tr(language,"frames remaining {remaining}/{total}",&[("remaining",remaining.to_string()),("total",command["frames"].as_u64().unwrap_or(0).to_string())]),
        Some("find")=>match search_remaining{Some(left)=>tr(language,"instructions remaining {remaining}/{total}",&[("remaining",left.to_string()),("total",command["settings"]["steps"].as_u64().unwrap_or(0).to_string())]),None=>language.text("search in progress").into()},
        Some("content-refresh")=>language.text("scan in progress").into(),
        Some("cheat-refresh")=>language.text("checking cheat catalog").into(),
        Some("select-id")=>language.text("selection in progress").into(),
        _=>language.text("in progress").into(),
    };
    let mut parameters=match command["action"].as_str(){
        Some("step")=>format!("step {} | keys 0x{:03x}",command["frames"],command["keys"].as_u64().unwrap_or(0)),
        Some("find") if command["settings"]["condition"]["kind"]=="pc"=>{
            let condition=&command["settings"]["condition"];
            format!("break 0x{:08x} {} | steps {}",condition["address"].as_u64().unwrap_or(0),if condition["thumb"]==true{"thumb"}else{"arm"},command["settings"]["steps"])
        },
        Some("find") if command["settings"]["condition"]["kind"]=="access"=>{
            let watch=&command["settings"]["condition"]["watch"];
            format!("watch {} 0x{:08x}..0x{:08x} | width {} | value {} | steps {}",if watch["write"]==true{"write"}else{"read"},watch["start"].as_u64().unwrap_or(0),watch["end"].as_u64().unwrap_or(0),watch["width"],watch["value"],command["settings"]["steps"])
        },
        Some("content-refresh")=>"content refresh".into(),
        Some("cheat-refresh")=>"cheats refresh".into(),
        Some("select-id")=>format!("select {} {}",command["slot"].as_str().unwrap_or("?"),clean(command["id"].as_str().unwrap_or("?"),512)),
        _=>serde_json::to_string(command).unwrap_or_else(|_|"?".into()),
    };
    if command["action"]=="find" {
        let settings=&command["settings"];
        if settings["window"].is_object(){let w=&settings["window"];parameters.push_str(&format!(" | hits {} | frames [{}..{})",w["hit_count"],w["start_frame"],w["end_frame"]));}
        if settings["guard"].is_object(){let g=&settings["guard"];parameters.push_str(&format!(" | RAM 0x{:08x} width {} = 0x{:x}",g["address"].as_u64().unwrap_or(0),g["width"],g["value"].as_u64().unwrap_or(0)));}
        if let Some(action)=settings["on_hit"].as_str(){parameters.push_str(&format!(" | on hit {}",clean(action,32)));}
        if settings["trigger"].is_object(){let t=&settings["trigger"];parameters.push_str(&format!(" | trigger {} | definition {}",clean(t["name"].as_str().unwrap_or("?"),48),clean(t["sha256"].as_str().unwrap_or("?"),64)));}
    }
    (format!("ID {id} | {progress}"),tr(language,"Parameters: {parameters}",&[("parameters",clean(&parameters,4096))]))
}

/// Summarize trigger results in the console while retaining the full JSON for copy.
pub(crate) fn result_lines(value: &Value) -> Vec<String> {
    let kind=value["kind"].as_str().unwrap_or("");
    if kind!="ffta-debug-trigger" && kind!="ffta-debug-trigger-result" {return Vec::new();}
    let mut lines=Vec::new();
    let definition=&value["definition"];
    if let Some(items)=definition.as_array() {
        for item in items.iter().take(32) {lines.push(format!("{} · {}",clean(item["name"].as_str().unwrap_or("?"),48),if item["enabled"]==true{"enabled"}else{"disabled"}));}
        if items.is_empty(){lines.push("No saved triggers.".into());}
    } else if definition.is_object() {
        lines.push(format!("Trigger {} · {}",clean(definition["name"].as_str().unwrap_or("?"),48),if definition["enabled"]==true{"enabled"}else{"disabled"}));
        let settings=&definition["action"]["settings"];
        let condition=&settings["condition"];
        let description=match condition["kind"].as_str() {
            Some("pc")=>format!("PC 0x{:08x} {}",condition["address"].as_u64().unwrap_or(0),if condition["thumb"]==true{"thumb"}else{"arm"}),
            Some("access")=>{let watch=&condition["watch"];format!("{} 0x{:08x}..0x{:08x} · width {} · value {}",if watch["write"]==true{"write"}else{"read"},watch["start"].as_u64().unwrap_or(0),watch["end"].as_u64().unwrap_or(0),watch["width"],watch["value"])},
            _=>clean(&condition.to_string(),220),
        };
        lines.push(format!("Condition: {description}"));
        if !settings["window"].is_null(){lines.push(format!("Window: {}",clean(&settings["window"].to_string(),180)));}
        if !settings["guard"].is_null(){lines.push(format!("RAM guard: {}",clean(&settings["guard"].to_string(),180)));}
        lines.push(format!("Step limit: {} · on hit: {}",settings["steps"],settings["on_hit"].as_str().unwrap_or("pause")));
    }
    let probe=&value["response"]["status"]["probe"];
    if probe.is_object() {
        lines.push(format!("{} · {} · steps {} · frames {}",probe["outcome"].as_str().unwrap_or("unknown"),probe["boundary"].as_str().unwrap_or("unknown boundary"),probe["steps"],probe["completed_frames"]));
        if probe["evaluation"].is_object(){let e=&probe["evaluation"];let guard=if e["last_guard_value"].is_null(){String::new()}else{format!(" · last guard {}",e["last_guard_value"])};lines.push(format!("Hits {} · candidates {}{guard} · {}",e["observed_hits"],e["candidate_count"],e["stop_reason"].as_str().unwrap_or("")));}
        if probe["post_action"].is_object(){lines.push(format!("Action {} · {}",probe["post_action"]["kind"].as_str().unwrap_or("?"),probe["post_action"]["outcome"].as_str().unwrap_or("?")));}
    }
    lines.push("Interpreter only. PC: before instruction; watch: after complete step.".into());
    lines.push("Paused idle required; watch needs audit feature and an exclusive observer.".into());
    lines
}

#[cfg(test)]
mod tests {
    use super::*;use serde_json::json;
    #[test]
    fn trigger_results_show_conditions_boundaries_and_conflicts() {
        let value=json!({"kind":"ffta-debug-trigger-result","definition":{"name":"alpha","enabled":true,"action":{"settings":{"condition":{"kind":"pc","address":134217728,"thumb":false},"steps":100,"window":{"hit_count":2},"on_hit":"freeze"}}},"response":{"status":{"probe":{"outcome":"hit","boundary":"before-interpreter-step","steps":4,"completed_frames":0,"evaluation":{"observed_hits":2,"candidate_count":3,"last_guard_value":5,"stop_reason":"target-hit-count"},"post_action":{"kind":"freeze","outcome":"completed"}}}}});
        let lines=result_lines(&value).join("\n");
        for expected in ["alpha", "enabled", "Condition: PC 0x08000000 arm", "Window:", "freeze", "before-interpreter-step", "Hits 2", "last guard 5", "exclusive observer", "Interpreter only"] {assert!(lines.contains(expected),"{expected}: {lines}");}
        assert!(result_lines(&json!({"kind":"other"})).is_empty());
    }
    #[test]
    fn pending_search_includes_all_extended_settings() {
        let search=json!({"request_id":"id-3","command":{"action":"find","settings":{"steps":5000000,"condition":{"kind":"pc","address":134217728,"thumb":true},"window":{"hit_count":42,"start_frame":3,"end_frame":99},"guard":{"address":33554432,"width":2,"value":4660},"on_hit":"freeze","trigger":{"name":"alpha","sha256":"a".repeat(64)}}}});
        let (progress,params)=pending(&search,0,Some(4500000),Language::default());
        for part in ["id-3", "4500000/5000000"]{assert!(progress.contains(part),"{progress}");}
        for part in ["0x08000000 thumb", "hits 42", "frames [3..99)", "RAM 0x02000000 width 2 = 0x1234", "on hit freeze", "trigger alpha", &"a".repeat(64)]{assert!(params.contains(part),"{params}");}
    }
    #[test]
    fn target_and_pending_progress_are_explicit_and_bounded(){
        let manifest=json!({"label":"Prova","pins":{"ffta.gba":"a".repeat(64)}});
        let en=Language::default();let it=Language::parse("it");
        let target=target(&manifest,"run-42",en);assert!(target.starts_with("Session ")&&target.contains("Prova")&&target.contains(&"a".repeat(64))&&target.contains("run-42"));
        assert!(super::target(&manifest,"run-42",it).starts_with("Sessione "));
        let braces=json!({"label":"{rom}","pins":{"ffta.gba":"a".repeat(64)}});
        assert!(super::target(&braces,"run-42",en).contains(&format!("Session {{rom}} | ROM {}", "a".repeat(64))));
        let long_label="A long session label with Unicode à and a clear ending";
        let long_target=super::target(&json!({"label":long_label,"pins":{"ffta.gba":"a".repeat(64)}}),"run-42",en);
        assert!(long_target.contains(&long_label.chars().take(48).collect::<String>()));
        let bytes=serde_json::to_vec(&manifest).unwrap();let hash=gba_core::ffta_rom::digest(&bytes);
        assert!(verified_target(&bytes,"run-42",&hash,en).unwrap().contains("Prova"));assert!(verified_target(&bytes,"run-42",&"0".repeat(64),en).is_err());
        let step=json!({"request_id":"id-1","command":{"action":"step","frames":10,"keys":1023}});
        let (progress,params)=pending(&step,3,None,en);assert!(progress.contains("frames remaining 3/10")&&progress.contains("id-1"));assert!(params.contains("0x3ff"));
        assert!(pending(&step,3,None,it).0.contains("frame rimasti 3/10"));
        let search=json!({"request_id":"id-2","command":{"action":"find","settings":{"steps":100,"condition":{"kind":"pc","address":134217728,"thumb":false}}}});
        let (progress,params)=pending(&search,0,Some(75),en);assert!(progress.contains("id-2")&&progress.contains("75/100"));assert!(params.contains("0x08000000 arm"));
        assert_eq!(pending(&Value::Null,0,None,en).0,"No pending job");assert_eq!(pending(&Value::Null,0,None,it).0,"Nessun job pendente");
        let long=json!({"request_id":"x".repeat(100),"command":{"action":"select-id","id":"y".repeat(512)}});
        let (progress,params)=pending(&long,0,None,en);assert!(progress.contains(&"x".repeat(100))&&params.contains(&"y".repeat(512)));
        let excessive=json!({"request_id":"x".repeat(500),"command":{"action":"select-id","id":"y".repeat(5000)}});
        let (progress,params)=pending(&excessive,0,None,en);assert!(progress.len()<160&&params.len()<600);
    }
}
