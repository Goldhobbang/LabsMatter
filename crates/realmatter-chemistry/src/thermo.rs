//! Thermochemistry helpers for chemistry.

use crate::core::cell::Cell;
use crate::core::material::MaterialProps;
use crate::kinetics::temperature_rise_from_enthalpy;

/// Apply heat release to a cell.
pub fn apply_heat_release(cell: &mut Cell, enthalpy_j: f32, mass_kg: f32, props: &MaterialProps) {
    if mass_kg <= 0.0 {
        return;
    }
    let d_t = temperature_rise_from_enthalpy(enthalpy_j, mass_kg, props.specific_heat);
    cell.temperature += d_t;
}

/// Compute cell mass (2D).
pub fn cell_mass(props: &MaterialProps) -> f32 {
    props.density * (0.01f32 * 0.01f32 * 0.01f32)
}
