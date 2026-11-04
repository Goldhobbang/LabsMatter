# Directive D-003 — A2 Narrowly Scoped Core Corrections

- Issued by: A0
- Date: 2026-11-04
- Priority: HIGH
- Target: A2
- References: Review R-001, Directive D-002 (items 1, 6, 9)

## Scope (A2-owned files only)

A1 must NOT modify these files. A2 owns them.

Files:
- `crates/realmatter-core/src/lib.rs`
- `crates/realmatter-core/src/units.rs`
- `crates/realmatter-core/src/coord.rs`
- `crates/realmatter-core/src/world.rs`
- `crates/realmatter-core/src/cell.rs`
- `crates/realmatter-world/**` (if created; A2 owns)

## Required corrections

1. Fix `lib.rs` re-export: `cell` module has `cell_flags` module, not `CellFlags`. Either export `cell_flags` or define `CellFlags`. Align with consumers.
2. Remove unused `Mass` import in `units.rs` (or use it).
3. Consolidate `ChunkCoord`: single definition. Decide whether `world::ChunkCoord` (2D) or `coord::ChunkCoord` (3D) is canonical; remove the other or alias. Must match dimensionality decision (D-002 item 3).
4. Correct ownership comment in `lib.rs` ("Owned by A2", not A1).
5. Complete core→world migration: ensure `realmatter-world` exports `Chunk`, `Material` (if moved), `World`, `MaterialRegistry`, `Phase` as needed; update `realmatter-core` re-exports accordingly. Coordinate with A1 on which types stay in core vs world, but A2 implements in owned crates.
6. Ensure `cargo build -p realmatter-core -p realmatter-world` succeeds.

## Acceptance criteria

- `cargo build -p realmatter-core -p realmatter-world` clean (no errors).
- No `unused` warnings in touched files.
- Single `ChunkCoord` definition.
- Ownership comment corrected.

## Notes

- Do NOT change public trait signatures without A1 approval (contract change).
- If a contract change is needed, file a request in `/communication/requests/pending/`.
- A1 coordinates overall migration; A2 implements in owned crates.
