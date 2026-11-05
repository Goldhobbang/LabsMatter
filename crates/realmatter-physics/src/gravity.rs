//! Newtonian gravity field.
//!
//! The reference solves the gravitational potential with an FFT convolution of
//! the mass grid against a fixed kernel. This engine solves the discrete
//! Poisson equation ∇²φ = −4πGρ on the same `CELL`-sized grid with a bounded
//! Jacobi relaxation, then takes the field as −∇φ. That is the same equation
//! the reference kernel approximates, so the result is distance-dependent,
//! mass-dependent and points at mass — but it is *not* bit-identical to the
//! FFT kernel (documented deviation, see `docs/tpt-parity/status.md`).

use realmatter_core::cell::Cell;
use realmatter_core::material::MaterialProps;

/// Air-cell size in simulation pixels (must match `air::CELL`).
pub const CELL: usize = 4;
/// Jacobi relaxation iterations per update. Bounded for a stable frame budget.
pub const SOLVER_ITERATIONS: usize = 24;
/// Force scale applied to the solved gradient.
pub const FORCE_SCALE: f32 = 0.02;

/// Solve the field from a mass grid and return the force per air cell.
///
/// `mass[cy * cells_x + cx]` is the total particle mass in that air cell.
/// Boundary cells are held at zero potential (Dirichlet), which matches the
/// reference's zero-force border behaviour.
pub fn solve_field(mass: &[f32], cells_x: usize, cells_y: usize) -> Vec<(f32, f32)> {
    let n = cells_x * cells_y;
    let mut phi = vec![0.0f32; n];
    let mut next = vec![0.0f32; n];
    let boundary = |x: usize, y: usize| x == 0 || y == 0 || x == cells_x - 1 || y == cells_y - 1;

    for _ in 0..SOLVER_ITERATIONS {
        for y in 1..cells_y - 1 {
            for x in 1..cells_x - 1 {
                let idx = y * cells_x + x;
                let lap = phi[idx - 1] + phi[idx + 1] + phi[idx - cells_x] + phi[idx + cells_x];
                next[idx] = (lap + mass[idx]) * 0.25;
            }
        }
        for i in 0..n {
            let (x, y) = (i % cells_x, i / cells_x);
            if boundary(x, y) {
                next[i] = 0.0;
            }
        }
        std::mem::swap(&mut phi, &mut next);
    }

    let mut force = vec![(0.0f32, 0.0f32); n];
    for y in 1..cells_y - 1 {
        for x in 1..cells_x - 1 {
            let idx = y * cells_x + x;
            let gx = (phi[idx + 1] - phi[idx - 1]) * 0.5 * FORCE_SCALE;
            let gy = (phi[idx + cells_x] - phi[idx - cells_x]) * 0.5 * FORCE_SCALE;
            force[idx] = (gx, gy);
        }
    }
    force
}

/// Build the mass grid from the particle grid and solve the field.
pub fn compute_field(
    cells: &[Cell],
    width: usize,
    height: usize,
    materials: &[MaterialProps],
) -> Vec<(f32, f32)> {
    let cx = (width / CELL).max(1);
    let cy = (height / CELL).max(1);
    let mut mass = vec![0.0f32; cx * cy];
    for y in 0..height.min(cy * CELL) {
        for x in 0..width.min(cx * CELL) {
            let cell = &cells[y * width + x];
            if cell.material == 0 {
                continue;
            }
            let d = materials
                .get(cell.material as usize)
                .map(|m| m.density)
                .unwrap_or(0.0);
            if d <= 0.0 {
                continue;
            }
            mass[(y / CELL) * cx + x / CELL] += d * 1.0e-6;
        }
    }
    solve_field(&mass, cx, cy)
}

/// Apply the field to mobile particles: velocity += force, then move one cell
/// along the strongest component. Powders and liquids only.
pub fn apply_field(
    cells: &mut [Cell],
    width: usize,
    height: usize,
    materials: &[MaterialProps],
    force: &[(f32, f32)],
) {
    let cx = (width / CELL).max(1);
    for y in (0..height).rev() {
        for x in 0..width {
            let i = y * width + x;
            let cell = &mut cells[i];
            if cell.material == 0 || cell.has_flag(realmatter_core::cell::cell_flags::FIXED) {
                continue;
            }
            let state = match materials.get(cell.material as usize) {
                Some(d) => d.state,
                None => continue,
            };
            if !matches!(
                state,
                realmatter_core::material::MaterialState::Powder
                    | realmatter_core::material::MaterialState::Liquid
            ) {
                continue;
            }
            let f = force[(y / CELL) * cx + (x / CELL)];
            cell.vx = (cell.vx + f.0).clamp(-4.0, 4.0);
            cell.vy = (cell.vy + f.1).clamp(-4.0, 4.0);
            let dx = cell.vx.round().clamp(-2.0, 2.0) as i32;
            let dy = cell.vy.round().clamp(-2.0, 2.0) as i32;
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                continue;
            }
            let j = ny as usize * width + nx as usize;
            if cells[j].is_empty() {
                cells.swap(i, j);
                cells[j].set_flag(realmatter_core::cell::cell_flags::UPDATED, true);
            }
        }
    }
}
