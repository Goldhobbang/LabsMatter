//! Gas movement, diffusion, pressure propagation, expansion, density-driven movement.

use crate::core;
use crate::core::cell::Cell;
use crate::core::cell::cell_flags;
use crate::core::material::MaterialProps;

/// Update pressure field using ideal gas law approximation.
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

/// Diffuse heat in gases.
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
            let coeff = (d * dt / dx2).min(0.25);
            if coeff <= 0.0 {
                continue;
            }
            let mut lap = 0.0f32;
            if x > 0 && !cells[i - 1].is_empty() { lap += temps[i - 1] - temps[i]; }
            if x + 1 < w && !cells[i + 1].is_empty() { lap += temps[i + 1] - temps[i]; }
            if y > 0 && !cells[i - w].is_empty() { lap += temps[i - w] - temps[i]; }
            if y + 1 < h && !cells[i + w].is_empty() { lap += temps[i + w] - temps[i]; }
            new_temps[i] = temps[i] + coeff * lap;
        }
    }
    for (cell, t) in cells.iter_mut().zip(new_temps.iter()) {
        if !cell.is_empty() {
            cell.temperature = *t;
        }
    }
}

/// Gas expansion and density-driven movement.
///
/// `active` optionally restricts source-cell processing to active chunks.
/// Reads of neighbor cells are unaffected (always permitted); writes into
/// inactive chunks are allowed (a gas cell may diffuse into an empty chunk —
/// the next mask rebuild picks it up). Skipping an empty source cell in an
/// inactive chunk is a no-op, so results are bit-identical to unmasked.
///
/// This entry point keeps the historical signature and assumes **standard
/// gravity**, delegating to [`diffuse_with_gravity`]. The tick pipeline should
/// call the gravity-aware variant so that a world with gravity switched off
/// stops buoyant gas transport (see the reference note on that function).
pub fn diffuse(
    cells: &mut [Cell],
    width: u32,
    height: u32,
    materials: &[MaterialProps],
    dt: f32,
    active: Option<&crate::core::chunk_mask::ActiveMask>,
) {
    diffuse_with_gravity(
        cells,
        width,
        height,
        materials,
        dt,
        core::units::constants::GRAVITY,
        active,
    );
}

/// Fraction of standard gravity below which a gas is treated as weightless.
///
/// The reference has no partial-gravity gas lift either: `GetGravityField`
/// returns a zero force for `GRAV_OFF` (`Simulation.cpp:2101-2105`), and a gas
/// only rises through `Gravity * particleGrav`
/// (`Simulation.cpp:2290` feeding `parts[i].vy += ... + neighbourhood.pGravY`
/// at `Simulation.cpp:2370`). One tenth of standard gravity leaves head-room
/// for a future partial-gravity mode while keeping the on/off contract exact.
pub const GAS_RISE_MIN_GRAVITY: f32 = core::units::constants::GRAVITY * 0.1;

/// Gravity-aware gas expansion and buoyant movement.
///
/// `gravity` is the vertical acceleration magnitude the caller integrates
/// (0.0 = gravity off). It gates the buoyant rise: a gas only moves upward
/// when gravity is present, matching the reference, where smoke rises because
/// its element `Gravity` is negative and `GetGravityField` multiplies it by
/// the gravity mode — with `GRAV_OFF` the product is zero, so smoke does not
/// rise. With no gravity the gases still diffuse sideways but never move
/// vertically.
///
/// Same `active` contract as [`diffuse`].
pub fn diffuse_with_gravity(
    cells: &mut [Cell],
    width: u32,
    height: u32,
    materials: &[MaterialProps],
    dt: f32,
    gravity: f32,
    active: Option<&crate::core::chunk_mask::ActiveMask>,
) {
    diffuse_temperature(cells, width, height, materials, dt);
    let w = width as usize;
    let h = height as usize;
    if cells.len() != w * h {
        return;
    }
    // Rise, then spread. The previous pass moved every gas cell into the
    // first empty neighbor in left, right, up, down order, so smoke and steam
    // marched to the left edge and oscillated there. Buoyancy is upward
    // (decreasing y, which is up on screen) and only exists while gravity
    // does. Sideways order alternates with position so a plume spreads
    // without an RNG.
    let buoyant = gravity.abs() >= GAS_RISE_MIN_GRAVITY;
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if let Some(m) = active {
                if !m.cell_active(x, y) {
                    continue;
                }
            }
            if cells[i].is_empty()
                || cells[i].has_flag(cell_flags::FIXED)
                || cells[i].has_flag(cell_flags::UPDATED)
            {
                continue;
            }
            let m = material_or_default(materials, cells[i].material);
            if m.state != core::material::MaterialState::Gas {
                continue;
            }
            let order: &[(i32, i32)] = if buoyant {
                if (x + y) & 1 == 0 {
                    &[(0, -1), (-1, -1), (1, -1), (-1, 0), (1, 0)]
                } else {
                    &[(0, -1), (1, -1), (-1, -1), (1, 0), (-1, 0)]
                }
            } else if (x + y) & 1 == 0 {
                // Gravity off: no buoyant lift, so vertical neighbours are
                // excluded outright; gases only spread sideways.
                &[(-1, 0), (1, 0)]
            } else {
                &[(1, 0), (-1, 0)]
            };
            for &(dx, dy) in order {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || ny < 0 || (nx as usize) >= w || (ny as usize) >= h {
                    continue;
                }
                let ni = ny as usize * w + nx as usize;
                if cells[ni].is_empty() && !cells[ni].has_flag(cell_flags::UPDATED) {
                    cells.swap(i, ni);
                    cells[ni].set_flag(cell_flags::UPDATED, true);
                    break;
                }
            }
        }
    }
}

/// Gas buoyancy and density-driven movement.
///
/// `active` optionally restricts source cells to active chunks (same safety
/// contract as [`diffuse`]).
pub fn apply_buoyancy(
    cells: &mut [Cell],
    width: u32,
    height: u32,
    materials: &[MaterialProps],
    dt: f32,
    gravity: f32,
    active: Option<&crate::core::chunk_mask::ActiveMask>,
) {
    let w = width as usize;
    let h = height as usize;
    if cells.len() != w * h {
        return;
    }
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if let Some(m) = active {
                if !m.cell_active(x, y) {
                    continue;
                }
            }
            if cells[i].is_empty() || cells[i].has_flag(cell_flags::FIXED) {
                continue;
            }
            let m = material_or_default(materials, cells[i].material);
            if m.state != core::material::MaterialState::Gas {
                continue;
            }
            // Hotter/lower density rises
            let temp_factor = (cells[i].temperature - 293.15) / 400.0;
            cells[i].vy += -gravity * temp_factor.max(-0.5) * dt * 0.8;
            // Pressure expansion
            if cells[i].pressure > core::units::constants::ATM_PRESSURE * 1.1 {
                cells[i].vy += -gravity * 0.1 * dt;
            }
        }
    }
}

#[inline]
fn material_or_default(
    materials: &[MaterialProps],
    id: core::material::MaterialId,
) -> &MaterialProps {
    core::material::material_ref(materials, id)
}
