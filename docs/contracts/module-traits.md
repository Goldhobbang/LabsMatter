# Contract: Module Traits

**Version:** 0.1.0
**Owner:** A1

## SimulationModule

```rust
pub trait SimulationModule: Send + Sync {
    fn name(&self) -> &'static str;
    fn init(&mut self, config: &WorldConfig) -> Result<()>;
    fn step(&mut self, dt: Time) -> Result<()>;
    fn shutdown(&mut self) {}
}
```

## PhysicsModule

```rust
pub trait PhysicsModule: SimulationModule {
    fn apply_forces(&mut self, cells: &mut [Cell], dt: Time) -> Result<()>;
    fn resolve_collisions(&mut self, cells: &mut [Cell], dt: Time) -> Result<()>;
}
```

## ChemistryModule

```rust
pub trait ChemistryModule: SimulationModule {
    fn react(&mut self, cells: &mut [Cell], dt: Time) -> Result<()>;
    fn can_react(&self, a: MaterialId, b: MaterialId) -> bool;
}
```

## RenderSink

```rust
pub trait RenderSink: Send + Sync {
    fn push_cell(&mut self, x: u32, y: u32, cell: &Cell);
    fn flush(&mut self);
}
```

## SerializationModule

```rust
pub trait SerializationModule: Send + Sync {
    fn serialize(&self, cells: &[Cell]) -> Result<Vec<u8>>;
    fn deserialize(&self, bytes: &[u8]) -> Result<Vec<Cell>>;
    fn version(&self) -> u32;
}
```