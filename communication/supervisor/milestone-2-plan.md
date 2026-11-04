# Milestone 2 Implementation Plan

- Owner: A0
- Updated: 2026-11-04
- Goal: Unblock complete workspace and prepare for first end-to-end Rust/WASM/browser integration

## Acceptance criteria (measurable, no mocks/tests created by A0)
- [ ] `cargo build` (workspace) exits 0 with no errors
- [ ] Core and Physics interfaces compatible (verified by A8)
- [ ] Persistence compiles and integrates with current core API
- [ ] WASM boundary contract is stable (published/consumed)
- [ ] Browser shell can initialize the actual simulation (integration path defined)
- [ ] No unresolved 2D/3D architecture conflicts (2D model confirmed)

## Tasks by owner

### A2 (Architecture, Core, World, Persistence)
1. **D-007**: Fix `realmatter-persist` to compile against current core API
   - Remove duplicate re-exports
   - Fix `MaterialRegistry` usage
   - Replace non-existent imports with current types
   - Acceptance: `cargo build -p realmatter-persist` and `cargo build` green
2. **WASM boundary**: Coordinate REQ-A5-001, publish/adapt WASM/JS boundary contract as needed
   - Acceptance: contract documented, consumed by A5

### A3 (Physics and Chemistry)
1. **Integration readiness**: Ensure Physics/Chemistry modules remain compatible with core
   - Acceptance: `cargo build -p realmatter-physics -p realmatter-chemistry` green (chemistry may need updates)

### A5 (Renderer and UI)
1. **Integration prep**: Prepare to swap mocks for real adapters once WASM boundary stable
   - Acceptance: adapter interfaces align with published contract

### A8 (Verification and integration audit)
1. **Verify D-007**: After A2 reports completion, run workspace build and compatibility check
   - Acceptance: verification report published
2. **Establish baseline**: Create `/tests/**` and `/benchmarks/**` only after workspace green per protocol

## Dependencies
- D-007 (A2) blocks workspace build → blocks A8 test baseline
- WASM boundary (A2) blocks A5 integration with real core
- Core/Physics already green

## Risks
- Persist may require significant API adaptation; avoid touching Core/Physics
- WASM boundary needs to be minimal and stable
- Keep offline constraint intact
