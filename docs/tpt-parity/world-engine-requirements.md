# World-Engine Requirements Inventory (Agent 1)

**Reference:** The Powder Toy `2b7221a46d378589b4de7683f16d3ebc92be6bd9`
**Owner:** A1
**Status:** K0/K1 baseline. Implementation status reflects the `realmatter-tpt` engine
plus the new `labsmatter-wasm` binding.

| # | System | Source reference | Planned representation | Owner | Dependencies | Status |
|---|--------|------------------|------------------------|-------|--------------|--------|
| 1 | Particle (ordinary) | `src/simulation/Particle.h`, `Simulation.h` | `realmatter_tpt::particle::Particle` (t, life, ctype, x, y, vx, vy, temp, tmp..tmp4, flags, dcolour) | A1 | — | Implemented |
| 2 | Particle (energy) | same | Overlapping `photons: Vec<u32>` map + shared `parts` | A1 | #1 | Implemented |
| 3 | Occupancy map | `Simulation::pmap` | `pmap: Vec<u32>` packing `(idx<<9)|type` | A1 | #1 | Implemented |
| 4 | Wall map | `SimulationData.cpp`, `WallType.h` | `bmap: Vec<u8>` per CELL (4x4 px) | A1 | — | Implemented (19 walls) |
| 5 | Wall electricity | `Simulation::emap` | `emap: Vec<u8>` per CELL | A1/A3 | #4 | Implemented |
| 6 | Air pressure field | `Air.h`, `Air.cpp` | `pv: Vec<f32>` per CELL | A1/A2 | — | Implemented |
| 7 | Air velocity field | `Air.h` | `vx, vy: Vec<f32>` per CELL | A1/A2 | — | Implemented |
| 8 | Ambient heat field | `Simulation::hv` | `hv: Vec<f32>` per CELL | A1/A2 | — | Implemented |
| 9 | Gravity field | `gravity/` | `grav_force, grav_mass` per CELL | A1/A2 | — | Implemented (approx.) |
| 10 | Glow buffer | `Renderer.cpp` | `glow: Vec<[f32;3]>` per pixel | A1 | — | Allocated |
| 11 | Decoration layer | `Sign.cpp` | Frontend-owned `deco: Vec<u32>` | A1/A5 | — | Implemented (wasm) |
| 12 | Element registry | `ElementDefs.h`, `SimulationData.cpp` | `elements_generated` (PT_NUM slots) | A3 | — | Implemented (generated) |
| 13 | Material properties | `Element.h` | `ElementDef` (colour, weight, heat, ...) | A3 | #12 | Implemented |
| 14 | Tick scheduler | `Simulation::UpdateParticles`, `BeforeSim` | `Sim::step()` (before_sim → update_particles → after_sim) | A1 | #1-#11 | Implemented |
| 15 | RNG | `Probability.h` | `Sim::rng: Rng` (single owner) | A1 | — | Implemented |
| 16 | Particle lifecycle | `Simulation::kill_part`, `create_part` | `create_part`, `kill_part`, `part_change_type`, `recalc` | A1 | #1, #3 | Implemented |
| 17 | Movement/collision | `Simulation::eval_move` | `movement_phase`, `can_move` table | A2 | #1, #3, #4 | Implemented |
| 18 | Phase transitions | element `.cpp` | `transition_phase` | A2/A3 | #1 | Implemented (partial) |
| 19 | Chemistry/reactions | element `.cpp` | `update_fn` dispatch | A3 | #1, #14 | Partial |
| 20 | Electricity | `ELEC.cpp`, `SPRK.cpp` | spark propagation (wasm step_spark legacy) | A3 | #1, #4 | Partial |
| 21 | Persistence (v2) | `GameSave.cpp` | v2 blob (parts, pmap-derived, bmap, emap, fields, settings, deco) | A1 | #1-#11 | Not started |
| 22 | WASM binding | `wasm-bindgen` | `labsmatter-wasm` crate | A1 | #14 | Implemented (minimal) |
| 23 | Frontend adapter | `web/src/adapters` | `tpt.js` + `tpt-materials.js` (pending) | A1/A5 | #22 | Not started |

### Notes

- The legacy `realmatter-wasm` (Cell-based) remains the shipping path until
  `labsmatter-wasm` reaches feature parity.
- `realmatter-tpt` is the fundamental engine; `labsmatter-wasm` is its browser
  binding. Both compile as of this milestone.
- No new test cases were created (per policy); verification scenarios are
  documented in `docs/tpt-parity/qa/`.
