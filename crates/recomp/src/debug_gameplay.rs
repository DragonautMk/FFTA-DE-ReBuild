//! Opt-in readonly observations of authenticated EUR task/script/unit contracts.
//! Addresses identify an observation, never a persistent actor or allocator ABI.
use gba_core::{Machine, cpu::FLAG_I, rom_data::{self, SourceRange}};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::Path, time::{Instant, SystemTime, UNIX_EPOCH}};

const BYTE_LIMIT: usize = 1024 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Settings {
    pub frames: u32,
    pub events: usize,
    pub family: String,
    pub context: Option<u32>,
}
impl Default for Settings {
    fn default() -> Self { Self { frames: 1, events: 512, family: "all".into(), context: None } }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=60).contains(&self.frames) || !(1..=512).contains(&self.events)
            || !["all", "task", "script", "unit"].contains(&self.family.as_str()) {
            return Err("Gameplay trace needs 1..60 frames, 1..512 events and family all/task/script/unit".into());
        }
        Ok(())
    }
}
struct Models {
    tasks: rom_data::task_descriptors::TaskDescriptors,
    primary: rom_data::script_commands::ScriptCommands,
    auxiliary: rom_data::auxiliary_scripts::AuxiliaryCommands,
    instructions: BTreeMap<u32, Value>,
    sprites: BTreeMap<u16, Value>,
    contracts: Value,
}
impl Models {
    fn from_rom(rom: &[u8]) -> Result<Self, String> {
        // Each catalog authenticates exact ROM and its existing source contract.
        let tasks=rom_data::task_descriptors::Catalog::from_rom(rom)?;
        let primary=rom_data::script_commands::Catalog::from_rom(rom)?;
        let archive=rom_data::script_archive::Catalog::from_rom(rom)?;
        let auxiliary=rom_data::auxiliary_scripts::Catalog::from_rom(rom)?;
        let units=rom_data::unit_obj_initial_pose::Catalog::from_rom(rom)?;
        let summary=|sources: &[SourceRange]|json!({"ranges":sources.len(),"bytes":sources.iter().map(|s|u64::from(s.length)).sum::<u64>(),"plan_sha256":gba_core::ffta_rom::digest(&serde_json::to_vec(sources).unwrap())});
        let contracts=json!({
            "task":{"guide":"docs/rom_data_tasks.md","source":summary(&tasks.sources)},
            "primary-script":{"guide":"docs/rom_data_scripts.md","commands":summary(&primary.sources),"archive":summary(&archive.sources)},
            "aux-script":{"guide":"docs/rom_data_scripts.md","source":summary(&auxiliary.sources)},
            "unit":{"guide":"docs/rom_data_unit_obj.md","source":summary(&units.sources)},
            "scope":"Authenticated source projections. Membership is correlation, not a runtime domain cap, execution proof or typed-service observation"});
        let mut instructions=BTreeMap::new();
        for p in &archive.decoded.programs {
            for i in &p.instructions { instructions.insert(i.source_offset,json!({"contract":"primary-script","resource":format!("script-archive/program/{}/instruction/{:08x}",p.ordinal,i.source_offset),"rom_offset":i.source_offset,"program":p.ordinal})); }
        }
        for b in &auxiliary.decoded.banks {
            for p in &b.programs { for i in &p.instructions { instructions.insert(i.source_offset,json!({"contract":"aux-script","resource":format!("auxiliary-scripts/bank/{}/program/{}/instruction/{:08x}",b.id,p.id,i.source_offset),"rom_offset":i.source_offset,"bank":b.id,"program":p.id})); } }
        }
        let sprites=units.decoded.bindings.iter().map(|s|(s.sprite_id,json!({"contract":"unit","resource":format!("unit-obj-initial-pose/sprite/{}",s.sprite_id),"sprite":s.sprite_id,"index_rom_offset":s.index_source,"sequence_rom_offset":s.sequence_source,"scope":"Default animation 0/part 0 projection, not the current animation or persistent unit identity"}))).collect();
        Ok(Self { tasks:tasks.decoded,primary:primary.decoded,auxiliary:auxiliary.commands,instructions,sprites,contracts })
    }
    fn descriptor(&self, at:u32)->Value {
        for (table,(_,_,name)) in self.tasks.tables().into_iter().zip(rom_data::task_descriptors::TABLES) {
            if let Some(rel)=at.checked_sub(table.source_offset) {
                if rel%8==0 && (rel/8) < table.entries.len() as u32 {
                    return json!({"contract":"task","resource":format!("engine/task-descriptors/{name}/{}",rel/8),"rom_offset":at,"ordinal":rel/8,"table":name,
                        "scope":"Physical descriptor row, not a global ID. Primary/secondary selectors alias; primary 345 overrides callback/size separately"});
                }
            }
        }
        json!({"contract":"task","rom_offset":at,"unknown":"Outside authenticated descriptor projection; no guest selector bound inferred"})
    }
}
fn reg(m:&Machine,n:usize,name:&str,meaning:&str)->Value {
    json!({"name":name,"value":m.cpu.regs[n],"source":{"kind":"register","register":n,"pc":m.cpu.regs[15]},"meaning":meaning})
}
fn raw(m:&Machine,at:u32,width:u32)->Result<(u32,Value),String> {
    if !matches!(at>>24,2|3|8..=13){return Err("Only RAM or ROM backing is inspected; no IO/protocol/unknown pointer dereference".into());}
    let b=gba_core::debug_memory::peek(m,at,width)?;
    let value=b.iter().enumerate().fold(0u32,|v,(i,b)|v|(u32::from(b.value)<<(8*i)));
    Ok((value,json!({"kind":if b[0].region=="rom-backing"{"ROM"}else{"RAM"},"address":at,"width":width,"region":b[0].region,"physical_offset":b[0].offset,"pc":m.cpu.regs[15],"read":"Readonly backing, not a bus read or typed-service event"})))
}
fn memory(m:&Machine,at:u32,width:u32,name:&str,meaning:&str)->Value {
    match raw(m,at,width) {
        Ok((value,source))=>json!({"name":name,"value":value,"source":source,"meaning":meaning}),
        Err(unknown)=>json!({"name":name,"value":null,"source":{"address":at,"width":width,"pc":m.cpu.regs[15]},"unknown":unknown,"meaning":meaning}),
    }
}
fn ram_field(m:&Machine,ctx:u32,off:u32,width:u32,name:&str,meaning:&str)->Value {
    if !matches!(ctx>>24,2|3){return json!({"name":name,"value":null,"source":{"context":ctx,"offset":off,"pc":m.cpu.regs[15]},"unknown":"Context is not a RAM pointer"});}
    memory(m,ctx.wrapping_add(off),width,name,meaning)
}
fn field_value(fields:&[Value],name:&str)->Option<u32> {fields.iter().find(|f|f["name"]==name)?.get("value")?.as_u64().map(|v|v as u32)}
fn signed_cursor(m:&Machine,ctx:u32,off:u32)->Value {
    let mut f=ram_field(m,ctx,off,2,"cursor","Signed 16-bit cursor, original wrapping advance and relative callbacks remain live");
    if let Some(v)=f["value"].as_u64(){f["signed"]=json!(v as u16 as i16);}
    f
}
fn effective(stream:u32,cursor:u16)->u32 { stream.wrapping_add((cursor as i16 as i32) as u32) }
fn instruction_eligible(m:&Machine)->bool {
    !m.bus.halted && !(!m.bus.real_bios && m.cpu.regs[15]==gba_core::machine::IRQ_RETURN_ADDR && m.cpu.mode()==gba_core::Mode::Irq)
        && !(m.bus.irq_pending() && !m.cpu.flag(FLAG_I))
}
fn boundary_kind(k:crate::NativeDispatch)->&'static str {
    match k{crate::NativeDispatch::Native=>"native",crate::NativeDispatch::Missing=>"missing",crate::NativeDispatch::System=>"system",crate::NativeDispatch::Hook=>"hook"}
}

pub(crate) struct Trace {
    settings:Settings, origin:Value, models:Models, started:Instant, active:bool,
    reason:&'static str, events:Vec<Value>, event_bytes:usize, filtered:u64, frames:u32, timeline:Vec<Value>,
}
fn base(root:&Path)->Result<std::path::PathBuf,String>{
    let mut path=root.to_path_buf();
    for part in ["Data","config","gameplay"] {
        path.push(part);
        match fs::symlink_metadata(&path) {Ok(m) if !m.is_dir() || m.file_type().is_symlink()=>return Err("Gameplay destination must contain ordinary directories".into()),Ok(_)=>{},Err(e) if e.kind()==std::io::ErrorKind::NotFound=>{},Err(e)=>return Err(e.to_string())}
    }
    Ok(path)
}
fn capacity(root:&Path)->Result<std::path::PathBuf,String> {
    let b=base(root)?;
    if b.exists() && fs::read_dir(&b).map_err(|e|e.to_string())?.count()>=8{return Err("Eight gameplay reports already exist; retain/archive before another trace".into());}
    Ok(b)
}
impl Trace {
    pub fn start(root:&Path,m:&Machine,settings:Settings,mut origin:Value)->Result<Self,String> {
        settings.validate()?;capacity(root)?;
        let models=Models::from_rom(&m.bus.rom)?;
        let manifest=crate::diagnostics::read(&root.join("debug-session.json"),1024*1024)?;
        let pins:Value=serde_json::from_slice(&manifest).map_err(|e|e.to_string())?;
        origin["session_manifest_sha256"]=json!(gba_core::ffta_rom::digest(&manifest));
        origin["session_pins"]=json!({"label":pins["label"],"rom":pins["pins"]["ffta.gba"],"seed":pins["pins"]["seed.ffstate"],"bios":pins["pins"]["gba_bios.bin"],"runtime":pins["pins"]["recomp.exe"],"dataset_manifest":pins["pins"]["Data/dataset/manifest.toml"]});
        origin["state_sha256"]=json!(gba_core::ffta_rom::digest(&m.save_state()?));
        origin["rom_sha256"]=json!(gba_core::ffta_rom::digest(&m.bus.rom));
        origin["bios"]=json!({"real":m.bus.real_bios,"sha256":gba_core::ffta_rom::digest(&m.bus.bios)});
        origin["frame"]=json!(m.bus.frames);origin["clock"]=json!(m.bus.clock);origin["keys"]=json!(m.bus.keys);
        Ok(Self{settings,origin,models,started:Instant::now(),active:true,reason:"collecting",events:Vec::new(),event_bytes:0,filtered:0,frames:0,timeline:Vec::new()})
    }
    pub fn enabled(&self)->bool{self.active}
    pub fn freeze(&mut self,reason:&'static str){if self.active{self.active=false;self.reason=reason;}}
    pub fn tick(&mut self){if self.started.elapsed().as_secs()>=30{self.freeze("30-second-limit");}}
    pub fn status(&self)->Value{json!({"collecting":self.active,"reason":self.reason,"events":self.events.len(),"event_bytes":self.event_bytes,"filtered_contract_events":self.filtered,"completed_frames":self.frames,"settings":self.settings})}
    pub fn native(&mut self,m:&Machine,k:crate::NativeDispatch){self.observe(m,boundary_kind(k));}
    pub fn observe(&mut self,m:&Machine,boundary:&str){
        if !self.active || !m.cpu.thumb() || !instruction_eligible(m){return;}
        let pc=m.cpu.regs[15];
        // Select only contract sites, keeping the ordinary step path cheap.
        let (family,event,ctx_reg)=match pc {
            0x080d7062|0x080d7064|0x080d70a2|0x080d7346|0x080d7348=>("task","descriptor-read",0),
            0x080d707a|0x080d707c|0x080d76b6=>("task","descriptor-read",1),
            0x081265d4=>("script","primary-fetch",5),0x081265fa=>("script","primary-dispatch",5),0x08126610=>("script","primary-advanced",5),
            0x0800a248=>("script","aux-fetch",5),0x0800a274=>("script","aux-dispatch",5),0x0800a286=>("script","aux-advanced",5),
            0x08022400=>("unit","task-update-entry",0),0x08022488=>("unit","task-countdown-checkpoint",4),0x080224e2=>("unit","task-update-exit",4),
            0x08022e20=>("unit","task-reset-entry",0),0x080233a8=>("unit","sprite-query",0),0x080233ce=>("unit","sprite-result",4),
            _=>return,
        };
        self.tick();if !self.active{return;}
        let ctx=m.cpu.regs[ctx_reg];
        if self.settings.family!="all" && self.settings.family!=family || self.settings.context.is_some_and(|c|c!=ctx){self.filtered=self.filtered.saturating_add(1);return;}
        let mut fields=vec![reg(m,ctx_reg,"context","Address at this observation only; descriptor address for task reads, actor/task context otherwise")];
        let mut resources=Vec::new();
        let unknown=match family {"task"=>"Task pool, allocated context, lifetime and final callback unknown here. Primary 345 has a separate callback/size override", "script"=>"Other context fields, global script/actor identity and control-flow graph unknown",_=>"Other actor/task fields, allocator lifetime, persistent unit ID/name and flag semantics unknown"};
        if family=="task" {
            let (off,width,name)=match pc {0x080d7064|0x080d707c|0x080d7348=>(4,2,"context_size"),0x080d70a2=>(6,2,"metadata_raw"),0x080d76b6=>(6,1,"metadata_low_byte"),_=>(0,4,"descriptor_callback")};
            fields.push(memory(m,ctx.wrapping_add(off),width,name,if off==6{"Opaque metadata, no invented flag semantics"}else{"Current descriptor backing before original load; not the final scheduled task"}));
            if off==0 {if let Some(f)=fields.last_mut(){f["sentinel_candidate"]=json!(f["value"]==json!(0));f["sentinel_meaning"]=json!("Nullable descriptor callback; null is distinct from an unknown read");}}
            let offset=raw(m,ctx,1).ok().and_then(|(_,p)|if p["kind"]=="ROM"{p["physical_offset"].as_u64().map(|n|n as u32)}else{None});
            resources.push(offset.map(|at|self.models.descriptor(at)).unwrap_or(json!({"contract":"task","unknown":"Descriptor pointer is not ROM backing"})));
        }else if family=="script" {
            let aux=event.starts_with("aux");let off=if aux{2}else{0};
            fields.push(signed_cursor(m,ctx,off));fields.push(ram_field(m,ctx,if aux{4}else{12},4,"stream","Live stream pointer"));
            if aux {fields.push(ram_field(m,ctx,0,1,"action","Raw action byte, not capped by eight projected banks"));fields.push(ram_field(m,ctx,1,1,"slot","Raw slot byte, no global slot cap"));}
            if event.ends_with("advanced") {
                if !aux {fields.push(reg(m,2,"callback_result","0x110 repeats primary fetch; other results return"));}
                fields.push(reg(m,4,"directory_row","Row retained across callback; cursor already stored by original interpreter"));
                fields.push(memory(m,m.cpu.regs[4],2,"advance","Original advance applied after callback-modified cursor, 16-bit wrapping"));
            } else if let (Some(stream),Some(cursor))=(field_value(&fields,"stream"),field_value(&fields,"cursor")) {
                let at=effective(stream,cursor as u16);
                let opcode=memory(m,at,1,"opcode","Byte at signed cursor; auxiliary opcode 0 terminates before dispatch");
                if let Some(op)=opcode["value"].as_u64(){
                    let op=op as u8;let row=if aux{rom_data::auxiliary_scripts::command_source_offset(op)}else{rom_data::script_commands::command_source_offset(op)};
                    fields.push(memory(m,0x08000000+row,2,"advance","Directory u16 advance; not normalized to a host branch"));
                    fields.push(memory(m,0x08000002+row,4,"directory_callback","Unaligned byte-assembled Thumb pointer; auxiliary opcode 0 row is preserved but not dispatched"));
                    let known=if aux{self.models.auxiliary.commands.get(op as usize).is_some()}else{self.models.primary.commands.get(op as usize).is_some()};
                    resources.push(json!({"contract":if aux{"aux-script"}else{"primary-script"},"resource":format!("engine/{}-command-directory/{op}",if aux{"auxiliary-script"}else{"script"}),"rom_offset":row,"in_projection":known,"unknown":if known{None}else{Some("Outside projection; raw backing retained, no opcode cap inferred")},"terminator":aux&&op==0}));
                }
                fields.push(opcode);
                if let Ok((_,source))=raw(m,at,1){if source["kind"]=="ROM"{let offset=source["physical_offset"].as_u64().unwrap() as u32;resources.push(self.models.instructions.get(&offset).cloned().unwrap_or(json!({"rom_offset":offset,"unknown":"No projected instruction boundary at this physical byte; RAM/header/fill/relative targets are not normalized"})));}}
            }
            if event.ends_with("dispatch"){fields.push(reg(m,if aux{2}else{3},"dispatch_callback","Actual assembled callback register at BL trampoline"));fields.push(reg(m,4,"directory_row","Actual directory pointer register"));}
        }else if event=="sprite-query" {
            fields.push(reg(m,1,"selector_output_pointer","Opaque output pointer passed by original caller"));
        }else if event=="sprite-result" {
            fields.push(reg(m,0,"sprite_raw_result","Raw helper result before low16 truncation"));
            let sprite=m.cpu.regs[0] as u16;
            fields.push(json!({"name":"sprite_low16","value":sprite,"source":{"kind":"derived-register","register":0,"operation":"low16","pc":pc},"sentinel_candidate":sprite==u16::MAX,"meaning":"FFFF remains a raw candidate sentinel; asset validity unknown"}));
            resources.push(self.models.sprites.get(&sprite).cloned().unwrap_or(json!({"contract":"unit","sprite":sprite,"unknown":"Outside default initial-pose projection; no live sprite cap or valid asset inferred"})));
        }else {
            if event=="task-reset-entry" {
                fields.push(reg(m,1,"sprite_argument","Raw sprite argument before original truncation"));fields.push(reg(m,2,"animation_argument","Raw animation selector"));fields.push(reg(m,3,"part_argument","Raw part selector"));
            }
            for (off,width,name,meaning) in [(0,4,"task_flags_raw","Raw word; flag meanings not generalized"),(6,2,"sprite","Sprite field in unit graphics task"),(10,2,"countdown","Scheduler signed 16-bit countdown, raw value retained"),(12,2,"sequence_cursor","Scheduler signed 16-bit sequence cursor"),(14,2,"frame_count","Sequence frame count"),(0x20,4,"marker_raw","Reset stores FFFFFFFF; general semantics unknown"),(0x34,4,"sequence_frames","Sequence frame-start pointer"),(0x38,4,"current_frame","Current 20-byte frame pointer")] {
                let mut f=ram_field(m,ctx,off,width,name,meaning);
                if matches!(off,10|12){if let Some(v)=f["value"].as_u64(){f["signed"]=json!(v as u16 as i16);}}
                if off==0x20{f["sentinel_candidate"]=json!(f["value"]==json!(u32::MAX));}
                fields.push(f);
            }
            if let Some(sprite)=field_value(&fields,"sprite"){resources.push(self.models.sprites.get(&(sprite as u16)).cloned().unwrap_or(json!({"contract":"unit","sprite":sprite,"unknown":"Outside default projection; sprite domain/lifetime unknown"})));}
        }
        self.push(json!({"sequence":self.events.len(),"family":family,"event":event,"context":ctx,"pc":pc,"thumb":true,"boundary":boundary,"frame":m.bus.frames,"clock":m.bus.clock,"keys":m.bus.keys,"fields":fields,"resources":resources,"unknown":unknown}));
    }
    fn push(&mut self,event:Value){
        let bytes=serde_json::to_vec(&event).unwrap().len();
        if self.event_bytes+bytes>BYTE_LIMIT{self.freeze("1MiB-event-byte-limit");return;}
        self.event_bytes+=bytes;self.events.push(event);
        if self.events.len()>=self.settings.events{self.freeze("event-limit");}
    }
    pub fn completed_frame(&mut self,m:&Machine){
        self.frames+=1;self.timeline.push(json!({"frame":m.bus.frames,"clock":m.bus.clock,"keys":m.bus.keys,"events_retained":self.events.len()}));
        if self.active && self.frames>=self.settings.frames{self.freeze("frame-limit");}
    }
    pub fn save(&mut self,root:&Path,current:Value)->Result<String,String>{
        let base=capacity(root)?;self.freeze("report-request");
        let report=json!({"schema":1,"kind":"ffta-gameplay-inspection","origin":self.origin,"current":current,"trace":self.status(),"contracts":self.models.contracts,"events":self.events,"frames":self.timeline,
            "scope":{"boundary":"Before eligible interpreter instruction or native dispatch; native block interiors are unobserved. Missing sites/events are not zero activity",
            "identity":"Context/actor/task addresses can be reused; no persistent unit identity, whole pool/allocator layout or unknown fields inferred",
            "provenance":"Per-field immutable register/RAM/ROM backing at observed PC. Calculated links identify source projection only; no guest bus reads, writes or typed/raw-service classification",
            "limits":"30 seconds/60 frames/512 events/1 MiB event bytes; filtered/frozen suffix unobserved. Eight private report folders, 2 MiB JSON each; no snapshot, pruning or overwrite",
            "acceptance":"No new ROM/DS/native/performance acceptance"}});
        let bytes=serde_json::to_vec(&report).map_err(|e|e.to_string())?;
        if bytes.len()>2*1024*1024{return Err("Gameplay JSON exceeds 2 MiB; frozen trace retained".into());}
        fs::create_dir_all(&base).map_err(|e|e.to_string())?;
        let name=format!("{}-{}",SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e|e.to_string())?.as_nanos(),std::process::id());
        let dir=base.join(&name);fs::create_dir(&dir).map_err(|e|e.to_string())?;
        let embedded=String::from_utf8(bytes.clone()).unwrap().replace('<',"\\u003c");
        fs::write(dir.join("index.html"),include_str!("debug_gameplay/report.html").replacen("/*REPORT*/",&embedded,1)).map_err(|e|e.to_string())?;
        fs::write(dir.join("gameplay.json"),bytes).map_err(|e|e.to_string())?;
        Ok(format!("Data/config/gameplay/{name}/index.html"))
    }
}

#[cfg(test)]mod tests {
    use super::*;
    #[test]fn signed_wrapping_safe_backing_sentinels_and_projection_aliases(){
        assert_eq!(effective(0x08000000,0xffff),0x07ffffff);assert_eq!(effective(0,0x8000),0xffff8000);assert_eq!(effective(u32::MAX,1),0);
        let mut m=Machine::new(vec![0;16]);m.bus.ewram[0..2].copy_from_slice(&0xffffu16.to_le_bytes());
        let before=m.save_state().unwrap();let f=signed_cursor(&m,0x02040000,0);assert_eq!(f["signed"],-1);assert_eq!(f["value"],65535);assert_eq!(f["source"]["physical_offset"],0);
        for addr in [0x04000130,0x0e000000,0xfffffff0]{assert!(raw(&m,addr,2).is_err());}
        assert!(ram_field(&m,0x08000000,0,2,"cursor","")["value"].is_null());
        assert_eq!(before,m.save_state().unwrap());
        assert_eq!(rom_data::task_descriptors::primary_source_offset(362),rom_data::task_descriptors::secondary_source_offset(363));
        assert_eq!(rom_data::task_descriptors::primary_source_offset(0),0x3997a8);
        assert!(Settings{events:513,..Default::default()}.validate().is_err());
        assert!(Trace::start(Path::new("."),&m,Default::default(),json!({})).is_err());
    }
    #[test]fn bounded_prefix_filters_unknowns_and_failed_report_retains_trace(){
        let m=Machine::new(vec![0;16]);let d=tempfile::tempdir().unwrap();let empty=rom_data::task_descriptors::DescriptorTable{source_offset:0,entries:vec![]};
        let models=Models{tasks:rom_data::task_descriptors::TaskDescriptors{primary:empty.clone(),auxiliary:empty.clone(),script:empty},primary:rom_data::script_commands::ScriptCommands{source_offset:0,commands:vec![]},auxiliary:rom_data::auxiliary_scripts::AuxiliaryCommands{source_offset:0,commands:vec![]},instructions:Default::default(),sprites:Default::default(),contracts:json!({"synthetic":true})};
        let mut t=Trace{settings:Settings{events:2,family:"script".into(),context:Some(0x02000000),..Default::default()},origin:json!({"synthetic":true}),models,started:Instant::now(),active:true,reason:"collecting",events:vec![],event_bytes:0,filtered:0,frames:0,timeline:vec![]};
        let mut m=m;m.cpu.set_thumb(true);m.cpu.regs[15]=0x080233ce;t.observe(&m,"interpreter");assert_eq!(t.filtered,1);
        t.settings.family="all".into();t.settings.context=None;m.cpu.regs[0]=0xffff;m.cpu.regs[4]=0x04000000;t.observe(&m,"interpreter");
        assert_eq!(t.events[0]["fields"][2]["sentinel_candidate"],true);assert!(t.events[0]["resources"][0]["unknown"].is_string());
        m.cpu.regs[15]=0x081265d4;m.cpu.regs[5]=0x02000000;
        m.bus.ewram[..2].copy_from_slice(&0xffffu16.to_le_bytes());m.bus.ewram[12..16].copy_from_slice(&0x02000100u32.to_le_bytes());m.bus.ewram[255]=200;
        let before=m.save_state().unwrap();t.observe(&m,"interpreter");assert_eq!(before,m.save_state().unwrap());
        assert_eq!(t.events[1]["fields"][1]["signed"],-1);assert_eq!(field_value(t.events[1]["fields"].as_array().unwrap(),"opcode"),Some(200));
        assert_eq!(t.events[1]["resources"][0]["in_projection"],false);
        assert_eq!(t.reason,"event-limit");assert_eq!(t.events.len(),2);
        let base=d.path().join("Data/config/gameplay");fs::create_dir_all(&base).unwrap();for i in 0..8{fs::create_dir(base.join(i.to_string())).unwrap();}
        assert!(t.save(d.path(),json!({})).is_err());assert_eq!(t.events.len(),2);assert_eq!(fs::read_dir(&base).unwrap().count(),8);
        t.active=true;t.events.clear();t.event_bytes=BYTE_LIMIT;t.push(json!({"too":"large"}));assert_eq!(t.reason,"1MiB-event-byte-limit");assert!(t.events.is_empty());
        t.active=true;t.event_bytes=0;t.started=Instant::now()-std::time::Duration::from_secs(31);t.tick();assert_eq!(t.reason,"30-second-limit");
    }
    #[test]
    #[ignore="explicit local ROM/snapshot/output only; no private fixture distributed"]
    fn retained_interpreter_observer_preserves_known_sequence(){
        let path=|key|std::path::PathBuf::from(std::env::var_os(key).expect(key));
        let rom=fs::read(path("FFTA_GAMEPLAY_ROM")).unwrap();let seed=fs::read(path("FFTA_GAMEPLAY_STATE")).unwrap();let out=path("FFTA_GAMEPLAY_OUTPUT");
        assert!(!out.exists());fs::create_dir_all(&out).unwrap();fs::write(out.join("debug-session.json"),"{\"validation_fixture\":true}").unwrap();
        let mut plain=crate::make_machine(rom.clone(),None);crate::save_state::load_bytes(&mut plain,&seed).unwrap();
        let mut observed=crate::make_machine(rom,None);crate::save_state::load_bytes(&mut observed,&seed).unwrap();
        let mut t=Trace::start(&out,&observed,Settings{frames:2,..Default::default()},json!({"validation":"Same interpreter with/without new observer, not a new DS acceptance","seed_sha256":gba_core::ffta_rom::digest(&seed),"test_executable_sha256":gba_core::ffta_rom::digest(&fs::read(std::env::current_exe().unwrap()).unwrap())})).unwrap();
        for keys in [0x3fe,0x3ff] {
            plain.bus.keys=keys;observed.bus.keys=keys;plain.run_frame(5_000_000);
            crate::run_frame_interpreter_observed(&mut observed,5_000_000,|m|t.observe(m,"interpreter"));
            assert_eq!(plain.save_state().unwrap(),observed.save_state().unwrap());assert!(observed.bus.frame_ready);t.completed_frame(&observed);
        }
        let current=json!({"frame":observed.bus.frames,"clock":observed.bus.clock,"state_sha256":gba_core::ffta_rom::digest(&observed.save_state().unwrap()),"observer_off_on_full_state_equal":true});
        println!("GAMEPLAY_REPORT {}",t.save(&out,current).unwrap());
        assert!(!t.events.is_empty(),"Negative prefix retained; select existing nearer snapshot if known sites absent");
    }
}
