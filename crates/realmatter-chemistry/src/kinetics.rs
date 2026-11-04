//! Reaction kinetics (Arrhenius, temperature dependence).

use crate::schema::ReactionParams;
use crate::core::units::constants::GAS_CONSTANT;

/// Compute Arrhenius rate constant k = A * exp(-Ea/(R*T)).
pub fn arrhenius(params: &ReactionParams, temperature: f32) -> f32 {
    if temperature <= 0.0 {
        return 0.0;
    }
    let r = GAS_CONSTANT;
    let k = params.pre_exponential * (-(params.activation_energy / (r * temperature))).exp();
    if k.is_finite() { k * params.rate_multiplier } else { 0.0 }
}

/// Compute reaction rate for a unit amount.
pub fn rate(params: &ReactionParams, temperature: f32) -> f32 {
    if temperature < params.min_temp || temperature > params.max_temp {
        return 0.0;
    }
    if params.combustion && temperature < params.ignition_temp {
        return 0.0;
    }
    arrhenius(params, temperature)
}

/// Temperature rise from reaction enthalpy: dT = -dH/(m*c_p).
pub fn temperature_rise_from_enthalpy(enthalpy_j: f32, mass_kg: f32, specific_heat: f32) -> f32 {
    if mass_kg <= 0.0 || specific_heat <= 0.0 {
        return 0.0;
    }
    // exothermic negative enthalpy -> temperature rise
    (-enthalpy_j) / (mass_kg * specific_heat)
}
