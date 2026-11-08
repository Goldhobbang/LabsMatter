# K5 Handoff — Independent Gameplay Verification (Agent B)

**Branch:** `agent1/kernel`
**Runtime:** experimental TPT engine via `?engine=tpt`. Production unchanged.
**Prior handoffs:** `K3-handoff.md`, `K4-handoff.md` (superseded by this one where
they overlap).

> K5 is implementation-ready for core/editing/rendering/tools/decoration/
> inspection, not gameplay-accepted. Only Agent B can accept it.

## 1. Run

```powershell
cd web
python -m http.server 5173
```
- Production: `http://localhost:5173/`
- Experimental TPT: `http://localhost:5173/?engine=tpt`

Rebuild:
```powershell
cargo build -p labsmatter-wasm --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/labsmatter_wasm.wasm `
  --out-dir web/pkg-tpt --target web --no-typescript
cd web; node scripts/build.mjs
```

HUD shows `TPT`/`PROD`. `window.labsmatter = { engine, sim, materials, renderer }`.

## 2. New in K5

### Editing
- Wall palette (19 walls) now paints, erases, and **renders** (base colour +
  powered `eglow`). Previously walls were invisible.
- Decoration editor: `Deco` toggle + colour picker + `Clear`; paints per-particle
  `dcolour`, blended into the render buffer, persisted in LMT2 v3.
- All existing brush/line/rect/fill/select/copy/stamp/rotate/undo/eyedropper
  operations continue to drive the authoritative `Sim`.

### Tools
- AMB+ / AMB− added (ambient heat field `hv`).
- HEAT/COOL now affect only the targeted particle (reference `strength*2`), not
  the ambient field; AIR/VAC magnitudes aligned to the reference.
- PGRV/NGRV have no browser button until the A2 gravity-well solver exists.

### Rendering
- Walls, decoration, energy particles (PHOT/NEUT/ELEC) now render.
- Fancy view tints hot particles; heat/pressure/velocity views unchanged.
- Grid, zoom, high-DPI handled by the existing renderer.

### Inspection API (CPU, no canvas readback)
`sim.getCellInfo(x,y)` → JSON with `type, px, py, temp, vx, vy, life, ctype,
tmp..tmp4, flags, dcolour, wall, emap, pressure, air_vx, air_vy, ambient,
grav_x, grav_y`.
Individual getters: `getCellMaterial/Temperature/Pressure/Vx/Vy/Ambient/Emap`,
`getGravityX/Y`, `getCellLife/Ctype/Tmp..Tmp4/Flags/Dcolour/Px/Py`,
`getEntityHead`, `getWorldSettings`, `getDeco`, `readRenderBuffer`.

### Bug fixes
- `get_cell_temperature` returned particle slot 0's temperature for empty pixels
  (pmap_id(0) == 0). Fixed.
- HEAT/COOL applied to particle slot 0 through empty pixels. Fixed.
- Reference `DefaultProperties` applied on creation (from K4).

## 3. A1 verification (not acceptance)

Build: `cargo build/test --workspace`, wasm release + bindgen — PASS.
Node (release WASM):
- Wall CELL renders `[128,128,128,255]`; `get_wall = 8`.
- Decoration set → render changes to the chosen colour; persisted across v3 save/load.
- `get_cell_info` returns all fields; PHOT life 680 / ctype 0x3FFFFFFF.
- AMB+ raises ambient `295 → 315`; wall/deco/tool operations do not crash.

Browser (Chrome headless, WebGL2):
- TPT ready, 0 console errors, tool/entity panels visible.
- Wall palette 19 buttons; painting sets wall id and renders.
- Deco toggle decorates particles (6 pixels in the check).
- AMB+ via UI changed ambient at the painted cell.
- v3 save→load preserved particle count; production panels hidden.

## 3a. Agent B K3 findings reconciliation

- **P1 “frontend not wired to TPT”** (reported against `c06a86f`): **fixed**. The
  current `main.js` selects `TptAdapter` when `?engine=tpt` (verified in browser:
  the loaded module exposes TPT-only exports, HUD shows `TPT`).
- **P2 “Loop edge may not wrap”**: placing a particle inside the outer 4-px
  margin (`y=254` in a 128-tall test) is expected to die — the reference kills
  particles in the border margin. Loop wrapping applies to the drawable area;
  A1 verification shows 200 sand particles stay alive and wrap (y range 4..123).
  No core defect found.
- **P2 Newtonian gravity weak / water spreading**: Agent 2 physics dependencies.

## 4. NOT verified / limitations

- Glow/fire bloom not rendered; air display sub-modes simplified.
- PGRV/NGRV gravity-well physics blocked on A2.
- Element-specific graphics callbacks (A3) not implemented.
- 6 element behaviours remain partial (STKM/STKM2/FIGH/PIPE/PSTN/SOAP).
- No independent gameplay acceptance yet.

## 5. Retest checklist

1. Editing: paint/erase all phases; brush shapes/sizes; line/rect/fill; select,
   copy/cut/paste/stamp/rotate/flip; undo/redo; eyedropper; zoom/pan.
2. Walls: paint all 19; verify visible, block particles, erase, powered glow
   (SPRK/ELEC near conductive/detector walls).
3. Tools: HEAT/COOL particles, AIR/VAC pressure, CYCL/WIND airflow, AMB+/AMB−.
4. Decoration: paint/erase/clear, colours, saved and reloaded, survives reload.
5. Rendering: normal/fancy/heat/pressure/velocity, walls, decoration, energy
   particles, grid, high-DPI, zoom alignment.
6. Inspection: `getCellInfo` vs visible material/temperature/pressure; confirm
   empty pixels report `type:0`, not slot 0 data.
7. Persistence: v3 round-trip incl. decoration/portal/entities; v1 import;
   invalid blob leaves world unchanged.
8. Entities: spawn/move/jump/element/fighter (element powers are A3).
9. Responsiveness, FPS and TPS separately under dense load.

## 6. Performance reference

K4/V8-class numbers (128×128, edge solid): empty ~10k simTPS, sparse 10% ~3.2k,
dense 50% ~0.8k, dense+Newtonian ~0.74k. K5 adds wall/decoration colouring per
pixel and larger inspection APIs (called only on demand); simulate cost is
unchanged except the render-buffer build now also samples walls/`dcolour`.
