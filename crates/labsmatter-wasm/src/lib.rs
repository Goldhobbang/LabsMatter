//! WASM binding for the LabsMatter TPT-parity engine (`realmatter-tpt`).
//!
//! Exposes a small, stable API mirroring `realmatter-wasm` so the frontend
//! adapter can drive the TPT engine. Coordinates are **pixels** (the engine's
//! native unit); the frontend adapter is responsible for any cell<->pixel
//! scaling and material-id mapping.

use std::cell::RefCell;

use realmatter_tpt::sim::{Sim, CELL};
use wasm_bindgen::prelude::*;

thread_local! {
    static SIM: RefCell<Option<Sim>> = RefCell::new(None);
    static RENDER: RefCell<Vec<u8>> = RefCell::new(Vec::new());
    static WIDTH: RefCell<u32> = RefCell::new(0);
    static HEIGHT: RefCell<u32> = RefCell::new(0);
}

fn with_sim<T>(f: impl FnOnce(&mut Sim) -> T, fallback: T) -> T {
    SIM.with(|s| {
        let mut g = s.borrow_mut();
        match g.as_mut() {
            Some(sim) => f(sim),
            None => fallback,
        }
    })
}

/// Initialize the simulation. `width`/`height` must be divisible by CELL (4).
#[wasm_bindgen]
pub fn init(width: u32, height: u32) -> Result<(), JsValue> {
    let w = if width == 0 { 256 } else { width };
    let h = if height == 0 { 256 } else { height };
    if w % CELL as u32 != 0 || h % CELL as u32 != 0 {
        return Err(JsValue::from_str("width/height must be divisible by 4"));
    }
    let sim = Sim::new(w as usize, h as usize);
    SIM.with(|s| *s.borrow_mut() = Some(sim));
    WIDTH.with(|x| *x.borrow_mut() = w);
    HEIGHT.with(|y| *y.borrow_mut() = h);
    RENDER.with(|r| *r.borrow_mut() = vec![0u8; (w * h * 4) as usize]);
    rebuild_render();
    Ok(())
}

fn rebuild_render() {
    SIM.with(|s| {
        let mut g = s.borrow_mut();
        if let Some(sim) = g.as_mut() {
            RENDER.with(|r| {
                let mut buf = r.borrow_mut();
                let n = sim.width * sim.height;
                if buf.len() != n * 4 {
                    buf.resize(n * 4, 0);
                }
                for i in 0..n {
                    let t = realmatter_tpt::pmap_type(if sim.photons[i] != 0 {
                        sim.photons[i]
                    } else {
                        sim.pmap[i]
                    });
                    let (rr, gg, bb) = if t == 0 {
                        (0u8, 0u8, 0u8)
                    } else {
                        let c = realmatter_tpt::element(t).colour;
                        (((c >> 16) & 0xFF) as u8, ((c >> 8) & 0xFF) as u8, (c & 0xFF) as u8)
                    };
                    let a = if t == 0 { 0u8 } else { 255u8 };
                    buf[i * 4] = rr;
                    buf[i * 4 + 1] = gg;
                    buf[i * 4 + 2] = bb;
                    buf[i * 4 + 3] = a;
                }
            });
        }
    });
}

#[wasm_bindgen]
pub fn step(_dt: f64) {
    with_sim(|sim| sim.step(), ());
    rebuild_render();
}

#[wasm_bindgen]
pub fn set_cell(x: i32, y: i32, material: u16) -> bool {
    let ok = with_sim(
        |sim| {
            if !sim.in_bounds(x, y) {
                return false;
            }
            if material == 0 {
                // kill any particle at this pixel
                let cell = sim.idx(x, y);
                let existing = if sim.photons[cell] != 0 {
                    sim.photons[cell]
                } else {
                    sim.pmap[cell]
                };
                if existing != 0 {
                    sim.kill_part(realmatter_tpt::pmap_id(existing) as usize);
                }
                return true;
            }
            sim.paint(x, y, material, 0);
            true
        },
        false,
    );
    if ok {
        rebuild_render();
    }
    ok
}

#[wasm_bindgen]
pub fn get_cell_material(x: i32, y: i32) -> u16 {
    with_sim(
        |sim| {
            if !sim.in_bounds(x, y) {
                return 0;
            }
            let cell = sim.idx(x, y);
            let r = if sim.photons[cell] != 0 {
                sim.photons[cell]
            } else {
                sim.pmap[cell]
            };
            realmatter_tpt::pmap_type(r)
        },
        0,
    )
}

#[wasm_bindgen]
pub fn get_cell_temperature(x: i32, y: i32) -> f32 {
    with_sim(
        |sim| {
            if !sim.in_bounds(x, y) {
                return 0.0;
            }
            let cell = sim.idx(x, y);
            let r = if sim.photons[cell] != 0 {
                sim.photons[cell]
            } else {
                sim.pmap[cell]
            };
            let id = realmatter_tpt::pmap_id(r) as usize;
            if id < sim.parts.len() {
                sim.parts[id].temp
            } else {
                0.0
            }
        },
        0.0,
    )
}

#[wasm_bindgen]
pub fn world_width() -> u32 {
    WIDTH.with(|w| *w.borrow())
}

#[wasm_bindgen]
pub fn world_height() -> u32 {
    HEIGHT.with(|h| *h.borrow())
}

#[wasm_bindgen]
pub fn reset() -> bool {
    let w = world_width();
    let h = world_height();
    if w == 0 || h == 0 {
        return false;
    }
    init(w, h).is_ok()
}

#[wasm_bindgen]
pub fn render_buffer_ptr() -> *const u8 {
    RENDER.with(|r| r.borrow().as_ptr())
}

#[wasm_bindgen]
pub fn render_buffer_len() -> usize {
    RENDER.with(|r| r.borrow().len())
}

#[wasm_bindgen]
pub fn tick_count() -> u64 {
    with_sim(|sim| sim.tick, 0)
}