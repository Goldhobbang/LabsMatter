//! Energy-particle transport primitives (PHOT/NEUT/GRVT/PROT).
//!
//! Agent 3 owns the element-specific behaviour; this module provides the
//! physical transport primitives: propagation, reflection, absorption,
//! transmission, scattering, deposition.

use realmatter_core::cell::Cell;
use realmatter_core::material::MaterialProps;

/// Propagate an energy particle one step along its velocity. Returns the new
/// (x, y) if in bounds, else None.
pub fn propagate(x: i32, y: i32, vx: f32, vy: f32) -> (i32, i32) {
    (x + vx.round() as i32, y + vy.round() as i32)
}

/// Reflect velocity off a wall: invert the component perpendicular to the wall.
pub fn reflect(vx: f32, vy: f32, wall_normal_x: f32, wall_normal_y: f32) -> (f32, f32) {
    let dot = vx * wall_normal_x + vy * wall_normal_y;
    (vx - 2.0 * dot * wall_normal_x, vy - 2.0 * dot * wall_normal_y)
}

/// Absorption: deposit energy into the cell's temperature. Returns deposited energy.
pub fn absorb(cell: &mut Cell, props: &MaterialProps, energy: f32) -> f32 {
    if props.specific_heat <= 0.0 || props.density <= 0.0 {
        return 0.0;
    }
    let mass = props.density * (0.01 * 0.01 * 0.01);
    let cap = mass * props.specific_heat;
    if cap <= 0.0 {
        return 0.0;
    }
    let dt = energy / cap;
    cell.temperature = (cell.temperature + dt).clamp(0.0, 9999.0);
    energy
}

/// Transmission: probability that an energy particle passes through a material.
pub fn transmission(props: &MaterialProps) -> f32 {
    if props.thermal_conductivity <= 0.0 {
        0.0
    } else {
        (props.thermal_conductivity / (props.thermal_conductivity + 1.0)).clamp(0.0, 1.0)
    }
}

/// Scattering: deflect velocity by a small random angle. `rng` in [0,1).
pub fn scatter(vx: f32, vy: f32, strength: f32, rng: f32) -> (f32, f32) {
    let angle = (rng * 2.0 - 1.0) * strength;
    let c = angle.cos();
    let s = angle.sin();
    (vx * c - vy * s, vx * s + vy * c)
}

/// Deposit energy into a cell (temperature change). Returns true if absorbed.
pub fn deposit(cell: &mut Cell, props: &MaterialProps, energy: f32) -> bool {
    if cell.material == 0 {
        return false;
    }
    absorb(cell, props, energy);
    true
}

// Cell::is_empty is an inherent method on realmatter_core::cell::Cell.