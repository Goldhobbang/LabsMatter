//! Material registry and material properties.

use serde::{Deserialize, Serialize};

pub type MaterialId = u16;

pub const MATERIAL_EMPTY: MaterialId = 0;
pub const MATERIAL_WALL: MaterialId = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaterialState {
    Solid,
    Liquid,
    Gas,
    Plasma,
    Powder,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaterialProps {
    pub id: MaterialId,
    pub name: String,
    pub state: MaterialState,
    pub density: f32,
    pub specific_heat: f32,
    pub thermal_conductivity: f32,
    pub melting_point: f32,
    pub boiling_point: f32,
    pub viscosity: f32,
    pub color: [f32; 4],
    pub flammable: bool,
    pub ignition_temp: f32,
    /// Air-coupling properties (reference `Element` fields).
    /// Fraction of local air velocity retained each tick (`AirLoss`).
    #[serde(default = "default_air_loss")]
    pub air_loss: f32,
    /// How much particle velocity is injected into the air field (`AirDrag`).
    #[serde(default)]
    pub air_drag: f32,
    /// How strongly the air field drags the particle (`Advection`).
    #[serde(default)]
    pub advection: f32,
    /// Particle velocity decay per tick (`Loss`).
    #[serde(default = "default_loss")]
    pub loss: f32,
    /// Random diffusion strength for gases (`Diffusion`).
    #[serde(default)]
    pub diffusion: f32,
    /// Pressure added to the air field per tick (`HotAir`).
    #[serde(default)]
    pub hot_air: f32,
    /// Conducts electricity (`PROP_CONDUCTS`).
    #[serde(default)]
    pub conducts: bool,
    /// Reference element weight (heavier sinks through lighter).
    #[serde(default = "default_weight")]
    pub weight: f32,
}

fn default_air_loss() -> f32 {
    1.0
}
fn default_loss() -> f32 {
    1.0
}
fn default_weight() -> f32 {
    50.0
}

pub trait MaterialRegistry: Send + Sync {
    fn get(&self, id: MaterialId) -> Option<&MaterialProps>;
    fn len(&self) -> usize;
    /// Enumerate all registered material properties (sorted by id is not
    /// required, but implementations should return a stable order).
    fn all_props(&self) -> Vec<MaterialProps> {
        Vec::new()
    }
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn name(&self, id: MaterialId) -> Option<&'static str> {
        self.get(id).map(|p| p.name.as_str()).map(|s| {
            // SAFETY: material names are interned for the lifetime of the registry
            // in practice; this is a temporary bridge until the registry owns
            // static strings. Do not use for dynamic names.
            unsafe { &*(s as *const str) }
        })
    }
}

/// Shared fallback props for unknown material ids. Returned by reference so
/// hot paths never clone (and never heap-allocate a `String`) on lookup.
fn fallback_props() -> &'static MaterialProps {
    use std::sync::OnceLock;
    static FALLBACK: OnceLock<MaterialProps> = OnceLock::new();
    FALLBACK.get_or_init(|| MaterialProps {
        id: u16::MAX,
        name: String::new(),
        state: MaterialState::Gas,
        density: 1.225,
        specific_heat: 1005.0,
        thermal_conductivity: 0.026,
        melting_point: f32::INFINITY,
        boiling_point: f32::INFINITY,
        viscosity: 0.0,
        color: [0.8, 0.9, 1.0, 0.3],
        flammable: false,
        ignition_temp: f32::INFINITY,
        air_loss: 1.0,
        air_drag: 0.0,
        advection: 0.0,
        loss: 1.0,
        diffusion: 0.0,
        hot_air: 0.0,
        conducts: false,
        weight: 50.0,
    })
}

/// Look up material props by id without cloning. Unknown ids yield a shared
/// air-like fallback. Hot simulation loops must use this instead of cloning
/// `MaterialProps` (which heap-allocates the `name` String per cell).
#[inline]
pub fn material_ref(materials: &[MaterialProps], id: MaterialId) -> &MaterialProps {
    match materials.get(id as usize) {
        Some(p) => p,
        None => fallback_props(),
    }
}