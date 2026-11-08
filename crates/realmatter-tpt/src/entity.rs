//! Player / stickman / fighter entity state and shared movement helpers.
//!
//! Reference: `src/simulation/Stickman.h` (`playerst`), `Simulation::player`,
//! `Simulation::player2`, `Simulation::fighters[]`, and the movement portion of
//! `src/simulation/elements/STKM.cpp`.
//!
//! Agent 1 owns this shared state and the generic locomotion / collision /
//! health helpers. Agent 3 owns the element-specific `STKM` / `STKM2` / `FIGH`
//! update callbacks that call into these helpers.
//!
//! The reference keeps *legs* as loose particle positions and moves the head
//! particle directly. We keep the same model: the entity is anchored to a head
//! particle of type `STKM`/`STKM2`/`FIGH`; legs are lightweight offsets.

use crate::element::element;
use crate::elements_generated::{props, pt};
use crate::sim::{pmap_id, restrict_flt, Sim, CELL, MAX_TEMP, MIN_TEMP};

/// Reference `MAX_FIGHTERS`.
pub const MAX_FIGHTERS: usize = 100;

/// Input command bits (reference `comm` / `pcomm`).
pub mod input {
    pub const LEFT: u8 = 0x01;
    pub const RIGHT: u8 = 0x02;
    pub const JUMP: u8 = 0x04;
    pub const SPAWN: u8 = 0x08;
}

/// Reference `playerst`.
#[derive(Clone, Copy, Debug)]
pub struct PlayerState {
    /// Current command bits.
    pub comm: u8,
    /// Previous-frame command bits (used for element spawning).
    pub pcomm: u8,
    /// Element power (e.g. `PT_LIGH`).
    pub elem: i32,
    /// Leg offsets relative to the head (x, y pairs).
    pub legs: [f32; 16],
    /// Leg accelerations.
    pub accs: [f32; 8],
    /// 1 when a stickman is spawned.
    pub spwn: u8,
    /// Frames since the last particle spawn.
    pub frames: u32,
    pub rocket_boots: bool,
    pub fan: bool,
    /// Particle id of the `SPWN` element that created it.
    pub spawn_id: i32,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            comm: 0,
            pcomm: 0,
            elem: 0,
            legs: [0.0; 16],
            accs: [0.0; 8],
            spwn: 0,
            frames: 0,
            rocket_boots: false,
            fan: false,
            spawn_id: -1,
        }
    }
}

/// Which entity a helper refers to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entity {
    Player1,
    Player2,
    Fighter(usize),
}

/// Result of an entity locomotion step.
#[derive(Clone, Copy, Debug, Default)]
pub struct StepReport {
    pub moved: bool,
    pub grounded: bool,
    pub died: bool,
}

impl Sim {
    /// The particle type used for a player slot.
    fn player_type(which: Entity) -> u16 {
        match which {
            Entity::Player1 => pt::STKM,
            Entity::Player2 => pt::STKM2,
            Entity::Fighter(_) => pt::FIGH,
        }
    }

    /// Borrow the player state for an entity.
    pub fn player_state(&self, which: Entity) -> Option<&PlayerState> {
        match which {
            Entity::Player1 => Some(&self.player),
            Entity::Player2 => Some(&self.player2),
            Entity::Fighter(i) => self.fighters.get(i),
        }
    }

    /// Mutably borrow the player state for an entity.
    pub fn player_state_mut(&mut self, which: Entity) -> Option<&mut PlayerState> {
        match which {
            Entity::Player1 => Some(&mut self.player),
            Entity::Player2 => Some(&mut self.player2),
            Entity::Fighter(i) => self.fighters.get_mut(i),
        }
    }

    /// Head particle id for a player slot. Unlike the reference we do not keep a
    /// pointer, so the head is located by scanning for the unique live particle
    /// of that type. The `create_allowed` callback guarantees uniqueness.
    pub fn entity_head(&self, which: Entity) -> Option<usize> {
        let t = Self::player_type(which);
        self.parts
            .iter()
            .position(|p| p.t == t && p.life > -1)
            .filter(|&i| self.parts[i].t == t)
    }

    /// Spawn a player/fighter head at `(x, y)`. Returns the particle index.
    ///
    /// Mirrors `Simulation::create_part(-1, x, y, PT_STKM/PT_STKM2/PT_FIGH)`
    /// plus the `player.spwn` bookkeeping.
    pub fn spawn_entity(&mut self, which: Entity, x: i32, y: i32) -> Option<usize> {
        // Only one STKM / STKM2 may exist at a time (reference create_allowed).
        if matches!(which, Entity::Player1 | Entity::Player2) && self.entity_head(which).is_some() {
            return None;
        }
        let t = Self::player_type(which);
        let index = match which {
            Entity::Fighter(i) => i,
            _ => 0,
        };
        let head = self.create_part(x, y, t)?;
        self.parts[head].life = 100;
        if let Some(st) = self.player_state_mut(which) {
            st.spwn = 1;
            st.comm = 0;
            st.pcomm = 0;
            st.frames = 0;
            st.spawn_id = -1;
            st.legs = [0.0; 16];
            st.accs = [0.0; 8];
            st.elem = if index < 0 { 0 } else { st.elem };
        }
        Some(head)
    }

    /// Remove a spawned entity and clear its state.
    pub fn despawn_entity(&mut self, which: Entity) {
        if let Some(head) = self.entity_head(which) {
            self.kill_part(head);
        }
        if let Some(st) = self.player_state_mut(which) {
            st.spwn = 0;
            st.comm = 0;
            st.pcomm = 0;
            st.frames = 0;
        }
    }

    /// Set the command bitmask for a player slot.
    pub fn set_entity_input(&mut self, which: Entity, comm: u8) {
        if let Some(st) = self.player_state_mut(which) {
            st.pcomm = st.comm;
            st.comm = comm;
        }
    }

    /// True when the head is standing on solid ground (or an object the entity
    /// cannot displace).
    fn entity_grounded(&self, head: usize, x: i32, y: i32) -> bool {
        let below = self.pmap_at(x, y + 1);
        if below != 0 {
            let bt = crate::sim::pmap_type(below);
            let def = element(bt);
            if (def.properties & props::TYPE_SOLID) != 0 || def.weight > element(pt::STKM).weight {
                return true;
            }
        }
        self.wall_at(x, y + 1) != 0
    }

    /// Generic locomotion + collision + environmental health for a stickman or
    /// fighter head. This is the shared world behaviour; element-specific
    /// reactions stay in `updates.rs`.
    ///
    /// Movement is tile-based like the reference: each axis is attempted
    /// separately through the `can_move` table and wall layer, never teleporting
    /// through solids.
    pub fn step_entity(&mut self, which: Entity) -> StepReport {
        let Some(head) = self.entity_head(which) else {
            return StepReport::default();
        };
        let x = (self.parts[head].x + 0.5) as i32;
        let y = (self.parts[head].y + 0.5) as i32;
        let comm = self.player_state(which).map(|p| p.comm).unwrap_or(0);
        let elem = self.player_state(which).map(|p| p.elem).unwrap_or(0);

        // Environmental death: acid / fire / immersion in a deadly element.
        let under = self.pmap_at(x, y);
        if under != 0 {
            let ut = crate::sim::pmap_type(under);
            let udef = element(ut);
            if udef.abbr == "ACID" || udef.abbr == "LAVA" {
                self.kill_part(head);
                if let Some(st) = self.player_state_mut(which) {
                    st.spwn = 0;
                }
                return StepReport { died: true, ..Default::default() };
            }
        }

        let grounded = self.entity_grounded(head, x, y);
        let mut moved = false;
        let dx = if comm & input::LEFT != 0 {
            -1
        } else if comm & input::RIGHT != 0 {
            1
        } else {
            0
        };
        if dx != 0 {
            let nx = x + dx;
            if self.entity_can_enter(nx, y) {
                self.parts[head].x = nx as f32;
                self.parts[head].y = y as f32;
                moved = true;
            }
        }
        let mut ny = y;
        if comm & input::JUMP != 0 && grounded {
            ny = y - 1;
        } else if !grounded {
            ny = y + 1;
        }
        if ny != y && self.entity_can_enter(x, ny) {
            self.parts[head].y = ny as f32;
            moved = true;
        }

        // Element power: spawn a particle in front of the entity (reference uses
        // `pcomm` so holding the key emits at a fixed cadence).
        if comm & input::SPAWN != 0
            && elem > 0
            && (elem as usize) < crate::elements_generated::PT_NUM
            && element(elem as u16).enabled
        {
            let sx = x + dx;
            if self.pmap_at(sx, y) == 0 {
                self.create_part(sx, y, elem as u16);
            }
        }

        if let Some(st) = self.player_state_mut(which) {
            st.frames = st.frames.wrapping_add(1);
        }
        StepReport { moved, grounded, died: false }
    }

    /// True when an entity head may occupy `(x, y)`.
    fn entity_can_enter(&self, x: i32, y: i32) -> bool {
        if !self.in_bounds(x, y) {
            return false;
        }
        if x < CELL as i32 || y < CELL as i32 {
            return false;
        }
        if self.wall_blocks(x, y, pt::STKM) {
            return false;
        }
        let r = self.pmap_at(x, y);
        if r == 0 {
            return true;
        }
        let rt = crate::sim::pmap_type(r);
        if rt == pt::STKM || rt == pt::STKM2 || rt == pt::FIGH {
            return false;
        }
        // The entity can displace liquids and gases but not solids.
        let def = element(rt);
        (def.properties & props::TYPE_SOLID) == 0
    }

    /// Allocate a fighter slot.
    pub fn spawn_fighter(&mut self, x: i32, y: i32) -> Option<(usize, usize)> {
        if self.fighters.len() < MAX_FIGHTERS {
            self.fighters.push(PlayerState::default());
        }
        let slot = self
            .fighters
            .iter()
            .position(|f| f.spwn == 0)
            .or_else(|| {
                if self.fighters.len() < MAX_FIGHTERS {
                    self.fighters.push(PlayerState::default());
                    Some(self.fighters.len() - 1)
                } else {
                    None
                }
            })?;
        let head = self.spawn_entity(Entity::Fighter(slot), x, y)?;
        self.parts[head].tmp = slot as i32;
        self.figh_count += 1;
        Some((slot, head))
    }

    /// Kill a fighter and free its slot.
    pub fn despawn_fighter(&mut self, slot: usize) {
        if let Some(head) = self.entity_head(Entity::Fighter(slot)) {
            self.kill_part(head);
        }
        if let Some(f) = self.fighters.get_mut(slot) {
            if f.spwn != 0 {
                f.spwn = 0;
                self.figh_count = (self.figh_count - 1).max(0);
            }
        }
    }

    /// Reference `parts_avg` helper used by several stickman checks.
    pub fn parts_avg_type(&self, ci: usize, ni: usize, t: u16) -> u16 {
        let (pci, pni) = (&self.parts[ci], &self.parts[ni]);
        let (mx, my) = if t == pt::INSL {
            (
                (((pci.x + 0.5) + (pni.x + 0.5)) / 2.0) as i32,
                (((pci.y + 0.5) + (pni.y + 0.5)) / 2.0) as i32,
            )
        } else {
            (
                ((pci.x + pni.x) / 2.0 + 0.5) as i32,
                ((pci.y + pni.y) / 2.0 + 0.5) as i32,
            )
        };
        let r = self.pmap_at(mx, my);
        if r == 0 {
            return 0;
        }
        let rt = crate::sim::pmap_type(r);
        if t == pt::INSL || rt == t {
            rt
        } else {
            0
        }
    }

    /// Clamp a temperature for entity health checks.
    pub fn clamp_entity_temp(&self, t: f32) -> f32 {
        restrict_flt(t, MIN_TEMP, MAX_TEMP)
    }
}
