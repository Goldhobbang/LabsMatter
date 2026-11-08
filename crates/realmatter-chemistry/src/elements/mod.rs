//! Element update entry points.
//!
//! The renderer calls [`color_for`] and [`stamp_overlay`]. The simulation
//! tick calls [`update`] once per frame after movement.

mod actors;
mod devices;
mod extra;
mod ids;
mod pipe;
mod piston;
mod sim;
mod soap;
mod spark;

pub use ids::*;
pub use sim::{split_trailer, ElemAux, ElementSim, Player};

use realmatter_core::cell::Cell;

/// One element pass. `gravity` is 0 down, 1 off, 2 radial. `wrap` is the
/// world edge mode.
pub fn update(
    cells: &mut [Cell],
    sim: &mut ElementSim,
    w: usize,
    h: usize,
    wrap: bool,
    gravity: u8,
) {
    if w == 0 || h == 0 || cells.len() != w * h {
        return;
    }
    sim.ensure(cells.len());
    sim.tick = sim.tick.wrapping_add(1);
    spark::decay_life(cells, sim);
    spark::update(cells, sim, w, h, wrap);
    spark::battery(cells, sim, w, h, wrap);
    devices::update(cells, sim, w, h, wrap);
    piston::update(cells, sim, w, h);
    pipe::update(cells, sim, w, h, wrap);
    soap::update(cells, sim, w, h, wrap);
    extra::update(cells, sim, w, h, wrap);
    actors::update(cells, sim, w, h, wrap, gravity);
    sim.wireless[0] = sim.wireless[1];
    sim.wireless[1] = [0; CHANNELS];
}

/// Element-specific color. `None` means the caller should use its own palette.
pub fn color_for(material: u16, aux: &ElemAux, temp: f32) -> Option<(u8, u8, u8, u8)> {
    match material {
        SPRK => {
            let (br, bg, bb, _) = ids::base_rgba(if aux.ctype == 0 { METL } else { aux.ctype });
            Some((br / 2 + 128, bg / 4 + 180, bb / 4, 255))
        }
        WIFI => {
            let q = channel_from_temp(temp) as f32;
            let f = 0.0628 * q;
            Some((
                (f.sin() * 127.0 + 128.0) as u8,
                ((f + 2.0).sin() * 127.0 + 128.0) as u8,
                ((f + 4.0).sin() * 127.0 + 128.0) as u8,
                255,
            ))
        }
        SWCH if aux.life >= 10 => Some((17, 217, 24, 255)),
        LCRY => {
            let k = (aux.life.clamp(0, 10) as u8) * 14;
            Some((80 + k, 80 + k / 2, 80 + k / 3, 255))
        }
        PSTN if aux.life > 0 => Some((110, 110, 40, 255)),
        PIPE | PPIP => Some(pipe::pipe_color(material, aux.ctype, aux.tmp, temp)),
        PUMP | GPMP | HSWC | PBCN if aux.life > 0 => {
            let (r, g, b, a) = ids::base_rgba(material);
            let k = (aux.life.min(10) as u16) * 8;
            Some((
                (r as u16 + k).min(255) as u8,
                (g as u16 + k / 2).min(255) as u8,
                b,
                a,
            ))
        }
        EMP if aux.life > 0 => Some((180, 220, 255, 255)),
        METL | IRON | COPR if temp > 800.0 => {
            let (r, g, b, a) = ids::base_rgba(material);
            let k = ((temp - 800.0) / 10.0).clamp(0.0, 80.0) as u8;
            Some((r.saturating_add(k), g, b, a))
        }
        _ => None,
    }
}

/// Draw stickman joints into the RGBA buffer without changing cells.
pub fn stamp_overlay(render: &mut [u8], sim: &ElementSim, w: usize, h: usize) {
    if w == 0 || render.len() < w * h * 4 {
        return;
    }
    for (x, y, r, g, b) in actors::overlays(sim) {
        if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
            continue;
        }
        let i = (y as usize * w + x as usize) * 4;
        if i + 3 >= render.len() {
            continue;
        }
        render[i] = r;
        render[i + 1] = g;
        render[i + 2] = b;
        render[i + 3] = 255;
    }
}
