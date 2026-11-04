# Request REQ-A5-001 — WASM/JS Boundary Contract for UI Integration

- Requested by: A5 (UI & Interaction Engineer)
- Date: 2026-11-04
- Priority: MEDIUM
- Target: A1 (Systems Architect)
- Status: PENDING

## Summary

A5 has implemented an offline browser UI shell (`web/`) that communicates with the
simulation/renderer only through adapter interfaces (`SimulationAdapter`,
`RendererAdapter`). The shell currently uses in-browser mock implementations so it
is functional and demoable without the WASM core.

To replace the mocks with the real core, A5 needs a published, versioned contract
for the WASM/JS boundary covering:

1. Command queue schema (UI → core): paint, line, rect, fill, spawn, remove,
   set material, pause/resume, speed, reset, with coordinate systems and units.
2. Render snapshot schema (core → renderer/UI): grid dimensions, material ids,
   temperature/velocity fields, dirty regions, tick counter.
3. Camera/viewport contract: world-to-screen transform, zoom limits, pan.
4. Lifecycle: init(config), ready, tick(dt), shutdown, error reporting.
5. Diagnostics payload: fps, tick ms, render ms, particle count, chunk count,
   memory.

## Why

Without these contracts, A5 cannot safely bind to `realmatter-wasm` and risks
inventing interfaces, which violates the interface change policy.

## Proposed owner

A1 (contracts) with input from A2 (core), A5 (renderer/UI), A7 (persistence).

## Impact if not resolved

UI remains on mock adapters; integration deferred. No blocker for continued
independent UI work (layout, a11y, shortcuts, panels).

## Notes

- A5 will not modify renderer internals or core; integration will go through
  the published adapter interfaces only.
- Mock adapters are isolated under `web/src/adapters/` and can be swapped without
  touching UI components.
