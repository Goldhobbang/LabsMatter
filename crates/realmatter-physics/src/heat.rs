//! Heat-driven phase transitions and enhanced diffusion.

use crate::core;
use crate::core::cell::Cell;
use crate::core::cell::cell_flags;
use crate::core::material::MaterialProps;

/// Symmetric heat conduction with energy conservation.
///
/// Each neighboring pair is visited exactly once (right and down neighbors
/// only, row-major order). The exchanged energy `dE` is removed from one
/// cell and added to the other, weighted by each cell's heat capacity, so
/// total thermal energy is conserved up to floating-point rounding.
/// Overshoot is prevented by clamping the exchange to half the pair's
/// temperature gap. Non-finite or negative temperatures are sanitized to
/// ambient before exchange.
///
/// `active` optionally restricts source cells to active chunks (see
/// [`diffuse`](crate::gas::diffuse) for the safety argument). Empty cells
/// never conduct, so skipping them is result-identical.
///
/// Returns the total exchanged energy (J) for diagnostics.
pub fn conduct(
    cells: &mut [Cell],
    width: u32,
    height: u32,
    materials: &[MaterialProps],
    dt: f32,
    active: Option<&crate::core::chunk_mask::ActiveMask>,
) -> f32 {
    let w = width as usize;
    let h = height as usize;
    if cells.len() != w * h {
        return 0.0;
    }
    const DX: f32 = 0.01; // cell size (m)
    let mut energy = 0.0f32;
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if let Some(m) = active {
                if !m.cell_active(x, y) {
                    continue;
                }
            }
            if cells[i].is_empty() {
                continue;
            }
            // Right and down neighbors only: each pair exactly once.
            for (dx, dy) in [(1usize, 0usize), (0, 1)] {
                let nx = x + dx;
                let ny = y + dy;
                if nx >= w || ny >= h {
                    continue;
                }
                let ni = ny * w + nx;
                if cells[ni].is_empty() {
                    continue;
                }
                let ma = material_or_default(materials, cells[i].material);
                let mb = material_or_default(materials, cells[ni].material);
                if ma.thermal_conductivity <= 0.0 && mb.thermal_conductivity <= 0.0 {
                    continue;
                }
                // Harmonic mean: the poorer conductor limits the pair.
                let ka = ma.thermal_conductivity.max(0.0);
                let kb = mb.thermal_conductivity.max(0.0);
                let k_eff = if ka + kb > 0.0 {
                    2.0 * ka * kb / (ka + kb)
                } else {
                    0.0
                };
                if k_eff <= 0.0 {
                    continue;
                }
                let mut ta = cells[i].temperature;
                let mut tb = cells[ni].temperature;
                if !ta.is_finite() {
                    ta = 293.15;
                }
                if !tb.is_finite() {
                    tb = 293.15;
                }
                ta = ta.max(0.0);
                tb = tb.max(0.0);
                let gap = tb - ta;
                if gap == 0.0 {
                    continue;
                }
                let capa = ma.density.max(1e-6) * ma.specific_heat.max(1e-6) * (DX * DX * DX);
                let capb = mb.density.max(1e-6) * mb.specific_heat.max(1e-6) * (DX * DX * DX);
                if capa <= 0.0 || capb <= 0.0 {
                    continue;
                }
                // Fourier: dE = k_eff * gap / DX * (DX*DX) * dt.
                let mut de = k_eff * gap / DX * (DX * DX) * dt;
                // Stability: never move more than a quarter of the gap's
                // energy equivalent (prevents oscillation/overshoot).
                let max_de = 0.25 * gap.abs() * (capa * capb / (capa + capb));
                if de.abs() > max_de {
                    de = max_de * de.signum();
                }
                cells[i].temperature = ta + de / capa;
                cells[ni].temperature = tb - de / capb;
                energy += de.abs();
            }
        }
    }
    energy
}

/// Melting, solidification, evaporation, condensation, boiling.
pub fn phase_transitions(cells: &mut [Cell], materials: &[MaterialProps]) {
    for cell in cells.iter_mut() {
        if cell.is_empty() {
            continue;
        }
        let m = material_or_default(materials, cell.material);
        // Melting: solid -> liquid
        if m.state == core::material::MaterialState::Solid
            && m.melting_point.is_finite()
            && cell.temperature >= m.melting_point
        {
            cell.temperature = m.melting_point;
            cell.set_flag(cell_flags::WET, true);
        }
        // Solidification: liquid -> solid
        if m.state == core::material::MaterialState::Liquid
            && m.melting_point.is_finite()
            && cell.temperature <= m.melting_point
        {
            cell.temperature = m.melting_point;
            cell.set_flag(cell_flags::WET, false);
        }
        // Boiling/Evaporation: liquid -> gas
        if m.state == core::material::MaterialState::Liquid
            && m.boiling_point.is_finite()
            && cell.temperature >= m.boiling_point
        {
            cell.temperature = m.boiling_point;
            // Mark for potential vaporization
        }
        // Condensation: gas -> liquid
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

#[inline]
fn material_or_default(
    materials: &[MaterialProps],
    id: core::material::MaterialId,
) -> &MaterialProps {
    core::material::material_ref(materials, id)
}
