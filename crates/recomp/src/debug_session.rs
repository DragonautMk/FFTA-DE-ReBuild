//! Dedicated portable copies: the normal process stays paused and owns its progress.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs::{self, File, OpenOptions}, io::Write,
    path::{Component, Path, PathBuf}, process::{Child, Command}};

const MANIFEST: &str = "debug-session.json";
const MAX_PAYLOAD: u64 = 768 * 1024 * 1024;
const MAX_TREE_FILES: usize = 4096;
const MAX_DEFAULT_SESSIONS: usize = 8;
fn exe_name() -> &'static str { if cfg!(windows) { "recomp.exe" } else { "recomp" } }

pub(crate) struct Source<'a> {
    pub rom: &'a [u8],
    pub bios: Option<&'a [u8]>,
    pub save: Option<&'a [u8]>,
    pub state: Option<&'a [u8]>,
    pub av: input_config::AvConfig,
    pub input: input_config::InputConfig,
    pub interp: bool,
    pub dataset: Option<&'a Path>,
    pub executable: &'a Path,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub(crate) struct Manifest {
    schema: u32,
    kind: String,
    pub label: String,
    interp: bool,
    real_bios: bool,
    has_state: bool,
    has_dataset: bool,
    // Session-relative names only. Mutable progress/config/cache are never pinned.
    pins: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    artifact_coordination: Option<String>,
}

pub(crate) struct Session {
    pub root: PathBuf,
    pub manifest: Manifest,
    _lease: File,
    _artifact_lease: crate::artifact_lease::Lease,
}

pub(crate) fn present() -> bool {
    input_config::exe_dir().is_some_and(|p| p.join(MANIFEST).exists())
}

fn project_variable(name: &std::ffi::OsStr) -> bool {
    let name = name.to_string_lossy().to_ascii_uppercase();
    ["FFTA_", "RECOMP_", "GBA_"].iter().any(|p| name.starts_with(p))
}

pub(crate) fn regular(path: &Path) -> Result<fs::Metadata, String> {
    let meta = fs::symlink_metadata(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut linked = meta.file_type().is_symlink();
    #[cfg(windows)] {
        use std::os::windows::fs::MetadataExt;
        linked |= meta.file_attributes() & 0x400 != 0; // FILE_ATTRIBUTE_REPARSE_POINT
    }
    #[cfg(unix)] {
        use std::os::unix::fs::MetadataExt;
        linked |= meta.is_file() && meta.nlink() != 1;
    }
    #[cfg(windows)]
    if meta.is_file() { linked |= windows_link_count(path)? != 1; }
    if linked || (!meta.is_file() && !meta.is_dir()) {
        return Err(format!("Session paths must be regular, unshared files/directories: {}", path.display()));
    }
    Ok(meta)
}

#[cfg(windows)]
fn windows_link_count(path: &Path) -> Result<u32, String> {
    use std::{ffi::c_void, os::windows::io::AsRawHandle};
    // BY_HANDLE_FILE_INFORMATION: 13 DWORDs, nNumberOfLinks at DWORD 10.
    #[link(name = "kernel32")]
    extern "system" { fn GetFileInformationByHandle(handle: *mut c_void, info: *mut u32) -> i32; }
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut info = [0u32; 13];
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), info.as_mut_ptr()) } == 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }
    Ok(info[10])
}

fn tree(root: &Path) -> Result<Vec<PathBuf>, String> {
    regular(root)?;
    let mut pending = vec![(root.to_owned(), 0)];
    let mut files = Vec::new();
    let mut total = 0u64;
    let mut entries = 0;
    while let Some((dir, depth)) = pending.pop() {
        if depth > 16 { return Err("Session directory nesting exceeds 16 levels".into()); }
        for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
            let path = entry.map_err(|e| e.to_string())?.path();
            let meta = regular(&path)?;
            entries += 1;
            if entries > MAX_TREE_FILES { return Err("Session exceeds 4096 entries".into()); }
            if meta.is_dir() { pending.push((path, depth + 1)); }
            else {
                total += meta.len();
                if total > 2 * 1024 * 1024 * 1024 { return Err("Session exceeds 2 GiB; review its local cache/captures".into()); }
                files.push(path);
            }
        }
    }
    Ok(files)
}

fn read_manifest(root: &Path) -> Result<Manifest, String> {
    tree(root)?;
    let m: Manifest = serde_json::from_slice(&crate::diagnostics::read(&root.join(MANIFEST), 1024 * 1024)?)
        .map_err(|e| format!("Invalid debug-session manifest: {e}"))?;
    if m.schema != 1 || m.kind != "ffta-debug-session" || m.pins.len() > MAX_TREE_FILES {
        return Err("Unsupported debug-session manifest".into());
    }
    for required in ["portable.txt", "ffta.gba", exe_name()] {
        if !m.pins.contains_key(required) { return Err(format!("Session pin missing: {required}")); }
    }
    for (name, expected) in &m.pins {
        if name.is_empty() || !Path::new(name).components().all(|p| matches!(p, Component::Normal(_))) {
            return Err("Session manifest paths must stay relative to its folder".into());
        }
        let bytes = crate::diagnostics::read(&root.join(name), MAX_PAYLOAD)?;
        if gba_core::ffta_rom::digest(&bytes) != *expected { return Err(format!("Session input changed: {name}")); }
    }
    for (enabled, name) in [(m.real_bios, "gba_bios.bin"), (m.has_state, "seed.ffstate"), (m.has_dataset, "Data/dataset/manifest.toml")] {
        if enabled != m.pins.contains_key(name) { return Err(format!("Inconsistent session input: {name}")); }
    }
    Ok(m)
}

pub(crate) fn current() -> Result<Option<Session>, String> {
    if !present() { return Ok(None); }
    if let Some((name, _)) = std::env::vars_os().find(|(name, _)| project_variable(name)) {
        return Err(format!("Isolated session refuses inherited {}. Use recomp debug-session open <folder> to launch with clean overrides.", name.to_string_lossy()));
    }
    let root = input_config::exe_dir().ok_or("Cannot locate session executable")?;
    open(&root).map(Some)
}

pub(crate) fn open(root: &Path) -> Result<Session, String> {
    let artifact_lease=crate::artifact_lease::Lease::acquire(root,"runtime")?;
    let manifest = read_manifest(root)?;
    if manifest.artifact_coordination.as_deref().is_some_and(|p|p!=crate::artifact_lease::PROTOCOL){return Err("Unsupported artifact coordination protocol".into());}
    let lease = OpenOptions::new().create(true).truncate(false).read(true).write(true)
        .open(root.join("session.lock")).map_err(|e| e.to_string())?;
    lease.try_lock().map_err(|_| "This debug session is already running")?;
    Ok(Session { root: root.to_owned(), manifest, _lease: lease, _artifact_lease:artifact_lease })
}

impl Session {
    pub fn rom(&self) -> PathBuf { self.root.join("ffta.gba") }
    pub fn dataset(&self) -> Option<PathBuf> { self.manifest.has_dataset.then(|| self.root.join("Data/dataset")) }
    pub fn state(&self) -> Option<PathBuf> { self.manifest.has_state.then(|| self.root.join("seed.ffstate")) }
    pub fn bios(&self) -> Option<PathBuf> { self.manifest.real_bios.then(|| self.root.join("gba_bios.bin")) }
    pub fn interp(&self) -> bool { self.manifest.interp }
    pub fn result(&self, result: &Result<(), String>) {
        let value = serde_json::json!({"schema":1,"kind":"ffta-debug-session-result","ok":result.is_ok(),
            "error": result.as_ref().err().map(|e| e.chars().take(2048).collect::<String>())});
        let write = (|| -> Result<(), String> {
            let mut file = tempfile::NamedTempFile::new_in(&self.root).map_err(|e| e.to_string())?;
            file.write_all(value.to_string().as_bytes()).map_err(|e| e.to_string())?;
            file.persist(self.root.join("last-result.json")).map_err(|e| e.error.to_string())?;
            Ok(())
        })();
        if let Err(error) = write { eprintln!("Cannot retain session result: {error}"); }
    }
}

pub(crate) fn default_destination() -> Result<PathBuf, String> {
    let base = match input_config::storage::portable_root() {
        Some(root) => root.join("Data/debug-sessions"),
        None => input_config::config_root().ok_or("Cannot locate debug-session storage")?.join("debug-sessions"),
    };
    if base.exists() && fs::read_dir(&base).map_err(|e| e.to_string())?.count() >= MAX_DEFAULT_SESSIONS {
        return Err("Eight debug sessions already exist. Reopen one or move retained sessions before creating another.".into());
    }
    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|e| e.to_string())?.as_millis();
    Ok(base.join(format!("session-{stamp}-{}", std::process::id())))
}

pub(crate) fn create(destination: &Path, source: Source<'_>) -> Result<PathBuf, String> {
    if crate::packfile::load().is_some() {
        return Err("Create debug sessions with the development runtime; pinned full-recomp packages are not copied by this tool".into());
    }
    if destination.exists() { return Err("Debug-session destination already exists; choose a new folder".into()); }
    crate::validate_rom("session ROM", source.rom)?;
    if source.bios.is_some_and(|b| b.len() != input_config::BIOS_SIZE) { return Err("BIOS must contain exactly 16384 bytes".into()); }
    let mut scratch = crate::make_machine(source.rom.to_vec(), source.bios);
    if let Some(save) = source.save {
        if scratch.bus.save_data().is_none_or(|s| s.len() != save.len()) { return Err("Cartridge save size does not match this ROM".into()); }
    }
    if let Some(state) = source.state { crate::save_state::load_bytes(&mut scratch, state)?; }
    if let Some(dataset) = source.dataset { crate::rom_data::diagnosis::inspect(source.rom, dataset)?; }
    let library = if source.interp { None } else { Some(recomp_core::tcc_memory::library_path().ok_or("Bundled TinyCC is required for a movable native session; use --interp or install tcc/libtcc beside the source runtime")?) };
    let executable = crate::diagnostics::read(source.executable, 256 * 1024 * 1024)?;
    let root = std::path::absolute(destination).map_err(|e| e.to_string())?;
    fs::create_dir_all(root.parent().ok_or("Session needs a parent directory")?).map_err(|e| e.to_string())?;
    fs::create_dir(&root).map_err(|e| e.to_string())?;
    let _artifact_lease=crate::artifact_lease::Lease::acquire(&root,"session-create")?;
    // Manifest is published last. I/O failures retain an incomplete private folder.
    fs::write(root.join(".gitignore"), "*\n").map_err(|e| e.to_string())?;
    let mut pins = BTreeMap::new();
    let mut total = 0;
    let mut put = |name: &str, bytes: &[u8], immutable: bool| -> Result<(), String> {
        total += bytes.len() as u64;
        if total > MAX_PAYLOAD { return Err("Session copy exceeds 768 MiB".into()); }
        let path = root.join(name);
        fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        let mut file = OpenOptions::new().write(true).create_new(true).open(path).map_err(|e| e.to_string())?;
        file.write_all(bytes).and_then(|_| file.sync_all()).map_err(|e| e.to_string())?;
        if immutable { pins.insert(name.to_owned(), gba_core::ffta_rom::digest(bytes)); }
        Ok(())
    };
    put(exe_name(), &executable, true)?;
    #[cfg(unix)] { use std::os::unix::fs::PermissionsExt; fs::set_permissions(root.join(exe_name()), fs::Permissions::from_mode(0o755)).map_err(|e| e.to_string())?; }
    put("portable.txt", b"Isolated local debug session\n", true)?;
    put("ffta.gba", source.rom, true)?;
    if let Some(bios) = source.bios { put("gba_bios.bin", bios, true)?; }
    if let Some(save) = source.save { put("Data/saves/ffta.sav", save, false)?; }
    if let Some(state) = source.state {
        put("seed.ffstate", state, true)?;
        put(&format!("Data/saves/SaveStates/{}/slot-1.ffstate", gba_core::ffta_rom::digest(source.rom)), state, false)?;
    }
    let mut av = source.av; av.debug_game_save_slot = 1;
    put("Data/config/av.cfg", av.serialize().as_bytes(), false)?;
    put("Data/config/input.cfg", source.input.serialize().as_bytes(), false)?;
    if let Some(dataset) = source.dataset {
        for file in tree(dataset)? {
            let relative = file.strip_prefix(dataset).map_err(|e| e.to_string())?.to_string_lossy().replace('\\', "/");
            put(&format!("Data/dataset/{relative}"), &crate::diagnostics::read(&file, MAX_PAYLOAD)?, true)?;
        }
    }
    if let Some(library) = library {
        let name = if cfg!(windows) { "libtcc.dll" } else if cfg!(target_os = "macos") { "libtcc.dylib" } else { "libtcc.so" };
        put(&format!("tcc/{name}"), &crate::diagnostics::read(&library, 64 * 1024 * 1024)?, true)?;
    }
    let label = root.file_name().unwrap_or_default().to_string_lossy().chars().filter(|c| !c.is_control()).take(48).collect();
    let manifest = Manifest { schema:1, kind:"ffta-debug-session".into(), label, interp:source.interp,
        real_bios:source.bios.is_some(), has_state:source.state.is_some(), has_dataset:source.dataset.is_some(), pins, artifact_coordination:Some(crate::artifact_lease::PROTOCOL.into()) };
    fs::write(root.join(MANIFEST), serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    Ok(root)
}

fn scrub(command: &mut Command, names: impl Iterator<Item = std::ffi::OsString>) {
    for name in names.filter(|name| project_variable(name)) { command.env_remove(name); }
}

/// Read only immutable source pins for preparing a NEW profile while its UI is open.
/// Does not grant a runtime/save writer lease or launch the selected session.
pub(crate) fn profile_source(root:&Path)->Result<(PathBuf,Option<PathBuf>,Option<PathBuf>,Option<Session>),String>{
    let same_root=input_config::exe_dir().and_then(|p|fs::canonicalize(p).ok())==fs::canonicalize(root).ok();
    let owner=crate::diagnostics::read(&root.join(crate::artifact_lease::NAME).join("owner.json"),4096).ok().and_then(|b|serde_json::from_slice::<serde_json::Value>(&b).ok());
    let owned=same_root && owner.as_ref().is_some_and(|v|v["pid"]==std::process::id() && v["role"]=="runtime");
    // Other sources retain their existing exclusive source lease; the active UI
    // may read its own pinned inputs while it already holds that exact lease.
    let lease=if owned{None}else{Some(open(root)?)};
    let m=read_manifest(root)?;
    Ok((root.join("ffta.gba"),m.real_bios.then(||root.join("gba_bios.bin")),m.has_dataset.then(||root.join("Data/dataset")),lease))
}

pub(crate) fn replay_command(root: &Path) -> Result<Command, String> {
    read_manifest(root)?;
    let mut command = Command::new(root.join(exe_name()));
    command.current_dir(root);
    scrub(&mut command, std::env::vars_os().map(|(name, _)| name));
    #[cfg(windows)] { use std::os::windows::process::CommandExt; command.creation_flags(0x08000000); }
    Ok(command)
}

pub(crate) fn launch(root: &Path) -> Result<Child, String> {
    launch_mode(root, false, false)
}

pub(crate) fn launch_recording(root: &Path) -> Result<Child, String> {
    launch_mode(root, true, false)
}

fn launch_mode(root: &Path, record: bool, paused: bool) -> Result<Child, String> {
    read_manifest(root)?;
    let mut command = Command::new(root.join(exe_name()));
    command.current_dir(root).arg("play");
    if record { command.arg("--record-input"); }
    if paused { command.arg("--debug-paused"); }
    scrub(&mut command, std::env::vars_os().map(|(name, _)| name));
    #[cfg(windows)] { use std::os::windows::process::CommandExt; command.creation_flags(0x08000000); }
    command.spawn().map_err(|e| format!("Cannot start debug-session copy: {e}"))
}

pub(crate) fn cmd(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("open") if args.len() == 2 || (args.len()==3 && args[2]=="--paused") => {
            let root = fs::canonicalize(&args[1]).map_err(|e| e.to_string())?;
            let status = launch_mode(&root, false, args.len()==3)?.wait().map_err(|e| e.to_string())?;
            if status.success() { Ok(()) } else { Err(format!("Debug-session process failed: {status}")) }
        }
        Some("inspect") if args.len() == 2 => {
            let root = fs::canonicalize(&args[1]).map_err(|e| e.to_string())?;
            let manifest = read_manifest(&root)?;
            println!("{}", serde_json::json!({"schema":1,"kind":"ffta-debug-session-inspection","root":root,"session":manifest}));
            Ok(())
        }
        Some("create") => create_cmd(&args[1..]),
        _ => Err("Use debug-session create ROM --dir DIR [options], open DIR, or inspect DIR".into()),
    }
}

fn create_cmd(args: &[String]) -> Result<(), String> {
    let mut request = crate::diagnostics::Request::default();
    let mut destination = None; let mut open = false;
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--open" => open = true, "--interp" => request.interp = true,
            "--no-bios" => request.no_bios = true, "--rom-data" => request.managed = true,
            "--dir" | "--sav" | "--state" | "--bios" | "--dataset" => {
                let value = PathBuf::from(it.next().ok_or_else(|| format!("{arg} needs a path"))?);
                match arg.as_str() { "--dir"=>destination=Some(value), "--sav"=>request.save=Some(value),
                    "--state"=>request.state=Some(value), "--bios"=>request.bios=Some(value), "--dataset"=>request.dataset=Some(value), _=>unreachable!() }
            }
            other if !other.starts_with('-') && request.rom.as_os_str().is_empty() => request.rom = other.into(),
            _ => return Err(format!("Unexpected debug-session argument {arg}")),
        }
    }
    if request.rom.as_os_str().is_empty() { return Err("create needs a ROM".into()); }
    if request.no_bios && request.bios.is_some() { return Err("--bios and --no-bios are mutually exclusive".into()); }
    if request.managed && request.dataset.is_some() { return Err("--dataset and --rom-data are mutually exclusive".into()); }
    let destination = destination.ok_or("create needs --dir DIR")?;
    let report = crate::diagnostics::collect(&request, None);
    if report["ok"] != true { return Err(format!("Session preflight failed: {}", report["issues"])); }
    let rom = crate::diagnostics::read(&request.rom, 32 * 1024 * 1024)?;
    let bios_path = if request.no_bios { None } else { request.bios.or_else(input_config::find_bios) };
    let bios = bios_path.as_ref().map(|p| crate::diagnostics::read(p, input_config::BIOS_SIZE as u64)).transpose()?;
    let save_path = request.save.or_else(|| report["cartridge_save"]["path"].as_str().map(PathBuf::from));
    let save = save_path.as_ref().filter(|p| p.exists()).map(|p| crate::diagnostics::read(p, 128 * 1024)).transpose()?;
    let state = request.state.as_ref().map(|p| crate::diagnostics::read(p, gba_core::save_state::MAX_STATE_BYTES as u64 + 160 * 240 * 4 + 48)).transpose()?;
    let dataset = report["dataset"]["directory"].as_str().map(PathBuf::from);
    let root = create(&destination, Source { rom:&rom, bios:bios.as_deref(), save:save.as_deref(), state:state.as_deref(),
        av:input_config::AvConfig::load(), input:input_config::InputConfig::load(), interp:request.interp,
        dataset:dataset.as_deref(), executable:&std::env::current_exe().map_err(|e| e.to_string())? })?;
    println!("{}", serde_json::json!({"schema":1,"kind":"ffta-debug-session-created","root":root,"opened":open}));
    if open { let status = launch(&root)?.wait().map_err(|e| e.to_string())?; if !status.success() { return Err(format!("Debug session failed: {status}")); } }
    Ok(())
}

#[cfg(test)]
mod tests;
