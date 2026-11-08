//! Simulation state and allocation, mirroring the reference `Simulation`.
//!
//! The grid is measured in pixels (`w` × `h`); the air/ wall layer uses
//! `CELL`-sized cells like the reference. `pmap` packs `(particle_index << 9) |
//! element_id` exactly as `PMAP()` does.

use crate::air::{
    AirMode, ConvectionMode, WALL_ALLOWAIR, WALL_ALLOWALLELEC, WALL_ALLOWENERGY, WALL_ALLOWGAS,
    WALL_ALLOWLIQUID, WALL_ALLOWPOWDER, WALL_BLOCKAIR, WALL_DESTROYALL, WALL_DETECT, WALL_ERASE,
    WALL_ERASEALL, WALL_EWALL, WALL_FAN, WALL_GRAV, WALL_STASIS, WALL_STREAM, WALL_WALL,
    WALL_WALLELEC,
};
use crate::element::element;
use crate::elements_generated::{props, PT_NUM};
use crate::entity::PlayerState;
use crate::particle::Particle;
use crate::portal::PortalStore;
use crate::rng::Rng;
use crate::wireless::WirelessStore;

pub const CELL: usize = 4;
pub const ISTP: f32 = CELL as f32 / 2.0;
pub const MAX_VELOCITY: f32 = 1.0e4;
pub const MAX_TEMP: f32 = 9999.0;
pub const MIN_TEMP: f32 = 0.0;
pub const MAX_PRESSURE: f32 = 256.0;
pub const MIN_PRESSURE: f32 = -256.0;
/// `R_TEMP` from the reference config (22 °C).
pub const R_TEMP: f32 = 22.0;
/// No transition.
pub const NT: i32 = -1;
/// Special (element-specific) transition marker.
pub const ST: i32 = -2;
pub const PMAPBITS: u32 = 9;
pub const PMAPMASK: u32 = (1 << PMAPBITS) - 1;

#[inline]
pub fn pmap_pack(id: u32, t: u16) -> u32 {
    (id << PMAPBITS) | (t as u32 & PMAPMASK)
}

#[inline]
pub fn pmap_id(r: u32) -> u32 {
    r >> PMAPBITS
}

#[inline]
pub fn pmap_type(r: u32) -> u16 {
    (r & PMAPMASK) as u16
}

#[inline]
pub fn restrict_flt(v: f32, lo: f32, hi: f32) -> f32 {
    if v.is_nan() {
        lo
    } else {
        v.clamp(lo, hi)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EdgeMode {
    Void = 0,
    Solid = 1,
    Loop = 2,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GravityMode {
    Vertical = 0,
    Off = 1,
    Radial = 2,
    Custom = 3,
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub air_mode: AirMode,
    pub convection_mode: ConvectionMode,
    pub gravity_mode: GravityMode,
    pub edge_mode: EdgeMode,
    /// Heat simulation on (`legacy_enable == false` means heat sim is active).
    pub heat_sim: bool,
    pub ambient_heat: bool,
    pub newtonian_gravity: bool,
    pub water_equal_test: bool,
    pub ambient_air_temp: f32,
    pub edge_pressure: f32,
    pub edge_velocity_x: f32,
    pub edge_velocity_y: f32,
    pub vorticity_coeff: f32,
    pub custom_gravity: (f32, f32),
    /// Frames per GoL generation.
    pub gol_speed: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            air_mode: AirMode::On,
            convection_mode: ConvectionMode::Boussinesq,
            gravity_mode: GravityMode::Vertical,
            edge_mode: EdgeMode::Void,
            heat_sim: true,
            ambient_heat: false,
            newtonian_gravity: false,
            water_equal_test: false,
            ambient_air_temp: R_TEMP + 273.15,
            edge_pressure: 0.0,
            edge_velocity_x: 0.0,
            edge_velocity_y: 0.0,
            vorticity_coeff: 0.1,
            custom_gravity: (0.0, 1.0),
            gol_speed: 10,
        }
    }
}

pub struct Sim {
    pub width: usize,
    pub height: usize,
    pub cells_x: usize,
    pub cells_y: usize,

    pub pmap: Vec<u32>,
    pub photons: Vec<u32>,
    pub parts: Vec<Particle>,
    /// Free slots (indices of dead particles) available for reuse.
    pub free: Vec<u32>,

    pub bmap: Vec<u8>,
    pub emap: Vec<u8>,
    pub fvx: Vec<f32>,
    pub fvy: Vec<f32>,

    pub vx: Vec<f32>,
    pub vy: Vec<f32>,
    pub pv: Vec<f32>,
    pub hv: Vec<f32>,
    pub ovx: Vec<f32>,
    pub ovy: Vec<f32>,
    pub opv: Vec<f32>,
    pub ohv: Vec<f32>,
    pub bmap_blockair: Vec<u8>,
    pub bmap_blockairh: Vec<u8>,

    /// Newtonian gravity force per air cell (x, y).
    pub grav_force: Vec<(f32, f32)>,
    /// Newtonian gravity mass per air cell.
    pub grav_mass: Vec<f32>,

    pub rng: Rng,
    pub settings: Settings,
    pub tick: u64,
    /// `can_move[mover][target]`: 0 bounce, 1 swap, 2 same space, 3 special.
    pub can_move: Vec<u8>,
    pub element_count: [u32; PT_NUM],
    /// Glow buffer for rendering (RGB per pixel).
    pub glow: Vec<[f32; 3]>,
    pub gol_counter: u32,
    pub emp_decor: i32,
    pub grav_mask: Vec<u32>,
    /// Render colour buffer, refreshed by `render`.
    pub colour: Vec<u32>,
    /// Kill reason counters (diagnostics).
    pub kill_stats: [u32; 8],

    // --- shared-world state owned by the world layer (Agent 1) -----------
    /// Persistent portal payloads (`Simulation::portalp`).
    pub portal: PortalStore,
    /// Persistent wireless channel state (`Simulation::wireless`).
    pub wireless: WirelessStore,
    /// Player 1 stickman state (`Simulation::player`).
    pub player: PlayerState,
    /// Player 2 stickman state (`Simulation::player2`).
    pub player2: PlayerState,
    /// Fighter registry (`Simulation::fighters[]`).
    pub fighters: Vec<PlayerState>,
    /// Live fighter count (`Simulation::fighcount`).
    pub figh_count: i32,
}

impl Sim {
    pub fn new(width: usize, height: usize) -> Self {
        assert!(width % CELL == 0 && height % CELL == 0);
        let cx = width / CELL;
        let cy = height / CELL;
        let n = cx * cy;
        let mut sim = Self {
            width,
            height,
            cells_x: cx,
            cells_y: cy,
            pmap: vec![0; width * height],
            photons: vec![0; width * height],
            parts: Vec::with_capacity(width * height / 4),
            free: Vec::new(),
            bmap: vec![0; n],
            emap: vec![0; n],
            fvx: vec![0.0; n],
            fvy: vec![0.0; n],
            vx: vec![0.0; n],
            vy: vec![0.0; n],
            pv: vec![0.0; n],
            hv: vec![0.0; n],
            ovx: vec![0.0; n],
            ovy: vec![0.0; n],
            opv: vec![0.0; n],
            ohv: vec![0.0; n],
            bmap_blockair: vec![0; n],
            bmap_blockairh: vec![0; n],
            grav_force: vec![(0.0, 0.0); n],
            grav_mass: vec![0.0; n],
            rng: Rng::default(),
            settings: Settings::default(),
            tick: 0,
            can_move: vec![0; PT_NUM * PT_NUM],
            element_count: [0; PT_NUM],
            glow: vec![[0.0; 3]; width * height],
            gol_counter: 0,
            emp_decor: 0,
            grav_mask: vec![u32::MAX; n],
            colour: vec![0; width * height],
            kill_stats: [0; 8],
            portal: PortalStore::new(),
            wireless: WirelessStore::new(),
            player: PlayerState::default(),
            player2: PlayerState::default(),
            fighters: Vec::new(),
            figh_count: 0,
        };
        sim.init_can_move();
        sim.clear_air();
        sim.clear_ambient_heat();
        sim
    }

    #[inline]
    pub fn idx(&self, x: i32, y: i32) -> usize {
        y as usize * self.width + x as usize
    }

    #[inline]
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height
    }

    #[inline]
    pub fn cell_idx(&self, cx: i32, cy: i32) -> usize {
        cy as usize * self.cells_x + cx as usize
    }

    #[inline]
    pub fn in_cell_bounds(&self, cx: i32, cy: i32) -> bool {
        cx >= 0 && cy >= 0 && (cx as usize) < self.cells_x && (cy as usize) < self.cells_y
    }

    // -- element move table ------------------------------------------------

    pub fn init_can_move(&mut self) {
        let n = PT_NUM;
        let cm = &mut self.can_move;
        // Nothing moves into an empty type.
        for d in 0..n {
            cm[d] = 0;
        }
        for m in 1..n {
            for d in 0..n {
                cm[m * n + d] = 1;
            }
        }
        let phot = crate::elements_generated::pt::PHOT as usize;
        for d in 1..n {
            cm[phot * n + d] = 2;
        }
        for m in 1..n {
            let em = element(m as u16);
            for d in 1..n {
                let ed = element(d as u16);
                if em.weight <= ed.weight || d as u16 == crate::elements_generated::pt::GEL {
                    cm[m * n + d] = 0;
                }
                if m as u16 == crate::elements_generated::pt::NEUT
                    && (ed.properties & props::PROP_NEUTPASS) != 0
                {
                    cm[m * n + d] = 2;
                }
                if m as u16 == crate::elements_generated::pt::NEUT
                    && (ed.properties & props::PROP_NEUTABSORB) != 0
                {
                    cm[m * n + d] = 1;
                }
                if m as u16 == crate::elements_generated::pt::NEUT
                    && (ed.properties & props::PROP_NEUTPENETRATE) != 0
                {
                    cm[m * n + d] = 1;
                }
                if d as u16 == crate::elements_generated::pt::NEUT
                    && (em.properties & props::PROP_NEUTPENETRATE) != 0
                {
                    cm[m * n + d] = 0;
                }
                if (em.properties & props::TYPE_ENERGY) != 0
                    && (ed.properties & props::TYPE_ENERGY) != 0
                {
                    cm[m * n + d] = 2;
                }
            }
        }
        let stkm = crate::elements_generated::pt::STKM as usize;
        let stkm2 = crate::elements_generated::pt::STKM2 as usize;
        let figh = crate::elements_generated::pt::FIGH as usize;
        for d in 0..n {
            let ed = element(d as u16);
            let mut stkm_move = 0;
            if (ed.properties & (props::TYPE_LIQUID | props::TYPE_GAS)) != 0 {
                stkm_move = 2;
            }
            if d == 0
                || d as u16 == crate::elements_generated::pt::PRTO
                || d as u16 == crate::elements_generated::pt::SPAWN
                || d as u16 == crate::elements_generated::pt::SPAWN2
            {
                stkm_move = 2;
            }
            cm[stkm * n + d] = stkm_move;
            cm[stkm2 * n + d] = stkm_move;
            cm[figh * n + d] = stkm_move;
            cm[crate::elements_generated::pt::SPRK as usize * n + d] = 0;
        }
        for m in 1..n {
            let em = element(m as u16);
            let p = |t: u16| t as usize;
            cm[m * n + p(crate::elements_generated::pt::BHOL)] = 1;
            cm[m * n + p(crate::elements_generated::pt::NBHL)] = 1;
            cm[m * n + p(crate::elements_generated::pt::STKM)] = 0;
            cm[m * n + p(crate::elements_generated::pt::STKM2)] = 0;
            cm[m * n + p(crate::elements_generated::pt::FIGH)] = 0;
            cm[m * n + p(crate::elements_generated::pt::INVIS)] = 3;
            cm[m * n + p(crate::elements_generated::pt::CNCT)] = 0;
            cm[m * n + p(crate::elements_generated::pt::PVOD)] = 3;
            cm[m * n + p(crate::elements_generated::pt::VOID)] = 3;
            cm[m * n + p(crate::elements_generated::pt::EMBR)] = 0;
            cm[p(crate::elements_generated::pt::EMBR) * n + m] = 0;
            if (em.properties & props::TYPE_ENERGY) != 0 {
                cm[m * n + p(crate::elements_generated::pt::VIBR)] = 1;
                cm[m * n + p(crate::elements_generated::pt::BVBR)] = 1;
            }
            if (em.properties & props::TYPE_PART) != 0 {
                cm[m * n + p(crate::elements_generated::pt::SAWD)] = 0;
            }
        }
        for d in 0..n {
            let ed = element(d as u16);
            if (ed.properties & props::PROP_PHOTPASS) != 0 {
                cm[phot * n + d] = 2;
            }
            let p = |t: u16| t as usize;
            let exempt = d as u16 == crate::elements_generated::pt::DMND
                || d as u16 == crate::elements_generated::pt::INSL
                || d as u16 == crate::elements_generated::pt::VOID
                || d as u16 == crate::elements_generated::pt::PVOD
                || d as u16 == crate::elements_generated::pt::VIBR
                || d as u16 == crate::elements_generated::pt::BVBR
                || d as u16 == crate::elements_generated::pt::PRTI
                || d as u16 == crate::elements_generated::pt::PRTO;
            if !exempt {
                cm[p(crate::elements_generated::pt::PROT) * n + d] = 2;
                cm[p(crate::elements_generated::pt::GRVT) * n + d] = 2;
            }
        }
        let mut set = |m: u16, d: u16, v: u8, cm: &mut Vec<u8>| {
            cm[m as usize * n + d as usize] = v;
        };
        use crate::elements_generated::pt as P;
        set(P::DEST, P::DMND, 0, cm);
        set(P::DEST, P::CLNE, 0, cm);
        set(P::DEST, P::PCLN, 0, cm);
        set(P::DEST, P::BCLN, 0, cm);
        set(P::DEST, P::PBCN, 0, cm);
        set(P::DEST, P::ROCK, 0, cm);
        set(P::NEUT, P::INVIS, 2, cm);
        set(P::ELEC, P::LCRY, 2, cm);
        set(P::ELEC, P::EXOT, 2, cm);
        set(P::ELEC, P::GLOW, 2, cm);
        set(P::PHOT, P::LCRY, 3, cm);
        set(P::PHOT, P::GPMP, 3, cm);
        set(P::PHOT, P::BIZR, 2, cm);
        set(P::ELEC, P::BIZR, 2, cm);
        set(P::PHOT, P::BIZRG, 2, cm);
        set(P::ELEC, P::BIZRG, 2, cm);
        set(P::PHOT, P::BIZRS, 2, cm);
        set(P::ELEC, P::BIZRS, 2, cm);
        set(P::BIZR, P::FILT, 2, cm);
        set(P::BIZRG, P::FILT, 2, cm);
        set(P::ANAR, P::WHOL, 1, cm);
        set(P::ANAR, P::NWHL, 1, cm);
        set(P::ELEC, P::DEUT, 1, cm);
        set(P::THDR, P::THDR, 2, cm);
        set(P::EMBR, P::EMBR, 2, cm);
        set(P::TRON, P::SWCH, 3, cm);
        set(P::ELEC, P::RSST, 2, cm);
        set(P::ELEC, P::RSSS, 2, cm);
    }

    #[inline]
    pub fn can_move(&self, m: u16, d: u16) -> u8 {
        self.can_move[m as usize * PT_NUM + d as usize]
    }

    // -- particle allocation ----------------------------------------------

    /// Create a particle of type `t` at pixel `(x, y)`. Returns the index, or
    /// `None` when out of bounds / no free slot.
    pub fn create_part(&mut self, x: i32, y: i32, t: u16) -> Option<usize> {
        if !self.in_bounds(x, y) || t == 0 || (t as usize) >= PT_NUM {
            return None;
        }
        let def = element(t);
        let i = if let Some(i) = self.free.pop() {
            self.parts[i as usize] = Particle::new(t, x as f32, y as f32);
            i as usize
        } else {
            if self.parts.len() >= self.width * self.height {
                return None;
            }
            self.parts.push(Particle::new(t, x as f32, y as f32));
            self.parts.len() - 1
        };
        let p = &mut self.parts[i];
        p.temp = def.default_temp;
        // Apply declared DefaultProperties (life, ctype, tmp...).
        self.apply_defaults(i, t);
        let cell = self.idx(x, y);
        if (def.properties & props::TYPE_ENERGY) != 0 {
            self.photons[cell] = pmap_pack(i as u32, t);
        } else {
            self.pmap[cell] = pmap_pack(i as u32, t);
        }
        self.element_count[t as usize] += 1;
        if let Some(f) = crate::updates::create_fn(t) {
            f(self, i, x, y, t as i32, 0);
        }
        Some(i)
    }

    fn apply_defaults(&mut self, i: usize, t: u16) {
        // DefaultProperties assignments in the reference vary per element. Only
        // the fields that are actually declared in element sources are set.
        let def = element(t);
        if def.abbr == "LIFE" {
            self.parts[i].life = 0;
        }
        // Reference `IGNT.cpp`: `DefaultProperties.life = 3`. Without this the
        // ignition element never advances to its spreading branch.
        if def.abbr == "IGNT" {
            self.parts[i].life = 3;
        }
    }

    /// Create in the first free cell at or near `(x, y)` (reference `create_part(-1, ...)`).
    pub fn create_part_auto(&mut self, x: i32, y: i32, t: u16) -> Option<usize> {
        if !self.in_bounds(x, y) {
            return None;
        }
        self.create_part(x, y, t)
    }

    /// Kill a particle, freeing its slot.
    pub fn kill_part(&mut self, i: usize) {
        self.kill_part_reason(i, 7);
    }

    /// Kill a particle with a reason tag (debug/diagnostics only).
    pub fn kill_part_reason(&mut self, i: usize, reason: usize) {
        if reason < self.kill_stats.len() {
            self.kill_stats[reason] += 1;
        }
        let (x, y, t) = {
            let p = &mut self.parts[i];
            if p.t == 0 {
                return;
            }
            (p.x + 0.5, p.y + 0.5, p.t)
        };
        let xi = x as i32;
        let yi = y as i32;
        if self.in_bounds(xi, yi) {
            let cell = self.idx(xi, yi);
            if self.pmap[cell] != 0 && pmap_id(self.pmap[cell]) as usize == i {
                self.pmap[cell] = 0;
            }
            if self.photons[cell] != 0 && pmap_id(self.photons[cell]) as usize == i {
                self.photons[cell] = 0;
            }
        }
        self.parts[i].t = 0;
        self.free.push(i as u32);
        if (t as usize) < PT_NUM && self.element_count[t as usize] > 0 {
            self.element_count[t as usize] -= 1;
        }
    }

    /// Change a particle's element id in place. Returns true if the change
    /// happened (the reference can refuse, e.g. a second STKM).
    pub fn part_change_type(&mut self, i: usize, x: i32, y: i32, t: i16) -> bool {
        if t == 0 {
            self.kill_part(i);
            return false;
        }
        if t < 0 || (t as usize) >= PT_NUM || !element(t as u16).enabled {
            return false;
        }
        let t = t as u16;
        if let Some(allowed) = crate::updates::create_allowed_fn(t) {
            if !allowed(self, i, x, y, t as i32) {
                self.kill_part(i);
                return false;
            }
        }
        let old = self.parts[i].t;
        let from = old;
        let def = element(t);
        self.parts[i].t = t;
        if (from as usize) < PT_NUM && self.element_count[from as usize] > 0 {
            self.element_count[from as usize] -= 1;
        }
        self.element_count[t as usize] += 1;
        if self.in_bounds(x, y) {
            let cell = self.idx(x, y);
            if (def.properties & props::TYPE_ENERGY) != 0 {
                self.photons[cell] = pmap_pack(i as u32, t);
                if self.pmap[cell] != 0 && pmap_id(self.pmap[cell]) as usize == i {
                    self.pmap[cell] = 0;
                }
            } else {
                self.pmap[cell] = pmap_pack(i as u32, t);
                if self.photons[cell] != 0 && pmap_id(self.photons[cell]) as usize == i {
                    self.photons[cell] = 0;
                }
            }
        }
        if let Some(f) = crate::updates::change_type_fn(t) {
            f(self, i, x, y, from as i32, t as i32);
        }
        true
    }

    /// Rebuild `pmap`/`photons` from the particle array and compact dead slots,
    /// mirroring `RecalcFreeParticles`.
    pub fn recalc(&mut self) {
        for v in self.pmap.iter_mut() {
            *v = 0;
        }
        for v in self.photons.iter_mut() {
            *v = 0;
        }
        let mut write = 0usize;
        for read in 0..self.parts.len() {
            if self.parts[read].t == 0 {
                continue;
            }
            if write != read {
                self.parts[write] = self.parts[read];
            }
            let p = &self.parts[write];
            let x = (p.x + 0.5) as i32;
            let y = (p.y + 0.5) as i32;
            if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
                let cell = y as usize * self.width + x as usize;
                let def = element(p.t);
                let packed = pmap_pack(write as u32, p.t);
                if (def.properties & props::TYPE_ENERGY) != 0 {
                    self.photons[cell] = packed;
                } else if self.pmap[cell] == 0
                    || pmap_type(self.pmap[cell]) != crate::elements_generated::pt::INVIS
                {
                    self.pmap[cell] = packed;
                }
            }
            write += 1;
        }
        self.parts.truncate(write);
        self.free.clear();
        self.element_count = [0; PT_NUM];
        for p in self.parts.iter() {
            if (p.t as usize) < PT_NUM {
                self.element_count[p.t as usize] += 1;
            }
        }
    }

    // -- gravity -----------------------------------------------------------

    /// `GetGravityField`.
    pub fn gravity_field(
        &self,
        x: i32,
        y: i32,
        particle_grav: f32,
        newton_grav: f32,
    ) -> (f32, f32) {
        let mut gx;
        let mut gy;
        match self.settings.gravity_mode {
            GravityMode::Vertical => {
                gx = 0.0;
                gy = particle_grav;
            }
            GravityMode::Off => {
                gx = 0.0;
                gy = 0.0;
            }
            GravityMode::Radial => {
                let xcntr = (self.width / 2) as f32;
                let ycntr = (self.height / 2) as f32;
                let dx = x as f32 - xcntr;
                let dy = y as f32 - ycntr;
                if dx != 0.0 || dy != 0.0 {
                    let d = 0.01 - (dx * dx + dy * dy).sqrt();
                    gx = particle_grav * (dx / d);
                    gy = particle_grav * (dy / d);
                } else {
                    gx = 0.0;
                    gy = 0.0;
                }
            }
            GravityMode::Custom => {
                gx = particle_grav * self.settings.custom_gravity.0;
                gy = particle_grav * self.settings.custom_gravity.1;
            }
        }
        if newton_grav != 0.0 {
            let cx = (x as usize / CELL).min(self.cells_x - 1);
            let cy = (y as usize / CELL).min(self.cells_y - 1);
            let f = self.grav_force[cy * self.cells_x + cx];
            let mask = self.grav_mask[cy * self.cells_x + cx] != 0;
            if mask {
                gx += newton_grav * f.0;
                gy += newton_grav * f.1;
            }
        }
        (gx, gy)
    }

    /// Recompute Newtonian gravity from the mass grid.
    ///
    /// The reference uses an FFT convolution with a fixed kernel; this
    /// implementation runs a bounded Jacobi relaxation of the discrete Poisson
    /// equation on the same grid, which is a documented approximation (see
    /// `docs/tpt-parity/architecture.md`).
    pub fn update_newtonian_gravity(&mut self) {
        if !self.settings.newtonian_gravity {
            self.grav_force.iter_mut().for_each(|f| *f = (0.0, 0.0));
            return;
        }
        let (cx, cy) = (self.cells_x, self.cells_y);
        let mut phi = vec![0.0f32; cx * cy];
        let mut next = vec![0.0f32; cx * cy];
        let boundary = |x: usize, y: usize| x == 0 || y == 0 || x == cx - 1 || y == cy - 1;
        for _ in 0..24 {
            for y in 1..cy - 1 {
                for x in 1..cx - 1 {
                    let idx = y * cx + x;
                    let lap = phi[idx - 1] + phi[idx + 1] + phi[idx - cx] + phi[idx + cx];
                    next[idx] = (lap + self.grav_mass[idx]) * 0.25;
                }
            }
            for i in 0..cx * cy {
                let (x, y) = (i % cx, i / cx);
                if boundary(x, y) {
                    next[i] = 0.0;
                }
            }
            std::mem::swap(&mut phi, &mut next);
        }
        for y in 1..cy - 1 {
            for x in 1..cx - 1 {
                let idx = y * cx + x;
                let gx = (phi[idx + 1] - phi[idx - 1]) * 0.5;
                let gy = (phi[idx + cx] - phi[idx - cx]) * 0.5;
                let mask = self.grav_mask[idx] != 0;
                self.grav_force[idx] = if mask { (gx, gy) } else { (0.0, 0.0) };
            }
        }
        self.grav_mass.iter_mut().for_each(|m| *m = 0.0);
    }

    // -- walls -------------------------------------------------------------

    pub fn set_wall(&mut self, x: i32, y: i32, w: u8) {
        if !self.in_bounds(x, y) {
            return;
        }
        let cx = (x as usize / CELL) as i32;
        let cy = (y as usize / CELL) as i32;
        if !self.in_cell_bounds(cx, cy) {
            return;
        }
        let idx = self.cell_idx(cx, cy);
        self.bmap[idx] = w;
        self.emap[idx] = 0;
    }

    pub fn wall_at(&self, x: i32, y: i32) -> u8 {
        if !self.in_bounds(x, y) {
            return 0;
        }
        self.bmap[self.cell_idx(x as i32 / CELL as i32, y as i32 / CELL as i32)]
    }

    /// Apply the currently selected edge mode to the wall map, mirroring the
    /// reference `Simulation::SetEdgeMode`.
    ///
    /// `EDGE_SOLID` places a `WL_WALL` ring on the outer CELLs so particles are
    /// blocked before they can reach the off-screen kill test. `EDGE_VOID` and
    /// `EDGE_LOOP` clear that ring (loop particles are wrapped in `do_move`).
    /// Call this after changing `settings.edge_mode` (including on load).
    pub fn apply_edge_mode(&mut self) {
        let (cx, cy) = (self.cells_x, self.cells_y);
        if cx < 2 || cy < 2 {
            return;
        }
        match self.settings.edge_mode {
            EdgeMode::Solid => {
                for x in 0..cx {
                    self.bmap[x] = WALL_WALL;
                    self.bmap[(cy - 1) * cx + x] = WALL_WALL;
                }
                for y in 1..cy - 1 {
                    self.bmap[y * cx] = WALL_WALL;
                    self.bmap[y * cx + (cx - 1)] = WALL_WALL;
                }
            }
            EdgeMode::Void | EdgeMode::Loop => {
                for x in 0..cx {
                    self.bmap[x] = 0;
                    self.bmap[(cy - 1) * cx + x] = 0;
                }
                for y in 1..cy - 1 {
                    self.bmap[y * cx] = 0;
                    self.bmap[y * cx + (cx - 1)] = 0;
                }
            }
        }
        self.refresh_air_block_maps();
    }

    /// Numerical stabilisation pass (Agent 1 world infrastructure).
    ///
    /// The reference relies on `restrict_flt` at every transition site; a missed
    /// site or an element-supplied NaN can otherwise poison positions, the air
    /// fields, and (via pmap rebuild) neighbouring cells. This clamps only
    /// non-finite / out-of-range values and is a no-op for healthy state.
    pub fn sanitize(&mut self) {
        let ambient = self.settings.ambient_air_temp;
        for p in self.parts.iter_mut() {
            if p.t == 0 {
                continue;
            }
            if !p.x.is_finite() {
                p.x = 0.5;
            }
            if !p.y.is_finite() {
                p.y = 0.5;
            }
            if !p.vx.is_finite() {
                p.vx = 0.0;
            }
            if !p.vy.is_finite() {
                p.vy = 0.0;
            }
            if !p.temp.is_finite() {
                p.temp = ambient;
            }
            p.temp = p.temp.clamp(MIN_TEMP, MAX_TEMP);
            p.vx = p.vx.clamp(-MAX_VELOCITY, MAX_VELOCITY);
            p.vy = p.vy.clamp(-MAX_VELOCITY, MAX_VELOCITY);
        }
        for v in self.pv.iter_mut() {
            if !v.is_finite() {
                *v = 0.0;
            }
        }
        for v in self.vx.iter_mut() {
            if !v.is_finite() {
                *v = 0.0;
            }
        }
        for v in self.vy.iter_mut() {
            if !v.is_finite() {
                *v = 0.0;
            }
        }
        for v in self.hv.iter_mut() {
            if !v.is_finite() {
                *v = ambient;
            }
        }
    }

    /// True when wall `w` blocks particles of element `t` from being in the
    /// cell at all (the reference's per-particle wall kill test).
    pub fn wall_kills(&self, w: u8, t: u16) -> bool {
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
    }

    /// True when wall `w` blocks movement of element `t` (reference
    /// `IsWallBlocking`, used by `eval_move`).
    pub fn wall_blocks(&self, x: i32, y: i32, t: u16) -> bool {
        let w = self.wall_at(x, y);
        if w == 0 {
            return false;
        }
        let p = element(t).properties;
        if w == WALL_WALL || w == WALL_WALLELEC || w == WALL_BLOCKAIR {
            return true;
        }
        if w == WALL_ALLOWAIR
            || (w == WALL_ALLOWGAS && (p & props::TYPE_GAS) == 0)
            || (w == WALL_ALLOWENERGY && (p & props::TYPE_ENERGY) == 0)
            || (w == WALL_ALLOWLIQUID && (p & props::TYPE_LIQUID) == 0)
            || (w == WALL_ALLOWPOWDER && (p & props::TYPE_PART) == 0)
        {
            return true;
        }
        if w == WALL_EWALL || w == WALL_WALLELEC {
            let cx = x / CELL as i32;
            let cy = y / CELL as i32;
            if self.in_cell_bounds(cx, cy) && self.emap[self.cell_idx(cx, cy)] == 0 {
                return true;
            }
        }
        if w == WALL_DETECT {
            if (p & props::TYPE_SOLID) != 0 {
                return true;
            }
        }
        if w == WALL_STASIS || w == WALL_GRAV || w == WALL_FAN || w == WALL_STREAM {
            return false;
        }
        false
    }

    // -- physics API for elements (Agent 3) -----------------------------------

    /// Add a pressure impulse to an air cell (clamped to reference range).
    pub fn add_pressure(&mut self, cx: i32, cy: i32, dp: f32) {
        if !self.in_cell_bounds(cx, cy) {
            return;
        }
        let idx = self.cell_idx(cx, cy);
        self.pv[idx] = (self.pv[idx] + dp).clamp(MIN_PRESSURE, MAX_PRESSURE);
    }

    /// Add an air-velocity impulse to an air cell.
    pub fn add_air_velocity(&mut self, cx: i32, cy: i32, dvx: f32, dvy: f32) {
        if !self.in_cell_bounds(cx, cy) {
            return;
        }
        let idx = self.cell_idx(cx, cy);
        self.vx[idx] = (self.vx[idx] + dvx).clamp(MIN_PRESSURE, MAX_PRESSURE);
        self.vy[idx] = (self.vy[idx] + dvy).clamp(MIN_PRESSURE, MAX_PRESSURE);
    }

    /// Whether wall type `w` allows airflow through it.
    pub fn wall_allows_air(&self, w: u8) -> bool {
        !(w == WALL_WALL
            || w == WALL_WALLELEC
            || w == WALL_BLOCKAIR
            || (w == WALL_EWALL))
    }

    /// Whether wall type `w` allows heat (ambient) through it.
    pub fn wall_allows_heat(&self, w: u8) -> bool {
        w != WALL_WALL && w != WALL_WALLELEC && w != WALL_BLOCKAIR
    }

    pub fn set_emap(&mut self, cx: i32, cy: i32) {
        if self.in_cell_bounds(cx, cy) {
            let idx = self.cell_idx(cx, cy);
            let w = self.bmap[idx];
            if w == WALL_DETECT {
                self.emap[idx] = 8;
            } else if w != 0 && w != WALL_STASIS {
                self.emap[idx] = 8;
            }
        }
    }

    // -- painting ----------------------------------------------------------

    /// Place (or replace) a particle at a pixel. Used by the element tool.
    pub fn paint(&mut self, x: i32, y: i32, t: u16, ctype: i32) {
        if !self.in_bounds(x, y) {
            return;
        }
        let cell = self.idx(x, y);
        let existing = if self.photons[cell] != 0 {
            self.photons[cell]
        } else {
            self.pmap[cell]
        };
        if existing != 0 {
            let id = pmap_id(existing) as usize;
            if id < self.parts.len() {
                let et = self.parts[id].t;
                if et == t {
                    return;
                }
                if let Some(draw) = crate::updates::ctype_draw_fn(t) {
                    if draw(self, id, t as i32, ctype) {
                        return;
                    }
                }
                // A ctype-carrying element placed on top of another element
                // records it, mirroring `basicCtypeDraw`.
                if crate::updates::carries_ctype(t) && (element(t).properties & props::PROP_NOCTYPEDRAW) == 0 {
                    self.kill_part(id);
                } else {
                    self.kill_part(id);
                }
            }
        }
        if let Some(i) = self.create_part(x, y, t) {
            if ctype != 0 && crate::updates::carries_ctype(t) {
                self.parts[i].ctype = ctype;
            }
        }
    }

    /// Draw a pixel of wall (element tool for the Walls section).
    pub fn paint_wall(&mut self, x: i32, y: i32, w: u8) {
        if w == WALL_ERASE {
            self.set_wall(x, y, 0);
        } else if w == WALL_ERASEALL {
            self.set_wall(x, y, 0);
            let cell = self.idx(x, y);
            let existing = if self.photons[cell] != 0 {
                self.photons[cell]
            } else {
                self.pmap[cell]
            };
            if existing != 0 {
                self.kill_part(pmap_id(existing) as usize);
            }
        } else {
            self.set_wall(x, y, w);
        }
    }
}
