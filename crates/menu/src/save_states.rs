//! Controller-friendly slot picker and embedded screenshot preview.
use super::{Layout, Menu, Page};

#[derive(Clone)]
pub struct StateSlot {
    pub occupied: bool,
    /// 240x160 RGBA game screenshot; empty for a legacy state or unavailable slot.
    pub preview: Vec<[u8; 4]>,
    pub status: String,
}
impl Default for StateSlot {
    fn default() -> Self {
        Self {
            occupied: false,
            preview: Vec::new(),
            status: "Empty".into(),
        }
    }
}

pub(super) fn layout(n:usize,w:usize,h:usize)->Layout {super::save_forms::layout(Page::SaveStates,n,w,h)}
pub(super) fn draw(menu:&Menu,px:&mut[[u8;4]],w:usize,h:usize){super::save_forms::draw(menu,px,w,h);}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preview_and_actions_fit_and_mouse_hits_match_at_supported_sizes() {
        for (w, h) in [(480, 640), (640, 480), (1280, 720), (3840, 2160)] {
            let mut menu = Menu::new(true, true);
            menu.open(Page::SaveStates);
            menu.state_slots[0] = StateSlot {
                occupied: true,
                preview: vec![[200, 20, 80, 255]; 240 * 160],
                status: "Saved".into(),
            };
            let l = menu.page_layout(w,h);
            assert!(l.rows_y + 5 * l.line_h <= h);
            for i in 0..5 {
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
            let mut pixels = vec![[0, 0, 0, 255]; w * h];
            draw(&menu, &mut pixels, w, h);
            assert!(pixels.iter().any(|p| *p == [200, 20, 80, 255]));
        }
    }
}
