//! Named, session-pinned definitions for the existing bounded search handler.
//! Saving or enabling a definition never starts a search.
use super::{parse, Command};
use crate::debug_control::Action;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

const MAX_TRIGGERS: usize = 32;
const MAX_BYTES: u64 = 16 * 1024;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Trigger {
    schema: u32,
    kind: String,
    name: String,
    manifest_sha256: String,
    rom_sha256: String,
    runtime_sha256: String,
    enabled: bool,
    action: Action,
}
impl Trigger {
    fn validate(&self) -> Result<(), String> {
        if self.schema != 1
            || self.kind != "ffta-debug-trigger"
            || !crate::debug_cheat_preset::name_valid(&self.name)
        {
            return Err("Invalid trigger schema or name".into());
        }
        if ![
            &self.manifest_sha256,
            &self.rom_sha256,
            &self.runtime_sha256,
        ]
        .iter()
        .all(|s| {
            s.len() == 64
                && s.bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        }) {
            return Err("Trigger needs exact lowercase session pins".into());
        }
        match &self.action {
            Action::Find { settings } if settings.trigger.is_none() => (),
            Action::Find { .. } => {
                return Err("Stored trigger definitions cannot embed a run identity".into())
            }
            _ => return Err("Only bounded PC and bus searches are supported".into()),
        }
        self.action.validate()
    }
    fn verify_session(&self, root: &Path) -> Result<(), String> {
        let (manifest, rom, runtime) = super::recipe::manifest(root)?;
        if (manifest, rom, runtime)
            != (
                self.manifest_sha256.clone(),
                self.rom_sha256.clone(),
                self.runtime_sha256.clone(),
            )
        {
            return Err("Session pins changed; trigger requires its original session".into());
        }
        Ok(())
    }
}
fn directory(root: &Path, create: bool) -> Result<PathBuf, String> {
    for path in [
        root.to_path_buf(),
        root.join("Data"),
        root.join("Data/config"),
    ] {
        if !crate::debug_session::regular(&path)?.is_dir() {
            return Err("Session configuration root is not a directory".into());
        }
    }
    let dir = root.join("Data/config/debug-triggers");
    if create {
        match fs::create_dir(&dir) {
            Ok(()) => (),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
            Err(e) => return Err(e.to_string()),
        }
    }
    if !crate::debug_session::regular(&dir)?.is_dir() {
        return Err("Trigger path is not a directory".into());
    }
    Ok(dir)
}
fn path(dir: &Path, name: &str) -> Result<PathBuf, String> {
    if !crate::debug_cheat_preset::name_valid(name) {
        return Err("Trigger name must contain 1..48 ASCII letters, digits, '-' or '_'".into());
    }
    Ok(dir.join(format!("trigger-{name}.json")))
}
fn lock(dir: &Path) -> Result<std::fs::File, String> {
    let p = dir.join(".trigger.lock");
    if p.exists() {
        crate::debug_session::regular(&p)?;
    }
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&p)
        .map_err(|e| e.to_string())?;
    crate::debug_session::regular(&p)?;
    file.try_lock()
        .map_err(|_| "Another trigger operation is in progress")?;
    Ok(file)
}
fn read(root: &Path, name: &str) -> Result<Trigger, String> {
    let file = path(&directory(root, false)?, name)?;
    if !crate::debug_session::regular(&file)?.is_file() {
        return Err("Trigger must be an ordinary file".into());
    }
    let trigger: Trigger = serde_json::from_slice(&crate::diagnostics::read(&file, MAX_BYTES)?)
        .map_err(|e| format!("Invalid trigger JSON: {e}"))?;
    trigger.validate()?;
    if trigger.name != name {
        return Err("Trigger name differs from the requested name".into());
    }
    trigger.verify_session(root)?;
    Ok(trigger)
}
fn parse_action(command: &str) -> Result<Action, String> {
    match parse(command)? {
        Command::Control(action @ Action::Find { .. }) => Ok(action),
        _ => Err("Use one bounded break or watch command".into()),
    }
}
/// Recheck the enabled definition and its exact action on the executing side.
pub(crate) fn verify_run(
    root: &Path,
    settings: &crate::debug_probe::Settings,
) -> Result<(), String> {
    let identity = settings
        .trigger
        .as_ref()
        .ok_or("Missing named trigger identity")?;
    let stored = read(root, &identity.name)?;
    if !stored.enabled {
        return Err("Named trigger was disabled before dispatch".into());
    }
    let bytes = serde_json::to_vec(&stored).map_err(|e| e.to_string())?;
    if gba_core::ffta_rom::digest(&bytes) != identity.sha256 {
        return Err("Named trigger changed before dispatch".into());
    }
    let mut selected = settings.clone();
    selected.trigger = None;
    let expected = serde_json::to_vec(&stored.action).map_err(|e| e.to_string())?;
    let actual =
        serde_json::to_vec(&Action::Find { settings: selected }).map_err(|e| e.to_string())?;
    if expected != actual {
        return Err("Named trigger action differs from its stored definition".into());
    }
    Ok(())
}
fn save(root: &Path, name: &str, command: &str) -> Result<Trigger, String> {
    let action = parse_action(command)?;
    let (manifest_sha256, rom_sha256, runtime_sha256) = super::recipe::manifest(root)?;
    let trigger = Trigger {
        schema: 1,
        kind: "ffta-debug-trigger".into(),
        name: name.into(),
        manifest_sha256,
        rom_sha256,
        runtime_sha256,
        enabled: false,
        action,
    };
    trigger.validate()?;
    let dir = directory(root, true)?;
    let _lock = lock(&dir)?;
    let count = fs::read_dir(&dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| match e {
            Ok(e) if e.file_name() == ".trigger.lock" => None,
            other => Some(other),
        })
        .take(MAX_TRIGGERS)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
        .len();
    if count >= MAX_TRIGGERS {
        return Err("Trigger directory is full".into());
    }
    let bytes = serde_json::to_vec_pretty(&trigger).map_err(|e| e.to_string())?;
    let mut tmp = tempfile::NamedTempFile::new_in(&dir).map_err(|e| e.to_string())?;
    tmp.write_all(&bytes)
        .and_then(|_| tmp.as_file().sync_all())
        .map_err(|e| e.to_string())?;
    tmp.persist_noclobber(path(&dir, name)?)
        .map_err(|e| e.error.to_string())?;
    Ok(trigger)
}
fn edit(root: &Path, name: &str, command: &str) -> Result<Trigger, String> {
    let action = parse_action(command)?;
    let dir = directory(root, false)?;
    let _lock = lock(&dir)?;
    let mut trigger = read(root, name)?;
    trigger.action = action;
    // A changed condition never inherits prior authorization to run.
    trigger.enabled = false;
    trigger.validate()?;
    let bytes = serde_json::to_vec_pretty(&trigger).map_err(|e| e.to_string())?;
    let mut tmp = tempfile::NamedTempFile::new_in(&dir).map_err(|e| e.to_string())?;
    tmp.write_all(&bytes)
        .and_then(|_| tmp.as_file().sync_all())
        .map_err(|e| e.to_string())?;
    tmp.persist(path(&dir, name)?)
        .map_err(|e| e.error.to_string())?;
    Ok(trigger)
}
fn enable(root: &Path, name: &str, enabled: bool) -> Result<Trigger, String> {
    let dir = directory(root, false)?;
    let _lock = lock(&dir)?;
    let mut trigger = read(root, name)?;
    trigger.enabled = enabled;
    let bytes = serde_json::to_vec_pretty(&trigger).map_err(|e| e.to_string())?;
    let mut tmp = tempfile::NamedTempFile::new_in(&dir).map_err(|e| e.to_string())?;
    tmp.write_all(&bytes)
        .and_then(|_| tmp.as_file().sync_all())
        .map_err(|e| e.to_string())?;
    tmp.persist(path(&dir, name)?)
        .map_err(|e| e.error.to_string())?;
    Ok(trigger)
}
fn remove(root: &Path, name: &str) -> Result<Trigger, String> {
    let dir = directory(root, false)?;
    let _lock = lock(&dir)?;
    let trigger = read(root, name)?;
    fs::remove_file(path(&dir, name)?).map_err(|e| e.to_string())?;
    Ok(trigger)
}
fn list(root: &Path) -> Result<Vec<Trigger>, String> {
    let candidate = root.join("Data/config/debug-triggers");
    match fs::symlink_metadata(&candidate) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            super::recipe::manifest(root)?;
            return Ok(Vec::new());
        }
        Err(e) => return Err(e.to_string()),
        Ok(_) => (),
    }
    let dir = directory(root, false)?;
    let entries = fs::read_dir(&dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| match e {
            Ok(e) if e.file_name() == ".trigger.lock" => None,
            other => Some(other),
        })
        .take(MAX_TRIGGERS + 1)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    if entries.len() > MAX_TRIGGERS {
        return Err("Trigger directory exceeds 32 entries".into());
    }
    let mut triggers = Vec::new();
    for entry in entries {
        let filename = entry.file_name();
        let Some(filename) = filename.to_str() else {
            continue;
        };
        let Some(name) = filename
            .strip_prefix("trigger-")
            .and_then(|s| s.strip_suffix(".json"))
        else {
            continue;
        };
        triggers.push(read(root, name)?);
    }
    triggers.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(triggers)
}
fn execution_scope() -> Value {
    json!({"backend":"interpreter only; native block dispatch cannot stop inside a block",
        "pc_boundary":"before an eligible interpreter instruction",
        "watch_boundary":"after a complete interpreter step including HLE, DMA and hardware ticks; first matching access per step",
        "observer_conflicts":"Search requires an idle paused copy without recording, replay or another search. Watch requires the audit feature and exclusive bus observer.",
        "capture_boundary":"first completed frame after the hit; neutral input when a frame must be finished",
        "freeze":"Requires a running recent-event buffer; freezes it at the matched search stop"})
}
fn run(root: &Path, name: &str, retry: Option<(String, String, u64)>) -> Value {
    let trigger = match read(root, name) {
        Ok(v) => v,
        Err(e) => return super::local_error("invalid-trigger", &e),
    };
    if !trigger.enabled {
        return super::local_error(
            "disabled",
            "Enable the trigger explicitly before running it",
        );
    }
    let identity = match serde_json::to_vec(&trigger) {
        Ok(bytes) => gba_core::ffta_rom::digest(&bytes),
        Err(e) => return super::local_error("invalid-trigger", &e.to_string()),
    };
    let mut action = trigger.action.clone();
    if let Action::Find { settings } = &mut action {
        settings.trigger = Some(crate::debug_probe::TriggerIdentity {
            name: name.into(),
            sha256: identity.clone(),
        });
    }
    let result = match retry {
        Some((run, request, deadline)) => crate::debug_control::retry_action_classified(
            root, action, 120_000, run, request, deadline,
        ),
        None => crate::debug_control::execute_action_classified(root, action, 120_000),
    };
    let response = match result {
        Ok(v) => v,
        Err(e) => e.value("ffta-debug-command"),
    };
    let outcome = if response["ok"] == true {
        response
            .pointer("/status/probe/outcome")
            .and_then(Value::as_str)
            .filter(|s| matches!(*s, "hit" | "not-reached"))
            .unwrap_or("unconfirmed")
    } else {
        response["outcome"].as_str().unwrap_or("unconfirmed")
    };
    let ok = response["ok"] == true && matches!(outcome, "hit" | "not-reached");
    let error = if ok {
        Value::Null
    } else {
        response
            .get("error")
            .filter(|v| !v.is_null())
            .cloned()
            .unwrap_or_else(|| json!("No confirmed search result"))
    };
    json!({"schema":1,"kind":"ffta-debug-trigger-result","ok":ok,"outcome":outcome,"error":error,"delivery":response["delivery"],"name":name,"trigger_sha256":identity,"definition":trigger,"response":response,"execution_scope":execution_scope(),"scope":"Explicit one-shot bounded interpreter search; enablement never starts execution"})
}
pub(super) fn retry(
    root: &Path,
    name: &str,
    run_id: String,
    request_id: String,
    deadline: u64,
) -> Value {
    run(root, name, Some((run_id, request_id, deadline)))
}
pub(super) fn execute(
    root: &Path,
    verb: &str,
    name: Option<&str>,
    argument: Option<&str>,
) -> Value {
    if verb == "run" {
        return run(root, name.unwrap_or_default(), None);
    }
    let result = match verb {
        "save" => save(root, name.unwrap_or_default(), argument.unwrap_or_default())
            .and_then(|v| serde_json::to_value(v).map_err(|e| e.to_string())),
        "edit" => edit(root, name.unwrap_or_default(), argument.unwrap_or_default())
            .and_then(|v| serde_json::to_value(v).map_err(|e| e.to_string())),
        "show" => read(root, name.unwrap_or_default())
            .and_then(|v| serde_json::to_value(v).map_err(|e| e.to_string())),
        "enable" | "disable" => enable(root, name.unwrap_or_default(), verb == "enable")
            .and_then(|v| serde_json::to_value(v).map_err(|e| e.to_string())),
        "remove" => remove(root, name.unwrap_or_default())
            .and_then(|v| serde_json::to_value(v).map_err(|e| e.to_string())),
        "list" => list(root).and_then(|v| serde_json::to_value(v).map_err(|e| e.to_string())),
        _ => Err("Unknown trigger command".into()),
    };
    match result {
        Ok(definition) => {
            json!({"schema":1,"kind":"ffta-debug-trigger","ok":true,"outcome":verb,"definition":definition,"execution_scope":execution_scope(),"scope":"Stored definitions are inert; run is always explicit"})
        }
        Err(e) => super::local_error("invalid-trigger", &e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        thread,
        time::{Duration, Instant},
    };

    fn fixture() -> (tempfile::TempDir, crate::debug_control::Server) {
        let t = tempfile::tempdir().unwrap();
        let root = t.path();
        fs::create_dir_all(root.join("Data/config")).unwrap();
        let exe = if cfg!(windows) {
            "recomp.exe"
        } else {
            "recomp"
        };
        for (name, bytes) in [
            ("portable.txt", b"portable".as_slice()),
            ("ffta.gba", b"PUBLIC ROM".as_slice()),
            (exe, b"PUBLIC executable".as_slice()),
        ] {
            fs::write(root.join(name), bytes).unwrap();
        }
        let hash = |name: &str| gba_core::ffta_rom::digest(&fs::read(root.join(name)).unwrap());
        let pins = json!({"portable.txt":hash("portable.txt"),"ffta.gba":hash("ffta.gba"),(exe):hash(exe)});
        fs::write(root.join("debug-session.json"), serde_json::to_vec(&json!({"schema":1,"kind":"ffta-debug-session","label":"Synthetic","interp":true,"real_bios":false,"has_state":false,"has_dataset":false,"pins":pins})).unwrap()).unwrap();
        let server = crate::debug_control::Server::new(root).unwrap();
        (t, server)
    }
    #[test]
    fn stored_definitions_are_inert_pinned_bounded_and_refuse_drift() {
        let (t, mut server) = fixture();
        let root = t.path();
        assert_eq!(
            super::super::execute(root, "trigger list")["definition"],
            json!([])
        );
        let saved =
            super::super::execute(root, "trigger save first 'break 0x08000000 arm --steps 1'");
        assert_eq!(saved["definition"]["enabled"], false);
        assert_eq!(
            super::super::execute(root, "trigger run first")["outcome"],
            "disabled"
        );
        assert!(server.poll().unwrap().is_none());
        assert_eq!(
            super::super::execute(root, "trigger enable first")["definition"]["enabled"],
            true
        );
        assert!(server.poll().unwrap().is_none());
        assert_eq!(
            super::super::execute(root, "trigger edit first 'step 1'")["ok"],
            false
        );
        assert_eq!(
            super::super::execute(root, "trigger show first")["definition"]["enabled"],
            true
        );
        let edited =
            super::super::execute(root, "trigger edit first 'break 0x08000004 arm --steps 2'");
        assert_eq!(edited["definition"]["enabled"], false);
        assert_eq!(edited["definition"]["action"]["settings"]["steps"], 2);
        assert!(server.poll().unwrap().is_none());
        assert_eq!(
            super::super::execute(root, "trigger save first 'break 0x08000004 arm --steps 1'")
                ["ok"],
            false
        );
        assert_eq!(
            super::super::execute(root, "trigger save bad 'step 1'")["ok"],
            false
        );
        assert_eq!(
            super::super::execute(root, "trigger save ../bad 'break 0x08000000 arm'")["ok"],
            false
        );
        let file = root.join("debug-session.json");
        let mut manifest: Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
        manifest["label"] = json!("Changed");
        fs::write(&file, serde_json::to_vec(&manifest).unwrap()).unwrap();
        assert_eq!(
            super::super::execute(root, "trigger run first")["outcome"],
            "invalid-trigger"
        );
        assert!(server.poll().unwrap().is_none());
    }
    #[test]
    fn executing_side_authenticates_name_enabled_state_and_exact_action() {
        let (t, mut server) = fixture();
        let root = t.path();
        save(root, "bound", "break 0x08000000 arm --steps 2").unwrap();
        let selected = enable(root, "bound", true).unwrap();
        let identity = gba_core::ffta_rom::digest(&serde_json::to_vec(&selected).unwrap());
        let Action::Find { mut settings } = selected.action else {
            panic!()
        };
        settings.trigger = Some(crate::debug_probe::TriggerIdentity {
            name: "bound".into(),
            sha256: identity,
        });
        assert!(verify_run(root, &settings).is_ok());
        let mut forged = settings.clone();
        forged.steps += 1;
        assert!(verify_run(root, &forged).unwrap_err().contains("differs"));
        edit(root, "bound", "break 0x08000004 arm --steps 3").unwrap();
        assert!(verify_run(root, &settings).is_err());
        enable(root, "bound", true).unwrap();
        assert!(verify_run(root, &settings).unwrap_err().contains("changed"));
        assert!(server.poll().unwrap().is_none());
    }
    #[test]
    fn retry_joins_the_original_named_request_without_reexecution() {
        use std::sync::{
            atomic::{AtomicBool, AtomicUsize, Ordering},
            Arc,
        };
        let (t, mut server) = fixture();
        let root = t.path();
        save(root, "once", "break 0x08000000 arm --steps 1").unwrap();
        enable(root, "once", true).unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let count = Arc::new(AtomicUsize::new(0));
        let done = stop.clone();
        let attempts = count.clone();
        let worker = thread::spawn(move || {
            while !done.load(Ordering::SeqCst) {
                if let Some(request) = server.poll().unwrap() {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    server
                        .reply(
                            &request,
                            json!({"paused":true,"probe":{"outcome":"hit","boundary":"synthetic"}}),
                            None,
                        )
                        .unwrap();
                }
                thread::sleep(Duration::from_millis(2));
            }
        });
        let first = super::super::execute(root, "trigger run once");
        assert_eq!(first["outcome"], "hit", "{first}");
        let response = &first["response"];
        let command = format!(
            "trigger retry once {} {} {}",
            response["run"].as_str().unwrap(),
            response["id"].as_str().unwrap(),
            response["deadline_unix_ms"].as_u64().unwrap()
        );
        let again = super::super::execute(root, &command);
        assert_eq!(again["response"], *response);
        assert_eq!(count.load(Ordering::SeqCst), 1);
        stop.store(true, Ordering::SeqCst);
        worker.join().unwrap();
    }
    #[test]
    fn explicit_runs_return_named_hit_and_miss_with_the_original_server_results() {
        let (t, mut server) = fixture();
        let root = t.path();
        for (name, pc) in [("hit", "0x08000000"), ("miss", "0x08000004")] {
            assert_eq!(
                super::super::execute(
                    root,
                    &format!("trigger save {name} 'break {pc} arm --steps 1'")
                )["ok"],
                true
            );
            assert_eq!(
                super::super::execute(root, &format!("trigger enable {name}"))["ok"],
                true
            );
        }
        let worker = thread::spawn(move || {
            for outcome in ["hit", "not-reached"] {
                let until = Instant::now() + Duration::from_secs(15);
                let request = loop {
                    if let Some(r) = server.poll().unwrap() {
                        break r;
                    }
                    assert!(Instant::now() < until);
                    thread::sleep(Duration::from_millis(2));
                };
                assert!(matches!(request.command, Action::Find { .. }));
                server
                    .reply(
                        &request,
                        json!({"paused":true,"probe":{"outcome":outcome,"boundary":"synthetic"}}),
                        None,
                    )
                    .unwrap();
            }
        });
        for (name, outcome) in [("hit", "hit"), ("miss", "not-reached")] {
            let value = super::super::execute(root, &format!("trigger run {name}"));
            assert_eq!(value["ok"], true, "{value}");
            assert_eq!(value["name"], name);
            assert_eq!(value["outcome"], outcome);
            assert_eq!(value["response"]["status"]["probe"]["outcome"], outcome);
            assert_eq!(value["trigger_sha256"].as_str().unwrap().len(), 64);
            assert!(value["response"]["id"].as_str().is_some());
        }
        worker.join().unwrap();
        assert_eq!(
            super::super::execute(root, "trigger disable hit")["definition"]["enabled"],
            false
        );
        assert_eq!(
            super::super::execute(root, "trigger run hit")["outcome"],
            "disabled"
        );
        assert_eq!(
            super::super::execute(root, "trigger remove hit")["definition"]["name"],
            "hit"
        );
        assert_eq!(super::super::execute(root, "trigger show hit")["ok"], false);
        assert_eq!(
            super::super::execute(root, "trigger list")["definition"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
    }
}
