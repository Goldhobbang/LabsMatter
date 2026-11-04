# REALMATTER — Architecture Specification

**Version:** 0.1.0 (draft)
**Owner:** A1 (Systems Architect)
**Status:** Draft — pending A0 review

## 1. Overview

REALMATTER is a high-performance, browser-based, offline physics and chemistry sandbox. The simulation core is written in Rust and compiled to WebAssembly (WASM). The browser hosts the WASM module, a WebGPU renderer, and a web-based UI. There is no mandatory backend; all computation runs client-side.

### 1.1 Goals

- Deterministic, reproducible simulation.
- High throughput: target 60 FPS at 1M+ particles on mid-range hardware.
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
│   ├── realmatter-core/       # world, grid, chunks, particles, materials, tick
│   ├── realmatter-physics/    # forces, collisions, fluids, gases
│   ├── realmatter-chemistry/  # reactions, bonds, thermodynamics
│   ├── realmatter-renderer/   # WebGPU rendering, camera
│   ├── realmatter-ui/         # controls, panels, input
│   ├── realmatter-persist/    # save/load, serialization
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
  ├── realmatter-physics  -> realmatter-core
  ├── realmatter-chemistry -> realmatter-core, realmatter-physics
  ├── realmatter-renderer -> realmatter-core
  ├── realmatter-ui       -> realmatter-core, realmatter-renderer
  └── realmatter-persist  -> realmatter-core
```

Rule: `realmatter-core` has no dependencies on other realmatter crates. All cross-module communication goes through core's public traits.

## 5. World model

### 5.1 Coordinate system

- Right-handed, Y-up.
- Units: meters (m) for length, kilograms (kg) for mass, seconds (s) for time, Kelvin (K) for temperature.
- World is a bounded 3D (or 2D mode) axis-aligned box. Default world size: 256 × 256 × 256 cells (configurable at init).

### 5.2 Grid & chunks

- The world is a uniform grid of **cells**. Each cell holds at most one particle (or empty).
- Cell size: configurable, default 0.01 m (1 cm). World extent = cells × cell_size.
- Chunks: 16 × 16 × 16 cells (4096 cells/chunk). Chunks are the unit of:
  - Memory allocation / deallocation
  - Dirty tracking for rendering
  - Parallel scheduling
  - Serialization
- Chunk coordinates: `ChunkCoord { x: i32, y: i32, z: i32 }`.
- Cell coordinates: `CellCoord { x: i32, y: i32, z: i32 }` (world-absolute).
- Conversion: `chunk = floor(cell / 16)`, `local = cell mod 16`.

### 5.3 Particle representation

A particle is a struct stored in a chunk's cell array (or a sparse sidecar for extended data).

```rust
// realmatter-core::particle
pub struct Particle {
    pub material: MaterialId,        // u16
    pub temperature: f32,            // K
    pub velocity: [f32; 3],          // m/s
    pub flags: ParticleFlags,        // u16 bitfield
    pub lifetime: f32,              // s, 0 = immortal
    pub custom: u32,                // module-private packed data
}
```

- `MaterialId`: `u16`, index into the material registry. `0` = empty/void.
- `ParticleFlags`: bitfield (burning, conducting, frozen, etc.) — exact bits defined in contract.
- Extended per-particle data (bonds, charge) lives in sidecar arrays owned by chemistry/physics, keyed by cell index, to keep the hot particle struct small.

### 5.4 Material registry

- Materials are data-driven: loaded from a bundled JSON manifest at init.
- Each material has: id, name, density (kg/m³), thermal conductivity, specific heat, melting/boiling points, color, phase (solid/liquid/gas), reactivity tags.
- Registry is immutable after init (read-only during simulation). Hot path uses `MaterialId` indexing into a flat `&[Material]` slice.
- Material ids are stable across versions (append-only; deprecation via flags, never reuse ids).

## 6. Numerical precision & units

- Length/position: f32 (meters). Cell grid is integer; world-space derived.
- Velocity, acceleration: f32 (m/s, m/s²).
- Temperature: f32 (K).
- Mass/density: f32.
- Time: f64 accumulator for tick scheduling; per-tick delta is f32.
- Accumulators (energy, impulse) that sum many small values: f64 internally, cast to f32 at write.
- No `as any`-style casts that lose precision silently; conversions must be explicit and documented.
- Determinism: identical inputs + identical tick count ⇒ identical state (bitwise) on the same target. Cross-platform bitwise equality is best-effort (WASM single target).

## 7. Simulation tick

### 7.1 Fixed timestep

- Fixed dt = 1/60 s (≈0.0166667 s). Configurable at init within [1/120, 1/30].
- Accumulator pattern: real elapsed time accumulates; simulation advances in fixed steps; remainder carried over.
- Max steps per frame: 8 (spiral-of-death guard). Excess time is dropped with a warning.

### 7.2 Tick order (deterministic)

Within one tick, phases execute in this order:

1. **Input apply** — UI writes pending commands into a command queue.
2. **Command drain** — core applies queued commands (spawn, remove, set material) deterministically (sorted by insertion order).
3. **Physics pre-step** — forces, gravity, external fields computed.
4. **Chemistry pre-step** — reaction eligibility, bond updates.
5. **Particle update** — integrate velocities, move particles, resolve collisions (per-chunk, deterministic chunk order).
6. **Chemistry post-step** — reactions triggered by new contacts, temperature changes.
7. **Physics post-step** — constraints, cleanup.
8. **Chunk maintenance** — dirty flags, neighbor updates, deallocation of empty chunks.
9. **Render snapshot** — produce read-only render data for the renderer (double-buffered).
10. **Persistence checkpoint** (optional, throttled).

### 7.3 Parallelism

- Chunks are the parallel unit. Within a phase, chunks may be processed in parallel provided:
  - No cross-chunk writes without synchronization (use ghost/border cells).
  - Iteration order over chunks is deterministic (sorted by ChunkCoord).
  - Reductions use deterministic accumulation (sorted order).
- Border cells: each chunk keeps a 1-cell halo copied from neighbors at phase start; writes to halo are deferred and merged.

### 7.4 Determinism rules

- No `HashMap` iteration in simulation paths; use `BTreeMap` or sorted vectors.
- No floating-point reductions in unspecified order.
- RNG (if any) is seeded, deterministic, per-chunk seeded by chunk coord + tick.
- Thread count must not affect results.

## 8. Module lifecycle

### 8.1 Initialization sequence

1. `init(config)` — validate config, allocate world, load material registry.
2. `load_materials(manifest)` — parse & validate; fail fast on invalid.
3. `ready()` — simulation ready; ticks may begin.
4. (runtime) `tick(dt)` / `render()` loop.
5. `shutdown()` — flush persistence, free resources.

### 8.2 Error handling

See `docs/error-handling.md`. Summary: core returns `Result<T, SimError>`; WASM boundary converts to JS exceptions; no panics in release hot path (panic = abort in WASM).

### 8.3 Configuration

`SimConfig` (versioned) includes: world dimensions (cells), cell size, chunk size (fixed 16), fixed dt, max steps/frame, material manifest bytes, seed.

## 9. Interfaces (contracts)

All public traits live in `realmatter-core::traits` and are re-exported. See `docs/contracts/`.

Key traits:
- `World`: read/write access to cells, chunks, particles.
- `MaterialRegistry`: material lookup.
- `PhysicsModule`: `pre_step`, `post_step`, forces.
- `ChemistryModule`: `pre_step`, `post_step`, reactions.
- `Renderer`: consumes render snapshot.
- `Persistence`: save/load world state.
- `CommandQueue`: UI → core commands.

## 10. Serialization & compatibility

See `docs/serialization.md`. World state serialized as versioned binary blob. Material registry serialized by id (not by content). Backward compat: reader supports N-1 versions.

## 11. Performance budgets

See `docs/performance-budgets.md`. Summary: 60 FPS @ 1M particles; tick ≤ 8 ms; render ≤ 8 ms; memory ≤ 512 MB.

## 12. Versioning

- Architecture spec: semver. Breaking changes require A0 approval and migration notes.
- Contracts: each contract file has its own version header.