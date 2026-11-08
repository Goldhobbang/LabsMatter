# Integration Handoff — Engine v2 (TPT runtime)

**Owner:** Agent 1 (integration lead)
**Status:** K2 browser integration landed. Production engine unchanged; TPT
runtime opt-in via `?engine=tpt`.
**Engine crate:** `realmatter-tpt` · **WASM crate:** `labsmatter-wasm` ·
**Frontend adapter:** `web/src/adapters/tpt.js` ·
**Palette mapping:** `web/src/tpt-materials.js`

---

## 1. Actual shared API signatures

### `realmatter_tpt::sim::Sim` (world owner)

```rust
pub struct Sim {
    pub width: usize, pub height: usize,          // pixels
    pub cells_x: usize, pub cells_y: usize,        // width/4, height/4
    pub pmap: Vec<u32>, pub photons: Vec<u32>,     // occupancy (ordinary/energy)
    pub parts: Vec<Particle>, pub free: Vec<u32>,
    pub bmap: Vec<u8>, pub emap: Vec<u8>,          // walls / wall electricity
    pub pv: Vec<f32>, pub vx: Vec<f32>, pub vy: Vec<f32>, pub hv: Vec<f32>,
    pub rng: Rng, pub settings: Settings, pub tick: u64,
    pub can_move: Vec<u8>, pub element_count: [u32; PT_NUM],
    /* glow, colour, grav_force, grav_mass, ... */
}
```

Key methods (stable for A2/A3):

- `create_part(x, y, t) -> Option<usize>`
- `kill_part(i)` / `kill_part_reason(i, reason)`
- `part_change_type(i, x, y, t) -> bool`
- `paint(x, y, t, ctype)` / `paint_wall(x, y, w)`
- `set_wall(x, y, w)` / `wall_at(x, y)` / `wall_blocks(x, y, t)` / `wall_kills(w, t)`
- `neighbourhood(i) -> Neighbourhood`
- `pmap_at(x, y)` / `type_at(x, y)` / `id_at(x, y)`
- `gravity_field(x, y, particle_grav, newton_grav) -> (f32, f32)`
- `step()` → `before_sim()` → `update_particles()` → `after_sim()`

### `realmatter_tpt::particle::Particle`

`t: u16`, `life: i32`, `ctype: i32`, `x/y/vx/vy/temp: f32`,
`tmp/tmp2/tmp3/tmp4: i32`, `flags: u32`, `dcolour: u32`, `pavg: [u8;4]`.

### `realmatter_tpt::rng::Rng`

`next`, `gen`, `gen_u32`, `between`, `chance`, `uniform01`, plus the new
`state() -> (u64,u64)` / `set_state(s0, s1)` used by v2 persistence.

---

## 2. WASM exports (`labsmatter-wasm`)

World/lifecycle:
`init(w,h)`, `step(dt)`, `reset()`, `world_width()`, `world_height()`,
`tick_count()`, `particle_count()`, `element_count(t)`.

Painting/reading:
`set_cell(x,y,pt_id)`, `paint_wall(x,y,wall_id)`, `get_wall(x,y)`,
`get_cell_material(x,y)`, `get_cell_temperature(x,y)`,
`get_cell_pressure(x,y)`, `get_cell_vx(x,y)`, `get_cell_vy(x,y)`.

Render: `render_buffer_ptr()`, `render_buffer_len()` (RGBA8 per pixel; 0 =
empty), `set_view_mode(0=material, 2=heat, 3=pressure, 4=velocity)`.

Settings: `set_gravity_mode`, `set_air_mode`, `set_edge_mode`,
`set_newtonian`, `set_ambient_heat`, `set_heat_sim`, `set_convection_mode`,
`cycle_gravity/air/edge`, `toggle_newtonian/ambient`, `sim_status`.

Persistence: `save_blob() -> Uint8Array` (v2, magic `LMT2`),
`load_blob(bytes) -> bool` (rejects legacy v1 blobs; world untouched on error).

---

## 3. Tick ordering (as implemented)

`Sim::step()`:

1. `before_sim()` — `update_air`, ambient heat (`update_airh`), Newtonian
   gravity, EMP decay, tick++, `recalc_and_life` (pmap rebuild + life decay),
   emap decay, air block maps, GoL.
2. `update_particles()` — per particle: off-screen kill, wall kill/stasis/
   detector, air momentum, neighbourhood, velocity (loss/advection/gravity/
   diffusion), `transition_phase`, `update_fn` (A3), `legacy_update` when heat
   is off, `movement_phase` (A2).
3. `after_sim()` — currently no-op.

This matches the reference `BeforeSim → UpdateParticles → AfterSim`. The
scheduler is authoritative; A2/A3 supply functions, not their own passes.

---

## 4. Known limitations (K2)

- **Border death:** particles entering the outer `CELL` margin are killed in
  both `Void` and `Solid` edge modes (edge walls not yet modelled). Falling
  matter must be caught by a drawn barrier. A2 territory.
- **Walls** are stored per 4x4 CELL; per-pixel wall painting selects the cell.
- **Electricity** is partial (A3).
- **Deterministic RNG state** is persisted; behaviour parity still depends on
  A3 implementing matching call order.
- **V2 save** stores particles, velocity, temp, lifetime, ctype/tmp fields,
  flags, dcolour, walls, emap, air fields, settings and RNG state. Decoration
  is a frontend overlay and is not stored in a TPT v2 blob.
- **Element coverage** is not parity: registered/menu-visible counts are 196 /
  175, but implemented update hooks are far fewer. Selecting an element in the
  palette does not mean its behaviour is complete.

## 5. Required changes from Agent 2 (physics)

1. Implement edge walls for `EdgeMode::Solid` so particles are blocked, not
   killed, at the border (reference `WallType`/boundary logic).
2. Confirm `update_air`, `movement_phase`, `transition_phase` call order and
   signatures match the contract; report any deviation.
3. Newtonian gravity is currently a Jacobi approximation — document or replace.

## 6. Required changes from Agent 3 (chemistry/elements)

1. Keep `update_fn` dispatch in `updates.rs` as the single entry point; do not
   mutate `parts`/`pmap` outside it.
2. Use `sim.rng` for every probabilistic decision (no local RNG).
3. If an interface change is needed (e.g. extra particle fields), update
   `docs/tpt-parity/contracts/engine-v2.md` first and notify A1.

## 7. Frontend adapter contract

`TptAdapter` implements the same interface as `WasmDirectAdapter`:
`init`, `step/tick`, `reset`, `setCell`, `paint/line/rect/fill`,
`applyCommand`, `getWorldSize`, `getCellMaterial/Temperature/Pressure/Vx/Vy`,
`readRenderBuffer`, `getDiagnostics`, `setViewMode`, `cycle*`, `toggle*`,
`simStatus`, `saveBlob`, `loadBlob`, `canPaint/canReset/canRead/canSave`,
`engine`.

Palette ids: PT element ids are used directly; walls are addressed as
`WALL_ID_BASE + wallId` (1000 + wall id) and routed to `paint_wall`.
`window.labsmatter` exposes `{ engine, sim, materials, renderer }` for
diagnostics only.

---

## K3 update

### Boundary semantics (`Sim::apply_edge_mode`)
Mirrors `Simulation::SetEdgeMode`:
- `Solid` installs `WL_WALL` (8) on the outer CELL ring; particles are blocked
  before the off-screen kill test, so no unintended loss.
- `Void` leaves the ring empty; particles leaving the drawable area are removed.
- `Loop` leaves the ring empty; `do_move`/`plan_move` wrap via `remainder_p`.
Called from `set_edge_mode`, `cycle_edge`, and after v2/v3 load.

### Stability
`Sim::sanitize()` runs at the start of `BeforeSim` and clamps non-finite
positions/velocities/temperatures and air fields. It is a no-op for healthy
state.

### Persistence
- Native format is now **v3** (`LMT2`): adds portal payload, wireless channels,
  and player/fighter entity state on top of v2 (particles, walls, fields,
  settings, RNG). v2 blobs still load.
- Legacy **v1** (`RMT\0`, optional `ZLIB`) imports through an explicit
  id → abbreviation → PT-id mapping. Mismatched magic is rejected without
  touching the world; the reason is returned by `last_load_report()`.

### Sim tools
`labsmatter-wasm::apply_tool` mirrors `realmatter-wasm` (heat/cool/pressure/
vacuum/PGRV/NGRV/mix/cyclone/wind), using `Sim::add_pressure` /
`add_air_velocity` and particle-temperature edits. No physics solver lives in
the adapter.

### Agent 3 integration record
The integrated dispatcher is `crates/realmatter-tpt/src/updates.rs`
(blob `c6a528c37c1b36d326702554a344e87342372c83`, 7817 lines at commit
`952cc42`). This is the version with 132/132 reference `Update` callbacks.
The `agent3/chemistry-elements` worktree at `779f6f6` holds an older 2900-line
baseline plus an uncommitted `sim.rs` experiment (`flood_tmp`, IGNT life) whose
functionality already exists in the integrated tree (local `flood_tmp` in
`updates.rs`, `IGNT` default added to `apply_defaults`). No overwrite occurred.

### Advanced infrastructure (entity/portal/wireless/piston/soap)

`entity.rs`, `portal.rs`, `transport.rs`, and `wireless.rs` are wired into
`Sim` and covered by save format v3. Verified in the built WASM bundle:

- **Boundary modes** (128×128, 200-cell sand block, 260 steps):
  Solid 200→200 (`wall(0,0)=8`), Void 200→0, Loop 200→200.
- **Portal persistence**: PRTI absorbs a SAND payload (portal_count=1), the world
  is saved and re-loaded into a fresh 32×32 world, portal_count is still 1 and
  `last_load_report()` reports success.
- **Entity persistence**: a spawned stickman walks right, survives save/load.
- **Piston**: a sparked PSTN extends its arm (1→3 PSTN particles).
- **Stickman**: spawn + `set_stickman_input(RIGHT)` moves the head 33→42 px over
  10 ticks without teleporting through a brick floor.

The earlier `infra_check` panic was a harness bug: the example kept a raw
particle index across `Sim::recalc()`, which compacts the particle list once per
tick. The engine is index-correct within a tick; external callers must re-resolve
with `Sim::entity_head`. The temporary harness was removed.

Residual element-specific work (Agent 3): full `PIPE` payload flow, `PSTN`
`FRME` recursion and `tmp3` flags, `SOAP` pop/detach edge cases, and the
reference `STKM`/`STKM2`/`FIGH` legs and element-power reactions.


---

## K4 update

### Element default properties
`crates/realmatter-tpt/src/element_defaults.rs` is generated by
`tools/tpt-inventory/gen_element_defaults.py` from the pinned reference JSON.
`Sim::apply_defaults` applies `life/ctype/tmp/tmp2/tmp3/tmp4/vx/vy` on creation.
`temp` remains `ElementDef::default_temp`. This fixed elements with
`PROP_LIFE_KILL_DEC` (e.g. PHOT, life 680) dying immediately after painting.

### Element inventory
`tools/tpt-inventory/status.py` now classifies Full / Partial / Unimplemented /
Generic / Passive / Disabled and reports enabled, menu-visible, selectable and
independent-verification counts. 48 elements that lack a reference `Update` are
classified as Generic (create/graphics/transition behaviour), not internal-only.

### Browser entity + tool UI (TPT only)
- Entity panel: spawn/despawn stickman, hold-to-move left/right/jump, element
  selector, fighter spawn, live status. Uses the existing WASM exports.
- Tool panel: heat/cool/pressure/vacuum/cyclone/wind (drag to apply) and
  gravity/air/edge/newtonian/ambient cycle buttons.
- Both panels are hidden in production and when the relevant exports are absent;
  no dead controls are shown.

### Transport API contract
`docs/tpt-parity/contracts/agent3-transport-api.md` documents the exact
`PortalStore`, `WirelessStore`, `Entity`, and stack/piston/SOAP signatures for
Agent 3.

### Persistence
v3 coverage verified for particles, walls/emap, fields, settings, RNG, portal
payloads, wireless and entities. v1 import additionally reports dropped v1 flag
bits. Failed loads remain atomic (world untouched).

### Verification
All 175 menu-visible elements were painted and stepped 3 ticks in the release
WASM: 174 create a particle (0:NONE is the eraser), no crash. Boundary
positions/properties, wall collision, portal persistence, entity stability, and
v3 round-trip were exercised.

---

## K5 update

### Rendering infrastructure
`rebuild_render` now composites: wall base colour (per CELL) with powered
`eglow` from `emap`, particle colour (normal / fancy heat tint / heat ramp /
pressure / velocity), and per-particle `dcolour` decoration. Energy particles
render. Walls and decoration were previously invisible.

### Decoration editor
`set_deco` / `get_deco` / `clear_all_deco` operate on `Particle.dcolour`
(persisted in LMT2 v3). Browser: `Deco` toggle + colour picker + `Clear`.

### Simulation tools
`apply_tool` kinds: 0 HEAT, 1 COOL, 2 AIR, 3 VAC, 4 PGRV, 5 NGRV, 6 MIX,
7 CYCL, 8 WIND, 9 AMBP, 10 AMBM. HEAT/COOL target the particle only; AIR/VAC
use reference pressure scale; PGRV/NGRV write the gravity-mass grid (full
gravity-well solver is an A2 dependency and has no browser button yet).

### Inspection API
`get_cell_info` (JSON) and individual getters expose type/position/temp/velocity/
life/ctype/tmp/tmp2/tmp3/tmp4/flags/dcolour/wall/emap/pressure/air velocity/
ambient/gravity, plus `get_world_settings` and `get_entity_head`. CPU-side, works
under any renderer.

### Bug fixes
- Empty pixels no longer report particle slot 0 temperature (`pmap_id(0)==0`).
- HEAT/COOL no longer heat slot 0 through empty pixels.

### Inventory
`docs/tpt-parity/inventory/application-features.md` tracks editing, simtools,
walls, rendering, UI/shortcuts and save coverage with reference/impl/owner/status.
