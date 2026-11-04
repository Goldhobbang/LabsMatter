# Contract: Simulation Tick

**Version:** 1.0.0
**Owner:** A1 (spec) / A2 (impl)

## Fixed Timestep

- `FIXED_DT = 1/60 s` (f64).
- Accumulator pattern: leftover time carries over.
- `MAX_SUBSTEPS = 4` per frame to avoid spiral of death.

## Tick Phase Order (deterministic)

1. **Input** — apply user edits and external forces.
2. **Chemistry** — evaluate & apply reactions (A4).
3. **Physics: Forces** — apply forces (gravity, pressure) (A3).
4. **Physics: Collisions** — resolve constraints (A3).
5. **Movement** — cellular automaton cell updates (A2).
6. **Thermal** — heat diffusion (A3/A4).
7. **Finalize** — clear `UPDATED` flags, age lifetimes, remove dead cells.
8. **Export** — push render data to RenderSink.

## Module Interface

Each module implements `SimulationModule::step(dt)`. The scheduler calls
modules in registered order within each phase. Phase boundaries are enforced
by the scheduler (A2).

## Determinism

- All loops iterate in row-major, sorted order.
- No parallel mutation without deterministic reduction.
- `UPDATED` flag prevents double-processing a cell in one tick.
