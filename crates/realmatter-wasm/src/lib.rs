//! REALMATTER WASM entrypoint (A2).
//!
//! Implements the approved WASM/JS API (`docs/contracts/wasm-api.md`) plus the
//! minimal viable exports requested in REQ-A5-001. Owns a single global 2D
//! [`World`](realmatter_world::World) behind a mutex; JS owns no memory.
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
//! Coordinates are world cell coordinates (integer, Y-up per architecture).
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

struct SimState {
    world: World,
    render: Vec<u8>,
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
            let (r, g, b, a) = rgba_for(&cell);
            let i = (y * w + x) * 4;
            state.render[i] = r;
            state.render[i + 1] = g;
            state.render[i + 2] = b;
            state.render[i + 3] = a;
        }
    }
}

fn rgba_for(cell: &Cell) -> (u8, u8, u8, u8) {
    match cell.material {
        0 => (0, 0, 0, 0),
        1 => (128, 128, 128, 255),
        2 => (217, 194, 122, 255),
        3 => (79, 195, 247, 255),
        4 => (141, 110, 99, 255),
        5 => (176, 190, 197, 230),
        6 => (255, 111, 0, 255),
        7 => (109, 76, 65, 255),
        8 => (144, 164, 174, 255),
        9 => (255, 61, 0, 255),
        10 => (118, 255, 3, 255),
        11 => (238, 238, 238, 255),
        12 => (97, 97, 97, 230),
        13 => (179, 229, 252, 255),
        14 => (128, 222, 234, 255),
        15 => (66, 66, 66, 255),
        _ => (255, 0, 255, 255),
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
    let mut state = SimState { world, render: Vec::new() };
    rebuild_render(&mut state);
    match global().lock() {
        Ok(mut g) => {
            *g = Some(state);
            Ok(())
        }
        Err(_) => Err(JsValue::from_str("sim lock poisoned")),
    }
}

/// Advance simulation by one fixed step (deterministic movement + finalize).
#[wasm_bindgen]
pub fn step(_dt: f64) {
    with_state(
        |state| {
            let w = state.world.width();
            let h = state.world.height();
            let cells = state.world.cells_mut();
            let state_of = |id: MaterialId| -> Option<MaterialState> {
                match id {
                    0 | 5 | 12 => Some(MaterialState::Gas),
                    1 | 7 | 8 | 13 | 14 => Some(MaterialState::Solid),
                    2 | 11 | 15 => Some(MaterialState::Powder),
                    3 | 4 | 9 | 10 => Some(MaterialState::Liquid),
                    6 => Some(MaterialState::Plasma),
                    _ => None,
                }
            };
            let density_of = |id: MaterialId| -> f32 {
                match id {
                    1 | 14 => 2500.0,
                    2 => 1600.0,
                    3 => 1000.0,
                    4 | 15 => 900.0,
                    5 => 0.6,
                    6 => 0.3,
                    7 => 700.0,
                    8 => 7874.0,
                    9 => 3100.0,
                    10 => 1050.0,
                    11 => 2160.0,
                    12 => 0.4,
                    13 => 917.0,
                    _ => 0.0,
                }
            };
            realmatter_world::move_powder_step(cells, w, h, &state_of, &density_of);
            realmatter_world::move_liquid_step(cells, w, h, &state_of, &density_of);
            state.world.finalize_tick();
            rebuild_render(state);
        },
        (),
    );
}

/// Set a cell's material. Returns true on success.
#[wasm_bindgen]
pub fn set_cell(x: i32, y: i32, material: u16) -> bool {
    with_state(
        |state| {
            let mut cell = Cell::default();
            cell.material = material;
            if material == 1 {
                cell.set_flag(realmatter_core::cell::cell_flags::FIXED, true);
            }
            match state.world.set(CellCoord::new(x, y), cell) {
                Ok(()) => {
                    rebuild_render(state);
                    true
                }
                Err(_) => false,
            }
        },
        false,
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

/// Clear all cells. Returns false if uninitialized.
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
                    rebuild_render(state);
                    true
                }
                Err(_) => false,
            }
        },
        false,
    )
}
