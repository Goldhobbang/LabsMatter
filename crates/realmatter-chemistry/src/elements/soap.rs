//! Soap films.
//!
//! Links are reciprocal cell indices (`tmp` forward, `tmp2` back) with the
//! reference ctype bits: 1 bubble, 2 forward mate, 4 back mate. A deleted or
//! reused index is dropped, and a broken middle link is stitched to the two
//! survivors. Wrap mode measures distance across the border. The air solver
//! does not expose TPT's signed `pv`, so a bubble also starts when a pump has
//! pushed this cell's pressure off atmosphere.

use realmatter_core::cell::{cell_flags, Cell};

use super::ids::*;
use super::sim::{at, ElementSim};

const BUBBLE: u16 = 1;
const FWD: u16 = 2;
const BACK: u16 = 4;

pub fn update(cells: &mut [Cell], sim: &mut ElementSim, w: usize, h: usize, wrap: bool) {
    let n = cells.len().min(sim.aux.len());
    for i in 0..n {
        if cells[i].material != SOAP {
            continue;
        }
        repair_one(cells, sim, i);
    }
    let mut starts = Vec::new();
    for i in 0..n {
        if cells[i].material == SOAP && sim.aux[i].ctype & BUBBLE == 0 {
            let crowded =
                (cells[i].pressure - ATM).abs() > 0.5 || neighbor_soap(cells, i, w, h) >= 2;
            if crowded {
                starts.push(i);
            }
        }
    }
    for i in starts {
        if cells[i].material == SOAP {
            sim.aux[i].ctype |= BUBBLE;
            sim.aux[i].life = 10;
        }
    }
    for i in 0..n {
        if cells[i].material != SOAP || sim.aux[i].ctype & BUBBLE == 0 {
            continue;
        }
        if sim.aux[i].life > 0 {
            sim.aux[i].life -= 1;
        }
        try_attach(cells, sim, i, w, h, wrap);
        pull(cells, sim, i, w, h, wrap);
        cells[i].set_flag(cell_flags::FIXED, sim.aux[i].ctype & (FWD | BACK) != 0);
    }
}

fn neighbor_soap(cells: &[Cell], i: usize, w: usize, h: usize) -> i32 {
    let x = (i % w) as i32;
    let y = (i / w) as i32;
    let mut n = 0;
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            if let Some((ni, _, _)) = at(w, h, x, y, dx, dy, false) {
                if cells[ni].material == SOAP {
                    n += 1;
                }
            }
        }
    }
    n
}

fn repair_one(cells: &[Cell], sim: &mut ElementSim, i: usize) {
    let fwd = sim.aux[i].tmp;
    let back = sim.aux[i].tmp2;
    let bad_fwd = sim.aux[i].ctype & FWD != 0 && !reciprocal(cells, sim, i, fwd, true);
    let bad_back = sim.aux[i].ctype & BACK != 0 && !reciprocal(cells, sim, i, back, false);
    if bad_fwd {
        sim.aux[i].ctype &= !FWD;
        sim.aux[i].tmp = -1;
    }
    if bad_back {
        sim.aux[i].ctype &= !BACK;
        sim.aux[i].tmp2 = -1;
    }
}

/// `forward` is true when `i` stores `other` in `tmp` and `other` should store `i` in `tmp2`.
fn reciprocal(cells: &[Cell], sim: &ElementSim, i: usize, other: i32, forward: bool) -> bool {
    if !soap_at(cells, other) {
        return false;
    }
    let o = other as usize;
    if forward {
        sim.aux[o].tmp2 == i as i32 && sim.aux[o].ctype & BACK != 0
    } else {
        sim.aux[o].tmp == i as i32 && sim.aux[o].ctype & FWD != 0
    }
}

fn soap_at(cells: &[Cell], index: i32) -> bool {
    index >= 0 && (index as usize) < cells.len() && cells[index as usize].material == SOAP
}

fn try_attach(cells: &mut [Cell], sim: &mut ElementSim, i: usize, w: usize, h: usize, wrap: bool) {
    if sim.aux[i].ctype & FWD != 0 && sim.aux[i].ctype & BACK != 0 {
        return;
    }
    let x = (i % w) as i32;
    let y = (i / w) as i32;
    for dy in -2..=2 {
        for dx in -2..=2 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let Some((ni, _, _)) = at(w, h, x, y, dx, dy, wrap) else {
                continue;
            };
            if ni == i || cells[ni].material != SOAP || sim.aux[ni].ctype & BUBBLE == 0 {
                continue;
            }
            if cells[ni].is_empty() {
                continue;
            }
            if solid_between(cells, w, h, x, y, dx, dy) {
                continue;
            }
            attach(sim, i, ni);
            if sim.aux[i].ctype & FWD != 0 && sim.aux[i].ctype & BACK != 0 {
                return;
            }
        }
    }
}

fn solid_between(cells: &[Cell], w: usize, h: usize, x: i32, y: i32, dx: i32, dy: i32) -> bool {
    if dx.abs() + dy.abs() <= 1 {
        return false;
    }
    let mx = x + dx.signum();
    let my = y + dy.signum();
    if let Some((ni, _, _)) = at(w, h, mx, my, 0, 0, false) {
        let id = cells[ni].material;
        return id != EMPTY && id != SOAP && id != GLAS && is_solid_obstacle(id);
    }
    false
}

pub fn attach(sim: &mut ElementSim, a: usize, b: usize) {
    if a == b || a >= sim.aux.len() || b >= sim.aux.len() {
        return;
    }
    if sim.aux[a].tmp == b as i32 || sim.aux[a].tmp2 == b as i32 {
        return;
    }
    if sim.aux[b].ctype & BACK == 0 && sim.aux[a].ctype & FWD == 0 {
        sim.aux[a].ctype |= FWD | BUBBLE;
        sim.aux[a].tmp = b as i32;
        sim.aux[b].ctype |= BACK | BUBBLE;
        sim.aux[b].tmp2 = a as i32;
    } else if sim.aux[b].ctype & FWD == 0 && sim.aux[a].ctype & BACK == 0 {
        sim.aux[a].ctype |= BACK | BUBBLE;
        sim.aux[a].tmp2 = b as i32;
        sim.aux[b].ctype |= FWD | BUBBLE;
        sim.aux[b].tmp = a as i32;
    }
}

pub fn detach(cells: &[Cell], sim: &mut ElementSim, i: usize) {
    if i >= sim.aux.len() {
        return;
    }
    let fwd = sim.aux[i].tmp;
    let back = sim.aux[i].tmp2;
    let had_fwd = sim.aux[i].ctype & FWD != 0;
    let had_back = sim.aux[i].ctype & BACK != 0;
    if had_fwd && had_back && soap_at(cells, fwd) && soap_at(cells, back) {
        // Middle link deleted: stitch the two neighbors together.
        let f = fwd as usize;
        let b = back as usize;
        sim.aux[b].tmp = fwd;
        sim.aux[b].ctype |= FWD;
        sim.aux[f].tmp2 = back;
        sim.aux[f].ctype |= BACK;
    } else {
        if had_fwd && soap_at(cells, fwd) {
            let f = fwd as usize;
            if sim.aux[f].tmp2 == i as i32 {
                sim.aux[f].ctype &= !BACK;
                sim.aux[f].tmp2 = -1;
            }
        }
        if had_back && soap_at(cells, back) {
            let b = back as usize;
            if sim.aux[b].tmp == i as i32 {
                sim.aux[b].ctype &= !FWD;
                sim.aux[b].tmp = -1;
            }
        }
    }
    sim.aux[i].ctype = 0;
    sim.aux[i].tmp = -1;
    sim.aux[i].tmp2 = -1;
}

fn pull(cells: &mut [Cell], sim: &mut ElementSim, i: usize, w: usize, h: usize, wrap: bool) {
    if sim.aux[i].ctype & FWD == 0 {
        return;
    }
    let other = sim.aux[i].tmp;
    if !soap_at(cells, other) {
        sim.aux[i].ctype &= !FWD;
        sim.aux[i].tmp = -1;
        return;
    }
    let o = other as usize;
    let (dx, dy) = delta(i, o, w, h, wrap);
    let dist2 = dx * dx + dy * dy;
    if dist2 > 36.0 {
        detach(cells, sim, i);
        cells[i].set_flag(cell_flags::FIXED, false);
        return;
    }
    let d = 9.0 / (dist2 + 9.0) - 0.5;
    cells[i].vx += dx * d;
    cells[i].vy += dy * d;
    cells[o].vx -= dx * d;
    cells[o].vy -= dy * d;
    if dist2 > 4.0 {
        let step_x = dx.signum() as i32;
        let step_y = dy.signum() as i32;
        let x = (i % w) as i32;
        let y = (i / w) as i32;
        if let Some((ni, _, _)) = at(w, h, x, y, step_x, step_y, wrap) {
            if cells[ni].is_empty() {
                sim.exchange(cells, i, ni);
            }
        }
    }
}

fn delta(a: usize, b: usize, w: usize, h: usize, wrap: bool) -> (f32, f32) {
    let ax = (a % w) as i32;
    let ay = (a / w) as i32;
    let bx = (b % w) as i32;
    let by = (b / w) as i32;
    let mut dx = (ax - bx) as f32;
    let mut dy = (ay - by) as f32;
    if wrap {
        if dx > w as f32 * 0.5 {
            dx -= w as f32;
        }
        if dx < -(w as f32) * 0.5 {
            dx += w as f32;
        }
        if dy > h as f32 * 0.5 {
            dy -= h as f32;
        }
        if dy < -(h as f32) * 0.5 {
            dy += h as f32;
        }
    }
    (dx, dy)
}
