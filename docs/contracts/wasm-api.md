# Contract: WASM / JS API

**Version:** 1.1.0 (M3: added `tick_count`, `reset`, `world_width/height`, `save_blob`/`load_blob` beyond the original minimal set)
**Owner:** A1 (spec) / A1 (scaffold) + agents

## Exported Functions (realmatter-wasm)

- `init(width: u32, height: u32) -> Result<(), JsValue>`
- `step(dt: f64)`
- `render_buffer_ptr() -> *const u8`
- `render_buffer_len() -> usize`
- `set_cell(x: i32, y: i32, material: u16) -> bool`
- `get_cell_material(x: i32, y: i32) -> u16`
- `get_cell_temperature(x: i32, y: i32) -> f32`
- `world_width() -> u32`, `world_height() -> u32`
- `reset() -> bool` (clears all cells AND resets tick counter to 0)
- `tick_count() -> u64` (simulation tick counter; restored on `load_blob`)
- `save_blob() -> Vec<u8>` / `load_blob(bytes: &[u8]) -> bool` (persist format v1)

## Live Tick Pipeline (M3)

`step()` executes the approved phase order from `simulation-tick.md` on the
world's cell slice (no duplicate state):

1. **Guards** — `stability::sanitize` (NaN/Inf clamp)
2. **Chemistry** — `ChemistryEngine::react` (registry: combustion/oxidation/dissolution)
3. **Physics forces** — gas pressure/diffusion/buoyancy, fluid viscosity/
   pressure/buoyancy, `heat::conduct`, `heat::phase_transitions`,
   `reaction::react_all` (incl. fire spread), `transfer::transfer_energy/mass`
4. **Movement** — powder + liquid cellular automaton (deterministic)
5. **Finalize** — `World::finalize_tick` (UPDATED flags, lifetimes, tick++)
6. **Export** — RGBA8 render buffer refresh

Paint temperatures: lava 1500K, fire 1200K, steam 373K, smoke 400K, ice 263K,
all others ambient 293.15K — so thermal interactions are observable on paint.

## Render Buffer Format

Interleaved per cell: `[material:u16, temp:f32, vx:f32, vy:f32]`? 
Final format TBD by A5. Placeholder: RGBA8 per cell (4 bytes) for color,
plus a separate float buffer for temperature. A5 to finalize.

## Memory

- WASM linear memory is the source of truth for simulation state.
- JS reads render buffers via typed arrays over WASM memory.
- No copying of full world each frame; use views.
