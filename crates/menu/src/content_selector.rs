//! Reusable content picker. No filesystem, guest, script or OS-dialog access.
use super::{canvas::Canvas, diagnostics, MenuAction, MenuInput};
use std::{cell::Cell, collections::BTreeSet};

#[derive(Clone, Default, Debug)]
pub struct ContentEntry {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub description: String,
    pub location: String,
    pub identity: String,
    pub status: String,
    pub compatibility: String,
    pub reason: String,
    pub available: bool,
    /// Machine-readable cheat metadata; display strings may be localized.
    pub cheat_filter: Option<CheatFilter>,
}

#[derive(Clone, Default, Debug)]
pub struct CheatFilter {
    pub armed: bool,
    pub declared_match: bool,
    pub plan_candidate: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheatShortcut { Details, Favorite, Favorites, Activate }
/// Pure host view state. Restoration never confirms or executes an item.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct ContentViewPreferences {
    pub query: String,
    pub kind: String,
    pub filter: usize,
    pub sort: usize,
    pub collection: usize,
    pub highlighted: Option<String>,
}
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct ContentHistory {
    pub favorites: Vec<String>,
    pub recent: Vec<String>,
}
fn preference_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 80
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-._".contains(&b))
}
const FILTERS: [&str; 4] = [
    "All states",
    "Available",
    "Declared match",
    "Needs attention",
];
const CHEAT_FILTERS: [&str; 6] = [
    "All states", "Plan candidate", "Declared match", "Needs attention", "Armed", "Not armed",
];
const SORTS: [&str; 3] = ["Name", "Type", "Recent"];
const COLLECTIONS: [&str; 3] = ["All content", "Favorites", "Recent"];
const ACTIONS: [&str; 7] = [
    "Select",
    "Favorite",
    "Details / preview",
    "Refresh",
    "Browse external file...",
    "Add folder...",
    "Back / cancel",
];
const CHEAT_ACTIONS: [&str; 8] = [
    "Details",
    "Favorite",
    "Activate...",
    "Reload",
    "Import catalog...",
    "Disarm all",
    "Back",
    "Show favorites",
];
const KEYS: [&str; 4] = [
    "abcdefghijklmnopqrstuvwxyz0123456789",
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789",
    "àèéìòùäëïöüáíóúâêîôûãõñçßøåæœ-_./:@ ",
    ",;=+*!?()[]{}<>%&|\"'`~$^#\\",
];
type Rect = [usize; 4];
fn inside(r: Rect, x: usize, y: usize) -> bool {
    x >= r[0] && x < r[0] + r[2] && y >= r[1] && y < r[1] + r[3]
}
#[derive(Clone, Copy)]
pub(super) struct Geometry {
    pub panel: Rect,
    pub search: Rect,
    pub controls: [Rect; 4],
    pub list: Rect,
    pub details: Rect,
    pub actions: [Rect; 8],
    pub row: usize,
    pub visible: usize,
    pub scale: f32,
}
pub(super) fn geometry(w: usize, h: usize) -> Geometry {
    if w<300 || h<150{return legacy_geometry(w,h);}
    let scale=(w as f32/627.).min(h as f32/218.).clamp(0.85,6.);
    let pad=(6.*scale).max(2.)as usize;let row=(22.*scale).max(18.)as usize;let gap=(3.*scale).max(2.)as usize;
    let pw=w.saturating_sub(pad*2);let ph=h.saturating_sub(pad*2);let(x,y)=(pad,pad);let inner=pw.saturating_sub(pad*2);
    let compact=w<440;let search_w=if compact{inner}else{inner/3};
    let search=[x+pad,y+row,search_w,row];
    let cw=if compact{inner.saturating_sub(3*gap)/4}else{inner.saturating_sub(search_w+4*gap)/4};
    let controls=std::array::from_fn(|i|[if compact{x+pad+i*(cw+gap)}else{x+pad+search_w+gap+i*(cw+gap)},if compact{search[1]+row+gap}else{search[1]},cw,row]);
    let top=controls[0][1]+row+gap;
    let buttons_top=(y+ph).saturating_sub(pad+4*row+2*gap);
    let area_h=buttons_top.saturating_sub(top+gap);
    let (list,details)=if compact{
        let height=area_h/2;([x+pad,top,inner,height],[x+pad,top+height+gap,inner,area_h.saturating_sub(height+gap)])
    }else{
        let left=inner.saturating_sub(gap)*52/100;([x+pad,top,left,area_h],[x+pad+left+gap,top,inner.saturating_sub(left+gap),area_h])
    };
    let bw=inner.saturating_sub(2*gap)/3;let actions=std::array::from_fn(|i|[x+pad+(i%3)*(bw+gap),buttons_top+(i/3)*(row+gap),bw,row]);
    Geometry{panel:[x,y,pw,ph],search,controls,list,details,actions,row,visible:(list[3]/row).max(1),scale}
}
fn legacy_geometry(w: usize, h: usize) -> Geometry {
    let scale = (w as f32 / 940.).min(h as f32 / 620.).clamp(0.1, 2.5);
    let pad = (14. * scale).max(2.) as usize;
    let row = (32. * scale).max(5.) as usize;
    let pw = w.saturating_sub(2 * pad).min((1150. * scale) as usize);
    let ph = h.saturating_sub(2 * pad);
    let x = w.saturating_sub(pw) / 2;
    let y = h.saturating_sub(ph) / 2;
    let inner = pw.saturating_sub(2 * pad);
    let gap = (8. * scale).max(2.) as usize;
    let search = [x + pad, y + pad + row, inner, row];
    let cw = inner.saturating_sub(3 * gap) / 4;
    let controls =
        std::array::from_fn(|i| [x + pad + i * (cw + gap), search[1] + row + gap, cw, row]);
    let top = controls[0][1] + row + gap;
    let buttons_top = (y + ph).saturating_sub(pad + row * 4 + gap * 3);
    let area_h = buttons_top.saturating_sub(top + gap);
    let left = inner.saturating_sub(gap) * 52 / 100;
    let list = [x + pad, top, left, area_h];
    let details = [
        list[0] + left + gap,
        top,
        inner.saturating_sub(left + gap),
        area_h,
    ];
    let bw = inner.saturating_sub(2 * gap) / 3;
    let actions = std::array::from_fn(|i| {
        [
            x + pad + (i % 3) * (bw + gap),
            buttons_top + (i / 3) * (row + gap),
            bw,
            row,
        ]
    });
    Geometry {
        panel: [x, y, pw, ph],
        search,
        controls,
        list,
        details,
        actions,
        row,
        visible: (area_h / row).max(1),
        scale,
    }
}
#[derive(Default)]
pub(super) struct Selector {
    pub entries: Vec<ContentEntry>,
    pub title: String,
    pub message: String,
    pub generation: u64,
    pub busy: bool,
    pub debug_copy: bool,
    pub editor_only: bool,
    pub cheat_mode: bool,
    text_limit: usize,
    pub confirmed: Option<String>,
    pub preview: Vec<[u8; 4]>,
    pub preview_message: String,
    visible: Vec<usize>,
    cursor: usize,
    focus: usize,
    first: Cell<usize>,
    query: String,
    required_kind: Vec<String>,
    kind: usize,
    filter: usize,
    sort: usize,
    collection: usize,
    favorites: BTreeSet<String>,
    recent: Vec<String>,
    editing: bool,
    choice_edit: Option<usize>,
    search_backup: String,
    search_backup_id: Option<String>,
    preedit: String,
    key: usize,
    key_page: usize,
    detail_mode: bool,
    detail_scroll: Cell<usize>,
    manage: Option<super::cheat_management::State>,
    preset_name_edit: bool,
}
impl Selector {
    fn layout(&self,w:usize,h:usize)->Geometry {
        let mut g=geometry(w,h);
        if self.cheat_mode && self.manage.is_none() && !self.editing && !self.detail_mode && w>=440 && h>=150 {
            let gap=(3.*g.scale).max(2.) as usize;
            let shift=g.row+gap;
            let top=g.actions[0][1].saturating_add(shift);
            g.list[3]=top.saturating_sub(g.list[1]+gap);
            g.details[3]=g.list[3];
            g.visible=(g.list[3]/g.row).max(1);
            let left=g.actions[0][0];let right=g.panel[0]+g.panel[2]-(left-g.panel[0]);
            let width=right.saturating_sub(left+3*gap)/4;
            g.actions=std::array::from_fn(|i|[left+(i%4)*(width+gap),top+(i/4)*(g.row+gap),width,g.row]);
        }
        g
    }
    pub fn cheat_shortcuts_available(&self)->bool{
        self.cheat_mode && self.manage.is_none() && !self.detail_mode && !self.editing && !self.choice_focus() && !self.busy
    }
    pub fn cheat_shortcut(&mut self,shortcut:CheatShortcut)->Option<MenuAction>{
        if !self.cheat_shortcuts_available(){return None;}
        self.focus=match shortcut{CheatShortcut::Details=>6,CheatShortcut::Favorite=>7,CheatShortcut::Activate=>8,CheatShortcut::Favorites=>13};
        self.confirm()
    }
    pub fn open_cheat_management(&mut self,sources:Vec<super::CheatSource>,presets:Vec<String>){
        if let Some(i)=self.selected(){
            let entry=&self.entries[i];
            let armed=entry.cheat_filter.as_ref().is_some_and(|f|f.armed);
            self.detail_mode=false;
            self.manage=Some(super::cheat_management::State::new(entry.id.clone(),entry.name.clone(),armed,sources,presets));
        }
    }
    pub fn cheat_managed_id(&self)->Option<&str>{self.manage.as_ref().map(|s|s.id.as_str())}
    fn cheat_armed(&self)->bool{self.selected().and_then(|i|self.entries[i].cheat_filter.as_ref()).is_some_and(|f|f.armed)}
    fn cheat_action_label(&self,index:usize)->&'static str{
        match index {2 if self.cheat_armed()=>"Disarm",2=>"Activate...",7 if self.collection==1=>"Show all",_=>CHEAT_ACTIONS[index]}
    }
    pub fn cheat_selected_source(&self)->Option<&super::CheatSource>{self.manage.as_ref()?.selected_source()}
    pub fn cheat_selected_parameters(&self)->Option<Vec<(&str,u16)>>{self.manage.as_ref()?.selected_parameters()}
    pub fn cheat_preview_reviewed(&self)->bool{self.manage.as_ref().is_some_and(|state|!state.preview.is_empty())&&!self.detail_mode}
    pub fn cheat_preset_name(&self)->Option<&str>{self.manage.as_ref().map(|s|s.preset_name.as_str())}
    pub fn cheat_preset_selected(&self)->Option<&str>{self.manage.as_ref()?.preset_selected.as_deref()}
    pub fn cheat_preview_token(&self)->Option<&str>{self.manage.as_ref().map(|s|s.preview_token.as_str()).filter(|s|!s.is_empty() && !self.detail_mode)}
    pub fn cheat_set_presets(&mut self,presets:Vec<String>){if let Some(s)=self.manage.as_mut(){s.presets=presets;s.preset_cursor=s.preset_cursor.min(s.presets.len().saturating_sub(1));}}
    pub fn cheat_set_preview_token(&mut self,token:String){if let Some(s)=self.manage.as_mut(){s.preview_token=token;}}
    pub fn cheat_management_active(&self)->bool{self.manage.is_some()}
    pub fn cheat_management_feedback(&mut self,message:String,preview:Option<String>){if let Some(state)=self.manage.as_mut(){state.feedback=message;if let Some(preview)=preview{state.preview=preview;self.detail_mode=true;self.detail_scroll.set(0);}else{state.preview.clear();state.preview_token.clear();}}else{self.message=message;}}
    fn cheat_details(&self,locale:&crate::i18n::Locale)->String{
        self.selected().map(|i|{let e=&self.entries[i];format!("{} · {} · {}\n{}\n{}\n{}",e.name,e.status,e.compatibility,e.reason,e.identity,e.description)})
            .unwrap_or_else(||locale.tr("No entry selected").into())
    }
    pub(super) fn description(&self,language:input_config::Language)->String {
        let kinds=self.kinds();let kind=kinds.get(self.kind).map(String::as_str).unwrap_or_default();
        crate::i18n::format(language,"{title}\nSearch: {query}\nType: {kind}\nAvailability: {filter}\nSort: {sort}\nCollection: {collection}",&[
            ("title",language.text(&self.title).into()),("query",self.query.clone()),
            ("kind",if self.kind==0{language.text(kind)}else{kind}.into()),
            ("filter",language.text(self.filters()[self.filter]).into()),("sort",language.text(SORTS[self.sort]).into()),
            ("collection",language.text(COLLECTIONS[self.collection]).into()),
        ])
    }

    pub(super) fn selection_state(&self)->Vec<(String,String)>{
        let page=if self.editor_only{"ToolEditor"}else if self.cheat_mode{"CheatCatalog"}else{"ContentSelector"};
        let label=self.focus_label();
        let id=if self.editing{format!("keyboard/{}/{}",self.key_page,self.key)}else if self.focus==5{self.highlighted_id().unwrap_or("empty").into()}else{format!("control/{}",self.focus)};
        let mut state=vec![("page".into(),page.into()),("item_id".into(),format!("content/{id}")),("label".into(),label),
            ("query".into(),self.query.clone()),("preedit".into(),self.preedit.clone()),("kind".into(),self.kind.to_string()),("availability".into(),self.filter.to_string()),
            ("sort".into(),self.sort.to_string()),("collection".into(),self.collection.to_string()),("editing".into(),self.editing.to_string()),
            ("details".into(),self.detail_mode.to_string()),("detail_scroll".into(),self.detail_scroll.get().to_string()),("busy".into(),self.busy.to_string()),
            ("highlighted_id".into(),self.highlighted_id().unwrap_or_default().into()),("confirmed_id".into(),self.confirmed.clone().unwrap_or_default()),
            ("generation".into(),self.generation.to_string()),("feedback".into(),self.message.clone()),("preview_feedback".into(),self.preview_message.clone())];
        if let Some(id)=self.highlighted_id(){state.push(("favorite".into(),self.favorites.contains(id).to_string()));}
        state.push(("choice_editing".into(),self.choice_focus().to_string()));
        if let Some(index)=self.choice_edit {state.push(("pending_choice".into(),index.to_string()));}
        state
    }
    pub fn preferences(&self) -> ContentViewPreferences {
        ContentViewPreferences {
            query: if self.editing {
                self.search_backup.clone()
            } else {
                self.query.clone()
            },
            kind: self.kinds().get(self.kind).cloned().unwrap_or_default(),
            filter: self.filter,
            sort: self.sort,
            collection: self.collection,
            highlighted: if self.editing {
                self.search_backup_id.clone()
            } else {
                self.highlighted_id().map(str::to_owned)
            },
        }
    }
    pub fn history(&self) -> ContentHistory {
        ContentHistory {
            favorites: self.favorites.iter().cloned().collect(),
            recent: self.recent.clone(),
        }
    }
    pub fn restore_preferences(
        &mut self,
        v: ContentViewPreferences,
        h: ContentHistory,
    ) -> Result<(), String> {
        if self.editing {
            return Err("Finish the focused editor before restoring preferences".into());
        }
        if v.query.chars().count() > 256
            || v.query.chars().any(char::is_control)
            || v.kind.len() > 64
            || v.filter >= self.filters().len()
            || v.sort >= SORTS.len()
            || v.collection >= COLLECTIONS.len()
            || v.highlighted.as_deref().is_some_and(|s| !preference_id(s))
            || h.favorites.len() > 512
            || h.recent.len() > 32
            || h.favorites
                .iter()
                .chain(&h.recent)
                .any(|s| !preference_id(s))
        {
            return Err("Invalid/oversized content preferences; current view preserved".into());
        }
        self.query = v.query;
        self.kind = self.kinds().iter().position(|k| *k == v.kind).unwrap_or(0);
        self.filter = v.filter;
        self.sort = v.sort;
        self.collection = v.collection;
        self.favorites = h.favorites.into_iter().collect();
        self.recent.clear();
        for id in h.recent {
            if !self.recent.contains(&id) {
                self.recent.push(id);
            }
        }
        self.rebuild(v.highlighted.as_deref());
        Ok(())
    }
    pub fn set(
        &mut self,
        title: String,
        entries: Vec<ContentEntry>,
        message: String,
        generation: u64,
    ) {
        self.choice_edit = None;
        let old = self.selected().map(|i| self.entries[i].id.clone());
        self.title = title;
        self.entries = entries.into_iter().take(if self.cheat_mode{2048}else{512}).collect();
        self.message = message;
        self.generation = generation;
        self.kind = self.kind.min(self.kinds().len().saturating_sub(1));
        self.rebuild(old.as_deref());
    }
    pub fn scope(&mut self, kind: Vec<String>, current: Option<String>) {
        if self.required_kind != kind {
            self.query.clear();
            self.kind = 0;
            self.filter = 0;
            self.collection = 0;
            self.focus = 0;
            self.finish_search(true);
            self.query.clear();
        }
        self.required_kind = kind;
        self.confirmed = current;
        self.rebuild(self.confirmed.clone().as_deref());
    }
    fn kinds(&self) -> Vec<String> {
        let mut kinds: Vec<_> = self.entries.iter().map(|e| e.kind.clone()).collect();
        kinds.sort();
        kinds.dedup();
        kinds.insert(0, "All types".into());
        kinds
    }
    fn filters(&self) -> &'static [&'static str] {
        if self.cheat_mode { &CHEAT_FILTERS } else { &FILTERS }
    }
    fn rebuild(&mut self, keep: Option<&str>) {
        let kinds = self.kinds();
        let query = self.query.to_lowercase();
        self.visible = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                (self.required_kind.is_empty() || self.required_kind.contains(&e.kind))
                    && (self.kind == 0 || kinds.get(self.kind) == Some(&e.kind))
                    && (if self.cheat_mode {
                        match (self.filter, &e.cheat_filter) {
                            (0, _) => true,
                            (1, Some(c)) => c.plan_candidate,
                            (2, Some(c)) => c.declared_match,
                            (3, Some(c)) => !c.plan_candidate,
                            (4, Some(c)) => c.armed,
                            (5, Some(c)) => !c.armed,
                            _ => false,
                        }
                    } else { match self.filter {
                        1 => e.available,
                        2 => e.compatibility == "declared-match",
                        3 => {
                            !e.available
                                || ["incompatible", "unsupported"]
                                    .contains(&e.compatibility.as_str())
                        }
                        _ => true,
                    }})
                    && (match self.collection {
                        1 => self.favorites.contains(&e.id),
                        2 => self.recent.contains(&e.id),
                        _ => true,
                    })
                    && (query.is_empty()
                        || format!(
                            "{} {} {} {} {}",
                            e.name, e.id, e.kind, e.location, e.description
                        )
                        .to_lowercase()
                        .contains(&query))
            })
            .map(|(i, _)| i)
            .collect();
        let rank = |e: &ContentEntry| {
            self.recent
                .iter()
                .position(|id| id == &e.id)
                .unwrap_or(usize::MAX)
        };
        self.visible.sort_by(|a, b| {
            let a = &self.entries[*a];
            let b = &self.entries[*b];
            let primary = match self.sort {
                1 => a.kind.cmp(&b.kind),
                2 => rank(a).cmp(&rank(b)),
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            };
            primary
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                .then_with(|| a.id.cmp(&b.id))
        });
        self.cursor = keep
            .and_then(|id| self.visible.iter().position(|i| self.entries[*i].id == id))
            .unwrap_or(0);
        self.first.set(0);
        self.preview.clear();
        self.preview_message.clear();
        self.detail_scroll.set(0);
    }
    pub fn selected(&self) -> Option<usize> {
        self.visible.get(self.cursor).copied()
    }
    pub fn highlighted_id(&self) -> Option<&str> {
        self.selected().map(|i| self.entries[i].id.as_str())
    }
    pub fn mark_selected(&mut self, id: &str) {
        self.confirmed = Some(id.into());
        self.recent.retain(|s| s != id);
        self.recent.insert(0, id.into());
        self.recent.truncate(32);
        self.message = "Selected; no tool was started.".into();
        let keep = self.highlighted_id().map(str::to_owned);
        self.rebuild(keep.as_deref());
    }
    pub fn editing(&self) -> bool {
        self.editing
    }
    pub fn choice_focus(&self)->bool {self.choice_edit.is_some()}
    fn choice_index(&self)->usize {match self.focus {1=>self.kind,2=>self.filter,3=>self.sort,4=>self.collection,_=>0}}
    pub fn start_editor(&mut self, title: String, value: String, limit: usize) {
        self.title = title;
        self.query = value;
        self.text_limit = limit.clamp(128, 2048);
        self.editor_only = true;
        self.start_search();
    }
    pub fn edited_value(&self) -> String {
        self.query.clone()
    }
    pub fn start_search(&mut self) {
        self.choice_edit = None;
        self.focus = 0;
        self.editing = true;
        self.search_backup = self.query.clone();
        self.search_backup_id = self.highlighted_id().map(str::to_owned);
        self.preedit.clear();
        self.key = 0;
    }
    pub fn text(&mut self, text: &str) {
        if !self.editing {
            return;
        }
        let remaining = self
            .text_limit
            .max(128)
            .saturating_sub(self.query.chars().count());
        self.query
            .extend(text.chars().filter(|c| !c.is_control()).take(remaining));
        self.rebuild(None);
    }
    pub fn preedit(&mut self, text: &str) {
        if self.editing {
            self.preedit = text.chars().take(self.text_limit.max(128)).collect();
        }
    }
    pub fn text_backspace(&mut self) {
        if self.editing {
            self.query.pop();
            self.rebuild(None);
        }
    }
    pub fn finish_search(&mut self, cancel: bool) {
        if self.preset_name_edit {
            if !cancel {if let Some(s)=self.manage.as_mut(){s.preset_name=self.query.clone();s.preview.clear();s.preview_token.clear();}}
            self.query=self.search_backup.clone();let id=self.search_backup_id.clone();self.editing=false;self.preset_name_edit=false;self.preedit.clear();self.rebuild(id.as_deref());return;
        }
        if let Some(index)=self.choice_edit.take() {
            if !cancel {
                match self.focus {1=>self.kind=index,2=>self.filter=index,3=>self.sort=index,4=>self.collection=index,_=>{}}
                self.rebuild(None);
            }
            return;
        }
        if !self.editing {
            return;
        }
        if cancel {
            self.query = self.search_backup.clone();
            let id = self.search_backup_id.clone();
            self.rebuild(id.as_deref());
        }
        self.editing = false;
        self.preedit.clear();
    }
    fn first_visible(&self, g: Geometry) -> usize {
        let mut first = self
            .first
            .get()
            .min(self.visible.len().saturating_sub(g.visible));
        if self.cursor < first {
            first = self.cursor;
        } else if self.cursor >= first + g.visible {
            first = self.cursor + 1 - g.visible;
        }
        self.first.set(first);
        first
    }
    fn move_cursor(&mut self, next: usize) {
        self.cursor = next.min(self.visible.len().saturating_sub(1));
        self.preview.clear();
        self.preview_message.clear();
        self.detail_scroll.set(0);
    }
    fn confirm(&mut self) -> Option<MenuAction> {
        match self.focus {
            0 => self.start_search(),
            1..=4 => {
                self.choice_edit=Some(self.choice_index());
            }
            5 if self.cheat_mode => return self.selected().map(|_|MenuAction::CheatManage),
            5 => self.focus = 6,
            6 => {
                if self.cheat_mode {if self.selected().is_some(){self.detail_mode=true;}else{self.message="No catalog: choose Import catalog.".into();}return None;}
                if self.busy {
                    self.message = "Wait for the current catalog operation, or cancel.".into();
                } else if let Some(index) = self.selected() {
                    if self.entries[index].available
                        && !["incompatible", "unsupported"]
                            .contains(&self.entries[index].compatibility.as_str())
                    {
                        return Some(MenuAction::ContentSelect {
                            index,
                            generation: self.generation,
                        });
                    } else {
                        self.message =
                            "This entry cannot be selected. Read its reason or refresh.".into();
                    }
                } else {
                    self.message =
                        "No matching content. Refresh or add a file/folder explicitly.".into();
                }
            }
            7 => {
                if let Some(i) = self.selected() {
                    let id = self.entries[i].id.clone();
                    if !self.favorites.remove(&id) && self.favorites.len() < 512 {
                        self.favorites.insert(id);
                    }
                    let keep = self.highlighted_id().map(str::to_owned);
                    self.rebuild(keep.as_deref());
                }
            }
            8 => {
                if self.cheat_mode{return self.selected().map(|_|if self.cheat_armed(){MenuAction::CheatRequest(super::CheatRequest::Disarm)}else{MenuAction::CheatManage});}
                self.detail_mode = true;
                if !self.busy {
                    if let Some(index) = self.selected() {
                        return Some(MenuAction::ContentPreview {
                            index,
                            generation: self.generation,
                        });
                    }
                }
            }
            9 if !self.busy => return Some(if self.cheat_mode{MenuAction::CheatReload}else{MenuAction::ContentRefresh}),
            10 if !self.busy => return Some(if self.cheat_mode{MenuAction::CheatChoose}else{MenuAction::ContentBrowseFile}),
            11 if !self.busy => return Some(if self.cheat_mode{MenuAction::CheatRequest(super::CheatRequest::DisarmAll)}else{MenuAction::ContentAddFolder}),
            12 => return Some(MenuAction::ContentCancel),
            13 if self.cheat_mode => {
                let keep=self.highlighted_id().map(str::to_owned);
                self.collection=if self.collection==1{0}else{1};
                self.rebuild(keep.as_deref());
            }
            _ => {}
        }
        None
    }
    pub fn tab(&mut self, back: bool) {
        if !self.editing {if let Some(manage)=self.manage.as_mut(){if !self.detail_mode && !manage.parameter_mode{manage.focus=(manage.focus+if back{6}else{1})%7;}return;}}
        if self.choice_focus(){self.finish_search(true);}
        if self.editing {
            self.finish_search(false);
        }
        self.detail_mode = false;
        let count=if self.cheat_mode{14}else{13};
        self.focus = (self.focus + if back { count-1 } else { 1 }) % count;
    }
    pub fn handle(&mut self, input: MenuInput) -> Option<MenuAction> {
        if self.manage.is_some() && !self.editing {
            if self.detail_mode {match input{MenuInput::Cancel|MenuInput::Confirm=>{self.detail_mode=false;if let Some(m)=self.manage.as_mut(){if !m.preset_mode{m.focus=3;}}},MenuInput::Up|MenuInput::Left=>self.detail_scroll.set(self.detail_scroll.get().saturating_sub(1)),_=>self.detail_scroll.set((self.detail_scroll.get()+1).min(8192))};return None;}
            if input==MenuInput::Cancel && self.manage.as_ref().is_some_and(|state|state.parameter_mode){self.manage.as_mut().unwrap().handle(MenuInput::Left);return None;}
            if input==MenuInput::Cancel{self.manage=None;self.focus=5;return None;}
            if input==MenuInput::Confirm && self.manage.as_ref().is_some_and(|s|s.preset_mode && s.focus==5) {
                let name=self.manage.as_ref().unwrap().preset_name.clone();self.start_search();self.query=name;self.preset_name_edit=true;return None;
            }
            return self.manage.as_mut().unwrap().handle(input).map(MenuAction::CheatRequest);
        }
        if let Some(index)=self.choice_edit {
            match input {
                MenuInput::Confirm=>self.finish_search(false),
                MenuInput::Cancel=>self.finish_search(true),
                _=>{
                    let count=match self.focus {1=>self.kinds().len(),2=>self.filters().len(),3=>SORTS.len(),_=>COLLECTIONS.len()};
                    self.choice_edit=Some((index+if matches!(input,MenuInput::Left|MenuInput::Up){count-1}else{1})%count);
                },
            }
            return None;
        }
        if self.editing {
            match input {
                MenuInput::Cancel => self.finish_search(true),
                MenuInput::Left => self.key = (self.key + 41) % 42,
                MenuInput::Right => self.key = (self.key + 1) % 42,
                MenuInput::Up => self.key = (self.key + 35) % 42,
                MenuInput::Down => self.key = (self.key + 7) % 42,
                MenuInput::Confirm => self.key_confirm(),
            };
            return None;
        }
        if self.detail_mode {
            match input {
                MenuInput::Cancel | MenuInput::Confirm => {
                    self.detail_mode = false;
                    self.focus = 5;
                }
                MenuInput::Up | MenuInput::Left => self
                    .detail_scroll
                    .set(self.detail_scroll.get().saturating_sub(1)),
                MenuInput::Down | MenuInput::Right => self
                    .detail_scroll
                    .set((self.detail_scroll.get() + 1).min(8192)),
            };
            return None;
        }
        match input {
            MenuInput::Cancel => return Some(MenuAction::ContentCancel),
            MenuInput::Confirm => return self.confirm(),
            MenuInput::Left | MenuInput::Right if self.focus == 5 => {
                let n = if input == MenuInput::Left {
                    self.cursor.saturating_sub(8)
                } else {
                    self.cursor + 8
                };
                self.move_cursor(n);
            }
            MenuInput::Up if self.focus == 5 && self.cursor > 0 => {
                self.move_cursor(self.cursor - 1)
            }
            MenuInput::Down if self.focus == 5 && self.cursor + 1 < self.visible.len() => {
                self.move_cursor(self.cursor + 1)
            }
            MenuInput::Up | MenuInput::Left => {let count=if self.cheat_mode{14}else{13};self.focus = (self.focus + count-1) % count;},
            MenuInput::Down | MenuInput::Right => {let count=if self.cheat_mode{14}else{13};self.focus = (self.focus + 1) % count;},
        }
        None
    }
    fn key_labels(&self) -> Vec<String> {
        let mut keys: Vec<_> = KEYS[self.key_page]
            .chars()
            .take(36)
            .map(|c| c.to_string())
            .collect();
        while keys.len() < 36 {
            keys.push(" ".into());
        }
        keys.extend(
            ["Space", "Erase", "Clear", "Done", "Cancel", "Aa/#"]
                .into_iter()
                .map(str::to_owned),
        );
        keys
    }
    fn key_confirm(&mut self) {
        if self.key < 36 {
            let s = self.key_labels()[self.key].clone();
            self.text(&s);
        } else {
            match self.key {
                36 => self.text(" "),
                37 => self.text_backspace(),
                38 => {
                    self.query.clear();
                    self.rebuild(None);
                }
                39 => self.finish_search(false),
                40 => self.finish_search(true),
                41 => self.key_page = (self.key_page + 1) % KEYS.len(),
                _ => {}
            }
        }
    }
    fn key_rect(&self, g: Geometry, i: usize) -> Rect {
        let gap = (5. * g.scale).max(1.) as usize;
        let bw = (g.actions[2][0]+g.actions[2][2]).saturating_sub(g.actions[0][0]).saturating_sub(6*gap)/7;
        let row = (g.row * 3 / 4).max(5);
        [
            g.list[0] + (i % 7) * (bw + gap),
            g.list[1] + (i / 7) * (row + gap),
            bw,
            row,
        ]
    }
    pub fn hover(&mut self, x: usize, y: usize, w: usize, h: usize) {
        if self.manage.is_some() && !self.editing{return;}
        if self.choice_focus(){return;}
        let g = self.layout(w, h);
        if self.editing {
            for i in 0..42 {
                if inside(self.key_rect(g, i), x, y) {
                    self.key = i;
                    return;
                }
            }
            return;
        }
        if self.detail_mode {
            return;
        }
        if inside(g.search, x, y) {
            self.focus = 0;
            return;
        }
        for i in 0..4 {
            if inside(g.controls[i], x, y) {
                self.focus = i + 1;
                return;
            }
        }
        if inside(g.list, x, y) {
            let row = (y - g.list[1]) / g.row;
            let i = self.first_visible(g) + row;
            if i < self.visible.len() {
                self.focus = 5;
                if self.cursor != i {
                    self.move_cursor(i);
                }
            }
            return;
        }
        for i in 0..if self.cheat_mode{8}else{7} {
            if inside(g.actions[i], x, y) {
                self.focus = i + 6;
                return;
            }
        }
    }
    pub fn click(
        &mut self,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        left: bool,
    ) -> Option<MenuAction> {
        if self.choice_focus(){return self.handle(if left{MenuInput::Confirm}else{MenuInput::Cancel});}
        if !left {
            return self.handle(MenuInput::Cancel);
        }
        let g = self.layout(w, h);
        if self.manage.is_some() && !self.editing {
            if self.detail_mode{return if y>=g.panel[1]+g.panel[3].saturating_sub(g.row){self.handle(MenuInput::Cancel)}else{None};}
            if inside(g.controls[0],x,y){self.manage.as_mut().unwrap().focus=0;return self.handle(MenuInput::Up);}
            if inside(g.controls[1],x,y){self.manage.as_mut().unwrap().focus=0;return self.handle(MenuInput::Down);}
            if inside(g.controls[2],x,y){self.manage.as_mut().unwrap().focus=0;return self.handle(MenuInput::Confirm);}
            if !self.manage.as_ref().unwrap().parameter_mode {for i in 0..6{if inside(g.actions[i],x,y){self.manage.as_mut().unwrap().focus=i+1;return self.handle(MenuInput::Confirm);}}}
            if inside(g.actions[6],x,y){return self.handle(MenuInput::Cancel);}
            return None;
        }
        if self.cheat_mode && self.detail_mode {
            if y>=g.panel[1]+g.panel[3].saturating_sub(g.row){return self.handle(MenuInput::Cancel);}
            return None;
        }
        if self.editing {
            for i in 0..42 {
                if inside(self.key_rect(g, i), x, y) {
                    self.key = i;
                    self.key_confirm();
                    return None;
                }
            }
            return None;
        }
        if self.detail_mode {
            for i in 0..7 {
                if inside(g.actions[i], x, y) {
                    self.detail_mode = false;
                    self.focus = i + 6;
                    return self.confirm();
                }
            }
            if inside(g.details, x, y) {
                return None;
            }
            self.detail_mode = false;
        }
        if inside(g.details, x, y) {
            self.focus = if self.cheat_mode{6}else{8};
            return self.confirm();
        }
        if inside(g.search, x, y)
            || g.controls.iter().any(|r| inside(*r, x, y))
            || g.actions.iter().take(if self.cheat_mode{8}else{7}).any(|r| inside(*r, x, y))
        {
            self.hover(x, y, w, h);
            return self.confirm();
        }
        self.hover(x, y, w, h);
        None
    }
    pub fn wheel(&mut self, up: bool) {
        if self.manage.is_some() && !self.detail_mode && !self.editing{self.manage.as_mut().unwrap().focus=0;self.handle(if up{MenuInput::Up}else{MenuInput::Down});return;}
        if self.choice_focus(){self.handle(if up{MenuInput::Up}else{MenuInput::Down});return;}
        if self.detail_mode {
            self.detail_scroll.set(if up {
                self.detail_scroll.get().saturating_sub(2)
            } else {
                (self.detail_scroll.get() + 2).min(8192)
            });
        } else if !self.editing {
            self.focus = 5;
            self.move_cursor(if up {
                self.cursor.saturating_sub(1)
            } else {
                self.cursor + 1
            });
        }
    }
    fn focus_label(&self)->String {
        if let Some(manage)=&self.manage {if !self.editing{return if manage.focus==0{if manage.preset_mode{manage.presets.get(manage.preset_cursor).cloned().unwrap_or_else(||"No preset".into())}else if manage.parameter_mode{manage.current_parameter().map_or_else(||"No value available".into(),|(name,value)|format!("Declared parameter: {name} · {value}"))}else{manage.sources.get(manage.cursor).map_or_else(||"No source".into(),|s|format!("Source {} · {}",s.index,s.name))}}else{(if manage.preset_mode{["Save preset","Prepare preset","Arm preset","Disarm all","Edit preset name","Back to sources"]}else{["Prepare preview","Apply once","Arm","Disarm","Disarm all","Presets"]})[manage.focus-1].into()};}}
        if self.editing {
            format!("Search keyboard: {}", self.key_labels()[self.key])
        } else if self.detail_mode {
            "Content details (Up/Down scroll, Cancel returns)".into()
        } else {
            match self.focus {
                0 => "Search".into(),
                1 => "Type".into(),
                2 => "Availability".into(),
                3 => "Sort".into(),
                4 => "Collection".into(),
                5 => self.selected().map_or_else(
                    || "Empty list".into(),
                    |i| format!("{} | {}", self.entries[i].name, self.entries[i].id),
                ),
                i => if self.cheat_mode{self.cheat_action_label(i-6)}else{ACTIONS[i-6]}.into(),
            }
        }
    }
    pub fn focus(&self, w: usize, h: usize) -> diagnostics::Focus {
        let g = self.layout(w, h);
        let first = self.first_visible(g);
        let label = self.focus_label();
        let bounds = if self.cheat_mode && self.detail_mode {
            g.panel
        } else if let Some(manage)=self.manage.as_ref().filter(|_|!self.editing) {
            if manage.focus==0{g.controls[2]}else{g.actions[manage.focus-1]}
        } else if self.editing {
            self.key_rect(g, self.key)
        } else {
            match self.focus {
                0 => g.search,
                1..=4 => g.controls[self.focus - 1],
                5 => [
                    g.list[0],
                    g.list[1] + (self.cursor.saturating_sub(first)) * g.row,
                    g.list[2],
                    g.row,
                ],
                i => g.actions[i - 6],
            }
        };
        diagnostics::Focus {
            page: if self.editor_only {
                "ToolEditor"
            } else if self.cheat_mode {
                "CheatCatalog"
            } else {
                "ContentSelector"
            }
            .into(),
            selected: self.focus,
            label,
            capture: self.editing.then(|| {
                if self.editor_only {
                    "Tool parameter text"
                } else {
                    "Content search text"
                }
                .into()
            }).or_else(||self.choice_focus().then(||"Filter choice: arrows · OK applies · Cancel discards".into())),
            panel: g.panel,
            selected_bounds: Some(bounds),
            first_visible: first,
            dropdown: None,
        }
    }
}
pub(super) fn button(c: &mut Canvas, r: Rect, text: &str, active: bool, size: f32) {
    let [x, y, w, h] = r;
    c.rect(
        x as f32,
        y as f32,
        w as f32,
        h as f32,
        3.,
        if active {
            [45, 78, 100]
        } else {
            [28, 56, 78]
        },
        if active { [35, 62, 82] } else { [28, 56, 78] },
        [89, 121, 139],
    );
    c.text(
        x as f32 + 5.,
        y as f32 + (h as f32 - size) / 2. - 1.,
        text,
        if active {
            [240, 215, 164]
        } else {
            [233, 239, 236]
        },
        size,
        false,
        (w as f32 - 10.).max(1.),
    );
}
pub(super) fn draw(s: &Selector, px: &mut [[u8; 4]], w: usize, h: usize, language: input_config::Language) {
    let locale = super::i18n::Locale(language);
    for p in px.iter_mut().take(w * h) {
        for v in &mut p[..3] {
            *v = (*v as u16 / 4) as u8;
        }
    }
    let mut g = s.layout(w, h);
    let [x, y, pw, ph] = g.panel;
    if s.editor_only{g.search[2]=pw.saturating_sub((g.search[0]-x)*2);}
    let mut c = Canvas::new(px, w, h);
    let size = (13. * g.scale).max(4.);
    let ink = [233, 239, 236];
    c.rect(
        x as f32,
        y as f32,
        pw as f32,
        ph as f32,
        7.,
        [28, 56, 78],
        [28, 56, 78],
        [89, 121, 139],
    );
    if s.cheat_mode && s.detail_mode {
        let text=s.manage.as_ref().map_or_else(||s.cheat_details(&locale),|m|m.preview.clone());
        c.text((x+10) as f32,(y+5) as f32,locale.tr(if s.manage.is_some(){"CHEAT PREVIEW"}else{"CHEAT DETAILS · READ ONLY"}),ink,size*1.1,true,pw.saturating_sub(20) as f32);
        let lines=Canvas::wrap_text(&text,size,pw.saturating_sub(24) as f32);
        let pitch=(size*1.5).ceil() as usize;let top=y+g.row+4;let bottom=(y+ph).saturating_sub(g.row);
        let rows=bottom.saturating_sub(top)/pitch.max(1);let offset=s.detail_scroll.get().min(lines.len().saturating_sub(rows));s.detail_scroll.set(offset);
        for (n,line) in lines.iter().skip(offset).take(rows).enumerate(){c.text((x+10) as f32,(top+n*pitch) as f32,line,ink,size,false,pw.saturating_sub(20) as f32);}
        c.text((x+10) as f32,bottom as f32,locale.tr("Up/Down: scroll · OK/Cancel: return"),ink,size,false,pw.saturating_sub(20) as f32);
        c.finish(px);return;
    }
    if let Some(m)=s.manage.as_ref().filter(|_|!s.editing) {
        let title=if m.preset_mode{locale.tr("CHEAT PRESETS").to_owned()}else{format!("{} · {}",locale.tr("CHEAT MANAGEMENT"),m.title)};
        c.text((x+10) as f32,(y+5) as f32,&title,ink,size*1.1,true,pw.saturating_sub(20) as f32);
        for (i,label) in (if m.preset_mode{["Previous preset","Next preset","Select preset"]}else if m.parameter_mode{["Previous value","Next value","Select value"]}else{["Previous source (short)","Next source (short)","Select source"]}).iter().enumerate(){button(&mut c,g.controls[i],locale.tr(label),i==2&&m.focus==0,size);}
        let source=if m.preset_mode{m.presets.get(m.preset_cursor).cloned().unwrap_or_else(||locale.tr("No preset").to_owned())}else if m.parameter_mode{m.current_parameter().map_or_else(||locale.tr("No value available").to_owned(),|(name,value)|format!("{}: {} · {}",locale.tr("Declared parameter"),name,value))}else{m.sources.get(m.cursor).map_or_else(||locale.tr("No source").to_owned(),|v|format!("{} · {} · {} · {}",v.index,v.name,v.master,locale.tr(match v.assessment.as_str(){"manual-once-subset"=>"Preview available","parameter-required"=>"Choose a declared value","decoded-outside-once-limits"=>"Outside supported limits","not-evaluated"=>"Not assessed",_=>"Source needs review"})))};
        let selection=if m.preset_mode{format!("{}: {} · {}: {}",locale.tr("Selected preset"),m.preset_selected.as_deref().unwrap_or("-"),locale.tr("Preset name"),m.preset_name)}else{m.selected_source().map_or_else(||locale.tr("No source selected").to_owned(),|v|format!("{} · {}{}",locale.tr("Selected source"),v.index,m.selected_parameters().map_or_else(String::new,|choices|choices.iter().map(|(name,value)|format!(" · {name}: {value}")).collect::<String>()))) };
        c.text((x+10) as f32,(g.controls[0][1]+g.row+3) as f32,&source,ink,size,false,pw.saturating_sub(20) as f32);
        c.text((x+10) as f32,(g.controls[0][1]+g.row*2+3) as f32,&format!("{} · {}",selection,locale.tr(&m.feedback)),ink,size,false,pw.saturating_sub(20) as f32);
        if !m.preset_mode {
            let active=s.entries.iter().find(|entry|entry.id==m.id).and_then(|entry|entry.cheat_filter.as_ref()).is_some_and(|filter|filter.armed);
            let hint=if active{"Active: disable to stop future writes"}else if m.selected_source().is_none(){"1. Select a source"}else if m.parameter_mode{"Select a declared value"}else if m.preview.is_empty(){"2. Prepare and review the preview"}else{"3. Activate or apply once"};
            c.text((x+10) as f32,(g.controls[0][1]+g.row*3+3) as f32,locale.tr(hint),ink,size,false,pw.saturating_sub(20) as f32);
        }
        if !m.parameter_mode {for (i,label) in (if m.preset_mode{["Save preset","Prepare preset","Arm preset","Disarm all","Edit preset name","Back to sources"]}else{["Prepare preview","Apply once","Arm","Disarm","Disarm all","Presets"]}).iter().enumerate(){button(&mut c,g.actions[i],locale.tr(label),m.focus==i+1,size);}}
        button(&mut c,g.actions[6],locale.tr("Back"),false,size);
        c.finish(px);return;
    }
    c.text(
        (x + 10) as f32,
        (y + 4) as f32,
        &format!(
            "{}{} / {}",
            if s.debug_copy { locale.tr("DEBUG COPY / ") } else { "" },
            if s.editor_only {
                locale.tr("EDIT DRAFT")
            } else {
                locale.tr("CONTENT")
            },
            locale.tr(&s.title)
        ),
        ink,
        size * 1.2,
        true,
        pw.saturating_sub(20) as f32,
    );
    let search = format!(
        "{}: {}{}{}",
        if s.preset_name_edit { locale.tr("Preset name") } else if s.editor_only { locale.tr("Value") } else { locale.tr("Search") },
        s.query,
        if s.editing { " | " } else { "" },
        s.preedit
    );
    let search=if s.choice_focus(){locale.tr("Filter choice: arrows · OK applies · Cancel discards").into()}else{search};
    button(&mut c, g.search, &search, s.focus == 0 || s.editing, size);
    let kinds = s.kinds();
    let mut indices=[s.kind,s.filter,s.sort,s.collection];
    if let Some(index)=s.choice_edit {indices[s.focus-1]=index;}
    let labels = [
        kinds[indices[0]].as_str(),
        s.filters()[indices[1]],
        SORTS[indices[2]],
        COLLECTIONS[indices[3]],
    ];
    if !s.editor_only {
        for i in 0..4 {
            let label=if s.cheat_mode && i==3{super::i18n::format(language,"List: {collection}",&[("collection",locale.tr(if indices[i]==0{"All (short)"}else{labels[i]}).into())])}else{locale.tr(labels[i]).into()};
            button(&mut c, g.controls[i], &label, s.focus == i + 1, size);
        }
    }
    if s.editing {
        let keys = s.key_labels();
        for (i, key) in keys.iter().enumerate() {
            button(&mut c, s.key_rect(g, i), if i < KEYS[s.key_page].chars().count() { key } else { locale.tr(key) }, s.key == i, size);
        }
        c.text(
            g.list[0] as f32,
            (g.list[1]+6*((g.row*3/4).max(5)+(5.*g.scale).max(1.)as usize))as f32,
            locale.tr("Type Unicode text or use arrows / controller. Done commits; Cancel restores."),
            ink,
            size,
            false,
            (g.panel[2] - 20) as f32,
        );
    } else {
        let first = s.first_visible(g);
        if s.visible.is_empty() {
            c.text(
                g.list[0] as f32 + 7.,
                g.list[1] as f32 + 7.,
                locale.tr("No matching content"),
                ink,
                size,
                true,
                g.list[2] as f32 - 14.,
            );
            c.text(
                g.list[0] as f32 + 7.,
                g.list[1] as f32 + g.row as f32,
                locale.tr(if s.cheat_mode{"Check search and filters, or import a catalog."}else{"Refresh or add a file/folder."}),
                ink,
                size,
                false,
                g.list[2] as f32 - 14.,
            );
        }
        for (n, i) in s.visible.iter().skip(first).take(g.visible).enumerate() {
            let e = &s.entries[*i];
            let label = if s.cheat_mode {
                format!("{}{}{}  [{}]",if s.favorites.contains(&e.id){"* "}else{""},
                    if e.cheat_filter.as_ref().is_some_and(|f|f.armed){format!("[{}] ",locale.tr("Cheat armed"))}else{String::new()},e.name,e.kind)
            } else { format!("{}{}  [{}] {}",if s.favorites.contains(&e.id){"* "}else{""},e.name,e.kind,e.id.get(8..16).unwrap_or(&e.id)) };
            button(
                &mut c,
                [
                    g.list[0],
                    g.list[1] + n * g.row,
                    g.list[2],
                    g.row.saturating_sub(2),
                ],
                &label,
                s.cursor == first + n,
                size,
            );
        }
        let mut lines = Canvas::wrap_text(locale.tr(&s.message), size, g.details[2].saturating_sub(12) as f32);
        if let Some(i) = s.selected() {
            let e = &s.entries[i];
            let text = if s.cheat_mode {
                s.cheat_details(&locale)
            } else {
                format!("{}\n{}\n{}\n{} / {}\n{}\n{}\n{}\n{}\nStatus: {}\nSelected ID: {}",
                    e.name,e.id,e.location,e.status,e.compatibility,e.description,e.identity,e.reason,s.preview_message,s.message,s.confirmed.as_deref().unwrap_or("none"))
            };
            lines = Canvas::wrap_text(&text, size, g.details[2].saturating_sub(12) as f32);
        }
        let line = (size * 1.5).ceil() as usize;
        let image_h = if s.preview.len() == 240 * 160 {
            (g.details[3] / 2).min(g.details[2] * 2 / 3)
        } else {
            0
        };
        let rows = g.details[3].saturating_sub(image_h + line) / line.max(1);
        let offset = s.detail_scroll.get().min(lines.len().saturating_sub(rows));
        s.detail_scroll.set(offset);
        for (n, text) in lines.iter().skip(offset).take(rows).enumerate() {
            c.text(
                (g.details[0] + 5) as f32,
                (g.details[1] + n * line) as f32,
                text,
                ink,
                size,
                false,
                g.details[2].saturating_sub(10) as f32,
            );
        }
        if s.preview.len() == 240 * 160 && image_h > 0 {
            let iw = (image_h * 3 / 2).min(g.details[2]);
            let iy = g.details[1] + g.details[3] - image_h;
            for yy in 0..image_h {
                for xx in 0..iw {
                    let at = ((iy + yy) * w + g.details[0] + xx) * 4;
                    if at + 4 <= c.pix.data().len() {
                        c.pix.data_mut()[at..at + 4].copy_from_slice(
                            &s.preview[(yy * 160 / image_h) * 240 + xx * 240 / iw],
                        );
                    }
                }
            }
        }
        for i in 0..if s.cheat_mode{8}else{7} {
            let label = if i == 1
                && s.selected()
                    .is_some_and(|n| s.favorites.contains(&s.entries[n].id))
            {
                locale.tr("Unfavorite")
            } else {
                if s.cheat_mode {s.cheat_action_label(i)}else{ACTIONS[i]}
            };
            button(
                &mut c,
                g.actions[i],
                locale.tr(label),
                s.focus == i + 6 || i == 2 && s.detail_mode,
                size,
            );
        }
    }
    if !s.editor_only {
        let confirmed = s
            .confirmed
            .as_ref()
            .map(|id| super::i18n::format(language, " | Selected {id}", &[("id", id.get(8..16).unwrap_or(id).into())]))
            .unwrap_or_default();
        let footer = super::i18n::format(language, "{count} items{working} | {message}{selected}", &[
            ("count", s.visible.len().to_string()),
            ("working", if s.busy { locale.tr(" / working") } else { "" }.into()),
            ("message", locale.tr(&s.message).into()),
            ("selected", confirmed),
        ]);
        c.text(
            (x + 10) as f32,
            (y + ph).saturating_sub(g.row) as f32,
            &footer,
            ink,
            size,
            false,
            pw.saturating_sub(20) as f32,
        );
    }
    c.finish(px);
}
#[cfg(test)]
mod tests;
