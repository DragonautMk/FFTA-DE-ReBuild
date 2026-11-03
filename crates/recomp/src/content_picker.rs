//! Host-only picker bridge. Tree scans/dialogs/actions stay explicit; saved pins
//! receive a bounded readonly file check and preferences never start content.
use crate::debug_content::{Index, Kind, Root, MAX_ROOTS};
use crate::debug_inputs::{self, Reference};
use menu::ContentSlot;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
    thread::JoinHandle,
};

enum ResultData {
    Restored(crate::debug_choices::memory::Restored),
    Scan(Index, crate::debug_choices::memory::Locations),
    Select(Reference),
    Preview(String, Vec<[u8; 4]>, String),
}
struct Job {
    view: u64,
    handle: JoinHandle<Result<ResultData, String>>,
}
struct IdJob {
    generation: u64,
    context: String,
    index: Arc<Index>,
    slot: ContentSlot,
    cancelled: bool,
    handle: JoinHandle<Result<Reference, String>>,
}
struct ScanJob {
    context: String,
    generation: u64,
    cancelled: bool,
    handle: JoinHandle<Result<(Index, crate::debug_choices::memory::Locations), String>>,
}
#[derive(Default)]
pub(crate) struct Picker {
    roots: Vec<Root>,
    index: Option<Arc<Index>>,
    selected: Option<Reference>,
    slot: Option<ContentSlot>,
    slots: BTreeMap<ContentSlot, Reference>,
    job: Option<Job>,
    id_job: Option<IdJob>,
    scan_job: Option<ScanJob>,
    generation: u64,
    view: u64,
    rom: Option<String>,
    context: Option<String>,
    view_open: bool,
    memory: Option<crate::debug_choices::memory::Memory>,
    locations: crate::debug_choices::memory::Locations,
    saved_issues: BTreeMap<ContentSlot, String>,
    restore_pending: bool,
    persistence_error: String,
}
fn rows(index: &Index) -> Vec<menu::ContentEntry> {
    index
        .entries
        .iter()
        .map(|e| menu::ContentEntry {
            id: e.id.clone(),
            name: e.name.clone(),
            kind: e.kind.name(),
            description: e.description.clone(),
            location: format!("{} / {}", e.root_id, e.relative_path),
            identity: format!(
                "SHA256 {} | ROM {} | {}",
                e.sha256.as_deref().unwrap_or("not hashed"),
                e.rom_sha256.as_deref().unwrap_or("unknown"),
                e.region.as_deref().unwrap_or("region unknown")
            ),
            status: e.availability.clone(),
            compatibility: e.compatibility.clone(),
            reason: format!("{} | {}", e.verification, e.reason),
            available: e.availability == "available",
        })
        .collect()
}
impl Picker {
    pub fn initialized(&self) -> bool {
        self.context.is_some()
    }
    pub fn initialize(&mut self, menu: &mut menu::Menu, rom: String) -> Result<(), String> {
        use crate::debug_choices::{memory::Memory, Anchors};
        let anchors = Anchors::current()?;
        let context = format!(
            "{}\n{}\n{rom}",
            anchors.app.display(),
            anchors.config.display()
        );
        if self.context.as_ref() == Some(&context) {
            return Ok(());
        }
        let defaults = if self.context.is_none() && !self.roots.is_empty() {
            self.roots.clone()
        } else {
            crate::debug_content::defaults()?
        };
        self.persist_now(menu);
        self.context = Some(context);
        self.rom = Some(rom.clone());
        self.view = self.view.wrapping_add(1);
        self.slots.clear();
        self.selected = None;
        self.index = None;
        self.locations.clear();
        self.saved_issues.clear();
        self.restore_pending = false;
        match Memory::load(anchors, &rom) {
            Ok(mut memory) => {
                self.roots = memory.roots(defaults);
                self.restore_pending = !memory.file.state.inputs.is_empty();
                for saved in &memory.file.state.inputs {
                    if let Ok(slot) = saved.slot() {
                        self.saved_issues.insert(
                            slot,
                            "Saved choice: waiting for readonly file verification".into(),
                        );
                    }
                }
                self.persistence_error.clear();
                self.memory = Some(memory);
            }
            Err(e) => {
                self.roots = defaults;
                self.memory = None;
                self.persistence_error = format!(
                    "Saved preferences unavailable; file preserved, choices remain volatile: {e}"
                );
                menu.diagnosis_result(self.persistence_error.clone());
            }
        }
        self.scope(menu);
        self.begin_restore(menu)?;
        Ok(())
    }
    fn begin_restore(&mut self, menu: &mut menu::Menu) -> Result<(), String> {
        if !self.restore_pending || self.busy() {
            return Ok(());
        }
        let memory = self
            .memory
            .as_ref()
            .ok_or("Saved choice context unavailable")?;
        let (context, anchors, inputs) = memory.restore_request();
        let rom = self.rom.clone().ok_or("No ROM context")?;
        self.restore_pending = false;
        self.start(menu, move || {
            Ok(ResultData::Restored(
                crate::debug_choices::memory::Memory::restore(context, anchors, inputs, rom),
            ))
        })
    }
    fn apply_preferences(&self, menu: &mut menu::Menu) {
        if let Some(memory) = &self.memory {
            let mut view = memory.view(self.slot, &self.locations);
            if view.highlighted.is_none() {
                view.highlighted = menu.content_highlight().map(str::to_owned);
            }
            let _ = menu.restore_content_preferences(view, memory.history(&self.locations));
        }
    }
    fn observe_preferences(&mut self, menu: &menu::Menu, force: bool) {
        if let Some(memory) = &mut self.memory {
            let (view, history) = menu.content_preferences();
            memory.observe(self.slot, view, history, &self.locations, force);
        }
    }
    pub fn persist_tick(&mut self, menu: &mut menu::Menu) -> bool {
        let mut changed = false;
        if menu.content_active() {
            self.observe_preferences(menu, false);
        }
        if let Some(memory) = &mut self.memory {
            let previous = memory.message.clone();
            memory.flush(false);
            if memory.message != previous && memory.message.starts_with("Preferences not saved") {
                changed = true;
                if menu.content_active() {
                    menu.content_message(memory.message.clone());
                } else {
                    menu.diagnosis_result(memory.message.clone());
                }
            }
        }
        if let Err(e) = self.begin_restore(menu) {
            changed |= self.persistence_error != e;
            self.persistence_error = e;
        }
        changed
    }
    pub fn persist_now(&mut self, menu: &menu::Menu) {
        if menu.content_active() {
            self.observe_preferences(menu, true);
        }
        if let Some(memory) = &mut self.memory {
            memory.flush(true);
        }
    }
    pub fn leave_view(&mut self, menu: &menu::Menu) {
        if self.view_open {
            self.observe_preferences(menu, true);
            self.view_open = false;
        }
        self.cancel();
    }
    pub fn open(&mut self, menu: &mut menu::Menu, rom: String) -> Result<(), String> {
        if menu.content_active() {
            self.observe_preferences(menu, true);
        }
        self.initialize(menu, rom)?;
        self.view = self.view.wrapping_add(1);
        self.slot = None;
        self.view_open = true;
        if self.roots.is_empty() {
            self.roots = crate::debug_content::defaults()?;
        }
        let entries = self.index.as_ref().map_or_else(Vec::new, |i| rows(i));
        menu.open_content(
            "Debug inputs".into(),
            entries,
            "Refresh scans declared roots. Highlighting never runs a tool.".into(),
            self.generation,
        );
        self.scope(menu);
        self.apply_preferences(menu);
        if !self.persistence_error.is_empty() {
            menu.content_message(self.persistence_error.clone());
        }
        menu.content_busy(self.busy());
        Ok(())
    }
    pub fn open_for(
        &mut self,
        menu: &mut menu::Menu,
        rom: String,
        slot: ContentSlot,
        message: String,
    ) -> Result<(), String> {
        self.open(menu, rom)?;
        self.slot = Some(slot);
        menu.open_content(
            slot.label().into(),
            self.index.as_ref().map_or_else(Vec::new, |i| rows(i)),
            String::new(),
            self.generation,
        );
        self.scope(menu);
        self.apply_preferences(menu);
        let current = self
            .slots
            .get(&slot)
            .map(|s| {
                format!(
                    "Current: {} | {} | {}",
                    s.entry.name, s.entry.id, s.entry.relative_path
                )
            })
            .unwrap_or_else(|| {
                self.saved_issues
                    .get(&slot)
                    .cloned()
                    .unwrap_or_else(|| "No current selection".into())
            });
        menu.content_message(format!(
            "{} | {} | Select only; return and invoke the tool separately.",
            message, current
        ));
        Ok(())
    }
    fn scope(&self, menu: &mut menu::Menu) {
        menu.content_scope(
            self.slot.map_or_else(Vec::new, |s| {
                debug_inputs::kinds(s)
                    .into_iter()
                    .map(|k| k.name())
                    .collect()
            }),
            self.slot
                .and_then(|s| self.slots.get(&s))
                .or(self.selected.as_ref().filter(|_| self.slot.is_none()))
                .map(|s| {
                    self.slot
                        .and_then(|slot| {
                            self.memory
                                .as_ref()
                                .and_then(|m| m.confirmed_id(slot, &self.locations))
                        })
                        .unwrap_or_else(|| s.entry.id.clone())
                }),
        );
        menu.set_tool_inputs(
            ContentSlot::ALL
                .into_iter()
                .filter_map(|slot| {
                    self.slots
                        .get(&slot)
                        .map(|s| {
                            (
                                slot,
                                format!(
                                    "{} | {} | needs action preflight",
                                    s.entry.name,
                                    &s.entry.id[8..16]
                                ),
                            )
                        })
                        .or_else(|| {
                            self.saved_issues
                                .get(&slot)
                                .map(|message| (slot, message.clone()))
                        })
                })
                .collect(),
        );
    }
    pub fn references(&self) -> BTreeMap<ContentSlot, Reference> {
        self.slots.clone()
    }
    pub fn reference(&self,slot:ContentSlot)->Option<&Reference>{self.slots.get(&slot)}
    pub fn rom_context(&self)->Option<&str>{self.rom.as_deref()}
    pub fn completion_ids(&self,slot:ContentSlot) -> Vec<String> {
        self.index.as_ref().map_or_else(Vec::new, |index| index.entries.iter()
            .filter(|entry| debug_inputs::kinds(slot).contains(&entry.kind) && entry.availability=="available")
            .map(|entry|entry.id.clone()).collect())
    }
    pub fn has_slot(&self, slot: ContentSlot) -> bool {
        self.slots.contains_key(&slot)
    }
    pub fn slot_path(&self, slot: ContentSlot, rom: &str) -> Result<PathBuf, String> {
        self.slots
            .get(&slot)
            .ok_or_else(|| format!("Select {} in Tool inputs first", slot.label()))?
            .recheck(slot, rom)
    }
    pub fn adopt(
        &mut self,
        slot: ContentSlot,
        path: &Path,
        rom: &str,
        menu: &mut menu::Menu,
    ) -> Result<(), String> {
        if self.id_job.is_some() {
            return Err("Wait for the current catalog operation".into());
        }
        self.initialize(menu, rom.into())?;
        let reference = Reference::explicit(path, slot, rom)?;
        if let Some(memory) = &mut self.memory {
            memory.record_input(slot, &reference);
        }
        self.saved_issues.remove(&slot);
        self.slots.insert(slot, reference);
        self.scope(menu);
        Ok(())
    }
    pub fn cancel(&mut self) {
        self.view = self.view.wrapping_add(1);
    }
    fn busy(&self) -> bool {
        self.job.is_some() || self.id_job.is_some() || self.scan_job.is_some()
    }
    /// Explicit headless refresh uses the same roots and bounded scanner as the
    /// selector, but does not open a view or commit a cancelled/stale result.
    pub fn refresh_control(&mut self) -> Result<(), String> {
        if self.busy() || self.restore_pending {return Err("Wait for the current catalog operation".into());}
        let context=self.context.clone().ok_or("No ROM context")?;
        let roots=self.roots.clone();let rom=self.rom.clone();
        let anchors=self.memory.as_ref().map(|m|m.anchors.clone());
        self.scan_job=Some(ScanJob{context,generation:self.generation,cancelled:false,handle:std::thread::spawn(move||{
            let index=Index::scan(roots,rom)?;
            let locations=anchors.as_ref().map(|a|crate::debug_choices::memory::Memory::locations(&index,a)).unwrap_or_default();
            Ok((index,locations))
        })});Ok(())
    }
    pub fn cancel_refresh_control(&mut self)->bool {
        if let Some(job)=&mut self.scan_job{job.cancelled=true;true}else{false}
    }
    pub fn poll_refresh_control(&mut self,menu:&mut menu::Menu)->Option<Result<(),String>> {
        if !self.scan_job.as_ref().is_some_and(|j|j.handle.is_finished()){return None;}
        let job=self.scan_job.take().unwrap();let result=job.handle.join().unwrap_or_else(|_|Err("Content scan worker failed".into()));
        if job.cancelled{return Some(Err("Content scan control-cancelled".into()));}
        if self.context.as_ref()!=Some(&job.context)||self.generation!=job.generation{return Some(Err("Content catalog or ROM context changed; refresh again".into()));}
        Some(result.map(|(index,locations)|{
            self.generation=self.generation.wrapping_add(1);
            if menu.content_active(){
                menu.open_content(self.slot.map_or("Debug inputs",|s|s.label()).into(),rows(&index),"Explicit scan complete; selection needs action preflight".into(),self.generation);
            }
            self.index=Some(Arc::new(index));self.locations=locations;self.scope(menu);
            if menu.content_active(){self.apply_preferences(menu);}
        }))
    }
    pub fn page(&self,offset:u32,limit:u8,generation:Option<u64>)->Result<Value,String>{
        let index=self.index.as_ref().ok_or("Refresh the content catalog first")?;
        if generation.is_some_and(|g|g!=self.generation){return Err("Content catalog generation changed; list again".into());}
        if !(1..=16).contains(&limit){return Err("Content page limit needs1..16".into());}
        let offset=offset as usize;let total=index.entries.len();
        if offset>total{return Err("Content page offset exceeds entry count".into());}
        let end=offset.saturating_add(limit as usize).min(total);
        let entries=index.entries[offset..end].iter().map(|e|json!({"id":e.id,"kind":e.kind,"name":e.name.chars().take(128).collect::<String>(),"name_truncated":e.name.chars().count()>128,"root_id":e.root_id,"relative_path":e.relative_path.chars().take(256).collect::<String>(),"path_truncated":e.relative_path.chars().count()>256,"sha256":e.sha256,"availability":e.availability,"compatibility":e.compatibility})).collect::<Vec<_>>();
        Ok(json!({"generation":self.generation,"offset":offset,"total":total,"next_offset":(end<total).then_some(end),"scan_truncated":index.stats.truncated,"entries":entries,"scope":"Bounded discovery; selection rechecks the original entry and action preflight"}))
    }
    /// Starts a readonly preflight. The caller must poll for its terminal result.
    pub fn select_id(&mut self, slot: ContentSlot, id: &str) -> Result<(), String> {
        if self.busy() || self.restore_pending {
            return Err("Wait for the current catalog operation".into());
        }
        let context = self.context.clone().ok_or("No ROM context")?;
        let index = self.index.clone().ok_or("Refresh the content catalog first")?;
        if !index.entries.iter().any(|entry| entry.id == id) {
            return Err("Unknown content ID; refresh the catalog first".into());
        }
        let worker_index = index.clone();
        let id = id.to_owned();
        self.id_job = Some(IdJob {
            generation: self.generation,
            context,
            index,
            slot,
            cancelled: false,
            handle: std::thread::spawn(move || Reference::resolve(worker_index, &id, Some(slot))),
        });
        Ok(())
    }
    pub fn cancel_id(&mut self) -> bool {
        if let Some(job) = &mut self.id_job {
            job.cancelled = true;
            true
        } else {
            false
        }
    }
    pub fn poll_id(&mut self, menu: &mut menu::Menu) -> Option<Result<String, String>> {
        if !self.id_job.as_ref().is_some_and(|job| job.handle.is_finished()) {
            return None;
        }
        let job = self.id_job.take().unwrap();
        let result = job.handle.join().unwrap_or_else(|_| Err("Content worker failed".into()));
        if menu.content_active() {
            menu.content_busy(false);
        }
        if job.cancelled {
            return Some(Err("Content selection control-cancelled".into()));
        }
        if self.context.as_ref() != Some(&job.context)
            || self.generation != job.generation
            || !self.index.as_ref().is_some_and(|index| Arc::ptr_eq(index, &job.index))
        {
            return Some(Err("Content catalog or ROM context changed; select again".into()));
        }
        Some(result.map(|reference| {
            let id = reference.entry.id.clone();
            if let Some(memory) = &mut self.memory {
                memory.record_input(job.slot, &reference);
            }
            self.saved_issues.remove(&job.slot);
            self.slots.insert(job.slot, reference);
            self.scope(menu);
            id
        }))
    }
    fn start(
        &mut self,
        menu: &mut menu::Menu,
        work: impl FnOnce() -> Result<ResultData, String> + Send + 'static,
    ) -> Result<(), String> {
        if self.busy() {
            return Err("Catalog operation is finishing; wait or cancel the selection.".into());
        }
        self.job = Some(Job {
            view: self.view,
            handle: std::thread::spawn(work),
        });
        menu.content_busy(true);
        menu.content_message("Working on selected catalog inputs; guest remains paused.".into());
        Ok(())
    }
    pub fn refresh(&mut self, menu: &mut menu::Menu) -> Result<(), String> {
        let roots = self.roots.clone();
        let rom = self.rom.clone();
        let anchors = self.memory.as_ref().map(|m| m.anchors.clone());
        self.start(menu, move || {
            let index = Index::scan(roots, rom)?;
            let locations = anchors
                .as_ref()
                .map(|a| crate::debug_choices::memory::Memory::locations(&index, a))
                .unwrap_or_default();
            Ok(ResultData::Scan(index, locations))
        })
    }
    pub fn external_result(
        &mut self,
        path: Option<PathBuf>,
        file: bool,
        menu: &mut menu::Menu,
    ) -> Result<(), String> {
        let Some(path) = path else {
            return Ok(());
        };
        if self.busy() {
            return Err("Wait for the current catalog operation before adding a root".into());
        }
        let root = if file {
            Root::single_file(&path)?
        } else {
            Root::new(&path, false)?
        };
        if self.roots.iter().any(|r| r.id == root.id) {
            menu.content_message(
                "This content root is already listed. Refresh explicitly if needed.".into(),
            );
            return Ok(());
        }
        if self.roots.len() >= MAX_ROOTS {
            return Err(
                "Eight content roots are already retained; the new root was not added.".into(),
            );
        }
        self.roots.push(root);
        if let Some(memory) = &mut self.memory {
            memory.record_roots(&self.roots);
        }
        self.refresh(menu)
    }
    pub fn browse_file(&mut self, menu: &mut menu::Menu) -> Result<(), String> {
        if self.busy() {
            return Err("Wait for the current catalog operation".into());
        }
        let path = rfd::FileDialog::new()
            .set_title("Browse external file - add only, never run")
            .pick_file();
        self.external_result(path, true, menu)
    }
    pub fn browse_folder(&mut self, menu: &mut menu::Menu) -> Result<(), String> {
        if self.busy() {
            return Err("Wait for the current catalog operation".into());
        }
        let path = rfd::FileDialog::new()
            .set_title("Add explicit content folder - never run")
            .pick_folder();
        self.external_result(path, false, menu)
    }
    pub fn choose(
        &mut self,
        menu: &mut menu::Menu,
        index: usize,
        generation: u64,
        preview: bool,
    ) -> Result<(), String> {
        if generation != self.generation {
            return Err("The catalog changed; select the item again".into());
        }
        let slot = self.slot;
        let catalog = self
            .index
            .clone()
            .ok_or("Refresh the content catalog first")?;
        let entry = catalog
            .entries
            .get(index)
            .ok_or("Unknown selected content row")?
            .clone();
        self.start(menu, move || {
            let reference =
                Reference::resolve(catalog, &entry.id, if preview { None } else { slot })?;
            let checked = &reference.entry;
            let path = &reference.path;
            if !preview {
                return Ok(ResultData::Select(reference));
            }
            if checked.kind != Kind::Snapshot {
                return Ok(ResultData::Preview(
                    checked.id.clone(),
                    Vec::new(),
                    "No embedded snapshot preview for this type. Full details remain scrollable."
                        .into(),
                ));
            }
            let bytes = crate::diagnostics::read(
                &path,
                gba_core::save_state::MAX_STATE_BYTES as u64 + 240 * 160 * 4 + 48,
            )?;
            if checked
                .sha256
                .as_ref()
                .is_none_or(|sha| *sha != gba_core::ffta_rom::digest(&bytes))
            {
                return Err("Snapshot changed or was not hashed; refresh before preview".into());
            }
            let preview = crate::save_state::preview(&bytes)?;
            let message = if preview.is_empty() {
                "Raw snapshot has no embedded image; no machine was loaded."
            } else {
                "Authenticated embedded preview; no machine was loaded."
            };
            Ok(ResultData::Preview(
                checked.id.clone(),
                preview,
                message.into(),
            ))
        })
    }
    pub fn poll(&mut self, menu: &mut menu::Menu) -> bool {
        if !self.job.as_ref().is_some_and(|j| j.handle.is_finished()) {
            return false;
        }
        let job = self.job.take().unwrap();
        let result = job
            .handle
            .join()
            .unwrap_or_else(|_| Err("Content worker failed".into()));
        menu.content_busy(false);
        let result = match result {
            Ok(ResultData::Restored(restored)) => {
                if self.context.as_ref() == Some(&restored.context) {
                    if let Some(memory) = &self.memory {
                        for (slot, original) in &restored.originals {
                            let current = memory
                                .file
                                .state
                                .inputs
                                .iter()
                                .find(|s| s.slot == original.slot);
                            if current != Some(original) {
                                continue;
                            }
                            if let Some(reference) = restored.references.get(slot) {
                                self.slots.insert(*slot, reference.clone());
                                self.saved_issues.remove(slot);
                            } else if let Some(issue) = restored.issues.get(slot) {
                                self.saved_issues.insert(*slot, issue.clone());
                            }
                        }
                    }
                    self.scope(menu);
                    if menu.content_active() {
                        self.apply_preferences(menu);
                        let message=self.slot.and_then(|slot|self.saved_issues.get(&slot)).cloned().unwrap_or_else(||"Saved references checked. Refresh the list explicitly; nothing started.".into());
                        menu.content_message(message);
                    }
                }
                return true;
            }
            other => other,
        };
        if job.view != self.view || !menu.content_active() {
            return true;
        }
        match result {
            Ok(ResultData::Scan(index, locations)) => {
                self.generation = self.generation.wrapping_add(1);
                let message = format!(
                    "{} entries | {} issues | {} | selection needs action preflight",
                    index.entries.len(),
                    index.issues.len(),
                    if index.stats.truncated {
                        "PARTIAL: scan cap"
                    } else {
                        "listing complete"
                    }
                );
                let message = if index.issues.is_empty() {
                    message
                } else {
                    format!(
                        "{} | First {} issues: {}",
                        message,
                        index.issues.len().min(8),
                        index
                            .issues
                            .iter()
                            .take(8)
                            .cloned()
                            .collect::<Vec<_>>()
                            .join(" | ")
                    )
                };
                menu.open_content(
                    self.slot.map_or("Debug inputs", |s| s.label()).into(),
                    rows(&index),
                    message,
                    self.generation,
                );
                self.index = Some(Arc::new(index));
                self.locations = locations;
                self.scope(menu);
                self.apply_preferences(menu);
            }
            Ok(ResultData::Select(selection)) => {
                menu.content_selected(&selection.entry.id);
                if let Some(slot) = self.slot {
                    if let Some(memory) = &mut self.memory {
                        memory.record_input(slot, &selection);
                    }
                    self.saved_issues.remove(&slot);
                    self.slots.insert(slot, selection.clone());
                }
                self.selected = Some(selection);
                self.scope(menu);
            }
            Ok(ResultData::Preview(id, pixels, message)) => {
                if menu.content_highlight() == Some(id.as_str()) {
                    menu.content_message(message.clone());
                    menu.content_preview(pixels, message);
                } else {
                    menu.content_message("Preview completed for a different row. Open Details for the current selection.".into());
                }
            }
            Ok(ResultData::Restored(_)) => unreachable!("restoration handled before view guard"),
            Err(error) => menu.content_message(error),
        }
        true
    }
    pub fn status(&self) -> Value {
        json!({"persistence":self.memory.as_ref().map(|m|json!({"file":m.file.path,"message":m.message,"saved_roles":m.file.state.inputs.len(),"saved_views":m.file.state.views.len(),"bookmarks":m.file.state.bookmarks.len(),"readonly_restore_pending":self.restore_pending})),"persistence_error":self.persistence_error,"saved_issues":self.saved_issues.iter().map(|(slot,message)|json!({"slot":slot.label(),"message":message})).collect::<Vec<_>>(),"slot":self.slot.map(|s|s.label()),"inputs":ContentSlot::ALL.into_iter().filter_map(|slot|self.slots.get(&slot).map(|s|json!({"slot":slot.label(),"id":s.entry.id,"kind":s.entry.kind,"relative_path":s.entry.relative_path,"sha256":s.entry.sha256,"scope":"Pinned host reference; rechecked before use; runtime ABI and decoded guest fields still need action preflight"}))).collect::<Vec<_>>(),"generation":self.generation,"busy":self.busy(),"roots":self.roots.len(),"entries":self.index.as_ref().map_or(0,|i|i.entries.len()),"selected":self.selected.as_ref().map(|s|json!({"id":s.entry.id,"kind":s.entry.kind,"root_id":s.entry.root_id,"relative_path":s.entry.relative_path,"sha256":s.entry.sha256,"path":s.path})),"scope":"Host content selection only; no execution or full action verification"})
    }
}
#[cfg(test)]
pub(crate) mod tests;
