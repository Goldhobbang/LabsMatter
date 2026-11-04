# Contract: Particle Representation

**Version:** 1.0.0
**Owner:** A1 (spec) / A2 (impl)

## In-Grid Particles

The base simulation stores one `Cell` per grid position. There is no separate
particle list for the grid. A cell's `material` identifies the substance;
per-cell state (temperature, velocity, flags, lifetime, pressure) lives in
`Cell`.

## Cell Struct (logical)

See `realmatter_core::cell::Cell`:

```
pub struct Cell {
    pub material: MaterialId,   // u16
    pub temperature: f32,       // Kelvin
    pub vx: f32,                // m/s
    pub vy: f32,                // m/s
    pub flags: u8,              // see cell_flags
    pub lifetime: f32,          // seconds; 0 = permanent
    pub pressure: f32,          // Pa
}
```

Size: 24 bytes (with padding). SoA storage recommended.

## Cell Flags

See `realmatter_core::cell::cell_flags`:
`BURNING, WET, FIXED, UPDATED, CHARGED, DISSOLVED`.

## Rigid Bodies (future)

Rigid bodies are out of scope for v0.1.0. When added, they will use a separate
list and interact with the grid via collision callbacks. Contract TBD.

## Empty Cell

`material == 0` (MATERIAL_EMPTY) represents air/empty space.
