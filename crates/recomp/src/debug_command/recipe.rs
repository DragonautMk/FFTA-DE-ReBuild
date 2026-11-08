//! Inert, session-pinned typed command recipes. This module never executes steps.
mod checkpoint;
use super::{Command, parse};
use crate::debug_control::{Action, SelectionSlot};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs::{self,OpenOptions}, io::Write, path::{Path, PathBuf}};

const MAX_BYTES:u64=64*1024;
const MAX_STEPS:usize=16;
const MAX_FRAMES:u32=3600;
const MAX_RECIPES:usize=32;

#[derive(Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Recipe {
    schema:u32,
    kind:String,
    name:String,
    manifest_sha256:String,
    rom_sha256:String,
    runtime_sha256:String,
    limits:Limits,
    steps:Vec<Action>,
    #[serde(default,skip_serializing_if="Vec::is_empty")]
    assertions:Vec<Assertion>,
    #[serde(default,skip_serializing_if="Vec::is_empty")]
    waits:Vec<Wait>,
}
#[derive(Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Limits {steps:u32,explicit_step_frames:u32,max_frames:u32,duration_ms:u32,output_bytes:u32,
    #[serde(default,skip_serializing_if="Option::is_none")]
    search_frame_budget:Option<u32>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    replay_frame_budget:Option<u32>,
}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Assertion {after_step:u32,pointer:String,expected:Value}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Wait {after_step:u32,pointer:String,expected:Value,timeout_ms:u32,interval_ms:u32,max_polls:u32}

fn valid_pointer(pointer:&str)->bool{
    if !pointer.starts_with('/') || pointer.len()>256 || pointer.chars().any(char::is_control){return false;}
    let mut chars=pointer.chars();while let Some(c)=chars.next(){if c=='~' && !matches!(chars.next(),Some('0'|'1')){return false;}}true
}
fn valid_scalar(value:&Value)->bool{match value{Value::Null|Value::Bool(_)=>true,Value::Number(n)=>n.is_i64()||n.is_u64(),Value::String(s)=>s.len()<=256&&!s.chars().any(char::is_control),_=>false}}
fn scalar(token:&str)->Result<Value,String>{
    let value=if let Some(text)=token.strip_prefix("text:"){Value::String(text.into())}else{serde_json::from_str(token).map_err(|_|"Value must be a JSON scalar or text:VALUE")?};
    if !valid_scalar(&value){return Err("Invalid bounded scalar".into());}Ok(value)
}
fn wait(command:&str,after_step:usize)->Result<Option<Wait>,String>{
    let tokens=super::tokenize(command)?;if tokens.first().map(String::as_str)!=Some("wait"){return Ok(None);}
    if tokens.len()!=6 || after_step==0{return Err("Use wait JSON-POINTER SCALAR TIMEOUT-MS INTERVAL-MS MAX-POLLS after a command".into());}
    let value=scalar(&tokens[2])?;
    let number=|index:usize|tokens[index].parse::<u32>().map_err(|_|"Invalid wait limit".to_string());
    let w=Wait{after_step:(after_step-1) as u32,pointer:tokens[1].clone(),expected:value,timeout_ms:number(3)?,interval_ms:number(4)?,max_polls:number(5)?};
    if !valid_wait(&w){return Err("Invalid bounded wait".into());}Ok(Some(w))
}
fn valid_wait(w:&Wait)->bool{valid_pointer(&w.pointer)&&valid_scalar(&w.expected)&&(20..=30_000).contains(&w.timeout_ms)&&(20..=1_000).contains(&w.interval_ms)&&(1..=128).contains(&w.max_polls)}
fn assertion(command:&str,after_step:usize)->Result<Option<Assertion>,String>{
    let tokens=super::tokenize(command)?;if tokens.first().map(String::as_str)!=Some("assert"){return Ok(None);}
    if tokens.len()!=3 || after_step==0{return Err("Use assert JSON-POINTER SCALAR after a command".into());}
    let expected=scalar(&tokens[2])?;
    if !valid_pointer(&tokens[1]) || !valid_scalar(&expected){return Err("Invalid bounded assertion pointer or scalar".into());}
    Ok(Some(Assertion{after_step:(after_step-1) as u32,pointer:tokens[1].clone(),expected}))
}

fn hash_valid(s:&str)->bool{s.len()==64 && s.bytes().all(|c|c.is_ascii_digit() || (b'a'..=b'f').contains(&c))}
fn validate_action(action:&Action)->Result<(),String>{
    action.validate()?;
    if matches!(action,Action::Resume|Action::Cancel{..}){return Err("Recipes cannot contain unbounded resume or cancellation commands".into());}
    Ok(())
}
fn search_frame_cost(action:&Action)->u32 {
    match action {
        Action::Find{settings} => crate::debug_probe::MAX_SEARCH_FRAMES as u32 + u32::from(settings.on_hit == Some(crate::debug_probe::HitAction::Capture)),
        _ => 0,
    }
}
fn replay_frame_cost(action:&Action)->Result<u32,String>{
    match action {
        Action::Replay{mode,max_frames:Some(frames)}=>frames.checked_mul(if matches!(mode,crate::debug_control::ReplayMode::Comparison){2}else{1}).ok_or("Replay frame budget overflow".into()),
        Action::Replay{max_frames:None,..}=>Err("Recipe replay requires an explicit --max-frames limit".into()),
        _=>Ok(0),
    }
}
fn replay_frame_budget(steps:&[Action])->Result<u32,String>{steps.iter().try_fold(0u32,|sum,action|sum.checked_add(replay_frame_cost(action)?).ok_or("Replay frame budget overflow".into()))}
impl Recipe {
    fn validate_run_frame_budget(&self)->Result<(),String>{
        let search=self.steps.iter().map(search_frame_cost).sum::<u32>();
        let replay=replay_frame_budget(&self.steps)?;
        if self.limits.explicit_step_frames.saturating_add(search).saturating_add(replay)>self.limits.max_frames{return Err("Combined step, search and replay frame budget exceeds 3600; no action sent".into());}Ok(())
    }
    fn validate(&self)->Result<(),String>{
        if self.schema!=1 || self.kind!="ffta-debug-command-recipe" || !crate::debug_cheat_preset::name_valid(&self.name){return Err("Invalid command recipe schema or name".into());}
        if ![&self.manifest_sha256,&self.rom_sha256,&self.runtime_sha256].iter().all(|v|hash_valid(v)){return Err("Recipe requires exact lowercase SHA-256 pins".into());}
        if self.steps.is_empty() || self.steps.len()>MAX_STEPS || self.limits.steps!=self.steps.len() as u32 || self.limits.explicit_step_frames>MAX_FRAMES || self.limits.max_frames!=MAX_FRAMES || self.limits.duration_ms!=120_000 || self.limits.output_bytes!=1024*1024{return Err("Invalid recipe limits".into());}
        let (mut frames,mut search_frames)=(0u32,0u32);
        for action in &self.steps {validate_action(action)?;match action{Action::Step{frames:n,..}=>frames=frames.checked_add(*n).ok_or("Frame limit overflow")?,Action::Find{..}=>search_frames=search_frames.checked_add(search_frame_cost(action)).ok_or("Search frame budget overflow")?,_=>{}}}
        if let Some(budget)=self.limits.search_frame_budget {if budget!=search_frames || frames.saturating_add(budget)>MAX_FRAMES{return Err("Invalid aggregate search frame budget".into());}}
        if let Some(budget)=self.limits.replay_frame_budget {if replay_frame_budget(&self.steps).ok()!=Some(budget) || frames.saturating_add(search_frames).saturating_add(budget)>MAX_FRAMES{return Err("Invalid aggregate replay frame budget".into());}}
        if self.steps.len()+self.assertions.len()+self.waits.len()>MAX_STEPS || self.assertions.iter().any(|a|a.after_step as usize>=self.steps.len()||!valid_pointer(&a.pointer)||!valid_scalar(&a.expected)) || self.assertions.windows(2).any(|w|w[0].after_step>w[1].after_step){return Err("Invalid recipe assertions".into());}
        if self.waits.iter().any(|w|w.after_step as usize>=self.steps.len()||!valid_wait(w)) || self.waits.windows(2).any(|w|w[0].after_step>=w[1].after_step){return Err("Invalid recipe waits".into());}
        if frames!=self.limits.explicit_step_frames{return Err("Recipe explicit frame count does not match its steps".into());}Ok(())
    }
}
pub(super) fn manifest(root:&Path)->Result<(String,String,String),String>{
    for path in [root.to_path_buf(),root.join("debug-session.json")] {crate::debug_session::regular(&path)?;}
    let bytes=crate::diagnostics::read(&root.join("debug-session.json"),1024*1024)?;
    let value:Value=serde_json::from_slice(&bytes).map_err(|e|e.to_string())?;
    if value["schema"]!=1 || value["kind"]!="ffta-debug-session"{return Err("Choose a compatible isolated session".into());}
    let rom=value["pins"]["ffta.gba"].as_str().ok_or("Session ROM pin missing")?;
    let runtime=value["pins"]["recomp.exe"].as_str().or_else(||value["pins"]["recomp"].as_str()).ok_or("Session runtime pin missing")?;
    if !hash_valid(rom)||!hash_valid(runtime){return Err("Session ROM or runtime pin is invalid".into());}
    Ok((gba_core::ffta_rom::digest(&bytes),rom.into(),runtime.into()))
}
fn directory(root:&Path,create:bool)->Result<PathBuf,String>{
    for path in [root.to_path_buf(),root.join("Data"),root.join("Data/config")] {if !crate::debug_session::regular(&path)?.is_dir(){return Err("Session configuration root is not a directory".into());}}
    let path=root.join("Data/config/debug-command-recipes");
    if create {match fs::create_dir(&path){Ok(())=>(),Err(e) if e.kind()==std::io::ErrorKind::AlreadyExists=>(),Err(e)=>return Err(e.to_string())}}
    if !crate::debug_session::regular(&path)?.is_dir(){return Err("Recipe path is not a directory".into());}Ok(path)
}
fn path(dir:&Path,name:&str)->Result<PathBuf,String>{
    if !crate::debug_cheat_preset::name_valid(name){return Err("Recipe name must contain 1..48 ASCII letters, digits, '-' or '_'".into());}
    Ok(dir.join(format!("recipe-{name}.json")))
}
fn action(command:&str)->Result<Action,String>{
    let action=match parse(command)? {
        Command::Control(action)=>action,
        Command::JobsStatus=>Action::Status,
        Command::Select{slot,id}=>Action::SelectId{slot:match slot{menu::ContentSlot::Replay=>SelectionSlot::Replay,menu::ContentSlot::Catalog=>SelectionSlot::Catalog,menu::ContentSlot::CheatCatalog=>SelectionSlot::CheatCatalog,_=>return Err("Unsupported selection slot".into())},id},
        _=>return Err("Recipe steps must be typed commands, not help, retry or another recipe".into()),
    };
    validate_action(&action)?;Ok(action)
}
fn save(root:&Path,name:&str,commands:&[String])->Result<Recipe,String>{
    if !crate::debug_cheat_preset::name_valid(name){return Err("Invalid recipe name".into());}
    if !(1..=MAX_STEPS).contains(&commands.len()){return Err("Recipe needs 1..16 commands".into());}
    let mut steps=Vec::new();let mut assertions=Vec::new();let mut waits=Vec::new();
    for command in commands {if let Some(check)=assertion(command,steps.len())?{assertions.push(check);}else if let Some(condition)=wait(command,steps.len())?{waits.push(condition);}else{steps.push(action(command)?);}}
    let frames=steps.iter().try_fold(0u32,|sum,a|sum.checked_add(if let Action::Step{frames,..}=a{*frames}else{0}).ok_or("Frame limit overflow"))?;
    let search_frames=steps.iter().map(search_frame_cost).sum::<u32>();
    let replay_frames=replay_frame_budget(&steps)?;
    if frames.saturating_add(search_frames).saturating_add(replay_frames)>MAX_FRAMES{return Err("Combined step, search and replay frame budget exceeds 3600".into());}
    let (manifest_sha256,rom_sha256,runtime_sha256)=manifest(root)?;
    let recipe=Recipe{schema:1,kind:"ffta-debug-command-recipe".into(),name:name.into(),manifest_sha256,rom_sha256,runtime_sha256,limits:Limits{steps:steps.len() as u32,explicit_step_frames:frames,max_frames:MAX_FRAMES,duration_ms:120_000,output_bytes:1024*1024,search_frame_budget:(search_frames>0).then_some(search_frames),replay_frame_budget:(replay_frames>0).then_some(replay_frames)},steps,assertions,waits};
    recipe.validate()?;let bytes=serde_json::to_vec_pretty(&recipe).map_err(|e|e.to_string())?;
    if bytes.len() as u64>MAX_BYTES{return Err("Recipe exceeds 64 KiB".into());}
    let dir=directory(root,true)?;
    let lock_path=dir.join(".recipe.lock");
    if lock_path.exists(){crate::debug_session::regular(&lock_path)?;}
    let lock=OpenOptions::new().read(true).write(true).create(true).open(&lock_path).map_err(|e|e.to_string())?;
    crate::debug_session::regular(&lock_path)?;
    lock.try_lock().map_err(|_|"Another recipe save is in progress")?;
    let entries=fs::read_dir(&dir).map_err(|e|e.to_string())?.filter_map(|e|match e{Ok(e) if e.file_name()==".recipe.lock"=>None,other=>Some(other)}).take(MAX_RECIPES).collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
    if entries.len()>=MAX_RECIPES{return Err("Recipe directory is full".into());}
    let mut temporary=tempfile::NamedTempFile::new_in(&dir).map_err(|e|e.to_string())?;
    temporary.write_all(&bytes).and_then(|_|temporary.as_file().sync_all()).map_err(|e|e.to_string())?;
    temporary.persist_noclobber(path(&dir,name)?).map_err(|e|e.error.to_string())?;Ok(recipe)
}
fn load(root:&Path,name:&str)->Result<Recipe,String>{
    let path=path(&directory(root,false)?,name)?;let recipe=read_recipe(&path)?;
    if recipe.name!=name{return Err("Recipe name differs from requested name".into());}
    verify_session(&recipe,root)?;Ok(recipe)
}
fn read_recipe(path:&Path)->Result<Recipe,String>{
    if !crate::debug_session::regular(path)?.is_file(){return Err("Recipe must be an ordinary file".into());}
    let recipe:Recipe=serde_json::from_slice(&crate::diagnostics::read(path,MAX_BYTES)?).map_err(|e|e.to_string())?;
    recipe.validate()?;Ok(recipe)
}
fn verify_session(recipe:&Recipe,root:&Path)->Result<(),String>{
    let (identity,rom,runtime)=manifest(root)?;
    if recipe.manifest_sha256!=identity || recipe.rom_sha256!=rom || recipe.runtime_sha256!=runtime{return Err("Session pins changed; recipe requires its original session".into());}Ok(())
}
/// Identify the distinct command format without interpreting its steps.
pub(crate) fn is_command_recipe(path:&Path)->Result<bool,String>{
    if !crate::debug_session::regular(path)?.is_file(){return Err("Recipe must be an ordinary file".into());}
    let value:Value=serde_json::from_slice(&crate::diagnostics::read(path,MAX_BYTES)?).map_err(|e|e.to_string())?;
    Ok(value["kind"]=="ffta-debug-command-recipe")
}
pub(crate) fn validate_file(path:&Path)->Result<(),String>{read_recipe(path).map(|_|())}
pub(crate) fn review_file(path:&Path,root:&Path)->Result<String,String>{review_with_identity(path,root).map(|(text,_)|text)}
pub(crate) fn review_with_identity(path:&Path,root:&Path)->Result<(String,String),String>{
    let (recipe,identity,evidence,_)=reviewed(path,root)?;
    let full=serde_json::to_string_pretty(&recipe).map_err(|e|e.to_string())?;
    if full.len() as u64>MAX_BYTES{return Err("Complete preview exceeds 64 KiB; use a smaller recipe".into());}
    Ok((format!("Typed command recipe preview; no action executed. Start requires this unchanged preview. Focus Details and use Left/Right to scroll, or use the mouse wheel.\n\nConcluded matching evidence: {evidence}\n\n{full}"),identity))
}
/// Bind the reviewed sequence to both selected paths and the session manifest.
fn reviewed(path:&Path,root:&Path)->Result<(Recipe,String,Value,String),String>{
    let recipe=read_recipe(path)?;verify_session(&recipe,root)?;
    let selected=fs::canonicalize(path).map_err(|e|e.to_string())?;
    let session=fs::canonicalize(root).map_err(|e|e.to_string())?;
    let (evidence,evidence_hash)=checkpoint::review_evidence(root,&recipe)?;
    let bytes=serde_json::to_vec(&(selected,session,&recipe,&evidence_hash)).map_err(|e|e.to_string())?;
    Ok((recipe,gba_core::ffta_rom::digest(&bytes),evidence,evidence_hash))
}
#[cfg(test)]
pub(crate) fn run_reviewed(path:&Path,root:&Path,identity:&str,cancel:&std::sync::atomic::AtomicBool)->Result<Value,String>{
    run_reviewed_with_reason(path,root,identity,cancel,None)
}
pub(crate) fn run_reviewed_with_reason(path:&Path,root:&Path,identity:&str,cancel:&std::sync::atomic::AtomicBool,reason:Option<&str>)->Result<Value,String>{
    let (selected,current,_,evidence_hash)=reviewed(path,root)?;
    if current!=identity{return Err("Recipe or session changed since Details; review it again".into());}
    let expected=gba_core::ffta_rom::digest(&serde_json::to_vec(&selected).map_err(|e|e.to_string())?);
    let stored=load(root,&selected.name)?;
    if gba_core::ffta_rom::digest(&serde_json::to_vec(&stored).map_err(|e|e.to_string())?)!=expected{return Err("The session's saved recipe differs from the reviewed selection".into());}
    checkpoint::run_reviewed(root,&selected.name,&expected,cancel,reason,&evidence_hash)
}
fn result(value:Result<Recipe,String>,operation:&str)->Value{
    match value {Ok(recipe)=>json!({"schema":1,"kind":"ffta-debug-command-recipe-preview","ok":true,"outcome":operation,"recipe":recipe,"scope":"Saving and preview never execute. The runner supports bounded frontend commands and replay jobs with an explicit frame cap."}),Err(error)=>super::local_error("invalid-request",&error)}
}
pub(super) fn save_command(root:&Path,name:&str,commands:&[String])->Value{result(save(root,name,commands),"saved")}
pub(super) fn preview_command(root:&Path,name:&str)->Value{
    match load(root,name).and_then(|recipe|{let (evidence,_)=checkpoint::review_evidence(root,&recipe)?;Ok((recipe,evidence))}){
        Ok((recipe,evidence))=>{let mut value=result(Ok(recipe),"preview");value["concluded_evidence"]=evidence;value},
        Err(error)=>super::local_error("invalid-request",&error),
    }
}
pub(super) fn prepare_command(root:&Path,name:&str,reason:Option<&str>)->Value{checkpoint::prepare_command(root,name,reason)}
pub(super) fn conclude_command(root:&Path,id:&str)->Value{checkpoint::conclude_command(root,id)}
pub(super) fn checkpoint_command(root:&Path,id:&str)->Value{checkpoint::inspect_command(root,id)}
pub(super) fn run_command(root:&Path,id:&str,resume:bool)->Value{checkpoint::run_command(root,id,resume)}
pub(super) fn recover_command(root:&Path,id:&str)->Value{checkpoint::recover_command(root,id)}
pub(super) fn cancel_command(root:&Path,id:&str)->Value{checkpoint::cancel_command(root,id)}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture()->tempfile::TempDir{
        let t=tempfile::tempdir().unwrap();fs::create_dir_all(t.path().join("Data/config")).unwrap();
        fs::write(t.path().join("debug-session.json"),serde_json::to_vec(&json!({"schema":1,"kind":"ffta-debug-session","pins":{"ffta.gba":"a".repeat(64),"recomp.exe":"b".repeat(64)}})).unwrap()).unwrap();t
    }
    #[test]
    fn saves_typed_sequence_without_executing_and_rechecks_pins(){
        let t=fixture();let commands=vec!["status".into(),"step 3 --keys 0x3ff".into()];
        let saved=super::super::execute(t.path(),"recipe save first 'status' 'step 3 --keys 0x3ff'");assert_eq!(saved["ok"],true);assert_eq!(saved["recipe"]["steps"][1]["action"],"step");assert_eq!(saved["recipe"]["limits"]["explicit_step_frames"],3);assert_eq!(saved["recipe"]["limits"]["max_frames"],MAX_FRAMES);
        assert_eq!(super::super::execute(t.path(),"recipe preview first")["recipe"],saved["recipe"]);assert_eq!(save_command(t.path(),"first",&commands)["ok"],false);
        assert!(!t.path().join("Data/config/debug-control").exists());
        let path=t.path().join("Data/config/debug-command-recipes/recipe-first.json");assert!(review_file(&path,t.path()).unwrap().contains("Start requires"));
        fs::write(t.path().join("debug-session.json"),b"{}").unwrap();assert_eq!(preview_command(t.path(),"first")["ok"],false);assert!(review_file(&path,t.path()).is_err());
    }
    #[test]
    fn rejects_unbounded_invalid_and_nested_steps_before_creating_directory(){
        let t=fixture();for command in ["resume","jobs cancel another","step 601","recipe preview nested","help","assert /status/paused true"]{assert_eq!(save_command(t.path(),"bad",&[command.into()])["ok"],false);}
        assert_eq!(save_command(t.path(),"../bad",&["status".into()])["ok"],false);
        assert_eq!(save_command(t.path(),"many",&vec!["status".into();17])["ok"],false);
        assert_eq!(save_command(t.path(),"frames",&vec!["step 600".into();7])["ok"],false);
        assert!(!t.path().join("Data/config/debug-command-recipes").exists());
    }
    #[test]
    fn assertions_are_bounded_typed_and_attached_to_previous_action(){
        let t=fixture();let commands=vec!["status".into(),"assert /status/paused true".into(),"assert /status/backend text:interp".into(),"step 1".into(),"assert /status/frame 7".into()];
        let value=save_command(t.path(),"checks",&commands);assert_eq!(value["ok"],true);assert_eq!(value["recipe"]["steps"].as_array().unwrap().len(),2);assert_eq!(value["recipe"]["assertions"][0]["expected"],true);assert_eq!(value["recipe"]["assertions"][1]["expected"],"interp");assert_eq!(value["recipe"]["assertions"][2]["after_step"],1);
        for bad in ["assert missing true","assert /bad~2 true","assert /status/value {}","assert /status/value 1.5","assert /status/value not-a-scalar"]{assert_eq!(save_command(t.path(),"bad",&["status".into(),bad.into()])["ok"],false);}
    }
    #[test]
    fn long_tool_preview_keeps_all_actions(){
        let t=fixture();let commands=vec![format!("profile select {}","x".repeat(512));16];
        assert_eq!(save_command(t.path(),"long",&commands)["ok"],true);
        let path=t.path().join("Data/config/debug-command-recipes/recipe-long.json");
        let review=review_file(&path,t.path()).unwrap();assert_eq!(review.matches(&"x".repeat(512)).count(),16);
        let preview=preview_command(t.path(),"long");assert_eq!(preview["recipe"]["steps"].as_array().unwrap().len(),16);
        assert_eq!(preview["recipe"]["steps"][15]["id"],"x".repeat(512));
    }
    #[test]
    fn wait_limits_are_explicit_and_legacy_serialization_is_preserved(){
        let t=fixture();let value=save_command(t.path(),"waited",&["step 1".into(),"wait /status/paused true 1000 20 4".into()]);assert_eq!(value["ok"],true);assert_eq!(value["recipe"]["waits"][0]["max_polls"],4);
        for bad in ["wait /status/x true 1000 20 0","wait /status/x true 1000 1 4","wait /status/x true 30001 20 4","wait /status/x true 1000 20 129","wait /bad~2 true 1000 20 4"]{assert_eq!(save_command(t.path(),"bad",&["status".into(),bad.into()])["ok"],false);}
        assert_eq!(save_command(t.path(),"duplicate",&["status".into(),"wait /status/x true 1000 20 1".into(),"wait /status/y true 1000 20 1".into()])["ok"],false);
        let legacy=save_command(t.path(),"legacy",&["status".into()]);assert!(legacy["recipe"].get("waits").is_none());
    }

    #[test]
    fn changed_recipe_cannot_use_an_older_review_identity(){
        let t=fixture();assert_eq!(save_command(t.path(),"edited",&["step 1".into()])["ok"],true);
        let file=t.path().join("Data/config/debug-command-recipes/recipe-edited.json");let (_,identity)=review_with_identity(&file,t.path()).unwrap();
        let mut value:Value=serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();value["steps"][0]["frames"]=json!(2);value["limits"]["explicit_step_frames"]=json!(2);fs::write(&file,serde_json::to_vec(&value).unwrap()).unwrap();
        let err=run_reviewed(&file,t.path(),&identity,&std::sync::atomic::AtomicBool::new(false)).unwrap_err();assert!(err.contains("changed since Details"));assert!(!t.path().join("Data/config/debug-command-runs").exists());
    }

    #[test]
    fn search_budget_counts_worst_case_frames_and_keeps_legacy_identity(){
        let t=fixture();let cmd="break 0x08000000 arm --steps 1".to_string();let value=save_command(t.path(),"search",&[cmd.clone(),"step 1".into()]);assert_eq!(value["ok"],true);assert_eq!(value["recipe"]["limits"]["search_frame_budget"],600);
        assert_eq!(save_command(t.path(),"too-many",&vec![cmd.clone();7])["ok"],false);
        assert_eq!(save_command(t.path(),"mixed",&[cmd.clone(),"step 600".into(),"step 600".into(),"step 600".into(),"step 600".into(),"step 600".into(),"step 1".into()])["ok"],false);
        let capture="break 0x08000000 arm --steps 1 --on-hit capture".to_string();
        let captured=save_command(t.path(),"capture-search",&[capture.clone()]);assert_eq!(captured["recipe"]["limits"]["search_frame_budget"],601);
        assert_eq!(save_command(t.path(),"capture-over-budget",&vec![capture;6])["ok"],false);
        let file=t.path().join("Data/config/debug-command-recipes/recipe-search.json");let mut legacy=value["recipe"].clone();legacy["limits"].as_object_mut().unwrap().remove("search_frame_budget");fs::write(&file,serde_json::to_vec(&legacy).unwrap()).unwrap();let loaded=load(t.path(),"search").unwrap();assert!(loaded.limits.search_frame_budget.is_none());assert!(loaded.validate_run_frame_budget().is_ok());assert_eq!(serde_json::to_value(loaded).unwrap(),legacy);
        let mut too_many=legacy.clone();too_many["steps"]=json!(vec![legacy["steps"][0].clone();7]);too_many["limits"]["steps"]=json!(7);too_many["limits"]["explicit_step_frames"]=json!(0);fs::write(&file,serde_json::to_vec(&too_many).unwrap()).unwrap();let loaded=load(t.path(),"search").unwrap();assert!(loaded.validate_run_frame_budget().is_err());
    }
    #[test]
    fn replay_budget_is_explicit_aggregated_and_preserves_legacy_refusal(){
        let t=fixture();let value=save_command(t.path(),"bounded",&["replay run plain --max-frames 10".into(),"step 1".into()]);assert_eq!(value["ok"],true);assert_eq!(value["recipe"]["limits"]["replay_frame_budget"],10);
        assert_eq!(save_command(t.path(),"compare",&["replay run compare --max-frames 1800".into()])["recipe"]["limits"]["replay_frame_budget"],3600);
        assert_eq!(save_command(t.path(),"too-many",&["replay run compare --max-frames 1800".into(),"step 1".into()])["ok"],false);
        let search="break 0x08000000 arm --steps 1".to_string();assert_eq!(save_command(t.path(),"combined",&[search.clone(),"replay run --max-frames 3000".into()])["ok"],true);
        assert_eq!(save_command(t.path(),"combined-over",&[search,"replay run --max-frames 3000".into(),"step 1".into()])["ok"],false);
        assert_eq!(save_command(t.path(),"unbounded",&["replay run".into()])["ok"],false);
        assert_eq!(save_command(t.path(),"zero",&["replay run --max-frames 0".into()])["ok"],false);
        let legacy=save_command(t.path(),"legacy-replay",&["status".into()]);let mut old=legacy["recipe"].clone();old["steps"][0]=json!({"action":"replay","mode":"plain"});
        let file=t.path().join("Data/config/debug-command-recipes/recipe-legacy-replay.json");fs::write(&file,serde_json::to_vec(&old).unwrap()).unwrap();let loaded=load(t.path(),"legacy-replay").unwrap();assert_eq!(serde_json::to_value(&loaded).unwrap(),old);assert!(loaded.validate_run_frame_budget().unwrap_err().contains("--max-frames"));
    }

}
