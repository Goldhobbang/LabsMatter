//! Heat conduction, temperature fields, specific/latent heat, phase transitions.

use crate::core;
use crate::core::cell::Cell;
use crate::core::material::MaterialProps;

pub fn conduct(cells: &mut [Cell], width: u32, height: u32, materials: &[MaterialProps], dt: f32) {
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
            let rho = m.density.max(1e-6);
            let c = m.specific_heat.max(1e-6);
            let k = m.thermal_conductivity.max(0.0);
            let dx2 = 0.01f32 * 0.01f32;
            let coeff = (k * dt / (rho * c * dx2)).min(0.24);
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

pub fn phase_transitions(cells: &mut [Cell], materials: &[MaterialProps]) {
    for cell in cells.iter_mut() {
        if cell.is_empty() {
            continue;
        }
        let m = material_or_default(materials, cell.material);
        if m.state == core::material::MaterialState::Solid
            && m.melting_point.is_finite()
            && cell.temperature >= m.melting_point
        {
            cell.temperature = m.melting_point;
            cell.set_flag(core::cell::cell_flags::WET, true);
        }
        if m.state == core::material::MaterialState::Liquid
            && m.boiling_point.is_finite()
            && cell.temperature >= m.boiling_point
        {
            cell.temperature = m.boiling_point;
        }
        if m.state == core::material::MaterialState::Liquid
            && m.melting_point.is_finite()
            && cell.temperature <= m.melting_point
        {
            cell.temperature = m.melting_point;
        }
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
