//! Piston and frame movement.
//!
//! The reference does not flood every connected frame. It walks the straight
//! piston shaft, then a perpendicular line of FRME (at most 15 either side).
//! Each frame pushes or pulls the stack in front of it. If `tmp3` requests a
//! hard failure, a blocked section cancels the whole move and nothing is
//! written. Cell fields and aux records move together.

use realmatter_core::cell::{cell_flags, Cell};

use super::ids::*;
use super::sim::ElementSim;

const MAX_FRAME: i32 = 15;
const DEFAULT_LIMIT: i32 = 31;
const DEFAULT_ARM: i32 = 255;
const FLAG_EXT_OBSTACLE: i32 = 0x01;
const FLAG_EXT_ARMLIMIT: i32 = 0x02;
const FLAG_RET_OBSTACLE: i32 = 0x04;
const FLAG_RET_TOOSHORT: i32 = 0x08;

pub fn update(cells: &mut [Cell], sim: &mut ElementSim, w: usize, h: usize) {
    let n = cells.len().min(sim.aux.len());
    let mut heads = Vec::new();
    for i in 0..n {
        if cells[i].material == PSTN && sim.aux[i].life == 0 {
            heads.push(i);
        }
    }
    for i in heads {
        if i >= cells.len() || cells[i].material != PSTN || sim.aux[i].life != 0 {
            continue;
        }
        let x = (i % w) as i32;
        let y = (i / w) as i32;
        let mut state = 0i32;
        for (dx, dy) in ortho_spark() {
            let nx = x + dx;
            let ny = y + dy;
            if let Some(ni) = idx(w, h, nx, ny) {
                if cells[ni].material == SPRK && sim.aux[ni].life == 3 {
                    state = if sim.aux[ni].ctype == PSCN { 2 } else { 1 };
                }
            }
        }
        if state == 0 {
            continue;
        }
        let max_size = if sim.aux[i].tmp > 0 {
            sim.aux[i].tmp
        } else {
            DEFAULT_LIMIT
        };
        let arm_limit = if sim.aux[i].tmp2 > 0 {
            sim.aux[i].tmp2
        } else {
            DEFAULT_ARM
        };
        let flags = sim.aux[i].tmp3;
        let block = sim.aux[i].ctype;
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let Some(ni) = idx(w, h, x + dx, y + dy) else {
                continue;
            };
            if cells[ni].material != PSTN || sim.aux[ni].life != 0 {
                continue;
            }
            if let Some(()) = drive(
                cells, sim, i, x, y, dx, dy, w, h, state, max_size, arm_limit, flags, block,
            ) {
                break;
            }
        }
    }
}

fn drive(
    cells: &mut [Cell],
    sim: &mut ElementSim,
    origin: usize,
    x: i32,
    y: i32,
    dx: i32,
    dy: i32,
    w: usize,
    h: usize,
    state: i32,
    max_size: i32,
    arm_limit: i32,
    flags: i32,
    block: u16,
) -> Option<()> {
    let mut piston_count = -1i32;
    let mut arm_count = 0i32;
    let mut end_x = x;
    let mut end_y = y;
    let mut found = false;
    let mut nxx = 0i32;
    let mut nyy = 0i32;
    for _step in 0..(w + h + 2) {
        let cx = x + nxx;
        let cy = y + nyy;
        if idx(w, h, cx, cy).is_none() {
            break;
        }
        let ci = idx(w, h, cx, cy).unwrap();
        if cells[ci].material == PSTN {
            if sim.aux[ci].life != 0 {
                arm_count += 1;
            } else if arm_count > 0 {
                end_x = cx;
                end_y = cy;
                found = true;
                break;
            } else {
                piston_count += ((cells[ci].temperature - 268.15) / 10.0).floor() as i32;
            }
        } else if nxx == 0 && nyy == 0 {
            piston_count += ((cells[origin].temperature - 268.15) / 10.0).floor() as i32;
        } else {
            end_x = cx;
            end_y = cy;
            found = true;
            break;
        }
        nxx += dx;
        nyy += dy;
    }
    if !found {
        return None;
    }
    let mut amount = piston_count;
    if state == 2 {
        if arm_count + amount > arm_limit {
            if flags & FLAG_EXT_ARMLIMIT != 0 {
                return None;
            }
            amount = arm_limit - arm_count;
        }
        if amount <= 0 {
            return None;
        }
        let cancel = flags & FLAG_EXT_OBSTACLE != 0;
        let moved = move_stack(
            cells, sim, end_x, end_y, dx, dy, w, h, max_size, amount, false, block, true, 0, cancel,
        );
        if moved > 0 {
            for j in 0..moved {
                if let Some(ci) = idx(w, h, end_x + dx * j, end_y + dy * j) {
                    if cells[ci].is_empty() {
                        cells[ci].material = PSTN;
                        cells[ci].temperature = cells[origin].temperature;
                        cells[ci].set_flag(cell_flags::FIXED, true);
                        sim.aux[ci] = Default::default();
                        sim.aux[ci].life = 1;
                    }
                }
            }
            return Some(());
        }
    } else if state == 1 {
        if amount > arm_count {
            if flags & FLAG_RET_TOOSHORT != 0 {
                return None;
            }
            amount = arm_count;
        }
        if arm_count > 0 && amount > 0 {
            let cancel = flags & FLAG_RET_OBSTACLE != 0;
            // The reference stops after the first shaft that can retract,
            // including a retract that finds nothing to pull.
            let _moved = move_stack(
                cells, sim, end_x, end_y, dx, dy, w, h, max_size, amount, true, block, true, 0,
                cancel,
            );
            return Some(());
        }
    }
    None
}

fn move_stack(
    cells: &mut [Cell],
    sim: &mut ElementSim,
    stack_x: i32,
    stack_y: i32,
    dx: i32,
    dy: i32,
    w: usize,
    h: usize,
    max_size: i32,
    mut amount: i32,
    retract: bool,
    block: u16,
    sticky: bool,
    depth: i32,
    cancel: bool,
) -> i32 {
    let Some(top) = idx(w, h, stack_x, stack_y) else {
        return 0;
    };
    if depth == 0 && cells[top].material == FRME {
        let new_y = i32::from(dx != 0);
        let new_x = i32::from(dy != 0);
        let rdx = if retract { -dx } else { dx };
        let rdy = if retract { -dy } else { dy };
        let mut max_right = MAX_FRAME;
        let mut max_left = MAX_FRAME;
        for c in i32::from(retract)..MAX_FRAME {
            let px = stack_x + c * new_x;
            let py = stack_y + c * new_y;
            let Some(pi) = idx(w, h, px, py) else {
                max_right = c;
                break;
            };
            if cells[pi].material != FRME {
                max_right = c;
                break;
            }
            let spaces = can_move(
                cells, sim, px, py, rdx, rdy, w, h, max_size, amount, retract, block,
            )
            .1;
            if spaces < amount {
                if cancel {
                    return 0;
                }
                amount = spaces;
            }
        }
        for c in 1..MAX_FRAME {
            let px = stack_x - c * new_x;
            let py = stack_y - c * new_y;
            let Some(pi) = idx(w, h, px, py) else {
                max_left = c;
                break;
            };
            if cells[pi].material != FRME {
                max_left = c;
                break;
            }
            let spaces = can_move(
                cells, sim, px, py, rdx, rdy, w, h, max_size, amount, retract, block,
            )
            .1;
            if spaces < amount {
                if cancel {
                    return 0;
                }
                amount = spaces;
            }
        }
        if amount <= 0 {
            return 0;
        }
        for c in 1..max_right {
            let px = stack_x + c * new_x;
            let py = stack_y + c * new_y;
            let sticky_c = idx(w, h, px, py).is_some_and(|pi| sim.aux[pi].tmp == 0);
            move_stack(
                cells, sim, px, py, dx, dy, w, h, max_size, amount, retract, block, sticky_c, 1,
                cancel,
            );
        }
        for c in 1..max_left {
            let px = stack_x - c * new_x;
            let py = stack_y - c * new_y;
            let sticky_c = idx(w, h, px, py).is_some_and(|pi| sim.aux[pi].tmp == 0);
            move_stack(
                cells, sim, px, py, dx, dy, w, h, max_size, amount, retract, block, sticky_c, 1,
                cancel,
            );
        }
        if retract {
            for j in 1..=amount {
                if let Some(ci) = idx(w, h, stack_x + dx * -j, stack_y + dy * -j) {
                    if cells[ci].material == PSTN {
                        cells[ci] = Cell::empty();
                        sim.aux[ci] = Default::default();
                    }
                }
            }
        }
        let sticky_top = sim.aux[top].tmp == 0;
        return move_stack(
            cells, sim, stack_x, stack_y, dx, dy, w, h, max_size, amount, retract, block,
            sticky_top, 1, cancel,
        );
    }

    if retract {
        if depth == 0 {
            for j in 1..=amount {
                if let Some(ci) = idx(w, h, stack_x + dx * -j, stack_y + dy * -j) {
                    if cells[ci].material == PSTN {
                        cells[ci] = Cell::empty();
                        sim.aux[ci] = Default::default();
                    }
                }
            }
        }
        let mut slots = Vec::new();
        let mut cx = stack_x;
        let mut cy = stack_y;
        for _ in 0..max_size.max(0) {
            let Some(ci) = idx(w, h, cx, cy) else { break };
            let id = cells[ci].material;
            if id == EMPTY || (block != 0 && id == block) || (!sticky && id != FRME) {
                break;
            }
            slots.push(ci);
            cx += dx;
            cy += dy;
        }
        if slots.is_empty() {
            return 0;
        }
        for &ci in &slots {
            let sx = (ci % w) as i32;
            let sy = (ci / w) as i32;
            let dest = idx(w, h, sx - dx * amount, sy - dy * amount);
            if let Some(di) = dest {
                if cells[di].is_empty() || di == ci {
                    sim.exchange(cells, ci, di);
                }
            }
        }
        return amount;
    }

    let (pushed, spaces, slots) = can_move(
        cells, sim, stack_x, stack_y, dx, dy, w, h, max_size, amount, false, block,
    );
    let current = pushed + spaces;
    if current <= 0 {
        return 0;
    }
    let mut possible = 0i32;
    for slot in slots.iter().rev() {
        if *slot < 0 {
            possible += 1;
            continue;
        }
        if possible == 0 {
            continue;
        }
        let ci = *slot as usize;
        if ci >= cells.len() {
            continue;
        }
        let sx = (ci % w) as i32;
        let sy = (ci / w) as i32;
        if let Some(di) = idx(w, h, sx + dx * possible, sy + dy * possible) {
            if cells[di].is_empty() {
                sim.exchange(cells, ci, di);
            }
        }
    }
    possible
}

/// `(pushed, spaces, slot list)` where `-1` is an empty gap.
fn can_move(
    cells: &[Cell],
    _sim: &ElementSim,
    stack_x: i32,
    stack_y: i32,
    dx: i32,
    dy: i32,
    w: usize,
    h: usize,
    max_size: i32,
    amount: i32,
    retract: bool,
    block: u16,
) -> (i32, i32, Vec<i32>) {
    if amount <= 0 {
        return (0, 0, Vec::new());
    }
    let mut spaces = 0i32;
    let mut current = 0i32;
    let mut slots = Vec::new();
    let mut cx = stack_x;
    let mut cy = stack_y;
    let limit = (max_size + amount).min((w + h) as i32);
    while current < limit {
        let Some(ci) = idx(w, h, cx, cy) else { break };
        let id = cells[ci].material;
        // STNE stands in for the reference wall map. Insulation is a particle
        // and is pushed with the stack, the same as any other solid.
        if id == STNE || (block != 0 && id == block) {
            return (current - spaces, spaces, slots);
        }
        if id == EMPTY {
            spaces += 1;
            slots.push(-1);
            current += 1;
            if spaces >= amount {
                break;
            }
        } else if current - spaces < max_size
            && (!retract || (id == FRME && cx == stack_x && cy == stack_y))
        {
            slots.push(ci as i32);
            current += 1;
        } else {
            return (current - spaces, spaces, slots);
        }
        cx += dx;
        cy += dy;
    }
    (current - spaces, spaces, slots)
}

fn ortho_spark() -> [(i32, i32); 8] {
    [
        (1, 0),
        (-1, 0),
        (0, 1),
        (0, -1),
        (2, 0),
        (-2, 0),
        (0, 2),
        (0, -2),
    ]
}

fn idx(w: usize, h: usize, x: i32, y: i32) -> Option<usize> {
    if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 {
        return None;
    }
    Some(y as usize * w + x as usize)
}
