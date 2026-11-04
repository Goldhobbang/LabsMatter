# Request REQ-A2-001 — Persist rewrite against stabilized 2D core/world API

- Requested by: A2 (Core Systems & Architecture Lead)
- Date: 2026-11-04
- Priority: HIGH
- Target: A0 (supervisor) / A7 (Persistence)
- Status: PENDING
- References: D-002, D-003, D-005, REQ-A8-001, docs/contracts/*, ADR-0003

## Summary

A2 has completed the 2D core→world migration in A2-owned paths. `cargo check`
passes for all crates except `realmatter-persist` (A7-owned), which still
imports a pre-contract API that no longer exists. A7 must rewrite
`realmatter-persist` against the stabilized API below. A2 cannot modify A7
files (protocol rule 2/15).

## Stabilized A2 API (do not change without A0 approval)

Core (`realmatter-core`):
- `cell::{Cell, cell_flags}` — 2D cell (material, temperature, vx, vy, flags,
  lifetime, pressure); flags BURNING/WET/FIXED/UPDATED/CHARGED/DISSOLVED.
- `coord::{CellCoord {x,y}, ChunkCoord {x,y}, CHUNK_SIZE = 32}` — single
  canonical definitions (2D).
- `chunk::{Chunk {coord, cells: Vec<Cell>, dirty}, CHUNK_EDGE = 32, CHUNK_VOLUME}`.
- `material::{MaterialId, MaterialProps, MaterialState, MaterialRegistry trait,
  MATERIAL_EMPTY, MATERIAL_WALL}` — registry is a trait; no `Material` struct,
  no `Phase` type, no `MaterialRegistry::new`.
- `error::{RealmatterError, Result}` — single `Result<T>` alias; no
  `SimError`/`SimResult`/`Persistence` exports.
- `world::WorldConfig {width, height, chunk_size, fixed_dt}` + `validate()`,
  `chunks_x/y()`, defaults 512x384/32/1-60. No `World` type in core.
- `particle` module is deprecated legacy (will be removed); do not use.
- `traits::{SimulationModule, PhysicsModule, ChemistryModule, RenderSink,
  SerializationModule}` — unchanged per contracts.

World (`realmatter-world`):
- `grid::World::new(config, Arc<dyn MaterialRegistry>) -> Result<World>`.
- Accessors: `config()`, `materials()`, `tick_count()`, `width()`,
  `height()`, `cells()`, `cells_mut()`, `cell(x,y)`, `cell_mut(x,y)`,
  `get(coord) -> Result<&Cell>`, `set(coord, cell) -> Result<()>`.
- Dirty tracking: `mark_dirty(x,y)`, `mark_rect_dirty(...)`,
  `take_dirty_chunks() -> Vec<ChunkCoord>`, `dirty_chunks()`.
- Lifecycle: `finalize_tick()` (clears UPDATED, ages lifetimes),
  `accumulate(elapsed) -> u32` (max 4 substeps).
- `movement::{move_powder_step, move_liquid_step, MovementStats}`.
- `tick::{TickScheduler, Phase, TickStats}`.

## Required A7 changes (owned by A7)

1. Remove duplicate re-exports in `persist/src/lib.rs:153-154` (E0252).
2. Replace imports of `Material`, `Phase`, `Persistence`, `SimError`,
   `SimResult`, `World` (from core), `chunk::`, `coord::`, `particle::`
   with the API above (`RealmatterError`/`Result`, `MaterialProps`,
   `MaterialRegistry` trait, `realmatter_world::World`, `ChunkCoord`).
3. Rewrite save format against `World::cells()` + `WorldConfig`
   (see `docs/serialization.md`); remove `MaterialRegistry::new(...)`,
   `world.materials.iter()`, `cell.particle`, `CHUNK_VOLUME` array
   assumptions, and the `#[cfg(test)]` module (protocol rule 11: no test
   files; document scenarios for A8 instead).
4. Accept `Arc<dyn MaterialRegistry>` for material storage.
5. Verify: `cargo check --workspace` and `cargo build --workspace` clean.

## Acceptance

- `cargo check --workspace` exits 0 (no errors).
- `cargo build --workspace` exits 0.
- No files outside `crates/realmatter-persist/**` modified by A7.

## Blockers

- None for A7; API above is final pending A0 approval.
