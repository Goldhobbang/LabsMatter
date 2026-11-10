//! Persistent wireless-channel state, mirroring `Simulation::wireless` in the
//! pinned reference (`src/simulation/Simulation.h`).
//!
//! The reference stores `int wireless[CHANNELS][2]`:
//! - `[ch][0]` — channel is active on the current frame,
//! - `[ch][1]` — channel should be active on the next frame.
//!
//! `WIFI` particles raise `[ch][1]` when they see a rising spark and read
//! `[ch][0]` to fire neighbouring conductors. `Simulation::AfterSim` promotes
//! column 1 to column 0 once per tick. This module owns that state so it is
//! part of `Sim`, serialisable, and advanced at a deterministic point in the
//! tick regardless of particle iteration order.

use crate::portal::PORTAL_CHANNELS;

/// Wireless channel state. Storage is lazily grown like [`crate::portal::PortalStore`].
#[derive(Clone, Debug, Default)]
pub struct WirelessStore {
    /// `[ch][0]` current, `[ch][1]` next.
    channels: Vec<[bool; 2]>,
    /// Reference `Simulation::ISWIRE` countdown. Set when a spark is observed
    /// (WIFI sets `ISWIRE = 2` in the reference) and decremented in `BeforeSim`.
    /// While > 0, pending channel state is promoted each `BeforeSim` pass.
    pub iswire: u32,
}

impl WirelessStore {
    pub fn new() -> Self {
        Self {
            channels: Vec::new(),
            iswire: 0,
        }
    }

    fn ensure(&mut self, channel: usize) {
        if self.channels.len() <= channel {
            self.channels.resize(channel + 1, [false, false]);
        }
    }

    /// Reference `wireless[ch][0]`: whether the channel is live this frame.
    #[inline]
    pub fn is_active(&self, channel: usize) -> bool {
        self.channels.get(channel).map(|c| c[0]).unwrap_or(false)
    }

    /// Reference `wireless[ch][1] = 1`: mark the channel active next frame.
    /// Out-of-range channels are rejected (no-op) because `PORTAL_CHANNELS` is
    /// the shared channel count (`int(MAX_TEMP - 73.15) / 100 + 2`, ref
    /// Simulation.h:139) and element updates clamp `tmp` into `0..CHANNELS-1`
    /// before indexing (e.g. WIFI.cpp:52-54); the reference would never index
    /// outside that range with a correctly clamped `tmp`.
    pub fn activate_next(&mut self, channel: usize) {
        if channel >= PORTAL_CHANNELS {
            return;
        }
        self.ensure(channel);
        self.channels[channel][1] = true;
    }

    /// Whether a neighbouring spark should be observed by WIFI (ref
    /// `.reference/tpt-src/src/simulation/elements/WIFI.cpp:75-78`): the
    /// neighbour is `PT_SPRK && parts[ID(r)].ctype != PT_NSCN &&
    /// parts[ID(r)].life >= 3`.
    pub fn spark_observes_channel(
        neighbour_type: u16,
        neighbour_ctype: i32,
        neighbour_life: i32,
    ) -> bool {
        use crate::elements_generated::pt;
        neighbour_type == pt::SPRK
            && neighbour_ctype != pt::NSCN as i32
            && neighbour_life >= 3
    }

    /// Reference `ISWIRE` bookkeeping: a spark was seen on a channel this frame.
    /// Returns true when this is a *new* wireless event (used by the frontend to
    /// draw the powerline overlay).
    ///
    /// Semantics match the reference (WIFI.cpp:75-78): on observation the code
    /// sets `wireless[tmp][1] = 1` and `ISWIRE = 2`. Promotion of `[1] -> [0]`
    /// happens only in `advance()` (BeforeSim, Simulation.cpp:3900-3908) while
    /// `iswire > 0`, so this does not itself promote or alter the current `[0]`.
    pub fn note_spark(&mut self, channel: usize) -> bool {
        if channel >= PORTAL_CHANNELS {
            return false;
        }
        let was = self.is_active(channel);
        self.activate_next(channel);
        // Reference WIFI.cpp sets `sim->ISWIRE = 2` on spark observation.
        self.iswire = 2;
        !was
    }

    /// Reference `Simulation::BeforeSim` wireless block (Simulation.cpp:3900-3908):
    /// when `ISWIRE > 0`, promote `wireless[q][1] -> [0]`, clear `[1]`, then
    /// decrement `ISWIRE`. Called once per tick before particle updates.
    ///
    /// Channels that were never allocated remain implicitly `[false,false]`, so
    /// the reference loop over all `CHANNELS` promotes them to false/stays false
    /// as well; promotion happens only when `iswire > 0`.
    pub fn advance(&mut self) {
        if self.iswire == 0 {
            return;
        }
        for c in self.channels.iter_mut() {
            c[0] = c[1];
            c[1] = false;
        }
        self.iswire = self.iswire.saturating_sub(1);
    }

    /// Number of channels that are active this frame (diagnostics / tests).
    pub fn active_count(&self) -> usize {
        self.channels.iter().filter(|c| c[0]).count()
    }

    pub fn iter_active(&self) -> impl Iterator<Item = usize> + '_ {
        self.channels
            .iter()
            .enumerate()
            .filter(|(_, c)| c[0])
            .map(|(i, _)| i)
    }

    pub fn clear(&mut self) {
        self.channels.clear();
    }
}
