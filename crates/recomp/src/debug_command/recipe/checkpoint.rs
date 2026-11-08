//! Durable recipe checkpoints. Preparing never publishes a request.
use super::{directory,load,manifest,Recipe};
mod runner;
mod conclusion;
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use std::{fs::{self,OpenOptions},io::Write,path::{Path,PathBuf}};

const MAX_RUNS:usize=32;
const MAX_CHECKPOINT:u64=1024*1024;
#[derive(Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Checkpoint {
    schema:u32,kind:String,id:String,recipe_name:String,recipe_sha256:String,
    manifest_sha256:String,session_run:String,steps:Vec<PlannedStep>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    deadline_unix_ms:Option<u64>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    reopening:Option<conclusion::Reopening>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    ui_snapshot:Option<crate::debug_ui::RecoveryPin>,
}
#[derive(Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct PlannedStep {
    index:u32,request_id:String,action_sha256:String,state:String,
    #[serde(default,skip_serializing_if="Option::is_none")]
    original_run:Option<String>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    expected_boundary:Option<crate::debug_control::ExpectedBoundary>,
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
    #[serde(default,skip_serializing_if="Option::is_none")]
    wait_expected_boundary:Option<crate::debug_control::ExpectedBoundary>,
}
fn is_zero(n:&u32)->bool{*n==0}
#[derive(Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct WaitObservation {poll:u32,id:String,response_sha256:String,present:bool,actual:Value,matched:bool,
    #[serde(default,skip_serializing_if="Option::is_none")]
    boundary:Option<crate::debug_control::ExpectedBoundary>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    inputs:Option<Vec<ObservedInput>>,
}
#[derive(Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct ObservedInput {slot:String,id:String,sha256:String,
    #[serde(default,skip_serializing_if="Option::is_none")]
    recovery_identity:Option<String>,
}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct AssertionCheck {pointer:String,matched:bool}
fn assertion_checks(recipe:&Recipe,index:usize,result:&Value)->Vec<AssertionCheck>{
    recipe.assertions.iter().filter(|a|a.after_step as usize==index).map(|a|AssertionCheck{pointer:a.pointer.clone(),matched:result.pointer(&a.pointer).is_some_and(|actual|actual==&a.expected)}).collect()
}
fn digest(data:&[u8])->String{gba_core::ffta_rom::digest(data)}
fn recipe_hash(recipe:&Recipe)->Result<String,String>{Ok(digest(&serde_json::to_vec(recipe).map_err(|e|e.to_string())?))}
fn ui_snapshot_basis(checkpoint:&Checkpoint)->Result<String,String>{let mut basis=checkpoint.clone();basis.ui_snapshot=None;Ok(digest(&serde_json::to_vec(&basis).map_err(|e|e.to_string())?))}
fn action_hash(action:&crate::debug_control::Action)->Result<String,String>{Ok(digest(&serde_json::to_vec(action).map_err(|e|e.to_string())?))}
fn request_id(id:&str,run:&str,index:usize,action:&str)->Result<String,String>{
    Ok(format!("recipe-{}",digest(&serde_json::to_vec(&("ffta-recipe-step-v1",id,run,index,action)).map_err(|e|e.to_string())?)))
}
fn valid_id(id:&str)->bool{id.starts_with("run-") && crate::debug_cheat_preset::name_valid(id)}
fn observed_boundary(step:&PlannedStep,allow_ui:bool)->Option<crate::debug_control::ExpectedBoundary>{
    // Existing wait observations do not record the collector's state; do not
    // infer an open trace from one of those older, incomplete observations.
    if allow_ui && step.wait_observation.is_some(){return None;}
    if let Some(observation)=&step.wait_observation{return observation.boundary.clone();}
    step.result.as_ref().and_then(|r|{
        let mut status=r["status"].clone();
        if allow_ui {if status["ui_trace"]["has_trace"]!=true{return None;}status["ui_trace"]["has_trace"]=json!(false);}
        crate::debug_control::ExpectedBoundary::from_status(&status).ok()
    })
}
fn status_inputs(status:&Value)->Option<Vec<ObservedInput>>{
    let picker=&status["content_picker"];if picker["busy"]!=false{return None;}
    let inputs=picker["inputs"].as_array()?;if inputs.len()>18{return None;}
    inputs.iter().map(|input|Some(ObservedInput{slot:input["slot"].as_str()?.to_owned(),id:input["id"].as_str()?.to_owned(),sha256:input["sha256"].as_str()?.to_owned(),recovery_identity:input["recovery_identity"].as_str().map(str::to_owned)})).collect()
}
fn boundary_for_prefix(checkpoint:&Checkpoint,recipe:&Recipe,prefix:usize)->Result<crate::debug_control::ExpectedBoundary,String>{
    boundary_for_prefix_mode(checkpoint,recipe,prefix,false)
}
fn boundary_for_prefix_mode(checkpoint:&Checkpoint,recipe:&Recipe,prefix:usize,allow_ui:bool)->Result<crate::debug_control::ExpectedBoundary,String>{
    let prior=checkpoint.steps.get(prefix.checked_sub(1).ok_or("No prior boundary")?).ok_or("No prior boundary")?;
    let mut boundary=observed_boundary(prior,allow_ui).ok_or("The last observation has no recoverable idle guest boundary")?;
    let mut selected=std::collections::BTreeMap::new();
    for (step,action) in checkpoint.steps.iter().zip(&recipe.steps).take(prefix){
        if let crate::debug_control::Action::SelectId{slot,id}=action {
            let result=step.result.as_ref().ok_or("Selected input has no terminal result")?;
            let input:crate::debug_choices::SavedInput=serde_json::from_value(result["status"]["recovery_input"].clone()).map_err(|_|"Selected input has no recoverable location and hash")?;
            if input.slot()!=Ok(slot.content_slot()) || !super::hash_valid(&input.sha256){return Err("Selected input identity is invalid".into());}
            let identity=digest(&serde_json::to_vec(&input).map_err(|e|e.to_string())?);
            let observed=status_inputs(&result["status"]).ok_or("Selected input was not stable at completion")?;
            if !observed.iter().any(|item|item.slot==slot.content_slot().label() && item.id==*id && item.sha256==input.sha256 && item.recovery_identity.as_deref()==Some(identity.as_str())){return Err("Selected input result does not match the requested identity".into());}
            selected.insert(input.slot.clone(),(input,identity,slot.content_slot().label()));
        }
    }
    if !selected.is_empty(){
        let observed=if let Some(wait)=&prior.wait_observation {wait.inputs.clone().ok_or("The last wait did not record selected inputs")?}else{status_inputs(&prior.result.as_ref().ok_or("Missing previous result")?["status"]).ok_or("Selected inputs were not stable at the previous boundary")?};
        for (input,identity,label) in selected.values(){if !observed.iter().any(|item|item.slot==*label && item.sha256==input.sha256 && item.recovery_identity.as_deref()==Some(identity.as_str())){return Err("Selected input changed before the recovery boundary".into());}}
        let inputs:Vec<_>=selected.into_values().map(|(input,_,_)|input).collect();
        if inputs.len()>3 || serde_json::to_vec(&inputs).map_err(|e|e.to_string())?.len()>2048{return Err("Recovered selections exceed the guarded request limit".into());}
        boundary.expected_inputs=Some(inputs);
    }
    Ok(boundary)
}
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
    if let Some(reopening)=&checkpoint.reopening{reopening.validate(&checkpoint.id)?;}
    if let Some(pin)=&checkpoint.ui_snapshot {
        pin.validate()?;let index=checkpoint.steps.iter().position(|s|s.state!="completed").ok_or("UI snapshot has no remaining step")?;
        if pin.recipe_run!=checkpoint.id || pin.source_run!=checkpoint.session_run || pin.checkpoint_sha256!=ui_snapshot_basis(checkpoint)?
            || checkpoint.steps[index].state!="planned" || !runner::ui_open_prefix(&recipe.steps,index){return Err("UI snapshot does not match the stopped recipe".into());}
    }
    let mut unfinished=false;
    for (i,(step,action)) in checkpoint.steps.iter().zip(&recipe.steps).enumerate(){
        let hash=action_hash(action)?;
        let step_run=step.original_run.as_deref().unwrap_or(&checkpoint.session_run);
        if step_run.is_empty() || step_run.len()>100 || step_run.chars().any(char::is_control) || (step.original_run.is_some() && !matches!(step.state.as_str(),"completed"|"waiting"|"wait-timeout")) || step.index!=i as u32 || step.action_sha256!=hash || step.request_id!=request_id(&checkpoint.id,step_run,i,&hash)? {return Err("Recipe checkpoint step identity is invalid".into());}
        if let Some(boundary)=&step.expected_boundary {
            let mut expected=boundary_for_prefix_mode(checkpoint,recipe,i,boundary.expected_ui.is_some()).ok();
            if let (Some(value),Some(pin))=(&mut expected,&boundary.expected_ui){if pin.recipe_run!=checkpoint.id || !runner::ui_open_prefix(&recipe.steps,i){return Err("UI recovery pin does not match the recipe prefix".into());}pin.validate()?;value.expected_ui=Some(pin.clone());}
            if i==0 || checkpoint.steps[i-1].state!="completed" || expected.as_ref()!=Some(boundary) {return Err("Recipe recovery boundary does not match the previous observation".into());}
        }
        if let Some(boundary)=&step.wait_expected_boundary {if step.state!="waiting" || boundary_for_prefix(checkpoint,recipe,i+1).ok().as_ref()!=Some(boundary){return Err("Recipe wait recovery boundary does not match the last observation".into());}}
        let terminal=if let (Some(deadline),Some(value))=(step.deadline_unix_ms,step.result.as_ref()){
            let fingerprint=crate::debug_control::planned_guarded_fingerprint(action.clone(),step_run,&step.request_id,deadline,&checkpoint.manifest_sha256,step.expected_boundary.clone())?;
            value["run"]==step_run && value["id"]==step.request_id && value["deadline_unix_ms"]==deadline && value["request_sha256"]==fingerprint
        }else{false};
        let checks=step.result.as_ref().map(|r|assertion_checks(recipe,i,r)).unwrap_or_default();
        let wait=recipe.waits.iter().find(|w|w.after_step as usize==i);
        let no_wait=step.wait_deadline_unix_ms.is_none() && step.wait_polls==0 && step.wait_observation.is_none() && step.wait_expected_boundary.is_none();
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
#[cfg(test)]
fn prepare(root:&Path,name:&str)->Result<Checkpoint,String>{prepare_with_reason(root,name,None,None)}
fn prepare_with_reason(root:&Path,name:&str,reason:Option<&str>,reviewed_evidence:Option<&str>)->Result<Checkpoint,String>{
    let recipe=load(root,name)?;let session_run=active_run(root,&recipe.manifest_sha256)?;
    let base=base(root,true)?;let lock_path=base.join(".run.lock");
    if lock_path.exists(){crate::debug_session::regular(&lock_path)?;}
    let lock=OpenOptions::new().read(true).write(true).create(true).open(&lock_path).map_err(|e|e.to_string())?;crate::debug_session::regular(&lock_path)?;lock.try_lock().map_err(|_|"Another recipe preparation is in progress")?;
    let reopening=conclusion::authorize(root,&base,&recipe,reason)?;
    if reviewed_evidence.is_some_and(|expected|conclusion::fingerprint_reopening(&reopening).ok().as_deref()!=Some(expected)){return Err("Concluded evidence changed since Details; review it again".into());}
    let entries=fs::read_dir(&base).map_err(|e|e.to_string())?.filter_map(|e|match e{Ok(e) if e.file_name()==".run.lock"=>None,other=>Some(other)}).take(MAX_RUNS).collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
    if entries.len()>=MAX_RUNS{return Err("Recipe run directory is full".into());}
    let temp=tempfile::Builder::new().prefix("run-").rand_bytes(16).tempdir_in(&base).map_err(|e|e.to_string())?;
    let id=temp.path().file_name().and_then(|s|s.to_str()).filter(|s|valid_id(s)).ok_or("Invalid generated run ID")?.to_owned();
    let mut steps=Vec::with_capacity(recipe.steps.len());
    for (i,action) in recipe.steps.iter().enumerate(){let hash=action_hash(action)?;steps.push(PlannedStep{index:i as u32,request_id:request_id(&id,&session_run,i,&hash)?,action_sha256:hash,state:"planned".into(),original_run:None,expected_boundary:None,deadline_unix_ms:None,result:None,checks:Vec::new(),wait_deadline_unix_ms:None,wait_polls:0,wait_observation:None,wait_expected_boundary:None});}
    let checkpoint=Checkpoint{schema:1,kind:"ffta-debug-command-checkpoint".into(),id,recipe_name:name.into(),recipe_sha256:recipe_hash(&recipe)?,manifest_sha256:recipe.manifest_sha256.clone(),session_run,steps,deadline_unix_ms:None,reopening,ui_snapshot:None};
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
#[derive(Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct UiSnapshotEnvelope {
    schema:u32,kind:String,recipe_run:String,source_run:String,manifest_sha256:String,
    checkpoint_sha256:String,boundary:crate::debug_control::ExpectedBoundary,snapshot:crate::debug_ui::TraceSnapshot,
}
fn ui_snapshot_path(root:&Path,recipe_run:&str,source_run:&str)->Result<PathBuf,String>{
    let dir=runner::run_dir(root,recipe_run)?;
    Ok(dir.join(format!("ui-collector-{}.json",digest(source_run.as_bytes()))))
}
fn read_ui_envelope(root:&Path,pin:&crate::debug_ui::RecoveryPin,boundary:&crate::debug_control::ExpectedBoundary)->Result<UiSnapshotEnvelope,String>{
    pin.validate()?;let path=ui_snapshot_path(root,&pin.recipe_run,&pin.source_run)?;
    if !crate::debug_session::regular(&path)?.is_file(){return Err("UI trace snapshot is unavailable".into());}
    let bytes=crate::diagnostics::read(&path,crate::debug_ui::MAX_SNAPSHOT as u64)?;
    if digest(&bytes)!=pin.snapshot_sha256{return Err("UI trace snapshot changed".into());}
    let envelope:UiSnapshotEnvelope=serde_json::from_slice(&bytes).map_err(|e|e.to_string())?;envelope.snapshot.validate()?;
    let mut plain=boundary.clone();plain.expected_ui=None;
    if envelope.schema!=1 || envelope.kind!="ffta-recipe-ui-snapshot" || envelope.recipe_run!=pin.recipe_run || envelope.source_run!=pin.source_run
        || envelope.checkpoint_sha256!=pin.checkpoint_sha256 || envelope.boundary!=plain || envelope.snapshot.source_run()!=Some(pin.source_run.as_str())
        || envelope.snapshot.manifest_sha256()!=Some(envelope.manifest_sha256.as_str()) {return Err("UI trace snapshot does not match its recovery boundary".into());}
    Ok(envelope)
}
pub(super) fn restore_ui_snapshot(root:&Path,pin:&crate::debug_ui::RecoveryPin,boundary:&crate::debug_control::ExpectedBoundary)->Result<crate::debug_ui::TraceSnapshot,String>{
    let envelope=read_ui_envelope(root,pin,boundary)?;if manifest(root)?.0!=envelope.manifest_sha256{return Err("UI trace snapshot belongs to another session".into());}Ok(envelope.snapshot)
}
/// Preserve an open UI collector only on normal close, for one unambiguous
/// stopped recipe whose last completed guest boundary is still current.
pub(super) fn retain_ui_on_close(root:&Path,source_run:&str,status:&Value,snapshot:crate::debug_ui::TraceSnapshot)->Result<bool,String>{
    snapshot.validate()?;let Some(start_id)=snapshot.start_request_id() else{return Ok(false)};
    if snapshot.source_run()!=Some(source_run) || status["ui_trace"]["has_trace"]!=true || status["ui_trace"]["start_request_id"]!=start_id
        || status["ui_trace"]["events"]!=snapshot.event_count() as u64 || status["ui_trace"]["frames"]!=snapshot.frames() as u64{return Err("UI trace changed while preparing its snapshot".into());}
    let mut plain=status.clone();plain["ui_trace"]["has_trace"]=json!(false);
    let current=crate::debug_control::ExpectedBoundary::from_status(&plain)?;
    let base=match base(root,false){Ok(path)=>path,Err(_)=>return Ok(false)};let mut owners=Vec::new();let mut count=0;
    for entry in fs::read_dir(&base).map_err(|e|e.to_string())? {
        let entry=entry.map_err(|e|e.to_string())?;let name=entry.file_name().to_string_lossy().into_owned();if name==".run.lock"{continue;}
        count+=1;if count>MAX_RUNS{return Err("Too many recipe runs for UI trace retention".into());}if !valid_id(&name){continue;}
        let dir=entry.path();if !crate::debug_session::regular(&dir).is_ok_and(|m|m.is_dir()){continue;}let Ok(execution)=runner::lock(&dir,".execution.lock") else{continue};let Ok(transition)=runner::lock(&dir,".transition.lock") else{continue};
        let Ok((checkpoint,active))=inspect(root,&name) else{continue};if !active || checkpoint.session_run!=source_run || checkpoint.ui_snapshot.is_some() || checkpoint.deadline_unix_ms.is_none_or(|d|d<=runner::now()) {continue;}
        let recipe=load(root,&checkpoint.recipe_name)?;let Some(index)=checkpoint.steps.iter().position(|s|s.state!="completed") else{continue};
        if index==0 || checkpoint.steps[index].state!="planned" || checkpoint.steps[index+1..].iter().any(|s|s.state!="planned") || !runner::ui_open_prefix(&recipe.steps,index) || runner::recovery_scope(&recipe.steps,index,true).is_err(){continue;}
        let Some(start)=recipe.steps[..index].iter().rposition(|action|matches!(action,crate::debug_control::Action::UiStart)) else{continue};
        if checkpoint.steps[start].request_id!=start_id || checkpoint.steps[start].result.as_ref().is_none_or(|r|r["status"]["ui_trace"]["start_request_id"]!=start_id){continue;}
        let Ok(boundary)=boundary_for_prefix_mode(&checkpoint,&recipe,index,true) else{continue};
        if boundary.state_sha256!=current.state_sha256 || boundary.frame!=current.frame || boundary.clock!=current.clock || boundary.backend!=current.backend {continue;}
        if let Some(inputs)=&boundary.expected_inputs {
            let Some(observed)=status_inputs(status) else{continue};
            if inputs.iter().any(|input|{let Ok(slot)=input.slot() else{return true};let Ok(bytes)=serde_json::to_vec(input) else{return true};let identity=digest(&bytes);!observed.iter().any(|item|item.slot==slot.label() && item.sha256==input.sha256 && item.recovery_identity.as_deref()==Some(identity.as_str()))}){continue;}
        }
        owners.push((dir,execution,transition,checkpoint,recipe,boundary));
    }
    if owners.is_empty(){return Ok(false);}if owners.len()!=1{return Err("UI trace belongs to more than one stopped recipe".into());}
    let (dir,_execution,_transition,mut checkpoint,recipe,boundary)=owners.pop().unwrap();
    if snapshot.manifest_sha256()!=Some(checkpoint.manifest_sha256.as_str()){return Err("UI trace session identity changed".into());}
    let checkpoint_sha256=ui_snapshot_basis(&checkpoint)?;
    let envelope=UiSnapshotEnvelope{schema:1,kind:"ffta-recipe-ui-snapshot".into(),recipe_run:checkpoint.id.clone(),source_run:source_run.into(),manifest_sha256:checkpoint.manifest_sha256.clone(),checkpoint_sha256:checkpoint_sha256.clone(),boundary,snapshot};
    let bytes=serde_json::to_vec(&envelope).map_err(|e|e.to_string())?;if bytes.len()>crate::debug_ui::MAX_SNAPSHOT{return Err("UI trace snapshot exceeds 2 MiB".into());}
    let mut file=tempfile::NamedTempFile::new_in(&dir).map_err(|e|e.to_string())?;file.write_all(&bytes).and_then(|_|file.as_file().sync_all()).map_err(|e|e.to_string())?;
    file.persist_noclobber(ui_snapshot_path(root,&checkpoint.id,source_run)?).map_err(|e|e.error.to_string())?;
    checkpoint.ui_snapshot=Some(crate::debug_ui::RecoveryPin{recipe_run:checkpoint.id.clone(),source_run:source_run.into(),checkpoint_sha256,snapshot_sha256:digest(&bytes)});
    runner::publish(&dir,&checkpoint,&recipe)?;Ok(true)
}
fn ui_pin_for_recovery(root:&Path,checkpoint:&Checkpoint,boundary:&crate::debug_control::ExpectedBoundary)->Result<crate::debug_ui::RecoveryPin,String>{
    let pin=checkpoint.ui_snapshot.clone().ok_or("Earlier UI trace snapshot is unavailable")?;
    if pin.recipe_run!=checkpoint.id || pin.source_run!=checkpoint.session_run || pin.checkpoint_sha256!=ui_snapshot_basis(checkpoint)?{return Err("UI trace snapshot does not match the checkpoint".into());}
    let envelope=read_ui_envelope(root,&pin,boundary)?;
    if envelope.manifest_sha256!=checkpoint.manifest_sha256{return Err("UI trace snapshot belongs to another session".into());}
    let recipe=load(root,&checkpoint.recipe_name)?;let index=checkpoint.steps.iter().position(|s|s.state!="completed").ok_or("No remaining recipe step")?;
    let start=recipe.steps[..index].iter().rposition(|action|matches!(action,crate::debug_control::Action::UiStart)).ok_or("No preceding UI start")?;
    if envelope.snapshot.start_request_id()!=Some(checkpoint.steps[start].request_id.as_str()){return Err("UI trace snapshot has a different start request".into());}
    Ok(pin)
}
/// Preserve only a successful terminal response that belongs to an already
/// persisted pending recipe step. This runs before the shared mailbox is
/// published; ordinary commands and wait observations do not create receipts.
pub(super) fn retain_terminal(root:&Path,value:&Value)->Result<bool,String>{
    let Some(id)=value["id"].as_str() else{return Ok(false)};
    let Some(hash)=id.strip_prefix("recipe-") else{return Ok(false)};
    if hash.len()!=64 || !hash.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)){return Ok(false);}
    if value["ok"]!=true || value["outcome"]!="ok"{return Ok(false);}
    let base=match base(root,false){Ok(path)=>path,Err(_)=>return Ok(false)};
    let mut owner=None;let mut count=0;
    for entry in fs::read_dir(&base).map_err(|e|e.to_string())? {
        let entry=entry.map_err(|e|e.to_string())?;let name=entry.file_name().to_string_lossy().into_owned();if name==".run.lock"{continue;}
        count+=1;if count>MAX_RUNS{return Err("Too many recipe run entries for terminal retention".into());}
        if !valid_id(&name){continue;}
        let Ok((checkpoint,active))=inspect(root,&name) else{continue};if !active{continue;}
        let Some(index)=checkpoint.steps.iter().position(|step|step.request_id==id && step.state=="pending") else{continue};
        let recipe=load(root,&checkpoint.recipe_name)?;
        if !runner::matches_pending(&checkpoint,&recipe,index,value)?{continue;}
        if owner.is_some(){return Err("Terminal response matches more than one recipe run".into());}
        owner=Some((entry.path(),index));
    }
    let Some((dir,index))=owner else{return Ok(false)};
    let bytes=serde_json::to_vec_pretty(value).map_err(|e|e.to_string())?;
    if bytes.len()>128*1024{return Err("Recipe terminal response exceeds the retention limit".into());}
    let path=dir.join(format!("terminal-{index:02}.json"));
    match fs::symlink_metadata(&path){
        Ok(_)=>{if !crate::debug_session::regular(&path)?.is_file(){return Err("Recipe terminal receipt must be a regular file".into());}
            let old:Value=serde_json::from_slice(&crate::diagnostics::read(&path,128*1024)?).map_err(|e|e.to_string())?;
            return if old==*value{Ok(true)}else{Err("Recipe terminal receipt already differs".into())};},
        Err(e) if e.kind()==std::io::ErrorKind::NotFound=>{},Err(e)=>return Err(e.to_string()),
    }
    let mut file=tempfile::NamedTempFile::new_in(&dir).map_err(|e|e.to_string())?;
    file.write_all(&bytes).and_then(|_|file.as_file().sync_all()).map_err(|e|e.to_string())?;
    file.persist_noclobber(path).map_err(|e|e.error.to_string())?;Ok(true)
}
pub(super) fn review_evidence(root:&Path,recipe:&Recipe)->Result<(Value,String),String>{conclusion::preview(root,recipe)}
pub(super) fn run_reviewed(root:&Path,name:&str,expected:&str,cancel:&std::sync::atomic::AtomicBool,reason:Option<&str>,reviewed_evidence:&str)->Result<Value,String>{
    let checkpoint=prepare_with_reason(root,name,reason,Some(reviewed_evidence))?;
    if checkpoint.recipe_sha256!=expected{return Err("Saved recipe changed after preview; no action sent".into());}
    runner::drive_with_cancel(root,&checkpoint.id,false,Some(cancel)).map_err(|e|format!("Checkpoint {} was prepared: {e}",checkpoint.id))
}
pub(super) fn prepare_command(root:&Path,name:&str,reason:Option<&str>)->Value{
    match prepare_with_reason(root,name,reason,None){Ok(checkpoint)=>json!({"schema":1,"kind":"ffta-debug-command-checkpoint-result","ok":true,"outcome":"prepared","checkpoint":checkpoint,"scope":"Identities persisted; no request sent. Readiness metadata does not prove process liveness. Run is a separate explicit command."}),Err(e)=>super::super::local_error("invalid-request",&e)}
}
pub(super) fn conclude_command(root:&Path,id:&str)->Value{conclusion::command(root,id)}
pub(super) fn inspect_command(root:&Path,id:&str)->Value{
    match inspect(root,id){Ok((checkpoint,active))=>json!({"schema":1,"kind":"ffta-debug-command-checkpoint-result","ok":true,"outcome":"inspected","checkpoint":checkpoint,"readiness_matches_run":active,"scope":"Readiness metadata does not prove process liveness."}),Err(e)=>super::super::local_error("invalid-request",&e)}
}
pub(super) fn run_command(root:&Path,id:&str,resume:bool)->Value{runner::command(root,id,resume)}
pub(super) fn recover_command(root:&Path,id:&str)->Value{runner::recover_command(root,id)}
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
