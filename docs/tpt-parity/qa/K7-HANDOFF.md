# K7 handoff — Agent B

Developer notes from the K7 pass. This is not independent acceptance.

| | |
|---|---|
| Role | Agent 1 (developer) |
| Branch | `cursor/k7-render-state-0fb0` |
| Parent | `cursor/tpt-parity-9315` at `76480ab` |
| TPT reference | `2b7221a46d378589b4de7683f16d3ebc92be6bd9` |
| K6 SHAs named in the task | Not in this clone. Work continued from the powder-toy branch above |
| Agent 2 / Agent 3 commits | None newer than the parent. Nothing merged |
| Browser used by the developer | Headless Chromium 156, SwiftShader, `http://127.0.0.1:5173/?engine=tpt` |
| Acceptance | **Not accepted.** Agent B plays it. |

## What to play

1. Open `http://localhost:5173/?engine=tpt`. The HUD should name `webgl2` and `realmatter_wasm`. If boot fails, the overlay must stay up. The legacy `simulation.js` engine must not start.
2. The console line `WebGPU init failed, falling back: No WebGPU adapter` is expected on this machine. It is not a pass and not a silent engine swap.
3. Place sand. Count should match the stamp (a radius-4 dab was 49 cells here). Ctrl+Z or Undo returns the count to 0. Redo restores the same cells, including velocity on non-spark materials.
4. Switch every view button: Normal, Fancy, Heat, Pressure, Velocity, AIRV, AIRP, AIRH, PERS, GRAV. Keys 1–5 are the first five; 6–9 and 0 are the new five. The HUD label must follow the button.
5. Heat lock, if you sample the WASM buffer (canvas `readPixels` after present is black because `preserveDrawingBuffer` is false): lava id 9 reads `(207,255,37,255)`, sand at 293 K reads `(14,0,188,255)`, empty reads `(5,8,15,0)`.
6. Normal sand with no noise offset reads `(217,194,122,255)`.
7. Fire in Normal should light the cell and a dim neighbour. Persistent should leave a fading trail and should not store the fire bloom in that trail.
8. Air pressure on empty space stays black. Gas blocks read blue because gauge pressure is far below one atmosphere (`(p-101325)/101325*8`, clamp 8). Air velocity stays dim because cell `vx`/`vy` are far below 8. Do not treat that as a missing scale factor.
9. Save, reset, load. A bad blob must leave the current world in place and put a `load failed:` string on the HUD.
10. Lose the WebGL context if you can (`WEBGL_lose_context`). The HUD note should say the context was lost, then restored, and the same WASM module should still be running.

## Status the developer will stand behind

| Feature | Status | Evidence |
|---|---|---|
| Shared `element_graphics` table and blend/fire bits | Implemented | Release WASM pixel checks in Node |
| Four-neighbour fire splat plus in-place 3×3 | Implemented | Fire core `(255,126,15)` and neighbour above the background, Node |
| Decoration mix | Not implemented | No decoration buffer. Documented in `k7-renderer.md` |
| Wall glow | Absent | No wall map |
| Warp | Absent | Fancy does not displace |
| WebGL2 composite | Developer-verified in the browser | HUD `webgl2`, ~58–61 FPS on a 49-cell world, compose 0.10–0.50 ms |
| WebGPU composite | Implemented, **not verified** | `requestAdapter` failed. Do not mark pass |
| Heat / pressure / velocity | Implemented | Heat pixels checked in Node. Pressure and velocity use the old cell ramps |
| Air velocity / pressure / heat | Partially implemented | Cell fields only. No `pv`/`vx`/`vy`/`hv` lattice. Agent 2 API is in `k7-renderer.md` |
| Persistent | Implemented | Decay by 1 per channel. Not independently accepted |
| Gravity overlay | Partially implemented | Configured down / off / centre. Not Newtonian |
| Full-cell undo, spark `vx` not rotated | Developer-verified | Browser 49 → 0 → 49. Node spark round-trip |
| SOAP, portals, pipes, players, fighters | Absent | Do not expect them |
| Browser FPS vs Node TPS | Separated | Node dense-sand step ~61 steps/s is physics. Browser empty/small world ~60 FPS |

## Do not accept as TPT parity

The palette is still ids 0–25. Air views are cell-grid approximations. Glow is the reference's exponential kernel shape on a 4×4 block, not a Gaussian, and not a wall-glow pass. WebGPU was not executed.
