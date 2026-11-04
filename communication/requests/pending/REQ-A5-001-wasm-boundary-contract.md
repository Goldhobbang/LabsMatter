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

Real WASM module loads and executes ticks (`init`, `step`, `render_buffer_ptr/len`)
but lacks interaction/read APIs. A5 needs the following exports (minimal viable):

### Required exports (wasm-bindgen, camelCase or snake_case acceptable)
1. `init(width: u32, height: u32) -> Result<(), JsValue>` (exists)
2. `step(dt: f64)` (exists)
3. `set_cell(x: i32, y: i32, material: u16) -> bool` — place/erase (material 0 = empty)
4. `get_cell_material(x: i32, y: i32) -> u16`
5. `get_world_size() -> [u32;2]` or `world_width()/world_height()`
6. `reset()` or `clear() -> bool`
7. `render_buffer_ptr() -> *const u8`, `render_buffer_len() -> usize` (exists)

### Memory/contract notes
- Coordinates are world cell coordinates (integer). Y-up per architecture.
- `material` uses core MaterialId (u16). 0 = empty/void.
- Return `bool` for mutators indicating success.
- Buffer format: if exposed, document layout or provide getters. Prefer getters
  first to avoid unsafe memory layout assumptions.
- Ownership: WASM owns memory; JS reads via views.

### Also needed (for UX)
- Diagnostics: particle count/chunk count accessible or derivable.
- Error reporting via `JsValue` on init failures.

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
