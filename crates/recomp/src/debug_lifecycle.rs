//! Frontend lifecycle metadata. No guest dispatch, filesystem or audio callback.
use crate::debug_events::{Clock,Correlation};
use serde_json::{json,Value};
use std::collections::BTreeMap;
use std::time::{Duration,Instant};

#[derive(Clone,Copy,Debug,PartialEq,Eq,PartialOrd,Ord)]
pub(crate) enum Family { Session,Machine,Step,Search,State,Recording,Replay,Workbench,Script,Dataset,Cache,Code,Gameplay,Audio,Ui,Recent,Report,Cheat }
impl Family {
    pub const ALL:[Self;18]=[Self::Session,Self::Machine,Self::Step,Self::Search,Self::State,Self::Recording,Self::Replay,Self::Workbench,Self::Script,Self::Dataset,Self::Cache,Self::Code,Self::Gameplay,Self::Audio,Self::Ui,Self::Recent,Self::Report,Self::Cheat];
    pub fn key(self)->&'static str {match self {Self::Session=>"session",Self::Machine=>"machine",Self::Step=>"step",Self::Search=>"search",Self::State=>"state",Self::Recording=>"recording",Self::Replay=>"replay",Self::Workbench=>"workbench",Self::Script=>"script",Self::Dataset=>"dataset",Self::Cache=>"cache",Self::Code=>"code",Self::Gameplay=>"gameplay",Self::Audio=>"audio",Self::Ui=>"ui",Self::Recent=>"recent",Self::Report=>"report",Self::Cheat=>"cheat"}}
}
const MAX_ACTIVE:usize=16;
const DETAIL_LIMIT:usize=4096;
const PROGRESS_INTERVAL:Duration=Duration::from_millis(250);
struct Active {context:Correlation,start_observed:bool,progress:Option<Value>,next_progress:Instant}
#[derive(Default)]
pub(crate) struct Jobs {active:BTreeMap<Family,Active>}
impl Jobs {
    /// Metadata ownership is retained even when collectors are off. The caller
    /// only builds/publishes event details when a consumer is actually active.
    pub fn begin(&mut self,family:Family,clock:&Clock,observed:bool)->Result<Correlation,String>{
        if self.active.contains_key(&family){return Err(format!("{} lifecycle job already active",family.key()));}
        if self.active.len()==MAX_ACTIVE{return Err("Lifecycle metadata job limit16 reached".into());}
        let mut context=clock.current();
        if context.job_id.is_none(){context.job_id=clock.request("job-identity").request_id;}
        self.active.insert(family,Active{context:context.clone(),start_observed:observed,progress:None,next_progress:Instant::now()});
        Ok(context)
    }
    pub fn context(&self,family:Family)->Option<Correlation>{self.active.get(&family).map(|j|j.context.clone())}
    pub fn finish(&mut self,family:Family)->Option<Correlation>{self.active.remove(&family).map(|j|j.context)}
    pub fn due(&self,family:Family)->bool{self.active.get(&family).is_some_and(|j|Instant::now()>=j.next_progress)}
    pub fn progress(&mut self,family:Family,details:Value)->Option<(Correlation,Value)>{
        let job=self.active.get_mut(&family)?;
        let now=Instant::now();if now<job.next_progress{return None;}
        job.next_progress=now+PROGRESS_INTERVAL;
        let details=bounded(details);
        if job.progress.as_ref()==Some(&details){return None;}
        job.progress=Some(details.clone());Some((job.context.clone(),details))
    }
    pub fn status(&self)->Value{
        json!({"active":self.active.iter().map(|(f,j)|json!({"family":f.key(),"request_id":j.context.request_id,"job_id":j.context.job_id,"start_observed":j.start_observed})).collect::<Vec<_>>(),
            "max_active":MAX_ACTIVE,"max_detail_bytes":DETAIL_LIMIT,"progress_interval_ms":PROGRESS_INTERVAL.as_millis(),"scope":"Frontend lifecycle metadata only; no job execution or guest state serialization"})
    }
}
pub(crate) fn bounded(details:Value)->Value{
    let bytes=serde_json::to_vec(&details).expect("JSON lifecycle details");
    if bytes.len()<=DETAIL_LIMIT{return details;}
    let text=String::from_utf8(bytes.clone()).expect("JSON UTF8");
    json!({"truncated":true,"bytes":bytes.len(),"sha256":gba_core::ffta_rom::digest(&bytes),"prefix":text.chars().take(256).collect::<String>()})
}
pub(crate) fn event(family:Family,phase:&str,details:Value)->Value{
    let outcome=details.get("outcome").and_then(Value::as_str).map(|v|v.chars().take(64).collect::<String>());
    json!({"family":family.key(),"phase":phase,"outcome":outcome,"details":bounded(details)})
}
pub(crate) fn coverage()->Value{
    let families=Family::ALL.into_iter().map(|family|{
        let (origin,frequency,requires,scope)=match family{
            Family::Session=>("frontend/session handlers","explicit capture baseline/copy/close boundaries","active common consumer; isolated copy for existing debug actions","Current run/ROM identity, copy handler result and close intent. Closing buffers are not implicitly exported; startup before capture is not reconstructed"),
            Family::Machine=>("frontend/shared control and menu","accepted control/toggle boundaries","existing pause/resume/cancel validation","Actual main machine pause/budget state; no new executor"),
            Family::Step|Family::Search=>("frontend/existing budget/probe","start/end plus changed status at most4Hz","accepted existing shared action","Original request/job context also surrounds existing execution ticks. Interruption/supersession are distinct from completion"),
            Family::State=>("frontend/state handlers","requested/result/restore boundaries","existing save/load/undo/restart/backup guards","Only existing operation outcomes, before discontinuity freeze; no observer-created state dumps or rollback"),
            Family::Recording=>("frontend/input writer","start/end plus changed writer metadata at most4Hz","existing recording checks","Expected next frame/input bytes, not claimed completed execution; legacy recording may have an unobserved start"),
            Family::Replay=>("frontend/isolated replay supervisor","start/cancel/end plus changed worker report at most4Hz","pinned profile and existing leases; UI can observe, unsaved recent buffer still refuses replay","Separate worker frame counts/run; no mutation or parity acceptance of main machine"),
            Family::Workbench|Family::Script=>("frontend/host worker owner","start/cancel/terminal, no stdout streaming","explicit existing request and adapter/runtime preflight","Actual terminal outcome, original draft/request context and stale-view provenance. Guest catalog inspection never executes guest scripts"),
            Family::Dataset|Family::Cache=>("replay worker preparation received by frontend","up to8retained preparation records/run","worker reached selected dataset attach/native binary load","Existing offline inputs attached/loaded, not new extraction/compilation. Worker host sequence/time is separate from parent publication order; initial CLI/cache build before frontend remains unavailable"),
            Family::Code=>("frontend/existing code inspector status","changed counters at most4Hz","code observer armed and supported backend","Existing selected dispatch totals/drop limits; no native block interior or complete instruction stream"),
            Family::Gameplay=>("frontend/existing gameplay inspector status","changed counters at most4Hz","existing bounded recognizer/model observer armed","Recognized existing transitions only; unknown/unsupported guest paths remain unknown"),
            Family::Audio=>("frontend/existing audio diagnostic status","changed counters at most4Hz","audio diagnostic armed; actual stream availability remains explicit","Existing block/frame/byte counters only. No new callback I/O, host clock reads in callbacks or PCM dump from this producer"),
            Family::Ui|Family::Recent=>("frontend/existing collector owner","arm/freeze/report and changed lifecycle state at most4Hz","explicit collector action","Own event counts/clock deliberately excluded to avoid feedback. Existing consumer caps and freeze policies still apply"),
            Family::Report=>("frontend/graphics and issue report handlers","explicit publication/result boundary","existing readonly report preflight","New artifact path only when the handler actually changed its output; refused generation may still publish an error artifact"),
            Family::Cheat=>("frontend guarded one-shot handlers","explicit prepare, apply and refusal boundaries","enabled collector and isolated paused interpreter session","Only manual one-shot preview/application/refusal; no maintained toggle, deactivation or guest hook chronology"),
        };
        json!({"family":family.key(),"origin":origin,"frequency":frequency,"requires":requires,"scope":scope,"state":if family==Family::Cheat{"partial"}else{"instrumented"}})
    }).collect::<Vec<_>>();
    json!({"schema":1,"kind":"ffta-runtime-event-coverage","families":families,"max_active_jobs":MAX_ACTIVE,"max_detail_bytes":DETAIL_LIMIT,"progress_interval_ms":250,"notes":"Coverage describes hook availability; platform validation is recorded separately. Payload details are bounded; request/job identity and outcome remain outside truncated details. Registry never dispatches jobs or guest instructions"})
}

#[cfg(test)]mod tests {
    use super::*;
    #[test]
    fn cheat_coverage_describes_one_shot_as_partial(){
        let value=coverage();let families=value["families"].as_array().unwrap();
        let cheat=families.iter().find(|v|v["family"]=="cheat").unwrap();
        assert_eq!(cheat["state"],"partial");assert!(cheat["scope"].as_str().unwrap().contains("one-shot"));
    }
    #[test]
    fn async_context_survives_other_requests_and_cannot_overwrite_a_live_job(){
        let clock=Clock::default();let mut jobs=Jobs::default();
        {let _request=clock.context(Correlation::request("start-1","menu-action",false));let context=jobs.begin(Family::Script,&clock,true).unwrap();assert_eq!(context.request_id.as_deref(),Some("start-1"));assert!(context.job_id.is_some());}
        let _other=clock.context(Correlation::request("later-cancel","local-control",false));
        assert!(jobs.begin(Family::Script,&clock,true).is_err());
        let context=jobs.finish(Family::Script).unwrap();assert_eq!(context.request_id.as_deref(),Some("start-1"));assert_ne!(context.job_id.as_deref(),Some("later-cancel"));assert!(jobs.finish(Family::Script).is_none());
    }
    #[test]
    fn existing_control_job_identity_and_unobserved_start_are_preserved(){
        let clock=Clock::default();let mut jobs=Jobs::default();
        let _request=clock.context(Correlation::request("actual-mailbox-step","local-control",true));
        let context=jobs.begin(Family::Step,&clock,false).unwrap();assert_eq!(context.job_id.as_deref(),Some("actual-mailbox-step"));assert_eq!(jobs.status()["active"][0]["start_observed"],false);
    }
    #[test]
    fn progress_is_bounded_throttled_deduplicated_and_terminal_is_not_throttled(){
        let clock=Clock::default();let mut jobs=Jobs::default();jobs.begin(Family::Replay,&clock,true).unwrap();
        let (_,large)=jobs.progress(Family::Replay,json!({"message":"è".repeat(10000)})).unwrap();assert_eq!(large["truncated"],true);assert!(serde_json::to_vec(&large).unwrap().len()<2048);
        assert!(!jobs.due(Family::Replay));assert!(jobs.progress(Family::Replay,json!({"changed":true})).is_none());
        let active=jobs.active.get_mut(&Family::Replay).unwrap();active.next_progress=Instant::now()-PROGRESS_INTERVAL;assert!(jobs.progress(Family::Replay,json!({"message":"è".repeat(10000)})).is_none());
        assert!(jobs.finish(Family::Replay).is_some());assert_eq!(jobs.status()["active"],json!([]));
    }
    #[test]
    fn metadata_capacity_refuses_without_losing_existing_jobs(){
        let clock=Clock::default();let mut jobs=Jobs::default();
        for family in Family::ALL.into_iter().take(MAX_ACTIVE){jobs.begin(family,&clock,false).unwrap();}
        assert!(jobs.begin(Family::Cheat,&clock,false).is_err());assert_eq!(jobs.active.len(),MAX_ACTIVE);assert!(jobs.context(Family::Session).is_some());
        assert!(serde_json::to_vec(&event(Family::Session,"baseline",json!({"rom":"public"}))).unwrap().len()<512);
    }
    #[test]
    fn collector_lifecycle_samples_do_not_observe_their_own_metadata_events(){
        let root=tempfile::tempdir().unwrap();std::fs::write(root.path().join("debug-session.json"),"{\"synthetic\":true}").unwrap();
        let machine=gba_core::Machine::new(vec![0;16]);let before=machine.save_state().unwrap();let clock=Clock::default();
        let mut ui=crate::debug_ui::Trace::default();ui.start_shared(json!({}),clock.clone()).unwrap();
        let mut recent=crate::debug_recent::Buffer::start_shared(root.path(),&machine,Default::default(),json!({}),json!(null),clock.clone()).unwrap();
        let ui_before=ui.lifecycle_status();let recent_before=recent.lifecycle_status();
        for i in 0..100{let row=clock.event(0,0,"runtime-event",event(Family::Ui,"progress",json!({"synthetic":i})));ui.accept(&row);recent.accept(&row);}
        assert_eq!(ui.lifecycle_status(),ui_before);assert_eq!(recent.lifecycle_status(),recent_before);
        assert_eq!(ui.status()["events"],100);assert_eq!(machine.save_state().unwrap(),before);
    }
}
