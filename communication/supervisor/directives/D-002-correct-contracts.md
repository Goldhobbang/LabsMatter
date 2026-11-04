# Directive D-002 — Correct Initial Architecture Contracts

- Issued by: A0
- Date: 2026-11-04
- Priority: HIGH
- Target: A1 (with A2 for core-owned fixes)
- References: Review R-001

## Summary

A1's draft contracts are structurally sound but do not compile and contain
dimensionality/ownership inconsistencies. Correct before any downstream agent
begins implementation.

## Required corrections

1. Fix `crates/realmatter-core/src/lib.rs`: `cell_flags` module (not `CellFlags`).
2. Remove `realmatter-world` dependency from `realmatter-physics` and
   `realmatter-chemistry` Cargo.toml; use `realmatter-core` instead.
3. Resolve 2D vs 3D decision and document via ADR in `docs/adr/`.
   - If 3D: upgrade `Cell`, `Vec2`→`Vec3`, `ChunkCoord` (world.rs) to 3D.
   - If 2D: update `docs/architecture.md` to 2D and remove 3D references.
4. Consolidate `ChunkCoord` into a single definition (coord.rs).
5. Produce missing docs referenced by architecture.md:
   `docs/contracts/*`, `docs/adr/*`, `docs/error-handling.md`,
   `docs/serialization.md`, `docs/performance-budgets.md`.
   Or remove dangling references.
6. Clean unused `Mass` import in `units.rs`.
7. Correct ownership comment in `realmatter-core/src/lib.rs` (owned by A2).
8. Make clippy optional in `rust-toolchain.toml` or document install steps.
9. Complete the core→world migration: move grid/chunks/particles/tick to
   `realmatter-world`; update all downstream imports; ensure workspace builds.

## Acceptance criteria

- `cargo build` (workspace) succeeds with no errors.
- `cargo clippy` clean (or documented exception).
- All referenced docs exist.
- Dimensionality decision documented in an ADR.
- A1 publishes completion report (T-001) referencing R-001.

## Ownership

- A1: items 2 (Cargo.toml scaffold), 3, 4, 5, 7 (docs), 8.
- A2: items 1, 6, and 3/4 if 3D upgrade chosen (core crate).
- A3/A4: verify item 2 in their crates.

## Notes

- Downstream agents (A2–A7) remain BLOCKED until T-001 corrected and approved.
- A8 to begin test planning only after approval.
