//! A local issue index references an existing portable replay; never sends payloads.
use serde_json::{Value,json};
use std::{fs,path::{Path,PathBuf,Component},time::{SystemTime,UNIX_EPOCH}};
const PS:&str = "$ErrorActionPreference = 'Stop'\n$root = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '../../../..')).Path\n& (Join-Path $root 'recomp.exe') debug-replay run (Join-Path $root 'replay.json') --json\nexit $LASTEXITCODE\n";
const SH:&str = "#!/bin/sh\nset -eu\nreport_dir=$(CDPATH= cd -- \"$(dirname -- \"$0\")\" && pwd)\nprofile_dir=$(CDPATH= cd -- \"$report_dir/../../../..\" && pwd)\nif [ -f \"$profile_dir/recomp.exe\" ]; then\n  exec wine64 \"$profile_dir/recomp.exe\" debug-replay run \"$profile_dir/replay.json\" --json\nfi\nexec \"$profile_dir/recomp\" debug-replay run \"$profile_dir/replay.json\" --json\n";
fn digest(b:&[u8])->String{gba_core::ffta_rom::digest(b)}
fn read(path:&Path,limit:u64)->Result<Vec<u8>,String>{crate::diagnostics::read(path,limit)}
pub(crate) fn validate_symptom(s:&str)->Result<(),String>{
    if s.len()>2048 || s.chars().any(|c|c.is_control() && c!='\n' && c!='\t'){return Err("Symptom must be at most 2048 bytes without control characters".into());}Ok(())
}
fn relative(s:&str)->bool{!s.is_empty() && Path::new(s).components().all(|c|matches!(c,Component::Normal(_))) && !s.contains('\\')}
fn href(s:&str)->String{s.bytes().map(|b|if b.is_ascii_alphanumeric() || b"-._~/".contains(&b){(b as char).to_string()}else{format!("%{b:02X}")}).collect()}
fn attachment(root:&Path,name:&str,role:&str,limit:u64)->Value{
    match read(&root.join(name),limit){Ok(bytes)=>json!({"role":role,"path":name,"href":format!("../../../../{}",href(name)),"bytes":bytes.len(),"sha256":digest(&bytes),"privacy":"private; do not distribute automatically"}),Err(error)=>json!({"role":role,"path":name,"error":error,"privacy":"private"})}
}
pub(crate) struct Created {pub path:PathBuf,pub compatible:bool}
pub(crate) struct IssueContext {
    pub session_root:PathBuf,
    pub timeline:Value,
    pub selections:Value,
    pub active_cheats:Value,
    pub reports:Vec<(String,String)>,
    pub reports_omitted:usize,
}
/// Build a relative link only to an ordinary, existing report beneath the
/// selected session. Moving the profile alone may break this private link.
fn session_link(issue_dir:&Path,session:&Path,name:&str)->Result<String,String>{
    if !relative(name)||!name.starts_with("Data/config/")||!name.ends_with("/index.html"){return Err("Report is not a session-relative report index".into());}
    let root=fs::canonicalize(session).map_err(|e|e.to_string())?;
    let mut current=root.clone();
    for component in Path::new(name).components(){if let Component::Normal(part)=component{current.push(part);crate::debug_session::regular(&current)?;}else{return Err("Invalid report path".into());}}
    if !current.is_file(){return Err("Report index is not a file".into());}
    let from:Vec<_>=issue_dir.components().collect();let to:Vec<_>=current.components().collect();
    let common=from.iter().zip(&to).take_while(|(a,b)|a==b).count();
    if common==0 || from.first()!=to.first(){return Err("Report is on another filesystem root".into());}
    let mut rel=PathBuf::new();for _ in common..from.len(){rel.push("..");}for component in &to[common..]{rel.push(component.as_os_str());}
    Ok(href(&rel.to_string_lossy().replace('\\',"/")))
}
pub(crate) fn create(profile:&Path,symptom:&str)->Result<Created,String>{
    create_with_context(profile,symptom,None)
}
pub(crate) fn create_with_context(profile:&Path,symptom:&str,context:Option<IssueContext>)->Result<Created,String>{
    validate_symptom(symptom)?;
    if profile.file_name().and_then(|s|s.to_str())!=Some("replay.json"){return Err("Select the existing profile's replay.json".into());}
    let root=fs::canonicalize(profile.parent().filter(|p|!p.as_os_str().is_empty()).unwrap_or(Path::new("."))).map_err(|e|e.to_string())?;
    let profile=root.join("replay.json");let base=root.join("Data/config/issues");
    for name in ["Data","Data/config","Data/config/issues"] {
        let path=root.join(name);
        if let Ok(m)=fs::symlink_metadata(&path){
            let linked=m.file_type().is_symlink();
            #[cfg(windows)]
            let linked={
                use std::os::windows::fs::MetadataExt;
                linked||m.file_attributes()&0x400!=0
            };
            if linked||!m.is_dir(){return Err("Issue output needs ordinary profile-local directories".into());}
        }
    }
    fs::create_dir_all(root.join("Data/config")).map_err(|e|e.to_string())?;
    let _writer=fs::OpenOptions::new().create(true).truncate(false).read(true).write(true).open(root.join("Data/config/issue.lock")).map_err(|e|e.to_string())?;
    _writer.try_lock().map_err(|_|"Another issue report is being written")?;
    if base.exists() && fs::read_dir(&base).map_err(|e|e.to_string())?.count()>=8{return Err("Eight issue reports already exist; retain/archive before another".into());}
    let bytes=read(&profile,64*1024);
    let declared=bytes.as_ref().ok().and_then(|b|serde_json::from_slice::<Value>(b).ok()).unwrap_or(Value::Null);
    let metadata=crate::debug_replay::issue_metadata(&profile);
    let compatible=metadata.is_ok();let error=metadata.as_ref().err().cloned();
    let mut warnings=Vec::<String>::new();
    let result=read(&root.join("Data/config/replay/result.json"),8192).ok().and_then(|b|serde_json::from_slice::<Value>(&b).ok());
    let result=result.filter(|v|{
        let ok=v["schema"]==1 && v["kind"]=="ffta-debug-replay-result"
            && ["runtime_sha256","rom_sha256","input_sha256"].iter().all(|k|v[*k].as_str().is_some_and(|s|s.len()==64&&s.bytes().all(|b|b.is_ascii_hexdigit())))
            && ["runtime_sha256","rom_sha256","input_sha256","library_sha256","dataset_sha256"].iter().all(|k|v[*k]==declared[*k])
            && v["cheat_timeline"]["timeline_sha256"]==declared["cheat_timeline_sha256"]
            && v["starting_cheat_ancestry"]["sha256"]==declared["start_cheat_ancestry_sha256"];
        if !ok{warnings.push("Retained result identity/schema differs; excluded from the issue conclusion".into());}ok
    });
    let mut comparison=Value::Null;
    if let Some(name)=result.as_ref().and_then(|r|r["comparison_report"].as_str()){
        if relative(name) && name.starts_with("comparisons/") && name.ends_with("/index.html"){
            let path=root.join(name).with_file_name("comparison.json");
            if let Ok(raw)=read(&path,2*1024*1024){
                if let Ok(c)=serde_json::from_slice::<Value>(&raw){
                    if c["schema"]==1 && c["kind"]=="ffta-first-divergence" && c["run"]==result.as_ref().unwrap()["run"]
                        && ["runtime_sha256","rom_sha256","input_sha256","start_sha256","library_sha256"].iter().all(|k|c["origin"][*k]==declared[*k]) {
                        comparison=json!({"href":format!("../../../../{}",href(name)),"sha256":digest(&raw),"outcome":c["outcome"],"first_difference":c["first_difference"],"different_components":c["different_components"],"message":c["message"],"scope":"Retained comparison metadata, not a new replay or correctness acceptance"});
                    }else{warnings.push("Comparison identity does not match selected profile/result".into());}
                }
            }else{warnings.push("Retained comparison metadata missing/unreadable".into());}
        }else{warnings.push("Comparison path is not a bounded profile-relative report".into());}
    }
    let outcome=result.as_ref().and_then(|r|r["outcome"].as_str()).unwrap_or("unknown-no-compatible-result");
    let next=if !compatible{"Restore the input identified by preflight, then create a new report; keep the failed report"}
        else{match outcome{"diverged" if !comparison.is_null()=>"Open the retained first-difference report and inspect the named components; the pinned command can reproduce this profile",
        "diverged"=>"Create a first-difference comparison from the same pinned inputs; endpoint difference alone does not locate the cause",
        "passed"|"matched"=>"Describe the visible symptom or choose a narrower recorded window; a matching endpoint does not exclude a UI/audio defect",
        "timeout"|"interrupted"|"error"|"incompatible"=>"Inspect the retained worker log and declared timeout/cancellation/preflight before choosing another run",
        _=>"Run the pinned command when ready; this report itself has not executed the case"}};
    let selected_frames=declared["comparison"]["frames"].as_u64().or_else(||declared["capture"]["limit_frames"].as_u64()).or_else(||declared["frames"].as_u64());
    let cheats=metadata.as_ref().ok().and_then(|value|value.get("cheat_timeline")).cloned().unwrap_or(Value::Null);
    let starting_cheats=metadata.as_ref().ok().and_then(|value|value.get("starting_cheat_ancestry")).cloned().unwrap_or(Value::Null);
    let mut summary=json!({"schema":1,"kind":"ffta-issue-summary","symptom":if symptom.is_empty(){"Not supplied"}else{symptom},"case":declared["case"],"preflight_compatible":compatible,
        "outcome":outcome,"frame_origin":declared["first_frame"],"recorded_frames":declared["frames"],"selected_frames":selected_frames,"completed":result.as_ref().map(|r|&r["completed"]),
        "first_difference":comparison["first_difference"],"different_components":comparison["different_components"],"cheat_timeline":cheats,"starting_cheat_ancestry":starting_cheats,
        "identities":{"runtime":declared["runtime_sha256"],"recording_runtime":declared["recording_runtime_sha256"],"rom":declared["rom_sha256"],"bios":declared["bios_sha256"],"dataset":declared["dataset_sha256"],"snapshot":declared["start_sha256"],"input":declared["input_sha256"],"expected_endpoint":declared["end_sha256"],"library":declared["library_sha256"]},
        "next_check":next,"sharing":"Metadata candidate only: review symptom, case and identities before sharing. No game bytes or absolute paths included"});
    let mut attachments=Vec::new();
    for (name,role,limit) in [("replay.json","pinned replay profile",64*1024),("debug-session.json","session pins",1024*1024),("input.input","private recorded input",8*1024*1024),("seed.ffstate","private start snapshot",65*1024*1024),("expected.ffstate","private endpoint",65*1024*1024),("Data/config/replay/result.json","retained replay result",8192)]{
        attachments.push(attachment(&root,name,role,limit));
    }
    attachments.push(attachment(&root,"ffta.gba","private ROM",32*1024*1024));
    if !declared["cheat_timeline_sha256"].is_null(){
        attachments.push(attachment(&root,"cheats-timeline.json","private recorded cheat applications",crate::debug_cheat_timeline::MAX_BYTES));
        attachments.push(attachment(&root,"catalogo.json","private source cheat catalog",4*1024*1024));
    }
    if !declared["start_cheat_ancestry_sha256"].is_null(){attachments.push(attachment(&root,"cheat-ancestry.json","private starting cheat ancestry",256*1024));}
    attachments.push(attachment(&root,if root.join("recomp.exe").exists(){"recomp.exe"}else{"recomp"},"selected runtime",256*1024*1024));
    if root.join("gba_bios.bin").exists(){attachments.push(attachment(&root,"gba_bios.bin","private BIOS",16384));}
    if !declared["library_sha256"].is_null(){attachments.push(attachment(&root,"native.bin","private native translation",256*1024*1024));}
    if !declared["dataset_sha256"].is_null(){attachments.push(attachment(&root,"Data/dataset/manifest.toml","private dataset manifest",gba_core::rom_data::MAX_MANIFEST_FILE_BYTES));}
    let name=format!("{}-{}",SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e|e.to_string())?.as_nanos(),std::process::id());let dir=base.join(name);
    let mut context_data=Value::Null;let mut timeline_bytes=None;let mut recipe_bytes=None;
    if let Some(context)=context {
        if context.reports.len()>8 || serde_json::to_vec(&context.timeline).map_err(|e|e.to_string())?.len()>512*1024 || serde_json::to_vec(&context.selections).map_err(|e|e.to_string())?.len()>128*1024 || serde_json::to_vec(&context.active_cheats).map_err(|e|e.to_string())?.len()>128*1024 {return Err("Issue context exceeds bounded limits".into());}
        let links:Vec<Value>=context.reports.iter().map(|(role,path)|match session_link(&dir,&context.session_root,path){Ok(link)=>json!({"role":role,"path":path,"href":link,"privacy":"private session report; move it with its session"}),Err(error)=>json!({"role":role,"path":path,"error":error,"privacy":"private"})}).collect();
        let bytes=serde_json::to_vec_pretty(&context.timeline).map_err(|e|e.to_string())?;
        attachments.push(json!({"role":"private issue timeline","path":"timeline.json","href":"timeline.json","bytes":bytes.len(),"sha256":digest(&bytes),"privacy":"private; review before sharing"}));timeline_bytes=Some(bytes);
        if context.timeline["recipe"]["recipe"].is_object(){let bytes=serde_json::to_vec_pretty(&context.timeline["recipe"]["recipe"]).map_err(|e|e.to_string())?;attachments.push(json!({"role":"private selected command recipe","path":"recipe.json","href":"recipe.json","bytes":bytes.len(),"sha256":digest(&bytes),"privacy":"private; selection does not prove execution"}));recipe_bytes=Some(bytes);}
        context_data=json!({"timeline":context.timeline,"selected_inputs":context.selections,"active_cheats":context.active_cheats,"related_reports":links,"related_reports_omitted":context.reports_omitted,"scope":"Private observed session context. Ordering and request IDs do not establish causation; profile replay remains the reproducible command."});
    }
    summary["session_context"]=if context_data.is_null(){json!({"available":false,"gap":"Offline report did not capture a live session timeline"})}else{json!({"available":true,"events":context_data["timeline"]["events"].as_array().map_or(0,Vec::len),"collector":context_data["timeline"]["collector"],"related_reports":context_data["related_reports"].as_array().map_or(0,Vec::len),"selected_recipe":!context_data["timeline"]["recipe"].is_null(),"privacy":"Full timeline, selected IDs, active cheats and recipe are private"})};
    let data=json!({"schema":1,"kind":"ffta-issue-report","summary":summary,"preflight":match metadata{Ok(v)=>json!({"compatible":true,"checked":v}),Err(e)=>json!({"compatible":false,"error":e})},
        "declared_profile":declared,"profile_file_sha256":bytes.as_ref().ok().map(|b|digest(b)),"retained_result":result,"comparison":comparison,"warnings":warnings,"attachments":attachments,"session_context":context_data,
        "commands":{"working_directory":"report folder; launchers resolve the profile from their own location","profile_relative_root":"../../../..","windows_script":"reproduce.ps1","posix_script":"reproduce.sh","argv":["profile's pinned runtime","debug-replay","run","profile's replay.json","--json"]},
        "builder":{"runtime_sha256":digest(&read(&std::env::current_exe().map_err(|e|e.to_string())?,256*1024*1024)?),"revision":env!("FFTA_BUILD_REVISION"),"target":env!("FFTA_BUILD_TARGET")},
        "privacy":{"issue_json":"Private paths/error details/pins; review before sharing","summary_json":"Metadata candidate, no source paths or game bytes; symptom can still be personal","payloads":"Referenced in place, never copied/uploaded by this operation"},
        "retained_result_scope":"Only schema and available runtime/ROM/input/library/dataset identity fields matched; freshness and settings of older results are not authenticated. Comparison origin is checked separately",
        "scope":"Readonly identities and retained evidence; no guest execution, new snapshot, benchmark or automatic send. Retained result may be historical and is not a new validation"});
    let json=serde_json::to_vec(&data).map_err(|e|e.to_string())?;if json.len()>2*1024*1024{return Err("Issue JSON exceeds 2 MiB".into());}
    fs::create_dir_all(&base).map_err(|e|e.to_string())?;
    fs::create_dir(&dir).map_err(|e|e.to_string())?;
    fs::write(dir.join(".gitignore"),"*\n").map_err(|e|e.to_string())?;
    fs::write(dir.join("summary.json"),serde_json::to_vec_pretty(&data["summary"]).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    fs::write(dir.join("reproduce.ps1"),PS).map_err(|e|e.to_string())?;fs::write(dir.join("reproduce.sh"),SH).map_err(|e|e.to_string())?;
    if let Some(bytes)=timeline_bytes{fs::write(dir.join("timeline.json"),bytes).map_err(|e|e.to_string())?;}
    if let Some(bytes)=recipe_bytes{fs::write(dir.join("recipe.json"),bytes).map_err(|e|e.to_string())?;}
    fs::write(dir.join("index.html"),include_str!("debug_issue/report.html").replacen("/*REPORT*/",&String::from_utf8(json.clone()).unwrap().replace('<',"\\u003c"),1)).map_err(|e|e.to_string())?;
    fs::write(dir.join("issue.json"),json).map_err(|e|e.to_string())?;
    if let Some(e)=error{eprintln!("Issue preflight: {e}");}
    Ok(Created{path:dir.join("index.html"),compatible})
}
pub(crate) fn cmd(args:&[String])->u8{
    let result=(||{
        if args.len()<2 || args[0]!="create"{return Err("Use debug-report create PROFILE/replay.json [--symptom TEXT]".into());}
        let symptom=if args.len()==4 && args[2]=="--symptom"{args[3].as_str()}else if args.len()==2{""}else{return Err("Use one --symptom TEXT".into());};
        create(Path::new(&args[1]),symptom)
    })();
    match result{Ok(r)=>{println!("{}",json!({"schema":1,"kind":"ffta-issue-result","outcome":if r.compatible{"created"}else{"incompatible"},"report":r.path,"guest_execution":false}));if r.compatible{0}else{3}},Err(message)=>{println!("{}",json!({"schema":1,"kind":"ffta-issue-result","outcome":"incompatible","message":message}));3}}
}

#[cfg(test)]mod tests{
    use super::*;
    #[test]fn error_report_escaping_relative_commands_and_capacity(){
        let d=tempfile::tempdir().unwrap();let p=d.path().join("replay.json");fs::write(&p,r#"{"frames":102,"comparison":{"frames":2}}"#).unwrap();let original=fs::read(&p).unwrap();
        let created=create(&p,"</script><img src=x> ' # %").unwrap();assert!(!created.compatible);let root=created.path.parent().unwrap();
        let json=fs::read(root.join("issue.json")).unwrap();let data:Value=serde_json::from_slice(&json).unwrap();assert_eq!(data["preflight"]["compatible"],false);
        assert_eq!(data["summary"]["recorded_frames"],102);assert_eq!(data["summary"]["selected_frames"],2);
        assert!(!fs::read_to_string(&created.path).unwrap().contains("</script><img"));assert_eq!(fs::read_to_string(root.join("reproduce.ps1")).unwrap(),PS);assert!(SH.contains("exec wine64"));assert_eq!(fs::read(&p).unwrap(),original);
        assert!(relative("comparisons/run/index.html"));assert!(!relative("../outside"));assert_eq!(href("a #'é"),"a%20%23%27%C3%A9");
        let lock=fs::OpenOptions::new().read(true).write(true).open(d.path().join("Data/config/issue.lock")).unwrap();lock.try_lock().unwrap();
        assert!(create(&p,"").err().unwrap().contains("Another issue"));drop(lock);
        let base=root.parent().unwrap();for i in 0..7{fs::create_dir(base.join(format!("keep-{i}"))).unwrap();}
        assert!(create(&p,"").is_err());assert_eq!(fs::read(root.join("issue.json")).unwrap(),json);assert!(validate_symptom(&"x".repeat(2049)).is_err());
    }
    #[test]fn private_timeline_recipe_and_report_links_are_bounded_and_reconstructable(){
        let d=tempfile::tempdir().unwrap();let profile_root=d.path().join("profile");let session=d.path().join("session");
        fs::create_dir_all(&profile_root).unwrap();let report=session.join("Data/config/gameplay/run/index.html");fs::create_dir_all(report.parent().unwrap()).unwrap();fs::write(&report,"private report").unwrap();
        let profile=profile_root.join("replay.json");fs::write(&profile,b"{}").unwrap();
        let timeline=json!({"schema":1,"kind":"ffta-issue-timeline","events":[{"event_id":"run:1","frame":3,"clock":10,"stage":"bookmark","data":{"label":"before <hit>"}}],"collector":{"evicted":2,"oversize":0},"coverage":"partial","causation":"not proven","privacy":"private","recipe":{"name":"demo","sha256":"a".repeat(64),"recipe":{"schema":1,"name":"demo","steps":[]}}});
        let context=IssueContext{session_root:session.clone(),timeline:timeline.clone(),selections:json!([{"id":"content-1"}]),active_cheats:json!({"active":true,"entries":[{"id":"CH-0001"}]}),reports:vec![("Gameplay".into(),"Data/config/gameplay/run/index.html".into()),("Unsafe".into(),"../outside/index.html".into())],reports_omitted:1};
        let result=create_with_context(&profile,"Observed",Some(context)).unwrap();let issue=result.path.parent().unwrap();let data:Value=serde_json::from_slice(&fs::read(issue.join("issue.json")).unwrap()).unwrap();
        assert_eq!(data["session_context"]["timeline"],timeline);assert_eq!(data["session_context"]["selected_inputs"][0]["id"],"content-1");assert_eq!(data["session_context"]["active_cheats"]["entries"][0]["id"],"CH-0001");assert_eq!(data["session_context"]["related_reports_omitted"],1);
        let link=data["session_context"]["related_reports"][0]["href"].as_str().unwrap();assert_eq!(fs::canonicalize(issue.join(link)).unwrap(),fs::canonicalize(&report).unwrap());assert!(data["session_context"]["related_reports"][1]["error"].is_string());
        assert_eq!(serde_json::from_slice::<Value>(&fs::read(issue.join("timeline.json")).unwrap()).unwrap(),timeline);assert!(issue.join("recipe.json").exists());assert!(issue.join("reproduce.sh").exists());
        let html=fs::read_to_string(&result.path).unwrap();assert!(html.contains("event-${i}")&&html.contains("Open related private report"));assert!(!html.contains("before <hit>"));
        assert_eq!(fs::read(&profile).unwrap(),b"{}");assert_eq!(fs::read(report).unwrap(),b"private report");
        let oversized=IssueContext{session_root:session,timeline:json!({"padding":"x".repeat(512*1024)}),selections:json!([]),active_cheats:json!({}),reports:vec![],reports_omitted:0};assert!(create_with_context(&profile,"",Some(oversized)).is_err());
    }
}
