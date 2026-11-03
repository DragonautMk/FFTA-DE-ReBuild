//! Durable recipe checkpoints. Preparing never publishes a request.
use super::{directory,load,manifest,Recipe};
mod runner;
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use std::{fs::{self,OpenOptions},io::Write,path::{Path,PathBuf}};

const MAX_RUNS:usize=32;
const MAX_CHECKPOINT:u64=1024*1024;
#[derive(Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Checkpoint {
    schema:u32,kind:String,id:String,recipe_name:String,recipe_sha256:String,
    manifest_sha256:String,session_run:String,steps:Vec<PlannedStep>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    deadline_unix_ms:Option<u64>,
}
#[derive(Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct PlannedStep {
    index:u32,request_id:String,action_sha256:String,state:String,
    #[serde(default,skip_serializing_if="Option::is_none")]
    deadline_unix_ms:Option<u64>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    result:Option<Value>,
    #[serde(default,skip_serializing_if="Vec::is_empty")]
    checks:Vec<AssertionCheck>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    wait_deadline_unix_ms:Option<u64>,
    #[serde(default,skip_serializing_if="is_zero")]
    wait_polls:u32,
    #[serde(default,skip_serializing_if="Option::is_none")]
    wait_observation:Option<WaitObservation>,
}
fn is_zero(n:&u32)->bool{*n==0}
#[derive(Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct WaitObservation {poll:u32,id:String,response_sha256:String,present:bool,actual:Value,matched:bool}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct AssertionCheck {pointer:String,matched:bool}
fn assertion_checks(recipe:&Recipe,index:usize,result:&Value)->Vec<AssertionCheck>{
    recipe.assertions.iter().filter(|a|a.after_step as usize==index).map(|a|AssertionCheck{pointer:a.pointer.clone(),matched:result.pointer(&a.pointer).is_some_and(|actual|actual==&a.expected)}).collect()
}
fn digest(data:&[u8])->String{gba_core::ffta_rom::digest(data)}
fn recipe_hash(recipe:&Recipe)->Result<String,String>{Ok(digest(&serde_json::to_vec(recipe).map_err(|e|e.to_string())?))}
fn action_hash(action:&crate::debug_control::Action)->Result<String,String>{Ok(digest(&serde_json::to_vec(action).map_err(|e|e.to_string())?))}
fn request_id(id:&str,run:&str,index:usize,action:&str)->Result<String,String>{
    Ok(format!("recipe-{}",digest(&serde_json::to_vec(&("ffta-recipe-step-v1",id,run,index,action)).map_err(|e|e.to_string())?)))
}
fn valid_id(id:&str)->bool{id.starts_with("run-") && crate::debug_cheat_preset::name_valid(id)}
fn active_run(root:&Path,manifest_sha:&str)->Result<String,String>{
    let dir=root.join("Data/config/debug-control");if !crate::debug_session::regular(&dir)?.is_dir(){return Err("No active control directory".into());}
    let file=dir.join("ready.json");if !crate::debug_session::regular(&file)?.is_file(){return Err("Control readiness must be a regular file".into());}
    let ready:Value=serde_json::from_slice(&crate::diagnostics::read(&file,128*1024)?).map_err(|e|e.to_string())?;
    if ready["schema"]!=1 || ready["kind"]!="ffta-debug-control" || ready["active"]!=true || ready["session"]["manifest_sha256"]!=manifest_sha{return Err("Active control session identity does not match the recipe".into());}
    if ready["capabilities"].as_array().is_none_or(|c|!c.contains(&json!("idempotent-request-v1"))){return Err("Active control session does not support safe request identity".into());}
    let run=ready["run"].as_str().filter(|s|!s.is_empty()&&s.len()<=100&&!s.chars().any(char::is_control)).ok_or("Invalid control run identity")?;
    Ok(run.into())
}
fn base(root:&Path,create:bool)->Result<PathBuf,String>{
    // Existing recipe storage establishes the same isolated configuration root.
    directory(root,false)?;
    let path=root.join("Data/config/debug-command-runs");
    if create {match fs::create_dir(&path){Ok(())=>(),Err(e) if e.kind()==std::io::ErrorKind::AlreadyExists=>(),Err(e)=>return Err(e.to_string())}}
    if !crate::debug_session::regular(&path)?.is_dir(){return Err("Recipe run path is not a directory".into());}Ok(path)
}
fn validate(checkpoint:&Checkpoint,recipe:&Recipe)->Result<(),String>{
    if checkpoint.schema!=1 || checkpoint.kind!="ffta-debug-command-checkpoint" || !valid_id(&checkpoint.id) || checkpoint.recipe_name!=recipe.name || checkpoint.manifest_sha256!=recipe.manifest_sha256 || checkpoint.recipe_sha256!=recipe_hash(recipe)? || checkpoint.session_run.is_empty() || checkpoint.session_run.len()>100 || checkpoint.session_run.chars().any(char::is_control) || checkpoint.steps.len()!=recipe.steps.len(){return Err("Recipe checkpoint identity differs from the selected recipe".into());}
    let mut unfinished=false;
    for (i,(step,action)) in checkpoint.steps.iter().zip(&recipe.steps).enumerate(){
        let hash=action_hash(action)?;
        if step.index!=i as u32 || step.action_sha256!=hash || step.request_id!=request_id(&checkpoint.id,&checkpoint.session_run,i,&hash)? {return Err("Recipe checkpoint step identity is invalid".into());}
        let terminal=if let (Some(deadline),Some(value))=(step.deadline_unix_ms,step.result.as_ref()){
            let fingerprint=crate::debug_control::planned_fingerprint(action.clone(),&checkpoint.session_run,&step.request_id,deadline,&checkpoint.manifest_sha256)?;
            value["run"]==checkpoint.session_run && value["id"]==step.request_id && value["deadline_unix_ms"]==deadline && value["request_sha256"]==fingerprint
        }else{false};
        let checks=step.result.as_ref().map(|r|assertion_checks(recipe,i,r)).unwrap_or_default();
        let wait=recipe.waits.iter().find(|w|w.after_step as usize==i);
        let no_wait=step.wait_deadline_unix_ms.is_none() && step.wait_polls==0 && step.wait_observation.is_none();
        let valid_wait=wait.is_some_and(|w|step.wait_deadline_unix_ms.is_some_and(|d|d<=checkpoint.deadline_unix_ms.unwrap_or(0)) && step.wait_polls<=w.max_polls && step.wait_observation.as_ref().is_none_or(|o|o.poll>0 && o.poll<=step.wait_polls && !o.id.is_empty() && o.id.len()<=100 && super::hash_valid(&o.response_sha256) && super::valid_scalar(&o.actual) && o.matched==(o.present && o.actual==w.expected)));
        let success_wait=if wait.is_some(){valid_wait && step.wait_observation.as_ref().is_some_and(|o|o.matched)}else{no_wait};
        match step.state.as_str(){
            "completed" if !unfinished && step.deadline_unix_ms==checkpoint.deadline_unix_ms && terminal && step.result.as_ref().is_some_and(|r|r["ok"]==true && r["kind"]=="ffta-debug-control-result") && step.checks==checks && checks.iter().all(|c|c.matched) && success_wait=>{},
            "assertion-failed" if !unfinished && step.deadline_unix_ms==checkpoint.deadline_unix_ms && terminal && step.result.as_ref().is_some_and(|r|r["ok"]==true && r["kind"]=="ffta-debug-control-result") && step.checks==checks && checks.iter().any(|c|!c.matched) && no_wait=>unfinished=true,
            "waiting" | "wait-timeout" if !unfinished && step.deadline_unix_ms==checkpoint.deadline_unix_ms && terminal && step.result.as_ref().is_some_and(|r|r["ok"]==true && r["kind"]=="ffta-debug-control-result") && step.checks==checks && checks.iter().all(|c|c.matched) && valid_wait && step.wait_observation.as_ref().is_none_or(|o|!o.matched)=>unfinished=true,
            "pending" if !unfinished && step.deadline_unix_ms==checkpoint.deadline_unix_ms && step.deadline_unix_ms.is_some() && step.result.is_none() && step.checks.is_empty() && no_wait=>unfinished=true,
            "failed" if !unfinished && step.deadline_unix_ms==checkpoint.deadline_unix_ms && terminal && step.result.as_ref().is_some_and(|r|r["ok"]==false) && step.checks.is_empty() && no_wait=>unfinished=true,
            "planned" if step.deadline_unix_ms.is_none() && step.result.is_none() && step.checks.is_empty() && no_wait=>unfinished=true,
            _=>return Err("Recipe checkpoint step order or state is invalid".into()),
        }
    }
    Ok(())
}
fn prepare(root:&Path,name:&str)->Result<Checkpoint,String>{
    let recipe=load(root,name)?;let session_run=active_run(root,&recipe.manifest_sha256)?;
    let base=base(root,true)?;let lock_path=base.join(".run.lock");
    if lock_path.exists(){crate::debug_session::regular(&lock_path)?;}
    let lock=OpenOptions::new().read(true).write(true).create(true).open(&lock_path).map_err(|e|e.to_string())?;crate::debug_session::regular(&lock_path)?;lock.try_lock().map_err(|_|"Another recipe preparation is in progress")?;
    let entries=fs::read_dir(&base).map_err(|e|e.to_string())?.filter_map(|e|match e{Ok(e) if e.file_name()==".run.lock"=>None,other=>Some(other)}).take(MAX_RUNS).collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
    if entries.len()>=MAX_RUNS{return Err("Recipe run directory is full".into());}
    let temp=tempfile::Builder::new().prefix("run-").rand_bytes(16).tempdir_in(&base).map_err(|e|e.to_string())?;
    let id=temp.path().file_name().and_then(|s|s.to_str()).filter(|s|valid_id(s)).ok_or("Invalid generated run ID")?.to_owned();
    let mut steps=Vec::with_capacity(recipe.steps.len());
    for (i,action) in recipe.steps.iter().enumerate(){let hash=action_hash(action)?;steps.push(PlannedStep{index:i as u32,request_id:request_id(&id,&session_run,i,&hash)?,action_sha256:hash,state:"planned".into(),deadline_unix_ms:None,result:None,checks:Vec::new(),wait_deadline_unix_ms:None,wait_polls:0,wait_observation:None});}
    let checkpoint=Checkpoint{schema:1,kind:"ffta-debug-command-checkpoint".into(),id,recipe_name:name.into(),recipe_sha256:recipe_hash(&recipe)?,manifest_sha256:recipe.manifest_sha256.clone(),session_run,steps,deadline_unix_ms:None};
    validate(&checkpoint,&recipe)?;let bytes=serde_json::to_vec_pretty(&checkpoint).map_err(|e|e.to_string())?;if bytes.len() as u64>MAX_CHECKPOINT{return Err("Checkpoint exceeds 64 KiB".into());}
    let file=temp.path().join("checkpoint.json");let mut out=OpenOptions::new().write(true).create_new(true).open(&file).map_err(|e|e.to_string())?;out.write_all(&bytes).and_then(|_|out.sync_all()).map_err(|e|e.to_string())?;drop(out);
    let _kept=temp.keep();Ok(checkpoint)
}
fn inspect(root:&Path,id:&str)->Result<(Checkpoint,bool),String>{
    if !valid_id(id){return Err("Invalid recipe run ID".into());}
    let dir=base(root,false)?.join(id);if !crate::debug_session::regular(&dir)?.is_dir(){return Err("Recipe run must be a directory".into());}
    let file=dir.join("checkpoint.json");if !crate::debug_session::regular(&file)?.is_file(){return Err("Checkpoint must be a regular file".into());}
    let checkpoint:Checkpoint=serde_json::from_slice(&crate::diagnostics::read(&file,MAX_CHECKPOINT)?).map_err(|e|e.to_string())?;
    if checkpoint.id!=id{return Err("Checkpoint run ID differs from its directory".into());}
    let recipe=load(root,&checkpoint.recipe_name)?;validate(&checkpoint,&recipe)?;
    let (current,_,_)=manifest(root)?;let active=active_run(root,&current).is_ok_and(|r|r==checkpoint.session_run);
    Ok((checkpoint,active))
}
pub(super) fn run_reviewed(root:&Path,name:&str,expected:&str,cancel:&std::sync::atomic::AtomicBool)->Result<Value,String>{
    let checkpoint=prepare(root,name)?;
    if checkpoint.recipe_sha256!=expected{return Err("Saved recipe changed after preview; no action sent".into());}
    runner::drive_with_cancel(root,&checkpoint.id,false,Some(cancel)).map_err(|e|format!("Checkpoint {} was prepared: {e}",checkpoint.id))
}
pub(super) fn prepare_command(root:&Path,name:&str)->Value{
    match prepare(root,name){Ok(checkpoint)=>json!({"schema":1,"kind":"ffta-debug-command-checkpoint-result","ok":true,"outcome":"prepared","checkpoint":checkpoint,"scope":"Identities persisted; no request sent. Readiness metadata does not prove process liveness. Run is a separate explicit command."}),Err(e)=>super::super::local_error("invalid-request",&e)}
}
pub(super) fn inspect_command(root:&Path,id:&str)->Value{
    match inspect(root,id){Ok((checkpoint,active))=>json!({"schema":1,"kind":"ffta-debug-command-checkpoint-result","ok":true,"outcome":"inspected","checkpoint":checkpoint,"readiness_matches_run":active,"scope":"Readiness metadata does not prove process liveness."}),Err(e)=>super::super::local_error("invalid-request",&e)}
}
pub(super) fn run_command(root:&Path,id:&str,resume:bool)->Value{runner::command(root,id,resume)}
pub(super) fn cancel_command(root:&Path,id:&str)->Value{runner::cancel_command(root,id)}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture()->tempfile::TempDir{
        let t=tempfile::tempdir().unwrap();fs::create_dir_all(t.path().join("Data/config/debug-control")).unwrap();
        let m=json!({"schema":1,"kind":"ffta-debug-session","pins":{"ffta.gba":"a".repeat(64),"recomp.exe":"b".repeat(64)}});let b=serde_json::to_vec(&m).unwrap();fs::write(t.path().join("debug-session.json"),&b).unwrap();
        let ready=json!({"schema":1,"kind":"ffta-debug-control","active":true,"run":"session-1","session":{"manifest_sha256":digest(&b)},"capabilities":["idempotent-request-v1"]});fs::write(t.path().join("Data/config/debug-control/ready.json"),serde_json::to_vec(&ready).unwrap()).unwrap();
        assert!(super::super::save(t.path(),"sample",&["step 3".into(),"inspect 0x02000000 2".into()]).is_ok());t
    }
    #[test]
    fn persists_distinct_step_ids_and_never_dispatches(){
        let t=fixture();let response=crate::debug_command::execute(t.path(),"recipe prepare sample");assert_eq!(response["ok"],true);let id=response["checkpoint"]["id"].as_str().unwrap();assert_eq!(crate::debug_command::execute(t.path(),&format!("recipe checkpoint {id}"))["readiness_matches_run"],true);
        let a=inspect(t.path(),id).unwrap().0;let b=prepare(t.path(),"sample").unwrap();assert_ne!(a.id,b.id);assert_ne!(a.steps[0].request_id,a.steps[1].request_id);assert_ne!(a.steps[0].request_id,b.steps[0].request_id);
        let (loaded,active)=inspect(t.path(),&a.id).unwrap();assert!(active);assert_eq!(loaded.steps[0].request_id,a.steps[0].request_id);assert_eq!(loaded.steps[0].state,"planned");assert!(!t.path().join("Data/config/debug-control/request.json").exists());
    }
    #[test]
    fn changed_recipe_and_incompatible_or_closed_session_are_not_silently_accepted(){
        let t=fixture();let c=prepare(t.path(),"sample").unwrap();let ready=t.path().join("Data/config/debug-control/ready.json");let mut v:Value=serde_json::from_slice(&fs::read(&ready).unwrap()).unwrap();v["run"]=json!("session-2");fs::write(&ready,serde_json::to_vec(&v).unwrap()).unwrap();assert!(!inspect(t.path(),&c.id).unwrap().1);
        let path=t.path().join("Data/config/debug-command-recipes/recipe-sample.json");let mut r:Value=serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();r["steps"][0]["frames"]=json!(4);r["limits"]["explicit_step_frames"]=json!(4);fs::write(path,serde_json::to_vec(&r).unwrap()).unwrap();assert!(inspect(t.path(),&c.id).is_err());
        v["capabilities"]=json!([]);fs::write(ready,serde_json::to_vec(&v).unwrap()).unwrap();assert!(prepare(t.path(),"sample").is_err());
    }
    #[test]
    fn altered_step_identity_is_refused(){
        let t=fixture();let c=prepare(t.path(),"sample").unwrap();let file=t.path().join("Data/config/debug-command-runs").join(&c.id).join("checkpoint.json");
        let mut v:Value=serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();v["steps"][0]["request_id"]=json!("other");fs::write(file,serde_json::to_vec(&v).unwrap()).unwrap();assert!(inspect(t.path(),&c.id).is_err());
    }
    #[test]
    fn missing_pointer_is_not_equal_to_explicit_null(){
        let t=fixture();let recipe=super::super::save(t.path(),"null-check",&["status".into(),"assert /status/value null".into()]).unwrap();
        assert!(assertion_checks(&recipe,0,&json!({"status":{"value":null}}))[0].matched);
        assert!(!assertion_checks(&recipe,0,&json!({"status":{}}))[0].matched);
    }
}
