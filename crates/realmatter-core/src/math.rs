//! Math primitives used across the simulation.
//!
//! We use `f32` for spatial state (positions, velocities) for performance and
//! WebGPU compatibility, and `f64` for time accumulation. See ADR-0002.

use glam::Vec2 as GlamVec2;

/// 2D vector (f32) used for positions, velocities, forces.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
#[repr(C)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    #[inline]
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    #[inline]
    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    #[inline]
    pub fn normalized(self) -> Self {
        let l = self.length();
        if l > 0.0 {
            Self {
                x: self.x / l,
                y: self.y / l,
            }
        } else {
            Self::ZERO
        }
    }

    #[inline]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }
}

impl std::ops::Add for Vec2 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Self;
    #[inline]
    fn mul(self, s: f32) -> Self {
        Self {
            x: self.x * s,
            y: self.y * s,
        }
    }
}

impl From<GlamVec2> for Vec2 {
    fn from(v: GlamVec2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

impl From<Vec2> for GlamVec2 {
    fn from(v: Vec2) -> Self {
        GlamVec2::new(v.x, v.y)
    }
}