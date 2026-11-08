# Application Feature Inventory (editing, tools, walls, rendering, UI, saves)

Reference: The-Powder-Toy @ `2b7221a46d378589b4de7683f16d3ebc92be6bd9`.
Owners: **A1** = core/integration, **A2** = physics, **A3** = element behaviour.

Status legend: **Done** = implemented and exercised in the K5 release WASM;
**Partial** = implemented but reference details missing; **Blocked** = depends on
another agent; **Missing** = not implemented. Verification is separate from
implementation; only Agent B gameplay acceptance counts as verified.

## 1. Editing tools (`src/simulation/Editing.cpp`, `ToolClasses.cpp`)

| Feature | Reference | LabsMatter | Owner | Status |
|---|---|---|---|---|
| Paint particle | `Tool::DrawPoint` | `setCell` → `Sim::paint` | A1 | Done |
| Erase particle | `Tool::ErasePoint` | material 0 → kill | A1 | Done |
| Brush size | `brushX/brushY` | `brushRx/brushRy` (0–24) | A1 | Done |
| Brush shapes | circle / square | circle / square / triangle | A1 | Done |
| Line | Shift-drag | Shift-drag (`lineCells`) | A1 | Done |
| Rectangle outline | Ctrl-drag | Ctrl-drag | A1 | Done |
| Filled rectangle | modifier | Ctrl-drag filled | A1 | Done |
| Flood fill | Ctrl+Shift click | `flood()` | A1 | Done |
| Selection | select tool | `tool: select` | A1 | Done |
| Copy / cut | clipboard | `copySel(clear)` | A1 | Done |
| Paste / stamp | stamp library | stamps in `localStorage` | A1 | Done |
| Stamp rotate / mirror | rotate/flip | `rotatePaste`/`flipPaste` | A1 | Done |
| Undo / redo | undo stack | per-cell material log (30 deep) | A1 | Done |
| Eyedropper | middle click | `sample()` | A1 | Done |
| Zoom / pan | wheel / alt-drag | wheel / alt-drag / arrows | A1 | Done |
| Wall paint | `Tool::DrawWalls` | wall palette id 1000+w → `paint_wall` | A1 | Done |
| Wall erase | wall ERASE | palette id 1000 (w=0) | A1 | Done |
| Decoration paint | `Tool::DrawDecoration` | `set_deco` (per-particle `dcolour`) | A1 | Done |
| Decoration erase | deco tool | `clear_all_deco` / ARGB 0 | A1 | Done |

## 2. Simulation tools (`src/simulation/simtools/`, 11 sources)

| Tool | Reference effect | `apply_tool` kind | Status |
|---|---|---|---|
| HEAT | particle temp `+= strength*2` | 0 | Done |
| COOL | particle temp `-= strength*2` | 1 | Done |
| AIR | `pv += strength*0.05` | 2 | Done |
| VAC | `pv -= strength*0.05` | 3 | Done |
| PGRV | adds gravity-well mass | 4 | Partial — A2 gravity-well solver |
| NGRV | removes gravity-well mass | 5 | Partial — A2 gravity-well solver |
| MIX | averages air velocity | 6 | Done (approximation) |
| CYCL | vortex air velocity | 7 | Done (approximation) |
| WIND | `vx/vy += drag*strength` | 8 | Done |
| AMBP | `hv += strength*2` | 9 | Done |
| AMBM | `hv -= strength*2` | 10 | Done |

Browser UI exposes HEAT, COOL, AIR+, VAC, CYCL, WIND, AMB+, AMB−. PGRV/NGRV have
no browser button until A2 provides the gravity-well solver (no fake controls).

## 3. Walls (`SimulationData.cpp`, 19 entries)

All 19 wall definitions are present in `elements_generated::WALLS` and selectable
in the palette "Walls" section. Wall paint/erase and per-CELL collision work.
Rendering draws the wall base colour and blends `eglow` when `emap != 0`
(powered/detector). Element-specific electrical wall behaviour is A3.

| Status | Notes |
|---|---|
| Done | paint, erase, collision (`wall_blocks`/`wall_kills`), base + powered rendering, persistence (`bmap`/`emap`) |
| Partial | streamline/fan visual variants; full powered-wall semantics (A3) |

## 4. Rendering / display (`src/graphics/Renderer.cpp`)

| Feature | LabsMatter | Status |
|---|---|---|
| Normal particle display | element colour per pixel | Done |
| Fancy (heat tint) | `set_view_mode(1)` | Done |
| Heat | `set_view_mode(2)` | Done |
| Pressure | `set_view_mode(3)` | Done |
| Velocity | `set_view_mode(4)` | Done |
| Wall rendering | per-CELL colour + `eglow` | Done |
| Decoration overlay | `dcolour` blended per particle | Done |
| Grid overlay | renderer `setOverlay("grid")` | Done |
| High-DPI | renderer DPR handling | Done |
| WebGL2 / WebGPU | renderer factory | Done (WebGL2 verified; WebGPU by Agent B) |
| Glow buffer | `Sim::glow` allocated, not rendered | Missing |
| Air display sub-modes (AIRP/AIRV/AIRH/AIRC/AIRW, persistent/warp) | simplified to heat/pressure/velocity | Partial |

## 5. UI / shortcuts (`src/gui/`, README controls)

| Feature | Status |
|---|---|
| Material search | Done (`E`) |
| Palette categories + walls | Done |
| Brush size/shape controls | Done |
| Pause / frame step | Done |
| Speed slider | Done |
| Grid / HUD toggles | Done |
| View mode buttons + keys | Done |
| Stamp library / paste | Done |
| Undo / redo | Done |
| Save / load | Done |
| Reset | Done |
| Simulation settings (gravity/air/edge/newton/ambient) | Done (TPT panel + keys) |
| Entity controls (STKM) | Done (spawn/despawn/move/jump/element) — element powers A3 |
| Fighter spawn | Done — AI A3 |
| Decoration editor | Done (colour input, paint mode, clear) |
| Text-input shortcut safety | Done (inputs/dialogs ignored) |

## 6. Save format

| State | LMT2 v3 |
|---|---|
| Ordinary + energy particles (all fields incl. `dcolour`) | Yes |
| Walls (`bmap`) + wall power (`emap`) | Yes |
| Air pressure/velocity, ambient heat | Yes |
| Gravity field, settings, RNG state | Yes |
| Portal payloads, wireless, entities | Yes |
| Decoration (`dcolour`) | Yes (per-particle) |
| Legacy RMT v1 import | Yes (id→abbr→PT id; unmapped/flags/deco reported) |
| Atomic failed load | Yes (world untouched + report) |

## 7. Known reference differences (do not claim parity)

- Glow/fire bloom not rendered; air display sub-modes simplified.
- PGRV/NGRV gravity-well physics depends on A2.
- Wall draw styles (streamline/fan visual) are simplified.
- Element-specific graphics callbacks (A3) are not implemented; rendering uses
  base element colours plus heat tint and decoration.
- 6 element behaviours remain partial (STKM/STKM2/FIGH/PIPE/PSTN/SOAP).
