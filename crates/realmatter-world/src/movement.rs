//! Deterministic cellular-automaton movement (A2).
//!
//! Phase 5 of the tick order (`simulation-tick.md`): integrate velocities,
//! move cells, resolve occupancy. Rules:
//!
//! - Row-major iteration, bottom-up for falling matter (so a cell moves at
//!   most once per tick; `UPDATED` flag enforces this).
//! - Powder (sand-like): try down, then down-left / down-right (deterministic
//!   order: straight first, then left, then right).
//! - Liquid: try down, then down-left / down-right, then left / right.
//! - Gas: handled by A3 (diffusion/buoyancy); not moved here.
//! - Fixed (`FIXED` flag) and empty cells never move.
//! - Occupancy: a move succeeds only into an empty cell; swaps are resolved
//!   by density (heavier sinks) when both cells are mobile.
//! - No RNG: tie-breaking is positional (left before right), so identical
//!   inputs produce identical outputs bitwise.

use realmatter_core::cell::{cell_flags, Cell};
use realmatter_core::material::{MaterialId, MaterialState};

/// Per-step movement statistics (for diagnostics / A8 scenarios).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MovementStats {
    pub moved: u32,
    pub swapped: u32,
    pub blocked: u32,
}

/// Material state lookup used by movement. Returns `None` for unknown ids
/// (treated as immobile solid).
pub type StateOf = dyn Fn(MaterialId) -> Option<MaterialState>;

#[inline]
fn is_mobile(cell: &Cell) -> bool {
    !cell.is_empty() && !cell.has_flag(cell_flags::FIXED) && !cell.has_flag(cell_flags::UPDATED)
}

#[inline]
fn can_enter(cell: &Cell) -> bool {
    cell.is_empty()
}

/// Attempt to move the cell at `i` into `j`. Returns true on success.
fn try_move(cells: &mut [Cell], i: usize, j: usize, stats: &mut MovementStats) -> bool {
    if cells[j].is_empty() {
        cells.swap(i, j);
        cells[j].set_flag(cell_flags::UPDATED, true);
        stats.moved += 1;
        true
    } else {
        stats.blocked += 1;
        false
    }
}

/// Density-driven swap: heavier cell sinks through lighter mobile cell.
/// Solids (wood, iron, ice, glass, wall) are structures, not fluids, so a
/// denser powder or liquid must not displace them.
fn try_sink_swap(
    cells: &mut [Cell],
    i: usize,
    j: usize,
    state_of: &StateOf,
    density_of: &dyn Fn(MaterialId) -> f32,
    stats: &mut MovementStats,
) -> bool {
    if cells[j].is_empty() || cells[j].has_flag(cell_flags::FIXED) {
        return false;
    }
    if !matches!(
        state_of(cells[j].material),
        Some(MaterialState::Powder | MaterialState::Liquid | MaterialState::Gas | MaterialState::Plasma)
    ) {
        return false;
    }
    let di = density_of(cells[i].material);
    let dj = density_of(cells[j].material);
    if di > dj && !cells[j].has_flag(cell_flags::UPDATED) {
        cells.swap(i, j);
        cells[j].set_flag(cell_flags::UPDATED, true);
        cells[i].set_flag(cell_flags::UPDATED, true);
        stats.swapped += 1;
        true
    } else {
        stats.blocked += 1;
        false
    }
}

/// One powder step over a row-major `cells` slice (`width x height`).
/// Bottom-up row order; within a row, x ascending.
///
/// `active` optionally restricts source cells to active chunks. A powder
/// cell may fall into an inactive (empty) chunk — the write is always
/// allowed and the next mask rebuild wakes that chunk. Skipping empty
/// sources is result-identical.
pub fn move_powder_step(
    cells: &mut [Cell],
    width: u32,
    height: u32,
    state_of: &StateOf,
    density_of: &dyn Fn(MaterialId) -> f32,
    active: Option<&realmatter_core::chunk_mask::ActiveMask>,
) -> MovementStats {
    let mut stats = MovementStats::default();
    let (w, h) = (width as usize, height as usize);
    if cells.len() != w * h || w == 0 || h == 0 {
        return stats;
    }
    for y in (0..h).rev() {
        for x in 0..w {
            let i = y * w + x;
            if let Some(m) = active {
                if !m.cell_active(x, y) {
                    continue;
                }
            }
            if !is_mobile(&cells[i]) {
                continue;
            }
            let is_powder = matches!(state_of(cells[i].material), Some(MaterialState::Powder));
            if !is_powder {
                continue;
            }
            // Down.
            if y + 1 < h {
                let j = (y + 1) * w + x;
                if can_enter(&cells[j]) {
                    try_move(cells, i, j, &mut stats);
                    continue;
                }
                if try_sink_swap(cells, i, j, state_of, density_of, &mut stats) {
                    continue;
                }
                // Down-left, then down-right (deterministic order).
                let mut done = false;
                for dx in [-1isize, 1] {
                    let nx = x as isize + dx;
                    if nx >= 0 && (nx as usize) < w {
                        let j = (y + 1) * w + nx as usize;
                        if can_enter(&cells[j]) {
                            try_move(cells, i, j, &mut stats);
                            done = true;
                            break;
                        }
                    }
                }
                if !done {
                    stats.blocked += 1;
                }
            } else {
                stats.blocked += 1;
            }
        }
    }
    stats
}

/// One liquid step over a row-major `cells` slice. Falls like powder, then
/// spreads sideways (left before right, deterministic).
///
/// `active` has the same contract as [`move_powder_step`].
pub fn move_liquid_step(
    cells: &mut [Cell],
    width: u32,
    height: u32,
    state_of: &StateOf,
    density_of: &dyn Fn(MaterialId) -> f32,
    active: Option<&realmatter_core::chunk_mask::ActiveMask>,
) -> MovementStats {
    let mut stats = MovementStats::default();
    let (w, h) = (width as usize, height as usize);
    if cells.len() != w * h || w == 0 || h == 0 {
        return stats;
    }
    for y in (0..h).rev() {
        for x in 0..w {
            let i = y * w + x;
            if let Some(m) = active {
                if !m.cell_active(x, y) {
                    continue;
                }
            }
            if !is_mobile(&cells[i]) {
                continue;
            }
            if !matches!(state_of(cells[i].material), Some(MaterialState::Liquid)) {
                continue;
            }
            // Fall first (same as powder).
            let mut moved = false;
            if y + 1 < h {
                let j = (y + 1) * w + x;
                if can_enter(&cells[j]) {
                    try_move(cells, i, j, &mut stats);
                    continue;
                }
                if try_sink_swap(cells, i, j, state_of, density_of, &mut stats) {
                    continue;
                }
                for dx in [-1isize, 1] {
                    let nx = x as isize + dx;
                    if nx >= 0 && (nx as usize) < w {
                        let j = (y + 1) * w + nx as usize;
                        if can_enter(&cells[j]) {
                            try_move(cells, i, j, &mut stats);
                            moved = true;
                            break;
                        }
                    }
                }
                if moved {
                    continue;
                }
            }
            // Spread sideways: left, then right.
            for dx in [-1isize, 1] {
                let nx = x as isize + dx;
                if nx >= 0 && (nx as usize) < w {
                    let j = y * w + nx as usize;
                    if can_enter(&cells[j]) {
                        try_move(cells, i, j, &mut stats);
                        moved = true;
                        break;
                    }
                }
            }
            if !moved {
                stats.blocked += 1;
            }
        }
    }
    stats
}
