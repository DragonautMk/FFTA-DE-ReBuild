//! Bounded interpreter search and immutable backing-memory diagnostics.
use gba_core::{
    debug_memory::{self, Watch},
    Machine,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;

pub(crate) const MAX_SEARCH_FRAMES: u64 = 600;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum Condition {
    Pc { address: u32, thumb: bool },
    Access { watch: Watch },
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Settings {
    pub condition: Condition,
    pub steps: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window: Option<Window>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guard: Option<RamGuard>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_hit: Option<HitAction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger: Option<TriggerIdentity>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TriggerIdentity {
    pub name: String,
    pub sha256: String,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum HitAction {
    Pause,
    Capture,
    Freeze,
}
/// Count matching instruction entries or matching interpreter steps inside
/// [start_frame, end_frame), measured in completed frames from search origin.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Window {
    pub start_frame: u64,
    pub end_frame: u64,
    pub hit_count: u32,
}
/// Side-effect-free equality predicate, observed before the candidate step.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RamGuard {
    pub address: u32,
    pub width: u8,
    pub value: u32,
}
impl RamGuard {
    fn validate(&self) -> Result<(), String> {
        let width = u32::from(self.width);
        if !matches!(width, 1 | 2 | 4)
            || self.address % width != 0
            || (width < 4 && self.value >= (1u32 << (width * 8)))
        {
            return Err("Guard needs aligned width 1/2/4 and a fitting value".into());
        }
        let end = self
            .address
            .checked_add(width - 1)
            .ok_or("Guard address overflow")?;
        if !((0x02000000..=0x0203ffff).contains(&self.address) && end <= 0x0203ffff
            || (0x03000000..=0x03007fff).contains(&self.address) && end <= 0x03007fff)
        {
            return Err("Guard permits only canonical EWRAM/IWRAM backing bytes".into());
        }
        Ok(())
    }
    fn observed(&self, m: &Machine) -> u32 {
        let (bytes, offset) = if self.address < 0x03000000 {
            (&m.bus.ewram, (self.address - 0x02000000) as usize)
        } else {
            (&m.bus.iwram, (self.address - 0x03000000) as usize)
        };
        (0..usize::from(self.width)).fold(0, |v, i| v | (u32::from(bytes[offset + i]) << (8 * i)))
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if self.steps == 0 || self.steps > 5_000_000 {
            return Err("Search needs 1..5000000 interpreter steps".into());
        }
        if self.window.as_ref().is_some_and(|w| {
            w.start_frame >= w.end_frame
                || w.end_frame > MAX_SEARCH_FRAMES
                || !(1..=10_000).contains(&w.hit_count)
        }) {
            return Err("Search window needs 0 <= start < end <= 600 and 1..10000 hits".into());
        }
        if let Some(identity) = &self.trigger {
            if !crate::debug_cheat_preset::name_valid(&identity.name)
                || identity.sha256.len() != 64
                || !identity
                    .sha256
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return Err(
                    "Named trigger needs a valid name and lowercase definition SHA-256".into(),
                );
            }
        }
        if let Some(guard) = &self.guard {
            guard.validate()?;
        }
        match &self.condition {
            Condition::Pc { address, thumb } if address % if *thumb { 2 } else { 4 } != 0 => {
                Err("PC must be aligned for the selected ARM/Thumb mode".into())
            }
            Condition::Access { watch } => watch.validate(),
            _ => Ok(()),
        }
    }
}
pub(crate) struct Search {
    pub settings: Settings,
    steps: u64,
    frames: u64,
    hits: u32,
    candidates: u64,
    last_guard: Option<u32>,
    pub result: Option<Value>,
    #[cfg(feature = "rom-data-read-audit")]
    capture: Option<debug_memory::Capture>,
}
fn eligible(m: &Machine) -> bool {
    !m.bus.halted
        && !(m.bus.irq_pending() && !m.cpu.flag(gba_core::cpu::FLAG_I))
        && !(!m.bus.real_bios
            && m.cpu.regs[15] == gba_core::machine::IRQ_RETURN_ADDR
            && m.cpu.mode() == gba_core::Mode::Irq)
}
impl Search {
    pub fn start(settings: Settings, native: bool) -> Result<Self, String> {
        settings.validate()?;
        if native {
            return Err("Instruction search requires interpreter: native dispatch returns at block boundaries".into());
        }
        #[cfg(not(feature = "rom-data-read-audit"))]
        if matches!(settings.condition, Condition::Access { .. }) {
            return Err("Bus watch requires --features rom-data-read-audit".into());
        }
        #[cfg(feature = "rom-data-read-audit")]
        let capture = match &settings.condition {
            Condition::Access { watch } => Some(debug_memory::Capture::start(watch.clone())?),
            _ => None,
        };
        Ok(Self {
            settings,
            steps: 0,
            frames: 0,
            hits: 0,
            candidates: 0,
            last_guard: None,
            result: None,
            #[cfg(feature = "rom-data-read-audit")]
            capture,
        })
    }
    fn in_window(&self) -> bool {
        self.settings
            .window
            .as_ref()
            .is_none_or(|w| self.frames >= w.start_frame && self.frames < w.end_frame)
    }
    fn matched(&mut self) -> bool {
        self.hits += 1;
        self.hits >= self.settings.window.as_ref().map_or(1, |w| w.hit_count)
    }
    fn guard_matches(&mut self, value: Option<u32>) -> bool {
        self.candidates += 1;
        self.last_guard = value;
        self.settings
            .guard
            .as_ref()
            .is_none_or(|g| value == Some(g.value))
    }
    fn finish(&mut self, mut result: Value, reason: &str) {
        if self.settings.window.is_some() || self.settings.guard.is_some() {
            let count_unit = match self.settings.condition {
                Condition::Pc { .. } => "eligible instruction entries",
                Condition::Access { .. } => "first matching access per complete interpreter step",
            };
            result["evaluation"] = json!({"window":self.settings.window,"guard":self.settings.guard,"guard_sample":"before candidate interpreter step","last_guard_value":self.last_guard,"candidate_count":self.candidates,"observed_hits":self.hits,"stop_reason":reason,"count_unit":count_unit});
        }
        self.result = Some(result);
    }
    /// Bounded slice; the caller drains output only at completed frame boundaries.
    pub fn tick(&mut self, m: &mut Machine) -> bool {
        for _ in 0..2000 {
            if self.result.is_some() {
                return false;
            }
            let (pc, thumb, clock, eligible_instruction) =
                (m.cpu.regs[15], m.cpu.thumb(), m.bus.clock, eligible(m));
            let site = || json!({"pc":pc,"thumb":thumb,"clock":clock,"eligible_instruction":eligible_instruction});
            if self
                .settings
                .window
                .as_ref()
                .is_some_and(|w| self.frames >= w.end_frame)
            {
                self.finish(json!({"outcome":"not-reached","boundary":"after-interpreter-step-or-origin","steps":self.steps,"completed_frames":self.frames}), "frame-window-ended");
                return false;
            }
            let pc_candidate = matches!(&self.settings.condition, Condition::Pc { address, thumb: mode } if eligible_instruction && pc == *address && thumb == *mode);
            if pc_candidate && self.in_window() {
                let guard = self.settings.guard.as_ref().map(|g| g.observed(m));
                if self.guard_matches(guard) && self.matched() {
                    self.finish(json!({"outcome":"hit","boundary":"before-interpreter-step","site":site(),"steps":self.steps,"completed_frames":self.frames}), "target-hit-count");
                    return false;
                }
            }
            if self.steps >= self.settings.steps || self.frames >= MAX_SEARCH_FRAMES {
                let reason = if self.steps >= self.settings.steps {
                    "step-limit"
                } else {
                    "frame-limit"
                };
                self.finish(json!({"outcome":"not-reached","boundary":"after-interpreter-step-or-origin","steps":self.steps,"completed_frames":self.frames}), reason);
                return false;
            }
            // Attribute the entire step, including HLE/DMA, to its starting frame.
            #[cfg(feature = "rom-data-read-audit")]
            let in_window = self.in_window();
            #[cfg(feature = "rom-data-read-audit")]
            let guard_before = if self.capture.is_some() && in_window {
                self.settings.guard.as_ref().map(|g| g.observed(m))
            } else {
                None
            };
            let frame = m.bus.frames;
            m.bus.frame_ready = false;
            m.bus.keys = 0x3ff;
            m.step();
            self.steps += 1;
            let completed = m.bus.frames != frame;
            if completed {
                self.frames += m.bus.frames.saturating_sub(frame);
            }
            #[cfg(feature = "rom-data-read-audit")]
            if let Some(hit) = self.capture.as_ref().and_then(|c| c.take_hit()) {
                if in_window && self.guard_matches(guard_before) && self.matched() {
                    self.finish(json!({"outcome":"hit","boundary":"after-interpreter-step-including-HLE-DMA-tick","site":site(),"access":hit,"steps":self.steps,"completed_frames":self.frames}), "target-hit-count");
                }
            }
            if self.result.is_some() || completed {
                return completed;
            }
        }
        false
    }
    pub fn remaining(&self) -> u64 {
        self.settings.steps.saturating_sub(self.steps)
    }
}

/// Preflight and publish an isolated derived point after a completed frame.
/// This is not the replay-derived DBG-08 bundle and has no input history.
fn capture_directory(root: &Path) -> Result<std::path::PathBuf, String> {
    for path in [
        root.to_path_buf(),
        root.join("Data"),
        root.join("Data/config"),
    ] {
        if !crate::debug_session::regular(&path)?.is_dir() {
            return Err("Trigger capture requires an isolated configuration directory".into());
        }
    }
    let dir = root.join("Data/config/debug-trigger-captures");
    match std::fs::create_dir(&dir) {
        Ok(()) => (),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
        Err(e) => return Err(e.to_string()),
    }
    if !crate::debug_session::regular(&dir)?.is_dir() {
        return Err("Trigger capture path is not a directory".into());
    }
    Ok(dir)
}
fn capture_lock(dir: &Path) -> Result<std::fs::File, String> {
    let p = dir.join(".capture.lock");
    if p.exists() {
        crate::debug_session::regular(&p)?;
    }
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&p)
        .map_err(|e| e.to_string())?;
    crate::debug_session::regular(&p)?;
    file.try_lock()
        .map_err(|_| "Another trigger capture is publishing")?;
    Ok(file)
}
fn check_capture_capacity(dir: &Path) -> Result<(), String> {
    let count = std::fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .filter_map(|v| match v {
            Ok(e) if e.file_name() == ".capture.lock" => None,
            other => Some(other),
        })
        .take(32)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
        .len();
    if count >= 32 {
        return Err("Trigger capture directory is full".into());
    }
    Ok(())
}
pub(crate) fn capture_preflight(root: &Path) -> Result<(), String> {
    let dir = capture_directory(root)?;
    let _lock = capture_lock(&dir)?;
    check_capture_capacity(&dir)
}
pub(crate) fn publish_capture(
    root: &Path,
    machine: &Machine,
    settings: &Settings,
    hit: &Value,
) -> Result<Value, String> {
    if hit["outcome"] != "hit" {
        return Err("Cannot capture without a matched trigger".into());
    }
    let dir = capture_directory(root)?;
    let _lock = capture_lock(&dir)?;
    check_capture_capacity(&dir)?;
    let manifest = crate::diagnostics::read(&root.join("debug-session.json"), 1024 * 1024)?;
    let state = machine.save_state()?;
    let staging = tempfile::Builder::new()
        .prefix(".capture-")
        .tempdir_in(&dir)
        .map_err(|e| e.to_string())?;
    let identifier = staging
        .path()
        .file_name()
        .and_then(|v| v.to_str())
        .ok_or("Invalid capture name")?
        .trim_start_matches('.')
        .to_owned();
    let state_sha256 = gba_core::ffta_rom::digest(&state);
    let info = json!({"schema":1,"kind":"ffta-debug-trigger-capture","id":identifier,"status":"derived-unvalidated","session_manifest_sha256":gba_core::ffta_rom::digest(&manifest),"rom_sha256":gba_core::ffta_rom::digest(&machine.bus.rom),"condition":settings,"hit":hit,"boundary":{"kind":"completed-frame","frame":machine.bus.frames,"clock":machine.bus.clock},"snapshot":"capture.ffstate","state_sha256":state_sha256,"scope":"Isolated trigger snapshot; no replay input or original recording is included"});
    std::fs::write(staging.path().join("capture.ffstate"), &state).map_err(|e| e.to_string())?;
    std::fs::write(
        staging.path().join("capture.json"),
        serde_json::to_vec_pretty(&info).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let destination = dir.join(&identifier);
    std::fs::rename(staging.path(), &destination).map_err(|e| e.to_string())?;
    Ok(
        json!({"id":identifier,"directory":destination.to_string_lossy(),"state_sha256":state_sha256,"frame":machine.bus.frames,"clock":machine.bus.clock}),
    )
}

pub(crate) fn inspect(
    m: &Machine,
    address: u32,
    length: u32,
    root: Option<&Path>,
    dataset: Option<&Path>,
) -> Result<Value, String> {
    let bytes = debug_memory::peek(m, address, length)?;
    let rom_sha = gba_core::ffta_rom::digest(&m.bus.rom);
    let mut labels = Vec::new();
    let mut label_sha = None;
    if let Some(path) = root
        .map(|p| p.join("debug-labels.toml"))
        .filter(|p| p.exists())
    {
        let raw = crate::diagnostics::read(&path, 1024 * 1024)?;
        let parsed = recomp_core::labels::Labels::parse(
            std::str::from_utf8(&raw).map_err(|e| e.to_string())?,
            &path,
            &rom_sha,
            m.bus.rom.len(),
        )?;
        label_sha = Some(gba_core::ffta_rom::digest(&raw));
        for (key, name) in parsed.names {
            let start = key & !1;
            let end = parsed
                .ends
                .get(&key)
                .copied()
                .unwrap_or(start.saturating_add(1));
            if start < address.saturating_add(length) && address < end {
                if labels.len() == 64 {
                    return Err("Too many labels in selected window".into());
                }
                if name.len() > 256 {
                    return Err("Inspect label name exceeds256 bytes".into());
                }
                labels.push(json!({"start":start,"end":end,"thumb":key&1!=0,"name":name,"scope":"declared label, not observed code extent"}));
            }
        }
    }
    let mut owners = Vec::new();
    let mut manifest_sha = None;
    if let Some(path) = dataset.map(|d| d.join("manifest.toml")) {
        let raw =
            crate::diagnostics::read(&path, gba_core::rom_data::MAX_MANIFEST_FILE_BYTES as u64)?;
        let manifest: gba_core::rom_data::Manifest =
            toml::from_str(std::str::from_utf8(&raw).map_err(|e| e.to_string())?)
                .map_err(|e| format!("Inspect manifest: {e}"))?;
        if manifest.schema != 1
            || manifest.source_rom_sha256 != rom_sha
            || manifest.source_rom_size as usize != m.bus.rom.len()
            || manifest.resources.len() > 512
        {
            return Err("Inspect manifest ROM/schema mismatch".into());
        }
        manifest_sha = Some(gba_core::ffta_rom::digest(&raw));
        let mut source_count = 0usize;
        let mut checked_bytes = 0u64;
        for resource in manifest.resources {
            source_count = source_count
                .checked_add(resource.sources.len())
                .ok_or("Too many source ranges")?;
            if source_count > 100000 {
                return Err("Inspect source ranges exceed limit".into());
            }
            for source in resource.sources {
                let end = source
                    .offset
                    .checked_add(source.length)
                    .ok_or("Invalid source range")?;
                if end as usize > m.bus.rom.len() {
                    return Err("Inspect source outside ROM".into());
                }
                if bytes.iter().any(|b| {
                    b.region == "rom-backing"
                        && (source.offset as usize..end as usize).contains(&b.offset)
                }) {
                    checked_bytes += u64::from(source.length);
                    if owners.len() == 64 || checked_bytes > 64 * 1024 * 1024 {
                        return Err("Inspect owner authentication exceeds limit".into());
                    }
                    if gba_core::ffta_rom::digest(&m.bus.rom[source.offset as usize..end as usize])
                        != source.sha256
                    {
                        return Err("Inspect source checksum mismatch".into());
                    }
                    if resource.id.len() > 128 || source.role.len() > 256 {
                        return Err("Inspect source metadata exceeds limit".into());
                    }
                    owners.push(json!({"id":resource.id,"offset":source.offset,"length":source.length,"role":source.role}));
                }
            }
        }
    }
    Ok(
        json!({"kind":"ffta-readonly-inspection","address":address,"length":length,"bytes":bytes,
        "registers":m.cpu.regs,"cpsr":m.cpu.cpsr,"thumb":m.cpu.thumb(),"frame":m.bus.frames,"clock":m.bus.clock,
        "labels":labels,"labels_sha256":label_sha,"known_source_owners":owners,"manifest_sha256":manifest_sha,
        "owner_scope":"authenticated source-plan overlap only; unknown without a match; not a typed-consumption claim",
        "memory_scope":"immutable backing bytes; no bus IO, RTC, EEPROM or backup protocol read"}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn machine() -> Machine {
        let words = [0xe5910000u32, 0xe5820000, 0xeafffffe];
        let mut m = Machine::new(words.iter().flat_map(|v| v.to_le_bytes()).collect());
        m.cpu.regs[15] = 0x08000000;
        m.cpu.regs[1] = 0x02000000;
        m.cpu.regs[2] = 0x02000004;
        m.bus.ewram[..4].copy_from_slice(&91u32.to_le_bytes());
        m
    }
    #[test]
    fn pc_exact_mode_and_before_step_stop_do_not_mutate_origin() {
        let mut m = machine();
        let seed = m.save_state().unwrap();
        let mut s = Search::start(
            Settings {
                condition: Condition::Pc {
                    address: 0x08000000,
                    thumb: false,
                },
                steps: 2,
                window: None,
                guard: None,
                on_hit: None,
                trigger: None,
            },
            false,
        )
        .unwrap();
        assert!(!s.tick(&mut m));
        assert_eq!(s.result.as_ref().unwrap()["steps"], 0);
        assert_eq!(seed, m.save_state().unwrap());
        let mut s = Search::start(
            Settings {
                condition: Condition::Pc {
                    address: 0x08000000,
                    thumb: true,
                },
                steps: 2,
                window: None,
                guard: None,
                on_hit: None,
                trigger: None,
            },
            false,
        )
        .unwrap();
        s.tick(&mut m);
        assert_eq!(s.result.as_ref().unwrap()["outcome"], "not-reached");
        assert_eq!(s.result.as_ref().unwrap()["steps"], 2);
    }
    #[cfg(feature = "rom-data-read-audit")]
    #[test]
    fn write_stop_matches_unobserved_execution_after_whole_step() {
        let mut m = machine();
        let seed = m.save_state().unwrap();
        let mut s = Search::start(
            Settings {
                condition: Condition::Access {
                    watch: Watch {
                        start: 0x02000004,
                        end: 0x02000008,
                        write: true,
                        width: Some(4),
                        value: Some(91),
                    },
                },
                steps: 3,
                window: None,
                guard: None,
                on_hit: None,
                trigger: None,
            },
            false,
        )
        .unwrap();
        s.tick(&mut m);
        assert_eq!(s.result.as_ref().unwrap()["steps"], 2);
        assert_eq!(s.result.as_ref().unwrap()["access"]["value"], 91);
        let actual = m.save_state().unwrap();
        drop(s);
        m.load_state(&seed).unwrap();
        m.bus.frame_ready = false;
        m.step();
        m.bus.frame_ready = false;
        m.step();
        assert_eq!(actual, m.save_state().unwrap());
    }
    #[test]
    fn legacy_search_settings_roundtrip_without_a_window_field() {
        let original =
            json!({"condition":{"kind":"pc","address":134217728,"thumb":false},"steps":100});
        let settings: Settings = serde_json::from_value(original.clone()).unwrap();
        assert!(settings.window.is_none());
        assert_eq!(serde_json::to_value(settings).unwrap(), original);
    }
    #[test]
    fn capture_publishes_a_bound_snapshot_only_for_a_hit() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path();
        std::fs::create_dir_all(root.join("Data/config")).unwrap();
        std::fs::write(root.join("debug-session.json"), b"synthetic session").unwrap();
        let m = machine();
        let before = m.save_state().unwrap();
        let settings = Settings {
            condition: Condition::Pc {
                address: 0x08000000,
                thumb: false,
            },
            steps: 1,
            window: None,
            guard: None,
            on_hit: Some(HitAction::Capture),
            trigger: Some(TriggerIdentity {
                name: "checkpoint".into(),
                sha256: "a".repeat(64),
            }),
        };
        capture_preflight(root).unwrap();
        assert!(publish_capture(root, &m, &settings, &json!({"outcome":"not-reached"})).is_err());
        let result =
            publish_capture(root, &m, &settings, &json!({"outcome":"hit","steps":0})).unwrap();
        let folder = std::path::PathBuf::from(result["directory"].as_str().unwrap());
        let saved = std::fs::read(folder.join("capture.ffstate")).unwrap();
        let metadata: Value =
            serde_json::from_slice(&std::fs::read(folder.join("capture.json")).unwrap()).unwrap();
        assert_eq!(saved, before);
        assert_eq!(m.save_state().unwrap(), before);
        assert_eq!(metadata["state_sha256"], gba_core::ffta_rom::digest(&saved));
        assert_eq!(metadata["boundary"]["frame"], m.bus.frames);
        assert_eq!(metadata["condition"]["trigger"]["name"], "checkpoint");
        assert_eq!(metadata["condition"]["trigger"]["sha256"], "a".repeat(64));
        assert_eq!(
            metadata["session_manifest_sha256"],
            gba_core::ffta_rom::digest(b"synthetic session")
        );
    }
    #[test]
    fn pc_and_ram_guard_are_side_effect_free_and_explain_both_results() {
        let mut m = machine();
        m.cpu.regs[15] = 0x08000008;
        let settings = |guard| Settings {
            condition: Condition::Pc {
                address: 0x08000008,
                thumb: false,
            },
            steps: 1,
            window: None,
            guard: Some(guard),
            on_hit: None,
            trigger: None,
        };
        let valid = |value| RamGuard {
            address: 0x02000000,
            width: 4,
            value,
        };
        let seed = m.save_state().unwrap();
        let mut hit = Search::start(settings(valid(91)), false).unwrap();
        hit.tick(&mut m);
        assert_eq!(hit.result.as_ref().unwrap()["outcome"], "hit");
        assert_eq!(
            hit.result.as_ref().unwrap()["evaluation"]["last_guard_value"],
            91
        );
        assert_eq!(seed, m.save_state().unwrap());
        let mut miss = Search::start(settings(valid(92)), false).unwrap();
        miss.tick(&mut m);
        let result = miss.result.as_ref().unwrap();
        assert_eq!(result["outcome"], "not-reached");
        assert_eq!(result["evaluation"]["observed_hits"], 0);
        assert_eq!(result["evaluation"]["last_guard_value"], 91);
        for guard in [
            RamGuard {
                address: 0x04000130,
                width: 2,
                value: 0,
            },
            RamGuard {
                address: 0x02000001,
                width: 2,
                value: 0,
            },
            RamGuard {
                address: 0x02000000,
                width: 1,
                value: 256,
            },
            RamGuard {
                address: 0x02000000,
                width: 0,
                value: 0,
            },
        ] {
            assert!(Search::start(settings(guard), false).is_err());
        }
    }
    #[cfg(feature = "rom-data-read-audit")]
    #[test]
    fn watch_guard_uses_pre_step_ram_not_post_write_value() {
        let mut m = machine();
        m.cpu.regs[15] = 0x08000004;
        m.cpu.regs[0] = 91;
        let mut s = Search::start(
            Settings {
                condition: Condition::Access {
                    watch: Watch {
                        start: 0x02000004,
                        end: 0x02000008,
                        write: true,
                        width: Some(4),
                        value: Some(91),
                    },
                },
                steps: 2,
                window: None,
                guard: Some(RamGuard {
                    address: 0x02000004,
                    width: 4,
                    value: 0,
                }),
                on_hit: None,
                trigger: None,
            },
            false,
        )
        .unwrap();
        s.tick(&mut m);
        let result = s.result.as_ref().unwrap();
        assert_eq!(result["outcome"], "hit");
        assert_eq!(result["evaluation"]["last_guard_value"], 0);
        assert_eq!(result["access"]["value"], 91);
        assert_eq!(m.bus.ewram[4], 91);
    }
    #[test]
    fn counted_pc_hits_and_exclusive_frame_window_are_explainable() {
        let mut m = machine();
        m.cpu.regs[15] = 0x08000008; // synthetic branch to itself
        let seed = m.save_state().unwrap();
        let settings = |start_frame, end_frame, hit_count, steps| Settings {
            condition: Condition::Pc {
                address: 0x08000008,
                thumb: false,
            },
            steps,
            window: Some(Window {
                start_frame,
                end_frame,
                hit_count,
            }),
            guard: None,
            on_hit: None,
            trigger: None,
        };
        let mut s = Search::start(settings(0, 3, 2, 10), false).unwrap();
        s.tick(&mut m);
        let hit = s.result.as_ref().unwrap();
        assert_eq!(hit["outcome"], "hit");
        assert_eq!(hit["steps"], 1);
        assert_eq!(hit["evaluation"]["observed_hits"], 2);
        assert_eq!(hit["evaluation"]["stop_reason"], "target-hit-count");
        let actual = m.save_state().unwrap();
        m.load_state(&seed).unwrap();
        m.bus.keys = 0x3ff;
        m.bus.frame_ready = false;
        m.step();
        assert_eq!(actual, m.save_state().unwrap());
        let mut skipped = Search::start(settings(1, 3, 1, 2), false).unwrap();
        skipped.tick(&mut m);
        assert_eq!(skipped.result.as_ref().unwrap()["outcome"], "not-reached");
        assert_eq!(
            skipped.result.as_ref().unwrap()["evaluation"]["observed_hits"],
            0
        );
        assert_eq!(
            skipped.result.as_ref().unwrap()["evaluation"]["stop_reason"],
            "step-limit"
        );
        let mut ended = Search::start(settings(0, 1, 1, 10), false).unwrap();
        ended.frames = 1;
        let before = m.save_state().unwrap();
        ended.tick(&mut m);
        assert_eq!(
            ended.result.as_ref().unwrap()["evaluation"]["stop_reason"],
            "frame-window-ended"
        );
        assert_eq!(before, m.save_state().unwrap());
        for (start, end, hits) in [(0, 0, 1), (3, 2, 1), (0, 601, 1), (0, 1, 0), (0, 1, 10001)] {
            assert!(Search::start(settings(start, end, hits, 10), false).is_err());
        }
    }
    #[cfg(feature = "rom-data-read-audit")]
    #[test]
    fn counted_watch_hits_are_distinct_complete_interpreter_steps() {
        let words = [0xe5820000u32, 0xeafffffdu32]; // store; branch back to store
        let mut m = Machine::new(words.iter().flat_map(|v| v.to_le_bytes()).collect());
        m.cpu.regs[15] = 0x08000000;
        m.cpu.regs[2] = 0x02000004;
        m.cpu.regs[0] = 91;
        let seed = m.save_state().unwrap();
        let mut s = Search::start(
            Settings {
                condition: Condition::Access {
                    watch: Watch {
                        start: 0x02000004,
                        end: 0x02000008,
                        write: true,
                        width: Some(4),
                        value: Some(91),
                    },
                },
                steps: 10,
                window: Some(Window {
                    start_frame: 0,
                    end_frame: 4,
                    hit_count: 2,
                }),
                guard: None,
                on_hit: None,
                trigger: None,
            },
            false,
        )
        .unwrap();
        s.tick(&mut m);
        let result = s.result.as_ref().unwrap();
        assert_eq!(result["outcome"], "hit");
        assert_eq!(result["steps"], 3);
        assert_eq!(result["evaluation"]["observed_hits"], 2);
        assert_eq!(result["access"]["value"], 91);
        let actual = m.save_state().unwrap();
        drop(s);
        m.load_state(&seed).unwrap();
        for _ in 0..3 {
            m.bus.keys = 0x3ff;
            m.bus.frame_ready = false;
            m.step();
        }
        assert_eq!(actual, m.save_state().unwrap());
    }
    #[test]
    fn bounds_backend_and_inactive_watch_are_rejected_before_execution() {
        let settings = |address, steps| Settings {
            condition: Condition::Pc {
                address,
                thumb: false,
            },
            steps,
            window: None,
            guard: None,
            on_hit: None,
            trigger: None,
        };
        assert!(Search::start(settings(0x08000000, 1), true).is_err());
        for (address, steps) in [(0x08000002, 1), (0x08000000, 0), (0x08000000, 5000001)] {
            assert!(Search::start(settings(address, steps), false).is_err());
        }
        #[cfg(not(feature = "rom-data-read-audit"))]
        assert!(Search::start(
            Settings {
                condition: Condition::Access {
                    watch: Watch {
                        start: 0x02000000,
                        end: 0x02000004,
                        write: false,
                        width: None,
                        value: None
                    }
                },
                steps: 1,
                window: None,
                guard: None,
                on_hit: None,
                trigger: None
            },
            false
        )
        .is_err());
    }
    #[test]
    fn authenticated_label_and_source_owner_are_provenance_only() {
        let m = machine();
        let seed = m.save_state().unwrap();
        let t = tempfile::tempdir().unwrap();
        let sha = gba_core::ffta_rom::digest(&m.bus.rom);
        std::fs::write(t.path().join("debug-labels.toml"),format!("format = \"gba-labels\"\nversion = 2\n[image]\nsha256 = \"{sha}\"\n[[functions]]\nname = \"synthetic reader\"\naddress = 0x08000000\nend = 0x0800000c\nmode = \"arm\"\n")).unwrap();
        let manifest = gba_core::rom_data::Manifest {
            schema: 1,
            extractor_revision: 1,
            decoder_revision: 1,
            source_rom_sha256: sha.clone(),
            source_rom_size: m.bus.rom.len() as u32,
            profile: "synthetic".into(),
            coverage: vec![],
            resources: vec![gba_core::rom_data::Resource {
                id: "synthetic.bytes".into(),
                path: "unused".into(),
                format: "test".into(),
                sha256: sha.clone(),
                dependencies: vec![],
                sources: vec![gba_core::rom_data::SourceRange {
                    offset: 0,
                    length: 12,
                    sha256: sha,
                    role: "test backing".into(),
                }],
            }],
        };
        std::fs::write(
            t.path().join("manifest.toml"),
            toml::to_string(&manifest).unwrap(),
        )
        .unwrap();
        let v = inspect(&m, 0x08000000, 8, Some(t.path()), Some(t.path())).unwrap();
        assert_eq!(v["labels"][0]["name"], "synthetic reader");
        assert_eq!(v["known_source_owners"][0]["id"], "synthetic.bytes");
        let mut bad = manifest;
        bad.resources[0].sources[0].sha256 = "0".repeat(64);
        std::fs::write(
            t.path().join("manifest.toml"),
            toml::to_string(&bad).unwrap(),
        )
        .unwrap();
        assert!(inspect(&m, 0x08000000, 8, None, Some(t.path())).is_err());
        assert_eq!(seed, m.save_state().unwrap());
    }
    #[test]
    fn readonly_inspection_and_foreign_labels_preserve_state() {
        let m = machine();
        let seed = m.save_state().unwrap();
        let t = tempfile::tempdir().unwrap();
        let v = inspect(&m, 0x08000000, 8, Some(t.path()), None).unwrap();
        assert_eq!(v["registers"][15], 0x08000000u32);
        assert_eq!(v["bytes"][0]["offset"], 0);
        std::fs::write(
            t.path().join("debug-labels.toml"),
            format!(
                "format = \"gba-labels\"\nversion = 2\n[image]\nsha256 = \"{}\"\n",
                "0".repeat(64)
            ),
        )
        .unwrap();
        assert!(inspect(&m, 0x08000000, 4, Some(t.path()), None).is_err());
        assert_eq!(seed, m.save_state().unwrap());
    }
}
