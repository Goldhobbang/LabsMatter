//! Material reaction definitions and reaction kinetics.

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
        if temperature <= 0.0 {
            return 0.0;
        }
        let r = core::units::constants::GAS_CONSTANT;
        self.pre_exponential * (-(self.activation_energy / (r * temperature))).exp()
    }
    pub fn can_proceed(&self, a: core::material::MaterialId, b: core::material::MaterialId, temp: f32) -> bool {
        let matches = (a == self.reactant_a && b == self.reactant_b)
            || (a == self.reactant_b && b == self.reactant_a);
        if !matches {
            return false;
        }
        if self.combustion && temp < self.ignition_temp {
            return false;
        }
        temp >= self.activation_energy / 100.0
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

pub fn react_all(cells: &mut [Cell], width: u32, height: u32, materials: &[MaterialProps], dt: f32) {
    let w = width as usize;
    let h = height as usize;
    if cells.len() != w * h {
        return;
    }
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if cells[i].is_empty() {
                continue;
            }
            for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || ny < 0 || (nx as usize) >= w || (ny as usize) >= h {
                    continue;
                }
                let ni = ny as usize * w + nx as usize;
                if cells[ni].is_empty() {
                    continue;
                }
                let a = cells[i].material;
                let b = cells[ni].material;
                if a == b {
                    continue;
                }
                let ma = material_or_default(materials, a);
                if ma.flammable && cells[i].temperature >= ma.ignition_temp {
                    cells[i].set_flag(cell_flags::BURNING, true);
                    cells[i].temperature += 50.0 * dt;
                    if cells[i].lifetime > 0.0 {
                        cells[i].lifetime = (cells[i].lifetime - dt).max(0.0);
                    }
                }
                if !ma.flammable && cells[i].material != 0 {
                    cells[i].temperature += 0.1 * dt;
                }
                if cells[i].has_flag(cell_flags::WET) && cells[ni].is_empty() {
                    cells[i].set_flag(cell_flags::DISSOLVED, true);
                }
            }
        }
    }
}

pub fn combustion(cells: &mut [Cell], materials: &[MaterialProps], dt: f32) {
    for cell in cells.iter_mut() {
        if cell.is_empty() {
            continue;
        }
        let m = material_or_default(materials, cell.material);
        if !m.flammable {
            continue;
        }
        if cell.temperature >= m.ignition_temp {
            cell.set_flag(cell_flags::BURNING, true);
            cell.temperature += 100.0 * dt;
            if cell.lifetime > 0.0 {
                cell.lifetime = (cell.lifetime - dt * 2.0).max(0.0);
            }
        }
        if cell.has_flag(cell_flags::BURNING) {
            cell.temperature += 20.0 * dt;
        }
    }
}

pub fn oxidation(cells: &mut [Cell], materials: &[MaterialProps], dt: f32) {
    for cell in cells.iter_mut() {
        if cell.is_empty() {
            continue;
        }
        let m = material_or_default(materials, cell.material);
        if m.flammable {
            continue;
        }
        if m.state == core::material::MaterialState::Solid {
            cell.temperature += 0.05 * dt;
        }
    }
}

pub fn dissolution(cells: &mut [Cell], width: u32, height: u32, dt: f32) {
    let w = width as usize;
    let h = height as usize;
    if cells.len() != w * h {
        return;
    }
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if cells[i].is_empty() {
                continue;
            }
            if !cells[i].has_flag(cell_flags::WET) {
                continue;
            }
            if !cells[i].has_flag(cell_flags::DISSOLVED) {
                continue;
            }
            for (dx, dy) in [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < h {
                    let ni = ny as usize * w + nx as usize;
                    if cells[ni].is_empty() {
                        cells[ni] = cells[i];
                        cells[i] = Cell::default();
                        break;
                    }
                }
            }
        }
    }
    let _ = dt;
}

fn material_or_default(materials: &[MaterialProps], id: core::material::MaterialId) -> MaterialProps {
    materials
        .get(id as usize)
        .cloned()
        .unwrap_or_else(|| crate::default_material_props(id))
}
