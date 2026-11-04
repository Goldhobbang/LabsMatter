# Contract: Units & Precision

**Version:** 1.0.0
**Owner:** A1

## Units

SI-derived. See `realmatter_core::units`:
- Length: meters (cell = 0.01 m).
- Mass: kilograms.
- Time: seconds.
- Temperature: Kelvin.

## Precision

- Spatial state (position, velocity, force): `f32`.
- Time accumulation: `f64`.
- Temperature, pressure, density: `f32`.

## Constants

- `GRAVITY = 9.80665 m/s^2`
- `GAS_CONSTANT = 8.314462618 J/(mol K)`
- `ATM_PRESSURE = 101325 Pa`
- `CELL_SIZE = 0.01 m`

## Floating Point Rules

- No `-ffast-math` / reassociation.
- Accumulation in deterministic order (sorted).
- Comparisons use epsilon where documented.
