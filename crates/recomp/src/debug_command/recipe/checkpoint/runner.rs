//! Bounded recipe dispatch with a persisted identity before each request.
use super::*;
use crate::debug_control::Action;
use std::{fs::File,time::{SystemTime,UNIX_EPOCH}};

fn now()->u64{SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis().min(u128::from(u64::MAX)) as u64}
fn run_dir(root:&Path,id:&str)->Result<PathBuf,String>{
    if !valid_id(id){return Err("Invalid recipe run ID".into());}
    let dir=base(root,false)?.join(id);if !crate::debug_session::regular(&dir)?.is_dir(){return Err("Recipe run must be a directory".into());}Ok(dir)
}
fn lock(dir:&Path,name:&str)->Result<File,String>{
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
    Action::Status|Action::Inspect{..}|Action::Step{..}|Action::Find{..}|
    Action::ContentRefresh|Action::ContentList{..}|Action::SelectId{..}|
    Action::CheatRefresh|Action::CheatList{..}|Action::CheatShow{..}|Action::CheatCapabilities{..}|
    Action::AudioStart{..}|Action::AudioReport|Action::CodeStart{..}|Action::CodeReport|
    Action::GameplayStart{..}|Action::GameplayReport|Action::UiStart|Action::UiReport|Action::Graphics{..}
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
pub(super) fn drive_with_cancel(root:&Path,id:&str,resume:bool,local_cancel:Option<&std::sync::atomic::AtomicBool>)->Result<Value,String>{
    let dir=run_dir(root,id)?;let _run_lock=lock(&dir,".execution.lock")?;
    let (mut checkpoint,active)=inspect(root,id)?;
    if !active{return Err("Readiness does not match the original control run".into());}
    let recipe=load(root,&checkpoint.recipe_name)?;
    recipe.validate_run_frame_budget()?;
    if recipe.steps.iter().any(|a|!supported(a)){return Err("unsupported action in recipe; no action sent. Use status, inspect, bounded step, catalog discovery/selection/queries, bounded capture/report commands or bounded searches".into());}
    {
        let _gate=lock(&dir,".transition.lock")?;
        let cancel=dir.join("cancel.json");
        if resume && cancel.exists(){crate::debug_session::regular(&cancel)?;fs::remove_file(cancel).map_err(|e|e.to_string())?;}
        if checkpoint.deadline_unix_ms.is_none(){checkpoint.deadline_unix_ms=Some(now().saturating_add(u64::from(recipe.limits.duration_ms)));publish(&dir,&checkpoint,&recipe)?;}
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
                checkpoint.steps[index].state="wait-timeout".into();publish(&dir,&checkpoint,&recipe)?;
                return Ok(result(&checkpoint,"wait-timeout","The bounded wait expired; no later step was sent."));
            }
            if active_run(root,&checkpoint.manifest_sha256).ok().as_deref()!=Some(&checkpoint.session_run){return Ok(result(&checkpoint,"unconfirmed","The control run changed before the wait observation."));}
            // Reserve each read before publication: crashes cannot increase the poll budget.
            checkpoint.steps[index].wait_polls+=1;publish(&dir,&checkpoint,&recipe)?;
            let remaining=deadline.saturating_sub(now());if remaining<20{continue;}
            let timeout=remaining.min(1_000);
            let response=if let Some(flag)=local_cancel {crate::debug_control::query_action_interruptible(root,Action::Status,timeout,flag)}else{crate::debug_control::execute_action_classified(root,Action::Status,timeout)};
            let value=match response{Ok(v)=>v,Err(e)=>{if local_cancel.is_some_and(|c|c.load(std::sync::atomic::Ordering::SeqCst)){return Ok(cancelled_in_flight(root,id,&checkpoint,true));}return Ok(result(&checkpoint,"unconfirmed",&format!("Wait observation unresolved ({}; delivery {}). Resume explicitly within the original bounds.",e.message,e.delivery)));}};
            if value["outcome"]=="interrupted"{return Ok(cancelled_in_flight(root,id,&checkpoint,true));}
            if value["kind"]!="ffta-debug-control-result" || value["ok"]!=true || value["run"]!=checkpoint.session_run {return Ok(result(&checkpoint,"unconfirmed","Wait observation was not a successful response from the original run."));}
            if now()>deadline {checkpoint.steps[index].state="wait-timeout".into();publish(&dir,&checkpoint,&recipe)?;return Ok(result(&checkpoint,"wait-timeout","The bounded wait expired before the observation arrived."));}
            let id=value["id"].as_str().filter(|s|!s.is_empty()&&s.len()<=100).ok_or("Invalid wait observation identity")?.to_owned();
            let observed=value.pointer(&condition.pointer).filter(|v|super::super::valid_scalar(v));
            let matched=observed.is_some_and(|v|v==&condition.expected);
            checkpoint.steps[index].wait_observation=Some(WaitObservation{poll:checkpoint.steps[index].wait_polls,id,response_sha256:digest(&serde_json::to_vec(&value).map_err(|e|e.to_string())?),present:observed.is_some(),actual:observed.cloned().unwrap_or(Value::Null),matched});
            if matched {checkpoint.steps[index].state="completed".into();}
            publish(&dir,&checkpoint,&recipe)?;
            if !matched {std::thread::sleep(std::time::Duration::from_millis(u64::from(condition.interval_ms).min(deadline.saturating_sub(now()))));}
            continue;
        }
        let step=&checkpoint.steps[index];let deadline=step.deadline_unix_ms.unwrap();
        let wait=deadline.saturating_sub(now()).clamp(20,120_000);
        let response=if let Some(flag)=local_cancel {crate::debug_control::planned_action_interruptible(root,recipe.steps[index].clone(),wait,checkpoint.session_run.clone(),step.request_id.clone(),deadline,flag)}else{crate::debug_control::planned_action_classified(root,recipe.steps[index].clone(),wait,checkpoint.session_run.clone(),step.request_id.clone(),deadline)};
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
                    checkpoint.steps[index].state="waiting".into();
                    checkpoint.steps[index].wait_deadline_unix_ms=Some(now().saturating_add(u64::from(condition.timeout_ms)).min(deadline));
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
    fn ambiguous_read_query_and_unsupported_steps_never_dispatch(){
        let (t,server,id)=fixture(&["status"]);let dir=run_dir(t.path(),&id).unwrap();let (mut c,_)=inspect(t.path(),&id).unwrap();let recipe=load(t.path(),"demo").unwrap();let deadline=now()+30_000;c.deadline_unix_ms=Some(deadline);c.steps[0].state="pending".into();c.steps[0].deadline_unix_ms=Some(deadline);publish(&dir,&c,&recipe).unwrap();
        assert_eq!(drive(t.path(),&id,true).unwrap()["outcome"],"unconfirmed");assert!(!t.path().join("Data/config/debug-control/query.json").exists());drop(server);
        let (t,server,id)=fixture(&["replay run"]);assert!(drive(t.path(),&id,false).unwrap_err().contains("unsupported action"));assert!(!t.path().join("Data/config/debug-control/request.json").exists());drop(server);
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
