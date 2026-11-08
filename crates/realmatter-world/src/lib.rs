//! REALMATTER world: grid, chunks, particles, tick scheduler.
//!
//! Owned by A2 (Simulation Core). Implements the approved 2D contracts:
//! `docs/contracts/world-grid.md`, `chunk-layout.md`, `simulation-tick.md`.

pub mod grid;
pub mod movement;
pub mod tick;

pub use realmatter_core as core;

pub use grid::World;
pub use movement::{
    move_liquid_flat, move_liquid_step, move_powder_step, move_toward_center, MovementStats,
};
pub use tick::{TickScheduler, TickStats};
