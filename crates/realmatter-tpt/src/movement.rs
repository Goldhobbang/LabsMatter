//! Movement: `eval_move`, `try_move`, `move`, `do_move`, `PlanMove` and
//! `MovementPhase`, ported from the reference.

use crate::air::{
    WALL_DESTROYALL, WALL_DETECT, WALL_EHOLE, WALL_EWALL, WALL_FAN, WALL_STASIS, WALL_STREAM,
    WALL_WALLELEC,
};
use crate::element::{element, is_type};
use crate::elements_generated::pt;
use crate::elements_generated::props;
use crate::particle::flags;
use std::cell::RefCell;
use crate::sim::{
    pmap_id, pmap_pack, pmap_type, Sim, CELL, ISTP, MAX_VELOCITY, PMAPMASK, MIN_PRESSURE,
    MAX_PRESSURE,
};

pub struct PlanResult {
    pub fin_x: i32,
    pub fin_y: i32,
    pub clear_x: i32,
    pub clear_y: i32,
    pub fin_xf: f32,
    pub fin_yf: f32,
    pub clear_xf: f32,
    pub clear_yf: f32,
    pub vx: f32,
    pub vy: f32,
}

#[inline]
fn round_pos(v: f32) -> i32 {
    (v + 0.5) as i32
}

fn remainder_p(x: f32, y: f32) -> f32 {
    let r = x - (x / y).round() * y;
    if r < 0.0 {
        r + y
    } else {
        r
    }
}

thread_local! {
    /// Reusable visited-generation buffer for `flood_water`. Avoids a
    /// full world-sized zeroing allocation on every liquid particle.
    static FLOOD_VISITED: RefCell<(Vec<u32>, u32)> = RefCell::new((Vec::new(), 1));
}

impl Sim {
    /// `eval_move`: 0 = blocked, 1 = swap/move, 2 = occupy same space.
    pub fn eval_move(&self, t: u16, nx: i32, ny: i32) -> u8 {
        if nx < 0 || ny < 0 || nx as usize >= self.width || ny as usize >= self.height {
            return 0;
        }
        let cell = self.idx(nx, ny);
        let mut r = self.pmap[cell];
        if r != 0 {
            let id = pmap_id(r) as usize;
            if id < self.parts.len() {
                r = (r & !PMAPMASK) | self.parts[id].t as u32;
            }
        }
        let rt = pmap_type(r);
        let mut result = self.can_move(t, rt);
        if result == 3 {
            result = match rt {
                x if x == pt::LCRY => {
                    let id = pmap_id(r) as usize;
                    if self.parts[id].life > 5 {
                        2
                    } else {
                        0
                    }
                }
                x if x == pt::GPMP => {
                    let id = pmap_id(r) as usize;
                    if self.parts[id].life < 10 {
                        2
                    } else {
                        0
                    }
                }
                x if x == pt::INVIS => {
                    let id = pmap_id(r) as usize;
                    let p = &self.parts[id];
                    let cx = nx / CELL as i32;
                    let cy = ny / CELL as i32;
                    let pv = if self.in_cell_bounds(cx, cy) {
                        self.pv[self.cell_idx(cx, cy)]
                    } else {
                        0.0
                    };
                    let lim = if p.tmp > 0 { p.tmp as f32 } else { 4.0 };
                    if pv.abs() > lim {
                        2
                    } else {
                        0
                    }
                }
                x if x == pt::PVOD || x == pt::VOID => {
                    let id = pmap_id(r) as usize;
                    let p = &self.parts[id];
                    if p.ctype == t as i32 && (!self.is_pvod(x == pt::PVOD) || p.tmp != 0) {
                        1
                    } else if p.ctype == 0 && !self.is_pvod(x == pt::PVOD) {
                        1
                    } else {
                        0
                    }
                }
                x if x == pt::SWCH => {
                    let id = pmap_id(r) as usize;
                    if self.parts[id].life >= 10 {
                        2
                    } else {
                        0
                    }
                }
                _ => result,
            };
        }
        if self.wall_blocks(nx, ny, t) {
            return 0;
        }
        if self.wall_at(nx, ny) == WALL_EHOLE {
            let cx = nx / CELL as i32;
            let cy = ny / CELL as i32;
            let powered = self.in_cell_bounds(cx, cy)
                && self.emap[self.cell_idx(cx, cy)] != 0;
            if !powered
                && (element(t).properties & props::TYPE_SOLID) == 0
                && (element(rt).properties & props::TYPE_SOLID) == 0
            {
                return 2;
            }
        }
        result
    }

    fn is_pvod(&self, _pvod: bool) -> bool {
        false
    }

    /// `try_move`: 0 = blocked, 1 = moved or swapped.
    pub fn try_move(&mut self, i: usize, x: i32, y: i32, nx: i32, ny: i32) -> i32 {
        if nx == x && ny == y {
            return 1;
        }
        if nx < 0 || ny < 0 || nx as usize >= self.width || ny as usize >= self.height {
            return 1;
        }
        let t = self.parts[i].t;
        let cell = self.idx(nx, ny);
        let mut r = self.pmap[cell];
        if r != 0 {
            let id = pmap_id(r) as usize;
            if id < self.parts.len() {
                r = (r & !PMAPMASK) | self.parts[id].t as u32;
            }
        }
        let rt = pmap_type(r);
        let mut e = self.eval_move(t, nx, ny);

        // Half-silvered mirror (BMTL).
        if e == 0
            && t == pt::PHOT
            && ((rt == pt::BMTL && self.rng.chance(1, 2))
                || pmap_type(self.pmap[self.idx(x, y)]) == pt::BMTL)
        {
            e = 2;
        }

        if e == 0 {
            // Energy particles transfer heat and impart properties.
            if !is_type(t, props::TYPE_ENERGY) {
                return 0;
            }
            return 0;
        }

        if e == 2 {
            // Same-space: photon heat conduction into the target.
            if t == pt::PHOT && r != 0 {
                let id = pmap_id(r) as usize;
                let t1 = self.parts[i].temp;
                let t2 = self.parts[id].temp;
                self.parts[id].temp = ((t1 + t2) * 0.5).clamp(0.0, 9999.0);
            }
            return 1;
        }

        // --- swap block ---------------------------------------------------
        match rt {
            x if x == pt::VOID || x == pt::PVOD => {
                self.kill_part(i);
                return 0;
            }
            x if x == pt::BHOL || x == pt::NBHL => {
                if r != 0 {
                    let id = pmap_id(r) as usize;
                    self.parts[id].temp += self.parts[i].temp * 0.5;
                    self.parts[id].temp = self.parts[id].temp.min(9999.0);
                }
                self.kill_part(i);
                return 0;
            }
            _ => {}
        }
        if t == pt::NEUT && (element(rt).properties & props::PROP_NEUTABSORB) != 0 {
            self.kill_part(i);
            return 0;
        }
        if t == pt::CNCT {
            // Refuse if a CNCT/ROCK sits below (prevents stacking).
            let below = if ny + 1 < self.height as i32 {
                pmap_type(self.pmap[self.idx(nx, ny + 1)])
            } else {
                0
            };
            if below == pt::CNCT || below == pt::ROCK {
                return 0;
            }
        }
        if self.wall_at(nx, ny) == WALL_EHOLE {
            let cx = nx / CELL as i32;
            let cy = ny / CELL as i32;
            if self.in_cell_bounds(cx, cy) && self.emap[self.cell_idx(cx, cy)] == 0 {
                return 0;
            }
        }
        let ri = pmap_id(r) as usize;
        if r != 0 && ri < self.parts.len() {
            // Swap: the target moves to the mover's previous position.
            if self.pmap[cell] != 0 && pmap_id(self.pmap[cell]) as usize == ri {
                self.pmap[cell] = 0;
            }
            let (ox, oy) = (self.parts[i].x, self.parts[i].y);
            self.parts[ri].x = ox;
            self.parts[ri].y = oy;
            let rx = round_pos(self.parts[ri].x);
            let ry = round_pos(self.parts[ri].y);
            if self.in_bounds(rx, ry) {
                let rcell = self.idx(rx, ry);
                self.pmap[rcell] = pmap_pack(ri as u32, self.parts[ri].t);
            }
        }
        1
    }

    /// `do_move`: plan-aware move with edge-loop wrapping.
    pub fn do_move(&mut self, i: usize, x: i32, y: i32, nxf: f32, nyf: f32) -> i32 {
        let mut nxf = nxf;
        let mut nyf = nyf;
        let mut nx = round_pos(nxf);
        let mut ny = round_pos(nyf);
        if self.settings.edge_mode == crate::sim::EdgeMode::Loop {
            let w = self.width as f32;
            let h = self.height as f32;
            if nx < CELL as i32 || nx >= self.width as i32 - CELL as i32 {
                nxf = remainder_p(nxf - CELL as f32 + 0.5, w - CELL as f32 * 2.0) + CELL as f32 - 0.5;
            }
            if ny < CELL as i32 || ny >= self.height as i32 - CELL as i32 {
                nyf = remainder_p(nyf - CELL as f32 + 0.5, h - CELL as f32 * 2.0) + CELL as f32 - 0.5;
            }
            nx = round_pos(nxf);
            ny = round_pos(nyf);
        }
        if self.parts[i].t == 0 {
            return 0;
        }
        let result = self.try_move(i, x, y, nx, ny);
        if result != 0 {
            if !self.move_particle(i, x, y, nxf, nyf) {
                return -1;
            }
            // Shock propagation: a moving particle nudges pressure in its
            // cell and the four orthogonal neighbours (air/liquid).
            let ccx = nx / CELL as i32;
            let ccy = ny / CELL as i32;
            if self.in_cell_bounds(ccx, ccy) {
                let vci = self.cell_idx(ccx, ccy);
                let p = self.pv[vci];
                for (rx, ry) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                    let ncx = ccx + rx;
                    let ncy = ccy + ry;
                    if !self.in_cell_bounds(ncx, ncy) {
                        continue;
                    }
                    let nci = self.cell_idx(ncx, ncy);
                    let np = self.pv[nci];
                    let delta = (p - np) * 0.08;
                    self.pv[nci] = (self.pv[nci] + delta).clamp(MIN_PRESSURE, MAX_PRESSURE);
                    self.pv[vci] = (self.pv[vci] - delta).clamp(MIN_PRESSURE, MAX_PRESSURE);
                }
            }
        }
        result
    }

    /// `move`: commit the position and patch `pmap`.
    pub fn move_particle(&mut self, i: usize, x: i32, y: i32, nxf: f32, nyf: f32) -> bool {
        let nx = round_pos(nxf);
        let ny = round_pos(nyf);
        self.parts[i].x = nxf;
        self.parts[i].y = nyf;
        if nx != x || ny != y {
            if self.in_bounds(x, y) {
                let cell = self.idx(x, y);
                if self.pmap[cell] != 0 && pmap_id(self.pmap[cell]) as usize == i {
                    self.pmap[cell] = 0;
                }
                if self.photons[cell] != 0 && pmap_id(self.photons[cell]) as usize == i {
                    self.photons[cell] = 0;
                }
            }
            if nx < CELL as i32
                || ny < CELL as i32
                || nx >= self.width as i32 - CELL as i32
                || ny >= self.height as i32 - CELL as i32
            {
                if self.settings.edge_mode != crate::sim::EdgeMode::Loop {
                    self.kill_part(i);
                    return false;
                }
            }
            if !self.in_bounds(nx, ny) {
                self.kill_part(i);
                return false;
            }
            let t = self.parts[i].t;
            let cell = self.idx(nx, ny);
            if (element(t).properties & props::TYPE_ENERGY) != 0 {
                self.photons[cell] = pmap_pack(i as u32, t);
            } else if t != 0 {
                self.pmap[cell] = pmap_pack(i as u32, t);
            }
        }
        true
    }

    /// `PlanMove<true>`.
    pub fn plan_move(&self, i: usize, x: i32, y: i32) -> PlanResult {
        let p = &self.parts[i];
        let mut vx = p.vx;
        let mut vy = p.vy;
        let mv0 = vx.abs().max(vy.abs());
        if mv0 < ISTP || mv0.is_nan() {
            let clear_xf = p.x;
            let clear_yf = p.y;
            return PlanResult {
                clear_x: x,
                clear_y: y,
                clear_xf,
                clear_yf,
                fin_xf: clear_xf + vx,
                fin_yf: clear_yf + vy,
                fin_x: round_pos(clear_xf + vx),
                fin_y: round_pos(clear_yf + vy),
                vx,
                vy,
            };
        }
        let mut mv = mv0;
        if mv > MAX_VELOCITY {
            vx *= MAX_VELOCITY / mv;
            vy *= MAX_VELOCITY / mv;
            mv = MAX_VELOCITY;
        }
        let dx = vx * ISTP / mv;
        let dy = vy * ISTP / mv;
        let mut fin_xf = p.x;
        let mut fin_yf = p.y;
        let mut fin_x = round_pos(fin_xf);
        let mut fin_y = round_pos(fin_yf);
        let t = p.t;
        let loop_mode = self.settings.edge_mode == crate::sim::EdgeMode::Loop;
        let ehole_start = self.in_bounds(fin_x, fin_y)
            && self.wall_at(fin_x, fin_y) == WALL_EHOLE
            && self.emap_at(fin_x, fin_y) == 0;
        loop {
            mv -= ISTP;
            fin_xf += dx;
            fin_yf += dy;
            fin_x = round_pos(fin_xf);
            fin_y = round_pos(fin_yf);
            if loop_mode {
                let w = self.width as f32;
                let h = self.height as f32;
                if !(fin_xf >= CELL as f32 - 0.5 && fin_xf < w - CELL as f32 - 0.5) {
                    fin_xf = remainder_p(fin_xf - CELL as f32 + 0.5, w - CELL as f32 * 2.0) + CELL as f32 - 0.5;
                }
                if !(fin_yf >= CELL as f32 - 0.5 && fin_yf < h - CELL as f32 - 0.5) {
                    fin_yf = remainder_p(fin_yf - CELL as f32 + 0.5, h - CELL as f32 * 2.0) + CELL as f32 - 0.5;
                }
                fin_x = round_pos(fin_xf);
                fin_y = round_pos(fin_yf);
            }
            if mv <= 0.0 {
                fin_xf = self.parts[i].x + vx;
                fin_yf = self.parts[i].y + vy;
                if loop_mode {
                    let w = self.width as f32;
                    let h = self.height as f32;
                    if !(fin_xf >= CELL as f32 - 0.5 && fin_xf < w - CELL as f32 - 0.5) {
                        fin_xf = remainder_p(fin_xf - CELL as f32 + 0.5, w - CELL as f32 * 2.0) + CELL as f32 - 0.5;
                    }
                    if !(fin_yf >= CELL as f32 - 0.5 && fin_yf < h - CELL as f32 - 0.5) {
                        fin_yf = remainder_p(fin_yf - CELL as f32 + 0.5, h - CELL as f32 * 2.0) + CELL as f32 - 0.5;
                    }
                }
                return PlanResult {
                    fin_x: round_pos(fin_xf),
                    fin_y: round_pos(fin_yf),
                    fin_xf,
                    fin_yf,
                    clear_xf: fin_xf - dx,
                    clear_yf: fin_yf - dy,
                    clear_x: round_pos(fin_xf - dx),
                    clear_y: round_pos(fin_yf - dy),
                    vx,
                    vy,
                };
            }
            if !self.in_bounds(fin_x, fin_y) {
                return PlanResult {
                    fin_x,
                    fin_y,
                    fin_xf,
                    fin_yf,
                    clear_xf: fin_xf - dx,
                    clear_yf: fin_yf - dy,
                    clear_x: round_pos(fin_xf - dx),
                    clear_y: round_pos(fin_yf - dy),
                    vx,
                    vy,
                };
            }
            let cell = self.idx(fin_x, fin_y);
            let pr = self.pmap[cell];
            let eval = self.eval_move(t, fin_x, fin_y);
            let special_three = self.can_move(t, pmap_type(pr)) == 3;
            if eval == 0
                || (special_three && eval == 1)
                || (t == pt::PHOT && pr != 0)
                || self.wall_at(fin_x, fin_y) == WALL_DESTROYALL
                || ehole_start
                    != (self.wall_at(fin_x, fin_y) == WALL_EHOLE
                        && self.emap_at(fin_x, fin_y) == 0)
            {
                return PlanResult {
                    fin_x,
                    fin_y,
                    fin_xf,
                    fin_yf,
                    clear_xf: fin_xf - dx,
                    clear_yf: fin_yf - dy,
                    clear_x: round_pos(fin_xf - dx),
                    clear_y: round_pos(fin_yf - dy),
                    vx,
                    vy,
                };
            }
            if self.wall_at(fin_x, fin_y) == WALL_DETECT {
                let cx = fin_x / CELL as i32;
                let cy = fin_y / CELL as i32;
                if self.in_cell_bounds(cx, cy) && self.emap[self.cell_idx(cx, cy)] < 8 {
                    // Set during movement (PlanMove<true>).
                    // Handled by the caller through `note_detector`.
                }
            }
        }
    }

    #[inline]
    fn emap_at(&self, x: i32, y: i32) -> u8 {
        let cx = x / CELL as i32;
        let cy = y / CELL as i32;
        if self.in_cell_bounds(cx, cy) {
            self.emap[self.cell_idx(cx, cy)]
        } else {
            0
        }
    }

    /// `MovementPhase`: dispatch by state class.
    pub fn movement_phase(&mut self, i: usize) {
        let p = self.parts[i];
        let t = p.t;
        let x = round_pos(p.x);
        let y = round_pos(p.y);
        let mut plan = self.plan_move(i, x, y);
        self.parts[i].vx = plan.vx;
        self.parts[i].vy = plan.vy;
        let stagnant = self.parts[i].flags & flags::STAGNANT != 0;
        self.parts[i].flags &= !flags::STAGNANT;

        if is_type(t, props::TYPE_ENERGY) {
            self.move_energy(i, x, y, &mut plan);
            return;
        }
        if element(t).falldown == 0 {
            // Gases / solids: bounce off obstacles, no lateral scanning.
            if self.do_move(i, x, y, plan.fin_xf, plan.fin_yf) == 0 {
                let vx = self.parts[i].vx;
                let vy = self.parts[i].vy;
                let bounce_x = if plan.fin_x != x { -0.5 } else { 0.0 };
                let bounce_y = if plan.fin_y != y { -0.5 } else { 0.0 };
                self.parts[i].vx = vx * (1.0 + bounce_x) * 0.5;
                self.parts[i].vy = vy * (1.0 + bounce_y) * 0.5;
                self.parts[i].flags |= flags::STAGNANT;
            }
            return;
        }

        // Powder / liquid.
        if self.do_move(i, x, y, plan.fin_xf, plan.fin_yf) == 0 {
            let moved = if plan.fin_x != x && self.do_move(i, x, y, plan.fin_xf, plan.clear_yf) != 0
            {
                true
            } else if plan.fin_y != y && self.do_move(i, x, y, plan.clear_xf, plan.fin_yf) != 0 {
                true
            } else {
                false
            };
            if !moved {
                let r = self.rng.between(0, 1) * 2 - 1;
                let vx = self.parts[i].vx;
                let vy = self.parts[i].vy;
                if (plan.clear_x != x || plan.clear_y != y) && (vx.abs() > 0.01 || vy.abs() > 0.01) {
                    let mut dx = vx - vy * r as f32;
                    let mut dy = vy + vx * r as f32;
                    let mv = dx.abs().max(dy.abs());
                    if mv != 0.0 {
                        dx /= mv;
                        dy /= mv;
                    }
                    if self.do_move(i, x, y, plan.clear_xf + dx, plan.clear_yf + dy) != 0 {
                        self.collide(i);
                        return;
                    }
                    let sw = dx;
                    dx = dy * r as f32;
                    dy = -sw * r as f32;
                    if self.do_move(i, x, y, plan.clear_xf + dx, plan.clear_yf + dy) != 0 {
                        self.collide(i);
                        return;
                    }
                }
                // Liquid lateral spreading.
                if element(t).falldown > 1 {
                    if self.liquid_spread(i, x, y, &plan, stagnant) {
                        self.collide(i);
                        return;
                    }
                }
                if (plan.clear_x != x || plan.clear_y != y)
                    && self.do_move(i, x, y, plan.clear_xf, plan.clear_yf) != 0
                {
                    self.collide(i);
                    return;
                }
                self.parts[i].flags |= flags::STAGNANT;
                self.collide(i);
            }
        }
    }

    #[inline]
    fn collide(&mut self, i: usize) {
        let c = element(self.parts[i].t).collision;
        self.parts[i].vx *= c;
        self.parts[i].vy *= c;
    }

    /// Horizontal equalisation scan used by liquids (`Falldown > 1`).
    fn liquid_spread(&mut self, i: usize, x: i32, y: i32, plan: &PlanResult, stagnant: bool) -> bool {
        let t = self.parts[i].t;
        let nt = 1; // conservative: always search the full radius
        let rt = if !stagnant || nt != 0 { 30 } else { 10 };
        let mut r = self.rng.between(0, 1) * 2 - 1;
        if r == 0 {
            r = 1;
        }
        let mut s = 0i32;
        let mut nx = x;
        let mut ny = y;
        let mut j = plan.clear_x + r;
        while j >= plan.clear_x - rt && j < plan.clear_x + rt && j >= 0 && j < self.width as i32 {
            let cell_fin = self.idx(j, plan.fin_y.max(0).min(self.height as i32 - 1));
            let ty = pmap_type(self.pmap[cell_fin]);
            let wall = self.wall_at(j, plan.fin_y) != 0;
            if ty != t || wall {
                s = self.do_move(i, x, y, j as f32, plan.fin_yf);
                if s != 0 {
                    nx = j;
                    ny = plan.fin_y;
                    break;
                }
            }
            if plan.fin_y != plan.clear_y {
                let cell_clr = self.idx(j, plan.clear_y.max(0).min(self.height as i32 - 1));
                let ty2 = pmap_type(self.pmap[cell_clr]);
                let wall2 = self.wall_at(j, plan.clear_y) != 0;
                if ty2 != t || wall2 {
                    s = self.do_move(i, x, y, j as f32, plan.clear_yf);
                    if s != 0 {
                        nx = j;
                        ny = plan.clear_y;
                        break;
                    }
                }
            }
            let cell_clr = self.idx(j, plan.clear_y.max(0).min(self.height as i32 - 1));
            let ty2 = pmap_type(self.pmap[cell_clr]);
            if ty2 != t || (self.wall_at(j, plan.clear_y) != 0 && self.wall_at(j, plan.clear_y) != WALL_STREAM) {
                break;
            }
            j += r;
        }
        if s == 1 {
            let dir = if self.parts[i].vy > 0.0 { 1 } else { -1 };
            let mut k = ny + dir;
            while k >= 0
                && k < self.height as i32
                && k >= ny - rt
                && k < ny + rt
            {
                let cell = self.idx(nx.max(0).min(self.width as i32 - 1), k);
                let ty = pmap_type(self.pmap[cell]);
                let wall = self.wall_at(nx, k) != 0;
                if (ty != t || wall) && self.do_move(i, nx, ny, nx as f32, k as f32) != 0 {
                    break;
                }
                if ty != t || (wall && self.wall_at(nx, k) != WALL_STREAM) {
                    break;
                }
                k += dir;
            }
            true
        } else if s == -1 {
            true
        } else {
            false
        }
    }

    /// Energy particles (`TYPE_ENERGY`): photons, neutrons, electricity, etc.
    fn move_energy(&mut self, i: usize, x: i32, y: i32, plan: &mut PlanResult) {
        let t = self.parts[i].t;
        let def = element(t);
        if self.parts[i].flags & flags::SKIPMOVE != 0 {
            self.parts[i].flags &= !flags::SKIPMOVE;
            return;
        }
        // Energy particles always try to move at unit steps along their velocity.
        let mut nx = plan.fin_x;
        let mut ny = plan.fin_y;
        if t == pt::PHOT {
            // Photons move one pixel at a time in their direction.
            let vx = self.parts[i].vx;
            let vy = self.parts[i].vy;
            if vx == 0.0 && vy == 0.0 {
                return;
            }
            nx = x + vx.signum() as i32;
            ny = y + vy.signum() as i32;
            // Multi-step along the dominant axis for speed > 1.
            let steps = vx.abs().max(vy.abs()).max(1.0) as i32;
            let mut cx = x;
            let mut cy = y;
            for _ in 0..steps.min(64) {
                let step_x = cx + vx.signum() as i32;
                let step_y = cy + vy.signum() as i32;
                let moved = self.do_move(i, cx, cy, step_x as f32, step_y as f32);
                if moved == 0 {
                    // Basic refraction at a material boundary: try swapping the
                    // velocity components once before giving up.
                    let target = if self.in_bounds(step_x, step_y) {
                        pmap_type(self.pmap[self.idx(step_x, step_y)])
                    } else {
                        0
                    };
                    if target == pt::GLAS
                        || target == pt::BGLA
                        || target == pt::WATR
                        || target == pt::ICEI
                    {
                        let (ovx, ovy) = (self.parts[i].vx, self.parts[i].vy);
                        self.parts[i].vx = ovy;
                        self.parts[i].vy = ovx;
                        if self.do_move(i, cx, cy, step_x as f32, step_y as f32) != 0 {
                            cx = round_pos(self.parts[i].x);
                            cy = round_pos(self.parts[i].y);
                            continue;
                        }
                        self.parts[i].vx = ovx;
                        self.parts[i].vy = ovy;
                    }
                    break;
                }
                cx = round_pos(self.parts[i].x);
                cy = round_pos(self.parts[i].y);
            }
            self.parts[i].life -= 1;
            if self.parts[i].life <= 0 || self.parts[i].t == 0 {
                if self.parts[i].t != 0 {
                    self.kill_part(i);
                }
            }
            return;
        }
        if t == pt::NEUT {
            if self.rng.chance(1, 10) {
                self.parts[i].vx = -self.parts[i].vx;
                self.parts[i].vy = -self.parts[i].vy;
            }
            self.parts[i].life -= 1;
            if self.parts[i].life <= 0 {
                self.kill_part(i);
                return;
            }
            if !self.in_bounds(nx, ny) && self.settings.edge_mode != crate::sim::EdgeMode::Loop {
                self.kill_part(i);
                return;
            }
            if self.do_move(i, x, y, nx as f32, ny as f32) == 0 {
                // In loop mode `do_move` wraps the target, so the stale `nx/ny`
                // must not be used for a pmap lookup.
                if !self.in_bounds(nx, ny) {
                    return;
                }
                let r = self.pmap[self.idx(nx, ny)];
                let rt = pmap_type(r);
                if rt == pt::GLAS && self.rng.chance(1, 10) {
                    self.parts[i].vx = -self.parts[i].vx;
                    self.parts[i].vy = -self.parts[i].vy;
                } else if rt == pt::GLAS || rt == pt::WATR || rt == pt::ICEI {
                    // Basic refraction: bend the neutron path instead of dying.
                    let (ovx, ovy) = (self.parts[i].vx, self.parts[i].vy);
                    self.parts[i].vx = ovy;
                    self.parts[i].vy = ovx;
                    if self.do_move(i, x, y, nx as f32, ny as f32) == 0 {
                        self.parts[i].vx = ovx;
                        self.parts[i].vy = ovy;
                        self.kill_part(i);
                    }
                } else {
                    self.kill_part(i);
                }
            }
            return;
        }
        if t == pt::ELEC {
            self.parts[i].life -= 1;
            if self.parts[i].life <= 0 {
                self.kill_part(i);
                return;
            }
        }
        if !self.in_bounds(nx, ny) && self.settings.edge_mode != crate::sim::EdgeMode::Loop {
            self.kill_part(i);
            return;
        }
        if self.do_move(i, x, y, nx as f32, ny as f32) == 0 {
            // Bounce along the blocked axis.
            if plan.fin_x != x {
                self.parts[i].vx = -self.parts[i].vx;
            }
            if plan.fin_y != y {
                self.parts[i].vy = -self.parts[i].vy;
            }
            self.parts[i].flags |= flags::STAGNANT;
        }
        let _ = def;
    }

    /// Horizontal liquid equalization (`water_equal_test` / `flood_water`).
    ///
    /// Port of `Simulation::flood_water`: when a connected body of liquid
    /// (Falldown == 2) has empty space above part of it, teleport the particle
    /// into that gap so the surface levels out. Deterministic (no RNG).
    pub fn flood_water(&mut self, i: usize, original_x: i32, original_y: i32) -> bool {
        if !self.settings.water_equal_test {
            return false;
        }
        if original_x < 0 || original_y < 0 || original_x >= self.width as i32 || original_y >= self.height as i32 {
            return false;
        }
        let t = self.parts[i].t;
        if t == 0 {
            return false;
        }
        if element(t).falldown != 2 {
            return false;
        }
        let start_cell = self.idx(original_x, original_y);
        if self.pmap[start_cell] == 0 {
            return false;
        }

        // Reuse a process-global visited buffer keyed by a generation counter so
        // we do not reallocate/zero a full world-sized bitmap per call. The
        // observed visited set is identical to the previous `Vec<u8>` bitmap.
        FLOOD_VISITED.with(|flood_cell| {
            let mut flood_state = flood_cell.borrow_mut();
            let needed = self.width * self.height;
            if flood_state.0.len() < needed {
                flood_state.0.resize(needed, 0);
            }
            flood_state.1 = flood_state.1.wrapping_add(1);
            if flood_state.1 == 0 {
                for v in flood_state.0.iter_mut() {
                    *v = 0;
                }
                flood_state.1 = 1;
            }
            let gen = flood_state.1;
            let visited = &mut flood_state.0;

            let mut stack: Vec<(i32, i32)> = Vec::new();
            stack.push((original_x, original_y));

            while let Some((x, y)) = stack.pop() {
            if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
                continue;
            }
            let base = self.idx(x, y);
            if visited[base] == gen {
                continue;
            }

            // Horizontal span of connected liquid on this row.
            let mut x1 = x;
            let mut x2 = x;
            while x1 >= CELL as i32 {
                let left = x1 - 1;
                if left < 0 {
                    break;
                }
                let cell = self.idx(left, y);
                let rt = pmap_type(self.pmap[cell]);
                if element(rt).falldown != 2 || visited[cell] == gen {
                    break;
                }
                x1 = left;
            }
            while x2 < self.width as i32 - CELL as i32 {
                let right = x2 + 1;
                if right >= self.width as i32 {
                    break;
                }
                let cell = self.idx(right, y);
                let rt = pmap_type(self.pmap[cell]);
                if element(rt).falldown != 2 || visited[cell] == gen {
                    break;
                }
                x2 = right;
            }

            // Mark the span and try to fill a gap above it.
            for xx in x1..=x2 {
                if xx < 0 || xx >= self.width as i32 {
                    continue;
                }
                let cell = self.idx(xx, y);
                visited[cell] = gen;
                if (y - 1) > original_y {
                    let ny = y - 1;
                    if ny >= 0 && ny < self.height as i32 {
                        let target = self.idx(xx, ny);
                        if self.pmap[target] == 0 {
                            // Deterministic choice: first empty cell in [xx, x2].
                            let mut chosen: Option<i32> = None;
                            if self.eval_move(t, xx, ny) != 0 {
                                chosen = Some(xx);
                            } else {
                                for cand in (xx + 1)..=x2 {
                                    if cand < 0 || cand >= self.width as i32 {
                                        break;
                                    }
                                    let tc = self.idx(cand, ny);
                                    if self.pmap[tc] == 0 && self.eval_move(t, cand, ny) != 0 {
                                        chosen = Some(cand);
                                        break;
                                    }
                                }
                            }
                            if let Some(cx) = chosen {
                                let res = self.do_move(i, original_x, original_y, cx as f32, ny as f32);
                                if res != 0 {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }

            // Explore connected liquid above/below.
            if y >= CELL as i32 + 1 {
                for xx in x1..=x2 {
                    if xx < 0 || xx >= self.width as i32 {
                        continue;
                    }
                    let up = y - 1;
                    let cell = self.idx(xx, up);
                    let rt = pmap_type(self.pmap[cell]);
                    if element(rt).falldown == 2 && visited[cell] != gen {
                        stack.push((xx, up));
                    }
                }
            }
            if y < self.height as i32 - CELL as i32 - 1 {
                for xx in x1..=x2 {
                    if xx < 0 || xx >= self.width as i32 {
                        continue;
                    }
                    let down = y + 1;
                    let cell = self.idx(xx, down);
                    let rt = pmap_type(self.pmap[cell]);
                    if element(rt).falldown == 2 && visited[cell] != gen {
                        stack.push((xx, down));
                    }
                }
            }
        }

        false
        })
    }
}

// Keep the imported wall constants referenced so unused warnings stay quiet on
// platforms where the branches are optimised out.
#[allow(dead_code)]
const _WALL_REFS: (u8, u8, u8, u8, u8) = (
    WALL_FAN,
    WALL_STASIS,
    WALL_EWALL,
    WALL_WALLELEC,
    WALL_STREAM,
);
