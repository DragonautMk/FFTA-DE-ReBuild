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
struct Limits {steps:u32,explicit_step_frames:u32,max_frames:u32,duration_ms:u32,output_bytes:u32}
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
impl Recipe {
    fn validate(&self)->Result<(),String>{
        if self.schema!=1 || self.kind!="ffta-debug-command-recipe" || !crate::debug_cheat_preset::name_valid(&self.name){return Err("Invalid command recipe schema or name".into());}
        if ![&self.manifest_sha256,&self.rom_sha256,&self.runtime_sha256].iter().all(|v|hash_valid(v)){return Err("Recipe requires exact lowercase SHA-256 pins".into());}
        if self.steps.is_empty() || self.steps.len()>MAX_STEPS || self.limits.steps!=self.steps.len() as u32 || self.limits.explicit_step_frames>MAX_FRAMES || self.limits.max_frames!=MAX_FRAMES || self.limits.duration_ms!=120_000 || self.limits.output_bytes!=1024*1024{return Err("Invalid recipe limits".into());}
        let mut frames=0u32;
        for action in &self.steps {validate_action(action)?;if let Action::Step{frames:n,..}=action{frames=frames.checked_add(*n).ok_or("Frame limit overflow")?;}}
        if self.steps.len()+self.assertions.len()+self.waits.len()>MAX_STEPS || self.assertions.iter().any(|a|a.after_step as usize>=self.steps.len()||!valid_pointer(&a.pointer)||!valid_scalar(&a.expected)) || self.assertions.windows(2).any(|w|w[0].after_step>w[1].after_step){return Err("Invalid recipe assertions".into());}
        if self.waits.iter().any(|w|w.after_step as usize>=self.steps.len()||!valid_wait(w)) || self.waits.windows(2).any(|w|w[0].after_step>=w[1].after_step){return Err("Invalid recipe waits".into());}
        if frames!=self.limits.explicit_step_frames{return Err("Recipe explicit frame count does not match its steps".into());}Ok(())
    }
}
fn manifest(root:&Path)->Result<(String,String,String),String>{
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
    let (manifest_sha256,rom_sha256,runtime_sha256)=manifest(root)?;
    let recipe=Recipe{schema:1,kind:"ffta-debug-command-recipe".into(),name:name.into(),manifest_sha256,rom_sha256,runtime_sha256,limits:Limits{steps:steps.len() as u32,explicit_step_frames:frames,max_frames:MAX_FRAMES,duration_ms:120_000,output_bytes:1024*1024},steps,assertions,waits};
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
    let (recipe,identity)=reviewed(path,root)?;
    let full=serde_json::to_string_pretty(&recipe).map_err(|e|e.to_string())?;
    if full.len() as u64>MAX_BYTES{return Err("Complete preview exceeds 64 KiB; use a smaller recipe".into());}
    Ok((format!("Typed command recipe preview; no action executed. Start requires this unchanged preview. Focus Details and use Left/Right to scroll, or use the mouse wheel.\n\n{full}"),identity))
}
/// Bind the reviewed sequence to both selected paths and the session manifest.
fn reviewed(path:&Path,root:&Path)->Result<(Recipe,String),String>{
    let recipe=read_recipe(path)?;verify_session(&recipe,root)?;
    let selected=fs::canonicalize(path).map_err(|e|e.to_string())?;
    let session=fs::canonicalize(root).map_err(|e|e.to_string())?;
    let bytes=serde_json::to_vec(&(selected,session,&recipe)).map_err(|e|e.to_string())?;
    Ok((recipe,gba_core::ffta_rom::digest(&bytes)))
}
pub(crate) fn run_reviewed(path:&Path,root:&Path,identity:&str,cancel:&std::sync::atomic::AtomicBool)->Result<Value,String>{
    let (selected,current)=reviewed(path,root)?;
    if current!=identity{return Err("Recipe or session changed since Details; review it again".into());}
    let expected=gba_core::ffta_rom::digest(&serde_json::to_vec(&selected).map_err(|e|e.to_string())?);
    let stored=load(root,&selected.name)?;
    if gba_core::ffta_rom::digest(&serde_json::to_vec(&stored).map_err(|e|e.to_string())?)!=expected{return Err("The session's saved recipe differs from the reviewed selection".into());}
    checkpoint::run_reviewed(root,&selected.name,&expected,cancel)
}
fn result(value:Result<Recipe,String>,operation:&str)->Value{
    match value {Ok(recipe)=>json!({"schema":1,"kind":"ffta-debug-command-recipe-preview","ok":true,"outcome":operation,"recipe":recipe,"scope":"Saving and preview never execute. The separate runner currently accepts only status, inspect and bounded step."}),Err(error)=>super::local_error("invalid-request",&error)}
}
pub(super) fn save_command(root:&Path,name:&str,commands:&[String])->Value{result(save(root,name,commands),"saved")}
pub(super) fn preview_command(root:&Path,name:&str)->Value{result(load(root,name),"preview")}
pub(super) fn prepare_command(root:&Path,name:&str)->Value{checkpoint::prepare_command(root,name)}
pub(super) fn checkpoint_command(root:&Path,id:&str)->Value{checkpoint::inspect_command(root,id)}
pub(super) fn run_command(root:&Path,id:&str,resume:bool)->Value{checkpoint::run_command(root,id,resume)}
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

}
