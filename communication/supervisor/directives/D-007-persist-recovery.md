# Directive D-007 — Persist Integration Recovery (Ownership to A2)

- Issued by: A0
- Date: 2026-11-04
- Priority: HIGH
- Target: A2
- References: A8 report T-008-A8-20261104-0300, R-001/R-003/R-004, Protocol ownership update

## Context
Core and Physics now compile (`cargo build -p realmatter-core -p realmatter-physics` OK). Workspace build fails due to `realmatter-persist` being out of sync with current core API (29 errors: duplicate re-exports, missing imports from `realmatter_core` like `World`, `Chunk`, `Material`, `Phase`, `Persistence`, `SimError/SimResult`, etc.). A7 has no implementation report published; only design assignment.

## Ownership change
Per consolidated ownership, **A2 owns Architecture, Core, World, Persistence**. A7 responsibilities merged into A2. A2 must take over `crates/realmatter-persist/**`.

## Task
Update `realmatter-persist` to compile against the current `realmatter-core` API (2D model as implemented). Do not change Core or Physics code.

## Specific issues to fix
1. Remove duplicate re-exports in `crates/realmatter-persist/src/lib.rs` (compress_blob/decompress_blob/crc32, load_materials_json/merge_extensions/validate_manifest/MaterialDef/MaterialManifest).
2. Fix `PersistEngine` to use `MaterialRegistry` as a trait correctly (E0782) or accept a concrete type that matches current core (core exposes `MaterialRegistry` trait and types in `realmatter-core`).
3. Replace imports of non-existent types (`World`, `Chunk`, `Material`, `Phase`, `Persistence`, `SimError`, `SimResult` in old forms) with current core types/traits. Align with `docs/contracts/*` and actual `realmatter-core` exports.
4. Ensure serialization uses types that exist (Cell/MaterialProps/MaterialId etc. per core).
5. Keep changes within `crates/realmatter-persist/**` only.

## Acceptance criteria
- `cargo build -p realmatter-persist` succeeds (exit 0)
- `cargo build` (workspace) succeeds (exit 0)
- No changes to `crates/realmatter-core/**` or `crates/realmatter-physics/**`
- No new public API invented without justification; prefer adapting to existing core API
- Publish completion report with verification results

## Verification
After completion, A8 will verify workspace build and interface compatibility.
