//! Physical units and constants.

pub type Length = f32;
pub type Mass = f32;
pub type Time = f64;
pub type Temperature = f32;

pub mod constants {
    pub const GRAVITY: f32 = 9.80665;
    pub const GAS_CONSTANT: f32 = 8.314462618;
    pub const SPEED_OF_LIGHT: f32 = 299_792_458.0;
    pub const ABSOLUTE_ZERO: f32 = 0.0;
    pub const ATM_PRESSURE: f32 = 101_325.0;
    pub const FIXED_DT: f64 = 1.0 / 60.0;
    pub const MAX_SUBSTEPS: u32 = 4;
    pub const CELL_SIZE: f32 = 0.01;
}
