//! Persistent visibility preference; existing debug action ownership checks remain separate.
use super::*;
const SEQUENCE: [MenuInput; 8] = [
    MenuInput::Up,
    MenuInput::Up,
    MenuInput::Down,
    MenuInput::Down,
    MenuInput::Left,
    MenuInput::Left,
    MenuInput::Right,
    MenuInput::Right,
];
const PREFIX: [usize; 8] = [0, 1, 0, 0, 0, 0, 0, 0];

pub(super) fn debug_page(page: Page) -> bool {
    matches!(
        page,
        Page::Debug
            | Page::DebugCatalog
            | Page::CheatCatalog
            | Page::ContentSelector
            | Page::ToolInputs
            | Page::DebugTools
            | Page::ScriptLibrary
            | Page::DebugTool
            | Page::CliReference
            | Page::GameSaves
            | Page::SwitchGameSave
            | Page::ReplaceGameSave
    )
}
impl Menu {
    pub fn debug_enabled(&self) -> bool {
        self.debug_enabled.get()
    }
    /// Configure visibility at startup, including an explicit manual opt-out.
    pub fn set_debug_enabled(&mut self, enabled: bool) {
        if self.debug_enabled.replace(enabled) == enabled {
            return;
        }
        self.cancel_debug_unlock();
        if !enabled {
            if self.atlas.domain == clan_atlas::Domain::Debug
                || debug_page(self.page)
                || self.navigation.parents.iter().any(|p| debug_page(p.page))
            {
                self.atlas_section(clan_atlas::Domain::Game, 0);
                self.navigation.parents.clear();
            }
            self.selected = self.selected.min(self.rows().len() - 1);
        }
    }
    pub fn cancel_debug_unlock(&mut self) {
        self.unlock_progress = 0;
    }

    pub(super) fn observe_debug_unlock(&mut self, input: MenuInput) -> bool {
        if self.debug_enabled()
            || self.capture.is_some()
            || self.content_text_focus()
            || self.dropdown.is_some()
            || matches!(input, MenuInput::Confirm | MenuInput::Cancel)
        {
            self.cancel_debug_unlock();
            return false;
        }
        while self.unlock_progress > 0 && SEQUENCE[self.unlock_progress] != input {
            self.unlock_progress = PREFIX[self.unlock_progress - 1];
        }
        if SEQUENCE[self.unlock_progress] == input {
            self.unlock_progress += 1;
        }
        if self.unlock_progress == SEQUENCE.len() {
            self.cancel_debug_unlock();
            true
        } else {
            false
        }
    }
    #[cfg(test)]
    pub(crate) fn unlocked_for_test(audio_live: bool, initial: bool) -> Self {
        let mut menu = Self::new(audio_live, initial);
        menu.set_debug_enabled(true);
        menu
    }
}

#[cfg(test)]
mod tests;
