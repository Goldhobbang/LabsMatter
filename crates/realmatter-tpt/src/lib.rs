//! LabsMatter parity engine ??a behavioural port of The Powder Toy's
//! simulation model (pinned reference `2b7221a46d378589b4de7683f16d3ebc92be6bd9`).
//!
//! The crate is deliberately independent of the older `realmatter-*` crates: it
//! owns the reference element ID space (0..=195 plus empty slots up to 512), the
//! `pmap` layout, the air/pressure/velocity fields and the tick pipeline.

pub mod air;
pub mod element;
pub mod elements_generated;
pub mod entity;
pub mod gol;
pub mod movement;
pub mod particle;
pub mod portal;
pub mod rng;
pub mod sim;
pub mod tick;
pub mod transport;
pub mod transition;
pub mod updates;
pub mod wireless;

pub use air::{AirMode, ConvectionMode};
pub use element::{element, pt_id_by_abbr, ElementDef};
pub use particle::{flags, Particle};
pub use rng::Rng;
pub use sim::{
    pmap_id, pmap_pack, pmap_type, EdgeMode, GravityMode, Settings, Sim, CELL, MAX_TEMP,
    MIN_TEMP, PMAPBITS, PMAPMASK,
};
pub use elements_generated::PT_NUM;
pub use tick::Neighbourhood;

/// Crate version, reported by the WASM boundary and the About dialog.
pub const ENGINE_NAME: &str = "LabsMatter TPT-parity engine";

/// Summary used by the UI/status tooling.
pub fn coverage() -> Coverage {
    let mut with_update = 0usize;
    let mut total = 0usize;
    let mut menu_visible = 0usize;
    for t in 1..elements_generated::PT_NUM as u16 {
        let def = element(t);
        if !def.enabled {
            continue;
        }
        total += 1;
        if def.has_update {
            with_update += 1;
        }
        if def.menu_visible {
            menu_visible += 1;
        }
    }
    Coverage {
        elements: total,
        menu_visible,
        reference_updates: with_update,
        implemented_updates: with_update.saturating_sub(updates::unimplemented_count()),
        walls: elements_generated::WALLS.len(),
        gol_rules: elements_generated::GOL_RULES.len(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Coverage {
    pub elements: usize,
    pub menu_visible: usize,
    pub reference_updates: usize,
    pub implemented_updates: usize,
    pub walls: usize,
    pub gol_rules: usize,
}
