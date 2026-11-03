//! Bounded, readonly discovery. An index is neither an execution plan nor proof
//! that a pinned profile, runtime, snapshot or dataset passes its real preflight.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::Read,
    path::{Component, Path, PathBuf},
};

pub(crate) const MAX_ROOTS: usize = 8;
const MAX_ENTRIES: usize = 512;
const MAX_NODES: usize = 8192;
const MAX_DEPTH: usize = 12;
const MAX_METADATA: u64 = 1024 * 1024;
const MAX_METADATA_TOTAL: u64 = 8 * MAX_METADATA;
const MAX_HASH_BYTES: u64 = 64 * 1024 * 1024;
const MAX_INDEX: u64 = 8 * 1024 * 1024;
const MAX_ISSUES: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Kind {
    Session,
    Recording,
    Input,
    ReplayProfile,
    Recipe,
    HostTool,
    GuestScriptCatalog,
    Campaign,
    Snapshot,
    Runtime,
    Dataset,
    NativeLibrary,
    Labels,
    KnownManifest,
    SaveCatalog,
    CheatCatalog,
    Report,
    Unknown,
}
impl Kind {
    pub(crate) fn name(self) -> String {
        serde_json::to_value(self).unwrap().as_str().unwrap().into()
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Root {
    pub id: String,
    pub path: PathBuf,
    /// portable traverses declared app subtrees only; explicit roots are opt-in.
    pub scope: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
}
impl Root {
    pub fn new(path: &Path, portable: bool) -> Result<Self, String> {
        let path = std::path::absolute(path).map_err(|e| e.to_string())?;
        if path.to_str().is_none_or(|s| s.len() > 2048)
            || path.components().any(|c| matches!(c, Component::ParentDir))
        {
            return Err(
                "Content root requires a Unicode path up to2048bytes without parent traversal"
                    .into(),
            );
        }
        let id = if portable {
            "portable".into()
        } else {
            format!("external-{}", digest(path.to_str().unwrap().as_bytes()))
        };
        Ok(Self {
            id,
            path,
            scope: if portable { "portable" } else { "explicit" }.into(),
            file: None,
        })
    }
    pub fn single_file(path: &Path) -> Result<Self, String> {
        let path = std::path::absolute(path).map_err(|e| e.to_string())?;
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .filter(|s| relative(s))
            .ok_or("Select one ordinary Unicode filename")?
            .to_owned();
        let mut root = Self::new(path.parent().ok_or("File needs a parent directory")?, false)?;
        root.id = format!(
            "file-{}",
            digest(path.to_str().ok_or("File path is not Unicode")?.as_bytes())
        );
        root.scope = "file".into();
        root.file = Some(name);
        Ok(root)
    }
    fn validated(&self) -> Result<Self, String> {
        let result = match self.scope.as_str() {
            "portable" | "explicit" if self.file.is_none() => {
                Self::new(&self.path, self.scope == "portable")?
            }
            "file" => {
                let name = self
                    .file
                    .as_deref()
                    .filter(|s| relative(s) && Path::new(s).components().count() == 1)
                    .ok_or("Invalid single-file root")?;
                Self::single_file(&self.path.join(name))?
            }
            _ => return Err("Unknown root scope".into()),
        };
        if result.id != self.id {
            return Err("Invalid declared root identity".into());
        }
        Ok(result)
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Entry {
    pub id: String,
    pub root_id: String,
    pub relative_path: String,
    pub selection_relative: String,
    pub kind: Kind,
    pub name: String,
    pub description: String,
    pub bytes: Option<u64>,
    pub sha256: Option<String>,
    pub rom_sha256: Option<String>,
    pub region: Option<String>,
    pub availability: String,
    pub compatibility: String,
    pub reason: String,
    pub verification: String,
    pub duplicate_ids: Vec<String>,
    pub duplicate_count: usize,
}
#[derive(Default, Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Stats {
    pub visited_nodes: usize,
    pub skipped_files: usize,
    pub skipped_directories: usize,
    pub hashed_bytes: u64,
    pub metadata_bytes: u64,
    pub truncated: bool,
    pub dropped_issues: usize,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Index {
    pub schema: u32,
    pub kind: String,
    pub roots: Vec<Root>,
    pub rom_sha256: Option<String>,
    pub entries: Vec<Entry>,
    pub issues: Vec<String>,
    pub stats: Stats,
    pub limits: Value,
    pub verification: String,
}
fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn valid_hash(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}
fn short(s: &str, n: usize) -> String {
    s.chars().filter(|c| !c.is_control()).take(n).collect()
}
fn relative(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 2048
        && !s.contains(['\\', ':'])
        && Path::new(s)
            .components()
            .all(|p| matches!(p, Component::Normal(_)))
}
/// Check the whole declared chain; never follow a discovered symlink/junction.
pub(crate) fn ordinary(path: &Path) -> Result<fs::Metadata, String> {
    for ancestor in path.ancestors().filter(|p| !p.as_os_str().is_empty()) {
        crate::debug_session::regular(ancestor)?;
        // Wine can expose Unix symlinks as ordinary Windows attributes. Resolve
        // each component against its resolved parent before reading/enumerating.
        if let (Some(parent), Some(name)) = (ancestor.parent(), ancestor.file_name()) {
            if !parent.as_os_str().is_empty() {
                let actual = fs::canonicalize(ancestor).map_err(|e| e.to_string())?;
                let expected = fs::canonicalize(parent)
                    .map_err(|e| e.to_string())?
                    .join(name);
                #[cfg(windows)]
                let same = actual.to_string_lossy().to_lowercase()
                    == expected.to_string_lossy().to_lowercase();
                #[cfg(not(windows))]
                let same = actual == expected;
                if !same {
                    return Err("Content path contains a nonordinary link/alias".into());
                }
            }
        }
    }
    crate::debug_session::regular(path)
}
fn file_kind(path: &Path) -> Option<Kind> {
    let name = path.file_name()?.to_str()?.to_ascii_lowercase();
    Some(match name.as_str() {
        "debug-session.json" => Kind::Session,
        "recording.json" => Kind::Recording,
        "debug-labels.toml" => Kind::Labels,
        "native.bin" => Kind::NativeLibrary,
        "replay.json" => Kind::ReplayProfile,
        "catalog.json" | "debug-catalog.json" => Kind::SaveCatalog,
        "catalogo.json" => Kind::CheatCatalog,
        "manifest.toml" => Kind::Dataset,
        "recomp" | "python" | "python3" => Kind::Runtime,
        "script-commands.json" | "script-archive.json" | "auxiliary-scripts.json" | "auxiliary-script-archive.json" => {
            Kind::GuestScriptCatalog
        }
        _ => match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
            "input" => Kind::Input,
            "ffstate" => Kind::Snapshot,
            "exe" => Kind::Runtime,
            "dll" | "so" | "dylib" | "o" | "elf" => Kind::NativeLibrary,
            "labels" => Kind::Labels,
            "py" | "ps1" | "sh" => Kind::HostTool,
            "html" | "htm" => Kind::Report,
            "json" => Kind::Unknown,
            "toml" if name.contains("manifest") || name.contains("plan") => Kind::KnownManifest,
            _ => return None,
        },
    })
}
fn metadata_kind(v: &Value, fallback: Kind) -> Kind {
    match v["kind"].as_str().unwrap_or("") {
        "ffta-debug-session" => Kind::Session,
        "ffta-input-recording" => Kind::Recording,
        "ffta-debug-replay-profile" => Kind::ReplayProfile,
        "ffta-debug-recipe" => Kind::Recipe,
        "ffta-debug-campaign" => Kind::Campaign,
        k if k.starts_with("ffta-")
            && (k.ends_with("-report") || k.ends_with("-result") || k.ends_with("-audit")) =>
        {
            Kind::Report
        }
        _ => fallback,
    }
}
fn same_stamp(a: &fs::Metadata, b: &fs::Metadata) -> bool {
    if a.len() != b.len()
        || a.modified().ok() != b.modified().ok()
        || a.created().ok() != b.created().ok()
    {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if a.dev() != b.dev() || a.ino() != b.ino() {
            return false;
        }
    }
    true
}
fn read_candidate(
    path: &Path,
    meta: &fs::Metadata,
    metadata: bool,
    stats: &mut Stats,
) -> Result<(Option<String>, Option<Vec<u8>>), String> {
    let hash = meta.len() <= MAX_HASH_BYTES.saturating_sub(stats.hashed_bytes);
    let parse = metadata
        && meta.len() <= MAX_METADATA
        && meta.len() <= MAX_METADATA_TOTAL.saturating_sub(stats.metadata_bytes);
    if !hash && !parse {
        return Ok((None, None));
    }
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    if !same_stamp(meta, &file.metadata().map_err(|e| e.to_string())?) {
        return Err("File changed while opening".into());
    }
    let mut hasher = Sha256::new();
    let mut data = parse.then(Vec::new);
    let mut count = 0u64;
    let mut buf = [0u8; 16384];
    loop {
        let n = file.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        count += n as u64;
        if count > meta.len() {
            return Err("File grew while indexing".into());
        }
        if hash {
            hasher.update(&buf[..n]);
            stats.hashed_bytes += n as u64;
        }
        if let Some(data) = &mut data {
            data.extend_from_slice(&buf[..n]);
            stats.metadata_bytes += n as u64;
        }
    }
    let after = ordinary(path)?;
    if count != meta.len()
        || !same_stamp(meta, &after)
        || !same_stamp(meta, &file.metadata().map_err(|e| e.to_string())?)
    {
        return Err("File changed while indexing".into());
    }
    Ok((
        hash.then(|| {
            hasher
                .finalize()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect()
        }),
        data,
    ))
}
fn inspect(root: &Root, rel: &str, expected_rom: Option<&str>, stats: &mut Stats) -> Entry {
    let path = root.path.join(rel);
    let fallback = file_kind(&path).unwrap_or(Kind::Unknown);
    let mut entry = Entry {
        id: String::new(),
        root_id: root.id.clone(),
        relative_path: rel.into(),
        selection_relative: rel.into(),
        kind: fallback,
        name: path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .chars()
            .take(128)
            .collect(),
        description: "Listed only; action-specific preflight is still required.".into(),
        bytes: None,
        sha256: None,
        rom_sha256: None,
        region: None,
        availability: "available".into(),
        compatibility: "unknown".into(),
        reason: "Not a full content/runtime/ROM verification; nothing was executed.".into(),
        verification: "listed-only".into(),
        duplicate_ids: Vec::new(),
        duplicate_count: 0,
    };
    let result = (|| -> Result<(), String> {
        let meta = ordinary(&path)?;
        if !meta.is_file() {
            return Err("Expected an ordinary file".into());
        }
        entry.bytes = Some(meta.len());
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let structured = entry.kind != Kind::Labels && (ext == "json" || ext == "toml");
        let (sha, bytes) = read_candidate(&path, &meta, structured, stats)?;
        entry.sha256 = sha;
        if entry.sha256.is_some() {
            entry.verification = "content-hash-only".into();
        }
        if structured {
            let Some(bytes) = bytes else {
                entry.reason="Structured metadata exceeds per-file/scan budget; not parsed or authenticated.".into();
                return Ok(());
            };
            let value: Value = if ext == "json" {
                serde_json::from_slice(&bytes).map_err(|_| "Malformed JSON metadata")?
            } else {
                toml::from_str(std::str::from_utf8(&bytes).map_err(|_| "Metadata is not UTF-8")?)
                    .map_err(|_| "Malformed TOML metadata")?
            };
            if !value.is_object() {
                return Err("Metadata must be an object".into());
            }
            entry.kind = metadata_kind(&value, fallback);
            for key in ["name", "label", "case", "title"] {
                if let Some(s) = value[key].as_str().filter(|s| !s.is_empty()) {
                    entry.name = short(s, 128);
                    break;
                }
            }
            for key in ["description", "scope", "note"] {
                if let Some(s) = value[key].as_str() {
                    entry.description = short(s, 512);
                    break;
                }
            }
            entry.rom_sha256 = value["rom_sha256"]
                .as_str()
                .or(value["source_rom_sha256"].as_str())
                .or(value["pins"]["ffta.gba"].as_str())
                .filter(|s| valid_hash(s))
                .map(str::to_ascii_lowercase);
            entry.region = value["region"].as_str().map(|s| short(s, 32));
            let required_kind = match entry.kind {
                Kind::Session => Some("ffta-debug-session"),
                Kind::Recording => Some("ffta-input-recording"),
                Kind::ReplayProfile => Some("ffta-debug-replay-profile"),
                Kind::Recipe => Some("ffta-debug-recipe"),
                Kind::Campaign => Some("ffta-debug-campaign"),
                _ => None,
            };
            if required_kind.is_some_and(|k| value["kind"].as_str() != Some(k)) {
                entry.compatibility = "unsupported".into();
                entry.reason = "Filename/type and declared metadata kind disagree.".into();
            } else if value["schema"].as_u64() != Some(1) && entry.kind != Kind::GuestScriptCatalog {
                entry.compatibility = "unsupported".into();
                entry.reason =
                    "Metadata schema is missing or unsupported; listing cannot enable execution."
                        .into();
            } else if entry.kind == Kind::Unknown {
                entry.compatibility = "unsupported".into();
                entry.reason="Unrecognized metadata kind; consult/import only, no adapter or execution inferred.".into();
            }
        }
        if let (Some(expected), Some(actual)) = (expected_rom, entry.rom_sha256.as_deref()) {
            if actual != expected {
                entry.compatibility = "incompatible".into();
                entry.reason = "Declared ROM hash differs from the selected ROM.".into();
            } else if entry.compatibility == "unknown" {
                entry.compatibility = "declared-match".into();
                entry.reason="Declared ROM hash matches; payload pins, format, backend and feature preflight remain unchecked.".into();
            }
        }
        Ok(())
    })();
    if let Err(error) = result {
        let missing =
            fs::symlink_metadata(&path).is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound);
        entry.availability = if missing {
            "missing"
        } else if error.contains("changed") || error.contains("grew") {
            "changed"
        } else if error.contains("regular") || error.contains("ordinary") {
            "nonordinary"
        } else {
            "unreadable"
        }
        .into();
        entry.reason = short(&error, 512);
        entry.sha256 = None;
    }
    if matches!(entry.kind, Kind::Session | Kind::Recording | Kind::Dataset) {
        entry.selection_relative = Path::new(rel)
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map_or_else(|| ".".into(), |p| p.to_string_lossy().into_owned());
    }
    entry.id = format!(
        "content-{}",
        digest(format!("{}\0{}\0{}", root.id, entry.kind.name(), rel).as_bytes())
    );
    entry
}
impl Index {
    fn issue(&mut self, message: String) {
        if self.issues.len() < MAX_ISSUES {
            self.issues.push(message)
        } else {
            self.stats.dropped_issues += 1;
        }
    }
    pub fn scan(roots: Vec<Root>, rom: Option<String>) -> Result<Self, String> {
        if roots.is_empty()
            || roots.len() > MAX_ROOTS
            || rom.as_ref().is_some_and(|s| !valid_hash(s))
        {
            return Err("Select1..8 declared roots and an optional64hex ROM hash".into());
        }
        let mut index=Self {schema:1,kind:"ffta-debug-content-index".into(),roots,rom_sha256:rom.map(|s|s.to_ascii_lowercase()),entries:Vec::new(),issues:Vec::new(),stats:Stats::default(),
            limits:json!({"roots":MAX_ROOTS,"entries":MAX_ENTRIES,"nodes":MAX_NODES,"depth":MAX_DEPTH,"hash_bytes":MAX_HASH_BYTES,"metadata_file_bytes":MAX_METADATA,"metadata_total_bytes":MAX_METADATA_TOTAL,"issues":MAX_ISSUES,"duplicate_links_per_entry":8,"index_bytes":MAX_INDEX}),
            verification:"Explicit bounded refresh only; content hashes are not full verification or execution. Truncation means unvisited contents are unknown.".into()};
        let mut declared = BTreeMap::new();
        for root in &index.roots {
            root.validated()?;
            if declared
                .insert(root.id.clone(), root.path.clone())
                .is_some_and(|old| old != root.path)
            {
                return Err("Conflicting root identities".into());
            }
        }
        let mut seen = BTreeSet::new();
        for root in index.roots.clone() {
            if !["portable", "explicit", "file"].contains(&root.scope.as_str()) {
                return Err("Unknown root scope".into());
            }
            if let Err(error) = ordinary(&root.path).and_then(|m| {
                if m.is_dir() {
                    Ok(m)
                } else {
                    Err("Root must be a directory".into())
                }
            }) {
                index.issue(format!("{}: {}", root.id, short(&error, 512)));
                continue;
            }
            let mut canonical = fs::canonicalize(&root.path).map_err(|e| e.to_string())?;
            if let Some(file) = &root.file {
                canonical.push(file);
            }
            if !seen.insert(canonical) {
                index.issue(format!("{}: duplicate-root; scanned once", root.id));
                continue;
            }
            if let Some(file) = &root.file {
                if index.entries.len() >= MAX_ENTRIES || index.stats.visited_nodes >= MAX_NODES {
                    index.stats.truncated = true;
                    continue;
                }
                index.stats.visited_nodes += 1;
                index.entries.push(inspect(
                    &root,
                    file,
                    index.rom_sha256.as_deref(),
                    &mut index.stats,
                ));
                continue;
            }
            let mut pending = vec![(root.path.clone(), 0usize)];
            while let Some((dir, depth)) = pending.pop() {
                if index.entries.len() >= MAX_ENTRIES || index.stats.visited_nodes >= MAX_NODES {
                    index.stats.truncated = true;
                    break;
                }
                if depth > MAX_DEPTH {
                    index.stats.truncated = true;
                    index.stats.skipped_directories += 1;
                    continue;
                }
                if let Err(error) = ordinary(&dir) {
                    index.issue(short(&error, 512));
                    continue;
                }
                let iter = match fs::read_dir(&dir) {
                    Ok(i) => i,
                    Err(e) => {
                        index.issue(short(&e.to_string(), 512));
                        continue;
                    }
                };
                let remaining = MAX_NODES - index.stats.visited_nodes;
                let mut paths = Vec::new();
                for item in iter.take(remaining + 1) {
                    match item {
                        Ok(e) => paths.push(e.path()),
                        Err(e) => index.issue(short(&e.to_string(), 512)),
                    }
                }
                if paths.len() > remaining {
                    paths.truncate(remaining);
                    index.stats.truncated = true;
                }
                paths.sort();
                for path in paths {
                    index.stats.visited_nodes += 1;
                    let Some(rel) = path
                        .strip_prefix(&root.path)
                        .ok()
                        .and_then(|p| p.to_str())
                        .map(|s| s.replace('\\', "/"))
                    else {
                        index.issue("Non-Unicode content path skipped".into());
                        continue;
                    };
                    if !relative(&rel) {
                        index.issue("Unsupported/overlong relative content path skipped".into());
                        continue;
                    }
                    let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    let metadata = fs::symlink_metadata(&path);
                    let is_dir = metadata
                        .as_ref()
                        .is_ok_and(|m| m.is_dir() && !m.file_type().is_symlink());
                    if is_dir {
                        if [".git", ".retention", "target", "node_modules"].contains(&name)
                            || (root.scope == "portable"
                                && depth == 0
                                && !["Data", "scripts", "cheats", "reports"].contains(&name))
                            || (root.scope == "portable"
                                && depth == 1
                                && dir.file_name().is_some_and(|n| n == "Data")
                                && ![
                                    "debug-sessions",
                                    "config",
                                    "saves",
                                    "cache",
                                    "resources",
                                    "dataset",
                                ]
                                .contains(&name))
                        {
                            index.stats.skipped_directories += 1;
                        } else {
                            pending.push((path, depth + 1));
                        }
                    } else if file_kind(&path).is_some() {
                        if index.entries.len() == MAX_ENTRIES {
                            index.stats.truncated = true;
                            break;
                        }
                        index.entries.push(inspect(
                            &root,
                            &rel,
                            index.rom_sha256.as_deref(),
                            &mut index.stats,
                        ));
                    } else if metadata.as_ref().is_ok_and(|m| m.file_type().is_symlink()) {
                        index.issue(format!("Nonordinary link skipped: {}", short(&rel, 200)));
                    } else {
                        index.stats.skipped_files += 1;
                    }
                }
            }
        }
        index.entries.sort_by(|a, b| a.id.cmp(&b.id));
        let mut duplicate = BTreeMap::<(String, String), Vec<String>>::new();
        for e in &index.entries {
            if let Some(sha) = &e.sha256 {
                duplicate
                    .entry((e.kind.name(), sha.clone()))
                    .or_default()
                    .push(e.id.clone());
            }
        }
        for e in &mut index.entries {
            if let Some(sha) = &e.sha256 {
                let group = &duplicate[&(e.kind.name(), sha.clone())];
                e.duplicate_count = group.len() - 1;
                e.duplicate_ids = group
                    .iter()
                    .filter(|id| **id != e.id)
                    .take(8)
                    .cloned()
                    .collect();
            }
        }
        Ok(index)
    }
    pub fn resolve(&self, id: &str, kind: Option<Kind>) -> Result<(Entry, PathBuf), String> {
        let entry = self.recheck(id, kind)?;
        if entry.availability != "available"
            || entry.kind == Kind::Unknown
            || ["incompatible", "unsupported"].contains(&entry.compatibility.as_str())
        {
            return Err(format!("{}: {}", entry.availability, entry.reason));
        }
        let root = self
            .roots
            .iter()
            .find(|r| r.id == entry.root_id)
            .ok_or("Missing selected root")?;
        let path = root.path.join(&entry.selection_relative);
        Ok((entry, path))
    }
    pub fn recheck(&self, id: &str, kind: Option<Kind>) -> Result<Entry, String> {
        if self.schema != 1
            || self.kind != "ffta-debug-content-index"
            || self.roots.len() > MAX_ROOTS
            || self.entries.len() > MAX_ENTRIES
        {
            return Err("Unsupported/oversized content index".into());
        }
        let old = self
            .entries
            .iter()
            .find(|e| e.id == id)
            .ok_or("Unknown content ID")?;
        if !relative(&old.relative_path) {
            return Err("Invalid relative content reference".into());
        }
        let root = self
            .roots
            .iter()
            .find(|r| r.id == old.root_id)
            .ok_or("Content root missing from index")?;
        root.validated()?;
        if root
            .file
            .as_ref()
            .is_some_and(|file| *file != old.relative_path)
        {
            return Err("Reference escapes the selected single-file root".into());
        }
        let mut current = inspect(
            root,
            &old.relative_path,
            self.rom_sha256.as_deref(),
            &mut Stats::default(),
        );
        if current.availability == "available"
            && (current.id != old.id
                || old
                    .sha256
                    .as_ref()
                    .is_some_and(|sha| current.sha256.as_ref() != Some(sha)))
        {
            current.availability = "changed".into();
            current.reason =
                "Content kind or hash changed since selection; refresh and select again.".into();
        }
        current.id = old.id.clone();
        if old.sha256.is_none() && current.availability == "available" {
            current.reason="Original listing had no content hash; recheck cannot establish unchanged bytes. Full preflight is required.".into();
            current.verification = "unverified-selection".into();
        }
        if kind.is_some_and(|k| current.kind != k) {
            current.compatibility = "incompatible".into();
            current.reason = "Selected content type does not match the requested role.".into();
        }
        Ok(current)
    }
}
pub(crate) fn defaults() -> Result<Vec<Root>, String> {
    if let Some(root) = input_config::storage::portable_root() {
        return Ok(vec![Root::new(&root, true)?]);
    }
    let mut roots = Vec::new();
    if let Some(root) = input_config::exe_dir() {
        roots.push(Root::new(&root, true)?);
    }
    if let Some(root) = input_config::config_root() {
        roots.push(Root::new(&root, false)?);
    }
    if let Ok(root) = crate::rom_data::diagnosis::storage_root() {
        if !roots.iter().any(|r| r.path == root) {
            roots.push(Root::new(&root, false)?);
        }
    }
    if let Some(root) = dirs::cache_dir() {
        roots.push(Root::new(&root.join("FFTARecomp"), false)?);
    }
    Ok(roots)
}
fn execute(args: &[String]) -> Result<(Value, bool, u8), String> {
    let json_output = args.last().is_some_and(|s| s == "--json");
    let args = if json_output {
        &args[..args.len() - 1]
    } else {
        args
    };
    match args.first().map(String::as_str) {
        Some("scan")=>{
            let mut roots=None;let mut added=Vec::new();let mut rom=None;let mut it=args[1..].iter();
            while let Some(k)=it.next(){let v=it.next().ok_or("Content option needs a value")?;match k.as_str(){
                "--root" if roots.is_none()=>roots=Some(vec![Root::new(Path::new(v),true)?]),
                "--add-root" if added.len()<MAX_ROOTS=>added.push(Root::new(Path::new(v),false)?),
                "--add-file" if added.len()<MAX_ROOTS=>added.push(Root::single_file(Path::new(v))?),
                "--rom-sha256" if rom.is_none()=>rom=Some(v.clone()),_=>return Err("Use one --root/--rom-sha256 and up to8 --add-root directories / --add-file inputs".into())}}
            let mut roots=match roots{Some(r)=>r,None if !added.is_empty()=>Vec::new(),None=>defaults()?};roots.extend(added);
            Ok((serde_json::to_value(Index::scan(roots,rom)?).map_err(|e|e.to_string())?,json_output,0))
        },
        Some("check") if args.len()==3 || args.len()==5 && args[3]=="--kind"=>{
            let kind=if args.len()==5{Some(serde_json::from_value::<Kind>(json!(args[4])).map_err(|_|"Unknown content kind")?)}else{None};
            let path=Path::new(&args[1]);ordinary(path)?;let index:Index=serde_json::from_slice(&crate::diagnostics::read(path,MAX_INDEX)?).map_err(|e|e.to_string())?;
            let item=index.recheck(&args[2],kind)?;let code=if item.availability!="available" || ["incompatible","unsupported"].contains(&item.compatibility.as_str()){3}else{0};
            Ok((json!({"schema":1,"kind":"ffta-debug-content-check","outcome":if code==0{"listed"}else{"refused"},"entry":item,"verification":"Readonly recheck, not action preflight or execution"}),json_output,code))
        },
        _=>Err("Use debug-content scan [--root PORTABLE --add-root DIR --add-file FILE --rom-sha256 SHA] [--json], or check INDEX ID [--kind KIND] [--json]".into()),
    }
}
pub(crate) fn cmd(args: &[String]) -> u8 {
    match execute(args) {
        Ok((value, json_output, code)) => {
            if json_output {
                println!("{value}")
            } else {
                println!("{}", serde_json::to_string_pretty(&value).unwrap())
            }
            code
        }
        Err(message) => {
            println!(
                "{}",
                json!({"schema":1,"kind":"ffta-debug-content-error","outcome":"invalid-request","message":short(&message,2048)})
            );
            1
        }
    }
}
#[cfg(test)]
mod tests;
