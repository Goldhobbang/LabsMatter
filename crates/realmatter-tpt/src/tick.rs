//! The tick pipeline: `BeforeSim` → `UpdateParticles` → `AfterSim`.

use crate::air::{
    WALL_DESTROYALL, WALL_DETECT, WALL_EHOLE, WALL_EWALL, WALL_STASIS, WALL_WALL, WALL_WALLELEC,
    WALL_BLOCKAIR, WALL_ALLOWAIR, WALL_ALLOWENERGY, WALL_ALLOWGAS, WALL_ALLOWLIQUID,
    WALL_ALLOWPOWDER,
};
use crate::element::element;
use crate::elements_generated::props;
use crate::particle::flags;
use crate::sim::{pmap_id, pmap_type, restrict_flt, Sim, CELL, MAX_TEMP, MIN_TEMP};

#[derive(Clone, Copy, Debug)]
pub struct Neighbourhood {
    pub surround: [u32; 8],
    pub surround_space: i32,
    pub nt: i32,
    pub pgrav_x: f32,
    pub pgrav_y: f32,
}

impl Sim {
    /// `BeforeSim`.
    pub fn before_sim(&mut self) {
        // 0. Numerical guard: clamp non-finite particle/field state before it
        //    can propagate through the air solver or pmap rebuild. No-op for
        //    healthy state (see `Sim::sanitize`).
        self.sanitize();
        // 1. Air.
        self.update_air();
        // 2. Ambient heat.
        if self.settings.ambient_heat {
            self.update_airh();
        }
        // 3. Newtonian gravity pass (this also clears the mass grid).
        if self.settings.newtonian_gravity {
            self.update_newtonian_gravity();
        }
        // 4. EMP decoration decay.
        if self.emp_decor > 0 {
            self.emp_decor -= self.emp_decor / 25 + 2;
        }
        if self.emp_decor < 0 {
            self.emp_decor = 0;
        }
        // 5. Tick counter.
        self.tick = self.tick.wrapping_add(1);
        // 6a. In Loop mode, particles that ended up in the outer CELL margin are
        //     wrapped to their counterpart (reference `remainder_p`) before the
        //     pmap rebuild, so they are not left (and later killed) in the
        //     margin.
        if self.settings.edge_mode == crate::sim::EdgeMode::Loop {
            let w = self.width as f32;
            let h = self.height as f32;
            for p in self.parts.iter_mut() {
                if p.t == 0 {
                    continue;
                }
                let xi = (p.x + 0.5) as i32;
                let yi = (p.y + 0.5) as i32;
                if xi < CELL as i32 || xi >= self.width as i32 - CELL as i32 {
                    p.x = crate::sim::remainder_p(p.x - CELL as f32 + 0.5, w - CELL as f32 * 2.0)
                        + CELL as f32
                        - 0.5;
                }
                if yi < CELL as i32 || yi >= self.height as i32 - CELL as i32 {
                    p.y = crate::sim::remainder_p(p.y - CELL as f32 + 0.5, h - CELL as f32 * 2.0)
                        + CELL as f32
                        - 0.5;
                }
            }
        }
        // 6. Rebuild pmap and apply life decay.
        // ref Simulation.cpp:3763-3766 - gravity mask recompute before RecalcFreeParticles/pmap rebuild
        if self.grav_wall_changed {
            self.update_gravity_mask();
            self.grav_wall_changed = false;
        }
        self.recalc_and_life();
        // 7. emap decrement / wall-ambient block maps (ref 3777-3785).
        for e in self.emap.iter_mut() {
            if *e != 0 {
                *e -= 1;
            }
        }
        self.refresh_air_block_maps();
        // 8. CheckStacking (ref 3786-3790).
        if self.rng.chance(1, 10) {
            self.check_stacking();
        }
        // 9. LOVE/LOLZ BeforeSim stamp (ref 3792-3860).
        if self.element_count[crate::elements_generated::pt::LOVE as usize] > 0
            || self.element_count[crate::elements_generated::pt::LOLZ as usize] > 0
        {
            crate::updates::before_sim_love_lolz(self);
        }
        // 10. Global WIRE previous-state snapshot (ref 3862-3876).
        //     Before particle updates, snapshot each WIRE's ctype into tmp so all
        //     WIRE updates in this tick read a consistent previous state.
        if self.element_count[crate::elements_generated::pt::WIRE as usize] > 0 {
            let wire = crate::elements_generated::pt::WIRE;
            for cell in 0..self.pmap.len() {
                let r = self.pmap[cell];
                if r == 0 {
                    continue;
                }
                if crate::sim::pmap_type(r) == wire {
                    let id = crate::sim::pmap_id(r) as usize;
                    if id < self.parts.len() && self.parts[id].t == wire {
                        self.parts[id].tmp = self.parts[id].ctype;
                    }
                }
            }
        }
        // 11. PPIP staged promotion (ref 3878-3890).
        if self.ppip_changed {
            for i in 0..self.parts.len() {
                if self.parts[i].t == 0 {
                    continue;
                }
                // Reference promotes only PT_PPIP; PIPE keeps staged bits 28..30 untouched.
                if self.parts[i].t != crate::elements_generated::pt::PPIP {
                    continue;
                }
                let tmp = self.parts[i].tmp as u32;
                let promoted = tmp | ((tmp & 0xE000_0000) >> 3);
                self.parts[i].tmp = (promoted & !0xE000_0000u32) as i32;
            }
            self.ppip_changed = false;
        }
        // 12. GoL (ref 3892-3898).
        if self.element_count[crate::elements_generated::pt::LIFE as usize] > 0 {
            self.gol_counter += 1;
            if self.gol_counter >= self.settings.gol_speed {
                self.gol_counter = 0;
                self.simulate_gol();
            }
        }
        // 13. ISWIRE wireless advance (ref 3900-3908).
        //     Reference: while `ISWIRE > 0`, promote wireless next->current and
        //     decrement `ISWIRE`. This runs in BeforeSim before particle updates;
        //     WIFI sets `ISWIRE = 2` during update via `WirelessStore::note_spark`,
        //     so a spark becomes observable on the following tick's BeforeSim promotion pass.
        self.wireless.advance();
        // 14. STKM spawn respawn (ref 3911-3914).
        //     `if (!player.spwn && player.spawnID >= 0) create_part(-1, ...)` (and player2).
        //     Respawn keeps `spawn_id` so the SPWN respawns after death. If creation
        //     fails (blocked/full), leave `spwn == 0` and retry next tick.
        self.respawn_from_spawn_points();
    }

    fn respawn_from_spawn_points(&mut self) {
        use crate::elements_generated::pt;
        use crate::entity::Entity;
        // Ref Simulation.cpp:3911-3914 reads parts[player.spawnID].x/y directly.
        // We also require the slot to still be a live SPAWN/SPAWN2 to avoid
        // dereferencing a reused/dead slot as a different particle.
        if self.player.spwn == 0 && self.player.spawn_id >= 0 {
            let sid = self.player.spawn_id as usize;
            if sid < self.parts.len() && self.parts[sid].t == pt::SPAWN {
                let x = (self.parts[sid].x + 0.5) as i32;
                let y = (self.parts[sid].y + 0.5) as i32;
                let _ = self.respawn_entity_preserving_spawn(Entity::Player1, x, y);
                // On failure (blocked/full) leave spwn==0 and keep spawn_id to retry.
            }
        }
        if self.player2.spwn == 0 && self.player2.spawn_id >= 0 {
            let sid = self.player2.spawn_id as usize;
            if sid < self.parts.len() && self.parts[sid].t == pt::SPAWN2 {
                let x = (self.parts[sid].x + 0.5) as i32;
                let y = (self.parts[sid].y + 0.5) as i32;
                let _ = self.respawn_entity_preserving_spawn(Entity::Player2, x, y);
            }
        }
    }

    /// `AfterSim`.
    pub fn after_sim(&mut self) {
        // Reference Simulation.cpp:3924-3929: AfterSim dispatches
        // `Element_EMP_Trigger(sim, emp_trigger_count)` when EMP (ctype=PT_EMP)
        // SPRK particles triggered this frame, then resets the counter.
        //
        // AfterSim must NOT mutate `emp_decor`. SPRK's per-spark EMP branch in
        // updates.rs (SPRK.cpp:281-283) already adds +3 and caps at 40 for each
        // trigger; reference `Element_EMP_Trigger` (EMP.cpp) does not add decor.
        // Full EMP trigger is dispatched from updates.rs.
        if self.emp_trigger_count > 0 {
            crate::updates::emp_trigger(self, self.emp_trigger_count);
            self.emp_trigger_count = 0;
        }
        // Wireless promotion is handled in `BeforeSim` under the `ISWIRE` countdown
        // (ref Simulation.cpp:3900-3908). Keep `AfterSim` free of duplicate
        // promotion to preserve that timing.
    }

    pub fn step(&mut self) {
        self.before_sim();
        self.update_particles(0, usize::MAX);
        self.after_sim();
    }

    /// Rebuild `pmap`, then decrement/kill particles according to life flags.
    fn recalc_and_life(&mut self) {
        let mut i = 0usize;
        while i < self.parts.len() {
            let t = self.parts[i].t;
            if t != 0 {
                let def = element(t);
                if (def.properties & props::PROP_LIFE_DEC) != 0 && self.parts[i].life > 0 {
                    self.parts[i].life -= 1;
                    if (def.properties & props::PROP_LIFE_KILL_DEC) != 0 && self.parts[i].life <= 0 {
                        self.kill_part(i);
                    }
                } else if (def.properties & props::PROP_LIFE_KILL) != 0 && self.parts[i].life <= 0 {
                    self.kill_part(i);
                }
            }
            i += 1;
        }
        self.recalc();
    }

    /// `GetNeighbourhood`.
    pub fn neighbourhood(&self, i: usize) -> Neighbourhood {
        let p = &self.parts[i];
        let t = p.t;
        let x = (p.x + 0.5) as i32;
        let y = (p.y + 0.5) as i32;
        let mut n = Neighbourhood {
            surround: [0; 8],
            surround_space: 0,
            nt: 0,
            pgrav_x: 0.0,
            pgrav_y: 0.0,
        };
        let mut j = 0;
        for nx in -1i32..2 {
            for ny in -1i32..2 {
                if nx != 0 || ny != 0 {
                    let r = if x + nx >= 0
                        && y + ny >= 0
                        && (x + nx) < self.width as i32
                        && (y + ny) < self.height as i32
                    {
                        self.pmap[self.idx(x + nx, y + ny)]
                    } else {
                        0
                    };
                    n.surround[j] = r;
                    j += 1;
                    if pmap_type(r) == 0 {
                        n.surround_space += 1;
                    }
                    if pmap_type(r) != t {
                        n.nt += 1;
                    }
                }
            }
        }
        let def = element(t);
        if (def.properties & props::TYPE_SOLID) == 0 && (def.gravity != 0.0 || def.newtonian_gravity != 0.0) {
            let (gx, gy) = self.gravity_field(x, y, def.gravity, def.newtonian_gravity);
            n.pgrav_x = gx;
            n.pgrav_y = gy;
        }
        n
    }

    /// `UpdateParticles`.
    pub fn update_particles(&mut self, start: usize, end: usize) {
        let limit = end.min(self.parts.len());
        for i in start..limit {
            if i >= self.parts.len() {
                break;
            }
            let t = self.parts[i].t;
            if t == 0 {
                continue;
            }
            let mut x = (self.parts[i].x + 0.5) as i32;
            let mut y = (self.parts[i].y + 0.5) as i32;

            // Off-screen kill.
            let in_margin = x < CELL as i32
                || y < CELL as i32
                || x >= self.width as i32 - CELL as i32
                || y >= self.height as i32 - CELL as i32;
            if in_margin {
                if self.settings.edge_mode != crate::sim::EdgeMode::Loop {
                    self.kill_part(i);
                    continue;
                }
                // Loop mode: wrap particles in the outer CELL margin using the
                // reference `remainder_p`, keeping pmap in sync.
                let w = self.width as f32;
                let h = self.height as f32;
                let old_x = x;
                let old_y = y;
                if x < CELL as i32 || x >= self.width as i32 - CELL as i32 {
                    self.parts[i].x = crate::sim::remainder_p(
                        self.parts[i].x - CELL as f32 + 0.5,
                        w - CELL as f32 * 2.0,
                    ) + CELL as f32
                        - 0.5;
                }
                if y < CELL as i32 || y >= self.height as i32 - CELL as i32 {
                    self.parts[i].y = crate::sim::remainder_p(
                        self.parts[i].y - CELL as f32 + 0.5,
                        h - CELL as f32 * 2.0,
                    ) + CELL as f32
                        - 0.5;
                }
                x = (self.parts[i].x + 0.5) as i32;
                y = (self.parts[i].y + 0.5) as i32;
                if old_x != x || old_y != y {
                    if self.in_bounds(old_x, old_y) {
                        let old_cell = self.idx(old_x, old_y);
                        if self.pmap[old_cell] != 0
                            && crate::sim::pmap_id(self.pmap[old_cell]) as usize == i
                        {
                            self.pmap[old_cell] = 0;
                        }
                        if self.photons[old_cell] != 0
                            && crate::sim::pmap_id(self.photons[old_cell]) as usize == i
                        {
                            self.photons[old_cell] = 0;
                        }
                    }
                    if self.in_bounds(x, y) {
                        let cell = self.idx(x, y);
                        let tt = self.parts[i].t;
                        if (crate::element::element(tt).properties & crate::elements_generated::props::TYPE_ENERGY)
                            != 0
                        {
                            self.photons[cell] = crate::sim::pmap_pack(i as u32, tt);
                        } else if tt != 0 {
                            self.pmap[cell] = crate::sim::pmap_pack(i as u32, tt);
                        }
                    }
                }
            }
            if !self.in_bounds(x, y) {
                self.kill_part(i);
                continue;
            }
            let cx = x / CELL as i32;
            let cy = y / CELL as i32;
            let cell = self.cell_idx(cx, cy);
            let w = self.bmap[cell];

            // Kill particles inside walls they cannot occupy.
            if w != 0 && self.wall_kills(w, t) && t != crate::elements_generated::pt::STKM {
                self.kill_part(i);
                continue;
            }
            // Stasis freezes particles completely.
            if w == WALL_STASIS && self.emap[cell] < 8 {
                continue;
            }
            if w == WALL_DETECT && self.emap[cell] < 8 {
                self.set_emap(cx, cy);
            }

            // Air momentum injection.
            let def = element(t);
            let vc = self.cell_idx(cx, cy);
            self.vx[vc] = self.vx[vc] * def.air_loss + def.air_drag * self.parts[i].vx;
            self.vy[vc] = self.vy[vc] * def.air_loss + def.air_drag * self.parts[i].vy;

            if def.hot_air != 0.0 {
                if t == crate::elements_generated::pt::GAS || t == crate::elements_generated::pt::NBLE
                {
                    if self.pv[vc] < 3.5 {
                        self.pv[vc] += 4.0 * def.hot_air * (3.5 - self.pv[vc]);
                    }
                } else {
                    self.pv[vc] += 4.0 * def.hot_air;
                }
            }

            let n = self.neighbourhood(i);

            // Velocity updates.
            if t != crate::elements_generated::pt::SPNG {
                self.parts[i].vx *= def.loss;
                self.parts[i].vy *= def.loss;
            }
            self.parts[i].vx += def.advection * self.vx[vc] + n.pgrav_x;
            self.parts[i].vy += def.advection * self.vy[vc] + n.pgrav_y;
            if def.diffusion != 0.0 {
                self.parts[i].vx += def.diffusion * (2.0 * self.rng.uniform01() - 1.0);
                self.parts[i].vy += def.diffusion * (2.0 * self.rng.uniform01() - 1.0);
            }
            self.parts[i].vx = self.parts[i].vx.clamp(-1.0e4, 1.0e4);
            self.parts[i].vy = self.parts[i].vy.clamp(-1.0e4, 1.0e4);

            let mut transition_occurred = self.transition_phase(i, &n);
            if self.parts[i].t == 0 {
                continue;
            }
            let mut t = self.parts[i].t;
            if transition_occurred {
                t = self.parts[i].t;
            }

            if let Some(update) = crate::updates::update_fn(t) {
                if update(self, i, x, y, n.surround_space, n.nt) != 0 {
                    continue;
                }
            }

            if !self.settings.heat_sim {
                crate::updates::legacy_update(self, i, x, y, n.surround_space, n.nt);
            }

            if self.parts[i].t == crate::elements_generated::pt::NONE || self.parts[i].t == 0 {
                continue;
            }
            if transition_occurred {
                continue;
            }
            if self.parts[i].vx == 0.0 && self.parts[i].vy == 0.0 {
                continue;
            }
            if self.parts[i].flags & flags::SKIPMOVE != 0 {
                self.parts[i].flags &= !flags::SKIPMOVE;
                continue;
            }
            self.movement_phase(i);

            // Particles that left the world (edge loop or fast movement)
            // are handled inside `move_particle`.
            let _ = &mut transition_occurred;
        }
    }

    /// Simple wall-field kill test used by `wall_kills`.
    pub fn wall_kills_t(&self, w: u8, t: u16) -> bool {
        if w == 0 {
            return false;
        }
        let p = element(t).properties;
        w == WALL_WALL
            || w == WALL_WALLELEC
            || w == WALL_ALLOWAIR
            || w == WALL_DESTROYALL
            || (w == WALL_ALLOWLIQUID && (p & props::TYPE_LIQUID) == 0)
            || (w == WALL_ALLOWPOWDER && (p & props::TYPE_PART) == 0)
            || (w == WALL_ALLOWGAS && (p & props::TYPE_GAS) == 0)
            || (w == WALL_ALLOWENERGY && (p & props::TYPE_ENERGY) == 0)
            || (w == WALL_EWALL && self.emap[0] == 0)
    }

    #[inline]
    pub fn pmap_at(&self, x: i32, y: i32) -> u32 {
        if self.in_bounds(x, y) {
            self.pmap[self.idx(x, y)]
        } else {
            0
        }
    }

    #[inline]
    pub fn type_at(&self, x: i32, y: i32) -> u16 {
        pmap_type(self.pmap_at(x, y))
    }

    #[inline]
    pub fn pmap_type_at(&self, x: i32, y: i32) -> u16 {
        self.type_at(x, y)
    }

    #[inline]
    pub fn id_at(&self, x: i32, y: i32) -> Option<usize> {
        let r = self.pmap_at(x, y);
        if r == 0 {
            None
        } else {
            let id = pmap_id(r) as usize;
            if id < self.parts.len() {
                Some(id)
            } else {
                None
            }
        }
    }

    /// Clamp a temperature like `restrict_flt(..., MIN_TEMP, MAX_TEMP)`.
    #[inline]
    pub fn clamp_temp(&self, t: f32) -> f32 {
        restrict_flt(t, MIN_TEMP, MAX_TEMP)
    }
}

// Silence "unused import" for constants only used by element update modules.
#[allow(dead_code)]
const _UNUSED: (u8, u8) = (WALL_BLOCKAIR, WALL_EHOLE);
