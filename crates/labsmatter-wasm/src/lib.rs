//! WASM binding for the LabsMatter TPT-parity engine (`realmatter-tpt`).
//!
//! Exposes a small, stable API mirroring `realmatter-wasm` so the frontend
//! adapter can drive the TPT engine. Coordinates are **pixels** (the engine's
//! native unit); the frontend adapter is responsible for any cell<->pixel
//! scaling and material-id mapping.
//!
//! Rendering is derived from live simulation state (particle id -> element
//! colour, plus heat/pressure/velocity field views). Nothing here invents
//! simulation data; the Rust `Sim` is authoritative.

use std::cell::RefCell;

use realmatter_tpt::air::{AirMode, ConvectionMode};
use realmatter_tpt::element::element;
use realmatter_tpt::particle::Particle;
use realmatter_tpt::sim::{EdgeMode, GravityMode, Sim, CELL};
use wasm_bindgen::prelude::*;

thread_local! {
    static SIM: RefCell<Option<Sim>> = RefCell::new(None);
    static RENDER: RefCell<Vec<u8>> = RefCell::new(Vec::new());
    static WIDTH: RefCell<u32> = RefCell::new(0);
    static HEIGHT: RefCell<u32> = RefCell::new(0);
    /// 0 material, 1 fancy (heat tint), 2 heat, 3 pressure, 4 velocity.
    static VIEW: RefCell<u8> = RefCell::new(0);
}

const MAGIC: &[u8; 4] = b"LMT2";
const FORMAT_VERSION: u16 = 2;

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

fn element_colour(t: u16) -> (u8, u8, u8) {
    let c = element(t).colour;
    (
        ((c >> 16) & 0xFF) as u8,
        ((c >> 8) & 0xFF) as u8,
        (c & 0xFF) as u8,
    )
}

fn heat_rgba(t: f32) -> (u8, u8, u8) {
    let k = ((t - 200.0) / 1600.0).clamp(0.0, 1.0);
    let r = (k * 255.0) as u8;
    let g = ((k * 2.2 - 0.7).clamp(0.0, 1.0) * 255.0) as u8;
    let b = ((1.0 - k).clamp(0.0, 1.0) * 200.0) as u8;
    (r, g, b)
}

fn rebuild_render() {
    let view = VIEW.with(|v| *v.borrow());
    SIM.with(|s| {
        let mut g = s.borrow_mut();
        let Some(sim) = g.as_mut() else { return };
        RENDER.with(|r| {
            let mut buf = r.borrow_mut();
            let n = sim.width * sim.height;
            if buf.len() != n * 4 {
                buf.resize(n * 4, 0);
            }
            let cells_x = sim.cells_x;
            for y in 0..sim.height {
                for x in 0..sim.width {
                    let i = y * sim.width + x;
                    let (rr, gg, bb, aa);
                    if view == 3 || view == 4 {
                        // Field views are valid for empty pixels too.
                        let cx = x / CELL;
                        let cy = y / CELL;
                        let ci = cy * cells_x + cx;
                        let (r, g, b) = if view == 3 {
                            let p = sim.pv[ci];
                            let k = ((p + 8.0) / 16.0).clamp(0.0, 1.0);
                            (
                                (40.0 + k * 200.0) as u8,
                                (80.0 + (0.5 - (k - 0.5).abs()) * 120.0) as u8,
                                (200.0 - k * 160.0) as u8,
                            )
                        } else {
                            let (vx, vy) = (sim.vx[ci], sim.vy[ci]);
                            let s = (vx * vx + vy * vy).sqrt();
                            let k = (s / 4.0).clamp(0.0, 1.0);
                            ((40.0 + k * 215.0) as u8, (180.0 - k * 100.0) as u8, 80)
                        };
                        rr = r;
                        gg = g;
                        bb = b;
                        aa = 255;
                    } else {
                        let packed = if sim.photons[i] != 0 {
                            sim.photons[i]
                        } else {
                            sim.pmap[i]
                        };
                        let t = realmatter_tpt::pmap_type(packed);
                        if t == 0 {
                            rr = 0;
                            gg = 0;
                            bb = 0;
                            aa = 0;
                        } else {
                            if view == 2 {
                                let id = realmatter_tpt::pmap_id(packed) as usize;
                                let temp = if id < sim.parts.len() {
                                    sim.parts[id].temp
                                } else {
                                    293.15
                                };
                                let (r, g, b) = heat_rgba(temp);
                                rr = r;
                                gg = g;
                                bb = b;
                            } else {
                                let (r, g, b) = element_colour(t);
                                rr = r;
                                gg = g;
                                bb = b;
                            }
                            aa = 255;
                        }
                    }
                    let o = i * 4;
                    buf[o] = rr;
                    buf[o + 1] = gg;
                    buf[o + 2] = bb;
                    buf[o + 3] = aa;
                }
            }
        });
    });
}

#[wasm_bindgen]
pub fn step(_dt: f64) {
    with_sim(|sim| sim.step(), ());
    rebuild_render();
}

/// Paint a particle (PT id space). `material == 0` erases the pixel.
#[wasm_bindgen]
pub fn set_cell(x: i32, y: i32, material: u16) -> bool {
    let ok = with_sim(
        |sim| {
            if !sim.in_bounds(x, y) {
                return false;
            }
            if material == 0 {
                let cell = sim.idx(x, y);
                let existing = if sim.photons[cell] != 0 {
                    sim.photons[cell]
                } else {
                    sim.pmap[cell]
                };
                if existing != 0 {
                    let id = realmatter_tpt::pmap_id(existing) as usize;
                    if id < sim.parts.len() {
                        sim.kill_part(id);
                    }
                }
                return true;
            }
            // Never let a wall palette id (or any unknown id) reach `paint`:
            // `paint` would kill the existing particle before failing to create
            // the unknown element, silently deleting matter.
            if (material as usize) >= realmatter_tpt::PT_NUM || !element(material).enabled {
                return false;
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

/// Set a wall in the CELL containing `(x, y)`. `w == 0` erases the wall.
#[wasm_bindgen]
pub fn paint_wall(x: i32, y: i32, w: u8) {
    with_sim(
        |sim| {
            sim.paint_wall(x, y, w);
        },
        (),
    );
    rebuild_render();
}

/// Wall id at `(x, y)` (0 = none).
#[wasm_bindgen]
pub fn get_wall(x: i32, y: i32) -> u8 {
    with_sim(
        |sim| {
            if !sim.in_bounds(x, y) {
                return 0;
            }
            let cx = x / CELL as i32;
            let cy = y / CELL as i32;
            if sim.in_cell_bounds(cx, cy) {
                sim.bmap[sim.cell_idx(cx, cy)]
            } else {
                0
            }
        },
        0,
    )
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

/// Air pressure at `(x, y)` in the reference's pressure units.
#[wasm_bindgen]
pub fn get_cell_pressure(x: i32, y: i32) -> f32 {
    with_sim(
        |sim| {
            if !sim.in_bounds(x, y) {
                return 0.0;
            }
            let cx = x / CELL as i32;
            let cy = y / CELL as i32;
            if sim.in_cell_bounds(cx, cy) {
                sim.pv[sim.cell_idx(cx, cy)]
            } else {
                0.0
            }
        },
        0.0,
    )
}

#[wasm_bindgen]
pub fn get_cell_vx(x: i32, y: i32) -> f32 {
    with_sim(
        |sim| {
            if !sim.in_bounds(x, y) {
                return 0.0;
            }
            let cx = x / CELL as i32;
            let cy = y / CELL as i32;
            if sim.in_cell_bounds(cx, cy) {
                sim.vx[sim.cell_idx(cx, cy)]
            } else {
                0.0
            }
        },
        0.0,
    )
}

#[wasm_bindgen]
pub fn get_cell_vy(x: i32, y: i32) -> f32 {
    with_sim(
        |sim| {
            if !sim.in_bounds(x, y) {
                return 0.0;
            }
            let cx = x / CELL as i32;
            let cy = y / CELL as i32;
            if sim.in_cell_bounds(cx, cy) {
                sim.vy[sim.cell_idx(cx, cy)]
            } else {
                0.0
            }
        },
        0.0,
    )
}

/// Live particle count (alive slots, ordinary + energy).
#[wasm_bindgen]
pub fn particle_count() -> u32 {
    with_sim(
        |sim| {
            sim.parts
                .iter()
                .filter(|p| p.t != 0)
                .count() as u32
        },
        0,
    )
}

/// Number of particles of one PT id.
#[wasm_bindgen]
pub fn element_count(t: u16) -> u32 {
    with_sim(
        |sim| sim.element_count.get(t as usize).copied().unwrap_or(0),
        0,
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
pub fn tick_count() -> u64 {
    with_sim(|sim| sim.tick, 0)
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
pub fn set_view_mode(mode: u32) {
    VIEW.with(|v| *v.borrow_mut() = mode.min(4) as u8);
    rebuild_render();
}

// ---- Runtime settings ----------------------------------------------------

#[wasm_bindgen]
pub fn set_gravity_mode(mode: u32) {
    with_sim(
        |sim| {
            sim.settings.gravity_mode = match mode {
                1 => GravityMode::Off,
                2 => GravityMode::Radial,
                3 => GravityMode::Custom,
                _ => GravityMode::Vertical,
            };
        },
        (),
    );
}

#[wasm_bindgen]
pub fn set_air_mode(mode: u32) {
    with_sim(
        |sim| {
            sim.settings.air_mode = match mode {
                1 => AirMode::PressureOff,
                2 => AirMode::VelocityOff,
                3 => AirMode::Off,
                _ => AirMode::On,
            };
        },
        (),
    );
}

#[wasm_bindgen]
pub fn set_edge_mode(mode: u32) {
    with_sim(
        |sim| {
            sim.settings.edge_mode = match mode {
                0 => EdgeMode::Void,
                2 => EdgeMode::Loop,
                _ => EdgeMode::Solid,
            };
        },
        (),
    );
}

#[wasm_bindgen]
pub fn set_newtonian(on: u32) {
    with_sim(|sim| sim.settings.newtonian_gravity = on != 0, ());
}

#[wasm_bindgen]
pub fn set_ambient_heat(on: u32) {
    with_sim(|sim| sim.settings.ambient_heat = on != 0, ());
}

#[wasm_bindgen]
pub fn set_heat_sim(on: u32) {
    with_sim(|sim| sim.settings.heat_sim = on != 0, ());
}

#[wasm_bindgen]
pub fn set_convection_mode(mode: u32) {
    with_sim(
        |sim| {
            sim.settings.convection_mode = match mode {
                0 => ConvectionMode::None,
                1 => ConvectionMode::Legacy,
                _ => ConvectionMode::Boussinesq,
            };
        },
        (),
    );
}

fn cycle(val: u8, n: u8) -> u8 {
    (val + 1) % n
}

#[wasm_bindgen]
pub fn cycle_gravity() {
    with_sim(
        |sim| {
            let next = cycle(sim.settings.gravity_mode as u8, 4);
            sim.settings.gravity_mode = match next {
                1 => GravityMode::Off,
                2 => GravityMode::Radial,
                3 => GravityMode::Custom,
                _ => GravityMode::Vertical,
            };
        },
        (),
    );
}

#[wasm_bindgen]
pub fn cycle_air() {
    with_sim(
        |sim| {
            let next = cycle(sim.settings.air_mode as u8, 4);
            sim.settings.air_mode = match next {
                1 => AirMode::PressureOff,
                2 => AirMode::VelocityOff,
                3 => AirMode::Off,
                _ => AirMode::On,
            };
        },
        (),
    );
}

#[wasm_bindgen]
pub fn cycle_edge() {
    with_sim(
        |sim| {
            let next = (sim.settings.edge_mode as u8 + 1) % 3;
            sim.settings.edge_mode = match next {
                0 => EdgeMode::Void,
                2 => EdgeMode::Loop,
                _ => EdgeMode::Solid,
            };
        },
        (),
    );
}

#[wasm_bindgen]
pub fn toggle_newtonian() {
    with_sim(|sim| sim.settings.newtonian_gravity = !sim.settings.newtonian_gravity, ());
}

#[wasm_bindgen]
pub fn toggle_ambient() {
    with_sim(|sim| sim.settings.ambient_heat = !sim.settings.ambient_heat, ());
}

#[wasm_bindgen]
pub fn sim_status() -> String {
    with_sim(
        |sim| {
            let g = ["down", "off", "radial", "custom"][(sim.settings.gravity_mode as usize).min(3)];
            let a = ["on", "press-off", "vel-off", "off"][(sim.settings.air_mode as usize).min(3)];
            let e = ["void", "solid", "loop"][(sim.settings.edge_mode as usize).min(2)];
            format!(
                "grav {} air {} edge {} n{} u{}",
                g,
                a,
                e,
                if sim.settings.newtonian_gravity { "+" } else { "-" },
                if sim.settings.ambient_heat { "+" } else { "-" },
            )
        },
        "sim?".into(),
    )
}

// ---- Persistence (format v2) --------------------------------------------
//
// Layout (little-endian):
//   magic "LMT2", u16 version, u32 width, u32 height, u64 tick,
//   settings (u8 * 6, f32 ambient_temp, f32 edge_pressure, f32 edge_vx,
//   f32 edge_vy, f32 custom_x, f32 custom_y, u32 gol_speed),
//   u64 rng_s0, u64 rng_s1,
//   u32 part_count, then per part (t u16, life i32, ctype i32, x/y/vx/vy/temp
//   f32, tmp/tmp2/tmp3/tmp4 i32, flags u32, dcolour u32, pavg [u8;4]),
//   per-cell fields (pv, vx, vy, hv f32 * cells), bmap u8*, emap u8*.
//
// Particle positions are pixels. pmap/photons are rebuilt from `parts` on load.

fn put_u16(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn put_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn put_u64(out: &mut Vec<u8>, v: u64) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn put_i32(out: &mut Vec<u8>, v: i32) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn put_f32(out: &mut Vec<u8>, v: f32) {
    out.extend_from_slice(&v.to_le_bytes());
}

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}
impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        if self.pos + n > self.data.len() {
            return Err("truncated blob".into());
        }
        let s = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }
    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, String> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64, String> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn i32(&mut self) -> Result<i32, String> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn f32(&mut self) -> Result<f32, String> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
}

/// Serialize the current world (format v2). Empty vec if uninitialized.
#[wasm_bindgen]
pub fn save_blob() -> Vec<u8> {
    with_sim(
        |sim| {
            let mut out = Vec::with_capacity(
                64 + sim.parts.len() * 44 + sim.cells_x * sim.cells_y * 18,
            );
            out.extend_from_slice(MAGIC);
            put_u16(&mut out, FORMAT_VERSION);
            put_u32(&mut out, sim.width as u32);
            put_u32(&mut out, sim.height as u32);
            put_u64(&mut out, sim.tick);

            out.push(sim.settings.gravity_mode as u8);
            out.push(sim.settings.air_mode as u8);
            out.push(sim.settings.edge_mode as u8);
            out.push(sim.settings.heat_sim as u8);
            out.push(sim.settings.ambient_heat as u8);
            out.push(sim.settings.newtonian_gravity as u8);
            put_f32(&mut out, sim.settings.ambient_air_temp);
            put_f32(&mut out, sim.settings.edge_pressure);
            put_f32(&mut out, sim.settings.edge_velocity_x);
            put_f32(&mut out, sim.settings.edge_velocity_y);
            put_f32(&mut out, sim.settings.custom_gravity.0);
            put_f32(&mut out, sim.settings.custom_gravity.1);
            put_u32(&mut out, sim.settings.gol_speed);

            let (s0, s1) = sim.rng.state();
            put_u64(&mut out, s0);
            put_u64(&mut out, s1);

            let alive: Vec<&Particle> = sim.parts.iter().filter(|p| p.t != 0).collect();
            put_u32(&mut out, alive.len() as u32);
            for p in alive {
                put_u16(&mut out, p.t);
                put_i32(&mut out, p.life);
                put_i32(&mut out, p.ctype);
                put_f32(&mut out, p.x);
                put_f32(&mut out, p.y);
                put_f32(&mut out, p.vx);
                put_f32(&mut out, p.vy);
                put_f32(&mut out, p.temp);
                put_i32(&mut out, p.tmp);
                put_i32(&mut out, p.tmp2);
                put_i32(&mut out, p.tmp3);
                put_i32(&mut out, p.tmp4);
                put_u32(&mut out, p.flags);
                put_u32(&mut out, p.dcolour);
                out.extend_from_slice(&p.pavg);
            }
            for i in 0..sim.pv.len() {
                put_f32(&mut out, sim.pv[i]);
                put_f32(&mut out, sim.vx[i]);
                put_f32(&mut out, sim.vy[i]);
                put_f32(&mut out, sim.hv[i]);
            }
            out.extend_from_slice(&sim.bmap);
            out.extend_from_slice(&sim.emap);
            out
        },
        Vec::new(),
    )
}

/// Load a v2 blob. Returns false (and leaves the current world untouched) on
/// any error, including a v1/legacy blob — IDs are never reinterpreted.
#[wasm_bindgen]
pub fn load_blob(bytes: &[u8]) -> bool {
    match decode_blob(bytes) {
        Ok(sim) => {
            let w = sim.width as u32;
            let h = sim.height as u32;
            SIM.with(|s| *s.borrow_mut() = Some(sim));
            WIDTH.with(|x| *x.borrow_mut() = w);
            HEIGHT.with(|y| *y.borrow_mut() = h);
            RENDER.with(|r| *r.borrow_mut() = vec![0u8; (w * h * 4) as usize]);
            rebuild_render();
            true
        }
        Err(e) => {
            // Surface the reason in the browser console via wasm-bindgen's panic
            // hook is too noisy; callers can distinguish false.
            let _ = e;
            false
        }
    }
}

fn decode_blob(bytes: &[u8]) -> Result<Sim, String> {
    let mut r = Reader::new(bytes);
    let magic = r.take(4)?;
    if magic != MAGIC {
        return Err("not a LabsMatter v2 blob (legacy/unknown magic); refusing to reinterpret".into());
    }
    let version = r.u16()?;
    if version != FORMAT_VERSION {
        return Err(format!("unsupported save version {version}"));
    }
    let width = r.u32()? as usize;
    let height = r.u32()? as usize;
    if width == 0 || height == 0 || width % CELL != 0 || height % CELL != 0 {
        return Err("invalid world dimensions in blob".into());
    }
    let mut sim = Sim::new(width, height);
    sim.tick = r.u64()?;

    sim.settings.gravity_mode = match r.u8()? {
        1 => GravityMode::Off,
        2 => GravityMode::Radial,
        3 => GravityMode::Custom,
        _ => GravityMode::Vertical,
    };
    sim.settings.air_mode = match r.u8()? {
        1 => AirMode::PressureOff,
        2 => AirMode::VelocityOff,
        3 => AirMode::Off,
        _ => AirMode::On,
    };
    sim.settings.edge_mode = match r.u8()? {
        0 => EdgeMode::Void,
        2 => EdgeMode::Loop,
        _ => EdgeMode::Solid,
    };
    sim.settings.heat_sim = r.u8()? != 0;
    sim.settings.ambient_heat = r.u8()? != 0;
    sim.settings.newtonian_gravity = r.u8()? != 0;
    sim.settings.ambient_air_temp = r.f32()?;
    sim.settings.edge_pressure = r.f32()?;
    sim.settings.edge_velocity_x = r.f32()?;
    sim.settings.edge_velocity_y = r.f32()?;
    sim.settings.custom_gravity = (r.f32()?, r.f32()?);
    sim.settings.gol_speed = r.u32()?;

    let s0 = r.u64()?;
    let s1 = r.u64()?;
    sim.rng.set_state(s0, s1);

    let part_count = r.u32()? as usize;
    sim.parts.clear();
    sim.parts.reserve(part_count);
    for _ in 0..part_count {
        let t = r.u16()?;
        let life = r.i32()?;
        let ctype = r.i32()?;
        let x = r.f32()?;
        let y = r.f32()?;
        let vx = r.f32()?;
        let vy = r.f32()?;
        let temp = r.f32()?;
        let tmp = r.i32()?;
        let tmp2 = r.i32()?;
        let tmp3 = r.i32()?;
        let tmp4 = r.i32()?;
        let flags = r.u32()?;
        let dcolour = r.u32()?;
        let pavg = r.take(4)?;
        if t == 0 {
            continue;
        }
        let mut p = Particle::new(t, x, y);
        p.life = life;
        p.ctype = ctype;
        p.vx = vx;
        p.vy = vy;
        p.temp = temp;
        p.tmp = tmp;
        p.tmp2 = tmp2;
        p.tmp3 = tmp3;
        p.tmp4 = tmp4;
        p.flags = flags;
        p.dcolour = dcolour;
        p.pavg.copy_from_slice(pavg);
        sim.parts.push(p);
    }

    let n = sim.cells_x * sim.cells_y;
    if sim.pv.len() != n {
        return Err("field size mismatch".into());
    }
    for i in 0..n {
        sim.pv[i] = r.f32()?;
        sim.vx[i] = r.f32()?;
        sim.vy[i] = r.f32()?;
        sim.hv[i] = r.f32()?;
    }
    let bmap = r.take(n)?.to_vec();
    let emap = r.take(n)?.to_vec();
    sim.bmap = bmap;
    sim.emap = emap;

    sim.recalc();
    Ok(sim)
}
