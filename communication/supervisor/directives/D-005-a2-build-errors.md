# Directive D-005 — A2 Core Build Error Corrections (from A8 review)

- Issued by: A0
- Date: 2026-11-04
- Priority: HIGH
- Target: A2 (with A1 coordination)
- References: A8 report T-008, Review R-001, Directives D-002, D-003

## Scope (A2-owned files only)

- `crates/realmatter-core/src/error.rs`
- `crates/realmatter-core/src/material.rs`
- `crates/realmatter-core/src/lib.rs`
- `crates/realmatter-core/src/world.rs`

A1 must NOT modify these. A1 coordinates contract decisions only.

## Required corrections

1. `error.rs`: Break circular alias. `SimResult<T> = Result<T, SimError>` and
   `Result<T> = SimResult<T>` is circular (E0391). Define `Result<T>` once as
   `std::result::Result<T, RealmatterError>` (or `SimError`) and remove the
   duplicate alias. Fix `SimResult<T>` generic arity (E0107).
2. `material.rs`: Remove duplicate `MaterialState`/`MaterialProps` type aliases
   that reference non-existent `Material`/`Phase` (E0428). Define them as real
   types/enums per approved contract (docs/contracts/material-registry.md).
   Remove `Copy` derive on type alias (E0206).
3. `lib.rs`: Align re-exports with actual module definitions. Remove re-exports
   of `Material`, `Phase`, `World` if not defined; add them only when defined.
   Fix `MaterialState` derives.
4. `world.rs`: Ensure `World` type exists or remove re-export. Reconcile 2D
   (approved contract) vs 3D per D-002 item 3 decision.
5. Ensure `cargo build -p realmatter-core` succeeds with no errors.

## Acceptance criteria

- `cargo build -p realmatter-core` exits 0.
- No E0391/E0428/E0432/E0206/E0107 errors.
- `cargo clippy -p realmatter-core` clean (or documented exception).

## Notes

- Dimensionality decision (2D vs 3D) must be resolved by A1 via ADR (D-002 item 3).
  A2 implements whichever is decided; do not unilaterally switch.
- If a contract change is needed, file a request in `/communication/requests/pending/`.
- A7 (persist) and A3 (physics) must update their crates after core stabilizes.
