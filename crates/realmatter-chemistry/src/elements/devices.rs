//! Powered devices that are not the spark itself: switches, pumps, thermistors'
//! neighbors, liquid crystal, and the electromagnetic pulse.

use realmatter_core::cell::{cell_flags, Cell};

use super::ids::*;
use super::sim::{at, ElementSim};

pub fn update(cells: &mut [Cell], sim: &mut ElementSim, w: usize, h: usize, wrap: bool) {
    wifi(cells, sim, w, h, wrap);
    switches(cells, sim, w, h, wrap);
    pumps(cells, sim, w, h, wrap);
    gravity_pumps(cells, sim, w, h, wrap);
    heat_switches(cells, sim, w, h, wrap);
    liquid_crystal(cells, sim, w, h, wrap);
    emp_pulse(cells, sim, w, h);
    powered_life(cells, sim);
}

fn wifi(cells: &mut [Cell], sim: &mut ElementSim, w: usize, h: usize, wrap: bool) {
    let n = cells.len().min(sim.aux.len());
    for i in 0..n {
        if cells[i].material != WIFI {
            continue;
        }
        let ch = channel_from_temp(cells[i].temperature);
        sim.aux[i].tmp = ch as i32;
        let active = sim.wireless[0][ch] != 0;
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
                if active
                    && matches!(cells[ni].material, NSCN | PSCN | INWR)
                    && sim.aux[ni].life == 0
                {
                    super::spark::ignite(cells, sim, ni, 4);
                }
                if cells[ni].material == SPRK && sim.aux[ni].ctype != NSCN && sim.aux[ni].life >= 3
                {
                    sim.wireless[1][ch] = 1;
                }
            }
        }
    }
}

fn switches(cells: &mut [Cell], sim: &mut ElementSim, w: usize, h: usize, wrap: bool) {
    let n = cells.len().min(sim.aux.len());
    for i in 0..n {
        if cells[i].material != SWCH {
            continue;
        }
        if sim.aux[i].life > 0 && sim.aux[i].life != 10 {
            sim.aux[i].life -= 1;
        }
    }
    for i in 0..n {
        if cells[i].material != SWCH {
            continue;
        }
        let x = (i % w) as i32;
        let y = (i / w) as i32;
        for dy in -2..=2 {
            for dx in -2..=2 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let Some((ni, nx, ny)) = at(w, h, x, y, dx, dy, wrap) else {
                    continue;
                };
                if super::sim::midpoint_blocks(cells, w, h, x, y, nx, ny) {
                    continue;
                }
                if cells[ni].material == SWCH {
                    if sim.aux[i].life >= 10 && sim.aux[ni].life < 10 && sim.aux[ni].life > 0 {
                        sim.aux[i].life = 9;
                    } else if sim.aux[i].life == 0 && sim.aux[ni].life >= 10 {
                        sim.aux[i].life = sim.aux[ni].life;
                    }
                } else if cells[ni].material == SPRK
                    && sim.aux[i].life == 10
                    && sim.aux[ni].life > 0
                    && !matches!(sim.aux[ni].ctype, PSCN | NSCN)
                {
                    sim.aux[i].ctype = SWCH;
                    sim.aux[i].life = 4;
                    cells[i].material = SPRK;
                    cells[i].set_flag(cell_flags::FIXED, true);
                }
            }
        }
    }
}

fn pumps(cells: &mut [Cell], sim: &mut ElementSim, w: usize, h: usize, wrap: bool) {
    let n = cells.len().min(sim.aux.len());
    for i in 0..n {
        if cells[i].material != PUMP {
            continue;
        }
        if sim.aux[i].life != 10 {
            continue;
        }
        let desired = cells[i].temperature - 273.15;
        let x = (i % w) as i32;
        let y = (i / w) as i32;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx != 0 && dy != 0 {
                    continue;
                }
                let Some((ni, _, _)) = at(w, h, x, y, dx, dy, wrap) else {
                    continue;
                };
                let pv = cells[ni].pressure - ATM;
                let next = pv + 0.1 * (desired - pv);
                cells[ni].pressure = ATM + next;
            }
        }
    }
}

fn gravity_pumps(cells: &mut [Cell], sim: &mut ElementSim, w: usize, h: usize, wrap: bool) {
    // The shared gravity field is owned by the physics solver and has no
    // per-cell mass write. While that API is absent, an active GPMP adds a
    // one-step velocity bias to adjacent mobile cells. The requested signature
    // is `set_gravity_bias(x: i32, y: i32, mass: f32)`.
    let n = cells.len().min(sim.aux.len());
    for i in 0..n {
        if cells[i].material != GPMP || sim.aux[i].life != 10 {
            continue;
        }
        let g = ((cells[i].temperature - 273.15) / 256.0).clamp(-1.0, 1.0);
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
                if cells[ni].material == GPMP {
                    if sim.aux[ni].life < 10 && sim.aux[ni].life > 0 {
                        sim.aux[i].life = 9;
                    } else if sim.aux[ni].life == 0 {
                        sim.aux[ni].life = 10;
                    }
                } else if is_mobile_matter(cells[ni].material) {
                    cells[ni].vy += g * 0.35;
                }
            }
        }
    }
}

fn heat_switches(cells: &mut [Cell], sim: &mut ElementSim, w: usize, h: usize, wrap: bool) {
    let n = cells.len().min(sim.aux.len());
    for i in 0..n {
        if cells[i].material != HSWC || sim.aux[i].life != 10 {
            continue;
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
                if cells[ni].material == HSWC {
                    if sim.aux[ni].life < 10 && sim.aux[ni].life > 0 {
                        sim.aux[i].life = 9;
                    } else if sim.aux[ni].life == 0 {
                        sim.aux[ni].life = 10;
                    }
                } else if cells[ni].material != EMPTY {
                    let avg = (cells[i].temperature + cells[ni].temperature) * 0.5;
                    cells[i].temperature = avg;
                    cells[ni].temperature = avg;
                }
            }
        }
    }
}

fn liquid_crystal(cells: &mut [Cell], sim: &mut ElementSim, w: usize, h: usize, wrap: bool) {
    let n = cells.len().min(sim.aux.len());
    for i in 0..n {
        if cells[i].material != LCRY {
            continue;
        }
        let (check, setto) = match sim.aux[i].tmp {
            1 => {
                if sim.aux[i].life <= 0 {
                    sim.aux[i].tmp = 0;
                } else {
                    sim.aux[i].life -= 2;
                    if sim.aux[i].life < 0 {
                        sim.aux[i].life = 0;
                    }
                    sim.aux[i].tmp2 = i32::from(sim.aux[i].life);
                }
                (3, 1)
            }
            0 => (3, 1),
            2 => {
                if sim.aux[i].life >= 10 {
                    sim.aux[i].tmp = 3;
                } else {
                    sim.aux[i].life += 2;
                    if sim.aux[i].life > 10 {
                        sim.aux[i].life = 10;
                    }
                    sim.aux[i].tmp2 = i32::from(sim.aux[i].life);
                }
                (0, 2)
            }
            3 => (0, 2),
            _ => {
                sim.aux[i].tmp = 0;
                sim.aux[i].life = 0;
                continue;
            }
        };
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
                if cells[ni].material == LCRY && sim.aux[ni].tmp == check {
                    sim.aux[ni].tmp = setto;
                }
            }
        }
    }
}

fn emp_pulse(cells: &mut [Cell], sim: &mut ElementSim, w: usize, h: usize) {
    let n = cells.len().min(sim.aux.len());
    let mut origins = Vec::new();
    for i in 0..n {
        if cells[i].material == EMP && sim.aux[i].life > 180 {
            origins.push(i);
        }
    }
    for i in origins {
        let x = (i % w) as i32;
        let y = (i / w) as i32;
        let radius = 8i32;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy > radius * radius {
                    continue;
                }
                let Some((ni, _, _)) = at(w, h, x, y, dx, dy, false) else {
                    continue;
                };
                if cells[ni].material == SPRK {
                    sim.aux[ni].life = 0;
                } else if matches!(cells[ni].material, METL | IRON | COPR | PSCN | NSCN)
                    && sim.rng_below(48) == 0
                {
                    cells[ni].material = BRMT;
                    cells[ni].set_flag(cell_flags::FIXED, true);
                    sim.aux[ni].life = 0;
                    sim.aux[ni].ctype = 0;
                }
            }
        }
    }
}

fn powered_life(cells: &mut [Cell], sim: &mut ElementSim) {
    let n = cells.len().min(sim.aux.len());
    for i in 0..n {
        if !matches!(cells[i].material, PUMP | GPMP | HSWC | PBCN | PCLN) {
            continue;
        }
        if sim.aux[i].life > 0 && sim.aux[i].life != 10 {
            sim.aux[i].life -= 1;
        }
    }
}
