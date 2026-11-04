//! Chemical bonds (placeholder for future extension).
//! Bonds are out of scope for v0.1 core; this module provides types only.

#[derive(Clone, Debug)]
pub struct Bond {
    pub a: (i32, i32),
    pub b: (i32, i32),
    pub energy: f32,
}

#[derive(Clone, Debug, Default)]
pub struct BondGraph {
    pub bonds: Vec<Bond>,
}
