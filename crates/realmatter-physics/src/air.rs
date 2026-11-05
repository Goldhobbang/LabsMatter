//! Air simulation: persistent velocity and pressure fields on a `CELL`-sized
//! staggered grid, ported from the reference `Air::update_air`.
//!
//! The field is the single source of truth for airflow: particles read it
//! (advection), write to it (momentum injection, hot air), and walls block it.
//! Nothing here is faked — pressure and velocity are computed by the same
//! pressure↔velocity relaxation the reference uses.

use realmatter_core::cell::Cell;
use realmatter_core::material::{MaterialProps, MaterialState};

/// Air cell size in simulation pixels, matching the reference `CELL`.
pub const CELL: usize = 4;

/// Reference air constants (`SimulationConfig.h`).
pub const AIR_TSTEPP: f32 = 0.3;
pub const AIR_TSTEPV: f32 = 0.4;
pub const AIR_VADV: f32 = 0.3;
pub const AIR_VLOSS: f32 = 0.999;
pub const AIR_PLOSS: f32 = 0.9999;
/// Ambient-heat advection distance multiplier (`Air.cpp`).
pub const ADV_DISTANCE_MULT: f32 = 0.7;
pub const MAX_PRESSURE: f32 = 256.0;
pub const MIN_PRESSURE: f32 = -256.0;

/// Air simulation modes, mirroring the reference `AirMode` enum.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AirMode {
    /// Full pressure + velocity.
    On = 0,
    /// Pressure field kept, velocity zeroed.
    PressureOff = 1,
    /// Velocity kept, pressure zeroed.
    VelocityOff = 2,
    /// Both zeroed (field still exists, reads return 0).
    Off = 3,
    /// Field frozen: no update at all.
    NoUpdate = 4,
}

impl AirMode {
    pub fn from_u32(v: u32) -> Self {
        match v {
            1 => AirMode::PressureOff,
            2 => AirMode::VelocityOff,
            3 => AirMode::Off,
            4 => AirMode::NoUpdate,
            _ => AirMode::On,
        }
    }

    /// Convert the three modes exposed by the LabsMatter controls.
    pub fn from_sim_mode(v: u32) -> Self {
        match v {
            1 => AirMode::Off,
            2 => AirMode::VelocityOff,
            _ => AirMode::On,
        }
    }
}

#[inline]
fn mix(a: f32, b: f32, f: f32) -> f32 {
    a + (b - a) * f
}

/// 3×3 normalised kernel `exp(-2(i² + j²))` (`Air::make_kernel`).
pub fn make_kernel() -> [f32; 9] {
    let mut k = [0.0f32; 9];
    let mut s = 0.0f32;
    for j in -1i32..2 {
        for i in -1i32..2 {
            let v = (-2.0f32 * (i * i + j * j) as f32).exp();
            k[(i + 1 + 3 * (j + 1)) as usize] = v;
            s += v;
        }
    }
    let inv = 1.0 / s;
    for v in k.iter_mut() {
        *v *= inv;
    }
    k
}

/// Persistent air field.
pub struct AirField {
    pub cells_x: usize,
    pub cells_y: usize,
    pub vx: Vec<f32>,
    pub vy: Vec<f32>,
    pub pv: Vec<f32>,
    ovx: Vec<f32>,
    ovy: Vec<f32>,
    opv: Vec<f32>,
    /// 1 = this air cell blocks airflow (wall / solid structure).
    pub blockair: Vec<u8>,
    kernel: [f32; 9],
}

impl AirField {
    pub fn new(width: usize, height: usize) -> Self {
        let cx = (width / CELL).max(1);
        let cy = (height / CELL).max(1);
        let n = cx * cy;
        Self {
            cells_x: cx,
            cells_y: cy,
            vx: vec![0.0; n],
            vy: vec![0.0; n],
            pv: vec![0.0; n],
            ovx: vec![0.0; n],
            ovy: vec![0.0; n],
            opv: vec![0.0; n],
            blockair: vec![0; n],
            kernel: make_kernel(),
        }
    }

    #[inline]
    pub fn idx(&self, cx: i32, cy: i32) -> Option<usize> {
        if cx >= 0 && cy >= 0 && (cx as usize) < self.cells_x && (cy as usize) < self.cells_y {
            Some(cy as usize * self.cells_x + cx as usize)
        } else {
            None
        }
    }

    #[inline]
    pub fn vx_at(&self, x: i32, y: i32) -> f32 {
        self.idx(x / CELL as i32, y / CELL as i32)
            .map(|i| self.vx[i])
            .unwrap_or(0.0)
    }

    #[inline]
    pub fn vy_at(&self, x: i32, y: i32) -> f32 {
        self.idx(x / CELL as i32, y / CELL as i32)
            .map(|i| self.vy[i])
            .unwrap_or(0.0)
    }

    #[inline]
    pub fn pv_at(&self, x: i32, y: i32) -> f32 {
        self.idx(x / CELL as i32, y / CELL as i32)
            .map(|i| self.pv[i])
            .unwrap_or(0.0)
    }

    /// Recompute the air-blocking map from the particle grid. Fixed solids
    /// (walls, stone, glass, metal structures) block airflow; everything else
    /// is permeable. This is the LabsMatter mapping of the reference wall
    /// layer, which has no separate storage in this engine yet.
    pub fn rebuild_blockair(&mut self, cells: &[Cell], width: usize, height: usize) {
        let (cx, cy) = (self.cells_x, self.cells_y);
        self.blockair.iter_mut().for_each(|b| *b = 0);
        for y in 0..height.min(cy * CELL) {
            for x in 0..width.min(cx * CELL) {
                let cell = &cells[y * width + x];
                if cell.material == 0 {
                    continue;
                }
                if cell.has_flag(realmatter_core::cell::cell_flags::FIXED) {
                    let i = (y / CELL) * cx + x / CELL;
                    self.blockair[i] = 1;
                }
            }
        }
    }

    /// Clear the whole field (used on reset / air-mode changes that need it).
    pub fn clear(&mut self) {
        self.vx.iter_mut().for_each(|v| *v = 0.0);
        self.vy.iter_mut().for_each(|v| *v = 0.0);
        self.pv.iter_mut().for_each(|v| *v = 0.0);
    }

    /// `Air::update_air`: one full pressure/velocity pass.
    pub fn update(&mut self, mode: AirMode) {
        if mode == AirMode::NoUpdate {
            return;
        }
        let (cx, cy) = (self.cells_x, self.cells_y);
        if cx < 3 || cy < 3 {
            match mode {
                AirMode::Off => self.clear(),
                AirMode::PressureOff => self.pv.fill(0.0),
                AirMode::VelocityOff => {
                    self.vx.fill(0.0);
                    self.vy.fill(0.0);
                }
                AirMode::On | AirMode::NoUpdate => {}
            }
            return;
        }
        let k = self.kernel;

        // Edge damping (reference mixes toward 0 at the border).
        for i in 0..cy {
            let row = i * cx;
            for x in [0usize, 1, cx - 2, cx - 1] {
                self.pv[row + x] = mix(0.0, self.pv[row + x], 0.8);
                self.vx[row + x] = mix(0.0, self.vx[row + x], 0.9);
                self.vy[row + x] = mix(0.0, self.vy[row + x], 0.9);
            }
        }
        for i in 0..cx {
            for y in [0usize, 1, cy - 2, cy - 1] {
                let idx = y * cx + i;
                self.pv[idx] = mix(0.0, self.pv[idx], 0.8);
                self.vx[idx] = mix(0.0, self.vx[idx], 0.9);
                self.vy[idx] = mix(0.0, self.vy[idx], 0.9);
            }
        }

        // Zero velocities next to air-blocking cells.
        for j in 1..cy - 1 {
            for i in 1..cx - 1 {
                if self.blockair[j * cx + i] != 0 {
                    self.vx[j * cx + i] = 0.0;
                    self.vx[j * cx + i - 1] = 0.0;
                    self.vx[j * cx + i + 1] = 0.0;
                    self.vy[j * cx + i] = 0.0;
                    self.vy[(j - 1) * cx + i] = 0.0;
                    self.vy[(j + 1) * cx + i] = 0.0;
                }
            }
        }

        // Pressure from velocity divergence.
        for y in 1..cy - 1 {
            for x in 1..cx - 1 {
                let idx = y * cx + x;
                let mut dp = 0.0f32;
                dp += self.vx[idx - 1] - self.vx[idx + 1];
                dp += self.vy[idx - cx] - self.vy[idx + cx];
                self.pv[idx] = mix(0.0, self.pv[idx], AIR_PLOSS) + dp * AIR_TSTEPP * 0.5;
            }
        }

        // Velocity from pressure gradient.
        for y in 1..cy - 1 {
            for x in 1..cx - 1 {
                let idx = y * cx + x;
                let mut dx = 0.0f32;
                let mut dy = 0.0f32;
                dx += self.pv[idx - 1] - self.pv[idx + 1];
                dy += self.pv[idx - cx] - self.pv[idx + cx];
                self.vx[idx] = mix(0.0, self.vx[idx], AIR_VLOSS) + dx * AIR_TSTEPV * 0.5;
                self.vy[idx] = mix(0.0, self.vy[idx], AIR_VLOSS) + dy * AIR_TSTEPV * 0.5;
                if self.blockair[idx - 1] != 0 || self.blockair[idx] != 0 || self.blockair[idx + 1] != 0 {
                    self.vx[idx] = 0.0;
                }
                if self.blockair[idx - cx] != 0 || self.blockair[idx] != 0 || self.blockair[idx + cx] != 0 {
                    self.vy[idx] = 0.0;
                }
            }
        }

        // Kernel advection into double buffers.
        for y in 0..cy {
            for x in 0..cx {
                let idx = y * cx + x;
                let mut dx = 0.0f32;
                let mut dy = 0.0f32;
                let mut dp = 0.0f32;
                for j in -1i32..2 {
                    for i in -1i32..2 {
                        let f = k[(i + 1 + 3 * (j + 1)) as usize];
                        let ny = y as i32 + j;
                        let nx = x as i32 + i;
                        let src = if ny > 0
                            && ny < cy as i32 - 1
                            && nx > 0
                            && nx < cx as i32 - 1
                            && self.blockair[ny as usize * cx + nx as usize] == 0
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
                if ((dx * ADV_DISTANCE_MULT).abs() > 1.0 || (dy * ADV_DISTANCE_MULT).abs() > 1.0)
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
                            || self.blockair[cj as usize * cx + ci as usize] != 0
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
                if self.blockair[idx] == 0
                    && i0 >= 2
                    && i0 < cx as i32 - 3
                    && j0 >= 2
                    && j0 < cy as i32 - 3
                    && fx >= 0.0
                    && fy >= 0.0
                {
                    let a = j0 as usize * cx + i0 as usize;
                    let b = j0 as usize * cx + i0 as usize + 1;
                    let c = (j0 as usize + 1) * cx + i0 as usize;
                    let d = (j0 as usize + 1) * cx + i0 as usize + 1;
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

                dp = dp.clamp(MIN_PRESSURE, MAX_PRESSURE);
                dx = dx.clamp(MIN_PRESSURE, MAX_PRESSURE);
                dy = dy.clamp(MIN_PRESSURE, MAX_PRESSURE);

                match mode {
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

    /// Tool / particle write access.
    pub fn add_velocity(&mut self, x: i32, y: i32, dvx: f32, dvy: f32) {
        if let Some(i) = self.idx(x / CELL as i32, y / CELL as i32) {
            self.vx[i] = (self.vx[i] + dvx).clamp(MIN_PRESSURE, MAX_PRESSURE);
            self.vy[i] = (self.vy[i] + dvy).clamp(MIN_PRESSURE, MAX_PRESSURE);
        }
    }

    pub fn add_pressure(&mut self, x: i32, y: i32, dp: f32) {
        if let Some(i) = self.idx(x / CELL as i32, y / CELL as i32) {
            self.pv[i] = (self.pv[i] + dp).clamp(MIN_PRESSURE, MAX_PRESSURE);
        }
    }
}

/// Per-particle air coupling, mirroring the reference `UpdateParticles`
/// steps 7–12 (momentum injection, hot air, velocity decay, advection,
/// diffusion). Runs before movement so the CA consumes real velocities.
pub fn apply_air_to_particles(
    cells: &mut [Cell],
    width: usize,
    height: usize,
    materials: &[MaterialProps],
    field: &mut AirField,
    mode: AirMode,
    diffusion_rng: &mut dyn FnMut() -> f32,
) {
    if matches!(mode, AirMode::Off | AirMode::NoUpdate) {
        return;
    }
    let update_velocity = mode != AirMode::VelocityOff;
    let cx = field.cells_x;
    for y in 0..height {
        for x in 0..width {
            let i = y * width + x;
            let cell = &cells[i];
            if cell.material == 0 {
                continue;
            }
            let def = match materials.get(cell.material as usize) {
                Some(d) => d,
                None => continue,
            };
            let ci = (y / CELL) * cx + (x / CELL);
            if ci >= field.vx.len() {
                continue;
            }
            // 7. Momentum injection into the field.
            if update_velocity {
                field.vx[ci] = field.vx[ci] * def.air_loss + def.air_drag * cell.vx;
                field.vy[ci] = field.vy[ci] * def.air_loss + def.air_drag * cell.vy;
            }
            // 8. Hot air (pressure source).
            if mode != AirMode::PressureOff && def.hot_air != 0.0 {
                field.pv[ci] = (field.pv[ci] + 4.0 * def.hot_air).clamp(MIN_PRESSURE, MAX_PRESSURE);
            }
        }
    }

    if !update_velocity {
        return;
    }

    // 10–12. Velocity decay, advection, diffusion.
    let fvx: Vec<f32> = field.vx.clone();
    let fvy: Vec<f32> = field.vy.clone();
    for y in 0..height {
        for x in 0..width {
            let i = y * width + x;
            let cell = &mut cells[i];
            if cell.material == 0 {
                continue;
            }
            let (loss, advection, diffusion) = match materials.get(cell.material as usize) {
                Some(d) => (d.loss, d.advection, d.diffusion),
                None => continue,
            };
            let ci = (y / CELL) * cx + (x / CELL);
            if ci >= fvx.len() {
                continue;
            }
            cell.vx *= loss;
            cell.vy *= loss;
            cell.vx += advection * fvx[ci];
            cell.vy += advection * fvy[ci];
            if diffusion != 0.0 {
                cell.vx += diffusion * (2.0 * diffusion_rng() - 1.0);
                cell.vy += diffusion * (2.0 * diffusion_rng() - 1.0);
            }
            // Clamp like the reference velocity budget.
            cell.vx = cell.vx.clamp(-8.0, 8.0);
            cell.vy = cell.vy.clamp(-8.0, 8.0);
        }
    }
}

/// Move mobile particles along their velocity (airflow-driven motion).
///
/// The reference integrates sub-pixel steps; this engine's CA moves whole
/// cells, so a particle moves `round(v)` cells per tick when |v| >= 0.5,
/// capped at 3 cells. Powders and liquids only; gases are advected by the
/// field directly.
pub fn integrate_velocity(
    cells: &mut [Cell],
    width: usize,
    height: usize,
    materials: &[MaterialProps],
    active: Option<&realmatter_core::chunk_mask::ActiveMask>,
) {
    let (w, h) = (width, height);
    for y in (0..h).rev() {
        for x in 0..w {
            let i = y * w + x;
            if let Some(m) = active {
                if !m.cell_active(x, y) {
                    continue;
                }
            }
            let cell = cells[i];
            if cell.material == 0 || cell.has_flag(realmatter_core::cell::cell_flags::FIXED) {
                continue;
            }
            let state = match materials.get(cell.material as usize) {
                Some(d) => d.state,
                None => continue,
            };
            if !matches!(state, MaterialState::Powder | MaterialState::Liquid) {
                continue;
            }
            let vx = cell.vx;
            let vy = cell.vy;
            let dx = vx.round().clamp(-3.0, 3.0) as i32;
            let dy = vy.round().clamp(-3.0, 3.0) as i32;
            if dx == 0 && dy == 0 {
                continue;
            }
            // Step along the dominant axis first, then the other.
            let steps = [(dy, 0i32), (0, dx)];
            for (sy, sx) in steps {
                if sy == 0 && sx == 0 {
                    continue;
                }
                let nx = x as i32 + sx;
                let ny = y as i32 + sy;
                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                    continue;
                }
                let j = ny as usize * w + nx as usize;
                if cells[j].is_empty() {
                    cells.swap(i, j);
                    cells[j].set_flag(realmatter_core::cell::cell_flags::UPDATED, true);
                    break;
                }
            }
        }
    }
}

/// Advect gas cells along the air field (wind pushing smoke/steam around).
pub fn advect_gases(
    cells: &mut [Cell],
    width: usize,
    height: usize,
    materials: &[MaterialProps],
    field: &AirField,
    mode: AirMode,
    active: Option<&realmatter_core::chunk_mask::ActiveMask>,
) {
    if mode == AirMode::Off || mode == AirMode::NoUpdate || mode == AirMode::VelocityOff {
        return;
    }
    let (w, h) = (width, height);
    for y in (0..h).rev() {
        for x in 0..w {
            let i = y * w + x;
            if let Some(m) = active {
                if !m.cell_active(x, y) {
                    continue;
                }
            }
            let cell = cells[i];
            if cell.material == 0 {
                continue;
            }
            let state = match materials.get(cell.material as usize) {
                Some(d) => d.state,
                None => continue,
            };
            if state != MaterialState::Gas {
                continue;
            }
            let vx = field.vx_at(x as i32, y as i32);
            let vy = field.vy_at(x as i32, y as i32);
            let dx = (vx * 0.5).round().clamp(-2.0, 2.0) as i32;
            let dy = (vy * 0.5).round().clamp(-2.0, 2.0) as i32;
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                continue;
            }
            let j = ny as usize * w + nx as usize;
            if cells[j].is_empty() {
                cells.swap(i, j);
            }
        }
    }
}
