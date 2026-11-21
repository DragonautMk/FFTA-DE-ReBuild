use eframe::egui::{self, RichText};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy)]
pub enum Kind {
    Rom,
    Folder,
}

pub enum Selection {
    Rom(PathBuf),
    Folder(PathBuf),
    Cancel,
}

struct Entry {
    path: PathBuf,
    name: String,
    directory: bool,
}

#[derive(Clone, PartialEq, Eq)]
enum Location {
    #[cfg(windows)]
    ThisPc,
    Folder(PathBuf),
}

pub struct PathPicker {
    kind: Kind,
    location: Location,
    history: Vec<Location>,
    history_index: usize,
    input: String,
    entries: Vec<Entry>,
    error: Option<String>,
}

impl PathPicker {
    pub fn new(kind: Kind) -> Self {
        #[cfg(windows)]
        let location = Location::ThisPc;
        #[cfg(not(windows))]
        let location = Location::Folder(dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")));
        let input = location_label(&location);
        let mut picker = Self {
            kind, location: location.clone(), history: vec![location], history_index: 0,
            input, entries: Vec::new(), error: None,
        };
        picker.refresh();
        picker
    }

    fn refresh(&mut self) {
        self.entries.clear();
        self.error = None;
        #[cfg(windows)]
        if matches!(self.location, Location::ThisPc) {
            for letter in b'A'..=b'Z' {
                let path = PathBuf::from(format!("{}:\\", letter as char));
                if path.is_dir() {
                    self.entries.push(Entry { name: path.display().to_string(), path, directory: true });
                }
            }
            return;
        }
        let Location::Folder(directory) = &self.location else { return };
        match std::fs::read_dir(directory) {
            Ok(items) => {
                for item in items.flatten() {
                    let path = item.path();
                    let directory = path.is_dir();
                    if directory || (matches!(self.kind, Kind::Rom) && is_gba(&path)) {
                        self.entries.push(Entry {
                            name: item.file_name().to_string_lossy().into_owned(),
                            path,
                            directory,
                        });
                    }
                }
                self.entries.sort_by(|a, b| {
                    b.directory.cmp(&a.directory)
                        .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                });
            }
            Err(error) => self.error = Some(format!("Cannot open folder: {error}")),
        }
    }

    fn visit(&mut self, location: Location) {
        if location == self.location { return; }
        self.history.truncate(self.history_index + 1);
        self.history.push(location.clone());
        self.history_index += 1;
        self.set_location(location);
    }

    fn set_location(&mut self, location: Location) {
        self.input = location_label(&location);
        self.location = location;
        self.refresh();
    }

    fn go_back(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            self.set_location(self.history[self.history_index].clone());
        }
    }

    fn go_forward(&mut self) {
        if self.history_index + 1 < self.history.len() {
            self.history_index += 1;
            self.set_location(self.history[self.history_index].clone());
        }
    }

    fn visit_folder(&mut self, path: PathBuf) {
        if path.is_dir() { self.visit(Location::Folder(path)); }
        else { self.error = Some("The folder does not exist or cannot be opened.".into()); }
    }

    fn open_input(&mut self) -> Option<Selection> {
        let path = PathBuf::from(self.input.trim());
        if path.is_dir() {
            self.visit_folder(path);
            None
        } else if matches!(self.kind, Kind::Rom) && path.is_file() && is_gba(&path) {
            Some(Selection::Rom(path))
        } else {
            self.error = Some("Enter an existing folder or a .gba ROM file.".into());
            None
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Option<Selection> {
        let mut open = true;
        let mut result = None;
        let title = match self.kind { Kind::Rom => "Choose GBA ROM", Kind::Folder => "Choose install folder" };
        egui::Window::new(title)
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_width(520.0)
            .min_width(280.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.add_enabled(self.history_index > 0, egui::Button::new("Back")).clicked() {
                        self.go_back();
                    }
                    if ui.add_enabled(self.history_index + 1 < self.history.len(), egui::Button::new("Forward")).clicked() {
                        self.go_forward();
                    }
                });
                ui.horizontal(|ui| {
                    let response = ui.add(egui::TextEdit::singleline(&mut self.input).desired_width(f32::INFINITY));
                    let enter = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    if ui.button("Open path").clicked() || enter {
                        result = self.open_input();
                    }
                });
                ui.label(RichText::new(location_label(&self.location)).small());
                ui.separator();
                let mut clicked = None;
                egui::ScrollArea::vertical()
                    .id_salt("installer_path_entries")
                    .max_height((ctx.content_rect().height() - 210.0).max(80.0))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if let Location::Folder(directory) = &self.location {
                            if let Some(parent) = directory.parent() {
                                if ui.selectable_label(false, "[..] Parent folder").clicked() {
                                    clicked = Some((parent.to_path_buf(), true));
                                }
                            }
                            #[cfg(windows)]
                            if directory.parent().is_none() && ui.selectable_label(false, "[..] This PC").clicked() {
                                clicked = Some((PathBuf::new(), true));
                            }
                        }
                        if self.entries.is_empty() {
                            ui.label("No matching files or folders here.");
                        }
                        for entry in &self.entries {
                            let prefix = if entry.directory { "[Folder] " } else { "[ROM] " };
                            if ui.selectable_label(false, format!("{prefix}{}", entry.name)).clicked() {
                                clicked = Some((entry.path.clone(), entry.directory));
                            }
                        }
                    });
                if let Some((path, directory)) = clicked {
                    #[cfg(windows)]
                    if directory && path.as_os_str().is_empty() { self.visit(Location::ThisPc); }
                    else if directory { self.visit_folder(path); }
                    #[cfg(not(windows))]
                    if directory { self.visit_folder(path); }
                    else { result = Some(Selection::Rom(path)); }
                }
                if let Some(error) = &self.error {
                    ui.colored_label(egui::Color32::from_rgb(229, 91, 91), error);
                }
                ui.separator();
                ui.horizontal(|ui| {
                    if matches!(self.kind, Kind::Folder) && ui.button("Use this folder").clicked() {
                        if let Location::Folder(directory) = &self.location {
                            result = Some(Selection::Folder(directory.clone()));
                        } else {
                            self.error = Some("Choose a drive or folder first.".into());
                        }
                    }
                    if ui.button("Cancel").clicked() { result = Some(Selection::Cancel); }
                });
            });
        if result.is_none() && !open { Some(Selection::Cancel) } else { result }
    }
}

fn location_label(location: &Location) -> String {
    match location {
        #[cfg(windows)]
        Location::ThisPc => "This PC".into(),
        Location::Folder(path) => path.display().to_string(),
    }
}

fn is_gba(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("gba"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rom_filter_accepts_uppercase_extension_only() {
        assert!(is_gba(Path::new("game.GBA")));
        assert!(!is_gba(Path::new("game.sav")));
        assert!(!is_gba(Path::new("game.gba.zip")));
    }

    #[test]
    fn navigation_discards_forward_history_after_new_folder() {
        let current = std::env::current_dir().unwrap();
        let mut picker = PathPicker::new(Kind::Folder);
        picker.visit(Location::Folder(current.clone()));
        picker.go_back();
        picker.visit(Location::Folder(current));
        assert_eq!(picker.history_index + 1, picker.history.len());
    }
}
