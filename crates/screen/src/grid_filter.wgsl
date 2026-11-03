// Antiderivative of the squared horizontal stripe profile (1-x^2-x^4+x^6)^2.
fn smear_f_x(z: f32) -> f32 {
    let z2 = z * z;
    var zn = z;
    var ret = 0.0;
    // coefficients of the odd-power antiderivative terms
    var co = array<f32, 7>(1.0, -2.0 / 3.0, -0.2, 4.0 / 7.0, -1.0 / 9.0, -2.0 / 11.0, 1.0 / 13.0);
    for (var i = 0; i < 7; i++) {
        ret += zn * co[i];
        zn *= z2;
    }
    return ret;
}

// Antiderivative of the squared row profile (1-2x^4+x^6)^2.
fn smear_f_y(z: f32) -> f32 {
    let z2 = z * z;
    var zn = z;
    var ret = 0.0;
    var co = array<f32, 7>(1.0, 0.0, -4.0 / 5.0, 2.0 / 7.0, 4.0 / 9.0, -4.0 / 11.0, 1.0 / 13.0);
    for (var i = 0; i < 7; i++) {
        ret += zn * co[i];
        zn *= z2;
    }
    return ret;
}

// Integral of an aperture profile (half-width d, centered at offset x from
// the output pixel center) over a footprint of width dx, normalized by dx.
fn smear_x(x: f32, dx: f32, d: f32) -> f32 {
    let zl = clamp((x - dx * 0.5) / d, -1.0, 1.0);
    let zh = clamp((x + dx * 0.5) / d, -1.0, 1.0);
    return d * (smear_f_x(zh) - smear_f_x(zl)) / dx;
}

fn smear_y(x: f32, dx: f32, d: f32) -> f32 {
    let zl = clamp((x - dx * 0.5) / d, -1.0, 1.0);
    let zh = clamp((x + dx * 0.5) / d, -1.0, 1.0);
    return d * (smear_f_y(zh) - smear_f_y(zl)) / dx;
}

// Band-limited stripe/row grid, integrated in linear light.
fn grid_sample(p: vec2f, du: vec2f) -> vec3f {
    // Subpixel-space horizontal coordinates: 3 stripes per source pixel.
    let sx = p.x * 3.0;
    let dux = du.x * 3.0;
    let ix = i32(floor(p.x));
    // Rows are centered at integer+0.5; take the two rows bracketing p.y.
    let iy0 = i32(floor(p.y - 0.5));

    var acc = vec3f(0.0);
    for (var ry = 0; ry <= 1; ry++) {
        let jy = iy0 + ry;
        let wy = smear_y((f32(jy) + 0.5) - p.y, du.y, P.row_d);
        if (wy <= 0.0) { continue; }
        for (var rx = -1; rx <= 1; rx++) {
            let jx = ix + rx;
            let texel = fetch(jx, jy);
            // Linearize with the grid model's own response (pure power law,
            // matching the published parameterization).
            let lin = pow(max(texel * P.gain, vec3f(0.0)), vec3f(P.gamma));
            for (var ch = 0; ch < 3; ch++) {
                // Stripe index within the pixel for this channel; the
                // panel's stripe order is BGR (blue leftmost).
                var stripe = f32(ch);
                if (P.bgr > 0.5) { stripe = f32(2 - ch); }
                let center = f32(jx) * 3.0 + stripe + 0.5;
                let wx = smear_x(center - sx, dux, P.stripe_d);
                acc[ch] += lin[ch] * wx * wy;
            }
        }
    }
    return pow(max(acc * P.tint, vec3f(0.0)), vec3f(1.0 / P.gamma));
}

// Both callers supply fetch() and the same named grid parameters.
fn apply_grid(flat: vec3f, p: vec2f, scale: vec2f) -> vec3f {
    let du = 1.0 / scale;
    // The grid needs headroom to resolve: below ~2x it is sub-Nyquist
    // physically and we fade it out rather than render a wrong residue.
    // EPX replaces the pixels themselves, and the grid works on the
    // original texels, so EPX switches the grid off automatically.
    var strength = P.grid_strength * smoothstep(1.5, 2.5, min(scale.x, scale.y));
    if (P.scale_mode > 2.5) {
        strength = 0.0;
    }
    if (strength <= 0.0) {
        return flat;
    } else {
        // The scaling mode also sets how wide a footprint the grid
        // integrates over: off = the full output pixel (exact
        // band-limited, softest, no moire); sharp = half of it (crisper
        // stripes); nearest = a point sample (crispest, may shimmer at
        // fractional scales, like any nearest-neighbour filter).
        var gk = 1.0;
        if (P.scale_mode > 1.5) {
            gk = 0.5;
        } else if (P.scale_mode > 0.5) {
            gk = 0.05;
        }
        return mix(flat, grid_sample(p, du * gk), strength);
    }
}
