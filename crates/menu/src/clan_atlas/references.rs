//! Read-only descriptions. These types have deliberately no executable callback.
use super::*;
use crate::MenuInput;
use std::sync::OnceLock;

#[derive(Clone, Debug)]
pub struct OperationReference {
    pub id: String,
    pub title: String,
    pub section: usize,
    pub details: String,
}
pub(crate) struct Task {
    pub id: &'static str,
    pub section: usize,
    pub done: bool,
    pub title: &'static str,
    pub details: &'static str,
}
pub(crate) fn planned() -> &'static [Task] {
    static TASKS: OnceLock<Vec<Task>> = OnceLock::new();
    TASKS.get_or_init(|| {
        include_str!("../../assets/debug_v2.tsv")
            .lines()
            .filter(|l| !l.starts_with('#'))
            .map(|l| {
                let f: Vec<_> = l.splitn(5, '\t').collect();
                Task {
                    id: f[0],
                    section: f[1].parse().expect("task group"),
                    done: f[2] == "true",
                    title: f[3],
                    details: f[4],
                }
            })
            .collect()
    })
}
pub fn reference_group(route:&str)->usize {
    DEBUG_SECTIONS.iter().position(|name|route.starts_with(&format!("Debug / {name} /"))).unwrap_or(0)
}

/// Localize the card's presentation, leaving the registry and route grouping
/// untouched. Unknown text and individual parameter values remain verbatim.
fn operation_details(language:input_config::Language, details:&str)->String {
    details.lines().map(|line| {
        if let Some(route)=line.strip_prefix("Route: ") {
            let route=route.split(" / ").map(|part|language.text(part)).collect::<Vec<_>>().join(" / ");
            return format!("{}: {route}",language.text("Route"));
        }
        if line=="CLI: No CLI command" {
            return format!("CLI: {}",language.text("No CLI command"));
        }
        for label in ["Prerequisites","Parameters","Effects","Result","Limits","Handler"] {
            if let Some(value)=line.strip_prefix(label).and_then(|tail|tail.strip_prefix(": ")) {
                return format!("{}: {value}",language.text(label));
            }
        }
        match line {
            "Menu available · existing handler" | "CLI only · no execution from this card" |
            "This card is read-only. Use the menu action or the explicit CLI." => language.text(line).to_owned(),
            _ => line.to_owned(),
        }
    }).collect::<Vec<_>>().join("\n")
}

impl Menu {
    pub fn atlas_reference_active(&self) -> bool {
        self.page == Page::CliReference || (self.page == Page::Debug && self.atlas.planned)
    }
    pub(crate) fn atlas_reference_items(&self) -> Vec<(String, String)> {
        let section = self.atlas.selected_section();
        if self.atlas.planned {
            planned().iter().filter(|t|section==0||t.section==section).map(|t|(
                format!("{} · {}",t.id,t.title),
                format!("{}\n{}\n{}\n{}",t.id,self.tr(if t.done {"Completed in the current checklist"}else{"Planned · unavailable"}),t.details,self.tr("Checklist reference; this view does not execute operations."))
            )).collect()
        } else {
            self.atlas
                .operations
                .iter()
                .filter(|o| section == 0 || o.section == section)
                .map(|o| (format!("{} · {}", o.id, o.title), operation_details(self.language(),&o.details)))
                .collect()
        }
    }
    pub(crate) fn atlas_reference_cursor(&self) -> usize {
        if self.atlas.planned {
            self.atlas.planned_cursor
        } else {
            self.atlas.reference_cursor
        }
    }
    pub(crate) fn atlas_reference_select(&mut self, index: usize) {
        let index = index.min(self.atlas_reference_items().len().saturating_sub(1));
        if self.atlas_reference_cursor() != index {
            self.atlas.detail_scroll = 0;
        }
        if self.atlas.planned {
            self.atlas.planned_cursor = index;
        } else {
            self.atlas.reference_cursor = index;
        }
    }
    pub(crate) fn atlas_reference_first(&self, w: usize, h: usize) -> (usize, usize, usize) {
        let g = geometry(w, h);
        let pitch = (g.scale_milli * 36 / 1000).max(25);
        let visible = g.list.h.saturating_sub(12) / pitch;
        let len = self.atlas_reference_items().len();
        let selected = self.atlas_reference_cursor().min(len.saturating_sub(1));
        let mut first = self
            .atlas
            .reference_scroll
            .get()
            .min(len.saturating_sub(visible));
        if selected < first {
            first = selected;
        }
        if selected >= first + visible.max(1) {
            first = selected + 1 - visible.max(1);
        }
        self.atlas.reference_scroll.set(first);
        (first, visible, pitch)
    }
    pub(crate) fn atlas_reference_input(&mut self, input: MenuInput) -> Option<crate::MenuAction> {
        let len = self.atlas_reference_items().len();
        let cursor = self.atlas_reference_cursor();
        match input {
            MenuInput::Up if len > 0 => self.atlas_reference_select((cursor + len - 1) % len),
            MenuInput::Down if len > 0 => self.atlas_reference_select((cursor + 1) % len),
            MenuInput::Left => {
                self.atlas.detail_scroll = self.atlas.detail_scroll.saturating_sub(3)
            }
            MenuInput::Right => {
                self.atlas.detail_scroll = self.atlas.detail_scroll.saturating_add(3).min(self.atlas.detail_limit.get())
            }
            MenuInput::Cancel => {
                if self.atlas.planned {
                    self.atlas.planned = false;
                } else {
                    self.back_to_main();
                }
            }
            _ => {}
        }
        None
    }
    pub(crate) fn atlas_reference_hover(&mut self, x: usize, y: usize, w: usize, h: usize) {
        let g = geometry(w, h);
        let (first, visible, pitch) = self.atlas_reference_first(w, h);
        if g.list.hit(x, y) && y >= g.list.y + 6 {
            let i = (y - g.list.y - 6) / pitch;
            if i < visible {
                self.atlas_reference_select(first + i);
            }
        }
    }
    /// Detail scrolling is local; pointer coordinates use the same fixed rectangles as drawing.
    pub fn atlas_reference_wheel(
        &mut self,
        up: bool,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
    ) -> bool {
        if !self.atlas_reference_active() {
            return false;
        }
        if geometry(w, h).detail.hit(x, y) {
            self.atlas.detail_scroll = if up {
                self.atlas.detail_scroll.saturating_sub(3)
            } else {
                self.atlas.detail_scroll.saturating_add(3).min(self.atlas.detail_limit.get())
            };
        } else {
            self.atlas_reference_input(if up { MenuInput::Up } else { MenuInput::Down });
        }
        true
    }
    /// Select the planned reference with the keyboard, without entering an action row.
    pub fn atlas_reference_tab(&mut self) {
        self.cancel_debug_unlock();
        if !self.debug_enabled(){return;}
        if matches!(self.page, Page::Debug | Page::CliReference) {
            self.atlas.planned = !self.atlas.planned;
            self.atlas.detail_scroll = 0;
            self.atlas.reference_scroll.set(0);
        }
    }
}

#[cfg(test)]
mod localization_tests {
    use super::*;
    use input_config::Language;

    #[test]
    fn readonly_operation_labels_and_routes_follow_the_language() {
        let original="control.step · Step\nMenu available · existing handler\nCLI: No CLI command\nRoute: Debug / Sessioni e casi / Console comandi\nParameters: --keys (mask): {Route} / value\nThis card is read-only. Use the menu action or the explicit CLI.";
        let en=operation_details(Language::default(),original);
        let it=operation_details(Language::parse("it"),original);
        assert!(en.contains("Route: Debug / Sessions and cases / Command console"));
        assert!(it.contains("Percorso: Debug / Sessioni e casi / Console comandi"));
        assert!(it.contains("Menu disponibile · handler esistente")&&it.contains("CLI: Nessun comando CLI"));
        assert!(en.contains("Parameters: --keys (mask): {Route} / value"));
        assert!(it.contains("Parametri: --keys (mask): {Route} / value"));
        assert_eq!(reference_group("Debug / Sessioni e casi / Console comandi"),1);
        assert!(original.contains("Route: Debug / Sessioni e casi"));
    }
}
