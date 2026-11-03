//! Bounded frame stepping in an isolated interactive session; no guest fields.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
mod ledger;

pub(crate) const MAX_STEP: u32 = 600;
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SelectionSlot { Replay, Catalog, CheatCatalog }
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ReplayMode { Plain, Audit, Capture, Comparison }
impl SelectionSlot {
    pub fn content_slot(self) -> menu::ContentSlot {
        match self { Self::Replay => menu::ContentSlot::Replay, Self::Catalog => menu::ContentSlot::Catalog, Self::CheatCatalog=>menu::ContentSlot::CheatCatalog }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "action", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum Action {
    Status,
    Cancel { request_id: String },
    SelectId { slot: SelectionSlot, id: String },
    ContentRefresh,
    ContentList { offset: u32, limit: u8, generation: Option<u64> },
    CheatRefresh,
    CheatList { offset:u32, limit:u8, generation:Option<String> },
    CheatShow { id:String, offset:u32, generation:Option<String> },
    CheatCapabilities { id:String, offset:u32, generation:Option<String> },
    CheatPrepare { id:String, provenance:u16, master:String, generation:String },
    CheatApply { id:String, provenance:u16, master:String, generation:String, preview:String },
    CheatArm { id:String, provenance:u16, master:String, generation:String, preview:String },
    CheatDisarm { id:Option<String> },
    CheatPresetSave { name:String },
    CheatPresetPrepare { name:String },
    CheatPresetArm { name:String, preview:String },
    Replay { mode: ReplayMode },
    Pause,
    Resume,
    Step {
        frames: u32,
        keys: u16,
    },
    AudioStart { settings: crate::debug_audio::Settings },
    AudioReport,
    CodeStart { settings: crate::debug_code::Settings },
    CodeReport,
    GameplayStart { settings: crate::debug_gameplay::Settings },
    GameplayReport,
    RecentStart { settings: crate::debug_recent::Settings },
    RecentFreeze,
    RecentReport,
    IssueReport {profile: Option<PathBuf>, symptom: String},
    UiStart,
    UiReport,
    Graphics {
        compare_last: bool,
    },
    Inspect {
        address: u32,
        length: u32,
    },
    Find {
        settings: crate::debug_probe::Settings,
    },
}
impl Action {
    fn read_query(&self)->bool{matches!(self,Self::Status|Self::ContentList{..}|Self::CheatList{..}|Self::CheatShow{..}|Self::CheatCapabilities{..})}
    fn query_channel(&self)->bool{self.read_query()||matches!(self,Self::Cancel{..})}
    fn cheat_capability(&self)->bool{matches!(self,Self::SelectId{slot:SelectionSlot::CheatCatalog,..}|Self::CheatRefresh|Self::CheatList{..}|Self::CheatShow{..}|Self::CheatCapabilities{..}|Self::CheatPrepare{..}|Self::CheatApply{..}|Self::CheatArm{..}|Self::CheatDisarm{..}|Self::CheatPresetSave{..}|Self::CheatPresetPrepare{..}|Self::CheatPresetArm{..})}
    pub(crate) fn validate(&self) -> Result<(), String> {
        if let Self::Step { frames, keys } = self {
            if !(1..=MAX_STEP).contains(frames) || *keys > 0x3ff {
                return Err("Step needs 1..600 frames and a 000..3ff active-low key mask".into());
            }
        }
        match self {
            Self::Cancel {request_id} if request_id.is_empty() || request_id.len()>100 => return Err("Cancel needs the exact pending request ID (1..100bytes)".into()),
            Self::SelectId {id,..} if id.is_empty() || id.len()>512 || id.chars().any(char::is_control) => return Err("Content ID must be 1..512 bytes without control characters".into()),
            Self::ContentList {limit,..} if !(1..=16).contains(limit) => return Err("Content page limit needs1..16".into()),
            Self::CheatList {limit,..} if !(1..=16).contains(limit) => return Err("Cheat page limit needs1..16".into()),
            Self::CheatShow{id,..}|Self::CheatCapabilities{id,..} if id.len()!=7 || !id.starts_with("CH-") || !id[3..].bytes().all(|b|b.is_ascii_digit())=>return Err("Cheat ID must be CH- followed by four digits".into()),
            Self::CheatList{generation:Some(g),..}|Self::CheatShow{generation:Some(g),..}|Self::CheatCapabilities{generation:Some(g),..} if g.len()!=64 || !g.bytes().all(|b|b.is_ascii_hexdigit())=>return Err("Cheat snapshot generation must be a SHA-256".into()),
            Self::CheatPrepare{id,master,generation,..}|Self::CheatApply{id,master,generation,..}|Self::CheatArm{id,master,generation,..}=>{
                let valid_id=|v:&str|v.len()==7 && v.starts_with("CH-") && v[3..].bytes().all(|b|b.is_ascii_digit());
                if !valid_id(id)||!valid_id(master)||generation.len()!=64||!generation.bytes().all(|b|b.is_ascii_hexdigit()){return Err("Cheat preparation needs CH IDs and a SHA-256 snapshot generation".into());}
                if let Self::CheatApply{preview,..}|Self::CheatArm{preview,..}=self {if preview.len()!=64||!preview.bytes().all(|b|b.is_ascii_hexdigit()){return Err("Apply or arm needs the exact preview SHA-256".into());}}
            },
            Self::CheatDisarm{id:Some(id)} if id.len()!=7 || !id.starts_with("CH-") || !id[3..].bytes().all(|b|b.is_ascii_digit())=>return Err("Cheat ID must be CH- followed by four digits".into()),
            Self::CheatPresetSave{name}|Self::CheatPresetPrepare{name}|Self::CheatPresetArm{name,..}=>{
                if !crate::debug_cheat_preset::name_valid(name){return Err("Preset name must contain 1..48 ASCII letters, digits, '-' or '_'".into());}
                if let Self::CheatPresetArm{preview,..}=self {if preview.len()!=64 || !preview.bytes().all(|b|b.is_ascii_hexdigit()){return Err("Preset arm needs the exact preview SHA-256".into());}}
            },
            Self::Inspect { address, length }
                if *length == 0 || *length > 256 || address.checked_add(*length - 1).is_none() =>
            {
                return Err("Inspect needs1..256 bytes without wrap".into())
            }
            Self::Find { settings } => settings.validate()?,
            Self::AudioStart { settings } => settings.validate()?,
            Self::CodeStart { settings } => settings.validate()?,
            Self::GameplayStart { settings } => settings.validate()?,
            Self::RecentStart { settings } => settings.validate()?,
            Self::IssueReport{profile,symptom}=>{
                crate::debug_issue::validate_symptom(symptom)?;
                if profile.as_ref().is_some_and(|p|p.to_string_lossy().len()>1024){return Err("Profile path exceeds1024bytes".into());}
                if serde_json::to_vec(self).map_err(|e|e.to_string())?.len()>3500{return Err("Issue request exceeds mailbox size; shorten path or symptom".into());}
            },
            _ => {}
        }
        Ok(())
    }
}
pub(crate) struct Control {
    pub remaining: u32,
    pub keys: u16,
    pub reason: &'static str,
}
impl Default for Control {
    fn default() -> Self {
        Self {
            remaining: 0,
            keys: 0x3ff,
            reason: "running",
        }
    }
}
impl Control {
    pub fn apply(&mut self, action: &Action, paused: bool, busy: bool) -> Result<bool, String> {
        action.validate()?;
        if matches!(action, Action::Status) {
            return Ok(paused);
        }
        if busy {
            return Err("Finish or cancel the replay before controlling this session".into());
        }
        match action {
            Action::Pause | Action::Cancel {..} => {
                self.remaining = 0;
                self.reason = if matches!(action,Action::Cancel{..}){"control-cancelled"}else{"pause-command"};
                Ok(true)
            }
            Action::Resume => {
                self.remaining = 0;
                self.reason = "running";
                Ok(false)
            }
            Action::Step { frames, keys } => {
                if !paused || self.remaining != 0 {
                    return Err("Pause first and wait for any pending step".into());
                }
                self.remaining = *frames;
                self.keys = *keys;
                self.reason = "stepping";
                Ok(true)
            }
            Action::SelectId{..} | Action::ContentRefresh | Action::ContentList{..} | Action::CheatRefresh | Action::CheatList{..} | Action::CheatShow{..} | Action::CheatCapabilities{..} | Action::CheatPrepare{..} | Action::CheatApply{..} | Action::CheatArm{..} | Action::CheatDisarm{..} | Action::CheatPresetSave{..} | Action::CheatPresetPrepare{..} | Action::CheatPresetArm{..} | Action::Replay{..} | Action::RecentStart{..} | Action::RecentFreeze | Action::RecentReport | Action::GameplayStart{..} | Action::GameplayReport | Action::IssueReport{..} | Action::CodeStart{..} | Action::CodeReport | Action::AudioStart{..} | Action::AudioReport | Action::UiStart | Action::UiReport | Action::Graphics { .. } | Action::Inspect { .. } | Action::Find { .. } => {
                Err("Inspection/search is owned by the isolated frontend".into())
            }
            Action::Status => unreachable!(),
        }
    }
    pub fn stop(&mut self, reason: &'static str) {
        self.remaining = 0;
        self.reason = reason;
    }
    pub fn completed_frame(&mut self, before: u64, after: u64) -> Result<(), String> {
        if self.remaining == 0 || after.checked_sub(before) != Some(1) {
            self.stop("incomplete-frame");
            return Err("Debug step did not complete exactly one guest frame".into());
        }
        self.remaining -= 1;
        if self.remaining == 0 {
            self.reason = "step-complete";
        }
        Ok(())
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Request {
    schema: u32,
    run: String,
    id: String,
    pub command: Action,
    #[serde(default)]
    deadline_unix_ms: Option<u64>,
    #[serde(default)]
    session_manifest_sha256: Option<String>,
    #[serde(skip)]
    query: bool,
}
fn unix_ms()->u64{SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis().min(u128::from(u64::MAX)) as u64}
impl Request {
    pub fn expired(&self)->bool{self.deadline_unix_ms.is_some_and(|d|unix_ms()>=d)}
    pub fn event_context(&self)->crate::debug_events::Correlation{
        crate::debug_events::Correlation::request(&self.id,"local-control",matches!(self.command,Action::Step{..}|Action::Find{..}|Action::SelectId{..}|Action::ContentRefresh|Action::CheatRefresh|Action::Replay{..}))
    }
    fn fingerprint(&self)->Result<String,String>{
        let mut bytes=serde_json::to_vec(self).map_err(|e|e.to_string())?;
        bytes.push(u8::from(self.query));Ok(gba_core::ffta_rom::digest(&bytes))
    }
}
fn stamp() -> String {
    format!(
        "{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
        std::process::id()
    )
}
fn put(dir: &Path, name: &str, value: &Value) -> Result<(), String> {
    let mut file = tempfile::NamedTempFile::new_in(dir).map_err(|e| e.to_string())?;
    file.write_all(&serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    file.persist(dir.join(name))
        .map_err(|e| e.error.to_string())?;
    Ok(())
}
fn read(path: &Path) -> Result<Value, String> {
    serde_json::from_slice(&crate::diagnostics::read(path, 128 * 1024)?).map_err(|e| e.to_string())
}
fn directory(root: &Path) -> PathBuf {
    root.join("Data/config/debug-control")
}
pub(crate) struct Server {
    root: PathBuf,
    dir: PathBuf,
    run: String,
    session: Value,
    pub pending: Option<Request>,
    outbox: Option<Outbox>,
    query_outbox: Option<Outbox>,
    ledger:ledger::Ledger<Value>,
}
struct Outbox { value:Value, context:crate::debug_events::Correlation }
impl Server {
    pub fn root(&self)->&Path { &self.root }
    pub fn event_session(&self)->crate::debug_events::Session{
        crate::debug_events::Session{run_id:self.run.clone(),manifest_sha256:self.session["manifest_sha256"].as_str().map(str::to_owned)}
    }
    pub fn new(root: &Path) -> Result<Self, String> {
        let manifest=crate::diagnostics::read(&root.join("debug-session.json"),1024*1024)?;
        let declared:Value=serde_json::from_slice(&manifest).map_err(|e|e.to_string())?;
        let session=json!({"manifest_sha256":gba_core::ffta_rom::digest(&manifest),
            "label":declared["label"].as_str().unwrap_or("").chars().take(128).collect::<String>()});
        let dir = directory(root);
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let server=Self{root:root.to_owned(),dir,run:stamp(),session,pending:None,outbox:None,query_outbox:None,ledger:Default::default()};
        put(&server.dir,"ready.json",&server.ready(true))?;
        Ok(server)
    }
    fn ready(&self,active:bool)->Value{
        json!({"schema":1,"kind":"ffta-debug-control","run":self.run,"active":active,
            "pid":std::process::id(),"session":self.session,
            "capabilities":["query-status","targeted-cancel","request-deadline","idempotent-request-v1","content-discovery-v1","replay-job-v1","cheat-catalog-v1","cheat-capabilities-v1","cheat-once-v1","cheat-maintain-v1","cheat-presets-v1"]})
    }
    pub fn pending_status(&self)->Value{
        self.pending.as_ref().map_or(Value::Null,|r|json!({"request_id":r.id,"operation_id":r.command.operation_id(),"command":r.command,"deadline_unix_ms":r.deadline_unix_ms}))
    }
    pub fn retry_status(&self)->Value{
        let (ids,max_ids,results,bytes,max_bytes)=self.ledger.stats();
        json!({"tracked_ids":ids,"max_ids":max_ids,"cached_results":results,"cached_serialized_bytes":bytes,"max_cached_serialized_bytes":max_bytes,"active_cancel_reserve":1})
    }
    /// A publication retry still belongs to the validated response owner, not
    /// to a different waiting command or an unparsed input file.
    pub fn publication_context(&self,query:bool)->Option<crate::debug_events::Correlation>{
        let outbox=if query{self.query_outbox.as_ref()}else{self.outbox.as_ref()}?;
        Some(outbox.context.clone())
    }
    pub fn pending_matches(&self,id:&str)->bool{self.pending.as_ref().is_some_and(|r|r.id==id)}
    pub fn deadline_expired(&self)->bool{self.pending.as_ref().is_some_and(Request::expired)}
    pub fn poll(&mut self)->Result<Option<Request>,String>{self.poll_channel(false)}
    pub fn poll_query(&mut self)->Result<Option<Request>,String>{self.poll_channel(true)}
    fn poll_channel(&mut self,query:bool)->Result<Option<Request>,String>{
        let channel=if query{"query"}else{"request"};
        self.flush_channel(query).map_err(|e|format!("{channel} response retry: {e}"))?;
        let path=self.dir.join(if query{"query.json"}else{"request.json"});
        let claim=self.dir.join(if query{"query.claim"}else{"request.claim"});
        // Claim the published name before reading: a client whose wait ended
        // may publish a replacement while this request is being processed.
        // Only this single session server owns the two fixed claim names.
        if !claim.try_exists().map_err(|e|format!("{channel} claim lookup: {e}"))? {
            match fs::rename(&path,&claim) {
                Ok(())=>{},
                Err(e) if e.kind()==std::io::ErrorKind::NotFound=>return Ok(None),
                Err(e)=>return Err(format!("{channel} claim: {e}")),
            }
        }
        // Keep the claim on transient open/read/remove failure. Terminal
        // oversized input is consumed so it cannot permanently block a channel.
        let mut file=fs::File::open(&claim).map_err(|e|format!("{channel} claim open: {e}"))?;
        let metadata=file.metadata().map_err(|e|format!("{channel} claim metadata: {e}"))?;
        if !metadata.is_file(){return Err(format!("{channel} claim is not a regular file"));}
        let mut bytes=Vec::new();
        if metadata.len()<=4096 {Read::by_ref(&mut file).take(4097).read_to_end(&mut bytes).map_err(|e|format!("{channel} claim read: {e}"))?;}
        drop(file);
        fs::remove_file(&claim).map_err(|e|format!("{channel} claim remove: {e}"))?;
        if metadata.len()>4096 || bytes.len()>4096{return Err(format!("{channel} claim exceeds 4096 bytes"));}
        let mut request:Request=serde_json::from_slice(&bytes).map_err(|e|e.to_string())?;
        request.query=query;
        if request.schema!=1 || request.run!=self.run || request.id.is_empty() || request.id.len()>100
            || request.session_manifest_sha256.as_ref().is_some_and(|h|Some(h.as_str())!=self.session["manifest_sha256"].as_str()){
            return Err("Stale, mismatched session or unsupported debug-control request".into());
        }
        if query && !request.command.query_channel(){
            return Err("Query mailbox only accepts readonly queries or targeted cancel".into());
        }
        if !query && matches!(request.command,Action::Cancel{..}){
            return Err("Targeted cancel requires the query mailbox".into());
        }
        if !request.command.read_query(){
            let fingerprint=request.fingerprint()?;
            let reserve=matches!(&request.command,Action::Cancel{request_id} if self.pending_matches(request_id));
            match self.ledger.admit(&request.id,&fingerprint,reserve){
                ledger::Admission::New=>{},
                // The original owner will publish the terminal result. Do not
                // send an intermediate response that a waiting client could
                // mistake for completion.
                ledger::Admission::Pending=>return Ok(None),
                ledger::Admission::Cached(value)=>{self.publish(&request,value)?;return Ok(None);},
                ledger::Admission::Unavailable=>{self.refuse_replay(&request,&fingerprint,"result-unavailable","Request already completed; its bounded result is no longer retained")?;return Ok(None);},
                ledger::Admission::Conflict=>{self.refuse_replay(&request,&fingerprint,"identity-conflict","Request ID was already used with different parameters")?;return Ok(None);},
                ledger::Admission::Full=>{self.refuse_replay(&request,&fingerprint,"capacity","Request identity capacity reached for this run")?;return Ok(None);},
            }
        }
        Ok(Some(request))
    }
    fn publish(&mut self,request:&Request,value:Value)->Result<(),String>{
        let outbox=Outbox{value,context:request.event_context()};
        if request.query{self.query_outbox=Some(outbox)}else{self.outbox=Some(outbox)}
        self.flush_channel(request.query)
    }
    fn refuse_replay(&mut self,request:&Request,fingerprint:&str,outcome:&str,error:&str)->Result<(),String>{
        let value=json!({"schema":1,"kind":"ffta-debug-control-result","run":self.run,"id":request.id,
            "session":self.session,"request_sha256":fingerprint,"deadline_unix_ms":request.deadline_unix_ms,"operation_id":request.command.operation_id(),"outcome":outcome,"ok":false,"error":error,"status":null});
        self.publish(request,value)
    }
    pub fn reply(&mut self,request:&Request,status:Value,error:Option<String>)->Result<(),String>{
        self.reply_classified(request,status,error,None)
    }
    fn reply_classified(&mut self,request:&Request,status:Value,mut error:Option<String>,classification:Option<&str>)->Result<(),String>{
        if request.expired() && error.is_none(){error=Some("Command completed after deadline; inspect returned state before retrying".into());}
        let outcome=if request.expired(){"timeout"}else if let Some(outcome)=classification{outcome}else if error.as_deref().is_some_and(|e|e.starts_with("Session closed")){"closed"}else if error.as_deref().is_some_and(|e|e.contains("control-cancelled")){"cancelled"}else if error.is_some(){"refused"}else{"ok"};
        let fingerprint=request.fingerprint()?;
        let value=json!({"schema":1,"kind":"ffta-debug-control-result","run":self.run,"id":request.id,
            "session":self.session,"request_sha256":fingerprint,"deadline_unix_ms":request.deadline_unix_ms,"operation_id":request.command.operation_id(),"outcome":outcome,"ok":error.is_none(),"error":error,"status":status});
        if !request.command.read_query(){
            let size=serde_json::to_vec_pretty(&value).map_err(|e|e.to_string())?.len();
            self.ledger.finish(&request.id,&fingerprint,value.clone(),size);
        }
        self.publish(request,value)
    }
    fn flush_channel(&mut self,query:bool)->Result<(),String>{
        let outbox=if query{&mut self.query_outbox}else{&mut self.outbox};
        if let Some(item)=outbox{
            put(&self.dir,if query{"query-result.json"}else{"result.json"},&item.value)?;
            *outbox=None;
        }Ok(())
    }
    pub fn inspection(&self,value:&Value)->Result<(),String>{put(&self.dir,"inspection.json",value)}
    pub fn finish(&mut self,status:Value,error:Option<String>)->Result<(),String>{self.finish_classified(status,error,None)}
    pub fn finish_classified(&mut self,mut status:Value,error:Option<String>,classification:Option<&str>)->Result<(),String>{
        if let Some(request)=self.pending.take(){
            // The caller's snapshot was captured just before this ownership
            // transition. A terminal reply must not advertise that same job
            // as still pending; retries retain their request/job context above.
            if let Some(object)=status.as_object_mut(){if object.contains_key("pending_control"){object.insert("pending_control".into(),Value::Null);}}
            self.reply_classified(&request,status,error,classification)?;
        }Ok(())
    }
}
impl Drop for Server {
    fn drop(&mut self){
        if let Some(request)=self.pending.take(){let _=self.reply(&request,Value::Null,Some("Session closed before command completion".into()));}
        for _ in 0..3{
            let primary=self.flush_channel(false);let query=self.flush_channel(true);
            if primary.is_ok()&&query.is_ok(){break;}
            std::thread::sleep(Duration::from_millis(2));
        }
        let _=put(&self.dir,"ready.json",&self.ready(false));
    }
}

pub(crate) fn parse(args: &[String]) -> Result<(PathBuf, Action), String> {
    if args.len() < 2 {
        return Err(
            "Use debug-control <status|cancel|pause|resume|step|inspect|break|watch|graphics|ui-start|ui-report|audio-start|audio-report|code-start|code-report|gameplay-start|gameplay-report|recent-start|recent-freeze|recent-report|issue-report> SESSION [options]"
                .into(),
        );
    }
    let hex = |s: &str| {
        u32::from_str_radix(s.trim_start_matches("0x"), 16)
            .map_err(|_| "Expected a hexadecimal address/value".to_owned())
    };
    let provenance = |s:&str| -> Result<u16,String> {if s.is_empty() || !s.bytes().all(|b|b.is_ascii_digit()) || (s.len()>1 && s.starts_with('0')){return Err("Invalid provenance index".into());}s.parse().map_err(|_|"Invalid provenance index".into())};
    let action = match args[0].as_str() {
        "cheat-select" if args.len()==3=>Action::SelectId{slot:SelectionSlot::CheatCatalog,id:args[2].clone()},
        "cheat-refresh" if args.len()==2=>Action::CheatRefresh,
        "cheat-prepare" if args.len()==6=>Action::CheatPrepare{id:args[2].clone(),provenance:provenance(&args[3])?,master:args[4].clone(),generation:args[5].clone()},
        "cheat-apply" if args.len()==7=>Action::CheatApply{id:args[2].clone(),provenance:provenance(&args[3])?,master:args[4].clone(),generation:args[5].clone(),preview:args[6].clone()},
        "cheat-arm" if args.len()==7=>Action::CheatArm{id:args[2].clone(),provenance:provenance(&args[3])?,master:args[4].clone(),generation:args[5].clone(),preview:args[6].clone()},
        "cheat-disarm" if args.len()==3=>Action::CheatDisarm{id:if args[2]=="all"{None}else{Some(args[2].clone())}},
        "cheat-preset-save" if args.len()==3=>Action::CheatPresetSave{name:args[2].clone()},
        "cheat-preset-prepare" if args.len()==3=>Action::CheatPresetPrepare{name:args[2].clone()},
        "cheat-preset-arm" if args.len()==4=>Action::CheatPresetArm{name:args[2].clone(),preview:args[3].clone()},
        "cheat-list" | "cheat-show" | "cheat-capabilities"=>{
            let show=args[0]!="cheat-list";let capabilities=args[0]=="cheat-capabilities";let(mut offset,mut limit,mut generation)=(0u32,16u8,None);let mut pos=2;
            let id=if show{let value=args.get(pos).ok_or("Cheat show needs a CH ID")?.clone();pos+=1;Some(value)}else{None};
            let mut seen=std::collections::BTreeSet::new();
            while pos<args.len(){let key=&args[pos];pos+=1;if !seen.insert(key){return Err("Duplicate cheat option".into());}let value=args.get(pos).ok_or("Cheat option needs a value")?;pos+=1;
                match key.as_str(){"--offset"=>offset=value.parse().map_err(|_|"Invalid cheat offset")?,"--limit" if !show=>limit=value.parse().map_err(|_|"Invalid cheat page limit")?,"--generation"=>generation=Some(value.clone()),_=>return Err("Use --offset N, --generation SHA and list --limit 1..16".into())}}
            if let Some(id)=id{if capabilities{Action::CheatCapabilities{id,offset,generation}}else{Action::CheatShow{id,offset,generation}}}else{Action::CheatList{offset,limit,generation}}
        },
        "replay-run" if args.len()==2 || (args.len()==4 && args[2]=="--mode")=>Action::Replay{mode:if args.len()==2{ReplayMode::Plain}else{match args[3].as_str(){"plain"=>ReplayMode::Plain,"audit"=>ReplayMode::Audit,"capture"=>ReplayMode::Capture,"compare"=>ReplayMode::Comparison,_=>return Err("Replay mode needs plain/audit/capture/compare".into())}}},
        "content-refresh" if args.len()==2=>Action::ContentRefresh,
        "content-list"=>{
            let(mut offset,mut limit,mut generation)=(0,16,None);let mut seen=std::collections::BTreeSet::new();let mut it=args[2..].iter();
            while let Some(key)=it.next(){if !seen.insert(key){return Err("Duplicate content list option".into());}let value=it.next().ok_or("Content list option needs a value")?;
                match key.as_str(){"--offset"=>offset=value.parse().map_err(|_|"Invalid content offset")?,"--limit"=>limit=value.parse().map_err(|_|"Invalid content limit")?,"--generation"=>generation=Some(value.parse().map_err(|_|"Invalid content generation")?),_=>return Err("Use --offset N --limit N --generation N".into())}}
            Action::ContentList{offset,limit,generation}
        },
        "cancel" if args.len()==4 && args[2]=="--request"=>Action::Cancel{request_id:args[3].clone()},
        "issue-report"=>{
            let mut profile=None;let mut symptom=String::new();let mut seen=std::collections::BTreeSet::new();let mut it=args[2..].iter();
            while let Some(key)=it.next(){if !seen.insert(key){return Err("Duplicate issue option".into());}let value=it.next().ok_or("Issue option needs a value")?;
                match key.as_str(){"--profile"=>profile=Some(PathBuf::from(value)),"--symptom"=>symptom=value.clone(),_=>return Err("Use --profile FILE / --symptom TEXT".into())}
            }Action::IssueReport{profile,symptom}
        },
        "recent-start"=>{
            let mut settings=crate::debug_recent::Settings::default();let mut seen=std::collections::BTreeSet::new();let mut it=args[2..].iter();
            let boolean=|v:&str|match v{"on"=>Ok(true),"off"=>Ok(false),_=>Err("Choose on/off")};
            while let Some(key)=it.next(){if !seen.insert(key){return Err("Duplicate recent option".into());}let value=it.next().ok_or("Recent option needs a value")?;
                match key.as_str(){"--events"=>settings.events=value.parse().map_err(|_|"Invalid event limit")?,
                    "--reads"=>settings.reads=boolean(value)?,"--on-error"=>settings.on_error=boolean(value)?,
                    "--start"=>settings.start=hex(value)?,"--end"=>settings.end=hex(value)?,_=>return Err("Unknown recent option".into())}
            }settings.validate()?;Action::RecentStart{settings}
        },
        "recent-freeze" if args.len()==2=>Action::RecentFreeze,
        "recent-report" if args.len()==2=>Action::RecentReport,
        "gameplay-start"=>{
            let mut settings=crate::debug_gameplay::Settings::default();let mut seen=std::collections::BTreeSet::new();let mut it=args[2..].iter();
            while let Some(key)=it.next(){if !seen.insert(key){return Err("Duplicate gameplay option".into());}let value=it.next().ok_or("Gameplay option needs a value")?;
                match key.as_str(){"--frames"=>settings.frames=value.parse().map_err(|_|"Invalid gameplay frame count")?,"--events"=>settings.events=value.parse().map_err(|_|"Invalid event limit")?,
                    "--family"=>settings.family=value.clone(),"--context"=>settings.context=Some(hex(value)?),_=>return Err("Unknown gameplay trace option".into())}
            }settings.validate()?;Action::GameplayStart{settings}
        },
        "gameplay-report" if args.len()==2=>Action::GameplayReport,
        "code-start"=>{
            let mut settings=crate::debug_code::Settings::default();let mut seen=std::collections::BTreeSet::new();let mut it=args[2..].iter();
            while let Some(key)=it.next(){if !seen.insert(key){return Err("Duplicate code trace option".into());}let value=it.next().ok_or("Code option needs a value")?;
                match key.as_str(){"--frames"=>settings.frames=value.parse().map_err(|_|"Invalid code frame count")?,"--events"=>settings.events=value.parse().map_err(|_|"Invalid event limit")?,
                    "--start"=>settings.start=hex(value)?,"--end"=>settings.end=hex(value)?,"--kind"=>settings.kind=value.clone(),
                    "--mode"=>settings.thumb=match value.as_str(){"arm"=>Some(false),"thumb"=>Some(true),"all"=>None,_=>return Err("Choose arm/thumb/all".into())},_=>return Err("Unknown code trace option".into())}
            }settings.validate()?;Action::CodeStart{settings}
        },
        "code-report" if args.len()==2=>Action::CodeReport,
        "audio-start"=>{
            let mut settings=crate::debug_audio::Settings::default();let mut i=2;let mut seen=std::collections::BTreeSet::new();
            while i<args.len(){let key=&args[i];if !seen.insert(key)||i+1>=args.len(){return Err("Duplicate/missing audio option".into());}
                match key.as_str(){"--frames"=>settings.frames=args[i+1].parse().map_err(|_|"Invalid audio frame count")?,"--streams"=>settings.streams=args[i+1].split(',').map(str::to_owned).collect(),_=>return Err("Use --frames N / --streams LIST".into())}i+=2;
            }settings.validate()?;Action::AudioStart{settings}
        },
        "audio-report" if args.len()==2=>Action::AudioReport,
        "ui-start" if args.len()==2=>Action::UiStart,
        "ui-report" if args.len()==2=>Action::UiReport,
        "graphics" if args.len()==2 || (args.len()==3 && args[2]=="--compare-last")=>Action::Graphics{compare_last:args.len()==3},
        "inspect" if args.len() == 4 => Action::Inspect {
            address: hex(&args[2])?,
            length: args[3]
                .parse()
                .map_err(|_| "Length must be decimal1..256")?,
        },
        "break" if args.len() == 4 || args.len() == 6 => {
            let thumb = match args[3].as_str() {
                "arm" => false,
                "thumb" => true,
                _ => return Err("Choose arm or thumb".into()),
            };
            let steps = if args.len() == 6 && args[4] == "--steps" {
                args[5].parse().map_err(|_| "Invalid step limit")?
            } else if args.len() == 4 {
                100000
            } else {
                return Err("Use --steps N".into());
            };
            Action::Find {
                settings: crate::debug_probe::Settings {
                    condition: crate::debug_probe::Condition::Pc {
                        address: hex(&args[2])?,
                        thumb,
                    },
                    steps,
                },
            }
        }
        "watch" if args.len() >= 5 => {
            let write = match args[2].as_str() {
                "read" => false,
                "write" => true,
                _ => return Err("Choose read or write".into()),
            };
            let mut steps = 100000;
            let mut width = None;
            let mut value = None;
            let mut seen = std::collections::BTreeSet::new();
            let mut it = args[5..].iter();
            while let Some(flag) = it.next() {
                if !seen.insert(flag) {
                    return Err("Duplicate watch option".into());
                }
                let v = it.next().ok_or("Watch option needs value")?;
                match flag.as_str() {
                    "--steps" => steps = v.parse().map_err(|_| "Invalid step limit")?,
                    "--width" => width = Some(v.parse().map_err(|_| "Invalid width")?),
                    "--value" => value = Some(hex(v)?),
                    _ => return Err("Use --steps N --width1/2/4 --value HEX".into()),
                }
            }
            Action::Find {
                settings: crate::debug_probe::Settings {
                    condition: crate::debug_probe::Condition::Access {
                        watch: gba_core::debug_memory::Watch {
                            start: hex(&args[3])?,
                            end: hex(&args[4])?,
                            write,
                            width,
                            value,
                        },
                    },
                    steps,
                },
            }
        }
        "status" if args.len() == 2 => Action::Status,
        "pause" if args.len() == 2 => Action::Pause,
        "resume" if args.len() == 2 => Action::Resume,
        "step" if matches!(args.len(), 3 | 5) => Action::Step {
            frames: args[2].parse().map_err(|_| "N must be 1..600")?,
            keys: if args.len() == 5 && args[3] == "--keys" {
                u16::from_str_radix(args[4].trim_start_matches("0x"), 16)
                    .map_err(|_| "Invalid hexadecimal key mask")?
            } else if args.len() == 3 {
                0x3ff
            } else {
                return Err("Use --keys HEX".into());
            },
        },
        _ => return Err(
            "Use debug-control <status|cancel|pause|resume|step|inspect|break|watch|graphics|ui-start|ui-report|audio-start|audio-report|code-start|code-report|gameplay-start|gameplay-report|recent-start|recent-freeze|recent-report|issue-report> SESSION [options]"
                .into(),
        ),
    };
    action.validate()?;
    Ok((PathBuf::from(&args[1]), action))
}
fn options(args:&[String])->Result<(Vec<String>,u64),String>{
    let mut filtered=Vec::new();let mut timeout=None;let mut it=args.iter();
    while let Some(arg)=it.next(){
        if arg=="--timeout-ms"{
            if timeout.is_some(){return Err("Duplicate --timeout-ms".into());}
            timeout=Some(it.next().ok_or("--timeout-ms needs a value")?.parse::<u64>().map_err(|_|"Invalid timeout")?);
        }else{filtered.push(arg.clone());}
    }
    let timeout=timeout.unwrap_or(120_000);
    if !(20..=120_000).contains(&timeout){return Err("Timeout needs20..120000milliseconds".into());}
    Ok((filtered,timeout))
}
fn client_lock(dir:&Path,query:bool)->Result<std::fs::File,String>{
    let lock=OpenOptions::new().create(true).truncate(false).read(true).write(true)
        .open(dir.join(if query{"query.lock"}else{"client.lock"})).map_err(|e|e.to_string())?;
    lock.try_lock().map_err(|_|if query{"Another status/cancel query is waiting"}else{"Another debug-control client is waiting for a result"})?;
    Ok(lock)
}
#[derive(Clone)]
struct RetryIdentity {run:String,id:String,deadline_unix_ms:u64}
#[derive(Debug)]
pub(crate) struct ClientFailure {pub outcome:&'static str,pub message:String,pub delivery:&'static str}
impl ClientFailure {
    fn new(outcome:&'static str,message:impl Into<String>,delivery:&'static str)->Self{Self{outcome,message:message.into(),delivery}}
    pub(crate) fn value(&self,kind:&str)->Value{json!({"schema":1,"kind":kind,"ok":false,"outcome":self.outcome,"error":self.message,"delivery":self.delivery})}
}

fn exchange(dir:&Path,ready:&Value,action:Action,query:bool,timeout:u64)->Result<Value,String>{
    exchange_with_retry(dir,ready,action,query,timeout,None)
}
fn response_matches(value:&Value,ready:&Value,request:&Request,fingerprint:&str,protected:bool)->bool{
    value["schema"]==1 && value["run"]==request.run && value["id"]==request.id && value["session"]==ready["session"]
        && (!protected||value["request_sha256"]==fingerprint)
}
fn exchange_with_retry(dir:&Path,ready:&Value,action:Action,query:bool,timeout:u64,retry:Option<&RetryIdentity>)->Result<Value,String>{
    let run=ready["run"].as_str().ok_or("Missing session run identity")?.to_owned();
    if retry.is_some_and(|r|r.run!=run){return Err("Retry run differs from the active session; no command sent".into());}
    let protected=ready["capabilities"].as_array().is_some_and(|c|c.contains(&json!("idempotent-request-v1")));
    if retry.is_some()&&!protected{return Err("This session does not support safe request retry".into());}
    let request=Request{schema:1,run:run.clone(),id:retry.map_or_else(stamp,|r|r.id.clone()),command:action,
        deadline_unix_ms:Some(retry.map_or_else(||unix_ms().saturating_add(timeout),|r|r.deadline_unix_ms)),
        session_manifest_sha256:Some(ready["session"]["manifest_sha256"].as_str().ok_or("Missing session identity")?.into()),query};
    let fingerprint=request.fingerprint()?;
    put(dir,if query{"query.json"}else{"request.json"},&serde_json::to_value(&request).map_err(|e|e.to_string())?)?;
    let start=Instant::now();
    loop{
        if let Ok(value)=read(&dir.join(if query{"query-result.json"}else{"result.json"})){
            if response_matches(&value,ready,&request,&fingerprint,protected){
                return Ok(value);
            }
        }
        if start.elapsed()>=Duration::from_millis(timeout){
            return Ok(json!({"schema":1,"kind":"ffta-debug-control-client","run":run,"id":request.id,"session":ready["session"],
                "ok":false,"outcome":"timeout","error":"No acknowledgement before deadline; use status to inspect the exact request. Expiry stops pending work at the next frontend poll, not inside a guest frame or blocked callback.",
                "deadline_unix_ms":request.deadline_unix_ms,"request_sha256":fingerprint,"server_acknowledged":false}));
        }
        if let Ok(value)=read(&dir.join("ready.json")){
            if value["run"]!=run || value["active"]!=true{
                return Ok(json!({"schema":1,"kind":"ffta-debug-control-client","run":run,"id":request.id,"session":ready["session"],
                    "ok":false,"outcome":"closed","error":"Session closed or reopened before acknowledgement"}));
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn parse_retry(args:&[String])->Result<(PathBuf,Action,u64,RetryIdentity),String>{
    const HELP:&str="Use retry SESSION --run RUN --request ID --deadline-unix-ms N [--timeout-ms N] -- COMMAND [ARGUMENTS]";
    if args.len()<10{return Err(HELP.into());}
    let separator=args[2..].iter().position(|s|s=="--").map(|i|i+2).ok_or(HELP)?;
    if separator<2||separator+1>=args.len(){return Err(HELP.into());}
    let mut run=None;let mut id=None;let mut deadline=None;let mut timeout=None;let mut it=args[2..separator].iter();
    while let Some(key)=it.next(){
        let value=it.next().ok_or(HELP)?;
        match key.as_str(){
            "--run" if run.is_none()=>run=Some(value.clone()),
            "--request" if id.is_none()=>id=Some(value.clone()),
            "--deadline-unix-ms" if deadline.is_none()=>deadline=Some(value.parse::<u64>().map_err(|_|"Invalid original deadline")?),
            "--timeout-ms" if timeout.is_none()=>timeout=Some(value.parse::<u64>().map_err(|_|"Invalid timeout")?),
            _=>return Err(HELP.into()),
        }
    }
    let run=run.ok_or(HELP)?;let id=id.ok_or(HELP)?;let deadline_unix_ms=deadline.ok_or(HELP)?;let timeout=timeout.unwrap_or(120_000);
    if [run.as_str(),id.as_str()].iter().any(|v|v.is_empty()||v.len()>100||v.chars().any(char::is_control)) {return Err("Retry run and request IDs need1..100 bytes without controls".into());}
    if !(20..=120_000).contains(&timeout){return Err("Timeout needs20..120000milliseconds".into());}
    let mut command=vec![args[separator+1].clone(),args[1].clone()];command.extend_from_slice(&args[separator+2..]);
    let (root,action)=parse(&command)?;if action.read_query(){return Err("Read-only query: request a fresh result instead".into());}
    Ok((root,action,timeout,RetryIdentity{run,id,deadline_unix_ms}))
}
fn execute(args:&[String])->Result<Value,ClientFailure>{
    if args.first().is_some_and(|v|v=="retry"){
        let (root,action,timeout,retry)=parse_retry(args).map_err(|e|ClientFailure::new("parse-error",e,"not-sent"))?;
        return execute_action_inner(&root,action,timeout,Some(&retry));
    }
    let (args,timeout)=options(args).map_err(|e|ClientFailure::new("parse-error",e,"not-sent"))?;
    let (root,action)=parse(&args).map_err(|e|ClientFailure::new("parse-error",e,"not-sent"))?;
    execute_action_classified(&root,action,timeout)
}
/// Shared mailbox transport for typed commands; never bypass identity or locks.
#[cfg(test)]
fn execute_action(root:&Path,action:Action,timeout:u64)->Result<Value,String>{
    execute_action_classified(root,action,timeout).map_err(|e|e.message)
}
pub(crate) fn execute_action_classified(root:&Path,action:Action,timeout:u64)->Result<Value,ClientFailure>{
    execute_action_inner(root,action,timeout,None)
}
/// Retry a previously identified typed request; a different run or payload
/// is rejected by the transport/ledger rather than executed as a new request.
pub(crate) fn retry_action_classified(root:&Path,action:Action,timeout:u64,run:String,id:String,deadline_unix_ms:u64)->Result<Value,ClientFailure>{
    if [run.as_str(),id.as_str()].iter().any(|v|v.is_empty()||v.len()>100||v.chars().any(char::is_control)) {
        return Err(ClientFailure::new("invalid-request","Retry run and request IDs need1..100 bytes without controls","not-sent"));
    }
    if action.read_query(){return Err(ClientFailure::new("invalid-request","Read-only query: request a fresh result instead","not-sent"));}
    execute_action_inner(root,action,timeout,Some(&RetryIdentity{run,id,deadline_unix_ms}))
}
fn execute_action_inner(root:&Path,action:Action,timeout:u64,retry:Option<&RetryIdentity>)->Result<Value,ClientFailure>{
    action.validate().map_err(|e|ClientFailure::new("invalid-request",e,"not-sent"))?;
    if !(20..=120_000).contains(&timeout){return Err(ClientFailure::new("invalid-request","Timeout needs20..120000milliseconds","not-sent"));}
    let manifest=crate::diagnostics::read(&root.join("debug-session.json"),1024*1024)
        .map_err(|_|ClientFailure::new("unavailable","Choose an isolated debug-session folder","not-sent"))?;
    let dir=directory(root);
    let query=action.query_channel();
    let _lock=client_lock(&dir,query).map_err(|e|ClientFailure::new("unavailable",e,"not-sent"))?;
    let ready=read(&dir.join("ready.json")).map_err(|e|ClientFailure::new("unavailable",e,"not-sent"))?;
    if ready["schema"]!=1 || ready["kind"]!="ffta-debug-control"{
        return Err(ClientFailure::new("incompatible","Unsupported debug-control protocol","not-sent"));
    }
    if ready["active"]!=true{return Err(ClientFailure::new("unavailable","No active debug-control session","not-sent"));}
    if ready["session"]["manifest_sha256"]!=gba_core::ffta_rom::digest(&manifest){
        return Err(ClientFailure::new("incompatible","Session identity mismatch or old channel; create a copy with the current runtime","not-sent"));
    }
    if ready["capabilities"].as_array().is_none_or(|c|!c.contains(&json!("targeted-cancel"))){
        return Err(ClientFailure::new("incompatible","This channel does not support query/cancel/deadline; use the current runtime","not-sent"));
    }
    if matches!(action,Action::ContentRefresh|Action::ContentList{..}) && ready["capabilities"].as_array().is_none_or(|c|!c.contains(&json!("content-discovery-v1"))){
        return Err(ClientFailure::new("incompatible","This session does not support content discovery; use the current runtime","not-sent"));
    }
    if action.cheat_capability() && ready["capabilities"].as_array().is_none_or(|c|!c.contains(&json!("cheat-catalog-v1"))){
        return Err(ClientFailure::new("incompatible","This session does not support cheat catalog queries; use the current runtime","not-sent"));
    }
    if matches!(action,Action::CheatCapabilities{..}) && ready["capabilities"].as_array().is_none_or(|c|!c.contains(&json!("cheat-capabilities-v1"))){return Err(ClientFailure::new("incompatible","This session does not support source capability queries","not-sent"));}
    if matches!(action,Action::CheatPrepare{..}|Action::CheatApply{..}) && ready["capabilities"].as_array().is_none_or(|c|!c.contains(&json!("cheat-once-v1"))){return Err(ClientFailure::new("incompatible","This session does not support guarded cheat one-shot","not-sent"));}
    if matches!(action,Action::CheatArm{..}|Action::CheatDisarm{..}) && ready["capabilities"].as_array().is_none_or(|c|!c.contains(&json!("cheat-maintain-v1"))){return Err(ClientFailure::new("incompatible","This session does not support maintained cheats","not-sent"));}
    if matches!(action,Action::CheatPresetSave{..}|Action::CheatPresetPrepare{..}|Action::CheatPresetArm{..}) && ready["capabilities"].as_array().is_none_or(|c|!c.contains(&json!("cheat-presets-v1"))){return Err(ClientFailure::new("incompatible","This session does not support cheat presets","not-sent"));}
    if matches!(action,Action::Replay{..}) && ready["capabilities"].as_array().is_none_or(|c|!c.contains(&json!("replay-job-v1"))){
        return Err(ClientFailure::new("incompatible","This session does not support replay jobs; use the current runtime","not-sent"));
    }
    if let Some(retry)=retry {
        if ready["run"].as_str()!=Some(retry.run.as_str()){return Err(ClientFailure::new("incompatible","Retry run differs from the active session; no command sent","not-sent"));}
        if ready["capabilities"].as_array().is_none_or(|c|!c.contains(&json!("idempotent-request-v1"))){return Err(ClientFailure::new("incompatible","This session does not support safe request retry","not-sent"));}
    }
    let result=if let Some(retry)=retry{exchange_with_retry(&dir,&ready,action,query,timeout,Some(retry))}else{exchange(&dir,&ready,action,query,timeout)};
    result.map_err(|e|ClientFailure::new("transport-error",e,"unknown"))
}
pub(crate) fn cmd(args:&[String])->Result<(),String>{
    match execute(args){
        Ok(value)=>{
            println!("{value}");
            if value["ok"]==true{Ok(())}else{Err(value["error"].as_str().unwrap_or("Control failed").into())}
        },
        Err(error)=>{
            println!("{}",error.value("ffta-debug-control-client"));
            Err(error.message)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn synthetic_server(root:&Path)->Server{
        fs::write(root.join("debug-session.json"),br#"{"label":"SYNTHETIC protocol only"}"#).unwrap();
        Server::new(root).unwrap()
    }
    fn synthetic_request(s:&Server,id:&str,command:Action,query:bool)->Request{
        Request{schema:1,run:s.run.clone(),id:id.into(),command,deadline_unix_ms:Some(unix_ms()+1000),
            session_manifest_sha256:Some(s.session["manifest_sha256"].as_str().unwrap().into()),query}
    }
    #[test]
    fn claimed_request_never_removes_a_later_publication(){
        let t=tempfile::tempdir().unwrap();let mut s=synthetic_server(t.path());
        for query in [false,true] {
            let (public,claim)=if query{("query.json","query.claim")}else{("request.json","request.claim")};
            let first=synthetic_request(&s,if query{"q-first"}else{"p-first"},if query{Action::Status}else{Action::Pause},query);
            let second=synthetic_request(&s,if query{"q-second"}else{"p-second"},if query{Action::Status}else{Action::Resume},query);
            put(&s.dir,public,&serde_json::to_value(&first).unwrap()).unwrap();fs::rename(s.dir.join(public),s.dir.join(claim)).unwrap();
            // The earlier client has stopped waiting and the next client has
            // published while the server already owns the first request.
            put(&s.dir,public,&serde_json::to_value(&second).unwrap()).unwrap();
            let a=if query{s.poll_query()}else{s.poll()}.unwrap().unwrap();assert_eq!(a.id,first.id);assert!(s.dir.join(public).exists());
            let b=if query{s.poll_query()}else{s.poll()}.unwrap().unwrap();assert_eq!(b.id,second.id);
            assert!(if query{s.poll_query()}else{s.poll()}.unwrap().is_none());
        }
    }
    #[test]
    fn stale_or_oversized_claim_does_not_execute_or_block_a_fresh_request(){
        let t=tempfile::tempdir().unwrap();let s=synthetic_server(t.path());
        let old=synthetic_request(&s,"old",Action::Pause,false);put(&s.dir,"request.claim",&serde_json::to_value(old).unwrap()).unwrap();drop(s);
        let mut s=Server::new(t.path()).unwrap();let fresh=synthetic_request(&s,"fresh",Action::Resume,false);put(&s.dir,"request.json",&serde_json::to_value(&fresh).unwrap()).unwrap();
        assert!(s.poll().err().unwrap().contains("Stale"));assert!(s.dir.join("request.json").exists());assert_eq!(s.poll().unwrap().unwrap().id,"fresh");
        fs::write(s.dir.join("query.claim"),vec![b'x';4097]).unwrap();let q=synthetic_request(&s,"fresh-query",Action::Status,true);put(&s.dir,"query.json",&serde_json::to_value(q).unwrap()).unwrap();
        assert!(s.poll_query().err().unwrap().contains("exceeds"));assert_eq!(s.poll_query().unwrap().unwrap().id,"fresh-query");
    }
    #[cfg(windows)]
    #[test]
    fn locked_claim_is_retried_without_removing_the_next_request(){
        use std::os::windows::fs::OpenOptionsExt;
        let t=tempfile::tempdir().unwrap();let mut s=synthetic_server(t.path());
        let first=synthetic_request(&s,"locked",Action::Pause,false);put(&s.dir,"request.claim",&serde_json::to_value(first).unwrap()).unwrap();
        let held=OpenOptions::new().read(true).share_mode(0).open(s.dir.join("request.claim")).unwrap();
        let next=synthetic_request(&s,"next",Action::Resume,false);put(&s.dir,"request.json",&serde_json::to_value(next).unwrap()).unwrap();
        assert!(s.poll().err().unwrap().contains("claim open"));assert!(s.dir.join("request.claim").exists());assert!(s.dir.join("request.json").exists());
        drop(held);assert_eq!(s.poll().unwrap().unwrap().id,"locked");assert_eq!(s.poll().unwrap().unwrap().id,"next");
    }
    #[test]
    fn retry_identity_deduplicates_pending_and_completed_requests(){
        let t=tempfile::tempdir().unwrap();let mut s=synthetic_server(t.path());
        let original=synthetic_request(&s,"same-id",Action::Step{frames:1,keys:0x3ff},false);
        put(&s.dir,"request.json",&serde_json::to_value(&original).unwrap()).unwrap();let accepted=s.poll().unwrap().unwrap();s.pending=Some(accepted.clone());
        put(&s.dir,"request.json",&serde_json::to_value(&original).unwrap()).unwrap();assert!(s.poll().unwrap().is_none());assert!(!s.dir.join("result.json").exists());assert!(s.pending_matches("same-id"));
        let mut changed=original.clone();changed.command=Action::Step{frames:2,keys:0x3ff};put(&s.dir,"request.json",&serde_json::to_value(&changed).unwrap()).unwrap();assert!(s.poll().unwrap().is_none());
        let conflict=read(&s.dir.join("result.json")).unwrap();assert_eq!(conflict["outcome"],"identity-conflict");assert!(s.pending_matches("same-id"));
        assert!(!response_matches(&conflict,&s.ready(true),&original,&original.fingerprint().unwrap(),true));assert!(response_matches(&conflict,&s.ready(true),&changed,&changed.fingerprint().unwrap(),true));
        s.finish(json!({"frame":1,"pending_control":s.pending_status()}),None).unwrap();let finished=read(&s.dir.join("result.json")).unwrap();
        assert_eq!(finished["outcome"],"ok");assert!(finished["status"]["pending_control"].is_null());
        put(&s.dir,"request.json",&serde_json::to_value(&original).unwrap()).unwrap();assert!(s.poll().unwrap().is_none());assert_eq!(read(&s.dir.join("result.json")).unwrap(),finished);
    }
    #[test]
    fn evicted_results_and_full_ledger_never_redispatch_and_reserve_cancel(){
        let t=tempfile::tempdir().unwrap();let mut s=synthetic_server(t.path());s.ledger=ledger::Ledger::limits(2,1,4096,4096);
        let one=synthetic_request(&s,"one",Action::Pause,false);let two=synthetic_request(&s,"two",Action::Resume,false);
        for r in [&one,&two]{put(&s.dir,"request.json",&serde_json::to_value(r).unwrap()).unwrap();let got=s.poll().unwrap().unwrap();s.reply(&got,json!({"counter":1}),None).unwrap();}
        put(&s.dir,"request.json",&serde_json::to_value(&one).unwrap()).unwrap();assert!(s.poll().unwrap().is_none());assert_eq!(read(&s.dir.join("result.json")).unwrap()["outcome"],"result-unavailable");
        let three=synthetic_request(&s,"three",Action::Pause,false);put(&s.dir,"request.json",&serde_json::to_value(&three).unwrap()).unwrap();assert!(s.poll().unwrap().is_none());assert_eq!(read(&s.dir.join("result.json")).unwrap()["outcome"],"capacity");
        s.pending=Some(synthetic_request(&s,"active",Action::Step{frames:1,keys:0x3ff},false));
        let wrong=synthetic_request(&s,"wrong",Action::Cancel{request_id:"other".into()},true);put(&s.dir,"query.json",&serde_json::to_value(&wrong).unwrap()).unwrap();assert!(s.poll_query().unwrap().is_none());assert_eq!(read(&s.dir.join("query-result.json")).unwrap()["outcome"],"capacity");
        let right=synthetic_request(&s,"right",Action::Cancel{request_id:"active".into()},true);put(&s.dir,"query.json",&serde_json::to_value(&right).unwrap()).unwrap();assert!(matches!(s.poll_query().unwrap().unwrap().command,Action::Cancel{..}));
    }
    #[test]
    fn explicit_retry_requires_original_identity_and_matching_run(){
        let a=|v:&[&str]|v.iter().map(|s|s.to_string()).collect::<Vec<_>>();
        let args=a(&["retry","case","--run","run-1","--request","id-1","--deadline-unix-ms","123","--timeout-ms","50","--","step","1","--keys","0x3ff"]);
        let (_,action,timeout,retry)=parse_retry(&args).unwrap();assert!(matches!(action,Action::Step{frames:1,keys:0x3ff}));assert_eq!(timeout,50);assert_eq!(retry.id,"id-1");assert_eq!(retry.deadline_unix_ms,123);
        assert!(parse_retry(&a(&["retry","case","--run","r","--request","i","--deadline-unix-ms","1","--","status"])).is_err());
        let t=tempfile::tempdir().unwrap();let s=synthetic_server(t.path());let wrong=RetryIdentity{run:"other".into(),id:"id".into(),deadline_unix_ms:123};
        assert!(exchange_with_retry(&s.dir,&s.ready(true),Action::Pause,false,20,Some(&wrong)).is_err());assert!(!s.dir.join("request.json").exists());
    }
    #[test]
    fn content_discovery_refuses_old_server_without_publishing(){
        let t=tempfile::tempdir().unwrap();let s=synthetic_server(t.path());let mut ready=s.ready(true);
        ready["capabilities"]=json!(["targeted-cancel"]);put(&s.dir,"ready.json",&ready).unwrap();
        for action in [Action::ContentRefresh,Action::ContentList{offset:0,limit:1,generation:None}] {
            assert!(execute_action(t.path(),action,20).unwrap_err().contains("does not support content discovery"));
        }
        assert!(!s.dir.join("request.json").exists());assert!(!s.dir.join("query.json").exists());
    }
    #[test]
    fn replay_capability_and_terminal_results_preserve_request_identity(){
        let t=tempfile::tempdir().unwrap();let mut s=synthetic_server(t.path());let mut ready=s.ready(true);
        ready["capabilities"]=json!(["targeted-cancel"]);put(&s.dir,"ready.json",&ready).unwrap();
        let action=Action::Replay{mode:ReplayMode::Plain};
        let error=execute_action_classified(t.path(),action.clone(),20).unwrap_err();
        assert_eq!((error.outcome,error.delivery),("incompatible","not-sent"));assert!(!s.dir.join("request.json").exists());
        let original=synthetic_request(&s,"replay-id",action,false);
        put(&s.dir,"request.json",&serde_json::to_value(&original).unwrap()).unwrap();s.pending=Some(s.poll().unwrap().unwrap());
        assert!(s.pending_matches("replay-id")&&!s.pending_matches("wrong"));
        s.finish_classified(json!({"pending_control":s.pending_status(),"replay":{"result":"interrupted"}}),Some("worker interrupted".into()),Some("cancelled")).unwrap();
        let result=read(&s.dir.join("result.json")).unwrap();assert_eq!(result["id"],"replay-id");assert_eq!(result["outcome"],"cancelled");assert_eq!(result["ok"],false);assert!(result["status"]["pending_control"].is_null());
        put(&s.dir,"request.json",&serde_json::to_value(&original).unwrap()).unwrap();assert!(s.poll().unwrap().is_none());assert_eq!(read(&s.dir.join("result.json")).unwrap(),result);
    }
    #[test]
    fn cheat_capability_is_required_before_selection_or_queries(){
        let t=tempfile::tempdir().unwrap();let s=synthetic_server(t.path());let mut ready=s.ready(true);ready["capabilities"]=json!(["targeted-cancel"]);put(&s.dir,"ready.json",&ready).unwrap();
        for action in [Action::SelectId{slot:SelectionSlot::CheatCatalog,id:"content-a".into()},Action::CheatRefresh,Action::CheatList{offset:0,limit:1,generation:None},Action::CheatShow{id:"CH-0001".into(),offset:0,generation:None}]{
            let e=execute_action_classified(t.path(),action,20).unwrap_err();assert_eq!((e.outcome,e.delivery),("incompatible","not-sent"));
        }
        assert!(!s.dir.join("request.json").exists()&&!s.dir.join("query.json").exists());
    }
    #[test]
    fn one_shot_requires_its_own_capability_before_send(){
        let t=tempfile::tempdir().unwrap();let s=synthetic_server(t.path());let mut ready=s.ready(true);ready["capabilities"]=json!(["targeted-cancel","cheat-catalog-v1"]);put(&s.dir,"ready.json",&ready).unwrap();
        let generation="a".repeat(64);let token="b".repeat(64);
        for action in [Action::CheatPrepare{id:"CH-0001".into(),provenance:0,master:"CH-0002".into(),generation:generation.clone()},Action::CheatApply{id:"CH-0001".into(),provenance:0,master:"CH-0002".into(),generation:generation.clone(),preview:token}]{
            let e=execute_action_classified(t.path(),action,20).unwrap_err();assert_eq!((e.outcome,e.delivery),("incompatible","not-sent"));
        }
        assert!(!s.dir.join("request.json").exists());
    }

    #[test]
    fn preset_actions_require_their_own_capability_before_send() {
        let t=tempfile::tempdir().unwrap();let s=synthetic_server(t.path());
        let mut ready=s.ready(true);ready["capabilities"]=json!(["targeted-cancel","cheat-catalog-v1","cheat-maintain-v1"]);put(&s.dir,"ready.json",&ready).unwrap();
        for action in [Action::CheatPresetSave{name:"sample".into()},Action::CheatPresetPrepare{name:"sample".into()},Action::CheatPresetArm{name:"sample".into(),preview:"a".repeat(64)}] {
            let e=execute_action_classified(t.path(),action,20).unwrap_err();assert_eq!((e.outcome,e.delivery),("incompatible","not-sent"));
        }
        assert!(!s.dir.join("request.json").exists());
    }
    #[test]
    fn source_capability_query_requires_its_own_version_before_send(){
        let t=tempfile::tempdir().unwrap();let s=synthetic_server(t.path());let mut ready=s.ready(true);ready["capabilities"]=json!(["targeted-cancel","cheat-catalog-v1"]);put(&s.dir,"ready.json",&ready).unwrap();
        let action=Action::CheatCapabilities{id:"CH-0001".into(),offset:0,generation:None};let error=execute_action_classified(t.path(),action,20).unwrap_err();assert_eq!((error.outcome,error.delivery),("incompatible","not-sent"));assert!(!s.dir.join("query.json").exists());
    }
    #[test]
    fn cheat_queries_share_query_mailbox_without_stealing_pending_owner(){
        let t=tempfile::tempdir().unwrap();let mut s=synthetic_server(t.path());
        s.pending=Some(synthetic_request(&s,"active",Action::CheatRefresh,false));let before=s.retry_status();
        let query=synthetic_request(&s,"page",Action::CheatList{offset:0,limit:1,generation:None},true);
        put(&s.dir,"query.json",&serde_json::to_value(&query).unwrap()).unwrap();let got=s.poll_query().unwrap().unwrap();assert!(matches!(got.command,Action::CheatList{..}));
        s.reply(&got,json!({"pending_control":s.pending_status()}),None).unwrap();assert!(s.pending_matches("active"));assert_eq!(s.retry_status(),before);
        assert_eq!(read(&s.dir.join("query-result.json")).unwrap()["operation_id"],"cheats.list");
        let query=synthetic_request(&s,"capability",Action::CheatCapabilities{id:"CH-0001".into(),offset:0,generation:None},true);
        put(&s.dir,"query.json",&serde_json::to_value(&query).unwrap()).unwrap();let got=s.poll_query().unwrap().unwrap();assert!(matches!(got.command,Action::CheatCapabilities{..}));
        s.reply(&got,json!({}),None).unwrap();assert!(s.pending_matches("active"));assert_eq!(s.retry_status(),before);assert_eq!(read(&s.dir.join("query-result.json")).unwrap()["operation_id"],"cheats.capabilities");
    }
    #[test]
    fn local_preflight_failure_categories_do_not_publish_requests(){
        let t=tempfile::tempdir().unwrap();let s=synthetic_server(t.path());
        let mut ready=s.ready(true);ready["capabilities"]=json!(["targeted-cancel"]);put(&s.dir,"ready.json",&ready).unwrap();
        let error=execute_action_classified(t.path(),Action::ContentRefresh,20).unwrap_err();
        assert_eq!((error.outcome,error.delivery),("incompatible","not-sent"));assert_eq!(error.value("example")["outcome"],"incompatible");
        ready["active"]=json!(false);put(&s.dir,"ready.json",&ready).unwrap();let error=execute_action_classified(t.path(),Action::Pause,20).unwrap_err();
        assert_eq!((error.outcome,error.delivery),("unavailable","not-sent"));assert!(!s.dir.join("request.json").exists());assert!(!s.dir.join("query.json").exists());
        let args=vec!["step".into(),"case".into(),"not-a-number".into()];assert_eq!(execute(&args).unwrap_err().outcome,"parse-error");
    }
    #[test]
    fn content_selection_action_validates_and_reports_cancelled_owner() {
        let t=tempfile::tempdir().unwrap();let mut s=synthetic_server(t.path());
        let action=Action::SelectId{slot:SelectionSlot::Catalog,id:"opaque id".into()};
        action.validate().unwrap();
        let wire=serde_json::to_value(&action).unwrap();
        let decoded:Action=serde_json::from_value(wire).unwrap();
        assert_eq!(decoded.operation_id(),"content.select");
        assert!(Action::SelectId{slot:SelectionSlot::Replay,id:"\n".into()}.validate().is_err());
        assert!(Action::SelectId{slot:SelectionSlot::Replay,id:"x".repeat(513)}.validate().is_err());
        s.pending=Some(synthetic_request(&s,"selection1",decoded,false));
        assert!(s.pending_matches("selection1")&&!s.pending_matches("other"));
        s.finish(json!({}),Some("Content selection control-cancelled".into())).unwrap();
        let result=read(&s.dir.join("result.json")).unwrap();
        assert_eq!(result["id"],"selection1");assert_eq!(result["outcome"],"cancelled");
    }
    #[test]
    fn pending_command_queries_and_targeted_cancel_have_separate_locks_results_and_ids(){
        let t=tempfile::tempdir().unwrap();let mut s=synthetic_server(t.path());
        let command=synthetic_request(&s,"step1",Action::Step{frames:600,keys:0x3ff},false);
        put(&s.dir,"request.json",&serde_json::to_value(&command).unwrap()).unwrap();
        let mut c=Control::default();c.apply(&command.command,true,false).unwrap();
        s.pending=Some(s.poll().unwrap().unwrap());
        let primary=client_lock(&s.dir,false).unwrap();
        assert!(client_lock(&s.dir,false).is_err());
        let query_lock=client_lock(&s.dir,true).unwrap();assert!(client_lock(&s.dir,true).is_err());
        let query=synthetic_request(&s,"status1",Action::Status,true);
        put(&s.dir,"query.json",&serde_json::to_value(&query).unwrap()).unwrap();
        let got=s.poll_query().unwrap().unwrap();
        s.reply(&got,json!({"pending_control":s.pending_status()}),None).unwrap();
        assert_eq!(read(&s.dir.join("query-result.json")).unwrap()["status"]["pending_control"]["request_id"],"step1");
        assert_eq!(read(&s.dir.join("query-result.json")).unwrap()["operation_id"],"control.status");
        assert_eq!(s.pending_status()["operation_id"],"control.step");
        assert!(s.pending_matches("step1")&&!s.pending_matches("old")&&!s.dir.join("result.json").exists());
        let cancel=synthetic_request(&s,"cancel1",Action::Cancel{request_id:"step1".into()},true);
        put(&s.dir,"query.json",&serde_json::to_value(&cancel).unwrap()).unwrap();
        let got=s.poll_query().unwrap().unwrap();c.apply(&got.command,true,false).unwrap();
        assert_eq!(c.reason,"control-cancelled");assert_eq!(c.remaining,0);
        s.finish(json!({"remaining":0,"pending_control":s.pending_status()}),Some("Step stopped: control-cancelled".into())).unwrap();
        s.reply(&got,json!({"pending_control":s.pending_status()}),None).unwrap();
        assert_eq!(read(&s.dir.join("result.json")).unwrap()["id"],"step1");
        assert_eq!(read(&s.dir.join("result.json")).unwrap()["outcome"],"cancelled");
        assert!(read(&s.dir.join("result.json")).unwrap()["status"]["pending_control"].is_null());
        assert_eq!(read(&s.dir.join("query-result.json")).unwrap()["id"],"cancel1");
        assert_eq!(read(&s.dir.join("query-result.json")).unwrap()["operation_id"],"control.cancel");
        drop((primary,query_lock));
    }
    #[test]
    fn protocol_consumes_stale_session_and_cross_channel_requests_without_execution(){
        let t=tempfile::tempdir().unwrap();let mut s=synthetic_server(t.path());
        for (field,value) in [("run",json!("old")),("schema",json!(2)),("session_manifest_sha256",json!("other"))]{
            let mut request=serde_json::to_value(synthetic_request(&s,"stale",Action::Pause,false)).unwrap();
            request[field]=value;put(&s.dir,"request.json",&request).unwrap();
            assert!(s.poll().is_err());assert!(!s.dir.join("request.json").exists());
        }
        let request=synthetic_request(&s,"wrong",Action::Step{frames:1,keys:0x3ff},true);
        put(&s.dir,"query.json",&serde_json::to_value(request).unwrap()).unwrap();assert!(s.poll_query().is_err());
        let request=synthetic_request(&s,"cancel",Action::Cancel{request_id:"id".into()},false);
        put(&s.dir,"request.json",&serde_json::to_value(request).unwrap()).unwrap();assert!(s.poll().is_err());
        let run=s.run.clone();drop(s);let s=Server::new(t.path()).unwrap();assert_ne!(s.run,run);
        assert_eq!(s.ready(true)["session"]["manifest_sha256"],gba_core::ffta_rom::digest(&fs::read(t.path().join("debug-session.json")).unwrap()));
        let mut s=s;s.pending=Some(synthetic_request(&s,"closing",Action::Step{frames:1,keys:0x3ff},false));
        let dir=s.dir.clone();drop(s);
        let closed=read(&dir.join("result.json")).unwrap();
        assert_eq!(closed["outcome"],"closed");assert_eq!(closed["id"],"closing");assert!(closed["status"].is_null());

    }
    #[test]
    fn deadlines_client_timeout_and_late_ack_are_structured_and_bounded(){
        let t=tempfile::tempdir().unwrap();let mut s=synthetic_server(t.path());let ready=s.ready(true);
        let value=exchange(&s.dir,&ready,Action::Step{frames:600,keys:0x3ff},false,20).unwrap();
        assert_eq!(value["outcome"],"timeout");assert_eq!(value["server_acknowledged"],false);
        let request=s.poll().unwrap().unwrap();assert!(request.expired());
        s.pending=Some(request.clone());assert!(s.deadline_expired());
        s.finish(json!({"frame":7,"pending_control":s.pending_status()}),Some("Step stopped: control-timeout".into())).unwrap();
        let result=read(&s.dir.join("result.json")).unwrap();
        assert_eq!(result["id"],value["id"]);assert_eq!(result["outcome"],"timeout");assert_eq!(result["ok"],false);
        assert!(result["status"]["pending_control"].is_null());
        // Late synchronous completion is also honest about a missed deadline.
        s.reply(&request,json!({"completed":true}),None).unwrap();
        assert_eq!(read(&s.dir.join("result.json")).unwrap()["ok"],false);
        let a=|v:&[&str]|v.iter().map(|s|s.to_string()).collect::<Vec<_>>();
        for v in [vec!["status","case","--timeout-ms","0"],vec!["status","case","--timeout-ms","120001"],vec!["status","case","--timeout-ms","20","--timeout-ms","30"]]{assert!(options(&a(&v)).is_err());}
        assert_eq!(options(&a(&["status","case","--timeout-ms","20"])).unwrap().1,20);
        assert!(parse(&a(&["cancel","case"])).is_err());
        assert!(matches!(parse(&a(&["cancel","case","--request","exact"])).unwrap().1,Action::Cancel{..}));
    }
    #[test]
    fn terminal_publication_retry_keeps_job_identity_and_cleared_pending(){
        let t=tempfile::tempdir().unwrap();let mut s=synthetic_server(t.path());
        s.pending=Some(synthetic_request(&s,"job-owner",Action::Step{frames:1,keys:0x3ff},false));
        let status=json!({"frame":1,"pending_control":s.pending_status()});
        fs::create_dir(s.dir.join("result.json")).unwrap();
        assert!(s.finish(status,None).is_err());assert!(s.pending.is_none());
        let context=s.publication_context(false).unwrap();assert_eq!(context.request_id.as_deref(),Some("job-owner"));assert_eq!(context.job_id.as_deref(),Some("job-owner"));
        assert!(s.poll().is_err());fs::remove_dir(s.dir.join("result.json")).unwrap();
        assert!(s.poll().unwrap().is_none());let result=read(&s.dir.join("result.json")).unwrap();
        assert_eq!(result["id"],"job-owner");assert!(result["status"]["pending_control"].is_null());assert_eq!(result["status"]["frame"],1);
        assert!(s.publication_context(false).is_none());
    }
    #[test]
    fn query_publication_failure_keeps_primary_ack_and_retries_same_query(){
        let t=tempfile::tempdir().unwrap();let mut s=synthetic_server(t.path());
        let q=synthetic_request(&s,"query",Action::Status,true);
        fs::create_dir(s.dir.join("query-result.json")).unwrap();
        assert!(s.reply(&q,json!({"v":1}),None).is_err());
        assert_eq!(s.publication_context(true).unwrap().request_id.as_deref(),Some("query"));
        assert!(s.publication_context(false).is_none());
        let p=synthetic_request(&s,"primary",Action::Pause,false);
        s.reply(&p,json!({"v":2}),None).unwrap();
        assert_eq!(read(&s.dir.join("result.json")).unwrap()["id"],"primary");
        fs::remove_dir(s.dir.join("query-result.json")).unwrap();
        assert!(s.poll_query().unwrap().is_none());
        assert!(s.publication_context(true).is_none());
        assert_eq!(read(&s.dir.join("query-result.json")).unwrap()["id"],"query");
    }
    #[test]
    fn recent_parser_bounds_and_frontend_ownership(){
        let args=|v:&[&str]|v.iter().map(|s|s.to_string()).collect::<Vec<_>>();
        assert!(matches!(parse(&args(&["recent-start","case","--events","16","--reads","on","--start","00000000","--end","01000000","--on-error","off"])).unwrap().1,Action::RecentStart{..}));
        assert!(matches!(parse(&args(&["recent-freeze","case"])).unwrap().1,Action::RecentFreeze));
        assert!(matches!(parse(&args(&["recent-report","case"])).unwrap().1,Action::RecentReport));
        for v in [vec!["recent-start","case","--events","0"],vec!["recent-start","case","--events","1025"],vec!["recent-start","case","--reads","yes"],vec!["recent-start","case","--start","01000000","--end","0"],vec!["recent-start","case","--on-error","on","--on-error","off"]]{assert!(parse(&args(&v)).is_err());}
        assert!(Control::default().apply(&Action::RecentFreeze,true,false).is_err());
    }
    #[test]
    fn gameplay_parser_bounds_filters_and_paused_ownership() {
        let args=|a:&[&str]|a.iter().map(|s|s.to_string()).collect::<Vec<_>>();
        assert!(matches!(parse(&args(&["gameplay-start","case","--frames","2","--family","script","--context","02000000"])).unwrap().1,Action::GameplayStart{..}));
        assert!(matches!(parse(&args(&["gameplay-report","case"])).unwrap().1,Action::GameplayReport));
        for a in [vec!["gameplay-start","case","--frames","61"],vec!["gameplay-start","case","--events","513"],vec!["gameplay-start","case","--family","other"],vec!["gameplay-start","case","--context","fffffffff"],vec!["gameplay-start","case","--family","unit","--family","unit"]] {assert!(parse(&args(&a)).is_err());}
        assert!(Control::default().apply(&Action::GameplayReport,true,false).is_err());
    }
    #[test]
    fn audio_commands_validate_window_and_stream_selection() {
        let args=|v:&[&str]|v.iter().map(|s|s.to_string()).collect::<Vec<_>>();
        assert!(matches!(parse(&args(&["audio-start","case","--frames","2","--streams","mixed,hle"])).unwrap().1,Action::AudioStart{..}));
        assert!(matches!(parse(&args(&["audio-report","case"])).unwrap().1,Action::AudioReport));
        for v in [vec!["audio-start","case","--frames","181"],vec!["audio-start","case","--streams","mixed,mixed"],vec!["audio-start","case","--frames","2","--frames","3"]]{assert!(parse(&args(&v)).is_err());}
    }

    #[test]
    fn issue_command_bounds_and_selected_profile() {
        let a=|v:&[&str]|v.iter().map(|s|s.to_string()).collect::<Vec<_>>();
        assert!(matches!(parse(&a(&["issue-report","session","--profile","case/replay.json","--symptom","Observed issue"])).unwrap().1,Action::IssueReport{..}));
        assert!(parse(&a(&["issue-report","case","--profile","one","--profile","two"])).is_err());
        assert!(Action::IssueReport{profile:None,symptom:"x".repeat(2049)}.validate().is_err());
        assert!(Action::IssueReport{profile:Some(PathBuf::from("\\".repeat(1000))),symptom:"\n".repeat(1500)}.validate().is_err());
    }

    #[test]
    fn code_commands_validate_bounded_filters() {
        let a=|v:&[&str]|v.iter().map(|s|s.to_string()).collect::<Vec<_>>();
        assert!(matches!(parse(&a(&["code-start","case","--frames","2","--start","08000000","--end","09000000","--mode","thumb","--kind","missing"])).unwrap().1,Action::CodeStart{..}));
        assert!(matches!(parse(&a(&["code-report","case"])).unwrap().1,Action::CodeReport));
        for v in [vec!["code-start","case","--events","513"],vec!["code-start","case","--kind","unknown"],vec!["code-start","case","--frames","0"],vec!["code-start","case","--mode","arm","--mode","thumb"]]{assert!(parse(&a(&v)).is_err());}
    }

    #[test]
    fn ui_commands_are_explicit_frontend_actions() {
        let args=|parts:&[&str]|parts.iter().map(|p|p.to_string()).collect::<Vec<_>>();
        assert!(matches!(parse(&args(&["ui-start","case"])).unwrap().1,Action::UiStart));
        assert!(matches!(parse(&args(&["ui-report","case"])).unwrap().1,Action::UiReport));
        assert!(parse(&args(&["ui-start","case","extra"])).is_err());
        assert!(Control::default().apply(&Action::UiReport,true,false).is_err());
    }
    #[test]
    fn exact_budget_and_invalid_commands_are_transactional() {
        let mut c = Control::default();
        assert!(c
            .apply(
                &Action::Step {
                    frames: 2,
                    keys: 0x3ff
                },
                false,
                false
            )
            .is_err());
        assert!(c
            .apply(
                &Action::Step {
                    frames: 0,
                    keys: 0x3ff
                },
                true,
                false
            )
            .is_err());
        assert!(c
            .apply(
                &Action::Step {
                    frames: 601,
                    keys: 0x3ff
                },
                true,
                false
            )
            .is_err());
        assert!(c
            .apply(
                &Action::Step {
                    frames: 1,
                    keys: 0x400
                },
                true,
                false
            )
            .is_err());
        assert!(c.apply(&Action::Resume, true, true).is_err());
        assert!(c
            .apply(
                &Action::Step {
                    frames: 2,
                    keys: 0x3fe
                },
                true,
                false
            )
            .unwrap());
        assert!(c
            .apply(
                &Action::Step {
                    frames: 1,
                    keys: 0x3ff
                },
                true,
                false
            )
            .is_err());
        c.completed_frame(42, 43).unwrap();
        assert_eq!(c.remaining, 1);
        c.completed_frame(43, 44).unwrap();
        assert_eq!(c.remaining, 0);
        assert_eq!(c.reason, "step-complete");
        assert!(!c.apply(&Action::Resume, true, false).unwrap());
        assert!(c.apply(&Action::Pause, false, false).unwrap());
        assert_eq!(c.remaining, 0);
    }
    #[test]
    fn guard_does_not_count_an_incomplete_or_multiple_frame() {
        for delta in [0, 2] {
            let mut c = Control::default();
            c.apply(
                &Action::Step {
                    frames: 2,
                    keys: 0x3ff,
                },
                true,
                false,
            )
            .unwrap();
            assert!(c.completed_frame(4, 4 + delta).is_err());
            assert_eq!(c.remaining, 0);
            assert_eq!(c.reason, "incomplete-frame");
        }
    }
    #[test]
    fn mailbox_is_run_bound_consumed_once_and_retained_in_constant_space() {
        let t = tempfile::tempdir().unwrap();
        fs::write(t.path().join("debug-session.json"),br#"{"label":"SYNTHETIC"}"#).unwrap();
        let mut s = Server::new(t.path()).unwrap();
        let request = Request {
            schema: 1,
            run: s.run.clone(),
            id: "one".into(),
            command: Action::Status,
            deadline_unix_ms:None,session_manifest_sha256:None,query:false,
        };
        put(
            &s.dir,
            "request.json",
            &serde_json::to_value(&request).unwrap(),
        )
        .unwrap();
        let r = s.poll().unwrap().unwrap();
        assert!(s.poll().unwrap().is_none());
        s.reply(&r, json!({"frame":42}), None).unwrap();
        assert_eq!(read(&s.dir.join("result.json")).unwrap()["id"], "one");
        let mut stale = request;
        stale.run = "old".into();
        put(
            &s.dir,
            "request.json",
            &serde_json::to_value(stale).unwrap(),
        )
        .unwrap();
        assert!(s.poll().is_err());
        let dir = s.dir.clone();
        drop(s);
        assert_eq!(read(&dir.join("ready.json")).unwrap()["active"], false);
    }
    #[test]
    fn failed_response_publication_retries_only_the_same_result() {
        let t = tempfile::tempdir().unwrap();
        fs::write(t.path().join("debug-session.json"),br#"{"label":"SYNTHETIC"}"#).unwrap();
        let mut s = Server::new(t.path()).unwrap();
        let r = Request {
            schema: 1,
            run: s.run.clone(),
            id: "once".into(),
            command: Action::Pause,
            deadline_unix_ms:None,session_manifest_sha256:None,query:false,
        };
        put(&s.dir, "request.json", &serde_json::to_value(&r).unwrap()).unwrap();
        let request = s.poll().unwrap().unwrap();
        // A directory at the destination deterministically blocks atomic replace.
        fs::create_dir(s.dir.join("result.json")).unwrap();
        assert!(s.reply(&request, json!({"counter":1}), None).is_err());
        assert!(s.outbox.is_some());
        assert!(!s.dir.join("request.json").exists());
        fs::remove_dir(s.dir.join("result.json")).unwrap();
        assert!(s.poll().unwrap().is_none());
        assert!(s.outbox.is_none());
        let result = read(&s.dir.join("result.json")).unwrap();
        assert_eq!(result["id"], "once");
        assert_eq!(result["status"]["counter"], 1);
    }

    #[test]
    fn diagnostic_parser_keeps_modes_ranges_values_and_bounds_explicit() {
        let a = |s: &str| s.split_whitespace().map(str::to_owned).collect::<Vec<_>>();
        assert!(matches!(parse(&a("graphics session --compare-last")).unwrap().1,Action::Graphics{compare_last:true}));
        assert!(parse(&a("graphics session --bad")).is_err());
        assert!(matches!(
            parse(&a("inspect session 08000000 256")).unwrap().1,
            Action::Inspect {
                address: 0x08000000,
                length: 256
            }
        ));
        assert!(matches!(
            parse(&a("break session 08000000 arm --steps 10"))
                .unwrap()
                .1,
            Action::Find { .. }
        ));
        assert!(matches!(
            parse(&a(
                "watch session write 02000000 02000004 --width 4 --value ff --steps 12"
            ))
            .unwrap()
            .1,
            Action::Find { .. }
        ));
        for text in [
            "inspect session 04000000 257",
            "break session 08000002 arm",
            "break session 08000000 bad",
            "break session 08000000 arm --steps 0",
            "watch session read 02000004 02000000",
            "watch session read 02000000 02000004 --width 3",
            "watch session read 02000000 02000004 --width 1 --value 100",
            "watch session read 02000000 02000004 --width 1 --width 2",
        ] {
            assert!(parse(&a(text)).is_err(), "{text}");
        }
    }

    #[test]
    fn parser_bounds_and_key_masks() {
        let args = |s: &str| s.split_whitespace().map(str::to_owned).collect::<Vec<_>>();
        assert!(matches!(
            parse(&args("step session 3 --keys 3fe")).unwrap().1,
            Action::Step {
                frames: 3,
                keys: 0x3fe
            }
        ));
        for s in [
            "step session 0",
            "step session 601",
            "step session 1 --keys 400",
            "resume session extra",
            "pause",
            "step session 1 --bad 3ff",
        ] {
            assert!(parse(&args(s)).is_err(), "{s}");
        }
    }
}
