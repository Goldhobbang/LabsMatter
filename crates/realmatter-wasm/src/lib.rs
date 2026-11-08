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

fn mat(
    id: u16,
    name: &str,
    state: MaterialState,
    density: f32,
    specific_heat: f32,
    thermal_conductivity: f32,
    melting_point: f32,
    flammable: bool,
    ignition_temp: f32,
) -> MaterialProps {
    MaterialProps {
        id,
        name: name.to_string(),
        state,
        density,
        specific_heat,
        thermal_conductivity,
        melting_point,
        boiling_point: f32::INFINITY,
        viscosity: 0.0,
        color: [1.0, 1.0, 1.0, 1.0],
        flammable,
        ignition_temp,
    }
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
        props.push(mat(
            17,
            "snow",
            MaterialState::Powder,
            300.0,
            2100.0,
            0.15,
            273.15,
            false,
            f32::INFINITY,
        ));
        props.push(mat(
            18,
            "copper",
            MaterialState::Solid,
            8960.0,
            385.0,
            400.0,
            f32::INFINITY,
            false,
            f32::INFINITY,
        ));
        props.push(mat(
            19,
            "nitro",
            MaterialState::Liquid,
            850.0,
            1700.0,
            0.15,
            250.0,
            true,
            420.0,
        ));
        props.push(mat(
            20,
            "hydrogen",
            MaterialState::Gas,
            0.09,
            14300.0,
            0.18,
            f32::INFINITY,
            true,
            773.0,
        ));
        props.push(mat(
            21,
            "co2",
            MaterialState::Gas,
            1.8,
            840.0,
            0.016,
            f32::INFINITY,
            false,
            f32::INFINITY,
        ));
        props.push(mat(
            22,
            "plant",
            MaterialState::Solid,
            400.0,
            1700.0,
            0.1,
            f32::INFINITY,
            true,
            500.0,
        ));
        props.push(mat(
            23,
            "gold",
            MaterialState::Powder,
            19300.0,
            129.0,
            300.0,
            1337.0,
            false,
            f32::INFINITY,
        ));
        props.push(mat(
            24,
            "dust",
            MaterialState::Powder,
            200.0,
            1000.0,
            0.05,
            f32::INFINITY,
            true,
            480.0,
        ));
        props.push(mat(
            25,
            "bomb",
            MaterialState::Solid,
            1500.0,
            1000.0,
            0.2,
            f32::INFINITY,
            false,
            f32::INFINITY,
        ));
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
        let max_id = builtin
            .props
            .iter()
            .map(|p| p.id as usize)
            .max()
            .unwrap_or(0);
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
    /// Fire buffer and persistent frame. Display-only; never read by physics.
    scratch: realmatter_renderer::RenderScratch,
    particles: u32,
    last_error: String,
    /// Milliseconds spent in the last full framebuffer compose.
    render_ms: f32,
    /// Reused chemistry engine (registry + material table built once per
    /// world, not per tick). Dimensions follow the world; callers must
    /// refresh it after `init`/`load_blob` replace the world.
    chemistry: realmatter_chemistry::ChemistryEngine,
    /// See [`realmatter_renderer::view_name`]. 0–9.
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
    let mut chem = realmatter_chemistry::ChemistryEngine::from_config(world.config());
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

fn now_ms() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or_else(|| js_sys::Date::now())
}

fn rebuild_render(state: &mut SimState) {
    let started = now_ms();
    let w = state.world.width() as usize;
    let h = state.world.height() as usize;
    let view = state.view;
    let gravity = state.gravity;
    let ambient = state.ambient;
    let mut scratch = std::mem::take(&mut state.scratch);
    let mut render = std::mem::take(&mut state.render);
    let count = realmatter_renderer::render(
        state.world.cells(),
        w,
        h,
        view,
        gravity,
        ambient,
        &mut scratch,
        &mut render,
    );
    state.scratch = scratch;
    state.render = render;
    state.particles = count;
    state.render_ms = (now_ms() - started) as f32;
}

fn paint_or_rebuild(state: &mut SimState, x: i32, y: i32, cell: &Cell) {
    let w = state.world.width() as usize;
    let h = state.world.height() as usize;
    if realmatter_renderer::mode_needs_full(state.view) || state.render.len() != w * h * 4 {
        rebuild_render(state);
        return;
    }
    realmatter_renderer::paint_cell(
        &mut state.render,
        w,
        h,
        x,
        y,
        cell,
        state.view,
        state.ambient,
    );
}

fn finite(v: f32, fallback: f32) -> f32 {
    if v.is_finite() {
        v
    } else {
        fallback
    }
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
        scratch: realmatter_renderer::RenderScratch::default(),
        particles: 0,
        last_error: String::new(),
        render_ms: 0.0,
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
            let mask = ActiveMask::build_from_cells(state.world.cells(), w, h, config.chunk_size);
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
            step_special(cells, w as usize, h as usize);
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
                apply_newtonian(cells, w as usize, h as usize, &state_of, &density_of);
            }
            if air == 0 {
                apply_air_advection(cells, w as usize, h as usize, &state_of);
                settle_heavy_gas(cells, w as usize, h as usize);
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
            if is_conductor(cells[ni].material) && cells[ni].lifetime <= 0.0 {
                next.push(ni);
            }
        }
        let back = if cells[i].vx > 17.0 { 18 } else { 8 };
        cells[i].material = back;
        cells[i].temperature = 293.15;
        cells[i].lifetime = 2.0;
        cells[i].vx = 0.0;
        cells[i].set_flag(realmatter_core::cell::cell_flags::FIXED, true);
    }
    for ni in next {
        if is_conductor(cells[ni].material) {
            cells[ni].vx = cells[ni].material as f32;
            cells[ni].material = 16;
            cells[ni].temperature = 900.0;
            cells[ni].set_flag(realmatter_core::cell::cell_flags::FIXED, true);
        }
    }
}

fn apply_air_advection(
    cells: &mut [Cell],
    w: usize,
    h: usize,
    state_of: &dyn Fn(MaterialId) -> Option<MaterialState>,
) {
    let mut planned = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let kind = state_of(cells[i].material);
            if !matches!(kind, Some(MaterialState::Powder | MaterialState::Liquid)) {
                continue;
            }
            let mut vx = 0.0f32;
            let mut vy = 0.0f32;
            let mut n = 0i32;
            for dy in -3i32..=3 {
                for dx in -3i32..=3 {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                        continue;
                    }
                    let ni = ny as usize * w + nx as usize;
                    // Material 0 is empty, but the registry types it as gas.
                    // Counting those cells made open air look like a rising
                    // current, so dust and the other powders were lifted one
                    // cell every frame and never reached the ground.
                    if cells[ni].is_empty() {
                        continue;
                    }
                    if matches!(state_of(cells[ni].material), Some(MaterialState::Gas)) {
                        vx += cells[ni].vx;
                        vy += cells[ni].vy;
                        n += 1;
                    }
                }
            }
            if n == 0 {
                continue;
            }
            vx /= n as f32;
            vy /= n as f32;
            let mx = if vx.abs() > 0.02 {
                vx.signum() as i32
            } else {
                0
            };
            // Powder and liquid already fall one cell this frame. An upward
            // air step of one cell cancels that fall, so only a strong upward
            // current (an explosion, not a warm plume) may lift them.
            let my = if vy < -2.0 {
                -1
            } else if vy > 0.02 {
                1
            } else {
                0
            };
            if mx != 0 || my != 0 {
                planned.push((i, mx, my));
            }
        }
    }
    for (i, dx, dy) in planned {
        let x = (i % w) as i32;
        let y = (i / w) as i32;
        let nx = x + dx;
        let ny = y + dy;
        if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
            continue;
        }
        let j = ny as usize * w + nx as usize;
        let dest_gas = matches!(state_of(cells[j].material), Some(MaterialState::Gas));
        if (cells[j].is_empty() || dest_gas) && !cells[i].is_empty() {
            cells.swap(i, j);
        }
    }
}

fn settle_heavy_gas(cells: &mut [Cell], w: usize, h: usize) {
    if h < 2 {
        return;
    }
    // Gas diffusion also steps CO2 upward. Two downward passes leave a net sink.
    for _ in 0..2 {
        for y in (0..h - 1).rev() {
            for x in 0..w {
                let i = y * w + x;
                let j = (y + 1) * w + x;
                if cells[i].material == 21 && cells[j].is_empty() {
                    cells.swap(i, j);
                }
            }
        }
    }
}

fn step_special(cells: &mut [Cell], w: usize, h: usize) {
    let mut grow = Vec::new();
    let mut burst = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            match cells[i].material {
                17 if cells[i].temperature >= 273.15 => {
                    cells[i].material = 3;
                    cells[i].temperature = 273.15;
                    cells[i].set_flag(realmatter_core::cell::cell_flags::FIXED, false);
                }
                22 => {
                    for (dx, dy) in [(-1i32, 0), (1, 0), (0, -1), (0, 1)] {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                            continue;
                        }
                        let ni = ny as usize * w + nx as usize;
                        if cells[ni].material == 3 {
                            grow.push(ni);
                            break;
                        }
                    }
                }
                25 if cells[i].temperature >= 600.0 => burst.push((x, y)),
                _ => {}
            }
        }
    }
    for ni in grow {
        if cells[ni].material == 3 {
            cells[ni].material = 22;
            cells[ni].temperature = 293.15;
            cells[ni].set_flag(realmatter_core::cell::cell_flags::FIXED, true);
        }
    }
    for (x, y) in burst {
        for dy in -3i32..=3 {
            for dx in -3i32..=3 {
                if dx * dx + dy * dy > 10 {
                    continue;
                }
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                    continue;
                }
                let ni = ny as usize * w + nx as usize;
                if cells[ni].material != 1 {
                    cells[ni].material = 6;
                    cells[ni].temperature = 1200.0;
                    cells[ni].lifetime = 0.6;
                    cells[ni].set_flag(realmatter_core::cell::cell_flags::FIXED, false);
                }
            }
        }
    }
}

fn is_conductor(id: u16) -> bool {
    id == 8 || id == 18
}

fn apply_newtonian(
    cells: &mut [Cell],
    w: usize,
    h: usize,
    state_of: &dyn Fn(MaterialId) -> Option<MaterialState>,
    density_of: &dyn Fn(MaterialId) -> f32,
) {
    let mut pulls: Vec<(usize, i32, i32)> = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if !matches!(state_of(cells[i].material), Some(MaterialState::Powder)) {
                continue;
            }
            let mut sx = 0.0f32;
            let mut sy = 0.0f32;
            for dy in -8i32..=8 {
                for dx in -8i32..=8 {
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
                    let dist2 = (dx * dx + dy * dy) as f32;
                    let mass = density_of(cells[ni].material).max(1.0);
                    sx += dx as f32 * mass / dist2;
                    sy += dy as f32 * mass / dist2;
                }
            }
            if sx.abs() > 1.0 || sy.abs() > 1.0 {
                pulls.push((i, sx.signum() as i32, sy.signum() as i32));
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

fn wrap_border(
    cells: &mut [Cell],
    w: usize,
    h: usize,
    state_of: &dyn Fn(MaterialId) -> Option<MaterialState>,
) {
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
            let was_empty = state.world.cell(x, y).is_none_or(|c| c.is_empty());
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
                    if was_empty && material != 0 {
                        state.particles = state.particles.saturating_add(1);
                    } else if !was_empty && material == 0 {
                        state.particles = state.particles.saturating_sub(1);
                    }
                    // One pixel, except modes whose colour depends on neighbours
                    // or on the persistent buffer.
                    paint_or_rebuild(state, x, y, &cell);
                    true
                }
                Err(_) => false,
            }
        },
        false,
    )
}

/// Restore or stamp a cell without resetting temperature, velocity, flags,
/// lifetime, or pressure. Spark stores its conductor id in `vx`; copying only
/// the material id would turn that spark into a fresh one. Non-finite numbers
/// are replaced so a bad stamp cannot poison the solver.
#[wasm_bindgen]
pub fn set_cell_full(
    x: i32,
    y: i32,
    material: u16,
    temperature: f32,
    vx: f32,
    vy: f32,
    flags: u8,
    lifetime: f32,
    pressure: f32,
) -> bool {
    with_state(
        |state| {
            let was_empty = state.world.cell(x, y).is_none_or(|c| c.is_empty());
            let cell = Cell {
                material,
                temperature: finite(temperature, initial_temperature(material)),
                vx: finite(vx, 0.0),
                vy: finite(vy, 0.0),
                flags,
                lifetime: finite(lifetime, 0.0),
                pressure: finite(pressure, 0.0),
            };
            match state.world.set(CellCoord::new(x, y), cell) {
                Ok(()) => {
                    let now_empty = material == 0;
                    if was_empty && !now_empty {
                        state.particles = state.particles.saturating_add(1);
                    } else if !was_empty && now_empty {
                        state.particles = state.particles.saturating_sub(1);
                    }
                    paint_or_rebuild(state, x, y, &cell);
                    true
                }
                Err(_) => false,
            }
        },
        false,
    )
}

/// `[material, temperature, vx, vy, flags, lifetime, pressure]`.
/// Empty / out of bounds is all zeros. There is no particle index to copy.
#[wasm_bindgen]
pub fn cell_state(x: i32, y: i32) -> Vec<f32> {
    with_state(
        |state| {
            state
                .world
                .cell(x, y)
                .map(pack_cell)
                .unwrap_or_else(|| vec![0.0; 7])
        },
        vec![0.0; 7],
    )
}

fn pack_cell(cell: &Cell) -> Vec<f32> {
    vec![
        cell.material as f32,
        cell.temperature,
        cell.vx,
        cell.vy,
        cell.flags as f32,
        cell.lifetime,
        cell.pressure,
    ]
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
        17 => 260.0,
        _ => 293.15,
    }
}

/// 0 Normal, 1 Fancy, 2 Heat, 3 Pressure, 4 Velocity,
/// 5 Air velocity, 6 Air pressure, 7 Air heat, 8 Persistent, 9 Gravity.
#[wasm_bindgen]
pub fn set_view_mode(mode: u32) {
    with_state(
        |state| {
            state.view = mode.min(realmatter_renderer::DISPLAY_MODES as u32 - 1) as u8;
            rebuild_render(state);
        },
        (),
    );
}

/// Occupied cells from the last full compose, adjusted by paints.
#[wasm_bindgen]
pub fn particle_count() -> u32 {
    with_state(|state| state.particles, 0)
}

/// Last full framebuffer compose, in milliseconds. Not a browser frame time.
#[wasm_bindgen]
pub fn render_millis() -> f32 {
    with_state(|state| state.render_ms, 0.0)
}

/// Active WASM module. The page must not substitute another engine for this.
#[wasm_bindgen]
pub fn module_id() -> String {
    "realmatter_wasm".to_string()
}

/// Last save/load failure. Empty after a successful load.
#[wasm_bindgen]
pub fn last_error() -> String {
    with_state(
        |state| state.last_error.clone(),
        "simulation is not initialized".into(),
    )
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
    with_state(
        |state| state.world.cell(x, y).map(|c| c.vx).unwrap_or(0.0),
        0.0,
    )
}

/// Vertical velocity, m/s (0 if out of bounds). Positive is downward.
#[wasm_bindgen]
pub fn get_cell_vy(x: i32, y: i32) -> f32 {
    with_state(
        |state| state.world.cell(x, y).map(|c| c.vy).unwrap_or(0.0),
        0.0,
    )
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
            state.scratch = realmatter_renderer::RenderScratch::default();
            state.last_error.clear();
            rebuild_render(state);
            true
        },
        false,
    )
}

/// Pointer to the RGBA8 render buffer (row-major, y outer / x inner).
#[wasm_bindgen]
pub fn render_buffer_ptr() -> *const u8 {
    with_state(|state| state.render.as_ptr(), std::ptr::null())
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
                        realmatter_chemistry::ChemistryEngine::from_config(state.world.config());
                    state.chemistry.set_materials(material_table().clone());
                    state.scratch = realmatter_renderer::RenderScratch::default();
                    state.last_error.clear();
                    rebuild_render(state);
                    true
                }
                Err(err) => {
                    state.last_error = format!("load failed: {err:?}");
                    false
                }
            }
        },
        false,
    )
}

#[cfg(test)]
mod air_tests {
    use super::apply_air_advection;
    use realmatter_core::cell::Cell;
    use realmatter_core::material::MaterialState;

    fn states(id: u16) -> Option<MaterialState> {
        match id {
            0 | 12 => Some(MaterialState::Gas),
            24 | 2 => Some(MaterialState::Powder),
            _ => None,
        }
    }

    #[test]
    fn open_air_does_not_lift_dust() {
        let (w, h) = (9usize, 9usize);
        let mut cells = vec![Cell::default(); w * h];
        let i = 4 * w + 4;
        cells[i].material = 24;
        apply_air_advection(&mut cells, w, h, &states);
        assert_eq!(cells[i].material, 24);
        assert_eq!(cells.iter().filter(|c| c.material == 24).count(), 1);
    }

    #[test]
    fn dust_still_falls_after_air_step() {
        let (w, h) = (5usize, 6usize);
        let mut cells = vec![Cell::default(); w * h];
        cells[1 * w + 2].material = 24;
        let state_of = |id| states(id);
        let density_of = |_id| 200.0f32;
        realmatter_world::move_powder_step(
            &mut cells,
            w as u32,
            h as u32,
            &state_of,
            &density_of,
            None,
        );
        apply_air_advection(&mut cells, w, h, &states);
        assert_eq!(
            cells[2 * w + 2].material,
            24,
            "dust should be one row lower"
        );
    }

    #[test]
    fn sideways_gas_blows_dust_without_lifting_it() {
        let (w, h) = (7usize, 7usize);
        let mut cells = vec![Cell::default(); w * h];
        cells[3 * w + 3].material = 24;
        cells[3 * w + 4].material = 12;
        cells[3 * w + 4].vx = 1.0;
        cells[2 * w + 3].material = 12;
        cells[2 * w + 3].vy = -0.4;
        apply_air_advection(&mut cells, w, h, &states);
        assert_eq!(
            cells[3 * w + 4].material,
            24,
            "dust slides with the sideways current"
        );
        assert_eq!(
            cells[2 * w + 3].material,
            12,
            "a mild plume does not lift dust"
        );
    }
}
