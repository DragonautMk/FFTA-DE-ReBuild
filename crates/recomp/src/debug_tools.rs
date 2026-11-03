//! Declared host adapters and readonly recipes. Nothing is launched by selection.
use crate::debug_workbench::Draft;
use menu::{ContentSlot as Slot, DebugTool as Tool};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
const LOG_LIMIT: usize = 8 * 1024 * 1024;
const MAX_RUNS: usize = 8;
const BOOTSTRAP: &str = "import runpy,sys;d,p=sys.argv[1:3];sys.path.insert(0,d);sys.argv=sys.argv[2:];runpy.run_path(p,run_name='__main__')";
struct Payload {
    name: &'static str,
    data: &'static [u8],
}
const HOME: &[Payload] = &[
    Payload {
        name: "report_home.py",
        data: include_bytes!("../../../scripts/report_home.py"),
    },
    Payload {
        name: "report_home.html",
        data: include_bytes!("../../../scripts/report_home.html"),
    },
];
const CAMPAIGN: &[Payload] = &[
    Payload {
        name: "debug_campaign.py",
        data: include_bytes!("../../../scripts/debug_campaign.py"),
    },
    Payload {
        name: "debug_campaign.html",
        data: include_bytes!("../../../scripts/debug_campaign.html"),
    },
    Payload {
        name: "artifact_coordinator.py",
        data: include_bytes!("../../../scripts/artifact_coordinator.py"),
    },
];
const ARTIFACTS: &[Payload] = &[
    Payload {
        name: "debug_artifacts.py",
        data: include_bytes!("../../../scripts/debug_artifacts.py"),
    },
    Payload {
        name: "debug_artifacts.html",
        data: include_bytes!("../../../scripts/debug_artifacts.html"),
    },
    Payload {
        name: "artifact_coordinator.py",
        data: include_bytes!("../../../scripts/artifact_coordinator.py"),
    },
    Payload {
        name: "local_artifacts.py",
        data: include_bytes!("../../../scripts/local_artifacts.py"),
    },
];
fn payload(tool: Tool) -> &'static [Payload] {
    match tool {
        Tool::Homepage => HOME,
        Tool::CampaignReport => CAMPAIGN,
        Tool::ArtifactPreview => ARTIFACTS,
        _ => &[],
    }
}
fn digest(bytes: &[u8]) -> String {
    gba_core::ffta_rom::digest(bytes)
}
fn read_json(path: &Path, limit: u64) -> Result<Value, String> {
    crate::debug_content::ordinary(path)?;
    serde_json::from_slice(&crate::diagnostics::read(path, limit)?).map_err(|e| e.to_string())
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Recipe {
    schema: u32,
    kind: String,
    label: String,
    steps: Vec<Step>,
}
#[derive(Deserialize)]
#[serde(tag = "operation_id", deny_unknown_fields)]
enum Step {
    #[serde(rename = "control.status")]
    Status {},
    #[serde(rename = "control.inspect")]
    Inspect { address: String, length: u32 },
}
impl Step {
    fn args(&self, session: &Path) -> Result<Vec<String>, String> {
        let mut args = match self {
            Self::Status {} => vec!["status".into(), session.to_string_lossy().into_owned()],
            Self::Inspect { address, length } => vec![
                "inspect".into(),
                session.to_string_lossy().into_owned(),
                address.clone(),
                length.to_string(),
            ],
        };
        crate::debug_control::parse(&args)?;
        args.extend(["--timeout-ms".into(), "10000".into()]);
        args.insert(0, "debug-control".into());
        Ok(args)
    }
}
fn recipe(path: &Path) -> Result<Recipe, String> {
    let v = read_json(path, 64 * 1024)?;
    let r: Recipe = serde_json::from_value(v).map_err(|e| format!("Readonly recipe: {e}"))?;
    if r.schema != 1
        || r.kind != "ffta-debug-recipe"
        || r.label.is_empty()
        || r.label.len() > 200
        || r.label.chars().any(char::is_control)
        || !(1..=16).contains(&r.steps.len())
    {
        return Err(
            "Recipe needs schema1, kindffta-debug-recipe, label1..200bytes and1..16readonly steps"
                .into(),
        );
    }
    for s in &r.steps {
        s.args(Path::new("selected-session"))?;
    }
    Ok(r)
}
pub(crate) fn input_preflight(slot: Slot, path: &Path) -> Result<(), String> {
    if slot == Slot::Recipe {
        recipe(path)?;
        return Ok(());
    }
    let v = read_json(path, 1024 * 1024)?;
    // Extractor projections predate envelope schemas. Validate their actual typed
    // format without adding metadata to the immutable source file.
    if slot == Slot::GuestScripts && v.get("schema").is_none() {
        use gba_core::rom_data::{
            auxiliary_scripts::AuxiliaryArchive, script_archive::ScriptArchive,
            script_commands::ScriptCommands,
        };
        let parse_error = |e: serde_json::Error| format!("Invalid typed guest catalog: {e}");
        return match path.file_name().and_then(|n| n.to_str()) {
            Some("script-commands.json") => serde_json::from_value::<ScriptCommands>(v)
                .map_err(parse_error)?
                .validate(),
            Some("script-archive.json") => serde_json::from_value::<ScriptArchive>(v)
                .map_err(parse_error)?
                .validate(),
            Some("auxiliary-script-archive.json" | "auxiliary-scripts.json") => {
                serde_json::from_value::<AuxiliaryArchive>(v)
                    .map_err(parse_error)?
                    .validate()
            }
            _ => Err("Unknown guest catalog projection".into()),
        };
    }
    if !v.is_object() || !v["schema"].is_u64() || v["schema"] != 1 {
        return Err("Catalog/manifest needs supported schema1 object".into());
    }
    if slot == Slot::Campaign
        && (v["kind"] != "ffta-debug-campaign"
            || v["cases"]
                .as_array()
                .is_none_or(|a| !(1..=32).contains(&a.len())))
    {
        return Err("Campaign needs kindffta-debug-campaign and1..32cases; adapter authenticates pinned evidence at Start".into());
    }
    Ok(())
}
fn guest(path: &Path) -> Result<String, String> {
    input_preflight(Slot::GuestScripts, path)?;
    let v = read_json(path, 1024 * 1024)?;
    let mut parts = vec![format!(
        "Readonly guest metadata\nSHA256 {}\nFile {}",
        digest(&crate::diagnostics::read(path, 1024 * 1024)?),
        path.display()
    )];
    for (key, value) in v.as_object().unwrap().iter().take(32) {
        parts.push(match value {
            Value::Array(a) => format!("{key}: {} declared entries", a.len()),
            Value::Object(a) => format!("{key}: {} declared fields", a.len()),
            _ => format!(
                "{key}: {}",
                value.to_string().chars().take(240).collect::<String>()
            ),
        });
    }
    parts.push("Declared catalog content is not independent verification. No guest instructions or host code executed. Gameplay inspector uses its existing authenticated ROM/model preflight.".into());
    Ok(parts.join("\n"))
}
fn runtime(d: &Draft) -> Result<(PathBuf, String), String> {
    let p = if d.tool == Tool::Recipe {
        d.optional(Slot::RuntimeB)?
            .unwrap_or(std::env::current_exe().map_err(|e| e.to_string())?)
    } else {
        d.path(Slot::HostRuntime).map_err(|e| format!("Python host runtime absent: {e}. Select an explicit Python3.11+ executable; no automatic install or Docker build."))?
    };
    crate::debug_content::ordinary(&p)?;
    Ok((
        p.clone(),
        digest(&crate::diagnostics::read(&p, 64 * 1024 * 1024)?),
    ))
}
fn check_payload(d: &Draft) -> Result<String, String> {
    let declared = payload(d.tool);
    if declared.is_empty() {
        return Ok(String::new());
    }
    let origin = if d.value("--source") == "external" {
        let selected = d.path(Slot::HostTool)?;
        if selected.file_name() != Some(std::ffi::OsStr::new(declared[0].name)) {
            return Err(format!(
                "Selected script must be the declared {} adapter",
                declared[0].name
            ));
        }
        let parent = selected.parent().ok_or("Script has no parent")?;
        for item in declared {
            let file = parent.join(item.name);
            crate::debug_content::ordinary(&file)?;
            if crate::diagnostics::read(&file, 2 * 1024 * 1024)? != item.data {
                return Err(format!("Changed declared script/dependency: {}. Reselecting does not authorize different code.", item.name));
            }
        }
        format!("Verified external source {}", selected.display())
    } else {
        "Embedded public source snapshot; staged only at explicit Start".into()
    };
    Ok(format!(
        "{origin}\n{}",
        declared
            .iter()
            .map(|p| format!("{} SHA256 {}", p.name, digest(p.data)))
            .collect::<Vec<_>>()
            .join("\n")
    ))
}
fn timeout(d: &Draft) -> Result<u64, String> {
    let v = d
        .value("--timeout-ms")
        .parse::<u64>()
        .map_err(|_| "Invalid host timeout")?;
    if !(20..=120000).contains(&v) {
        return Err("Host timeout needs20..120000ms".into());
    }
    Ok(v)
}
fn arguments(d: &Draft, report: &Path) -> Result<Vec<Vec<String>>, String> {
    let p = |slot| d.path(slot).map(|p| p.to_string_lossy().into_owned());
    Ok(match d.tool {
        Tool::Homepage => vec![vec!["--root".into(), p(Slot::Session)?, "--stdout".into()]],
        Tool::CampaignReport => {
            let mut a = vec![
                "report".into(),
                p(Slot::Campaign)?,
                "--out".into(),
                report.to_string_lossy().into_owned(),
            ];
            let case = d.value("--case");
            if case.is_empty() {
                return Err("Select one campaign case explicitly; no default all-case scan".into());
            }
            let manifest = read_json(&d.path(Slot::Campaign)?, 1024 * 1024)?;
            if !manifest["cases"]
                .as_array()
                .is_some_and(|cases| cases.iter().any(|v| v["id"] == case))
            {
                return Err("Selected case ID is absent from campaign manifest".into());
            }
            if !case.is_empty() {
                if case.len() > 64
                    || !case
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
                {
                    return Err("Case must be a portable1..64byte identifier or empty".into());
                }
                a.extend(["--case".into(), case.into()]);
            }
            vec![a]
        }
        Tool::ArtifactPreview => vec![vec![
            "preview".into(),
            "--session".into(),
            p(Slot::Session)?,
            "--out".into(),
            report.to_string_lossy().into_owned(),
        ]],
        Tool::Recipe => {
            let r = recipe(&d.path(Slot::Recipe)?)?;
            let s = d.path(Slot::Session)?;
            r.steps
                .iter()
                .map(|step| step.args(&s))
                .collect::<Result<Vec<_>, _>>()?
        }
        _ => return Err("Not an executable adapter".into()),
    })
}
pub(crate) fn description(tool: Tool) -> String {
    if tool == Tool::GuestScripts {
        return "Guest catalog v1: bounded readonly metadata view. Inspect catalog reads the selected file; Gameplay inspector opens its form and requires existing authenticated ROM/model checks. Nothing starts on selection.".into();
    }
    format!("Declared adapter v1: {}\n{}\n{}\nDetails verifies typed references without starting a process. Start is explicit; Cancel retains logs. Output: config/debug-tool-runs/<new-id>/;8runs,120s maximum,8MiB per stream. No overwrite or artifact deletion.", tool.label(),payload(tool).iter().map(|p|format!("{} SHA256 {}",p.name,digest(p.data))).collect::<Vec<_>>().join("\n"),if tool==Tool::Recipe {"Readonly schema1 recipe:1..16 control.status/control.inspect steps; selected recomp runtime. Python/PowerShell/Bash/Docker not used."} else {"Explicit Python3.11+ stdlib executable required. Embedded source by default, optional exact-hash external source. Direct argv adapter; PowerShell/Bash wrappers and Docker not used or installed."})
}
pub(crate) fn details(d: &Draft) -> Result<String, String> {
    if d.tool == Tool::GuestScripts {
        return guest(&d.path(Slot::GuestScripts)?);
    }
    let limits = timeout(d)?;
    let (rt, hash) = runtime(d)?;
    let source = check_payload(d)?;
    let argv = arguments(d, Path::new("NEW-RUN/report"))?;
    Ok(format!("Adapter v1: {}\n{source}\nRuntime {} SHA256 {hash}\n{}\nTyped argv: {argv:?}\nOutput: NEW config/debug-tool-runs/<id>/, request.json/result.json and capped stdout/stderr; {}\nLimits: {limits}ms total,8MiB per stream/process,16readonly recipe steps,8retained runs. Details verifies files only; no runtime/version process is started. Cancel retains diagnostics.",d.tool.label(),rt.display(),if d.tool==Tool::Recipe {"Requires selected recomp runtime; Python/PowerShell/Bash/Docker not used. Only control.status/control.inspect; shared CLI parsers."}else{"Requires explicitly selected Python3.11+ stdlib. PowerShell/Bash wrappers and Docker are not used by this direct adapter; no download/build. Runtime version is checked at Start."},match d.tool{Tool::Homepage=>"homepage.html (readonly root, existing index preserved)",Tool::CampaignReport=>"report/index.html and campaign metadata; no replay",Tool::ArtifactPreview=>"report/index.html and reviewed plan preview; no deletion or apply",_=>"one JSON stdout per readonly command; no guest steps"}))
}
fn ordinary_tree(path: &Path) -> Result<(), String> {
    let mut p = PathBuf::new();
    for c in path.components() {
        p.push(c);
        if p.exists() {
            crate::debug_content::ordinary(&p)?;
        }
    }
    Ok(())
}
fn new_run(config: &Path) -> Result<PathBuf, String> {
    let base = config.join("debug-tool-runs");
    ordinary_tree(&base)?;
    if base.exists()
        && fs::read_dir(&base)
            .map_err(|e| e.to_string())?
            .take(MAX_RUNS + 1)
            .count()
            >= MAX_RUNS
    {
        return Err(
            "Eight host tool runs retained; explicitly archive them before another Start".into(),
        );
    }
    fs::create_dir_all(&base).map_err(|e| e.to_string())?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let root = base.join(format!("{stamp}-{}", std::process::id()));
    fs::create_dir(&root).map_err(|e| e.to_string())?;
    Ok(root)
}
fn publish(path: &Path, v: &Value) -> Result<(), String> {
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    f.write_all(&serde_json::to_vec_pretty(v).map_err(|e| e.to_string())?)
        .and_then(|_| f.sync_all())
        .map_err(|e| e.to_string())
}
fn log_reader(
    mut src: impl Read + Send + 'static,
    path: PathBuf,
    exceeded: Arc<AtomicBool>,
) -> thread::JoinHandle<Result<usize, String>> {
    thread::spawn(move || {
        let mut f = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        let mut written = 0;
        let mut b = [0; 8192];
        loop {
            let n = src.read(&mut b).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            let keep = n.min(LOG_LIMIT.saturating_sub(written));
            f.write_all(&b[..keep]).map_err(|e| e.to_string())?;
            written += keep;
            if n > keep {
                exceeded.store(true, Ordering::SeqCst);
            }
        }
        f.sync_all().map_err(|e| e.to_string())?;
        Ok(written)
    })
}
#[cfg(windows)]
mod tree {
    use super::*;
    use std::{ffi::c_void, os::windows::io::AsRawHandle};
    type Handle = *mut c_void;
    #[repr(C)]
    #[derive(Default)]
    struct Basic {
        process_time: i64,
        job_time: i64,
        flags: u32,
        min: usize,
        max: usize,
        active: u32,
        affinity: usize,
        priority: u32,
        scheduling: u32,
    }
    #[repr(C)]
    #[derive(Default)]
    struct Extended {
        basic: Basic,
        io: [u64; 6],
        process_memory: usize,
        job_memory: usize,
        peak_process: usize,
        peak_job: usize,
    }
    #[repr(C)]
    #[derive(Default)]
    struct ThreadEntry {
        size: u32,
        usage: u32,
        id: u32,
        owner: u32,
        base: i32,
        delta: i32,
        flags: u32,
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn CreateJobObjectW(attr: *const c_void, name: *const u16) -> Handle;
        fn SetInformationJobObject(job: Handle, class: u32, info: *const c_void, len: u32) -> i32;
        fn AssignProcessToJobObject(job: Handle, process: Handle) -> i32;
        fn CloseHandle(handle: usize) -> i32;
        fn CreateToolhelp32Snapshot(flags: u32, pid: u32) -> Handle;
        fn Thread32First(snapshot: Handle, entry: *mut ThreadEntry) -> i32;
        fn Thread32Next(snapshot: Handle, entry: *mut ThreadEntry) -> i32;
        fn OpenThread(access: u32, inherit: i32, id: u32) -> Handle;
        fn ResumeThread(thread: Handle) -> u32;
    }
    pub struct Tree(Handle);
    impl Tree {
        pub fn attach(child: &Child) -> Result<Self, String> {
            unsafe {
                let h = CreateJobObjectW(std::ptr::null(), std::ptr::null());
                if h.is_null() {
                    return Err(format!(
                        "CreateJobObject: {}",
                        std::io::Error::last_os_error()
                    ));
                }
                let job = Self(h);
                let mut limits = Extended::default();
                limits.basic.flags = 0x2000;
                if SetInformationJobObject(
                    h,
                    9,
                    &limits as *const _ as *const c_void,
                    std::mem::size_of::<Extended>() as u32,
                ) == 0
                    || AssignProcessToJobObject(h, child.as_raw_handle()) == 0
                {
                    return Err(format!(
                        "Process tree containment unavailable: {}",
                        std::io::Error::last_os_error()
                    ));
                }
                // Spawn is suspended: no source instruction can create an escaping
                // descendant before job membership. std ChildExt thread access is
                // unstable, so locate this new process's sole suspended thread.
                let snapshot = CreateToolhelp32Snapshot(4, 0);
                if snapshot as isize == -1 {
                    return Err("Cannot inspect suspended host process threads".into());
                }
                let mut entry = ThreadEntry::default();
                entry.size = std::mem::size_of::<ThreadEntry>() as u32;
                let mut found = std::ptr::null_mut();
                let mut next = Thread32First(snapshot, &mut entry);
                for _ in 0..16384 {
                    if next == 0 {
                        break;
                    }
                    if entry.owner == child.id() {
                        found = OpenThread(2, 0, entry.id);
                        break;
                    }
                    entry.size = std::mem::size_of::<ThreadEntry>() as u32;
                    next = Thread32Next(snapshot, &mut entry);
                }
                CloseHandle(snapshot as usize);
                if found.is_null() {
                    return Err(
                        "Cannot obtain suspended host process thread; nothing executed".into(),
                    );
                }
                let resumed = ResumeThread(found);
                CloseHandle(found as usize);
                if resumed != 1 {
                    return Err("Cannot resume contained host process".into());
                }
                Ok(job)
            }
        }
    }
    impl Drop for Tree {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.0 as usize);
            }
        }
    }
}
#[cfg(unix)]
mod tree {
    use super::*;
    unsafe extern "C" {
        fn kill(pid: i32, signal: i32) -> i32;
    }
    pub struct Tree(i32);
    impl Tree {
        pub fn attach(child: &Child) -> Result<Self, String> {
            Ok(Self(child.id() as i32))
        }
    }
    impl Drop for Tree {
        fn drop(&mut self) {
            unsafe {
                kill(-self.0, 9);
            }
        }
    }
}
#[derive(Debug)]
struct ProcessResult {
    outcome: &'static str,
    code: Option<i32>,
    stdout: usize,
    stderr: usize,
}
fn process(
    command: &mut Command,
    root: &Path,
    name: &str,
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<ProcessResult, String> {
    if cancel.load(Ordering::SeqCst) {
        return Err("Cancelled before process launch".into());
    }
    if Instant::now() >= deadline {
        return Err("Deadline reached before process launch".into());
    }
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000 | 0x00000004);
    }
    let mut child = command
        .spawn()
        .map_err(|e| format!("Cannot start declared host runtime: {e}"))?;
    let tree = match tree::Tree::attach(&child) {
        Ok(t) => t,
        Err(e) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(e);
        }
    };
    let exceeded = Arc::new(AtomicBool::new(false));
    let out = log_reader(
        child.stdout.take().unwrap(),
        root.join(format!("{name}.stdout")),
        exceeded.clone(),
    );
    let err = log_reader(
        child.stderr.take().unwrap(),
        root.join(format!("{name}.stderr")),
        exceeded.clone(),
    );
    let mut outcome = "completed";
    let status = loop {
        if cancel.load(Ordering::SeqCst) {
            outcome = "cancelled";
            break None;
        }
        if Instant::now() >= deadline {
            outcome = "timeout";
            break None;
        }
        if exceeded.load(Ordering::SeqCst) {
            outcome = "log-limit";
            break None;
        }
        match child.try_wait() {
            Ok(Some(s)) => break Some(s),
            Ok(None) => (),
            Err(_) => {
                outcome = "worker-error";
                break None;
            }
        }
        thread::sleep(Duration::from_millis(10));
    };
    // Closing the job/group also stops descendants retaining inherited pipe handles.
    drop(tree);
    let status = if let Some(s) = status {
        s
    } else {
        let _ = child.kill();
        child.wait().map_err(|e| e.to_string())?
    };
    let stdout = out.join().map_err(|_| "stdout collector panicked")??;
    let stderr = err.join().map_err(|_| "stderr collector panicked")??;
    if exceeded.load(Ordering::SeqCst) {
        outcome = "log-limit";
    } else if outcome == "completed" && !status.success() {
        outcome = "worker-error";
    }
    Ok(ProcessResult {
        outcome,
        code: status.code(),
        stdout,
        stderr,
    })
}
fn python_command(runtime: &Path) -> Command {
    let mut c = Command::new(runtime);
    c.args(["-I", "-X", "utf8", "-S", "-B", "-u"]);
    c.env_remove("PYTHONPATH").env_remove("PYTHONHOME");
    c
}
fn verify_version(root: &Path) -> Result<String, String> {
    let out = crate::diagnostics::read(&root.join("version.stdout"), LOG_LIMIT as u64)?;
    let err = crate::diagnostics::read(&root.join("version.stderr"), LOG_LIMIT as u64)?;
    let v = format!(
        "{}{}",
        String::from_utf8_lossy(&out),
        String::from_utf8_lossy(&err)
    );
    let nums = v
        .trim()
        .strip_prefix("Python ")
        .ok_or("Host executable did not identify itself as Python")?
        .split('.')
        .take(2)
        .map(|s| s.parse::<u32>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Invalid Python version")?;
    if nums.len() != 2 || nums[0] != 3 || nums[1] < 11 {
        return Err("Python3.11+ required".into());
    }
    Ok(v.trim().into())
}
pub(crate) struct RunOutput { pub message:String, pub observation:Value }
#[cfg(test)]
pub(crate) fn run(d:Draft,cancel:Arc<AtomicBool>)->Result<String,String>{run_observed(d,cancel).map(|r|r.message)}
pub(crate) fn run_observed(d: Draft, cancel: Arc<AtomicBool>) -> Result<RunOutput, String> {
    let summary = details(&d)?;
    if d.tool == Tool::GuestScripts {
        return Ok(RunOutput{message:summary,observation:json!({"outcome":"completed","operation":"guest-script-inspection","guest_execution":false,"host_process_started":false})});
    }
    let (rt, rt_hash) = runtime(&d)?;
    let deadline = Instant::now() + Duration::from_millis(timeout(&d)?);
    let root = new_run(&d.config)?;
    let steps = arguments(&d, &root.join("report"))?;
    let declared = payload(d.tool);
    let source = root.join("source");
    if !declared.is_empty() {
        fs::create_dir(&source).map_err(|e| e.to_string())?;
        for item in declared {
            let mut f = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(source.join(item.name))
                .map_err(|e| e.to_string())?;
            f.write_all(item.data).map_err(|e| e.to_string())?;
        }
    }
    publish(
        &root.join("request.json"),
        &json!({"schema":1,"kind":"ffta-debug-host-request","adapter":format!("{:?}",d.tool),"version":1,"details":summary,"runtime":rt,"runtime_sha256":rt_hash,"steps":steps,"payload":declared.iter().map(|p|json!({"name":p.name,"sha256":digest(p.data)})).collect::<Vec<_>>()}),
    )?;
    let mut results = Vec::new();
    let execute = (|| -> Result<&'static str, String> {
        if digest(&crate::diagnostics::read(&rt, 64 * 1024 * 1024)?) != rt_hash {
            return Err("Host runtime changed before launch".into());
        }
        if !declared.is_empty() {
            let r = process(
                python_command(&rt).arg("--version"),
                &root,
                "version",
                &cancel,
                deadline,
            )?;
            let outcome = r.outcome;
            results.push(json!({"step":"version","outcome":outcome,"exit_code":r.code,"stdout_bytes":r.stdout,"stderr_bytes":r.stderr}));
            if outcome != "completed" {
                return Ok(outcome);
            }
            let version = verify_version(&root)?;
            results.push(json!({"python_version":version}));
        }
        for (i, args) in steps.iter().enumerate() {
            // Recheck selected script/dependency identities immediately before each launch.
            check_payload(&d)?;
            if digest(&crate::diagnostics::read(&rt, 64 * 1024 * 1024)?) != rt_hash {
                return Err("Host runtime changed before launch".into());
            }
            let mut command = if declared.is_empty() {
                Command::new(&rt)
            } else {
                let mut c = python_command(&rt);
                c.args(["-c", BOOTSTRAP])
                    .arg(&source)
                    .arg(source.join(declared[0].name));
                c
            };
            command.args(args).current_dir(&root);
            let name = format!("step-{i:02}");
            let r = process(&mut command, &root, &name, &cancel, deadline)?;
            let outcome = r.outcome;
            results.push(json!({"step":i,"argv":args,"outcome":outcome,"exit_code":r.code,"stdout_bytes":r.stdout,"stderr_bytes":r.stderr}));
            if outcome != "completed" {
                return Ok(outcome);
            }
            if d.tool == Tool::Homepage {
                fs::rename(
                    root.join(format!("{name}.stdout")),
                    root.join("homepage.html"),
                )
                .map_err(|e| e.to_string())?;
            }
        }
        Ok("completed")
    })();
    let (outcome, error) = match execute {
        Ok(o) => (o, None),
        Err(e) => (
            if cancel.load(Ordering::SeqCst) {
                "cancelled"
            } else {
                "refused"
            },
            Some(e),
        ),
    };
    publish(
        &root.join("result.json"),
        &json!({"schema":1,"kind":"ffta-debug-host-result","outcome":outcome,"error":error,"steps":results,"output":root,"guest_execution":false}),
    )?;
    let observation=json!({"outcome":outcome,"error":error.as_ref().map(|e|e.chars().take(512).collect::<String>()),"error_truncated":error.as_ref().is_some_and(|e|e.chars().count()>512),"output":root.to_string_lossy().chars().take(1024).collect::<String>(),"output_path_truncated":root.to_string_lossy().chars().count()>1024,"result_records":results.len(),"guest_execution":false,"scope":"Declared process attempts; exact step outcomes remain in result.json"});
    Ok(RunOutput{observation,message:format!(
        "Host tool {outcome}. Logs/result: {}{}",
        root.display(),
        error.map(|e| format!("\n{e}")).unwrap_or_default()
    )})
}

#[cfg(test)]
mod tests;
