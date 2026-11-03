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
    let label=clean(manifest["label"].as_str().unwrap_or("?"),12);
    let hash=manifest["pins"]["ffta.gba"].as_str().filter(|h|h.len()==64&&h.bytes().all(|c|c.is_ascii_hexdigit())).unwrap_or("");
    let rom=if hash.is_empty(){"?".into()}else{format!("{}...",&hash[..12])};
    tr(language,"Session {label} | ROM {rom} | run {run}",&[("label",label),("rom",rom),("run",clean(run,32))])
}
pub(crate) fn verified_target(bytes:&[u8],run:&str,expected:&str,language:Language)->Result<String,String>{
    if gba_core::ffta_rom::digest(bytes)!=expected{return Err("Session manifest changed; reopen the debug copy".into());}
    let manifest:Value=serde_json::from_slice(bytes).map_err(|e|e.to_string())?;Ok(target(&manifest,run,language))
}

pub(crate) fn pending(value:&Value,remaining:u32,search_remaining:Option<u64>,language:Language)->(String,String) {
    if value.is_null(){return (language.text("No pending job").into(),language.text("Parameters: -").into());}
    let id=clean(value["request_id"].as_str().unwrap_or("?"),32);
    let command=&value["command"];
    let progress=match command["action"].as_str(){
        Some("step")=>tr(language,"frames remaining {remaining}/{total}",&[("remaining",remaining.to_string()),("total",command["frames"].as_u64().unwrap_or(0).to_string())]),
        Some("find")=>match search_remaining{Some(left)=>tr(language,"instructions remaining {remaining}/{total}",&[("remaining",left.to_string()),("total",command["settings"]["steps"].as_u64().unwrap_or(0).to_string())]),None=>language.text("search in progress").into()},
        Some("content-refresh")=>language.text("scan in progress").into(),
        Some("cheat-refresh")=>language.text("checking cheat catalog").into(),
        Some("select-id")=>language.text("selection in progress").into(),
        _=>language.text("in progress").into(),
    };
    let parameters=match command["action"].as_str(){
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
        Some("select-id")=>format!("select {} {}",command["slot"].as_str().unwrap_or("?"),clean(command["id"].as_str().unwrap_or("?"),72)),
        _=>serde_json::to_string(command).unwrap_or_else(|_|"?".into()),
    };
    (format!("ID {id} | {progress}"),tr(language,"Parameters: {parameters}",&[("parameters",clean(&parameters,256))]))
}

#[cfg(test)]
mod tests {
    use super::*;use serde_json::json;
    #[test]
    fn target_and_pending_progress_are_explicit_and_bounded(){
        let manifest=json!({"label":"Prova","pins":{"ffta.gba":"a".repeat(64)}});
        let en=Language::default();let it=Language::parse("it");
        let target=target(&manifest,"run-42",en);assert!(target.starts_with("Session ")&&target.contains("Prova")&&target.contains("aaaaaaaaaaaa...")&&target.contains("run-42"));
        assert!(super::target(&manifest,"run-42",it).starts_with("Sessione "));
        let braces=json!({"label":"{rom}","pins":{"ffta.gba":"a".repeat(64)}});
        assert!(super::target(&braces,"run-42",en).contains("Session {rom} | ROM aaaaaaaaaaaa..."));
        let bytes=serde_json::to_vec(&manifest).unwrap();let hash=gba_core::ffta_rom::digest(&bytes);
        assert!(verified_target(&bytes,"run-42",&hash,en).unwrap().contains("Prova"));assert!(verified_target(&bytes,"run-42",&"0".repeat(64),en).is_err());
        let step=json!({"request_id":"id-1","command":{"action":"step","frames":10,"keys":1023}});
        let (progress,params)=pending(&step,3,None,en);assert!(progress.contains("frames remaining 3/10")&&progress.contains("id-1"));assert!(params.contains("0x3ff"));
        assert!(pending(&step,3,None,it).0.contains("frame rimasti 3/10"));
        let search=json!({"request_id":"id-2","command":{"action":"find","settings":{"steps":100,"condition":{"kind":"pc","address":134217728,"thumb":false}}}});
        let (progress,params)=pending(&search,0,Some(75),en);assert!(progress.contains("id-2")&&progress.contains("75/100"));assert!(params.contains("0x08000000 arm"));
        assert_eq!(pending(&Value::Null,0,None,en).0,"No pending job");assert_eq!(pending(&Value::Null,0,None,it).0,"Nessun job pendente");
        let long=json!({"request_id":"x".repeat(500),"command":{"action":"select-id","id":"y".repeat(500)}});
        let (progress,params)=pending(&long,0,None,en);assert!(progress.len()<160&&params.len()<300);
    }
}
