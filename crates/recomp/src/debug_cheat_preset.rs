//! Named cheat sets are inert data. Loading never enables a plan; the caller
//! must recheck the catalog, ROM, source lines and all group conflicts.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fs, io::Write, path::{Path, PathBuf}};

const MAX_BYTES: u64 = 16 * 1024;
const MAX_PRESETS: usize = 32;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Member {
    pub id: String,
    pub provenance_index: u16,
    pub master_id: String,
    pub entry_lines_sha256: String,
    pub master_lines_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter: Option<crate::debug_cheat_parameters::Choice>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<crate::debug_cheat_parameters::Choice>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Preset {
    pub schema: u32,
    pub kind: String,
    pub name: String,
    pub catalog_sha256: String,
    pub rom_sha256: String,
    pub members: Vec<Member>,
}

pub(crate) fn name_valid(name: &str) -> bool {
    !name.is_empty() && name.len() <= 48
        && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}
fn hash_valid(hash: &str) -> bool {
    hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn id_valid(id: &str) -> bool {
    id.len() == 7 && id.starts_with("CH-") && id[3..].bytes().all(|b| b.is_ascii_digit())
}
impl Preset {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.schema != 1 || self.kind != "ffta-debug-cheat-preset" || !name_valid(&self.name) {
            return Err("Invalid cheat preset schema or name".into());
        }
        if !hash_valid(&self.catalog_sha256) || !hash_valid(&self.rom_sha256) {
            return Err("Cheat preset needs exact lowercase catalog and ROM SHA-256 hashes".into());
        }
        if self.members.is_empty() || self.members.len() > 8 {
            return Err("A cheat preset needs 1..8 members".into());
        }
        let mut ids = BTreeSet::new();
        for member in &self.members {
            if !id_valid(&member.id) || !id_valid(&member.master_id) || member.id == member.master_id
                || !ids.insert(&member.id) || !hash_valid(&member.entry_lines_sha256)
                || !hash_valid(&member.master_lines_sha256) {
                return Err("Invalid or duplicate cheat preset member".into());
            }
            if let Some(choice)=&member.parameter { choice.validate()?; }
            if member.parameter.is_some() && !member.parameters.is_empty() || member.parameters.len()>8 {return Err("Invalid preset parameter choices".into());}
            let mut names=BTreeSet::new();for choice in &member.parameters{choice.validate()?;if !names.insert(&choice.name){return Err("Duplicate preset parameter".into());}}
        }
        Ok(())
    }
}

fn directory(config: &Path, create: bool) -> Result<PathBuf, String> {
    let metadata = crate::debug_content::ordinary(config)?;
    if !metadata.is_dir() { return Err("Configuration root is not a directory".into()); }
    let dir = config.join("debug-cheat-presets");
    if create {
        match fs::create_dir(&dir) {
            Ok(()) => (),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
            Err(e) => return Err(e.to_string()),
        }
    }
    let metadata = crate::debug_content::ordinary(&dir)?;
    if !metadata.is_dir() { return Err("Cheat preset path is not a directory".into()); }
    Ok(dir)
}
fn file(dir: &Path, name: &str) -> Result<PathBuf, String> {
    if !name_valid(name) { return Err("Preset name must contain 1..48 ASCII letters, digits, '-' or '_'".into()); }
    Ok(dir.join(format!("preset-{name}.json")))
}

/// Publish a new file without overwriting an existing preset.
pub(crate) fn save(config: &Path, preset: &Preset) -> Result<(), String> {
    preset.validate()?;
    let bytes = serde_json::to_vec_pretty(preset).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_BYTES { return Err("Cheat preset exceeds 16 KiB".into()); }
    let dir = directory(config, true)?;
    if fs::read_dir(&dir).map_err(|e| e.to_string())?.take(MAX_PRESETS + 1).count() >= MAX_PRESETS {
        return Err("Cheat preset directory is full".into());
    }
    let destination = file(&dir, &preset.name)?;
    let mut temporary = tempfile::NamedTempFile::new_in(&dir).map_err(|e| e.to_string())?;
    temporary.write_all(&bytes).and_then(|_| temporary.as_file().sync_all()).map_err(|e| e.to_string())?;
    temporary.persist_noclobber(destination).map_err(|e| e.error.to_string())?;
    Ok(())
}

pub(crate) fn load(config: &Path, name: &str) -> Result<Preset, String> {
    let dir = directory(config, false)?;
    let path = file(&dir, name)?;
    crate::debug_content::ordinary(&path)?;
    let bytes = crate::diagnostics::read(&path, MAX_BYTES)?;
    let preset: Preset = serde_json::from_slice(&bytes).map_err(|e| format!("Invalid cheat preset JSON: {e}"))?;
    preset.validate()?;
    if preset.name != name { return Err("Cheat preset name differs from the requested name".into()); }
    Ok(preset)
}

/// Enumerate only valid preset filenames; contents are authenticated on prepare.
pub(crate) fn list(config: &Path) -> Result<Vec<String>, String> {
    crate::debug_content::ordinary(config)?;
    let candidate=config.join("debug-cheat-presets");
    match fs::symlink_metadata(&candidate) {
        Err(e) if e.kind()==std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.to_string()),
        Ok(_) => (),
    }
    let dir=directory(config,false)?;let entries=fs::read_dir(&dir).map_err(|e|e.to_string())?.take(MAX_PRESETS+1).collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
    if entries.len()>MAX_PRESETS{return Err("Cheat preset directory exceeds 32 entries".into());}
    let mut names=Vec::new();
    for entry in entries {
        let filename=entry.file_name();let Some(filename)=filename.to_str() else{continue};
        let Some(name)=filename.strip_prefix("preset-").and_then(|s|s.strip_suffix(".json")) else{continue};
        if name_valid(name) && crate::debug_content::ordinary(&entry.path()).is_ok_and(|m|m.is_file()){names.push(name.to_owned());}
    }
    names.sort();Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn preset() -> Preset {
        Preset { schema: 1, kind: "ffta-debug-cheat-preset".into(), name: "shop-test".into(),
            catalog_sha256: "a".repeat(64), rom_sha256: "b".repeat(64), members: vec![Member {
                id: "CH-0001".into(), provenance_index: 0, master_id: "CH-0002".into(),
                entry_lines_sha256: "c".repeat(64), master_lines_sha256: "d".repeat(64), parameter: None, parameters: Vec::new(),
            }] }
    }
    #[test]
    fn named_preset_roundtrip_is_inert_and_does_not_overwrite() {
        let root = tempfile::tempdir().unwrap(); let p = preset();
        save(root.path(), &p).unwrap(); assert_eq!(load(root.path(), &p.name).unwrap(), p);
        assert_eq!(list(root.path()).unwrap(), vec!["shop-test"]);
        let mut changed = p.clone(); changed.members[0].provenance_index = 1;
        assert!(save(root.path(), &changed).is_err()); assert_eq!(load(root.path(), &p.name).unwrap(), p);
        assert!(load(root.path(), "../shop-test").is_err());
    }
    #[test]
    fn parameter_choice_roundtrips_and_legacy_members_remain_unchanged() {
        let mut p=preset();let legacy=serde_json::to_value(&p).unwrap();assert!(legacy["members"][0].get("parameter").is_none());
        p.members[0].parameter=Some(crate::debug_cheat_parameters::Choice{name:"amount".into(),value:10});
        let root=tempfile::tempdir().unwrap();save(root.path(),&p).unwrap();assert_eq!(load(root.path(),&p.name).unwrap(),p);
        p.members[0].parameter.as_mut().unwrap().name="../bad".into();assert!(p.validate().is_err());
    }
    #[test]
    fn multiple_parameter_choices_are_retained_and_unique(){
        let mut p=preset();p.members[0].parameters=vec![crate::debug_cheat_parameters::Choice{name:"race".into(),value:1},crate::debug_cheat_parameters::Choice{name:"job".into(),value:2}];
        let root=tempfile::tempdir().unwrap();save(root.path(),&p).unwrap();assert_eq!(load(root.path(),&p.name).unwrap(),p);
        p.members[0].parameters[1].name="race".into();assert!(p.validate().is_err());
        p.members[0].parameters[1].name="job".into();p.members[0].parameter=Some(p.members[0].parameters[0].clone());assert!(p.validate().is_err());
    }
    #[test]
    fn malformed_and_ambiguous_members_are_rejected() {
        let mut p = preset(); p.members.push(p.members[0].clone()); assert!(p.validate().is_err());
        p = preset(); p.members[0].master_id = p.members[0].id.clone(); assert!(p.validate().is_err());
        p = preset(); p.catalog_sha256 = "A".repeat(64); assert!(p.validate().is_err());
        p = preset(); p.name = "../bad".into(); assert!(p.validate().is_err());
    }
    #[cfg(unix)]
    #[test]
    fn linked_preset_directory_is_refused() {
        let root = tempfile::tempdir().unwrap(); let other = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(other.path(), root.path().join("debug-cheat-presets")).unwrap();
        assert!(save(root.path(), &preset()).is_err());
    }
}
