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
}

pub trait MaterialRegistry: Send + Sync {
    fn get(&self, id: MaterialId) -> Option<&MaterialProps>;
    fn len(&self) -> usize;
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