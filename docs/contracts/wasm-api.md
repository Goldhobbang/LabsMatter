# Contract: WASM / JS API

**Version:** 1.0.0
**Owner:** A1 (spec) / A1 (scaffold) + agents

## Exported Functions (realmatter-wasm)

- `init(width: u32, height: u32) -> Result<(), JsValue>`
- `step(dt: f64)`
- `render_buffer_ptr() -> *const u8`
- `render_buffer_len() -> usize`
- `set_cell(x: i32, y: i32, material: u16) -> bool`
- `get_cell_material(x: i32, y: i32) -> u16`
- `get_cell_temperature(x: i32, y: i32) -> f32`

## Render Buffer Format

Interleaved per cell: `[material:u16, temp:f32, vx:f32, vy:f32]`? 
Final format TBD by A5. Placeholder: RGBA8 per cell (4 bytes) for color,
plus a separate float buffer for temperature. A5 to finalize.

## Memory

- WASM linear memory is the source of truth for simulation state.
- JS reads render buffers via typed arrays over WASM memory.
- No copying of full world each frame; use views.
