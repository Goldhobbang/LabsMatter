# Contract: Module Interfaces

**Version:** 1.0.0
**Owner:** A1

## Traits (in realmatter_core::traits)

- `SimulationModule`: name, init, step, shutdown.
- `PhysicsModule`: extends SimulationModule; apply_forces, resolve_collisions.
- `ChemistryModule`: extends SimulationModule; react, can_react.
- `RenderSink`: push_cell, flush.
- `SerializationModule`: serialize, deserialize, version.

## Rules

- Modules must not hold references to other modules' internals; communicate
  via the shared `World`/cell slices passed by the scheduler.
- A module may read any cell; writes are scoped to its phase.
- Cross-module data flows through `Cell` fields and `MaterialProps`.

## Error Propagation

All fallible operations return `Result<T, RealmatterError>`. Modules must not
panic on expected runtime conditions.
