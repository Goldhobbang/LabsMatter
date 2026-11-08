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
use realmatter_tpt::entity::{Entity, PlayerState};
use realmatter_tpt::particle::Particle;
use realmatter_tpt::portal::PortalPayload;
use realmatter_tpt::sim::{EdgeMode, GravityMode, Sim, CELL, MAX_TEMP, MIN_TEMP};
use wasm_bindgen::prelude::*;

thread_local! {
    static SIM: RefCell<Option<Sim>> = RefCell::new(None);
    static RENDER: RefCell<Vec<u8>> = RefCell::new(Vec::new());
    static WIDTH: RefCell<u32> = RefCell::new(0);
    static HEIGHT: RefCell<u32> = RefCell::new(0);
    /// 0 material, 1 fancy (heat tint), 2 heat, 3 pressure, 4 velocity.
    static VIEW: RefCell<u8> = RefCell::new(0);
    /// Human-readable result of the most recent load attempt (v2 or v1).
    static LOAD_REPORT: RefCell<String> = RefCell::new(String::new());
}

fn set_report(msg: impl Into<String>) {
    LOAD_REPORT.with(|r| *r.borrow_mut() = msg.into());
}

const MAGIC: &[u8; 4] = b"LMT2";
/// Format 3 adds persistent portal payloads, wireless channel state and the
/// stickman/fighter entity registry. Format 2 saves are still readable.
const FORMAT_VERSION: u16 = 3;

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

/// Advance the simulation without rebuilding the CPU render buffer. Used by
/// diagnostics/benchmarks to measure pure simulation cost separately from the
/// render-buffer build. The frontend does not call this; `step` remains the
/// normal entry point so rendering never goes stale during play.
#[wasm_bindgen]
pub fn step_sim_only(_dt: f64) {
    with_sim(|sim| sim.step(), ());
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

// ---- Entity control (stickmen / fighters) --------------------------------

fn player_entity(which: u32) -> Entity {
    if which == 1 {
        Entity::Player2
    } else {
        Entity::Player1
    }
}

/// Spawn player `which` (0 = player 1, 1 = player 2) at a pixel.
#[wasm_bindgen]
pub fn spawn_stickman(which: u32, x: i32, y: i32) -> bool {
    with_sim(|sim| sim.spawn_entity(player_entity(which), x, y).is_some(), false)
}

/// Despawn player `which`.
#[wasm_bindgen]
pub fn despawn_stickman(which: u32) -> bool {
    let had = with_sim(|sim| sim.entity_head(player_entity(which)).is_some(), false);
    if had {
        with_sim(|sim| sim.despawn_entity(player_entity(which)), ());
    }
    had
}

/// Set command bits: `1` left, `2` right, `4` jump, `8` spawn element.
#[wasm_bindgen]
pub fn set_stickman_input(which: u32, comm: u32) {
    with_sim(|sim| sim.set_entity_input(player_entity(which), comm as u8), ());
}

/// Set the element power spawned by the `SPAWN` command bit.
#[wasm_bindgen]
pub fn set_stickman_element(which: u32, t: u16) {
    with_sim(
        |sim| {
            if let Some(st) = sim.player_state_mut(player_entity(which)) {
                st.elem = t as i32;
            }
        },
        (),
    );
}

#[wasm_bindgen]
pub fn stickman_alive(which: u32) -> bool {
    with_sim(|sim| sim.entity_head(player_entity(which)).is_some(), false)
}

/// Spawn a fighter. Returns its slot, or -1 when the registry is full.
#[wasm_bindgen]
pub fn spawn_fighter(x: i32, y: i32) -> i32 {
    with_sim(
        |sim| match sim.spawn_fighter(x, y) {
            Some((slot, _head)) => slot as i32,
            None => -1,
        },
        -1,
    )
}

#[wasm_bindgen]
pub fn despawn_fighter(slot: u32) -> bool {
    let slot = slot as usize;
    let had = with_sim(|sim| sim.entity_head(Entity::Fighter(slot)).is_some(), false);
    if had {
        with_sim(|sim| sim.despawn_fighter(slot), ());
    }
    had
}

#[wasm_bindgen]
pub fn fighter_count() -> u32 {
    with_sim(|sim| sim.figh_count.max(0) as u32, 0)
}

#[wasm_bindgen]
pub fn set_fighter_input(slot: u32, comm: u32) {
    with_sim(|sim| sim.set_entity_input(Entity::Fighter(slot as usize), comm as u8), ());
}

/// Number of particles currently held in portal buffers.
#[wasm_bindgen]
pub fn portal_count() -> u32 {
    with_sim(|sim| sim.portal.len() as u32, 0)
}

/// Number of wireless channels active this frame.
#[wasm_bindgen]
pub fn wireless_active() -> u32 {
    with_sim(|sim| sim.wireless.active_count() as u32, 0)
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
            // Reference `SetEdgeMode`: solid installs a border wall ring so
            // particles are blocked instead of falling into the kill margin.
            sim.apply_edge_mode();
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
            sim.apply_edge_mode();
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

            // --- format 3 additions -------------------------------------
            put_u32(&mut out, sim.portal.len() as u32);
            for (channel, direction, slot, p) in sim.portal.iter_nonempty() {
                put_u16(&mut out, channel as u16);
                out.push(direction as u8);
                out.push(slot as u8);
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
            }

            let active: Vec<usize> = sim.wireless.iter_active().collect();
            put_u32(&mut out, active.len() as u32);
            for ch in active {
                put_u16(&mut out, ch as u16);
            }

            put_player(&mut out, &sim.player);
            put_player(&mut out, &sim.player2);
            put_u32(&mut out, sim.fighters.len() as u32);
            for f in &sim.fighters {
                put_player(&mut out, f);
            }
            put_i32(&mut out, sim.figh_count);
            out
        },
        Vec::new(),
    )
}

fn put_player(out: &mut Vec<u8>, p: &PlayerState) {
    out.push(p.comm);
    out.push(p.pcomm);
    put_i32(out, p.elem);
    for v in p.legs {
        put_f32(out, v);
    }
    for v in p.accs {
        put_f32(out, v);
    }
    out.push(p.spwn);
    put_u32(out, p.frames);
    out.push(p.rocket_boots as u8);
    out.push(p.fan as u8);
    put_i32(out, p.spawn_id);
}

fn read_player(r: &mut Reader<'_>) -> Result<PlayerState, String> {
    let mut p = PlayerState::default();
    p.comm = r.u8()?;
    p.pcomm = r.u8()?;
    p.elem = r.i32()?;
    for v in p.legs.iter_mut() {
        *v = r.f32()?;
    }
    for v in p.accs.iter_mut() {
        *v = r.f32()?;
    }
    p.spwn = r.u8()?;
    p.frames = r.u32()?;
    p.rocket_boots = r.u8()? != 0;
    p.fan = r.u8()? != 0;
    p.spawn_id = r.i32()?;
    Ok(p)
}

fn install_sim(sim: Sim) {
    let w = sim.width as u32;
    let h = sim.height as u32;
    SIM.with(|s| *s.borrow_mut() = Some(sim));
    WIDTH.with(|x| *x.borrow_mut() = w);
    HEIGHT.with(|y| *y.borrow_mut() = h);
    RENDER.with(|r| *r.borrow_mut() = vec![0u8; (w * h * 4) as usize]);
    rebuild_render();
}

/// Reason / summary of the most recent `load_blob` call. Empty before any
/// load. On success of a legacy import this includes how many particles were
/// migrated and which state could not be represented.
#[wasm_bindgen]
pub fn last_load_report() -> String {
    LOAD_REPORT.with(|r| r.borrow().clone())
}

/// Load a save. Accepts the native v2 format (`LMT2`) and the legacy v1
/// Cell-based format (`RMT\0`, optionally `ZLIB`-wrapped). The active world is
/// left untouched on any error. Legacy material ids are mapped through their
/// canonical TPT abbreviation, never reused numerically.
#[wasm_bindgen]
pub fn load_blob(bytes: &[u8]) -> bool {
    if bytes.len() >= 4 && &bytes[0..4] == MAGIC {
        match decode_blob(bytes) {
            Ok(sim) => {
                install_sim(sim);
                set_report("loaded LabsMatter v2/v3 world");
                true
            }
            Err(e) => {
                set_report(format!("v2 load failed: {e}"));
                false
            }
        }
    } else {
        match import_v1(bytes) {
            Ok((sim, report)) => {
                install_sim(sim);
                set_report(report);
                true
            }
            Err(e) => {
                set_report(format!("legacy import failed: {e}"));
                false
            }
        }
    }
}

/// Canonical TPT abbreviation for a legacy (v1) registry id.
fn v1_abbr(id: u16) -> Option<&'static str> {
    match id {
        0 => Some("NONE"),
        1 => Some("STNE"),
        _ => {
            let a = realmatter_core::tpt_elements::abbr_of(id);
            if !a.is_empty() {
                Some(a)
            } else {
                realmatter_core::tpt_elements::ELEMENT_SPECS
                    .iter()
                    .find(|s| s.id == id)
                    .map(|s| s.abbr)
            }
        }
    }
}

/// Migrate a legacy v1 (Cell-based) save into the TPT particle model.
///
/// Mapping rules:
/// - one v1 cell -> one TPT pixel (both render at 4 CSS px per unit);
/// - material id -> canonical abbreviation -> current PT id (explicit, no
///   numeric reuse);
/// - temperature, velocity, and lifetime (seconds -> 60 Hz frames) preserved;
/// - per-cell pressure averaged into the TPT air-pressure CELL grid;
/// - v1 flags and the decoration layer have no TPT equivalent and are reported.
fn import_v1(bytes: &[u8]) -> Result<(Sim, String), String> {
    let raw = realmatter_persist::format::decompress_blob(bytes)?;
    let save = realmatter_persist::format::decode(&raw)?;
    let cfg = &save.header.config;
    let vw = cfg.width as i32;
    let vh = cfg.height as i32;
    if vw <= 0 || vh <= 0 {
        return Err("invalid v1 dimensions".into());
    }
    // TPT requires dimensions divisible by CELL; pad up.
    let w = ((vw as usize) + CELL - 1) / CELL * CELL;
    let h = ((vh as usize) + CELL - 1) / CELL * CELL;
    let mut sim = Sim::new(w, h);
    sim.tick = save.header.tick_count;

    let cs = cfg.chunk_size as i32;
    let mut migrated = 0u32;
    let mut unmapped = 0u32;
    let mut pressure = vec![(0.0f32, 0u32); sim.pv.len()];

    for (_coord, snap) in &save.chunks {
        for (i, cell) in snap.cells.iter().enumerate() {
            if cell.material == 0 {
                continue;
            }
            let lx = (i as i32) % cs;
            let ly = (i as i32) / cs;
            let gx = snap.coord.x * cs + lx;
            let gy = snap.coord.y * cs + ly;
            if gx < 0 || gy < 0 || gx >= vw || gy >= vh {
                continue;
            }
            let t = v1_abbr(cell.material).and_then(realmatter_tpt::pt_id_by_abbr);
            let Some(t) = t else {
                unmapped += 1;
                continue;
            };
            if let Some(pid) = sim.create_part(gx, gy, t) {
                sim.parts[pid].temp = cell.temperature.clamp(MIN_TEMP, MAX_TEMP);
                sim.parts[pid].vx = if cell.vx.is_finite() { cell.vx } else { 0.0 };
                sim.parts[pid].vy = if cell.vy.is_finite() { cell.vy } else { 0.0 };
                if cell.lifetime > 0.0 {
                    sim.parts[pid].life = (cell.lifetime * 60.0)
                        .round()
                        .clamp(0.0, i32::MAX as f32) as i32;
                }
                migrated += 1;
            }
            let cx = gx / CELL as i32;
            let cy = gy / CELL as i32;
            if cx >= 0 && cy >= 0 && (cx as usize) < sim.cells_x && (cy as usize) < sim.cells_y {
                let idx = cy as usize * sim.cells_x + cx as usize;
                if cell.pressure.is_finite() {
                    pressure[idx].0 += cell.pressure;
                    pressure[idx].1 += 1;
                }
            }
        }
    }
    for (idx, (sum, n)) in pressure.iter().enumerate() {
        if *n > 0 {
            sim.pv[idx] = sum / *n as f32;
        }
    }
    sim.recalc();

    let deco_cells = save
        .deco
        .as_ref()
        .map(|d| d.iter().filter(|c| **c != 0).count())
        .unwrap_or(0);
    let mut report = format!(
        "migrated v1 world: {migrated} particles, {}x{} cells",
        vw, vh
    );
    if unmapped > 0 {
        report.push_str(&format!(", {unmapped} unmapped materials skipped"));
    }
    if deco_cells > 0 {
        report.push_str(&format!(
            ", {deco_cells} decoration cells not representable in v2 (omitted)"
        ));
    }
    Ok((sim, report))
}

fn decode_blob(bytes: &[u8]) -> Result<Sim, String> {
    let mut r = Reader::new(bytes);
    let magic = r.take(4)?;
    if magic != MAGIC {
        return Err("not a LabsMatter v2 blob (legacy/unknown magic); refusing to reinterpret".into());
    }
    let version = r.u16()?;
    if version != 2 && version != FORMAT_VERSION {
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

    if version >= 3 {
        // Portal payloads.
        let portal_count = r.u32()? as usize;
        for _ in 0..portal_count {
            let channel = r.u16()? as usize;
            let direction = r.u8()? as usize;
            let slot = r.u8()? as usize;
            let payload = PortalPayload {
                t: r.u16()?,
                life: r.i32()?,
                ctype: r.i32()?,
                x: r.f32()?,
                y: r.f32()?,
                vx: r.f32()?,
                vy: r.f32()?,
                temp: r.f32()?,
                tmp: r.i32()?,
                tmp2: r.i32()?,
                tmp3: r.i32()?,
                tmp4: r.i32()?,
                flags: r.u32()?,
                dcolour: r.u32()?,
            };
            sim.portal.restore(channel, direction, slot, payload);
        }
        // Wireless: channels saved as active become active on the next tick.
        let active_count = r.u32()? as usize;
        for _ in 0..active_count {
            let ch = r.u16()? as usize;
            sim.wireless.activate_next(ch);
        }
        sim.wireless.advance();
        // Entities.
        sim.player = read_player(&mut r)?;
        sim.player2 = read_player(&mut r)?;
        let fighter_count = r.u32()? as usize;
        sim.fighters.clear();
        for _ in 0..fighter_count {
            sim.fighters.push(read_player(&mut r)?);
        }
        sim.figh_count = r.i32()?;
    }

    // Re-establish the edge-mode wall ring from the restored setting (reference
    // `SetEdgeMode` semantics), so a Solid save cannot lose its boundary.
    sim.apply_edge_mode();

    sim.recalc();
    Ok(sim)
}
