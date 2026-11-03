// Band-limited screen-grid + scaling pass.
//
// The grid algorithm: every subpixel stripe (3 vertical BGR stripes per
// source pixel) and every scanline row has a smooth polynomial aperture
// profile; for each output pixel we integrate that profile analytically
// over the output pixel's source-space footprint (the antiderivative is
// closed-form). All frequency content above the output Nyquist is removed
// exactly, so the grid cannot moire at any window size, integer scale or
// not, and energy is conserved as the window scales. This is a clean-room
// implementation of the published band-limited-grid integration math
// (aperture profiles (1-x^2-x^4+x^6)^2 horizontally, (1-2x^4+x^6)^2
// vertically).
//
// With grid_strength = 0 the pass degrades to exact area-averaged ("sharp
// pixel") scaling in display-encoded space: pixel-perfect at integer
// scales, alias-free at fractional ones.

struct Params {
    src_size: vec2f,     // visible view size in source pixels (cols, rows), may be fractional
    dst_origin: vec2f,   // letterbox offset, physical px
    dst_size: vec2f,     // scaled image size, physical px
    grid_strength: f32,  // 0 = flat scaling, 1 = full grid
    gain: f32,           // pre-linearization gain inside the grid model
    gamma: f32,          // grid model linearization exponent (pure pow)
    tint: f32,           // stripe self-transmission (diagonal subpixel tint)
    stripe_d: f32,       // stripe aperture half-width, subpixel units
    row_d: f32,          // row aperture half-width, pixel units
    bgr: f32,            // 1.0 = blue stripe leftmost (panel order)
    srgb_surface: f32,   // 1.0 = surface view re-encodes sRGB; pre-decode output
    view_x0: f32,        // first visible source column (zoom crop, may be fractional)
    view_y0: f32,        // first visible source row (zoom crop)
    tex_w: f32,          // uploaded frame size in texels
    tex_h: f32,
    scale_mode: f32,     // 0 = off (exact area average), 1 = nearest, 2 = sharp bilinear, 3 = EPX
}

@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var<uniform> P: Params;

struct VsOut {
    @builtin(position) pos: vec4f,
}

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VsOut {
    // Fullscreen triangle.
    var out: VsOut;
    let x = f32(i32(vi & 1u) * 4 - 1);
    let y = f32(i32(vi >> 1u) * 4 - 1);
    out.pos = vec4f(x, y, 0.0, 1.0);
    return out;
}

fn fetch(x: i32, y: i32) -> vec3f {
    // Absolute texel coordinates inside the uploaded frame. The zoom crop is
    // applied to the sampling position in fs_main, so pixels just outside the
    // crop are real neighbours rather than clamped copies of the edge.
    let c = vec2i(clamp(x, 0, i32(P.tex_w) - 1), clamp(y, 0, i32(P.tex_h) - 1));
    return textureLoad(src, c, 0).rgb;
}

// 1-D overlap of the box footprint [p-du/2, p+du/2] with source cell
// [j, j+1], normalized by du: exact area-average ("sharp pixel") weight.
fn box_overlap(j: f32, p: f32, du: f32) -> f32 {
    let lo = max(j, p - du * 0.5);
    let hi = min(j + 1.0, p + du * 0.5);
    return max(hi - lo, 0.0) / du;
}

// Exact area-averaged scaling in encoded space.
fn flat_sample(p: vec2f, du: vec2f) -> vec3f {
    let ix = i32(floor(p.x));
    let iy = i32(floor(p.y));
    var acc = vec3f(0.0);
    for (var dy = -1; dy <= 1; dy++) {
        let wy = box_overlap(f32(iy + dy), p.y, du.y);
        if (wy <= 0.0) { continue; }
        for (var dx = -1; dx <= 1; dx++) {
            let wx = box_overlap(f32(ix + dx), p.x, du.x);
            if (wx <= 0.0) { continue; }
            acc += fetch(ix + dx, iy + dy) * wx * wy;
        }
    }
    return acc;
}

// Nearest neighbour: the texel under the output pixel center.
fn nearest_sample(p: vec2f) -> vec3f {
    return fetch(i32(floor(p.x)), i32(floor(p.y)));
}

// Sharp bilinear: texels stay flat (pixel-perfect) and only the seam between
// two texels is blended, over exactly one output pixel. Equal-width pixels at
// integer scales, clean edges without blur at fractional ones.
fn sharp_sample(p: vec2f, scale: vec2f) -> vec3f {
    let e = round(p);                              // nearest texel edge
    let w = clamp((p - e) * scale + vec2f(0.5), vec2f(0.0), vec2f(1.0));
    let ex = i32(e.x);
    let ey = i32(e.y);
    let top = mix(fetch(ex - 1, ey - 1), fetch(ex, ey - 1), w.x);
    let bot = mix(fetch(ex - 1, ey), fetch(ex, ey), w.x);
    return mix(top, bot, w.y);
}

// EPX / Scale2x: each source texel becomes 2x2 sub-pixels; a sub-pixel takes
// the value of its adjacent neighbour when two orthogonal neighbours agree
// and the opposite pair does not (rounds diagonals, keeps flat areas flat).
// (x, y) are coordinates on the 2x grid.
fn epx_px(x: i32, y: i32) -> vec3f {
    let ix = x >> 1u;
    let iy = y >> 1u;
    let qx = x & 1;
    let qy = y & 1;
    let m = fetch(ix, iy);
    let a = fetch(ix, iy - 1);     // up
    let b = fetch(ix + 1, iy);     // right
    let c = fetch(ix - 1, iy);     // left
    let d = fetch(ix, iy + 1);     // down
    var r = m;
    if (qx == 0 && qy == 0) {
        if (all(c == a) && any(c != d) && any(a != b)) { r = a; }
    } else if (qx == 1 && qy == 0) {
        if (all(a == b) && any(a != c) && any(b != d)) { r = b; }
    } else if (qx == 0 && qy == 1) {
        if (all(d == c) && any(d != b) && any(c != a)) { r = c; }
    } else {
        if (all(b == d) && any(b != a) && any(d != c)) { r = d; }
    }
    return r;
}

// EPX image sampled like sharp_sample, on the 2x grid: flat sub-pixels with
// only the seam between two of them blended over one output pixel.
fn epx_sample(p: vec2f, scale: vec2f) -> vec3f {
    let p2 = p * 2.0;
    let s2 = scale * 0.5;
    let e = round(p2);
    let w = clamp((p2 - e) * s2 + vec2f(0.5), vec2f(0.0), vec2f(1.0));
    let ex = i32(e.x);
    let ey = i32(e.y);
    let top = mix(epx_px(ex - 1, ey - 1), epx_px(ex, ey - 1), w.x);
    let bot = mix(epx_px(ex - 1, ey), epx_px(ex, ey), w.x);
    return mix(top, bot, w.y);
}

// Piecewise sRGB EOTF (decode): inverse of the encode an sRGB surface view
// applies on write. Our pipeline's output is already display-encoded bytes;
// when the only available surface format is *Srgb the hardware would encode
// them AGAIN, so we pre-decode the final color and the re-encode lands the
// intended bytes. Exact piecewise curve, not pow(2.2).
fn srgb_eotf(c: vec3f) -> vec3f {
    let lo = c / 12.92;
    let hi = pow((c + vec3f(0.055)) / 1.055, vec3f(2.4));
    return select(hi, lo, c <= vec3f(0.04045));
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4f {
    let q = in.pos.xy - P.dst_origin;
    var rgb = vec3f(0.0); // letterbox black (fixed point of the EOTF)
    if (q.x >= 0.0 && q.y >= 0.0 && q.x < P.dst_size.x && q.y < P.dst_size.y) {
        let scale = P.dst_size / P.src_size;
        let p = q / scale + vec2f(P.view_x0, P.view_y0); // source-space position, pixel units
        let du = 1.0 / scale;       // source pixels per output pixel

        var flat: vec3f;
        if (P.scale_mode > 2.5) {
            flat = epx_sample(p, scale);
        } else if (P.scale_mode > 1.5) {
            flat = sharp_sample(p, scale);
        } else if (P.scale_mode > 0.5) {
            flat = nearest_sample(p);
        } else {
            flat = flat_sample(p, du);
        }
        rgb = apply_grid(flat, p, scale);
    }
    if (P.srgb_surface > 0.5) {
        rgb = srgb_eotf(clamp(rgb, vec3f(0.0), vec3f(1.0)));
    }
    return vec4f(rgb, 1.0);
}
