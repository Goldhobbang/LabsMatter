//! Air simulation: pressure (`pv`), velocity (`vx`/`vy`) and ambient heat (`hv`).
//!
//! Direct behavioural port of the reference `Air::update_air` and
//! `Air::update_airh` on a `CELL`-sized staggered grid.

use crate::element::element;
use crate::elements_generated::pt;
use crate::sim::{Sim, CELL, MAX_PRESSURE, MAX_TEMP, MIN_PRESSURE, MIN_TEMP};

pub const AIR_TSTEPP: f32 = 0.3;
pub const AIR_TSTEPV: f32 = 0.4;
pub const AIR_VADV: f32 = 0.3;
pub const AIR_VLOSS: f32 = 0.999;
pub const AIR_PLOSS: f32 = 0.9999;
/// Ambient heat advection distance multiplier.
const ADV_DISTANCE_MULT: f32 = 0.7;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AirMode {
    On = 0,
    PressureOff = 1,
    VelocityOff = 2,
    Off = 3,
    NoUpdate = 4,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConvectionMode {
    None = 0,
    Legacy = 1,
    Boussinesq = 2,
}

/// 3x3 normalised kernel `exp(-2(i^2 + j^2))`, matching `Air::make_kernel`.
pub fn kernel() -> [f32; 9] {
    let mut k = [0.0f32; 9];
    let mut s = 0.0f32;
    for j in -1i32..2 {
        for i in -1i32..2 {
            let v = (-2.0f32 * (i * i + j * j) as f32).exp();
            k[((i + 1) + 3 * (j + 1)) as usize] = v;
            s += v;
        }
    }
    let inv = 1.0 / s;
    for v in k.iter_mut() {
        *v *= inv;
    }
    k
}

#[inline]
fn mix(a: f32, b: f32, f: f32) -> f32 {
    a + (b - a) * f
}

impl Sim {
    /// Ambient air temperature used for the "edge" of the world.
    #[inline]
    pub fn ambient_air_temp(&self) -> f32 {
        self.settings.ambient_air_temp
    }

    pub fn clear_air(&mut self) {
        let n = self.cells_x * self.cells_y;
        let p = self.settings.edge_pressure;
        let vx = self.settings.edge_velocity_x;
        let vy = self.settings.edge_velocity_y;
        self.pv.iter_mut().for_each(|v| *v = p);
        self.vx.iter_mut().for_each(|v| *v = vx);
        self.vy.iter_mut().for_each(|v| *v = vy);
        debug_assert_eq!(n, self.pv.len());
    }

    pub fn clear_ambient_heat(&mut self) {
        let t = self.settings.ambient_air_temp;
        self.hv.iter_mut().for_each(|v| *v = t);
    }

    /// `Air::update_air`: one pressure/velocity pass.
    pub fn update_air(&mut self) {
        if self.settings.air_mode == AirMode::NoUpdate {
            return;
        }
        let (cx, cy) = (self.cells_x, self.cells_y);
        let k = kernel();
        let ep = self.settings.edge_pressure;
        let evx = self.settings.edge_velocity_x;
        let evy = self.settings.edge_velocity_y;

        // --- edge damping -------------------------------------------------
        for i in 0..cy {
            let row = i * cx;
            for x in [0usize, 1, cx - 2, cx - 1] {
                self.pv[row + x] = mix(ep, self.pv[row + x], 0.8);
                self.vx[row + x] = mix(evx, self.vx[row + x], 0.9);
                self.vy[row + x] = mix(evy, self.vy[row + x], 0.9);
            }
        }
        for i in 0..cx {
            for y in [0usize, 1, cy - 2, cy - 1] {
                let idx = y * cx + i;
                self.pv[idx] = mix(ep, self.pv[idx], 0.8);
                self.vx[idx] = mix(evx, self.vx[idx], 0.9);
                self.vy[idx] = mix(evy, self.vy[idx], 0.9);
            }
        }

        // --- zero velocities next to air-blocking walls --------------------
        for j in 1..cy - 1 {
            for i in 1..cx - 1 {
                if self.bmap_blockair[j * cx + i] != 0 {
                    self.vx[j * cx + i] = 0.0;
                    self.vx[j * cx + i - 1] = 0.0;
                    self.vx[j * cx + i + 1] = 0.0;
                    self.vy[j * cx + i] = 0.0;
                    self.vy[(j - 1) * cx + i] = 0.0;
                    self.vy[(j + 1) * cx + i] = 0.0;
                }
            }
        }

        // --- pressure from velocity ---------------------------------------
        for y in 1..cy - 1 {
            for x in 1..cx - 1 {
                let idx = y * cx + x;
                let mut dp = 0.0f32;
                dp += self.vx[idx - 1] - self.vx[idx + 1];
                dp += self.vy[idx - cx] - self.vy[idx + cx];
                self.pv[idx] = mix(ep, self.pv[idx], AIR_PLOSS) + dp * AIR_TSTEPP * 0.5;
            }
        }

        // --- velocity from pressure ---------------------------------------
        for y in 1..cy - 1 {
            for x in 1..cx - 1 {
                let idx = y * cx + x;
                let mut dx = 0.0f32;
                let mut dy = 0.0f32;
                dx += self.pv[idx - 1] - self.pv[idx + 1];
                dy += self.pv[idx - cx] - self.pv[idx + cx];
                self.vx[idx] = mix(evx, self.vx[idx], AIR_VLOSS) + dx * AIR_TSTEPV * 0.5;
                self.vy[idx] = mix(evy, self.vy[idx], AIR_VLOSS) + dy * AIR_TSTEPV * 0.5;
                if self.bmap_blockair[idx - 1] != 0
                    || self.bmap_blockair[idx] != 0
                    || self.bmap_blockair[idx + 1] != 0
                {
                    self.vx[idx] = 0.0;
                }
                if self.bmap_blockair[idx - cx] != 0
                    || self.bmap_blockair[idx] != 0
                    || self.bmap_blockair[idx + cx] != 0
                {
                    self.vy[idx] = 0.0;
                }
            }
        }

        // --- advect + confine vorticity into the double buffers -----------
        let vort = self.settings.vorticity_coeff;
        let fan_active = self.bmap.iter().any(|w| *w == WALL_FAN);
        for y in 0..cy {
            for x in 0..cx {
                let idx = y * cx + x;
                let mut dx = 0.0f32;
                let mut dy = 0.0f32;
                let mut dp = 0.0f32;
                for j in -1i32..2 {
                    for i in -1i32..2 {
                        let f = k[((i + 1) + 3 * (j + 1)) as usize];
                        let ny = y as i32 + j;
                        let nx = x as i32 + i;
                        let src = if ny > 0
                            && ny < cy as i32 - 1
                            && nx > 0
                            && nx < cx as i32 - 1
                            && self.bmap_blockair[ny as usize * cx + nx as usize] == 0
                        {
                            ny as usize * cx + nx as usize
                        } else {
                            idx
                        };
                        dx += self.vx[src] * f;
                        dy += self.vy[src] * f;
                        dp += self.pv[src] * f;
                    }
                }

                // Far-away advection, stopping at intervening walls.
                let mut tx = x as f32 - dx * ADV_DISTANCE_MULT;
                let mut ty = y as f32 - dy * ADV_DISTANCE_MULT;
                if ((dx * ADV_DISTANCE_MULT).abs() > 1.0
                    || (dy * ADV_DISTANCE_MULT).abs() > 1.0)
                    && tx >= 2.0
                    && tx < cx as f32 - 2.0
                    && ty >= 2.0
                    && ty < cy as f32 - 2.0
                {
                    let (step_x, step_y, limit) = if dx.abs() > dy.abs() {
                        let sx = if dx < 0.0 { 1.0 } else { -1.0 };
                        (sx, -dy / dx.abs(), (dx * ADV_DISTANCE_MULT).abs() as i32)
                    } else {
                        let sy = if dy < 0.0 { 1.0 } else { -1.0 };
                        (-dx / dy.abs(), sy, (dy * ADV_DISTANCE_MULT).abs() as i32)
                    };
                    tx = x as f32;
                    ty = y as f32;
                    let mut step = 0;
                    while step < limit {
                        tx += step_x;
                        ty += step_y;
                        let cx_i = (tx + 0.5) as i32;
                        let cy_i = (ty + 0.5) as i32;
                        if cx_i < 0
                            || cy_i < 0
                            || cx_i >= cx as i32
                            || cy_i >= cy as i32
                            || self.bmap_blockair[cy_i as usize * cx + cx_i as usize] != 0
                        {
                            tx -= step_x;
                            ty -= step_y;
                            break;
                        }
                        step += 1;
                    }
                    if step == limit {
                        tx = x as f32 - dx * ADV_DISTANCE_MULT;
                        ty = y as f32 - dy * ADV_DISTANCE_MULT;
                    }
                }
                let i0 = tx as i32;
                let j0 = ty as i32;
                let fx = tx - i0 as f32;
                let fy = ty - j0 as f32;
                if self.bmap_blockair[idx] == 0
                    && i0 >= 2
                    && i0 < cx as i32 - 3
                    && j0 >= 2
                    && j0 < cy as i32 - 3
                    && fx >= 0.0
                    && fy >= 0.0
                {
                    let (a, b, c, d) = (
                        j0 as usize * cx + i0 as usize,
                        j0 as usize * cx + i0 as usize + 1,
                        (j0 as usize + 1) * cx + i0 as usize,
                        (j0 as usize + 1) * cx + i0 as usize + 1,
                    );
                    dx *= 1.0 - AIR_VADV;
                    dy *= 1.0 - AIR_VADV;
                    dx += AIR_VADV * (1.0 - fx) * (1.0 - fy) * self.vx[a];
                    dy += AIR_VADV * (1.0 - fx) * (1.0 - fy) * self.vy[a];
                    dx += AIR_VADV * fx * (1.0 - fy) * self.vx[b];
                    dy += AIR_VADV * fx * (1.0 - fy) * self.vy[b];
                    dx += AIR_VADV * (1.0 - fx) * fy * self.vx[c];
                    dy += AIR_VADV * (1.0 - fx) * fy * self.vy[c];
                    dx += AIR_VADV * fx * fy * self.vx[d];
                    dy += AIR_VADV * fx * fy * self.vy[d];
                }

                // Vorticity confinement.
                if vort > 0.0 && x > 1 && x < cx - 2 && y > 1 && y < cy - 2 {
                    let dwx = (self.vorticity_bmap(y, x + 1).abs()
                        - self.vorticity_bmap(y, x - 1).abs())
                        * 0.5;
                    let dwy = (self.vorticity_bmap(y + 1, x).abs()
                        - self.vorticity_bmap(y - 1, x).abs())
                        * 0.5;
                    let norm = (dwx * dwx + dwy * dwy).sqrt();
                    let w = self.vorticity_bmap(y, x);
                    dx += vort / 5.0 * dwy / (norm + 0.001) * w;
                    dy += vort / 5.0 * (-dwx) / (norm + 0.001) * w;
                }

                // Fans.
                if fan_active && self.bmap[idx] == WALL_FAN {
                    dx += self.fvx[idx];
                    dy += self.fvy[idx];
                }

                dp = dp.clamp(MIN_PRESSURE, MAX_PRESSURE);
                dx = dx.clamp(MIN_PRESSURE, MAX_PRESSURE);
                dy = dy.clamp(MIN_PRESSURE, MAX_PRESSURE);

                match self.settings.air_mode {
                    AirMode::On => {}
                    AirMode::PressureOff => dp = 0.0,
                    AirMode::VelocityOff => {
                        dx = 0.0;
                        dy = 0.0;
                    }
                    AirMode::Off => {
                        dx = 0.0;
                        dy = 0.0;
                        dp = 0.0;
                    }
                    AirMode::NoUpdate => {}
                }

                self.ovx[idx] = dx;
                self.ovy[idx] = dy;
                self.opv[idx] = dp;
            }
        }
        self.vx.copy_from_slice(&self.ovx);
        self.vy.copy_from_slice(&self.ovy);
        self.pv.copy_from_slice(&self.opv);
    }

    #[inline]
    fn vorticity_bmap(&self, y: usize, x: usize) -> f32 {
        let cx = self.cells_x;
        if x > 1 && x < cx - 2 && y > 1 && y < self.cells_y - 2 {
            let idx = y * cx + x;
            let dvydx = if self.bmap_blockair[idx] != 0
                || self.bmap_blockair[idx + 1] != 0
                || self.bmap_blockair[idx - 1] != 0
            {
                0.0
            } else {
                self.vy[idx + 1] - self.vy[idx - 1]
            };
            let dvxdy = if self.bmap_blockair[idx] != 0
                || self.bmap_blockair[idx + cx] != 0
                || self.bmap_blockair[idx - cx] != 0
            {
                0.0
            } else {
                self.vx[idx + cx] - self.vx[idx - cx]
            };
            (dvydx - dvxdy) * 0.5
        } else {
            0.0
        }
    }

    /// `Air::update_airh`: ambient heat transport and air convection.
    pub fn update_airh(&mut self) {
        let (cx, cy) = (self.cells_x, self.cells_y);
        let k = kernel();
        let ambient = self.settings.ambient_air_temp;

        for i in 0..cy {
            self.hv[i * cx] = ambient;
            self.hv[i * cx + 1] = ambient;
            self.hv[i * cx + cx - 2] = ambient;
            self.hv[i * cx + cx - 1] = ambient;
        }
        for i in 0..cx {
            self.hv[i] = ambient;
            self.hv[cx + i] = ambient;
            self.hv[(cy - 2) * cx + i] = ambient;
            self.hv[(cy - 1) * cx + i] = ambient;
        }

        let convection = self.settings.convection_mode;
        for y in 0..cy {
            for x in 0..cx {
                let idx = y * cx + x;
                let mut dh = 0.0f32;
                let mut dx = 0.0f32;
                let mut dy = 0.0f32;
                for j in -1i32..2 {
                    for i in -1i32..2 {
                        let f = k[((i + 1) + 3 * (j + 1)) as usize];
                        let (ny, nx) = (y as i32 + j, x as i32 + i);
                        let src = if ny > 0
                            && ny < cy as i32 - 1
                            && nx > 0
                            && nx < cx as i32 - 1
                            && (self.bmap_blockairh[ny as usize * cx + nx as usize] & 0x8) == 0
                        {
                            ny as usize * cx + nx as usize
                        } else {
                            idx
                        };
                        dh += self.hv[src] * f;
                        dx += self.vx[src] * f;
                        dy += self.vy[src] * f;
                    }
                }

                // Far-away temperature sampling.
                let mut tx = x as f32 - dx * ADV_DISTANCE_MULT;
                let mut ty = y as f32 - dy * ADV_DISTANCE_MULT;
                if ((dx * ADV_DISTANCE_MULT).abs() > 1.0
                    || (dy * ADV_DISTANCE_MULT).abs() > 1.0)
                    && tx >= 2.0
                    && tx < cx as f32 - 2.0
                    && ty >= 2.0
                    && ty < cy as f32 - 2.0
                {
                    let (step_x, step_y, limit) = if dx.abs() > dy.abs() {
                        let sx = if dx < 0.0 { 1.0 } else { -1.0 };
                        (sx, -dy / dx.abs(), (dx * ADV_DISTANCE_MULT).abs() as i32)
                    } else {
                        let sy = if dy < 0.0 { 1.0 } else { -1.0 };
                        (-dx / dy.abs(), sy, (dy * ADV_DISTANCE_MULT).abs() as i32)
                    };
                    tx = x as f32;
                    ty = y as f32;
                    let mut step = 0;
                    while step < limit {
                        tx += step_x;
                        ty += step_y;
                        let ci = (tx + 0.5) as i32;
                        let cj = (ty + 0.5) as i32;
                        if ci < 0
                            || cj < 0
                            || ci >= cx as i32
                            || cj >= cy as i32
                            || (self.bmap_blockairh[cj as usize * cx + ci as usize] & 0x8) != 0
                        {
                            tx -= step_x;
                            ty -= step_y;
                            break;
                        }
                        step += 1;
                    }
                    if step == limit {
                        tx = x as f32 - dx * ADV_DISTANCE_MULT;
                        ty = y as f32 - dy * ADV_DISTANCE_MULT;
                    }
                }
                let i0 = tx as i32;
                let j0 = ty as i32;
                let fx = tx - i0 as f32;
                let fy = ty - j0 as f32;
                if (self.bmap_blockairh[idx] & 0x8) == 0
                    && i0 >= 0
                    && i0 < cx as i32 - 1
                    && j0 >= 0
                    && j0 < cy as i32 - 1
                    && fx >= 0.0
                    && fy >= 0.0
                {
                    let odh = dh;
                    let get = |s: &Self, ci: usize, cj: usize| -> f32 {
                        let si = cj * cx + ci;
                        if (s.bmap_blockairh[si] & 0x8) != 0 {
                            odh
                        } else {
                            s.hv[si]
                        }
                    };
                    let (a, b, c, d) = (
                        (i0 as usize, j0 as usize),
                        (i0 as usize + 1, j0 as usize),
                        (i0 as usize, j0 as usize + 1),
                        (i0 as usize + 1, j0 as usize + 1),
                    );
                    dh *= 1.0 - AIR_VADV;
                    dh += AIR_VADV * (1.0 - fx) * (1.0 - fy) * get(self, a.0, a.1);
                    dh += AIR_VADV * fx * (1.0 - fy) * get(self, b.0, b.1);
                    dh += AIR_VADV * (1.0 - fx) * fy * get(self, c.0, c.1);
                    dh += AIR_VADV * fx * fy * get(self, d.0, d.1);
                }
                if (self.bmap_blockairh[idx] & 0x8) != 0 {
                    dh = self.hv[idx];
                }
                dh = dh.clamp(MIN_TEMP, MAX_TEMP);
                self.ohv[idx] = dh;

                // Air convection.
                let mut dvx = self.vx[idx];
                let mut dvy = self.vy[idx];
                if x >= 2 && x < cx - 2 && y >= 2 && y < cy - 2 {
                    let (gx, gy) = self.gravity_field((x * CELL) as i32, (y * CELL) as i32, -1.0, -1.0);
                    match convection {
                        ConvectionMode::Legacy => {
                            let weight = ((self.hv[idx] - self.hv[idx - 1]) * gx
                                + (self.hv[idx] - self.hv[idx - cx]) * gy)
                                / 5000.0;
                            if weight > 0.0 && (self.bmap_blockairh[idx - cx] & 0x8) == 0 {
                                dvx += weight * gx;
                                dvy += weight * gy;
                            }
                        }
                        ConvectionMode::Boussinesq => {
                            let mut gx = gx;
                            let mut gy = gy;
                            let mag = (gx * gx + gy * gy).sqrt();
                            if mag > 10.0 {
                                gx /= 0.1 * mag;
                                gy /= 0.1 * mag;
                            }
                            let mut weight = (self.hv[idx] - ambient) / 10000.0;
                            if weight > 0.01 {
                                weight = 0.01;
                            }
                            dvx += weight * gx;
                            dvy += weight * gy;
                        }
                        ConvectionMode::None => {}
                    }
                }
                self.vx[idx] = dvx.clamp(MIN_PRESSURE, MAX_PRESSURE);
                self.vy[idx] = dvy.clamp(MIN_PRESSURE, MAX_PRESSURE);
            }
        }
        self.hv.copy_from_slice(&self.ohv);

        // Air also blocks ambient heat where heat insulators sit.
        self.refresh_air_block_maps();
    }

    /// Recompute `bmap_blockair` / `bmap_blockairh` from walls and particles,
    /// equivalent to the reference's per-tick wall/ambient map pass plus the
    /// insulator scan from `ApproximateBlockAirMaps`.
    pub fn refresh_air_block_maps(&mut self) {
        let (cx, cy) = (self.cells_x, self.cells_y);
        for cyi in 0..cy {
            for cxi in 0..cx {
                let idx = cyi * cx + cxi;
                let w = self.bmap[idx];
                let em = self.emap[idx];
                let block = w == WALL_WALL
                    || w == WALL_WALLELEC
                    || w == WALL_BLOCKAIR
                    || (w == WALL_EWALL && em == 0);
                self.bmap_blockair[idx] = u8::from(block);
                self.bmap_blockairh[idx] = if block || w == WALL_GRAV { 0x8 } else { 0 };
            }
        }
        // Particles that conduct heat poorly also block ambient heat.
        // Reference: Air::ApproximateBlockAirMaps uses IsHeatInsulator(parts[i])
        // (instance-aware: HSWC life!=10, PIPE/PPIP without PFLAG_CAN_CONDUCT)
        // plus TTAN/RSSS and a randomized conductance threshold for the rest.
        const PFLAG_CAN_CONDUCT: i32 = 0x0000_0001;
        for i in 0..self.parts.len() {
            let t = self.parts[i].t;
            if t == 0 {
                continue;
            }
            let def = element(t);
            if t == pt::TTAN || t == pt::RSSS {
                let x = (self.parts[i].x + 0.5) as i32 / CELL as i32;
                let y = (self.parts[i].y + 0.5) as i32 / CELL as i32;
                if x >= 0 && y >= 0 && (x as usize) < cx && (y as usize) < cy {
                    let idx = y as usize * cx + x as usize;
                    self.bmap_blockair[idx] = 1;
                    self.bmap_blockairh[idx] = 0x8;
                }
                continue;
            }
            let is_insulator = self.is_heat_insulator(i)
                || def.heat_conduct <= (self.rng.next() % 250) as u8;
            if is_insulator {
                let x = (self.parts[i].x + 0.5) as i32 / CELL as i32;
                let y = (self.parts[i].y + 0.5) as i32 / CELL as i32;
                if x >= 0 && y >= 0 && (x as usize) < cx && (y as usize) < cy {
                    let idx = y as usize * cx + x as usize;
                    // Reference: only increments if not already fully blocked (&0x8==0)
                    if (self.bmap_blockairh[idx] & 0x8) == 0 {
                        self.bmap_blockairh[idx] = self.bmap_blockairh[idx].saturating_add(1);
                    }
                }
            }
            let _ = PFLAG_CAN_CONDUCT;
        }
    }

    /// `Air::Invert`.
    pub fn invert_air(&mut self) {
        for v in self.pv.iter_mut() {
            *v = -*v;
        }
        for v in self.vx.iter_mut() {
            *v = -*v;
        }
        for v in self.vy.iter_mut() {
            *v = -*v;
        }
    }
}

// Wall ids, mirrored from the reference `SimulationData.h` (`WL_*`).
pub const WALL_ERASE: u8 = 0;
pub const WALL_WALLELEC: u8 = 1;
pub const WALL_EWALL: u8 = 2;
pub const WALL_DETECT: u8 = 3;
pub const WALL_STREAM: u8 = 4;
pub const WALL_FAN: u8 = 5;
pub const WALL_ALLOWLIQUID: u8 = 6;
pub const WALL_DESTROYALL: u8 = 7;
pub const WALL_WALL: u8 = 8;
pub const WALL_ALLOWAIR: u8 = 9;
pub const WALL_ALLOWPOWDER: u8 = 10;
pub const WALL_ALLOWALLELEC: u8 = 11;
pub const WALL_EHOLE: u8 = 12;
pub const WALL_ALLOWGAS: u8 = 13;
pub const WALL_GRAV: u8 = 14;
pub const WALL_ALLOWENERGY: u8 = 15;
pub const WALL_BLOCKAIR: u8 = 16;
pub const WALL_ERASEALL: u8 = 17;
pub const WALL_STASIS: u8 = 18;
/// Number of user-placeable walls.
pub const UI_WALLCOUNT: u8 = 19;
