//! REALMATTER WASM entrypoint.
//!
//! Scaffolded by A1. Internal logic is owned by respective agents; this crate
//! wires modules together and exposes the JS API.

use wasm_bindgen::prelude::*;

/// Initialize the simulation. Called once from JS.
#[wasm_bindgen]
pub fn init(width: u32, height: u32) -> Result<(), JsValue> {
    let _ = (width, height);
    // TODO: A2/A5 wire world + renderer.
    Ok(())
}

/// Advance simulation by one fixed step.
#[wasm_bindgen]
pub fn step(_dt: f64) {
    // TODO: dispatch to modules.
}

/// Returns a pointer to the render buffer (placeholder).
#[wasm_bindgen]
pub fn render_buffer_ptr() -> *const u8 {
    std::ptr::null()
}

/// Returns render buffer length in bytes (placeholder).
#[wasm_bindgen]
pub fn render_buffer_len() -> usize {
    0
}

/// Set a cell's material. Returns true on success.
#[wasm_bindgen]
pub fn set_cell(_x: i32, _y: i32, _material: u16) -> bool {
    // TODO: A2 wire world.
    false
}

/// Get a cell's material id.
#[wasm_bindgen]
pub fn get_cell_material(_x: i32, _y: i32) -> u16 {
    0
}

/// Get a cell's temperature (Kelvin).
#[wasm_bindgen]
pub fn get_cell_temperature(_x: i32, _y: i32) -> f32 {
    0.0
}
