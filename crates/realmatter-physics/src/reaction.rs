//! Combustion, fire spread, oxidation, dissolution, mixing - visible behavior.

use crate::core;
use crate::core::cell::Cell;
use crate::core::cell::cell_flags;
use crate::core::material::MaterialProps;

#[derive(Clone, Debug)]
pub struct Reaction {
    pub id: u32,
    pub reactant_a: core::material::MaterialId,
    pub reactant_b: core::material::MaterialId,
    pub product: core::material::MaterialId,
    pub activation_energy: f32,
    pub pre_exponential: f32,
    pub heat_release: f32,
    pub combustion: bool,
    pub ignition_temp: f32,
}

impl Reaction {
    pub fn rate_constant(&self, temperature: f32) -> f32 {
        if temperature <= 0.0 || !temperature.is_finite() {
            return 0.0;
        }
        let r = core::units::constants::GAS_CONSTANT;
        let k = self.pre_exponential * (-(self.activation_energy / (r * temperature))).exp();
        if k.is_finite() && k >= 0.0 { k.min(1e6) } else { 0.0 }
    }
    pub fn can_proceed(&self, a: core::material::MaterialId, b: core::material::MaterialId, temp: f32) -> bool {
        let matches = (a == self.reactant_a && b == self.reactant_b)
            || (a == self.reactant_b && b == self.reactant_a);
        if !matches {
            return false;
        }
        if self.combustion && (!temp.is_finite() || temp < self.ignition_temp) {
            return false;
        }
        temp.is_finite() && temp >= self.activation_energy / 200.0
    }
}

#[derive(Clone, Debug, Default)]
pub struct ReactionRegistry {
    pub reactions: Vec<Reaction>,
}

impl ReactionRegistry {
    pub fn new() -> Self {
        Self { reactions: Vec::new() }
    }
    pub fn add(&mut self, r: Reaction) {
        self.reactions.push(r);
    }
    pub fn find(&self, a: core::material::MaterialId, b: core::material::MaterialId) -> Option<&Reaction> {
        self.reactions.iter().find(|r| {
            (r.reactant_a == a && r.reactant_b == b) || (r.reactant_a == b && r.reactant_b == a)
        })
    }
}

pub fn can_react(a: core::material::MaterialId, b: core::material::MaterialId) -> bool {
    if a == 0 || b == 0 {
        return false;
    }
    a != b
}

pub fn react_all(
    cells: &mut [Cell],
    width: u32,
    height: u32,
    materials: &[MaterialProps],
    dt: f32,
    active: Option<&crate::core::chunk_mask::ActiveMask>,
) {
    let w = width as usize;
    let h = height as usize;
    if cells.len() != w * h {
        return;
    }
    let mut processed = vec![false; cells.len()];
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if let Some(m) = active {
                if !m.cell_active(x, y) {
                    continue;
                }
            }
            if cells[i].is_empty() || processed[i] {
                continue;
            }
            let a = cells[i].material;
            let ma = material_or_default(materials, a);
            // Fire cells (plasma) ignite adjacent fuel and burn out over time.
            // Fire itself is not "flammable fuel", so it needs its own branch:
            // without this, painted fire can never spread (the combustion
            // branch below only runs for flammable materials above ignition).
            if a == 6 {
                for (dx, dy) in [(-1i32,0i32),(1,0),(0,-1),(0,1)] {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx < 0 || ny < 0 || (nx as usize) >= w || (ny as usize) >= h {
                        continue;
                    }
                    let ni = ny as usize * w + nx as usize;
                    if cells[ni].is_empty() {
                        continue;
                    }
                    let mb = material_or_default(materials, cells[ni].material);
                    if mb.flammable && mb.ignition_temp.is_finite()
                        && cells[ni].temperature.is_finite()
                        && cells[ni].temperature < mb.ignition_temp
                    {
                        cells[ni].temperature = mb.ignition_temp;
                    }
                }
                // Fire consumes itself: burn for a while, then become smoke.
                if cells[i].lifetime == 0.0 {
                    cells[i].lifetime = 8.0;
                }
                cells[i].lifetime = (cells[i].lifetime - dt).max(0.0);
                if cells[i].lifetime == 0.0 {
                    cells[i].material = 12; // smoke
                    cells[i].temperature = 400.0;
                    cells[i].vx = 0.0;
                    cells[i].vy = 0.0;
                    cells[i].flags = 0;
                    cells[i].pressure = 101325.0;
                }
                processed[i] = true;
                continue;
            }
            // Combustion and fire propagation
            if ma.flammable && cells[i].temperature.is_finite() && cells[i].temperature >= ma.ignition_temp {
                cells[i].set_flag(cell_flags::BURNING, true);
                let mass = ma.density * (0.01 * 0.01 * 0.01);
                if mass > 0.0 {
                    cells[i].temperature += (70000.0 / (mass * ma.specific_heat.max(1.0))) * dt.min(0.016);
                }
                if cells[i].lifetime == 0.0 {
                    cells[i].lifetime = 5.0;
                }
                if cells[i].lifetime > 0.0 {
                    cells[i].lifetime = (cells[i].lifetime - dt).max(0.0);
                    if cells[i].lifetime == 0.0 {
                        cells[i].material = 0;
                        cells[i].vx = 0.0;
                        cells[i].vy = 0.0;
                        cells[i].flags = 0;
                        cells[i].pressure = 101325.0;
                        processed[i] = true;
                        continue;
                    }
                }
                // Spread fire: heat adjacent fuel to its ignition point so it
                // catches fire on a subsequent tick (observable propagation).
                for (dx, dy) in [(-1i32,0i32),(1,0),(0,-1),(0,1)] {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx < 0 || ny < 0 || (nx as usize) >= w || (ny as usize) >= h {
                        continue;
                    }
                    let ni = ny as usize * w + nx as usize;
                    if cells[ni].is_empty() {
                        continue;
                    }
                    let mb = material_or_default(materials, cells[ni].material);
                    if mb.flammable && mb.ignition_temp.is_finite()
                        && cells[ni].temperature.is_finite()
                        && cells[ni].temperature < mb.ignition_temp
                    {
                        cells[ni].temperature = mb.ignition_temp;
                    }
                }
                processed[i] = true;
                continue;
            }
            // Oxidation
            if !ma.flammable && ma.state == core::material::MaterialState::Solid && cells[i].temperature.is_finite() && cells[i].temperature > 450.0 {
                cells[i].temperature += 0.05 * dt;
                processed[i] = true;
                continue;
            }
            // Dissolution and mixing
            if cells[i].has_flag(cell_flags::WET) {
                cells[i].set_flag(cell_flags::DISSOLVED, true);
                for (dx, dy) in [(-1i32,0i32),(1,0),(0,-1),(0,1)] {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx < 0 || ny < 0 || (nx as usize) >= w || (ny as usize) >= h {
                        continue;
                    }
                    let ni = ny as usize * w + nx as usize;
                    if cells[ni].is_empty() {
                        cells[ni] = cells[i];
                        cells[i] = Cell::default();
                        break;
                    }
                    if cells[ni].material != cells[i].material && cells[ni].material != 0 {
                        let t_avg = (cells[i].temperature + cells[ni].temperature) * 0.5;
                        if t_avg.is_finite() {
                            cells[i].temperature = t_avg;
                            cells[ni].temperature = t_avg;
                        }
                    }
                }
                processed[i] = true;
                continue;
            }
            processed[i] = true;
        }
    }
}

pub fn combustion(cells: &mut [Cell], materials: &[MaterialProps], dt: f32) {
    if cells.is_empty() {
        return;
    }
    let side = ((cells.len() as f32).sqrt()) as u32;
    if side > 0 {
        let h = (cells.len() as u32) / side;
        react_all(cells, side, h, materials, dt, None);
    }
}

pub fn oxidation(cells: &mut [Cell], materials: &[MaterialProps], dt: f32) {
    for cell in cells.iter_mut() {
        if cell.is_empty() {
            continue;
        }
        if !cell.temperature.is_finite() {
            cell.temperature = 293.15;
            continue;
        }
        let m = material_or_default(materials, cell.material);
        if m.flammable {
            continue;
        }
        if m.state == core::material::MaterialState::Solid && cell.temperature > 400.0 {
            cell.temperature += 0.12 * dt;
        }
    }
}

pub fn dissolution(cells: &mut [Cell], width: u32, height: u32, materials: &[MaterialProps], dt: f32) {
    react_all(cells, width, height, materials, dt, None);
}

fn material_or_default(materials: &[MaterialProps], id: core::material::MaterialId) -> MaterialProps {
    materials
        .get(id as usize)
        .cloned()
        .unwrap_or_else(|| crate::default_material_props(id))
}
