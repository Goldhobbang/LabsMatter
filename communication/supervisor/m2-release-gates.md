# M2 Release Gates

- Owner: A0
- Date: 2026-11-04
- Status: GATES DEFINED

## Gate 1: Full Rust Workspace Compilation
- **Requirement**: `cargo build` exits 0
- **Owner**: A2
- **Status**: BLOCKED (D-007)
- **Criteria**: No errors; warnings acceptable if documented
- **Verification**: A8 runs `cargo build`

## Gate 2: Correct 2D Persistence API Compatibility
- **Requirement**: persist compiles against current 2D core API
- **Owner**: A2
- **Status**: BLOCKED
- **Criteria**: No duplicate re-exports; correct MaterialRegistry usage; no non-existent imports
- **Verification**: A8 checks build + interface compatibility

## Gate 3: Stable Rust/WASM Interface
- **Requirement**: WASM boundary contract published and stable
- **Owner**: A2 (with A5 input)
- **Status**: BLOCKED (REQ-A5-001)
- **Criteria**: Contract covers init/tick/commands/render snapshot/lifecycle/diagnostics
- **Verification**: A5 confirms consumption; A8 verifies exports

## Gate 4: Browser Initialization Using Real WASM Module
- **Requirement**: Browser loads WASM and initializes world
- **Owner**: A5 (with A2)
- **Status**: BLOCKED
- **Criteria**: No mocks used for core init; adapters bound to real exports
- **Verification**: A8 integration check

## Gate 5: Deterministic Simulation Stepping
- **Requirement**: Fixed timestep, deterministic
- **Owner**: A3 (with A2)
- **Status**: READY (interface) / UNVERIFIED
- **Criteria**: Physics/Chemistry use fixed dt; no nondeterministic iteration
- **Verification**: A8 (after integration)

## Gate 6: Material Placement and Rendering
- **Requirement**: Input commands place materials, renderer displays
- **Owner**: A5 (with A2)
- **Status**: BLOCKED
- **Criteria**: Command queue works end-to-end
- **Verification**: A8

## Gate 7: Physics/Chemistry Execution
- **Requirement**: Modules execute in tick order
- **Owner**: A3
- **Status**: READY
- **Criteria**: PhysicsModule/ChemistryModule integrated
- **Verification**: A8

## Gate 8: Save/Load Functionality
- **Requirement**: Persistence compiles and serializes world state
- **Owner**: A2
- **Status**: BLOCKED
- **Criteria**: Save/load round-trip works
- **Verification**: A8

## Gate 9: Error Handling and Recovery
- **Requirement**: Proper error types, no panics in hot paths
- **Owner**: A2
- **Status**: PARTIAL
- **Criteria**: Uses core error types
- **Verification**: A8

## Note
Distinguish implemented (code exists) vs verified (tested/integrated). Most gates unverified until integration.
