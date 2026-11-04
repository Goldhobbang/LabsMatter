# Request REQ-A8-001 — Workspace does not build (realmatter-core)

- Requested by: A8 (Integration & Verification Engineer)
- Date: 2026-11-04
- Priority: CRITICAL
- Target: A0 (supervisor) / A1 (contracts) / A2 (core)
- Status: PENDING

## Summary

`cargo build` (workspace) fails. `realmatter-core` does not compile due to:
1. Circular type aliases in `error.rs`: `SimResult<T> = Result<T, SimError>` and
   `Result<T> = SimResult<T>`.
2. Duplicate definitions in `material.rs`: `MaterialState` and `MaterialProps`
   redefined as type aliases to non-existent `Material`/`Phase`.
3. `lib.rs` re-exports `Material`, `Phase`, `World` which are not defined in
   their modules.
4. Dimensionality drift: lib.rs declares 3D modules (chunk, coord, particle) while
   material.rs/error.rs/world.rs mix 2D and 3D definitions; approved contracts
   (docs/contracts/*) are 2D.

## Impact

All downstream crates (physics, persist, chemistry, wasm) fail to compile
transitively. No tests or benchmarks can be created. Integration verification
blocked.

## Requested action

A1/A2 reconcile core to the approved 2D contracts (or publish an ADR switching
to 3D with a complete migration), fix circular aliases and duplicate defs, and
republish T-001. A0 to re-review before unblocking downstream agents.

## Evidence

See `/communication/agents/A8/reports/T-008-A8-20261104-0245.md`.
