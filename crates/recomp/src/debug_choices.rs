//! Versioned host preferences. Decoding/rebasing never resolves or starts content.
use crate::debug_content::Kind;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    path::{Component, Path, PathBuf},
};
pub(crate) const MAX_BYTES: usize = 256 * 1024;
const MAX_PATH: usize = 2048;
#[derive(Clone, Debug)]
pub(crate) struct Anchors {
    pub app: PathBuf,
    pub config: PathBuf,
}
impl Anchors {
    pub fn current() -> Result<Self, String> {
        Ok(Self {
            app: input_config::storage::portable_root()
                .or_else(input_config::exe_dir)
                .ok_or("No app root for saved choices")?,
            config: input_config::config_root().ok_or("No config root for saved choices")?,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(
    tag = "root",
    content = "path",
    rename_all = "kebab-case",
    deny_unknown_fields
)]
pub(crate) enum Location {
    App(String),
    Config(String),
    External(String),
}
fn relative(value: &str) -> bool {
    value == "."
        || (!value.is_empty()
            && value.len() <= MAX_PATH
            && !value.contains(['\\', ':'])
            && !value.chars().any(char::is_control)
            && Path::new(value)
                .components()
                .all(|c| matches!(c, Component::Normal(_)))
            && !value
                .split('/')
                .any(|p| p.is_empty() || p == "." || p == ".."))
}
fn portable_relative(path: &Path, base: &Path) -> Option<String> {
    let name = path.strip_prefix(base).ok()?.to_str()?.replace('\\', "/");
    let name = if name.is_empty() { ".".into() } else { name };
    relative(&name).then_some(name)
}
impl Location {
    pub fn capture(path: &Path, anchors: &Anchors) -> Result<Self, String> {
        // Inputs have already passed the content service's ordinary-file guard.
        // Canonical bases normalize drive/extended spelling before containment.
        crate::debug_content::ordinary(path)?;
        let path = std::fs::canonicalize(path).map_err(|e| e.to_string())?;
        let app = std::fs::canonicalize(&anchors.app).map_err(|e| e.to_string())?;
        let loc = if let Some(p) = portable_relative(&path, &app) {
            Self::App(p)
        } else {
            let config = match std::fs::canonicalize(&anchors.config) {
                Ok(p) => Some(p),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
                Err(e) => return Err(e.to_string()),
            };
            if let Some(p) = config
                .as_ref()
                .and_then(|base| portable_relative(&path, base))
            {
                Self::Config(p)
            } else {
                Self::External(path.to_str().ok_or("Saved path is not Unicode")?.into())
            }
        };
        loc.validate()?;
        Ok(loc)
    }
    pub fn validate(&self) -> Result<(), String> {
        let valid = match self {
            Self::App(p) | Self::Config(p) => relative(p),
            Self::External(p) => {
                p.len() <= MAX_PATH
                    && !p.chars().any(char::is_control)
                    && Path::new(p).is_absolute()
                    && !Path::new(p)
                        .components()
                        .any(|c| matches!(c, Component::ParentDir))
            }
        };
        if valid {
            Ok(())
        } else {
            Err("Invalid saved choice location; use a bounded relative internal or absolute explicit external path".into())
        }
    }
    pub fn resolve(&self, anchors: &Anchors) -> Result<PathBuf, String> {
        self.validate()?;
        // No stat, scan, dialog or fallback here. Missing choices stay explainable.
        Ok(match self {
            Self::App(p) => anchors.app.join(p),
            Self::Config(p) => anchors.config.join(p),
            Self::External(p) => p.into(),
        })
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SavedRoot {
    pub previous_id: String,
    pub location: Location,
    /// Preserve explicit file/folder provenance even when located inside app.
    pub origin: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SavedInput {
    pub slot: String,
    pub file: Location,
    pub sha256: String,
    pub kind: Kind,
}
impl SavedInput {
    pub fn capture(
        slot: menu::ContentSlot,
        reference: &crate::debug_inputs::Reference,
        anchors: &Anchors,
    ) -> Result<Self, String> {
        Ok(Self {
            slot: format!("{slot:?}"),
            file: Location::capture(&reference.source_file()?, anchors)?,
            sha256: reference
                .entry
                .sha256
                .clone()
                .ok_or("Cannot persist an unhashed reference")?,
            kind: reference.entry.kind,
        })
    }
    pub fn slot(&self) -> Result<menu::ContentSlot, String> {
        menu::ContentSlot::ALL
            .into_iter()
            .find(|s| format!("{s:?}") == self.slot)
            .ok_or("Unknown saved input role".into())
    }
    pub fn restore(
        &self,
        anchors: &Anchors,
        rom: &str,
    ) -> Result<crate::debug_inputs::Reference, String> {
        let reference = crate::debug_inputs::Reference::explicit(
            &self.file.resolve(anchors)?,
            self.slot()?,
            rom,
        )?;
        if reference.entry.kind != self.kind
            || reference.entry.sha256.as_deref() != Some(self.sha256.as_str())
        {
            return Err("Saved input changed kind or SHA256; select it again explicitly".into());
        }
        Ok(reference)
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct View {
    pub scope: String,
    pub query: String,
    pub kind: String,
    pub filter: usize,
    pub sort: usize,
    pub collection: usize,
    pub highlighted: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Bookmark {
    pub previous_id: String,
    pub file: Location,
    pub favorite: bool,
    pub recent: Option<usize>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct State {
    pub schema: u32,
    pub kind: String,
    pub roots: Vec<SavedRoot>,
    pub inputs: Vec<SavedInput>,
    pub views: Vec<View>,
    pub bookmarks: Vec<Bookmark>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            schema: 1,
            kind: "ffta-debug-choices".into(),
            roots: Vec::new(),
            inputs: Vec::new(),
            views: Vec::new(),
            bookmarks: Vec::new(),
        }
    }
}
fn hash(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 80
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-._".contains(&b))
}
impl State {
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > MAX_BYTES {
            return Err("Saved choices exceed256KiB".into());
        }
        let state: Self =
            serde_json::from_slice(bytes).map_err(|e| format!("Invalid saved choices: {e}"))?;
        state.validate()?;
        Ok(state)
    }
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let bytes = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        if bytes.len() > MAX_BYTES {
            return Err("Saved choices exceed256KiB; previous state must be preserved".into());
        }
        Ok(bytes)
    }
    fn validate(&self) -> Result<(), String> {
        if self.schema != 1
            || self.kind != "ffta-debug-choices"
            || self.roots.len() > 8
            || self.inputs.len() > menu::ContentSlot::ALL.len()
            || self.views.len() > menu::ContentSlot::ALL.len() + 1
            || self.bookmarks.len() > 562
        {
            return Err("Unsupported/oversized saved choices schema".into());
        }
        let slots = menu::ContentSlot::ALL
            .into_iter()
            .map(|s| format!("{s:?}"))
            .collect::<BTreeSet<_>>();
        let mut unique = BTreeSet::new();
        for r in &self.roots {
            r.location.validate()?;
            if !id(&r.previous_id)
                || !unique.insert(r.previous_id.clone())
                || !["portable", "added-file", "added-folder", "app-default"]
                    .contains(&r.origin.as_str())
            {
                return Err("Invalid/duplicate saved root".into());
            }
        }
        unique.clear();
        for r in &self.inputs {
            r.file.validate()?;
            if !slots.contains(&r.slot) || !unique.insert(r.slot.clone()) || !hash(&r.sha256) {
                return Err("Invalid/duplicate saved input role/hash".into());
            }
        }
        unique.clear();
        for v in &self.views {
            if !(v.scope == "all" || slots.contains(&v.scope))
                || !unique.insert(v.scope.clone())
                || v.query.chars().count() > 256
                || v.kind.len() > 64
                || v.filter >= 4
                || v.sort >= 3
                || v.collection >= 3
                || v.highlighted.as_deref().is_some_and(|s| !id(s))
            {
                return Err("Invalid/duplicate saved selector view".into());
            }
        }
        unique.clear();
        let mut recent = BTreeSet::new();
        let mut favorites = 0;
        for b in &self.bookmarks {
            b.file.validate()?;
            favorites += usize::from(b.favorite);
            if !id(&b.previous_id)
                || !unique.insert(b.previous_id.clone())
                || b.recent.is_some_and(|r| r >= 32 || !recent.insert(r))
            {
                return Err("Invalid/duplicate saved bookmark".into());
            }
        }
        if favorites > 512 {
            return Err("Too many saved favorites".into());
        }
        Ok(())
    }
}
pub(crate) struct FileState {
    pub state: State,
    pub path: PathBuf,
    digest: Option<String>,
}
fn ordinary_existing_tree(path: &Path) -> Result<(), String> {
    let mut p = PathBuf::new();
    for c in path.components() {
        p.push(c);
        // A Windows prefix alone names a volume (e.g. \\?\Z:), not a
        // directory. Validate the rooted path after RootDir is appended.
        if matches!(c, Component::Prefix(_)) {
            continue;
        }
        match std::fs::symlink_metadata(&p) {
            Ok(_) => {
                crate::debug_content::ordinary(&p)?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(())
}
impl FileState {
    pub fn load(anchors: &Anchors) -> Result<Self, String> {
        let path = anchors.config.join("debug-choices.json");
        ordinary_existing_tree(&path)?;
        match std::fs::symlink_metadata(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self {
                state: State::default(),
                path,
                digest: None,
            }),
            Err(e) => Err(e.to_string()),
            Ok(_) => {
                let bytes = crate::diagnostics::read(&path, MAX_BYTES as u64)?;
                Ok(Self {
                    state: State::decode(&bytes)?,
                    path,
                    digest: Some(gba_core::ffta_rom::digest(&bytes)),
                })
            }
        }
    }
    pub fn save(&mut self) -> Result<(), String> {
        use std::io::Write;
        let bytes = self.state.encode()?;
        ordinary_existing_tree(&self.path)?;
        let actual = match std::fs::symlink_metadata(&self.path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e.to_string()),
            Ok(_) => Some(gba_core::ffta_rom::digest(&crate::diagnostics::read(
                &self.path,
                MAX_BYTES as u64,
            )?)),
        };
        if actual != self.digest {
            return Err(
                "Preferences changed externally; previous file preserved. Reopen before saving."
                    .into(),
            );
        }
        let next = gba_core::ffta_rom::digest(&bytes);
        if self.digest.as_deref() == Some(&next) {
            return Ok(());
        }
        let parent = self
            .path
            .parent()
            .ok_or("Preferences need a config parent")?;
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        temporary
            .write_all(&bytes)
            .and_then(|_| temporary.as_file().sync_all())
            .map_err(|e| e.to_string())?;
        // Same-directory atomic publication; never remove the old file first.
        temporary.persist(&self.path).map_err(|e| e.to_string())?;
        self.digest = Some(next);
        Ok(())
    }
}

pub(crate) mod memory;
#[cfg(test)]
mod tests;
