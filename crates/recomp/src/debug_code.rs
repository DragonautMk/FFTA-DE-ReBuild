//! Bounded pre-dispatch observations, separate from static labels and code extent.
use gba_core::{Machine, cpu::FLAG_I};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{fs, path::Path, time::{Instant, SystemTime, UNIX_EPOCH}};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Settings {
    pub frames: u32,
    pub events: usize,
    pub start: u32,
    pub end: u32,
    pub thumb: Option<bool>,
    pub kind: String,
}
impl Default for Settings {
    fn default() -> Self {
        Self { frames: 1, events: 512, start: 0, end: u32::MAX, thumb: None, kind: "all".into() }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=60).contains(&self.frames) || !(1..=512).contains(&self.events) || self.start >= self.end
            || !["all", "native", "missing", "system", "hook", "interpreter"].contains(&self.kind.as_str()) {
            return Err("Code trace needs1..60frames/1..512events, exclusive START<END and kind all/native/missing/system/hook/interpreter".into());
        }
        Ok(())
    }
}
fn system(m: &Machine) -> Option<&'static str> {
    if m.bus.halted { Some("halt-or-wake") }
    else if !m.bus.real_bios && m.cpu.regs[15] == gba_core::machine::IRQ_RETURN_ADDR && m.cpu.mode() == gba_core::Mode::Irq { Some("hle-irq-return") }
    else if m.bus.irq_pending() && !m.cpu.flag(FLAG_I) { Some("pending-irq") }
    else { None }
}
fn prefix(s: &str, count: usize) -> String { s.chars().take(count).collect() }

pub(crate) struct Trace {
    settings: Settings,
    origin: Value,
    labels: recomp_core::labels::Labels,
    label_identity: Value,
    started: Instant,
    active: bool,
    reason: &'static str,
    events: Vec<Value>,
    observed: u64,
    filtered: u64,
    frames: u32,
}
pub(crate) fn capacity(root: &Path) -> Result<(), String> {
    let base = root.join("Data/config/code");
    if base.exists() && fs::read_dir(base).map_err(|e|e.to_string())?.count() >= 8 {
        return Err("Eight code reports already exist; retain/archive before another trace".into());
    }
    Ok(())
}
impl Trace {
    pub fn start(root: &Path, m: &Machine, table: Option<&crate::BlockTable>, settings: Settings, mut origin: Value, selected_labels:Option<&Path>) -> Result<Self, String> {
        settings.validate()?;
        capacity(root)?;
        let default_path=root.join("debug-labels.toml");
        let path=selected_labels.unwrap_or(&default_path);
        let (labels, label_identity) = if path.exists() {
            let bytes = crate::diagnostics::read(&path, 1024*1024)?;
            let labels = recomp_core::labels::Labels::parse(std::str::from_utf8(&bytes).map_err(|e|e.to_string())?, &path,
                &gba_core::ffta_rom::digest(&m.bus.rom), m.bus.rom.len())?;
            let identity = json!({"file":path,"sha256":gba_core::ffta_rom::digest(&bytes),"rom_entries":labels.rom.len(),"iwram_entries":labels.iwram.len(),"reserved_entries":labels.reserved.len(),"skipped":labels.skipped,
                "named_candidates":labels.names.iter().take(64).map(|(key,name)|json!({"key":key,"name":prefix(name,128),"name_truncated":name.chars().count()>128,"end":labels.ends.get(key)})).collect::<Vec<_>>(),
                "named_candidates_omitted":labels.names.len().saturating_sub(64),"scope":"Declared static hints, not executed instructions, function extents or extracted-code coverage"});
            (labels, identity)
        } else { (Default::default(), json!({"file":null,"scope":"No selected label file"})) };
        let manifest = crate::diagnostics::read(&root.join("debug-session.json"), 1024*1024)?;
        origin["session_manifest_sha256"] = json!(gba_core::ffta_rom::digest(&manifest));
        origin["session_manifest"] = serde_json::from_slice(&manifest).map_err(|e|e.to_string())?;
        origin["state_sha256"] = json!(gba_core::ffta_rom::digest(&m.save_state()?));
        origin["rom_sha256"] = json!(gba_core::ffta_rom::digest(&m.bus.rom));
        origin["bios"] = json!({"real":m.bus.real_bios,"sha256":gba_core::ffta_rom::digest(&m.bus.bios)});
        origin["frame"] = json!(m.bus.frames); origin["clock"] = json!(m.bus.clock);
        origin["native"] = match table {
            Some(t) => {
                let source = t.source.as_ref().map(|p|crate::diagnostics::read(p,256*1024*1024).map(|b|json!({"path":p,"sha256_at_trace_start":gba_core::ffta_rom::digest(&b),"bytes":b.len()}))).transpose()?;
                json!({"enabled":true,"table_entries":t.len,"source":source,"source_scope":"Path retained at load; file hashed at trace start. Loaded memory image/function extent is not authenticated by this later file read"})
            },
            None => json!({"enabled":false}),
        };
        Ok(Self { settings, origin, labels, label_identity, started:Instant::now(), active:true, reason:"collecting", events:Vec::with_capacity(512), observed:0, filtered:0, frames:0 })
    }
    pub fn enabled(&self) -> bool { self.active }
    pub fn freeze(&mut self, reason: &'static str) { if self.active { self.active=false; self.reason=reason; } }
    pub fn tick(&mut self) { if self.started.elapsed().as_secs() >= 30 { self.freeze("30-second-limit"); } }
    pub fn status(&self) -> Value {
        json!({"collecting":self.active,"reason":self.reason,"events":self.events.len(),"observed_dispatches":self.observed,"filtered_dispatches":self.filtered,"completed_frames":self.frames,"settings":self.settings})
    }
    pub fn native(&mut self, m: &Machine, kind: crate::NativeDispatch) {
        let name = match kind { crate::NativeDispatch::Native=>"native",crate::NativeDispatch::Missing=>"missing",crate::NativeDispatch::System=>"system",crate::NativeDispatch::Hook=>"hook" };
        self.entry(m,name);
    }
    fn entry(&mut self, m: &Machine, kind: &'static str) {
        if !self.active { return; }
        self.tick(); if !self.active { return; }
        self.observed=self.observed.saturating_add(1);
        let pc=m.cpu.regs[15]; let thumb=m.cpu.thumb(); let key=pc|u32::from(thumb);
        if pc<self.settings.start || pc>=self.settings.end || self.settings.thumb.is_some_and(|t|t!=thumb) || (self.settings.kind!="all" && self.settings.kind!=kind) {
            self.filtered=self.filtered.saturating_add(1);return;
        }
        if self.events.len()==self.settings.events { self.freeze("event-limit");return; }
        let system_reason=system(m);
        let mut disassembly=None; let mut backing=json!(null);
        if system_reason.is_none() {
            match gba_core::debug_memory::peek(m,pc,if thumb{2}else{4}) {
                Ok(bytes)=>{
                    let raw=bytes.iter().map(|b|b.value).collect::<Vec<_>>();
                    let i=if thumb{armv4t::decode_thumb(u16::from_le_bytes(raw[..2].try_into().unwrap()),pc)}else{armv4t::decode_arm(u32::from_le_bytes(raw[..4].try_into().unwrap()),pc)};
                    disassembly=Some(i.disasm());
                    backing=json!({"region":bytes[0].region,"physical_offset":bytes[0].offset,"bytes":raw,"meaning":"Readonly current backing bytes at entry; not a bus fetch or compiled block body"});
                },
                Err(error)=>{backing=json!({"unavailable":error});},
            }
        }
        let mut matches=0usize;
        let mut names=Vec::new();
        for (label,name) in &self.labels.names {
            let start=label&!1;let end=self.labels.ends.get(label).copied().unwrap_or(start.saturating_add(1));
            if label&1==u32::from(thumb) && (start..end).contains(&pc) {
                matches+=1;
                if names.len()<4 { names.push(json!({"key":label,"name":prefix(name,128),"name_truncated":name.chars().count()>128,"declared_end":self.labels.ends.get(label)})); }
            }
        }
        let reason=match kind {"native"=>"compiled-table-entry-selected","missing"=>"no-block-for-exact-pc-and-mode","system"=>system_reason.unwrap_or("system-step"),"hook"=>"intentional-battle-mouse-or-sprite-interpreter-hook",_=>"interpreter-selected-no-native-lookup"};
        self.events.push(json!({"sequence":self.events.len(),"frame":m.bus.frames,"clock":m.bus.clock,"pc":pc,"thumb":thumb,"key":key,"cpsr":m.cpu.cpsr,
            "kind":kind,"reason":reason,"system_reason":system_reason,"backing":backing,"disassembly":disassembly,
            "compiled_entry":if kind=="native"{Some(key)}else{None},"compiled_lookup_performed":matches!(kind,"native"|"missing"),
            "labels":names,"label_matches_omitted":matches.saturating_sub(4),"declared_entry":self.labels.rom.contains(&key)||self.labels.iwram.contains(&key)}));
    }
    pub fn interpreter(&mut self, m:&Machine) {
        self.entry(m,if system(m).is_some(){"system"}else{"interpreter"});
    }
    pub fn run_interpreter(&mut self, m: &mut Machine, max_steps:u64) {
        crate::run_frame_interpreter_observed(m,max_steps,|m|self.interpreter(m));
    }
    pub fn completed_frame(&mut self) {
        self.frames+=1;if self.active && self.frames>=self.settings.frames {self.freeze("frame-limit");}
    }
    pub fn save(&mut self,root:&Path,current:Value)->Result<String,String> {
        capacity(root)?;self.freeze("report-request");
        let data=json!({"schema":1,"kind":"ffta-code-inspection","origin":self.origin,"current":current,"trace":self.status(),"labels":self.label_identity,"events":self.events,
            "scope":{"boundary":"Before native dispatch or interpreter step. A native event identifies the selected table key only; instructions inside a block are not traced. System steps can sleep/service IRQ without executing the displayed PC",
                "labels":"Declared static ROM/RAM hints; a matching label is correlation, not observed function extent or extracted-code percentage",
                "backing":"ROM backing and current RAM bytes remain distinct. RAM self modification/native-memory identity and host function addresses are unknown; no bus IO/fetch side effects",
                "limits":"Explicit bounded prefix; filtered, event/time/frame limits and stop reasons visible. Events after collection stops are unobserved, not zero",
                "acceptance":"No new ROM/DS/native parity/performance acceptance; no snapshot or guest writes"}});
        let bytes=serde_json::to_vec(&data).map_err(|e|e.to_string())?;
        if bytes.len()>2*1024*1024{return Err("Code metadata exceeds2MiB; frozen trace retained".into());}
        let name=format!("{}-{}",SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e|e.to_string())?.as_nanos(),std::process::id());
        let base=root.join("Data/config/code");fs::create_dir_all(&base).map_err(|e|e.to_string())?;
        let dir=base.join(&name);fs::create_dir(&dir).map_err(|e|e.to_string())?;
        let embedded=String::from_utf8(bytes.clone()).unwrap().replace('<',"\\u003c");
        fs::write(dir.join("index.html"),include_str!("debug_code/report.html").replacen("/*REPORT*/",&embedded,1)).map_err(|e|e.to_string())?;
        fs::write(dir.join("code.json"),bytes).map_err(|e|e.to_string())?;
        Ok(format!("Data/config/code/{name}/index.html"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn root()->tempfile::TempDir {
        let d=tempfile::tempdir().unwrap();fs::write(d.path().join("debug-session.json"),"{\"synthetic\":true}").unwrap();d
    }
    #[test]
    fn readonly_entry_filters_bounds_and_foreign_labels() {
        let r=root();let mut m=Machine::new(0xeafffffeu32.to_le_bytes().to_vec());
        let before=m.save_state().unwrap();let mut t=Trace::start(r.path(),&m,None,Settings{events:2,..Default::default()},json!({}),None).unwrap();
        t.entry(&m,"interpreter");t.entry(&m,"interpreter");t.entry(&m,"interpreter");
        assert_eq!(before,m.save_state().unwrap());assert_eq!(t.events.len(),2);assert_eq!(t.reason,"event-limit");assert!(t.events[0]["disassembly"].as_str().unwrap().contains('b'));
        m.bus.iwram[..4].copy_from_slice(&0xe1a00000u32.to_le_bytes());m.cpu.regs[15]=0x03008000;
        let mut t=Trace::start(r.path(),&m,None,Settings{start:0x03000000,end:0x04000000,..Default::default()},json!({}),None).unwrap();t.entry(&m,"interpreter");
        assert_eq!(t.events[0]["backing"]["region"],"iwram");assert_eq!(t.events[0]["backing"]["physical_offset"],0);
        t.settings.thumb=Some(true);t.entry(&m,"interpreter");assert_eq!(t.filtered,1);
        fs::write(r.path().join("debug-labels.toml"),format!("gba-labels v1\nrom-sha256 {}\nrom 08000000 a\n","0".repeat(64))).unwrap();
        assert!(Trace::start(r.path(),&m,None,Default::default(),json!({}),None).is_err());
    }
    #[test]
    fn mixed_native_transition_preserves_full_state_and_counts() {
        extern "C" fn translated(_: *const gba_core::capi::RtApi,m:*mut std::ffi::c_void)->u32 {gba_core::capi::rt_interp_one(m)}
        let r=root();let rom=[0xe3a00001u32,0xeafffffdu32].into_iter().flat_map(u32::to_le_bytes).collect::<Vec<_>>();
        let table=crate::BlockTable::from_blocks(&[gba_core::capi::RcgBlock{key:crate::ROM_BASE,func:translated}]).unwrap();
        let mut plain=Machine::new(rom.clone());let mut traced=Machine::new(rom);
        let p=&mut plain as *mut Machine as *mut std::ffi::c_void;let expected=crate::run_frame_native(&mut plain,&table,p,12);
        let mut trace=Trace::start(r.path(),&traced,Some(&table),Default::default(),json!({"synthetic_native":"Rust extern block invokes rt_interp_one; no generated-ROM parity claim"}),None).unwrap();
        let p=&mut traced as *mut Machine as *mut std::ffi::c_void;
        let actual=crate::run_frame_native_traced(&mut traced,&table,p,12,|m,k|trace.native(m,k),|_,_,_|{});
        assert_eq!(expected,actual);assert_eq!(actual,(6,6));assert_eq!(plain.save_state().unwrap(),traced.save_state().unwrap());
        assert_eq!(trace.events[0]["kind"],"native");assert_eq!(trace.events[1]["kind"],"missing");assert_eq!(trace.events[1]["reason"],"no-block-for-exact-pc-and-mode");
        let mut plain=Machine::new(0xeafffffeu32.to_le_bytes().to_vec());let mut traced=Machine::new(0xeafffffeu32.to_le_bytes().to_vec());
        let mut trace=Trace::start(r.path(),&traced,None,Default::default(),json!({}),None).unwrap();plain.run_frame(20);trace.run_interpreter(&mut traced,20);
        assert_eq!(plain.save_state().unwrap(),traced.save_state().unwrap());
    }

    #[test]
    #[ignore = "requires explicit local ROM/snapshot/existing-library/output; no private fixture is distributed"]
    fn retained_native_observer_preserves_one_frame() {
        let path=|name:&str|std::path::PathBuf::from(std::env::var_os(name).expect(name));
        let rom=fs::read(path("FFTA_CODE_ROM")).unwrap();
        let seed=fs::read(path("FFTA_CODE_STATE")).unwrap();
        let library=path("FFTA_CODE_LIBRARY");let out=path("FFTA_CODE_OUTPUT");
        assert!(!out.exists());fs::create_dir_all(&out).unwrap();
        fs::write(out.join("debug-session.json"),b"{\"validation_fixture\":true,\"scope\":\"No interactive session or cartridge writer\"}").unwrap();
        let mut plain=crate::make_machine(rom.clone(),None);crate::save_state::load_bytes(&mut plain,&seed).unwrap();
        let mut observed=crate::make_machine(rom,None);crate::save_state::load_bytes(&mut observed,&seed).unwrap();
        let (_code,table)=crate::load_native(&library).unwrap();
        let p=&mut plain as *mut Machine as *mut std::ffi::c_void;let expected=crate::run_frame_native(&mut plain,&table,p,5_000_000);
        let mut trace=Trace::start(&out,&observed,Some(&table),Settings::default(),json!({"snapshot_sha256":gba_core::ffta_rom::digest(&seed),"input":"Original snapshot keys, no host input or new recording", "validation":"Affected observer versus same existing native library, not interpreter/native parity","test_executable_sha256":gba_core::ffta_rom::digest(&fs::read(std::env::current_exe().unwrap()).unwrap())}),None).unwrap();
        let p=&mut observed as *mut Machine as *mut std::ffi::c_void;
        let actual=crate::run_frame_native_traced(&mut observed,&table,p,5_000_000,|m,k|trace.native(m,k),|_,_,_|{});
        assert_eq!(expected,actual);assert!(observed.bus.frame_ready);assert!(actual.0>0);
        assert_eq!(plain.save_state().unwrap(),observed.save_state().unwrap());trace.completed_frame();
        assert!(trace.events.iter().any(|e|e["kind"]=="native"));
        let current=json!({"frame":observed.bus.frames,"clock":observed.bus.clock,"state_sha256":gba_core::ffta_rom::digest(&observed.save_state().unwrap()),"native_blocks":actual.0,"fallback_steps":actual.1,"observed_unobserved_full_state_and_counts_equal":true});
        let report=trace.save(&out,current).unwrap();println!("CODE_REPORT {report}");
    }
}
