# Persistence Blocker Resolution Plan

- Owner: A0
- Date: 2026-11-04
- Target: A2 (persistence owner)
- Evidence: cargo check -p realmatter-persist shows 10 errors

## Root causes (from diagnostics)

1. **Duplicate re-exports (E0255)**: `compress_blob`, `decompress_blob`, `crc32` defined multiple times (format.rs + lib.rs re-exports)
2. **Wrong crate dependency/imports (E0432)**: `realmatter_world` imported but not used/missing; core split means types moved
3. **Obsolete types (E0425)**: `Material`, `MaterialId` used but not imported from current core API
4. **Error conversion (E0277)**: `?` can't convert error to `String` (need proper error mapping)

## Minimal corrections (A2-owned files only)

### crates/realmatter-persist/src/format.rs
- Remove duplicate definitions/re-exports of compress_blob/decompress_blob/crc32
- Update imports to use core types only
- Fix error handling

### crates/realmatter-persist/src/lib.rs
- Remove duplicate re-exports
- Replace imports with canonical 2D core API:
  - `use realmatter_core::{Cell, cell_flags, coord::{CellCoord,ChunkCoord}, material::{MaterialId,MaterialProps,MaterialRegistry}, error::{RealmatterError,Result}};`
  - `use realmatter_world::World;` (world is separate crate now)
- Fix PersistEngine to use trait correctly
- Fix error conversions

### crates/realmatter-persist/src/material_ext.rs
- Update to use MaterialProps/MaterialRegistry trait
- Remove MaterialId/MaterialDef if obsolete or adapt

### crates/realmatter-persist/Cargo.toml
- Ensure deps: realmatter-core, realmatter-world, serde, bincode (no realmatter-physics/chemistry needed)

## Required evidence from A2
1. `cargo check -p realmatter-persist` exits 0
2. `cargo check --workspace` exits 0  
3. `cargo build --workspace` exits 0
4. Completion report with changed files and verification
5. Commit SHA if committed (A2 may not commit per protocol, note working tree)

## Next steps
- A2 executes fixes
- A8 verifies
- After green, proceed to REQ-A5-001 (WASM boundary)
