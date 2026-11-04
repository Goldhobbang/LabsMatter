//! Reaction schema definitions.

use crate::core::material::MaterialId;
use serde::{Deserialize, Serialize};

pub type ReactionId = u32;

/// Reaction equation (stoichiometric, simplified).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReactionEquation {
    /// Reactant material ids and coefficients.
    pub reactants: Vec<(MaterialId, u32)>,
    /// Product material ids and coefficients.
    pub products: Vec<(MaterialId, u32)>,
}

/// Reaction parameters.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReactionParams {
    /// Activation energy (J/mol).
    pub activation_energy: f32,
    /// Pre-exponential factor A.
    pub pre_exponential: f32,
    /// Enthalpy change (J) per reaction; negative = exothermic.
    pub enthalpy: f32,
    /// Rate multiplier.
    pub rate_multiplier: f32,
    /// Minimum temperature for reaction (K).
    pub min_temp: f32,
    /// Maximum temperature (K) or f32::INFINITY.
    pub max_temp: f32,
    /// Requires ignition for combustion.
    pub combustion: bool,
    /// Ignition temperature (K).
    pub ignition_temp: f32,
    /// Reaction occurs in presence of water (WET flag).
    pub requires_wet: bool,
}

/// Full reaction schema.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReactionSchema {
    pub id: ReactionId,
    pub name: String,
    pub equation: ReactionEquation,
    pub params: ReactionParams,
    pub tags: u32,
}

impl Default for ReactionParams {
    fn default() -> Self {
        Self {
            activation_energy: 10000.0,
            pre_exponential: 1.0,
            enthalpy: -1000.0,
            rate_multiplier: 1.0,
            min_temp: 273.15,
            max_temp: f32::INFINITY,
            combustion: false,
            ignition_temp: f32::INFINITY,
            requires_wet: false,
        }
    }
}
