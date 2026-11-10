//! Surface-normal and glass-refraction helpers, ported from the TPT reference
//! (`Simulation.cpp` / `Simulation.h`, pinned reference).
//!
//! Functions/constants here follow the reference structure closely so that the
//! optics path stays reference-faithful.

use crate::elements_generated::pt;
use crate::particle::Particle;
use crate::sim::{pmap_id, pmap_type, Sim, PMAPMASK};

pub const SURF_RANGE: i32 = 10;
pub const NORMAL_MIN_EST: i32 = 3;
pub const NORMAL_INTERP: i32 = 20;
pub const NORMAL_FRAC: f32 = 16.0;
pub const REFRACT: i32 = 0x8000_0000u32 as i32;
pub const GLASS_IOR: f32 = 1.9;
pub const GLASS_DISP: f32 = 0.07;

/// `Simulation::GetNormalResult` (Simulation.h).
#[derive(Clone, Copy, Debug)]
pub struct GetNormalResult {
    pub success: bool,
    pub nx: f32,
    pub ny: f32,
    pub lx: i32,
    pub ly: i32,
    pub rx: i32,
    pub ry: i32,
}

impl GetNormalResult {
    pub const fn fail() -> Self {
        Self {
            success: false,
            nx: 0.0,
            ny: 0.0,
            lx: 0,
            ly: 0,
            rx: 0,
            ry: 0,
        }
    }
}

/// `direction_to_map(dx, dy, t)` (Simulation.cpp:1584-1611), active variant.
pub fn direction_to_map(dx: f32, dy: f32, _t: i32) -> u32 {
    ((dx >= 0.0) as u32)
        | ((((dx + dy) >= 0.0) as u32) << 1)
        | (((dy >= 0.0) as u32) << 2)
        | ((((dy - dx) >= 0.0) as u32) << 3)
        | (((dx <= 0.0) as u32) << 4)
        | ((((dx + dy) <= 0.0) as u32) << 5)
        | (((dy <= 0.0) as u32) << 6)
        | ((((dy - dx) <= 0.0) as u32) << 7)
}

impl Sim {
    /// `Simulation::get_wavelength_bin` (Simulation.cpp).
    ///
    /// Uses the simulation RNG and consumes exactly one `rng.gen()` call when
    /// the random branch is reached, and zero when the early return happens.
    /// Operates on `parts[i].ctype` only.
    #[allow(dead_code)]
    pub(crate) fn get_wavelength_bin(&mut self, i: usize) -> i32 {
        let wm = self.parts[i].ctype as u32;
        if (wm & 0x3FFF_FFFF) == 0 {
            return -1;
        }

        let mut w0: i32 = 30;
        let mut w_m: i32 = 0;
        // Portable path (matches the #else branch) for exact bit behaviour.
        for bit in 0..30i32 {
            if (wm & (1u32 << bit)) != 0 {
                if bit < w0 {
                    w0 = bit;
                }
                if bit > w_m {
                    w_m = bit;
                }
            }
        }

        if w_m - w0 < 5 {
            return w_m + w0;
        }

        let r = self.rng.gen() as i32;

        let mut idx = ((r >> 1) % (w_m - w0 - 4)) + w0;
        if idx < 0 {
            idx += w_m - w0 - 4;
        }

        if (r & 1) != 0 {
            self.parts[i].ctype = ((wm & 0x1Fu32.wrapping_shl(idx as u32)) as i32);
            (idx + 2) * 2
        } else {
            self.parts[i].ctype = ((wm & 0xFu32.wrapping_shl(idx as u32)) as i32);
            (idx + 2) * 2 - 1
        }
    }
    /// `Simulation::is_blocking` (Simulation.cpp:1613-1624).
    pub(crate) fn is_blocking(&self, t: i32, x: i32, y: i32) -> bool {
        if (t & REFRACT) != 0 {
            if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
                return false;
            }
            let cell = self.idx(x, y);
            let mut r = self.pmap[cell];
            if r != 0 {
                let id = pmap_id(r) as usize;
                if id < self.parts.len() {
                    r = (r & !PMAPMASK) | self.parts[id].t as u32;
                }
            }
            let rt = pmap_type(r);
            return rt == pt::GLAS || rt == pt::BGLA;
        }

        self.eval_move(t as u16, x, y) == 0
    }

    /// `Simulation::is_boundary` (Simulation.cpp:1626-1633).
    pub(crate) fn is_boundary(&self, pt: i32, x: i32, y: i32) -> bool {
        if !self.is_blocking(pt, x, y) {
            return false;
        }
        if self.is_blocking(pt, x, y - 1)
            && self.is_blocking(pt, x, y + 1)
            && self.is_blocking(pt, x - 1, y)
            && self.is_blocking(pt, x + 1, y)
        {
            return false;
        }
        true
    }

    /// `Simulation::find_next_boundary` (Simulation.cpp:1635-1672).
    pub(crate) fn find_next_boundary(
        &self,
        pt: i32,
        x: &mut i32,
        y: &mut i32,
        dm: i32,
        em: &mut i32,
        reverse: bool,
    ) -> bool {
        const DX: [i32; 8] = [1, 1, 0, -1, -1, -1, 0, 1];
        const DY: [i32; 8] = [0, 1, 1, 1, 0, -1, -1, -1];
        const DE: [i32; 8] = [0x83, 0x07, 0x0E, 0x1C, 0x38, 0x70, 0xE0, 0xC1];

        if *x <= 0
            || *x >= self.width as i32 - 1
            || *y <= 0
            || *y >= self.height as i32 - 1
        {
            return false;
        }

        let mut dm = dm;
        if *em != -1 {
            dm &= DE[*em as usize];
        }

        let mut mask: u32 = 0;
        for i in 0..8 {
            if (dm & (1 << i)) != 0 && self.is_blocking(pt, *x + DX[i], *y + DY[i]) {
                mask |= 1u32 << i;
            }
        }
        for i in 0..8 {
            let n = if reverse { (i + 1) & 7 } else { (i + 7) & 7 };
            if (mask & (1u32 << i)) != 0 && (mask & (1u32 << n)) == 0 {
                *x += DX[i];
                *y += DY[i];
                *em = i as i32;
                return true;
            }
        }

        false
    }

    /// `Simulation::get_normal` (Simulation.cpp:1674-1717).
    pub(crate) fn get_normal(&self, pt: i32, x: i32, y: i32, dx: f32, dy: f32) -> GetNormalResult {
        if dx == 0.0 && dy == 0.0 {
            return GetNormalResult::fail();
        }

        if !self.is_boundary(pt, x, y) {
            return GetNormalResult::fail();
        }

        let ldm = if (pt & REFRACT) != 0 {
            0xFF
        } else {
            direction_to_map(-dy, dx, pt) as i32
        };
        let rdm = if (pt & REFRACT) != 0 {
            0xFF
        } else {
            direction_to_map(dy, -dx, pt) as i32
        };
        let (mut lx, mut ly) = (x, y);
        let (mut rx, mut ry) = (x, y);
        let (mut lv, mut rv) = (true, true);
        let (mut lm, mut rm): (i32, i32) = (-1, -1);

        let mut j = 0i32;
        for _ in 0..SURF_RANGE {
            if lv {
                lv = self.find_next_boundary(pt, &mut lx, &mut ly, ldm, &mut lm, true);
            }
            if rv {
                rv = self.find_next_boundary(pt, &mut rx, &mut ry, rdm, &mut rm, false);
            }
            j += (lv as i32) + (rv as i32);
            if !lv && !rv {
                break;
            }
        }

        if j < NORMAL_MIN_EST {
            return GetNormalResult::fail();
        }

        if lx == rx && ly == ry {
            return GetNormalResult::fail();
        }
        let ex = (rx - lx) as f32;
        let ey = (ry - ly) as f32;
        let r = 1.0 / ex.hypot(ey);
        let nx = ey * r;
        let ny = -ex * r;

        GetNormalResult {
            success: true,
            nx,
            ny,
            lx,
            ly,
            rx,
            ry,
        }
    }

    /// `Simulation::get_normal_interp<true>` (PhotoelectricEffect = true).
    pub(crate) fn get_normal_interp_phot(
        &mut self,
        pt: i32,
        mut x0: f32,
        mut y0: f32,
        mut dx: f32,
        mut dy: f32,
    ) -> GetNormalResult {
        dx /= NORMAL_FRAC;
        dy /= NORMAL_FRAC;

        let mut i = 0;
        let mut x = 0i32;
        let mut y = 0i32;
        while i < NORMAL_INTERP {
            x = (x0 + 0.5) as i32;
            y = (y0 + 0.5) as i32;
            if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
                return GetNormalResult::fail();
            }
            if self.is_boundary(pt, x, y) {
                break;
            }
            x0 += dx;
            y0 += dy;
            i += 1;
        }
        if i >= NORMAL_INTERP {
            return GetNormalResult::fail();
        }

        if pt == pt::PHOT as i32 {
            self.photoelectric_effect(x, y);
        }

        self.get_normal(pt, x, y, dx, dy)
    }

    /// `Simulation::get_normal_interp<false>`.
    #[allow(dead_code)]
    pub(crate) fn get_normal_interp(
        &self,
        pt: i32,
        mut x0: f32,
        mut y0: f32,
        mut dx: f32,
        mut dy: f32,
    ) -> GetNormalResult {
        dx /= NORMAL_FRAC;
        dy /= NORMAL_FRAC;

        let mut i = 0;
        while i < NORMAL_INTERP {
            let x = (x0 + 0.5) as i32;
            let y = (y0 + 0.5) as i32;
            if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
                return GetNormalResult::fail();
            }
            if self.is_boundary(pt, x, y) {
                return self.get_normal(pt, x, y, dx, dy);
            }
            x0 += dx;
            y0 += dy;
            i += 1;
        }
        GetNormalResult::fail()
    }

    /// `Simulation::photoelectric_effect(nx, ny)` (Simulation.cpp:1568-1582).
    pub(crate) fn photoelectric_effect(&mut self, nx: i32, ny: i32) {
        if nx < 0 || ny < 0 || nx >= self.width as i32 || ny >= self.height as i32 {
            return;
        }
        let cell = self.idx(nx, ny);
        let r = self.pmap[cell];
        if r == 0 {
            return;
        }
        let rid = pmap_id(r) as usize;
        if rid >= self.parts.len() {
            return;
        }
        if self.parts[rid].t != pt::PSCN || self.parts[rid].life != 0 {
            return;
        }
        let has_nscn = |x: i32, y: i32, sim: &Sim| -> bool {
            if x < 0 || y < 0 || x >= sim.width as i32 || y >= sim.height as i32 {
                return false;
            }
            let rr = sim.pmap[sim.idx(x, y)];
            if rr == 0 {
                return false;
            }
            let id = pmap_id(rr) as usize;
            id < sim.parts.len() && sim.parts[id].t == pt::NSCN
        };
        if has_nscn(nx - 1, ny, self)
            || has_nscn(nx + 1, ny, self)
            || has_nscn(nx, ny - 1, self)
            || has_nscn(nx, ny + 1, self)
        {
            self.parts[rid].ctype = pt::PSCN as i32;
            self.part_change_type(rid, nx, ny, pt::SPRK as i16);
            if rid < self.parts.len() && self.parts[rid].t == pt::SPRK {
                self.parts[rid].life = 4;
            }
        }
    }
}
