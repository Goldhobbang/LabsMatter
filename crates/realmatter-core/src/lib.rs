//! REALMATTER core: shared types, math, units, errors, and public traits.
//!
//! Owned by A2 (Simulation Core). Implements the approved 2D contracts in
//! `docs/contracts/` (ADR-0003: 2D grid model).

pub mod cell;
pub mod chunk;
pub mod chunk_mask;
pub mod coord;
pub mod error;
pub mod material;
pub mod math;
pub mod particle;
pub mod traits;
pub mod tpt_elements;
pub mod units;
pub mod world_config;

pub use cell::{Cell, cell_flags};
pub use chunk::{Chunk, CHUNK_EDGE, CHUNK_VOLUME};
pub use chunk_mask::ActiveMask;
pub use coord::{CellCoord, ChunkCoord, CHUNK_SIZE};
pub use error::{RealmatterError, Result};
pub use material::{MaterialId, MaterialProps, MaterialState, MaterialRegistry, MATERIAL_EMPTY, MATERIAL_WALL};
pub use math::Vec2;
pub use traits::{ChemistryModule, PhysicsModule, RenderSink, SerializationModule, SimulationModule};
pub use units::{Length, Mass, Temperature, Time};
pub use world_config::WorldConfig;
