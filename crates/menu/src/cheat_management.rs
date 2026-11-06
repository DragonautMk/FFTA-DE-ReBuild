//! Host-only selection state for a source-bound cheat. The guest is never
//! accessed here; actions are requests to the existing shared handlers.
use super::MenuInput;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Source {
    pub index: u16,
    pub name: String,
    pub master: String,
    pub assessment: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request { Prepare, ApplyOnce, Arm, Disarm, DisarmAll, PresetSave, PresetPrepare, PresetArm }

#[derive(Default)]
pub(super) struct State {
    pub id: String,
    pub title: String,
    pub sources: Vec<Source>,
    pub cursor: usize,
    pub selected: Option<usize>,
    pub focus: usize,
    pub feedback: String,
    pub preview: String,
    pub preview_token: String,
    pub presets: Vec<String>,
    pub preset_cursor: usize,
    pub preset_selected: Option<String>,
    pub preset_name: String,
    pub preset_mode: bool,
}

impl State {
    pub fn new(id: String, title: String, armed: bool, sources: Vec<Source>, presets: Vec<String>) -> Self {
        Self { id, title, sources, presets, focus: if armed { 4 } else { 0 }, ..Self::default() }
    }
    pub fn selected_source(&self) -> Option<&Source> { self.selected.and_then(|index| self.sources.get(index)) }
    pub fn handle(&mut self, input: MenuInput) -> Option<Request> {
        if self.focus == 0 {
            if self.preset_mode {
                match input {
                    MenuInput::Up => self.preset_cursor = self.preset_cursor.saturating_sub(1),
                    MenuInput::Down => self.preset_cursor = (self.preset_cursor + 1).min(self.presets.len().saturating_sub(1)),
                    MenuInput::Right => self.focus = 1,
                    MenuInput::Confirm => {
                        self.preset_selected = self.presets.get(self.preset_cursor).cloned();
                        self.preview.clear(); self.preview_token.clear(); self.feedback.clear();
                    }
                    _ => {}
                }
                return None;
            }
            match input {
                MenuInput::Up => self.cursor = self.cursor.saturating_sub(1),
                MenuInput::Down => self.cursor = (self.cursor + 1).min(self.sources.len().saturating_sub(1)),
                MenuInput::Right => self.focus = 1,
                MenuInput::Confirm if !self.sources.is_empty() => {
                    self.selected = Some(self.cursor); self.preview.clear(); self.preview_token.clear(); self.feedback.clear();
                    self.focus = 1;
                }
                _ => {}
            }
        } else {
            match input {
                MenuInput::Left => self.focus = 0,
                MenuInput::Up => self.focus = self.focus.saturating_sub(1).max(1),
                MenuInput::Down => self.focus = (self.focus + 1).min(6),
                MenuInput::Confirm if self.focus == 6 => {
                    self.preset_mode = !self.preset_mode; self.focus = 0;
                    self.preview.clear(); self.preview_token.clear(); self.feedback.clear();
                }
                MenuInput::Confirm if self.preset_mode => return match self.focus {
                    1 => Some(Request::PresetSave), 2 => Some(Request::PresetPrepare),
                    3 => Some(Request::PresetArm), 4 => Some(Request::DisarmAll), _ => None,
                },
                MenuInput::Confirm => return Some(match self.focus {
                    1 => Request::Prepare, 2 => Request::ApplyOnce, 3 => Request::Arm,
                    4 => Request::Disarm, _ => Request::DisarmAll,
                }),
                _ => {}
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_is_explicit_and_changing_source_clears_preview() {
        let mut state = State::new("CH-0001".into(), "Example".into(), false, vec![Source { index: 2, name: "one".into(), master: "CH-0002".into(), assessment: "candidate".into() }, Source { index: 5, name: "two".into(), master: "CH-0003".into(), assessment: "unresolved".into() }], vec![]);
        assert!(state.selected_source().is_none());
        state.handle(MenuInput::Right); assert_eq!(state.handle(MenuInput::Confirm), Some(Request::Prepare));
        assert!(state.selected_source().is_none());
        state.handle(MenuInput::Left); state.handle(MenuInput::Down); state.handle(MenuInput::Confirm);
        assert_eq!(state.selected_source().unwrap().index, 5);
        state.preview = "old preview".into(); state.handle(MenuInput::Left); state.handle(MenuInput::Up); state.handle(MenuInput::Confirm);
        assert_eq!(state.selected_source().unwrap().index, 2); assert!(state.preview.is_empty());
    }
}
