//! Shared particle graphics contract.
//!
//! Mirrors the reference `ElementGraphics.h` model: an element's graphics
//! callback returns a draw mode plus a base colour and a separate fire/glow
//! colour. The renderer combines them (flat, additive, glow, bloom).
//!
//! This module owns the *infrastructure* (mode flags, default colour, hot-glow
//! tint, decoration blending). Element-specific rules live in
//! [`element_graphics`] and are intentionally small; Agent 3 can extend the
//! match with the remaining reference `graphics()` behaviours without touching
//! the renderer.

use crate::element::element;
use crate::elements_generated::props;
use crate::particle::{self, Particle};

/// Draw mode flags (`PMODE_*`).
pub mod pmode {
    pub const NONE: u32 = 0;
    pub const FLAT: u32 = 0x0000_0001;
    pub const BLOB: u32 = 0x0000_0002;
    pub const BLUR: u32 = 0x0000_0004;
    pub const GLOW: u32 = 0x0000_0008;
    pub const SPARK: u32 = 0x0000_0010;
    pub const FLARE: u32 = 0x0000_0020;
    pub const LFLARE: u32 = 0x0000_0040;
    pub const ADD: u32 = 0x0000_0080;
    pub const BLEND: u32 = 0x0000_0100;
    /// Disables per-particle decoration blending (`NO_DECO`).
    pub const NO_DECO: u32 = 0x0000_1000;
    /// Draw fire colour as decoration (`DECO_FIRE`).
    pub const DECO_FIRE: u32 = 0x0000_2000;
    /// Mask covering all `PMODE_*` draw-style bits (reference `PMODE`).
    pub const PMODE_MASK: u32 = 0x0000_0FFF;
}

/// Fire/glow mode flags (`FIRE_*`).
pub mod firemode {
    pub const ADD: u32 = 0x0001_0000;
    pub const BLEND: u32 = 0x0002_0000;
    pub const SPARK: u32 = 0x0004_0000;
}

/// One rendered pixel contribution for a particle.
#[derive(Clone, Copy, Debug, Default)]
pub struct GfxPixel {
    pub mode: u32,
    /// Base colour.
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
    /// Additive fire/glow colour (used when `mode` has GLOW/ADD/SPARK).
    pub fr: u8,
    pub fg: u8,
    pub fb: u8,
    pub fa: u8,
}

impl GfxPixel {
    #[inline]
    pub fn flat(r: u8, g: u8, b: u8) -> Self {
        Self {
            mode: pmode::FLAT,
            r,
            g,
            b,
            a: 255,
            ..Default::default()
        }
    }

    #[inline]
    pub fn has_glow(&self) -> bool {
        self.fa != 0
            && (self.mode & (pmode::GLOW | pmode::ADD | pmode::SPARK | firemode::ADD | firemode::SPARK)) != 0
    }
}

#[inline]
fn rgb(c: u32) -> (u8, u8, u8) {
    (((c >> 16) & 0xFF) as u8, ((c >> 8) & 0xFF) as u8, (c & 0xFF) as u8)
}

#[inline]
fn clamp_u8(v: i32) -> u8 {
    v.clamp(0, 255) as u8
}

#[inline]
fn clamp_u8f(v: f32) -> u8 {
    if v <= 0.0 { 0 } else if v >= 255.0 { 255 } else { v as u8 }
}

#[inline]
fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round().clamp(0.0, 255.0) as u8
}

#[inline]
fn gradient_sample(stops: &[(u8, u8, u8, f32)], t: f32) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    if t <= stops[0].3 { return (stops[0].0, stops[0].1, stops[0].2); }
    let last = stops.len() - 1;
    if t >= stops[last].3 { return (stops[last].0, stops[last].1, stops[last].2); }
    let mut i = 0;
    while i + 1 < stops.len() && t > stops[i + 1].3 { i += 1; }
    let a = stops[i];
    let b = stops[i + 1];
    let lt = if b.3 <= a.3 { 0.0 } else { (t - a.3) / (b.3 - a.3) };
    (lerp_u8(a.0, b.0, lt), lerp_u8(a.1, b.1, lt), lerp_u8(a.2, b.2, lt))
}

fn flame_table_at(life: i32) -> (u8, u8, u8) {
    // ref: Renderer.cpp:1572 flameTable 200 steps
    const STOPS: [(u8, u8, u8, f32); 4] = [
        (0x00, 0x00, 0x00, 0.00),
        (0x60, 0x30, 0x0F, 0.50),
        (0xDF, 0xBF, 0x6F, 0.90),
        (0xAF, 0x9F, 0x0F, 1.00),
    ];
    let idx = (life - 1).clamp(0, 199) as f32 / 199.0;
    gradient_sample(&STOPS, idx)
}

fn plasma_table_at(life: i32) -> (u8, u8, u8) {
    // ref: Renderer.cpp:1578 plasmaTable 200 steps
    const STOPS: [(u8, u8, u8, f32); 5] = [
        (0x00, 0x00, 0x00, 0.00),
        (0x30, 0x10, 0x40, 0.25),
        (0x30, 0x10, 0x60, 0.50),
        (0xAF, 0xFF, 0xFF, 0.90),
        (0xAF, 0xFF, 0xFF, 1.00),
    ];
    let idx = (life - 1).clamp(0, 199) as f32 / 199.0;
    gradient_sample(&STOPS, idx)
}

fn clfm_table_at(life: i32) -> (u8, u8, u8) {
    // ref: Renderer.cpp clfmTable 200 steps
    const STOPS: [(u8, u8, u8, f32); 8] = [
        (0x00, 0x00, 0x00, 0.00),
        (0x0A, 0x09, 0x17, 0.10),
        (0x19, 0x16, 0x3C, 0.20),
        (0x28, 0x28, 0x5E, 0.30),
        (0x34, 0x3E, 0x77, 0.40),
        (0x49, 0x76, 0x9A, 0.60),
        (0x57, 0xA0, 0xB4, 0.80),
        (0x5E, 0xC4, 0xC6, 1.00),
    ];
    let idx = (life - 1).clamp(0, 199) as f32 / 199.0;
    gradient_sample(&STOPS, idx)
}

fn wavelength_to_rgb(wl: i32) -> (u8, u8, u8) {
    let (mut r, mut g, mut b) = (0i32, 0i32, 0i32);
    for x in 0..12 {
        r += (wl >> (x + 18)) & 1;
        b += (wl >> x) & 1;
    }
    for x in 0..12 {
        g += (wl >> (x + 9)) & 1;
    }
    let scale = 624 / (r + g + b + 1);
    (clamp_u8(r * scale), clamp_u8(g * scale), clamp_u8(b * scale))
}

fn filt_wavelength(p: &Particle) -> i32 {
    if p.ctype & 0x3FFF_FFFF != 0 {
        p.ctype & 0x3FFF_FFFF
    } else {
        let temp_bin = ((p.temp - 273.0) * 0.025) as i32;
        let temp_bin = temp_bin.clamp(0, 25);
        0x1F_i32 << temp_bin
    }
}

/// Compute the reference-style pixel for particle `p` of type `t`.
pub fn for_particle(t: u16, p: &Particle) -> GfxPixel {
    let def = element(t);
    let (mut r, mut g, mut b) = rgb(def.colour);
    let mut out = GfxPixel::flat(r, g, b);

    // PROP_HOT_GLOW: tint toward white/yellow near the element's high
    // temperature (reference `Renderer.cpp`).
    if (def.properties & props::PROP_HOT_GLOW) != 0 {
        let high = def.high_temperature;
        if high.is_finite() && p.temp > high - 800.0 {
            let gradv = std::f32::consts::PI / (2.0 * high - (high - 800.0));
            let caddress = if p.temp > high {
                high - (high - 800.0)
            } else {
                p.temp - (high - 800.0)
            };
            r = clamp_u8(r as i32 + (gradv * caddress).sin().mul_add(226.0, 0.0) as i32);
            g = clamp_u8(g as i32 - (gradv * caddress * 4.55).sin().mul_add(34.0, 0.0) as i32);
            b = clamp_u8(b as i32 - (gradv * caddress * 2.22).sin().mul_add(64.0, 0.0) as i32);
        }
    }

    // Element-specific graphics (shared contract; extend here).
    element_graphics(t, p, &mut out);

    // Per-particle decoration (ARGB) blended onto the base colour.
    if p.dcolour != 0 && (out.mode & pmode::NO_DECO) == 0 {
        let da = (p.dcolour >> 24) & 0xFF;
        if da != 0 {
            let (dr, dg, db) = rgb(p.dcolour & 0x00FF_FFFF);
            r = ((da as u32 * dr as u32 + (256 - da) as u32 * out.r as u32) >> 8) as u8;
            g = ((da as u32 * dg as u32 + (256 - da) as u32 * out.g as u32) >> 8) as u8;
            b = ((da as u32 * db as u32 + (256 - da) as u32 * out.b as u32) >> 8) as u8;
        }
    }
    out.r = r;
    out.g = g;
    out.b = b;
    out
}

/// Element-specific graphics rules.
///
/// Element-specific rules translated from reference `static int graphics(...)`
/// callbacks where feasible. Many callbacks remain TODO because they require
/// renderer/simulation context not exposed to this contract.
fn element_graphics(t: u16, p: &Particle, out: &mut GfxPixel) {
    use crate::elements_generated::pt;
    match t {
        // ref: FIRE.cpp:405
        pt::FIRE => {
            let (cr, cg, cb) = flame_table_at(p.life);
            out.r = cr; out.g = cg; out.b = cb;
            out.fa = 255; out.fr = cr; out.fg = cg; out.fb = cb;
            out.mode = pmode::NONE | firemode::ADD;
        }
        // ref: PLSM.cpp:54
        pt::PLSM => {
            let (cr, cg, cb) = plasma_table_at(p.life);
            out.r = cr; out.g = cg; out.b = cb;
            out.fa = 255; out.fr = cr; out.fg = cg; out.fb = cb;
            out.mode = pmode::GLOW | pmode::ADD | firemode::ADD;
        }
        // ref: LAVA.cpp:55
        pt::LAVA => {
            let life = p.life;
            out.r = clamp_u8(life * 2 + 0xE0);
            out.g = clamp_u8((life * 1 + 0x50).min(192));
            out.b = clamp_u8((life / 2 + 0x10).min(128));
            out.fa = 40; out.fr = out.r; out.fg = out.g; out.fb = out.b;
            out.mode |= firemode::ADD | pmode::BLUR;
        }
        // ref: SPRK.cpp:413
        pt::SPRK => {
            out.fa = 60;
            out.fr = out.r / 2;
            out.fg = out.g / 2;
            out.fb = out.b / 2;
            out.mode |= firemode::SPARK;
        }
        // ref: ELEC.cpp:148
        pt::ELEC => {
            out.fa = 70;
            out.fr = out.r; out.fg = out.g; out.fb = out.b;
            out.mode |= firemode::ADD;
        }
        // ref: PHOT.cpp:149
        pt::PHOT => {
            let (r, g, b) = wavelength_to_rgb(p.ctype);
            out.r = r; out.g = g; out.b = b;
            out.fa = 100; out.fr = r; out.fg = g; out.fb = b;
            out.mode &= !pmode::FLAT;
            out.mode |= firemode::ADD | pmode::ADD | pmode::NO_DECO;
            if (p.flags & particle::flags::MOVABLE) != 0 {
                out.mode &= !pmode::NO_DECO;
            }
        }
        // ref: NEUT.cpp:251
        pt::NEUT => {
            out.fa = 120;
            out.fr = 10; out.fg = 80; out.fb = 120;
            out.mode |= firemode::ADD;
        }
        // TODO(GLOW.cpp:93): needs gfctx.rng for random threshold.
        pt::GLOW => {}
        // ref: BRAY.cpp:50
        pt::BRAY => {
            let mut trans: u8 = 255;
            if p.tmp == 0 {
                trans = (p.life.saturating_mul(7)).clamp(0, 255) as u8;
                let wl = p.ctype & 0x3FFF_FFFF;
                if wl != 0 {
                    let (r, g, b) = wavelength_to_rgb(wl);
                    out.r = r; out.g = g; out.b = b;
                }
            } else if p.tmp == 1 {
                trans = (p.life / 4).clamp(0, 255) as u8;
                let wl = p.ctype & 0x3FFF_FFFF;
                if wl != 0 {
                    let (r, g, b) = wavelength_to_rgb(wl);
                    out.r = r; out.g = g; out.b = b;
                }
            } else if p.tmp == 2 {
                trans = (p.life.saturating_mul(100)).clamp(0, 255) as u8;
                out.r = 255; out.g = 150; out.b = 50;
            }
            out.a = trans;
            out.mode &= !pmode::PMODE_MASK;
            out.mode |= pmode::BLEND | pmode::GLOW;
        }
        // TODO(ARAY.cpp): no reference graphics callback.
        pt::ARAY => {}
        // ref: FILT.cpp:51
        pt::FILT => {
            let wl = filt_wavelength(p);
            let (r, g, b) = wavelength_to_rgb(wl);
            out.r = r; out.g = g; out.b = b;
            out.a = if p.life > 0 && p.life <= 4 { (127 + p.life * 30).clamp(0, 255) as u8 } else { 127 };
            out.mode &= !pmode::PMODE_MASK;
            out.mode |= pmode::BLEND;
        }
        // ref: TRON.cpp:169 (colour/index mapping differs from reference hue table; TODO for exact tron_colours table)
        pt::TRON => {}
        // PIPE / PPIP: emulate stored particle graphics (reference PIPE.cpp:~400, props_pipe_to_part/Element_PIPE_graphics).
        // Keeps existing behaviour and works for stored/portal-like embedded payload rendering (see graphics.rs:285 note in report).
        pt::PIPE | pt::PPIP => {
            use crate::elements_generated::PT_NUM;
            let ct = p.ctype;
            let emulatable = ct > 0
                && (ct as usize) < PT_NUM
                && element(ct as u16).enabled
                && !matches!(ct as u16, pt::STKM | pt::STKM2 | pt::FIGH | pt::PIPE | pt::PPIP);
            if emulatable {
                let mut stored = Particle::new(ct as u16, p.x, p.y);
                stored.temp = p.temp;
                stored.life = p.tmp2;
                stored.tmp = p.tmp3;
                stored.ctype = p.tmp4;
                *out = for_particle(ct as u16, &stored);
            } else {
                match p.tmp & 0x000C_0000 {
                    0x0004_0000 => { out.r = 50; out.g = 1; out.b = 1; }
                    0x0008_0000 => { out.r = 1; out.g = 50; out.b = 1; }
                    0x000C_0000 => { out.r = 1; out.g = 1; out.b = 50; }
                    _ => {
                        if t == pt::PPIP { out.r = 68; out.g = 68; out.b = 102; }
                        else { out.r = 68; out.g = 68; out.b = 68; }
                    }
                }
                if p.tmp & 0x0002_0000 != 0 {
                    out.a = 128;
                }
            }
            if t == pt::PPIP && p.tmp & 0x0200_0000 != 0 {
                out.mode |= pmode::FLAT | pmode::BLEND;
                if out.a > 150 || out.a == 0 || out.a == 255 {
                    out.a = 150;
                }
            }
        }
        // ref: LCRY.cpp:107 (deco-aware branch needs gfctx.ren decorationLevel; base brightness path translated)
        pt::LCRY => {
            let q = p.tmp2.min(10).max(0);
            let v = clamp_u8(0x50 + q * 10);
            out.r = v; out.g = v; out.b = v;
            out.mode |= pmode::NO_DECO;
        }
        // ref: DEUT.cpp:148
        pt::DEUT => {
            if p.life >= 240 {
                out.fa = 60;
                out.r = clamp_u8(out.r as i32 + 255);
                out.g = clamp_u8(out.g as i32 + 255);
                out.b = clamp_u8(out.b as i32 + 255);
                out.fr = out.r; out.fg = out.g; out.fb = out.b;
                out.mode |= pmode::GLOW | firemode::ADD;
            } else if p.life > 0 {
                out.r = clamp_u8(out.r as i32 + p.life);
                out.g = clamp_u8(out.g as i32 + p.life * 2);
                out.b = clamp_u8(out.b as i32 + p.life * 3);
                out.mode |= pmode::BLUR;
            } else {
                out.mode |= pmode::BLUR;
            }
        }
        // ref: BIZR.cpp:Element_BIZR_graphics (shared by BIZR/BIZRG/BIZRS)
        pt::BIZR | pt::BIZRG | pt::BIZRS => {
            let wl = p.ctype & 0x3FFF_FFFF;
            if wl != 0 {
                let (r, g, b) = wavelength_to_rgb(wl);
                out.r = r; out.g = g; out.b = b;
            }
            let brightness = p.vx.abs() + p.vy.abs();
            if brightness > 0.0 {
                let b = brightness / 5.0;
                out.fa = 255;
                out.fr = clamp_u8f(out.r as f32 * b);
                out.fg = clamp_u8f(out.g as f32 * b);
                out.fb = clamp_u8f(out.b as f32 * b);
                out.mode |= firemode::ADD;
            }
            out.mode |= pmode::BLUR;
        }
        // ref: VIBR.cpp:Element_VIBR_graphics (shared by VIBR/BVBR)
        pt::VIBR | pt::BVBR => {
            let gradient = p.tmp / 10;
            if gradient >= 100 || p.life != 0 {
                let v = ((750.0f32 - p.life as f32) / 170.0).exp().sin().abs() * 200.0;
                let c = clamp_u8f(v);
                out.r = c;
                if p.tmp2 != 0 { out.g = c; out.b = 255; } else { out.g = 255; out.b = c; }
                out.fa = 90; out.fr = out.r; out.fg = out.g; out.fb = out.b;
                out.mode = pmode::NONE;
                out.mode |= firemode::BLEND;
            } else {
                let g = (gradient as f32).clamp(0.0, 100.0);
                out.r = clamp_u8(out.r as i32 + (g * 2.0).clamp(0.0, 255.0) as i32);
                out.g = clamp_u8(out.g as i32 + (g * 2.0).clamp(0.0, 175.0) as i32);
                out.b = clamp_u8(out.b as i32 + (g * 2.0).clamp(0.0, 255.0) as i32);
                out.fa = clamp_u8f((g * 0.6).clamp(0.0, 60.0));
                out.fr = out.r / 2; out.fg = out.g / 2; out.fb = out.b / 2;
                out.mode |= firemode::BLEND;
            }
        }
        // ref: SLCN.cpp:109
        pt::SLCN => {
            const SLCN: [(u8, u8, u8); 16] = [
                (0x5A, 0x66, 0x79), (0x68, 0x78, 0xA1), (0xAB, 0xBF, 0xDD), (0x83, 0x84, 0x90),
                (0xBC, 0xCD, 0xDF), (0x82, 0xA0, 0xD2), (0x5B, 0x66, 0x80), (0x23, 0x2C, 0x3B),
                (0x48, 0x50, 0x67), (0x8B, 0x9A, 0xB6), (0xAD, 0xB1, 0xC1), (0xC3, 0xC6, 0xD1),
                (0x85, 0x94, 0xAD), (0x26, 0x2F, 0x47), (0xA9, 0xAE, 0xBC), (0xC2, 0xE1, 0xF7),
            ];
            let idx = ((p.tmp >> 12) & 15) as usize;
            let curr = SLCN[idx];
            let (r, g, b) = if (p.tmp & 0x800) != 0 {
                let next = SLCN[((p.tmp >> 16) & 15) as usize];
                (((curr.0 as u16 + next.0 as u16) / 2) as u8,
                 ((curr.1 as u16 + next.1 as u16) / 2) as u8,
                 ((curr.2 as u16 + next.2 as u16) / 2) as u8)
            } else { curr };
            out.r = r; out.g = g; out.b = b;
            // rnd sparkle (p.tmp fields deterministic) ref: SLCN.cpp:125
            let rnd = ((p.tmp & 0xFFFF) as u32).wrapping_mul(((p.tmp >> 16) & 0xFFFF) as u32);
            if rnd % 887 == 0 {
                out.mode |= pmode::FLARE | pmode::GLOW;
            }
        }
        // ref: CFLM.cpp:51
        pt::CFLM => {
            let (cr, cg, cb) = clfm_table_at(p.life / 2);
            out.r = cr; out.g = cg; out.b = cb;
            out.fa = 255; out.fr = cr; out.fg = cg; out.fb = cb;
            out.mode = pmode::NONE | firemode::ADD;
        }
        // ref: THDR.cpp:95
        pt::THDR => {
            out.fa = 160; out.fr = 144; out.fg = 192; out.fb = 255;
            out.mode |= firemode::ADD;
        }
        // ref: LIGH.cpp:302
        pt::LIGH => {
            out.fa = 120;
            out.r = 235; out.g = 245; out.b = 255;
            out.fr = out.r; out.fg = out.g; out.fb = out.b;
            out.mode |= pmode::GLOW | firemode::ADD | pmode::DECO_FIRE;
        }
        // ref: EMBR.cpp:77
        pt::EMBR => {
            if (p.ctype & 0x00FF_FFFF) != 0 {
                out.r = ((p.ctype >> 16) & 0xFF) as u8;
                out.g = ((p.ctype >> 8) & 0xFF) as u8;
                out.b = (p.ctype & 0xFF) as u8;
                let maxc = out.r.max(out.g).max(out.b).max(1) as f32;
                if maxc < 60.0 {
                    let m = 60.0 / maxc;
                    out.r = clamp_u8f(out.r as f32 * m);
                    out.g = clamp_u8f(out.g as f32 * m);
                    out.b = clamp_u8f(out.b as f32 * m);
                }
            } else if p.tmp != 0 {
                out.r = 255; out.g = 255; out.b = 255;
            }
            out.fr = out.r; out.fg = out.g; out.fb = out.b;
            if p.tmp == 1 {
                out.mode = firemode::ADD | pmode::BLEND | pmode::GLOW;
                out.fa = ((p.life - 15) * 4).clamp(0, 255) as u8;
                out.a = ((p.life + 15) * 4).clamp(0, 255) as u8;
            } else if p.tmp == 2 {
                out.mode = pmode::FLAT | firemode::ADD;
                out.fa = 255;
            } else {
                out.mode = pmode::SPARK | pmode::ADD;
                if p.life < 64 { out.a = (4 * p.life).clamp(0, 255) as u8; }
            }
        }
        // ref: SMKE.cpp:49
        pt::SMKE => {
            out.r = 55; out.g = 55; out.b = 55;
            out.fa = 75; out.fr = 55; out.fg = 55; out.fb = 55;
            out.mode = pmode::NONE | firemode::BLEND;
        }
        // ref: SWCH.cpp:102
        pt::SWCH => {
            if p.life >= 10 {
                out.r = 17; out.g = 217; out.b = 24;
                out.mode |= pmode::GLOW;
            }
        }
        // ref: PUMP.cpp:113
        pt::PUMP => {
            let lifemod = (p.life.min(10).max(0) * 19) as i32;
            out.b = clamp_u8(out.b as i32 + lifemod);
        }
        // ref: HSWC.cpp:94
        pt::HSWC => {
            let lifemod = (p.life.min(10).max(0) * 19) as i32;
            out.r = clamp_u8(out.r as i32 + lifemod);
        }
        // ref: PVOD.cpp:87
        pt::PVOD => {
            let lifemod = (p.life.min(10).max(0) * 16) as i32;
            out.r = clamp_u8(out.r as i32 + lifemod);
        }
        // ref: SPNG.cpp:216
        pt::SPNG => {
            let d = p.life * 15;
            out.r = clamp_u8(out.r as i32 - d).max(50);
            out.g = clamp_u8(out.g as i32 - d).max(50);
            out.b = clamp_u8(out.b as i32 - d).max(20);
        }
        // ref: CRMC.cpp:62
        pt::CRMC => {
            let z = (p.tmp2 - 2) * 8;
            out.r = clamp_u8(out.r as i32 + z);
            out.g = clamp_u8(out.g as i32 + z);
            out.b = clamp_u8(out.b as i32 + z);
        }
        // ref: PTNM.cpp:267
        pt::PTNM => {
            if p.tmp != 0 { out.mode |= pmode::FLARE; }
        }
        // ref: INVIS.cpp:65
        pt::INVIS => {
            if p.tmp2 != 0 {
                out.a = 100; out.r = 15; out.g = 0; out.b = 150;
                out.mode = pmode::BLEND;
            }
        }
        // ref: WIFI.cpp:88
        pt::WIFI => {
            let q = ((p.temp - 73.15) / 100.0 + 1.0) as f32;
            let freq = 0.0628f32;
            out.r = clamp_u8f((freq * q).sin() * 127.0 + 128.0);
            out.g = clamp_u8f((freq * q + 2.0).sin() * 127.0 + 128.0);
            out.b = clamp_u8f((freq * q + 4.0).sin() * 127.0 + 128.0);
        }
        // ref: PRTI.cpp:152 (EFFECT flags not represented in this contract)
        pt::PRTI => {
            out.fa = 8; out.fr = 255; out.fg = 0; out.fb = 0;
            out.mode &= !pmode::PMODE_MASK;
            out.mode |= pmode::ADD;
        }
        // ref: PRTO.cpp:181 (EFFECT flags not represented in this contract)
        pt::PRTO => {
            out.fa = 8; out.fr = 0; out.fg = 0; out.fb = 255;
            out.mode &= !pmode::PMODE_MASK;
            out.mode |= pmode::ADD;
        }
        // ref: LITH.cpp:254
        pt::LITH => {
            if p.life >= 1000 {
                out.r = 0xFF; out.g = 0xA0; out.b = 0x40;
                out.mode |= pmode::FLARE | pmode::GLOW;
            } else if p.ctype > 0 {
                // Deterministic substitute for gfctx.rng.between(ctype/3, ctype)
                let v = p.ctype / 2;
                let mut mult = v / 15;
                if mult > 6 { mult = 6; }
                out.r = clamp_u8(out.r as i32 - 30 * mult);
                out.b = clamp_u8(out.b as i32 + 20 * mult);
                out.mode |= pmode::FLARE | pmode::GLOW;
            }
        }
        // ref: FIRW.cpp:121
        pt::FIRW => {
            if p.tmp > 0 { out.mode |= pmode::GLOW; }
        }
        // TODO(GRAV.cpp:62): needs gfctx.sim->currentTick.
        // TODO(PIPE.cpp:~400): detailed parity w/ graphicscache/pixel_mode; base emulation kept above.
        // TODO(PIPE emulation): stored particle deco transfer (PFLAG_PARTICLE_DECO) not represented in Particle here.
        // TODO(TRON.cpp:169): needs exact tron_colours table/index mapping.
        // TODO(ARAY.cpp): no graphics callback in reference (default).
        _ => {}
    }
}



