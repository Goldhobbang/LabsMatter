# Request REQ-A8-002 — Persistence API mismatch with Core

- Requested by: A8 (Integration & Verification Engineer)
- Date: 2026-11-04
- Priority: HIGH
- Target: A2 (Core) with coordination to A7 (Persistence)
- Status: PENDING
- References: T-008-A8-20261104-0345.md, A3 report T-003-A3-20261104-0248.md

## Summary

`realmatter-persist` is written against the old 3D architecture and does not match current core API (2D). Workspace build blocked by 35 errors in persist.

## Root cause
- Persist expects: `realmatter_core::chunk`, `realmatter_core::coord`, `realmatter_core::particle`, `realmatter_core::material::Material` (struct), `realmatter_core::material::Phase`, `realmatter_core::Persistence` trait, `realmatter_core::SimError`/`SimResult`, `realmatter_core::World` with `chunks()`, `cell()`, `cell_mut()`, `tick_count`, `time_accumulator`.
- Current core (2D, approved): has `Cell { material, temperature, vx, vy, flags, lifetime, pressure }`, `MaterialProps`/`MaterialState`/`MaterialRegistry`, `WorldConfig`, `World { config, materials: Arc<dyn MaterialRegistry> }`, `RealmatterError`/`Result`, traits `SimulationModule/PhysicsModule/ChemistryModule/RenderSink/SerializationModule`. No chunk/coord/particle modules.

## File-specific correction instructions

**For A2 (core) — minimal compatibility if approved:**
1. `crates/realmatter-core/src/lib.rs`: Consider re-exporting `Persistence` trait if persist needs it, or add `SimError`/`SimResult` aliases. Current: no `Persistence`.
2. `crates/realmatter-core/src/error.rs`: Add `pub type SimError = RealmatterError; pub type SimResult<T> = Result<T, SimError>;` if needed.
3. `crates/realmatter-core/src/material.rs`: Add `pub type Material = MaterialProps; pub type Phase = MaterialState;` if needed.
4. `crates/realmatter-core/src/traits.rs`: Add `Persistence` trait if needed: `fn save(&self, world: &World) -> Result<Vec<u8>>; fn load(&self, bytes: &[u8]) -> Result<World>;` — but requires World API.

**For A7 (persist) — recommended:**
Rewrite persist to use current 2D core API. Remove references to chunk/coord/particle modules. Use `Cell` directly. Adapt to `World { config, materials }`. This is cleaner than expanding core API.

## Recommendation
A7 should update persist crate to match approved 2D contracts. A2 provides any minimal aliases only if explicitly approved by A1/A0.

## Evidence
See `communication/agents/A8/reports/T-008-A8-20261104-0345.md`
