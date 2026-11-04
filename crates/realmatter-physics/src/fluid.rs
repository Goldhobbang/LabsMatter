//! Fluid behavior beyond core movement: viscosity, pressure gradients, buoyancy.

use crate::core;
use crate::core::cell::Cell;
use crate::core::material::MaterialProps;

pub fn apply_viscosity(cells: &mut [Cell], materials: &[MaterialProps], dt: f32) {
    for cell in cells.iter_mut() {
        if cell.is_empty() {
            continue;
        }
        let m = material_or_default(materials, cell.material);
        let damp = (1.0 - (m.viscosity * dt).min(0.9)).max(0.0);
        cell.vx *= damp;
        cell.vy *= damp;
    }
}

pub fn apply_pressure_gradient(cells: &mut [Cell], width: u32, height: u32, dt: f32) {
    let w = width as usize;
    let h = height as usize;
    if cells.len() != w * h {
        return;
    }
    let pressures: Vec<f32> = cells.iter().map(|c| c.pressure).collect();
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if cells[i].is_empty() {
                continue;
            }
            let mut gx = 0.0f32;
            let mut gy = 0.0f32;
            if x + 1 < w { gx += pressures[i + 1] - pressures[i]; }
            if x > 0 { gx -= pressures[i] - pressures[i - 1]; }
            if y + 1 < h { gy += pressures[i + w] - pressures[i]; }
            if y > 0 { gy -= pressures[i] - pressures[i - w]; }
            cells[i].vx -= gx * dt * 0.5;
            cells[i].vy -= gy * dt * 0.5;
        }
    }
}

pub fn apply_buoyancy(cells: &mut [Cell], width: u32, height: u32, materials: &[MaterialProps], dt: f32, gravity: f32) {
    let w = width as usize;
    let h = height as usize;
    if cells.len() != w * h {
        return;
    }
    let rho_ref = 1.225f32;
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if cells[i].is_empty() {
                continue;
            }
            let m = material_or_default(materials, cells[i].material);
            let buoy = (rho_ref - m.density) / rho_ref.max(1e-6);
            cells[i].vy += buoy * gravity * dt * 0.1;
        }
    }
}

pub fn apply_surface_tension(cells: &mut [Cell], width: u32, height: u32, dt: f32) {
    let w = width as usize;
    let h = height as usize;
    if cells.len() != w * h || w < 3 || h < 3 {
        return;
    }
    let vxs: Vec<f32> = cells.iter().map(|c| c.vx).collect();
    let vys: Vec<f32> = cells.iter().map(|c| c.vy).collect();
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let i = y * w + x;
            if cells[i].is_empty() {
                continue;
            }
            let mat = cells[i].material;
            let mut boundary = false;
            for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < h {
                    let ni = ny as usize * w + nx as usize;
                    if cells[ni].material != mat {
                        boundary = true;
                    }
                }
            }
            if boundary {
                cells[i].vx = (vxs[i] + vxs[i - 1] + vxs[i + 1] + vxs[i - w] + vxs[i + w]) * 0.2;
                cells[i].vy = (vys[i] + vys[i - 1] + vys[i + 1] + vys[i - w] + vys[i + w]) * 0.2;
            }
        }
    }
    let _ = dt;
}

fn material_or_default(materials: &[MaterialProps], id: core::material::MaterialId) -> MaterialProps {
    materials
        .get(id as usize)
        .cloned()
        .unwrap_or_else(|| crate::default_material_props(id))
}
