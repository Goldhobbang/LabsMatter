//! Cell (particle-in-grid) representation.

use serde::{Deserialize, Serialize};

use crate::material::MaterialId;
use crate::units::Temperature;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[repr(C)]
pub struct Cell {
    pub material: MaterialId,
    pub temperature: Temperature,
    pub vx: f32,
    pub vy: f32,
    pub flags: u8,
    pub lifetime: f32,
    pub pressure: f32,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            material: 0,
            temperature: 293.15,
            vx: 0.0,
            vy: 0.0,
            flags: 0,
            lifetime: 0.0,
            pressure: 0.0,
        }
    }
}

impl Cell {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.material == 0
    }

    pub fn has_flag(&self, flag: u8) -> bool {
        self.flags & flag != 0
    }

    pub fn set_flag(&mut self, flag: u8, value: bool) {
        if value {
            self.flags |= flag;
        } else {
            self.flags &= !flag;
        }
    }
}

pub mod cell_flags {
    pub const BURNING: u8 = 1 << 0;
    pub const WET: u8 = 1 << 1;
    pub const FIXED: u8 = 1 << 2;
    pub const UPDATED: u8 = 1 << 3;
    pub const CHARGED: u8 = 1 << 4;
    pub const DISSOLVED: u8 = 1 << 5;
}