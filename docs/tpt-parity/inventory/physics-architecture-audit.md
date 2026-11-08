# Physics Architecture Audit — Legacy vs TPT Runtime

Reference: The-Powder-Toy/The-Powder-Toy @ `2b7221a46d378589b4de7683f16d3ebc92be6bd9`

## Summary

The authoritative runtime is `crates/realmatter-tpt/` (particle-list + pmap model).
The legacy `crates/realmatter-physics/` (cell-based) and `crates/realmatter-wasm/` remain
the shipping path until v2 reaches parity. Physics algorithms must ultimately affect
`realmatter-tpt::Sim` state; duplicate implementations are documented below.

## Module audit

| Module | Legacy location | Legacy uses? | TPT location | TPT uses? | Duplicate? | Missing TPT integration | Proposed authoritative impl | Required A1 interface |
|---|---|---|---|---|---|---|---|---|
| Air solver | `realmatter-physics/src/air.rs` (AirField) | yes (wasm step) | `realmatter-tpt/src/air.rs` (Sim fields) | yes (before_sim) | yes (two AirField impls) | convection modes already in tpt; ambient heat in tpt | keep tpt as authoritative; reuse algorithms | none |
| Gravity (Newtonian) | `realmatter-physics/src/gravity.rs` (Jacobi) | yes (wasm step) | `realmatter-tpt/src/sim.rs` (update_newtonian_gravity) | yes (before_sim) | yes (two Jacobi solvers) | gravity blocking mask (grav_mask) in tpt | keep tpt; document Jacobi deviation | none |
| Gravity modes | `realmatter-physics/src/gravity.rs` (gravity_vector) | no | `realmatter-tpt/src/sim.rs` (gravity_field) | yes | partial | radial/custom already in tpt | keep tpt | none |
| Movement/CA | `realmatter-world/src/movement.rs` | yes (wasm step) | `realmatter-tpt/src/movement.rs` | yes (update_particles) | yes (two movement systems) | elasticity/restitution partial | keep tpt particle movement | none |
| Heat conduction | `realmatter-physics/src/heat.rs` (conduct) | yes (wasm step) | none (ambient-air based) | n/a | no | tpt uses ambient air heat, not particle conduction | tpt ambient model is reference-faithful | none |
| Phase transitions | `realmatter-physics/src/heat.rs` | yes | `realmatter-tpt/src/transition.rs` | yes | yes | pressure-dependent transitions in tpt? check | keep tpt transition_phase | none |
| Gas diffusion/buoyancy | `realmatter-physics/src/gas.rs` | yes | `realmatter-tpt` element updates | partial | yes | — | keep tpt | none |
| Fluid pressure/viscosity | `realmatter-physics/src/fluid.rs` | yes | `realmatter-tpt` movement | partial | yes | — | keep tpt | none |
| Stability/sanitize | `realmatter-physics/src/stability.rs` | yes | none (tpt has restrict_flt) | partial | no | tpt lacks sanitize pass | add sanitize to tpt before_sim | A1: call sanitize in before_sim |
| Energy transport | `realmatter-physics/src/energy.rs` | no | `realmatter-tpt/src/movement.rs` (move_energy) | yes | partial | refraction/scattering missing | add primitives to tpt | none |
| Shock/pressure impulses | none | no | `Sim::add_pressure` (new) | available | no | element callers (A3) | expose API | A3: call add_pressure |
| Wall permeability | `realmatter-physics` (rebuild_blockair) | yes | `realmatter-tpt` (bmap_blockair) | yes | yes | per-wall permeability partial | keep tpt | none |
| Sim tools | `realmatter-wasm` (apply_tool) | yes | `labsmatter-wasm` (none) | no | yes | tools not exposed in tpt wasm | A1: add apply_tool to labsmatter-wasm | A1: implement |

## Negative pressure correction

- `realmatter-physics/src/stability.rs`: previously clamped `cell.pressure` to `[0, 256]`. Corrected to sanitize only non-finite values, preserving signed range. Thermodynamic pressure (Pa) and gauge air pressure (±256) are distinct; magnitude clamping belongs to the air-field layer (`AirField` already clamps `pv` to `[-256, 256]`).
- `realmatter-tpt`: already uses signed `MIN_PRESSURE..MAX_PRESSURE` for `pv`; no change needed.

## TPT integration status

- Air/ambient heat/convection: already in tpt (`update_air`, `update_airh`, `ConvectionMode`).
- Newtonian gravity: in tpt (`update_newtonian_gravity`), Jacobi approximation.
- Movement: in tpt (`movement_phase`, `can_move` table).
- Phase transitions: in tpt (`transition_phase`).
- Energy particles: in tpt (`move_energy`).
- New physics APIs added: `Sim::add_pressure`, `Sim::add_air_velocity`, `Sim::wall_allows_air`, `Sim::wall_allows_heat`.

## Required Agent 1 changes

1. Call a sanitize pass in `Sim::before_sim` (or expose for A2 to implement).
2. Add `apply_tool` export to `labsmatter-wasm` mirroring `realmatter-wasm`.
3. Persist rng state in v2 save format (currently TODO).
4. Wire `Sim::add_pressure` callers from element updates (A3).

## Required Agent 3 changes

1. Use `Sim::add_pressure` / `add_air_velocity` for explosive/force elements.
2. Provide restitution/friction material properties if needed (currently `collision` field).
3. Coordinate phase-transition products with `transition_phase`.
