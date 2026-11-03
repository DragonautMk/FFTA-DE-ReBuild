//! Bounded ROM read evidence on isolated replay profiles; no extraction or byte dumps.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
const MANIFEST_LIMIT: u64 = gba_core::rom_data::MAX_MANIFEST_FILE_BYTES;
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Settings {
    pub preset: String,
    pub from: u64,
    pub to: u64,
    pub ranges: Vec<(u32, u32)>,
    pub limit: usize,
    pub known_sha256: Option<String>,
}
pub(crate) struct Request {
    pub settings: Settings,
    pub known: Option<PathBuf>,
    pub dataset: Option<PathBuf>,
}
pub(crate) fn validate(s: &Settings, frames: u64) -> Result<(), String> {
    if !["all", "bus", "host"].contains(&s.preset.as_str())
        || s.from >= s.to
        || s.to > frames
        || !(1..=50000).contains(&s.limit)
        || s.ranges.len() > 32
        || s.ranges.iter().any(|&(a, b)| a >= b || b > 0x02000000)
    {
        return Err("Invalid audit preset/window/range/event limit".into());
    }
    Ok(())
}
fn manifest(bytes: &[u8], rom: &[u8]) -> Result<gba_core::rom_data::Manifest, String> {
    let m: gba_core::rom_data::Manifest =
        toml::from_str(std::str::from_utf8(bytes).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    if m.schema != 1
        || m.source_rom_sha256 != gba_core::ffta_rom::digest(rom)
        || m.source_rom_size as usize != rom.len()
        || m.resources.len() > 512
    {
        return Err("Known manifest ROM/schema mismatch".into());
    }
    let mut count = 0;
    let mut ids = BTreeSet::new();
    for r in &m.resources {
        if !ids.insert(&r.id) {
            return Err("Duplicate known resource ID".into());
        }
        for s in &r.sources {
            count += 1;
            if count > 100000
                || s.offset
                    .checked_add(s.length)
                    .is_none_or(|end| end as usize > rom.len())
            {
                return Err("Known source ranges exceed bounds".into());
            }
        }
    }
    Ok(m)
}
pub(crate) fn prepare(
    request: &Request,
    rom: &[u8],
    frames: u64,
    dataset: Option<&Path>,
) -> Result<(Settings, Option<Vec<u8>>), String> {
    if !cfg!(feature = "rom-data-read-audit") {
        return Err("ROM audit not compiled; use --features rom-data-read-audit".into());
    }
    let mut s = request.settings.clone();
    if s.to == u64::MAX {
        s.to = frames;
    }
    validate(&s, frames)?;
    let known = request
        .known
        .clone()
        .or_else(|| dataset.map(|d| d.join("manifest.toml")));
    let bytes = known
        .map(|p| crate::diagnostics::read(&p, MANIFEST_LIMIT))
        .transpose()?;
    if let Some(bytes) = &bytes {
        manifest(bytes, rom)?;
        s.known_sha256 = Some(gba_core::ffta_rom::digest(bytes));
    }
    Ok((s, bytes))
}
#[cfg(any(feature = "rom-data-read-audit", test))]
fn merge(mut ranges: Vec<(u32, u32)>) -> Vec<(u32, u32)> {
    ranges.retain(|&(a, b)| a < b);
    ranges.sort_unstable();
    let mut out: Vec<(u32, u32)> = Vec::new();
    for (a, b) in ranges {
        if let Some(last) = out.last_mut().filter(|v| a <= v.1) {
            last.1 = last.1.max(b);
        } else {
            out.push((a, b));
        }
    }
    out
}
#[cfg(any(feature = "rom-data-read-audit", test))]
fn bytes(ranges: &[(u32, u32)]) -> u64 {
    ranges.iter().map(|&(a, b)| u64::from(b - a)).sum()
}
#[cfg(any(feature = "rom-data-read-audit", test))]
fn intersection(a: &[(u32, u32)], b: &[(u32, u32)]) -> Vec<(u32, u32)> {
    let (mut i, mut j) = (0, 0);
    let mut out = Vec::new();
    while i < a.len() && j < b.len() {
        let lo = a[i].0.max(b[j].0);
        let hi = a[i].1.min(b[j].1);
        if lo < hi {
            out.push((lo, hi));
        }
        if a[i].1 <= b[j].1 {
            i += 1;
        } else {
            j += 1;
        }
    }
    out
}

pub(crate) struct Audit {
    settings: Settings,
    known: Option<gba_core::rom_data::Manifest>,
    #[cfg(feature = "rom-data-read-audit")]
    capture: gba_core::rom_data::read_audit::TraceCapture,
    observed_frames: u64,
}
impl Audit {
    pub fn start(root: &Path, settings: &Settings, rom: &[u8]) -> Result<Self, String> {
        let known = if let Some(hash) = &settings.known_sha256 {
            let bytes = crate::diagnostics::read(&root.join("known-plan.toml"), MANIFEST_LIMIT)?;
            if gba_core::ffta_rom::digest(&bytes) != *hash {
                return Err("Known manifest changed".into());
            }
            Some(manifest(&bytes, rom)?)
        } else {
            None
        };
        #[cfg(feature = "rom-data-read-audit")]
        {
            use gba_core::rom_data::read_audit::{Reader, TraceCapture, TraceFilter};
            let readers = match settings.preset.as_str() {
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
            let capture = TraceCapture::start(
                TraceFilter {
                    readers,
                    ranges: settings.ranges.clone(),
                },
                settings.limit,
            )?;
            Ok(Self {
                settings: settings.clone(),
                known,
                capture,
                observed_frames: 0,
            })
        }
        #[cfg(not(feature = "rom-data-read-audit"))]
        {
            let _ = (known, settings);
            Err("ROM audit not compiled; choose an audit-enabled runtime".into())
        }
    }
    pub fn frame(&mut self, relative: u64, guest: u64) {
        let enabled = relative >= self.settings.from && relative < self.settings.to;
        if enabled {
            self.observed_frames += 1;
        }
        #[cfg(feature = "rom-data-read-audit")]
        self.capture.frame(enabled.then_some(guest));
        #[cfg(not(feature = "rom-data-read-audit"))]
        let _ = guest;
    }
    pub fn finish(
        self,
        completed: u64,
        outcome: &str,
        case: &str,
        run: &str,
        rom_sha256: &str,
    ) -> Value {
        #[cfg(feature = "rom-data-read-audit")]
        {
            use gba_core::rom_data::read_audit::Service;
            let trace = self.capture.finish();
            let ranges = merge(
                trace
                    .events
                    .iter()
                    .map(|e| {
                        (
                            e.key.read.physical_offset,
                            e.key.read.physical_offset + e.key.read.length as u32,
                        )
                    })
                    .collect(),
            );
            let known = merge(
                self.known
                    .as_ref()
                    .into_iter()
                    .flat_map(|m| &m.resources)
                    .flat_map(|r| &r.sources)
                    .map(|s| (s.offset, s.offset + s.length))
                    .collect(),
            );
            let within = intersection(&ranges, &known);
            let total = bytes(&ranges);
            let mut resources = Vec::new();
            if let Some(m) = &self.known {
                for r in &m.resources {
                    let plan = merge(
                        r.sources
                            .iter()
                            .map(|s| (s.offset, s.offset + s.length))
                            .collect(),
                    );
                    let used = intersection(&ranges, &plan);
                    if !used.is_empty() {
                        resources.push(json!({"id":r.id,"observed_distinct_bytes":bytes(&used),"physical_intervals":used}));
                    }
                }
            }
            let service = |kind| {
                let spans = merge(
                    trace
                        .events
                        .iter()
                        .filter(|e| e.key.service == kind)
                        .map(|e| {
                            (
                                e.key.read.physical_offset,
                                e.key.read.physical_offset + e.key.read.length as u32,
                            )
                        })
                        .collect(),
                );
                json!({"distinct_bytes":bytes(&spans),"physical_intervals":spans})
            };
            let bus = merge(
                trace
                    .events
                    .iter()
                    .filter(|e| e.key.read.reader == gba_core::rom_data::read_audit::Reader::Bus)
                    .map(|e| {
                        (
                            e.key.read.physical_offset,
                            e.key.read.physical_offset + e.key.read.length as u32,
                        )
                    })
                    .collect(),
            );
            let host = merge(
                trace
                    .events
                    .iter()
                    .filter(|e| e.key.read.reader != gba_core::rom_data::read_audit::Reader::Bus)
                    .map(|e| {
                        (
                            e.key.read.physical_offset,
                            e.key.read.physical_offset + e.key.read.length as u32,
                        )
                    })
                    .collect(),
            );
            json!({"schema":1,"kind":"ffta-rom-read-audit","run":run,"case":case,"rom_sha256":rom_sha256,
                "window":{"relative_from":self.settings.from,"relative_to_exclusive":self.settings.to,"frames_entered":self.observed_frames,"replay_completed":completed,"complete":completed>=self.settings.to,"replay_outcome":outcome},
                "filter":{"preset":self.settings.preset,"physical_ranges":self.settings.ranges,"unique_event_limit":self.settings.limit},
                "truncated":trace.truncated,"matched_requests":trace.matched_requests,"dropped_requests":trace.dropped_requests,"retained_events":trace.events.len(),
                "fetch_bus_requests_excluded":trace.fetch_requests_excluded,"observed_distinct_bytes":total,"physical_intervals":ranges,
                "bus":{"distinct_bytes":bytes(&bus),"physical_intervals":bus},"host":{"distinct_bytes":bytes(&host),"physical_intervals":host},"bus_host_shared_distinct_bytes":bytes(&intersection(&bus,&host)),
                "serving":{"typed":service(Service::Typed),"raw":service(Service::Raw),"unclassified":service(Service::Unclassified)},
                "known_plan":{"compared":self.known.is_some(),"manifest_sha256":self.settings.known_sha256,"within_distinct_bytes":self.known.as_ref().map(|_|bytes(&within)),"outside_distinct_bytes":self.known.as_ref().map(|_|total-bytes(&within)),"resources":resources,"authentication":"ROM-identity matched source map; no re-extraction, ownership or new scene acceptance inferred"},
                "events":trace.events,
                "coverage":{"readers":["Bus","AudioValue","SpritePeek","SpriteList","ObjSourceTable","KeyboardWidths"],"instruction_scope":"Interpreter PC/mode may be inherited by HLE/DMA; not a direct-load assertion. None is unscoped/native, not an invented PC.","not_observed":["unselected direct host reads and typed HLE fast paths","other threads","folded native constants","code/signature authentication","GPIO/EEPROM and failed/out-of-image reads"],"fetches":"Interpreter instruction fetches excluded; native instruction execution is not a ROM-data fetch count.","absence":"Outside filters/window, dropped events and missing hooks do not prove unread bytes."}})
        }
        #[cfg(not(feature = "rom-data-read-audit"))]
        {
            let _ = (
                self.known,
                self.observed_frames,
                completed,
                outcome,
                case,
                run,
                rom_sha256,
            );
            json!({"schema":1,"kind":"ffta-rom-read-audit","error":"not compiled"})
        }
    }
}
pub(crate) fn create_cmd(args: &[String]) -> Result<PathBuf, String> {
    create_from_menu(args,false)
}
pub(crate) fn create_from_menu(args:&[String],validate_only:bool)->Result<PathBuf,String>{
    let mut settings = Settings {
        preset: "all".into(),
        from: 0,
        to: u64::MAX,
        ranges: Vec::new(),
        limit: 20000,
        known_sha256: None,
    };
    let mut known = None;
    let mut dataset = None;
    let mut common = Vec::new();
    let mut it = args.iter();
    let number = |s: &str| -> Result<u64, String> {
        if let Some(hex) = s.strip_prefix("0x") {
            u64::from_str_radix(hex, 16)
        } else {
            s.parse()
        }
        .map_err(|_| format!("Invalid audit number: {s}"))
    };
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--preset" | "--from" | "--to" | "--range" | "--limit" | "--known-manifest"
            | "--dataset" => {
                let value = it.next().ok_or_else(|| format!("{arg} needs a value"))?;
                match arg.as_str() {
                    "--preset" => settings.preset = value.clone(),
                    "--from" => settings.from = number(value)?,
                    "--to" => settings.to = number(value)?,
                    "--limit" => {
                        settings.limit =
                            usize::try_from(number(value)?).map_err(|_| "Audit limit overflow")?
                    }
                    "--known-manifest" => known = Some(PathBuf::from(value)),
                    "--dataset" => dataset = Some(PathBuf::from(value)),
                    "--range" => {
                        let (a, b) = value
                            .split_once(':')
                            .ok_or("Use a physical half-open range START:END")?;
                        let a = u32::try_from(number(a)?).map_err(|_| "Audit range overflow")?;
                        let b = u32::try_from(number(b)?).map_err(|_| "Audit range overflow")?;
                        settings.ranges.push((a, b));
                    }
                    _ => unreachable!(),
                }
            }
            _ => common.push(arg.clone()),
        }
    }
    crate::debug_replay::create_dispatch(
        &common,
        Some(&Request {
            settings,
            known,
            dataset,
        }),None,None,validate_only,
    )
}
pub(crate) fn cmd(args: &[String]) -> u8 {
    if args.first().is_some_and(|a| a == "create") {
        return match create_cmd(&args[1..]) {
            Ok(profile) => {
                println!(
                    "{}",
                    json!({"schema":1,"kind":"ffta-rom-audit-created","profile":profile})
                );
                0
            }
            Err(e) => {
                println!(
                    "{}",
                    json!({"schema":1,"kind":"ffta-rom-audit-error","message":e})
                );
                eprintln!("{e}");
                3
            }
        };
    }
    if args.first().is_some_and(|s| s == "run" || s == "cancel") && args.len() >= 2 {
        if let Err(message) = crate::debug_replay::Job::require_audit(Path::new(&args[1])) {
            println!(
                "{}",
                json!({"schema":1,"kind":"ffta-rom-audit-error","message":message})
            );
            eprintln!("{message}");
            return 3;
        }
    }
    crate::debug_replay::cmd(args)
}
#[cfg(test)]
mod tests;
