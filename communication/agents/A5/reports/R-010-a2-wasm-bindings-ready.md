# Report R-010 — A2 WASM bindings ready (web/pkg regenerated)

- Agent: A2 (Persistence/WASM)
- Date: 2026-11-04
- References: REQ-A5-001, docs/contracts/wasm-api.md, D-009
- Artifacts updated: web/pkg/realmatter_wasm_bg.wasm, web/pkg/realmatter_wasm.js

## Build steps (reproducible)
1. `cargo build -p realmatter-wasm --target wasm32-unknown-unknown` (dev)
2. `wasm-bindgen target/wasm32-unknown-unknown/debug/realmatter_wasm.wasm --out-dir web/pkg --target web --no-typescript`

WASM bindgen: matched to Cargo.lock (wasm-bindgen 0.2.130). wasm-bindgen-cli updated to 0.2.130.

## Verified exports (from generated bindings + wasm)
From web/pkg/realmatter_wasm.js:
- init(width: u32, height: u32) -> Result<(), JsValue>
- step(dt: f64)
- set_cell(x: i32, y: i32, material: u16) -> bool
- get_cell_material(x: i32, y: i32) -> u16
- get_cell_temperature(x: i32, y: i32) -> f32
- world_width() -> u32
- world_height() -> u32
- reset() -> bool
- render_buffer_ptr() -> *const u8
- render_buffer_len() -> usize
- save_blob() -> Uint8Array
- load_blob(bytes: Uint8Array) -> bool

## Notes / contract
- Coordinates world cell coords (integer), Y-up. material 0 = empty/void.
- set_cell marks material==1 (wall) as FIXED. Returns false on error/out of bounds.
- render_buffer is RGBA8 per cell, row-major, y outer/x inner (4 bytes/cell). ptr/len from exports.
- save_blob/load_blob use persist format v1 (A2 PersistEngine). load_blob rebuilds render buffer on success.
- Builtin materials (ids): 0 empty, 1 wall, 2 sand, 3 water. Map UI materials accordingly.
- JS owns no memory; read via views over WASM memory (bindings handle memory access).

## Integration guidance for A5
- Use the regenerated web/pkg bindings (real exports now available). wasm-direct adapter can call set_cell/get_cell_*/world_*/reset/save_blob/load_blob directly.
- For rendering, prefer reading render_buffer via ptr/len (or keep existing path).
- Fallbacks remain safe if uninitialized.

## Blockers / follow-ups
- None from A2 side for the requested minimal viable exports. If you need additional APIs (e.g. different command API shape), request via A1 with concrete use case.
