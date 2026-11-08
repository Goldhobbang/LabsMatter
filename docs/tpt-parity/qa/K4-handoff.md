# K4 Handoff — Independent Gameplay Verification (Agent B)

**Branch:** `agent1/kernel`
**Build:** see the commit SHA in `communication/agents/A1/reports/K4-A1-*.md`
**Runtime:** experimental TPT engine via `?engine=tpt`. Production is unchanged.
**Prior K3 audit:** `docs/tpt-parity/qa/K3-retest-agentB.md` (code-level, no browser).

> K4 is implementation-ready, not gameplay-accepted. Only Agent B can accept it.

## 1. How to run

```powershell
cd web
python -m http.server 5173
```
- Production: `http://localhost:5173/`
- Experimental TPT: `http://localhost:5173/?engine=tpt`

The HUD shows `TPT`/`PROD`. `window.labsmatter` exposes `{ engine, sim, materials, renderer }`.

Rebuild if needed:
```powershell
cargo build -p labsmatter-wasm --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/labsmatter_wasm.wasm `
  --out-dir web/pkg-tpt --target web --no-typescript
cd web; node scripts/build.mjs
```

## 2. New in K4

- **Element default properties** (`element_defaults.rs`, generated from the pinned
  reference): particles now start with the reference `DefaultProperties.life/ctype/
  tmp/tmp2/tmp3/tmp4/vx/vy`. Fixes, for example, PHOT dying on the first tick
  (reference life = 680).
- **Browser entity controls** (TPT only): spawn/despawn stickman, hold-to-move
  left/right/jump, element selector (175 elements), fighter spawn, live status.
- **Browser simulation tools** (TPT only): Heat, Cool, Air+, Vac, Cyclone, Wind,
  plus Gravity/Air/Edge/Newtonian/Ambient cycle buttons. Selecting a tool and
  dragging applies it at the cursor.
- **Accurate element inventory**: Full 126, Partial 6, Generic 48, Passive 15,
  Disabled 1. `Generic` elements have create/graphics/transition behaviour and are
  no longer lumped into "internal-only".
- **Persistence**: v3 state coverage extended/verified (particles, walls, emap,
  air fields, settings, RNG, portal payloads, wireless, entities); v1 import now
  reports dropped v1 flag bits in addition to unmapped materials and decoration.
- **Shared transport API contract** for Agent 3: `docs/tpt-parity/contracts/agent3-transport-api.md`.

## 3. Verified by A1 (not acceptance)

Build: `cargo build/test --workspace`, wasm release+bindgen — PASS.

Browser (Chrome headless, WebGL2, 1280×800):

| Check | TPT | PROD |
|---|---|---|
| load, console errors | ready, 0 | ready, 0 |
| tool panel / entity panel | visible / visible | hidden / hidden |
| env status | `grav down air on edge void` | hidden |
| heat tool drag (UI) | max temp 293 → 433 K | n/a |
| gravity cycle button | status changed | n/a |
| spawn stickman (UI) | alive | n/a |
| fighter spawn (UI) | 1 | n/a |
| despawn stickman | not alive | n/a |
| v3 save→load | 140 → 140 | n/a |

Node-level (real release wasm):
- 175 menu-visible elements each painted and stepped 3 ticks: 174 create a
  particle; only 0:NONE (eraser) creates none; **no crash**.
- Boundary: Solid 200→200 (max y 123, border starts 124), Void 200→0, Loop 200→200.
- Wall: 0 sand inside a wall CELL; temp preserved exactly across save/load.
- Portal: payload stored 1 → save/load → 1 → emitted after reload.
- Entity: alive after 120 ticks with input; survives save/load.
- Invalid coordinates rejected; `get_cell_material`/`get_wall` return 0.

## 4. NOT verified / known limitations

- **Wireless**: `WirelessStore` ordering is present and `Sim::after_sim` promotes
  it, but a simple WIFI+conductor layout did not raise `wireless_active` in the A1
  smoke test. Element-level WIFI behaviour needs Agent 3 work and a proper spark
  source. Do not treat wireless as working.
- **PIPE / PSTN / SOAP / STKM / STKM2 / FIGH**: partial (see inventory). They no
  longer crash in smoke tests and shared storage persists, but reference behaviour
  is incomplete.
- Electricity is partial; GoL unverified; energy-particle refraction unverified.
- No independent gameplay acceptance yet.

## 5. Persistence compatibility

| Input | Behaviour |
|---|---|
| LMT2 v3 (current) | full load: particles, walls/emap, fields, settings, RNG, portal, wireless, entities |
| LMT2 v2 | loads; v3-only sections default empty |
| RMT v1 (`RMT\0`, optional `ZLIB`) | imported via id → abbreviation → PT id; temp/vx/vy/lifetime preserved; pressure averaged; unmapped materials, v1 flags, decoration reported |
| Unknown/corrupt | rejected; active world unchanged; reason in `last_load_report()` |

## 6. Agent B retest checklist

1. Boundary: Solid/Void/Loop with powders/liquids/gases and energy particles;
   confirm positions/properties, not only counts.
2. Tools: heat/cool/pressure/vacuum/cyclone/wind; gravity/air/edge/newton/ambient
   cycle buttons.
3. Entity: spawn, move left/right, jump, element select, fighter spawn/despawn;
   confirm no teleport or disappearance.
4. Air/thermal: pressure propagation, freezing/boiling, fire spread.
5. Electricity: SPRK/PSCN/NSCN/SWCH/LCRY.
6. Energy particles: PHOT/NEUT/ELEC creation and movement.
7. Save/load: v3 round-trip, v1 import, invalid blob rejection, reset after load.
8. Rendering: orientation, cursor alignment, zoom/pan, heat/pressure/velocity views.
9. Performance under a dense world (report separately FPS vs TPS).

## 6a. Pixel-level QA without canvas readback (resolves Agent B P1)

Agent B noted that a WebGPU canvas cannot be read with 2D `getImageData`. The TPT
adapter already exposes CPU-side data, so no canvas readback is needed:

| Need | Call (from the page console) |
|---|---|
| pixel colour | `sim.readRenderBuffer()` → `Uint8Array` RGBA per pixel |
| material at (x,y) | `sim.getCellMaterial(x, y)` |
| temperature | `sim.getCellTemperature(x, y)` |
| pressure / velocity | `sim.getCellPressure(x, y)`, `sim.getCellVx/Vy(x, y)` |
| particle count / tick | `sim.getDiagnostics().particles`, `.tick` |
| wall id | `sim.mod.get_wall(x, y)` |

These read the live WASM state, not the rendered canvas, and work under any
renderer. The K4 checks in §3 were produced this way.

Also note: `apply_tool` and RNG-state persistence (Agent B's two P1 items) are
already present in this build:
- `sim.mod.apply_tool(kind, x, y, radius, dx, dy)` — heat/cool/pressure/vacuum/
  PGRV/NGRV/mix/cyclone/wind.
- Save format v3 stores both Xoroshiro words (`Rng::state/set_state`), so a
  loaded world continues the same random sequence.

## 7. Performance reference

See the K3 handoff table (same layout); K4 adds an O(particles) sanitize pass and
the generated defaults lookup (O(1) per creation). No hot-loop allocation was
added.
