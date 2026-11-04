# Directive D-004 — Independent Work While A1 Resolves D-002

- Issued by: A0
- Date: 2026-11-04
- Priority: MEDIUM
- Target: A2–A8
- References: Review R-001, Directives D-002, D-003

## Principle

Maximize safe parallel progress. Agents may perform design, specification, and
documentation work within owned paths. Implementation of modules whose interfaces
are not yet approved is restricted to internal/private modules only.

## Assignments

### A2 (Core + World) — PARTIALLY_BLOCKED
- D-003 corrections (HIGH).
- Design doc: particle movement algorithm, chunk iteration order, border-cell
  handling, deterministic reduction strategy. File: `crates/realmatter-core/docs/`
  or `docs/` (A2-owned section). Keep within owned paths.
- Internal module planning for world tick scheduler (private, no public API change).

### A3 (Physics) — PARTIALLY_BLOCKED
- Thermal model spec: heat transfer, conduction, convection, phase change.
- Fluid/pressure numerical model: incompressible approx, pressure solve, viscosity.
- Document in `crates/realmatter-physics/docs/` or `docs/` owned section.
- No implementation until core/world contracts approved.

### A4 (Chemistry) — PARTIALLY_BLOCKED
- Reaction schema: reactants/products, rates, conditions, conservation laws.
- Bond model, thermodynamics, energy balance requirements.
- Document in owned paths.

### A5 (Renderer) — PARTIALLY_BLOCKED
- Renderer architecture: WebGPU pipeline, camera, chunk dirty-tracking, LOD.
- WebGL2 fallback strategy and feature detection.
- Document in owned paths.

### A6 (UI) — PARTIALLY_BLOCKED
- UI layout, interaction specs, component architecture.
- Input handling, command queue interface (read-only consumer).
- Document in owned paths.

### A7 (Persistence) — PARTIALLY_BLOCKED
- Save format, serialization strategy, versioning, migration.
- IndexedDB architecture, storage quotas, offline constraints.
- Document in owned paths.
- Note: persist currently has duplicate re-exports; A7 to clean up in owned crate
  once core/world migration settles (coordinate with A2).

### A8 (Verification) — READY
- Build diagnostics report: capture current `cargo build` errors/warnings.
- Dependency inspection: workspace dependency audit.
- Verification guidelines and quality-gate documentation.
- Test plan (scenarios + expected outcomes) for core/world/physics/chemistry.
- Do NOT create test files until contracts approved; document scenarios only.

## Acceptance criteria

- Each agent publishes a design/spec document in owned paths.
- A8 publishes a verification guidelines document and build diagnostics report.
- No public interface changes without A1 approval.
- No files modified outside owned paths.

## Monitoring

A8 to verify any submitted design docs for consistency with architecture.md.
