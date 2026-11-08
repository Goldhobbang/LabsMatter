//! Spark conduction.
//!
//! Life is decremented before this pass, matching the reference frame prologue.
//! A spark created during the pass has life 4 and does not conduct until the
//! next frame, so a wire cannot flood in one tick. Conduction reaches the
//! chebyshev-2 neighborhood except the far corners, and a cell of insulation
//! on the midpoint blocks the jump.

use realmatter_core::cell::{cell_flags, Cell};

use super::ids::*;
use super::sim::{at, midpoint_blocks, spark_birth_life, warm_new_spark, ElementSim};

pub fn decay_life(cells: &mut [Cell], sim: &mut ElementSim) {
    let n = cells.len().min(sim.aux.len());
    for i in 0..n {
        if sim.aux[i].life > 0 && life_decays(cells[i].material) {
            sim.aux[i].life -= 1;
        }
    }
}

pub fn update(cells: &mut [Cell], sim: &mut ElementSim, w: usize, h: usize, wrap: bool) {
    let n = cells.len().min(sim.aux.len());
    // Recover the previous shell's spark, which stored the conductor id in vx.
    for i in 0..n {
        if cells[i].material == SPRK && sim.aux[i].ctype == 0 {
            let guessed = cells[i].vx.round() as u16;
            sim.aux[i].ctype = if conducts(guessed) { guessed } else { IRON };
            if sim.aux[i].life <= 0 {
                sim.aux[i].life = 4;
            }
            cells[i].vx = 0.0;
        }
    }
    for i in 0..n {
        if cells[i].material == SPRK && sim.aux[i].life <= 0 {
            restore(cells, sim, i);
        }
    }
    for i in 0..n {
        if cells[i].material != SPRK {
            continue;
        }
        let (x, y) = ((i % w) as i32, (i / w) as i32);
        conduct_from(cells, sim, i, x, y, w, h, wrap);
    }
}

fn restore(cells: &mut [Cell], sim: &mut ElementSim, i: usize) {
    let mut ct = sim.aux[i].ctype;
    if ct == 0 || ct == SPRK || ct > BRMT {
        ct = METL;
    }
    if ct == RSST {
        cells[i] = Cell::empty();
        sim.aux[i] = Default::default();
        return;
    }
    let mut life: i16 = 4;
    if ct == WATR {
        life = 64;
    } else if ct == SLTW {
        life = 54;
    } else if ct == SWCH {
        life = 14;
    }
    if matches!(ct, WATR | SLTW | PSCN | NSCN | INWR) {
        cells[i].temperature = 295.15;
    }
    cells[i].material = ct;
    cells[i].vx = 0.0;
    cells[i].vy = 0.0;
    sim.aux[i].ctype = 0;
    sim.aux[i].life = life;
    cells[i].set_flag(cell_flags::FIXED, is_solid_obstacle(ct) || ct == SPRK);
    if matches!(ct, WATR | SLTW | RSST | SOAP) {
        cells[i].set_flag(cell_flags::FIXED, false);
    }
}

fn conduct_from(
    cells: &mut [Cell],
    sim: &mut ElementSim,
    i: usize,
    x: i32,
    y: i32,
    w: usize,
    h: usize,
    wrap: bool,
) {
    let sender = {
        let ct = sim.aux[i].ctype;
        if ct == 0 {
            METL
        } else {
            ct
        }
    };
    if sender == SPRK {
        cells[i] = Cell::empty();
        sim.aux[i] = Default::default();
        return;
    }
    if sender == NTCT || sender == PTCT {
        if cells[i].temperature > 295.0 {
            cells[i].temperature -= 2.5;
        }
    }
    if sender == IRON {
        electrolyze(cells, sim, x, y, w, h, wrap);
    }
    let life = sim.aux[i].life;
    for dy in -2..=2 {
        for dx in -2..=2 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let Some((ni, nx, ny)) = at(w, h, x, y, dx, dy, wrap) else {
                continue;
            };
            if ni >= cells.len() {
                continue;
            }
            let receiver = cells[ni].material;
            if (dx.abs() + dy.abs()) >= 4 && sender != SWCH && receiver != SWCH {
                continue;
            }
            let insulated = midpoint_blocks(cells, w, h, x, y, nx, ny);
            if handle_special(
                cells, sim, i, ni, sender, receiver, life, dx, dy, insulated, w, h, wrap,
            ) {
                continue;
            }
            if insulated || !((conducts(receiver)) || receiver == INST) {
                continue;
            }
            if !allow(
                sender,
                receiver,
                cells[i].temperature,
                cells[ni].temperature,
            ) {
                continue;
            }
            if sim.aux[ni].life != 0 {
                continue;
            }
            if receiver == WATR || receiver == SLTW {
                if life < 3 {
                    ignite(cells, sim, ni, spark_birth_life(receiver));
                }
            } else if receiver == INST {
                if life < 4 {
                    flood_inst(cells, sim, nx, ny, w, h);
                }
            } else if life < 4 {
                ignite(cells, sim, ni, spark_birth_life(receiver));
            }
        }
    }
}

/// Returns true when the neighbor has been fully handled (do not also conduct).
fn handle_special(
    cells: &mut [Cell],
    sim: &mut ElementSim,
    _i: usize,
    ni: usize,
    sender: u16,
    receiver: u16,
    life: i16,
    dx: i32,
    dy: i32,
    insulated: bool,
    w: usize,
    h: usize,
    _wrap: bool,
) -> bool {
    match receiver {
        SWCH if !insulated && life < 4 => {
            if sender == PSCN && sim.aux[ni].life < 10 {
                sim.aux[ni].life = 10;
            } else if sender == NSCN {
                sim.aux[ni].ctype = 0;
                sim.aux[ni].life = 9;
            }
            true
        }
        SPRK if !insulated && life < 4 => {
            if sim.aux[ni].ctype == SWCH && sender == NSCN {
                cells[ni].material = SWCH;
                sim.aux[ni].ctype = 0;
                sim.aux[ni].life = 9;
                cells[ni].set_flag(cell_flags::FIXED, true);
            } else if matches!(sim.aux[ni].ctype, NTCT | PTCT) && sender == METL {
                cells[ni].temperature = 473.0;
            }
            true
        }
        PUMP | GPMP | HSWC | PBCN | PCLN if life < 4 => {
            if sender == PSCN {
                sim.aux[ni].life = 10;
            } else if sender == NSCN && sim.aux[ni].life >= 10 {
                sim.aux[ni].life = 9;
            }
            true
        }
        LCRY if dx.abs() < 2 && dy.abs() < 2 && life < 4 => {
            if sender == PSCN && sim.aux[ni].tmp == 0 {
                sim.aux[ni].tmp = 2;
            } else if sender == NSCN && sim.aux[ni].tmp == 3 {
                sim.aux[ni].tmp = 1;
            }
            true
        }
        PPIP | PIPE if life == 3 && !insulated && matches!(sender, NSCN | PSCN | INST) => {
            if receiver == PPIP {
                let pause = sender == NSCN;
                let _ = (w, h);
                flood_ppip(cells, sim, ni, w, h, pause);
            }
            true
        }
        NTCT | PTCT | INWR if sender == METL && !insulated && life < 4 => {
            cells[ni].temperature = 473.0;
            receiver == NTCT || receiver == PTCT
        }
        EMP if sim.aux[ni].life == 0 && life > 0 && life < 4 => {
            sim.aux[ni].life = 220;
            true
        }
        _ => false,
    }
}

fn allow(sender: u16, receiver: u16, sender_temp: f32, receiver_temp: f32) -> bool {
    if receiver == sender && receiver != INST {
        return true;
    }
    match sender {
        INST => return receiver == NSCN,
        SWCH => {
            if matches!(receiver, PSCN | NSCN | WATR | SLTW | NTCT | PTCT | INWR) {
                return false;
            }
        }
        NTCT => {
            return receiver == PSCN || (receiver == NSCN && sender_temp > 373.0);
        }
        PTCT => {
            return receiver == PSCN || (receiver == NSCN && sender_temp < 373.0);
        }
        INWR => return receiver == NSCN || receiver == PSCN,
        _ => {}
    }
    match receiver {
        NTCT => return sender == NSCN || (sender == PSCN && receiver_temp > 373.0),
        PTCT => return sender == NSCN || (sender == PSCN && receiver_temp < 373.0),
        INWR => return sender == NSCN || sender == PSCN,
        INST => return sender == PSCN,
        PSCN => return sender != NSCN,
        _ => {}
    }
    true
}

pub fn ignite(cells: &mut [Cell], sim: &mut ElementSim, i: usize, life: i16) {
    if i >= cells.len() || sim.aux[i].life != 0 || cells[i].material == SPRK {
        return;
    }
    let underlying = cells[i].material;
    if !(conducts(underlying) || underlying == INST) {
        return;
    }
    sim.aux[i].ctype = underlying;
    sim.aux[i].life = life;
    cells[i].material = SPRK;
    warm_new_spark(&mut cells[i], underlying);
    cells[i].set_flag(cell_flags::FIXED, true);
}

fn electrolyze(
    cells: &mut [Cell],
    sim: &mut ElementSim,
    x: i32,
    y: i32,
    w: usize,
    h: usize,
    wrap: bool,
) {
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let Some((ni, _, _)) = at(w, h, x, y, dx, dy, wrap) else {
                continue;
            };
            if !matches!(cells[ni].material, WATR | SLTW) {
                continue;
            }
            let roll = sim.rng_below(100);
            if roll == 0 {
                cells[ni].material = CO2;
                cells[ni].set_flag(cell_flags::FIXED, false);
            } else if roll < 3 {
                cells[ni].material = HYGN;
                cells[ni].set_flag(cell_flags::FIXED, false);
            }
        }
    }
}

fn flood_inst(cells: &mut [Cell], sim: &mut ElementSim, sx: i32, sy: i32, w: usize, h: usize) {
    if !sparkable_inst(cells, sim, sx, sy, w, h) {
        return;
    }
    let mut stack = vec![(sx, sy)];
    let mut guard = 0;
    while let Some((x, y)) = stack.pop() {
        guard += 1;
        if guard > w.saturating_mul(h).saturating_add(8) {
            break;
        }
        if !sparkable_inst(cells, sim, x, y, w, h) {
            continue;
        }
        let mut x1 = x;
        let mut x2 = x;
        while sparkable_inst(cells, sim, x1 - 1, y, w, h) {
            x1 -= 1;
        }
        while sparkable_inst(cells, sim, x2 + 1, y, w, h) {
            x2 += 1;
        }
        for xx in x1..=x2 {
            if let Some((i, _, _)) = at(w, h, xx, y, 0, 0, false) {
                ignite(cells, sim, i, 4);
            }
        }
        if y >= 2
            && x1 == x2
            && is_inst(cells, sim, x1 - 1, y - 1, w, h)
            && is_inst(cells, sim, x1, y - 1, w, h)
            && is_inst(cells, sim, x1 + 1, y - 1, w, h)
            && !is_inst(cells, sim, x1 - 1, y - 2, w, h)
            && is_inst(cells, sim, x1, y - 2, w, h)
            && !is_inst(cells, sim, x1 + 1, y - 2, w, h)
        {
            if sparkable_inst(cells, sim, x1, y - 2, w, h) {
                stack.push((x1, y - 2));
            }
        } else if y >= 1 {
            for xx in x1..=x2 {
                if sparkable_inst(cells, sim, xx, y - 1, w, h)
                    && (xx == x1
                        || xx == x2
                        || y + 1 >= h as i32
                        || !is_inst(cells, sim, xx, y + 1, w, h)
                        || is_inst(cells, sim, xx + 1, y + 1, w, h)
                        || is_inst(cells, sim, xx - 1, y + 1, w, h))
                {
                    stack.push((xx, y - 1));
                }
            }
        }
        if y + 2 < h as i32
            && x1 == x2
            && is_inst(cells, sim, x1 - 1, y + 1, w, h)
            && is_inst(cells, sim, x1, y + 1, w, h)
            && is_inst(cells, sim, x1 + 1, y + 1, w, h)
            && !is_inst(cells, sim, x1 - 1, y + 2, w, h)
            && is_inst(cells, sim, x1, y + 2, w, h)
            && !is_inst(cells, sim, x1 + 1, y + 2, w, h)
        {
            if sparkable_inst(cells, sim, x1, y + 2, w, h) {
                stack.push((x1, y + 2));
            }
        } else if y + 1 < h as i32 {
            for xx in x1..=x2 {
                if sparkable_inst(cells, sim, xx, y + 1, w, h)
                    && (xx == x1
                        || xx == x2
                        || y <= 0
                        || !is_inst(cells, sim, xx, y - 1, w, h)
                        || is_inst(cells, sim, xx + 1, y - 1, w, h)
                        || is_inst(cells, sim, xx - 1, y - 1, w, h))
                {
                    stack.push((xx, y + 1));
                }
            }
        }
    }
}

fn sparkable_inst(cells: &[Cell], sim: &ElementSim, x: i32, y: i32, w: usize, h: usize) -> bool {
    let Some((i, _, _)) = at(w, h, x, y, 0, 0, false) else {
        return false;
    };
    cells[i].material == INST && sim.aux.get(i).is_some_and(|a| a.life == 0)
}

fn is_inst(cells: &[Cell], sim: &ElementSim, x: i32, y: i32, w: usize, h: usize) -> bool {
    let Some((i, _, _)) = at(w, h, x, y, 0, 0, false) else {
        return false;
    };
    cells[i].material == INST
        || (cells[i].material == SPRK && sim.aux.get(i).is_some_and(|a| a.ctype == INST))
}

fn flood_ppip(
    cells: &mut [Cell],
    sim: &mut ElementSim,
    start: usize,
    w: usize,
    h: usize,
    pause: bool,
) {
    if start >= cells.len() || !matches!(cells[start].material, PPIP) {
        return;
    }
    let mut stack = vec![start];
    let mut seen = vec![false; cells.len()];
    let mut guard = 0;
    while let Some(i) = stack.pop() {
        guard += 1;
        if guard > cells.len() + 2 || seen[i] {
            continue;
        }
        seen[i] = true;
        if cells[i].material != PPIP {
            continue;
        }
        if pause {
            sim.aux[i].tmp |= 1;
        } else {
            sim.aux[i].tmp &= !1;
        }
        let x = (i % w) as i32;
        let y = (i / w) as i32;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                if let Some((ni, _, _)) = at(w, h, x, y, dx, dy, false) {
                    if ni < cells.len() && !seen[ni] && cells[ni].material == PPIP {
                        stack.push(ni);
                    }
                }
            }
        }
    }
}

pub fn battery(cells: &mut [Cell], sim: &mut ElementSim, w: usize, h: usize, wrap: bool) {
    let n = cells.len().min(sim.aux.len());
    let mut sources = Vec::new();
    for i in 0..n {
        if cells[i].material == BTRY {
            sources.push(i);
        }
    }
    for i in sources {
        let x = (i % w) as i32;
        let y = (i / w) as i32;
        for dy in -2..=2 {
            for dx in -2..=2 {
                if dx == 0 && dy == 0 || (dx as i32).abs() + (dy as i32).abs() >= 4 {
                    continue;
                }
                let Some((ni, nx, ny)) = at(w, h, x, y, dx, dy, wrap) else {
                    continue;
                };
                if midpoint_blocks(cells, w, h, x, y, nx, ny) {
                    continue;
                }
                let rt = cells[ni].material;
                if conducts(rt)
                    && !matches!(rt, WATR | SLTW | NTCT | PTCT | INWR)
                    && sim.aux[ni].life == 0
                {
                    ignite(cells, sim, ni, 4);
                }
            }
        }
    }
}
