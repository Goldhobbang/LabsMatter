//! REALMATTER physics: forces, collisions, fluids, gases, thermodynamics.
//!
//! Owned by A3 (Thermodynamics, Fluids & Chemistry). Implements the
//! `PhysicsModule` (and supporting thermal/reaction) contracts from
//! `realmatter-core`. This crate depends only on `realmatter-core`.

pub use realmatter_core as core;

pub mod approx;
pub mod fluid;
pub mod gas;
pub mod heat;
pub mod reaction;
pub mod stability;
pub mod transfer;

use core::cell::Cell;
use core::cell::cell_flags;
use core::error::Result;
use core::material::MaterialProps;
use core::material::MaterialState;
use core::traits::{ChemistryModule, PhysicsModule, SimulationModule};
use core::units::Time;
use core::world::WorldConfig;

/// Default material properties used when no registry entry exists (air).
pub fn default_material_props(id: core::material::MaterialId) -> MaterialProps {
    MaterialProps {
        id,
        name: "unknown",
        state: MaterialState::Gas,
        density: 1.225,
        specific_heat: 1005.0,
        thermal_conductivity: 0.026,
        melting_point: f32::INFINITY,
        boiling_point: f32::INFINITY,
        viscosity: 0.0,
        color: [0.8, 0.9, 1.0, 0.3],
        flammable: false,
        ignition_temp: f32::INFINITY,
    }
}

/// Physics engine implementing the core contracts.
pub struct PhysicsEngine {
    width: u32,
    height: u32,
    materials: Vec<MaterialProps>,
    pub gravity: f32,
    pub damping: f32,
}

impl PhysicsEngine {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            materials: Vec::new(),
            gravity: core::units::constants::GRAVITY,
            damping: 0.999,
        }
    }

    pub fn from_config(config: &WorldConfig) -> Self {
        Self::new([config.width as i32, config.height as i32, 1])
    }

    pub fn set_materials(&mut self, materials: Vec<MaterialProps>) {
        let max_id = materials.iter().map(|m| m.id as usize).max().unwrap_or(0);
        let mut table = vec![default_material_props(0); max_id + 1];
        for m in materials {
            if (m.id as usize) < table.len() {
                table[m.id as usize] = m;
            }
        }
        self.materials = table;
    }

    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn material(&self, id: core::material::MaterialId) -> MaterialProps {
        self.materials
            .get(id as usize)
            .cloned()
            .unwrap_or_else(|| default_material_props(id))
    }

    #[inline]
    pub fn idx(&self, x: u32, y: u32) -> usize {
        (y as usize) * (self.width as usize) + (x as usize)
    }

    #[inline]
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height
    }

    pub fn thermal_step(&mut self, cells: &mut [Cell], dt: Time) -> Result<()> {
        heat::conduct(cells, self.width, self.height, &self.materials, dt as f32);
        heat::phase_transitions(cells, &self.materials);
        Ok(())
    }

    pub fn gas_step(&mut self, cells: &mut [Cell], dt: Time) -> Result<()> {
        gas::update_pressure(cells, &self.materials);
        gas::diffuse(cells, self.width, self.height, &self.materials, dt as f32);
        gas::apply_buoyancy(cells, self.width, self.height, &self.materials, dt as f32, self.gravity);
        Ok(())
    }

    pub fn fluid_step(&mut self, cells: &mut [Cell], dt: Time) -> Result<()> {
        fluid::apply_viscosity(cells, &self.materials, dt as f32);
        fluid::apply_pressure_gradient(cells, self.width, self.height, dt as f32);
        fluid::apply_buoyancy(cells, self.width, self.height, &self.materials, dt as f32, self.gravity);
        Ok(())
    }
}

impl SimulationModule for PhysicsEngine {
    fn name(&self) -> &'static str {
        "physics"
    }
    fn init(&mut self, config: &WorldConfig) -> Result<()> {
        self.width = config.width;
        self.height = config.height;
        Ok(())
    }
    fn step(&mut self, _dt: Time) -> Result<()> {
        Ok(())
    }
}

impl PhysicsModule for PhysicsEngine {
    fn apply_forces(&mut self, cells: &mut [Cell], dt: Time) -> Result<()> {
        let w = self.width;
        let h = self.height;
        let dtf = dt as f32;
        // Gravity.
        for y in 0..h {
            for x in 0..w {
                let i = self.idx(x, y);
                if cells[i].is_empty() || cells[i].has_flag(cell_flags::FIXED) {
                    continue;
                }
                cells[i].vy += -self.gravity * dtf;
                cells[i].vx *= self.damping;
                cells[i].vy *= self.damping;
            }
        }
        // Pressure gradient.
        let pressures: Vec<f32> = cells.iter().map(|c| c.pressure).collect();
        for y in 0..h {
            for x in 0..w {
                let i = self.idx(x, y);
                if cells[i].is_empty() || cells[i].has_flag(cell_flags::FIXED) {
                    continue;
                }
                let mut gx = 0.0f32;
                let mut gy = 0.0f32;
                if x + 1 < w { gx += pressures[self.idx(x + 1, y)] - pressures[i]; }
                if x > 0 { gx -= pressures[i] - pressures[self.idx(x - 1, y)]; }
                if y + 1 < h { gy += pressures[self.idx(x, y + 1)] - pressures[i]; }
                if y > 0 { gy -= pressures[i] - pressures[self.idx(x, y - 1)]; }
                let m = self.material(cells[i].material);
                let inv_mass = if m.density > 0.0 { 1.0 / m.density } else { 0.0 };
                cells[i].vx -= gx * inv_mass * dtf * 0.5;
                cells[i].vy -= gy * inv_mass * dtf * 0.5;
            }
        }
        Ok(())
    }

    fn resolve_collisions(&mut self, cells: &mut [Cell], _dt: Time) -> Result<()> {
        let w = self.width;
        let h = self.height;
        // Boundaries.
        for x in 0..w {
            let top = self.idx(x, 0);
            let bot = self.idx(x, h - 1);
            cells[top].vx = 0.0;
            cells[top].vy = 0.0;
            cells[bot].vx = 0.0;
            cells[bot].vy = 0.0;
        }
        for y in 0..h {
            let left = self.idx(0, y);
            let right = self.idx(w - 1, y);
            cells[left].vx = 0.0;
            cells[left].vy = 0.0;
            cells[right].vx = 0.0;
            cells[right].vy = 0.0;
        }
        // Walls.
        for cell in cells.iter_mut() {
            if cell.material == core::material::MATERIAL_WALL {
                cell.vx = 0.0;
                cell.vy = 0.0;
                cell.set_flag(cell_flags::FIXED, true);
            }
        }
        Ok(())
    }
}

impl ChemistryModule for PhysicsEngine {
    fn react(&mut self, cells: &mut [Cell], dt: Time) -> Result<()> {
        reaction::react_all(cells, self.width, self.height, &self.materials, dt as f32);
        Ok(())
    }
    fn can_react(&self, a: core::material::MaterialId, b: core::material::MaterialId) -> bool {
        reaction::can_react(a, b)
    }
}
