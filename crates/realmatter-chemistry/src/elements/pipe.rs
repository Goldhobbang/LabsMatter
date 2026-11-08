//! Pipes.
//!
//! `ctype` is the stored particle. `tmp` bit 0 pauses a PPIP. Bits 8–9 are a
//! phase so a line of pipe prefers to hand its cargo to the next phase instead
//! of shuffling randomly forever. Graphics blend the stored material's color
//! into the pipe, and a powered PPIP is brighter than a paused one.

use realmatter_core::cell::{cell_flags, Cell};

use super::ids::*;
use super::sim::{at, ElementSim};

const PAUSED: i32 = 1;

pub fn update(cells: &mut [Cell], sim: &mut ElementSim, w: usize, h: usize, wrap: bool) {
    let n = cells.len().min(sim.aux.len());
    for i in 0..n {
        if !matches!(cells[i].material, PIPE | PPIP) {
            continue;
        }
        if sim.aux[i].tmp & 0x300 == 0 {
            let phase = ((i % w) + (i / w)) & 3;
            sim.aux[i].tmp |= (phase as i32) << 8;
        }
        if cells[i].material == PPIP && sim.aux[i].tmp & PAUSED != 0 {
            continue;
        }
        let phase = ((sim.aux[i].tmp >> 8) & 3) as i32;
        let x = (i % w) as i32;
        let y = (i / w) as i32;
        if sim.aux[i].ctype != 0 {
            let want = (phase + 1) & 3;
            if !push_to_phase(cells, sim, i, x, y, w, h, wrap, want) {
                eject(cells, sim, i, x, y, w, h, wrap);
            }
        } else {
            ingest(cells, sim, i, x, y, w, h, wrap);
        }
    }
}

fn push_to_phase(
    cells: &mut [Cell],
    sim: &mut ElementSim,
    i: usize,
    x: i32,
    y: i32,
    w: usize,
    h: usize,
    wrap: bool,
    want: i32,
) -> bool {
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        let Some((ni, _, _)) = at(w, h, x, y, dx, dy, wrap) else {
            continue;
        };
        if !matches!(cells[ni].material, PIPE | PPIP) {
            continue;
        }
        if cells[ni].material == PPIP && sim.aux[ni].tmp & PAUSED != 0 {
            continue;
        }
        let phase = (sim.aux[ni].tmp >> 8) & 3;
        if phase == want && sim.aux[ni].ctype == 0 {
            sim.aux[ni].ctype = sim.aux[i].ctype;
            sim.aux[ni].tmp2 = cells[i].temperature as i32;
            sim.aux[i].ctype = 0;
            return true;
        }
    }
    false
}

fn eject(
    cells: &mut [Cell],
    sim: &mut ElementSim,
    i: usize,
    x: i32,
    y: i32,
    w: usize,
    h: usize,
    wrap: bool,
) {
    let cargo = sim.aux[i].ctype;
    if cargo == 0 || cargo == PIPE || cargo == PPIP {
        sim.aux[i].ctype = 0;
        return;
    }
    for (dx, dy) in [(0, 1), (1, 0), (0, -1), (-1, 0)] {
        let Some((ni, _, _)) = at(w, h, x, y, dx, dy, wrap) else {
            continue;
        };
        if cells[ni].is_empty() {
            cells[ni].material = cargo;
            cells[ni].temperature = if sim.aux[i].tmp2 > 0 {
                sim.aux[i].tmp2 as f32
            } else {
                293.15
            };
            cells[ni].set_flag(cell_flags::FIXED, is_solid_obstacle(cargo));
            sim.aux[ni] = Default::default();
            sim.aux[i].ctype = 0;
            sim.aux[i].tmp2 = 0;
            return;
        }
    }
}

fn ingest(
    cells: &mut [Cell],
    sim: &mut ElementSim,
    i: usize,
    x: i32,
    y: i32,
    w: usize,
    h: usize,
    wrap: bool,
) {
    for (dx, dy) in [(0, 1), (1, 0), (0, -1), (-1, 0)] {
        let Some((ni, _, _)) = at(w, h, x, y, dx, dy, wrap) else {
            continue;
        };
        let id = cells[ni].material;
        if is_mobile_matter(id) && !matches!(id, PIPE | PPIP) {
            sim.aux[i].ctype = id;
            sim.aux[i].tmp2 = cells[ni].temperature as i32;
            cells[ni] = Cell::empty();
            sim.aux[ni] = Default::default();
            return;
        }
    }
}

pub fn pipe_color(id: u16, ctype: u16, tmp: i32, temp: f32) -> (u8, u8, u8, u8) {
    let paused = id == PPIP && tmp & PAUSED != 0;
    let phase = ((tmp >> 8) & 3) as u8;
    let (mut r, mut g, mut b, a) = if id == PPIP {
        (68, 68, 110, 255)
    } else {
        (70, 70, 70, 255)
    };
    if ctype != 0 {
        let (cr, cg, cb, _) = super::ids::base_rgba(ctype);
        r = ((r as u16 + cr as u16) / 2) as u8;
        g = ((g as u16 + cg as u16) / 2) as u8;
        b = ((b as u16 + cb as u16) / 2) as u8;
    }
    match phase {
        0 => r = r.saturating_add(28),
        1 => g = g.saturating_add(28),
        2 => b = b.saturating_add(28),
        _ => {
            r = r.saturating_add(12);
            g = g.saturating_add(12);
            b = b.saturating_add(12);
        }
    }
    if paused {
        r = r / 2;
        g = g / 2;
        b = b / 2;
    } else if id == PPIP {
        r = r.saturating_add(16);
        g = g.saturating_add(16);
    }
    if temp > 800.0 {
        let k = ((temp - 800.0) / 800.0).clamp(0.0, 1.0);
        r = r.saturating_add((80.0 * k) as u8);
    }
    (r, g, b, a)
}
