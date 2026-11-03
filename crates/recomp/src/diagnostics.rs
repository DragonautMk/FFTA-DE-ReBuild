//! Shared, opt-in diagnosis. Collection never executes guest code or writes inputs.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{fs::File, io::{Read, Write}, path::{Path, PathBuf}};

const MAX_INPUT: u64 = 16 * 1024 * 1024;
const MAX_REPORT: u64 = 512 * 1024;

#[derive(Default)]
pub(crate) struct Request {
    pub rom: PathBuf,
    pub bios: Option<PathBuf>,
    pub no_bios: bool,
    pub save: Option<PathBuf>,
    pub state: Option<PathBuf>,
    pub slot: Option<u8>,
    pub input: Option<PathBuf>,
    pub recording: bool,
    pub dataset: Option<PathBuf>,
    pub managed: bool,
    pub interp: bool,
    pub require_audit: bool,
    pub out: Option<PathBuf>,
}

pub(crate) fn default_report_path() -> Result<PathBuf, String> {
    Ok(input_config::config_root().ok_or("Cannot locate configuration storage")?
        .join("diagnostics").join("session.json"))
}

fn absolute(path: &Path) -> PathBuf {
    std::path::absolute(path).unwrap_or_else(|_| path.to_owned())
}

pub(crate) fn read(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let mut file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let meta = file.metadata().map_err(|e| e.to_string())?;
    if !meta.is_file() || meta.len() > limit {
        return Err(format!("{}: expected a regular file of at most {limit} bytes", path.display()));
    }
    let mut bytes = Vec::new();
    Read::by_ref(&mut file).take(limit + 1).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit { return Err("File grew beyond its size limit".into()); }
    Ok(bytes)
}

fn identity(path: &Path, optional: bool) -> Result<Value, String> {
    if optional && !path.exists() {
        return Ok(json!({"path": absolute(path), "exists": false}));
    }
    let mut file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let meta = file.metadata().map_err(|e| e.to_string())?;
    if !meta.is_file() || meta.len() > 512 * 1024 * 1024 {
        return Err(format!("{}: not a bounded regular file", path.display()));
    }
    let mut hash = Sha256::new();
    let mut buffer = [0; 65536];
    let mut length = 0u64;
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 { break; }
        length += n as u64;
        if length > 512 * 1024 * 1024 { return Err("File grew beyond its size limit".into()); }
        hash.update(&buffer[..n]);
    }
    Ok(json!({"path": absolute(path), "exists": true, "bytes": length,
        "sha256": hash.finalize().iter().map(|b| format!("{b:02x}")).collect::<String>()}))
}

fn issue(issues: &mut Vec<Value>, code: &str, message: String, remedy: &str) {
    issues.push(json!({"severity": "error", "code": code, "message": message, "remedy": remedy}));
}

fn checked<T>(issues: &mut Vec<Value>, code: &str, result: Result<T, String>, remedy: &str) -> Option<T> {
    match result { Ok(value) => Some(value), Err(error) => { issue(issues, code, error, remedy); None } }
}

pub(crate) fn collect(request: &Request, live: Option<&gba_core::Machine>) -> Value {
    let mut issues = Vec::new();
    let rom_file = checked(&mut issues, "rom-file", identity(&request.rom, false), "Select an existing GBA ROM file.");
    let rom = match live {
        Some(machine) => Some(machine.bus.rom.clone()),
        None => checked(&mut issues, "rom", read(&request.rom, 32 << 20)
            .and_then(|rom| { crate::validate_rom(&request.rom.to_string_lossy(), &rom)?; Ok(rom) }),
            "Select a GBA cartridge image, not a save, archive or report."),
    };
    let rom_hash = rom.as_deref().map(gba_core::ffta_rom::digest);
    if live.is_some() && rom_file.as_ref().is_some_and(|f| f["sha256"].as_str() != rom_hash.as_deref()) {
        issue(&mut issues, "rom-changed", "ROM file differs from the running image".into(), "Restore the original ROM file before reproducing this session.");
    }
    let profile = match rom_hash.as_deref() {
        Some(gba_core::ffta_rom::EUROPE_SHA256) => "EUR",
        Some(gba_core::ffta_rom::SHA256) => "USA",
        _ => "unrecognized",
    };
    let bios_path = if request.no_bios { None } else { request.bios.clone().or_else(input_config::find_bios) };
    let bios_bytes = if let Some(machine) = live { Some(machine.bus.bios.clone()) }
        else if let Some(path) = &bios_path {
            checked(&mut issues, "bios", read(path, input_config::BIOS_SIZE as u64).and_then(|b| {
                if b.len() != input_config::BIOS_SIZE { Err("BIOS must contain exactly 16384 bytes".into()) } else { Ok(b) }
            }), "Select a 16384-byte BIOS image with --bios, or explicitly choose --no-bios.")
        } else { None };
    let real_bios = live.map_or(bios_bytes.is_some(), |m| m.bus.real_bios);
    let bios_file = bios_path.as_ref().and_then(|p| checked(&mut issues, "bios-file", identity(p, false), "Select an existing BIOS file."));
    let bios_hash = if real_bios { bios_bytes.as_deref().map(gba_core::ffta_rom::digest) }
        else { Some(gba_core::ffta_rom::digest(&vec![0; input_config::BIOS_SIZE])) };
    if live.is_some() && real_bios && bios_file.as_ref().is_some_and(|f| f["sha256"].as_str() != bios_hash.as_deref()) {
        issue(&mut issues, "bios-changed", "BIOS file differs from the running image".into(),
            "Restore the BIOS used to start this session before reproducing it.");
    }
    let bios = json!({"mode": if real_bios { "real" } else { "hle" },
        "file": bios_file, "effective_sha256": bios_hash});
    let save_path = request.save.clone().or_else(|| checked(&mut issues, "save-selection",
        crate::save_path::inspect(&request.rom), "Select the intended cartridge save with --sav FILE."));
    let save = save_path.as_ref().and_then(|p| checked(&mut issues, "save-file", identity(p, request.save.is_none() || live.is_some()),
        "Select an existing save with --sav FILE; omit it for normal new-game selection."));
    let mut state_path = request.state.clone();
    if let (Some(slot), Some(save), Some(rom)) = (request.slot, &save_path, &rom) {
        let mut slots = crate::save_state::Slots::new(save, rom);
        slots.set_extended(true);
        state_path = checked(&mut issues, "state-slot", slots.path(slot), "Choose a snapshot slot between 1 and 99.");
    }
    let state = state_path.as_ref().and_then(|p| checked(&mut issues, "snapshot-file", identity(p, live.is_some()),
        "Select an existing snapshot or an occupied slot for this ROM and save directory."));
    // Preflight decodes into a disposable machine, never into a running session.
    if live.is_none() && state_path.is_some() {
        if let Some(rom) = &rom {
            let mut scratch = crate::make_machine(rom.clone(), bios_bytes.as_deref());
            checked(&mut issues, "snapshot", crate::save_state::load(&mut scratch, state_path.as_ref().unwrap()),
                "Use a compatible snapshot with the same ROM, BIOS and state format.");
        }
    }
    let input = request.input.as_ref().and_then(|p| checked(&mut issues, "input-file", identity(p, false), "Select an existing input file."));
    if let Some(path) = &request.input {
        // A live recording can be between writes: identify it, but do not claim a validated replay.
        if !request.recording {
            checked(&mut issues, "input-script", read(path, MAX_INPUT).and_then(|bytes| {
                let text = std::str::from_utf8(&bytes).map_err(|e| e.to_string())?;
                let script = crate::InputScript::parse(text, &path.to_string_lossy())?;
                if script.0.iter().any(|&(_, keys)| keys > 0x3ff) { return Err("Input key mask exceeds 10 GBA buttons".into()); }
                Ok(())
            }), "Use a UTF-8 gba-input v1 recording with valid frame numbers and key masks.");
        }
    }
    let dataset_path = request.dataset.clone().or_else(|| {
        if request.managed { rom.as_deref().and_then(|rom| checked(&mut issues, "dataset-selection",
            crate::rom_data::diagnosis::managed_directory(rom), "Run recomp rom-data prepare <rom>, then repeat diagnosis.")) }
        else { None }
    });
    let dataset = match (dataset_path.as_ref(), rom.as_deref()) {
        (Some(path), Some(rom)) => checked(&mut issues, "dataset", crate::rom_data::diagnosis::inspect(rom, path),
            "Select data prepared by this binary for the same ROM, or run recomp rom-data prepare <rom>.").unwrap_or(json!({"directory": absolute(path), "validation": "failed"})),
        _ => json!({"enabled": false}),
    };
    let active_dataset = live.and_then(|m| m.bus.rom_resources()).map(|r| json!({
        "source_sha256": r.source_sha256(), "dataset_sha256": r.dataset_sha256()}));
    if let Some(active) = &active_dataset {
        if dataset["dataset_sha256"] != active["dataset_sha256"] {
            issue(&mut issues, "dataset-changed", "On-disk dataset differs from the attached live resources".into(),
                "Keep the original generation and restart diagnosis with its directory.");
        }
    }
    let audit = cfg!(feature = "rom-data-read-audit");
    if request.require_audit && !audit {
        issue(&mut issues, "feature-rom-audit", "ROM read audit is not compiled into this binary".into(),
            "Choose a binary built with --features rom-data-read-audit; this does not itself start an audit.");
    }
    let executable = std::env::current_exe().ok().and_then(|p| checked(&mut issues, "executable", identity(&p, false), "Select a readable runtime executable."));
    let config_file = |p: Option<PathBuf>| p.and_then(|p| identity(&p, true).ok());
    let out = request.out.as_ref().map(|p| absolute(p));
    json!({
        "schema": 1, "kind": "ffta-session-diagnosis", "ok": issues.is_empty(),
        "context": if live.is_some() { "live-paused" } else { "preflight-no-execution" },
        "profile": profile,
        "build": {"version": env!("CARGO_PKG_VERSION"), "revision": env!("FFTA_BUILD_REVISION"),
            "dirty_at_build": env!("FFTA_BUILD_DIRTY"), "target": env!("FFTA_BUILD_TARGET"),
            "profile": env!("FFTA_BUILD_PROFILE"), "executable": executable,
            "features": {"rom_data_read_audit": audit}},
        "execution": {"mode": if request.interp { "interpreter" } else if live.is_some() { "native-with-fallback" } else { "native-requested-not-loaded" },
            "frame": live.map(|m| m.bus.frames), "clock": live.map(|m| m.bus.clock),
            "native_availability_checked": live.is_some()},
        "rom": {"file": rom_file, "effective_sha256": rom_hash}, "bios": bios,
        "cartridge_save": save, "game_save_slot": save_path.as_ref().map(|p| crate::game_saves::GameSaves::new(p).current),
        "snapshot": {"file": state, "selected_slot": request.slot,
            "meaning": if live.is_some() { "picker-selection-not-live-state" } else { "validated-start-point" }},
        "input": {"file": input, "recording_in_progress": request.recording,
            "validation": if request.recording { "not-validated-active-recording" } else { "parser-only-no-reachability-claim" }},
        "dataset": dataset, "active_dataset": active_dataset,
        "storage": {"portable_root": input_config::storage::portable_root(), "config_root": input_config::config_root(),
            "input_config": config_file(input_config::default_path()), "av_config": config_file(input_config::av_path()),
            "resources_root": crate::rom_data::diagnosis::storage_root().ok(),
            "report": out, "report_directory": out.as_ref().and_then(|p| p.parent())},
        "issues": issues,
        "limits": ["No guest execution, native translation or performance measurement.",
            "Input parsing does not prove scene reachability or snapshot-relative frame origin.",
            "Contains local paths and file identities; review before sharing."]
    })
}

pub(crate) fn write_report(path: &Path, report: &Value) -> Result<(), String> {
    // Only replace our own bounded report. Never clobber an arbitrary selected file.
    if std::fs::symlink_metadata(path).is_ok() {
        let meta = std::fs::symlink_metadata(path).map_err(|e| e.to_string())?;
        if !meta.file_type().is_file() { return Err("Report output must not be a link or directory".into()); }
        let previous: Value = serde_json::from_slice(&read(path, MAX_REPORT)?).map_err(|_| "Output already exists and is not a diagnosis report")?;
        if previous["kind"] != "ffta-session-diagnosis" || previous["schema"] != 1 {
            return Err("Refusing to overwrite a file that is not a diagnosis report".into());
        }
    }
    let bytes = serde_json::to_vec_pretty(report).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_REPORT { return Err("Diagnosis report exceeds 512 KiB".into()); }
    let absolute = absolute(path);
    let parent = absolute.parent().ok_or("Report path needs a parent directory")?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    file.write_all(&bytes).map_err(|e| e.to_string())?;
    file.as_file().sync_all().map_err(|e| e.to_string())?;
    file.persist(path).map_err(|e| e.error.to_string())?;
    Ok(())
}

pub(crate) fn cmd(args: &[String]) -> Result<(), String> {
    let (request, json_output) = parse(args)?;
    let report = collect(&request, None);
    if let Some(path) = &request.out { write_report(path, &report)?; }
    if json_output { println!("{}", serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?); }
    else {
        println!("Diagnosis: {} | profile={} | mode={}", if report["ok"] == true { "OK" } else { "FAILED" }, report["profile"], report["execution"]["mode"]);
        if let Some(path) = &request.out { println!("Report: {}", absolute(path).display()); }
        for i in report["issues"].as_array().unwrap() { println!("{}: {}\n  {}", i["code"], i["message"], i["remedy"]); }
        if request.out.is_none() { println!("Use --json for identities and paths, or --out FILE to retain a report."); }
    }
    if report["ok"] == true { Ok(()) } else { Err("Session preflight failed; see diagnosis issues and remedies".into()) }
}

fn parse(args: &[String]) -> Result<(Request, bool), String> {
    let mut r = Request::default();
    let mut json = false;
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--json" => json = true,
            "--interp" => r.interp = true,
            "--no-bios" => r.no_bios = true,
            "--rom-data" => r.managed = true,
            "--require-rom-audit" => r.require_audit = true,
            "--bios" | "--sav" | "--state" | "--input" | "--dataset" | "--out" => {
                let value = PathBuf::from(it.next().ok_or_else(|| format!("{arg} needs a path"))?);
                match arg.as_str() {
                    "--bios" => r.bios = Some(value), "--sav" => r.save = Some(value),
                    "--state" => r.state = Some(value), "--input" => r.input = Some(value),
                    "--dataset" => r.dataset = Some(value), "--out" => r.out = Some(value), _ => unreachable!(),
                }
            }
            "--slot" => {
                let slot = it.next().ok_or("--slot needs a number")?.parse::<u8>().map_err(|_| "--slot must be 1..99")?;
                if !(1..=99).contains(&slot) { return Err("--slot must be 1..99".into()); }
                r.slot = Some(slot);
            }
            other if other.starts_with('-') => return Err(format!("Unknown diagnosis option {other}")),
            _ if r.rom.as_os_str().is_empty() => r.rom = PathBuf::from(arg),
            _ => return Err(format!("Unexpected diagnosis argument {arg}")),
        }
    }
    if r.rom.as_os_str().is_empty() { return Err("diagnose needs a ROM path".into()); }
    if r.no_bios && r.bios.is_some() { return Err("--bios and --no-bios are mutually exclusive".into()); }
    if r.slot.is_some() && r.state.is_some() { return Err("--slot and --state are mutually exclusive".into()); }
    if r.managed && r.dataset.is_some() { return Err("--rom-data and --dataset are mutually exclusive".into()); }
    Ok((r, json))
}

#[cfg(test)]
mod tests;
