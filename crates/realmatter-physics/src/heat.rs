//! Heat conduction, temperature fields, specific/latent heat, phase transitions.
//!
//! Realistic heat diffusion between neighboring cells with stability guards.

use crate::core;
use crate::core::cell::Cell;
use crate::core::cell::cell_flags;
use crate::core::material::MaterialProps;

/// Fourier heat conduction on 2D grid using explicit finite differences.
/// Returns total energy exchanged (for conservation tracking).
pub fn conduct(cells: &mut [Cell], width: u32, height: u32, materials: &[MaterialProps], dt: f32) -> f32 {
    let w = width as usize;
    let h = height as usize;
    if cells.len() != w * h {
        return 0.0;
    }
    let temps: Vec<f32> = cells.iter().map(|c| c.temperature).collect();
    let mut new_temps = temps.clone();
    let mut energy = 0.0f32;
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if cells[i].is_empty() {
                continue;
            }
            let m = material_or_default(materials, cells[i].material);
            if m.thermal_conductivity <= 0.0 {
                continue;
            }
            let rho = m.density.max(1e-6);
            let c = m.specific_heat.max(1e-6);
            let k = m.thermal_conductivity;
            let dx2 = 0.01f32 * 0.01f32;
            // Diffusion coefficient D = k/(rho*c), clamped for stability (CFL-like)
            let d = (k / (rho * c)).min(1e-3);
            let coeff = (d * dt / dx2).min(0.25);
            if coeff <= 0.0 {
                continue;
            }
            // Laplacian over 4-neighbors
            let mut lap = 0.0f32;
            let mut neighbors = 0u32;
            if x > 0 && !cells[i - 1].is_empty() {
                lap += temps[i - 1] - temps[i];
                neighbors += 1;
            }
            if x + 1 < w && !cells[i + 1].is_empty() {
                lap += temps[i + 1] - temps[i];
                neighbors += 1;
            }
            if y > 0 && !cells[i - w].is_empty() {
                lap += temps[i - w] - temps[i];
                neighbors += 1;
            }
            if y + 1 < h && !cells[i + w].is_empty() {
                lap += temps[i + w] - temps[i];
                neighbors += 1;
            }
            if neighbors > 0 {
                let delta = coeff * lap;
                new_temps[i] = temps[i] + delta;
                energy += delta.abs() * rho * c * (0.01 * 0.01 * 0.01);
            }
        }
    }
    for (cell, t) in cells.iter_mut().zip(new_temps.iter()) {
        if !cell.is_empty() {
            cell.temperature = *t;
        }
    }
    energy
}

/// Temperature-dependent phase transitions with heat effects.
pub fn phase_transitions(cells: &mut [Cell], materials: &[MaterialProps]) {
    for cell in cells.iter_mut() {
        if cell.is_empty() {
            continue;
        }
        let m = material_or_default(materials, cell.material);
        // Melting
        if m.state == core::material::MaterialState::Solid
            && m.melting_point.is_finite()
            && cell.temperature >= m.melting_point
        {
            cell.temperature = m.melting_point;
            cell.set_flag(cell_flags::WET, true);
        }
        // Freezing
        if m.state == core::material::MaterialState::Liquid
            && m.melting_point.is_finite()
            && cell.temperature <= m.melting_point
        {
            cell.temperature = m.melting_point;
            cell.set_flag(cell_flags::WET, false);
        }
        // Boiling
        if m.state == core::material::MaterialState::Liquid
            && m.boiling_point.is_finite()
            && cell.temperature >= m.boiling_point
        {
            cell.temperature = m.boiling_point;
        }
        // Condensation
        if m.state == core::material::MaterialState::Gas
            && m.boiling_point.is_finite()
            && cell.temperature <= m.boiling_point
        {
            cell.temperature = m.boiling_point;
        }
    }
}

pub fn specific_heat_energy(mass: f32, specific_heat: f32, delta_t: f32) -> f32 {
    mass * specific_heat * delta_t
}

pub fn temperature_change(energy: f32, mass: f32, specific_heat: f32) -> f32 {
    if mass <= 0.0 || specific_heat <= 0.0 {
        return 0.0;
    }
    energy / (mass * specific_heat)
}

pub fn latent_heat_energy(mass: f32, latent_heat: f32) -> f32 {
    mass * latent_heat
}

pub fn cell_mass(props: &MaterialProps) -> f32 {
    props.density * (0.01f32 * 0.01f32 * 0.01f32)
}

fn material_or_default(materials: &[MaterialProps], id: core::material::MaterialId) -> MaterialProps {
    materials
        .get(id as usize)
        .cloned()
        .unwrap_or_else(|| crate::default_material_props(id))
}
