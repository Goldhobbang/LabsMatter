//! Gas movement, diffusion, and pressure approximations.

use crate::core;
use crate::core::cell::Cell;
use crate::core::material::MaterialProps;

pub fn update_pressure(cells: &mut [Cell], materials: &[MaterialProps]) {
    for cell in cells.iter_mut() {
        if cell.is_empty() {
            cell.pressure = core::units::constants::ATM_PRESSURE;
            continue;
        }
        let m = material_or_default(materials, cell.material);
        if m.state != core::material::MaterialState::Gas {
            cell.pressure = core::units::constants::ATM_PRESSURE;
            continue;
        }
        let rho = m.density.max(1e-6);
        let r_specific = core::units::constants::GAS_CONSTANT / (rho / 1.225).max(1e-3);
        let p = rho * r_specific * cell.temperature;
        cell.pressure = if p.is_finite() { p } else { core::units::constants::ATM_PRESSURE };
    }
}

pub fn diffuse_temperature(cells: &mut [Cell], width: u32, height: u32, materials: &[MaterialProps], dt: f32) {
    let w = width as usize;
    let h = height as usize;
    if cells.len() != w * h {
        return;
    }
    let temps: Vec<f32> = cells.iter().map(|c| c.temperature).collect();
    let mut new_temps = temps.clone();
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if cells[i].is_empty() {
                continue;
            }
            let m = material_or_default(materials, cells[i].material);
            if m.state != core::material::MaterialState::Gas {
                continue;
            }
            let d = 2e-5f32;
            let dx2 = 0.01f32 * 0.01f32;
            let coeff = (d * dt / dx2).min(0.24);
            let mut lap = 0.0f32;
            if x > 0 { lap += temps[i - 1] - temps[i]; }
            if x + 1 < w { lap += temps[i + 1] - temps[i]; }
            if y > 0 { lap += temps[i - w] - temps[i]; }
            if y + 1 < h { lap += temps[i + w] - temps[i]; }
            new_temps[i] = temps[i] + coeff * lap;
        }
    }
    for (cell, t) in cells.iter_mut().zip(new_temps.iter()) {
        if !cell.is_empty() {
            cell.temperature = *t;
        }
    }
}

pub fn diffuse(cells: &mut [Cell], width: u32, height: u32, materials: &[MaterialProps], dt: f32) {
    diffuse_temperature(cells, width, height, materials, dt);
    let w = width as usize;
    let h = height as usize;
    if cells.len() != w * h {
        return;
    }
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if cells[i].is_empty() {
                continue;
            }
            let m = material_or_default(materials, cells[i].material);
            if m.state != core::material::MaterialState::Gas {
                continue;
            }
            let pi = cells[i].pressure;
            let mut best = i;
            let mut best_p = pi;
            for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < h {
                    let ni = ny as usize * w + nx as usize;
                    if cells[ni].is_empty() && cells[ni].pressure < best_p {
                        best_p = cells[ni].pressure;
                        best = ni;
                    }
                }
            }
            if best != i {
                cells[best] = cells[i];
                cells[i] = Cell::default();
            }
        }
    }
    let _ = dt;
}

pub fn apply_buoyancy(cells: &mut [Cell], width: u32, height: u32, materials: &[MaterialProps], dt: f32, gravity: f32) {
    let w = width as usize;
    let h = height as usize;
    if cells.len() != w * h {
        return;
    }
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if cells[i].is_empty() {
                continue;
            }
            let m = material_or_default(materials, cells[i].material);
            if m.state != core::material::MaterialState::Gas {
                continue;
            }
            let temp_factor = (cells[i].temperature - 293.15) / 293.15;
            cells[i].vy += -gravity * temp_factor * dt * 0.5;
        }
    }
}

fn material_or_default(materials: &[MaterialProps], id: core::material::MaterialId) -> MaterialProps {
    materials
        .get(id as usize)
        .cloned()
        .unwrap_or_else(|| crate::default_material_props(id))
}
