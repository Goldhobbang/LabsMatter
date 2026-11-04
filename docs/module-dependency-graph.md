# REALMATTER — Module Dependency Graph

**Version:** 0.1.0
**Owner:** A1

## Crates

| Crate | Role | Agent |
|-------|------|-------|
| realmatter-core | World, grid, chunks, particles, materials, tick, traits | A2 |
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
  ├─ realmatter-physics      -> realmatter-core
  ├─ realmatter-chemistry    -> realmatter-core, realmatter-physics
  ├─ realmatter-renderer     -> realmatter-core
  ├─ realmatter-ui           -> realmatter-core, realmatter-renderer
  └─ realmatter-persist      -> realmatter-core

realmatter-core               -> (no realmatter deps; only external crates)
realmatter-physics            -> realmatter-core
realmatter-chemistry          -> realmatter-core, realmatter-physics
realmatter-renderer          -> realmatter-core
realmatter-ui                -> realmatter-core, realmatter-renderer
realmatter-persist           -> realmatter-core
```

## Rules

1. `realmatter-core` MUST NOT depend on any other realmatter crate. It defines the shared traits and types.
2. No crate may depend on `realmatter-wasm` (it is the leaf integration crate).
3. No circular dependencies.
4. Cross-module communication occurs only through core's public traits (`World`, `MaterialRegistry`, `CommandQueue`, etc.).
5. Physics and chemistry may read particle data via `World`; they must not mutate particles except through documented mutation APIs.
6. Renderer and UI are read-only consumers of the render snapshot; they never mutate simulation state.
7. Persistence reads/writes world state only through the `Persistence` trait and serialization format.

## External dependencies (approved)

- `wasm-bindgen`, `js-sys`, `web-sys`
- `wgpu` (WebGPU)
- `glam` (math) — or `nalgebra`; decision pending A1 ADR
- `serde` (for material manifests / config only, not hot path)
- `bincode` or custom binary (serialization) — decision pending
- `rayon` (WASM threads) — optional, feature-gated
- `egui` + `egui-wgpu` + `egui-winit` (UI) — pending A6

## Forbidden

- `std::collections::HashMap` in simulation hot paths (use BTreeMap or sorted Vec).
- `println!` / `dbg!` in release hot path.
- `unsafe` outside audited modules (core memory, SIMD). All `unsafe` requires A1 review.