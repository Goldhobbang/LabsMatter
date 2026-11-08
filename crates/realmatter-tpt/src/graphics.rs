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
use crate::particle::Particle;

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
            r = (r as i32 + (gradv * caddress).sin().mul_add(226.0, 0.0) as i32).clamp(0, 255) as u8;
            g = (g as i32 - (gradv * caddress * 4.55).sin().mul_add(34.0, 0.0) as i32).clamp(0, 255) as u8;
            b = (b as i32 - (gradv * caddress * 2.22).sin().mul_add(64.0, 0.0) as i32).clamp(0, 255) as u8;
        }
    }

    // Element-specific graphics (shared contract; extend here).
    element_graphics(t, p, &mut out);

    // Per-particle decoration (ARGB) blended onto the base colour.
    if p.dcolour != 0 {
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
/// Only the high-impact emissive elements are specialised here. Everything else
/// uses the flat element colour (plus hot-glow/decoration, applied above). This
/// is the extension point for Agent 3's remaining `graphics()` callbacks.
fn element_graphics(t: u16, p: &Particle, out: &mut GfxPixel) {
    use crate::elements_generated::pt;
    match t {
        // Fire / plasma / lava: additive flame, colour varies with life.
        pt::FIRE | pt::PLSM => {
            let k = (p.life.max(0) as f32 / 120.0).clamp(0.0, 1.0);
            out.mode = pmode::FLAT | pmode::GLOW | firemode::ADD;
            out.fr = 255;
            out.fg = (120.0 + 135.0 * k) as u8;
            out.fb = (20.0 + 60.0 * (1.0 - k)) as u8;
            out.fa = 220;
        }
        pt::LAVA => {
            out.mode = pmode::FLAT | pmode::GLOW | firemode::ADD;
            out.fr = 255;
            out.fg = 110;
            out.fb = 20;
            out.fa = 110;
        }
        // Sparks / electricity: additive spark.
        pt::SPRK => {
            out.mode = pmode::FLAT | pmode::ADD | pmode::SPARK | firemode::SPARK;
            out.fr = 255;
            out.fg = 255;
            out.fb = 120;
            out.fa = 255;
        }
        pt::ELEC => {
            out.mode = pmode::FLAT | pmode::ADD | pmode::SPARK | firemode::SPARK;
            out.fr = 180;
            out.fg = 230;
            out.fb = 255;
            out.fa = 230;
        }
        // Photons / neutrons: energy particles, additive.
        pt::PHOT => {
            out.mode = pmode::FLAT | pmode::ADD;
            out.fr = 120;
            out.fg = 255;
            out.fb = 255;
            out.fa = 220;
        }
        pt::NEUT => {
            out.mode = pmode::FLAT | pmode::ADD;
            out.fr = 160;
            out.fg = 255;
            out.fb = 160;
            out.fa = 220;
        }
        // Glow / rays: soft glow.
        pt::GLOW => {
            out.mode = pmode::FLAT | pmode::GLOW | firemode::ADD;
            out.fr = out.r;
            out.fg = out.g;
            out.fb = out.b;
            out.fa = 200;
        }
        pt::BRAY | pt::ARAY => {
            out.mode = pmode::FLAT | pmode::ADD | firemode::ADD;
            out.fr = out.r;
            out.fg = out.g;
            out.fb = out.b;
            out.fa = 200;
        }
        pt::TRON => {
            // Hue stored in tmp bits 7..16 (reference TRON graphics).
            let hue = ((p.tmp >> 7) & 0x1FF) as f32 / 511.0 * 6.283;
            out.r = (hue.sin().mul_add(127.0, 128.0) as i32).clamp(0, 255) as u8;
            out.g = ((hue + 2.094).sin().mul_add(127.0, 128.0) as i32).clamp(0, 255) as u8;
            out.b = ((hue + 4.188).sin().mul_add(127.0, 128.0) as i32).clamp(0, 255) as u8;
        }
        // PIPE / PPIP: emulate the stored particle's colour, otherwise show the
        // reference colour state (red/green/blue) or the base pipe colour.
        pt::PIPE | pt::PPIP => {
            use crate::elements_generated::PT_NUM;
            let ct = p.ctype;
            if ct > 0 && (ct as usize) < PT_NUM && element(ct as u16).enabled {
                let (r, g, b) = rgb(element(ct as u16).colour);
                out.r = r;
                out.g = g;
                out.b = b;
                out.mode = pmode::FLAT;
            } else {
                match p.tmp & 0x000C_0000 {
                    0x0004_0000 => {
                        out.r = 50;
                        out.g = 1;
                        out.b = 1;
                    }
                    0x0008_0000 => {
                        out.r = 1;
                        out.g = 50;
                        out.b = 1;
                    }
                    0x000C_0000 => {
                        out.r = 1;
                        out.g = 1;
                        out.b = 50;
                    }
                    _ => {
                        out.r = 68;
                        out.g = 68;
                        out.b = 68;
                    }
                }
                if t == pt::PPIP {
                    out.r = out.r.saturating_add(20);
                    out.b = out.b.saturating_add(34);
                }
                if p.tmp & 0x0002_0000 != 0 {
                    out.a = 128; // initializing
                }
            }
            if t == pt::PPIP && p.tmp & 0x0200_0000 != 0 {
                out.mode = pmode::FLAT | pmode::BLEND;
                out.a = 150; // paused
            }
        }
        // LCRY: brightness tracks the charge level (reference `tmp2`).
        pt::LCRY => {
            let q = p.tmp2.clamp(0, 10) as u8;
            let v = 0x50u16.saturating_add((q as u16) * 10) as u8;
            out.r = v;
            out.g = v;
            out.b = v;
        }
        // DEUT: life tints toward blue and glows at high compression.
        pt::DEUT => {
            let l = p.life.clamp(0, 255) as u8;
            out.r = out.r.saturating_add(l);
            out.g = out.g.saturating_add(l / 2);
            out.b = out.b.saturating_add(l);
            if p.life >= 240 {
                out.mode = pmode::FLAT | pmode::GLOW | firemode::ADD;
                out.fr = 255;
                out.fg = 255;
                out.fb = 255;
                out.fa = 60;
            }
        }
        // BIZR: wavelength bitmask decoded to RGB (reference graphics).
        pt::BIZR | pt::BIZRG | pt::BIZRS => {
            let wl = p.ctype & 0x3FFF_FFFF;
            if wl != 0 {
                let (mut r, mut g, mut b) = (0i32, 0i32, 0i32);
                for x in 0..12 {
                    r += (wl >> (x + 18)) & 1;
                    b += (wl >> x) & 1;
                }
                for x in 0..12 {
                    g += (wl >> (x + 9)) & 1;
                }
                let scale = 624 / (r + g + b + 1);
                out.r = (r * scale).clamp(0, 255) as u8;
                out.g = (g * scale).clamp(0, 255) as u8;
                out.b = (b * scale).clamp(0, 255) as u8;
            }
        }
        // VIBR/BVBR: energy gradient (reference graphics).
        pt::VIBR | pt::BVBR => {
            let gradient = (p.tmp / 10).clamp(0, 100) as u8;
            out.r = out.r.saturating_add(gradient.saturating_mul(2));
            out.g = out.g.saturating_add(gradient.saturating_mul(2).min(175));
            out.b = out.b.saturating_add(gradient.saturating_mul(2));
            if p.life != 0 || p.tmp >= 1000 {
                out.mode = pmode::FLAT | pmode::GLOW | firemode::ADD;
                out.fr = 255;
                out.fg = 160;
                out.fb = 255;
                out.fa = 90;
            }
        }
        // SLCN: sparkle colour table (reference `SLCN_COLOUR[16]`).
        pt::SLCN => {
            const SLCN: [(u8, u8, u8); 16] = [
                (0x5A, 0x66, 0x79),
                (0x68, 0x78, 0xA1),
                (0xAB, 0xBF, 0xDD),
                (0x83, 0x84, 0x90),
                (0xBC, 0xCD, 0xDF),
                (0x82, 0xA0, 0xD2),
                (0x5B, 0x66, 0x80),
                (0x23, 0x2C, 0x3B),
                (0x48, 0x50, 0x67),
                (0x8B, 0x9A, 0xB6),
                (0xAD, 0xB1, 0xC1),
                (0xC3, 0xC6, 0xD1),
                (0x85, 0x94, 0xAD),
                (0x26, 0x2F, 0x47),
                (0xA9, 0xAE, 0xBC),
                (0xC2, 0xE1, 0xF7),
            ];
            let idx = ((p.tmp >> 12) & 15) as usize;
            let (r, g, b) = SLCN[idx];
            out.r = r;
            out.g = g;
            out.b = b;
        }
        _ => {}
    }
}
