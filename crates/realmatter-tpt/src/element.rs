//! Element definitions, mirroring `Element` from The Powder Toy.
//!
//! The table in [`crate::elements_generated`] is generated from the pinned
//! reference source; this module only defines the shape of a definition and
//! the helpers that the simulation uses to query it.

use crate::elements_generated::{ELEMENTS, PT_NUM};

/// Element definition. Field-for-field compatible with the reference
/// `Element` struct for the properties the simulation consumes.
#[derive(Clone, Copy, Debug)]
pub struct ElementDef {
    pub id: u16,
    pub abbr: &'static str,
    pub name: &'static str,
    pub identifier: &'static str,
    pub description: &'static str,
    /// Base colour, `0xRRGGBB` (sRGB).
    pub colour: u32,
    pub menu_visible: bool,
    pub menu_section: u8,
    pub enabled: bool,

    pub advection: f32,
    pub air_drag: f32,
    pub air_loss: f32,
    pub loss: f32,
    pub collision: f32,
    pub gravity: f32,
    pub newtonian_gravity: f32,
    pub diffusion: f32,
    pub hot_air: f32,
    /// 0 = does not fall, 1 = powder, >= 2 = liquid.
    pub falldown: u8,

    pub flammable: i32,
    pub explosive: i32,
    pub meltable: i32,
    pub hardness: i32,
    pub photon_reflect: u32,
    pub weight: i32,
    pub heat_conduct: u8,
    pub heat_capacity: f32,

    pub properties: u32,
    pub carries_type_in: u32,

    pub low_pressure: f32,
    pub low_pressure_transition: i32,
    pub high_pressure: f32,
    pub high_pressure_transition: i32,
    pub low_temperature: f32,
    pub low_temperature_transition: i32,
    pub high_temperature: f32,
    pub high_temperature_transition: i32,

    /// `DefaultProperties.temp`.
    pub default_temp: f32,

    pub has_update: bool,
    pub has_graphics: bool,
}

/// Placeholder for an element slot that exists in the reference ID space but
/// has no element (the single disabled slot at ID 146, and all IDs above the
/// reference maximum).
pub const EMPTY_ELEMENT: ElementDef = ElementDef {
    id: 0,
    abbr: "",
    name: "",
    identifier: "",
    description: "",
    colour: 0xFF00FF,
    menu_visible: false,
    menu_section: 0,
    enabled: false,
    advection: 0.0,
    air_drag: 0.0,
    air_loss: 1.0,
    loss: 1.0,
    collision: 0.0,
    gravity: 0.0,
    newtonian_gravity: 1.0,
    diffusion: 0.0,
    hot_air: 0.0,
    falldown: 0,
    flammable: 0,
    explosive: 0,
    meltable: 0,
    hardness: 30,
    photon_reflect: 0x3FFF_FFFF,
    weight: 50,
    heat_conduct: 128,
    heat_capacity: 1.0,
    properties: 0,
    carries_type_in: 0,
    low_pressure: -257.0,
    low_pressure_transition: -1,
    high_pressure: 257.0,
    high_pressure_transition: -1,
    low_temperature: -1.0,
    low_temperature_transition: -1,
    high_temperature: 10000.0,
    high_temperature_transition: -1,
    default_temp: 293.15,
    has_update: false,
    has_graphics: false,
};

/// Look up an element definition by numeric id. Unknown ids return the shared
/// empty definition (never panics, matching the reference's "no element" use).
#[inline]
pub fn element(t: u16) -> &'static ElementDef {
    match ELEMENTS.get(t as usize) {
        Some(def) => def,
        None => &EMPTY_ELEMENT,
    }
}

/// Look up an element id by its reference abbreviation (e.g. `"SAND"`).
///
/// Used by the legacy v1 save importer to map old registry ids through their
/// canonical TPT abbreviation instead of reusing numeric ids (which differ
/// between the two id spaces). Only enabled elements are considered.
pub fn pt_id_by_abbr(abbr: &str) -> Option<u16> {
    ELEMENTS
        .iter()
        .find(|e| e.enabled && e.abbr.eq_ignore_ascii_case(abbr))
        .map(|e| e.id)
}

/// True when `t` is a real element (inside the reference ID space and enabled).
#[inline]
pub fn is_element(t: i32) -> bool {
    t > 0 && (t as usize) < PT_NUM && ELEMENTS[t as usize].enabled
}

/// Number of element slots in the reference ID space (512).
pub const fn pt_num() -> usize {
    PT_NUM
}

#[inline]
pub fn is_type(t: u16, flags: u32) -> bool {
    element(t).properties & flags != 0
}

#[inline]
pub fn is_conductive(t: u16) -> bool {
    is_type(t, crate::elements_generated::props::PROP_CONDUCTS)
}

#[inline]
pub fn is_liquid(t: u16) -> bool {
    is_type(t, crate::elements_generated::props::TYPE_LIQUID)
}

#[inline]
pub fn is_gas(t: u16) -> bool {
    is_type(t, crate::elements_generated::props::TYPE_GAS)
}

#[inline]
pub fn is_powder(t: u16) -> bool {
    is_type(t, crate::elements_generated::props::TYPE_PART)
}

#[inline]
pub fn is_energy(t: u16) -> bool {
    is_type(t, crate::elements_generated::props::TYPE_ENERGY)
}

#[inline]
pub fn is_solid(t: u16) -> bool {
    is_type(t, crate::elements_generated::props::TYPE_SOLID)
}
