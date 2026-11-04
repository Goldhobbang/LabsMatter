# Directive D-001 — Request Initial Architecture Contracts

- Issued by: A0
- Date: 2026-11-04
- Priority: HIGH
- Target: A1

## Request

A1, please produce the initial architecture contracts for REALMATTER. This is the foundational deliverable blocking all other agents.

## Required Deliverables

1. Project skeleton & build configuration (package.json, tsconfig, bundler config) under A1 ownership.
2. Shared type contracts in `/src/core/**` and `/src/contracts/**`:
   - Simulation tick / time step types
   - Entity / body / particle interfaces
   - Material / substance definitions
   - Reaction / interaction event types
   - Renderer interface (abstract, no impl)
   - Storage / persistence interface
   - Performance budget constants
3. Ownership boundary verification document: `/communication/agents/A1/reports/OWNERSHIP-BOUNDARIES.md` (or as report).
4. Interface versioning & change policy.

## Acceptance Criteria

- All shared interfaces compile (type check clean).
- No implementation of physics/chemistry/render/ui/data/perf logic in A1 scope (contracts only).
- Offline constraint documented.
- Performance budgets defined with measurable thresholds.

## Dependencies

- None (first task).

## Notes

- Other agents (A2..A8) are IDLE pending these contracts.
- Do not modify files outside `/src/core/**`, `/src/contracts/**`, and build config.
- Publish completion report when done.
