//! Deterministic RNG matching the reference's `tpt-rand` (xoroshiro128+).
//!
//! Call order matters for behavioural parity, so this module exposes exactly
//! the same helpers with the same semantics as the reference:
//! `next`, `gen`, `between`, `chance`, `uniform01`.

#[derive(Clone, Debug)]
pub struct Rng {
    s0: u64,
    s1: u64,
}

#[inline]
fn rotl(x: u64, k: u32) -> u64 {
    x.rotate_left(k)
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self { s0: seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1, s1: 614 }
    }

    pub fn seed(&mut self, sd: u64) {
        self.s0 = sd;
        self.s1 = sd;
    }

    /// Raw 64-bit step (reference `next()`).
    #[inline]
    pub fn next(&mut self) -> u64 {
        let s0 = self.s0;
        let mut s1 = self.s1;
        let result = s0.wrapping_add(s1);
        s1 ^= s0;
        self.s0 = rotl(s0, 55) ^ s1 ^ (s1 << 14);
        self.s1 = rotl(s1, 36);
        result
    }

    /// `next() & 0x7FFFFFFF`
    #[inline]
    pub fn gen(&mut self) -> u32 {
        (self.next() & 0x7FFF_FFFF) as u32
    }

    /// `next() & 0xFFFFFFFF`
    #[inline]
    pub fn gen_u32(&mut self) -> u32 {
        self.next() as u32
    }

    /// Reference `between(l, u)`: inclusive range.
    #[inline]
    pub fn between(&mut self, l: i32, u: i32) -> i32 {
        let span = (u as u32).wrapping_sub(l as u32).wrapping_add(1);
        ((self.next() % span as u64) as u32 as i32).wrapping_add(l)
    }

    /// Reference `chance(n, d)`: true with probability `n / d`.
    #[inline]
    pub fn chance(&mut self, n: i32, d: u32) -> bool {
        if n < 0 {
            return false;
        }
        (self.next() % d as u64) < n as u64
    }

    /// Reference `uniform01()`: `[0, 1]` inclusive of 1.0.
    #[inline]
    pub fn uniform01(&mut self) -> f32 {
        (self.next() as u32) as f32 / (u32::MAX as f32)
    }
}

impl Default for Rng {
    fn default() -> Self {
        Self::new(0x1234_5678_9ABC_DEF0)
    }
}
