//! Mass and energy transfer between interacting materials.

use crate::core;
use crate::core::cell::Cell;
use crate::core::cell::cell_flags;
use crate::core::material::MaterialProps;

pub fn transfer_energy(cells: &mut [Cell], width: u32, height: u32, materials: &[MaterialProps], dt: f32) -> f32 {
    let w = width as usize;
    let h = height as usize;
    if cells.len() != w * h {
        return 0.0;
    }
    let mut total = 0.0f32;
    let temps: Vec<f32> = cells.iter().map(|c| c.temperature).collect();
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if cells[i].is_empty() {
                continue;
            }
            let m = material_or_default(materials, cells[i].material);
            let mass = m.density * (0.01 * 0.01 * 0.01);
            let cap = mass * m.specific_heat;
            for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || ny < 0 || (nx as usize) >= w || (ny as usize) >= h {
                    continue;
                }
                let ni = ny as usize * w + nx as usize;
                if cells[ni].is_empty() {
                    continue;
                }
                let nm = material_or_default(materials, cells[ni].material);
                let nmass = nm.density * (0.01 * 0.01 * 0.01);
                let ncap = nmass * nm.specific_heat;
                let denom = cap + ncap;
                if denom <= 0.0 {
                    continue;
                }
                let dtemp = temps[ni] - temps[i];
                let exchange = (cap * ncap / denom) * dtemp * dt * 0.1;
                if cap > 0.0 {
                    cells[i].temperature += exchange / cap;
                }
                if ncap > 0.0 {
                    cells[ni].temperature -= exchange / ncap;
                }
                total += exchange.abs();
            }
        }
    }
    total
}

pub fn transfer_mass(cells: &mut [Cell], width: u32, height: u32, dt: f32) -> u32 {
    let w = width as usize;
    let h = height as usize;
    if cells.len() != w * h {
        return 0;
    }
    let mut moved = 0u32;
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if cells[i].is_empty() {
                continue;
            }
            if cells[i].flags & cell_flags::DISSOLVED == 0 {
                continue;
            }
            for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < h {
                    let ni = ny as usize * w + nx as usize;
                    if cells[ni].is_empty() {
                        cells[ni] = cells[i];
                        cells[i] = Cell::default();
                        moved += 1;
                        break;
                    }
                }
            }
        }
    }
    let _ = dt;
    moved
}

fn material_or_default(materials: &[MaterialProps], id: core::material::MaterialId) -> MaterialProps {
    materials
        .get(id as usize)
        .cloned()
        .unwrap_or_else(|| crate::default_material_props(id))
}
