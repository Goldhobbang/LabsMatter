//! Movement: `eval_move`, `try_move`, `move`, `do_move`, `PlanMove` and
//! `MovementPhase`, ported from the reference.

use crate::air::{
    WALL_ALLOWENERGY, WALL_DESTROYALL, WALL_DETECT, WALL_EHOLE, WALL_EWALL, WALL_FAN, WALL_STASIS,
    WALL_STREAM, WALL_WALLELEC,
};
use crate::element::{element, is_type};
use crate::elements_generated::pt;
use crate::elements_generated::props;
use crate::particle::flags;
use crate::portal::PortalPayload;
use std::cell::RefCell;
use crate::sim::{pmap_id, pmap_pack, pmap_type, Sim, CELL, ISTP, MAX_VELOCITY, PMAPMASK};

// Reference `portal_rx/portal_ry` (Simulation.h).
const PORTAL_RX: [i32; 8] = [-1, 0, 1, 1, 1, 0, -1, -1];
const PORTAL_RY: [i32; 8] = [-1, -1, -1, 0, 1, 1, 1, 0];

#[inline]
fn isign(v: i32) -> i32 {
    if v > 0 { 1 } else if v < 0 { -1 } else { 0 }
}

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
    pub(crate) fn eval_move(&self, t: u16, nx: i32, ny: i32) -> u8 {
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
            // ref: Simulation.cpp:1212 — `int rt = TYP(r);` resolved from eval_move's r.
            let rid: Option<usize> = if r != 0 {
                let id = pmap_id(r) as usize;
                if id < self.parts.len() && self.parts[id].t != 0 {
                    Some(id)
                } else {
                    None
                }
            } else {
                None
            };
            let rt = rid.map(|id| self.parts[id].t).unwrap_or(0);
            // ref: Simulation.cpp:1213-1219 — a hard hit shatters WOOD into SAWD.
            // This runs BEFORE the TYPE_ENERGY early-out (ref:1220-1221).
            if rt == pt::WOOD {
                if let Some(id) = rid {
                    let vel = (self.parts[i].vx * self.parts[i].vx
                        + self.parts[i].vy * self.parts[i].vy)
                        .sqrt();
                    if vel > 5.0 {
                        self.part_change_type(id, nx, ny, pt::SAWD as i16);
                    }
                }
            }
            // Blocked: energy particles may conduct heat or be captured (TPT try_move).
            if !is_type(t, props::TYPE_ENERGY) {
                return 0;
            }
            // PHOT heat conduction (reference: !legacy_enable && PHOT && r).
            if self.settings.heat_sim && t == pt::PHOT {
                if let Some(id) = rid {
                    if rt == pt::COAL || rt == pt::BCOL {
                        self.parts[id].temp = self.parts[i].temp;
                    }
                    if (rt as usize) < crate::elements_generated::PT_NUM
                        && rt != pt::FILT
                        && !self.is_heat_insulator(id)
                    {
                        let t1 = self.parts[i].temp;
                        let t2 = self.parts[id].temp;
                        let avg = ((t1 + t2) * 0.5).clamp(0.0, 9999.0);
                        self.parts[i].temp = avg;
                        self.parts[id].temp = avg;
                    }
                }
            } else if (t == pt::NEUT || t == pt::ELEC)
                && matches!(rt, x if x == pt::CLNE || x == pt::PCLN || x == pt::BCLN || x == pt::PBCN)
            {
                // Stamp clone ctype from the blocked energy particle if empty.
                if let Some(id) = rid {
                    if self.parts[id].ctype == 0 {
                        self.parts[id].ctype = t as i32;
                    }
                }
            }
            if rt == pt::PRTI {
                // TYPE_ENERGY capture into PRTI/portal (TPT try_move).
                if let Some(prti_id) = rid {
                    let mut count = 0usize;
                    for c in 0..8usize {
                        if isign(x - nx) == isign(PORTAL_RX[c]) && isign(y - ny) == isign(PORTAL_RY[c]) {
                            count = c;
                            break;
                        }
                    }
                    count %= 8;
                    let mut ch = ((self.parts[prti_id].temp - 73.15) / 100.0 + 1.0) as i32;
                    if ch >= crate::portal::PORTAL_CHANNELS as i32 {
                        ch = crate::portal::PORTAL_CHANNELS as i32 - 1;
                    } else if ch < 0 {
                        ch = 0;
                    }
                    self.parts[prti_id].tmp = ch;
                    let payload = PortalPayload::from_particle(&self.parts[i]);
                    if self.portal.store(ch as usize, count, payload) {
                        self.kill_part(i);
                    }
                }
            }
            return 0;
        }

        if e == 2 {
            // Same-space interactions (TPT try_move). Only clearly mapped, safe cases.
            let rid: Option<usize> = if r != 0 {
                let id = pmap_id(r) as usize;
                if id < self.parts.len() && self.parts[id].t != 0 {
                    Some(id)
                } else {
                    None
                }
            } else {
                None
            };
            let rt = rid.map(|id| self.parts[id].t).unwrap_or(0);
            if t == pt::PHOT {
                if let Some(id) = rid {
                    // ref: Simulation.cpp:1262-1354 — case PT_PHOT switch, in order.
                    // Note: no heat conduction here in the reference; PHOT heat
                    // conduction only runs in the blocked (!e) path (ref:1222-1229).
                    match rt {
                        x if x == pt::GLOW => {
                            // Ref: Simulation.cpp:1266-1272
                            if self.parts[id].life == 0 && self.rng.chance(1, 30) {
                                self.parts[id].life = 120;
                                self.create_gain_photon(i);
                            }
                        }
                        x if x == pt::FILT => {
                            // TODO(ref:Simulation.cpp:1273-1274): needs
                            // Element_FILT_interactWavelengths (no port exists yet).
                        }
                        x if x == pt::C5 => {
                            // Ref: Simulation.cpp:1276-1312
                            if self.parts[id].life > 0
                                && ((self.parts[id].ctype as u32
                                    & self.parts[i].ctype as u32
                                    & 0xFFFF_FFC0)
                                    != 0)
                            {
                                let vx = ((self.parts[id].tmp << 16) >> 16) as f32 / 255.0;
                                let vy = (self.parts[id].tmp >> 16) as f32 / 255.0;
                                let pci = &self.parts[i];
                                let vn = pci.vx * pci.vx + pci.vy * pci.vy;
                                if pci.vx + vx == 0.0 && pci.vy + vy == 0.0 {
                                    // Resulting velocity would be 0: shoot the photon
                                    // off at a 90 degree angle instead.
                                    self.parts[i].vx = vy;
                                    self.parts[i].vy = -vx;
                                } else {
                                    self.parts[i].ctype =
                                        ((self.parts[id].ctype & self.parts[i].ctype) >> 6) as i32;
                                    // Add momentum of photons to each other.
                                    self.parts[i].vx += vx;
                                    self.parts[i].vy += vy;
                                    // Normalize velocity to original value.
                                    let pci = &self.parts[i];
                                    let vn = vn / (pci.vx * pci.vx + pci.vy * pci.vy);
                                    let vn = vn.sqrt();
                                    self.parts[i].vx *= vn;
                                    self.parts[i].vy *= vn;
                                }
                                self.parts[id].life = 0;
                                self.parts[id].ctype = 0;
                            } else if self.parts[id].ctype == 0
                                && (self.parts[i].ctype as u32 & 0xFFFF_FFC0) != 0
                            {
                                self.parts[id].life = 1;
                                self.parts[id].ctype = self.parts[i].ctype;
                                self.parts[id].tmp = (0xFFFF
                                    & ((self.parts[i].vx * 255.0) as i32))
                                    | (0xFFFF_0000u32 as i32
                                        & ((self.parts[i].vy * 16711680.0) as i32));
                                self.parts[id].tmp2 = (0xFFFF
                                    & (((self.parts[i].x - x as f32) * 255.0) as i32))
                                    | (0xFFFF_0000u32 as i32
                                        & (((self.parts[i].y - y as f32) * 16711680.0) as i32));
                                self.kill_part(i);
                            }
                        }
                        x if x == pt::INVIS => {
                            // Ref: Simulation.cpp:1313-1325
                            let cx = nx / CELL as i32;
                            let cy = ny / CELL as i32;
                            let pv = if self.in_cell_bounds(cx, cy) {
                                self.pv[self.cell_idx(cx, cy)]
                            } else {
                                0.0
                            };
                            let pr = if self.parts[id].tmp > 0 {
                                self.parts[id].tmp as f32
                            } else {
                                4.0
                            };
                            if pv >= -pr && pv <= pr {
                                //@ PHOT + INVIS -> NEUT + INVIS
                                self.part_change_type(i, x as i32, y as i32, pt::NEUT as i16);
                                self.parts[i].ctype = 0;
                            }
                        }
                        x if x == pt::BIZR || x == pt::BIZRG || x == pt::BIZRS => {
                            //@ ref: Simulation.cpp:1326-1332 — PHOT + BIZR/BIZRG/BIZRS -> ELEC + BIZR/...
                            self.part_change_type(i, x as i32, y as i32, pt::ELEC as i16);
                            self.parts[i].ctype = 0;
                        }
                        x if x == pt::H2 => {
                            // Ref: Simulation.cpp:1333-1344
                            if (self.parts[i].tmp & 0x1) == 0 {
                                //@ PHOT + H2 -> PROT + ELEC
                                self.part_change_type(i, x as i32, y as i32, pt::PROT as i16);
                                self.parts[i].ctype = 0;
                                self.parts[i].tmp2 = 0x1;
                                // TODO(ref:1341): needs a create_part(p >= 0) reuse form --
                                // the reference reuses ID(r)'s slot so the H2 becomes ELEC in
                                // place; this call allocates a new particle and leaves the H2
                                // alive, so one extra live particle results.
                                self.create_part(x as i32, y as i32, pt::ELEC);
                                return 1;
                            }
                        }
                        x if x == pt::GPMP => {
                            // Ref: Simulation.cpp:1345-1352
                            if self.parts[id].life == 0 {
                                //@ PHOT + GPMP -> GRVT + GPMP
                                self.part_change_type(i, x as i32, y as i32, pt::GRVT as i16);
                                self.parts[i].tmp = (self.parts[id].temp - 273.15) as i32;
                            }
                        }
                        _ => {}
                    }
                }
            } else if t == pt::NEUT {
                //@ ref: Simulation.cpp:1356-1361 — NEUT + GLAS/BGLA -> NEUT + GLAS/BGLA + PHOT
                let _ = rid;
                if rt == pt::GLAS || rt == pt::BGLA {
                    if self.rng.chance(1, 10) {
                        self.create_cherenkov_photon(i);
                    }
                }
            } else if t == pt::ELEC {
                //@ ref: Simulation.cpp:1362-1369 — ELEC + GLOW -> PHOT + GLOW
                if rt == pt::GLOW {
                    self.part_change_type(i, x, y, pt::PHOT as i16);
                    self.parts[i].ctype = 0x3FFF_FFFF;
                }
            } else if t == pt::PROT {
                //@ ref: Simulation.cpp:1370-1374 — PROT + INVIS -> NEUT + INVIS
                if rt == pt::INVIS {
                    self.part_change_type(i, x, y, pt::NEUT as i16);
                }
            } else if t == pt::BIZR || t == pt::BIZRG {
                //@ ref: Simulation.cpp:1375-1379 — BIZR/BIZRG + FILT
                if rt == pt::FILT {
                    // TODO(ref:Simulation.cpp:1375-1379): needs
                    // Element_FILT_interactWavelengths (no port exists yet).
                }
            }
            return 1;
        }

        // --- swap block ---------------------------------------------------
        // ref: Simulation.cpp:1385-1434 — target-type switch.
        match rt {
            x if x == pt::VOID || x == pt::PVOD => {
                // ref: Simulation.cpp:1387-1392 — void/portal void eats the mover;
                // void ctype was already checked in eval_move.
                self.kill_part(i);
                return 0;
            }
            x if x == pt::BHOL || x == pt::NBHL => {
                // ref: Simulation.cpp:1393-1401 — blackhole eats the mover.
                if self.settings.heat_sim
                    && r != 0
                    && (pmap_id(r) as usize) < self.parts.len()
                {
                    let id = pmap_id(r) as usize;
                    self.parts[id].temp = crate::sim::restrict_flt(
                        self.parts[id].temp + self.parts[i].temp / 2.0,
                        crate::sim::MIN_TEMP,
                        crate::sim::MAX_TEMP,
                    );
                }
                self.kill_part(i);
                return 0;
            }
            x if x == pt::WHOL || x == pt::NWHL => {
                // ref: Simulation.cpp:1402-1414 — whitehole eats ANAR.
                if t == pt::ANAR {
                    if self.settings.heat_sim
                        && r != 0
                        && (pmap_id(r) as usize) < self.parts.len()
                    {
                        let id = pmap_id(r) as usize;
                        self.parts[id].temp = crate::sim::restrict_flt(
                            self.parts[id].temp - (crate::sim::MAX_TEMP - self.parts[i].temp) / 2.0,
                            crate::sim::MIN_TEMP,
                            crate::sim::MAX_TEMP,
                        );
                    }
                    self.kill_part(i);
                    return 0;
                }
            }
            x if x == pt::DEUT => {
                // ref: Simulation.cpp:1415-1424 — ELEC feeding a DEUT grows it.
                if t == pt::ELEC {
                    if r != 0 && (pmap_id(r) as usize) < self.parts.len() {
                        let id = pmap_id(r) as usize;
                        if self.parts[id].life < 6000 {
                            self.parts[id].life += 1;
                        }
                        self.parts[id].temp = 0.0;
                    }
                    self.kill_part(i);
                    return 0;
                }
            }
            x if x == pt::VIBR || x == pt::BVBR => {
                // ref: Simulation.cpp:1425-1433 — energy particles excite VIBR.
                if is_type(t, props::TYPE_ENERGY) {
                    if r != 0 && (pmap_id(r) as usize) < self.parts.len() {
                        let id = pmap_id(r) as usize;
                        self.parts[id].tmp += 20;
                    }
                    self.kill_part(i);
                    return 0;
                }
            }
            _ => {}
        }
        // ref: Simulation.cpp:1436-1464 — mover-type switch.
        if t == pt::NEUT && (element(rt).properties & props::PROP_NEUTABSORB) != 0 {
            self.kill_part(i);
            return 0;
        }
        if t == pt::CNCT {
            // ref: Simulation.cpp:1445-1459 — CNCT cannot sink into another CNCT or
            // ROCK when gravity points that way (prevents stacking); the offset is
            // taken from the gravity field at the mover's current cell.
            let (g_x, g_y) = self.gravity_field(
                x,
                y,
                element(pt::CNCT).gravity,
                element(pt::CNCT).gravity,
            );
            let mut offset_x = 0i32;
            let mut offset_y = 0i32;
            if g_x > 0.0 {
                offset_x = 1;
            } else if g_x < 0.0 {
                offset_x = -1;
            }
            if g_y > 0.0 {
                offset_y = 1;
            } else if g_y < 0.0 {
                offset_y = -1;
            }
            let different_position = (offset_x != 0) != (offset_y != 0);
            let destination_below = (nx - x) * offset_x > 0 || (ny - y) * offset_y > 0;
            if different_position && destination_below {
                let bx = x + offset_x;
                let by = y + offset_y;
                if self.in_bounds(bx, by) {
                    let bt = pmap_type(self.pmap[self.idx(bx, by)]);
                    if bt == pt::CNCT || bt == pt::ROCK {
                        return 0;
                    }
                }
            }
        }
        if t == pt::GBMB && self.parts[i].life > 0 {
            // ref: Simulation.cpp:1460-1463
            return 0;
        }
        // ref: Simulation.cpp:1466-1467 — an unpowered void hole cell only lets
        // things out into another (unpowered) void hole cell.
        let src_ehole =
            self.wall_at(x, y) == WALL_EHOLE && self.emap_at(x, y) == 0;
        let dst_ehole = self.wall_at(nx, ny) == WALL_EHOLE && self.emap_at(nx, ny) == 0;
        if src_ehole && !dst_ehole {
            return 0;
        }
        let ri = pmap_id(r) as usize;
        if r != 0 && ri < self.parts.len() {
            if self.parts[i].t == pt::NEUT {
                // ref: Simulation.cpp:1470-1492 — the mover is a neutron:
                // NEUTPENETRATE material under the neutron gets dragged around.
                if !self.in_bounds(x, y) {
                    return 1;
                }
                let s = self.pmap[self.idx(x, y)];
                let sid = if s != 0 { pmap_id(s) as usize } else { usize::MAX };
                if sid != usize::MAX
                    && sid < self.parts.len()
                    && (element(self.parts[sid].t).properties & props::PROP_NEUTPENETRATE) == 0
                {
                    // Not a NEUTPENETRATE particle underneath: move only the
                    // neutron itself.
                    return 1;
                }
                if self.wall_at(x, y) == WALL_ALLOWENERGY {
                    // ref: Simulation.cpp:1478-1479 — do not drag the target
                    // particle into an energy only wall.
                    return 1;
                }
                if sid != usize::MAX && sid < self.parts.len() {
                    // ref: Simulation.cpp:1482-1485
                    self.parts[sid].x = nx as f32;
                    self.parts[sid].y = ny as f32;
                }
                if self.in_bounds(nx, ny) {
                    let dcell = self.idx(nx, ny);
                    if sid != usize::MAX && sid < self.parts.len() {
                        self.pmap[dcell] = (s & !PMAPMASK) | self.parts[sid].t as u32;
                    } else {
                        self.pmap[dcell] = 0;
                    }
                }
                self.parts[ri].x = x as f32;
                self.parts[ri].y = y as f32;
                let scell = self.idx(x, y);
                self.pmap[scell] = pmap_pack(ri as u32, self.parts[ri].t);
                return 1;
            }
            // Swap: the target moves to the mover's previous position
            // (ref: Simulation.cpp:1494-1500).
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
        if result != 0 && !self.move_particle(i, x, y, nxf, nyf) {
            return -1;
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

        // STKM/STKM2/FIGH head movement (ref Simulation.cpp:2893-2955), runs after
        // plan_move and FLAG_STAGNANT read/clear, before TYPE_ENERGY dispatch.
        if t == pt::STKM || t == pt::STKM2 || t == pt::FIGH {
            // Head passes through anything; uses plan.vx/vy already written above.
            self.parts[i].x += self.parts[i].vx;
            self.parts[i].y += self.parts[i].vy;
            let mut nx = (self.parts[i].x + 0.5) as i32;
            let mut ny = (self.parts[i].y + 0.5) as i32;
            if self.settings.edge_mode == crate::sim::EdgeMode::Loop {
                let x_ok = nx >= CELL as i32 && nx < self.width as i32 - CELL as i32;
                let y_ok = ny >= CELL as i32 && ny < self.height as i32 - CELL as i32;
                let oldnx = nx;
                let oldny = ny;
                if !x_ok {
                    self.parts[i].x = remainder_p(
                        self.parts[i].x - CELL as f32 + 0.5,
                        self.width as f32 - CELL as f32 * 2.0,
                    ) + CELL as f32
                        - 0.5;
                    nx = (self.parts[i].x + 0.5) as i32;
                }
                if !y_ok {
                    self.parts[i].y = remainder_p(
                        self.parts[i].y - CELL as f32 + 0.5,
                        self.height as f32 - CELL as f32 * 2.0,
                    ) + CELL as f32
                        - 0.5;
                    ny = (self.parts[i].y + 0.5) as i32;
                }

                if !x_ok || !y_ok {
                    // Adjust stickman legs/accs and damp head velocity.
                    let stickman = match t {
                        x if x == pt::STKM => Some(&mut self.player),
                        x if x == pt::STKM2 => Some(&mut self.player2),
                        x if x == pt::FIGH => {
                            let slot = self.parts[i].tmp;
                            if slot >= 0 && (slot as usize) < self.fighters.len() {
                                self.fighters.get_mut(slot as usize)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };
                    if let Some(stickman) = stickman {
                        let mut j = 0usize;
                        while j < 16 {
                            stickman.legs[j] += (nx - oldnx) as f32;
                            stickman.legs[j + 1] += (ny - oldny) as f32;
                            stickman.accs[j / 2] *= 0.95;
                            j += 2;
                        }
                    }
                    self.parts[i].vy *= 0.95;
                    self.parts[i].vx *= 0.95;
                }
            }
            if ny != y || nx != x {
                if self.in_bounds(x, y) {
                    let cell = self.idx(x, y);
                    if self.pmap[cell] != 0 && pmap_id(self.pmap[cell]) as usize == i {
                        self.pmap[cell] = 0;
                    } else if self.photons[cell] != 0 && pmap_id(self.photons[cell]) as usize == i {
                        self.photons[cell] = 0;
                    }
                }
                if nx < CELL as i32
                    || nx >= self.width as i32 - CELL as i32
                    || ny < CELL as i32
                    || ny >= self.height as i32 - CELL as i32
                {
                    self.kill_part(i);
                    return;
                }
                if !self.in_bounds(nx, ny) {
                    self.kill_part(i);
                    return;
                }
                let cell = self.idx(nx, ny);
                // Pre-existing deviation retained: STKM/FIGH are written into pmap here
                // even though the reference create_part path does not insert them.
                if (element(t).properties & props::TYPE_ENERGY) != 0 {
                    self.photons[cell] = pmap_pack(i as u32, t);
                } else if t != 0 {
                    self.pmap[cell] = pmap_pack(i as u32, t);
                }
            }
            return;
        }

        if is_type(t, props::TYPE_ENERGY) {
            self.move_energy(i, x, y, &mut plan, stagnant);
            return;
        }
        if element(t).falldown == 0 {
            // Gases / solids (but not powders): bounce off obstacles.
            // ref: Simulation.cpp:3102-3127
            if self.do_move(i, x, y, plan.fin_xf, plan.fin_yf) == 0 {
                // ref: Simulation.cpp:3107-3108
                if self.parts[i].t == 0 {
                    return;
                }
                // Clamp the bounce so it never attempts more than one pixel per axis.
                // ref: Simulation.cpp:3110-3113 (ISTP is `CELL/2`, an int).
                let istp = ISTP as i32;
                let mut fin_x = plan.fin_x;
                let mut fin_y = plan.fin_y;
                if fin_x > x + istp {
                    fin_x = x + istp;
                }
                if fin_x < x - istp {
                    fin_x = x - istp;
                }
                if fin_y > y + istp {
                    fin_y = y + istp;
                }
                if fin_y < y - istp {
                    fin_y = y - istp;
                }
                let collision = element(t).collision;
                // ref: Simulation.cpp:3114-3124 — try the mirrored destinations and
                // damp only the velocity component that was moving into the wall.
                if self.do_move(i, x, y, (2 * x - fin_x) as f32, fin_y as f32) != 0 {
                    self.parts[i].vx *= collision;
                } else if self.do_move(i, x, y, fin_x as f32, (2 * y - fin_y) as f32) != 0 {
                    self.parts[i].vy *= collision;
                } else {
                    // ref: Simulation.cpp:3122-3126 — neither mirror destination is
                    // free, so just damp both components; the reference sets no
                    // flags here.
                    self.parts[i].vx *= collision;
                    self.parts[i].vy *= collision;
                }
            }
            return;
        }

        // Powder / liquid.
        // ref: Simulation.cpp:3129-3141 — water equalization first, then movement.
        if self.settings.water_equal_test
            && element(t).falldown == 2
            && self.rng.chance(1, 200)
        {
            if self.flood_water(i, x, y) {
                return;
            }
        }
        if self.do_move(i, x, y, plan.fin_xf, plan.fin_yf) == 0 {
            // ref: Simulation.cpp:3140-3141
            if self.parts[i].t == 0 {
                return;
            }
            let collision = element(t).collision;
            let moved = if plan.fin_x != x && self.do_move(i, x, y, plan.fin_xf, plan.clear_yf) != 0
            {
                // ref: Simulation.cpp:3144-3145
                self.parts[i].vx *= collision;
                self.parts[i].vy *= collision;
                true
            } else if plan.fin_y != y && self.do_move(i, x, y, plan.clear_xf, plan.fin_yf) != 0 {
                // ref: Simulation.cpp:3149-3150
                self.parts[i].vx *= collision;
                self.parts[i].vy *= collision;
                true
            } else {
                false
            };
            if !moved {
                // ref: Simulation.cpp:3154-3156 — the gravity field and the local
                // neighbour state come from GetNeighbourhood. The reference reuses
                // the neighbourhood computed in UpdateParticles (ref:2360); here it
                // is recomputed from current state (identical unless an update
                // function changed nearby particles this frame).
                let neighbourhood = self.neighbourhood(i);
                let r = self.rng.between(0, 1) * 2 - 1;
                let vx = self.parts[i].vx;
                let vy = self.parts[i].vy;
                if (plan.clear_x != x
                    || plan.clear_y != y
                    || neighbourhood.nt != 0
                    || neighbourhood.surround_space != 0)
                    && (vx.abs() > 0.01 || vy.abs() > 0.01)
                {
                    // Allow diagonal movement if target position is blocked, but no
                    // point trying this if the particle is stuck in a block of
                    // identical particles.
                    // ref: Simulation.cpp:3162-3167
                    let mut dx = vx - vy * r as f32;
                    let mut dy = vy + vx * r as f32;
                    let mv = dx.abs().max(dy.abs());
                    dx /= mv;
                    dy /= mv;
                    if self.do_move(i, x, y, plan.clear_xf + dx, plan.clear_yf + dy) != 0 {
                        self.collide(i);
                        return;
                    }
                    // ref: Simulation.cpp:3174-3178
                    let sw = dx;
                    dx = dy * r as f32;
                    dy = -sw * r as f32;
                    if self.do_move(i, x, y, plan.clear_xf + dx, plan.clear_yf + dy) != 0 {
                        self.collide(i);
                        return;
                    }
                }
                // Liquid lateral spreading.
                // ref: Simulation.cpp:3186-3235 (vertical) / 3236-3332 (gravity
                // field) — see `liquid_spread` for the deviations.
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

    /// Port of `Simulation::create_gain_photon` (Simulation.cpp:2015-2050)
    fn create_gain_photon(&mut self, pp: usize) {
        // MaxParts guard
        if self.parts.len() >= self.width * self.height {
            return;
        }
        let lr = 2.0 * self.rng.between(0, 1) as f32 - 1.0;
        let xx = self.parts[pp].x - lr * 0.3 * self.parts[pp].vy;
        let yy = self.parts[pp].y + lr * 0.3 * self.parts[pp].vx;
        let nx = (xx + 0.5) as i32;
        let ny = (yy + 0.5) as i32;
        if nx < 0 || ny < 0 || nx >= self.width as i32 || ny >= self.height as i32 {
            return;
        }
        let g = self.pmap[self.idx(nx, ny)];
        if g == 0 {
            return;
        }
        let gid = pmap_id(g) as usize;
        if gid >= self.parts.len() {
            return;
        }
        if self.parts[gid].t != pt::GLOW {
            return;
        }
        let old_temp = self.parts[gid].temp;
        let i = match self.create_part(nx, ny, pt::PHOT) {
            Some(i) => i,
            None => return,
        };
        self.parts[i].x = xx;
        self.parts[i].y = yy;
        self.parts[i].vx = self.parts[pp].vx;
        self.parts[i].vy = self.parts[pp].vy;
        self.parts[i].temp = old_temp;
        let mut temp_bin = ((self.parts[i].temp - 273.0) * 0.25) as i32;
        if temp_bin < 0 {
            temp_bin = 0;
        }
        if temp_bin > 25 {
            temp_bin = 25;
        }
        self.parts[i].ctype = 0x1F << temp_bin;
    }

    /// Port of `Simulation::create_cherenkov_photon` (Simulation.cpp:2052-2086)
    fn create_cherenkov_photon(&mut self, pp: usize) {
        if self.parts.len() >= self.width * self.height {
            return;
        }
        let nx = (self.parts[pp].x + 0.5) as i32;
        let ny = (self.parts[pp].y + 0.5) as i32;
        if nx < 0 || ny < 0 || nx >= self.width as i32 || ny >= self.height as i32 {
            return;
        }
        let g = self.pmap[self.idx(nx, ny)];
        if g == 0 {
            return;
        }
        let gid = pmap_id(g) as usize;
        if gid >= self.parts.len() {
            return;
        }
        let gt = self.parts[gid].t;
        if gt != pt::GLAS && gt != pt::BGLA {
            return;
        }
        let hv = (self.parts[pp].vx * self.parts[pp].vx + self.parts[pp].vy * self.parts[pp].vy).sqrt();
        if hv < 1.44 {
            return;
        }
        let old_temp = self.parts[gid].temp;
        let i = match self.create_part(nx, ny, pt::PHOT) {
            Some(i) => i,
            None => return,
        };
        let lr = 2.0 * self.rng.between(0, 1) as f32 - 1.0;
        self.parts[i].ctype = 0x00000F80;
        self.parts[i].x = self.parts[pp].x;
        self.parts[i].y = self.parts[pp].y;
        self.parts[i].temp = old_temp;
        self.parts[i].vx = self.parts[pp].vx - lr * 2.5 * self.parts[pp].vy;
        self.parts[i].vy = self.parts[pp].vy + lr * 2.5 * self.parts[pp].vx;
        let r = 1.269 / ((self.parts[i].vx * self.parts[i].vx + self.parts[i].vy * self.parts[i].vy).sqrt());
        self.parts[i].vx *= r;
        self.parts[i].vy *= r;
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
    ///
    /// Reference: `Simulation.cpp:2955-3090` (PHOT pre-step, movement/stagnant,
    /// and reflection). `stagnant` is computed once in `movement_phase` (flag read
    /// then cleared) and passed in, matching the reference `MovementPhase`.
    /// Life decay is NOT done here: `recalc_and_life` (tick.rs) applies
    /// `PROP_LIFE_DEC` once per frame for PHOT, NEUT, ELEC (verified in
    /// `elements_generated.rs`: PHOT 0x00014050, NEUT 0x00014010, ELEC 0x00014010
    /// all include `PROP_LIFE_DEC` 0x10), avoiding a double-decrement.
    fn move_energy(&mut self, i: usize, x: i32, y: i32, plan: &mut PlanResult, stagnant: bool) {
        // Reference Simulation.cpp:3014 (PHOT branch) — SKIPMOVE short-circuit.
        if self.parts[i].flags & flags::SKIPMOVE != 0 {
            self.parts[i].flags &= !flags::SKIPMOVE;
            return;
        }
        let t = self.parts[i].t;
        // ref: PLNT.h:13 (PLNT_COLOUR = 6); Simulation.cpp:3074-3082
        const PLNT_COLOUR: i32 = 6;

        // PHOT pre-step: glass refraction / total internal reflection (Simulation.cpp:2955-3013).
        if t == pt::PHOT {
            if self.eval_move(pt::PHOT, plan.fin_x, plan.fin_y) != 0 {
                let rt = self.pmap_type_at(plan.fin_x, plan.fin_y);
                let lt = self.pmap_type_at(x, y);
                let rt_glas = rt == pt::GLAS || rt == pt::BGLA;
                let lt_glas = lt == pt::GLAS || lt == pt::BGLA;
                if rt_glas != lt_glas {
                    let gn = self.get_normal_interp_phot(
                        crate::normal::REFRACT | t as i32,
                        self.parts[i].x,
                        self.parts[i].y,
                        self.parts[i].vx,
                        self.parts[i].vy,
                    );
                    if !gn.success {
                        self.kill_part(i);
                        return;
                    }
                    let mut nrx = gn.nx;
                    let mut nry = gn.ny;
                    let r = self.get_wavelength_bin(i);
                    if r == -1 || (self.parts[i].ctype as u32 & 0x3FFF_FFFF) == 0 {
                        self.kill_part(i);
                        return;
                    }
                    let mut nn =
                        crate::normal::GLASS_IOR - crate::normal::GLASS_DISP * (r - 30) as f32 / 30.0;
                    nn *= nn;
                    let enter = rt_glas && !lt_glas;
                    if enter {
                        nrx = -nrx;
                        nry = -nry;
                        nn = 1.0 / nn;
                    }
                    let ct1 = self.parts[i].vx * nrx + self.parts[i].vy * nry;
                    let ct2 = 1.0 - (nn * nn) * (1.0 - ct1 * ct1);
                    if ct2 < 0.0 {
                        // Total internal reflection (Simulation.cpp:2995-3004).
                        self.parts[i].vx -= 2.0 * ct1 * nrx;
                        self.parts[i].vy -= 2.0 * ct1 * nry;
                        plan.fin_xf = self.parts[i].x;
                        plan.fin_yf = self.parts[i].y;
                        plan.fin_x = x;
                        plan.fin_y = y;
                    } else {
                        let ct2s = ct2.sqrt() - nn * ct1;
                        self.parts[i].vx = nn * self.parts[i].vx + ct2s * nrx;
                        self.parts[i].vy = nn * self.parts[i].vy + ct2s * nry;
                    }
                }
            }
        }

        // Movement / stagnant (Simulation.cpp:3014-3023).
        let blocked = if stagnant {
            self.do_move(i, x, y, plan.fin_x as f32, plan.fin_y as f32) == 0
        } else {
            self.do_move(i, x, y, plan.fin_xf, plan.fin_yf) == 0
        };
        if stagnant && blocked {
            if self.parts[i].t != 0 {
                self.kill_part(i);
            }
            return;
        }

        if blocked {
            // Reflection block (Simulation.cpp:3024-3090), entered only when the
            // non-stagnant do_move fails.
            if self.parts[i].t == 0 {
                return;
            }
            self.parts[i].flags |= flags::STAGNANT;
            if t == pt::NEUT && self.rng.chance(1, 10) {
                self.kill_part(i);
                return;
            }
            let r = self.pmap_at(plan.fin_x, plan.fin_y);
            let rt = pmap_type(r);
            let rid = if r == 0 { 0usize } else { pmap_id(r) as usize };
            if (rt == pt::PIPE || rt == pt::PPIP)
                && rid < self.parts.len()
                && self.parts[rid].t == rt
                && self.parts[rid].ctype == 0
            {
                // Element_PIPE_transfer_part_to_pipe (Simulation.cpp:3057-3066).
                crate::updates::part_to_pipe(self, i, rid);
                return;
            }
            if t == pt::PHOT {
                let mut mask = element(rt).photon_reflect;
                if rt == pt::LITH {
                    if rid < self.parts.len() && self.parts[rid].t == rt {
                        let wl_bin = (self.parts[rid].ctype / 4).clamp(0, 25);
                        mask = 0x1F_u32 << (wl_bin as u32);
                    }
                } else if rt == pt::SEED {
                    if rid < self.parts.len() && self.parts[rid].t == rt {
                        let colour = (self.parts[rid].ctype >> PLNT_COLOUR) & 0x3f;
                        if colour & 0b110000 == 0 {
                            mask |= 1 << 25;
                        }
                        if colour & 0b001100 == 0 {
                            mask |= 1 << 15;
                        }
                        if colour & 0b000011 == 0 {
                            mask |= 1 << 5;
                        }
                    }
                }
                self.parts[i].ctype &= mask as i32;
            }
            // Reflection normal (Simulation.cpp:3084-3089): uses `t`, not `REFRACT|t`.
            let gn = self.get_normal_interp_phot(
                t as i32,
                self.parts[i].x,
                self.parts[i].y,
                self.parts[i].vx,
                self.parts[i].vy,
            );
            if gn.success {
                let (nrx, nry) = (gn.nx, gn.ny);
                if rt == pt::CRMC {
                    let mut rr = self.rng.between(-50, 50) as f32 * 0.01;
                    rr = rr * rr * rr;
                    let (rx, ry) = (rr.cos(), rr.sin());
                    let anrx = rx * nrx + ry * nry;
                    let anry = rx * nry - ry * nrx;
                    let dp = anrx * self.parts[i].vx + anry * self.parts[i].vy;
                    self.parts[i].vx -= 2.0 * dp * anrx;
                    self.parts[i].vy -= 2.0 * dp * anry;
                } else {
                    let dp = nrx * self.parts[i].vx + nry * self.parts[i].vy;
                    self.parts[i].vx -= 2.0 * dp * nrx;
                    self.parts[i].vy -= 2.0 * dp * nry;
                }
                // Leave actual movement to next frame (STAGNANT already set).
            } else {
                if t != pt::NEUT {
                    self.kill_part(i);
                }
                return;
            }
            if t == pt::PHOT && (self.parts[i].ctype as u32 & 0x3FFF_FFFF) == 0 {
                self.kill_part(i);
                return;
            }
        }
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
