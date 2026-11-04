//! Particle representation (legacy 3D scaffold — NOT part of the 2D contract).
//!
//! DEPRECATED: the approved 2D model stores one [`Cell`](crate::cell::Cell)
//! per grid position (see `docs/contracts/particle-representation.md`); there
//! is no separate particle list. This module is kept only so historical
//! references resolve while downstream crates migrate; it will be removed in a
//! future version. New code must use `crate::cell::Cell`.

use serde::{Deserialize, Serialize};

/// Material id. 0 = void/empty.
pub type MaterialId = u16;

/// Particle flags bitfield (raw u16).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ParticleFlags(pub u16);

impl ParticleFlags {
    pub const BURNING: u16 = 1 << 0;
    pub const CONDUCTING: u16 = 1 << 1;
    pub const FROZEN: u16 = 1 << 2;
    pub const FIXED: u16 = 1 << 3;
    pub const DIRTY: u16 = 1 << 4;

    pub fn contains(&self, bit: u16) -> bool {
        self.0 & bit != 0
    }
    pub fn insert(&mut self, bit: u16) {
        self.0 |= bit;
    }
    pub fn remove(&mut self, bit: u16) {
        self.0 &= !bit;
    }
}

/// Legacy 3D particle struct (deprecated; see module docs).
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[repr(C)]
pub struct Particle {
    pub material: MaterialId,
    pub temperature: f32,
    pub velocity: [f32; 3],
    pub flags: ParticleFlags,
    pub lifetime: f32,
    pub custom: u32,
}

impl Particle {
    pub const VOID: Particle = Particle {
        material: 0,
        temperature: 293.15,
        velocity: [0.0; 3],
        flags: ParticleFlags(0),
        lifetime: 0.0,
        custom: 0,
    };

    pub fn is_void(&self) -> bool {
        self.material == 0
    }
}
