//! Portable, supervised functional replays. The normal game is never the worker.
use gba_core::Machine;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::{Child, Stdio},
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

const PROFILE: &str = "replay.json";
const MAX_INPUT: u64 = 8 * 1024 * 1024;
const MAX_FRAMES: u64 = 360_000;
static INTERRUPTED: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Profile {
    schema: u32,
    kind: String,
    pub case: String,
    runtime_sha256: String,
    recording_runtime_sha256: String,
    rom_sha256: String,
    bios_sha256: String,
    dataset_sha256: Option<String>,
    library_sha256: Option<String>,
    start_sha256: String,
    input_sha256: String,
    end_sha256: String,
    pub frames: u64,
    first_frame: u64,
    first_clock: u64,
    final_clock: u64,
    initial_keys: u16,
    extra: usize,
    extra_y: usize,
    pub timeout_ms: u64,
    stop: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit: Option<crate::debug_rom_audit::Settings>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capture: Option<crate::debug_capture::Settings>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comparison: Option<crate::debug_compare::Settings>,
}
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Report {
    schema: u32,
    kind: String,
    pub run: String,
    pub case: String,
    pub outcome: String,
    pub completed: u64,
    pub frames: u64,
    pub message: String,
    runtime_sha256: String,
    rom_sha256: String,
    dataset_sha256: Option<String>,
    input_sha256: String,
    library_sha256: Option<String>,
    state_sha256: Option<String>,
    clock: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_report: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_truncated: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_window_complete: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capture_catalog: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comparison_report: Option<String>,
    #[serde(default,skip_serializing_if="Vec::is_empty")]
    pub preparation:Vec<serde_json::Value>,
}
impl Report {
    fn new(p: &Profile, run: &str, outcome: &str, completed: u64, message: &str) -> Self {
        Self {
            schema: 1,
            kind: "ffta-debug-replay-result".into(),
            run: run.into(),
            case: p.case.clone(),
            outcome: outcome.into(),
            completed,
            frames: p.comparison.as_ref().map_or_else(||p.capture.as_ref().map_or(p.frames, |s| s.limit_frames),|s|s.frames),
            message: message.chars().take(2048).collect(),
            runtime_sha256: p.runtime_sha256.clone(),
            rom_sha256: p.rom_sha256.clone(),
            dataset_sha256: p.dataset_sha256.clone(),
            input_sha256: p.input_sha256.clone(),
            library_sha256: p.library_sha256.clone(),
            state_sha256: None,
            clock: None,
            audit_report: None,
            audit_truncated: None,
            audit_window_complete: None,
            capture_catalog: None,
            comparison_report: None,
            preparation:Vec::new(),
        }
    }
    pub fn code(&self) -> u8 {
        match self.outcome.as_str() {
            "passed" | "captured" | "matched" => 0,
            "not-reached" => 7,
            "diverged" => 2,
            "incompatible" => 3,
            "timeout" => 4,
            "interrupted" => 5,
            "running" => 0,
            _ => 6,
        }
    }
    pub fn short(&self) -> String {
        if self.comparison_report.is_some() {
            return if self.outcome=="diverged"{format!("DIFF FRAME {} | Report in profile/comparisons",self.completed)}
                else{format!("{} {}/{} | Report in profile/comparisons",self.outcome.to_ascii_uppercase(),self.completed,self.frames)};
        }
        if let Some(path) = &self.capture_catalog {
            let _ = path;
            return format!("CAPTURED {}/{} | Catalog in profile/captures", self.completed, self.frames);
        }
        if self.outcome == "not-reached" {
            return format!("NOT REACHED {}/{} | No snapshot", self.completed, self.frames);
        }
        if self.audit_report.is_some() {
            let status = if self.audit_truncated == Some(true) {
                "TRUNCATED"
            } else if self.audit_window_complete == Some(true) {
                "saved"
            } else {
                "PARTIAL"
            };
            return format!(
                "{}: {}/{} | ROM audit {status}",
                self.outcome, self.completed, self.frames
            );
        }
        format!(
            "{}: {}/{} frames. {}",
            self.outcome, self.completed, self.frames, self.message
        )
    }
}
fn digest(bytes: &[u8]) -> String {
    gba_core::ffta_rom::digest(bytes)
}
fn read(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    crate::diagnostics::read(path, limit)
}
fn json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let mut file = tempfile::NamedTempFile::new_in(path.parent().ok_or("Missing replay parent")?)
        .map_err(|e| e.to_string())?;
    file.write_all(&serde_json::to_vec(value).map_err(|e| e.to_string())?)
        .and_then(|_| file.as_file().sync_all())
        .map_err(|e| e.to_string())?;
    // Windows readers/AV may transiently deny replacement. Keep the old JSON
    // intact and retry only that sharing/permission error for at most 32 ms.
    for attempt in 0..=8 {
        match file.persist(path) {
            Ok(_) => return Ok(()),
            Err(error)
                if error.error.kind() == std::io::ErrorKind::PermissionDenied && attempt < 8 =>
            {
                file = error.file;
                std::thread::sleep(Duration::from_millis(4));
            }
            Err(error) => return Err(error.error.to_string()),
        }
    }
    unreachable!()
}
fn put(root: &Path, name: &str, bytes: &[u8]) -> Result<(), String> {
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join(name))
        .map_err(|e| e.to_string())?;
    f.write_all(bytes)
        .and_then(|_| f.sync_all())
        .map_err(|e| e.to_string())
}
fn outputs(root: &Path) -> PathBuf {
    root.join("Data/config/replay")
}
fn load(path: &Path) -> Result<(PathBuf, Profile), String> {
    let path = fs::canonicalize(path).map_err(|e| e.to_string())?;
    if path.file_name().and_then(|s| s.to_str()) != Some(PROFILE) {
        return Err("Select the profile's replay.json".into());
    }
    let p: Profile = serde_json::from_slice(&read(&path, 64 * 1024)?)
        .map_err(|e| format!("Replay profile: {e}"))?;
    if p.schema != 1
        || p.kind != "ffta-debug-replay-profile"
        || p.frames > MAX_FRAMES
        || !(1..=120000).contains(&p.timeout_ms)
        || p.stop != if p.comparison.is_some() {"first-divergence"} else if p.capture.is_some() {"capture-trigger"} else {"exact-endpoint"}
        || p.case.is_empty()
        || p.case.len() > 64
        || p.case.chars().any(char::is_control)
        || p.extra > gba_core::mem::MAX_EXTRA
        || p.extra_y > gba_core::mem::MAX_EXTRA_Y
    {
        return Err("Invalid replay profile bounds, case or stop condition".into());
    }
    if let Some(audit) = &p.audit {
        crate::debug_rom_audit::validate(audit, p.frames)?;
    }
    if let Some(capture) = &p.capture {
        crate::debug_capture::validate(capture, p.frames)?;
        if p.audit.is_some() || p.library_sha256.is_some() {
            return Err("Capture profiles require interpreter-only replay without a simultaneous audit".into());
        }
    }
    if let Some(comparison)=&p.comparison {
        crate::debug_compare::validate(comparison,p.frames,p.library_sha256.is_some())?;
        if p.audit.is_some() || p.capture.is_some(){return Err("Comparison profiles cannot also audit or capture triggers".into());}
    }
    Ok((path.parent().unwrap().into(), p))
}
fn checked(root: &Path, name: &str, limit: u64, expected: &str) -> Result<Vec<u8>, String> {
    let bytes = read(&root.join(name), limit)?;
    if digest(&bytes) != expected {
        return Err(format!("Replay input changed: {name}"));
    }
    Ok(bytes)
}

/// Readonly report preflight. Does not acquire a lease, execute or load native code.
pub(crate) fn issue_metadata(path: &Path) -> Result<serde_json::Value, String> {
    let (root,p)=load(path)?;
    let _unstarted=crate::debug_session::replay_command(&root)?;
    checked(&root,executable(),256*1024*1024,&p.runtime_sha256)?;
    checked(&root,"ffta.gba",32*1024*1024,&p.rom_sha256)?;
    checked(&root,"seed.ffstate",gba_core::save_state::MAX_STATE_BYTES as u64,&p.start_sha256)?;
    checked(&root,"expected.ffstate",gba_core::save_state::MAX_STATE_BYTES as u64,&p.end_sha256)?;
    let bytes=checked(&root,"input.input",MAX_INPUT,&p.input_sha256)?;
    let _script=crate::debug_recording::script(&bytes,p.frames)?;
    if let Some(hash)=&p.library_sha256 {checked(&root,"native.bin",256*1024*1024,hash)?;}
    if let Some(hash)=&p.dataset_sha256 {checked(&root,"Data/dataset/manifest.toml",gba_core::rom_data::MAX_MANIFEST_FILE_BYTES,hash)?;}
    let bios=if root.join("gba_bios.bin").exists(){read(&root.join("gba_bios.bin"),16384)?}else{Machine::new(vec![0;256]).bus.bios};
    if digest(&bios)!=p.bios_sha256{return Err("Replay BIOS mismatch".into());}
    let manifest=read(&root.join("debug-session.json"),1024*1024)?;
    Ok(serde_json::json!({"profile":p,"session_manifest":serde_json::from_slice::<serde_json::Value>(&manifest).map_err(|e|e.to_string())?,
        "session_manifest_sha256":digest(&manifest),"scope":"Profile bounds, session pins, payload hashes and input parser checked without guest execution. Worker still validates decoded snapshots, runtime compatibility and actual endpoint"}))
}
fn executable() -> &'static str {
    if cfg!(windows) {
        "recomp.exe"
    } else {
        "recomp"
    }
}

#[cfg(test)]
pub(crate) fn create(
    recording: &Path,
    session: &Path,
    destination: &Path,
    case: &str,
    timeout_ms: u64,
    runtime: &Path,
    library: Option<&Path>,
) -> Result<PathBuf, String> {
    create_inner(
        recording,
        session,
        destination,
        case,
        timeout_ms,
        runtime,
        library,
        None,
        None,
        None,
        false,
    )
}

fn create_inner(
    recording: &Path,
    session: &Path,
    destination: &Path,
    case: &str,
    timeout_ms: u64,
    runtime: &Path,
    library: Option<&Path>,
    audit: Option<&crate::debug_rom_audit::Request>,
    capture: Option<&crate::debug_capture::Request>,
    comparison: Option<&crate::debug_compare::Request>,
    validate_only: bool,
) -> Result<PathBuf, String> {
    if case.is_empty()
        || case.len() > 64
        || case.chars().any(char::is_control)
        || !(1..=120000).contains(&timeout_ms)
    {
        return Err("Case must be 1..64 printable characters; timeout must be 1..120000 ms".into());
    }
    if capture.is_some() && (library.is_some() || audit.is_some()) {
        return Err("Capture profiles require interpreter-only replay without a simultaneous audit".into());
    }
    if comparison.is_some() && (audit.is_some() || capture.is_some()) {return Err("Comparison profiles cannot also audit or capture triggers".into());}
    if destination.exists(){return Err("Profile destination already exists; prepare a new profile instead of overwriting.".into());}
    let (session_rom,session_bios,session_dataset,_source_lease)=crate::debug_session::profile_source(session)?;
    let data = crate::debug_recording::export(recording)?;
    let m = &data.manifest;
    let rom = read(&session_rom, 32 * 1024 * 1024)?;
    let bios = session_bios.map(|p| read(&p, 16384)).transpose()?;
    let dataset = session_dataset;
    let dataset_sha256 = dataset
        .as_ref()
        .map(|p| {
            read(
                &p.join("manifest.toml"),
                gba_core::rom_data::MAX_MANIFEST_FILE_BYTES,
            )
            .map(|b| digest(&b))
        })
        .transpose()?;
    let mut machine = crate::make_machine(rom.clone(), bios.as_deref());
    if digest(&rom) != m.rom_sha256
        || digest(&machine.bus.bios) != m.bios_sha256
        || dataset_sha256 != m.dataset_manifest_sha256
    {
        return Err("Recording and session ROM/BIOS/dataset differ".into());
    }
    let dataset = comparison.and_then(|r| r.dataset.clone()).or_else(||capture.and_then(|r| r.dataset.clone())).or_else(||audit.and_then(|r| r.dataset.clone())).or(dataset);
    let dataset_sha256 = dataset
        .as_ref()
        .map(|p| {
            read(
                &p.join("manifest.toml"),
                gba_core::rom_data::MAX_MANIFEST_FILE_BYTES,
            )
            .map(|b| digest(&b))
        })
        .transpose()?;
    machine.load_state(&data.end)?;
    if machine.bus.frames != m.last_frame || machine.bus.clock != m.last_clock {
        return Err("Recording endpoint metadata mismatch".into());
    }
    machine.load_state(&data.start)?;
    if machine.bus.frames != m.first_frame
        || machine.bus.clock != m.first_clock
        || machine.bus.keys != m.initial_keys
        || m.first_frame.checked_add(m.frames) != Some(m.last_frame)
    {
        return Err("Recording origin/frame range mismatch".into());
    }
    let native = library.map(|p| read(p, 256 * 1024 * 1024)).transpose()?;
    let audit_prepared = audit
        .map(|request| crate::debug_rom_audit::prepare(request, &rom, m.frames, dataset.as_deref()))
        .transpose()?;
    let capture_prepared = capture
        .map(|request| crate::debug_capture::prepare(request, &rom, m.frames, dataset.as_deref()))
        .transpose()?;
    let comparison_prepared=comparison.map(|request|crate::debug_compare::prepare(request,m.frames,native.is_some())).transpose()?;
    // Authenticate remaining source inputs before either validation returns or output exists.
    read(runtime,256*1024*1024)?;
    if let Some(dataset)=dataset.as_deref(){crate::rom_data::diagnosis::inspect(&rom,dataset)?;}
    if validate_only{return Ok(destination.join(PROFILE));}
    let root = crate::debug_session::create(
        destination,
        crate::debug_session::Source {
            rom: &rom,
            bios: bios.as_deref(),
            save: machine.bus.save_data(),
            state: Some(&data.start),
            av: input_config::AvConfig::parse(
                &String::from_utf8(read(&recording.join("av.cfg"), 1024 * 1024)?)
                    .map_err(|e| e.to_string())?,
            ),
            input: input_config::InputConfig::load(),
            interp: native.is_none(),
            dataset: dataset.as_deref(),
            executable: runtime,
        },
    )?;
    put(&root, "input.input", &data.input)?;
    put(&root, "expected.ffstate", &data.end)?;
    if let Some(bytes) = &native {
        put(&root, "native.bin", bytes)?;
    }
    if let Some((_, Some(bytes))) = &audit_prepared {
        put(&root, "known-plan.toml", bytes)?;
    }
    if let Some((_, Some(bytes))) = &capture_prepared {
        put(&root, "capture-plan.toml", bytes)?;
    }
    let p = Profile {
        schema: 1,
        kind: "ffta-debug-replay-profile".into(),
        case: case.into(),
        runtime_sha256: digest(&read(&root.join(executable()), 256 * 1024 * 1024)?),
        recording_runtime_sha256: m.executable_sha256.clone(),
        rom_sha256: m.rom_sha256.clone(),
        bios_sha256: m.bios_sha256.clone(),
        dataset_sha256,
        library_sha256: native.as_deref().map(digest),
        start_sha256: digest(&data.start),
        input_sha256: digest(&data.input),
        end_sha256: digest(&data.end),
        frames: m.frames,
        first_frame: m.first_frame,
        first_clock: m.first_clock,
        final_clock: m.last_clock,
        initial_keys: m.initial_keys,
        extra: m.extra,
        extra_y: m.extra_y,
        timeout_ms,
        stop: if comparison_prepared.is_some() {"first-divergence"} else if capture_prepared.is_some() {"capture-trigger"} else {"exact-endpoint"}.into(),
        audit: audit_prepared.map(|(settings, _)| settings),
        capture: capture_prepared.map(|(settings, _)| settings),
        comparison: comparison_prepared,
    };
    // Published last: partial copies never look runnable.
    put(
        &root,
        PROFILE,
        &serde_json::to_vec_pretty(&p).map_err(|e| e.to_string())?,
    )?;
    Ok(root.join(PROFILE))
}

fn install_interrupt() {
    INTERRUPTED.store(false, Ordering::Relaxed);
    #[cfg(unix)]
    unsafe {
        extern "C" fn handler(_: i32) {
            INTERRUPTED.store(true, Ordering::Relaxed);
        }
        extern "C" {
            fn signal(signal: i32, handler: usize) -> usize;
        }
        signal(2, handler as *const () as usize);
        signal(15, handler as *const () as usize);
    }
    #[cfg(windows)]
    unsafe {
        extern "system" fn handler(kind: u32) -> i32 {
            if kind <= 2 {
                INTERRUPTED.store(true, Ordering::Relaxed);
                1
            } else {
                0
            }
        }
        #[link(name = "kernel32")]
        extern "system" {
            fn SetConsoleCtrlHandler(
                handler: Option<extern "system" fn(u32) -> i32>,
                add: i32,
            ) -> i32;
        }
        SetConsoleCtrlHandler(Some(handler), 1);
    }
}

fn execute(
    p: &Profile,
    run: &str,
    machine: &mut Machine,
    input: &crate::InputScript,
    end: &[u8],
    table: Option<&crate::BlockTable>,
    mut audit: Option<&mut crate::debug_rom_audit::Audit>,
    mut capture: Option<&mut crate::debug_capture::Observer>,
    mut control: impl FnMut(u64) -> Option<(&'static str, String)>,
) -> Report {
    let mut completed = 0;
    let outcome = loop {
        if let Some((kind, message)) = control(completed) {
            break Report::new(p, run, kind, completed, &message);
        }
        if let Some(capture) = capture.as_deref_mut() {
            let reached = capture.reached(completed, machine.bus.frames);
            if reached || completed >= capture.settings.limit_frames {
                let mut r = Report::new(p, run, if reached {"captured"} else {"not-reached"}, completed,
                    if reached {"Trigger observed; stopped at the next completed-frame boundary. Derived variant only."}
                    else {"Trigger not observed within the recorded frame limit; no snapshot or scene promotion."});
                let state = match machine.save_state() {
                    Ok(state)=>state, Err(e)=>break Report::new(p,run,"error",completed,&e),
                };
                r.state_sha256=Some(digest(&state));r.clock=Some(machine.bus.clock);
                break r;
            }
        }
        if completed == p.frames {
            let actual = match machine.save_state() {
                Ok(s) => s,
                Err(e) => break Report::new(p, run, "error", completed, &e),
            };
            let ok = actual == end
                && machine.bus.clock == p.final_clock
                && machine.bus.frames == p.first_frame + p.frames;
            let mut r = Report::new(
                p,
                run,
                if ok { "passed" } else { "diverged" },
                completed,
                if ok {
                    "Exact endpoint state and clock match."
                } else {
                    "Endpoint state or clock differs."
                },
            );
            r.state_sha256 = Some(digest(&actual));
            r.clock = Some(machine.bus.clock);
            break r;
        }
        if let Some(audit) = audit.as_deref_mut() {
            audit.frame(completed, machine.bus.frames);
        }
        if let Some(capture) = capture.as_deref_mut() {capture.frame(machine.bus.frames);}
        machine.bus.keys = input.keys_at(completed);
        if let Err(e) = input.apply_mouse(&mut machine.bus, completed) {
            break Report::new(p, run, "incompatible", completed, &e);
        }
        let before = machine.bus.frames;
        if let Some(table) = table {
            let ptr = machine as *mut Machine as *mut std::ffi::c_void;
            crate::run_frame_native(machine, table, ptr, 5_000_000);
        } else {
            machine.run_frame(5_000_000);
        }
        if machine.bus.frames != before + 1 {
            break Report::new(p, run, "error", completed, "Frame step guard exceeded.");
        }
        machine.bus.audio_buf.clear();
        machine.bus.psg_tap.clear();
        machine.bus.hle_tap.clear();
        machine.bus.fifo_tap[0].clear();
        machine.bus.fifo_tap[1].clear();
        completed += 1;
    };
    outcome
}

fn preparation(p:&Profile,root:&Path,run:&str,events:&mut Vec<serde_json::Value>,started:Instant,family:&str,outcome:&str,details:serde_json::Value){
    if events.len()==8{return;}
    let bytes=serde_json::to_vec(&details).expect("Preparation JSON");
    let details=if bytes.len()>512{serde_json::json!({"truncated":true,"bytes":bytes.len(),"sha256":digest(&bytes),"prefix":String::from_utf8(bytes).expect("JSON UTF8").chars().take(64).collect::<String>()})}else{details};
    events.push(serde_json::json!({"sequence":events.len(),"worker_host_elapsed_ns":started.elapsed().as_nanos().min(u128::from(u64::MAX))as u64,"family":family,"outcome":outcome,"details":details,"scope":"Worker preparation boundary, not guest instruction chronology; loaded inputs are not newly generated datasets/cache"}));
    let mut report=Report::new(p,run,"preparing",0,"Preparing selected worker inputs.");report.preparation=events.clone();
    // Observation failure never prevents the existing validated operation.
    if let Err(error)=json(&outputs(root).join("progress.json"),&report){events.last_mut().unwrap()["publication_error"]=serde_json::json!(error.chars().take(256).collect::<String>());}
}
fn worker(path: &Path, run: &str) -> Result<Report, String> {
    if run.is_empty()
        || run.len() > 64
        || !run.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err("Invalid replay run identifier".into());
    }
    let (root, p) = load(path)?;
    let preparation_started=Instant::now();let mut preparation_events=Vec::with_capacity(4);
    let _session = crate::debug_session::open(&root)?;
    if digest(&read(
        &std::env::current_exe().map_err(|e| e.to_string())?,
        256 * 1024 * 1024,
    )?) != p.runtime_sha256
    {
        return Err("Selected replay executable differs".into());
    }
    let rom = checked(&root, "ffta.gba", 32 * 1024 * 1024, &p.rom_sha256)?;
    let bios = _session.bios().map(|f| read(&f, 16384)).transpose()?;
    let mut machine = crate::make_machine(rom, bios.as_deref());
    if digest(&machine.bus.bios) != p.bios_sha256 {
        return Err("Replay BIOS mismatch".into());
    }
    let dataset = _session.dataset();
    preparation(&p,&root,run,&mut preparation_events,preparation_started,"dataset","requested",serde_json::json!({"manifest_sha256":p.dataset_sha256}));
    if dataset
        .as_ref()
        .map(|d| {
            read(
                &d.join("manifest.toml"),
                gba_core::rom_data::MAX_MANIFEST_FILE_BYTES,
            )
            .map(|b| digest(&b))
        })
        .transpose()?
        != p.dataset_sha256
    {
        return Err("Replay dataset mismatch".into());
    }
    if let Some(d) = dataset {
        let result=crate::rom_data::attach_from_directory(&mut machine, &d);
        preparation(&p,&root,run,&mut preparation_events,preparation_started,"dataset",if result.is_ok(){"attached"}else{"refused"},serde_json::json!({"manifest_sha256":p.dataset_sha256,"error":result.as_ref().err().map(|e|e.chars().take(512).collect::<String>())}));
        result?;
    }else{preparation(&p,&root,run,&mut preparation_events,preparation_started,"dataset","not-selected",serde_json::json!({}));}
    let start = checked(
        &root,
        "seed.ffstate",
        gba_core::save_state::MAX_STATE_BYTES as u64,
        &p.start_sha256,
    )?;
    let end = checked(
        &root,
        "expected.ffstate",
        gba_core::save_state::MAX_STATE_BYTES as u64,
        &p.end_sha256,
    )?;
    let bytes = checked(&root, "input.input", MAX_INPUT, &p.input_sha256)?;
    let input = crate::debug_recording::script(&bytes, p.frames)?;
    machine.load_state(&end)?;
    if p.first_frame.checked_add(p.frames) != Some(machine.bus.frames)
        || machine.bus.clock != p.final_clock
    {
        return Err("Replay endpoint metadata mismatch".into());
    }
    machine.load_state(&start)?;
    if machine.bus.frames != p.first_frame
        || machine.bus.clock != p.first_clock
        || machine.bus.keys != p.initial_keys
        || p.first_frame.checked_add(p.frames).is_none()
    {
        return Err("Replay origin mismatch".into());
    }
    preparation(&p,&root,run,&mut preparation_events,preparation_started,"cache","requested",serde_json::json!({"library_sha256":p.library_sha256}));
    let native = if let Some(hash) = &p.library_sha256 {
        checked(&root, "native.bin", 256 * 1024 * 1024, hash)?;
        let result=crate::load_native(&root.join("native.bin"));
        preparation(&p,&root,run,&mut preparation_events,preparation_started,"cache",if result.is_ok(){"loaded"}else{"refused"},serde_json::json!({"library_sha256":hash,"error":result.as_ref().err().map(|e|e.chars().take(512).collect::<String>())}));
        Some(result?)
    } else {
        preparation(&p,&root,run,&mut preparation_events,preparation_started,"cache","not-selected",serde_json::json!({}));
        None
    };
    struct View(usize, usize);
    impl Drop for View {
        fn drop(&mut self) {
            gba_core::mem::set_extra(self.0);
            gba_core::mem::set_extra_y(self.1);
        }
    }
    let _view = View(gba_core::mem::extra(), gba_core::mem::extra_y());
    gba_core::mem::set_extra(p.extra);
    gba_core::mem::set_extra_y(p.extra_y);
    let out = outputs(&root);
    let started = Instant::now();
    let mut last = Instant::now() - Duration::from_secs(1);
    install_interrupt();
    let mut audit = p
        .audit
        .as_ref()
        .map(|s| crate::debug_rom_audit::Audit::start(&root, s, &machine.bus.rom))
        .transpose()?;
    let mut capture = p.capture.as_ref().map(|s|crate::debug_capture::Observer::start(&root,s,&machine.bus.rom)).transpose()?;
    let control = |completed| {
            if INTERRUPTED.load(Ordering::Relaxed)
                || read(&out.join("cancel.json"), 1024)
                    .ok()
                    .is_some_and(|b| b == run.as_bytes())
            {
                return Some((
                    "interrupted",
                    "Replay cancelled; original progress is untouched.".into(),
                ));
            }
            if started.elapsed() >= Duration::from_millis(p.timeout_ms) {
                return Some(("timeout", "Replay wall-clock timeout.".into()));
            }
            if last.elapsed() >= Duration::from_millis(100) {
                let mut state = Report::new(&p, run, "running", completed, "Replaying isolated case.");
                state.preparation=preparation_events.clone();
                if let Err(e) = json(&out.join("progress.json"), &state) {
                    return Some(("error", e));
                }
                last = Instant::now();
            }
            None
        };
    let mut report=if let Some(settings)=&p.comparison {
        let origin=serde_json::json!({"case":p.case,"profile_settings_sha256":digest(&serde_json::to_vec(&p).map_err(|e|e.to_string())?),
            "runtime_sha256":p.runtime_sha256,"recording_runtime_sha256":p.recording_runtime_sha256,
            "rom_sha256":p.rom_sha256,"bios_sha256":p.bios_sha256,"dataset_manifest_sha256":p.dataset_sha256,
            "reference_dataset":null,"library_sha256":p.library_sha256,"start_sha256":p.start_sha256,"input_sha256":p.input_sha256,
            "first_frame":p.first_frame,"first_clock":p.first_clock,"initial_keys":p.initial_keys,"extra":p.extra,"extra_y":p.extra_y,
            "recorded_frames":p.frames,"recorded_endpoint":"not verified by this comparison"});
        let result=crate::debug_compare::run(&root,run,settings,origin,&mut machine,&start,&input,native.as_ref().map(|n|&n.1),control)?;
        let mut report=Report::new(&p,run,&result.outcome,result.completed,&result.message);
        report.comparison_report=Some(result.report);report.state_sha256=Some(result.state_sha256);report.clock=Some(result.clock);report
    }else{execute(&p,run,&mut machine,&input,&end,native.as_ref().map(|n|&n.1),audit.as_mut(),capture.as_mut(),control)};
    if let Some(audit) = audit {
        let mut result = audit.finish(
            report.completed,
            &report.outcome,
            &p.case,
            run,
            &p.rom_sha256,
        );
        result["runtime_sha256"] = p.runtime_sha256.clone().into();
        result["dataset_manifest_sha256"] = p.dataset_sha256.clone().into();
        result["input_sha256"] = p.input_sha256.clone().into();
        json(&out.join("rom-audit.json"), &result)?;
        report.audit_report = Some("Data/config/replay/rom-audit.json".into());
        report.audit_truncated = result["truncated"].as_bool();
        report.audit_window_complete = result["window"]["complete"].as_bool();
        report
            .message
            .push_str(if report.audit_truncated == Some(true) {
                " ROM audit truncated; inspect dropped requests."
            } else if report.audit_window_complete == Some(true) {
                " ROM audit saved."
            } else {
                " ROM audit window incomplete."
            });
    }
    if report.outcome == "captured" {
        let context=serde_json::json!({"profile_settings_sha256":digest(&serde_json::to_vec(&p).map_err(|e|e.to_string())?),
            "runtime_sha256":p.runtime_sha256,"recording_runtime_sha256":p.recording_runtime_sha256,
            "rom_sha256":p.rom_sha256,"bios_sha256":p.bios_sha256,"dataset_manifest_sha256":p.dataset_sha256,
            "start_sha256":p.start_sha256,"input_sha256":p.input_sha256,"first_frame":p.first_frame,
            "first_clock":p.first_clock,"initial_keys":p.initial_keys,"relative_completed":report.completed,
            "input_end_exclusive":report.completed,"extra":p.extra,"extra_y":p.extra_y});
        report.capture_catalog=Some(crate::debug_capture::publish(&root,run,&p.case,&machine,
            capture.as_ref().ok_or("Missing capture observer")?,context,crate::debug_capture::Inputs{state:&start,input:&bytes})?);
    }
    report.preparation=preparation_events;
    json(&out.join("result.json"), &report)?;
    Ok(report)
}

pub(crate) struct Job {
    pub profile: PathBuf,
    root: PathBuf,
    p: Profile,
    pub run: String,
    child: Child,
    _lease: File,
    started: Instant,
    cancelled: Option<Instant>,
    last: Option<Report>,
    done: bool,
}
impl Job {
    pub fn start(path: &Path) -> Result<Self, String> {
        let (root, p) = load(path)?;
        let mut command = crate::debug_session::replay_command(&root)?; // Validate paths before the lease file.
        let lease = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(root.join("replay.lock"))
            .map_err(|e| e.to_string())?;
        lease
            .try_lock()
            .map_err(|_| "This replay profile is already running")?;
        let session = crate::debug_session::open(&root)?; // Acquire the supervisor lease first; duplicates cannot race the worker's session lease.
        checked(&root, executable(), 256 * 1024 * 1024, &p.runtime_sha256)?;
        if p.capture.is_some() {crate::debug_capture::capacity(&root)?;}
        if p.comparison.is_some() {crate::debug_compare::capacity(&root)?;}
        let out = outputs(&root);
        fs::create_dir_all(&out).map_err(|e| e.to_string())?;
        let run = format!(
            "{}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_nanos(),
            std::process::id()
        );
        let initial = Report::new(&p, &run, "running", 0, "Starting isolated replay.");
        json(&out.join("progress.json"), &initial)?;
        let log = File::create(out.join("worker.log")).map_err(|e| e.to_string())?;
        command
            .arg("debug-replay")
            .arg("worker")
            .arg(root.join(PROFILE))
            .arg(&run)
            .stdout(Stdio::from(log.try_clone().map_err(|e| e.to_string())?))
            .stderr(Stdio::from(log));
        drop(session);
        let child = command.spawn().map_err(|e| e.to_string())?;
        Ok(Self {
            profile: root.join(PROFILE),
            root,
            p,
            run,
            child,
            _lease: lease,
            started: Instant::now(),
            cancelled: None,
            last: Some(initial),
            done: false,
        })
    }
    pub fn require_comparison(path:&Path)->Result<(),String>{
        if load(path)?.1.comparison.is_none(){return Err("Choose a comparison profile created with debug-compare create".into());}Ok(())
    }
    pub fn start_comparison(path:&Path)->Result<Self,String>{Self::require_comparison(path)?;Self::start(path)}
    pub fn require_capture(path: &Path) -> Result<(), String> {
        if load(path)?.1.capture.is_none() {return Err("Choose a capture profile created with debug-capture create".into());}
        Ok(())
    }
    pub fn start_capture(path: &Path) -> Result<Self, String> {
        Self::require_capture(path)?;Self::start(path)
    }
    pub fn require_audit(path: &Path) -> Result<(), String> {
        let (_, p) = load(path)?;
        if p.audit.is_none() {
            return Err(
                "Choose an audited replay profile created with debug-rom-audit create".into(),
            );
        }
        Ok(())
    }
    pub fn start_audit(path: &Path) -> Result<Self, String> {
        Self::require_audit(path)?;
        Self::start(path)
    }
    pub fn progress(&self) -> Option<&Report> {
        self.last.as_ref()
    }
    pub fn cancel(&mut self) -> Result<(), String> {
        if self.cancelled.is_none() {
            fs::write(outputs(&self.root).join("cancel.json"), self.run.as_bytes())
                .map_err(|e| e.to_string())?;
            self.cancelled = Some(Instant::now());
        }
        Ok(())
    }
    pub fn poll(&mut self) -> Result<Option<Report>, String> {
        if self.done {
            return Ok(self.last.clone());
        }
        if let Ok(bytes) = read(&outputs(&self.root).join("progress.json"), 32 * 1024) {
            if let Ok(r) = serde_json::from_slice::<Report>(&bytes) {
                if r.run == self.run {
                    self.last = Some(r);
                }
            }
        }
        let expired = self.started.elapsed() > Duration::from_millis(self.p.timeout_ms);
        let mut forced = None;
        if expired
            || self
                .cancelled
                .is_some_and(|t| t.elapsed() > Duration::from_secs(2))
        {
            self.child.kill().ok();
            forced = Some(if self.cancelled.is_some() {
                "interrupted"
            } else {
                "timeout"
            });
        }
        let Some(status) = self.child.try_wait().map_err(|e| e.to_string())? else {
            return Ok(None);
        };
        let saved = read(&outputs(&self.root).join("result.json"), 32 * 1024)
            .ok()
            .and_then(|b| serde_json::from_slice::<Report>(&b).ok())
            .filter(|r| r.run == self.run && r.code() as i32 == status.code().unwrap_or(-1));
        let detail = read(&outputs(&self.root).join("worker.log"), 64 * 1024)
            .ok()
            .and_then(|b| String::from_utf8(b).ok())
            .and_then(|log| {
                log.lines().rev().find_map(|line| {
                    serde_json::from_str::<serde_json::Value>(line)
                        .ok()
                        .and_then(|v| v["message"].as_str().map(String::from))
                })
            });
        let mut result=saved.unwrap_or_else(||Report::new(&self.p,&self.run,forced.unwrap_or(if status.code()==Some(3){"incompatible"}else{"error"}),self.last.as_ref().map_or(0,|r|r.completed),if forced.is_some(){"Replay supervisor stopped the worker."}else{detail.as_deref().unwrap_or("Worker failed before its result. Inspect Data/config/replay/worker.log; the selected runtime must support debug-replay.")}));
        if result.preparation.is_empty(){if let Some(previous)=&self.last{result.preparation=previous.preparation.clone();}}
        json(&outputs(&self.root).join("result.json"), &result)?;
        self.last = Some(result.clone());
        self.done = true;
        Ok(Some(result))
    }
}
impl Drop for Job {
    fn drop(&mut self) {
        if !self.done {
            self.child.kill().ok();
            self.child.wait().ok();
            let mut r = Report::new(
                &self.p,
                &self.run,
                "interrupted",
                self.last.as_ref().map_or(0, |r| r.completed),
                "Replay owner closed.",
            );
            if let Some(previous)=&self.last{r.preparation=previous.preparation.clone();}
            let _ = json(&outputs(&self.root).join("result.json"), &r);
        }
    }
}

fn error(message: &str) -> u8 {
    eprintln!("{message}");
    println!(
        "{}",
        serde_json::json!({"schema":1,"kind":"ffta-debug-replay-result","outcome":"incompatible","message":message})
    );
    3
}
pub(crate) fn cmd(args: &[String]) -> u8 {
    match args.first().map(String::as_str) {
        Some("worker") if args.len()==3=>match worker(Path::new(&args[1]),&args[2]) {Ok(r)=>r.code(),Err(e)=>error(&e)},
        Some("run") if args.len()==2||args.len()==3&&args[2]=="--json"=>{
            install_interrupt();let mut job=match Job::start(Path::new(&args[1])){Ok(j)=>j,Err(e)=>return error(&e)};
            let mut previous=None;
            loop {
                if INTERRUPTED.load(Ordering::Relaxed){if let Err(e)=job.cancel(){return error(&e);}}
                match job.poll(){Ok(Some(r))=>{println!("{}",serde_json::to_string(&r).unwrap());return r.code();},Ok(None)=>{},Err(e)=>return error(&e)}
                if let Some(r)=job.progress(){let marker=(r.completed,r.message.clone());if previous.as_ref()!=Some(&marker){println!("{}",serde_json::to_string(r).unwrap());previous=Some(marker);}}
                std::thread::sleep(Duration::from_millis(30));
            }
        },
        Some("cancel") if args.len()==2=>{let result=(||->Result<(),String>{let(root,_)=load(Path::new(&args[1]))?;
            let _command=crate::debug_session::replay_command(&root)?;
            let lease=OpenOptions::new().read(true).write(true).open(root.join("replay.lock")).map_err(|_|"No active replay for this profile")?;
            if lease.try_lock().is_ok(){return Err("No active replay for this profile".into());}
            let report:Report=serde_json::from_slice(&read(&outputs(&root).join("progress.json"),32*1024)?).map_err(|e|e.to_string())?;fs::write(outputs(&root).join("cancel.json"),report.run.as_bytes()).map_err(|e|e.to_string())?;Ok(())})();match result{Ok(())=>{println!("{}",serde_json::json!({"schema":1,"kind":"ffta-debug-replay-cancel","requested":true}));0},Err(e)=>error(&e)}},
        Some("create")=>match create_cmd(&args[1..]){Ok(path)=>{println!("{}",serde_json::json!({"schema":1,"kind":"ffta-debug-replay-created","profile":path}));0},Err(e)=>error(&e)},
        _=>error("Use debug-replay create --recording DIR --session DIR --dir DIR --case NAME [--runtime FILE] [--library FILE] [--timeout-ms N], run PROFILE, or cancel PROFILE"),
    }
}
fn create_cmd(args: &[String]) -> Result<PathBuf, String> {
    create_cmd_audited(args, None)
}
pub(crate) fn create_cmd_audited(
    args: &[String],
    audit: Option<&crate::debug_rom_audit::Request>,
) -> Result<PathBuf, String> {
    create_cmd_inner(args, audit, None, None,false)
}
#[cfg(test)]
pub(crate) fn create_cmd_captured(args: &[String], capture: &crate::debug_capture::Request) -> Result<PathBuf,String> {
    create_cmd_inner(args,None,Some(capture),None,false)
}
#[cfg(test)]
pub(crate) fn create_cmd_comparison(args:&[String],comparison:&crate::debug_compare::Request)->Result<PathBuf,String>{create_cmd_inner(args,None,None,Some(comparison),false)}
pub(crate) fn create_dispatch(args: &[String], audit: Option<&crate::debug_rom_audit::Request>,capture:Option<&crate::debug_capture::Request>,comparison:Option<&crate::debug_compare::Request>,validate_only:bool)->Result<PathBuf,String>{create_cmd_inner(args,audit,capture,comparison,validate_only)}
fn create_cmd_inner(args: &[String], audit: Option<&crate::debug_rom_audit::Request>, capture: Option<&crate::debug_capture::Request>, comparison: Option<&crate::debug_compare::Request>,validate_only:bool) -> Result<PathBuf,String> {
    let mut recording = None;
    let mut session = None;
    let mut directory = None;
    let mut case = None;
    let mut runtime = None;
    let mut library = None;
    let mut timeout = 120000;
    let mut it = args.iter();
    while let Some(k) = it.next() {
        let v = it.next().ok_or_else(|| format!("{k} needs a value"))?;
        match k.as_str() {
            "--recording" => recording = Some(PathBuf::from(v)),
            "--session" => session = Some(PathBuf::from(v)),
            "--dir" => directory = Some(PathBuf::from(v)),
            "--case" => case = Some(v.as_str()),
            "--runtime" => runtime = Some(PathBuf::from(v)),
            "--library" => library = Some(PathBuf::from(v)),
            "--timeout-ms" => timeout = v.parse().map_err(|_| "Invalid timeout")?,
            _ => return Err(format!("Unknown replay argument {k}")),
        }
    }
    create_inner(
        &recording.ok_or("Missing --recording")?,
        &session.ok_or("Missing --session")?,
        &directory.ok_or("Missing --dir")?,
        case.ok_or("Missing --case")?,
        timeout,
        &runtime.unwrap_or(std::env::current_exe().map_err(|e| e.to_string())?),
        library.as_deref(),
        audit,
        capture,
        comparison,
        validate_only,
    )
}

#[cfg(test)]
mod tests;
