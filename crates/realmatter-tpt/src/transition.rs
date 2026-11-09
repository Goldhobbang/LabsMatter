//! `TransitionPhase`: heat conduction, phase changes and pressure transitions.

use crate::element::{element, is_element};
use crate::particle::flags;
use crate::elements_generated::pt;
use crate::elements_generated::props;
use crate::sim::{pmap_id, pmap_type, restrict_flt, Sim, CELL, MAX_TEMP, MIN_TEMP, NT, ST};
use crate::tick::Neighbourhood;

impl Sim {
    /// `SimulationData::IsHeatInsulator`.
    pub fn is_heat_insulator(&self, i: usize) -> bool {
        let p = &self.parts[i];
        let def = element(p.t);
        def.heat_conduct == 0
            || (p.t == pt::HSWC && p.life != 10)
            || ((p.t == pt::PIPE || p.t == pt::PPIP) && (p.tmp & 0x8) == 0)
    }

    /// `SimulationData::HeatCapacityOf`.
    pub fn heat_capacity_of(&self, i: usize) -> f32 {
        let p = &self.parts[i];
        let def = element(p.t);
        if (p.t == pt::PIPE || p.t == pt::PPIP) && is_element(p.ctype) {
            def.heat_capacity + element(p.ctype as u16).heat_capacity
        } else {
            def.heat_capacity
        }
    }

    /// `TransitionPhase`. Returns true when the particle changed type.
    pub fn transition_phase(&mut self, i: usize, n: &Neighbourhood) -> bool {
        let mut t = self.parts[i].t;
        let mut x = (self.parts[i].x + 0.5) as i32;
        let mut y = (self.parts[i].y + 0.5) as i32;
        let mut transition_occurred = false;

        if self.settings.heat_sim {
            let mut gel_scale = 1.0f32;
            if t == pt::GEL {
                gel_scale = self.parts[i].tmp as f32 * 2.55;
            }

            // Liquid convection: swap temperature with the cell gravity pulls toward.
            if (element(t).properties & props::TYPE_LIQUID) != 0
                && (t != pt::GEL || gel_scale > (1 + self.rng.between(0, 254)) as f32)
            {
                let (gx, gy) = self.gravity_field(x, y, -2.0, -2.0);
                let off_x = ((gx + x as f32).round() as i32).clamp(x - 1, x + 1);
                let off_y = ((gy + y as f32).round() as i32).clamp(y - 1, y + 1);
                if off_x != x || off_y != y {
                    if let Some(rid) = self.id_at(off_x, off_y) {
                        if self.parts[rid].t == t && self.parts[i].temp > self.parts[rid].temp {
                            let tmp = self.parts[i].temp;
                            self.parts[i].temp = self.parts[rid].temp;
                            self.parts[rid].temp = tmp;
                        }
                    }
                }
            }

            let conduct = element(t).heat_conduct as f32 * gel_scale;
            if t != 0 && !self.is_heat_insulator(i) && self.rng.chance(conduct as i32, 250) {
                let vc = self.cell_idx(x / CELL as i32, y / CELL as i32);

                // Heat exchange with ambient air.
                if self.settings.ambient_heat && (element(t).properties & props::PROP_NOAMBHEAT) == 0 {
                    let dtemp = self.hv[vc] - self.parts[i].temp;
                    let hc = self.heat_capacity_of(i);
                    let alpha = (0.4 * hc).min(0.04);
                    self.parts[i].temp = restrict_flt(self.parts[i].temp + alpha * dtemp / hc, MIN_TEMP, MAX_TEMP);
                    self.hv[vc] = restrict_flt(self.hv[vc] - alpha * dtemp, MIN_TEMP, MAX_TEMP);
                }

                // Conduction with the 8 neighbours: all involved particles are
                // set to the capacity-weighted equilibrium temperature.
                let mut surround_hconduct = [i; 8];
                let mut hc_total = 0.0f32;
                let mut c_heat = 0.0f32;
                for (j, r) in n.surround.iter().enumerate() {
                    if *r == 0 {
                        continue;
                    }
                    let rid = pmap_id(*r) as usize;
                    if rid >= self.parts.len() || rid == i {
                        continue;
                    }
                    let rt = pmap_type(*r);
                    if !self.can_conduct(t, rt) {
                        continue;
                    }
                    surround_hconduct[j] = rid;
                    let hc = self.heat_capacity_of(rid);
                    c_heat += self.parts[rid].temp * hc;
                    hc_total += hc;
                }
                let hc = self.heat_capacity_of(i);
                c_heat += self.parts[i].temp * hc;
                hc_total += hc;
                let pt = restrict_flt(c_heat / hc_total, MIN_TEMP, MAX_TEMP);
                self.parts[i].temp = pt;
                for rid in surround_hconduct {
                    self.parts[rid].temp = pt;
                }

                let mut ctemph = pt;
                let mut ctempl = pt;
                let pv = self.pv[vc];
                // Boiling/freezing point shifts with pressure.
                if ((element(t).properties & props::TYPE_LIQUID) != 0
                    && is_element_or_none(element(t).high_temperature_transition)
                    && (element(element(t).high_temperature_transition as u16).properties
                        & props::TYPE_GAS)
                        != 0)
                    || t == pt::LNTG
                    || t == pt::SLTW
                {
                    ctemph -= 2.0 * pv;
                } else if ((element(t).properties & props::TYPE_GAS) != 0
                    && is_element_or_none(element(t).low_temperature_transition)
                    && (element(element(t).low_temperature_transition as u16).properties
                        & props::TYPE_LIQUID)
                        != 0)
                    || t == pt::WTRV
                {
                    ctempl -= 2.0 * pv;
                }

                let mut s = true;

                // Fix ice with ctype = 0.
                if (t == pt::ICEI || t == pt::SNOW)
                    && (!is_element(self.parts[i].ctype) || self.parts[i].ctype == pt::ICEI as i32 || self.parts[i].ctype == pt::SNOW as i32)
                {
                    self.parts[i].ctype = pt::WATR as i32;
                }

                let def = element(t);
                if def.high_temperature_transition != NT && ctemph >= def.high_temperature {
                    if def.high_temperature_transition != ST {
                        if t == pt::FOG {
                            self.parts[i].ctype = 0;
                        }
                        t = def.high_temperature_transition as u16;
                    } else if t == pt::ICEI || t == pt::SNOW {
                        let ct = self.parts[i].ctype;
                        if ct > 0 && (ct as usize) < crate::elements_generated::PT_NUM && ct as u16 != t {
                            let ctd = element(ct as u16);
                            if ctd.low_temperature_transition == pt::ICEI as i32
                                || ctd.low_temperature_transition == pt::SNOW as i32
                            {
                                if pt < ctd.low_temperature {
                                    s = false;
                                }
                            } else if pt < 273.15 {
                                s = false;
                            }
                            if s {
                                t = ct as u16;
                                self.parts[i].ctype = 0;
                                self.parts[i].life = 0;
                            }
                        } else {
                            s = false;
                        }
                    } else if t == pt::SLTW {
                        t = if self.rng.chance(1, 4) { pt::SALT } else { pt::WTRV };
                    } else if t == pt::BRMT {
                        if self.parts[i].ctype == pt::TUNG as i32 {
                            if ctemph < element(pt::TUNG).high_temperature {
                                s = false;
                            } else {
                                t = pt::LAVA;
                                self.parts[i].t = pt::TUNG;
                            }
                        } else if ctemph >= def.high_temperature {
                            t = pt::LAVA;
                        } else {
                            s = false;
                        }
                    } else if t == pt::CRMC {
                        let pvh = (pv + self.hv_now((x - 2) / CELL as i32, y / CELL as i32)
                            + self.hv_now((x + 2) / CELL as i32, y / CELL as i32)
                            + self.hv_now(x / CELL as i32, (y - 2) / CELL as i32)
                            + self.hv_now(x / CELL as i32, (y + 2) / CELL as i32))
                            * 2.0;
                        let pres = pvh.max(0.0);
                        if ctemph < pres + element(pt::CRMC).high_temperature {
                            s = false;
                        } else {
                            t = pt::LAVA;
                        }
                    } else if t == pt::RIME {
                        if self.parts[i].tmp > 5 {
                            t = pt::ACID;
                            self.parts[i].life = 25 + 5 * self.parts[i].tmp;
                            self.parts[i].tmp = 0;
                        } else {
                            t = if self.parts[i].ctype == pt::DSTW as i32 {
                                pt::DSTW
                            } else {
                                pt::WATR
                            };
                            self.parts[i].ctype = 0;
                        }
                    } else {
                        s = false;
                    }
                } else if def.low_temperature_transition != NT && ctempl < def.low_temperature {
                    if def.low_temperature_transition != ST {
                        t = def.low_temperature_transition as u16;
                    } else if t == pt::WTRV {
                        t = if pt < 273.0 { pt::RIME } else { pt::DSTW };
                    } else if t == pt::LAVA {
                        let ct = self.parts[i].ctype;
                        if ct > 0
                            && (ct as usize) < crate::elements_generated::PT_NUM
                            && ct as u16 != pt::LAVA
                            && element(ct as u16).enabled
                        {
                            let ctd = element(ct as u16);
                            if ct as u16 == pt::THRM && pt >= element(pt::BMTL).high_temperature {
                                s = false;
                            } else if (ct as u16 == pt::VIBR || ct as u16 == pt::BVBR) && pt >= 273.15 {
                                s = false;
                            } else if ct as u16 == pt::TUNG {
                                if pt >= ctd.high_temperature {
                                    s = false;
                                }
                            } else if ct as u16 == pt::CRMC {
                                let pvh = (pv + self.hv_now((x - 2) / CELL as i32, y / CELL as i32)
                                    + self.hv_now((x + 2) / CELL as i32, y / CELL as i32)
                                    + self.hv_now(x / CELL as i32, (y - 2) / CELL as i32)
                                    + self.hv_now(x / CELL as i32, (y + 2) / CELL as i32))
                                    * 2.0;
                                let pres = pvh.max(0.0);
                                if ctemph >= pres + element(pt::CRMC).high_temperature {
                                    s = false;
                                }
                            } else if ctd.high_temperature_transition == pt::LAVA as i32
                                || ct as u16 == pt::HEAC
                            {
                                if pt >= ctd.high_temperature {
                                    s = false;
                                }
                            } else if pt >= 973.0 {
                                s = false;
                            }
                            if s {
                                t = ct as u16;
                                self.parts[i].ctype = 0;
                                if t == pt::THRM {
                                    self.parts[i].tmp = 0;
                                    t = pt::BMTL;
                                }
                                if t == pt::PLUT {
                                    self.parts[i].tmp = 0;
                                    t = pt::LAVA;
                                }
                            }
                        } else if pt < 973.0 {
                            t = pt::STNE;
                        } else {
                            s = false;
                        }
                    } else {
                        s = false;
                    }
                } else {
                    s = false;
                }

                if s {
                    let old_t = self.parts[i].t;
                    if t == pt::ICEI || t == pt::LAVA || t == pt::SNOW {
                        self.parts[i].ctype = old_t as i32;
                    }
                    if t == pt::RIME {
                        self.parts[i].ctype = pt::DSTW as i32;
                    }
                    if !(t == pt::ICEI && self.parts[i].ctype == pt::FRZW as i32) && t != pt::ACID {
                        self.parts[i].life = 0;
                    }
                    if t == pt::FIRE {
                        self.parts[i].tmp = 0;
                        if old_t == pt::SEED {
                            self.parts[i].ctype = 0;
                            self.parts[i].tmp2 = 0;
                            self.parts[i].tmp3 = 0;
                            self.parts[i].tmp4 = 0;
                        }
                    }
                    if (element(t).properties & props::TYPE_GAS) != 0
                        && (element(old_t).properties & props::TYPE_GAS) == 0
                    {
                        self.pv[vc] += 0.5;
                    }
                    if t == pt::NONE {
                        self.kill_part(i);
                        return true;
                    }
                    if self.part_change_type(i, x, y, t as i16) {
                        if t == pt::FIRE || t == pt::PLSM || t == pt::CFLM {
                            self.parts[i].life = self.rng.between(120, 169);
                        }
                        if t == pt::LAVA {
                            match self.parts[i].ctype as u16 {
                                pt::BRMT => self.parts[i].ctype = pt::BMTL as i32,
                                pt::SAND => self.parts[i].ctype = pt::GLAS as i32,
                                pt::BGLA => self.parts[i].ctype = pt::GLAS as i32,
                                pt::PQRT => self.parts[i].ctype = pt::QRTZ as i32,
                                pt::LITH if self.parts[i].tmp2 > 3 => {
                                    self.parts[i].ctype = pt::GLAS as i32
                                }
                                _ => {}
                            }
                            self.parts[i].life = self.rng.between(240, 359);
                        }
                        transition_occurred = true;
                    }
                }

                self.parts[i].temp = restrict_flt(self.parts[i].temp, MIN_TEMP, MAX_TEMP);
                if t == pt::LAVA {
                    self.parts[i].life =
                        restrict_flt((self.parts[i].temp - 700.0) / 7.0, 0.0, 400.0) as i32;
                    if self.parts[i].ctype == pt::THRM as i32 && self.parts[i].tmp > 0 {
                        self.parts[i].tmp -= 1;
                        self.parts[i].temp = 3500.0;
                    }
                    if self.parts[i].ctype == pt::PLUT as i32 && self.parts[i].tmp > 0 {
                        self.parts[i].tmp -= 1;
                        self.parts[i].temp = MAX_TEMP;
                    }
                }
            } else {
                let vc = self.cell_idx(x / CELL as i32, y / CELL as i32);
                if (self.bmap_blockairh[vc] & 0x8) == 0 && self.bmap_blockairh[vc] < 0x7 {
                    self.bmap_blockairh[vc] += 1;
                }
                self.parts[i].temp = restrict_flt(self.parts[i].temp, MIN_TEMP, MAX_TEMP);
            }
        }

        // GoL cells cool down over time.
        if t == pt::LIFE {
            self.parts[i].temp = restrict_flt(self.parts[i].temp - 50.0, MIN_TEMP, MAX_TEMP);
        }

        // Sparks triggered by conductive walls.
        if (element(t).properties & props::PROP_CONDUCTS) != 0 || t == pt::SPRK {
            let mut nx = x % CELL as i32;
            if nx == 0 {
                nx = x / CELL as i32 - 1;
            } else if nx == CELL as i32 - 1 {
                nx = x / CELL as i32 + 1;
            } else {
                nx = x / CELL as i32;
            }
            let mut ny = y % CELL as i32;
            if ny == 0 {
                ny = y / CELL as i32 - 1;
            } else if ny == CELL as i32 - 1 {
                ny = y / CELL as i32 + 1;
            } else {
                ny = y / CELL as i32;
            }
            if self.in_cell_bounds(nx, ny) {
                let wc = self.cell_idx(nx, ny);
                if t != pt::SPRK {
                    if self.emap[wc] == 12 && self.parts[i].life == 0 && self.bmap[wc] != crate::air::WALL_STASIS
                    {
                        self.part_change_type(i, x, y, pt::SPRK as i16);
                        self.parts[i].life = 4;
                        self.parts[i].ctype = t as i32;
                        t = pt::SPRK;
                    }
                } else if matches!(
                    self.bmap[wc],
                    crate::air::WALL_DETECT
                        | crate::air::WALL_EWALL
                        | crate::air::WALL_ALLOWLIQUID
                        | crate::air::WALL_WALLELEC
                        | crate::air::WALL_ALLOWALLELEC
                        | crate::air::WALL_EHOLE
                ) {
                    self.set_emap(nx, ny);
                }
            }
        }

        // Basic explosion from the element's Explosive flag.
        let vc = self.cell_idx(x / CELL as i32, y / CELL as i32);
        if (element(t).explosive & 2) != 0 && self.pv[vc] > 2.5 {
            self.parts[i].life = self.rng.between(180, 259);
            self.parts[i].temp = restrict_flt(
                element(pt::FIRE).default_temp + (element(t).flammable / 2) as f32,
                MIN_TEMP,
                MAX_TEMP,
            );
            t = pt::FIRE;
            self.part_change_type(i, x, y, t as i16);
            self.pv[vc] += 0.25;
        }

        // Pressure / high-gravity transitions.
        {
            let gravtot = {
                let gcell = self.grav_force[self.cell_idx(x / CELL as i32, y / CELL as i32)];
                gcell.0.abs() + gcell.1.abs()
            };
            let def = element(t);
            let pv = self.pv[vc];
            let mut s = true;
            let mut to = t;
            if def.high_pressure_transition != NT && pv > def.high_pressure {
                if def.high_pressure_transition != ST {
                    to = def.high_pressure_transition as u16;
                } else if t == pt::BMTL {
                    if pv > 2.5 {
                        to = pt::BRMT;
                    } else if pv > 1.0 && self.parts[i].tmp == 1 {
                        to = pt::BRMT;
                    } else {
                        s = false;
                    }
                } else {
                    s = false;
                }
            } else if def.low_pressure_transition != NT
                && pv < def.low_pressure
                && gravtot <= def.low_pressure / 4.0
            {
                if def.low_pressure_transition != ST {
                    to = def.low_pressure_transition as u16;
                } else {
                    s = false;
                }
            } else if def.high_pressure_transition != NT && gravtot > def.high_pressure / 4.0 {
                if def.high_pressure_transition != ST {
                    to = def.high_pressure_transition as u16;
                } else if t == pt::BMTL {
                    if gravtot > 0.625 {
                        to = pt::BRMT;
                    } else if gravtot > 0.25 && self.parts[i].tmp == 1 {
                        to = pt::BRMT;
                    } else {
                        s = false;
                    }
                } else {
                    s = false;
                }
            } else {
                s = false;
            }
            if s {
                if to == pt::NONE {
                    self.kill_part(i);
                    return true;
                }
                self.parts[i].life = 0;
                if to == pt::BRMT {
                    self.parts[i].ctype = 0;
                    self.parts[i].tmp = 0;
                    self.parts[i].tmp2 = 0;
                    self.parts[i].tmp3 = 0;
                    self.parts[i].tmp4 = 0;
                }
                if self.part_change_type(i, x, y, to as i16) {
                    if to == pt::FIRE {
                        self.parts[i].life = self.rng.between(120, 169);
                    }
                    transition_occurred = true;
                }
            }
        }
        // Newtonian gravity: particles with mass (tmp) and heavy elements affect
        // gravity wells. Heavy elements without an explicit mass update receive
        // a default mass derived from their weight. This runs in transition
        // (not updates) so it stays within the owned module boundary.
        {
            let cx = x / CELL as i32;
            let cy = y / CELL as i32;
            if self.in_cell_bounds(cx, cy) {
                let vc = self.cell_idx(cx, cy);
                let def = element(t);
                let heavy = def.weight >= 80;
                let has_grav_mass_update = matches!(
                    t,
                    pt::NWHL | pt::GRVT | pt::GPMP | pt::BOMB | pt::GBMB | pt::FRAY
                );
                if !has_grav_mass_update {
                    if self.parts[i].tmp != 0 {
                        self.grav_mass[vc] += 0.001 * self.parts[i].tmp as f32;
                    } else if heavy {
                        if self.parts[i].tmp == 0 {
                            self.parts[i].tmp = def.weight;
                        }
                        self.grav_mass[vc] += 0.001 * def.weight as f32;
                    }
                }
            }
        }

        // Horizontal liquid equalization (water_equal_test).
        let ti = self.parts[i].t;
        if self.settings.water_equal_test
            && ti != 0
            && (element(ti).properties & props::TYPE_LIQUID) != 0
            && element(ti).falldown == 2
        {
            let ex = (self.parts[i].x + 0.5) as i32;
            let ey = (self.parts[i].y + 0.5) as i32;
            if self.flood_water(i, ex, ey) {
                self.parts[i].flags |= flags::SKIPMOVE;
            }
        }
        let _ = &mut x;
        let _ = &mut y;
        transition_occurred
    }

    fn can_conduct(&self, t: u16, rt: u16) -> bool {
        if rt == 0 || self.is_heat_insulator_by_type(rt) {
            return false;
        }
        if t == pt::FILT && (rt == pt::BRAY || rt == pt::BIZR || rt == pt::BIZRG) {
            return false;
        }
        if rt == pt::FILT && (t == pt::BRAY || t == pt::PHOT || t == pt::BIZR || t == pt::BIZRG) {
            return false;
        }
        if (t == pt::ELEC && rt == pt::DEUT) || (t == pt::DEUT && rt == pt::ELEC) {
            return false;
        }
        true
    }

    fn is_heat_insulator_by_type(&self, t: u16) -> bool {
        let def = element(t);
        def.heat_conduct == 0 || (t == pt::HSWC) || ((t == pt::PIPE || t == pt::PPIP))
    }

    #[inline]
    fn hv_now(&self, cx: i32, cy: i32) -> f32 {
        if self.in_cell_bounds(cx, cy) {
            self.pv[self.cell_idx(cx, cy)]
        } else {
            0.0
        }
    }

    /// GoL step, implemented in `gol.rs`.
    pub fn simulate_gol(&mut self) {
        crate::gol::simulate_gol(self);
    }
}

#[inline]
pub fn is_element_or_none(t: i32) -> bool {
    t == 0 || (t > 0 && (t as usize) < crate::elements_generated::PT_NUM && element(t as u16).enabled)
}
