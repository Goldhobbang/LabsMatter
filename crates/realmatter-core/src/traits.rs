//! Public module traits (interfaces) for REALMATTER.

use crate::cell::Cell;
use crate::error::Result;
use crate::material::MaterialId;
use crate::units::Time;
use crate::world::WorldConfig;

pub trait SimulationModule: Send + Sync {
    fn name(&self) -> &'static str;
    fn init(&mut self, config: &WorldConfig) -> Result<()>;
    fn step(&mut self, dt: Time) -> Result<()>;
    fn shutdown(&mut self) {}
}

pub trait PhysicsModule: SimulationModule {
    fn apply_forces(&mut self, cells: &mut [Cell], dt: Time) -> Result<()>;
    fn resolve_collisions(&mut self, cells: &mut [Cell], dt: Time) -> Result<()>;
}

pub trait ChemistryModule: SimulationModule {
    fn react(&mut self, cells: &mut [Cell], dt: Time) -> Result<()>;
    fn can_react(&self, a: MaterialId, b: MaterialId) -> bool;
}

pub trait RenderSink: Send + Sync {
    fn push_cell(&mut self, x: u32, y: u32, cell: &Cell);
    fn flush(&mut self);
}

pub trait SerializationModule: Send + Sync {
    fn serialize(&self, cells: &[Cell]) -> Result<Vec<u8>>;
    fn deserialize(&self, bytes: &[u8]) -> Result<Vec<Cell>>;
    fn version(&self) -> u32;
}