//! Bounded recent metadata, independent of guest state and prefix captures.
use gba_core::Machine;
use serde::{Deserialize,Serialize};
use serde_json::{Value,json};
use std::{collections::VecDeque,fs,path::Path,time::{Instant,SystemTime,UNIX_EPOCH}};

const BYTE_LIMIT:usize=256*1024;
const EVENT_LIMIT:usize=8*1024;
const READ_KEYS:usize=16;
#[derive(Clone,Debug,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Settings {
    pub events:usize,
    pub reads:bool,
    pub start:u32,
    pub end:u32,
    pub on_error:bool,
}
impl Default for Settings {
    fn default()->Self{Self{events:1024,reads:true,start:0,end:0x02000000,on_error:true}}
}
impl Settings {
    pub fn validate(&self)->Result<(),String>{
        if !(1..=1024).contains(&self.events)||self.start>=self.end||self.end>0x02000000{
            return Err("Recent events need1..1024entries and an exclusive physical ROM range START<END<=02000000".into());
        }Ok(())
    }
}
fn short(s:&str,n:usize)->String{s.chars().take(n).collect()}
fn boundary(m:&Machine)->Value{json!({"frame":m.bus.frames,"clock":m.bus.clock,"pc":m.cpu.regs[15],"thumb":m.cpu.thumb(),"keys":m.bus.keys})}
pub(crate) fn current(m:&Machine)->Result<Value,String>{
    let mut value=boundary(m);value["state_sha256"]=json!(gba_core::ffta_rom::digest(&m.save_state()?));Ok(value)
}
fn base(root:&Path)->Result<std::path::PathBuf,String>{
    let mut path=root.to_owned();
    for part in ["Data","config","recent"]{
        path.push(part);
        match fs::symlink_metadata(&path){
            Ok(meta)=>{
                let linked=meta.file_type().is_symlink();
                #[cfg(windows)]
                let linked={
                    use std::os::windows::fs::MetadataExt;
                    linked||meta.file_attributes()&0x400!=0
                };
                if linked||!meta.is_dir(){return Err("Recent output requires ordinary directories".into());}
            },
            Err(e) if e.kind()==std::io::ErrorKind::NotFound=>{},
            Err(e)=>return Err(e.to_string()),
        }
    }Ok(path)
}
pub(crate) fn capacity(root:&Path)->Result<std::path::PathBuf,String>{
    let path=base(root)?;
    if path.exists()&&fs::read_dir(&path).map_err(|e|e.to_string())?.count()>=8{
        return Err("Eight recent-event reports already exist; retain/archive before another capture".into());
    }Ok(path)
}
struct Entry{value:Value,bytes:usize}
pub(crate) struct Buffer {
    clock:crate::debug_events::Clock,
    console:Option<std::rc::Weak<std::cell::RefCell<crate::debug_console::Window>>>,
    settings:Settings,origin:Value,started:Instant,active:bool,reason:&'static str,
    entries:VecDeque<Entry>,bytes:usize,next:u64,evicted:u64,oversize:u64,
    frame_attempts:u64,completed_frames:u64,read_dropped:u64,read_unavailable:u64,
    last_input:Option<u16>,last_scene:Option<Value>,frozen_at:Option<Value>,
    published:Option<String>,write_error:Option<String>,
}
impl Buffer {
    fn new(settings:Settings,origin:Value)->Self{
        let count=settings.events;
        Self{clock:Default::default(),console:None,settings,origin,started:Instant::now(),active:true,reason:"rolling",
            entries:VecDeque::with_capacity(count),bytes:0,next:0,evicted:0,oversize:0,
            frame_attempts:0,completed_frames:0,read_dropped:0,read_unavailable:0,
            last_input:None,last_scene:None,frozen_at:None,published:None,write_error:None}
    }
    #[cfg(test)]
    pub fn start(root:&Path,m:&Machine,settings:Settings,origin:Value,scene:Value)->Result<Self,String>{
        Self::start_shared(root,m,settings,origin,scene,Default::default())
    }
    #[cfg(test)]
    pub fn start_shared(root:&Path,m:&Machine,settings:Settings,origin:Value,scene:Value,clock:crate::debug_events::Clock)->Result<Self,String>{
        Self::start_shared_observing(root,m,settings,origin,scene,clock,None)
    }
    pub fn start_shared_observing(root:&Path,m:&Machine,settings:Settings,mut origin:Value,scene:Value,clock:crate::debug_events::Clock,console:Option<std::rc::Weak<std::cell::RefCell<crate::debug_console::Window>>>)->Result<Self,String>{
        settings.validate()?;capacity(root)?;
        let manifest=crate::diagnostics::read(&root.join("debug-session.json"),1024*1024)?;
        let declared:Value=serde_json::from_slice(&manifest).map_err(|e|e.to_string())?;
        origin["session_manifest_sha256"]=json!(gba_core::ffta_rom::digest(&manifest));
        origin["session_pins"]=json!({"label":declared["label"],"rom":declared["pins"]["ffta.gba"],"seed":declared["pins"]["seed.ffstate"],"runtime":declared["pins"]["recomp.exe"],"dataset_manifest":declared["pins"]["Data/dataset/manifest.toml"]});
        origin["start"]=current(m)?;origin["rom_sha256"]=json!(gba_core::ffta_rom::digest(&m.bus.rom));
        origin["bios"]=json!({"real":m.bus.real_bios,"sha256":gba_core::ffta_rom::digest(&m.bus.bios)});
        if serde_json::to_vec(&origin).map_err(|e|e.to_string())?.len()>128*1024{return Err("Recent origin exceeds128KiB".into());}
        let mut buffer=Self::new(settings,origin);
        buffer.clock=clock;buffer.console=console;
        buffer.event(m,"armed",json!({"scope":"Recent rolling metadata; host time is independent of guest clocks"}));
        buffer.input(m);buffer.scene(m,scene);Ok(buffer)
    }
    pub fn enabled(&self)->bool{self.active}
    pub fn lifecycle_status(&self)->Value{
        json!({"collecting":self.active,"reason":self.reason,"completed_guest_frames":self.completed_frames,"read_requests_dropped":self.read_dropped,"unavailable_read_frames":self.read_unavailable,"report":self.published})
    }
    pub fn saved(&self)->bool{self.published.is_some()}
    pub fn event(&mut self,m:&Machine,stage:&str,data:Value){
        if !self.active{return;}
        let event=self.clock.event(m.bus.frames,m.bus.clock,stage,data);
        self.accept(&event);
        if let Some(console)=self.console.as_ref().and_then(|weak|weak.upgrade()){console.borrow_mut().accept(&event);}
    }
    #[cfg(test)]
    fn push(&mut self,frame:u64,clock:u64,stage:&str,data:Value){
        if !self.active{return;}
        let event=self.clock.event(frame,clock,stage,data);
        self.accept(&event);
    }
    pub fn accept(&mut self,event:&crate::debug_events::Event){
        if !self.active{return;}
        let sequence=self.next;self.next=self.next.saturating_add(1);
        let host_ms=self.started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
        let value=event.value(sequence,host_ms);
        let bytes=serde_json::to_vec(&value).unwrap().len();
        if bytes>EVENT_LIMIT{self.oversize=self.oversize.saturating_add(1);return;}
        while self.entries.len()>=self.settings.events||self.bytes+bytes>BYTE_LIMIT {
            let entry=self.entries.pop_front().unwrap();self.bytes-=entry.bytes;self.evicted=self.evicted.saturating_add(1);
        }
        self.bytes+=bytes;self.entries.push_back(Entry{value,bytes});
    }
    pub fn input(&mut self,m:&Machine){
        if self.active&&self.last_input!=Some(m.bus.keys){
            self.last_input=Some(m.bus.keys);
            self.event(m,"guest-input",json!({"keys":m.bus.keys,"meaning":"Actual active-low key latch before frame execution, not a raw controller poll"}));
        }
    }
    pub fn scene(&mut self,m:&Machine,scene:Value){
        let scene=if serde_json::to_vec(&scene).unwrap().len()>2048{json!({"unknown":"Scene metadata exceeded2048bytes"})}else{scene};
        if self.active&&self.last_scene.as_ref()!=Some(&scene){
            self.event(m,"recognized-ui-transition",json!({"previous":self.last_scene,"current":scene,"scope":"Partial frontend recognizer/signature; includes selection changes, not global scene identity"}));
            self.last_scene=Some(scene);
        }
    }
    pub fn freeze(&mut self,m:&Machine,reason:&'static str){
        if self.active{
            self.event(m,"freeze",json!({"reason":reason}));self.seal(m,reason);
        }
    }
    fn seal(&mut self,m:&Machine,reason:&'static str){self.frozen_at=Some(boundary(m));self.active=false;self.reason=reason;}
    #[cfg(test)]
    pub fn error(&mut self,m:&Machine,message:&str)->bool{
        if !self.active{return false;}
        self.event(m,"error",json!({"message":short(message,1024),"message_truncated":message.chars().count()>1024,"pc":m.cpu.regs[15],"keys":m.bus.keys}));
        self.trigger_error(m)
    }
    pub fn trigger_error(&mut self,m:&Machine)->bool{
        if self.active&&self.settings.on_error{self.seal(m,"error-trigger");true}else{false}
    }
    pub fn status(&self)->Value{
        json!({"collecting":self.active,"reason":self.reason,"settings":self.settings,
            "events":self.entries.len(),"event_bytes":self.bytes,"max_event_bytes":EVENT_LIMIT,"max_buffer_bytes":BYTE_LIMIT,
            "lifetime":{"attempted_events":self.next,"evicted_events":self.evicted,"oversize_dropped":self.oversize,"frame_attempts":self.frame_attempts,"completed_frames":self.completed_frames,
                "read_requests_dropped":self.read_dropped,"unavailable_read_frames":self.read_unavailable},
            "window":{"first":self.entries.front().map(|e|json!({"sequence":e.value["sequence"],"host_ms":e.value["host_ms"],"frame":e.value["frame"],"clock":e.value["clock"]})),
                "last":self.entries.back().map(|e|json!({"sequence":e.value["sequence"],"host_ms":e.value["host_ms"],"frame":e.value["frame"],"clock":e.value["clock"]}))},
            "last_observed_input":self.last_input,"last_observed_scene":self.last_scene,"frozen_at":self.frozen_at,
            "report":self.published,"write_error":self.write_error,"event_clock":self.clock.status()})
    }
    pub fn save(&mut self,root:&Path,m:&Machine)->Result<String,String>{
        if let Some(path)=&self.published{return Ok(path.clone());}
        self.freeze(m,"report-request");
        let result=self.publish(root,m);
        if let Err(error)=&result{self.write_error=Some(short(error,1024));}
        if let Ok(path)=&result{self.published=Some(path.clone());self.write_error=None;}
        result
    }
    fn publish(&self,root:&Path,m:&Machine)->Result<String,String>{
        let base=capacity(root)?;
        let report=json!({"schema":1,"kind":"ffta-recent-events","origin":self.origin,"current":current(m)?,"buffer":self.status(),
            "events":self.entries.iter().map(|e|&e.value).collect::<Vec<_>>(),"event_coverage":crate::debug_events::coverage(),
            "scope":{"window":"Recent metadata, not a complete replay. Sequence gaps/evictions/oversize and actual first/last boundaries are explicit. Lifetime totals are not window totals; post-freeze events are unobserved",
            "scene":"Partial recognized UI/signature transitions, including selections. Unrecognized or disabled recognizer is unknown, not absence of a scene",
            "reads":"Up to16selected unique instrumented ROM read keys per frame, actual typed/raw/unclassified service. Sorted aggregated keys, not within-frame chronology; instruction scope may include HLE/DMA and is unknown inside native blocks. Fetches excluded. Dropped/disabled/unavailable are not zero usage",
            "backend":"Actual native dispatch category counters when observed; interpreter-selected has no native lookup. Native block interiors and other error/reader hooks are not covered",
            "storage":"1024events/256KiB serialized event bytes/8KiB per event,8folders/2MiBJSON; no automatic overwrite/prune or new snapshot",
            "timing":"Host elapsed time and guest frame/clock remain distinct; restores freeze discontinuity. Only explicit start/save hashes full state, no continuous whole-machine dump",
            "acceptance":"No ROM/scene/native parity/performance acceptance or guest/save/native ABI change"}});
        let bytes=serde_json::to_vec(&report).map_err(|e|e.to_string())?;
        if bytes.len()>2*1024*1024{return Err("Recent JSON exceeds2MiB; frozen context retained".into());}
        fs::create_dir_all(&base).map_err(|e|e.to_string())?;
        let name=format!("{}-{}",SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e|e.to_string())?.as_nanos(),std::process::id());
        let dir=base.join(&name);fs::create_dir(&dir).map_err(|e|e.to_string())?;
        fs::write(dir.join(".gitignore"),"*\n").map_err(|e|e.to_string())?;
        let embedded=String::from_utf8(bytes.clone()).unwrap().replace('<',"\\u003c");
        fs::write(dir.join("index.html"),include_str!("debug_recent/report.html").replacen("/*REPORT*/",&embedded,1)).map_err(|e|e.to_string())?;
        fs::write(dir.join("recent.json"),bytes).map_err(|e|e.to_string())?;
        Ok(format!("Data/config/recent/{name}/index.html"))
    }
}

/// One existing frame loop's readonly source observer plus actual dispatch counts.
pub(crate) struct Frame {
    from:u64,clock:u64,keys:u16,native:bool,dispatch:[u64;4],read_state:&'static str,read_error:Option<String>,
    #[cfg(feature="rom-data-read-audit")]
    reads:Option<gba_core::rom_data::read_audit::TraceCapture>,
}
impl Frame {
    pub fn start(buffer:&mut Buffer,m:&Machine,native:bool)->Self{
        buffer.frame_attempts=buffer.frame_attempts.saturating_add(1);buffer.input(m);
        let frame=Self{from:m.bus.frames,clock:m.bus.clock,keys:m.bus.keys,native,dispatch:[0;4],read_state:if buffer.settings.reads{"feature-unavailable"}else{"disabled"},read_error:None,
            #[cfg(feature="rom-data-read-audit")]reads:None};
        #[cfg(feature="rom-data-read-audit")]
        let mut frame=frame;
        #[cfg(feature="rom-data-read-audit")]
        if buffer.settings.reads{
            use gba_core::rom_data::read_audit::{TraceCapture,TraceFilter};
            match TraceCapture::start(TraceFilter{readers:vec![],ranges:vec![(buffer.settings.start,buffer.settings.end)]},READ_KEYS){
                Ok(reads)=>{reads.frame(Some(m.bus.frames));frame.reads=Some(reads);frame.read_state="observed";},
                Err(error)=>{frame.read_state="observer-unavailable";frame.read_error=Some(short(&error,1024));},
            }
        }
        frame
    }
    pub fn native(&mut self,kind:crate::NativeDispatch){
        let index=match kind{crate::NativeDispatch::Native=>0,crate::NativeDispatch::Missing=>1,crate::NativeDispatch::System=>2,crate::NativeDispatch::Hook=>3};
        self.dispatch[index]=self.dispatch[index].saturating_add(1);
    }
    pub fn finish(self,buffer:&mut Buffer,m:&Machine,complete:bool){
        if complete{buffer.completed_frames=buffer.completed_frames.saturating_add(1);}
        let reads=json!({"state":self.read_state,"error":self.read_error,"max_unique_keys":READ_KEYS,"scope":"Per-frame sorted aggregate, not chronological reads"});
        #[cfg(feature="rom-data-read-audit")]
        let mut reads=reads;
        #[cfg(feature="rom-data-read-audit")]
        if let Some(capture)=self.reads{
            let report=capture.finish();
            buffer.read_dropped=buffer.read_dropped.saturating_add(report.dropped_requests);
            reads["selected"]=json!(report);
        }
        if !["observed","disabled"].contains(&self.read_state){buffer.read_unavailable=buffer.read_unavailable.saturating_add(1);}
        buffer.event(m,"rom-read-batch",reads);
        buffer.event(m,"frame",json!({"from_frame":self.from,"from_clock":self.clock,"keys":self.keys,"complete":complete,
            "backend":if self.native{"native-with-fallback"}else{"interpreter"},
            "actual_dispatch":if self.native{Some(json!({"native":self.dispatch[0],"missing":self.dispatch[1],"system":self.dispatch[2],"hook":self.dispatch[3]}))}else{None},
            "native_lookup_performed":self.native&&(self.dispatch[0]>0||self.dispatch[1]>0)}));
    }
}

#[cfg(test)]mod tests;
