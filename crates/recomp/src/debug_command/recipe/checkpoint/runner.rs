//! Bounded recipe dispatch with a persisted identity before each request.
use super::*;
use crate::debug_control::Action;
use std::{fs::File,time::{SystemTime,UNIX_EPOCH}};

fn now()->u64{SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis().min(u128::from(u64::MAX)) as u64}
pub(super) fn run_dir(root:&Path,id:&str)->Result<PathBuf,String>{
    if !valid_id(id){return Err("Invalid recipe run ID".into());}
    let dir=base(root,false)?.join(id);if !crate::debug_session::regular(&dir)?.is_dir(){return Err("Recipe run must be a directory".into());}Ok(dir)
}
pub(super) fn lock(dir:&Path,name:&str)->Result<File,String>{
    let path=dir.join(name);if path.exists(){crate::debug_session::regular(&path)?;}
    let file=OpenOptions::new().read(true).write(true).create(true).open(&path).map_err(|e|e.to_string())?;
    crate::debug_session::regular(&path)?;file.try_lock().map_err(|_|"Recipe run is busy")?;Ok(file)
}
fn publish(dir:&Path,checkpoint:&Checkpoint,recipe:&Recipe)->Result<(),String>{
    validate(checkpoint,recipe)?;let bytes=serde_json::to_vec_pretty(checkpoint).map_err(|e|e.to_string())?;
    if bytes.len() as u64>MAX_CHECKPOINT || bytes.len()>recipe.limits.output_bytes as usize{return Err("Checkpoint output limit exceeded".into());}
    let mut file=tempfile::NamedTempFile::new_in(dir).map_err(|e|e.to_string())?;
    file.write_all(&bytes).and_then(|_|file.as_file().sync_all()).map_err(|e|e.to_string())?;
    file.persist(dir.join("checkpoint.json")).map_err(|e|e.error.to_string())?;Ok(())
}
fn supported(action:&Action)->bool{matches!(action,
    Action::Status|Action::Pause|Action::Inspect{..}|Action::Step{..}|Action::Find{..}|
    Action::ContentRefresh|Action::ContentList{..}|Action::SelectId{..}|
    Action::CheatRefresh|Action::CheatList{..}|Action::CheatShow{..}|Action::CheatCapabilities{..}|
    Action::CheatPrepare{..}|Action::CheatApply{..}|Action::CheatArm{..}|Action::CheatDisarm{..}|
    Action::CheatPresetSave{..}|Action::CheatPresetPrepare{..}|Action::CheatPresetArm{..}|
    Action::AudioStart{..}|Action::AudioReport|Action::CodeStart{..}|Action::CodeReport|
    Action::GameplayStart{..}|Action::GameplayReport|Action::UiStart|Action::UiReport|Action::Graphics{..}|
    Action::RecentStart{..}|Action::RecentFreeze|Action::RecentReport|Action::IssueReport{..}|Action::Replay{max_frames:Some(_),..}
)}
fn result(checkpoint:&Checkpoint,outcome:&str,message:&str)->Value{
    let ok=matches!(outcome,"completed"|"cancelled"|"cancellation-pending");
    json!({"schema":1,"kind":"ffta-debug-command-recipe-run","ok":ok,"outcome":outcome,"message":message,"error":if ok{None}else{Some(message)},"checkpoint":checkpoint})
}
fn cancelled_in_flight(root:&Path,id:&str,checkpoint:&Checkpoint,query:bool)->Value{
    let saved=cancel_command(root,id);
    if saved["ok"]!=true{return result(checkpoint,"unconfirmed","The client stopped waiting but could not persist cancellation; inspect the pending request.");}
    if query {result(checkpoint,"cancelled","Cancellation recorded; an observation may still complete, but no later action was sent.")}
    else {result(checkpoint,"cancellation-pending","Cancellation recorded; the reserved action may still execute. Its outcome is unconfirmed.")}
}
fn drive(root:&Path,id:&str,resume:bool)->Result<Value,String>{drive_with_cancel(root,id,resume,None)}
fn recovery_scope(steps:&[Action],index:usize)->Result<(),String>{
    // Completed start/report pairs no longer need their collector in memory.
    // An open or unmatched collector must never be recreated from a guest state.
    let (mut audio,mut code,mut gameplay,mut ui)=(false,false,false,false);
    for action in &steps[..index] {
        match action {
            Action::Status|Action::Pause|Action::Inspect{..}|Action::Step{..}|Action::ContentRefresh|Action::ContentList{..}|Action::SelectId{..}=>{},
            Action::Find{settings} if matches!(settings.on_hit,None|Some(crate::debug_probe::HitAction::Pause))=>{},
            Action::AudioStart{..} if !audio=>audio=true,
            Action::AudioReport if audio=>audio=false,
            Action::CodeStart{..} if !code=>code=true,
            Action::CodeReport if code=>code=false,
            Action::GameplayStart{..} if !gameplay=>gameplay=true,
            Action::GameplayReport if gameplay=>gameplay=false,
            Action::UiStart if !ui=>ui=true,
            Action::UiReport if ui=>ui=false,
            _=>return Err("Completed steps use host context that cross-run recovery cannot restore".into()),
        }
    }
    if audio || code || gameplay || ui {return Err("An earlier collector is still open and cannot be restored".into());}
    let (mut audio,mut code,mut gameplay,mut ui,mut graphics,mut content,mut cheat)=(false,false,false,false,false,false,false);
    for action in &steps[index..] {
        match action {
            Action::Status|Action::Pause|Action::Inspect{..}|Action::Step{..}=>{},
            Action::Find{settings} if !matches!(settings.on_hit,Some(crate::debug_probe::HitAction::Freeze))=>{},
            Action::AudioStart{..}=>audio=true,
            Action::AudioReport if audio=>audio=false,
            Action::CodeStart{..}=>code=true,
            Action::CodeReport if code=>code=false,
            Action::GameplayStart{..}=>gameplay=true,
            Action::GameplayReport if gameplay=>gameplay=false,
            Action::UiStart=>ui=true,
            Action::UiReport if ui=>ui=false,
            Action::Graphics{compare_last:false}=>graphics=true,
            Action::Graphics{compare_last:true} if graphics=>{},
            Action::ContentRefresh=>content=true,
            Action::ContentList{..} if content=>{},
            Action::SelectId{..} if content=>cheat=false,
            Action::CheatRefresh=>cheat=true,
            Action::CheatList{..}|Action::CheatShow{..}|Action::CheatCapabilities{..} if cheat=>{},
            _=>return Err("Remaining steps need host context that cross-run recovery cannot restore".into()),
        }
    }
    Ok(())
}
/// A terminal response left by the old server can resolve a pending step, but
/// absence or a mismatch never authorizes resending it in the new run.
fn retained_pending(root:&Path,checkpoint:&Checkpoint,recipe:&Recipe,index:usize)->Result<Value,String>{
    let step=&checkpoint.steps[index];let action=&recipe.steps[index];
    let deadline=step.deadline_unix_ms.ok_or("Pending request has no original deadline")?;
    let fingerprint=crate::debug_control::planned_guarded_fingerprint(action.clone(),&checkpoint.session_run,&step.request_id,deadline,&checkpoint.manifest_sha256,step.expected_boundary.clone())?;
    let name=if action.read_query(){"query-result.json"}else{"result.json"};let path=root.join("Data/config/debug-control").join(name);
    if !crate::debug_session::regular(&path)?.is_file(){return Err("Retained terminal response is unavailable".into());}
    let value:Value=serde_json::from_slice(&crate::diagnostics::read(&path,128*1024)?).map_err(|e|e.to_string())?;
    if value["schema"]!=1 || value["kind"]!="ffta-debug-control-result" || value["ok"]!=true || value["outcome"]!="ok" || value.get("error")!=Some(&Value::Null) || !value["status"].is_object()
        || value["run"]!=checkpoint.session_run || value["id"]!=step.request_id || value["deadline_unix_ms"]!=deadline || value["request_sha256"]!=fingerprint
        || value["session"]["manifest_sha256"]!=checkpoint.manifest_sha256 || value["operation_id"]!=action.operation_id() {
        return Err("Retained response is absent, unsuccessful or does not match the pending request".into());
    }
    Ok(value)
}
fn wait_deadline(checkpoint:&Checkpoint,recipe:&Recipe,response:&Value,timeout_ms:u32,require_origin:bool)->Result<u64,String>{
    let deadline=checkpoint.deadline_unix_ms.ok_or("Recipe deadline is unavailable")?;
    let origin=match response.get("completed_unix_ms") {
        Some(value)=>value.as_u64().ok_or("Terminal response has an invalid completion time")?,
        None if !require_origin=>now(),
        None=>return Err("The retained response has no completion time for its wait".into()),
    };
    if origin<deadline.saturating_sub(u64::from(recipe.limits.duration_ms)) || origin>now() || origin>=deadline{return Err("Terminal response completion time is outside the original recipe budget".into());}
    Ok(origin.saturating_add(u64::from(timeout_ms)).min(deadline))
}
/// Rebind a stopped, unambiguous checkpoint without dispatching an action.
/// Open collectors started before the boundary are not restored. Completed
/// start/report pairs retain their recorded results; new collectors may be
/// started by the remaining steps.
fn recover(root:&Path,id:&str)->Result<Value,String>{
    let dir=run_dir(root,id)?;let _run_lock=lock(&dir,".execution.lock")?;let _gate=lock(&dir,".transition.lock")?;
    let (mut checkpoint,active)=inspect(root,id)?;
    if active{return Err("This checkpoint already belongs to the active run; use resume".into());}
    let recipe=load(root,&checkpoint.recipe_name)?;
    let new_run=active_run(root,&checkpoint.manifest_sha256)?;
    if new_run==checkpoint.session_run{return Err("The original run is still active".into());}
    let ready:Value=serde_json::from_slice(&crate::diagnostics::read(&root.join("Data/config/debug-control/ready.json"),128*1024)?).map_err(|e|e.to_string())?;
    if ready["run"]!=new_run || ready["capabilities"].as_array().is_none_or(|c|!c.contains(&json!("guest-boundary-guard-v1"))){return Err("The new run does not support a guarded recovery".into());}
    if let Some(index)=checkpoint.steps.iter().position(|step|step.state!="completed") {
        if checkpoint.steps[index].state=="pending" {
            let response=retained_pending(root,&checkpoint,&recipe,index)?;
            let checks=assertion_checks(&recipe,index,&response);
            let wait=recipe.waits.iter().find(|wait|wait.after_step as usize==index);
            let until=if checks.iter().all(|check|check.matched){wait.map(|wait|wait_deadline(&checkpoint,&recipe,&response,wait.timeout_ms,true)).transpose()?}else{None};
            checkpoint.steps[index].state=if checks.iter().any(|check|!check.matched){"assertion-failed"}else if until.is_some_and(|deadline|deadline<=now()){ "wait-timeout" }else if until.is_some(){"waiting"}else{"completed"}.into();
            checkpoint.steps[index].wait_deadline_unix_ms=until;
            checkpoint.steps[index].checks=checks;checkpoint.steps[index].result=Some(response);
            publish(&dir,&checkpoint,&recipe)?;
            let passed=matches!(checkpoint.steps[index].state.as_str(),"completed"|"waiting");
            let completed=passed && checkpoint.steps.iter().all(|step|step.state=="completed");
            let expired=checkpoint.steps[index].state=="wait-timeout";
            return Ok(json!({"schema":1,"kind":"ffta-debug-command-recipe-run","ok":passed,"outcome":if completed{"completed"}else if passed{"resolved-pending"}else if expired{"wait-timeout"}else{"assertion-failed"},"message":if completed{"Recorded the matching terminal response; all steps are complete and no action was sent."}else if passed{"Recorded the matching terminal response; no action was sent. Inspect and recover again to continue."}else if expired{"The original wait deadline elapsed; no later action was sent."}else{"The recorded response failed an assertion; no later action was sent."},"error":if passed{None}else if expired{Some("The original wait deadline elapsed")}else{Some("An assertion failed")},"checkpoint":checkpoint}));
        }
    }
    if checkpoint.deadline_unix_ms.is_none_or(|deadline|now()>=deadline){return Err("The original recipe deadline has elapsed; no action sent".into());}
    let index=checkpoint.steps.iter().position(|s|s.state!="completed").ok_or("All recipe steps already completed")?;
    let waiting=checkpoint.steps[index].state=="waiting";
    if (!waiting && (index==0 || checkpoint.steps[index].state!="planned")) || checkpoint.steps[index+1..].iter().any(|s|s.state!="planned") {return Err("Recovery requires completed steps or a recorded wait followed only by unsent steps; unresolved actions cannot be repeated".into());}
    let suffix=if waiting {index+1}else{index};
    recovery_scope(&recipe.steps,suffix)?;
    let prior=if waiting {&checkpoint.steps[index]}else{&checkpoint.steps[index-1]};
    if waiting {
        let condition=recipe.waits.iter().find(|w|w.after_step as usize==index).ok_or("Missing wait condition")?;
        if prior.wait_deadline_unix_ms.is_none_or(|deadline|now()>=deadline) || prior.wait_polls>=condition.max_polls{return Err("The original wait budget has elapsed; no observation sent".into());}
        if prior.wait_polls>0 && prior.wait_observation.as_ref().is_none_or(|observation|observation.poll!=prior.wait_polls){return Err("The last wait observation is unconfirmed; no observation sent".into());}
    }
    let boundary=boundary_for_prefix(&checkpoint,&recipe,suffix)?;
    if boundary.expected_inputs.is_some() && ready["capabilities"].as_array().is_none_or(|c|!c.contains(&json!("recipe-selected-inputs-v1"))){return Err("The new run cannot guard recovered selections".into());}
    let _unstarted=crate::debug_session::replay_command(root)?;
    if let Some(inputs)=&boundary.expected_inputs {
        // Recheck content and action-specific preflight outside the frontend
        // thread. The frontend separately checks the restored role and path
        // immediately before dispatch; actions also recheck before using it.
        let anchors=crate::debug_choices::Anchors{app:root.to_owned(),config:root.join("Data/config")};
        for input in inputs {input.restore(&anchors,&recipe.rom_sha256).map_err(|e|format!("Recovered selection unavailable: {e}"))?;}
    }
    if checkpoint.deadline_unix_ms.is_none_or(|deadline|now()>=deadline) || (waiting && checkpoint.steps[index].wait_deadline_unix_ms.is_none_or(|deadline|now()>=deadline)) {return Err("The original recovery deadline elapsed during input verification; no action sent".into());}
    let old_run=checkpoint.session_run.clone();
    for step in &mut checkpoint.steps[..suffix] {if step.original_run.is_none(){step.original_run=Some(old_run.clone());}}
    checkpoint.session_run=new_run.clone();
    for (i,step) in checkpoint.steps.iter_mut().enumerate().skip(suffix){step.request_id=request_id(id,&new_run,i,&step.action_sha256)?;step.expected_boundary=None;}
    if waiting {checkpoint.steps[index].wait_expected_boundary=Some(boundary);}else{checkpoint.steps[index].expected_boundary=Some(boundary);}
    publish(&dir,&checkpoint,&recipe)?;
    Ok(json!({"schema":1,"kind":"ffta-debug-command-recipe-run","ok":true,"outcome":"rebound","message":"Checkpoint rebound; restore the exact guest boundary and explicitly resume. No action was sent.","checkpoint":checkpoint}))
}
pub(super) fn drive_with_cancel(root:&Path,id:&str,resume:bool,local_cancel:Option<&std::sync::atomic::AtomicBool>)->Result<Value,String>{
    let dir=run_dir(root,id)?;let _run_lock=lock(&dir,".execution.lock")?;
    let (mut checkpoint,active)=inspect(root,id)?;
    if !active{return Err("Readiness does not match the original control run".into());}
    let recipe=load(root,&checkpoint.recipe_name)?;
    recipe.validate_run_frame_budget()?;
    if recipe.steps.iter().any(|a|!supported(a)){return Err("Unsupported action in recipe; no action sent".into());}
    {
        let _gate=lock(&dir,".transition.lock")?;
        let cancel=dir.join("cancel.json");
        if resume && cancel.exists(){crate::debug_session::regular(&cancel)?;fs::remove_file(cancel).map_err(|e|e.to_string())?;}
        if checkpoint.deadline_unix_ms.is_none(){
            let base=base(root,false)?;let _global=lock(&base,".run.lock")?;
            super::conclusion::check_start(root,&checkpoint,&recipe)?;
            checkpoint.deadline_unix_ms=Some(now().saturating_add(u64::from(recipe.limits.duration_ms)));publish(&dir,&checkpoint,&recipe)?;
        }
    }
    // Check immutable input bytes, rather than trusting only manifest claims.
    // Time spent checking inputs counts against the persisted run deadline.
    let _unstarted=crate::debug_session::replay_command(root)?;
    loop {
        let (index,fresh,waiting)= {
            let _gate=lock(&dir,".transition.lock")?;
            let cancel=dir.join("cancel.json");
            if local_cancel.is_some_and(|c|c.load(std::sync::atomic::Ordering::SeqCst)) && !cancel.exists(){
                let mut marker=OpenOptions::new().write(true).create_new(true).open(&cancel).map_err(|e|e.to_string())?;
                marker.write_all(b"{\"cancel\":true}\n").and_then(|_|marker.sync_all()).map_err(|e|e.to_string())?;
            }
            if cancel.exists(){
                crate::debug_session::regular(&cancel)?;
                return if checkpoint.steps.iter().any(|s|s.state=="pending") {
                    Ok(result(&checkpoint,"cancellation-pending","Cancellation requested; the reserved step may still be active and its outcome is unconfirmed."))
                }else{Ok(result(&checkpoint,"cancelled","Cancellation recorded between steps; use resume explicitly."))};
            }
            let Some(index)=checkpoint.steps.iter().position(|s|s.state!="completed") else{return Ok(result(&checkpoint,"completed","All recipe steps completed."));};
            if checkpoint.steps[index].state=="assertion-failed"{return Ok(result(&checkpoint,"assertion-failed","An assertion failed; no later step was sent."));}
            if checkpoint.steps[index].state=="failed"{return Ok(result(&checkpoint,"failed","A step failed; no later step was sent."));}
            if checkpoint.steps[index].state=="wait-timeout"{return Ok(result(&checkpoint,"wait-timeout","The bounded wait expired; no later step was sent."));}
            let waiting=checkpoint.steps[index].state=="waiting";
            let fresh=checkpoint.steps[index].state=="planned";
            if fresh {
                let deadline=checkpoint.deadline_unix_ms.unwrap();if now()>=deadline{return Ok(result(&checkpoint,"timeout","Recipe deadline elapsed before the next step."));}
                checkpoint.steps[index].state="pending".into();checkpoint.steps[index].deadline_unix_ms=Some(deadline);
                // This atomic replacement must complete before the mailbox write.
                publish(&dir,&checkpoint,&recipe)?;
            } else if !waiting && recipe.steps[index].read_query(){
                return Ok(result(&checkpoint,"unconfirmed","A previous read query is ambiguous and has no retry ledger; it was not repeated."));
            }
            (index,fresh,waiting)
        };
        if waiting {
            let condition=recipe.waits.iter().find(|w|w.after_step as usize==index).ok_or("Missing wait condition")?;
            let deadline=checkpoint.steps[index].wait_deadline_unix_ms.ok_or("Missing wait deadline")?;
            if checkpoint.steps[index].wait_polls>=condition.max_polls || deadline.saturating_sub(now())<20 {
                checkpoint.steps[index].state="wait-timeout".into();checkpoint.steps[index].wait_expected_boundary=None;publish(&dir,&checkpoint,&recipe)?;
                return Ok(result(&checkpoint,"wait-timeout","The bounded wait expired; no later step was sent."));
            }
            if active_run(root,&checkpoint.manifest_sha256).ok().as_deref()!=Some(&checkpoint.session_run){return Ok(result(&checkpoint,"unconfirmed","The control run changed before the wait observation."));}
            // Reserve each read before publication: crashes cannot increase the poll budget.
            checkpoint.steps[index].wait_polls+=1;publish(&dir,&checkpoint,&recipe)?;
            let remaining=deadline.saturating_sub(now());if remaining<20{continue;}
            // Even an ordinary status serializes the guest and can take more
            // than a second. Its reserved identity is never retried, and the
            // original wait deadline and cancellation marker still apply.
            let timeout=remaining.min(120_000);
            let poll_id=format!("recipe-wait-{}",digest(&serde_json::to_vec(&(&checkpoint.id,&checkpoint.session_run,index,checkpoint.steps[index].wait_polls)).map_err(|e|e.to_string())?));
            let response=crate::debug_control::planned_wait_observation(root,timeout,checkpoint.session_run.clone(),poll_id,deadline,checkpoint.steps[index].wait_expected_boundary.clone(),local_cancel,&dir.join("cancel.json"));
            let value=match response{Ok(v)=>v,Err(e)=>{if local_cancel.is_some_and(|c|c.load(std::sync::atomic::Ordering::SeqCst)){return Ok(cancelled_in_flight(root,id,&checkpoint,true));}return Ok(result(&checkpoint,"unconfirmed",&format!("Wait observation unresolved ({}; delivery {}). Resume explicitly within the original bounds.",e.message,e.delivery)));}};
            if value["outcome"]=="interrupted"{return Ok(cancelled_in_flight(root,id,&checkpoint,true));}
            if value["kind"]!="ffta-debug-control-result" || value["ok"]!=true || value["run"]!=checkpoint.session_run {
                let detail=value["error"].as_str().or_else(||value["outcome"].as_str()).unwrap_or("missing or mismatched response").chars().take(256).collect::<String>();
                return Ok(result(&checkpoint,"unconfirmed",&format!("Wait observation unresolved: {detail}")));
            }
            if now()>deadline {checkpoint.steps[index].state="wait-timeout".into();checkpoint.steps[index].wait_expected_boundary=None;publish(&dir,&checkpoint,&recipe)?;return Ok(result(&checkpoint,"wait-timeout","The bounded wait expired before the observation arrived."));}
            let id=value["id"].as_str().filter(|s|!s.is_empty()&&s.len()<=100).ok_or("Invalid wait observation identity")?.to_owned();
            let observed=value.pointer(&condition.pointer).filter(|v|super::super::valid_scalar(v));
            let matched=observed.is_some_and(|v|v==&condition.expected);
            checkpoint.steps[index].wait_observation=Some(WaitObservation{poll:checkpoint.steps[index].wait_polls,id,response_sha256:digest(&serde_json::to_vec(&value).map_err(|e|e.to_string())?),present:observed.is_some(),actual:observed.cloned().unwrap_or(Value::Null),matched,boundary:crate::debug_control::ExpectedBoundary::from_status(&value["status"]).ok(),inputs:status_inputs(&value["status"])});
            checkpoint.steps[index].wait_expected_boundary=None;
            if matched {checkpoint.steps[index].state="completed".into();}
            publish(&dir,&checkpoint,&recipe)?;
            if !matched {std::thread::sleep(std::time::Duration::from_millis(u64::from(condition.interval_ms).min(deadline.saturating_sub(now()))));}
            continue;
        }
        let step=&checkpoint.steps[index];let deadline=step.deadline_unix_ms.unwrap();
        let wait=deadline.saturating_sub(now()).clamp(20,120_000);
        let response=if let Some(boundary)=step.expected_boundary.clone() {
            crate::debug_control::planned_guarded_action(root,recipe.steps[index].clone(),wait,checkpoint.session_run.clone(),step.request_id.clone(),deadline,boundary,local_cancel,None)
        }else if let Some(flag)=local_cancel {crate::debug_control::planned_action_interruptible(root,recipe.steps[index].clone(),wait,checkpoint.session_run.clone(),step.request_id.clone(),deadline,flag)}else{crate::debug_control::planned_action_classified(root,recipe.steps[index].clone(),wait,checkpoint.session_run.clone(),step.request_id.clone(),deadline)};
        let value=match response{Ok(v)=>v,Err(e)=>{if local_cancel.is_some_and(|c|c.load(std::sync::atomic::Ordering::SeqCst)){return Ok(cancelled_in_flight(root,id,&checkpoint,false));}return Ok(result(&checkpoint,"unconfirmed",&format!("Request may be unresolved ({}; delivery {}). Use resume only for a ledger-protected step.",e.message,e.delivery)));}};
        if value["outcome"]=="interrupted"{return Ok(cancelled_in_flight(root,id,&checkpoint,false));}
        if value["kind"]!="ffta-debug-control-result"{return Ok(result(&checkpoint,"unconfirmed","No terminal server response; retain the same identity for recovery."));}
        let existing=checkpoint.steps.iter().filter_map(|s|s.result.as_ref()).map(|v|serde_json::to_vec(v).map(|b|b.len()).unwrap_or(usize::MAX)).sum::<usize>();
        let bytes=serde_json::to_vec(&value).map_err(|e|e.to_string())?;
        if existing.saturating_add(bytes.len())>recipe.limits.output_bytes.saturating_sub(64*1024) as usize {
            checkpoint.steps[index].state="failed".into();checkpoint.steps[index].result=Some(json!({"kind":"ffta-recipe-output-limit","ok":false,"run":checkpoint.session_run,"id":checkpoint.steps[index].request_id,"deadline_unix_ms":deadline,"request_sha256":value["request_sha256"],"response_sha256":digest(&bytes),"response_bytes":bytes.len()}));
        } else {
            let checks=if value["ok"]==true{assertion_checks(&recipe,index,&value)}else{Vec::new()};
            checkpoint.steps[index].state=if value["ok"]!=true{"failed"}else if checks.iter().any(|c|!c.matched){"assertion-failed"}else{"completed"}.into();
            checkpoint.steps[index].checks=checks;checkpoint.steps[index].result=Some(value);
            if checkpoint.steps[index].state=="completed" {
                if let Some(condition)=recipe.waits.iter().find(|w|w.after_step as usize==index){
                    match wait_deadline(&checkpoint,&recipe,checkpoint.steps[index].result.as_ref().unwrap(),condition.timeout_ms,false){
                        Ok(until)=>{checkpoint.steps[index].state="waiting".into();checkpoint.steps[index].wait_deadline_unix_ms=Some(until);},
                        Err(_)=>{checkpoint.steps[index].state="wait-timeout".into();checkpoint.steps[index].wait_deadline_unix_ms=Some(now().min(deadline));},
                    }
                }
            }
        }
        if serde_json::to_vec_pretty(&checkpoint).map_err(|e|e.to_string())?.len()>recipe.limits.output_bytes as usize {
            let full=checkpoint.steps[index].result.take().ok_or("Missing terminal result")?;
            let full_bytes=serde_json::to_vec(&full).map_err(|e|e.to_string())?;
            checkpoint.steps[index].state="failed".into();checkpoint.steps[index].checks.clear();checkpoint.steps[index].wait_deadline_unix_ms=None;checkpoint.steps[index].result=Some(json!({"kind":"ffta-recipe-output-limit","ok":false,"run":checkpoint.session_run,"id":checkpoint.steps[index].request_id,"deadline_unix_ms":deadline,"request_sha256":full["request_sha256"],"response_sha256":digest(&full_bytes),"response_bytes":full_bytes.len()}));
        }
        publish(&dir,&checkpoint,&recipe)?;
        if checkpoint.steps[index].state=="assertion-failed"{return Ok(result(&checkpoint,"assertion-failed","An assertion failed; no later step was sent."));}
        if checkpoint.steps[index].state=="failed"{return Ok(result(&checkpoint,"failed","A step failed or exceeded the output limit; no later step was sent."));}
        let _=fresh;
    }
}
pub(super) fn command(root:&Path,id:&str,resume:bool)->Value{
    match drive(root,id,resume){Ok(v)=>v,Err(e)=>super::super::super::local_error("invalid-request",&e)}
}
pub(super) fn recover_command(root:&Path,id:&str)->Value{
    match recover(root,id){Ok(v)=>v,Err(e)=>super::super::super::local_error("invalid-request",&e)}
}
pub(super) fn cancel_command(root:&Path,id:&str)->Value{
    let result=(||->Result<(),String>{
        let dir=run_dir(root,id)?;let _gate=lock(&dir,".transition.lock")?;let _=inspect(root,id)?;
        let path=dir.join("cancel.json");if path.exists(){crate::debug_session::regular(&path)?;return Ok(());}
        let mut f=OpenOptions::new().write(true).create_new(true).open(path).map_err(|e|e.to_string())?;
        f.write_all(b"{\"cancel\":true}\n").and_then(|_|f.sync_all()).map_err(|e|e.to_string())?;Ok(())
    })();
    match result{Ok(())=>json!({"schema":1,"kind":"ffta-debug-command-recipe-run","ok":true,"outcome":"cancellation-requested","message":"The reserved step may finish; later steps will not start."}),Err(e)=>super::super::super::local_error("invalid-request",&e)}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::{Arc,atomic::{AtomicBool,AtomicUsize,Ordering}},thread,time::Duration};
    fn fixture(commands:&[&str])->(tempfile::TempDir,crate::debug_control::Server,String){
        let t=tempfile::tempdir().unwrap();let root=t.path();fs::create_dir_all(root.join("Data/config")).unwrap();
        let exe=if cfg!(windows){"recomp.exe"}else{"recomp"};
        for (name,bytes) in [("portable.txt",b"portable".as_slice()),("ffta.gba",b"PUBLIC ROM".as_slice()),(exe,b"PUBLIC executable".as_slice())]{fs::write(root.join(name),bytes).unwrap();}
        let hash=|name:&str|digest(&fs::read(root.join(name)).unwrap());
        let pins=json!({"portable.txt":hash("portable.txt"),"ffta.gba":hash("ffta.gba"),(exe):hash(exe)});
        fs::write(root.join("debug-session.json"),serde_json::to_vec(&json!({"schema":1,"kind":"ffta-debug-session","label":"Synthetic","interp":true,"real_bios":false,"has_state":false,"has_dataset":false,"pins":pins})).unwrap()).unwrap();
        let server=crate::debug_control::Server::new(root).unwrap();
        super::super::super::save(root,"demo",&commands.iter().map(|s|s.to_string()).collect::<Vec<_>>()).unwrap();
        let checkpoint=super::super::prepare(root,"demo").unwrap();(t,server,checkpoint.id)
    }
    #[test]
    fn concluded_checkpoint_requires_reason_for_equivalent_new_runs(){
        let (t,server,id)=fixture(&["status"]);let root=t.path().to_owned();
        let early=super::super::prepare(&root,"demo").unwrap();
        assert_eq!(super::super::conclusion::command(&root,&early.id)["ok"],false);
        let recipe_path=root.join("Data/config/debug-command-recipes/recipe-demo.json");let (_,preview)=super::super::super::review_with_identity(&recipe_path,&root).unwrap();
        let stop=Arc::new(AtomicBool::new(false));let count=Arc::new(AtomicUsize::new(0));let worker=serve(server,root.clone(),id.clone(),false,stop.clone(),count.clone());
        let done=drive(&root,&id,false).unwrap();assert_eq!(done["outcome"],"completed");assert_eq!(count.load(Ordering::SeqCst),1);
        assert!(super::super::prepare(&root,"demo").is_ok()); // Completion alone does not mark evidence concluded.
        let marked=super::super::conclusion::command(&root,&id);assert_eq!(marked["outcome"],"concluded");
        assert_eq!(super::super::conclusion::command(&root,&id)["ok"],false);
        assert!(super::super::super::run_reviewed_with_reason(&recipe_path,&root,&preview,&AtomicBool::new(false),Some("Reason")).unwrap_err().contains("changed since Details"));
        assert!(super::super::prepare(&root,"demo").err().unwrap().contains("reopening reason"));
        assert!(super::super::prepare_with_reason(&root,"demo",Some("   "),None).err().unwrap().contains("reopening reason"));
        assert!(super::super::prepare_with_reason(&root,"demo",Some(&"x".repeat(513)),None).err().unwrap().contains("512"));
        assert!(drive(&root,&early.id,false).unwrap_err().contains("reopening reason"));
        assert_eq!(count.load(Ordering::SeqCst),1);
        super::super::super::save(&root,"renamed",&["status".into()]).unwrap();
        assert!(super::super::prepare(&root,"renamed").err().unwrap().contains("reopening reason"));
        let renewed=super::super::prepare_with_reason(&root,"renamed",Some("The observed behavior changed"),None).unwrap();
        let provenance=serde_json::to_value(renewed.reopening.as_ref().unwrap()).unwrap();assert_eq!(provenance["reason"],"The observed behavior changed");assert_eq!(provenance["evidence"][0]["run"],id);
        let completed=drive(&root,&renewed.id,false).unwrap();assert_eq!(completed["outcome"],"completed");assert_eq!(completed["checkpoint"]["reopening"],provenance);assert_eq!(count.load(Ordering::SeqCst),2);
        let original=root.join("Data/config/debug-command-runs").join(&id).join("checkpoint.json");let mut bytes=fs::read(&original).unwrap();bytes.push(b' ');fs::write(&original,bytes).unwrap();
        assert!(super::super::prepare_with_reason(&root,"demo",Some("A reason"),None).err().unwrap().contains("checkpoint changed"));assert_eq!(count.load(Ordering::SeqCst),2);
        stop.store(true,Ordering::SeqCst);worker.join().unwrap();
    }
    #[test]
    fn explicitly_concluded_negative_result_also_requires_reopening_reason(){
        let (t,server,id)=fixture(&["status","assert /status/frame 999","step 1"]);let root=t.path().to_owned();
        let stop=Arc::new(AtomicBool::new(false));let count=Arc::new(AtomicUsize::new(0));let worker=serve(server,root.clone(),id.clone(),false,stop.clone(),count.clone());
        assert_eq!(drive(&root,&id,false).unwrap()["outcome"],"assertion-failed");assert_eq!(count.load(Ordering::SeqCst),1);
        assert_eq!(super::super::conclusion::command(&root,&id)["outcome"],"concluded");assert!(super::super::prepare(&root,"demo").err().unwrap().contains("reopening reason"));
        super::super::super::save(&root,"changed-assert",&["status".into(),"assert /status/frame 1".into(),"step 1".into()]).unwrap();assert!(super::super::prepare(&root,"changed-assert").err().unwrap().contains("reopening reason"));
        let reopened=super::super::prepare_with_reason(&root,"demo",Some("The assertion changed"),None).unwrap();assert!(reopened.reopening.is_some());assert_eq!(count.load(Ordering::SeqCst),1);
        stop.store(true,Ordering::SeqCst);worker.join().unwrap();
    }
    #[test]
    fn recent_capture_and_issue_actions_remain_ordered_and_are_not_repeated(){
        let (t,server,id)=fixture(&["pause","recent-start","step 1","recent-freeze","recent-report","issue-report --symptom example"]);let root=t.path().to_owned();
        let stop=Arc::new(AtomicBool::new(false));let count=Arc::new(AtomicUsize::new(0));let worker=serve(server,root.clone(),id.clone(),false,stop.clone(),count.clone());
        let done=drive(&root,&id,false).unwrap();assert_eq!(done["outcome"],"completed");assert_eq!(count.load(Ordering::SeqCst),6);
        let operations=done["checkpoint"]["steps"].as_array().unwrap().iter().map(|step|step["result"]["operation_id"].as_str().unwrap()).collect::<Vec<_>>();
        assert_eq!(operations,["control.pause","control.recent-start","control.step","control.recent-freeze","control.recent-report","control.issue-report"]);
        let again=drive(&root,&id,true).unwrap();assert_eq!(again["checkpoint"],done["checkpoint"]);assert_eq!(count.load(Ordering::SeqCst),6);
        stop.store(true,Ordering::SeqCst);worker.join().unwrap();
    }
    #[test]
    fn a_refused_cheat_action_stops_the_recipe_without_repeating_prior_actions(){
        let generation="a".repeat(64);let preview="b".repeat(64);let commands=[format!("cheats prepare CH-0001 0 CH-0002 {generation}"),format!("cheats apply-once CH-0001 0 CH-0002 {generation} {preview}"),"step 1".into()];
        let inputs=commands.iter().map(String::as_str).collect::<Vec<_>>();let (t,mut server,id)=fixture(&inputs);let root=t.path().to_owned();
        let stop=Arc::new(AtomicBool::new(false));let count=Arc::new(AtomicUsize::new(0));let end=stop.clone();let calls=count.clone();
        let worker=thread::spawn(move||{while !end.load(Ordering::SeqCst){if let Ok(Some(request))=server.poll(){let n=calls.fetch_add(1,Ordering::SeqCst)+1;let error=(n==2).then(||"Preview changed".to_owned());let _=server.reply(&request,json!({"paused":true,"frame":1,"remaining":0}),error);}thread::sleep(Duration::from_millis(2));}});
        let failed=drive(&root,&id,false).unwrap();assert_eq!(failed["outcome"],"failed");assert_eq!(failed["checkpoint"]["steps"][0]["state"],"completed");assert_eq!(failed["checkpoint"]["steps"][1]["state"],"failed");assert_eq!(failed["checkpoint"]["steps"][2]["state"],"planned");assert_eq!(count.load(Ordering::SeqCst),2);
        assert_eq!(drive(&root,&id,true).unwrap()["outcome"],"failed");assert_eq!(count.load(Ordering::SeqCst),2);stop.store(true,Ordering::SeqCst);worker.join().unwrap();
    }
    fn serve(mut server:crate::debug_control::Server,root:PathBuf,id:String,cancel_first:bool,stop:Arc<AtomicBool>,count:Arc<AtomicUsize>)->thread::JoinHandle<()> {
        thread::spawn(move||{
            while !stop.load(Ordering::SeqCst){
                for query in [true,false] {
                    let request=match if query{server.poll_query()}else{server.poll()}{Ok(request)=>request,Err(_)=>continue};
                    if let Some(request)=request {
                        let n=count.fetch_add(1,Ordering::SeqCst)+1;
                        if cancel_first && n==1 {assert_eq!(cancel_command(&root,&id)["ok"],true);}
                        // A failed publication remains in the server outbox; the next poll retries it.
                        let _=server.reply(&request,json!({"paused":true,"frame":n,"remaining":0}),None);
                    }
                }
                thread::sleep(Duration::from_millis(2));
            }
        })
    }
    #[test]
    fn cross_run_recovery_preserves_completed_identity_and_guards_first_remaining_action(){
        let (t,mut server,id)=fixture(&["step 1","step 2"]);let root=t.path().to_owned();let worker_id=id.clone();
        let status=json!({"paused":true,"remaining":0,"state_sha256":"a".repeat(64),"frame":1,"clock":100,"backend":"interp","cheat_armed":{"active":false},"replay":{"active":false},"ui_trace":{"has_trace":false}});
        let first_status=status.clone();
        let worker=thread::spawn(move||loop{if let Ok(Some(request))=server.poll(){assert_eq!(cancel_command(&root,&worker_id)["ok"],true);server.reply(&request,first_status,None).unwrap();break;}thread::sleep(Duration::from_millis(2));});
        let first=drive(t.path(),&id,false).unwrap();assert_eq!(first["outcome"],"cancelled");worker.join().unwrap();
        let old_run=first["checkpoint"]["session_run"].as_str().unwrap().to_owned();let old_id=first["checkpoint"]["steps"][0]["request_id"].as_str().unwrap().to_owned();
        let mut server=crate::debug_control::Server::new(t.path()).unwrap();
        let recovered=recover(t.path(),&id).unwrap();assert_eq!(recovered["outcome"],"rebound");assert_eq!(recovered["checkpoint"]["steps"][0]["original_run"],old_run);assert_eq!(recovered["checkpoint"]["steps"][0]["request_id"],old_id);
        assert!(!t.path().join("Data/config/debug-control/request.json").exists());
        let worker=thread::spawn(move||loop{if let Ok(Some(request))=server.poll(){request.expected_boundary.as_ref().unwrap().check(&status).unwrap();server.reply(&request,status,None).unwrap();break;}thread::sleep(Duration::from_millis(2));});
        let finished=drive(t.path(),&id,true).unwrap();assert_eq!(finished["outcome"],"completed");assert_eq!(finished["checkpoint"]["steps"][0]["request_id"],old_id);worker.join().unwrap();
    }
    #[test]
    fn cross_run_recovery_refuses_pending_action_and_host_dependent_remaining_steps(){
        for commands in [vec!["step 1","step 2"],vec!["step 1","audio-report"],vec!["audio-start","step 2"]] {
            let (t,server,id)=fixture(&commands);let dir=run_dir(t.path(),&id).unwrap();let (mut checkpoint,_)=inspect(t.path(),&id).unwrap();let recipe=load(t.path(),"demo").unwrap();
            let deadline=now()+30_000;checkpoint.deadline_unix_ms=Some(deadline);checkpoint.steps[0].state="pending".into();checkpoint.steps[0].deadline_unix_ms=Some(deadline);publish(&dir,&checkpoint,&recipe).unwrap();
            if commands[1]=="audio-report" || commands[0]=="audio-start" {
                checkpoint.steps[0].state="completed".into();
                checkpoint.steps[0].result=Some(json!({"kind":"ffta-debug-control-result","ok":true,"run":checkpoint.session_run,"id":checkpoint.steps[0].request_id,"deadline_unix_ms":deadline,"request_sha256":crate::debug_control::planned_fingerprint(recipe.steps[0].clone(),&checkpoint.session_run,&checkpoint.steps[0].request_id,deadline,&checkpoint.manifest_sha256).unwrap(),"status":{"paused":true,"remaining":0,"state_sha256":"a".repeat(64),"frame":1,"clock":100,"backend":"interp","cheat_armed":{"active":false},"replay":{"active":false},"ui_trace":{"has_trace":false}}}));publish(&dir,&checkpoint,&recipe).unwrap();
            }
            drop(server);let _new=crate::debug_control::Server::new(t.path()).unwrap();let before=fs::read(dir.join("checkpoint.json")).unwrap();
            assert!(recover(t.path(),&id).is_err());assert_eq!(fs::read(dir.join("checkpoint.json")).unwrap(),before);assert!(!t.path().join("Data/config/debug-control/request.json").exists());
        }
    }
    #[test]
    fn cross_run_scope_requires_closed_prior_collectors_and_new_suffix_collectors(){
        let parse=|commands:&[&str]|commands.iter().map(|s|super::super::super::action(s).unwrap()).collect::<Vec<_>>();
        assert!(recovery_scope(&parse(&["status","audio-start","step 1","audio-report"]),1).is_ok());
        assert!(recovery_scope(&parse(&["status","code-start","step 1","code-report"]),1).is_ok());
        assert!(recovery_scope(&parse(&["status","gameplay-start","step 1","gameplay-report"]),1).is_ok());
        assert!(recovery_scope(&parse(&["status","ui-start","step 1","ui-report"]),1).is_ok());
        assert!(recovery_scope(&parse(&["status","graphics","graphics --compare-last"]),1).is_ok());
        assert!(recovery_scope(&parse(&["audio-start","step 1","audio-report"]),1).is_err());
        for (start,report) in [("audio-start","audio-report"),("code-start","code-report"),("gameplay-start","gameplay-report"),("ui-start","ui-report")] {
            assert!(recovery_scope(&parse(&[start,"step 1",report,"step 1"]),3).is_ok());
            assert!(recovery_scope(&parse(&[start,"step 1",report,"step 1"]),2).is_err());
            assert!(recovery_scope(&parse(&[report,"step 1"]),1).is_err());
            assert!(recovery_scope(&parse(&[start,start,report,"step 1"]),3).is_err());
            assert!(recovery_scope(&parse(&[start,report,report,"step 1"]),2).is_err());
        }
        assert!(recovery_scope(&parse(&["status","audio-report"]),1).is_err());
        assert!(recovery_scope(&parse(&["status","graphics --compare-last"]),1).is_err());
        assert!(recovery_scope(&parse(&["status","content list"]),1).is_err());
        assert!(recovery_scope(&parse(&["status","content refresh","content list","profile select chosen"]),1).is_ok());
        assert!(recovery_scope(&parse(&["status","cheats list"]),1).is_err());
        assert!(recovery_scope(&parse(&["status","cheats refresh","cheats list"]),1).is_ok());
    }
    #[test]
    fn cross_run_recovery_preserves_completed_collector_reports_without_repeating_them(){
        for (start,report) in [("audio-start","audio-report"),("code-start","code-report"),("gameplay-start","gameplay-report"),("ui-start","ui-report")] {
            let (t,mut server,id)=fixture(&[start,report,"step 1"]);let root=t.path().to_owned();let worker_id=id.clone();
            let status=json!({"paused":true,"remaining":0,"state_sha256":"a".repeat(64),"frame":1,"clock":100,"backend":"interp","cheat_armed":{"active":false},"replay":{"active":false},"ui_trace":{"has_trace":false},"report":"retained"});let first_status=status.clone();
            let worker=thread::spawn(move||{let mut seen=0;while seen<2{if let Ok(Some(request))=server.poll(){seen+=1;if seen==2{assert_eq!(cancel_command(&root,&worker_id)["ok"],true);}let _=server.reply(&request,first_status.clone(),None);}thread::sleep(Duration::from_millis(2));}});
            let first=drive(t.path(),&id,false).unwrap();assert_eq!(first["outcome"],"cancelled");worker.join().unwrap();let previous=first["checkpoint"]["steps"].as_array().unwrap()[..2].to_vec();
            let mut server=crate::debug_control::Server::new(t.path()).unwrap();assert_eq!(recover(t.path(),&id).unwrap()["outcome"],"rebound");
            let worker=thread::spawn(move||loop{if let Ok(Some(request))=server.poll(){assert!(matches!(request.command,Action::Step{..}));request.expected_boundary.as_ref().unwrap().check(&status).unwrap();let _=server.reply(&request,status,None);break;}thread::sleep(Duration::from_millis(2));});
            let done=drive(t.path(),&id,true).unwrap();assert_eq!(done["outcome"],"completed");for (i,old) in previous.iter().enumerate(){assert_eq!(done["checkpoint"]["steps"][i]["result"],old["result"]);assert_eq!(done["checkpoint"]["steps"][i]["request_id"],old["request_id"]);}worker.join().unwrap();
        }
    }
    #[test]
    fn cross_run_recovery_can_start_and_finish_a_new_collector(){
        let (t,mut server,id)=fixture(&["status","audio-start","step 1","audio-report"]);let root=t.path().to_owned();let worker_id=id.clone();
        let status=json!({"paused":true,"remaining":0,"state_sha256":"a".repeat(64),"frame":1,"clock":100,"backend":"interp","cheat_armed":{"active":false},"replay":{"active":false},"ui_trace":{"has_trace":false}});
        let first_status=status.clone();
        let worker=thread::spawn(move||loop{if let Ok(Some(request))=server.poll_query(){assert_eq!(cancel_command(&root,&worker_id)["ok"],true);server.reply(&request,first_status,None).unwrap();break;}thread::sleep(Duration::from_millis(2));});
        let first=drive(t.path(),&id,false).unwrap();assert_eq!(first["outcome"],"cancelled");worker.join().unwrap();
        let mut server=crate::debug_control::Server::new(t.path()).unwrap();assert_eq!(recover(t.path(),&id).unwrap()["outcome"],"rebound");
        let worker=thread::spawn(move||{let mut seen=Vec::new();while seen.len()<3{if let Ok(Some(request))=server.poll(){if seen.is_empty(){request.expected_boundary.as_ref().unwrap().check(&status).unwrap();}else{assert!(request.expected_boundary.is_none());}seen.push(format!("{:?}",request.command));server.reply(&request,status.clone(),None).unwrap();}thread::sleep(Duration::from_millis(2));}seen});
        let done=drive(t.path(),&id,true).unwrap();assert_eq!(done["outcome"],"completed");let seen=worker.join().unwrap();
        assert!(seen[0].starts_with("AudioStart"));assert!(seen[1].starts_with("Step"));assert!(seen[2].starts_with("AudioReport"));
        assert_eq!(done["checkpoint"]["steps"][0]["result"],first["checkpoint"]["steps"][0]["result"]);
    }
    #[test]
    fn cross_run_pending_uses_only_the_exact_retained_terminal_response(){
        for first_command in ["status","step 1"] {
            let (t,mut server,id)=fixture(&[first_command,"step 1"]);let dir=run_dir(t.path(),&id).unwrap();let (mut checkpoint,_)=inspect(t.path(),&id).unwrap();let recipe=load(t.path(),"demo").unwrap();let deadline=now()+30_000;
            checkpoint.deadline_unix_ms=Some(deadline);checkpoint.steps[0].state="pending".into();checkpoint.steps[0].deadline_unix_ms=Some(deadline);publish(&dir,&checkpoint,&recipe).unwrap();
            let root=t.path().to_owned();let action=recipe.steps[0].clone();let run=checkpoint.session_run.clone();let request_id=checkpoint.steps[0].request_id.clone();
            let client=thread::spawn(move||crate::debug_control::planned_action_classified(&root,action,10_000,run,request_id,deadline).unwrap());
            let start=std::time::Instant::now();let request=loop{if let Ok(Some(request))=if first_command=="status"{server.poll_query()}else{server.poll()}{break request;}assert!(start.elapsed()<Duration::from_secs(10));thread::sleep(Duration::from_millis(2));};
            let status=json!({"paused":true,"remaining":0,"state_sha256":"a".repeat(64),"frame":1,"clock":100,"backend":"interp","cheat_armed":{"active":false},"replay":{"active":false},"ui_trace":{"has_trace":false}});
            let _=server.reply(&request,status.clone(),None);while !client.is_finished(){let _=if first_command=="status"{server.poll_query()}else{server.poll()};thread::sleep(Duration::from_millis(2));}
            let original=client.join().unwrap();assert_eq!(original["ok"],true);drop(server);let mut new_server=crate::debug_control::Server::new(t.path()).unwrap();
            let path=t.path().join("Data/config/debug-control").join(if first_command=="status"{"query-result.json"}else{"result.json"});let valid=fs::read(&path).unwrap();let before=fs::read(dir.join("checkpoint.json")).unwrap();
            for (key,value) in [("run",json!("different")),("id",json!("different")),("request_sha256",json!("different")),("operation_id",json!("different")),("deadline_unix_ms",json!(deadline+1)),("ok",json!(false)),("error",json!("refused")),("outcome",json!("timeout")),("status",Value::Null)] {let mut altered:Value=serde_json::from_slice(&valid).unwrap();altered[key]=value;fs::write(&path,serde_json::to_vec(&altered).unwrap()).unwrap();assert!(recover(t.path(),&id).is_err(),"{key}");assert_eq!(fs::read(dir.join("checkpoint.json")).unwrap(),before);}
            let mut altered:Value=serde_json::from_slice(&valid).unwrap();altered["session"]["manifest_sha256"]=json!("different");fs::write(&path,serde_json::to_vec(&altered).unwrap()).unwrap();assert!(recover(t.path(),&id).is_err());assert_eq!(fs::read(dir.join("checkpoint.json")).unwrap(),before);
            fs::write(&path,&valid).unwrap();let resolved=recover(t.path(),&id).unwrap();assert_eq!(resolved["outcome"],"resolved-pending");assert_eq!(resolved["checkpoint"]["steps"][0]["result"],original);assert_eq!(resolved["checkpoint"]["steps"][1]["state"],"planned");
            let rebound=recover(t.path(),&id).unwrap();assert_eq!(rebound["outcome"],"rebound");let stop=Arc::new(AtomicBool::new(false));let count=Arc::new(AtomicUsize::new(0));let end=stop.clone();let n=count.clone();
            let worker=thread::spawn(move||{while !end.load(Ordering::SeqCst){if let Ok(Some(request))=new_server.poll(){n.fetch_add(1,Ordering::SeqCst);request.expected_boundary.as_ref().unwrap().check(&status).unwrap();let _=new_server.reply(&request,status.clone(),None);}thread::sleep(Duration::from_millis(2));}});
            let done=drive(t.path(),&id,true).unwrap();assert_eq!(done["outcome"],"completed");assert_eq!(done["checkpoint"]["steps"][0]["result"],original);assert_eq!(count.load(Ordering::SeqCst),1);stop.store(true,Ordering::SeqCst);worker.join().unwrap();
        }
    }
    #[test]
    fn retained_pending_final_result_and_assertion_failure_are_terminal(){
        for commands in [vec!["status"],vec!["status","assert /status/frame 2"],vec!["status","wait /status/frame 2 30000 20 4"]] {
            let (t,server,id)=fixture(&commands);let dir=run_dir(t.path(),&id).unwrap();let (mut checkpoint,_)=inspect(t.path(),&id).unwrap();let recipe=load(t.path(),"demo").unwrap();let deadline=now()+30_000;
            checkpoint.deadline_unix_ms=Some(deadline);let step=&mut checkpoint.steps[0];step.state="pending".into();step.deadline_unix_ms=Some(deadline);publish(&dir,&checkpoint,&recipe).unwrap();
            let response=json!({"schema":1,"kind":"ffta-debug-control-result","ok":true,"outcome":"ok","error":null,"run":checkpoint.session_run,"id":checkpoint.steps[0].request_id,"deadline_unix_ms":deadline,"request_sha256":crate::debug_control::planned_fingerprint(recipe.steps[0].clone(),&checkpoint.session_run,&checkpoint.steps[0].request_id,deadline,&checkpoint.manifest_sha256).unwrap(),"session":{"manifest_sha256":checkpoint.manifest_sha256},"operation_id":"control.status","status":{"frame":1}});
            let path=t.path().join("Data/config/debug-control/query-result.json");fs::write(&path,serde_json::to_vec(&response).unwrap()).unwrap();drop(server);let _new=crate::debug_control::Server::new(t.path()).unwrap();let before=fs::read(dir.join("checkpoint.json")).unwrap();
            if commands.len()==2 && commands[1].starts_with("wait "){assert!(recover(t.path(),&id).is_err());assert_eq!(fs::read(dir.join("checkpoint.json")).unwrap(),before);}else{
                let resolved=recover(t.path(),&id).unwrap();let expected=if commands.len()==1{"completed"}else{"assertion-failed"};assert_eq!(resolved["outcome"],expected);assert_eq!(resolved["checkpoint"]["steps"][0]["result"],response);
            }
            assert!(!t.path().join("Data/config/debug-control/query.json").exists());assert!(!t.path().join("Data/config/debug-control/request.json").exists());
        }
    }
    #[test]
    fn retained_pending_wait_uses_the_original_completion_time_and_never_redispatches(){
        let (t,mut server,id)=fixture(&["status","wait /status/frame 2 20000 20 64","step 1"]);let dir=run_dir(t.path(),&id).unwrap();let (mut checkpoint,_)=inspect(t.path(),&id).unwrap();let recipe=load(t.path(),"demo").unwrap();let deadline=now()+30_000;
        checkpoint.deadline_unix_ms=Some(deadline);checkpoint.steps[0].state="pending".into();checkpoint.steps[0].deadline_unix_ms=Some(deadline);publish(&dir,&checkpoint,&recipe).unwrap();
        let root=t.path().to_owned();let action=recipe.steps[0].clone();let run=checkpoint.session_run.clone();let request_id=checkpoint.steps[0].request_id.clone();let client=thread::spawn(move||crate::debug_control::planned_action_classified(&root,action,10_000,run,request_id,deadline).unwrap());
        let start=std::time::Instant::now();let request=loop{if let Ok(Some(request))=server.poll_query(){break request;}assert!(start.elapsed()<Duration::from_secs(10));thread::sleep(Duration::from_millis(2));};
        let status=|frame:u64,hash:&str|json!({"paused":true,"remaining":0,"state_sha256":hash.repeat(64),"frame":frame,"clock":frame*100,"backend":"interp","cheat_armed":{"active":false},"replay":{"active":false},"ui_trace":{"has_trace":false}});
        let _=server.reply(&request,status(1,"a"),None);while !client.is_finished(){let _=server.poll_query();thread::sleep(Duration::from_millis(2));}let original=client.join().unwrap();assert_eq!(original["ok"],true);drop(server);let mut new_server=crate::debug_control::Server::new(t.path()).unwrap();
        let path=t.path().join("Data/config/debug-control/query-result.json");let valid=fs::read(&path).unwrap();let before=fs::read(dir.join("checkpoint.json")).unwrap();
        for invalid in [Value::Null,json!("invalid"),json!(deadline+1),json!(deadline.saturating_sub(120_001))] {let mut altered:Value=serde_json::from_slice(&valid).unwrap();if invalid.is_null(){altered.as_object_mut().unwrap().remove("completed_unix_ms");}else{altered["completed_unix_ms"]=invalid;}fs::write(&path,serde_json::to_vec(&altered).unwrap()).unwrap();assert!(recover(t.path(),&id).is_err());assert_eq!(fs::read(dir.join("checkpoint.json")).unwrap(),before);}
        fs::write(&path,&valid).unwrap();let resolved=recover(t.path(),&id).unwrap();assert_eq!(resolved["outcome"],"resolved-pending");assert_eq!(resolved["checkpoint"]["steps"][0]["state"],"waiting");assert_eq!(resolved["checkpoint"]["steps"][0]["result"],original);assert_eq!(resolved["checkpoint"]["steps"][0]["wait_polls"].as_u64().unwrap_or(0),0);assert_eq!(resolved["checkpoint"]["steps"][0]["wait_deadline_unix_ms"],json!((original["completed_unix_ms"].as_u64().unwrap()+20_000).min(deadline)));
        let rebound=recover(t.path(),&id).unwrap();assert_eq!(rebound["outcome"],"rebound");let worker=thread::spawn(move||{let mut queries=0;let mut actions=0;while actions<1{if let Ok(Some(request))=new_server.poll_query(){queries+=1;let value=if queries==1{let v=status(1,"a");request.expected_boundary.as_ref().unwrap().check(&v).unwrap();v}else{status(2,"b")};let _=new_server.reply(&request,value,None);}if let Ok(Some(request))=new_server.poll(){actions+=1;let _=new_server.reply(&request,status(3,"c"),None);}thread::sleep(Duration::from_millis(2));}(queries,actions)});
        let done=drive(t.path(),&id,true).unwrap();assert_eq!(done["outcome"],"completed");assert_eq!(done["checkpoint"]["steps"][0]["result"],original);assert_eq!(done["checkpoint"]["steps"][0]["wait_polls"],2);assert_eq!(worker.join().unwrap(),(2,1));
    }
    #[test]
    fn retained_pending_wait_expiry_stops_without_dispatch(){
        let (t,server,id)=fixture(&["status","wait /status/frame 2 20 20 4","step 1"]);let dir=run_dir(t.path(),&id).unwrap();let (mut checkpoint,_)=inspect(t.path(),&id).unwrap();let recipe=load(t.path(),"demo").unwrap();let deadline=now()+30_000;
        checkpoint.deadline_unix_ms=Some(deadline);checkpoint.steps[0].state="pending".into();checkpoint.steps[0].deadline_unix_ms=Some(deadline);publish(&dir,&checkpoint,&recipe).unwrap();
        let response=json!({"schema":1,"kind":"ffta-debug-control-result","ok":true,"outcome":"ok","error":null,"run":checkpoint.session_run,"id":checkpoint.steps[0].request_id,"deadline_unix_ms":deadline,"completed_unix_ms":now()-1_000,"request_sha256":crate::debug_control::planned_fingerprint(recipe.steps[0].clone(),&checkpoint.session_run,&checkpoint.steps[0].request_id,deadline,&checkpoint.manifest_sha256).unwrap(),"session":{"manifest_sha256":checkpoint.manifest_sha256},"operation_id":"control.status","status":{"frame":1}});
        fs::write(t.path().join("Data/config/debug-control/query-result.json"),serde_json::to_vec(&response).unwrap()).unwrap();drop(server);let _new=crate::debug_control::Server::new(t.path()).unwrap();
        let result=recover(t.path(),&id).unwrap();assert_eq!(result["outcome"],"wait-timeout");assert_eq!(result["checkpoint"]["steps"][0]["state"],"wait-timeout");assert_eq!(result["checkpoint"]["steps"][0]["result"],response);assert_eq!(result["checkpoint"]["steps"][1]["state"],"planned");assert!(!t.path().join("Data/config/debug-control/query.json").exists());assert!(!t.path().join("Data/config/debug-control/request.json").exists());
    }
    #[test]
    fn cross_run_recovery_preserves_a_selected_input_without_reselecting_it(){
        let (t,mut server,id)=fixture(&["content refresh","catalog select chosen","step 1"]);let root=t.path().to_owned();let worker_id=id.clone();
        let catalog=root.join("catalog.json");fs::write(&catalog,br#"{"schema":1,"cases":[]}"#).unwrap();
        let reference=crate::debug_inputs::Reference::explicit(&catalog,menu::ContentSlot::Catalog,&"a".repeat(64)).unwrap();
        let anchors=crate::debug_choices::Anchors{app:root.clone(),config:root.join("Data/config")};let pin=crate::debug_choices::SavedInput::capture(menu::ContentSlot::Catalog,&reference,&anchors).unwrap();
        let identity=digest(&serde_json::to_vec(&pin).unwrap());
        let status=json!({"paused":true,"remaining":0,"state_sha256":"a".repeat(64),"frame":1,"clock":100,"backend":"interp","cheat_armed":{"active":false},"replay":{"active":false},"ui_trace":{"has_trace":false},"content_picker":{"busy":false,"inputs":[{"slot":menu::ContentSlot::Catalog.label(),"id":"chosen","sha256":pin.sha256.clone(),"recovery_identity":identity}]},"recovery_input":pin});
        let first_status=status.clone();let worker=thread::spawn(move||{let mut n=0;while n<2{if let Ok(Some(request))=server.poll(){n+=1;if n==2{assert_eq!(cancel_command(&root,&worker_id)["ok"],true);}let _=server.reply(&request,first_status.clone(),None);}thread::sleep(Duration::from_millis(2));}});
        let first=drive(t.path(),&id,false).unwrap();assert_eq!(first["outcome"],"cancelled");worker.join().unwrap();let old=first["checkpoint"]["steps"][1]["result"].clone();
        let recipe=load(t.path(),"demo").unwrap();let mut changed=first["checkpoint"].clone();changed["steps"][1]["result"]["status"]["content_picker"]["inputs"][0]["id"]=json!("different");let altered:Checkpoint=serde_json::from_value(changed).unwrap();assert!(boundary_for_prefix(&altered,&recipe,2).is_err());
        let mut legacy=first["checkpoint"].clone();legacy["steps"][1]["result"]["status"]["recovery_input"]=Value::Null;let older:Checkpoint=serde_json::from_value(legacy).unwrap();assert!(boundary_for_prefix(&older,&recipe,2).is_err());
        let mut server=crate::debug_control::Server::new(t.path()).unwrap();let rebound=recover(t.path(),&id).unwrap();assert_eq!(rebound["outcome"],"rebound");assert_eq!(rebound["checkpoint"]["steps"][2]["expected_boundary"]["expected_inputs"][0]["slot"],"Catalog");
        let worker=thread::spawn(move||loop{if let Ok(Some(request))=server.poll(){let guard=request.expected_boundary.as_ref().unwrap();assert_eq!(guard.expected_inputs.as_ref().unwrap().len(),1);guard.check(&status).unwrap();let mut restored=status;restored["content_picker"]["inputs"][0]["id"]=json!("restored-id");let _=server.reply(&request,restored,None);break;}thread::sleep(Duration::from_millis(2));});
        let done=drive(t.path(),&id,true).unwrap();assert_eq!(done["outcome"],"completed");assert_eq!(done["checkpoint"]["steps"][1]["result"],old);worker.join().unwrap();
        let completed:Checkpoint=serde_json::from_value(done["checkpoint"].clone()).unwrap();assert!(boundary_for_prefix(&completed,&recipe,3).is_ok());
    }
    #[test]
    fn cross_run_wait_retains_action_identity_and_original_poll_budget(){
        let (t,mut server,id)=fixture(&["status","wait /status/frame 2 30000 20 64","step 1"]);let root=t.path().to_owned();let worker_id=id.clone();
        let status=|frame:u64,hash:&str|json!({"paused":true,"remaining":0,"state_sha256":hash.repeat(64),"frame":frame,"clock":frame*100,"backend":"interp","cheat_armed":{"active":false},"replay":{"active":false},"ui_trace":{"has_trace":false}});
        let first_status=status(1,"a");
        let worker=thread::spawn(move||{let mut n=0;while n<2 {if let Ok(Some(request))=server.poll_query(){n+=1;if n==2{assert_eq!(cancel_command(&root,&worker_id)["ok"],true);}let _=server.reply(&request,first_status.clone(),None);if n==2{let until=std::time::Instant::now()+Duration::from_secs(10);while server.poll_query().is_err(){assert!(std::time::Instant::now()<until);thread::sleep(Duration::from_millis(2));}}}thread::sleep(Duration::from_millis(2));}});
        let first=drive(t.path(),&id,false).unwrap();assert_eq!(first["outcome"],"cancelled");assert_eq!(first["checkpoint"]["steps"][0]["state"],"waiting");assert_eq!(first["checkpoint"]["steps"][0]["wait_polls"],1);worker.join().unwrap();
        let mut server=crate::debug_control::Server::new(t.path()).unwrap();let recovered=recover(t.path(),&id).unwrap();assert_eq!(recovered["outcome"],"rebound");assert_eq!(recovered["checkpoint"]["steps"][0]["result"],first["checkpoint"]["steps"][0]["result"]);assert_eq!(recovered["checkpoint"]["steps"][0]["wait_polls"],1);
        let worker=thread::spawn(move||{let mut queries=0;let mut actions=0;while actions<1{if let Ok(Some(request))=server.poll_query(){queries+=1;let value=if queries==1{let v=status(1,"a");request.expected_boundary.as_ref().unwrap().check(&v).unwrap();v}else{assert!(request.expected_boundary.is_none());status(2,"b")};thread::sleep(Duration::from_millis(1_200));let _=server.reply(&request,value,None);}if let Ok(Some(request))=server.poll(){actions+=1;let _=server.reply(&request,status(3,"c"),None);}thread::sleep(Duration::from_millis(2));}(queries,actions)});
        let done=drive(t.path(),&id,true).unwrap();assert_eq!(done["outcome"],"completed");assert_eq!(done["checkpoint"]["steps"][0]["wait_polls"],3);assert_eq!(done["checkpoint"]["steps"][0]["result"],first["checkpoint"]["steps"][0]["result"]);assert_eq!(worker.join().unwrap(),(2,1));
    }
    #[test]
    fn completed_wait_recovery_uses_the_observed_boundary(){
        let (t,server,id)=fixture(&["status","wait /status/frame 1 30000 20 64","step 1"]);let dir=run_dir(t.path(),&id).unwrap();let (mut checkpoint,_)=inspect(t.path(),&id).unwrap();let recipe=load(t.path(),"demo").unwrap();let deadline=now()+30_000;
        let status=|frame:u64,hash:&str|json!({"paused":true,"remaining":0,"state_sha256":hash.repeat(64),"frame":frame,"clock":frame*100,"backend":"interp","cheat_armed":{"active":false},"replay":{"active":false},"ui_trace":{"has_trace":false}});
        checkpoint.deadline_unix_ms=Some(deadline);let step=&mut checkpoint.steps[0];step.state="completed".into();step.deadline_unix_ms=Some(deadline);step.wait_deadline_unix_ms=Some(deadline);step.wait_polls=1;
        step.result=Some(json!({"kind":"ffta-debug-control-result","ok":true,"run":checkpoint.session_run,"id":step.request_id,"deadline_unix_ms":deadline,"request_sha256":crate::debug_control::planned_fingerprint(recipe.steps[0].clone(),&checkpoint.session_run,&step.request_id,deadline,&checkpoint.manifest_sha256).unwrap(),"status":status(0,"a")}));
        step.wait_observation=Some(WaitObservation{poll:1,id:"observed".into(),response_sha256:"b".repeat(64),present:true,actual:json!(1),matched:true,boundary:crate::debug_control::ExpectedBoundary::from_status(&status(1,"b")).ok(),inputs:None});publish(&dir,&checkpoint,&recipe).unwrap();drop(server);
        let _server=crate::debug_control::Server::new(t.path()).unwrap();let rebound=recover(t.path(),&id).unwrap();
        assert_eq!(rebound["checkpoint"]["steps"][1]["expected_boundary"]["frame"],1);assert_eq!(rebound["checkpoint"]["steps"][1]["expected_boundary"]["state_sha256"],"b".repeat(64));
    }
    #[test]
    fn cross_run_wait_refuses_an_unconfirmed_observation_or_exhausted_budget(){
        for case in ["missing","exhausted","stale"]{
            let (t,server,id)=fixture(&["status","wait /status/frame 1 30000 20 4","step 1"]);let dir=run_dir(t.path(),&id).unwrap();let (mut checkpoint,_)=inspect(t.path(),&id).unwrap();let recipe=load(t.path(),"demo").unwrap();let deadline=now()+30_000;
            checkpoint.deadline_unix_ms=Some(deadline);let step=&mut checkpoint.steps[0];step.state="waiting".into();step.deadline_unix_ms=Some(deadline);step.wait_deadline_unix_ms=Some(deadline);step.wait_polls=match case{"exhausted"=>4,"stale"=>2,_=>1};
            step.result=Some(json!({"kind":"ffta-debug-control-result","ok":true,"run":checkpoint.session_run,"id":step.request_id,"deadline_unix_ms":deadline,"request_sha256":crate::debug_control::planned_fingerprint(recipe.steps[0].clone(),&checkpoint.session_run,&step.request_id,deadline,&checkpoint.manifest_sha256).unwrap(),"status":{"paused":true,"remaining":0,"state_sha256":"a".repeat(64),"frame":1,"clock":100,"backend":"interp","cheat_armed":{"active":false},"replay":{"active":false},"ui_trace":{"has_trace":false}}}));
            if case=="stale" {step.wait_observation=Some(WaitObservation{poll:1,id:"old-observation".into(),response_sha256:"b".repeat(64),present:true,actual:json!(0),matched:false,boundary:crate::debug_control::ExpectedBoundary::from_status(&step.result.as_ref().unwrap()["status"]).ok(),inputs:None});}
            publish(&dir,&checkpoint,&recipe).unwrap();
            drop(server);let _new=crate::debug_control::Server::new(t.path()).unwrap();let before=fs::read(dir.join("checkpoint.json")).unwrap();assert!(recover(t.path(),&id).is_err());assert_eq!(fs::read(dir.join("checkpoint.json")).unwrap(),before);assert!(!t.path().join("Data/config/debug-control/query.json").exists());
        }
    }
    #[test]
    fn cross_run_guarded_wait_remains_cancellable_while_the_server_is_slow(){
        let (t,server,id)=fixture(&["status","wait /status/frame 1 30000 20 4","step 1"]);let dir=run_dir(t.path(),&id).unwrap();let (mut checkpoint,_)=inspect(t.path(),&id).unwrap();let recipe=load(t.path(),"demo").unwrap();let deadline=now()+30_000;
        checkpoint.deadline_unix_ms=Some(deadline);let step=&mut checkpoint.steps[0];step.state="waiting".into();step.deadline_unix_ms=Some(deadline);step.wait_deadline_unix_ms=Some(deadline);step.wait_polls=1;
        let status=json!({"paused":true,"remaining":0,"state_sha256":"a".repeat(64),"frame":0,"clock":100,"backend":"interp","cheat_armed":{"active":false},"replay":{"active":false},"ui_trace":{"has_trace":false}});
        step.result=Some(json!({"kind":"ffta-debug-control-result","ok":true,"run":checkpoint.session_run,"id":step.request_id,"deadline_unix_ms":deadline,"request_sha256":crate::debug_control::planned_fingerprint(recipe.steps[0].clone(),&checkpoint.session_run,&step.request_id,deadline,&checkpoint.manifest_sha256).unwrap(),"status":status}));
        step.wait_observation=Some(WaitObservation{poll:1,id:"observed".into(),response_sha256:"b".repeat(64),present:true,actual:json!(0),matched:false,boundary:crate::debug_control::ExpectedBoundary::from_status(&status).ok(),inputs:None});publish(&dir,&checkpoint,&recipe).unwrap();drop(server);
        let mut server=crate::debug_control::Server::new(t.path()).unwrap();recover(t.path(),&id).unwrap();let (sent,received)=std::sync::mpsc::channel();let (release,done)=std::sync::mpsc::channel();
        let worker=thread::spawn(move||{loop{if let Ok(Some(request))=server.poll_query(){assert!(request.expected_boundary.is_some());sent.send(()).unwrap();let _=done.recv_timeout(Duration::from_secs(5));break;}thread::sleep(Duration::from_millis(2));}});
        let root=t.path().to_owned();let run_id=id.clone();let runner=thread::spawn(move||drive(&root,&run_id,true).unwrap());received.recv_timeout(Duration::from_secs(5)).unwrap();assert_eq!(cancel_command(t.path(),&id)["ok"],true);
        let result=runner.join().unwrap();assert_eq!(result["outcome"],"cancelled");assert_eq!(result["checkpoint"]["steps"][0]["state"],"waiting");assert_eq!(result["checkpoint"]["steps"][0]["wait_polls"],2);assert_eq!(result["checkpoint"]["steps"][1]["state"],"planned");release.send(()).unwrap();worker.join().unwrap();
    }
    #[test]
    fn ordinary_wait_remains_cancellable_during_slow_observation(){
        for local in [false,true] {
            let (t,mut server,id)=fixture(&["status","wait /status/frame 1 30000 20 4","step 1"]);
            let (sent,received)=std::sync::mpsc::channel();let (release,done)=std::sync::mpsc::channel();
            let worker=thread::spawn(move||{let mut count=0;loop{if let Ok(Some(request))=server.poll_query(){count+=1;if count==1{server.reply(&request,json!({"frame":0}),None).unwrap();}else{assert!(request.expected_boundary.is_none());sent.send(()).unwrap();let _=done.recv_timeout(Duration::from_secs(5));break;}}thread::sleep(Duration::from_millis(2));}});
            let root=t.path().to_owned();let run_id=id.clone();let flag=Arc::new(AtomicBool::new(false));let runner_flag=flag.clone();
            let runner=thread::spawn(move||drive_with_cancel(&root,&run_id,false,local.then_some(runner_flag.as_ref())).unwrap());
            received.recv_timeout(Duration::from_secs(5)).unwrap();let start=std::time::Instant::now();
            if local{flag.store(true,Ordering::SeqCst);}else{assert_eq!(cancel_command(t.path(),&id)["ok"],true);}
            let result=runner.join().unwrap();assert!(start.elapsed()<Duration::from_secs(2));assert_eq!(result["outcome"],"cancelled");assert_eq!(result["checkpoint"]["steps"][0]["state"],"waiting");assert_eq!(result["checkpoint"]["steps"][1]["state"],"planned");assert!(!t.path().join("Data/config/debug-control/request.json").exists());release.send(()).unwrap();worker.join().unwrap();
        }
    }
    #[test]
    fn slow_wait_observation_never_sends_a_later_step_after_expiry(){
        let (t,mut server,id)=fixture(&["status","wait /status/frame 1 1500 20 4","step 1"]);
        let worker=thread::spawn(move||{let mut count=0;loop{if let Ok(Some(request))=server.poll_query(){count+=1;if count==1{server.reply(&request,json!({"frame":0}),None).unwrap();}else{thread::sleep(Duration::from_millis(1_700));let _=server.reply(&request,json!({"frame":1}),None);break;}}thread::sleep(Duration::from_millis(2));}});
        let result=drive(t.path(),&id,false).unwrap();assert!(matches!(result["outcome"].as_str(),Some("unconfirmed"|"wait-timeout")));assert_eq!(result["checkpoint"]["steps"][1]["state"],"planned");assert!(!t.path().join("Data/config/debug-control/request.json").exists());worker.join().unwrap();
    }
    #[test]
    fn cancellation_between_steps_resumes_without_repeating_completed_step(){
        let (t,server,id)=fixture(&["step 1","step 2"]);let stop=Arc::new(AtomicBool::new(false));let count=Arc::new(AtomicUsize::new(0));
        let worker=serve(server,t.path().to_owned(),id.clone(),true,stop.clone(),count.clone());
        let first=drive(t.path(),&id,false).unwrap();assert_eq!(first["outcome"],"cancelled");assert_eq!(count.load(Ordering::SeqCst),1);assert_eq!(first["checkpoint"]["steps"][0]["state"],"completed");
        let resumed=drive(t.path(),&id,true).unwrap();assert_eq!(resumed["outcome"],"completed");assert_eq!(count.load(Ordering::SeqCst),2);
        assert_eq!(drive(t.path(),&id,true).unwrap()["outcome"],"completed");assert_eq!(count.load(Ordering::SeqCst),2);
        stop.store(true,Ordering::SeqCst);worker.join().unwrap();
    }
    #[test]
    fn read_query_then_step_completes_in_order(){
        let (t,server,id)=fixture(&["status","step 1"]);let stop=Arc::new(AtomicBool::new(false));let count=Arc::new(AtomicUsize::new(0));
        let worker=serve(server,t.path().to_owned(),id.clone(),false,stop.clone(),count.clone());
        let done=drive(t.path(),&id,false).unwrap();assert_eq!(done["outcome"],"completed");assert_eq!(count.load(Ordering::SeqCst),2);
        assert_eq!(done["checkpoint"]["steps"][0]["result"]["operation_id"],"control.status");assert_eq!(done["checkpoint"]["steps"][1]["result"]["operation_id"],"control.step");
        stop.store(true,Ordering::SeqCst);worker.join().unwrap();
    }
    #[test]
    fn lost_result_recovers_same_identity_without_second_execution(){
        let (t,server,id)=fixture(&["step 1","assert /status/frame 1","step 2"]);let stop=Arc::new(AtomicBool::new(false));let count=Arc::new(AtomicUsize::new(0));
        let worker=serve(server,t.path().to_owned(),id.clone(),false,stop.clone(),count.clone());
        let dir=run_dir(t.path(),&id).unwrap();let (mut c,_)=inspect(t.path(),&id).unwrap();let recipe=load(t.path(),"demo").unwrap();let deadline=now()+30_000;c.deadline_unix_ms=Some(deadline);c.steps[0].state="pending".into();c.steps[0].deadline_unix_ms=Some(deadline);publish(&dir,&c,&recipe).unwrap();
        let reply=crate::debug_control::planned_action_classified(t.path(),recipe.steps[0].clone(),10_000,c.session_run.clone(),c.steps[0].request_id.clone(),deadline).unwrap();assert_eq!(reply["ok"],true);assert_eq!(count.load(Ordering::SeqCst),1);
        // Simulate loss after the server reply but before checkpoint publication.
        let resumed=drive(t.path(),&id,true).unwrap();assert_eq!(resumed["outcome"],"completed");assert_eq!(resumed["checkpoint"]["steps"][0]["checks"][0]["matched"],true);assert_eq!(count.load(Ordering::SeqCst),2);
        stop.store(true,Ordering::SeqCst);worker.join().unwrap();
    }
    #[test]
    fn ambiguous_read_query_and_unbounded_replay_never_dispatch(){
        let (t,server,id)=fixture(&["status"]);let dir=run_dir(t.path(),&id).unwrap();let (mut c,_)=inspect(t.path(),&id).unwrap();let recipe=load(t.path(),"demo").unwrap();let deadline=now()+30_000;c.deadline_unix_ms=Some(deadline);c.steps[0].state="pending".into();c.steps[0].deadline_unix_ms=Some(deadline);publish(&dir,&c,&recipe).unwrap();
        assert_eq!(drive(t.path(),&id,true).unwrap()["outcome"],"unconfirmed");assert!(!t.path().join("Data/config/debug-control/query.json").exists());drop(server);
        let (t,server,_)=fixture(&["status"]);let rejected=super::super::super::save_command(t.path(),"unbounded",&["replay run".into()]);assert_eq!(rejected["ok"],false);assert!(!t.path().join("Data/config/debug-control/request.json").exists());drop(server);
    }
    #[test]
    fn cancellation_with_unresolved_step_is_not_reported_as_completed(){
        let (t,server,id)=fixture(&["step 1"]);let dir=run_dir(t.path(),&id).unwrap();let (mut c,_)=inspect(t.path(),&id).unwrap();let recipe=load(t.path(),"demo").unwrap();let deadline=now()+30_000;
        c.deadline_unix_ms=Some(deadline);c.steps[0].state="pending".into();c.steps[0].deadline_unix_ms=Some(deadline);publish(&dir,&c,&recipe).unwrap();
        assert_eq!(cancel_command(t.path(),&id)["outcome"],"cancellation-requested");assert_eq!(drive(t.path(),&id,false).unwrap()["outcome"],"cancellation-pending");
        assert!(!t.path().join("Data/config/debug-control/request.json").exists());drop(server);
    }
    #[test]
    fn expired_deadline_refuses_new_dispatch(){
        let (t,server,id)=fixture(&["step 1"]);let dir=run_dir(t.path(),&id).unwrap();let (mut c,_)=inspect(t.path(),&id).unwrap();let recipe=load(t.path(),"demo").unwrap();c.deadline_unix_ms=Some(now().saturating_sub(1));publish(&dir,&c,&recipe).unwrap();
        let result=drive(t.path(),&id,false).unwrap();assert_eq!(result["outcome"],"timeout");assert_eq!(result["ok"],false);assert!(!t.path().join("Data/config/debug-control/request.json").exists());drop(server);
    }
    #[test]
    fn refused_step_stops_the_batch_and_is_not_reissued(){
        let (t,mut server,id)=fixture(&["step 1","step 2"]);let stop=Arc::new(AtomicBool::new(false));let count=Arc::new(AtomicUsize::new(0));let worker_stop=stop.clone();let worker_count=count.clone();
        let worker=thread::spawn(move||{while !worker_stop.load(Ordering::SeqCst){if let Ok(Some(request))=server.poll(){worker_count.fetch_add(1,Ordering::SeqCst);let _=server.reply(&request,json!({"paused":true}),Some("Synthetic refusal".into()));}thread::sleep(Duration::from_millis(2));}});
        let result=drive(t.path(),&id,false).unwrap();assert_eq!(result["outcome"],"failed");assert_eq!(result["ok"],false);assert_eq!(count.load(Ordering::SeqCst),1);
        assert_eq!(drive(t.path(),&id,true).unwrap()["outcome"],"failed");assert_eq!(count.load(Ordering::SeqCst),1);stop.store(true,Ordering::SeqCst);worker.join().unwrap();
    }
    #[test]
    fn output_limit_records_terminal_identity_and_stops_later_steps(){
        let commands=vec!["step 1";10];let (t,mut server,id)=fixture(&commands);let stop=Arc::new(AtomicBool::new(false));let count=Arc::new(AtomicUsize::new(0));let worker_stop=stop.clone();let worker_count=count.clone();
        let worker=thread::spawn(move||{let padding="x".repeat(110_000);while !worker_stop.load(Ordering::SeqCst){if let Ok(Some(request))=server.poll(){worker_count.fetch_add(1,Ordering::SeqCst);let _=server.reply(&request,json!({"paused":true,"padding":padding}),None);}thread::sleep(Duration::from_millis(2));}});
        let result=drive(t.path(),&id,false).unwrap();assert_eq!(result["outcome"],"failed");let count_after=count.load(Ordering::SeqCst);assert!((2..10).contains(&count_after));
        assert_eq!(result["checkpoint"]["steps"][count_after-1]["result"]["kind"],"ffta-recipe-output-limit");assert_eq!(drive(t.path(),&id,true).unwrap()["outcome"],"failed");assert_eq!(count.load(Ordering::SeqCst),count_after);
        stop.store(true,Ordering::SeqCst);worker.join().unwrap();
    }
    #[test]
    fn assertions_pass_or_stop_without_reissuing_the_completed_action(){
        let (t,server,id)=fixture(&["step 1","assert /status/paused true","assert /status/frame 1","step 2","assert /status/frame 2"]);let stop=Arc::new(AtomicBool::new(false));let count=Arc::new(AtomicUsize::new(0));let worker=serve(server,t.path().to_owned(),id.clone(),false,stop.clone(),count.clone());
        let done=drive(t.path(),&id,false).unwrap();assert_eq!(done["outcome"],"completed");assert_eq!(done["checkpoint"]["steps"][0]["checks"].as_array().unwrap().len(),2);assert_eq!(count.load(Ordering::SeqCst),2);stop.store(true,Ordering::SeqCst);worker.join().unwrap();
        let (t,server,id)=fixture(&["step 1","assert /status/paused false","step 2"]);let stop=Arc::new(AtomicBool::new(false));let count=Arc::new(AtomicUsize::new(0));let worker=serve(server,t.path().to_owned(),id.clone(),false,stop.clone(),count.clone());
        let failed=drive(t.path(),&id,false).unwrap();assert_eq!(failed["outcome"],"assertion-failed");assert_eq!(failed["checkpoint"]["steps"][0]["checks"][0]["matched"],false);assert_eq!(count.load(Ordering::SeqCst),1);
        assert_eq!(drive(t.path(),&id,true).unwrap()["outcome"],"assertion-failed");assert_eq!(count.load(Ordering::SeqCst),1);
        let file=run_dir(t.path(),&id).unwrap().join("checkpoint.json");let mut changed:Value=serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();changed["steps"][0]["checks"][0]["matched"]=json!(true);fs::write(file,serde_json::to_vec(&changed).unwrap()).unwrap();assert!(inspect(t.path(),&id).is_err());stop.store(true,Ordering::SeqCst);worker.join().unwrap();
    }
    #[test]
    fn waits_observe_status_and_stop_at_the_bound_without_repeating_actions(){
        let (t,server,id)=fixture(&["step 1","wait /status/frame 3 3000 20 5","step 2"]);let stop=Arc::new(AtomicBool::new(false));let count=Arc::new(AtomicUsize::new(0));let worker=serve(server,t.path().to_owned(),id.clone(),false,stop.clone(),count.clone());
        let done=drive(t.path(),&id,false).unwrap();assert_eq!(done["outcome"],"completed");assert_eq!(done["checkpoint"]["steps"][0]["wait_polls"],2);assert_eq!(done["checkpoint"]["steps"][0]["wait_observation"]["matched"],true);assert_eq!(count.load(Ordering::SeqCst),4);
        assert_eq!(drive(t.path(),&id,true).unwrap()["outcome"],"completed");assert_eq!(count.load(Ordering::SeqCst),4);stop.store(true,Ordering::SeqCst);worker.join().unwrap();
        let (t,server,id)=fixture(&["step 1","wait /status/frame 99 3000 20 2","step 2"]);let stop=Arc::new(AtomicBool::new(false));let count=Arc::new(AtomicUsize::new(0));let worker=serve(server,t.path().to_owned(),id.clone(),false,stop.clone(),count.clone());
        let failed=drive(t.path(),&id,false).unwrap();assert_eq!(failed["outcome"],"wait-timeout");assert_eq!(count.load(Ordering::SeqCst),3);assert_eq!(drive(t.path(),&id,true).unwrap()["outcome"],"wait-timeout");assert_eq!(count.load(Ordering::SeqCst),3);stop.store(true,Ordering::SeqCst);worker.join().unwrap();
    }
    #[test]
    fn resumed_wait_does_not_repeat_the_completed_action(){
        let (t,server,id)=fixture(&["step 1","wait /status/frame 2 3000 20 5","step 2"]);let stop=Arc::new(AtomicBool::new(false));let count=Arc::new(AtomicUsize::new(0));let worker=serve(server,t.path().to_owned(),id.clone(),true,stop.clone(),count.clone());
        let cancelled=drive(t.path(),&id,false).unwrap();assert_eq!(cancelled["outcome"],"cancelled");assert_eq!(cancelled["checkpoint"]["steps"][0]["state"],"waiting");assert_eq!(count.load(Ordering::SeqCst),1);
        let done=drive(t.path(),&id,true).unwrap();assert_eq!(done["outcome"],"completed");assert_eq!(count.load(Ordering::SeqCst),3);stop.store(true,Ordering::SeqCst);worker.join().unwrap();
    }

    #[test]
    fn local_cancellation_stops_waiting_for_an_unacknowledged_action(){
        let (t,mut server,id)=fixture(&["step 1","step 2"]);let flag=Arc::new(AtomicBool::new(false));let runner_flag=flag.clone();let root=t.path().to_owned();let run_id=id.clone();
        let runner=thread::spawn(move||drive_with_cancel(&root,&run_id,false,Some(&runner_flag)).unwrap());
        let started=std::time::Instant::now();let request=loop{if let Ok(Some(r))=server.poll(){break r;}assert!(started.elapsed()<Duration::from_secs(10));thread::sleep(Duration::from_millis(2));};
        flag.store(true,Ordering::SeqCst);let value=runner.join().unwrap();assert_eq!(value["outcome"],"cancellation-pending");assert_eq!(value["checkpoint"]["steps"][0]["state"],"pending");assert!(run_dir(t.path(),&id).unwrap().join("cancel.json").exists());
        let _=server.reply(&request,json!({"paused":true,"frame":1,"remaining":0}),None);
        let stop=Arc::new(AtomicBool::new(false));let count=Arc::new(AtomicUsize::new(0));let worker=serve(server,t.path().to_owned(),id.clone(),false,stop.clone(),count.clone());
        let done=drive(t.path(),&id,true).unwrap();assert_eq!(done["outcome"],"completed");assert_eq!(count.load(Ordering::SeqCst),1);stop.store(true,Ordering::SeqCst);worker.join().unwrap();
    }

    #[test]
    fn catalog_actions_are_ordered_and_a_failed_selection_stops_the_recipe(){
        let commands=["content refresh","content list --limit 2","profile select chosen","cheats refresh","cheats list --limit 2","cheats show CH-0001","cheats capabilities CH-0001","step 1"];
        let (t,mut server,id)=fixture(&commands);let stop=Arc::new(AtomicBool::new(false));let seen=Arc::new(std::sync::Mutex::new(Vec::new()));let worker_stop=stop.clone();let worker_seen=seen.clone();
        let worker=thread::spawn(move||{while !worker_stop.load(Ordering::SeqCst){for query in [true,false]{let request=match if query{server.poll_query()}else{server.poll()}{Ok(v)=>v,Err(_)=>continue};if let Some(request)=request{worker_seen.lock().unwrap().push(format!("{:?}",request.command));let _=server.reply(&request,json!({"paused":true}),None);}}thread::sleep(Duration::from_millis(2));}});
        let result=drive(t.path(),&id,false).unwrap();assert_eq!(result["outcome"],"completed");let actions=seen.lock().unwrap();assert_eq!(actions.len(),8);for (actual,expected) in actions.iter().zip(["ContentRefresh","ContentList","SelectId","CheatRefresh","CheatList","CheatShow","CheatCapabilities","Step"]){assert!(actual.starts_with(expected),"{actual}");}drop(actions);
        assert!(result["checkpoint"]["steps"].as_array().unwrap().iter().all(|s|s["state"]=="completed"));assert_eq!(drive(t.path(),&id,true).unwrap()["outcome"],"completed");assert_eq!(seen.lock().unwrap().len(),8);stop.store(true,Ordering::SeqCst);worker.join().unwrap();
        let (t,mut server,id)=fixture(&["profile select missing","step 1"]);let stop=Arc::new(AtomicBool::new(false));let count=Arc::new(AtomicUsize::new(0));let end=stop.clone();let n=count.clone();let worker=thread::spawn(move||{while !end.load(Ordering::SeqCst){if let Ok(Some(request))=server.poll(){n.fetch_add(1,Ordering::SeqCst);let _=server.reply(&request,json!({"paused":true}),Some("Selection unavailable".into()));}thread::sleep(Duration::from_millis(2));}});
        let failed=drive(t.path(),&id,false).unwrap();assert_eq!(failed["outcome"],"failed");assert_eq!(failed["checkpoint"]["steps"][1]["state"],"planned");assert_eq!(drive(t.path(),&id,true).unwrap()["outcome"],"failed");assert_eq!(count.load(Ordering::SeqCst),1);stop.store(true,Ordering::SeqCst);worker.join().unwrap();
    }
    #[test]
    fn lost_catalog_selection_recovers_without_reapplying_and_ambiguous_catalog_read_stops(){
        let (t,server,id)=fixture(&["profile select chosen","step 1"]);let stop=Arc::new(AtomicBool::new(false));let count=Arc::new(AtomicUsize::new(0));let worker=serve(server,t.path().to_owned(),id.clone(),false,stop.clone(),count.clone());
        let dir=run_dir(t.path(),&id).unwrap();let (mut c,_)=inspect(t.path(),&id).unwrap();let recipe=load(t.path(),"demo").unwrap();let deadline=now()+30_000;c.deadline_unix_ms=Some(deadline);c.steps[0].state="pending".into();c.steps[0].deadline_unix_ms=Some(deadline);publish(&dir,&c,&recipe).unwrap();
        assert_eq!(crate::debug_control::planned_action_classified(t.path(),recipe.steps[0].clone(),10_000,c.session_run.clone(),c.steps[0].request_id.clone(),deadline).unwrap()["ok"],true);assert_eq!(count.load(Ordering::SeqCst),1);
        assert_eq!(drive(t.path(),&id,true).unwrap()["outcome"],"completed");assert_eq!(count.load(Ordering::SeqCst),2);stop.store(true,Ordering::SeqCst);worker.join().unwrap();
        let (t,server,id)=fixture(&["content list","step 1"]);let dir=run_dir(t.path(),&id).unwrap();let (mut c,_)=inspect(t.path(),&id).unwrap();let recipe=load(t.path(),"demo").unwrap();let deadline=now()+30_000;c.deadline_unix_ms=Some(deadline);c.steps[0].state="pending".into();c.steps[0].deadline_unix_ms=Some(deadline);publish(&dir,&c,&recipe).unwrap();assert_eq!(drive(t.path(),&id,true).unwrap()["outcome"],"unconfirmed");assert!(!t.path().join("Data/config/debug-control/query.json").exists());drop(server);
    }

    #[test]
    fn bounded_capture_actions_complete_in_order_and_duplicate_report_does_not_run(){
        let commands=["ui-start","step 1","ui-report","graphics","audio-start --frames 1 --streams mixed","audio-report","code-start --frames 1 --events 2","code-report","gameplay-start --frames 1 --events 2","gameplay-report"];
        let (t,mut server,id)=fixture(&commands);let stop=Arc::new(AtomicBool::new(false));let seen=Arc::new(std::sync::Mutex::new(Vec::new()));let end=stop.clone();let calls=seen.clone();
        let worker=thread::spawn(move||{while !end.load(Ordering::SeqCst){if let Ok(Some(request))=server.poll(){calls.lock().unwrap().push(format!("{:?}",request.command));let _=server.reply(&request,json!({"paused":true}),None);}thread::sleep(Duration::from_millis(2));}});
        let result=drive(t.path(),&id,false).unwrap();assert_eq!(result["outcome"],"completed");let actions=seen.lock().unwrap();assert_eq!(actions.len(),10);for (actual,expected) in actions.iter().zip(["UiStart","Step","UiReport","Graphics","AudioStart","AudioReport","CodeStart","CodeReport","GameplayStart","GameplayReport"]){assert!(actual.starts_with(expected),"{actual}");}drop(actions);
        assert_eq!(drive(t.path(),&id,true).unwrap()["outcome"],"completed");assert_eq!(seen.lock().unwrap().len(),10);stop.store(true,Ordering::SeqCst);worker.join().unwrap();
    }
    #[test]
    fn lost_capture_report_response_is_recovered_with_original_identity(){
        let (t,server,id)=fixture(&["ui-report","step 1"]);let stop=Arc::new(AtomicBool::new(false));let count=Arc::new(AtomicUsize::new(0));let worker=serve(server,t.path().to_owned(),id.clone(),false,stop.clone(),count.clone());
        let dir=run_dir(t.path(),&id).unwrap();let (mut c,_)=inspect(t.path(),&id).unwrap();let recipe=load(t.path(),"demo").unwrap();let deadline=now()+30_000;c.deadline_unix_ms=Some(deadline);c.steps[0].state="pending".into();c.steps[0].deadline_unix_ms=Some(deadline);publish(&dir,&c,&recipe).unwrap();
        assert_eq!(crate::debug_control::planned_action_classified(t.path(),recipe.steps[0].clone(),10_000,c.session_run.clone(),c.steps[0].request_id.clone(),deadline).unwrap()["ok"],true);assert_eq!(count.load(Ordering::SeqCst),1);
        assert_eq!(drive(t.path(),&id,true).unwrap()["outcome"],"completed");assert_eq!(count.load(Ordering::SeqCst),2);stop.store(true,Ordering::SeqCst);worker.join().unwrap();
    }

    #[test]
    fn search_not_reached_can_be_asserted_and_stops_following_steps(){
        let (t,mut server,id)=fixture(&["break 0x08000000 arm --steps 1","assert /status/probe/outcome text:hit","step 1"]);let stop=Arc::new(AtomicBool::new(false));let count=Arc::new(AtomicUsize::new(0));let end=stop.clone();let n=count.clone();
        let worker=thread::spawn(move||{while !end.load(Ordering::SeqCst){if let Ok(Some(request))=server.poll(){n.fetch_add(1,Ordering::SeqCst);let _=server.reply(&request,json!({"paused":true,"probe":{"outcome":"not-reached","steps":1,"completed_frames":0}}),None);}thread::sleep(Duration::from_millis(2));}});
        let result=drive(t.path(),&id,false).unwrap();assert_eq!(result["outcome"],"assertion-failed");assert_eq!(result["checkpoint"]["steps"][1]["state"],"planned");assert_eq!(drive(t.path(),&id,true).unwrap()["outcome"],"assertion-failed");assert_eq!(count.load(Ordering::SeqCst),1);stop.store(true,Ordering::SeqCst);worker.join().unwrap();
    }

}
