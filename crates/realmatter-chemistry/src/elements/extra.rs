//! Clones, portals, and a bounded set of energy particles.
//!
//! These follow the reference control flow that fits a single cell per site:
//! clones copy `ctype` into one empty neighbor, portals move mobile cells
//! between a temperature channel, neutrons heat plutonium, electrons ignite
//! conductors, photons cross glass.

use realmatter_core::cell::{cell_flags, Cell};

use super::ids::*;
use super::sim::{at, ElementSim, PortalCargo};

pub fn update(cells: &mut [Cell], sim: &mut ElementSim, w: usize, h: usize, wrap: bool) {
    clones(cells, sim, w, h, wrap);
    portals(cells, sim, w, h, wrap);
    neutrons(cells, sim, w, h);
    electrons(cells, sim, w, h);
    photons(cells, sim, w, h);
}

fn clones(cells: &mut [Cell], sim: &mut ElementSim, w: usize, h: usize, wrap: bool) {
    let n = cells.len().min(sim.aux.len());
    for i in 0..n {
        let id = cells[i].material;
        if !matches!(id, CLNE | PCLN | PBCN) {
            continue;
        }
        if sim.aux[i].ctype == 0 {
            sample_ctype(cells, sim, i, w, h, wrap);
        }
        let powered = id == CLNE || sim.aux[i].life == 10;
        if !powered || sim.aux[i].ctype == 0 {
            continue;
        }
        let ctype = sim.aux[i].ctype;
        if matches!(ctype, CLNE | PCLN | PBCN | SPRK | STKM | STK2 | FIGH) {
            continue;
        }
        let x = (i % w) as i32;
        let y = (i / w) as i32;
        let rot = (sim.tick as i32 + x + y) & 3;
        let dirs = [(1, 0), (0, 1), (-1, 0), (0, -1)];
        for k in 0..4 {
            let (dx, dy) = dirs[(rot as usize + k) & 3];
            let Some((ni, _, _)) = at(w, h, x, y, dx, dy, wrap) else {
                continue;
            };
            if cells[ni].is_empty() {
                cells[ni].material = ctype;
                cells[ni].temperature = cells[i].temperature.min(1200.0);
                cells[ni].set_flag(cell_flags::FIXED, is_solid_obstacle(ctype));
                sim.aux[ni] = Default::default();
                if id == PBCN {
                    sim.aux[i].tmp2 += 1;
                    if sim.aux[i].tmp2 > 24 {
                        cells[i] = Cell::empty();
                        sim.aux[i] = Default::default();
                    }
                }
                break;
            }
        }
    }
}

fn sample_ctype(cells: &[Cell], sim: &mut ElementSim, i: usize, w: usize, h: usize, wrap: bool) {
    let x = (i % w) as i32;
    let y = (i / w) as i32;
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let Some((ni, _, _)) = at(w, h, x, y, dx, dy, wrap) else {
                continue;
            };
            let rt = cells[ni].material;
            if rt != EMPTY
                && !matches!(
                    rt,
                    CLNE | PCLN | PBCN | SPRK | PSCN | NSCN | STKM | STK2 | FIGH
                )
            {
                sim.aux[i].ctype = rt;
                return;
            }
        }
    }
}

fn portals(cells: &mut [Cell], sim: &mut ElementSim, w: usize, h: usize, wrap: bool) {
    let n = cells.len().min(sim.aux.len());
    let mut ins = Vec::new();
    for i in 0..n {
        if cells[i].material == PRTI {
            ins.push(i);
        }
    }
    for i in ins {
        let ch = channel_from_temp(cells[i].temperature);
        sim.aux[i].tmp = ch as i32;
        if sim.portals[ch].len() >= 8 {
            continue;
        }
        let x = (i % w) as i32;
        let y = (i / w) as i32;
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let Some((ni, _, _)) = at(w, h, x, y, dx, dy, wrap) else {
                continue;
            };
            let id = cells[ni].material;
            if is_mobile_matter(id) {
                sim.portals[ch].push(PortalCargo {
                    material: id,
                    temp: cells[ni].temperature,
                    ctype: sim.aux[ni].ctype,
                });
                cells[ni] = Cell::empty();
                sim.aux[ni] = Default::default();
                break;
            }
        }
    }
    let mut outs = Vec::new();
    for i in 0..cells.len() {
        if cells[i].material == PRTO {
            outs.push(i);
        }
    }
    for i in outs {
        let ch = channel_from_temp(cells[i].temperature);
        if sim.portals[ch].is_empty() {
            continue;
        }
        let x = (i % w) as i32;
        let y = (i / w) as i32;
        for (dx, dy) in [(0, 1), (1, 0), (0, -1), (-1, 0)] {
            let Some((ni, _, _)) = at(w, h, x, y, dx, dy, wrap) else {
                continue;
            };
            if cells[ni].is_empty() {
                let cargo = sim.portals[ch].remove(0);
                cells[ni].material = cargo.material;
                cells[ni].temperature = cargo.temp;
                cells[ni].set_flag(cell_flags::FIXED, is_solid_obstacle(cargo.material));
                sim.aux[ni].ctype = cargo.ctype;
                break;
            }
        }
    }
}

fn neutrons(cells: &mut [Cell], sim: &mut ElementSim, w: usize, h: usize) {
    let n = cells.len().min(sim.aux.len());
    let mut moving = Vec::new();
    for i in 0..n {
        if cells[i].material == NEUT {
            moving.push(i);
        }
    }
    for i in moving {
        if cells[i].material != NEUT {
            continue;
        }
        let dir = sim.aux[i].tmp.rem_euclid(4);
        let (dx, dy) = dir_vec(dir);
        let x = (i % w) as i32;
        let y = (i / w) as i32;
        let Some((ni, _, _)) = at(w, h, x, y, dx, dy, false) else {
            cells[i] = Cell::empty();
            sim.aux[i] = Default::default();
            continue;
        };
        match cells[ni].material {
            EMPTY | STM | SMKE | HYGN | CO2 | GLAS | WATR => {
                sim.exchange(cells, i, ni);
            }
            PLUT => {
                cells[ni].temperature = (cells[ni].temperature + 80.0).min(5000.0);
                if sim.rng_below(3) == 0 {
                    let side = (dir + 1) & 3;
                    let (sx, sy) = dir_vec(side);
                    if let Some((sj, _, _)) = at(w, h, x, y, sx, sy, false) {
                        if cells[sj].is_empty() {
                            cells[sj].material = NEUT;
                            cells[sj].temperature = 400.0;
                            sim.aux[sj].tmp = side;
                        }
                    }
                }
                if sim.rng_below(2) == 0 {
                    cells[i] = Cell::empty();
                    sim.aux[i] = Default::default();
                }
            }
            _ => {
                cells[ni].temperature = (cells[ni].temperature + 15.0).min(5000.0);
                cells[i] = Cell::empty();
                sim.aux[i] = Default::default();
            }
        }
    }
}

fn electrons(cells: &mut [Cell], sim: &mut ElementSim, w: usize, h: usize) {
    let n = cells.len().min(sim.aux.len());
    let mut moving = Vec::new();
    for i in 0..n {
        if cells[i].material == ELEC {
            moving.push(i);
        }
    }
    for i in moving {
        if cells[i].material != ELEC {
            continue;
        }
        let dir = sim.aux[i].tmp.rem_euclid(4);
        let (dx, dy) = dir_vec(dir);
        let x = (i % w) as i32;
        let y = (i / w) as i32;
        let Some((ni, _, _)) = at(w, h, x, y, dx, dy, false) else {
            cells[i] = Cell::empty();
            sim.aux[i] = Default::default();
            continue;
        };
        if conducts(cells[ni].material) && sim.aux[ni].life == 0 {
            super::spark::ignite(
                cells,
                sim,
                ni,
                super::sim::spark_birth_life(cells[ni].material),
            );
            cells[i] = Cell::empty();
            sim.aux[i] = Default::default();
        } else if cells[ni].is_empty() || matches!(cells[ni].material, GLAS | WATR) {
            sim.exchange(cells, i, ni);
        } else {
            cells[i] = Cell::empty();
            sim.aux[i] = Default::default();
        }
    }
}

fn photons(cells: &mut [Cell], sim: &mut ElementSim, w: usize, h: usize) {
    let n = cells.len().min(sim.aux.len());
    let mut moving = Vec::new();
    for i in 0..n {
        if cells[i].material == PHOT {
            moving.push(i);
        }
    }
    for i in moving {
        if cells[i].material != PHOT {
            continue;
        }
        let dir = if sim.aux[i].tmp == 0 {
            0
        } else {
            sim.aux[i].tmp.rem_euclid(4)
        };
        let (dx, dy) = dir_vec(dir);
        let x = (i % w) as i32;
        let y = (i / w) as i32;
        let Some((ni, _, _)) = at(w, h, x, y, dx, dy, false) else {
            cells[i] = Cell::empty();
            sim.aux[i] = Default::default();
            continue;
        };
        match cells[ni].material {
            EMPTY | GLAS | WATR => sim.exchange(cells, i, ni),
            _ => {
                cells[ni].temperature = (cells[ni].temperature + 25.0).min(5000.0);
                cells[i] = Cell::empty();
                sim.aux[i] = Default::default();
            }
        }
    }
}

fn dir_vec(dir: i32) -> (i32, i32) {
    match dir {
        1 => (0, 1),
        2 => (-1, 0),
        3 => (0, -1),
        _ => (1, 0),
    }
}
