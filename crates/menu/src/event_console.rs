//! Rendering only; the frontend owns event collection, filtering and input.
use crate::canvas::Canvas;

pub struct Display<'a> {
    pub language: input_config::Language,
    pub status: &'a str,
    pub filter: &'a str,
    pub range: &'a str,
    pub notice: &'a str,
    pub lines: &'a [String],
}

/// Draw into a caller-owned surface. Long rows are clipped by the native font
/// renderer; no event payload is modified to fit the display.
pub fn draw(view: &Display<'_>, pixels: &mut [[u8; 4]], width: usize, height: usize) {
    if width == 0 || height == 0 || width.checked_mul(height) != Some(pixels.len()) { return; }
    pixels.fill([15, 20, 27, 255]);
    let mut canvas = Canvas::new(pixels, width, height);
    let w = width as f32;
    let h = height as f32;
    let margin = 12.0;
    let available = (w - margin * 2.0).max(0.0);
    canvas.text(margin, 10.0, view.language.text("Event console"), [232, 224, 195], 20.0, true, available);
    canvas.text(margin, 35.0, view.status, [181, 204, 205], 13.0, false, available);
    canvas.text(margin, 54.0, view.filter, [231, 210, 147], 13.0, false, available);
    canvas.text(margin, 71.0, view.range, [165, 183, 187], 12.0, false, available);
    let bottom = (h - 56.0).max(91.0);
    for (i, line) in view.lines.iter().enumerate() {
        let y = 91.0 + i as f32 * 17.0;
        if y + 16.0 > bottom { break; }
        canvas.text(margin, y, line, [218, 226, 226], 13.0, false, available);
    }
    canvas.text(margin, (h - 50.0).max(0.0), view.notice, [240, 187, 130], 12.0, false, available);
    canvas.text(margin, (h - 31.0).max(0.0), view.language.text("/ Search  C Category  L Level  P Pause view  A Poll  ↑↓ Scroll  End Follow"), [165, 183, 187], 12.0, false, available);
    canvas.text(margin, (h - 16.0).max(0.0), view.language.text("Ctrl+C Copy  Ctrl+S Save  Esc Hide"), [165, 183, 187], 12.0, false, available);
    canvas.finish(pixels);
}
