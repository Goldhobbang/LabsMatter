# Contract: Core Types

**Version:** 0.1.0
**Owner:** A1

## Cell

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

Default: material=0, temperature=293.15, velocities 0, flags 0, lifetime 0, pressure 0.

## Cell flags (`cell_flags` module)

| Constant | Bit |
|----------|-----|
| BURNING | 1<<0 |
| WET | 1<<1 |
| FIXED | 1<<2 |
| UPDATED | 1<<3 |
| CHARGED | 1<<4 |
| DISSOLVED | 1<<5 |

## Math

```rust
pub struct Vec2 { pub x: f32, pub y: f32 }
```

## Units

- `Length = f32` (meters)
- `Mass = f32` (kg)
- `Time = f64` (seconds)
- `Temperature = f32` (Kelvin)

Constants: GRAVITY=9.80665, GAS_CONSTANT=8.314, FIXED_DT=1/60, MAX_SUBSTEPS=4, CELL_SIZE=0.01, ATM_PRESSURE=101325.0.