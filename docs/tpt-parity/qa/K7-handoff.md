# K7 Handoff — Rendering Fidelity, Editing Integrity & Core Integration (Agent B)

**Branch:** `agent1/kernel`
**Runtime:** experimental TPT engine via `?engine=tpt`; production unchanged.
**Prior:** `K6-handoff.md` (superseded where overlapping).

> K7 improves renderer fidelity, editing state preservation, inspection APIs and
> performance. It is implementation-ready, not gameplay-accepted.

## 1. Run

```powershell
cd web
python -m http.server 5173
```
- Production: `http://localhost:5173/`
- TPT: `http://localhost:5173/?engine=tpt`

Rebuild:
```powershell
cargo build -p labsmatter-wasm --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/labsmatter_wasm.wasm `
  --out-dir web/pkg-tpt --target web --no-typescript
cd web; node scripts/build.mjs
```

## 2. New in K7

### Graphics callback infrastructure
- `realmatter-tpt::graphics`: `GfxPixel { mode, base rgba, fire rgba }`,
  `pmode::*`/`firemode::*` (mirrors `ElementGraphics.h`), `for_particle`.
- `PROP_HOT_GLOW` reference tint, decoration blend, curated element rules
  (FIRE/PLSM, LAVA, SPRK, ELEC, PHOT, NEUT, GLOW, BRAY/ARAY, TRON hue).
- Agent 3 extends `graphics::element_graphics` for remaining callbacks.

### Glow / fire / bloom
- Fancy view fills a second RGBA8 glow buffer; WebGL2 blit adds a 4-neighbour
  box blur (bloom). WebGPU has the equivalent second-texture path (unverified
  here — no adapter).
- Walls contribute `eglow` when powered; particles contribute fire colour.
- `set_glow_enabled` toggles the effect.

### Display modes
- Normal (0), Fancy (1, glow), Heat (2), Pressure (3), Velocity (4),
  **Gravity (5, `grav_force`)**. UI adds a `GRAV` button.
- Air sub-modes (AIRC/AIRW/persistent) remain simplified.

### Full-state editing
- `set_cell_ex(...)` restores life/ctype/tmp..tmp4/flags/dcolour/temp.
- Copy/paste, stamps, rotate/flip, undo/redo store `getCellInfo` snapshots and
  restore through `set_cell_ex` + `paint_wall`. Special state is no longer
  reduced to a bare material id.

### Inspection API
- `get_cell_info(x,y)` JSON with all particle fields + wall/emap/pressure/air/
  ambient/gravity; individual getters; `get_world_settings`; `get_entity_head`;
  `get_deco`; `readRenderBuffer`/`readGlowBuffer`.

### Bug fixes
- Empty pixels no longer report particle slot 0 temperature (`pmap_id(0)==0`).
- HEAT/COOL no longer heat slot 0 through empty pixels.

## 3. Verified by A1 (not acceptance)

Build: `cargo build/test --workspace`, wasm release + bindgen, `web/scripts/build.mjs`.

Node (release WASM):
- Normal view: glow buffer all zero. Fancy: FIRE glow `[255,255,20,220]`,
  LAVA `[255,110,20,110]`, SPRK `[255,255,120,255]`, PHOT `[120,255,255,220]`,
  SAND zero. `set_glow_enabled(false)` → zero.
- `set_cell_ex` round-trips FIRE type/life/ctype/temp/decoration.

Browser (Chrome headless, WebGL2):
- TPT `TptAdapter`/`labsmatter-wasm`, 0 console errors, 0 shader-link warnings.
- Fancy: glow buffer 262144 B, 1 non-empty pixel (decorated FIRE); Normal: 0.
- Gravity view renders; v3 save/load 1→1.
- UI copy→paste preserved FIRE `type 4`, `dcolour 0xFF00FF00`, `life 163`,
  `temp 695.15`.
- Production `WasmDirectAdapter`/`realmatter-wasm` unchanged.

## 4. Performance (Node, release WASM 594 KB, 128×128, edge solid)

| Scenario | Sim TPS | Sim+render TPS | Render ms |
|---|---:|---:|---:|
| empty normal | 10,695 | 4,985 | 0.107 |
| sparse 10% normal | 3,018 | 2,040 | 0.159 |
| dense 50% normal | 740 | 653 | 0.180 |
| dense 50% fancy | 526 | 441 | 0.365 |
| dense 50% heat | 459 | 432 | 0.137 |

Glow adds ~0.18 ms/frame at dense 50%; still >7× the 60 TPS target at this size.
Browser FPS 45–60.

## 5. NOT verified / limitations

- WebGPU glow path implemented but **not verified** (no adapter available here).
- Bloom is a 4-tap box blur, not a multi-pass Gaussian.
- Air display sub-modes (AIRC/AIRW/persistent/warp) not implemented.
- Element-specific graphics rules are curated (extension point for Agent 3).
- PGRV/NGRV gravity-well physics depends on Agent 2.
- Advanced elements (PIPE/PSTN/SOAP/STKM/STKM2/FIGH) remain partial per Agent 3.
- No independent gameplay acceptance.

## 6. Retest checklist

1. Fancy vs Normal: fire/lava/spark/photon glow and bloom; toggle glow.
2. Display modes: Normal/Fancy/Heat/Pressure/Velocity/Gravity; decoration.
3. Copy/paste/stamp/rotate/flip/undo/redo preserve life/ctype/tmp/flags/decoration/wall.
4. Walls render and glow when powered.
5. Tools (heat/cool/air/vac/amb/mix/cycl/wind) modify authoritative state.
6. Save/load v3 incl. decoration; v1 import; invalid load is atomic.
7. Dense-world FPS/TPS with glow on/off.
8. `getCellInfo` vs visible material/temperature/pressure; empty pixels report
   `type:0`, not slot 0 data.
