# Contract: Material Registry

**Version:** 0.1.0
**Owner:** A1

## MaterialId

`pub type MaterialId = u16;` — 0 = empty/air, 1 = wall. Append-only; never reuse ids.

## MaterialProps

```rust
pub struct MaterialProps {
    pub id: MaterialId,
    pub name: &'static str,
    pub state: MaterialState, // Solid/Liquid/Gas/Plasma/Powder
    pub density: f32,
    pub specific_heat: f32,
    pub thermal_conductivity: f32,
    pub melting_point: f32,
    pub boiling_point: f32,
    pub viscosity: f32,
    pub color: [f32; 4],
    pub flammable: bool,
    pub ignition_temp: f32,
}
```

## MaterialRegistry trait

```rust
pub trait MaterialRegistry: Send + Sync {
    fn get(&self, id: MaterialId) -> Option<&MaterialProps>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool; // default
    fn name(&self, id: MaterialId) -> Option<&'static str>; // default
}
```

Implementations must be immutable after init.