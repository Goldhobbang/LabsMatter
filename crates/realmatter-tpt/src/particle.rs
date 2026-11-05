//! Particle representation, mirroring `Particle` in the reference.
//!
//! Field order and widths follow `Simulation.h` / `Particle.h` closely enough
//! that behaviour and save mapping stay straightforward.

/// Simulation flags stored in `Particle::flags`.
pub mod flags {
    /// Set for one tick when a particle failed to move; cleared at the start of
    /// the next `MovementPhase`.
    pub const STAGNANT: u32 = 0x0000_0001;
    /// Photons: skip movement for one frame.
    pub const SKIPMOVE: u32 = 0x0000_0002;
    /// Compatibility flag for old saves (moving SPNG / decorated PHOT).
    pub const MOVABLE: u32 = 0x0000_0008;
}

#[derive(Clone, Copy, Debug)]
pub struct Particle {
    /// Element id (`PT_*`). 0 means a dead slot.
    pub t: u16,
    pub life: i32,
    pub ctype: i32,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub temp: f32,
    pub tmp3: i32,
    pub tmp4: i32,
    pub flags: u32,
    pub tmp: i32,
    pub tmp2: i32,
    /// Decoration colour (ARGB). 0 means none.
    pub dcolour: u32,
    /// Cached display colour written by the renderer.
    pub pavg: [u8; 4],
}

impl Default for Particle {
    fn default() -> Self {
        Self::new(0, 0.0, 0.0)
    }
}

impl Particle {
    #[inline]
    pub fn new(t: u16, x: f32, y: f32) -> Self {
        Self {
            t,
            life: 0,
            ctype: 0,
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            temp: 293.15,
            tmp3: 0,
            tmp4: 0,
            flags: 0,
            tmp: 0,
            tmp2: 0,
            dcolour: 0,
            pavg: [0, 0, 0, 0],
        }
    }

    #[inline]
    pub fn alive(&self) -> bool {
        self.t != 0
    }
}
