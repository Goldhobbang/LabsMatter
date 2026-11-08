//! Persistent portal payload storage, mirroring `Simulation::portalp` in the
//! pinned reference (`src/simulation/Simulation.h`).
//!
//! Reference layout: `Particle portalp[CHANNELS][8][80]`, where `CHANNELS` is
//! derived from the temperature range (`int(MAX_TEMP - 73) / 100 + 2`). Slot 0
//! of each direction ring is the "first stored" particle; PRTO reads with an
//! offset of four positions plus a random `-1..=1`, which is why the buffer is
//! an ordinary array and not a queue.
//!
//! This module is deliberately owned by the world layer (Agent 1). Element
//! handlers (Agent 3) only read/write through the [`PortalStore`] API so the
//! payloads are part of `Sim` and therefore serialisable.

use crate::particle::Particle;

/// Reference `CHANNELS = int(MAX_TEMP - 73) / 100 + 2` with `MAX_TEMP = 9999`.
pub const PORTAL_CHANNELS: usize = 101;
/// Eight directions around the portal (`portal_rx` / `portal_ry`).
pub const PORTAL_DIRECTIONS: usize = 8;
/// Reference stores up to 80 particles per direction.
pub const PORTAL_SLOTS: usize = 80;

/// A stored particle. Field-for-field the subset of `Particle` that affects the
/// re-emitted particle's behaviour.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PortalPayload {
    pub t: u16,
    pub life: i32,
    pub ctype: i32,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub temp: f32,
    pub tmp: i32,
    pub tmp2: i32,
    pub tmp3: i32,
    pub tmp4: i32,
    pub flags: u32,
    pub dcolour: u32,
}

impl PortalPayload {
    /// An empty slot is identified by `t == 0`, matching the reference's
    /// `memset(..., 0, sizeof(Particle))` clear.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.t == 0
    }

    /// Capture a live particle for transport.
    pub fn from_particle(p: &Particle) -> Self {
        Self {
            t: p.t,
            life: p.life,
            ctype: p.ctype,
            x: p.x,
            y: p.y,
            vx: p.vx,
            vy: p.vy,
            temp: p.temp,
            tmp: p.tmp,
            tmp2: p.tmp2,
            tmp3: p.tmp3,
            tmp4: p.tmp4,
            flags: p.flags,
            dcolour: p.dcolour,
        }
    }

    /// Write this payload back onto a freshly created particle, preserving the
    /// destination position and velocity when the stored payload has none
    /// (reference PRTO does exactly this for PIPE-sourced particles).
    pub fn apply_to(&self, p: &mut Particle, keep_velocity_if_zero: bool) {
        let (vx, vy) = if keep_velocity_if_zero && self.vx == 0.0 && self.vy == 0.0 {
            (p.vx, p.vy)
        } else {
            (self.vx, self.vy)
        };
        p.t = self.t;
        p.life = self.life;
        p.ctype = self.ctype;
        p.vx = vx;
        p.vy = vy;
        p.temp = self.temp;
        p.tmp = self.tmp;
        p.tmp2 = self.tmp2;
        p.tmp3 = self.tmp3;
        p.tmp4 = self.tmp4;
        p.flags = self.flags;
        p.dcolour = self.dcolour;
    }
}

/// Dense-but-lazy portal buffer. Storage grows only to the highest channel that
/// is actually used, so a typical world does not pay for all 101 channels.
#[derive(Clone, Debug, Default)]
pub struct PortalStore {
    slots: Vec<PortalPayload>,
}

impl PortalStore {
    pub fn new() -> Self {
        Self { slots: Vec::new() }
    }

    /// Slot index for `(channel, direction, slot)`.
    #[inline]
    fn index(channel: usize, direction: usize, slot: usize) -> usize {
        (channel * PORTAL_DIRECTIONS + direction) * PORTAL_SLOTS + slot
    }

    fn ensure(&mut self, channel: usize) {
        let need = (channel + 1) * PORTAL_DIRECTIONS * PORTAL_SLOTS;
        if self.slots.len() < need {
            self.slots.resize(need, PortalPayload::default());
        }
    }

    /// Insert into the first free slot for a direction. Returns `false` when the
    /// direction buffer is full (80 particles), matching the reference.
    pub fn store(&mut self, channel: usize, direction: usize, payload: PortalPayload) -> bool {
        if channel >= PORTAL_CHANNELS || direction >= PORTAL_DIRECTIONS {
            return false;
        }
        self.ensure(channel);
        for slot in 0..PORTAL_SLOTS {
            let idx = Self::index(channel, direction, slot);
            if self.slots[idx].is_empty() {
                self.slots[idx] = payload;
                return true;
            }
        }
        false
    }

    /// Read the first stored payload in a direction, if any.
    pub fn peek(&self, channel: usize, direction: usize) -> Option<PortalPayload> {
        if channel >= PORTAL_CHANNELS || direction >= PORTAL_DIRECTIONS {
            return None;
        }
        let base = Self::index(channel, direction, 0);
        if base >= self.slots.len() {
            return None;
        }
        self.slots[base..base + PORTAL_SLOTS]
            .iter()
            .copied()
            .find(|p| !p.is_empty())
    }

    /// Remove and return the first stored payload in a direction.
    pub fn take(&mut self, channel: usize, direction: usize) -> Option<PortalPayload> {
        if channel >= PORTAL_CHANNELS || direction >= PORTAL_DIRECTIONS {
            return None;
        }
        let base = Self::index(channel, direction, 0);
        if base >= self.slots.len() {
            return None;
        }
        for slot in 0..PORTAL_SLOTS {
            let idx = base + slot;
            if !self.slots[idx].is_empty() {
                let out = self.slots[idx];
                self.slots[idx] = PortalPayload::default();
                return Some(out);
            }
        }
        None
    }

    /// True when any direction of a channel holds a payload.
    pub fn channel_occupied(&self, channel: usize) -> bool {
        for direction in 0..PORTAL_DIRECTIONS {
            if self.peek(channel, direction).is_some() {
                return true;
            }
        }
        false
    }

    pub fn clear(&mut self) {
        self.slots.clear();
    }

    /// Iterate non-empty payloads with their coordinates, for persistence.
    pub fn iter_nonempty(&self) -> impl Iterator<Item = (usize, usize, usize, PortalPayload)> + '_ {
        self.slots
            .iter()
            .enumerate()
            .filter(|(_, p)| !p.is_empty())
            .map(|(idx, p)| {
                let slot = idx % PORTAL_SLOTS;
                let rest = idx / PORTAL_SLOTS;
                let direction = rest % PORTAL_DIRECTIONS;
                let channel = rest / PORTAL_DIRECTIONS;
                (channel, direction, slot, *p)
            })
    }

    /// Restore one payload at a known coordinate (used by the loader).
    pub fn restore(&mut self, channel: usize, direction: usize, slot: usize, payload: PortalPayload) {
        if channel >= PORTAL_CHANNELS || direction >= PORTAL_DIRECTIONS || slot >= PORTAL_SLOTS {
            return;
        }
        self.ensure(channel);
        self.slots[Self::index(channel, direction, slot)] = payload;
    }

    pub fn len(&self) -> usize {
        self.slots.iter().filter(|p| !p.is_empty()).count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
