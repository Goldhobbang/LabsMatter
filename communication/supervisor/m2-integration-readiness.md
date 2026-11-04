# M2 Integration Readiness Report

- Owner: A0
- Date: 2026-11-04
- Commit: c092dcc
- Status: INTEGRATION READY PREPARATION

## 1. M2 Integration Readiness Matrix

| Boundary | Status | Implementation | Notes |
|----------|--------|----------------|-------|
| Core ↔ Physics | READY | A3 implemented PhysicsModule/ChemistryModule against 2D Cell/MaterialProps. Builds OK. | Approved (R-004). No changes needed to Core. |
| Core ↔ Chemistry | READY (interface) | A3 implements ChemistryModule. | Integration pending tick scheduler/world API usage. |
| Core ↔ Persistence | BLOCKED | A2 assigned D-007. Current API mismatch (29 errors). | Must fix without touching Core/Physics. |
| Core ↔ WASM | PARTIALLY READY | wasm-bindgen scaffold exists. API contract not published. | REQ-A5-001 pending; A2 to coordinate. |
| WASM ↔ Renderer | BLOCKED | Depends on snapshot schema. | Blocked on WASM boundary contract. |
| Renderer ↔ UI/Web | PARTIALLY READY | Web shell approved (mock adapters). Renderer design spec published. | Can swap adapters once contract exists. |

## 2. Confirmed Blockers and Responsible Agents

| Blocker | Responsible | Priority | Evidence |
|---------|-------------|----------|----------|
| D-007: Persist API incompatibility | A2 | HIGH | cargo build workspace fails (23 errors + duplicates) |
| REQ-A5-001: WASM/JS boundary contract | A2 (with A5 input) | MEDIUM | A5 request pending; blocks real integration |
| M2 end-to-end runtime | A2,A3,A5,A8 | HIGH | Compilation blocked by persist |

## 3. Cross-Module Interface Requirements

| Interface | Required | Owner | Status |
|-----------|----------|-------|--------|
| Cell/MaterialProps/MaterialId/Traits | Exists in core | A2 | OK |
| PhysicsModule/ChemistryModule | Implemented by A3 | A3 | OK |
| Persistence API (save/load) | Needs alignment to core | A2 | BLOCKED |
| WASM exports (init/tick/render/commands) | Contract needed | A2 | BLOCKED |
| Render snapshot | Contract needed | A2+A5 | BLOCKED |
| Command queue | Contract needed | A2+A5 | BLOCKED |

## 4. Minimum End-to-End Implementation Sequence (M2 Vertical Slice)

| Step | Action | Owner | Status |
|------|--------|-------|--------|
| 1 | Initialize 2D world | A2 | Core/World exist; integration wiring needed |
| 2 | Register materials | A2 | MaterialRegistry exists |
| 3 | Place particles through browser input | A5+A2 | UI shell exists (mocks); needs real path |
| 4 | Advance deterministic simulation ticks | A2+A3 | Physics ready; scheduler integration needed |
| 5 | Apply physics and chemistry updates | A3 | Implemented; needs integration |
| 6 | Transfer world state to renderer | A2+A5 | Needs snapshot contract |
| 7 | Display updated simulation | A5 | Web shell ready; needs real data |
| 8 | Save and restore world state | A2 | Persistence blocked |

## 5. Acceptance Criteria for M2
- [ ] `cargo build` (workspace) exits 0
- [ ] `cargo test` passes (A8 will establish)
- [ ] Core-Physics-Chemistry integrated and deterministic
- [ ] Persistence save/load compiles and works
- [ ] WASM boundary contract published and consumed
- [ ] Browser loads WASM, initializes world, runs ticks, renders
- [ ] No 2D/3D conflicts

## 6. Next Actionable Directives
- **D-007** already issued to A2 (persist recovery) — active
- **REQ-A5-001** needs A2 to publish WASM/JS boundary contract (coordinate with A5)
- A8 to verify after A2 reports completion
- A5 to continue independent work; ready to integrate when contract exists
- A3 to ensure compatibility during integration

## Risk Assessment
- **Confirmed:** Persist blocks workspace build (high)
- **Confirmed:** WASM boundary blocks real browser integration (medium)
- **Speculative:** Runtime determinism/performance — cannot verify until integrated

## Conclusion
M2 path is clear. Primary blocker is D-007. Once workspace builds, focus shifts to WASM boundary and end-to-end wiring.
