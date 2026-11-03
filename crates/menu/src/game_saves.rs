//! Controller-friendly selection of numbered, raw cartridge saves.
use super::{Layout, Menu, Page};

#[derive(Clone)]
pub struct GameSaveSlot {
    pub name: String,
    pub status: String,
    pub occupied: bool,
    pub available: bool,
    pub current: bool,
}

pub(super) fn layout(n:usize,w:usize,h:usize)->Layout {super::save_forms::layout(Page::GameSaves,n,w,h)}
pub(super) fn draw(menu:&Menu,px:&mut[[u8;4]],w:usize,h:usize){super::save_forms::draw(menu,px,w,h);}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn game_save_picker_draws_and_mouse_actions_match_at_supported_sizes() {
        for (w, h) in [(480, 640), (640, 480), (1280, 720), (3840, 2160)] {
            let mut menu = Menu::unlocked_for_test(true, true);
            menu.set_game_saves(
                (1..=99)
                    .map(|slot| GameSaveSlot {
                        name: format!("ffta-slot-{slot:02}.sav"),
                        status: "Saved".into(),
                        occupied: true,
                        available: true,
                        current: slot == 99,
                    })
                    .collect(),
            );
            menu.open(Page::GameSaves);
            assert_eq!(menu.game_save_slot, 98);
            let l = menu.page_layout(w,h);
            assert!(l.rows_y + 4 * l.line_h <= h);
            for i in 0..4 {
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
            assert!(pixels.iter().any(|p| *p != [0, 0, 0, 255]));
        }
    }
}
