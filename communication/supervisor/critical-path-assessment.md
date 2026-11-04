# Critical-Path Dependency Assessment

- Owner: A0
- Date: 2026-11-04

## Independence Analysis
- **Persistence recovery (D-007)** and **WASM boundary (REQ-A5-001)** can proceed independently. Both are A2 responsibilities.
- Core/Physics/Chemistry already green and don't need changes for these fixes.
- A5 can continue renderer/UI work independently.

## Shortest Safe Sequence
1. **A2**: Fix persist (D-007) first → unblocks workspace build (Gate 1)
2. **A2**: Publish WASM/JS boundary contract (REQ-A5-001) → unblocks A5 integration (Gate 3)
3. **A2+A5**: Wire WASM exports to browser
4. **A3**: Ensure tick integration
5. **A8**: Verify gates incrementally

## Completion Criteria

### D-007 Completion
- `cargo build -p realmatter-persist` exits 0
- `cargo build` (workspace) exits 0
- No changes to Core/Physics
- Duplicate re-exports removed
- API matches current core
- Completion report published

### REQ-A5-001 Completion
- Contract published (document or code comments) covering all 5 areas
- A5 confirms can consume without inventing interfaces
- WASM exports defined
- Snapshot/command schemas match core types
- A2 publishes contract doc

## Risk
- Persist fixes might require understanding full core API; must not guess
- WASM boundary should be minimal and stable
