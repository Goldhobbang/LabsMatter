//! REALMATTER chemistry: reactions, bonds, thermodynamics.
//!
//! Owned by A4. Depends on realmatter-core and realmatter-physics.
//! Provides a data-driven reaction registry, schema/validation, and
//! evaluation engine that operates on the core Cell model (2D).

pub use realmatter_core as core;
pub use realmatter_physics as physics;

pub mod bonds;
pub mod engine;
pub mod registry;
pub mod schema;
pub mod thermo;
pub mod validation;
pub mod kinetics;

pub use engine::ChemistryEngine;
pub use registry::{ReactionDefinition, ReactionRegistry};
pub use schema::{ReactionId, ReactionSchema};
