//! Shared transport primitives: piston stacks and soap links.
//!
//! Both behaviours are declared in element sources (`PSTN.cpp`, `SOAP.cpp`) but
//! require world-level operations that must not be re-implemented inside an
//! element update:
//!
//! - `PSTN` needs to displace an ordered stack of particles through the world
//!   without deleting anything (`CanMoveStack` / `MoveStack`).
//! - `SOAP` needs persistent particle-to-particle links (`tmp`/`tmp2` indices)
//!   with attach/detach and a tension solve.
//!
//! Agent 1 owns these operations; Agent 3 calls them from the element handlers.

use crate::elements_generated::pt;
use crate::sim::{pmap_id, pmap_pack, pmap_type, Sim, CELL};

/// Result of a stack probe. `pushed` are the particle slots that can move,
/// `spaces` is how many free cells exist ahead of them.
#[derive(Clone, Debug, Default)]
pub struct StackProbe {
    pub pushed: Vec<usize>,
    pub spaces: i32,
}

impl Sim {
    /// Reference `PSTN::CanMoveStack` (simplified to the fields that affect the
    /// observable result). Walks from `(stack_x, stack_y)` along the direction,
    /// stopping at a wall, a non-movable obstacle or `max_size`.
    pub fn can_move_stack(
        &self,
        stack_x: i32,
        stack_y: i32,
        dir_x: i32,
        dir_y: i32,
        max_size: i32,
        amount: i32,
    ) -> StackProbe {
        let mut probe = StackProbe::default();
        if amount <= 0 || (dir_x == 0 && dir_y == 0) {
            return probe;
        }
        let mut pos_x = stack_x;
        let mut pos_y = stack_y;
        let mut current = 0;
        let mut spaces = 0;
        while current < max_size + amount {
            if !self.in_bounds(pos_x, pos_y) {
                break;
            }
            if self.wall_at(pos_x, pos_y) != 0 {
                break;
            }
            let r = self.pmap_at(pos_x, pos_y);
            if r == 0 {
                spaces += 1;
                probe.spaces = spaces;
                if spaces >= amount {
                    break;
                }
            } else {
                let id = pmap_id(r) as usize;
                // Only real, movable particles are pushed (not walls/other pistons
                // are allowed as an arm, matching the reference).
                probe.pushed.push(id);
                current += 1;
                if current >= max_size {
                    break;
                }
            }
            pos_x += dir_x;
            pos_y += dir_y;
        }
        probe
    }

    /// Reference `PSTN::MoveStack`. Moves the probed particles by `amount` cells
    /// along the direction when there is room. Returns how many cells the front
    /// particle moved.
    ///
    /// This never deletes obstructing particles: if there is no room it returns
    /// 0 and leaves the world unchanged.
    pub fn move_stack(
        &mut self,
        stack_x: i32,
        stack_y: i32,
        dir_x: i32,
        dir_y: i32,
        max_size: i32,
        amount: i32,
    ) -> i32 {
        let probe = self.can_move_stack(stack_x, stack_y, dir_x, dir_y, max_size, amount);
        let shift = probe.spaces.min(amount);
        if shift <= 0 || probe.pushed.is_empty() {
            return 0;
        }
        // Move from the far end of the stack so we never overwrite a particle
        // that has not moved yet.
        for &id in probe.pushed.iter().rev() {
            if id >= self.parts.len() || self.parts[id].t == 0 {
                continue;
            }
            let src_x = (self.parts[id].x + 0.5) as i32;
            let src_y = (self.parts[id].y + 0.5) as i32;
            let dst_x = src_x + dir_x * shift;
            let dst_y = src_y + dir_y * shift;
            if !self.in_bounds(dst_x, dst_y) {
                continue;
            }
            let src_cell = self.idx(src_x, src_y);
            if self.pmap_at(src_x, src_y) != 0 && pmap_id(self.pmap[src_cell]) as usize == id {
                self.pmap[src_cell] = 0;
            }
            self.parts[id].x = dst_x as f32;
            self.parts[id].y = dst_y as f32;
            let dst_cell = self.idx(dst_x, dst_y);
            self.pmap[dst_cell] = pmap_pack(id as u32, self.parts[id].t);
        }
        shift
    }

    /// Reference `PSTN` extension: create `count` new `PSTN` arm particles ahead
    /// of `(x, y)` along the direction if the cells are free. Returns the number
    /// created. Does not run inside a particle update that already holds a
    /// mutable borrow of `self.parts` for the same index.
    pub fn extend_piston_arm(
        &mut self,
        x: i32,
        y: i32,
        dir_x: i32,
        dir_y: i32,
        count: i32,
        life: i32,
    ) -> i32 {
        let mut created = 0;
        let mut tx = x;
        let mut ty = y;
        for _ in 0..count.max(0) {
            tx += dir_x;
            ty += dir_y;
            if !self.in_bounds(tx, ty) {
                break;
            }
            let cell = self.idx(tx, ty);
            if self.pmap[cell] != 0 || self.photons[cell] != 0 {
                break;
            }
            if self.wall_at(tx, ty) != 0 {
                break;
            }
            if let Some(np) = self.create_part(tx, ty, pt::PSTN) {
                self.parts[np].life = life;
                created += 1;
            } else {
                break;
            }
        }
        created
    }

    /// Retract up to `count` `PSTN` arm particles behind `(x, y)`.
    pub fn retract_piston_arm(&mut self, x: i32, y: i32, dir_x: i32, dir_y: i32, count: i32) -> i32 {
        let mut removed = 0;
        let mut tx = x;
        let mut ty = y;
        for _ in 0..count.max(0) {
            tx -= dir_x;
            ty -= dir_y;
            if !self.in_bounds(tx, ty) {
                break;
            }
            let r = self.pmap_at(tx, ty);
            if r == 0 || pmap_type(r) != pt::PSTN {
                break;
            }
            let id = pmap_id(r) as usize;
            if id >= self.parts.len() {
                break;
            }
            self.kill_part(id);
            removed += 1;
        }
        removed
    }

    // -- SOAP links --------------------------------------------------------

    /// True when `i` currently has a "first mate" link.
    #[inline]
    pub fn soap_has_front(&self, i: usize) -> bool {
        i < self.parts.len() && (self.parts[i].ctype & 0x02) != 0
    }

    /// True when `i` currently has a "back mate" link.
    #[inline]
    pub fn soap_has_back(&self, i: usize) -> bool {
        i < self.parts.len() && (self.parts[i].ctype & 0x04) != 0
    }

    /// Reference `Element_SOAP_detach`: sever both links of `i` and clear the
    /// reciprocal flags so no dangling index survives.
    pub fn soap_detach(&mut self, i: usize) {
        if i >= self.parts.len() || self.parts[i].t != pt::SOAP {
            return;
        }
        let front = self.parts[i].tmp;
        let back = self.parts[i].tmp2;
        if self.parts[i].ctype & 0x02 != 0
            && front >= 0
            && (front as usize) < self.parts.len()
            && self.parts[front as usize].t == pt::SOAP
            && self.parts[front as usize].ctype & 0x04 != 0
        {
            self.parts[front as usize].ctype ^= 0x04;
        }
        if self.parts[i].ctype & 0x04 != 0
            && back >= 0
            && (back as usize) < self.parts.len()
            && self.parts[back as usize].t == pt::SOAP
            && self.parts[back as usize].ctype & 0x02 != 0
        {
            self.parts[back as usize].ctype ^= 0x02;
        }
        self.parts[i].ctype = 0;
    }

    /// Reference `attach`: link `i1` to `i2` in the first free direction.
    pub fn soap_attach(&mut self, i1: usize, i2: usize) {
        if i1 >= self.parts.len() || i2 >= self.parts.len() {
            return;
        }
        if i1 == i2 {
            return;
        }
        if self.parts[i2].ctype & 0x04 == 0 {
            self.parts[i1].ctype |= 0x02;
            self.parts[i1].tmp = i2 as i32;
            self.parts[i2].ctype |= 0x04;
            self.parts[i2].tmp2 = i1 as i32;
        } else if self.parts[i2].ctype & 0x02 == 0 {
            self.parts[i1].ctype |= 0x04;
            self.parts[i1].tmp2 = i2 as i32;
            self.parts[i2].ctype |= 0x02;
            self.parts[i2].tmp = i1 as i32;
        }
    }

    /// Reference `Element_SOAP_neighourLoop`: one constraint solve between two
    /// linked soap particles. `stiffness` is the reference `9/(d²+9)-0.5`
    /// coefficient scaled by the caller.
    pub fn soap_link_step(&mut self, i: usize, j: usize, stiffness: f32) {
        if i >= self.parts.len() || j >= self.parts.len() {
            return;
        }
        let dx = self.parts[i].x - self.parts[j].x;
        let dy = self.parts[i].y - self.parts[j].y;
        let d = stiffness;
        self.parts[j].vx -= dx * d;
        self.parts[j].vy -= dy * d;
        self.parts[i].vx += dx * d;
        self.parts[i].vy += dy * d;
    }

    /// Default soap constraint coefficient for a pair separated by `(dx, dy)`.
    pub fn soap_constraint(dx: f32, dy: f32) -> f32 {
        9.0 / (dx * dx + dy * dy + 9.0) - 0.5
    }

    /// Map a world pixel to its air cell index (used by transport helpers).
    #[inline]
    pub fn air_cell_of(&self, x: i32, y: i32) -> Option<usize> {
        let cx = x / CELL as i32;
        let cy = y / CELL as i32;
        if self.in_cell_bounds(cx, cy) {
            Some(self.cell_idx(cx, cy))
        } else {
            None
        }
    }
}
