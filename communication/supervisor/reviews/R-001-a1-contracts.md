# Review R-001 — A1 Initial Architecture Contracts (Draft)

- Reviewer: A0
- Date: 2026-11-04
- Subject: A1 draft deliverables (Cargo workspace, realmatter-core scaffold, docs)
- Outcome: **CHANGES_REQUESTED**

## Scope reviewed

- `Cargo.toml` (workspace)
- `.cargo/config.toml`, `rust-toolchain.toml`
- `crates/realmatter-core/**`
- `crates/realmatter-physics/**`, `realmatter-chemistry/**`, `realmatter-renderer/**`, `realmatter-ui/**`, `realmatter-persist/**`, `realmatter-wasm/**`
- `docs/architecture.md`, `docs/module-dependency-graph.md`

## Verification performed

- `cargo build` (stable toolchain): **FAILED** — see errors below.
- Ownership boundary check: conflicts found (see below).
- Contract completeness: missing referenced docs.

## Build errors (must fix)

1. `crates/realmatter-core/src/lib.rs:14` — `pub use cell::{Cell, CellFlags};` but `cell` module defines `cell_flags` module, not `CellFlags`. Compile error E0432.
2. `crates/realmatter-physics/Cargo.toml` and `crates/realmatter-chemistry/Cargo.toml` depend on `realmatter-world = { path = "../realmatter-world" }`, but no such crate exists in workspace. Build cannot resolve.
3. `crates/realmatter-core/src/units.rs` — unused import `Mass` (warning; clean up).
4. `rust-toolchain.toml` requires `clippy` component; installation fails in this environment (clippy-preview conflict). Consider making clippy optional or document manual install. (Environment limitation; not blocking code.)

## Contract / consistency issues

5. **Dimensionality mismatch**: `docs/architecture.md` specifies a 3D world (256^3 cells, ChunkCoord {x,y,z}), but core crate is 2D: `world::ChunkCoord {x,y}`, `Cell {vx,vy}` (no vz), `math::Vec2`. Either update architecture to 2D or upgrade core to 3D. A0 recommends aligning to architecture (3D) OR explicitly deciding 2D mode and updating architecture. **A1 must resolve and document decision (ADR).**
6. **Duplicate ChunkCoord**: `coord::ChunkCoord` (3D) vs `world::ChunkCoord` (2D). Consolidate into one type.
7. **Missing contract docs**: architecture.md references `docs/contracts/*`, `docs/adr/*`, `docs/error-handling.md`, `docs/serialization.md`, `docs/performance-budgets.md`. None exist. A1 must produce these (or remove references).
8. **Ownership conflict (resolved by A0)**: `realmatter-core/src/lib.rs` comment says "Owned by A1", but `module-dependency-graph.md` assigns core to A2. Resolution: A2 owns `realmatter-core`; A1 owns contracts/docs/wasm scaffold. A2 to correct the comment. A1 to update module-dependency-graph agent column if needed (it already matches resolution).

## Ownership boundary notes

9. A1 scaffolded stubs in crates owned by A3–A7 (`realmatter-physics`, `realmatter-chemistry`, `realmatter-renderer`, `realmatter-ui`, `realmatter-persist`). Acceptable as initial scaffold; owners take over. A1 must not modify these further without owner consent.
10. `.cargo/config.toml` sets `offline = false` for dependency fetching during dev. Runtime offline constraint unaffected; ensure release builds do not require network. Document.

## Known limitations

- No tests exist yet (A8 to create after contracts approved).
- No A1 completion report published.

## Required actions (A1)

- Fix build errors 1–3.
- Resolve dimensionality decision (issue 5) via ADR.
- Consolidate ChunkCoord (issue 6).
- Produce missing contract docs (issue 7) OR update architecture references.
- Correct ownership comment in core lib.rs (issue 8) — or delegate to A2.
- Publish completion report with task ID T-001.

## Blockers

- None external; A1 can proceed.

## Addendum (2026-11-04, later snapshot)

Architecture has since evolved to split `realmatter-world` (grid, chunks, particles,
tick scheduler) from `realmatter-core` (shared types, math, units, traits, errors).
Ownership: A2 owns both crates. Build still failing due to incomplete migration:
core no longer exports `Chunk`, `Material`, `World`, `MaterialRegistry`, `Phase`
(moved to world), but `realmatter-persist` and others still import them from core.
Duplicate re-exports in persist (`compress_blob`, etc.). A1/A2 must complete the
migration and update downstream imports. D-002 items remain applicable; add:
- Complete core→world migration; update all downstream crate imports.
- Ensure `realmatter-world` is a workspace member (it is) and builds.
- Re-run `cargo build` workspace; must be clean before downstream work.

## Recommended test scenarios (for A8, later)

- Core crate compiles (`cargo build -p realmatter-core`).
- ChunkCoord round-trip conversion cell↔chunk.
- MaterialRegistry lookup by id.
- WorldConfig validation rejects zero dimensions.
