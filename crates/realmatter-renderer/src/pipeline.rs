//! Frame composition. Fire uses the reference coarse buffer: particles write
//! a 4×4 block, a precomputed `exp(-0.1 r²)` kernel splats that block, then
//! a 3×3 weighted blur decays it. That is the TPT fire path, not a Gaussian.

use realmatter_core::cell::Cell;
use realmatter_core::units::constants::ATM_PRESSURE;

use crate::{
    element_graphics, ElementGraphics, DECO_FIRE, FIRE_ADD, FIRE_BLEND, FIRE_SPARK, NO_DECO,
    PMODE_ADD, PMODE_BLEND, PMODE_BLUR, PMODE_FLAT, PMODE_GLOW, PMODE_SPARK,
};

pub const DISPLAY_MODES: u8 = 10;
const CELL: usize = 4;
const KERNEL: usize = CELL * 3;
/// Canvas clear colour. GPU shaders use the same bytes.
const BG: [u8; 3] = [5, 8, 15];

#[derive(Clone, Debug)]
pub struct RenderScratch {
    fire_r: Vec<u8>,
    fire_g: Vec<u8>,
    fire_b: Vec<u8>,
    cw: usize,
    ch: usize,
    /// Decayed RGB from the previous persistent frame. Length `w * h * 3`.
    persistent: Vec<u8>,
    pw: usize,
    ph: usize,
}

impl Default for RenderScratch {
    fn default() -> Self {
        Self {
            fire_r: Vec::new(),
            fire_g: Vec::new(),
            fire_b: Vec::new(),
            cw: 0,
            ch: 0,
            persistent: Vec::new(),
            pw: 0,
            ph: 0,
        }
    }
}

impl RenderScratch {
    fn ensure_fire(&mut self, width: usize, height: usize) {
        let cw = width.div_ceil(CELL);
        let ch = height.div_ceil(CELL);
        if self.cw == cw && self.ch == ch && self.fire_r.len() == cw * ch {
            return;
        }
        self.cw = cw;
        self.ch = ch;
        self.fire_r = vec![0; cw * ch];
        self.fire_g = vec![0; cw * ch];
        self.fire_b = vec![0; cw * ch];
    }

    fn clear_fire(&mut self) {
        self.fire_r.fill(0);
        self.fire_g.fill(0);
        self.fire_b.fill(0);
    }
}

#[derive(Clone, Copy)]
struct Flags {
    fire: bool,
    glow: bool,
    blur: bool,
    spark: bool,
    heat_tint: bool,
    persistent: bool,
    gravity: bool,
    /// 0 normal, 1 heat, 2 particle pressure, 3 particle velocity, 4 air heat.
    particles: u8,
    /// 0 none, 1 alt velocity, 2 velocity, 3 pressure, 4 heat.
    air: u8,
}

fn flags_for(mode: u8) -> Flags {
    match mode {
        1 => Flags {
            fire: true,
            glow: true,
            blur: true,
            spark: true,
            heat_tint: true,
            persistent: false,
            gravity: false,
            particles: 0,
            air: 0,
        },
        2 => Flags {
            fire: false,
            glow: false,
            blur: false,
            spark: false,
            heat_tint: false,
            persistent: false,
            gravity: false,
            particles: 1,
            air: 0,
        },
        3 => Flags {
            particles: 2,
            ..no_fx()
        },
        4 => Flags {
            particles: 3,
            ..no_fx()
        },
        5 => Flags {
            air: 2,
            particles: 0,
            ..no_fx()
        },
        6 => Flags {
            air: 3,
            particles: 0,
            ..no_fx()
        },
        7 => Flags {
            air: 4,
            particles: 4,
            ..no_fx()
        },
        8 => Flags {
            persistent: true,
            particles: 0,
            ..no_fx()
        },
        9 => Flags {
            fire: true,
            spark: true,
            gravity: true,
            particles: 0,
            ..no_fx()
        },
        _ => Flags {
            fire: true,
            spark: true,
            particles: 0,
            ..no_fx()
        },
    }
}

fn no_fx() -> Flags {
    Flags {
        fire: false,
        glow: false,
        blur: false,
        spark: false,
        heat_tint: false,
        persistent: false,
        gravity: false,
        particles: 0,
        air: 0,
    }
}

pub fn view_name(mode: u8) -> &'static str {
    match mode {
        0 => "Normal",
        1 => "Fancy",
        2 => "Heat",
        3 => "Pressure",
        4 => "Velocity",
        5 => "Air velocity",
        6 => "Air pressure",
        7 => "Air heat",
        8 => "Persistent",
        9 => "Gravity",
        _ => "Normal",
    }
}

/// Air blocks, persistent history, and gravity ticks read more than one cell,
/// so a one-pixel paint is not enough.
pub fn mode_needs_full(mode: u8) -> bool {
    matches!(mode, 5 | 6 | 7 | 8 | 9)
}

/// Compose `cells` into `out` (RGBA8, row-major). Returns the occupied count.
/// `gravity`: 0 down, 1 off, 2 toward centre. `ambient` is the existing
/// ambient-heat flag; it is not an air-temperature field.
pub fn render(
    cells: &[Cell],
    width: usize,
    height: usize,
    mode: u8,
    gravity: u8,
    ambient: bool,
    scratch: &mut RenderScratch,
    out: &mut Vec<u8>,
) -> u32 {
    let n = width.saturating_mul(height);
    if width == 0 || height == 0 || cells.len() < n {
        out.clear();
        return 0;
    }
    out.resize(n * 4, 0);
    let flags = flags_for(mode.min(DISPLAY_MODES - 1));
    scratch.ensure_fire(width, height);
    if !flags.fire {
        scratch.clear_fire();
    }
    if flags.persistent {
        if scratch.pw != width || scratch.ph != height || scratch.persistent.len() != n * 3 {
            scratch.persistent = vec![0; n * 3];
            scratch.pw = width;
            scratch.ph = height;
        }
        blit_persistent(out, &scratch.persistent, n);
    } else if scratch.pw != 0 {
        scratch.persistent.clear();
        scratch.pw = 0;
        scratch.ph = 0;
        fill_bg(out, n);
    } else {
        fill_bg(out, n);
    }

    if flags.air == 1 || flags.air == 2 || flags.air == 3 {
        paint_air_blocks(cells, out, width, height, flags.air);
    } else if flags.air == 4 && ambient {
        let (r, g, b, _) = heat_rgba(295.15);
        fill_rgb(out, n, r, g, b, 255);
    }

    let mut count = 0u32;
    for y in 0..height {
        for x in 0..width {
            let cell = cells[y * width + x];
            if cell.material == 0 {
                continue;
            }
            count += 1;
            draw_particle(out, width, height, x, y, &cell, flags, scratch);
        }
    }

    if flags.persistent {
        store_persistent(&mut scratch.persistent, out, n);
    }
    if flags.fire {
        splat_fire(out, width, height, scratch);
    }
    if flags.gravity {
        paint_gravity(out, width, height, gravity);
    }
    count
}

/// Repaint one cell in modes that do not depend on neighbours.
pub fn paint_cell(
    out: &mut [u8],
    width: usize,
    height: usize,
    x: i32,
    y: i32,
    cell: &Cell,
    mode: u8,
    ambient: bool,
) {
    if x < 0 || y < 0 || x as usize >= width || y as usize >= height {
        return;
    }
    if out.len() < width * height * 4 {
        return;
    }
    let flags = flags_for(mode.min(DISPLAY_MODES - 1));
    if flags.air == 4 && ambient && cell.material == 0 {
        let (r, g, b, _) = heat_rgba(295.15);
        put(out, width, x as usize, y as usize, r, g, b, 255);
        return;
    }
    if cell.material == 0 {
        put(out, width, x as usize, y as usize, BG[0], BG[1], BG[2], 0);
        return;
    }
    // Start from the background so an additive material does not keep the
    // previous cell's colour. The fire halo updates on the next full frame.
    put(out, width, x as usize, y as usize, BG[0], BG[1], BG[2], 0);
    // Single-cell paints skip the fire buffer; the next full frame splats it.
    let mut quiet = flags;
    quiet.fire = false;
    quiet.persistent = false;
    quiet.gravity = false;
    let dummy = &mut RenderScratch::default();
    draw_particle(
        out, width, height, x as usize, y as usize, cell, quiet, dummy,
    );
}

fn fill_bg(out: &mut [u8], n: usize) {
    for i in 0..n {
        let o = i * 4;
        out[o] = BG[0];
        out[o + 1] = BG[1];
        out[o + 2] = BG[2];
        out[o + 3] = 0;
    }
}

fn fill_rgb(out: &mut [u8], n: usize, r: u8, g: u8, b: u8, a: u8) {
    for i in 0..n {
        let o = i * 4;
        out[o] = r;
        out[o + 1] = g;
        out[o + 2] = b;
        out[o + 3] = a;
    }
}

fn blit_persistent(out: &mut [u8], persistent: &[u8], n: usize) {
    for i in 0..n {
        let p = i * 3;
        let o = i * 4;
        if persistent[p] | persistent[p + 1] | persistent[p + 2] == 0 {
            out[o] = BG[0];
            out[o + 1] = BG[1];
            out[o + 2] = BG[2];
            out[o + 3] = 0;
        } else {
            out[o] = persistent[p];
            out[o + 1] = persistent[p + 1];
            out[o + 2] = persistent[p + 2];
            out[o + 3] = 255;
        }
    }
}

fn store_persistent(persistent: &mut [u8], out: &[u8], n: usize) {
    for i in 0..n {
        let o = i * 4;
        let p = i * 3;
        if out[o + 3] == 0 {
            persistent[p] = 0;
            persistent[p + 1] = 0;
            persistent[p + 2] = 0;
        } else {
            persistent[p] = decay(out[o]);
            persistent[p + 1] = decay(out[o + 1]);
            persistent[p + 2] = decay(out[o + 2]);
        }
    }
}

fn decay(c: u8) -> u8 {
    c.saturating_sub(1)
}

fn draw_particle(
    out: &mut [u8],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    cell: &Cell,
    flags: Flags,
    scratch: &mut RenderScratch,
) {
    if flags.particles == 1 || flags.particles == 4 {
        let (r, g, b, _) = heat_rgba(cell.temperature);
        put(out, width, x, y, r, g, b, 255);
        return;
    }
    if flags.particles == 2 {
        let (r, g, b) = pressure_rgba(cell.pressure);
        put(out, width, x, y, r, g, b, 255);
        return;
    }
    if flags.particles == 3 {
        let (r, g, b) = velocity_rgba(cell.vx, cell.vy);
        put(out, width, x, y, r, g, b, 255);
        return;
    }

    let mut gfx = element_graphics(cell.material);
    if gfx.vary {
        let n = ((x as i32)
            .wrapping_mul(13)
            .wrapping_add((y as i32).wrapping_mul(7))
            & 15) as i32
            - 8;
        gfx.r = add_u8(gfx.r, n);
        gfx.g = add_u8(gfx.g, n);
        gfx.b = add_u8(gfx.b, n);
    }
    if flags.heat_tint && cell.temperature > 500.0 {
        let hot = heat_rgba(cell.temperature);
        let t = ((cell.temperature - 500.0) / 1000.0).clamp(0.0, 0.75);
        gfx.r = lerp(gfx.r, hot.0, t);
        gfx.g = lerp(gfx.g, hot.1, t);
        gfx.b = lerp(gfx.b, hot.2, t);
    }
    // Fire painted with a fractional lifetime (bomb debris) fades. Lifetime 0
    // means "no countdown", not "invisible". Conductor cooldown uses lifetime
    // on iron, which is not material 6, so it is left alone.
    if cell.material == 6 && cell.lifetime > 0.0 && cell.lifetime <= 1.0 {
        let s = cell.lifetime;
        gfx.a = (gfx.a as f32 * s) as u8;
        gfx.fire_a = (gfx.fire_a as f32 * s) as u8;
    }
    apply_decoration(&mut gfx, None);

    let mut mode = gfx.pixel_mode;
    let allow = render_allow(flags);
    if (mode & FIRE_ADD) != 0 && (allow & FIRE_ADD) == 0 {
        mode |= PMODE_GLOW;
    }
    if (mode & FIRE_BLEND) != 0 && (allow & FIRE_BLEND) == 0 {
        mode |= PMODE_BLUR;
    }
    if (mode & PMODE_BLUR) != 0 && (allow & PMODE_BLUR) == 0 {
        mode |= PMODE_FLAT;
    }
    if (mode & PMODE_GLOW) != 0 && (allow & PMODE_GLOW) == 0 {
        mode |= PMODE_BLEND;
    }
    mode &= allow;

    if (mode & PMODE_FLAT) != 0 {
        put(out, width, x, y, gfx.r, gfx.g, gfx.b, 255);
    }
    if (mode & PMODE_BLEND) != 0 {
        blend(
            out, width, height, x as i32, y as i32, gfx.r, gfx.g, gfx.b, gfx.a,
        );
    }
    if (mode & PMODE_ADD) != 0 {
        add_px(
            out, width, height, x as i32, y as i32, gfx.r, gfx.g, gfx.b, gfx.a,
        );
    }
    if (mode & PMODE_GLOW) != 0 {
        draw_glow(out, width, height, x as i32, y as i32, &gfx);
    }
    if (mode & PMODE_BLUR) != 0 {
        draw_blur(out, width, height, x as i32, y as i32, &gfx);
    }
    if (mode & PMODE_SPARK) != 0 {
        draw_spark(out, width, height, x as i32, y as i32, &gfx);
    }
    if flags.fire {
        write_fire(scratch, x, y, &gfx);
    }
}

fn render_allow(flags: Flags) -> u32 {
    let mut bits = PMODE_FLAT | PMODE_BLEND | PMODE_ADD;
    if flags.fire {
        bits |= FIRE_ADD | FIRE_BLEND | FIRE_SPARK;
    }
    if flags.glow {
        bits |= PMODE_GLOW;
    }
    if flags.blur {
        bits |= PMODE_BLUR;
    }
    if flags.spark {
        bits |= PMODE_SPARK;
    }
    bits
}

/// Reference decoration blend. `deco` is `None` until a decoration buffer
/// exists: base colour is unchanged, then fire colour would be mixed only
/// when `DECO_FIRE` is set. Order is base, decoration, fire.
fn apply_decoration(gfx: &mut ElementGraphics, deco: Option<(u8, u8, u8, u8)>) {
    let Some((a, r, g, b)) = deco else {
        return;
    };
    if a == 0 || (gfx.pixel_mode & NO_DECO) != 0 && (gfx.pixel_mode & DECO_FIRE) == 0 {
        return;
    }
    let deca = a.saturating_add(1);
    if (gfx.pixel_mode & NO_DECO) == 0 {
        gfx.r = mix8(deca, r, gfx.r);
        gfx.g = mix8(deca, g, gfx.g);
        gfx.b = mix8(deca, b, gfx.b);
    }
    if (gfx.pixel_mode & DECO_FIRE) != 0 {
        gfx.fire_r = mix8(deca, r, gfx.fire_r);
        gfx.fire_g = mix8(deca, g, gfx.fire_g);
        gfx.fire_b = mix8(deca, b, gfx.fire_b);
    }
}

fn mix8(deca: u8, deco: u8, base: u8) -> u8 {
    ((deca as u16 * deco as u16 + (256 - deca as u16) * base as u16) >> 8) as u8
}

fn write_fire(scratch: &mut RenderScratch, x: usize, y: usize, gfx: &ElementGraphics) {
    if gfx.fire_a == 0 || scratch.cw == 0 {
        return;
    }
    let bx = x / CELL;
    let by = y / CELL;
    if bx >= scratch.cw || by >= scratch.ch {
        return;
    }
    let i = by * scratch.cw + bx;
    let mode = gfx.pixel_mode;
    if (mode & FIRE_BLEND) != 0 {
        let firea = (gfx.fire_a / 2) as u16;
        scratch.fire_r[i] = mix_fire(firea, gfx.fire_r, scratch.fire_r[i]);
        scratch.fire_g[i] = mix_fire(firea, gfx.fire_g, scratch.fire_g[i]);
        scratch.fire_b[i] = mix_fire(firea, gfx.fire_b, scratch.fire_b[i]);
    }
    if (mode & FIRE_ADD) != 0 {
        let firea = (gfx.fire_a / 8) as u16;
        scratch.fire_r[i] = add_fire(firea, gfx.fire_r, scratch.fire_r[i]);
        scratch.fire_g[i] = add_fire(firea, gfx.fire_g, scratch.fire_g[i]);
        scratch.fire_b[i] = add_fire(firea, gfx.fire_b, scratch.fire_b[i]);
    }
    if (mode & FIRE_SPARK) != 0 {
        let firea = (gfx.fire_a / 4) as u16;
        scratch.fire_r[i] = mix_fire(firea, gfx.fire_r, scratch.fire_r[i]);
        scratch.fire_g[i] = mix_fire(firea, gfx.fire_g, scratch.fire_g[i]);
        scratch.fire_b[i] = mix_fire(firea, gfx.fire_b, scratch.fire_b[i]);
    }
}

fn mix_fire(firea: u16, src: u8, dst: u8) -> u8 {
    ((firea * src as u16 + (255 - firea) * dst as u16) >> 8) as u8
}

fn add_fire(firea: u16, src: u8, dst: u8) -> u8 {
    (((firea * src as u16) >> 8) + dst as u16).min(255) as u8
}

fn splat_fire(out: &mut [u8], width: usize, height: usize, scratch: &mut RenderScratch) {
    let kernel = fire_kernel();
    let cw = scratch.cw;
    let ch = scratch.ch;
    for j in 0..ch {
        for i in 0..cw {
            let idx = j * cw + i;
            let r = scratch.fire_r[idx];
            let g = scratch.fire_g[idx];
            let b = scratch.fire_b[idx];
            if r | g | b != 0 {
                let origin_x = i * CELL;
                let origin_y = j * CELL;
                for ky in 0..KERNEL {
                    for kx in 0..KERNEL {
                        let a = kernel[ky][kx];
                        if a == 0 {
                            continue;
                        }
                        let x = origin_x as i32 + kx as i32 - CELL as i32;
                        let y = origin_y as i32 + ky as i32 - CELL as i32;
                        add_fire_px(out, width, height, x, y, r, g, b, a);
                    }
                }
            }
            // In-place 3×3: centre counts eight times, each neighbour once, divide by 16, then drop 4.
            // Left/above neighbours are already the decayed value, matching Renderer::render_fire.
            let mut acc_r = r as u16 * 8;
            let mut acc_g = g as u16 * 8;
            let mut acc_b = b as u16 * 8;
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = i as i32 + dx;
                    let ny = j as i32 + dy;
                    if nx < 0 || ny < 0 || nx >= cw as i32 || ny >= ch as i32 {
                        continue;
                    }
                    let n = ny as usize * cw + nx as usize;
                    acc_r += scratch.fire_r[n] as u16;
                    acc_g += scratch.fire_g[n] as u16;
                    acc_b += scratch.fire_b[n] as u16;
                }
            }
            scratch.fire_r[idx] = decay_fire(acc_r / 16);
            scratch.fire_g[idx] = decay_fire(acc_g / 16);
            scratch.fire_b[idx] = decay_fire(acc_b / 16);
        }
    }
}

fn decay_fire(v: u16) -> u8 {
    if v > 4 {
        (v - 4) as u8
    } else {
        0
    }
}

fn fire_kernel() -> &'static [[u16; KERNEL]; KERNEL] {
    use std::sync::OnceLock;
    static K: OnceLock<[[u16; KERNEL]; KERNEL]> = OnceLock::new();
    K.get_or_init(|| {
        let mut temp = [[0f32; KERNEL]; KERNEL];
        for y in 0..CELL {
            for x in 0..CELL {
                for j in -(CELL as i32)..CELL as i32 {
                    for i in -(CELL as i32)..CELL as i32 {
                        let yy = (y as i32 + CELL as i32 + j) as usize;
                        let xx = (x as i32 + CELL as i32 + i) as usize;
                        let d = (i * i + j * j) as f32;
                        temp[yy][xx] += (-0.1 * d).exp();
                    }
                }
            }
        }
        let mut out = [[0u16; KERNEL]; KERNEL];
        for y in 0..KERNEL {
            for x in 0..KERNEL {
                let v = 255.0 * temp[y][x] / (CELL * CELL) as f32;
                out[y][x] = v as u16;
            }
        }
        out
    })
}

fn paint_air_blocks(cells: &[Cell], out: &mut [u8], width: usize, height: usize, kind: u8) {
    let cw = width.div_ceil(CELL);
    let ch = height.div_ceil(CELL);
    for by in 0..ch {
        for bx in 0..cw {
            let mut vx = 0f32;
            let mut vy = 0f32;
            let mut p = 0f32;
            let mut t = 0f32;
            let mut n = 0f32;
            let x0 = bx * CELL;
            let y0 = by * CELL;
            for ly in 0..CELL {
                let y = y0 + ly;
                if y >= height {
                    break;
                }
                for lx in 0..CELL {
                    let x = x0 + lx;
                    if x >= width {
                        break;
                    }
                    let c = cells[y * width + x];
                    if c.material == 0 {
                        continue;
                    }
                    vx += c.vx;
                    vy += c.vy;
                    p += c.pressure;
                    t += c.temperature;
                    n += 1.0;
                }
            }
            let (r, g, b) = if n == 0.0 {
                (0, 0, 0)
            } else {
                air_rgb(kind, vx / n, vy / n, p / n, t / n)
            };
            for ly in 0..CELL {
                let y = y0 + ly;
                if y >= height {
                    break;
                }
                for lx in 0..CELL {
                    let x = x0 + lx;
                    if x >= width {
                        break;
                    }
                    put(out, width, x, y, r, g, b, 255);
                }
            }
        }
    }
}

/// `kind`: 1 alt velocity, 2 velocity, 3 pressure, 4 heat.
/// Gauge is `(p - 1 atm) / 1 atm * 8`, so the colour clamp of 8 is one
/// atmosphere. This is not TPT's `pv` unit; that lattice is not in the sim.
fn air_rgb(kind: u8, vx: f32, vy: f32, pressure: f32, temp: f32) -> (u8, u8, u8) {
    let pv = ((pressure - ATM_PRESSURE) / ATM_PRESSURE) * 8.0;
    match kind {
        3 => pressure_to_colour(pv),
        2 => (
            clamp_u8(vx.abs(), 8.0),
            clamp_u8(pv.max(0.0), 8.0),
            clamp_u8(vy.abs(), 8.0),
        ),
        4 => {
            let (r, g, b, _) = heat_rgba(temp);
            (r, g, b)
        }
        _ => {
            let mut r = clamp_u8(vx.abs(), 24.0) as i32 + clamp_u8(vy.abs(), 20.0) as i32;
            let g = clamp_u8(vx.abs(), 20.0) as i32 + clamp_u8(vy.abs(), 24.0) as i32;
            let mut b = clamp_u8(vx.abs(), 24.0) as i32 + clamp_u8(vy.abs(), 20.0) as i32;
            if pv > 0.0 {
                r += clamp_u8(pv, 16.0) as i32;
            } else {
                b += clamp_u8(-pv, 16.0) as i32;
            }
            (r.min(255) as u8, g.min(255) as u8, b.min(255) as u8)
        }
    }
}

fn pressure_to_colour(pres: f32) -> (u8, u8, u8) {
    if pres > 0.0 {
        (clamp_u8(pres, 8.0), 0, 0)
    } else {
        (0, 0, clamp_u8(-pres, 8.0))
    }
}

fn clamp_u8(v: f32, max: f32) -> u8 {
    v.clamp(0.0, max) as u8
}

fn paint_gravity(out: &mut [u8], width: usize, height: usize, gravity: u8) {
    if gravity == 1 || width < 8 || height < 8 {
        return;
    }
    let cx = (width / 2) as i32;
    let cy = (height / 2) as i32;
    let mut y = 4usize;
    while y < height {
        let mut x = 4usize;
        while x < width {
            let (dx, dy) = if gravity == 2 {
                let sx = (cx - x as i32).signum();
                let sy = (cy - y as i32).signum();
                (sx, sy)
            } else {
                (0, 1)
            };
            for i in 0..3 {
                let px = x as i32 + dx * i;
                let py = y as i32 + dy * i;
                add_px(out, width, height, px, py, 255, 255, 255, 90);
            }
            x += 8;
        }
        y += 8;
    }
}

/// Unchanged heat scale. Lava at 1500 K is (207, 255, 37). Sand at 293 K is (14, 0, 188).
pub fn heat_rgba(t: f32) -> (u8, u8, u8, u8) {
    let k = ((t - 200.0) / 1600.0).clamp(0.0, 1.0);
    let r = (k * 255.0) as u8;
    let g = ((k * 2.2 - 0.7).clamp(0.0, 1.0) * 255.0) as u8;
    let b = ((1.0 - k).clamp(0.0, 1.0) * 200.0) as u8;
    (r, g, b, 255)
}

fn pressure_rgba(pressure: f32) -> (u8, u8, u8) {
    let k = ((pressure - 90_000.0) / 40_000.0).clamp(0.0, 1.0);
    (
        (40.0 + k * 200.0) as u8,
        (80.0 + (0.5 - (k - 0.5).abs()) * 120.0) as u8,
        (200.0 - k * 160.0) as u8,
    )
}

fn velocity_rgba(vx: f32, vy: f32) -> (u8, u8, u8) {
    let s = (vx * vx + vy * vy).sqrt();
    let k = (s / 8.0).clamp(0.0, 1.0);
    ((40.0 + k * 215.0) as u8, (180.0 - k * 100.0) as u8, 80)
}

fn draw_glow(out: &mut [u8], w: usize, h: usize, x: i32, y: i32, gfx: &ElementGraphics) {
    let cola1 = (5 * gfx.a as u16) / 255;
    add_px(
        out,
        w,
        h,
        x,
        y,
        gfx.r,
        gfx.g,
        gfx.b,
        (192 * gfx.a as u16 / 255) as u8,
    );
    for (dx, dy, a) in [(1, 0, 96u16), (-1, 0, 96), (0, 1, 96), (0, -1, 96)] {
        add_px(
            out,
            w,
            h,
            x + dx,
            y + dy,
            gfx.r,
            gfx.g,
            gfx.b,
            (a * gfx.a as u16 / 255) as u8,
        );
    }
    let a = cola1 as u8;
    for dx in 1..6 {
        add_px(out, w, h, x, y - dx, gfx.r, gfx.g, gfx.b, a);
        add_px(out, w, h, x, y + dx, gfx.r, gfx.g, gfx.b, a);
        add_px(out, w, h, x - dx, y, gfx.r, gfx.g, gfx.b, a);
        add_px(out, w, h, x + dx, y, gfx.r, gfx.g, gfx.b, a);
        for dy in 1..6 {
            if dx + dy > 7 {
                continue;
            }
            add_px(out, w, h, x + dx, y - dy, gfx.r, gfx.g, gfx.b, a);
            add_px(out, w, h, x - dx, y + dy, gfx.r, gfx.g, gfx.b, a);
            add_px(out, w, h, x + dx, y + dy, gfx.r, gfx.g, gfx.b, a);
            add_px(out, w, h, x - dx, y - dy, gfx.r, gfx.g, gfx.b, a);
        }
    }
}

fn draw_blur(out: &mut [u8], w: usize, h: usize, x: i32, y: i32, gfx: &ElementGraphics) {
    for dy in -3i32..=3 {
        for dx in -3i32..=3 {
            let manhattan = dx.abs() + dy.abs();
            let a = if manhattan < 2 && dx.abs() != 2 && dy.abs() != 2 {
                30
            } else if manhattan == 2 {
                10
            } else if manhattan <= 3 && manhattan != 0 {
                20
            } else {
                0
            };
            if a != 0 {
                blend(out, w, h, x + dx, y + dy, gfx.r, gfx.g, gfx.b, a);
            }
        }
    }
}

fn draw_spark(out: &mut [u8], w: usize, h: usize, x: i32, y: i32, gfx: &ElementGraphics) {
    // Flicker is deterministic. The reference adds `rng % 20`.
    let mut gradv = 16.0f32;
    let mut i = 0;
    while gradv > 0.5 && i < 12 {
        let s = (gradv as i32).clamp(0, 255) as u16;
        let r = (gfx.r as u16 * s / 255) as u8;
        let g = (gfx.g as u16 * s / 255) as u8;
        let b = (gfx.b as u16 * s / 255) as u8;
        add_px(out, w, h, x + i, y, r, g, b, 255);
        add_px(out, w, h, x - i, y, r, g, b, 255);
        add_px(out, w, h, x, y + i, r, g, b, 255);
        add_px(out, w, h, x, y - i, r, g, b, 255);
        gradv /= 1.5;
        i += 1;
    }
}

fn put(out: &mut [u8], width: usize, x: usize, y: usize, r: u8, g: u8, b: u8, a: u8) {
    let i = (y * width + x) * 4;
    if i + 3 >= out.len() {
        return;
    }
    out[i] = r;
    out[i + 1] = g;
    out[i + 2] = b;
    out[i + 3] = a;
}

fn blend(out: &mut [u8], w: usize, h: usize, x: i32, y: i32, r: u8, g: u8, b: u8, a: u8) {
    if a == 0 || x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
        return;
    }
    let i = (y as usize * w + x as usize) * 4;
    if a == 255 {
        out[i] = r;
        out[i + 1] = g;
        out[i + 2] = b;
        out[i + 3] = 255;
        return;
    }
    let inv = 255 - a as u16;
    out[i] = ((a as u16 * r as u16 + inv * out[i] as u16) / 255) as u8;
    out[i + 1] = ((a as u16 * g as u16 + inv * out[i + 1] as u16) / 255) as u8;
    out[i + 2] = ((a as u16 * b as u16 + inv * out[i + 2] as u16) / 255) as u8;
    out[i + 3] = 255;
}

fn add_px(out: &mut [u8], w: usize, h: usize, x: i32, y: i32, r: u8, g: u8, b: u8, a: u8) {
    if a == 0 || x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
        return;
    }
    let i = (y as usize * w + x as usize) * 4;
    let ar = (a as u16 * r as u16) / 255;
    let ag = (a as u16 * g as u16) / 255;
    let ab = (a as u16 * b as u16) / 255;
    out[i] = (out[i] as u16 + ar).min(255) as u8;
    out[i + 1] = (out[i + 1] as u16 + ag).min(255) as u8;
    out[i + 2] = (out[i + 2] as u16 + ab).min(255) as u8;
    if ar | ag | ab != 0 {
        out[i + 3] = 255;
    }
}

fn add_fire_px(out: &mut [u8], w: usize, h: usize, x: i32, y: i32, r: u8, g: u8, b: u8, a: u16) {
    if a == 0 || x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
        return;
    }
    let i = (y as usize * w + x as usize) * 4;
    let ar = (a * r as u16) / 255;
    let ag = (a * g as u16) / 255;
    let ab = (a * b as u16) / 255;
    out[i] = (out[i] as u16 + ar).min(255) as u8;
    out[i + 1] = (out[i + 1] as u16 + ag).min(255) as u8;
    out[i + 2] = (out[i + 2] as u16 + ab).min(255) as u8;
    if ar | ag | ab != 0 {
        out[i + 3] = 255;
    }
}

fn add_u8(c: u8, d: i32) -> u8 {
    (c as i32 + d).clamp(0, 255) as u8
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    ((a as f32) * (1.0 - t) + (b as f32) * t) as u8
}
