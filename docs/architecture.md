# LabsMatter — Architecture Specification

> Historical note: this document was authored under the project's original
> name, REALMATTER. Internal crate names still use the `realmatter-` prefix.
> The product name is **LabsMatter**.

**Version:** 0.1.0 (draft)
**Owner:** A1 (Systems Architect)
**Status:** Draft — pending A0 review

## 1. Overview

REALMATTER is a high-performance, browser-based, offline physics and chemistry sandbox. The simulation core is written in Rust and compiled to WebAssembly (WASM). The browser hosts the WASM module, a WebGPU renderer, and a web-based UI. There is no mandatory backend; all computation runs client-side.

### 1.1 Goals

- Deterministic, reproducible simulation.
- High throughput on mid-range hardware.
- Offline-first: no network dependency at runtime.
- Extensible material/reaction system via data-driven registries.
- Stable, versioned public contracts between modules.

### 1.2 Non-goals (v1)

- Multiplayer / networked simulation.
- Server-side persistence.
- Photorealistic rendering.

## 2. Technology stack

| Layer | Technology |
|-------|------------|
| Simulation core | Rust (stable) |
| WASM bridge | wasm-bindgen, wasm-pack |
| Renderer | WebGPU (wgpu crate) |
| UI | egui (web) or HTML/JS overlay (TBD by A6) |
| Build | Cargo workspace, trunk/wasm-pack |
| Serialization | Custom binary format (versioned) + optional JSON for materials |
| Storage | Browser IndexedDB (via A7) |

## 3. Workspace layout

```
realmatter/
├── Cargo.toml                 # workspace root
├── rust-toolchain.toml
├── crates/
│   ├── realmatter-core/       # shared types, math, units, traits, errors
│   ├── realmatter-world/      # grid, chunks, particles, tick scheduler (A2)
│   ├── realmatter-physics/    # forces, collisions, fluids, gases (A3)
│   ├── realmatter-chemistry/  # reactions, bonds, thermodynamics (A4)
│   ├── realmatter-renderer/   # WebGPU rendering, camera (A5)
│   ├── realmatter-ui/         # controls, panels, input (A6)
│   ├── realmatter-persist/    # save/load, serialization (A7)
│   └── realmatter-wasm/       # wasm-bindgen entrypoint (integration)
├── docs/
├── communication/
└── web/                       # HTML/JS/CSS shell (A5/A6)
```

## 4. Module dependency graph

See `docs/module-dependency-graph.md`.

Summary (acyclic, layered):

```
realmatter-wasm
  ├── realmatter-core
  ├── realmatter-world        -> realmatter-core
  ├── realmatter-physics      -> realmatter-core, realmatter-world
  ├── realmatter-chemistry    -> realmatter-core, realmatter-world, realmatter-physics
  ├── realmatter-renderer     -> realmatter-core
  ├── realmatter-ui           -> realmatter-core, realmatter-renderer
  └── realmatter-persist      -> realmatter-core
```

Rule: `realmatter-core` has no dependencies on other realmatter crates. All cross-module communication goes through core's public traits.

## 5. World model

### 5.1 Coordinate system

- 2D grid (x, y). Right-handed, Y-up in render space.
- Units: meters (m) for length, kilograms (kg) for mass, seconds (s) for time, Kelvin (K) for temperature.
- World is a bounded 2D grid of cells. Default world size: 512 × 384 cells (configurable at init).

### 5.2 Grid & chunks

- The world is a uniform grid of **cells**. Each cell holds one material plus per-cell state.
- Cell size: configurable, default 0.01 m (1 cm). World extent = cells × cell_size.
- Chunks: 32 × 32 cells. Chunks are the unit of memory allocation, dirty tracking, parallel scheduling, serialization.
- Chunk coordinates: `ChunkCoord { x: i32, y: i32 }`.
- World config: `WorldConfig { width, height, chunk_size, fixed_dt }`.

### 5.3 Cell representation

A cell is a struct (logical contract; storage may be SoA internally):

```rust
pub struct Cell {
    pub material: MaterialId,   // u16
    pub temperature: f32,       // K
    pub vx: f32,                // m/s
    pub vy: f32,                // m/s
    pub flags: u8,
    pub lifetime: f32,          // s; 0 = permanent
    pub pressure: f32,          // Pa
}
```

- `MaterialId`: `u16`, index into the material registry. `0` = empty/void.
- Cell flags: bitfield (burning, wet, fixed, updated, charged, dissolved) — see `cell_flags` module.
- Extended per-cell data lives in sidecar arrays owned by chemistry/physics.

### 5.4 Material registry

- Materials are data-driven: loaded from a bundled manifest at init.
- Each material has: id, name, density, specific heat, thermal conductivity, melting/boiling points, viscosity, color, state, flammability, ignition temp.
- Registry is immutable after init. Hot path uses `MaterialId` indexing.
- Material ids are stable across versions (append-only; deprecation via flags, never reuse ids).
- Reserved ids: 0 = empty/air, 1 = wall.

## 6. Numerical precision & units

- Length/position: f32 (meters). Grid is integer; world-space derived.
- Velocity, acceleration: f32 (m/s, m/s²).
- Temperature: f32 (K).
- Time: f64 accumulator for tick scheduling; per-tick delta is f64 (Time).
- Mass/density: f32.
- Accumulators that sum many small values: f64 internally, cast to f32 at write.
- Determinism: identical inputs + identical tick count ⇒ identical state (bitwise) on the same target.

## 7. Simulation tick

### 7.1 Fixed timestep

- Fixed dt = 1/60 s. Configurable at init.
- Accumulator pattern: real elapsed time accumulates; simulation advances in fixed steps; remainder carried over.
- Max sub-steps per frame: 4 (spiral-of-death guard).

### 7.2 Tick order (deterministic)

Within one tick, phases execute in this order:

1. Input apply — UI writes pending commands.
2. Command drain — core applies queued commands deterministically.
3. Physics pre-step — forces, gravity, external fields.
4. Chemistry pre-step — reaction eligibility, bond updates.
5. Particle/cell update — integrate velocities, move, resolve collisions.
6. Chemistry post-step — reactions triggered by new contacts.
7. Physics post-step — constraints, cleanup.
8. Chunk maintenance — dirty flags, neighbor updates, deallocation.
9. Render snapshot — produce read-only render data (double-buffered).

### 7.3 Determinism rules

- No `HashMap` iteration in simulation paths; use `BTreeMap` or sorted vectors.
- No floating-point reductions in unspecified order.
- RNG (if any) is seeded, deterministic.
- Thread count must not affect results.

## 8. Module lifecycle

### 8.1 Initialization sequence

1. `init(config)` — validate config, allocate world, load material registry.
2. `load_materials(manifest)` — parse & validate; fail fast on invalid.
3. `ready()` — simulation ready; ticks may begin.
4. (runtime) `step(dt)` loop.
5. `shutdown()` — flush persistence, free resources.

### 8.2 Error handling

See `docs/error-handling.md`. Summary: core returns `Result<T, RealmatterError>`; WASM boundary converts to JS exceptions; no panics in release hot path.

### 8.3 Configuration

`WorldConfig` (versioned) includes: width, height (cells), chunk_size, fixed_dt.

## 9. Interfaces (contracts)

All public traits live in `realmatter-core::traits`. See `docs/contracts/`.

Key traits:
- `SimulationModule`: name, init, step, shutdown.
- `PhysicsModule`: apply_forces, resolve_collisions.
- `ChemistryModule`: react, can_react.
- `RenderSink`: push_cell, flush.
- `SerializationModule`: serialize, deserialize, version.
- `MaterialRegistry`: get, len, name.

## 10. Serialization & compatibility

See `docs/serialization.md`. World state serialized as versioned binary blob. Material registry serialized by id.

## 11. Performance budgets

See `docs/performance-budgets.md`.

## 12. Versioning

- Architecture spec: semver. Breaking changes require A0 approval and migration notes.
- Contracts: each contract file has its own version header.