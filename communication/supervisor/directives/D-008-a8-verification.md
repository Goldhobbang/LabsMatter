# Directive D-008 — A8 Verification of A2 Persistence + WASM Boundary

- Issued by: A0
- Date: 2026-11-04
- Priority: HIGH
- Target: A8
- References: T-007-A2-20261104-1500, D-007, REQ-A2-001, REQ-A5-001, M2 gates

## Scope
Independently verify A2's reported completion of Persistence recovery and WASM boundary implementation. Do not trust unverified claims.

## Verification tasks
1. **Workspace build**: Run `cargo check --workspace` and `cargo build --workspace`. Report actual exit codes and output.
2. **Persist isolation**: Run `cargo check -p realmatter-persist`. Confirm no 3D/core-legacy imports.
3. **Save/load**: Test round-trip logic via code inspection and build. Document manual procedures for covered scenarios.
4. **Corruption handling**: Verify load_blob handles bad magic/truncated/bad checksum gracefully (no panic).
5. **Determinism**: Review tick/order against docs; flag nondeterminism.
6. **WASM contract**: Verify exports match REQ-A5-001/docs/contracts/wasm-api.md (init/step/set_cell/get_cell*/world_*/reset/render_buffer_*/save_blob/load_blob).
7. **Render buffer**: Check ptr/len consistency, RGBA8 layout, bounds.
8. **World coords/bounds**: Verify 2D, bounds checking.
9. **Reset/init**: Verify behavior.
10. **Browser readiness**: Assess adapter compatibility.

## Evidence required
- PASS/FAIL/NOT_VERIFIED per category
- Actual commands + results
- Commit SHA (note working tree if no commit)
- Reproducible failure evidence
- Release-gate recommendation (APPROVE/REJECT/CONDITIONAL)

## Constraints
- No new test files/mocks
- Use existing infrastructure
- Document manual procedures for uncovered scenarios
- Publish report in communication/agents/A8/reports/
