# Contract: Engine v2 (TPT-Parity World Engine)

**Version:** 2.0.0-draft
**Owner:** A1 (spec + impl)
**Status:** Published for Agent 2 (physics) and Agent 3 (chemistry/elements) to implement against.
**Reference:** The Powder Toy `2b7221a46d378589b4de7683f16d3ebc92be6bd9`

This contract supersedes the v0.x contracts in `docs/contracts/` for the
TPT-parity engine. The legacy `realmatter-core`/`realmatter-world` Cell-based
engine remains the shipping WASM path until the v2 engine is wired; v2 is the
target architecture. Agents 2 and 3 must implement against this contract.

---

## 1. Particle Representation

The v2 engine uses a **particle-list + occupancy-map** model, mirroring
`Particle` / `pmap` in the reference (`src/simulation/Particle.h`,
`Simulation.h`). A simplistic one-cell-one-particle grid is explicitly
rejected: photons, neutrons, sparks, and overlapping energy particles must be
able to coexist with ordinary matter.

### 1.1 Particle struct

```rust
pub struct Particle {
    pub t: u16,        // element id (PT_*). 0 = dead slot.
    pub life: i32,     // lifetime counter (frames); 0 = permanent unless PROP_LIFE_KILL
    pub ctype: i32,    // secondary element id (carried by SPRK, CLNE, etc.)
    pub x: f32,        // pixel position
    pub y: f32,
    pub vx: f32,       // velocity
    pub vy: f32,
    pub temp: f32,     // Kelvin
    pub tmp: i32,      // general temporary property 0
    pub tmp2: i32,     // general temporary property 1
    pub tmp3: i32,     // general temporary property 2
    pub tmp4: i32,     // general temporary property 3
    pub flags: u32,    // STAGNANT, SKIPMOVE, MOVABLE, ...
    pub dcolour: u32,  // decoration colour (ARGB); 0 = none
    pub pavg: [u8; 4], // cached display colour
}
```

Reference: `src/simulation/Particle.h`. Field order/widths follow the reference
closely enough that behaviour and save mapping stay straightforward.

### 1.2 Occupancy maps

- `pmap: Vec<u32>` — one u32 per pixel. Packs `(particle_index << 9) | element_id`
  (`PMAPBITS = 9`, `PMAPMASK = 0x1FF`). 0 = empty.
- `photons: Vec<u32>` — parallel map for energy particles (photons, neutrons,
  sparks, ...). Energy particles occupy `photons`, ordinary particles occupy
  `pmap`, so they overlap without collision.
- `parts: Vec<Particle>` — particle storage; dead slots are recycled via `free`.
- `free: Vec<u32>` — free slot indices.

Helpers: `pmap_pack(id, t)`, `pmap_id(r)`, `pmap_type(r)`.

### 1.3 Element ID scheme

Element ids are the reference `PT_*` constants (`src/simulation/ElementDefs.h`),
0..`PT_NUM` (currently ~196 plus empty slots up to 512). 0 = empty/void.
Stable across versions; never reinterpret. Frontend-facing material ids are a
separate, smaller palette space mapped to PT ids by the adapter layer
(`tpt-materials`).

### 1.4 Material property schema

See `realmatter-tpt::element::ElementDef` and `props` module. Each element
declares: `name`, `abbr`, `default_temp`, `weight`, `gravity`, `loss`,
`air_loss`, `air_drag`, `advection`, `diffusion`, `hot_air`, `flammable`,
`explosive`, `meltable`, `boilable`, `properties` (bitmask: TYPE_SOLID,
TYPE_LIQUID, TYPE_GAS, TYPE_ENERGY, TYPE_PART, ...), `update_fn` (optional),
`create_fn`, `change_type_fn`, `ctype_draw_fn`, `carries_ctype`.

Material properties are immutable per element id and shared (no per-particle
copy).

---

## 2. World Layers

The world supports the following independent layers (all owned by A1):

| Layer | Storage | Purpose |
|-------|---------|---------|
| Ordinary particles | `pmap` + `parts` | Matter (sand, water, ...) |
| Energy particles | `photons` + `parts` | Photons, neutrons, sparks, ... |
| Wall map | `bmap: Vec<u8>` (per CELL) | Wall type per 4x4 cell |
| Wall electricity | `emap: Vec<u8>` (per CELL) | Powered-wall state / detector countdown |
| Air pressure | `pv: Vec<f32>` (per CELL) | Pressure field |
| Air velocity | `vx, vy: Vec<f32>` (per CELL) | Velocity field |
| Ambient heat | `hv: Vec<f32>` (per CELL) | Ambient heat field |
| Gravity field | `grav_force, grav_mass` (per CELL) | Newtonian gravity |
| Glow | `glow: Vec<[f32;3]>` (per pixel) | Render glow |
| Decoration | frontend-owned `deco: Vec<u32>` | ARGB overlay |

Grid dimensions: `width x height` pixels, with `cells_x = width/CELL`,
`cells_y = height/CELL` (CELL = 4). All field arrays are sized to
`cells_x * cells_y`.

---

## 3. Wall Infrastructure

19 wall entries (derived from `SimulationData.cpp`). Constants live in
`realmatter-tpt::air` (WALL_*). Behaviours:

- `WALL_WALL` (0) — ordinary wall, blocks all particles.
- `WALL_WALLELEC` — electrically controlled wall (powered by ELEC).
- `WALL_EWALL` — E-Hole wall.
- `WALL_GRATE` / permeable variants — liquid/powder/gas/energy permeable.
- `WALL_ALLOWAIR`, `WALL_BLOCKAIR` — air-only / air-blocking.
- `WALL_STREAM` — streamline.
- `WALL_FAN` — fan (pushes air).
- `WALL_DETECT` — detector (sets emap countdown).
- `WALL_ABSORB` — absorbing.
- `WALL_STASIS` — freezes particles.
- `WALL_GRAV` — gravity-blocking.
- `WALL_ERASE`, `WALL_ERASEALL` — erasing tools.
- `WALL_DESTROYALL` — destroys all particles in cell.

A1 owns wall state/storage; A2 implements physical-field interaction with
walls; A3 implements electrically activated wall behaviour.

---

## 4. Tick Pipeline (v2)

Formal phase order, justified against `Simulation::UpdateParticles` and
`BeforeSim`/`AfterSim`:

1. **Input & mutation commands** — apply paint/erase/tool commands queued by
   frontend. (Frontend → WASM boundary.)
2. **World & field preparation** — `BeforeSim`: air update, ambient heat,
   Newtonian gravity, EMP decay, tick counter, recalc pmap + life decay, wall
   block maps, GoL.
3. **Air/pressure updates** — `update_air()` (reference `Air::update_air`).
4. **Gravity calculations** — `update_newtonian_gravity()` (if enabled).
5. **Ambient heat & thermal exchange** — `update_airh()`, conduction.
6. **Pressure/temperature transitions** — `transition_phase()` per particle.
7. **Element update functions** — `update_fn()` per particle (A3).
8. **Chemical/electrical reactions** — handled inside update fns + dedicated
   passes (A3).
9. **Particle movement & collision** — `movement_phase()` per particle, using
   `can_move` table.
10. **Lifetime/cleanup** — `recalc_and_life`, kill off-screen, wall kills.
11. **Render-state publication** — refresh `colour` buffer.

Reference entry point: `Sim::step()` calls `before_sim()` → `update_particles()`
→ `after_sim()`.

### Conflict policy

- Only the tick scheduler (A1) mutates particle storage during a tick.
- A2 (physics) and A3 (chemistry/elements) implement *functions* invoked by
  the scheduler; they must not independently mutate the same particle in
  conflicting phases. Phase boundaries are enforced by `Sim::step`.
- Deterministic RNG is owned by `Sim::rng` (A1). A2/A3 call `sim.rng.*`;
  they must not instantiate their own RNG.

---

## 5. World Read/Write API

A1 provides stable APIs for A2/A3 (see `realmatter-tpt::sim::Sim`):

- `create_part(x, y, t) -> Option<usize>` — create particle.
- `kill_part(i)` / `kill_part_reason(i, reason)` — destroy.
- `part_change_type(i, x, y, t) -> bool` — convert material.
- `paint(x, y, t, ctype)` / `paint_wall(x, y, w)` — tool entry points.
- `set_wall(x, y, w)` / `wall_at(x, y)` / `wall_blocks(...)` / `wall_kills(...)`.
- `neighbourhood(i) -> Neighbourhood` — 8-neighbor pmap + gravity.
- `pmap_at(x, y)`, `type_at(x, y)`, `id_at(x, y)`.
- `gravity_field(x, y, particle_grav, newton_grav) -> (f32, f32)`.

A2/A3 must use these; they must not reach into `parts`/`pmap` directly except
via the documented helpers.

---

## 6. Physics Update Interfaces (A2)

A2 implements, against this contract:

- `update_air()` — pressure/velocity field update (`Air::update_air`).
- `movement_phase(i)` — particle movement using `can_move` table.
- `transition_phase(i, n) -> bool` — phase transitions (melt/boil/condense).
- Gravity field computation (Newtonian).
- Thermal conduction.

These are called from `Sim::update_particles` and `Sim::before_sim`.

---

## 7. Chemistry Update Interfaces (A3)

A3 implements element update functions registered in `updates` module:

- `update_fn(t) -> Option<fn(&mut Sim, i, x, y, space, nt) -> i32>`
- `create_fn`, `change_type_fn`, `ctype_draw_fn`, `carries_ctype`.
- `legacy_update` for heat-sim-off mode.

A3 owns element definitions in `elements_generated.rs` and `element.rs`.

---

## 8. Deterministic RNG Ownership

- Single `Rng` instance owned by `Sim` (`sim.rng`).
- All probabilistic behaviour (diffusion, reactions) draws from it.
- Seeded deterministically (default seed fixed). Save/load must preserve seed
  state for reproducibility (TODO: persist rng state).

---

## 9. Persistence Version Strategy

- v2 save format is a new version (not v1). v1 (Cell-based) saves are not
  compatible; migration is not required for v2 (clean break) but the loader
  must reject v1 blobs with a clear error rather than misinterpret.
- v2 persists: `width, height, tick, rng state, parts, pmap/photons (derived),
  bmap, emap, pv, vx, vy, hv, settings, deco`.
- Version field in header; loader checks version and refuses unknown.

---

## 10. WASM Export Strategy

A1 wires `realmatter-tpt::Sim` into a WASM crate (`labsmatter-wasm`) exposing:

- `init(width, height) -> Result<(), JsValue>` (width/height must be divisible
  by CELL=4; default 256x256).
- `step(dt: f64)` — calls `sim.step()`.
- `set_cell(x, y, material_pt_id) -> bool` — paints a particle (PT id space).
- `get_cell_material(x, y) -> u16` — PT id at pixel (from pmap/photons).
- `get_cell_temperature(x, y) -> f32`.
- `world_width() -> u32`, `world_height() -> u32` (pixels).
- `reset() -> bool`.
- `render_buffer_ptr() -> *const u8`, `render_buffer_len() -> usize` (RGBA8
  per pixel, derived from `sim.colour`).
- `save_blob() -> Vec<u8>` / `load_blob(bytes) -> bool` (v2 format).
- Tool exports: `apply_tool(kind, x, y, radius, dx, dy)`.
- Mode exports: `set_gravity_mode`, `set_air_mode`, etc.
- Decoration: `set_deco`, `get_deco`, `set_deco_visible`, `clear_all_deco`.

Frontend adapter (`tpt.js`) maps frontend material ids ↔ PT ids via
`tpt-materials.js`.

---

## 11. Migration Notes

- The legacy `realmatter-wasm` (Cell-based) remains the shipping path until
  `labsmatter-wasm` reaches feature parity. Both may coexist during transition.
- No silent reinterpretation of material ids: frontend palette ids and PT ids
  are distinct spaces connected only by the adapter's explicit table.
- A1 owns integration; A2/A3 implement against this contract.
