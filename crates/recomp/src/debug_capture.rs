//! Bounded first-hit captures on isolated replay workers. No scene promotion.
use gba_core::Machine;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum Condition {
    Resource {
        id: String,
        preset: String,
        manifest_sha256: String,
    },
    Instruction {
        pc: u32,
        thumb: bool,
        opcode: u32,
    },
    Frame {
        completed: u64,
    },
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Settings {
    pub condition: Condition,
    pub limit_frames: u64,
}
pub(crate) struct Request {
    pub settings: Settings,
    pub manifest: Option<PathBuf>,
    pub dataset: Option<PathBuf>,
}
fn hash(bytes: &[u8]) -> String {
    gba_core::ffta_rom::digest(bytes)
}
pub(crate) fn validate(s: &Settings, frames: u64) -> Result<(), String> {
    if s.limit_frames == 0 || s.limit_frames > frames {
        return Err("Capture frame limit must be 1..recorded frames".into());
    }
    match &s.condition {
        Condition::Resource {
            id,
            preset,
            manifest_sha256,
        } if id.is_empty()
            || id.len() > 128
            || id.chars().any(char::is_control)
            || !["all", "bus", "host"].contains(&preset.as_str())
            || manifest_sha256.len() != 64
            || !manifest_sha256.bytes().all(|b| b.is_ascii_hexdigit()) =>
        {
            return Err("Invalid resource capture condition".into())
        }
        Condition::Instruction { pc, thumb, opcode }
            if !(0x08000000..0x0e000000).contains(pc)
                || pc % if *thumb { 2 } else { 4 } != 0
                || *thumb && *opcode > 0xffff =>
        {
            return Err("Invalid aligned ROM PC/mode/opcode".into())
        }
        Condition::Frame { completed } if *completed == 0 || *completed > s.limit_frames => {
            return Err("Frame event must be 1..capture frame limit".into())
        }
        _ => {}
    }
    Ok(())
}
fn source_ranges(bytes: &[u8], rom: &[u8], id: &str) -> Result<Vec<(u32, u32)>, String> {
    let m: gba_core::rom_data::Manifest =
        toml::from_str(std::str::from_utf8(bytes).map_err(|e| e.to_string())?)
            .map_err(|e| format!("Capture manifest: {e}"))?;
    if m.schema != 1
        || m.source_rom_sha256 != hash(rom)
        || m.source_rom_size as usize != rom.len()
        || m.resources.len() > 512
    {
        return Err("Capture manifest ROM/schema mismatch".into());
    }
    let mut ids = std::collections::BTreeSet::new();
    if m.resources.iter().any(|r| !ids.insert(&r.id)) {
        return Err("Duplicate capture resource ID".into());
    }
    let r = m
        .resources
        .iter()
        .find(|r| r.id == id)
        .ok_or("Resource absent from selected manifest")?;
    if r.sources.is_empty() || r.sources.len() > 100000 {
        return Err("Capture resource needs bounded source ranges".into());
    }
    if r.sources.iter().map(|s| u64::from(s.length)).sum::<u64>() > 128 * 1024 * 1024 {
        return Err("Capture source authentication exceeds 128 MiB".into());
    }
    let mut ranges = Vec::new();
    for s in &r.sources {
        let end = s
            .offset
            .checked_add(s.length)
            .filter(|&e| s.length > 0 && e as usize <= rom.len())
            .ok_or("Capture source out of ROM")?;
        if hash(&rom[s.offset as usize..end as usize]) != s.sha256 {
            return Err("Capture source hash differs from ROM".into());
        }
        ranges.push((s.offset, end));
    }
    ranges.sort_unstable();
    let mut merged: Vec<(u32, u32)> = Vec::new();
    for (a, b) in ranges {
        if let Some(last) = merged.last_mut().filter(|v| a <= v.1) {
            last.1 = last.1.max(b);
        } else {
            merged.push((a, b));
        }
    }
    Ok(merged)
}
fn instruction(rom: &[u8], pc: u32, thumb: bool, opcode: u32) -> Result<(), String> {
    let offset = (pc & 0x01ffffff) as usize;
    let width = if thumb { 2 } else { 4 };
    let actual = rom
        .get(offset..offset + width)
        .ok_or("Capture PC outside ROM image")?;
    if actual != &opcode.to_le_bytes()[..width] {
        return Err("Capture PC opcode differs from selected ROM".into());
    }
    let decoded = if thumb {
        armv4t::decode_thumb(opcode as u16, pc)
    } else {
        armv4t::decode_arm(opcode, pc)
    };
    if matches!(decoded.op, armv4t::Op::Undefined { .. }) {
        return Err("Capture PC does not decode as a supported instruction".into());
    }
    Ok(())
}
pub(crate) fn prepare(
    request: &Request,
    rom: &[u8],
    frames: u64,
    dataset: Option<&Path>,
) -> Result<(Settings, Option<Vec<u8>>), String> {
    if !cfg!(feature = "rom-data-read-audit") {
        return Err("Capture triggers not compiled; use --features rom-data-read-audit".into());
    }
    let mut s = request.settings.clone();
    if s.limit_frames == u64::MAX {
        s.limit_frames = frames;
    }
    let bytes = if let Condition::Resource {
        id,
        manifest_sha256,
        ..
    } = &mut s.condition
    {
        let path = request
            .manifest
            .clone()
            .or_else(|| dataset.map(|d| d.join("manifest.toml")))
            .ok_or("Resource trigger needs --manifest or --dataset")?;
        let bytes = crate::diagnostics::read(&path, gba_core::rom_data::MAX_MANIFEST_FILE_BYTES)?;
        source_ranges(&bytes, rom, id)?;
        *manifest_sha256 = hash(&bytes);
        Some(bytes)
    } else {
        None
    };
    validate(&s, frames)?;
    if let Condition::Instruction { pc, thumb, opcode } = &s.condition {
        instruction(rom, *pc, *thumb, *opcode)?;
    }
    Ok((s, bytes))
}

pub(crate) struct Observer {
    pub settings: Settings,
    pub hit: Option<Value>,
    #[cfg(feature = "rom-data-read-audit")]
    capture: Option<gba_core::rom_data::read_audit::TriggerCapture>,
}
impl Observer {
    pub fn start(root: &Path, s: &Settings, rom: &[u8]) -> Result<Self, String> {
        #[cfg(feature = "rom-data-read-audit")]
        {
            use gba_core::rom_data::read_audit::{
                ExecutionSite, Reader, TriggerCapture, TriggerCondition,
            };
            let condition = match &s.condition {
                Condition::Frame { .. } => None,
                Condition::Instruction { pc, thumb, opcode } => {
                    instruction(rom, *pc, *thumb, *opcode)?;
                    Some(TriggerCondition::Instruction(ExecutionSite {
                        pc: *pc,
                        thumb: *thumb,
                    }))
                }
                Condition::Resource {
                    id,
                    preset,
                    manifest_sha256,
                } => {
                    let bytes = crate::diagnostics::read(
                        &root.join("capture-plan.toml"),
                        gba_core::rom_data::MAX_MANIFEST_FILE_BYTES,
                    )?;
                    if hash(&bytes) != *manifest_sha256 {
                        return Err("Capture manifest changed".into());
                    }
                    let readers = match preset.as_str() {
                        "bus" => vec![Reader::Bus],
                        "host" => vec![
                            Reader::AudioValue,
                            Reader::SpritePeek,
                            Reader::SpriteList,
                            Reader::ObjSourceTable,
                            Reader::KeyboardWidths,
                        ],
                        _ => Vec::new(),
                    };
                    Some(TriggerCondition::Read {
                        ranges: source_ranges(&bytes, rom, id)?,
                        readers,
                    })
                }
            };
            Ok(Self {
                settings: s.clone(),
                hit: None,
                capture: condition.map(TriggerCapture::start).transpose()?,
            })
        }
        #[cfg(not(feature = "rom-data-read-audit"))]
        {
            let _ = (root, s, rom);
            Err("Capture triggers not compiled in selected worker".into())
        }
    }
    pub fn frame(&self, guest: u64) {
        #[cfg(feature = "rom-data-read-audit")]
        if let Some(c) = &self.capture {
            c.frame(Some(guest));
        }
        #[cfg(not(feature = "rom-data-read-audit"))]
        let _ = guest;
    }
    pub fn reached(&mut self, completed: u64, guest: u64) -> bool {
        if completed == 0 {
            return false;
        }
        if let Condition::Frame { completed: target } = &self.settings.condition {
            if completed == *target {
                self.hit = Some(
                    json!({"kind":"completed-frame","guest_frame":guest,"relative_completed":completed}),
                );
            }
        }
        #[cfg(feature = "rom-data-read-audit")]
        if let Some(hit) = self.capture.as_ref().and_then(|c| c.hit()) {
            self.hit = Some(serde_json::to_value(hit).expect("trigger metadata"));
        }
        self.hit.is_some()
    }
}

pub(crate) fn capacity(root: &Path) -> Result<(), String> {
    let parent = root.join("captures");
    match fs::symlink_metadata(&parent) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.to_string()),
        Ok(meta) if !meta.is_dir() || meta.file_type().is_symlink() => {
            return Err("Capture directory must be a real local directory".into())
        }
        Ok(_) => {}
    }
    let entries = fs::read_dir(parent)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    if entries.len() >= 8 {
        return Err(
            "Eight capture bundles retained; choose a new profile or archive bundles explicitly"
                .into(),
        );
    }
    Ok(())
}
pub(crate) struct Inputs<'a> {
    pub state: &'a [u8],
    pub input: &'a [u8],
}

/// Publish a self-contained derived point. Retention is bounded; never overwrite
/// a captured point or silently promote the source case's validation status.
pub(crate) fn publish(
    root: &Path,
    run: &str,
    case: &str,
    m: &Machine,
    observer: &Observer,
    origin: Value,
    inputs: Inputs<'_>,
) -> Result<String, String> {
    if run.is_empty()
        || run.len() > 64
        || !run.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err("Invalid capture run identifier".into());
    }
    if observer.hit.is_none() {
        return Err("Cannot publish a capture without a hit".into());
    }
    if origin["start_sha256"].as_str() != Some(hash(inputs.state).as_str())
        || origin["input_sha256"].as_str() != Some(hash(inputs.input).as_str())
    {
        return Err("Capture origin/input fingerprints differ from executed inputs".into());
    }
    capacity(root)?;
    let parent = root.join("captures");
    fs::create_dir_all(&parent).map_err(|e| e.to_string())?;
    let destination = parent.join(run);
    if fs::symlink_metadata(&destination).is_ok() {
        return Err("Capture run destination already exists".into());
    }
    let staging = tempfile::Builder::new()
        .prefix(".capture-")
        .tempdir_in(&parent)
        .map_err(|e| e.to_string())?;
    let state = m.save_state()?;
    for (name, bytes) in [
        ("capture.ffstate", state.as_slice()),
        ("origin.ffstate", inputs.state),
        ("input.input", inputs.input),
    ] {
        fs::write(staging.path().join(name), bytes).map_err(|e| e.to_string())?;
    }
    let region = match gba_core::ffta_rom::version(&m.bus.rom) {
        Some(gba_core::ffta_rom::Version::Europe) => "EUR",
        Some(gba_core::ffta_rom::Version::Usa) => "USA",
        None => "",
    };
    let manifest = json!({"schema":1,"kind":"ffta-debug-capture","run":run,"case":case,"status":"derived-unvalidated",
        "origin":origin,"condition":observer.settings,"hit":observer.hit,"boundary":{"kind":"completed-frame","guest_frame":m.bus.frames,"clock":m.bus.clock},
        "state_sha256":hash(&state),"input":"input.input","origin_state":"origin.ffstate","snapshot":"capture.ffstate",
        "limits":["First instrumented hit on the worker thread; fetches excluded; not complete ROM coverage.","Snapshot is at the completed frame AFTER the hit, not at its exact instruction/read.","Original recorded input retained; only frames before relative_completed were applied.","Derived variant only; no new DS scene acceptance."]});
    let catalog = json!({"schema":1,"cases":[],"variants":[{"catalog_key":format!("CAP/{run}"),"name":format!("{case} derived capture {run}"),"file":"capture.ffstate","sha256":hash(&state),"region":region,"origin":"capture.json; original input.input and origin.ffstate","scope":"Derived trigger capture; unvalidated variant. See capture.json for identity, hit, safe boundary and collector limits."}]});
    fs::write(
        staging.path().join("capture.json"),
        serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        staging.path().join("debug-catalog.json"),
        serde_json::to_vec_pretty(&catalog).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let path = staging.keep();
    fs::rename(&path, &destination).map_err(|e| {
        format!(
            "Capture publication failed; retained {}: {e}",
            path.display()
        )
    })?;
    Ok(format!("captures/{run}/debug-catalog.json"))
}

pub(crate) fn cmd(args: &[String]) -> u8 {
    let error = |e: String| {
        println!(
            "{}",
            json!({"schema":1,"kind":"ffta-debug-capture-result","outcome":"incompatible","message":e})
        );
        3
    };
    if args.first().map(String::as_str) == Some("create") {
        return match create_cmd(&args[1..]) {
            Ok(p) => {
                println!(
                    "{}",
                    json!({"schema":1,"kind":"ffta-debug-capture-created","profile":p})
                );
                0
            }
            Err(e) => error(e),
        };
    }
    if matches!(args.first().map(String::as_str), Some("run" | "cancel")) && args.len() >= 2 {
        if let Err(e) = crate::debug_replay::Job::require_capture(Path::new(&args[1])) {
            return error(e);
        }
        return crate::debug_replay::cmd(args);
    }
    error("Use debug-capture create [replay create options] --resource ID --manifest FILE [--preset all|bus|host], or --pc HEX --mode arm|thumb --opcode HEX, or --frame N; [--limit-frames N]. Then run/cancel replay.json".into())
}
fn create_cmd(args: &[String]) -> Result<PathBuf, String> {
    create_from_menu(args,false)
}
pub(crate) fn create_from_menu(args:&[String],validate_only:bool)->Result<PathBuf,String>{
    let (mut resource, mut manifest, mut dataset, mut pc, mut mode, mut opcode, mut frame) =
        (None, None, None, None, None, None, None);
    let (mut preset, mut limit) = ("all".to_owned(), u64::MAX);
    let mut rest = Vec::new();
    let mut it = args.iter();
    while let Some(k) = it.next() {
        let v = it.next().ok_or_else(|| format!("{k} needs a value"))?;
        match k.as_str() {
            "--resource" => resource = Some(v.clone()),
            "--manifest" => manifest = Some(PathBuf::from(v)),
            "--dataset" => dataset = Some(PathBuf::from(v)),
            "--pc" => {
                pc = Some(
                    u32::from_str_radix(v.trim_start_matches("0x"), 16)
                        .map_err(|_| "Invalid PC hex")?,
                )
            }
            "--mode" => mode = Some(v.clone()),
            "--opcode" => {
                opcode = Some(
                    u32::from_str_radix(v.trim_start_matches("0x"), 16)
                        .map_err(|_| "Invalid opcode hex")?,
                )
            }
            "--frame" => frame = Some(v.parse::<u64>().map_err(|_| "Invalid frame event")?),
            "--limit-frames" => limit = v.parse().map_err(|_| "Invalid frame limit")?,
            "--preset" => preset = v.clone(),
            _ => {
                rest.push(k.clone());
                rest.push(v.clone());
            }
        }
    }
    if usize::from(resource.is_some()) + usize::from(pc.is_some()) + usize::from(frame.is_some())
        != 1
    {
        return Err("Choose exactly one resource, PC or frame trigger".into());
    }
    if resource.is_none() && (manifest.is_some() || preset != "all")
        || pc.is_none() && (mode.is_some() || opcode.is_some())
    {
        return Err("Trigger options do not match the selected condition".into());
    }
    let condition = if let Some(id) = resource {
        Condition::Resource {
            id,
            preset,
            manifest_sha256: String::new(),
        }
    } else if let Some(pc) = pc {
        let thumb = match mode.as_deref() {
            Some("arm") => false,
            Some("thumb") => true,
            _ => return Err("PC trigger requires --mode arm|thumb".into()),
        };
        Condition::Instruction {
            pc,
            thumb,
            opcode: opcode.ok_or("PC trigger requires --opcode")?,
        }
    } else {
        Condition::Frame {
            completed: frame.unwrap(),
        }
    };
    crate::debug_replay::create_dispatch(
        &rest,None,
        Some(&Request {
            settings: Settings {
                condition,
                limit_frames: limit,
            },
            manifest,
            dataset,
        }),None,validate_only,
    )
}

#[cfg(test)]
mod tests;
