//! Immutable frozen-register projections. These are not scanline-history captures.
use super::*;
use serde::Serialize;

#[derive(Serialize)]
pub struct Layer {
    pub name: String,
    pub kind: &'static str,
    pub width: usize,
    pub height: usize,
    #[serde(skip)]
    pub pixels: Vec<u16>,
}
#[derive(Serialize)]
pub struct Object {
    pub slot: usize,
    pub attrs: [u16; 3],
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub affine: bool,
    pub disabled: bool,
    pub matrix: [i16; 4],
    pub owner: u32,
}
#[derive(Serialize)]
pub struct Frozen {
    pub layers: Vec<Layer>,
    pub objects: Vec<Object>,
    pub window_controls: Vec<u8>,
    pub dispcnt: u16,
    pub bgcnt: [u16; 4],
    pub scroll: [[u16; 2]; 4],
    pub window_registers: [u16; 6],
    pub blend: [u16; 3],
}
fn layer(name: String, kind: &'static str, width: usize, height: usize, pixels: Vec<u16>) -> Layer {
    Layer {
        name,
        kind,
        width,
        height,
        pixels,
    }
}
/// Uses immutable renderer helpers; no bus reads, line stepping or render_scanline.
pub fn capture(mem: &MemMap) -> Result<Frozen, String> {
    if mem.io.len() != 0x400
        || mem.vram.len() != 0x18000
        || mem.oam.len() != 0x400
        || mem.palette.len() != 0x400
        || mem
            .ffta_sprites
            .active
            .as_ref()
            .is_some_and(|s| s.len() > 4096)
    {
        return Err("Unexpected PPU buffer/host-object bounds".into());
    }
    let d = io16(mem, 0);
    let mode = d & 7;
    let width = render_width();
    let mut layers = Vec::new();
    for bg in 0..4 {
        if d & (0x100 << bg) == 0 {
            continue;
        }
        let text = mode == 0 || mode == 1 && bg < 2;
        let affine = mode == 1 && bg == 2 || mode == 2 && bg >= 2;
        if text {
            let mut pixels = vec![TRANSPARENT; 240 * 160];
            for y in 0..160 {
                let mut row = [TRANSPARENT; MAX_RENDER_WIDTH];
                render_text_bg_hardware(mem, bg, y, &mut row, 0, 240);
                pixels[y * 240..(y + 1) * 240].copy_from_slice(&row[..240]);
            }
            layers.push(layer(
                format!("bg{bg}"),
                "text-frozen-native-projection",
                240,
                160,
                pixels,
            ));
        } else if affine {
            // The original helper's live affine reference is scanline-specific.
            // Expose the raw map instead of inventing the previous160 references.
            let cnt = io16(mem, 8 + 2 * bg);
            let size = 16usize << ((cnt >> 14) & 3);
            let side = size * 8;
            let mut pixels = vec![TRANSPARENT; side * side];
            let map = ((cnt >> 8) & 31) as usize * 0x800;
            let chars = ((cnt >> 2) & 3) as usize * 0x4000;
            for y in 0..side {
                for x in 0..side {
                    let tile = mem
                        .vram
                        .get(map + y / 8 * size + x / 8)
                        .copied()
                        .unwrap_or(0) as usize;
                    let off = chars + tile * 64 + y % 8 * 8 + x % 8;
                    let index = if off < 0x10000 {
                        mem.vram[off] as usize
                    } else {
                        0
                    };
                    if index != 0 {
                        pixels[y * side + x] = pal16(mem, index);
                    }
                }
            }
            layers.push(layer(
                format!("bg{bg}"),
                "affine-raw-map-no-screen-transform",
                side,
                side,
                pixels,
            ));
        } else if (3..=5).contains(&mode) && bg == 2 {
            let mut pixels = vec![TRANSPARENT; 240 * 160];
            for y in 0..160 {
                let mut row = [TRANSPARENT; MAX_RENDER_WIDTH];
                render_bitmap_bg(mem, mode, d, y, &mut row);
                pixels[y * 240..(y + 1) * 240].copy_from_slice(&row[extra()..extra() + 240]);
            }
            layers.push(layer(
                "bg2".into(),
                "bitmap-frozen-native-projection",
                240,
                160,
                pixels,
            ));
        }
    }
    let mut objects = Vec::new();
    for slot in 0..128 {
        let attrs = std::array::from_fn(|i| io_oam(mem, slot * 8 + i * 2));
        let [a0, a1, _] = attrs;
        let affine = a0 & 0x100 != 0;
        let shape = (a0 >> 14) as usize;
        let (w, h) = if shape < 3 {
            OBJ_SIZES[shape][(a1 >> 14) as usize]
        } else {
            (0, 0)
        };
        let x = (a1 & 511) as i32;
        let y = (a0 & 255) as i32;
        let group = ((a1 >> 9) & 31) as usize * 32;
        objects.push(Object {
            slot,
            attrs,
            x: if x >= 240 { x - 512 } else { x },
            y: if y >= 160 { y - 256 } else { y },
            width: w,
            height: h,
            affine,
            disabled: !affine && a0 & 0x200 != 0 || shape == 3,
            matrix: std::array::from_fn(|i| io_oam(mem, group + 6 + i * 8) as i16),
            owner: mem.ffta_sprites.oam_owners.get(slot).copied().unwrap_or(0),
        });
    }
    let mut obj_pixels = vec![TRANSPARENT; width * 160];
    let mut controls = vec![0; 240 * 160];
    for y in 0..160 {
        let mut row = [OBJ_EMPTY; MAX_RENDER_WIDTH];
        let mut win = [false; MAX_RENDER_WIDTH];
        if d & 0x1000 != 0 {
            render_sprites_extent(mem, d, y, &mut row, &mut win, extra(), width);
        }
        for x in 0..width {
            obj_pixels[y * width + x] = row[x].color;
        }
        let window = UiWindowRow::new(mem, d, y);
        for x in 0..240 {
            controls[y * 240 + x] = window.control(x as u8, win[x + extra()]) as u8;
        }
    }
    layers.push(layer(
        "objects".into(),
        "frozen-renderer-OBJ-projection-no-final-owner-proof",
        width,
        160,
        obj_pixels,
    ));
    Ok(Frozen {
        layers,
        objects,
        window_controls: controls,
        dispcnt: d,
        bgcnt: std::array::from_fn(|b| io16(mem, 8 + 2 * b)),
        scroll: std::array::from_fn(|b| [io16(mem, 16 + 4 * b) & 511, io16(mem, 18 + 4 * b) & 511]),
        window_registers: std::array::from_fn(|i| io16(mem, 0x40 + i * 2)),
        blend: std::array::from_fn(|i| io16(mem, 0x50 + i * 2)),
    })
}
fn io_oam(mem: &MemMap, off: usize) -> u16 {
    u16::from_le_bytes([mem.oam[off], mem.oam[off + 1]])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Machine;
    #[test]
    fn frozen_text_flips_palette_windows_and_oam_preserve_complete_state() {
        crate::mem::set_extra(0);
        crate::mem::set_extra_y(0);
        let mut m = Machine::new(vec![0; 192]);
        m.bus.io[0..2].copy_from_slice(&0x2100u16.to_le_bytes());
        m.bus.io[8..10].copy_from_slice(&0x0100u16.to_le_bytes());
        m.bus.vram[0x800..0x802].copy_from_slice(&0x1401u16.to_le_bytes());
        m.bus.vram[32..64].fill(0x12);
        m.bus.palette[34..36].copy_from_slice(&0x03e0u16.to_le_bytes());
        m.bus.io[0x40..0x42].copy_from_slice(&0x0008u16.to_le_bytes());
        m.bus.io[0x44..0x46].copy_from_slice(&0x0008u16.to_le_bytes());
        m.bus.io[0x48] = 1;
        let before = m.save_state().unwrap();
        let dump = capture(&m.bus).unwrap();
        assert_eq!(before, m.save_state().unwrap());
        assert_eq!(dump.layers[0].pixels[0], 0x03e0);
        assert_eq!(dump.window_controls[0], 1);
        assert_eq!(dump.window_controls[8], 0);
        assert_eq!(dump.objects.len(), 128);
    }
    #[test]
    fn affine_map_and_bitmap_views_are_explicit_not_raster_history() {
        crate::mem::set_extra(0);
        crate::mem::set_extra_y(0);
        let mut m = Machine::new(vec![]);
        m.bus.io[..2].copy_from_slice(&0x0402u16.to_le_bytes());
        m.bus.palette[2..4].copy_from_slice(&31u16.to_le_bytes());
        m.bus.vram[..64].fill(1);
        let dump = capture(&m.bus).unwrap();
        assert_eq!(dump.layers[0].kind, "affine-raw-map-no-screen-transform");
        assert_eq!(dump.layers[0].width, 128);
        m.bus.io[..2].copy_from_slice(&0x0403u16.to_le_bytes());
        m.bus.vram[..2].copy_from_slice(&0x7c00u16.to_le_bytes());
        let dump = capture(&m.bus).unwrap();
        assert_eq!(dump.layers[0].pixels[0], 0x7c00);
    }
}
