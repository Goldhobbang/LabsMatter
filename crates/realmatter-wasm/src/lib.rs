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
        Self {
            props: realmatter_core::tpt_elements::builtin_materials(),
        }
    }
}

impl MaterialRegistry for StaticRegistry {
    fn get(&self, id: MaterialId) -> Option<&MaterialProps> {
        self.props.iter().find(|p| p.id == id)
    }
    fn len(&self) -> usize {
        self.props.len()
    }
    fn all_props(&self) -> Vec<MaterialProps> {
        self.props.clone()
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
                air_loss: 1.0,
                air_drag: 0.0,
                advection: 0.0,
                loss: 1.0,
                diffusion: 0.0,
                hot_air: 0.0,
                conducts: false,
                weight: 50.0,
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
    /// Persistent air velocity/pressure field (reference `Air`).
    air_field: realmatter_physics::air::AirField,
    /// Newtonian gravity field per air cell (force x, y).
    grav_field: Vec<(f32, f32)>,
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
    /// Convection mode: 0 none, 1 legacy, 2 boussinesq.
    convection: u8,
    /// Decoration layer: one ARGB colour per cell, 0 = none.
    deco: Vec<u32>,
    /// Decoration layer visibility toggle.
    deco_visible: bool,
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
    if state.deco.len() != w * h {
        state.deco.resize(w * h, 0);
    }
    for y in 0..h {
        for x in 0..w {
            let cell = state.world.cell(x as i32, y as i32).cloned().unwrap_or_default();
            let (mut r, mut g, mut b, mut a) =
                rgba_for(&cell, x as i32, y as i32, state.view, Some(&state.air_field));
            if state.deco_visible {
                let d = state.deco[y * w + x];
                if d != 0 {
                    let (dr, dg, db, da) = argb_rgba(d);
                    let t = da as f32 / 255.0;
                    r = blend(r, dr, t);
                    g = blend(g, dg, t);
                    b = blend(b, db, t);
                    a = 255;
                }
            }
            let i = (y * w + x) * 4;
            state.render[i] = r;
            state.render[i + 1] = g;
            state.render[i + 2] = b;
            state.render[i + 3] = a;
        }
    }
}

/// Repaint a single pixel (used by per-cell painting and decoration edits).
fn repaint_pixel(state: &mut SimState, x: usize, y: usize) {
    let w = state.world.width() as usize;
    let h = state.world.height() as usize;
    if x >= w || y >= h || state.render.len() != w * h * 4 {
        rebuild_render(state);
        return;
    }
    let cell = state.world.cell(x as i32, y as i32).cloned().unwrap_or_default();
    let (mut r, mut g, mut b, mut a) =
        rgba_for(&cell, x as i32, y as i32, state.view, Some(&state.air_field));
    if state.deco_visible && state.deco.len() == w * h {
        let d = state.deco[y * w + x];
        if d != 0 {
            let (dr, dg, db, da) = argb_rgba(d);
            let t = da as f32 / 255.0;
            r = blend(r, dr, t);
            g = blend(g, dg, t);
            b = blend(b, db, t);
            a = 255;
        }
    }
    let i = (y * w + x) * 4;
    state.render[i] = r;
    state.render[i + 1] = g;
    state.render[i + 2] = b;
    state.render[i + 3] = a;
}

/// ARGB u32 → RGBA8 tuple.
fn argb_rgba(argb: u32) -> (u8, u8, u8, u8) {
    (
        ((argb >> 16) & 0xFF) as u8,
        ((argb >> 8) & 0xFF) as u8,
        (argb & 0xFF) as u8,
        ((argb >> 24) & 0xFF) as u8,
    )
}

fn blend(base: u8, over: u8, t: f32) -> u8 {
    (base as f32 * (1.0 - t) + over as f32 * t) as u8
}

fn rgba_for(
    cell: &Cell,
    x: i32,
    y: i32,
    mode: u8,
    air_field: Option<&realmatter_physics::air::AirField>,
) -> (u8, u8, u8, u8) {
    if cell.material == 0 && !matches!(mode, 3 | 4) {
        return (0, 0, 0, 0);
    }
    // Colour comes from the registry (reference element colours), so every
    // registered material renders correctly without a hardcoded table.
    let (mut r, mut g, mut b, a) = match material_table().get(cell.material as usize) {
        Some(def) => (
            (def.color[0] * 255.0) as u8,
            (def.color[1] * 255.0) as u8,
            (def.color[2] * 255.0) as u8,
            (def.color[3] * 255.0) as u8,
        ),
        None => (255, 0, 255, 255),
    };
    // Per-particle colour variation for granular/liquid materials, matching
    // the reference's per-particle shade noise.
    if matches!(
        material_table()
            .get(cell.material as usize)
            .map(|m| m.state),
        Some(MaterialState::Powder | MaterialState::Liquid)
    ) {
        let n = ((x.wrapping_mul(13).wrapping_add(y.wrapping_mul(7))) & 15) as i32 - 8;
        r = add_u8(r, n);
        g = add_u8(g, n);
        b = add_u8(b, n);
    }
    match mode {
        2 => return heat_rgba(cell.temperature),
        3 => {
            // Pressure view reads the air field, not per-particle pressure.
            let p = air_field
                .map(|f| f.pv_at(x, y))
                .unwrap_or(cell.pressure);
            let k = ((p + 8.0) / 16.0).clamp(0.0, 1.0);
            return (
                (40.0 + k * 200.0) as u8,
                (80.0 + (0.5 - (k - 0.5).abs()) * 120.0) as u8,
                (200.0 - k * 160.0) as u8,
                255,
            );
        }
        4 => {
            // Velocity view reads the air field.
            let (vx, vy) = air_field
                .map(|f| (f.vx_at(x, y), f.vy_at(x, y)))
                .unwrap_or((cell.vx, cell.vy));
            let s = (vx * vx + vy * vy).sqrt();
            let k = (s / 4.0).clamp(0.0, 1.0);
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
    let (ww, wh) = (world.width() as usize, world.height() as usize);
    let chemistry = make_chemistry(&world);
    let mut state = SimState {
        world,
        render: Vec::new(),
        chemistry,
        air_field: realmatter_physics::air::AirField::new(ww, wh),
        grav_field: Vec::new(),
        view: 0,
        gravity: 0,
        air: 0,
        edge: 1,
        newton: false,
        ambient: false,
        convection: 2,
        deco: vec![0; ww * wh],
        deco_visible: true,
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
            // Air field: persistent velocity + pressure, updated every tick
            // (reference `Air::update_air`). Air modes gate what the field
            // produces, not whether it exists.
            let air_mode = realmatter_physics::air::AirMode::from_sim_mode(air as u32);
            state
                .air_field
                .rebuild_blockair(cells, w as usize, h as usize);
            state.air_field.update(air_mode);
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
                state.air_field.convection_mode =
                    realmatter_physics::air::ConvectionMode::from_u32(state.convection as u32);
                state.air_field.update_ambient_heat(
                    realmatter_core::units::constants::GRAVITY,
                );
                for y in 0..h as usize {
                    for x in 0..w as usize {
                        let i = y * w as usize + x;
                        if cells[i].is_empty() || !cells[i].temperature.is_finite() {
                            continue;
                        }
                        let target = state.air_field.hv_at(x as i32, y as i32);
                        cells[i].temperature += (target - cells[i].temperature) * 0.01;
                    }
                }
            }
            heat::phase_transitions(cells, materials);
            reaction::react_all(cells, w, h, materials, dt, active);
            transfer::transfer_energy(cells, w, h, materials, dt, active);
            transfer::transfer_mass(cells, w, h, dt, active);
            stability::clamp_velocities(cells, stability::budgets::MAX_VELOCITY);
            stability::sanitize(cells, materials);

            // 2b. Air coupling: particles read the field (advection) and write
            // to it (momentum, hot air). Deterministic pseudo-diffusion uses a
            // fixed sequence so runs stay reproducible.
            {
                let mut rng_state = 0x2545_F491_4F6C_DD1Du64;
                let mut next_uniform = move || {
                    rng_state ^= rng_state << 13;
                    rng_state ^= rng_state >> 7;
                    rng_state ^= rng_state << 17;
                    ((rng_state >> 11) as f32) / ((1u64 << 53) as f32)
                };
                realmatter_physics::air::apply_air_to_particles(
                    cells,
                    w as usize,
                    h as usize,
                    materials,
                    &mut state.air_field,
                    air_mode,
                    &mut next_uniform,
                );
            }

            // 3. Movement (powder + liquid cellular automaton).
            // Use the registry table. A hardcoded match had grouped gunpowder
            // with oil at 900 kg/m³ while the material is 1000 kg/m³.
            let state_of = |id: MaterialId| -> Option<MaterialState> {
                materials.get(id as usize).map(|m| m.state)
            };
            let density_of = |id: MaterialId| -> f32 {
                materials.get(id as usize).map(|m| m.density).unwrap_or(0.0)
            };
            // Airflow-driven displacement happens before the CA so wind can
            // push powders and liquids around.
            if air_mode != realmatter_physics::air::AirMode::Off
                && air_mode != realmatter_physics::air::AirMode::VelocityOff
            {
                realmatter_physics::air::integrate_velocity(
                    cells,
                    w as usize,
                    h as usize,
                    materials,
                    active,
                );
                realmatter_physics::air::advect_gases(
                    cells,
                    w as usize,
                    h as usize,
                    materials,
                    &state.air_field,
                    air_mode,
                    active,
                );
            }
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
            let mut grav_force: Vec<(f32, f32)> = Vec::new();
            if newton {
                let force = realmatter_physics::gravity::compute_field(cells, w as usize, h as usize, materials);
                realmatter_physics::gravity::apply_field(cells, w as usize, h as usize, materials, &force);
                grav_force = force;
            }
            step_spark(cells, w as usize, h as usize, materials);
            match edge {
                0 => clear_border(cells, w as usize, h as usize),
                2 => wrap_border(cells, w as usize, h as usize, &state_of),
                _ => {}
            }

            // 4. Finalize + render refresh.
            state.grav_field = grav_force;
            state.world.finalize_tick();
            rebuild_render(state);
        },
        (),
    );
}

/// Electrical conduction, generalised from the QA-tested spark-on-iron rule.
///
/// Reference model (`SPRK.cpp`): a spark lives 4 frames on the conductor it is
/// travelling through (`ctype` records which), then the conductor reverts. A
/// spark electrifies neighbouring conductive elements that are not currently
/// sparking and have cooled down (`life == 0`). Insulators never conduct.
///
/// LabsMatter mapping: `SPRK` (id 16) is the spark; the carrier id is split
/// across its otherwise-unused velocity components, `lifetime` counts down
/// the spark, and `CHARGED` prevents backtracking for one tick.
fn step_spark(cells: &mut [Cell], w: usize, h: usize, materials: &[MaterialProps]) {
    let conducts = |id: u16| -> bool {
        materials
            .get(id as usize)
            .is_some_and(|m| m.conducts)
    };
    let cooling: Vec<bool> = cells
        .iter()
        .map(|cell| cell.has_flag(realmatter_core::cell::cell_flags::CHARGED))
        .collect();
    for cell in cells.iter_mut() {
        if cell.material == 16 && cell.lifetime > 0.0 {
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
        for (i, was_cooling) in cooling.into_iter().enumerate() {
            if was_cooling && conducts(cells[i].material) {
                cells[i].set_flag(realmatter_core::cell::cell_flags::CHARGED, false);
            }
        }
        return;
    }
    let mut next: Vec<(usize, u16)> = Vec::new();
    for i in sparks {
        let x = i % w;
        let y = i / w;
        if cells[i].lifetime <= 0.0 {
            // Revert to the conductor the spark was travelling through.
            let carrier = cells[i].vx.round() as u16 + cells[i].vy.round() as u16 * 50;
            let back = if conducts(carrier) { carrier } else { 8 };
            cells[i].material = back;
            cells[i].lifetime = 0.0;
            cells[i].vx = 0.0;
            cells[i].vy = 0.0;
            let fixed = materials
                .get(back as usize)
                .is_some_and(|m| m.state == MaterialState::Solid);
            cells[i].set_flag(realmatter_core::cell::cell_flags::FIXED, fixed);
            cells[i].set_flag(realmatter_core::cell::cell_flags::CHARGED, true);
            continue;
        }
        for (dx, dy) in [(-1isize, 0isize), (1, 0), (0, -1), (0, 1)] {
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if nx < 0 || ny < 0 || nx >= w as isize || ny >= h as isize {
                continue;
            }
            let ni = ny as usize * w + nx as usize;
            let cell = &cells[ni];
            // Fresh conductor only: not sparking, cooldown elapsed.
            if conducts(cell.material)
                && cell.material != 16
                && cell.lifetime <= 0.0
                && !cooling[ni]
            {
                next.push((ni, cell.material));
            }
        }
    }
    for (ni, material) in next {
        cells[ni].material = 16;
        cells[ni].lifetime = 4.0;
        cells[ni].vx = (material % 50) as f32;
        cells[ni].vy = (material / 50) as f32;
        cells[ni].set_flag(realmatter_core::cell::cell_flags::CHARGED, false);
        cells[ni].set_flag(realmatter_core::cell::cell_flags::FIXED, true);
    }
    for (i, was_cooling) in cooling.into_iter().enumerate() {
        if was_cooling && conducts(cells[i].material) {
            cells[i].set_flag(realmatter_core::cell::cell_flags::CHARGED, false);
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
pub fn set_convection_mode(mode: u32) {
    with_state(|s| s.convection = (mode as u8) % 3, ());
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
            if material == 16 {
                cell.lifetime = 4.0;
                cell.vx = 8.0;
                cell.vy = 0.0;
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
                        repaint_pixel(state, x as usize, y as usize);
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

/// Simulation tools, mirroring the reference simtools ordering.
/// 0 heat, 1 cool, 2 air (pressure), 3 vac, 4 PGRV, 5 NGRV, 6 mix, 7 cyclone,
/// 8 wind (uses dx/dy as the drag direction).
#[wasm_bindgen]
pub fn apply_tool(kind: u32, x: i32, y: i32, radius: u32, dx: f32, dy: f32) {
    use realmatter_physics::air::AirMode;
    with_state(
        |state| {
            let r = radius.max(1) as i32;
            let mode = realmatter_physics::air::AirMode::from_sim_mode(state.air as u32);
            if mode == AirMode::NoUpdate {
                return;
            }
            let w = state.world.width() as usize;
            let h = state.world.height() as usize;
            for oy in -r..=r {
                for ox in -r..=r {
                    if ox * ox + oy * oy > r * r {
                        continue;
                    }
                    let px = x + ox;
                    let py = y + oy;
                    if px < 0 || py < 0 || px >= w as i32 || py >= h as i32 {
                        continue;
                    }
                    match kind {
                        0 => heat_cell(state, px, py, 20.0),
                        1 => heat_cell(state, px, py, -20.0),
                        2 => state.air_field.add_pressure(px, py, 0.6),
                        3 => state.air_field.add_pressure(px, py, -0.6),
                        4 => state.air_field.add_pressure(px, py, 0.05),
                        5 => state.air_field.add_pressure(px, py, -0.05),
                        6 => mix_air_cell(state, px, py),
                        7 => cyclone_cell(state, px, py, dx, dy),
                        8 => state
                            .air_field
                            .add_velocity(px, py, dx * 0.02, dy * 0.02),
                        9 => {
                            state.air_field.ambient_air_temp =
                                (state.air_field.ambient_air_temp - 5.0).max(0.0);
                        }
                        10 => {
                            state.air_field.ambient_air_temp =
                                (state.air_field.ambient_air_temp + 5.0).min(9999.0);
                        }
                        _ => {}
                    }
                }
            }
            rebuild_render(state);
        },
        (),
    );
}

fn heat_cell(state: &mut SimState, x: i32, y: i32, delta: f32) {
    if let Some(cell) = state.world.cell_mut(x, y) {
        if cell.temperature.is_finite() {
            cell.temperature = (cell.temperature + delta).clamp(0.0, 9999.0);
        }
    }
}

fn mix_air_cell(state: &mut SimState, x: i32, y: i32) {
    let cx = x / 4;
    let cy = y / 4;
    let (fcx, fcy) = (state.air_field.cells_x, state.air_field.cells_y);
    if cx <= 0 || cy <= 0 || cx >= fcx as i32 - 1 || cy >= fcy as i32 - 1 {
        return;
    }
    let idx = cy as usize * fcx + cx as usize;
    let mut dvx = 0.0;
    let mut dvy = 0.0;
    for (dx, dy, wt) in [(-1i32, 0i32, 0.25), (1, 0, 0.25), (0, -1, 0.25), (0, 1, 0.25)] {
        let ni = (cy + dy) as usize * fcx + (cx + dx) as usize;
        dvx += state.air_field.vx[ni] * wt;
        dvy += state.air_field.vy[ni] * wt;
    }
    state.air_field.vx[idx] = state.air_field.vx[idx] * 0.5 + dvx;
    state.air_field.vy[idx] = state.air_field.vy[idx] * 0.5 + dvy;
}

fn cyclone_cell(state: &mut SimState, x: i32, y: i32, dx: f32, dy: f32) {
    let cx = x / 4;
    let cy = y / 4;
    let (fcx, fcy) = (state.air_field.cells_x, state.air_field.cells_y);
    if cx <= 0 || cy <= 0 || cx >= fcx as i32 - 1 || cy >= fcy as i32 - 1 {
        return;
    }
    let idx = cy as usize * fcx + cx as usize;
    let (mut ux, mut uy) = (dx, dy);
    if ux == 0.0 && uy == 0.0 {
        ux = 1.0;
    }
    let len = (ux * ux + uy * uy).sqrt();
    ux /= len;
    uy /= len;
    state.air_field.vx[idx] += -uy * 0.5 + ux * 0.2;
    state.air_field.vy[idx] += ux * 0.5 + uy * 0.2;
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

// ---- Decoration layer ----------------------------------------------------

/// Paint a decoration colour (ARGB) on a cell. `argb == 0` erases.
#[wasm_bindgen]
pub fn set_deco(x: i32, y: i32, argb: u32) -> bool {
    with_state(
        |state| {
            let w = state.world.width() as usize;
            let h = state.world.height() as usize;
            if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
                return false;
            }
            let i = y as usize * w + x as usize;
            if state.deco.len() != w * h {
                state.deco.resize(w * h, 0);
            }
            state.deco[i] = argb;
            repaint_pixel(state, x as usize, y as usize);
            true
        },
        false,
    )
}

#[wasm_bindgen]
pub fn get_deco(x: i32, y: i32) -> u32 {
    with_state(
        |state| {
            let w = state.world.width() as usize;
            if x < 0 || y < 0 || x >= w as i32 || y >= state.world.height() as i32 {
                return 0;
            }
            state.deco.get(y as usize * w + x as usize).copied().unwrap_or(0)
        },
        0,
    )
}

/// Show or hide the decoration layer without erasing it.
#[wasm_bindgen]
pub fn set_deco_visible(visible: bool) {
    with_state(
        |state| {
            state.deco_visible = visible;
            rebuild_render(state);
        },
        (),
    );
}

#[wasm_bindgen]
pub fn deco_visible() -> bool {
    with_state(|state| state.deco_visible, false)
}

/// Erase every decoration colour.
#[wasm_bindgen]
pub fn clear_all_deco() {
    with_state(
        |state| {
            state.deco.iter_mut().for_each(|c| *c = 0);
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

/// Air pressure at a pixel, from the persistent air field (0 if out of bounds).
#[wasm_bindgen]
pub fn get_cell_pressure(x: i32, y: i32) -> f32 {
    with_state(|state| state.air_field.pv_at(x, y), 0.0)
}

/// Air velocity x at a pixel, from the persistent air field.
#[wasm_bindgen]
pub fn get_cell_vx(x: i32, y: i32) -> f32 {
    with_state(|state| state.air_field.vx_at(x, y), 0.0)
}

/// Air velocity y at a pixel, from the persistent air field. Positive is downward.
#[wasm_bindgen]
pub fn get_cell_vy(x: i32, y: i32) -> f32 {
    with_state(|state| state.air_field.vy_at(x, y), 0.0)
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
            state.air_field.clear();
            state.grav_field.clear();
            state.deco.fill(0);
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
            let deco = if state.deco.iter().any(|c| *c != 0) {
                Some(state.deco.clone())
            } else {
                None
            };
            engine
                .save_full_with_deco(&mut state.world, None, deco)
                .unwrap_or_default()
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
                    // Decoration layer: v1 saves have none, v2 saves carry one.
                    let (w, h) = (
                        state.world.width() as usize,
                        state.world.height() as usize,
                    );
                    state.deco = vec![0; w * h];
                    if let Ok(raw) = realmatter_persist::format::decompress_blob(bytes) {
                        if let Ok(save) = realmatter_persist::format::decode(&raw) {
                            if let Some(deco) = save.deco {
                                if deco.len() == w * h {
                                    state.deco = deco;
                                }
                            }
                        }
                    }
                    state.air_field =
                        realmatter_physics::air::AirField::new(w, h);
                    state.grav_field.clear();
                    rebuild_render(state);
                    true
                }
                Err(_) => false,
            }
        },
        false,
    )
}
