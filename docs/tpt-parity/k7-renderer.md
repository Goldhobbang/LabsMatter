# K7 renderer, display modes, and editing integrity

Branch base: `cursor/tpt-parity-9315` (`76480ab`). The K6 kernel names in the task (`agent1/kernel`, `graphics.rs`, `GfxPixel`, `5c334572`, `2c3bb906`, `b250a6c0`) are not in this repository. No Agent 2 or Agent 3 branch exists beyond that history. This work continues from the Powder Toy shell that is actually here.

Reference renderer: The Powder Toy `2b7221a46d378589b4de7683f16d3ebc92be6bd9`, `src/graphics/Renderer.cpp` and `src/simulation/ElementGraphics.h`.

The simulation stays authoritative. `realmatter-renderer` only reads `Cell` and writes one RGBA8 buffer. WebGL2, WebGPU, and Canvas2D upload that buffer. They do not run physics.

## Feature inventory

| Feature | Status | Verified here | Notes |
|---|---|---|---|
| Normal colours for solids and powders | Implemented | Developer, pixel spot-check of the heat scale and sand flat colour | Same bytes as before for flat materials |
| Alpha composition (steam, smoke, glass, water) | Implemented | Developer | Straight alpha is blended over `(5, 8, 15)` in the CPU buffer. GPUs draw that RGB opaquely |
| Additive fire and spark | Implemented | Developer | `FIRE` adds, `SPRK` draws a deterministic cross and writes `FIRE_SPARK` |
| Fire buffer and decay | Implemented | Developer, timed | TPT kernel: 4×4 block, `exp(-0.1 r²)` splat, then `(8·self + neighbours) / 16 - 4`. Not a Gaussian |
| Lava halo | Implemented | Developer | `FIRE_ADD` on lava. Other elements do not glow unless `element_graphics` says so |
| Fancy heat tint | Implemented | Not re-pixel-checked | Same 500 K mix as before, plus fire |
| Fancy warp / gravity lens | Blocked | — | Needs `gravity_force`. Not faked |
| Heat view | Implemented | Formula kept: lava 1500 K → `(207, 255, 37)`, sand 293 K → `(14, 0, 188)` | Empty cells stay clear |
| Particle pressure view | Implemented | Not independently pixel-checked | Previous absolute-pascal ramp. Not TPT air pressure |
| Particle velocity view | Implemented | Not independently pixel-checked | Previous speed ramp |
| Air velocity | Partially implemented | Developer | Block average of **cell** `vx`, `vy`, and gauge pressure. Empty blocks are black. See scale below |
| Air pressure | Partially implemented | Developer | Same blocks. Positive gauge is red, negative is blue, clamp 8 |
| Air heat | Partially implemented | Developer | Particle heat. With ambient on, empty cells use the 295.15 K setpoint. No `hv` lattice |
| Persistent trails | Implemented | Developer | RGB decays by 1 per channel per composed frame. Fire is not stored in the trail |
| Gravity overlay | Partially implemented | Developer | Down, off, or toward centre. Not Newtonian vectors |
| Decoration paint | Not implemented | — | Blend order is in the renderer. There is no decoration buffer. No empty dialog |
| Wall glow | Not implemented | — | No wall layer. Stone is a cell |
| WebGL2 upload | Implemented | Browser, see handoff | `texSubImage2D` when the size is unchanged. Context loss is logged and restored |
| WebGPU upload | Implemented | **Not verified. No adapter in this environment** | Same RGBA buffer, nearest filter, row pitch padded to 256. Device loss asks the page to fall back to WebGL2 |
| Copy / stamp / undo of temperature, velocity, flags, lifetime, pressure | Implemented | Developer | Spark `vx` is the conductor id and is not rotated |
| SOAP, portals, pipes, players, fighters | Not in this world | — | Nothing to retarget. Do not invent indices |
| Save format | Unchanged | Existing iron round-trip still uses RMT v1 | `Cell` was already the chunk payload |

Do not treat this table as TPT parity.

## Graphics callback

`element_graphics(material) -> ElementGraphics` is the shared output Agent 3 can replace:

`r, g, b, a, pixel_mode, fire_a, fire_r, fire_g, fire_b`

`pixel_mode` uses the reference bit values (`PMODE_*`, `FIRE_*`, `NO_DECO`, `DECO_FIRE`).

Draw order matches the reference: flat, blend, add, glow, blur, spark, then decoration (no source today), then the fire buffer. Decoration would mix into the base colour unless `NO_DECO`, and into the fire colour when `DECO_FIRE`.

`GRAPHICS_CALLBACK_GAPS` in `crates/realmatter-renderer/src/lib.rs` lists callback inputs this world does not store (ctype, tmp, life-as-int, dcolour, pipe particles, per-element function pointer, stickmen, walls, air lattice, gravity field, emp).

Lifetime: material 6 with `0 < lifetime ≤ 1` scales alpha and fire. Lifetime 0 stays fully visible. Iron cooldown lifetime is not a fire fade.

## Air and gravity fields that are missing

Until Agent 2 (or whoever owns the solver) exposes these, the views above stay approximations:

```text
air_sample(block_x: i32, block_y: i32) -> { pv: f32, vx: f32, vy: f32, hv: f32 }
```

One sample per 4×4 block. `pv` is TPT gauge pressure (0 ambient, colour clamps at ±8). `vx`/`vy` are cells per frame (colour clamps at 8). `hv` is air temperature in kelvin.

```text
gravity_force(block_x: i32, block_y: i32) -> { fx: f32, fy: f32 }
```

Needed for a Newtonian overlay and for `DISPLAY_WARP` lensing. The gravity view currently draws only the configured field: down, off, or toward the centre.

Current display scale, because cells store pascals rather than `pv`:

```text
gauge = (pressure - 101325) / 101325 * 8
```

One atmosphere of gauge is the colour clamp. Gas pressure in this solver is far below one atmosphere, so air-pressure blocks that contain gas read as blue, and empty blocks stay black. Air-velocity red/blue stay dim because cell velocities are far below 8. That is the field, not a hidden scale factor.

## Editing

`set_cell` still paints a fresh cell (material, characteristic temperature, solid/`SPRK` fixed flag).

`set_cell_full` writes material, temperature, vx, vy, flags, lifetime, and pressure. Non-finite values are replaced. Undo stores the full before and after cell, so a multi-material stamp redoes as itself rather than as one brush id. Stamps rotate position with `(dx, dy) -> (dy, -dx)` and rotate velocity the same way, except material 16, whose `vx` is the conductor to restore (8 iron or 18 copper).

There is no particle index, portal channel, pipe payload, SOAP link, player slot, or fighter slot in this grid. Copying those by index would be wrong; they are absent, not silently duplicated.

## Performance notes (developer, not browser FPS)

Node timings are simulation steps plus the CPU compose. They are not frames per second.

Measured on a 256×256 world, release WASM, Node 22, after the change. Compose is `render_millis()` around `set_view_mode` (the physics step is not included). Step TPS includes physics and compose.

| Case | Result |
|---|---|
| Empty Normal compose | 0.280 ms |
| Dense sand Normal compose | 0.770 ms |
| Dense sand Fancy compose | 0.792 ms |
| Dense sand Heat compose | 0.728 ms |
| Fire 128² Normal compose | 0.943 ms |
| Dense sand Persistent compose | 0.862 ms |
| Dense sand air-pressure compose | 0.864 ms |
| Checkerboard ~32k cells, 20 steps | 174.6 ms, about 114.6 steps/s, reported compose 2.150 ms |
| Dense sand, 10 steps | 162.8 ms, about 61.4 steps/s, compose 0.700 ms |
| Dense sand Fancy, 10 steps | 148.1 ms, about 67.5 steps/s, compose 0.800 ms |
| Fire 128², 8 steps | 46.6 ms, about 171.8 steps/s, compose 1.125 ms |
| Persistent, 8 steps | 42.7 ms, compose 0.875 ms |

Compose stays under 1 ms on these grids. A dense-sand step is about 16 ms because of the physics pass, not the glow splat. These numbers are not browser FPS.

What changed in the browser path:

- The old per-frame JS scan of the RGBA buffer for the particle counter is skipped when `particle_count()` is present.
- WebGL reuses the texture with `texSubImage2D` when the grid size is unchanged.
- WebGPU reuses its uniform array.
- Fire splat runs only for blocks that currently hold fire, and only in Normal, Fancy, and Gravity.

Headless Chromium 156, SwiftShader, `http://127.0.0.1:5173/?engine=tpt`, empty world then a 49-cell stamp: HUD read about 58–61 FPS, compose 0.10–0.50 ms, backend `webgl2`, module `realmatter_wasm`. Console: `WebGPU init failed, falling back: No WebGPU adapter`. `navigator.gpu` was present. That is not a WebGPU pass.

## Runtime

- WASM init failure stops on the error overlay. `src/adapters/simulation.js` is not started in its place.
- Boot checks `module_id() == "realmatter_wasm"`.
- WebGL context loss is logged, the GL objects are dropped, and restore rebuilds them. The simulation object is kept.
- WebGPU device loss sets `fallbackRequested`. The page then attaches WebGL2 and says so. It does not swap the simulation.
- A failed load sets `last_error()` and leaves the current world in place. The HUD shows that string.
- WebGPU was not executed here. Do not mark it pass.
