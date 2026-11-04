//! REALMATTER core: shared types, math, units, errors, and public traits.

pub mod cell;
pub mod error;
pub mod material;
pub mod math;
pub mod traits;
pub mod units;
pub mod world;

pub use cell::{Cell, cell_flags};
pub use error::{RealmatterError, Result};
pub use material::{MaterialId, MaterialProps, MaterialState, MaterialRegistry, MATERIAL_EMPTY, MATERIAL_WALL};
pub use math::Vec2;
pub use traits::{
    ChemistryModule, PhysicsModule, RenderSink, SerializationModule, SimulationModule,
};
pub use units::{Length, Mass, Temperature, Time};
pub use world::WorldConfig;