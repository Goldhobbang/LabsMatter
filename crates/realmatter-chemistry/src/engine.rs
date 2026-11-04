//! Chemistry reaction evaluation engine.

use crate::core;
use crate::core::cell::Cell;
use crate::core::cell::cell_flags;
use crate::core::material::MaterialProps;
use crate::kinetics::rate;
use crate::registry::ReactionRegistry;
use crate::thermo::{apply_heat_release, cell_mass};
use crate::core::traits::{ChemistryModule, SimulationModule};
use crate::core::world_config::WorldConfig;
use crate::core::units::Time;
use crate::core::error::Result;

/// Chemistry engine implementing ChemistryModule.
pub struct ChemistryEngine {
    width: u32,
    height: u32,
    registry: ReactionRegistry,
    materials: Vec<MaterialProps>,
}

impl ChemistryEngine {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            registry: ReactionRegistry::default_set(),
            materials: Vec::new(),
        }
    }

    pub fn from_config(config: &WorldConfig) -> Self {
        Self::new(config.width, config.height)
    }

    pub fn set_registry(&mut self, registry: ReactionRegistry) {
        self.registry = registry;
    }

    pub fn set_materials(&mut self, materials: Vec<MaterialProps>) {
        let max_id = materials.iter().map(|m| m.id as usize).max().unwrap_or(0);
        let mut table = vec![
            MaterialProps {
                id: 0,
                name: "air".to_string(),
                state: core::material::MaterialState::Gas,
                density: 1.225,
                specific_heat: 1005.0,
                thermal_conductivity: 0.026,
                melting_point: f32::INFINITY,
                boiling_point: f32::INFINITY,
                viscosity: 0.0,
                color: [0.8, 0.9, 1.0, 0.3],
                flammable: false,
                ignition_temp: f32::INFINITY,
            };
            max_id + 1
        ];
        for m in materials {
            let id = m.id as usize;
            if id < table.len() {
                table[id] = m;
            }
        }
        self.materials = table;
    }

    pub fn material(&self, id: core::material::MaterialId) -> MaterialProps {
        self.materials
            .get(id as usize)
            .cloned()
            .unwrap_or_else(|| {
                MaterialProps {
                    id,
                    name: "unknown".to_string(),
                    state: core::material::MaterialState::Gas,
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
            })
    }

    #[inline]
    pub fn idx(&self, x: u32, y: u32) -> usize {
        (y as usize) * (self.width as usize) + (x as usize)
    }
}

impl SimulationModule for ChemistryEngine {
    fn name(&self) -> &'static str {
        "chemistry"
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

impl ChemistryModule for ChemistryEngine {
    fn react(
        &mut self,
        cells: &mut [Cell],
        dt: Time,
    ) -> Result<()> {
        self.react_masked(cells, dt, None)
    }

    fn can_react(&self, a: core::material::MaterialId, b: core::material::MaterialId) -> bool {
        !self.registry.find_for_pair(a, b).is_empty()
    }
}

impl ChemistryEngine {
    /// Reaction pass with optional active-chunk skipping. Only empty source
    /// cells in inactive chunks are skipped (no-op); neighbor reads and
    /// writes into inactive chunks are unrestricted, so cross-boundary
    /// reactions and wake-up behave exactly as unmasked.
    pub fn react_masked(
        &mut self,
        cells: &mut [Cell],
        dt: Time,
        active: Option<&core::chunk_mask::ActiveMask>,
    ) -> Result<()> {
        let w = self.width;
        let h = self.height;
        let dtf = dt as f32;
        if cells.len() != (w as usize) * (h as usize) {
            return Ok(());
        }
        for y in 0..h {
            for x in 0..w {
                let i = self.idx(x, y);
                if let Some(m) = active {
                    if !m.cell_active(x as usize, y as usize) {
                        continue;
                    }
                }
                if cells[i].is_empty() {
                    continue;
                }
                let a_mat = cells[i].material;
                let ma = self.material(a_mat);
                // Check neighbors for reactions.
                for (dx, dy) in [(-1i32,0i32),(1,0),(0,-1),(0,1)] {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx < 0 || ny < 0 || (nx as u32) >= w || (ny as u32) >= h {
                        continue;
                    }
                    let ni = self.idx(nx as u32, ny as u32);
                    if cells[ni].is_empty() {
                        continue;
                    }
                    let b_mat = cells[ni].material;
                    let candidates = self.registry.find_for_pair(a_mat, b_mat);
                    for rid in candidates {
                        if let Some(def) = self.registry.get(rid) {
                            if !def.enabled {
                                continue;
                            }
                            let params = &def.schema.params;
                            // Check conditions.
                            if params.requires_wet && !cells[i].has_flag(cell_flags::WET) && !cells[ni].has_flag(cell_flags::WET) {
                                continue;
                            }
                            if params.combustion {
                                if !ma.flammable {
                                    continue;
                                }
                                if cells[i].temperature < params.ignition_temp {
                                    continue;
                                }
                            }
                            let r = rate(params, cells[i].temperature);
                            if r <= 0.0 {
                                continue;
                            }
                            // Apply reaction with probability-like factor.
                            if r * dtf > 0.001 {
                                // Combustion/oxidation/dissolution effects.
                                if params.combustion {
                                    cells[i].set_flag(cell_flags::BURNING, true);
                                    cells[i].lifetime = cells[i].lifetime.max(1.0);
                                    apply_heat_release(&mut cells[i], params.enthalpy, cell_mass(&ma), &ma);
                                    // Consume fuel slowly.
                                    if cells[i].lifetime > 0.0 {
                                        cells[i].lifetime = (cells[i].lifetime - dtf * 0.5).max(0.0);
                                        if cells[i].lifetime == 0.0 {
                                            cells[i].material = 0;
                                            cells[i].vx = 0.0;
                                            cells[i].vy = 0.0;
                                        }
                                    }
                                } else if params.requires_wet {
                                    cells[i].set_flag(cell_flags::WET, true);
                                    cells[i].set_flag(cell_flags::DISSOLVED, true);
                                    // Try to move to empty neighbor.
                                    if cells[ni].is_empty() {
                                        cells[ni] = cells[i];
                                        cells[i] = Cell::default();
                                    }
                                } else {
                                    // Oxidation/slow reaction.
                                    apply_heat_release(&mut cells[i], params.enthalpy, cell_mass(&ma), &ma);
                                    cells[i].temperature += (params.enthalpy.abs() / (ma.specific_heat * cell_mass(&ma))).min(5.0) * dtf * 0.01;
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
