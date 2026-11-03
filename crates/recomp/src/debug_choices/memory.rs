//! Runtime binding over the bounded serialized model; no tool execution API.
use super::*;
use crate::{
    debug_content::{Index, Root},
    debug_inputs::Reference,
};
use menu::{ContentHistory, ContentSlot as Slot, ContentViewPreferences};
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};
pub(crate) type Locations = BTreeMap<String, Location>;
pub(crate) struct Restored {
    pub context: String,
    pub references: BTreeMap<Slot, Reference>,
    pub issues: BTreeMap<Slot, String>,
    pub originals: BTreeMap<Slot, SavedInput>,
}
pub(crate) struct Memory {
    pub anchors: Anchors,
    pub context: String,
    pub file: FileState,
    root_bindings: BTreeMap<String, SavedRoot>,
    default_ids: Vec<String>,
    dirty: Option<Instant>,
    sampled: Option<Instant>,
    pub message: String,
}
fn scope(slot: Option<Slot>) -> String {
    slot.map_or_else(|| "all".into(), |s| format!("{s:?}"))
}
impl Memory {
    pub fn load(anchors: Anchors, rom: &str) -> Result<Self, String> {
        let file = FileState::load(&anchors)?;
        let context = format!(
            "{}\n{}\n{rom}",
            anchors.app.display(),
            anchors.config.display()
        );
        Ok(Self {
            anchors,
            context,
            file,
            root_bindings: BTreeMap::new(),
            default_ids: Vec::new(),
            dirty: None,
            sampled: None,
            message: "Saved choices loaded as host metadata; no scan or execution.".into(),
        })
    }
    pub fn roots(&mut self, defaults: Vec<Root>) -> Vec<Root> {
        self.default_ids = defaults.iter().map(|r| r.id.clone()).collect();
        let mut roots = defaults;
        for saved in &self.file.state.roots {
            if saved.origin == "portable" {
                continue;
            }
            let result = (|| {
                let path = saved.location.resolve(&self.anchors)?;
                if saved.origin == "added-file" {
                    Root::single_file(&path)
                } else {
                    Root::new(&path, false)
                }
            })();
            match result {
                Ok(root) => {
                    let mut binding = saved.clone();
                    binding.previous_id = root.id.clone();
                    self.root_bindings.insert(root.id.clone(), binding);
                    if !roots.iter().any(|r| r.id == root.id) && roots.len() < 8 {
                        roots.push(root);
                    }
                }
                Err(e) => self.message = format!("Saved root unavailable: {e}"),
            }
        }
        roots
    }
    pub fn record_roots(&mut self, roots: &[Root]) {
        let mut saved = Vec::new();
        for root in roots {
            let path = root
                .file
                .as_ref()
                .map_or_else(|| root.path.clone(), |name| root.path.join(name));
            let origin = if root.scope == "portable" {
                "portable"
            } else if root.scope == "file" {
                "added-file"
            } else if self.default_ids.contains(&root.id) {
                "app-default"
            } else {
                "added-folder"
            };
            let item = Location::capture(&path, &self.anchors)
                .map(|location| SavedRoot {
                    previous_id: root.id.clone(),
                    location,
                    origin: origin.into(),
                })
                .or_else(|_| {
                    self.root_bindings
                        .get(&root.id)
                        .cloned()
                        .ok_or("New root is missing; it has not been persisted".to_owned())
                });
            if let Ok(item) = item {
                self.root_bindings.insert(root.id.clone(), item.clone());
                saved.push(item);
            }
        }
        if serde_json::to_value(&saved).ok() != serde_json::to_value(&self.file.state.roots).ok() {
            self.file.state.roots = saved;
            self.dirty = Some(Instant::now());
        }
    }
    pub fn record_input(&mut self, slot: Slot, reference: &Reference) {
        match SavedInput::capture(slot, reference, &self.anchors) {
            Ok(saved) => {
                self.file.state.inputs.retain(|r| r.slot != saved.slot);
                self.file.state.inputs.push(saved);
                self.dirty = Some(Instant::now());
            }
            Err(e) => {
                self.message = format!("Choice remains active but could not be persisted: {e}")
            }
        }
    }
    pub fn restore_request(&self) -> (String, Anchors, Vec<SavedInput>) {
        (
            self.context.clone(),
            self.anchors.clone(),
            self.file.state.inputs.clone(),
        )
    }
    pub fn restore(
        context: String,
        anchors: Anchors,
        inputs: Vec<SavedInput>,
        rom: String,
    ) -> Restored {
        let mut out = Restored {
            context,
            references: BTreeMap::new(),
            issues: BTreeMap::new(),
            originals: inputs
                .iter()
                .filter_map(|s| s.slot().ok().map(|slot| (slot, s.clone())))
                .collect(),
        };
        let mut left = 64 * 1024 * 1024u64;
        for saved in inputs {
            let Ok(slot) = saved.slot() else { continue };
            let result = (|| {
                let path = saved.file.resolve(&anchors)?;
                let meta = crate::debug_content::ordinary(&path)?;
                if !meta.is_file() {
                    return Err("Saved input is no longer an ordinary file".into());
                }
                if meta.len() > left {
                    return Err("Saved-input primary-file budget reached64MiB; select this role explicitly to verify".into());
                }
                left -= meta.len();
                saved.restore(&anchors, &rom)
            })();
            match result {
                Ok(reference) => {
                    out.references.insert(slot, reference);
                }
                Err(e) => {
                    out.issues
                        .insert(slot, format!("Saved choice unavailable: {e}"));
                }
            }
        }
        out
    }
    pub fn locations(index: &Index, anchors: &Anchors) -> Locations {
        let mut out = Locations::new();
        for entry in &index.entries {
            if let Some(root) = index.roots.iter().find(|r| r.id == entry.root_id) {
                if let Ok(loc) = Location::capture(&root.path.join(&entry.relative_path), anchors) {
                    out.insert(entry.id.clone(), loc);
                }
            }
        }
        out
    }
    fn display_id(&self, old: &str, locations: &Locations) -> String {
        if locations.contains_key(old) {
            return old.into();
        }
        self.file
            .state
            .bookmarks
            .iter()
            .find(|b| b.previous_id == old)
            .and_then(|b| {
                locations
                    .iter()
                    .find(|(_, loc)| **loc == b.file)
                    .map(|(id, _)| id.clone())
            })
            .unwrap_or_else(|| old.into())
    }
    pub fn history(&self, locations: &Locations) -> ContentHistory {
        let mut h = ContentHistory::default();
        let mut recent = Vec::new();
        for b in &self.file.state.bookmarks {
            let id = self.display_id(&b.previous_id, locations);
            if b.favorite && !h.favorites.contains(&id) {
                h.favorites.push(id.clone());
            }
            if let Some(rank) = b.recent {
                recent.push((rank, id));
            }
        }
        recent.sort();
        for (_, id) in recent {
            if !h.recent.contains(&id) {
                h.recent.push(id);
            }
        }
        h.recent.truncate(32);
        h
    }
    pub fn view(&self, slot: Option<Slot>, locations: &Locations) -> ContentViewPreferences {
        let mut p = ContentViewPreferences::default();
        if let Some(v) = self
            .file
            .state
            .views
            .iter()
            .find(|v| v.scope == scope(slot))
        {
            p.query = v.query.clone();
            p.kind = v.kind.clone();
            p.filter = v.filter;
            p.sort = v.sort;
            p.collection = v.collection;
            p.highlighted = v
                .highlighted
                .as_ref()
                .map(|s| self.display_id(s, locations));
        }
        p
    }
    pub fn confirmed_id(&self, slot: Slot, locations: &Locations) -> Option<String> {
        self.file
            .state
            .inputs
            .iter()
            .find(|s| s.slot == scope(Some(slot)))
            .and_then(|s| {
                locations
                    .iter()
                    .find(|(_, loc)| **loc == s.file)
                    .map(|(id, _)| id.clone())
            })
    }
    pub fn observe(
        &mut self,
        slot: Option<Slot>,
        view: ContentViewPreferences,
        history: ContentHistory,
        locations: &Locations,
        force: bool,
    ) {
        let now = Instant::now();
        if !force
            && self
                .sampled
                .is_some_and(|t| now.duration_since(t) < Duration::from_millis(200))
        {
            return;
        }
        self.sampled = Some(now);
        let before = self.file.state.encode().ok();
        let key = scope(slot);
        self.file.state.views.retain(|v| v.scope != key);
        self.file.state.views.push(View {
            scope: key,
            query: view.query,
            kind: view.kind,
            filter: view.filter,
            sort: view.sort,
            collection: view.collection,
            highlighted: view.highlighted,
        });
        let old = self.file.state.bookmarks.clone();
        let mut wanted = history.favorites.clone();
        wanted.extend(history.recent.clone());
        wanted.extend(
            self.file
                .state
                .views
                .iter()
                .filter_map(|v| v.highlighted.clone()),
        );
        wanted.sort();
        wanted.dedup();
        let mut bookmarks = Vec::new();
        for id in wanted {
            let loc = locations.get(&id).cloned().or_else(|| {
                old.iter()
                    .find(|b| b.previous_id == id)
                    .map(|b| b.file.clone())
            });
            if let Some(file) = loc {
                bookmarks.push(Bookmark {
                    favorite: history.favorites.contains(&id),
                    recent: history.recent.iter().position(|r| *r == id),
                    previous_id: id,
                    file,
                });
            }
        }
        self.file.state.bookmarks = bookmarks;
        if self.file.state.encode().ok() != before {
            self.dirty = Some(now);
        }
    }
    pub fn flush(&mut self, force: bool) {
        if !force
            && !self
                .dirty
                .is_some_and(|t| t.elapsed() >= Duration::from_millis(400))
        {
            return;
        }
        match self.file.save() {
            Ok(()) => {
                self.message = "Choices saved; restoring them never starts a tool.".into();
            }
            Err(e) => {
                self.message = format!("Preferences not saved; previous file preserved: {e}");
            }
        }
        self.dirty = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_internal_roots_and_bookmarks_rebase_after_a_unicode_folder_move() {
        let t = tempfile::tempdir().unwrap();
        let app = t.path().join("Before è");
        let extra = app.join("Explicit è folder");
        std::fs::create_dir_all(&extra).unwrap();
        std::fs::create_dir_all(app.join("Data/config")).unwrap();
        std::fs::write(
            extra.join("NEVER.py"),
            b"raise RuntimeError('favorite must never execute')",
        )
        .unwrap();
        let a = Anchors {
            app: app.clone(),
            config: app.join("Data/config"),
        };
        let mut memory = Memory::load(a.clone(), &"a".repeat(64)).unwrap();
        let mut roots = memory.roots(vec![Root::new(&app, true).unwrap()]);
        roots.push(Root::new(&extra, false).unwrap());
        memory.record_roots(&roots);
        assert_eq!(memory.file.state.roots[1].origin, "added-folder");
        assert!(matches!(
            memory.file.state.roots[1].location,
            Location::App(_)
        ));
        let index = Index::scan(roots, Some("a".repeat(64))).unwrap();
        let entry = index
            .entries
            .iter()
            .find(|e| e.relative_path == "NEVER.py")
            .unwrap();
        let old = entry.id.clone();
        let locs = Memory::locations(&index, &a);
        memory.observe(
            None,
            ContentViewPreferences {
                query: "NEVER".into(),
                collection: 1,
                highlighted: Some(old.clone()),
                ..Default::default()
            },
            ContentHistory {
                favorites: vec![old.clone()],
                recent: vec![old.clone()],
            },
            &locs,
            true,
        );
        memory.flush(true);
        assert!(memory.file.path.exists());
        let moved = t.path().join("After move è spaces");
        std::fs::rename(&app, &moved).unwrap();
        let a = Anchors {
            app: moved.clone(),
            config: moved.join("Data/config"),
        };
        let mut memory = Memory::load(a.clone(), &"b".repeat(64)).unwrap();
        let roots = memory.roots(vec![Root::new(&moved, true).unwrap()]);
        let index = Index::scan(roots, Some("b".repeat(64))).unwrap();
        let new = index
            .entries
            .iter()
            .find(|e| e.relative_path == "NEVER.py")
            .unwrap()
            .id
            .clone();
        assert_ne!(old, new);
        let locs = Memory::locations(&index, &a);
        assert_eq!(
            memory.history(&locs),
            ContentHistory {
                favorites: vec![new.clone()],
                recent: vec![new.clone()]
            }
        );
        assert_eq!(memory.view(None, &locs).highlighted, Some(new));
        assert!(!moved.join("Data/config/debug-tool-runs").exists());
    }
    #[test]
    fn missing_pins_remain_saved_and_detected_external_preference_edits_are_preserved() {
        let t = tempfile::tempdir().unwrap();
        let a = Anchors {
            app: t.path().into(),
            config: t.path().join("Data/config"),
        };
        let path = t.path().join("runtime.exe");
        std::fs::write(&path, b"PUBLIC NOT EXECUTABLE").unwrap();
        let reference = Reference::explicit(&path, Slot::RuntimeB, &"a".repeat(64)).unwrap();
        let mut memory = Memory::load(a, &"a".repeat(64)).unwrap();
        memory.record_input(Slot::RuntimeB, &reference);
        memory.flush(true);
        let pin = memory.file.state.inputs[0].sha256.clone();
        std::fs::remove_file(path).unwrap();
        let (c, a, pins) = memory.restore_request();
        let result = Memory::restore(c, a, pins, "a".repeat(64));
        assert!(result.references.is_empty());
        assert!(result.issues.contains_key(&Slot::RuntimeB));
        assert_eq!(memory.file.state.inputs[0].sha256, pin);
        std::fs::write(&memory.file.path, b"USER MODIFIED").unwrap();
        memory.flush(true);
        assert!(memory.message.contains("previous file preserved"));
        assert_eq!(std::fs::read(&memory.file.path).unwrap(), b"USER MODIFIED");
    }
}
