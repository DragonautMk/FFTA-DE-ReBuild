//! Bounded interpreter search and immutable backing-memory diagnostics.
use gba_core::{
    debug_memory::{self, Watch},
    Machine,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;

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
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if self.steps == 0 || self.steps > 5_000_000 {
            return Err("Search needs 1..5000000 interpreter steps".into());
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
            result: None,
            #[cfg(feature = "rom-data-read-audit")]
            capture,
        })
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
            if let Condition::Pc { address, thumb } = &self.settings.condition {
                if eligible(m) && m.cpu.regs[15] == *address && m.cpu.thumb() == *thumb {
                    self.result = Some(
                        json!({"outcome":"hit","boundary":"before-interpreter-step","site":site(),"steps":self.steps,"completed_frames":self.frames}),
                    );
                    return false;
                }
            }
            if self.steps >= self.settings.steps || self.frames >= 600 {
                self.result = Some(
                    json!({"outcome":"not-reached","boundary":"after-interpreter-step-or-origin","steps":self.steps,"completed_frames":self.frames}),
                );
                return false;
            }
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
            if let Some(hit) = self.capture.as_ref().and_then(|c| c.hit()) {
                self.result = Some(
                    json!({"outcome":"hit","boundary":"after-interpreter-step-including-HLE-DMA-tick","site":site(),"access":hit,"steps":self.steps,"completed_frames":self.frames}),
                );
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
    fn bounds_backend_and_inactive_watch_are_rejected_before_execution() {
        let settings = |address, steps| Settings {
            condition: Condition::Pc {
                address,
                thumb: false,
            },
            steps,
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
                steps: 1
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
