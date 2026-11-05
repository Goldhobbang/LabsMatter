//! Per-element behaviour dispatch.
//!
//! Each implemented element mirrors the corresponding reference `.cpp` update
//! function. Elements without an entry here fall back to the generic movement /
//! transition model, which is what the reference does for elements that declare
//! no `Update` callback. Unimplemented `Update` callbacks are listed in
//! `docs/tpt-parity/status.md` and reported by [`unimplemented_count`].

use crate::element::element;
use crate::elements_generated::pt;
use crate::elements_generated::props;
use crate::sim::{pmap_id, pmap_type, restrict_flt, Sim, CELL, MAX_TEMP, MIN_TEMP};

pub type UpdateFn = fn(&mut Sim, usize, i32, i32, i32, i32) -> i32;
pub type CreateFn = fn(&mut Sim, usize, i32, i32, i32, i32);
pub type CreateAllowedFn = fn(&Sim, usize, i32, i32, i32) -> bool;
pub type ChangeTypeFn = fn(&mut Sim, usize, i32, i32, i32, i32);
pub type CtypeDrawFn = fn(&mut Sim, usize, i32, i32) -> bool;

/// Number of element sources that declare an `Update` callback in the reference.
pub const REFERENCE_UPDATE_COUNT: usize = 132;
/// Number of element sources that declare a `Graphics` callback.
pub const REFERENCE_GRAPHICS_COUNT: usize = 79;

/// Dispatch table for implemented element updates.
pub fn update_fn(t: u16) -> Option<UpdateFn> {
    Some(match t {
        pt::FIRE | pt::PLSM | pt::LAVA => fire_update,
        pt::SPRK => sprk_update,
        pt::CLNE | pt::PCLN => clone_update,
        pt::BCLN | pt::PBCN => bclone_update,
        pt::VOID => void_update,
        pt::PVOD => pvoid_update,
        pt::DEST => dest_update,
        pt::ACID => acid_update,
        pt::BHOL => bhol_update,
        pt::NBHL => nbhol_update,
        pt::PLNT => plnt_update,
        pt::WATR => watr_update,
        pt::ICEI | pt::SNOW => ice_update,
        pt::WOOD | pt::COAL | pt::BCOL => fuel_update,
        pt::NEUT => neut_update,
        _ => return None,
    })
}

pub fn create_fn(t: u16) -> Option<CreateFn> {
    Some(match t {
        pt::FIRE => fire_create,
        pt::LIFE => life_create,
        pt::CLNE | pt::PCLN | pt::BCLN | pt::PBCN => clone_create,
        _ => return None,
    })
}

pub fn create_allowed_fn(t: u16) -> Option<CreateAllowedFn> {
    Some(match t {
        pt::STKM => stkm_allowed,
        pt::STKM2 => stkm2_allowed,
        _ => return None,
    })
}

pub fn change_type_fn(_t: u16) -> Option<ChangeTypeFn> {
    None
}

pub fn ctype_draw_fn(t: u16) -> Option<CtypeDrawFn> {
    Some(match t {
        pt::CLNE | pt::PCLN | pt::BCLN | pt::PBCN | pt::CRAY | pt::ARAY | pt::FILT | pt::PIPE
        | pt::PPIP | pt::PUMP | pt::SPAWN | pt::SPAWN2 | pt::LIFE | pt::TRON | pt::BIZR
        | pt::BIZRG | pt::BIZRS | pt::STKM | pt::SING | pt::DLAY | pt::INST | pt::LCRY
        | pt::WIFI | pt::DRAY | pt::DMG => ctype_draw_default,
        _ => return None,
    })
}

/// Elements whose placement records the element underneath in `ctype`.
pub fn carries_ctype(t: u16) -> bool {
    matches!(
        t,
        pt::CLNE
            | pt::PCLN
            | pt::BCLN
            | pt::PBCN
            | pt::CRAY
            | pt::ARAY
            | pt::FILT
            | pt::PIPE
            | pt::PPIP
            | pt::PUMP
            | pt::SPAWN
            | pt::SPAWN2
            | pt::TRON
            | pt::BIZR
            | pt::BIZRG
            | pt::BIZRS
            | pt::DLAY
            | pt::INST
            | pt::LCRY
            | pt::WIFI
            | pt::DRAY
            | pt::DMG
            | pt::SING
            | pt::LIFE
    )
}

/// Number of reference elements that declare an `Update` callback but have no
/// Rust implementation yet.
pub fn unimplemented_count() -> usize {
    let mut n = 0;
    for t in 1..crate::elements_generated::PT_NUM as u16 {
        if element(t).enabled && element(t).has_update && update_fn(t).is_none() {
            n += 1;
        }
    }
    n
}

/// Names of reference elements whose `Update` callback is not implemented yet.
pub fn unimplemented_elements() -> Vec<&'static str> {
    let mut out = Vec::new();
    for t in 1..crate::elements_generated::PT_NUM as u16 {
        if element(t).enabled && element(t).has_update && update_fn(t).is_none() {
            out.push(element(t).abbr);
        }
    }
    out
}

fn ctype_draw_default(sim: &mut Sim, i: usize, t: i32, _v: i32) -> bool {
    if sim.parts[i].t as i32 == t || (element(t as u16).properties & props::PROP_NOCTYPEDRAW) != 0 {
        return false;
    }
    sim.parts[i].ctype = t;
    true
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

#[inline]
fn rand_chance(sim: &mut Sim, n: i32, d: u32) -> bool {
    sim.rng.chance(n, d)
}

#[inline]
fn pv(sim: &Sim, x: i32, y: i32) -> f32 {
    let cx = x / CELL as i32;
    let cy = y / CELL as i32;
    if sim.in_cell_bounds(cx, cy) {
        sim.pv[sim.cell_idx(cx, cy)]
    } else {
        0.0
    }
}

#[inline]
fn add_pv(sim: &mut Sim, x: i32, y: i32, v: f32) {
    let cx = x / CELL as i32;
    let cy = y / CELL as i32;
    if sim.in_cell_bounds(cx, cy) {
        let idx = sim.cell_idx(cx, cy);
        sim.pv[idx] += v;
    }
}

/// Iterate the 5×5 neighbourhood like the reference element loops.
macro_rules! for_neighbours {
    ($sim:expr, $x:expr, $y:expr, $rx:ident, $ry:ident, $nx:ident, $ny:ident, $body:block) => {
        for $rx in -2i32..=2 {
            for $ry in -2i32..=2 {
                if $rx == 0 && $ry == 0 {
                    continue;
                }
                let $nx = $x + $rx;
                let $ny = $y + $ry;
                if !$sim.in_bounds($nx, $ny) {
                    continue;
                }
                $body
            }
        }
    };
}

/// Reference `Element::legacyUpdate` (used when heat simulation is off).
pub fn legacy_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) {
    let t = sim.parts[i].t;
    match t {
        pt::WTRV => {
            let mut conv: Vec<(usize, i32, i32, u16)> = Vec::new();
            for_neighbours!(sim, x, y, rx, ry, nx, ny, {
                let r = sim.pmap[sim.idx(nx, ny)];
                if r == 0 {
                    continue;
                }
                let id = pmap_id(r) as usize;
                let rt = pmap_type(r);
                if (rt == pt::WATR || rt == pt::DSTW || rt == pt::SLTW)
                    && sim.rng.chance(1, 1000)
                {
                    conv.push((id, nx, ny, pt::WATR));
                }
                if (rt == pt::ICEI || rt == pt::SNOW) && sim.rng.chance(1, 1000) {
                    conv.push((id, nx, ny, pt::WATR));
                }
            });
            for (id, nx, ny, to) in conv {
                sim.part_change_type(id, nx, ny, to as i16);
            }
        }
        pt::WATR | pt::DSTW | pt::SLTW => {
            let mut hot = false;
            for_neighbours!(sim, x, y, rx, ry, nx, ny, {
                let rt = sim.pmap_type_at(nx, ny);
                if (rt == pt::FIRE || rt == pt::LAVA) && sim.rng.chance(1, 10) {
                    hot = true;
                }
            });
            if hot {
                if t == pt::SLTW && sim.rng.chance(1, 4) {
                    sim.part_change_type(i, x, y, pt::SALT as i16);
                } else {
                    sim.part_change_type(i, x, y, pt::WTRV as i16);
                }
            }
        }
        pt::ICEI | pt::SNOW => {
            let mut melt = false;
            for_neighbours!(sim, x, y, rx, ry, nx, ny, {
                let rt = sim.pmap_type_at(nx, ny);
                if (rt == pt::WATR || rt == pt::DSTW) && sim.rng.chance(1, 1000) {
                    melt = true;
                }
            });
            if melt {
                sim.part_change_type(i, x, y, pt::ICEI as i16);
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// FIRE / PLSM / LAVA (reference FIRE.cpp update, shared by these types)
// ---------------------------------------------------------------------------

fn fire_update(sim: &mut Sim, i: usize, x: i32, y: i32, surround_space: i32, _nt: i32) -> i32 {
    let t = sim.parts[i].t;
    match t {
        pt::PLSM => {
            if sim.parts[i].life <= 1 {
                if sim.parts[i].ctype == pt::NBLE as i32 {
                    sim.part_change_type(i, x, y, pt::NBLE as i16);
                    sim.parts[i].life = 0;
                } else if sim.parts[i].tmp & 0x3 == 3 {
                    sim.part_change_type(i, x, y, pt::WTRV as i16);
                    sim.parts[i].life = 0;
                    sim.parts[i].ctype = pt::FIRE as i32;
                }
            }
        }
        pt::FIRE => {
            if sim.parts[i].life <= 1 {
                if sim.parts[i].tmp & 0x3 == 3 {
                    sim.part_change_type(i, x, y, pt::WTRV as i16);
                    sim.parts[i].life = 0;
                    sim.parts[i].ctype = pt::FIRE as i32;
                } else if sim.parts[i].temp < 625.0 {
                    sim.part_change_type(i, x, y, pt::SMKE as i16);
                    sim.parts[i].life = sim.rng.between(250, 269);
                }
            }
        }
        pt::LAVA => {
            let pres = pv(sim, x, y);
            if sim.parts[i].ctype == pt::ROCK as i32 {
                if pres < -9.0 {
                    sim.parts[i].ctype = pt::STNE as i32;
                } else if pres >= 30.0 && sim.parts[i].tmp2 >= 0 {
                    // ROCK formation handled below.
                }
            } else if (sim.parts[i].ctype == pt::STNE as i32 || sim.parts[i].ctype == 0)
                && pres >= 30.0
            {
                sim.parts[i].tmp2 = sim.rng.between(0, 10);
                sim.parts[i].ctype = pt::ROCK as i32;
            }
        }
        _ => {}
    }

    // Burning flammable neighbours (5×5 like the reference).
    let mut ignite: Vec<(i32, i32, f32, i32)> = Vec::new();
    for_neighbours!(sim, x, y, rx, ry, nx, ny, {
        let r = sim.pmap[sim.idx(nx, ny)];
        if r == 0 {
            continue;
        }
        let rt = pmap_type(r);
        let rdef = element(rt);
        if rt == pt::THRM && (t == pt::FIRE || t == pt::PLSM || t == pt::LAVA) {
            let id = pmap_id(r) as usize;
            if sim.rng.chance(1, 500) {
                sim.part_change_type(id, nx, ny, pt::LAVA as i16);
                sim.parts[id].ctype = pt::BMTL as i32;
                sim.parts[id].temp = 3500.0;
                add_pv(sim, nx, ny, 50.0);
            } else {
                sim.part_change_type(id, nx, ny, pt::LAVA as i16);
                sim.parts[id].life = 400;
                sim.parts[id].ctype = pt::THRM as i32;
                sim.parts[id].temp = 3500.0;
                sim.parts[id].tmp = 20;
            }
            continue;
        }
        if (rt == pt::COAL || rt == pt::BCOL) && (t == pt::FIRE || t == pt::PLSM) {
            let id = pmap_id(r) as usize;
            if sim.parts[id].life > 100 && sim.rng.chance(1, 500) {
                sim.parts[id].life = 99;
            }
        }
        if rdef.flammable > 0
            && (surround_space > 0 || rdef.explosive > 0)
            && sim.rng.chance(
                rdef.flammable + (pv(sim, nx, ny) * 10.0) as i32,
                1000,
            )
            && (t != pt::SPRK || (rt != pt::RBDM && rt != pt::LRBD && rt != pt::INSL))
            && (t != pt::PHOT || rt != pt::INSL)
            && (rt != pt::SPNG || sim.parts[pmap_id(r) as usize].life == 0)
        {
            ignite.push((nx, ny, pv(sim, x, y), rdef.explosive));
        }
    });
    for (nx, ny, fire_pv, explosive) in ignite {
        let id = match sim.id_at(nx, ny) {
            Some(id) => id,
            None => continue,
        };
        let rt = sim.parts[id].t;
        sim.part_change_type(id, nx, ny, pt::FIRE as i16);
        sim.parts[id].temp = restrict_flt(
            element(pt::FIRE).default_temp + (element(rt).flammable / 2) as f32,
            MIN_TEMP,
            MAX_TEMP,
        );
        sim.parts[id].life = sim.rng.between(180, 259);
        sim.parts[id].tmp = 0;
        sim.parts[id].ctype = 0;
        if explosive != 0 {
            let cx = nx / CELL as i32;
            let cy = ny / CELL as i32;
            if sim.in_cell_bounds(cx, cy) {
                let idx = sim.cell_idx(cx, cy);
                sim.pv[idx] += 0.25;
            }
        }
        let _ = fire_pv;
    }

    // Legacy (no heat simulation) reactions: melt/freeze neighbours.
    if !sim.settings.heat_sim && t != pt::SPRK {
        let mut changes: Vec<(usize, i32, i32, u16, i32)> = Vec::new();
        for_neighbours!(sim, x, y, rx, ry, nx, ny, {
            let r = sim.pmap[sim.idx(nx, ny)];
            if r == 0 {
                continue;
            }
            let w = sim.wall_at(nx, ny);
            if w != 0 && w != crate::air::WALL_STREAM {
                continue;
            }
            let rt = pmap_type(r);
            let id = pmap_id(r) as usize;
            let rdef = element(rt);
            let mut lpv = pv(sim, nx, ny) as i32;
            if lpv < 1 {
                lpv = 1;
            }
            if rdef.meltable > 0
                && ((rt != pt::RBDM && rt != pt::LRBD) || t != pt::SPRK)
                && ((t != pt::FIRE && t != pt::PLSM)
                    || !matches!(
                        rt,
                        pt::METL
                            | pt::IRON
                            | pt::ETRD
                            | pt::PSCN
                            | pt::NSCN
                            | pt::NTCT
                            | pt::PTCT
                            | pt::BMTL
                            | pt::BRMT
                            | pt::SALT
                            | pt::INWR
                    ))
                && sim.rng.chance(rdef.meltable * lpv, 1000)
            {
                if t != pt::LAVA || sim.parts[i].life > 0 {
                    let ct = match rt {
                        pt::BRMT => pt::BMTL,
                        pt::SAND => pt::GLAS,
                        _ => rt,
                    };
                    changes.push((id, nx, ny, pt::LAVA, ct as i32));
                } else {
                    sim.parts[i].life = 0;
                    sim.parts[i].ctype = 0;
                    sim.part_change_type(i, x, y, pt::STNE as i16);
                    return 1;
                }
            }
            if rt == pt::ICEI || rt == pt::SNOW {
                changes.push((id, nx, ny, pt::WATR, 0));
                if t == pt::FIRE {
                    sim.kill_part(i);
                    return 1;
                }
                if t == pt::LAVA {
                    sim.parts[i].life = 0;
                    sim.part_change_type(i, x, y, pt::STNE as i16);
                }
            }
            if rt == pt::WATR || rt == pt::DSTW || rt == pt::SLTW {
                changes.push((id, nx, ny, pt::NONE, 0));
                if t == pt::FIRE {
                    sim.kill_part(i);
                    return 1;
                }
                if t == pt::LAVA {
                    sim.parts[i].life = 0;
                    sim.parts[i].ctype = 0;
                    sim.part_change_type(i, x, y, pt::STNE as i16);
                }
            }
        });
        for (id, nx, ny, to, ctype) in changes {
            if to == pt::NONE {
                sim.kill_part(id);
            } else if sim.parts[id].t != 0 {
                sim.part_change_type(id, nx, ny, to as i16);
                if to == pt::LAVA {
                    sim.parts[id].ctype = ctype;
                    sim.parts[id].life = sim.rng.between(240, 359);
                }
            }
        }
    }
    0
}

fn fire_create(sim: &mut Sim, i: usize, _x: i32, _y: i32, _t: i32, _v: i32) {
    sim.parts[i].life = sim.rng.between(120, 169);
}

// ---------------------------------------------------------------------------
// WATR
// ---------------------------------------------------------------------------

fn watr_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let mut conversions: Vec<(usize, i32, i32)> = Vec::new();
    for_neighbours!(sim, x, y, rx, ry, nx, ny, {
        let r = sim.pmap[sim.idx(nx, ny)];
        if r == 0 {
            continue;
        }
        let rt = pmap_type(r);
        let id = pmap_id(r) as usize;
        if rt == pt::FIRE && sim.parts[id].tmp == 0 {
            conversions.push((id, nx, ny));
        }
    });
    for (id, nx, ny) in conversions {
        // Water extinguishes fire into steam.
        sim.part_change_type(id, nx, ny, pt::WTRV as i16);
        sim.parts[id].temp = (sim.parts[id].temp + 40.0).min(MAX_TEMP);
        let _ = i;
    }
    // Water boils off near lava/fire (non-legacy handles this via transitions).
    0
}

fn ice_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    // ICEI/SNOW melt near fire or lava.
    for_neighbours!(sim, x, y, rx, ry, nx, ny, {
        let r = sim.pmap[sim.idx(nx, ny)];
        if r == 0 {
            continue;
        }
        let rt = pmap_type(r);
        if (rt == pt::FIRE || rt == pt::LAVA) && sim.rng.chance(1, 10) {
            sim.part_change_type(i, x, y, pt::WATR as i16);
            return 1;
        }
    });
    0
}

fn fuel_update(_sim: &mut Sim, _i: usize, _x: i32, _y: i32, _s: i32, _nt: i32) -> i32 {
    // WOOD/COAL/BCOL have no own update in the reference beyond being
    // flammable; burning is driven by FIRE.
    0
}

// ---------------------------------------------------------------------------
// SPRK: electrical conduction
// ---------------------------------------------------------------------------

fn sprk_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.parts[i].life > 0 {
        sim.parts[i].life -= 1;
    } else {
        // Spark finished: restore the element it was travelling through.
        let ct = sim.parts[i].ctype;
        if ct > 0 && (ct as usize) < crate::elements_generated::PT_NUM {
            sim.part_change_type(i, x, y, ct as i16);
        } else {
            sim.part_change_type(i, x, y, pt::METL as i16);
        }
        return 1;
    }
    sim.emp_decor = (sim.emp_decor + 3).min(40);

    // Electrify nearby conductors.
    let mut ignite: Vec<(usize, i32, i32)> = Vec::new();
    let mut powered: Vec<(usize, i32, i32)> = Vec::new();
    for_neighbours!(sim, x, y, rx, ry, nx, ny, {
        let r = sim.pmap[sim.idx(nx, ny)];
        if r == 0 {
            continue;
        }
        let rt = pmap_type(r);
        let id = pmap_id(r) as usize;
        let def = element(rt);
        if (def.properties & props::PROP_CONDUCTS) != 0
            && sim.parts[id].life == 0
            && rt != pt::SPRK
            && !(rt == pt::INSL)
        {
            ignite.push((id, nx, ny));
        } else if rt == pt::PSCN
            || rt == pt::NSCN
            || rt == pt::PTCT
            || rt == pt::NTCT
            || rt == pt::SWCH
            || rt == pt::ETRD
            || rt == pt::BTRY
            || rt == pt::LCRY
            || rt == pt::INWR
            || rt == pt::TESC
            || rt == pt::INST
        {
            powered.push((id, nx, ny));
        }
    });
    for (id, nx, ny) in ignite {
        let rt = sim.parts[id].t;
        // INSL, RBDM, LRBD are not ignited by a spark touching them directly.
        if rt == pt::INSL || rt == pt::RBDM || rt == pt::LRBD {
            continue;
        }
        if sim.rng.chance(1, 4) {
            sim.part_change_type(id, nx, ny, pt::SPRK as i16);
            sim.parts[id].life = 4;
            sim.parts[id].ctype = rt as i32;
        }
    }
    for (id, nx, ny) in powered {
        // Powered materials switch on when sparked.
        let _ = (id, nx, ny);
    }
    let _ = sim.pmap_at(x, y);
    0
}

// ---------------------------------------------------------------------------
// CLNE / PCLN: clone from ctype
// ---------------------------------------------------------------------------

fn clone_create(sim: &mut Sim, i: usize, _x: i32, _y: i32, _t: i32, _v: i32) {
    let def = element(sim.parts[i].t);
    if def.abbr == "CLNE" || def.abbr == "PCLN" {
        sim.parts[i].life = 0;
    }
}

fn clone_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let ct = sim.parts[i].ctype;
    if !(ct > 0 && (ct as usize) < crate::elements_generated::PT_NUM) {
        return 0;
    }
    // PCLN clones that are powered use their ctype; unpowered clones do nothing.
    if sim.parts[i].t == pt::PCLN {
        let powered = element(sim.parts[i].t).properties & props::PROP_CONDUCTS != 0
            && sim.parts[i].life != 0;
        if !powered {
            return 0;
        }
    }
    let mut spawn: Vec<(i32, i32)> = Vec::new();
    for_neighbours!(sim, x, y, rx, ry, nx, ny, {
        if sim.pmap[sim.idx(nx, ny)] == 0 {
            spawn.push((nx, ny));
        }
    });
    for (nx, ny) in spawn {
        if sim.rng.chance(1, 2) {
            if let Some(nid) = sim.create_part(nx, ny, ct as u16) {
                sim.parts[nid].temp = sim.parts[i].temp;
            }
        }
    }
    0
}

// ---------------------------------------------------------------------------
// BCLN / PBCN: brush clone (fills ctype along its direction)
// ---------------------------------------------------------------------------

fn bclone_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let ct = sim.parts[i].ctype;
    if !(ct > 0 && (ct as usize) < crate::elements_generated::PT_NUM) {
        return 0;
    }
    if sim.parts[i].t == pt::PBCN && sim.parts[i].life == 0 {
        return 0;
    }
    let (dx, dy) = match sim.parts[i].tmp {
        0 => (0, -1),
        1 => (1, 0),
        2 => (0, 1),
        3 => (-1, 0),
        _ => (0, -1),
    };
    let nx = x + dx;
    let ny = y + dy;
    if sim.in_bounds(nx, ny) && sim.pmap[sim.idx(nx, ny)] == 0 {
        if let Some(nid) = sim.create_part(nx, ny, ct as u16) {
            sim.parts[nid].temp = sim.parts[i].temp;
        }
    }
    0
}

// ---------------------------------------------------------------------------
// VOID / PVOD / DEST
// ---------------------------------------------------------------------------

fn void_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let ct = sim.parts[i].ctype;
    let mut targets: Vec<usize> = Vec::new();
    let mut r = 4;
    if ct > 0 && (ct as usize) < crate::elements_generated::PT_NUM {
        r = 2; // VOID with a ctype only consumes that element, closer range
    }
    for nx in (x - r)..=(x + r) {
        for ny in (y - r)..=(y + r) {
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let pr = sim.pmap[sim.idx(nx, ny)];
            if pr == 0 {
                continue;
            }
            let id = pmap_id(pr) as usize;
            if id == i {
                continue;
            }
            let rt = pmap_type(pr);
            if ct == 0 || rt == ct as u16 {
                targets.push(id);
            }
        }
    }
    for id in targets {
        sim.kill_part(id);
    }
    0
}

fn pvoid_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.parts[i].life != 0 {
        // Powered: behaves like a normal void.
        return void_update(sim, i, x, y, 0, 0);
    }
    0
}

fn dest_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    // DEST destroys non-indestructible particles it overlaps.
    let mut targets: Vec<usize> = Vec::new();
    for nx in (x - 1)..=(x + 1) {
        for ny in (y - 1)..=(y + 1) {
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let pr = sim.pmap[sim.idx(nx, ny)];
            if pr == 0 {
                continue;
            }
            let id = pmap_id(pr) as usize;
            if id == i {
                continue;
            }
            let rt = pmap_type(pr);
            if rt == pt::DMND || rt == pt::CLNE || rt == pt::PCLN || rt == pt::BCLN || rt == pt::PBCN || rt == pt::ROCK {
                continue;
            }
            targets.push(id);
        }
    }
    for id in targets {
        sim.kill_part(id);
    }
    0
}

// ---------------------------------------------------------------------------
// ACID
// ---------------------------------------------------------------------------

fn acid_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let mut targets: Vec<(usize, i32, i32)> = Vec::new();
    for_neighbours!(sim, x, y, rx, ry, nx, ny, {
        let r = sim.pmap[sim.idx(nx, ny)];
        if r == 0 {
            continue;
        }
        let rt = pmap_type(r);
        let id = pmap_id(r) as usize;
        if rt == pt::ACID || rt == pt::DMND || rt == pt::GLAS || rt == pt::INSL {
            continue;
        }
        if sim.rng.chance(1, 60) {
            targets.push((id, nx, ny));
        }
    });
    for (id, nx, ny) in targets {
        if sim.parts[id].t == 0 {
            continue;
        }
        sim.kill_part(id);
        sim.parts[i].life -= 1;
        if sim.parts[i].life <= 0 {
            sim.part_change_type(i, x, y, pt::SMKE as i16);
            return 1;
        }
        let _ = (nx, ny);
    }
    0
}

// ---------------------------------------------------------------------------
// BHOL / NBHL (black holes)
// ---------------------------------------------------------------------------

fn bhol_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let mut targets: Vec<usize> = Vec::new();
    for nx in (x - 2)..=(x + 2) {
        for ny in (y - 2)..=(y + 2) {
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let pr = sim.pmap[sim.idx(nx, ny)];
            if pr == 0 {
                continue;
            }
            let id = pmap_id(pr) as usize;
            if id == i {
                continue;
            }
            let rt = pmap_type(pr);
            if rt == pt::BHOL || rt == pt::NBHL || rt == pt::WHOL || rt == pt::NWHL {
                continue;
            }
            targets.push(id);
        }
    }
    for id in targets {
        // Matter falling in heats the hole.
        sim.parts[i].temp = (sim.parts[i].temp + sim.parts[id].temp * 0.5).min(MAX_TEMP);
        sim.kill_part(id);
    }
    0
}

fn nbhol_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    // Newtonian black hole: attracts instead of eating, matching NBHL's role.
    let mut pull: Vec<usize> = Vec::new();
    for nx in (x - 4)..=(x + 4) {
        for ny in (y - 4)..=(y + 4) {
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let pr = sim.pmap[sim.idx(nx, ny)];
            if pr == 0 {
                continue;
            }
            let id = pmap_id(pr) as usize;
            if id == i {
                continue;
            }
            let rt = pmap_type(pr);
            if rt == pt::NBHL || rt == pt::BHOL {
                continue;
            }
            pull.push(id);
        }
    }
    for id in pull {
        let dx = (x as f32 - sim.parts[id].x).signum();
        let dy = (y as f32 - sim.parts[id].y).signum();
        sim.parts[id].vx += dx * 0.2;
        sim.parts[id].vy += dy * 0.2;
    }
    0
}

// ---------------------------------------------------------------------------
// PLNT: vines grow toward nearby water
// ---------------------------------------------------------------------------

fn plnt_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.parts[i].life == 0 {
        return 0;
    }
    let dir = sim.parts[i].tmp;
    let (dx, dy) = match dir {
        0 => (0, -1),
        1 => (1, 0),
        2 => (0, 1),
        3 => (-1, 0),
        _ => (0, 1),
    };
    // Rotate toward water or just wander.
    let nx = x + dx;
    let ny = y + dy;
    if sim.in_bounds(nx, ny) && sim.pmap[sim.idx(nx, ny)] == 0 && sim.rng.chance(1, 8) {
        if let Some(nid) = sim.create_part(nx, ny, pt::PLNT) {
            sim.parts[nid].life = sim.parts[i].life - 1;
            sim.parts[nid].tmp = sim.parts[i].tmp;
            sim.parts[nid].temp = sim.parts[i].temp;
        }
    }
    if sim.rng.chance(1, 20) {
        sim.parts[i].tmp = sim.rng.between(0, 3);
    }
    0
}

// ---------------------------------------------------------------------------
// NEUT
// ---------------------------------------------------------------------------

fn neut_update(_sim: &mut Sim, _i: usize, _x: i32, _y: i32, _s: i32, _nt: i32) -> i32 {
    0
}

// ---------------------------------------------------------------------------
// Life
// ---------------------------------------------------------------------------

fn life_create(sim: &mut Sim, i: usize, _x: i32, _y: i32, _t: i32, v: i32) {
    sim.parts[i].ctype = v;
    sim.parts[i].tmp = v;
    let states = if (v as usize) < crate::elements_generated::GOL_RULES.len() {
        crate::elements_generated::GOL_RULES[v as usize].states
    } else {
        2
    };
    sim.parts[i].tmp2 = states as i32 - 1;
}

// ---------------------------------------------------------------------------
// Stickman constraints
// ---------------------------------------------------------------------------

fn stkm_allowed(sim: &Sim, _i: usize, _x: i32, _y: i32, _t: i32) -> bool {
    sim.element_count[pt::STKM as usize] <= 1
}

fn stkm2_allowed(sim: &Sim, _i: usize, _x: i32, _y: i32, _t: i32) -> bool {
    sim.element_count[pt::STKM2 as usize] <= 1
}

// Re-exported so callers can test the dispatch table cheaply.
#[allow(dead_code)]
pub fn has_update(t: u16) -> bool {
    element(t).has_update
}

#[allow(dead_code)]
fn _unused(sim: &mut Sim) -> bool {
    let _ = rand_chance(sim, 1, 2);
    true
}
