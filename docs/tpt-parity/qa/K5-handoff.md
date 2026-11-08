# K5 Handoff — Independent Gameplay Verification (Agent B)

**Branch:** `agent1/kernel` (Agent 3 integration: `agent3/chemistry-elements`)
**Runtime:** experimental TPT engine via `?engine=tpt`. Production unchanged.
**Prior handoffs:** `K3-handoff.md`, `K4-handoff.md` (superseded by this one where
they overlap).

> K5 is implementation-ready for core/editing/rendering/tools/decoration/
> inspection and for the six previously-partial element families, not
> gameplay-accepted. Only Agent B can accept it.

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

## 2. New in K5 (Agent 1)

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

## 3. New in K5 (Agent 3 — element behavior)

| Element | Change |
|---|---|
| WIFI | Reference transmit/receive confirmed; needs a spark source to transmit |
| PIPE / PPIP | Real payload storage/transport, direction, intake/output, PRTI hand-off, PPIP triggers, auto-config |
| PSTN | Temperature-driven extension/retraction, `tmp3` flags, FRME push |
| SOAP | Bubble formation, chain pop, obstacle detach, second-order constraints |
| STKM / STKM2 | Hazard damage, leg phase, element-power spawn, death cleanup |
| FIGH | Nearest-target pursuit, melee, death cleanup |

## 4. A1 verification (not acceptance)

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

### Agent 3 WASM evidence (not acceptance)
- WIFI: `BTRY → METL → WIFI(tx)`, `WIFI(rx) → NSCN → METL`; `wireless_active ≥ 1`
  and the receiver wire sparks.
- PIPE: after auto-configuration, a PIPE line absorbs an adjacent SAND.
- Entity: stickman alive, fighter spawn, v3 save/load round-trip.
- Native: PIPE emits past an open end; PSTN 1→4→3; SOAP 10 bubbles/10 links;
  STKM dies to acid; FIGH closes to dx=1 and kills.

## 5. NOT verified / limitations

- Glow/fire bloom not rendered; air display sub-modes simplified.
- PGRV/NGRV gravity-well physics blocked on A2.
- Element-specific graphics callbacks not rendered (base colours only).
- 6 element families remain classified Partial (see §8).
- No independent gameplay acceptance yet.

## 6. Retest checklist (core/editing)

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
8. Entities: spawn/move/jump/element/fighter.
9. Responsiveness, FPS and TPS separately under dense load.

## 7. Element checklist (Agent 3)

### WIFI
1. `BTRY → METL wire → WIFI` (transmitter); `WIFI → NSCN → METL` (receiver),
   same temperature/channel.
2. Receiver wire should spark while the transmitter has a live spark;
   `wireless_active() ≥ 1`.
3. Break the transmitter spark: channel deactivates within one tick.
4. Negative control: a lone WIFI with no spark must **not** transmit.

### PIPE / PPIP
1. Paint a PIPE line; after ~60 ticks it builds a `BRCK` border and colours.
2. Erase one `BRCK` as the exit.
3. Drop `SAND`/`WATR` at the other end: it travels and emerges at the exit; no
   payload loss.
4. Put `PRTI` at a pipe end: payload transfers to the portal and reappears at
   the matching `PRTO`.
5. Spark a `PPIP` with `PSCN`/`NSCN`/`INST`: pause/reverse toggles; border glows
   when paused.

### PSTN
1. `PSCN` spark: arm extends by temperature (~2 cells/frame) and displaces
   particles, never deletes them.
2. `NSCN` spark: arm retracts.
3. `tmp3` flags (0x01 obstacle, 0x02 arm-limit, 0x04 retract-obstacle,
   0x08 retract-too-short) produce the expected abort/limit.
4. A `FRME` row in front is pushed with the arm.

### SOAP
1. Under a pressure change it bubbles and links into chains.
2. Driven into a solid/wall with exhausted life: pops and detaches cleanly.
3. Break a link mid-chain: links repair or clean up, no corruption.

### STKM / STKM2 / FIGH
1. Walk left/right, jump; legs alternate while walking.
2. `ACID`/`LAVA` kill; fire damages then kills.
3. Element power emits in front of the entity.
4. A fighter pursues, melees, and kills; fighter count stays correct; two
   players and two fighters act independently.

### Generic / passive
- Paint no-Update elements (OIL, METL, GAS, WAX, …): reference default
  life/ctype/tmp and declared phase transitions must apply. See
  `docs/tpt-parity/inventory/generic-passive-audit.md`.

## 8. Known remaining gaps (do not report as passing)

- PIPE: HEAC flood trigger and exact PIPE graphics emulation.
- PSTN: full recursive `MoveStack`/`FRME` shared-stack propagation.
- SOAP: full chain rewiring when reconnecting mid-chain.
- STKM/STKM2: reference leg-particle physics and portal interaction.
- FIGH: reference fighter element powers and multi-fighter brawling.

## 9. Performance reference

K4/V8-class numbers (128×128, edge solid): empty ~10k simTPS, sparse 10% ~3.2k,
dense 50% ~0.8k, dense+Newtonian ~0.74k. K5 adds wall/decoration colouring per
pixel and larger inspection APIs (called only on demand); the PIPE/PPIP and
entity handlers add O(particles) work only for those elements. Simulate cost is
otherwise unchanged except the render-buffer build now also samples walls and
`dcolour`.
