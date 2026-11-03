//! Explicit, thread-local observation of the existing host-menu text renderer.
use ab_glyph::{Font, FontRef, ScaleFont};
use std::cell::RefCell;

#[derive(Debug)]
pub struct TextRun {
    pub text: String,
    pub font: &'static str,
    pub size: f32,
    pub origin: [f32; 2],
    pub max_width: f32,
    pub measured_chars: usize,
    pub source_limit: bool,
    pub rendered_chars: usize,
    pub advance: f32,
    pub full_advance: f32,
    pub ascent: f32,
    pub descent: f32,
    pub ink_bounds: Option<[f32; 4]>,
    pub canvas: [u32; 2],
    pub width_stopped: bool,
    pub canvas_clipped: bool,
}
#[derive(Default)]
pub struct TextCapture {
    pub runs: Vec<TextRun>,
    pub dropped: usize,
}
pub struct Focus {
    pub page: String,
    pub selected: usize,
    pub label: String,
    pub capture: Option<String>,
    pub panel: [usize; 4],
    pub selected_bounds: Option<[usize; 4]>,
    pub first_visible: usize,
    pub dropdown: Option<DropdownFocus>,
}
pub struct DropdownFocus {
    pub label: String,
    pub index: usize,
    pub option: String,
    pub panel: [usize; 4],
    pub first_visible: usize,
}
thread_local! { static CAPTURE: RefCell<Option<TextCapture>> = const { RefCell::new(None) }; }

/// Observe a disposable redraw. Nested observation is refused; panic unwinding clears it.
pub fn capture(draw: impl FnOnce()) -> Result<TextCapture, &'static str> {
    CAPTURE.with(|slot| {
        if slot.borrow().is_some() {
            return Err("Text capture already active");
        }
        *slot.borrow_mut() = Some(TextCapture::default());
        Ok(())
    })?;
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            CAPTURE.with(|slot| {
                slot.borrow_mut().take();
            });
        }
    }
    let reset = Reset;
    draw();
    let result = CAPTURE.with(|slot| slot.borrow_mut().take().unwrap());
    drop(reset);
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn observe(
    font: &FontRef<'_>,
    x: f32,
    y: f32,
    text: &str,
    size: f32,
    title: bool,
    max: f32,
    w: u32,
    h: u32,
) {
    CAPTURE.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(capture) = slot.as_mut() else {
            return;
        };
        if capture.runs.len() == 512 {
            capture.dropped += 1;
            return;
        }
        let scaled = font.as_scaled(size);
        let (mut full, mut pen, mut last, mut drawn_last) = (x, x, None, None);
        let (mut measured, mut rendered, mut stopped) = (0, 0, false);
        let mut ink: Option<[f32; 4]> = None;
        for ch in text.chars().take(4096) {
            measured += 1;
            let id = font.glyph_id(ch);
            if let Some(prev) = last {
                full += scaled.kern(prev, id);
            }
            let advance = scaled.h_advance(id);
            full += advance;
            last = Some(id);
            if stopped {
                continue;
            }
            if let Some(prev) = drawn_last {
                pen += scaled.kern(prev, id);
            }
            if pen + advance > x + max {
                stopped = true;
                continue;
            }
            if let Some(outline) = font.outline_glyph(
                id.with_scale_and_position(size, ab_glyph::point(pen, y + scaled.ascent())),
            ) {
                let b = outline.px_bounds();
                ink = Some(match ink {
                    Some(a) => [
                        a[0].min(b.min.x),
                        a[1].min(b.min.y),
                        a[2].max(b.max.x),
                        a[3].max(b.max.y),
                    ],
                    None => [b.min.x, b.min.y, b.max.x, b.max.y],
                });
            }
            pen += advance;
            drawn_last = Some(id);
            rendered += 1;
        }
        capture.runs.push(TextRun {
            text: text.chars().take(256).collect(),
            font: if title {
                "Cinzel wght700"
            } else {
                "Inter wght600"
            },
            size,
            origin: [x, y],
            max_width: max,
            measured_chars: measured,
            source_limit: text.chars().nth(4096).is_some(),
            rendered_chars: rendered,
            advance: pen - x,
            full_advance: full - x,
            ascent: scaled.ascent(),
            descent: scaled.descent(),
            ink_bounds: ink,
            canvas: [w, h],
            width_stopped: stopped,
            canvas_clipped: ink
                .is_some_and(|b| b[0] < 0. || b[1] < 0. || b[2] > w as f32 || b[3] > h as f32),
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn observer_preserves_pixels_and_attributes_width_stop_and_canvas_clip() {
        let mut plain = vec![[12, 34, 56, 255]; 100 * 40];
        let mut observed = plain.clone();
        let draw = |pixels: &mut [[u8; 4]]| {
            let mut c = crate::canvas::Canvas::new(pixels, 100, 40);
            c.text(4., 4., "AVATAR LONG TEXT", [255; 3], 16., false, 24.);
            c.text(92., 30., "XYZ", [255; 3], 18., true, 100.);
            c.finish(pixels);
        };
        draw(&mut plain);
        let capture = capture(|| draw(&mut observed)).unwrap();
        assert_eq!(plain, observed);
        assert_eq!(capture.runs.len(), 2);
        let a = &capture.runs[0];
        assert!(
            a.width_stopped && a.rendered_chars < a.measured_chars && a.full_advance > a.max_width
        );
        assert!(!a.canvas_clipped);
        assert!(capture.runs[1].canvas_clipped);
        assert!(super::capture(|| assert!(super::capture(|| {}).is_err())).is_ok());
        assert!(super::capture(|| {}).unwrap().runs.is_empty());
    }
}
