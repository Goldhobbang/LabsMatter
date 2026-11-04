# REALMATTER — Task Dependency Graph

- Owner: A0
- Updated: 2026-11-04
- Format: Task ID | Agent | Required contracts | Dependencies | Owned files | Status | Acceptance

## Tasks

| Task ID | Agent | Required contracts | Dependencies | Owned files | Status | Acceptance criteria |
|---------|-------|--------------------|--------------|-------------|--------|---------------------|
| T-001 | A1 | Architecture spec v0.1 | none | docs/**, Cargo.toml, .cargo/**, rust-toolchain.toml, crates/realmatter-wasm/** | IN_PROGRESS (D-002) | cargo build workspace clean; contract docs exist; dimensionality ADR |
| T-002 | A2 | T-001 corrected | T-001 | crates/realmatter-core/**, crates/realmatter-world/** | PARTIALLY_BLOCKED | core+world build clean; ChunkCoord consolidated; ownership comment fixed |
| T-002b | A2 | none (design) | none | crates/realmatter-core/** (docs) | READY | particle movement design doc published |
| T-003 | A3 | T-001, T-002 | T-001, T-002 | crates/realmatter-physics/** | PARTIALLY_BLOCKED | thermal/fluid/pressure spec doc; impl after contracts |
| T-003b | A3 | none (design) | none | crates/realmatter-physics/** (docs) | READY | thermal/fluid/pressure spec doc published |
| T-004 | A4 | T-001, T-002, T-003 | T-001, T-002, T-003 | crates/realmatter-chemistry/** | PARTIALLY_BLOCKED | reaction schema + conservation doc |
| T-004b | A4 | none (design) | none | crates/realmatter-chemistry/** (docs) | READY | reaction schema doc published |
| T-005 | A5 | T-001, T-002 | T-001, T-002 | crates/realmatter-renderer/** | PARTIALLY_BLOCKED | renderer arch + fallback doc |
| T-005b | A5 | none (design) | none | crates/realmatter-renderer/** (docs) | READY | renderer arch doc published |
| T-006 | A6 | T-001, T-002, T-005 | T-001, T-002, T-005 | crates/realmatter-ui/** | PARTIALLY_BLOCKED | UI layout/interaction spec doc |
| T-006b | A6 | none (design) | none | crates/realmatter-ui/** (docs) | READY | UI spec doc published |
| T-007 | A7 | T-001, T-002 | T-001, T-002 | crates/realmatter-persist/** | PARTIALLY_BLOCKED | save format + IndexedDB arch doc |
| T-007b | A7 | none (design) | none | crates/realmatter-persist/** (docs) | READY | persistence spec doc published |
| T-008 | A8 | none (read-only) | none | tests/**, benchmarks/** (read-only elsewhere) | READY | build diagnostics report; verification guidelines; quality-gate docs; test scenarios documented |

## Dependency edges

```
T-001 (A1) ──► T-002 (A2) ──► T-003 (A3) ──► T-004 (A4)
   │              │              │
   │              ├──► T-005 (A5) ──► T-006 (A6)
   │              └──► T-007 (A7)
   └──► T-008 (A8) [read-only, no contract dep]
```

Design tasks (T-002b, T-003b, T-004b, T-005b, T-006b, T-007b) have no dependencies and are READY now.

## Auto-unblock rule

When T-001 transitions to APPROVED:
- T-002 → READY (implementation portion)
- T-008 → already READY

When T-002 transitions to APPROVED:
- T-003, T-005, T-007 → READY

When T-003 transitions to APPROVED:
- T-004 → READY

When T-005 transitions to APPROVED:
- T-006 → READY

## Additional tasks (from A5/A8 reports)

| Task ID | Agent | Required contracts | Dependencies | Owned files | Status | Acceptance criteria |
|---------|-------|--------------------|--------------|-------------|--------|---------------------|
| T-UI-001 | A5 | REQ-A5-001 (WASM/JS boundary) | none (mocks) | web/** | APPROVED (milestone 1) | node --check pass; HTTP 200; manual browser smoke test |
| T-008 | A8 | none (read-only) | none | tests/**, benchmarks/** (read-only elsewhere) | IN_PROGRESS | build diagnostics report published; verification guidelines |

## Current hard blockers

- T-001 (A1): contracts not yet approved (D-002 open).
- T-002 (A2): core/world build broken (D-003 open).

## Monitoring

A0 polls `/communication/agents/*/reports/` for new completion reports.
See `/communication/tools/monitor-reports.ps1`.
