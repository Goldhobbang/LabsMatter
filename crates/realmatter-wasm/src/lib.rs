//! REALMATTER WASM entrypoint (A2 + lead integration).
//!
//! Implements the approved WASM/JS API (`docs/contracts/wasm-api.md`) plus the
//! minimal viable exports requested in REQ-A5-001. Owns a single global 2D
//! [`World`](realmatter_world::World) behind a mutex; JS owns no memory.
//!
//! The per-tick pipeline (`step`) follows the approved phase order from
//! `docs/contracts/simulation-tick.md` (see `realmatter-world/src/tick.rs`):
//! Chemistry → PhysicsForces → PhysicsCollisions → Movement → Thermal →
//! Finalize → Export(render). Physics/chemistry phases reuse the existing
//! `realmatter-physics` / `realmatter-chemistry` engines operating directly
//! on the world's cell slice (no duplicate simulation state).
//!
//! Exported functions (wasm-bindgen):
//! - `init(width, height) -> Result<(), JsValue>`
//! - `step(dt: f64)`
//! - `set_cell(x, y, material) -> bool`
//! - `get_cell_material(x, y) -> u16`
//! - `get_cell_temperature(x, y) -> f32`
//! - `world_width() -> u32`, `world_height() -> u32`
//! - `reset() -> bool` (clear all cells)
//! - `render_buffer_ptr() -> *const u8`, `render_buffer_len() -> usize`
//!   (RGBA8 per cell, row-major, y outer / x inner; 4 bytes per cell)
//! - `save_blob() -> Vec<u8>` / `load_blob(bytes: &[u8]) -> bool`
//!   (persist format v1; errors reported as `false`, details in console)
//!
//! Coordinates are world cell coordinates (integer). Y increases downward,
//! matching the viewport and the cellular-automaton fall direction.
//! `material` is core MaterialId (u16); 0 = empty/void.

use std::sync::{Arc, Mutex, OnceLock};

use realmatter_core::cell::Cell;
use realmatter_core::coord::CellCoord;
use realmatter_core::material::{MaterialId, MaterialProps, MaterialRegistry, MaterialState};
use realmatter_core::world_config::WorldConfig;
use realmatter_world::World;
use wasm_bindgen::prelude::*;

#[derive(Debug)]
struct StaticRegistry {
    props: Vec<MaterialProps>,
}

impl StaticRegistry {
    fn builtin() -> Self {
        let mut props = Vec::new();
        props.push(MaterialProps {
            id: 0,
            name: "empty".to_string(),
            state: MaterialState::Gas,
            density: 0.0,
            specific_heat: 1005.0,
            thermal_conductivity: 0.026,
            melting_point: f32::INFINITY,
            boiling_point: f32::INFINITY,
            viscosity: 0.0,
            color: [0.0, 0.0, 0.0, 0.0],
            flammable: false,
            ignition_temp: f32::INFINITY,
        });
        props.push(MaterialProps {
            id: 1,
            name: "wall".to_string(),
            state: MaterialState::Solid,
            density: 2500.0,
            specific_heat: 840.0,
            thermal_conductivity: 1.0,
            melting_point: f32::INFINITY,
            boiling_point: f32::INFINITY,
            viscosity: 0.0,
            color: [0.5, 0.5, 0.5, 1.0],
            flammable: false,
            ignition_temp: f32::INFINITY,
        });
        props.push(MaterialProps {
            id: 2,
            name: "sand".to_string(),
            state: MaterialState::Powder,
            density: 1600.0,
            specific_heat: 830.0,
            thermal_conductivity: 0.3,
            melting_point: 1700.0,
            boiling_point: 2200.0,
            viscosity: 0.0,
            color: [0.85, 0.72, 0.45, 1.0],
            flammable: false,
            ignition_temp: f32::INFINITY,
        });
        props.push(MaterialProps {
            id: 3,
            name: "water".to_string(),
            state: MaterialState::Liquid,
            density: 1000.0,
            specific_heat: 4184.0,
            thermal_conductivity: 0.6,
            melting_point: 273.15,
            boiling_point: 373.15,
            viscosity: 0.001,
            color: [0.31, 0.77, 0.97, 1.0],
            flammable: false,
            ignition_temp: f32::INFINITY,
        });
        // Extended builtin materials mirror web/src/materials.js ids so the
        // palette paints real, correctly colored cells. Powders/liquids move
        // in step(); solids are FIXED; gases/plasma are static (diffusion and
        // combustion live in the physics/chemistry engines, not yet wired
        // into the WASM tick — see module docs).
        props.push(MaterialProps {
            id: 4,
            name: "oil".to_string(),
            state: MaterialState::Liquid,
            density: 900.0,
            specific_heat: 2000.0,
            thermal_conductivity: 0.15,
            melting_point: 250.0,
            boiling_point: 573.0,
            viscosity: 0.05,
            color: [0.55, 0.43, 0.39, 1.0],
            flammable: true,
            ignition_temp: 520.0,
        });
        props.push(MaterialProps {
            id: 5,
            name: "steam".to_string(),
            state: MaterialState::Gas,
            density: 0.6,
            specific_heat: 2080.0,
            thermal_conductivity: 0.025,
            melting_point: f32::INFINITY,
            boiling_point: f32::INFINITY,
            viscosity: 0.0,
            color: [0.69, 0.74, 0.77, 0.9],
            flammable: false,
            ignition_temp: f32::INFINITY,
        });
        props.push(MaterialProps {
            id: 6,
            name: "fire".to_string(),
            state: MaterialState::Plasma,
            density: 0.3,
            specific_heat: 1000.0,
            thermal_conductivity: 0.1,
            melting_point: f32::INFINITY,
            boiling_point: f32::INFINITY,
            viscosity: 0.0,
            color: [1.0, 0.44, 0.0, 1.0],
            flammable: false,
            ignition_temp: f32::INFINITY,
        });
        props.push(MaterialProps {
            id: 7,
            name: "wood".to_string(),
            state: MaterialState::Solid,
            density: 700.0,
            specific_heat: 1700.0,
            thermal_conductivity: 0.12,
            melting_point: f32::INFINITY,
            boiling_point: f32::INFINITY,
            viscosity: 0.0,
            color: [0.43, 0.30, 0.26, 1.0],
            flammable: true,
            ignition_temp: 560.0,
        });
        props.push(MaterialProps {
            id: 8,
            name: "iron".to_string(),
            state: MaterialState::Solid,
            density: 7874.0,
            specific_heat: 450.0,
            thermal_conductivity: 80.0,
            melting_point: 1811.0,
            boiling_point: 3134.0,
            viscosity: 0.0,
            color: [0.56, 0.64, 0.68, 1.0],
            flammable: false,
            ignition_temp: f32::INFINITY,
        });
        props.push(MaterialProps {
            id: 9,
            name: "lava".to_string(),
            state: MaterialState::Liquid,
            density: 3100.0,
            specific_heat: 1000.0,
            thermal_conductivity: 1.5,
            melting_point: 973.0,
            boiling_point: f32::INFINITY,
            viscosity: 0.1,
            color: [1.0, 0.24, 0.0, 1.0],
            flammable: false,
            ignition_temp: f32::INFINITY,
        });
        props.push(MaterialProps {
            id: 10,
            name: "acid".to_string(),
            state: MaterialState::Liquid,
            density: 1050.0,
            specific_heat: 3500.0,
            thermal_conductivity: 0.5,
            melting_point: 240.0,
            boiling_point: 360.0,
            viscosity: 0.002,
            color: [0.46, 1.0, 0.01, 1.0],
            flammable: false,
            ignition_temp: f32::INFINITY,
        });
        props.push(MaterialProps {
            id: 11,
            name: "salt".to_string(),
            state: MaterialState::Powder,
            density: 2160.0,
            specific_heat: 880.0,
            thermal_conductivity: 0.5,
            melting_point: 1074.0,
            boiling_point: 1686.0,
            viscosity: 0.0,
            color: [0.93, 0.93, 0.93, 1.0],
            flammable: false,
            ignition_temp: f32::INFINITY,
        });
        props.push(MaterialProps {
            id: 12,
            name: "smoke".to_string(),
            state: MaterialState::Gas,
            density: 0.4,
            specific_heat: 1000.0,
            thermal_conductivity: 0.02,
            melting_point: f32::INFINITY,
            boiling_point: f32::INFINITY,
            viscosity: 0.0,
            color: [0.38, 0.38, 0.38, 0.9],
            flammable: false,
            ignition_temp: f32::INFINITY,
        });
        props.push(MaterialProps {
            id: 13,
            name: "ice".to_string(),
            state: MaterialState::Solid,
            density: 917.0,
            specific_heat: 2100.0,
            thermal_conductivity: 2.2,
            melting_point: 273.15,
            boiling_point: f32::INFINITY,
            viscosity: 0.0,
            color: [0.70, 0.90, 0.99, 1.0],
            flammable: false,
            ignition_temp: f32::INFINITY,
        });
        props.push(MaterialProps {
            id: 14,
            name: "glass".to_string(),
            state: MaterialState::Solid,
            density: 2500.0,
            specific_heat: 840.0,
            thermal_conductivity: 1.0,
            melting_point: 1673.0,
            boiling_point: f32::INFINITY,
            viscosity: 0.0,
            color: [0.50, 0.87, 0.92, 1.0],
            flammable: false,
            ignition_temp: f32::INFINITY,
        });
        props.push(MaterialProps {
            id: 15,
            name: "gunpowder".to_string(),
            state: MaterialState::Powder,
            density: 1000.0,
            specific_heat: 1000.0,
            thermal_conductivity: 0.2,
            melting_point: f32::INFINITY,
            boiling_point: f32::INFINITY,
            viscosity: 0.0,
            color: [0.26, 0.26, 0.26, 1.0],
            flammable: true,
            ignition_temp: 500.0,
        });
        Self { props }
    }
}

impl MaterialRegistry for StaticRegistry {
    fn get(&self, id: MaterialId) -> Option<&MaterialProps> {
        self.props.iter().find(|p| p.id == id)
    }
    fn len(&self) -> usize {
        self.props.len()
    }
}

/// Material table indexed by material id, for the physics/chemistry slice
/// APIs (`materials.get(id as usize)`). Gaps are filled with air-like
/// defaults so unknown ids never panic. Cached in a `OnceLock`: the builtin
/// set is immutable, so sharing it across ticks is safe and avoids a
/// per-tick allocation + 16 clones.
fn material_table() -> &'static Vec<MaterialProps> {
    use std::sync::OnceLock;
    static TABLE: OnceLock<Vec<MaterialProps>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let builtin = StaticRegistry::builtin();
        let max_id = builtin.props.iter().map(|p| p.id as usize).max().unwrap_or(0);
        let mut table = vec![
            MaterialProps {
                id: 0,
                name: "air".to_string(),
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
            };
            max_id + 1
        ];
        for p in builtin.props {
            let i = p.id as usize;
            if i < table.len() {
                table[i] = p;
            }
        }
        table
    })
}

struct SimState {
    world: World,
    render: Vec<u8>,
    /// Reused chemistry engine (registry + material table built once per
    /// world, not per tick). Dimensions follow the world; callers must
    /// refresh it after `init`/`load_blob` replace the world.
    chemistry: realmatter_chemistry::ChemistryEngine,
    /// 0 material, 1 fancy, 2 heat, 3 pressure, 4 velocity.
    view: u8,
    /// 0 down, 1 off, 2 toward center.
    gravity: u8,
    /// 0 on, 1 off, 2 pressure only (gases do not move).
    air: u8,
    /// 0 void (border deleted), 1 solid, 2 wrap.
    edge: u8,
    newton: bool,
    ambient: bool,
}

fn make_chemistry(world: &World) -> realmatter_chemistry::ChemistryEngine {
    let mut chem =
        realmatter_chemistry::ChemistryEngine::from_config(world.config());
    chem.set_materials(material_table().clone());
    chem
}

fn global() -> &'static Mutex<Option<SimState>> {
    static CELL: OnceLock<Mutex<Option<SimState>>> = OnceLock::new();
    CELL.get_or_init(|| Mutex::new(None))
}

fn with_state<T>(f: impl FnOnce(&mut SimState) -> T, fallback: T) -> T {
    let lock = global();
    let mut guard = match lock.lock() {
        Ok(g) => g,
        Err(_) => return fallback,
    };
    match guard.as_mut() {
        Some(s) => f(s),
        None => fallback,
    }
}

fn rebuild_render(state: &mut SimState) {
    let w = state.world.width() as usize;
    let h = state.world.height() as usize;
    state.render.resize(w * h * 4, 0);
    for y in 0..h {
        for x in 0..w {
            let cell = state.world.cell(x as i32, y as i32).cloned().unwrap_or_default();
            let (r, g, b, a) = rgba_for(&cell, x as i32, y as i32, state.view);
            let i = (y * w + x) * 4;
            state.render[i] = r;
            state.render[i + 1] = g;
            state.render[i + 2] = b;
            state.render[i + 3] = a;
        }
    }
}

fn rgba_for(cell: &Cell, x: i32, y: i32, mode: u8) -> (u8, u8, u8, u8) {
    if cell.material == 0 {
        return (0, 0, 0, 0);
    }
    let (mut r, mut g, mut b, a) = match cell.material {
        0 => (0, 0, 0, 0),
        1 => (128, 128, 128, 255),
        2 => (217, 194, 122, 255),
        3 => (79, 195, 247, 220),
        4 => (141, 110, 99, 230),
        5 => (176, 190, 197, 140),
        6 => (255, 111, 0, 255),
        7 => (109, 76, 65, 255),
        8 => (144, 164, 174, 255),
        9 => (255, 61, 0, 255),
        10 => (118, 255, 3, 220),
        11 => (238, 238, 238, 255),
        12 => (97, 97, 97, 120),
        13 => (179, 229, 252, 230),
        14 => (128, 222, 234, 180),
        15 => (66, 66, 66, 255),
        16 => (255, 240, 80, 255),
        _ => (255, 0, 255, 255),
    };
    if cell.material != 0 && matches!(cell.material, 2 | 3 | 4 | 10 | 11 | 15) {
        let n = ((x.wrapping_mul(13).wrapping_add(y.wrapping_mul(7))) & 15) as i32 - 8;
        r = add_u8(r, n);
        g = add_u8(g, n);
        b = add_u8(b, n);
    }
    match mode {
        2 => return heat_rgba(cell.temperature),
        3 => {
            let k = ((cell.pressure - 90_000.0) / 40_000.0).clamp(0.0, 1.0);
            return (
                (40.0 + k * 200.0) as u8,
                (80.0 + (0.5 - (k - 0.5).abs()) * 120.0) as u8,
                (200.0 - k * 160.0) as u8,
                255,
            );
        }
        4 => {
            let s = (cell.vx * cell.vx + cell.vy * cell.vy).sqrt();
            let k = (s / 8.0).clamp(0.0, 1.0);
            return ((40.0 + k * 215.0) as u8, (180.0 - k * 100.0) as u8, 80, 255);
        }
        1 if cell.temperature > 500.0 => {
            let hot = heat_rgba(cell.temperature);
            let t = ((cell.temperature - 500.0) / 1000.0).clamp(0.0, 0.75);
            r = ((r as f32) * (1.0 - t) + (hot.0 as f32) * t) as u8;
            g = ((g as f32) * (1.0 - t) + (hot.1 as f32) * t) as u8;
            b = ((b as f32) * (1.0 - t) + (hot.2 as f32) * t) as u8;
        }
        _ => {}
    }
    (r, g, b, a)
}

fn add_u8(c: u8, d: i32) -> u8 {
    (c as i32 + d).clamp(0, 255) as u8
}

fn heat_rgba(t: f32) -> (u8, u8, u8, u8) {
    let k = ((t - 200.0) / 1600.0).clamp(0.0, 1.0);
    let r = (k * 255.0) as u8;
    let g = ((k * 2.2 - 0.7).clamp(0.0, 1.0) * 255.0) as u8;
    let b = ((1.0 - k).clamp(0.0, 1.0) * 200.0) as u8;
    (r, g, b, 255)
}

/// Initialize the simulation. Called once from JS.
#[wasm_bindgen]
pub fn init(width: u32, height: u32) -> Result<(), JsValue> {
    let mut config = WorldConfig::default();
    if width > 0 {
        config.width = width;
    }
    if height > 0 {
        config.height = height;
    }
    config
        .validate()
        .map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
    let registry: Arc<dyn MaterialRegistry> = Arc::new(StaticRegistry::builtin());
    let world = World::new(config, registry).map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
    let chemistry = make_chemistry(&world);
    let mut state = SimState {
        world,
        render: Vec::new(),
        chemistry,
        view: 0,
        gravity: 0,
        air: 0,
        edge: 1,
        newton: false,
        ambient: false,
    };
    rebuild_render(&mut state);
    match global().lock() {
        Ok(mut g) => {
            *g = Some(state);
            Ok(())
        }
        Err(_) => Err(JsValue::from_str("sim lock poisoned")),
    }
}

/// Advance simulation by one fixed step.
///
/// Deterministic pipeline (approved tick phase order):
/// 1. Chemistry (registry reactions: combustion / oxidation / dissolution)
/// 2. Physics forces (gas pressure + diffusion + buoyancy, fluid viscosity +
///    pressure + buoyancy, heat conduction, phase transitions, transfers)
/// 3. Movement (powder + liquid cellular automaton)
/// 4. Finalize (UPDATED flags, lifetimes, tick count) + render refresh.
///
/// Operates directly on the world's cell slice; no duplicate state.
/// `_dt` is accepted for API compatibility; the fixed step comes from
/// `WorldConfig::fixed_dt` (1/60 s).
#[wasm_bindgen]
pub fn step(_dt: f64) {
    use realmatter_core::chunk_mask::ActiveMask;
    use realmatter_core::traits::ChemistryModule;
    use realmatter_physics::{fluid, gas, heat, reaction, stability, transfer};
    with_state(
        |state| {
            let dt = state.world.config().fixed_dt as f32;
            let w = state.world.width();
            let h = state.world.height();
            let config = state.world.config().clone();
            let materials: &Vec<MaterialProps> = material_table();
            // Active-chunk mask: non-empty chunks + 8-neighbors. Rebuilt every
            // tick so regions wake up as soon as a cell is written into them.
            // Masked phases skip only empty source cells (proven no-ops), so
            // results are bit-identical to the unmasked pipeline.
            let mask = ActiveMask::build_from_cells(
                state.world.cells(),
                w,
                h,
                config.chunk_size,
            );
            let gravity = state.gravity;
            let air = state.air;
            let edge = state.edge;
            let newton = state.newton;
            let ambient = state.ambient;
            let active = Some(&mask);
            let cells = state.world.cells_mut();

            // 0. Numerical guards first (NaN/Inf sanitize).
            stability::sanitize(cells, materials);

            // 1. Chemistry: registry-driven reactions (engine reused across
            // ticks; no per-tick allocation).
            {
                let _ = state.chemistry.react_masked(cells, dt as f64, active);
            }

            // 2. Physics forces.
            gas::update_pressure(cells, materials);
            // Air off: no gas transport. Pressure-only: keep the field, skip motion.
            if air == 0 {
                gas::diffuse(cells, w, h, materials, dt, active);
                gas::apply_buoyancy(
                    cells,
                    w,
                    h,
                    materials,
                    dt,
                    realmatter_core::units::constants::GRAVITY,
                    active,
                );
            }
            fluid::apply_viscosity(cells, materials, dt);
            fluid::apply_pressure_gradient(cells, w, h, dt, active);
            fluid::apply_buoyancy(
                cells,
                w,
                h,
                materials,
                dt,
                realmatter_core::units::constants::GRAVITY,
                active,
            );
            heat::conduct(cells, w, h, materials, dt, active);
            if ambient {
                for cell in cells.iter_mut() {
                    if cell.is_empty() || !cell.temperature.is_finite() {
                        continue;
                    }
                    cell.temperature += (295.15 - cell.temperature) * 0.01;
                }
            }
            heat::phase_transitions(cells, materials);
            reaction::react_all(cells, w, h, materials, dt, active);
            transfer::transfer_energy(cells, w, h, materials, dt, active);
            transfer::transfer_mass(cells, w, h, dt, active);
            stability::clamp_velocities(cells, stability::budgets::MAX_VELOCITY);
            stability::sanitize(cells, materials);

            // 3. Movement (powder + liquid cellular automaton).
            // Use the registry table. A hardcoded match had grouped gunpowder
            // with oil at 900 kg/m³ while the material is 1000 kg/m³.
            let state_of = |id: MaterialId| -> Option<MaterialState> {
                materials.get(id as usize).map(|m| m.state)
            };
            let density_of = |id: MaterialId| -> f32 {
                materials.get(id as usize).map(|m| m.density).unwrap_or(0.0)
            };
            match gravity {
                1 => {
                    realmatter_world::move_liquid_flat(cells, w, h, &state_of, active);
                }
                2 => {
                    realmatter_world::move_toward_center(cells, w, h, &state_of, active);
                }
                _ => {
                    realmatter_world::move_powder_step(cells, w, h, &state_of, &density_of, active);
                    realmatter_world::move_liquid_step(cells, w, h, &state_of, &density_of, active);
                }
            }
            if newton {
                apply_newtonian(cells, w as usize, h as usize, &state_of);
            }
            step_spark(cells, w as usize, h as usize);
            match edge {
                0 => clear_border(cells, w as usize, h as usize),
                2 => wrap_border(cells, w as usize, h as usize, &state_of),
                _ => {}
            }

            // 4. Finalize + render refresh.
            state.world.finalize_tick();
            rebuild_render(state);
        },
        (),
    );
}

fn step_spark(cells: &mut [Cell], w: usize, h: usize) {
    for cell in cells.iter_mut() {
        if cell.material == 8 && cell.lifetime > 0.0 {
            cell.lifetime = (cell.lifetime - 1.0).max(0.0);
        }
    }
    let mut sparks = Vec::new();
    for (i, cell) in cells.iter().enumerate() {
        if cell.material == 16 {
            sparks.push(i);
        }
    }
    if sparks.is_empty() {
        return;
    }
    let mut next = Vec::new();
    for i in sparks {
        let x = i % w;
        let y = i / w;
        for (dx, dy) in [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)] {
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if nx < 0 || ny < 0 || nx >= w as isize || ny >= h as isize {
                continue;
            }
            let ni = ny as usize * w + nx as usize;
            // Fresh iron only. A conductor that just stopped sparking keeps a
            // short lifetime so the pulse does not bounce back on the next frame.
            if cells[ni].material == 8 && cells[ni].lifetime <= 0.0 {
                next.push(ni);
            }
        }
        cells[i].material = 8;
        cells[i].temperature = 293.15;
        cells[i].lifetime = 2.0;
        cells[i].set_flag(realmatter_core::cell::cell_flags::FIXED, true);
    }
    for ni in next {
        if cells[ni].material == 8 {
            cells[ni].material = 16;
            cells[ni].temperature = 900.0;
            cells[ni].set_flag(realmatter_core::cell::cell_flags::FIXED, true);
        }
    }
}

fn apply_newtonian(cells: &mut [Cell], w: usize, h: usize, state_of: &dyn Fn(MaterialId) -> Option<MaterialState>) {
    let mut pulls: Vec<(usize, i32, i32)> = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if !matches!(state_of(cells[i].material), Some(MaterialState::Powder)) {
                continue;
            }
            let mut sx = 0i32;
            let mut sy = 0i32;
            for dy in -4i32..=4 {
                for dx in -4i32..=4 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                        continue;
                    }
                    let ni = ny as usize * w + nx as usize;
                    if cells[ni].is_empty() {
                        continue;
                    }
                    sx += dx.signum();
                    sy += dy.signum();
                }
            }
            if sx != 0 || sy != 0 {
                pulls.push((i, sx.signum(), sy.signum()));
            }
        }
    }
    for (i, dx, dy) in pulls {
        let x = i % w;
        let y = i / w;
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;
        if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
            continue;
        }
        let j = ny as usize * w + nx as usize;
        if cells[j].is_empty() && !cells[i].is_empty() {
            cells.swap(i, j);
        }
    }
}

fn clear_border(cells: &mut [Cell], w: usize, h: usize) {
    if w == 0 || h == 0 {
        return;
    }
    for x in 0..w {
        cells[x] = Cell::default();
        cells[(h - 1) * w + x] = Cell::default();
    }
    for y in 0..h {
        cells[y * w] = Cell::default();
        cells[y * w + (w - 1)] = Cell::default();
    }
}

fn wrap_border(cells: &mut [Cell], w: usize, h: usize, state_of: &dyn Fn(MaterialId) -> Option<MaterialState>) {
    if w < 2 || h < 2 {
        return;
    }
    let mobile = |id| {
        matches!(
            state_of(id),
            Some(MaterialState::Powder | MaterialState::Liquid | MaterialState::Gas)
        )
    };
    for x in 1..w - 1 {
        let bot = (h - 1) * w + x;
        let top = x;
        if mobile(cells[bot].material) && cells[top].is_empty() {
            cells.swap(bot, top);
        }
    }
    for y in 1..h - 1 {
        let right = y * w + (w - 1);
        let left = y * w;
        if mobile(cells[right].material) && cells[left].is_empty() {
            cells.swap(right, left);
        }
    }
}

fn cycle(cur: u8, n: u8) -> u8 {
    (cur + 1) % n
}

/// Gravity: 0 down, 1 off, 2 toward center.
#[wasm_bindgen]
pub fn set_gravity_mode(mode: u32) {
    with_state(|s| s.gravity = (mode as u8) % 3, ());
}

/// Air: 0 on, 1 off, 2 pressure only.
#[wasm_bindgen]
pub fn set_air_mode(mode: u32) {
    with_state(|s| s.air = (mode as u8) % 3, ());
}

/// Edge: 0 void, 1 solid, 2 wrap.
#[wasm_bindgen]
pub fn set_edge_mode(mode: u32) {
    with_state(|s| s.edge = (mode as u8) % 3, ());
}

#[wasm_bindgen]
pub fn set_newtonian(on: u32) {
    with_state(|s| s.newton = on != 0, ());
}

#[wasm_bindgen]
pub fn set_ambient_heat(on: u32) {
    with_state(|s| s.ambient = on != 0, ());
}

#[wasm_bindgen]
pub fn cycle_gravity() {
    with_state(|s| s.gravity = cycle(s.gravity, 3), ());
}
#[wasm_bindgen]
pub fn cycle_air() {
    with_state(|s| s.air = cycle(s.air, 3), ());
}
#[wasm_bindgen]
pub fn cycle_edge() {
    with_state(|s| s.edge = cycle(s.edge, 3), ());
}
#[wasm_bindgen]
pub fn toggle_newtonian() {
    with_state(|s| s.newton = !s.newton, ());
}
#[wasm_bindgen]
pub fn toggle_ambient() {
    with_state(|s| s.ambient = !s.ambient, ());
}

/// Compact HUD string: gravity, air, edge, newton, ambient.
#[wasm_bindgen]
pub fn sim_status() -> String {
    with_state(
        |s| {
            format!(
                "grav {} air {} edge {} n{} u{}",
                ["down", "off", "radial"][s.gravity as usize],
                ["on", "off", "press"][s.air as usize],
                ["void", "solid", "wrap"][s.edge as usize],
                if s.newton { "+" } else { "-" },
                if s.ambient { "+" } else { "-" },
            )
        },
        "sim?".into(),
    )
}

/// Set a cell's material. Returns true on success.
///
/// New cells start at the material's characteristic temperature so thermal
/// interactions are observable: lava/fire/steam/smoke paint hot, ice paints
/// cold, everything else paints at ambient (293.15 K). Temperatures come
/// from the builtin material properties (boiling/melting points), not from
/// hardcoded magic numbers.
#[wasm_bindgen]
pub fn set_cell(x: i32, y: i32, material: u16) -> bool {
    with_state(
        |state| {
            let mut cell = Cell::default();
            cell.material = material;
            cell.temperature = initial_temperature(material);
            // Solids are structures. Only wall used to be flagged, so sand and
            // water density-swapped through wood, ice, and glass.
            let solid = material_table()
                .get(material as usize)
                .is_some_and(|m| m.state == MaterialState::Solid);
            if solid || material == 16 {
                cell.set_flag(realmatter_core::cell::cell_flags::FIXED, true);
            }
            match state.world.set(CellCoord::new(x, y), cell) {
                Ok(()) => {
                    // Incremental render update: repaint only this pixel.
                    // Full rebuilds happen on init/step/reset/load, so per-cell
                    // painting (brush strokes) stays O(1) instead of O(W*H).
                    let w = state.world.width() as usize;
                    let h = state.world.height() as usize;
                    if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h
                        && state.render.len() == w * h * 4
                    {
                        let (r, g, b, a) = rgba_for(&cell, x, y, state.view);
                        let i = (y as usize * w + x as usize) * 4;
                        state.render[i] = r;
                        state.render[i + 1] = g;
                        state.render[i + 2] = b;
                        state.render[i + 3] = a;
                    } else {
                        rebuild_render(state);
                    }
                    true
                }
                Err(_) => false,
            }
        },
        false,
    )
}

/// Characteristic paint temperature for a builtin material id.
fn initial_temperature(material: u16) -> f32 {
    match material {
        // lava: above iron's melting point so it visibly heats neighbors.
        9 => 1500.0,
        // fire: above all ignition temps so it ignites adjacent fuel.
        6 => 1200.0,
        // steam / smoke: hot vapor.
        5 => 373.15,
        12 => 400.0,
        // ice: below water's melting point.
        13 => 263.15,
        16 => 900.0,
        _ => 293.15,
    }
}

/// 0 material, 1 fancy (heat tint), 2 heat, 3 pressure, 4 velocity.
/// Colors come from the live cell fields, not from invented values.
#[wasm_bindgen]
pub fn set_view_mode(mode: u32) {
    with_state(
        |state| {
            state.view = mode.min(4) as u8;
            rebuild_render(state);
        },
        (),
    );
}

/// Get a cell's material id (0 if out of bounds / uninitialized).
#[wasm_bindgen]
pub fn get_cell_material(x: i32, y: i32) -> u16 {
    with_state(
        |state| state.world.cell(x, y).map(|c| c.material).unwrap_or(0),
        0,
    )
}

/// Pressure in pascals (0 if out of bounds).
#[wasm_bindgen]
pub fn get_cell_pressure(x: i32, y: i32) -> f32 {
    with_state(
        |state| state.world.cell(x, y).map(|c| c.pressure).unwrap_or(0.0),
        0.0,
    )
}

/// Horizontal velocity, m/s (0 if out of bounds).
#[wasm_bindgen]
pub fn get_cell_vx(x: i32, y: i32) -> f32 {
    with_state(|state| state.world.cell(x, y).map(|c| c.vx).unwrap_or(0.0), 0.0)
}

/// Vertical velocity, m/s (0 if out of bounds). Positive is downward.
#[wasm_bindgen]
pub fn get_cell_vy(x: i32, y: i32) -> f32 {
    with_state(|state| state.world.cell(x, y).map(|c| c.vy).unwrap_or(0.0), 0.0)
}

/// Get a cell's temperature (Kelvin; 0.0 if out of bounds / uninitialized).
#[wasm_bindgen]
pub fn get_cell_temperature(x: i32, y: i32) -> f32 {
    with_state(
        |state| state.world.cell(x, y).map(|c| c.temperature).unwrap_or(0.0),
        0.0,
    )
}

/// World width in cells (0 if uninitialized).
#[wasm_bindgen]
pub fn world_width() -> u32 {
    with_state(|state| state.world.width(), 0)
}

/// World height in cells (0 if uninitialized).
#[wasm_bindgen]
pub fn world_height() -> u32 {
    with_state(|state| state.world.height(), 0)
}

/// Simulation tick counter (0 if uninitialized). Restored on load.
#[wasm_bindgen]
pub fn tick_count() -> u64 {
    with_state(|state| state.world.tick_count(), 0)
}

/// Clear all cells and reset the tick counter. Returns false if uninitialized.
#[wasm_bindgen]
pub fn reset() -> bool {
    with_state(
        |state| {
            let w = state.world.width() as i32;
            let h = state.world.height() as i32;
            for y in 0..h {
                for x in 0..w {
                    let _ = state.world.set(CellCoord::new(x, y), Cell::default());
                }
            }
            state.world.set_tick_count(0);
            state.world.set_time_accumulator(0.0);
            rebuild_render(state);
            true
        },
        false,
    )
}

/// Pointer to the RGBA8 render buffer (row-major, y outer / x inner).
#[wasm_bindgen]
pub fn render_buffer_ptr() -> *const u8 {
    with_state(
        |state| state.render.as_ptr(),
        std::ptr::null(),
    )
}

/// Length of the render buffer in bytes.
#[wasm_bindgen]
pub fn render_buffer_len() -> usize {
    with_state(|state| state.render.len(), 0)
}

/// Serialize the world to a persist-format blob (empty vec if uninitialized).
#[wasm_bindgen]
pub fn save_blob() -> Vec<u8> {
    with_state(
        |state| {
            let registry: Arc<dyn MaterialRegistry> = Arc::new(StaticRegistry::builtin());
            let mut engine = realmatter_persist::PersistEngine::new(registry);
            engine.save_full(&mut state.world, None).unwrap_or_default()
        },
        Vec::new(),
    )
}

/// Load a persist-format blob into the world. Returns false on failure.
#[wasm_bindgen]
pub fn load_blob(bytes: &[u8]) -> bool {
    with_state(
        |state| {
            let registry: Arc<dyn MaterialRegistry> = Arc::new(StaticRegistry::builtin());
            let engine = realmatter_persist::PersistEngine::new(registry);
            match engine.load_blob(bytes) {
                Ok(world) => {
                    state.world = world;
                    // World dimensions may have changed; refresh the reused
                    // chemistry engine so its width/height stay in sync.
                    state.chemistry =
                        realmatter_chemistry::ChemistryEngine::from_config(
                            state.world.config(),
                        );
                    state.chemistry.set_materials(material_table().clone());
                    rebuild_render(state);
                    true
                }
                Err(_) => false,
            }
        },
        false,
    )
}
