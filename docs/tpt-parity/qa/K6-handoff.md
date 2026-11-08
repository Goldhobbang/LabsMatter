# K6 Handoff — Rendering, Editing & Core Integration (Agent B)

**Branch:** `agent1/kernel`
**Runtime:** experimental TPT engine via `?engine=tpt`; production unchanged.
**Prior:** `K5-handoff.md`, `K3/K5` Agent B reports (preserved).

> K6 adds graphics-callback infrastructure, glow/fire/bloom, a gravity view, and
> full-state editing. It is implementation-ready, not gameplay-accepted.

## 1. Run

```powershell
cd web
python -m http.server 5173
```
- Production: `http://localhost:5173/`
- TPT: `http://localhost:5173/?engine=tpt`

Diagnostics: `window.labsmatter.{requestedEngine, engine, adapter, wasmIdentity,
sim, renderer}`. HUD shows the active adapter (`MISMATCH` if it disagrees with
the URL).

## 2. New in K6

### Graphics callback infrastructure (`realmatter-tpt::graphics`)
- `GfxPixel { mode, base rgba, fire/glow rgba }` and `pmode`/`firemode` flags
  mirroring `ElementGraphics.h`.
- `for_particle(t, p)` applies: element base colour, `PROP_HOT_GLOW` tint
  (reference formula), per-particle decoration blend, and curated element rules
  (FIRE/PLSM/LAVA glow, SPRK/ELEC add+spark, PHOT/NEUT add, GLOW, BRAY/ARAY,
  TRON hue). Agent 3 extends `element_graphics` for remaining callbacks.

### Glow / fire / bloom
- WASM builds a second RGBA8 **glow buffer** in the fancy view; zero elsewhere.
  Exports: `glow_buffer_ptr/len`, `set_glow_enabled`, `glow_enabled`.
- WebGL2 blit shader samples the glow texture with a 4-neighbour box blur and
  adds it (cheap bloom). WebGPU renderer has the equivalent second texture +
  shader path (unverified here: no WebGPU adapter in this environment).
- Walls contribute `eglow` when powered; particles contribute their fire colour.

### Display modes
- Normal (flat), **Fancy** (glow/bloom), Heat, Pressure, Velocity, **Gravity**
  (new; `grav_force` field). `set_view_mode` clamps 0..5; UI adds a `GRAV` button.

### Editing preserves full state
- New `set_cell_ex(x,y,t,life,ctype,tmp,tmp2,tmp3,tmp4,flags,dcolour,temp)`.
- Copy/paste, stamps, rotate/flip and undo/redo now snapshot `getCellInfo`
  (all particle fields + wall) and restore through `set_cell_ex` + `paint_wall`.
  Verified: FIRE copied with `dcolour=0xFF00FF00`, `life=163`, `temp=695.15`.

### Tools
- Unchanged semantics: HEAT/COOL particle-only, AIR/VAC pressure, AMBP/AMBM
  ambient, MIX/CYCL/WIND air. PGRV/NGRV still have no browser button (A2
  gravity-well solver dependency).

## 3. Persistence
- LMT2 v3 unchanged; decoration (`dcolour`), particle fields, walls/emap,
  fields, settings, RNG, portal/wireless/entities all saved. Glow is derived and
  not saved. Failed loads remain atomic.

## 4. Verified by A1 (not acceptance)

Build: `cargo build/test --workspace`, wasm release + bindgen, `web/scripts/build.mjs`.

Node (release WASM):
- Normal view: glow buffer all zero. Fancy: FIRE glow `[255,255,20,220]`,
  LAVA `[255,110,20,110]`, SPRK `[255,255,120,255]`, PHOT `[120,255,255,220]`,
  SAND zero. `set_glow_enabled(false)` → zero.
- `set_cell_ex` round-trips FIRE type/life/ctype/temp/decoration.

Browser (Chrome headless, WebGL2):
- Identity: `TptAdapter` / `labsmatter-wasm` (prod: `WasmDirectAdapter` /
  `realmatter-wasm`), 0 console errors, 0 shader-link warnings.
- Fancy: glow buffer 262144 B, 85 non-empty pixels (FIRE paint). Normal: 0.
- Gravity view renders; v3 save/load 85→85.
- UI copy → paste preserved FIRE `type 4`, `dcolour FF00FF00`, `life 163`,
  `temp 695.15` at the pasted offset.

## 5. Performance (Node, release WASM 594 KB, 128×128, edge solid)

| Scenario | Sim TPS | Sim+render TPS | Render ms/frame |
|---|---:|---:|---:|
| empty normal | 10,695 | 4,985 | 0.107 |
| empty fancy | 12,599 | 5,124 | 0.116 |
| sparse 10% normal | 3,018 | 2,040 | 0.159 |
| sparse 10% fancy | 3,177 | 2,253 | 0.129 |
| dense 50% normal | 740 | 653 | 0.180 |
| dense 50% fancy | 526 | 441 | 0.365 |
| dense 50% heat | 459 | 432 | 0.137 |

Glow roughly doubles CPU render-buffer cost at dense 50% (~0.18 ms), still far
above the 60 TPS target at this size. Browser FPS 45–60.

## 6. Known limitations

- WebGPU glow path implemented but **not verified** (no adapter available here).
- Bloom is a 4-tap box blur, not a multi-pass Gaussian.
- Air display sub-modes (AIRC/AIRW/persistent) not implemented; heat/pressure/
  velocity/gravity only.
- Element-specific graphics rules are curated (extension point for Agent 3);
  not all 79 reference graphics callbacks are ported.
- PGRV/NGRV gravity-well physics depends on Agent 2.
- Advanced elements (PIPE/PSTN/SOAP/STKM/STKM2/FIGH) remain partial per Agent 3.
- No independent gameplay acceptance.

## 7. Retest checklist

1. Fancy vs Normal: fire/lava/spark/photon glow and bloom; toggle glow.
2. Display modes: Normal/Fancy/Heat/Pressure/Velocity/Gravity; decoration.
3. Copy/paste/stamp/rotate/flip/undo/redo preserve life/ctype/tmp/flags/decoration/wall.
4. Walls render and glow when powered.
5. Tools (heat/cool/air/vac/amb/mix/cycl/wind) modify authoritative state.
6. Save/load v3 incl. decoration; v1 import; invalid load is atomic.
7. Dense-world FPS/TPS with glow on/off.
