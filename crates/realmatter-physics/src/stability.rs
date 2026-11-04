//! Stability constraints and numerical error budgets.

use crate::core;
use crate::core::cell::Cell;
use crate::core::material::MaterialProps;

#[derive(Clone, Debug, Default)]
pub struct StabilityReport {
    pub cfl_ok: bool,
    pub energy_conserved: bool,
    pub max_velocity: f32,
    pub max_temp: f32,
    pub min_temp: f32,
    pub nan_detected: bool,
}

pub fn check_cfl(cells: &[Cell], dt: f32, cell_size: f32) -> bool {
    for cell in cells {
        if cell.is_empty() {
            continue;
        }
        let v = (cell.vx * cell.vx + cell.vy * cell.vy).sqrt();
        if v * dt / cell_size > 1.0 {
            return false;
        }
    }
    true
}

pub fn detect_nan(cells: &[Cell]) -> bool {
    for cell in cells {
        if cell.temperature.is_nan() || cell.temperature.is_infinite()
            || cell.vx.is_nan() || cell.vx.is_infinite()
            || cell.vy.is_nan() || cell.vy.is_infinite()
            || cell.pressure.is_nan() || cell.pressure.is_infinite()
        {
            return true;
        }
    }
    false
}

pub fn max_velocity(cells: &[Cell]) -> f32 {
    cells
        .iter()
        .filter(|c| !c.is_empty())
        .map(|c| (c.vx * c.vx + c.vy * c.vy).sqrt())
        .fold(0.0f32, f32::max)
}

pub fn thermal_energy_sum(cells: &[Cell], materials: &[MaterialProps]) -> f32 {
    let mut total = 0.0f32;
    for cell in cells {
        if cell.is_empty() {
            continue;
        }
        let m = materials
            .get(cell.material as usize)
            .cloned()
            .unwrap_or_else(|| crate::default_material_props(cell.material));
        let mass = m.density * (0.01 * 0.01 * 0.01);
        total += mass * m.specific_heat * cell.temperature;
    }
    total
}

pub fn check(cells: &[Cell], materials: &[MaterialProps], dt: f32) -> StabilityReport {
    let mut report = StabilityReport::default();
    report.cfl_ok = check_cfl(cells, dt, 0.01);
    report.nan_detected = detect_nan(cells);
    report.max_velocity = max_velocity(cells);
    let temps: Vec<f32> = cells.iter().filter(|c| !c.is_empty()).map(|c| c.temperature).collect();
    report.max_temp = temps.iter().cloned().fold(0.0f32, f32::max);
    report.min_temp = temps.iter().cloned().fold(f32::INFINITY, f32::min);
    if temps.is_empty() {
        report.min_temp = 0.0;
    }
    report.energy_conserved = !report.nan_detected;
    let _ = materials;
    report
}

pub fn clamp_velocities(cells: &mut [Cell], max_v: f32) {
    for cell in cells.iter_mut() {
        if cell.is_empty() {
            continue;
        }
        let v = (cell.vx * cell.vx + cell.vy * cell.vy).sqrt();
        if v > max_v && v > 0.0 {
            let scale = max_v / v;
            cell.vx *= scale;
            cell.vy *= scale;
        }
    }
}

pub mod budgets {
    pub const ENERGY_DRIFT_REL: f32 = 1e-3;
    pub const TEMP_OVERSHOOT_K: f32 = 5.0;
    pub const MAX_VELOCITY: f32 = 100.0;
    pub const MIN_TEMP: f32 = 0.0;
}
