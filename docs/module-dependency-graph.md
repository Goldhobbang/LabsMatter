# REALMATTER — Module Dependency Graph

**Version:** 0.1.0
**Owner:** A1

## Crates

| Crate | Role | Agent |
|-------|------|-------|
| realmatter-core | Shared types, math, units, traits, errors | A1 |
| realmatter-world | Grid, chunks, particles, tick scheduler | A2 |
| realmatter-physics | Forces, collisions, fluids, gases | A3 |
| realmatter-chemistry | Reactions, bonds, thermodynamics | A4 |
| realmatter-renderer | WebGPU rendering, camera | A5 |
| realmatter-ui | Controls, panels, input | A6 |
| realmatter-persist | Save/load, serialization | A7 |
| realmatter-wasm | wasm-bindgen entrypoint, JS glue | A1 (scaffold) + all |

## Dependency edges (allowed)

```
realmatter-wasm
  ├─ realmatter-core
  ├─ realmatter-world        -> realmatter-core
  ├─ realmatter-physics      -> realmatter-core, realmatter-world
  ├─ realmatter-chemistry    -> realmatter-core, realmatter-world, realmatter-physics
  ├─ realmatter-renderer     -> realmatter-core
  ├─ realmatter-ui           -> realmatter-core, realmatter-renderer
  └─ realmatter-persist      -> realmatter-core

realmatter-core               -> (no realmatter deps; only external crates)
realmatter-world              -> realmatter-core
realmatter-physics            -> realmatter-core, realmatter-world
realmatter-chemistry          -> realmatter-core, realmatter-world, realmatter-physics
realmatter-renderer          -> realmatter-core
realmatter-ui                -> realmatter-core, realmatter-renderer
realmatter-persist           -> realmatter-core
```

## Rules

1. `realmatter-core` MUST NOT depend on any other realmatter crate. It defines the shared traits and types.
2. No crate may depend on `realmatter-wasm` (it is the leaf integration crate).
3. No circular dependencies.
4. Cross-module communication occurs only through core's public traits.
5. Physics and chemistry may read cell data via world; they must not mutate except through documented mutation APIs.
6. Renderer and UI are read-only consumers of render data; they never mutate simulation state.
7. Persistence reads/writes world state only through the serialization format.

## External dependencies (approved)

- `wasm-bindgen`, `js-sys`, `web-sys`
- `wgpu` (WebGPU)
- `glam` (math)
- `serde` (manifests/config only, not hot path)
- `bincode` (serialization)
- `rayon` (WASM threads) — optional, feature-gated
- `egui` + `egui-wgpu` + `egui-winit` (UI) — pending A6

## Forbidden

- `std::collections::HashMap` in simulation hot paths (use BTreeMap or sorted Vec).
- `println!` / `dbg!` in release hot path.
- `unsafe` outside audited modules. All `unsafe` requires A1 review.