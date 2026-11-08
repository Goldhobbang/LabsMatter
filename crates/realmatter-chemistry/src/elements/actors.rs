//! Stickman and fighter.
//!
//! Legs are eight Verlet points (current, previous) with the reference joint
//! constraints. Feet feel gravity; the head is biased against gravity and is
//! tied to the hips. STKM and STK2 keep separate player records. Fighters
//! reuse the same body and pick the nearer living player. The reference
//! fighter does not target other fighters.

use realmatter_core::cell::{cell_flags, Cell};

use super::ids::*;
use super::sim::{at, init_legs, ElementSim, Player};

const LEFT: u8 = 0x01;
const RIGHT: u8 = 0x02;
const JUMP: u8 = 0x04;
const ACTION: u8 = 0x08;

pub fn update(
    cells: &mut [Cell],
    sim: &mut ElementSim,
    w: usize,
    h: usize,
    wrap: bool,
    gravity: u8,
) {
    reconcile(cells, sim, w, h);
    assign_fighters(cells, sim, w, h);
    for slot in 0..2 {
        if sim.players[slot].present {
            let kind = if slot == 0 { STKM } else { STK2 };
            run(cells, sim, slot, false, kind, w, h, wrap, gravity);
        }
    }
    let count = sim.fighters.len();
    for slot in 0..count {
        if sim.fighters[slot].present {
            run(cells, sim, slot, true, FIGH, w, h, wrap, gravity);
        }
    }
}

fn reconcile(cells: &mut [Cell], sim: &mut ElementSim, w: usize, h: usize) {
    for slot in 0..2 {
        let id = if slot == 0 { STKM } else { STK2 };
        let idx = sim.players[slot].index;
        let alive = idx >= 0 && (idx as usize) < cells.len() && cells[idx as usize].material == id;
        if !alive {
            sim.players[slot].present = false;
            sim.players[slot].index = -1;
        } else {
            let i = idx as usize;
            let px = (i % w) as f32;
            let py = (i / w) as f32;
            if sim.players[slot].legs[1] == 0.0 && sim.players[slot].legs[5] == 0.0 {
                init_legs(&mut sim.players[slot], px, py);
                sim.players[slot].x = px;
                sim.players[slot].y = py;
            }
            sim.players[slot].present = true;
        }
    }
    let mut freed = 0u16;
    for (slot, f) in sim.fighters.iter_mut().enumerate() {
        let idx = f.index;
        let alive = f.present
            && idx >= 0
            && (idx as usize) < cells.len()
            && cells[idx as usize].material == FIGH
            && sim
                .aux
                .get(idx as usize)
                .is_some_and(|a| a.tmp == slot as i32);
        if f.present && !alive {
            *f = Player::default();
            freed += 1;
        }
    }
    sim.fighcount = sim.fighcount.saturating_sub(freed);
    let _ = h;
}

fn assign_fighters(cells: &mut [Cell], sim: &mut ElementSim, w: usize, h: usize) {
    let n = cells.len().min(sim.aux.len());
    for i in 0..n {
        if cells[i].material != FIGH {
            continue;
        }
        let slot = sim.aux[i].tmp;
        if slot < 0 || slot as usize >= sim.fighters.len() || !sim.fighters[slot as usize].present {
            let alloc = sim.alloc_fighter(i);
            if alloc < 0 {
                cells[i] = Cell::empty();
                continue;
            }
            sim.aux[i].tmp = alloc;
            sim.init_fighter_at(alloc as usize, i, (i % w) as i32, (i / w) as i32);
            if sim.aux[i].ctype != 0 {
                sim.fighters[alloc as usize].elem = sim.aux[i].ctype;
            }
        } else {
            let tx = (i % w) as f32;
            let ty = (i / w) as f32;
            let p = &mut sim.fighters[slot as usize];
            p.index = i as i32;
            if p.legs[1] == 0.0 && p.legs[5] == 0.0 {
                init_legs(p, tx, ty);
                p.x = tx;
                p.y = ty;
            }
            if (p.x - tx).abs() > 3.0 || (p.y - ty).abs() > 3.0 {
                p.x = tx;
                p.y = ty;
            }
        }
    }
    let n = cells.len().min(sim.aux.len());
    for i in 0..n {
        if cells[i].material == FIGH {
            let elem = sim.aux[i].ctype;
            let slot = sim.aux[i].tmp;
            if elem != 0 && slot >= 0 {
                if let Some(f) = sim.fighters.get_mut(slot as usize) {
                    f.elem = elem;
                }
            }
        }
    }
    let _ = h;
}

fn run(
    cells: &mut [Cell],
    sim: &mut ElementSim,
    slot: usize,
    fighter: bool,
    kind: u16,
    w: usize,
    h: usize,
    wrap: bool,
    gravity: u8,
) {
    let index = if fighter {
        sim.fighters[slot].index
    } else {
        sim.players[slot].index
    };
    if index < 0 || index as usize >= cells.len() || cells[index as usize].material != kind {
        return;
    }
    let i = index as usize;
    if fighter {
        steer_fighter(sim, slot);
    }
    {
        let p = if fighter {
            &mut sim.fighters[slot]
        } else {
            &mut sim.players[slot]
        };
        p.pcomm = p.comm;
        p.frames = p.frames.saturating_add(1);
    }
    let (gvx, gvy) = gravity_vec(gravity, sim_ref_xy(sim, slot, fighter), w, h);
    integrate(sim, slot, fighter, gvx, gvy, cells, w, h);
    collide_feet(sim, slot, fighter, cells, w, h);
    if damage_and_pick(cells, sim, slot, fighter, i, w, h, wrap) {
        return;
    }
    if action_pressed(sim, slot, fighter) {
        use_element(cells, sim, slot, fighter, i, w, h, wrap);
    }
    step_body(cells, sim, slot, fighter, i, w, h);
    let p = if fighter {
        &sim.fighters[slot]
    } else {
        &sim.players[slot]
    };
    if p.present {
        let idx = p.index;
        if idx >= 0 {
            if let Some(aux) = sim.aux.get_mut(idx as usize) {
                aux.ctype = p.elem;
                aux.life = p.life;
            }
        }
    }
}

fn sim_ref_xy(sim: &ElementSim, slot: usize, fighter: bool) -> (f32, f32) {
    let p = if fighter {
        &sim.fighters[slot]
    } else {
        &sim.players[slot]
    };
    (p.x, p.y)
}

fn gravity_vec(mode: u8, (x, y): (f32, f32), w: usize, h: usize) -> (f32, f32) {
    match mode {
        1 => (0.0, 0.0),
        2 => {
            let cx = w as f32 * 0.5;
            let cy = h as f32 * 0.5;
            let dx = x - cx;
            let dy = y - cy;
            let mag = (dx * dx + dy * dy).sqrt().max(0.001);
            (dx / mag, dy / mag)
        }
        _ => (0.0, 1.0),
    }
}

fn integrate(
    sim: &mut ElementSim,
    slot: usize,
    fighter: bool,
    gvx: f32,
    gvy: f32,
    cells: &[Cell],
    w: usize,
    h: usize,
) {
    let p = if fighter {
        &mut sim.fighters[slot]
    } else {
        &mut sim.players[slot]
    };
    let dt = 0.9f32;
    let (mvx, mvy) = unit_gravity(gvx, gvy);
    p.vx -= gvx * dt;
    p.vy -= gvy * dt;
    let acc = p.accs;
    verlet(p, 0, 2, acc[0] * dt * dt);
    verlet(p, 1, 3, acc[1] * dt * dt);
    verlet(p, 4, 6, (acc[2] + gvx) * dt * dt);
    verlet(p, 5, 7, (acc[3] + gvy) * dt * dt);
    verlet(p, 8, 10, acc[4] * dt * dt);
    verlet(p, 9, 11, acc[5] * dt * dt);
    verlet(p, 12, 14, (acc[6] + gvx) * dt * dt);
    verlet(p, 13, 15, (acc[7] + gvy) * dt * dt);
    p.accs = [0.0; 8];

    let comm = p.comm;
    let left_foot_blocked = foot_blocked(cells, w, h, p.legs[4], p.legs[5]);
    let right_foot_blocked = foot_blocked(cells, w, h, p.legs[12], p.legs[13]);
    let gx = (p.legs[4] + p.legs[12]) * 0.5 - gvy;
    let gy = (p.legs[5] + p.legs[13]) * 0.5 + gvx;
    let dl = (gx - p.legs[4]).powi(2) + (gy - p.legs[5]).powi(2);
    let dr = (gx - p.legs[12]).powi(2) + (gy - p.legs[13]).powi(2);

    if comm & LEFT != 0 {
        if dl > dr && left_foot_blocked {
            p.accs[2] = -3.0 * mvy - 3.0 * mvx;
            p.accs[3] = 3.0 * mvx - 3.0 * mvy;
            p.accs[0] = -mvy;
            p.accs[1] = mvx;
        } else if dl <= dr && right_foot_blocked {
            p.accs[6] = -3.0 * mvy - 3.0 * mvx;
            p.accs[7] = 3.0 * mvx - 3.0 * mvy;
            p.accs[0] = -mvy;
            p.accs[1] = mvx;
        }
    }
    if comm & RIGHT != 0 {
        if dl < dr && left_foot_blocked {
            p.accs[2] = 3.0 * mvy - 3.0 * mvx;
            p.accs[3] = -3.0 * mvx - 3.0 * mvy;
            p.accs[0] = mvy;
            p.accs[1] = -mvx;
        } else if dl >= dr && right_foot_blocked {
            p.accs[6] = 3.0 * mvy - 3.0 * mvx;
            p.accs[7] = -3.0 * mvx - 3.0 * mvy;
            p.accs[0] = mvy;
            p.accs[1] = -mvx;
        }
    }
    if comm & JUMP != 0 && (left_foot_blocked || right_foot_blocked) {
        p.vx -= 4.0 * mvx;
        p.vy -= 4.0 * mvy;
        p.accs[2] -= mvx;
        p.accs[6] -= mvx;
        p.accs[3] -= mvy;
        p.accs[7] -= mvy;
    }

    constrain(p, 0, 1, 4, 5, 25.0);
    constrain(p, 8, 9, 12, 13, 25.0);
    constrain_head(p, 0, 1, 36.0);
    constrain_head(p, 8, 9, 36.0);
    separate_pair(p, 4, 5, 12, 13, gvx, gvy);
    separate_pair(p, 0, 1, 8, 9, gvx, gvy);
    p.vx *= 0.9;
    p.vy *= 0.9;
    if !p.vx.is_finite() {
        p.vx = 0.0;
    }
    if !p.vy.is_finite() {
        p.vy = 0.0;
    }
    for v in &mut p.legs {
        if !v.is_finite() {
            *v = p.x;
        }
    }
}

fn unit_gravity(gvx: f32, gvy: f32) -> (f32, f32) {
    let mut mvx = gvx;
    let mut mvy = gvy;
    let tmp = mvx.abs().max(mvy.abs());
    if tmp < 0.001 {
        mvx = 0.0;
        mvy = 1.0;
    }
    let mag = (mvx * mvx + mvy * mvy).sqrt().max(0.001);
    (mvx / mag, mvy / mag)
}

fn verlet(p: &mut Player, cur: usize, prev: usize, acc: f32) {
    let pp = 2.0 * p.legs[cur] - p.legs[prev] + acc;
    p.legs[prev] = p.legs[cur];
    p.legs[cur] = pp;
}

fn constrain(p: &mut Player, ax: usize, ay: usize, bx: usize, by: usize, rest: f32) {
    let dx = p.legs[ax] - p.legs[bx];
    let dy = p.legs[ay] - p.legs[by];
    let d = rest / (dx * dx + dy * dy + rest) - 0.5;
    p.legs[bx] -= dx * d;
    p.legs[by] -= dy * d;
    p.legs[ax] += dx * d;
    p.legs[ay] += dy * d;
}

fn constrain_head(p: &mut Player, lx: usize, ly: usize, rest: f32) {
    let dx = p.legs[lx] - p.x;
    let dy = p.legs[ly] - p.y;
    let d = rest / (dx * dx + dy * dy + rest) - 0.5;
    p.vx -= dx * d;
    p.vy -= dy * d;
    p.legs[lx] += dx * d;
    p.legs[ly] += dy * d;
}

fn separate_pair(p: &mut Player, ax: usize, ay: usize, bx: usize, by: usize, gvx: f32, gvy: f32) {
    let dx = p.legs[ax] - p.legs[bx];
    let dy = p.legs[ay] - p.legs[by];
    if dx * dx + dy * dy >= 16.0 {
        return;
    }
    let tvx = -gvy;
    let tvy = gvx;
    let mag = (tvx * tvx + tvy * tvy).sqrt();
    if mag < 1e-4 {
        return;
    }
    let (ux, uy) = (tvx / mag, tvy / mag);
    if ax == 4 {
        p.accs[2] -= 0.2 * ux;
        p.accs[3] -= 0.2 * uy;
        p.accs[6] += 0.2 * ux;
        p.accs[7] += 0.2 * uy;
    } else {
        p.accs[0] -= 0.2 * ux;
        p.accs[1] -= 0.2 * uy;
        p.accs[4] += 0.2 * ux;
        p.accs[5] += 0.2 * uy;
    }
}

fn collide_feet(
    sim: &mut ElementSim,
    slot: usize,
    fighter: bool,
    cells: &[Cell],
    w: usize,
    h: usize,
) {
    let p = if fighter {
        &mut sim.fighters[slot]
    } else {
        &mut sim.players[slot]
    };
    for (cx, cy, px, py) in [(4, 5, 6, 7), (12, 13, 14, 15)] {
        if foot_blocked(cells, w, h, p.legs[cx], p.legs[cy]) {
            let t = p.legs[cx];
            p.legs[cx] = p.legs[px];
            p.legs[px] = t;
            let t = p.legs[cy];
            p.legs[cy] = p.legs[py];
            p.legs[py] = t;
        }
    }
}

fn foot_blocked(cells: &[Cell], w: usize, h: usize, x: f32, y: f32) -> bool {
    let xi = x.round() as i32;
    let yi = y.round() as i32;
    if xi < 0 || yi < 0 || xi >= w as i32 || yi >= h as i32 {
        return false;
    }
    let id = cells[yi as usize * w + xi as usize].material;
    is_solid_obstacle(id) && id != STKM && id != STK2 && id != FIGH
}

fn damage_and_pick(
    cells: &mut [Cell],
    sim: &mut ElementSim,
    slot: usize,
    fighter: bool,
    i: usize,
    w: usize,
    h: usize,
    wrap: bool,
) -> bool {
    let (x, y) = ((i % w) as i32, (i / w) as i32);
    if cells[i].temperature < 243.0 {
        let p = if fighter {
            &mut sim.fighters[slot]
        } else {
            &mut sim.players[slot]
        };
        p.life -= 1;
    } else if cells[i].temperature < 309.6 {
        cells[i].temperature += 1.0;
    }
    let life_now = if fighter {
        sim.fighters[slot].life
    } else {
        sim.players[slot].life
    };
    if life_now < 1 {
        kill_actor(cells, sim, slot, fighter, i);
        return true;
    }
    for dy in -2..=2 {
        for dx in -2..=2 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let Some((ni, _, _)) = at(w, h, x, y, dx, dy, wrap) else {
                continue;
            };
            let id = cells[ni].material;
            if id == EMPTY {
                continue;
            }
            if is_holdable(id) {
                let p = if fighter {
                    &mut sim.fighters[slot]
                } else {
                    &mut sim.players[slot]
                };
                p.elem = id;
            }
            if id == PLNT {
                let p = if fighter {
                    &mut sim.fighters[slot]
                } else {
                    &mut sim.players[slot]
                };
                if p.life < 100 {
                    p.life = (p.life + 5).min(100);
                    cells[ni] = Cell::empty();
                    if ni < sim.aux.len() {
                        sim.aux[ni] = Default::default();
                    }
                }
            }
            if id == PRTI {
                if teleport(cells, sim, slot, fighter, i, ni, w, h) {
                    return false;
                }
            }
            let mut damage = 0i16;
            if id == SPRK {
                damage += 32 + (sim.rng_below(20) as i16);
            }
            if !matches!(id, INSL | GLAS | WOOD)
                && (cells[ni].temperature >= 323.0 || cells[ni].temperature <= 243.0)
            {
                damage += 2;
            }
            if id == ACID {
                damage += 5;
            }
            if damage > 0 {
                let p = if fighter {
                    &mut sim.fighters[slot]
                } else {
                    &mut sim.players[slot]
                };
                if damage >= p.life {
                    kill_actor(cells, sim, slot, fighter, i);
                    return true;
                }
                p.life -= damage;
            }
        }
    }
    false
}

fn is_holdable(id: u16) -> bool {
    is_mobile_matter(id) && !matches!(id, STKM | STK2 | FIGH)
}

fn action_pressed(sim: &ElementSim, slot: usize, fighter: bool) -> bool {
    let p = if fighter {
        &sim.fighters[slot]
    } else {
        &sim.players[slot]
    };
    p.comm & ACTION != 0 && p.frames > 4
}

fn use_element(
    cells: &mut [Cell],
    sim: &mut ElementSim,
    slot: usize,
    fighter: bool,
    i: usize,
    w: usize,
    h: usize,
    wrap: bool,
) {
    let p = if fighter {
        &sim.fighters[slot]
    } else {
        &sim.players[slot]
    };
    let dir = if p.pcomm & RIGHT != 0 {
        1
    } else if p.pcomm & LEFT != 0 {
        -1
    } else {
        0
    };
    let x = (i % w) as i32 + dir * 2;
    let y = (i / w) as i32 + if dir == 0 { -2 } else { 0 };
    let Some((ni, _, _)) = at(w, h, x, y, 0, 0, wrap) else {
        return;
    };
    let elem = p.elem;
    if conducts(cells[ni].material) && cells[ni].material != EMPTY {
        super::spark::ignite(
            cells,
            sim,
            ni,
            super::sim::spark_birth_life(cells[ni].material),
        );
        return;
    }
    if cells[ni].is_empty() && elem != EMPTY && elem != STKM && elem != STK2 && elem != FIGH {
        cells[ni].material = elem;
        cells[ni].temperature = if elem == FIRE { 1200.0 } else { 293.15 };
        cells[ni].set_flag(cell_flags::FIXED, is_solid_obstacle(elem));
        if ni < sim.aux.len() {
            sim.aux[ni] = Default::default();
        }
        let p = if fighter {
            &mut sim.fighters[slot]
        } else {
            &mut sim.players[slot]
        };
        p.frames = 0;
        let kick = if dir == 0 { 0.0 } else { dir as f32 * 0.8 };
        cells[ni].vx = kick;
    }
}

fn step_body(
    cells: &mut [Cell],
    sim: &mut ElementSim,
    slot: usize,
    fighter: bool,
    i: usize,
    w: usize,
    h: usize,
) {
    let p = if fighter {
        &mut sim.fighters[slot]
    } else {
        &mut sim.players[slot]
    };
    p.vx = p.vx.clamp(-4.0, 4.0);
    p.vy = p.vy.clamp(-4.0, 4.0);
    p.x += p.vx;
    p.y += p.vy;
    let nx = p.x.round() as i32;
    let ny = p.y.round() as i32;
    if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
        p.x = (i % w) as f32;
        p.y = (i / w) as f32;
        p.vx = 0.0;
        p.vy = 0.0;
        return;
    }
    let ni = ny as usize * w + nx as usize;
    if ni == i {
        return;
    }
    let dest = cells[ni].material;
    if dest != EMPTY && is_solid_obstacle(dest) {
        p.x = (i % w) as f32;
        p.y = (i / w) as f32;
        p.vx *= 0.2;
        p.vy *= 0.2;
        return;
    }
    if dest == EMPTY || is_mobile_matter(dest) {
        sim.exchange(cells, i, ni);
        let p = if fighter {
            &mut sim.fighters[slot]
        } else {
            &mut sim.players[slot]
        };
        p.index = ni as i32;
        if !p.x.is_finite() || !p.y.is_finite() {
            p.x = nx as f32;
            p.y = ny as f32;
            p.vx = 0.0;
            p.vy = 0.0;
        }
    }
}

fn teleport(
    cells: &mut [Cell],
    sim: &mut ElementSim,
    slot: usize,
    fighter: bool,
    i: usize,
    portal: usize,
    w: usize,
    h: usize,
) -> bool {
    let ch = channel_from_temp(cells[portal].temperature);
    let exit = cells.iter().enumerate().find(|(idx, c)| {
        c.material == PRTO && channel_from_temp(c.temperature) == ch && *idx != portal
    });
    let Some((ei, _)) = exit else { return false };
    let ex = (ei % w) as i32;
    let ey = (ei / w) as i32;
    for (dx, dy) in [(0, 1), (0, -1), (1, 0), (-1, 0)] {
        let nx = ex + dx;
        let ny = ey + dy;
        if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
            continue;
        }
        let ni = ny as usize * w + nx as usize;
        if cells[ni].is_empty() {
            sim.exchange(cells, i, ni);
            let p = if fighter {
                &mut sim.fighters[slot]
            } else {
                &mut sim.players[slot]
            };
            p.index = ni as i32;
            p.x = nx as f32;
            p.y = ny as f32;
            init_legs(p, p.x, p.y);
            return true;
        }
    }
    false
}

fn kill_actor(cells: &mut [Cell], sim: &mut ElementSim, slot: usize, fighter: bool, i: usize) {
    cells[i] = Cell::empty();
    if i < sim.aux.len() {
        sim.aux[i] = Default::default();
    }
    if fighter {
        sim.free_fighter_slot(slot);
    } else {
        sim.players[slot] = Player::default();
    }
}

fn steer_fighter(sim: &mut ElementSim, slot: usize) {
    let (fx, fy) = {
        let f = &sim.fighters[slot];
        (f.x, f.y)
    };
    let mut best: Option<(f32, f32, f32)> = None;
    for p in &sim.players {
        if !p.present {
            continue;
        }
        let dx = p.x - fx;
        let dy = p.y - fy;
        let d2 = dx * dx + dy * dy;
        if best.map(|(_, _, b)| d2 < b).unwrap_or(true) {
            best = Some((p.x, p.y, d2));
        }
    }
    let Some((tx, ty, d2)) = best else {
        sim.fighters[slot].comm = 0;
        return;
    };
    let mut comm = 0u8;
    if d2 < 600.0 {
        comm |= ACTION;
    } else if tx < fx {
        comm |= LEFT;
        if ty + 2.0 < fy {
            comm |= JUMP;
        }
    } else {
        comm |= RIGHT;
        if ty + 2.0 < fy {
            comm |= JUMP;
        }
    }
    let left_blocked = {
        let f = &sim.fighters[slot];
        // Jump when the feet were not able to walk last frame and the target is not beside us.
        f.accs[2] == 0.0 && f.accs[6] == 0.0 && (tx - fx).abs() > 1.5 && ty >= fy
    };
    if left_blocked && d2 >= 600.0 {
        comm |= JUMP;
    }
    sim.fighters[slot].comm = comm;
}

pub fn overlays(sim: &ElementSim) -> Vec<(i32, i32, u8, u8, u8)> {
    let mut out = Vec::new();
    for (n, p) in sim.players.iter().enumerate() {
        if p.present {
            let color = if n == 0 {
                (240, 208, 160)
            } else {
                (160, 192, 240)
            };
            push_stick(&mut out, p, color);
        }
    }
    for f in &sim.fighters {
        if f.present {
            push_stick(&mut out, f, (255, 224, 160));
        }
    }
    out
}

fn push_stick(out: &mut Vec<(i32, i32, u8, u8, u8)>, p: &Player, color: (u8, u8, u8)) {
    for (x, y) in [(0, 1), (4, 5), (8, 9), (12, 13)] {
        out.push((
            p.legs[x].round() as i32,
            p.legs[y].round() as i32,
            color.0,
            color.1,
            color.2,
        ));
    }
}
