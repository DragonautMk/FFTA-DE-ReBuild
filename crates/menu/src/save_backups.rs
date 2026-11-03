//! In-menu restore selection, with a safe default confirmation.
use super::{Layout, Menu, Page};
#[derive(Clone)]
pub struct BackupInfo {
    pub generation: Option<u64>,
    pub status: String,
}
impl Default for BackupInfo {
    fn default() -> Self {
        Self {
            generation: None,
            status: "Empty".into(),
        }
    }
}
pub(super) fn layout(page:Page,n:usize,w:usize,h:usize)->Layout {super::save_forms::layout(page,n,w,h)}
pub(super) fn draw(menu:&Menu,px:&mut[[u8;4]],w:usize,h:usize){super::save_forms::draw(menu,px,w,h);}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn backup_panels_fit_and_mouse_geometry_matches() {
        for (w, h) in [(480, 640), (1280, 720), (3840, 2160)] {
            for page in [Page::SaveBackups, Page::RestoreBackup] {
                let mut menu = Menu::new(true, true);
                menu.open(page);
                let n = crate::rows_of(page).len();
                let l = menu.page_layout(w,h);
                assert!(l.rows_y + n * l.line_h <= h);
                for i in 0..n {
                    assert_eq!(
                        menu.row_at(
                            l.panel_x + l.pad + 1,
                            l.rows_y + i * l.line_h + l.line_h / 2,
                            w,
                            h
                        ),
                        Some(i)
                    );
                }
                let mut px = vec![[0, 0, 0, 255]; w * h];
                draw(&menu, &mut px, w, h);
            }
        }
    }
}
