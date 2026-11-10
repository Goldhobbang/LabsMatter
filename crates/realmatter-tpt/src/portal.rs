//! Persistent portal payload storage, mirroring `Simulation::portalp` in the
//! pinned reference (`src/simulation/Simulation.h`).
//!
//! Reference layout: `Particle portalp[CHANNELS][8][80]`, where `CHANNELS` is
//! derived from the temperature range (`int(MAX_TEMP - 73.15) / 100 + 2`, as
//! used by PRTI/PRTO/WIFI to compute `parts[i].tmp`). Slot 0 of each direction
//! ring is the "first stored" particle; PRTO reads with an offset of four
//! positions plus a random `-1..=1`, which is why the buffer is an ordinary
//! array and not a queue.
//!
//! This module is deliberately owned by the world layer (Agent 1). Element
//! handlers (Agent 3) only read/write through the [`PortalStore`] API so the
//! payloads are part of `Sim` and therefore serialisable.

use crate::particle::Particle;

/// Reference `CHANNELS = int(MAX_TEMP - 73.15) / 100 + 2` with `MAX_TEMP = 9999`
/// (`.reference/tpt-src/src/simulation/Simulation.h:139`, used by
/// `.reference/tpt-src/src/simulation/elements/PRTI.cpp:68-72`, `PRTO.cpp:63-65`).
/// `int((9999.0 - 73.15) / 100.0 + 2.0) == 101`.
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
    ///
    /// Contract (ref `.reference/tpt-src/src/simulation/elements/PRTO.cpp:103-136`):
    /// the caller copies the stored payload wholesale onto the freshly created
    /// particle (`parts[np] = sim->portalp[...]`), then overrides `x`/`y` with
    /// the emission cell (`x+rx`, `y+ry`). If the stored `vx == 0.0f && vy == 0.0f`,
    /// the caller must keep the freshly created particle's velocity (PRTO samples
    /// `tmp_vx/tmp_vy` from the new particle before the copy and restores them);
    /// otherwise the stored velocity is used. This helper implements that
    /// field-wise restore but does not set `x`/`y` (the caller must). STKM/FIGH
    /// bookkeeping is not performed here and belongs to the caller.
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

    /// First empty slot index for `(channel, direction)`, scanning `nnx` in
    /// `0..80` for a slot whose payload is empty (ref PRTI.cpp:96-97:
    /// `if (!sim->portalp[parts[i].tmp][count][nnx].type)`).
    pub fn first_empty_slot(&self, channel: usize, direction: usize) -> Option<usize> {
        if channel >= PORTAL_CHANNELS || direction >= PORTAL_DIRECTIONS {
            return None;
        }
        let base = Self::index(channel, direction, 0);
        if base >= self.slots.len() {
            return Some(0);
        }
        for slot in 0..PORTAL_SLOTS {
            if self.slots[base + slot].is_empty() {
                return Some(slot);
            }
        }
        None
    }

    /// Payload at a specific slot, or `None` for empty/out-of-range
    /// `(channel, direction, slot)`.
    pub fn payload_at(
        &self,
        channel: usize,
        direction: usize,
        slot: usize,
    ) -> Option<PortalPayload> {
        if channel >= PORTAL_CHANNELS || direction >= PORTAL_DIRECTIONS || slot >= PORTAL_SLOTS {
            return None;
        }
        let idx = Self::index(channel, direction, slot);
        let p = *self.slots.get(idx)?;
        if p.is_empty() {
            None
        } else {
            Some(p)
        }
    }

    /// Set a specific slot (overwriting any existing payload). Out-of-range
    /// coordinates are ignored.
    pub fn set_slot(
        &mut self,
        channel: usize,
        direction: usize,
        slot: usize,
        payload: PortalPayload,
    ) {
        if channel >= PORTAL_CHANNELS || direction >= PORTAL_DIRECTIONS || slot >= PORTAL_SLOTS {
            return;
        }
        self.ensure(channel);
        self.slots[Self::index(channel, direction, slot)] = payload;
    }

    /// Clear a specific slot to empty (ref PRTO.cpp:89,136:
    /// `memset(&portalp[ch][dir][nnx], 0, sizeof(Particle))`).
    pub fn clear_slot(&mut self, channel: usize, direction: usize, slot: usize) {
        self.set_slot(channel, direction, slot, PortalPayload::default());
    }

    /// Insert into the first free slot for a direction. Returns the slot index
    /// written, or `None` when the direction buffer is full (80 particles),
    /// matching the reference (PRTI.cpp:96-118).
    pub fn store_index(
        &mut self,
        channel: usize,
        direction: usize,
        payload: PortalPayload,
    ) -> Option<usize> {
        let slot = self.first_empty_slot(channel, direction)?;
        self.set_slot(channel, direction, slot, payload);
        Some(slot)
    }

    /// Insert into the first free slot for a direction. Returns `false` when the
    /// direction buffer is full (80 particles), matching the reference.
    pub fn store(&mut self, channel: usize, direction: usize, payload: PortalPayload) -> bool {
        self.store_index(channel, direction, payload).is_some()
    }

    /// PRTO emission direction for a given neighbour `count` and per-iteration
    /// jitter (ref PRTO.cpp:78: `(count + sim->rng.between(-1, 1) + 4) % 8`).
    /// Uses `rem_euclid(8)` for a non-negative result.
    ///
    /// The caller must draw `rng.between(-1, 1)` **inside** the `nnx` slot-scan
    /// loop, once per iteration (PRTO.cpp:76-80), and PRTO reads
    /// `portalp[parts[i].tmp][randomness][nnx]` for that iteration.
    pub fn reverse_direction(count: usize, jitter: i32) -> usize {
        ((count as i32) + jitter + 4).rem_euclid(8) as usize
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

/// Port of `orbitalparts_get` from
/// `.reference/tpt-src/src/simulation/orbitalparts.h:3-15`.
///
/// Splits two 32-bit blocks (`block1` = `parts[i].life`, `block2` =
/// `parts[i].ctype`) into four per-orbital values each: low byte in `[0]` up to
/// high byte in `[3]`. Slots `[0]..[2]` are unsigned byte extracts
/// (`block & 0xFF`, `(block & 0xFF00) >> 8`, `(block & 0xFF0000) >> 16`),
/// giving ranges `[0]..[2] = 0..=255`. Slot `[3]` is sign-extended because the
/// reference uses `int >> 24` (C arithmetic right shift on signed `int`), so we
/// compute `(block >> 24)` on `i32`, giving `[3] = -128..=127`
/// (`resN[3] = (blockN & 0xFF000000) >> 24` in the reference).
/// Returns `(resblock1, resblock2)`.
pub fn orbital_get(life: i32, ctype: i32) -> ([i32; 4], [i32; 4]) {
    let b1 = life;
    let b2 = ctype;
    let res1 = [
        b1 & 0x0000_00FF,
        (b1 & 0x0000_FF00) >> 8,
        (b1 & 0x00FF_0000) >> 16,
        b1 >> 24,
    ];
    let res2 = [
        b2 & 0x0000_00FF,
        (b2 & 0x0000_FF00) >> 8,
        (b2 & 0x00FF_0000) >> 16,
        b2 >> 24,
    ];
    (res1, res2)
}

/// Port of `orbitalparts_set` from
/// `.reference/tpt-src/src/simulation/orbitalparts.h:17-33`.
///
/// Packs four per-orbital values into two 32-bit blocks written back to
/// `parts[i].life` and `parts[i].ctype`. Bit packing (same masks/shifts as
/// reference): for each block, `(res[0] & 0xFF) | ((res[1] & 0xFF) << 8) |
/// ((res[2] & 0xFF) << 16) | ((res[3] & 0xFF) << 24)`. Returns `(life, ctype)`.
pub fn orbital_set(orbd: &[i32; 4], orbl: &[i32; 4]) -> (i32, i32) {
    // `& 0xFF` on each (possibly negative, e.g. slot [3] = -128..=127 from
    // orbital_get) reproduces C's byte extraction/truncation before packing.
    let b1: u32 = ((orbd[0] as u32) & 0xFF)
        | (((orbd[1] as u32) & 0xFF) << 8)
        | (((orbd[2] as u32) & 0xFF) << 16)
        | (((orbd[3] as u32) & 0xFF) << 24);
    let b2: u32 = ((orbl[0] as u32) & 0xFF)
        | (((orbl[1] as u32) & 0xFF) << 8)
        | (((orbl[2] as u32) & 0xFF) << 16)
        | (((orbl[3] as u32) & 0xFF) << 24);
    (b1 as i32, b2 as i32)
}
