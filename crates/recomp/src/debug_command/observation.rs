//! Bounded presentation of already-owned frontend state. Never dispatches jobs.
use serde_json::Value;
use input_config::Language;
use menu::i18n::format as tr;

fn clean(value:&str,limit:usize)->String {
    let mut chars=value.chars().filter(|c|!c.is_control());
    let mut text:String=chars.by_ref().take(limit).collect();
    if chars.next().is_some(){text.push_str("...");}text
}

/// Live pinned rows are a bounded projection; full sources remain in status.
pub(crate) fn pin_lines(status:&Value)->Vec<menu::command_console::PinLine>{
    use menu::command_console::{PinLine,PinState};
    let Some(items)=status["pins"].as_array() else{return Vec::new()};
    if items.is_empty(){return Vec::new();}
    let filter=status["filter"].as_str().unwrap_or("all");
    let selected:Vec<_>=items.iter().filter(|p|match filter{"changed"=>p["changed"]==true,"stale"=>p["stale"]==true,_=>true}).collect();
    let offset=status["offset"].as_u64().unwrap_or(0) as usize;
    let mut out=vec![PinLine{text:format!("Pins: {filter} · {} matches · offset {offset}",selected.len()),state:PinState::Normal}];
    for pin in selected.iter().skip(offset).take(3){
        let state=if pin["stale"]==true{PinState::Stale}else if pin["changed"]==true{PinState::Changed}else{PinState::Normal};
        let mark=match state{PinState::Normal=>"=",PinState::Changed=>"CHANGED",PinState::Stale=>"STALE"};
        let label=clean(pin["label"].as_str().unwrap_or("?"),48);let owner=clean(pin["owner"].as_str().unwrap_or("?"),48);
        let previous=clean(&pin["previous"].to_string(),72);let current=clean(&pin["value"].to_string(),72);
        let source=&pin["source"];let origin=match source["kind"].as_str(){
            Some("register")=>format!("r{}",source["register"]),
            Some("readonly-backing")=>format!("0x{:08X}",source["address"].as_u64().unwrap_or(0)),
            Some("decoded-context-field")=>format!("{}/{} @0x{:08X}",source["family"].as_str().unwrap_or("?"),source["field"].as_str().unwrap_or("?"),source["context"].as_u64().unwrap_or(0)),
            _=>"unknown".into(),
        };
        out.push(PinLine{text:format!("{mark} {label} [{owner}] {origin}: {previous} → {current}"),state});
    }
    out
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

pub(crate) fn pending(value:&Value,remaining:u32,search_remaining:Option<u64>,replay_progress:Option<(u64,u64)>,language:Language)->(String,String) {
    if value.is_null(){return (language.text("No pending job").into(),language.text("Parameters: -").into());}
    let id=clean(value["request_id"].as_str().unwrap_or("?"),100);
    let command=&value["command"];
    let progress=match command["action"].as_str(){
        Some("step")=>tr(language,"frames remaining {remaining}/{total}",&[("remaining",remaining.to_string()),("total",command["frames"].as_u64().unwrap_or(0).to_string())]),
        Some("find")=>match search_remaining{Some(left)=>tr(language,"instructions remaining {remaining}/{total}",&[("remaining",left.to_string()),("total",command["settings"]["steps"].as_u64().unwrap_or(0).to_string())]),None=>language.text("search in progress").into()},
        Some("content-refresh")=>language.text("scan in progress").into(),
        Some("cheat-refresh")=>language.text("checking cheat catalog").into(),
        Some("select-id")=>language.text("selection in progress").into(),
        Some("replay")=>match replay_progress {
            Some((completed,total))=>tr(language,"replay frames {completed}/{total}",&[("completed",completed.to_string()),("total",total.to_string())]),
            None=>language.text("in progress").into(),
        },
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

/// Summarize the current operation, never an unrelated value retained in status.
/// Full, unmodified results remain available in the copied JSON.
pub(crate) fn result_lines(value: &Value, command:&str, language:Language) -> Vec<String> {
    let kind=value["kind"].as_str().unwrap_or("");
    if kind=="ffta-debug-control-result" {return control_result_lines(value,command,language);}
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

fn control_result_lines(value:&Value,command:&str,language:Language)->Vec<String>{
    // Failed actions can carry an older status snapshot. Do not present any of
    // its retained reports or inspection data as the result of this request.
    if value["ok"]!=true{return Vec::new();}
    let status=&value["status"];
    let operation=value["operation_id"].as_str().unwrap_or("");
    let mut lines=Vec::new();
    match operation {
        "control.inspect"=>{
            let inspection=&status["inspection"];
            if let (Some(address),Some(length),Some(bytes))=(inspection["address"].as_u64(),inspection["length"].as_u64(),inspection["bytes"].as_array()){
                let shown=bytes.iter().take(16).map(|b|b["value"].as_u64().filter(|v|*v<=255).map_or_else(||"??".into(),|v|format!("{v:02X}"))).collect::<Vec<_>>().join(" ");
                let suffix=if bytes.len()>16{" …"}else{""};
                lines.push(tr(language,"Inspection {address} · {length} bytes: {bytes}",&[("address",format!("0x{address:08X}")),("length",length.to_string()),("bytes",format!("{shown}{suffix}"))]));
            }
        },
        "content.list"|"cheats.list"=>{
            let page=&status[if operation=="content.list"{"content_page"}else{"cheat_page"}];
            if let (Some(offset),Some(total),Some(entries))=(page["offset"].as_u64(),page["total"].as_u64(),page["entries"].as_array()){
                let next=page["next_offset"].as_u64().map_or_else(||"-".into(),|n|n.to_string());
                let key=if operation=="content.list"{"Content page {offset}/{total} · next {next}"}else{"Cheat page {offset}/{total} · next {next}"};
                lines.push(tr(language,key,&[("offset",offset.to_string()),("total",total.to_string()),("next",next)]));
                for entry in entries.iter().take(3){
                    let id=clean(entry["id"].as_str().unwrap_or("?"),72);
                    let name=clean(entry[if operation=="content.list"{"name"}else{"title"}].as_str().unwrap_or("?"),100);
                    lines.push(format!("{id} · {name}"));
                }
                if entries.len()>3{lines.push(language.text("More entries in copied JSON.").into());}
            }
        },
        "cheats.show"|"cheats.capabilities"=>{
            let detail=&status[if operation=="cheats.show"{"cheat_detail"}else{"cheat_capabilities"}];
            if let (Some(id),Some(offset),Some(total))=(detail["id"].as_str(),detail["offset_bytes"].as_u64(),detail["total_bytes"].as_u64()){
                let next=detail["next_offset_bytes"].as_u64().map_or_else(||"-".into(),|n|n.to_string());
                lines.push(tr(language,"Detail {id}: bytes {offset}/{total} · next {next}",&[("id",clean(id,72)),("offset",offset.to_string()),("total",total.to_string()),("next",next)]));
            }
        },
        "content.select"=>{
            let requested=match super::parse(command){
                Ok(super::Command::Select{slot,id})=>Some((slot,id)),
                Ok(super::Command::Retry{action:crate::debug_control::Action::SelectId{slot,id},..})=>Some((slot.content_slot(),id)),
                _=>None,
            };
            if let Some((slot,id))=requested {
                if status["content_picker"]["inputs"].as_array().is_some_and(|inputs|inputs.iter().any(|input|input["slot"]==slot.label() && input["id"]==id)){
                    lines.push(tr(language,"Selected content: {id}",&[("id",clean(&id,100))]));
                }
            }
        },
        "content.scan"=>{
            let picker=&status["content_picker"];
            if let (Some(entries),Some(generation))=(picker["entries"].as_u64(),picker["generation"].as_u64()){
                lines.push(tr(language,"Content scan: {entries} entries · generation {generation}",&[("entries",entries.to_string()),("generation",generation.to_string())]));
            }
        },
        "cheats.prepare-once"|"cheats.preset-prepare"=>{
            let preview=&status["cheat_preview"];
            if let Some(token)=preview["token"].as_str().filter(|t|t.len()==64 && t.bytes().all(|b|b.is_ascii_hexdigit())){
                lines.push(tr(language,"Preview token: {token} · review copied JSON",&[("token",token.into())]));
            }
        },
        "cheats.apply-once"=>{
            let applied=&status["cheat_once"];
            if let (Some(id),Some(writes))=(applied["id"].as_str(),applied["writes"].as_u64()){
                lines.push(tr(language,"Applied once: {id} · writes {writes}",&[("id",clean(id,72)),("writes",writes.to_string())]));
            }
        },
        "cheats.arm"|"cheats.disarm"|"cheats.preset-arm"=>{
            if let Some(entries)=status["cheat_armed"]["entries"].as_array(){
                lines.push(tr(language,"Maintained cheats: {count} · disarm does not undo writes",&[("count",entries.len().to_string())]));
            }
        },
        "control.break"|"control.watch"=>{
            let probe=&status["probe"];
            if let Some(outcome)=probe["outcome"].as_str(){lines.push(tr(language,"Search: {outcome} · {boundary} · steps {steps}",&[("outcome",clean(outcome,48)),("boundary",clean(probe["boundary"].as_str().unwrap_or("?"),64)),("steps",probe["steps"].to_string())]));}
        },
        "replay.run"|"audit.run"|"capture.run"|"compare.run"=>{
            let report=&status["replay"]["result"]["report"];
            if let (Some(outcome),Some(completed),Some(total))=(report["outcome"].as_str(),report["completed"].as_u64(),report["frames"].as_u64()){
                lines.push(tr(language,"Replay: {outcome} · frames {completed}/{total}",&[("outcome",clean(outcome,48)),("completed",completed.to_string()),("total",total.to_string())]));
            }
        },
        _=>{
            let report=match operation {
                "control.graphics"=>&status["graphics_report"],"control.ui-report"=>&status["ui_trace"]["last_report"],
                "control.audio-report"=>&status["audio_report"],"control.code-report"=>&status["code_report"],
                "control.gameplay-report"=>&status["gameplay_report"],"control.recent-report"=>&status["recent_report"],
                "control.issue-report"=>&status["issue_report"],_=>&Value::Null,
            };
            if let Some(path)=report.as_str(){lines.push(tr(language,"Report: {path}",&[("path",clean(path,200))]));}
        },
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;use serde_json::json;
    #[test]
    fn control_results_are_scoped_bounded_and_do_not_present_stale_failures(){
        let en=Language::default();
        let response=|operation:&str,status:Value|json!({"kind":"ffta-debug-control-result","ok":true,"operation_id":operation,"status":status});
        let inspection=response("control.inspect",json!({"inspection":{"address":134217728,"length":20,"bytes":[{"value":46},{"value":0},{"value":234}]}}));
        assert!(result_lines(&inspection,"inspect 0x08000000 20",en)[0].contains("0x08000000 · 20 bytes: 2E 00 EA"));
        let page=response("content.list",json!({"content_page":{"offset":2,"total":9,"next_offset":6,"entries":[{"id":"entry-1","name":"Rome"},{"id":"entry-2","name":"日"},{"id":"entry-3","name":"C"},{"id":"entry-4","name":"D"}]}}));
        let lines=result_lines(&page,"content list",en);assert_eq!(lines.len(),5);assert!(lines[0].contains("2/9 · next 6"));assert!(lines[2].contains("entry-2 · 日"));assert_eq!(lines[4],"More entries in copied JSON.");
        let detail=response("cheats.capabilities",json!({"cheat_capabilities":{"id":"CH-0001","offset_bytes":10,"total_bytes":40,"next_offset_bytes":20}}));
        assert!(result_lines(&detail,"cheats capabilities CH-0001",en)[0].contains("CH-0001: bytes 10/40 · next 20"));
        let report=response("control.audio-report",json!({"audio_report":"Data/config/audio/report.html"}));
        assert!(result_lines(&report,"audio-report",en)[0].contains("Data/config/audio/report.html"));
        let mut refused=report.clone();refused["ok"]=json!(false);assert!(result_lines(&refused,"audio-report",en).is_empty());
        let wrong_operation=response("control.step",json!({"inspection":{"address":134217728,"length":1,"bytes":[{"value":46}]},"audio_report":"stale"}));
        assert!(result_lines(&wrong_operation,"step 1",en).is_empty());
        let selection=response("content.select",json!({"content_picker":{"selected":{"id":"stale-cursor"},"inputs":[{"slot":"Replay / audit / capture / compare profile","id":"profile-1"}]}}));
        assert_eq!(result_lines(&selection,"profile select profile-1",en),vec!["Selected content: profile-1"]);
        assert_eq!(result_lines(&selection,"retry run req 123 -- profile select profile-1",en),vec!["Selected content: profile-1"]);
        assert!(result_lines(&selection,"profile select stale-cursor",en).is_empty());
        assert!(result_lines(&selection,"catalog select profile-1",en).is_empty());
        let replay=response("replay.run",json!({"replay":{"result":{"report":{"outcome":"completed","completed":102,"frames":102}}}}));
        assert!(result_lines(&replay,"replay run",en)[0].contains("completed · frames 102/102"));
        let pending_replay=json!({"request_id":"job-1","command":{"action":"replay","mode":"plain"}});
        assert!(pending(&pending_replay,0,None,Some((10,102)),en).0.contains("replay frames 10/102"));
        assert!(pending(&pending_replay,0,None,None,en).0.contains("in progress"));
        assert!(pending(&pending_replay,0,None,Some((10,102)),Language::parse("it")).0.contains("frame replay 10/102"));
    }
    #[test]
    fn report_and_replay_summaries_follow_the_current_operation(){
        let en=Language::default();
        let status=json!({"graphics_report":"graphics.html","ui_trace":{"last_report":"ui.html"},"audio_report":"audio.html","code_report":"code.html","gameplay_report":"gameplay.html","recent_report":"recent.html","issue_report":"issue.html","replay":{"result":{"report":{"outcome":"completed","completed":102,"frames":102}}},"probe":{"outcome":"hit","boundary":"before-step","steps":7}});
        for (operation,path) in [("control.graphics","graphics.html"),("control.ui-report","ui.html"),("control.audio-report","audio.html"),("control.code-report","code.html"),("control.gameplay-report","gameplay.html"),("control.recent-report","recent.html"),("control.issue-report","issue.html")]{
            let value=json!({"kind":"ffta-debug-control-result","ok":true,"operation_id":operation,"status":status});
            let lines=result_lines(&value,"",en);assert_eq!(lines,vec![format!("Report: {path}")]);
        }
        for operation in ["replay.run","audit.run","capture.run","compare.run"]{
            let value=json!({"kind":"ffta-debug-control-result","ok":true,"operation_id":operation,"status":status});
            assert_eq!(result_lines(&value,"",en),vec!["Replay: completed · frames 102/102"]);
        }
        for operation in ["control.break","control.watch"]{
            let value=json!({"kind":"ffta-debug-control-result","ok":true,"operation_id":operation,"status":status});
            assert_eq!(result_lines(&value,"",en),vec!["Search: hit · before-step · steps 7"]);
        }
        let preview=json!({"kind":"ffta-debug-control-result","ok":true,"operation_id":"cheats.prepare-once","status":{"cheat_preview":{"token":"a".repeat(64)}}});
        assert!(result_lines(&preview,"",en)[0].contains(&"a".repeat(64)));
        let mut invalid_preview=preview.clone();invalid_preview["status"]["cheat_preview"]["token"]=json!("stale");assert!(result_lines(&invalid_preview,"",en).is_empty());
        let once=json!({"kind":"ffta-debug-control-result","ok":true,"operation_id":"cheats.apply-once","status":{"cheat_once":{"id":"CH-0001","writes":2}}});
        assert_eq!(result_lines(&once,"",en),vec!["Applied once: CH-0001 · writes 2"]);
        let armed=json!({"kind":"ffta-debug-control-result","ok":true,"operation_id":"cheats.disarm","status":{"cheat_armed":{"entries":[]}}});
        assert_eq!(result_lines(&armed,"",en),vec!["Maintained cheats: 0 · disarm does not undo writes"]);
        let no_status=json!({"kind":"ffta-debug-control-result","ok":true,"operation_id":"control.inspect","status":null});
        assert!(result_lines(&no_status,"",en).is_empty());
    }
    #[test]
    fn trigger_results_show_conditions_boundaries_and_conflicts() {
        let value=json!({"kind":"ffta-debug-trigger-result","definition":{"name":"alpha","enabled":true,"action":{"settings":{"condition":{"kind":"pc","address":134217728,"thumb":false},"steps":100,"window":{"hit_count":2},"on_hit":"freeze"}}},"response":{"status":{"probe":{"outcome":"hit","boundary":"before-interpreter-step","steps":4,"completed_frames":0,"evaluation":{"observed_hits":2,"candidate_count":3,"last_guard_value":5,"stop_reason":"target-hit-count"},"post_action":{"kind":"freeze","outcome":"completed"}}}}});
        let lines=result_lines(&value,"trigger show alpha",Language::default()).join("\n");
        for expected in ["alpha", "enabled", "Condition: PC 0x08000000 arm", "Window:", "freeze", "before-interpreter-step", "Hits 2", "last guard 5", "exclusive observer", "Interpreter only"] {assert!(lines.contains(expected),"{expected}: {lines}");}
        assert!(result_lines(&json!({"kind":"other"}),"",Language::default()).is_empty());
    }
    #[test]
    fn pending_search_includes_all_extended_settings() {
        let search=json!({"request_id":"id-3","command":{"action":"find","settings":{"steps":5000000,"condition":{"kind":"pc","address":134217728,"thumb":true},"window":{"hit_count":42,"start_frame":3,"end_frame":99},"guard":{"address":33554432,"width":2,"value":4660},"on_hit":"freeze","trigger":{"name":"alpha","sha256":"a".repeat(64)}}}});
        let (progress,params)=pending(&search,0,Some(4500000),None,Language::default());
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
        let (progress,params)=pending(&step,3,None,None,en);assert!(progress.contains("frames remaining 3/10")&&progress.contains("id-1"));assert!(params.contains("0x3ff"));
        assert!(pending(&step,3,None,None,it).0.contains("frame rimasti 3/10"));
        let search=json!({"request_id":"id-2","command":{"action":"find","settings":{"steps":100,"condition":{"kind":"pc","address":134217728,"thumb":false}}}});
        let (progress,params)=pending(&search,0,Some(75),None,en);assert!(progress.contains("id-2")&&progress.contains("75/100"));assert!(params.contains("0x08000000 arm"));
        assert_eq!(pending(&Value::Null,0,None,None,en).0,"No pending job");assert_eq!(pending(&Value::Null,0,None,None,it).0,"Nessun job pendente");
        let long=json!({"request_id":"x".repeat(100),"command":{"action":"select-id","id":"y".repeat(512)}});
        let (progress,params)=pending(&long,0,None,None,en);assert!(progress.contains(&"x".repeat(100))&&params.contains(&"y".repeat(512)));
        let excessive=json!({"request_id":"x".repeat(500),"command":{"action":"select-id","id":"y".repeat(5000)}});
        let (progress,params)=pending(&excessive,0,None,None,en);assert!(progress.len()<160&&params.len()<600);
    }
    #[test]
    fn pins_show_sources_changes_stale_and_filter(){
        use menu::command_console::PinState;
        let pins=json!({"filter":"changed","offset":0,"pins":[
            {"label":"x","owner":"a","previous":1,"value":2,"changed":true,"stale":false,"source":{"kind":"register","register":3}},
            {"label":"y","owner":"b","previous":2,"value":2,"changed":false,"stale":true,"source":{"kind":"readonly-backing","address":33554432}}
        ]});
        let lines=pin_lines(&pins);assert_eq!(lines.len(),2);assert_eq!(lines[1].state,PinState::Changed);assert!(lines[1].text.contains("r3")&&lines[1].text.contains("1 → 2"));
        let mut stale=pins;stale["filter"]=json!("stale");let lines=pin_lines(&stale);assert_eq!(lines[1].state,PinState::Stale);assert!(lines[1].text.contains("0x02000000"));
    }
}
