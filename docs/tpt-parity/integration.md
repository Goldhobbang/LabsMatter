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
