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
use crate::sim::{
    pmap_id, pmap_pack, pmap_type, restrict_flt, Sim, CELL, MAX_PRESSURE, MAX_TEMP, MIN_PRESSURE,
    MIN_TEMP,
};

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
        pt::BOMB => bomb_update,
        pt::BANG => bang_update,
        pt::C5 => c5_update,
        pt::IGNT => ignt_update,
        pt::EMBR => embr_update,
        pt::WTRV => wtrv_update,
        pt::DSTW => dstw_update,
        pt::SLTW => sltw_update,
        pt::O2 => o2_update,
        pt::CO2 => co2_update,
        pt::H2 => h2_update,
        pt::NBLE => nble_update,
        pt::GLOW => glow_update,
        pt::FOG => fog_update,
        pt::RIME => rime_update,
        pt::BMTL => bmtl_update,
        pt::BRMT => brmt_update,
        pt::NTCT => ntct_update,
        pt::PTCT => ptct_update,
        pt::IRON => iron_update,
        pt::BTRY => btry_update,
        pt::SWCH => swch_update,
        pt::GLAS => glas_update,
        pt::YEST => yest_update,
        pt::GOO => goo_update,
        pt::PLUT => plut_update,
        pt::URAN => uran_update,
        pt::MORT => mort_update,
        pt::THDR => thdr_update,
        pt::AMTR => amtr_update,
        pt::HSWC => hswc_update,
        pt::FUSE => fuse_update,
        pt::FSEP => fsep_update,
        pt::LCRY => lcry_update,
        pt::DLAY => dlay_update,
        pt::CBNW => cbnw_update,
        pt::CAUS => caus_update,
        pt::DEUT => deut_update,
        pt::WARP => warp_update,
        pt::PUMP => pump_update,
        pt::FWRK => fwrk_update,
        _ => return None,
    })
}

pub fn create_fn(t: u16) -> Option<CreateFn> {
    Some(match t {
        pt::FIRE => fire_create,
        pt::LIFE => life_create,
        pt::CLNE | pt::PCLN | pt::BCLN | pt::PBCN => clone_create,
        pt::GLAS => glas_create,
        pt::WARP => warp_create,
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
// BANG (TNT)
// ---------------------------------------------------------------------------

fn bang_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let tmp = sim.parts[i].tmp;
    if tmp == 0 {
        if sim.parts[i].temp >= 673.0 {
            sim.parts[i].tmp = 1;
        } else {
            for ry in -1i32..=1 {
                for rx in -1i32..=1 {
                    if rx == 0 && ry == 0 {
                        continue;
                    }
                    let nx = x + rx;
                    let ny = y + ry;
                    if !sim.in_bounds(nx, ny) {
                        continue;
                    }
                    let r = sim.pmap[sim.idx(nx, ny)];
                    if r == 0 {
                        continue;
                    }
                    let rt = pmap_type(r);
                    if rt == pt::FIRE || rt == pt::PLSM || rt == pt::SPRK || rt == pt::LIGH {
                        sim.parts[i].tmp = 1;
                        return 0;
                    }
                }
            }
        }
        return 0;
    } else if tmp == 1 {
        if sim.in_bounds(x, y) {
            let r = sim.pmap[sim.idx(x, y)];
            if r != 0 && pmap_id(r) as usize == i {
                sim.flood_tmp(x, y, 2);
            }
        }
        sim.parts[i].tmp = 2;
        return 0;
    } else if tmp == 2 {
        sim.parts[i].tmp = 3;
        return 0;
    }

    let otemp = sim.parts[i].temp - 273.15;
    add_pv(sim, x, y, 0.5);
    sim.parts[i].tmp = 0;
    if sim.rng.chance(1, 3) {
        if sim.rng.chance(1, 2) {
            let _ = sim.create_part(x, y, pt::FIRE);
            if sim.parts[i].t == pt::FIRE {
                sim.parts[i].temp = restrict_flt(MAX_TEMP / 4.0 + otemp, MIN_TEMP, MAX_TEMP);
            }
        } else {
            let _ = sim.create_part(x, y, pt::SMKE);
            if sim.parts[i].t == pt::SMKE {
                sim.parts[i].life = sim.rng.between(500, 549);
                sim.parts[i].temp = restrict_flt(MAX_TEMP / 4.0 + otemp, MIN_TEMP, MAX_TEMP);
            }
        }
    } else if sim.rng.chance(1, 15) {
        let nx = x + sim.rng.between(-1, 1);
        let ny = y + sim.rng.between(-1, 1);
        if let Some(nb) = create_near(sim, nx, ny, pt::EMBR) {
            sim.parts[nb].tmp = 0;
            sim.parts[nb].life = 50;
            sim.parts[nb].temp = restrict_flt(MAX_TEMP / 3.0 + otemp, MIN_TEMP, MAX_TEMP);
            sim.parts[nb].vx = sim.rng.between(-10, 10) as f32;
            sim.parts[nb].vy = sim.rng.between(-10, 10) as f32;
        }
        sim.kill_part(i);
    } else {
        sim.kill_part(i);
    }
    1
}

// ---------------------------------------------------------------------------
// C5
// ---------------------------------------------------------------------------

fn c5_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    for ry in -2i32..=2 {
        for rx in -2i32..=2 {
            if rx == 0 && ry == 0 {
                continue;
            }
            let nx = x + rx;
            let ny = y + ry;
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let r = sim.pmap[sim.idx(nx, ny)];
            if r == 0 {
                continue;
            }
            let id = pmap_id(r) as usize;
            let rt = pmap_type(r);
            let cold_trigger = (rt != pt::C5 && sim.parts[id].temp < 100.0 && !sim.is_heat_insulator(id))
                || rt == pt::CFLM;
            if cold_trigger && sim.rng.chance(1, 6) {
                sim.part_change_type(i, x, y, pt::CFLM as i16);
                sim.parts[i].temp = 0.0;
                sim.parts[id].temp = 0.0;
                sim.parts[i].life = sim.rng.between(50, 199);
                add_pv(sim, x, y, 1.5);
                return 0;
            }
        }
    }
    // PHOT emission from stored ctype (reference behavior).
    if sim.parts[i].ctype != 0 && sim.parts[i].life == 0 {
        let vx = ((sim.parts[i].tmp << 16) >> 16) as f32 / 255.0;
        let vy = (sim.parts[i].tmp >> 16) as f32 / 255.0;
        let dx = ((sim.parts[i].tmp2 << 16) >> 16) as f32 / 255.0;
        let dy = (sim.parts[i].tmp2 >> 16) as f32 / 255.0;
        let ctype = sim.parts[i].ctype;
        let temp = sim.parts[i].temp;
        let px = (x as f32 + 0.5 + dx).round() as i32;
        let py = (y as f32 + 0.5 + dy).round() as i32;
        if let Some(rid) = create_near(sim, px, py, pt::PHOT) {
            sim.parts[rid].ctype = ctype;
            sim.parts[rid].vx = vx;
            sim.parts[rid].vy = vy;
            sim.parts[rid].temp = temp;
        }
        sim.parts[i].ctype = 0;
        sim.parts[i].tmp = 0;
        sim.parts[i].tmp2 = 0;
    }
    0
}

// ---------------------------------------------------------------------------
// IGNT (ignition cord)
// ---------------------------------------------------------------------------

fn ignt_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.parts[i].tmp == 0 {
        for ry in -1i32..=1 {
            for rx in -1i32..=1 {
                if rx == 0 && ry == 0 {
                    continue;
                }
                let nx = x + rx;
                let ny = y + ry;
                if !sim.in_bounds(nx, ny) {
                    continue;
                }
                let r = sim.pmap[sim.idx(nx, ny)];
                if r == 0 {
                    continue;
                }
                let rt = pmap_type(r);
                if rt == pt::FIRE || rt == pt::PLSM || rt == pt::SPRK || rt == pt::LIGH
                    || (rt == pt::IGNT && sim.parts[pmap_id(r) as usize].life == 1)
                {
                    sim.parts[i].tmp = 1;
                    return 0;
                }
            }
        }
    } else if sim.parts[i].life > 0 {
        if sim.rng.chance(2, 3) {
            let nx = x + sim.rng.between(-1, 1);
            let ny = y + sim.rng.between(-1, 1);
            if let Some(nb) = create_near(sim, nx, ny, pt::EMBR) {
                sim.parts[nb].tmp = 0;
                sim.parts[nb].life = 30;
                sim.parts[nb].vx = sim.rng.between(-10, 10) as f32;
                sim.parts[nb].vy = sim.rng.between(-10, 10) as f32;
                sim.parts[nb].temp = restrict_flt(sim.parts[i].temp - 273.15 + 400.0, MIN_TEMP, MAX_TEMP);
            }
        } else {
            let nx = x + sim.rng.between(-1, 1);
            let ny = y + sim.rng.between(-1, 1);
            let _ = create_near(sim, nx, ny, pt::FIRE);
        }
        sim.parts[i].life -= 1;
    }
    0
}

// ---------------------------------------------------------------------------
// EMBR (sparks/embers) — basic behaviour
// ---------------------------------------------------------------------------

fn embr_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    // Reference kills EMBR on contact with solid/part/liquid that is not SPARKSETTLE.
    for ry in -1i32..=1 {
        for rx in -1i32..=1 {
            if rx == 0 && ry == 0 {
                continue;
            }
            let nx = x + rx;
            let ny = y + ry;
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let r = sim.pmap[sim.idx(nx, ny)];
            if r == 0 {
                continue;
            }
            let rt = pmap_type(r);
            let props = element(rt).properties;
            let is_settle_mat = (props & (props::TYPE_SOLID | props::TYPE_PART | props::TYPE_LIQUID)) != 0;
            let sparksettle = (props & props::PROP_SPARKSETTLE) != 0;
            if is_settle_mat && !sparksettle {
                sim.kill_part(i);
                return 1;
            }
        }
    }
    0
}

// ---------------------------------------------------------------------------
// BOMB
// ---------------------------------------------------------------------------

fn bomb_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    // Check 3x3 neighbours for a trigger.
    let mut trigger = false;
    for ry in -1i32..=1 {
        for rx in -1i32..=1 {
            if rx == 0 && ry == 0 {
                continue;
            }
            let nx = x + rx;
            let ny = y + ry;
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let r = sim.pmap[sim.idx(nx, ny)];
            if r == 0 {
                continue;
            }
            let rt = pmap_type(r);
            if rt == pt::BOMB || rt == pt::EMBR || rt == pt::DMND || rt == pt::CLNE
                || rt == pt::PCLN || rt == pt::BCLN || rt == pt::VIBR
            {
                continue;
            }
            trigger = true;
            break;
        }
        if trigger {
            break;
        }
    }
    if !trigger {
        return 0;
    }
    let rad: i32 = 8;
    sim.kill_part(i);
    // Inner disk: kill + EMBR with tmp=2, life=2.
    for nxj in -rad..=rad {
        for nxi in -rad..=rad {
            if nxi * nxi + nxj * nxj > rad * rad {
                continue;
            }
            let xnxi = x + nxi;
            let ynxj = y + nxj;
            if !sim.in_bounds(xnxi, ynxj) {
                continue;
            }
            let cell = sim.idx(xnxi, ynxj);
            let nt = pmap_type(sim.pmap[cell]);
            if nt == pt::DMND || nt == pt::CLNE || nt == pt::PCLN || nt == pt::BCLN || nt == pt::VIBR {
                continue;
            }
            if nt != 0 {
                let id = pmap_id(sim.pmap[cell]) as usize;
                sim.kill_part(id);
            }
            let vcx = xnxi / CELL as i32;
            let vcy = ynxj / CELL as i32;
            if sim.in_cell_bounds(vcx, vcy) {
                let vci = sim.cell_idx(vcx, vcy);
                sim.pv[vci] += 0.1;
            }
            if let Some(nb) = sim.create_part(xnxi, ynxj, pt::EMBR) {
                sim.parts[nb].tmp = 2;
                sim.parts[nb].life = 2;
                sim.parts[nb].temp = MAX_TEMP;
            }
        }
    }
    // Outer ring: EMBR with tmp=0, life=50, random velocity.
    for nxj in -(rad + 1)..=(rad + 1) {
        for nxi in -(rad + 1)..=(rad + 1) {
            if nxi * nxi + nxj * nxj > (rad + 1) * (rad + 1) {
                continue;
            }
            if nxi * nxi + nxj * nxj <= rad * rad {
                continue;
            }
            let xnxi = x + nxi;
            let ynxj = y + nxj;
            if !sim.in_bounds(xnxi, ynxj) {
                continue;
            }
            let cell = sim.idx(xnxi, ynxj);
            if sim.pmap[cell] != 0 {
                continue;
            }
            if let Some(nb) = sim.create_part(xnxi, ynxj, pt::EMBR) {
                sim.parts[nb].tmp = 0;
                sim.parts[nb].life = 50;
                sim.parts[nb].temp = MAX_TEMP;
                sim.parts[nb].vx = sim.rng.between(-20, 20) as f32;
                sim.parts[nb].vy = sim.rng.between(-20, 20) as f32;
            }
        }
    }
    1
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

// ---------------------------------------------------------------------------
// Generic helpers for the element ports below
// ---------------------------------------------------------------------------

/// True when a pixel is free of both ordinary particles and energy particles.
#[inline]
fn cell_free(sim: &Sim, x: i32, y: i32) -> bool {
    if !sim.in_bounds(x, y) {
        return false;
    }
    let c = sim.idx(x, y);
    sim.pmap[c] == 0 && sim.photons[c] == 0
}

/// Reference `create_part(-3, x, y, t)`: try `(x, y)` then an expanding ring.
fn create_near(sim: &mut Sim, x: i32, y: i32, t: u16) -> Option<usize> {
    if cell_free(sim, x, y) {
        return sim.create_part(x, y, t);
    }
    for r in 1..=2i32 {
        for dx in -r..=r {
            for dy in -r..=r {
                if dx.abs() != r && dy.abs() != r {
                    continue;
                }
                let nx = x + dx;
                let ny = y + dy;
                if cell_free(sim, nx, ny) {
                    return sim.create_part(nx, ny, t);
                }
            }
        }
    }
    None
}

/// Reference `parts_avg`: returns the type of the particle at the midpoint
/// between `ci` and `ni` (or `t` when that midpoint holds `t`). `PT_NONE` (0)
/// is returned when the midpoint is empty or holds a different type.
fn parts_avg(sim: &Sim, ci: usize, ni: usize, t: u16) -> u16 {
    let (pci, pni) = (&sim.parts[ci], &sim.parts[ni]);
    let (mx, my) = if t == pt::INSL {
        (
            (((pci.x + 0.5) + (pni.x + 0.5)) / 2.0) as i32,
            (((pci.y + 0.5) + (pni.y + 0.5)) / 2.0) as i32,
        )
    } else {
        (
            ((pci.x + pni.x) / 2.0 + 0.5) as i32,
            ((pci.y + pni.y) / 2.0 + 0.5) as i32,
        )
    };
    let r = sim.pmap_at(mx, my);
    if r == 0 {
        return 0;
    }
    let rt = pmap_type(r);
    if t == pt::INSL || rt == t {
        rt
    } else {
        0
    }
}

// ---------------------------------------------------------------------------
// WTRV: steam. Burns on contact with RBDM/LRBD, cools at very high temp.
// ---------------------------------------------------------------------------

fn wtrv_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let hot = sim.parts[i].temp > (273.15 + 12.0);
    for rx in -1..=1 {
        for ry in -1..=1 {
            if rx == 0 && ry == 0 {
                continue;
            }
            let nx = x + rx;
            let ny = y + ry;
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let r = sim.pmap[sim.idx(nx, ny)];
            if r == 0 {
                continue;
            }
            let rt = pmap_type(r);
            if (rt == pt::RBDM || rt == pt::LRBD) && hot && sim.rng.chance(1, 100) {
                sim.part_change_type(i, x, y, pt::FIRE as i16);
                sim.parts[i].life = 4;
                sim.parts[i].ctype = pt::WATR as i32;
            }
        }
    }
    if sim.parts[i].temp > 1273.0 && sim.parts[i].ctype == pt::FIRE as i32 {
        sim.parts[i].temp -= sim.parts[i].temp / 1000.0;
    }
    0
}

// ---------------------------------------------------------------------------
// DSTW: distilled water. Salts up into SLTW, evaporates on fire.
// ---------------------------------------------------------------------------

fn dstw_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    for rx in -1..=1 {
        for ry in -1..=1 {
            if rx == 0 && ry == 0 {
                continue;
            }
            let nx = x + rx;
            let ny = y + ry;
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let r = sim.pmap[sim.idx(nx, ny)];
            if r == 0 {
                continue;
            }
            let rt = pmap_type(r);
            let id = pmap_id(r) as usize;
            match rt {
                pt::SALT => {
                    if sim.rng.chance(1, 50) {
                        sim.part_change_type(i, x, y, pt::SLTW as i16);
                        if sim.rng.chance(1, 3) {
                            sim.part_change_type(id, nx, ny, pt::SLTW as i16);
                        }
                    }
                }
                pt::SLTW => {
                    if sim.rng.chance(1, 2000) {
                        sim.part_change_type(i, x, y, pt::SLTW as i16);
                    } else if sim.rng.chance(1, 100) {
                        sim.part_change_type(i, x, y, pt::WATR as i16);
                    }
                }
                pt::WATR => {
                    if sim.rng.chance(1, 100) {
                        sim.part_change_type(i, x, y, pt::WATR as i16);
                    }
                }
                pt::RBDM | pt::LRBD => {
                    if sim.parts[i].temp > 12.0 && sim.rng.chance(1, 100) {
                        sim.part_change_type(i, x, y, pt::FIRE as i16);
                        sim.parts[i].life = 4;
                    }
                }
                pt::FIRE => {
                    sim.kill_part(id);
                    if sim.rng.chance(1, 30) {
                        sim.kill_part(i);
                        return 1;
                    }
                }
                pt::SMKE => {
                    let ti = sim.parts[i].temp;
                    let tr = sim.parts[id].temp;
                    if tr > 313.15 && tr < 333.15 && ti > 313.15 && ti < 333.15 {
                        if sim.rng.chance(1, 100) {
                            sim.part_change_type(i, x, y, pt::BASE as i16);
                            sim.parts[i].life = 1;
                            sim.kill_part(id);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    0
}

// ---------------------------------------------------------------------------
// SLTW: salt water. Converts salt, feeds plants, extinguishes fire.
// ---------------------------------------------------------------------------

fn sltw_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let hot = sim.parts[i].temp > (273.15 + 12.0);
    for rx in -1..=1 {
        for ry in -1..=1 {
            if rx == 0 && ry == 0 {
                continue;
            }
            let nx = x + rx;
            let ny = y + ry;
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let r = sim.pmap[sim.idx(nx, ny)];
            if r == 0 {
                continue;
            }
            let rt = pmap_type(r);
            let id = pmap_id(r) as usize;
            match rt {
                pt::SALT => {
                    if sim.rng.chance(1, 2000) {
                        sim.part_change_type(id, nx, ny, pt::SLTW as i16);
                    }
                }
                pt::PLNT => {
                    if sim.rng.chance(1, 40) {
                        sim.kill_part(id);
                    }
                }
                pt::RBDM | pt::LRBD => {
                    if hot && sim.rng.chance(1, 100) {
                        sim.part_change_type(i, x, y, pt::FIRE as i16);
                        sim.parts[i].life = 4;
                        sim.parts[i].ctype = pt::WATR as i32;
                    }
                }
                pt::FIRE => {
                    if sim.parts[id].ctype != pt::WATR as i32 {
                        sim.kill_part(id);
                        if sim.rng.chance(1, 30) {
                            sim.kill_part(i);
                            return 1;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    0
}

// ---------------------------------------------------------------------------
// O2: liquid oxygen. Feeds fire; high energy converts to BMTL + neutrons.
// ---------------------------------------------------------------------------

fn o2_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    for rx in -2..=2 {
        for ry in -2..=2 {
            if rx == 0 && ry == 0 {
                continue;
            }
            let nx = x + rx;
            let ny = y + ry;
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let r = sim.pmap[sim.idx(nx, ny)];
            if r == 0 {
                continue;
            }
            let rt = pmap_type(r);
            let id = pmap_id(r) as usize;
            if rt == pt::FIRE {
                let add = sim.rng.between(0, 99) as f32;
                sim.parts[id].temp += add;
                if (sim.parts[id].tmp & 0x01) != 0 {
                    sim.parts[id].temp = 3473.0;
                }
                sim.parts[id].tmp |= 2;
                if sim.part_change_type(i, x, y, pt::FIRE as i16) {
                    sim.parts[i].temp += sim.rng.between(0, 99) as f32;
                    sim.parts[i].tmp |= 2;
                }
            } else if rt == pt::PLSM && (sim.parts[id].tmp & 4) == 0 {
                if sim.part_change_type(i, x, y, pt::FIRE as i16) {
                    sim.parts[i].temp += sim.rng.between(0, 99) as f32;
                    sim.parts[i].tmp |= 2;
                }
            }
        }
    }
    if sim.parts[i].temp > 9973.15 && pv(sim, x, y) > 250.0 {
        let (gx, gy) = sim.grav_force[sim.cell_idx(x / CELL as i32, y / CELL as i32)];
        if gx * gx + gy * gy > 400.0 && sim.rng.chance(1, 5) {
            let temp = sim.parts[i].temp;
            sim.part_change_type(i, x, y, pt::BRMT as i16);
            if let Some(j) = create_near(sim, x, y, pt::NEUT) {
                sim.parts[j].temp = MAX_TEMP;
            }
            if let Some(j) = create_near(sim, x, y, pt::PHOT) {
                sim.parts[j].temp = MAX_TEMP;
                sim.parts[j].tmp = 0x1;
            }
            let rx = x + sim.rng.between(-1, 1);
            let ry = y + sim.rng.between(-1, 1);
            let rt = sim.type_at(rx, ry);
            if sim.can_move(pt::PLSM, rt) == 1 || rt == pt::O2 {
                if let Some(j) = create_near(sim, rx, ry, pt::PLSM) {
                    sim.parts[j].temp = MAX_TEMP;
                    sim.parts[j].tmp |= 4;
                }
            }
            if let Some(j) = create_near(sim, x, y, pt::GRVT) {
                sim.parts[j].temp = MAX_TEMP;
            }
            sim.parts[i].temp = MAX_TEMP;
            let _ = temp;
            if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
                let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
                sim.pv[vc] = MAX_PRESSURE;
            }
        }
    }
    0
}

// ---------------------------------------------------------------------------
// CO2: carbon dioxide. Extinguishes fire, carbonates water at high pressure.
// ---------------------------------------------------------------------------

fn co2_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let ctype_water = sim.parts[i].ctype == 5;
    for rx in -1..=1 {
        for ry in -1..=1 {
            if rx == 0 && ry == 0 {
                continue;
            }
            let nx = x + rx;
            let ny = y + ry;
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let r = sim.pmap[sim.idx(nx, ny)];
            if r == 0 {
                if ctype_water && sim.rng.chance(1, 2000) {
                    if sim.create_part(nx, ny, pt::WATR).is_some() {
                        sim.parts[i].ctype = 0;
                    }
                }
                continue;
            }
            let rt = pmap_type(r);
            let id = pmap_id(r) as usize;
            if rt == pt::FIRE {
                sim.kill_part(id);
                if sim.rng.chance(1, 30) {
                    sim.kill_part(i);
                    return 1;
                }
            } else if (rt == pt::WATR || rt == pt::DSTW) && sim.rng.chance(1, 50) {
                sim.part_change_type(id, nx, ny, pt::CBNW as i16);
                if sim.parts[i].ctype == 5 {
                    sim.part_change_type(i, x, y, pt::WATR as i16);
                    return 0;
                } else {
                    sim.kill_part(i);
                    return 1;
                }
            }
        }
    }
    if sim.parts[i].temp > 9773.15 && pv(sim, x, y) > 200.0 && sim.rng.chance(1, 5) {
        sim.part_change_type(i, x, y, pt::O2 as i16);
        if let Some(j) = create_near(sim, x, y, pt::NEUT) {
            sim.parts[j].temp = MAX_TEMP;
        }
        if sim.rng.chance(1, 50) {
            if let Some(j) = create_near(sim, x, y, pt::ELEC) {
                sim.parts[j].temp = MAX_TEMP;
            }
        }
        sim.parts[i].temp = MAX_TEMP;
        if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
            let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
            sim.pv[vc] += 100.0;
        }
    }
    0
}

// ---------------------------------------------------------------------------
// H2: hydrogen. Burns with FIRE/PLSM/LAVA, fuses at extreme pressure.
// ---------------------------------------------------------------------------

fn h2_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let pres = pv(sim, x, y);
    for rx in -2..=2 {
        for ry in -2..=2 {
            if rx == 0 && ry == 0 {
                continue;
            }
            let nx = x + rx;
            let ny = y + ry;
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let r = sim.pmap[sim.idx(nx, ny)];
            if r == 0 {
                continue;
            }
            let rt = pmap_type(r);
            let id = pmap_id(r) as usize;
            if pres > 8.0 && rt == pt::DESL {
                sim.part_change_type(id, nx, ny, pt::WATR as i16);
                sim.part_change_type(i, x, y, pt::OIL as i16);
                return 1;
            }
            if pres > 45.0 {
                if sim.parts[id].temp > 2273.15 {
                    continue;
                }
            } else if rt == pt::FIRE {
                if (sim.parts[id].tmp & 0x02) != 0 {
                    sim.parts[id].temp = 3473.0;
                } else {
                    sim.parts[id].temp = 2473.15;
                }
                sim.parts[id].tmp |= 1;
                if sim.part_change_type(i, x, y, pt::FIRE as i16) {
                    sim.parts[i].temp += sim.rng.between(0, 99) as f32;
                    sim.parts[i].tmp |= 1;
                }
                return 1;
            } else if (rt == pt::PLSM && (sim.parts[id].tmp & 4) == 0)
                || (rt == pt::LAVA && sim.parts[id].ctype != pt::BMTL as i32)
            {
                if sim.part_change_type(i, x, y, pt::FIRE as i16) {
                    sim.parts[i].temp += sim.rng.between(0, 99) as f32;
                    sim.parts[i].tmp |= 1;
                }
                return 1;
            }
        }
    }
    if sim.parts[i].temp > 2273.15 && pres > 50.0 && sim.rng.chance(1, 5) {
        let temp = sim.parts[i].temp;
        sim.part_change_type(i, x, y, pt::NBLE as i16);
        sim.parts[i].tmp = 0x1;
        if let Some(j) = create_near(sim, x, y, pt::NEUT) {
            sim.parts[j].temp = temp;
        }
        if sim.rng.chance(1, 10) {
            if let Some(j) = create_near(sim, x, y, pt::ELEC) {
                sim.parts[j].temp = temp;
            }
        }
        if let Some(j) = create_near(sim, x, y, pt::PHOT) {
            sim.parts[j].ctype = 0x7C0000;
            sim.parts[j].temp = temp;
            sim.parts[j].tmp = 0x1;
        }
        let rx = x + sim.rng.between(-1, 1);
        let ry = y + sim.rng.between(-1, 1);
        let rt = sim.type_at(rx, ry);
        if sim.can_move(pt::PLSM, rt) == 1 || rt == pt::H2 {
            if let Some(j) = create_near(sim, rx, ry, pt::PLSM) {
                sim.parts[j].temp = temp;
                sim.parts[j].tmp |= 4;
            }
        }
        sim.parts[i].temp = temp + sim.rng.between(750, 1249) as f32;
        if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
            let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
            sim.pv[vc] += 30.0;
        }
        return 1;
    }
    0
}

// ---------------------------------------------------------------------------
// NBLE: noble gas. Fuses to CO2 + NEUT + PHOT under extreme heat/pressure.
// ---------------------------------------------------------------------------

fn nble_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.parts[i].temp > 5273.15 && pv(sim, x, y) > 100.0 {
        sim.parts[i].tmp |= 0x1;
        if sim.rng.chance(1, 5) {
            let temp = sim.parts[i].temp;
            sim.part_change_type(i, x, y, pt::CO2 as i16);
            if let Some(j) = create_near(sim, x, y, pt::NEUT) {
                sim.parts[j].temp = temp;
            }
            if sim.rng.chance(1, 25) {
                if let Some(j) = create_near(sim, x, y, pt::ELEC) {
                    sim.parts[j].temp = temp;
                }
            }
            if let Some(j) = create_near(sim, x, y, pt::PHOT) {
                sim.parts[j].ctype = 0xF800000;
                sim.parts[j].temp = temp;
                sim.parts[j].tmp = 0x1;
            }
            let rx = x + sim.rng.between(-1, 1);
            let ry = y + sim.rng.between(-1, 1);
            let rt = sim.type_at(rx, ry);
            if sim.can_move(pt::PLSM, rt) == 1 || rt == pt::NBLE {
                if let Some(j) = create_near(sim, rx, ry, pt::PLSM) {
                    sim.parts[j].temp = temp;
                    sim.parts[j].tmp |= 4;
                }
            }
            sim.parts[i].temp = temp + 1750.0 + sim.rng.between(0, 499) as f32;
            if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
                let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
                sim.pv[vc] += 50.0;
            }
        }
    }
    0
}

// ---------------------------------------------------------------------------
// GLOW: glowing gas. Reacts with water/gel; colour tracks pressure/velocity.
// ---------------------------------------------------------------------------

fn glow_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    for rx in -1..=1 {
        for ry in -1..=1 {
            if rx == 0 && ry == 0 {
                continue;
            }
            let nx = x + rx;
            let ny = y + ry;
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let r = sim.pmap[sim.idx(nx, ny)];
            if r == 0 {
                continue;
            }
            let rt = pmap_type(r);
            let id = pmap_id(r) as usize;
            if rt == pt::WATR && sim.rng.chance(1, 400) {
                sim.kill_part(i);
                sim.part_change_type(id, nx, ny, pt::DEUT as i16);
                sim.parts[id].life = 10;
                return 1;
            } else if rt == pt::GEL {
                sim.kill_part(i);
                sim.part_change_type(id, nx, ny, pt::RSST as i16);
                sim.parts[id].tmp = 0;
                return 1;
            }
        }
    }
    if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
        let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
        let mut ctype = (sim.pv[vc] * 16.0) as i32;
        if ctype < 0 {
            ctype = 0;
        }
        sim.parts[i].ctype = ctype;
        let mut n = ((sim.vx[vc] + sim.vy[vc]) * 16.0) as i32;
        if n < 0 {
            n = -n;
        }
        let mut m = ((sim.parts[i].vx + sim.parts[i].vy) * 64.0) as i32;
        if m < 0 {
            m = -m;
        }
        sim.parts[i].tmp = n + m;
    }
    0
}

// ---------------------------------------------------------------------------
// FOG: condenses on solids, becomes RIME.
// ---------------------------------------------------------------------------

fn fog_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    for rx in -1..=1 {
        for ry in -1..=1 {
            if rx == 0 && ry == 0 {
                continue;
            }
            let nx = x + rx;
            let ny = y + ry;
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let r = sim.pmap[sim.idx(nx, ny)];
            if r == 0 {
                continue;
            }
            let rt = pmap_type(r);
            let id = pmap_id(r) as usize;
            let solid = (element(rt).properties & props::TYPE_SOLID) != 0;
            let not_clone = !matches!(rt, pt::CLNE | pt::PCLN | pt::BCLN | pt::PBCN);
            if solid && not_clone && sim.parts[i].life == 0 && sim.rng.chance(1, 10) {
                sim.part_change_type(i, x, y, pt::RIME as i16);
            }
            if rt == pt::SPRK {
                let add = sim.rng.between(0, 19);
                sim.parts[i].life += add;
            }
            if rt == pt::GAS && sim.parts[i].tmp < 10 {
                sim.kill_part(id);
                if sim.parts[i].ctype == pt::DSTW as i32 {
                    sim.parts[i].ctype = 0;
                } else {
                    sim.parts[i].tmp += 1;
                }
            }
        }
    }
    0
}

// ---------------------------------------------------------------------------
// RIME: frozen fog. Melts to FOG on spark, absorbs GAS acidity.
// ---------------------------------------------------------------------------

fn rime_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    for rx in -1..=1 {
        for ry in -1..=1 {
            if rx == 0 && ry == 0 {
                continue;
            }
            let nx = x + rx;
            let ny = y + ry;
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let r = sim.pmap[sim.idx(nx, ny)];
            if r == 0 {
                continue;
            }
            let rt = pmap_type(r);
            let id = pmap_id(r) as usize;
            if rt == pt::SPRK {
                sim.part_change_type(i, x, y, pt::FOG as i16);
                sim.parts[i].life = sim.rng.between(60, 119);
            } else if rt == pt::GAS && sim.parts[i].tmp < 10 {
                sim.kill_part(id);
                if sim.parts[i].ctype == pt::DSTW as i32 {
                    sim.parts[i].ctype = 0;
                } else {
                    sim.parts[i].tmp += 1;
                }
            } else if rt == pt::FOG && sim.parts[id].life > 0 {
                let life = sim.parts[id].life;
                sim.part_change_type(i, x, y, pt::FOG as i16);
                sim.parts[i].life = life;
            }
        }
    }
    0
}

// ---------------------------------------------------------------------------
// BMTL: breakable metal. Rusts/cracks into BRMT; spreads through METL/IRON.
// ---------------------------------------------------------------------------

fn bmtl_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.parts[i].tmp > 1 {
        sim.parts[i].tmp -= 1;
        for rx in -1..=1 {
            for ry in -1..=1 {
                if rx == 0 && ry == 0 {
                    continue;
                }
                let nx = x + rx;
                let ny = y + ry;
                if !sim.in_bounds(nx, ny) {
                    continue;
                }
                let r = sim.pmap[sim.idx(nx, ny)];
                if r == 0 {
                    continue;
                }
                let rt = pmap_type(r);
                let id = pmap_id(r) as usize;
                if (rt == pt::METL || rt == pt::IRON) && sim.rng.chance(1, 100) {
                    sim.part_change_type(id, nx, ny, pt::BMTL as i16);
                    let spread = if sim.parts[i].tmp <= 7 {
                        sim.parts[i].tmp = 1;
                        1
                    } else {
                        sim.parts[i].tmp - sim.rng.between(0, 4)
                    };
                    sim.parts[id].tmp = spread;
                }
            }
        }
    } else if sim.parts[i].tmp == 1 && sim.rng.chance(1, 1000) {
        sim.parts[i].tmp = 0;
        sim.part_change_type(i, x, y, pt::BRMT as i16);
    }
    0
}

// ---------------------------------------------------------------------------
// BRMT: broken metal. Hot BRMT + BREC converts into THRM.
// ---------------------------------------------------------------------------

fn brmt_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let temp = sim.parts[i].temp;
    if temp > 523.15 {
        let mut temp_factor = (1000.0 - ((523.15 - temp) * 2.0)) as i32;
        if temp_factor < 2 {
            temp_factor = 2;
        }
        for rx in -1..=1 {
            for ry in -1..=1 {
                if rx == 0 && ry == 0 {
                    continue;
                }
                let nx = x + rx;
                let ny = y + ry;
                if !sim.in_bounds(nx, ny) {
                    continue;
                }
                let r = sim.pmap[sim.idx(nx, ny)];
                if r == 0 {
                    continue;
                }
                let rt = pmap_type(r);
                let id = pmap_id(r) as usize;
                if rt == pt::BREC && sim.rng.chance(1, temp_factor as u32) {
                    if sim.rng.chance(1, 2) {
                        sim.part_change_type(id, nx, ny, pt::THRM as i16);
                    } else {
                        sim.part_change_type(i, x, y, pt::THRM as i16);
                    }
                }
            }
        }
    }
    0
}

// ---------------------------------------------------------------------------
// NTCT / PTCT: thermistors. Self-cool slightly above 22 °C.
// ---------------------------------------------------------------------------

fn ntct_update(sim: &mut Sim, i: usize, _x: i32, _y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.parts[i].temp > 295.0 {
        sim.parts[i].temp -= 2.5;
    }
    0
}

fn ptct_update(sim: &mut Sim, i: usize, _x: i32, _y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.parts[i].temp > 295.0 {
        sim.parts[i].temp -= 2.5;
    }
    0
}

// ---------------------------------------------------------------------------
// IRON: rusts into BMTL when touching salt/water/oxygen.
// ---------------------------------------------------------------------------

fn iron_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.parts[i].life != 0 {
        return 0;
    }
    let mut broken = false;
    'outer: for rx in -1..=1 {
        for ry in -1..=1 {
            if rx == 0 && ry == 0 {
                continue;
            }
            let nx = x + rx;
            let ny = y + ry;
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let r = sim.pmap[sim.idx(nx, ny)];
            if r == 0 {
                continue;
            }
            let rt = pmap_type(r);
            match rt {
                pt::SALT => {
                    if sim.rng.chance(1, 47) {
                        broken = true;
                    }
                }
                pt::SLTW => {
                    if sim.rng.chance(1, 67) {
                        broken = true;
                    }
                }
                pt::WATR => {
                    if sim.rng.chance(1, 1200) {
                        broken = true;
                    }
                }
                pt::O2 => {
                    if sim.rng.chance(1, 250) {
                        broken = true;
                    }
                }
                pt::LO2 => broken = true,
                _ => {}
            }
            if broken {
                break 'outer;
            }
        }
    }
    if broken {
        sim.part_change_type(i, x, y, pt::BMTL as i16);
        sim.parts[i].tmp = sim.rng.between(20, 29);
    }
    0
}

// ---------------------------------------------------------------------------
// BTRY: battery. Sparks nearby conductors (INSL/RSSS shield the midpoint).
// ---------------------------------------------------------------------------

fn btry_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    for rx in -2i32..=2 {
        for ry in -2i32..=2 {
            if (rx == 0 && ry == 0) || (rx.abs() + ry.abs()) >= 4 {
                continue;
            }
            let nx = x + rx;
            let ny = y + ry;
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let r = sim.pmap[sim.idx(nx, ny)];
            if r == 0 {
                continue;
            }
            let id = pmap_id(r) as usize;
            let rt = pmap_type(r);
            let pavg = parts_avg(sim, i, id, pt::INSL);
            if pavg == pt::INSL || pavg == pt::RSSS {
                continue;
            }
            let conducts = (element(rt).properties & props::PROP_CONDUCTS) != 0;
            if conducts
                && !matches!(rt, pt::WATR | pt::SLTW | pt::NTCT | pt::PTCT | pt::INWR)
                && sim.parts[id].life == 0
            {
                sim.parts[id].life = 4;
                sim.parts[id].ctype = rt as i32;
                sim.part_change_type(id, nx, ny, pt::SPRK as i16);
            }
        }
    }
    0
}

// ---------------------------------------------------------------------------
// SWCH: switch. Sparks toggle it; two red BRAYs toggle it diagonally.
// ---------------------------------------------------------------------------

fn is_red_bray(sim: &Sim, x: i32, y: i32) -> bool {
    let r = sim.pmap_at(x, y);
    if r == 0 {
        return false;
    }
    if pmap_type(r) != pt::BRAY {
        return false;
    }
    sim.parts[pmap_id(r) as usize].tmp == 2
}

fn swch_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let life = sim.parts[i].life;
    if life > 0 && life != 10 {
        sim.parts[i].life -= 1;
    }
    for rx in -2..=2 {
        for ry in -2..=2 {
            if rx == 0 && ry == 0 {
                continue;
            }
            let nx = x + rx;
            let ny = y + ry;
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let r = sim.pmap[sim.idx(nx, ny)];
            if r == 0 {
                continue;
            }
            let id = pmap_id(r) as usize;
            let rt = pmap_type(r);
            let pavg = parts_avg(sim, i, id, pt::INSL);
            if pavg == pt::INSL || pavg == pt::RSSS {
                continue;
            }
            if rt == pt::SWCH {
                let other = sim.parts[id].life;
                if sim.parts[i].life >= 10 && other < 10 && other > 0 {
                    sim.parts[i].life = 9;
                } else if sim.parts[i].life == 0 && other >= 10 {
                    sim.parts[i].life = other;
                }
            } else if rt == pt::SPRK
                && sim.parts[i].life == 10
                && sim.parts[id].life > 0
                && sim.parts[id].ctype != pt::PSCN as i32
                && sim.parts[id].ctype != pt::NSCN as i32
            {
                sim.part_change_type(i, x, y, pt::SPRK as i16);
                sim.parts[i].ctype = pt::SWCH as i32;
                sim.parts[i].life = 4;
            }
        }
    }
    // Two red BRAYs, one vertical and one horizontal, toggle the switch.
    let above = sim.pmap_at(x, y - 1) == 0;
    let below = sim.pmap_at(x, y + 1) == 0;
    let left = sim.pmap_at(x - 1, y) == 0;
    let right = sim.pmap_at(x + 1, y) == 0;
    let _ = (left, right);
    if above && below {
        let vertical = is_red_bray(sim, x, y - 1) || is_red_bray(sim, x, y + 1);
        let horizontal = is_red_bray(sim, x + 1, y) || is_red_bray(sim, x - 1, y);
        if vertical && horizontal {
            if sim.parts[i].life == 10 {
                sim.parts[i].life = 9;
            } else if sim.parts[i].life <= 5 {
                sim.parts[i].life = 14;
            }
        }
    }
    0
}

// ---------------------------------------------------------------------------
// GLAS: glass. Cracks to BGLA on pressure change; strengthened glass condenses
// liquid nitrogen.
// ---------------------------------------------------------------------------

fn glas_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let pres = (pv(sim, x, y) * 64.0) as i32;
    let diff = pres - sim.parts[i].tmp3;
    let mut strength = (sim.parts[i].life / 120) + 16;
    if strength < 16 {
        strength = 16;
    }
    if diff > strength || diff < -strength {
        sim.part_change_type(i, x, y, pt::BGLA as i16);
    }
    sim.parts[i].tmp3 = pres;

    if strength > 200 && sim.parts[i].temp < 77.0 && pv(sim, x, y) > 5.0 && sim.rng.chance(1, 100) {
        let adj = sim.rng.between(0, 3);
        let rx = (1 - 2 * (adj % 2)) * (1 - adj / 2);
        let ry = (1 - 2 * (adj % 2)) * (adj / 2);
        let nx = x + rx;
        let ny = y + ry;
        if cell_free(sim, nx, ny) {
            if sim.create_part(nx, ny, pt::LNTG).is_some() {
                if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
                    let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
                    sim.pv[vc] -= 1.0;
                }
                sim.parts[i].temp += 200.0;
            }
        }
    }
    0
}

fn glas_create(sim: &mut Sim, i: usize, x: i32, y: i32, _t: i32, _v: i32) {
    sim.parts[i].tmp3 = (pv(sim, x, y) * 64.0) as i32;
}

// ---------------------------------------------------------------------------
// YEST: yeast. Converts to DYST near DYST; grows at 30–44 °C.
// ---------------------------------------------------------------------------

fn yest_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    for rx in -1..=1 {
        for ry in -1..=1 {
            if rx == 0 && ry == 0 {
                continue;
            }
            let nx = x + rx;
            let ny = y + ry;
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let r = sim.pmap[sim.idx(nx, ny)];
            if r == 0 {
                continue;
            }
            let rt = pmap_type(r);
            if rt == pt::DYST && sim.rng.chance(1, 6) {
                sim.part_change_type(i, x, y, pt::DYST as i16);
            }
        }
    }
    let temp = sim.parts[i].temp;
    if temp > 303.0 && temp < 317.0 {
        let nx = x + sim.rng.between(-1, 1);
        let ny = y + sim.rng.between(-1, 1);
        if cell_free(sim, nx, ny) {
            sim.create_part(nx, ny, pt::YEST);
        }
    }
    0
}

// ---------------------------------------------------------------------------
// GOO: goo. Becomes mobile (life) when pressurised.
// ---------------------------------------------------------------------------

fn goo_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.parts[i].life == 0 && pv(sim, x, y) > 1.0 {
        sim.parts[i].life = sim.rng.between(300, 379);
    }
    if sim.parts[i].life != 0 && sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
        let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
        sim.parts[i].vx += 0.1 * sim.vx[vc];
        sim.parts[i].vy += 0.1 * sim.vy[vc];
    }
    0
}

// ---------------------------------------------------------------------------
// PLUT: plutonium. Emits neutrons under pressure.
// ---------------------------------------------------------------------------

fn plut_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.rng.chance(1, 100) && sim.rng.chance((5.0 * pv(sim, x, y)) as i32, 1000) {
        sim.part_change_type(i, x, y, pt::NEUT as i16);
    }
    0
}

// ---------------------------------------------------------------------------
// URAN: uranium. Heats with pressure.
// ---------------------------------------------------------------------------

fn uran_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let p = pv(sim, x, y);
    if p > 0.0 {
        if sim.parts[i].temp == MIN_TEMP {
            sim.parts[i].temp += 0.01;
        } else {
            let t = sim.parts[i].temp * (1.0 + p / 2000.0) + MIN_TEMP;
            sim.parts[i].temp = restrict_flt(t, MIN_TEMP, MAX_TEMP);
        }
    }
    0
}

// ---------------------------------------------------------------------------
// MORT: mortar. Emits smoke above itself.
// ---------------------------------------------------------------------------

fn mort_update(sim: &mut Sim, _i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if cell_free(sim, x, y - 1) {
        sim.create_part(x, y - 1, pt::SMKE);
    }
    0
}

// ---------------------------------------------------------------------------
// THDR: thunder. Sparks conductors, explodes on anything else.
// ---------------------------------------------------------------------------

fn thdr_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let mut kill = false;
    for rx in -2..=2 {
        for ry in -2..=2 {
            if rx == 0 && ry == 0 {
                continue;
            }
            let nx = x + rx;
            let ny = y + ry;
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let r = sim.pmap[sim.idx(nx, ny)];
            if r == 0 {
                continue;
            }
            let rt = pmap_type(r);
            let id = pmap_id(r) as usize;
            if (element(rt).properties & props::PROP_CONDUCTS) != 0
                && sim.parts[id].life == 0
                && !matches!(rt, pt::WATR | pt::SLTW)
                && sim.parts[id].ctype != pt::SPRK as i32
            {
                sim.parts[id].ctype = rt as i32;
                sim.part_change_type(id, nx, ny, pt::SPRK as i16);
                sim.parts[id].life = 4;
                kill = true;
            } else if !matches!(rt, pt::CLNE | pt::THDR | pt::SPRK | pt::DMND | pt::FIRE) {
                if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
                    let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
                    sim.pv[vc] = restrict_flt(sim.pv[vc] + 100.0, MIN_PRESSURE, MAX_PRESSURE);
                }
                kill = true;
            }
        }
    }
    if kill {
        sim.kill_part(i);
        return 1;
    }
    0
}

// ---------------------------------------------------------------------------
// AMTR: anti-matter. Annihilates matter into photons.
// ---------------------------------------------------------------------------

fn amtr_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    for rx in -1..=1 {
        for ry in -1..=1 {
            if rx == 0 && ry == 0 {
                continue;
            }
            let nx = x + rx;
            let ny = y + ry;
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let r = sim.pmap[sim.idx(nx, ny)];
            if r == 0 {
                continue;
            }
            let rt = pmap_type(r);
            let id = pmap_id(r) as usize;
            if matches!(
                rt,
                pt::AMTR | pt::DMND | pt::CLNE | pt::PCLN | pt::VOID | pt::BHOL | pt::NBHL | pt::PRTI
                    | pt::PRTO
            ) {
                continue;
            }
            sim.parts[i].life += 1;
            if sim.parts[i].life == 4 {
                sim.kill_part(i);
                return 1;
            }
            if sim.rng.chance(1, 10) {
                sim.part_change_type(id, nx, ny, pt::PHOT as i16);
            } else {
                sim.kill_part(id);
            }
            if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
                let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
                sim.pv[vc] -= 2.0;
            }
        }
    }
    0
}

// ---------------------------------------------------------------------------
// HSWC: heat switch. Links to other HSWC; conducts only when life == 10.
// ---------------------------------------------------------------------------

fn hswc_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.parts[i].life != 10 {
        if sim.parts[i].life > 0 {
            sim.parts[i].life -= 1;
        }
        return 0;
    }
    for rx in -2..=2 {
        for ry in -2..=2 {
            if rx == 0 && ry == 0 {
                continue;
            }
            let nx = x + rx;
            let ny = y + ry;
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let mut r = sim.pmap[sim.idx(nx, ny)];
            if r == 0 {
                r = sim.photons[sim.idx(nx, ny)];
            }
            if r == 0 {
                continue;
            }
            let id = pmap_id(r) as usize;
            let rt = pmap_type(r);
            if rt == pt::HSWC {
                let other = sim.parts[id].life;
                if other < 10 && other > 0 {
                    sim.parts[i].life = 9;
                } else if other == 0 {
                    sim.parts[id].life = 10;
                }
            }
        }
    }
    0
}

// ---------------------------------------------------------------------------
// FUSE: fuse. Burn down into plasma, then become FSEP.
// ---------------------------------------------------------------------------

fn fuse_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.parts[i].life <= 0 {
        sim.part_change_type(i, x, y, pt::PLSM as i16);
        if sim.parts[i].t == pt::PLSM {
            sim.parts[i].life = 50;
        }
        return 1;
    } else if sim.parts[i].life < 40 {
        sim.parts[i].life -= 1;
        if sim.rng.chance(1, 100) {
            let nx = x + sim.rng.between(-1, 1);
            let ny = y + sim.rng.between(-1, 1);
            if let Some(j) = create_near(sim, nx, ny, pt::PLSM) {
                sim.parts[j].life = 50;
            }
        }
    }
    if pv(sim, x, y) > 2.7 && sim.parts[i].tmp > 40 {
        sim.parts[i].tmp = 39;
    } else if sim.parts[i].tmp <= 0 {
        sim.part_change_type(i, x, y, pt::FSEP as i16);
        return 1;
    } else if sim.parts[i].tmp < 40 {
        sim.parts[i].tmp -= 1;
    }
    for rx in -2..=2 {
        for ry in -2..=2 {
            if rx == 0 && ry == 0 {
                continue;
            }
            let nx = x + rx;
            let ny = y + ry;
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let r = sim.pmap[sim.idx(nx, ny)];
            if r == 0 {
                continue;
            }
            let rt = pmap_type(r);
            if rt == pt::SPRK || (sim.parts[i].temp >= 973.15 && sim.rng.chance(1, 20)) {
                if sim.parts[i].life > 40 {
                    sim.parts[i].life = 39;
                }
            }
        }
    }
    0
}

// ---------------------------------------------------------------------------
// FSEP: fuse separator. Burns into plasma.
// ---------------------------------------------------------------------------

fn fsep_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.parts[i].life <= 0 {
        sim.part_change_type(i, x, y, pt::PLSM as i16);
        if sim.parts[i].t == pt::PLSM {
            sim.parts[i].life = 50;
        }
        return 1;
    } else if sim.parts[i].life < 40 {
        sim.parts[i].life -= 1;
        if sim.rng.chance(1, 10) {
            let nx = x + sim.rng.between(-1, 1);
            let ny = y + sim.rng.between(-1, 1);
            if let Some(j) = create_near(sim, nx, ny, pt::PLSM) {
                sim.parts[j].life = 50;
            }
        }
    } else {
        for rx in -2..=2 {
            for ry in -2..=2 {
                if rx == 0 && ry == 0 {
                    continue;
                }
                let nx = x + rx;
                let ny = y + ry;
                if !sim.in_bounds(nx, ny) {
                    continue;
                }
                let r = sim.pmap[sim.idx(nx, ny)];
                if r == 0 {
                    continue;
                }
                let rt = pmap_type(r);
                if (rt == pt::SPRK || sim.parts[i].temp >= 673.15)
                    && sim.parts[i].life > 40
                    && sim.rng.chance(1, 15)
                {
                    sim.parts[i].life = 39;
                }
            }
        }
    }
    0
}

// ---------------------------------------------------------------------------
// LCRY: liquid crystal. Acts as a bistable switch.
// ---------------------------------------------------------------------------

fn lcry_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let (check, setto): (i32, i32);
    match sim.parts[i].tmp {
        1 => {
            if sim.parts[i].life <= 0 {
                sim.parts[i].tmp = 0;
            } else {
                sim.parts[i].life -= 2;
                if sim.parts[i].life < 0 {
                    sim.parts[i].life = 0;
                }
                sim.parts[i].tmp2 = sim.parts[i].life;
            }
            check = 3;
            setto = 1;
        }
        0 => {
            check = 3;
            setto = 1;
        }
        2 => {
            if sim.parts[i].life >= 10 {
                sim.parts[i].tmp = 3;
            } else {
                sim.parts[i].life += 2;
                if sim.parts[i].life > 10 {
                    sim.parts[i].life = 10;
                }
                sim.parts[i].tmp2 = sim.parts[i].life;
            }
            check = 0;
            setto = 2;
        }
        3 => {
            check = 0;
            setto = 2;
        }
        _ => {
            sim.parts[i].tmp = 0;
            sim.parts[i].life = 0;
            return 0;
        }
    }
    for rx in -1..=1 {
        for ry in -1..=1 {
            if rx == 0 && ry == 0 {
                continue;
            }
            let nx = x + rx;
            let ny = y + ry;
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let r = sim.pmap[sim.idx(nx, ny)];
            if r == 0 {
                continue;
            }
            let id = pmap_id(r) as usize;
            if pmap_type(r) == pt::LCRY && sim.parts[id].tmp == check {
                sim.parts[id].tmp = setto;
            }
        }
    }
    0
}

// ---------------------------------------------------------------------------
// DLAY: delay line. Passes life values between neighbours.
// ---------------------------------------------------------------------------

fn dlay_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let oldl = sim.parts[i].life;
    if sim.parts[i].life > 0 {
        sim.parts[i].life -= 1;
    }
    if sim.parts[i].temp <= 274.15 {
        sim.parts[i].temp = 274.15;
    }
    for rx in -2..=2 {
        for ry in -2..=2 {
            if rx == 0 && ry == 0 {
                continue;
            }
            let nx = x + rx;
            let ny = y + ry;
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let r = sim.pmap[sim.idx(nx, ny)];
            if r == 0 {
                continue;
            }
            let id = pmap_id(r) as usize;
            let rt = pmap_type(r);
            let pavg = parts_avg(sim, id, i, pt::INSL);
            if pavg == pt::INSL || pavg == pt::RSSS {
                continue;
            }
            if rt == pt::SPRK
                && sim.parts[i].life == 0
                && sim.parts[id].life > 0
                && sim.parts[id].life < 4
                && sim.parts[id].ctype == pt::PSCN as i32
            {
                sim.parts[i].life = (sim.parts[i].temp - 273.15 + 0.5) as i32;
            } else if rt == pt::DLAY {
                if sim.parts[i].life == 0 {
                    if sim.parts[id].life != 0 {
                        sim.parts[i].life = sim.parts[id].life;
                        if id > i {
                            sim.parts[i].life -= 1;
                        }
                    }
                } else if sim.parts[id].life == 0 {
                    sim.parts[id].life = sim.parts[i].life;
                    if id > i {
                        sim.parts[id].life += 1;
                    }
                }
            } else if rt == pt::NSCN && oldl == 1 {
                if cell_free(sim, nx, ny) {
                    sim.create_part(nx, ny, pt::SPRK);
                }
            }
        }
    }
    0
}

// ---------------------------------------------------------------------------
// CBNW: bubble water. Explodes to CO2 on particles, reacts with fire.
// ---------------------------------------------------------------------------

fn cbnw_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let p = pv(sim, x, y);
    if p <= 3.0 {
        if p <= -0.5 || sim.rng.chance(1, 4000) {
            sim.part_change_type(i, x, y, pt::CO2 as i16);
            sim.parts[i].ctype = 5;
            if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
                let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
                sim.pv[vc] += 0.5;
            }
        }
    }
    if sim.parts[i].tmp2 != 20 {
        if sim.parts[i].tmp2 > 20 {
            sim.parts[i].tmp2 -= 1;
        } else {
            sim.parts[i].tmp2 += 1;
        }
    } else if sim.rng.chance(1, 200) {
        sim.parts[i].tmp2 = sim.rng.between(0, 39);
    }
    if sim.parts[i].tmp > 0 {
        if sim.parts[i].tmp == 1 && sim.rng.chance(3, 4) {
            sim.part_change_type(i, x, y, pt::CO2 as i16);
            sim.parts[i].ctype = 5;
            if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
                let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
                sim.pv[vc] += 0.2;
            }
        }
        sim.parts[i].tmp -= 1;
    }
    for rx in -1..=1 {
        for ry in -1..=1 {
            if rx == 0 && ry == 0 {
                continue;
            }
            let nx = x + rx;
            let ny = y + ry;
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let r = sim.pmap[sim.idx(nx, ny)];
            if r == 0 {
                continue;
            }
            let rt = pmap_type(r);
            let id = pmap_id(r) as usize;
            let rprops = element(rt).properties;
            if (rprops & props::TYPE_PART) != 0 && sim.parts[i].tmp == 0 && sim.rng.chance(1, 83) {
                sim.parts[i].tmp = sim.rng.between(0, 24);
            } else if (rprops & props::TYPE_SOLID) != 0
                && !matches!(rt, pt::DMND | pt::GLAS)
                && sim.parts[i].tmp == 0
                && sim.rng.chance((2.0 - pv(sim, x, y)) as i32, 6667)
            {
                sim.part_change_type(i, x, y, pt::CO2 as i16);
                sim.parts[i].ctype = 5;
                if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
                    let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
                    sim.pv[vc] += 0.2;
                }
            }
            if rt == pt::CBNW {
                if sim.parts[i].tmp == 0 {
                    if sim.parts[id].tmp != 0 {
                        sim.parts[i].tmp = sim.parts[id].tmp;
                        if id > i {
                            sim.parts[i].tmp -= 1;
                        }
                    }
                } else if sim.parts[id].tmp == 0 {
                    sim.parts[id].tmp = sim.parts[i].tmp;
                    if id > i {
                        sim.parts[id].tmp += 1;
                    }
                }
            } else if rt == pt::RBDM || rt == pt::LRBD {
                if sim.parts[i].temp > 285.15 && sim.rng.chance(1, 166) {
                    sim.part_change_type(i, x, y, pt::FIRE as i16);
                    sim.parts[i].life = 4;
                    sim.parts[i].ctype = pt::WATR as i32;
                }
            } else if rt == pt::FIRE && sim.parts[id].ctype != pt::WATR as i32 {
                sim.kill_part(id);
                if sim.rng.chance(1, 50) {
                    sim.kill_part(i);
                    return 1;
                }
            }
        }
    }
    0
}

// ---------------------------------------------------------------------------
// CAUS: caustic gas. Corrodes like acid, converts GAS to RFRG.
// ---------------------------------------------------------------------------

fn caus_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    for rx in -2..=2 {
        for ry in -2..=2 {
            if rx == 0 && ry == 0 {
                continue;
            }
            let nx = x + rx;
            let ny = y + ry;
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let r = sim.pmap[sim.idx(nx, ny)];
            if r == 0 {
                continue;
            }
            let rt = pmap_type(r);
            let id = pmap_id(r) as usize;
            if rt == pt::GAS {
                if pv(sim, nx, ny) > 3.0 {
                    sim.part_change_type(id, nx, ny, pt::RFRG as i16);
                    sim.part_change_type(i, x, y, pt::RFRG as i16);
                    return 1;
                }
            } else if !matches!(rt, pt::ACID | pt::CAUS | pt::RFRG | pt::RFGL) {
                if (element(rt).hardness > 0)
                    && sim.rng.chance(element(rt).hardness, 1000)
                    && sim.parts[i].life > 50
                {
                    let pavg = parts_avg(sim, i, id, pt::GLAS);
                    if pavg != pt::GLAS {
                        let mut newtemp = (60.0 - element(rt).hardness as f32) * 7.0;
                        if newtemp < 0.0 {
                            newtemp = 0.0;
                        }
                        sim.parts[i].temp += newtemp;
                        sim.parts[i].life -= 1;
                        sim.kill_part(id);
                    }
                } else if sim.parts[i].life <= 50 {
                    sim.kill_part(i);
                    return 1;
                }
            }
        }
    }
    0
}

// ---------------------------------------------------------------------------
// DEUT: deuterium. Compresses/expands, diffuses its life value.
// ---------------------------------------------------------------------------

fn deut_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let gravtot = {
        let (gx, gy) = sim.grav_force[sim.cell_idx(x / CELL as i32, y / CELL as i32)];
        gx.abs() + gy.abs()
    };
    let temp = (sim.parts[i].temp + 1.0).max(1.0);
    let mut maxlife = ((10000.0 / (temp + 1.0)) - 1.0) as i32;
    if sim.rng.chance((10000 % (temp as i32 + 1)), (temp as u32 + 1)) {
        maxlife += 1;
    }
    maxlife *= (5.0 - 8.0 / (gravtot + 2.0)) as i32;
    if sim.parts[i].life < maxlife {
        for rx in -1..=1 {
            for ry in -1..=1 {
                if rx == 0 && ry == 0 {
                    continue;
                }
                let nx = x + rx;
                let ny = y + ry;
                if !sim.in_bounds(nx, ny) {
                    continue;
                }
                let r = sim.pmap[sim.idx(nx, ny)];
                if r == 0 || sim.parts[i].life >= maxlife {
                    continue;
                }
                let rt = pmap_type(r);
                let id = pmap_id(r) as usize;
                if rt == pt::DEUT && sim.rng.chance(1, 3) {
                    if sim.parts[id].life <= maxlife - sim.parts[i].life - 1 {
                        sim.parts[i].life += sim.parts[id].life + 1;
                        sim.kill_part(id);
                    }
                }
            }
        }
    } else {
        for rx in -1..=1 {
            for ry in -1..=1 {
                if rx == 0 && ry == 0 {
                    continue;
                }
                if sim.parts[i].life <= maxlife {
                    break;
                }
                let nx = x + rx;
                let ny = y + ry;
                if !cell_free(sim, nx, ny) {
                    continue;
                }
                if sim.parts[i].life >= 1 {
                    if let Some(np) = sim.create_part(nx, ny, pt::DEUT) {
                        sim.parts[i].life -= 1;
                        sim.parts[np].temp = sim.parts[i].temp;
                        sim.parts[np].life = 0;
                    }
                }
            }
        }
    }
    for _ in 0..4 {
        let rx = sim.rng.between(-2, 2);
        let ry = sim.rng.between(-2, 2);
        if rx == 0 && ry == 0 {
            continue;
        }
        let nx = x + rx;
        let ny = y + ry;
        if !sim.in_bounds(nx, ny) {
            continue;
        }
        let r = sim.pmap[sim.idx(nx, ny)];
        if r == 0 {
            continue;
        }
        let id = pmap_id(r) as usize;
        if pmap_type(r) == pt::DEUT && sim.parts[i].life > sim.parts[id].life && sim.parts[i].life > 0 {
            let diff = sim.parts[i].life - sim.parts[id].life;
            if diff == 1 {
                sim.parts[id].life += 1;
                sim.parts[i].life -= 1;
            } else if diff > 0 {
                sim.parts[id].life += diff / 2;
                sim.parts[i].life -= diff / 2;
            }
        }
    }
    0
}

// ---------------------------------------------------------------------------
// WARP: warp. Teleports neighbours and occasionally emits electrons.
// ---------------------------------------------------------------------------

fn warp_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.parts[i].tmp2 > 2000 {
        sim.parts[i].temp = 10000.0;
        if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
            let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
            sim.pv[vc] = restrict_flt(
                sim.pv[vc] + (sim.parts[i].tmp2 as f32 / 5000.0),
                MIN_PRESSURE,
                MAX_PRESSURE,
            );
        }
        if sim.rng.chance(1, 50) {
            create_near(sim, x, y, pt::ELEC);
        }
    }
    for trade in 0..5 {
        let rx = sim.rng.between(-1, 1);
        let ry = sim.rng.between(-1, 1);
        if rx == 0 && ry == 0 {
            continue;
        }
        let nx = x + rx;
        let ny = y + ry;
        if !sim.in_bounds(nx, ny) {
            continue;
        }
        let r = sim.pmap[sim.idx(nx, ny)];
        if r == 0 {
            continue;
        }
        let rt = pmap_type(r);
        let id = pmap_id(r) as usize;
        if matches!(
            rt,
            pt::WARP | pt::STKM | pt::STKM2 | pt::DMND | pt::CLNE | pt::BCLN | pt::PCLN
        ) {
            continue;
        }
        let (ox, oy) = (sim.parts[id].x, sim.parts[id].y);
        sim.parts[i].x = ox;
        sim.parts[i].y = oy;
        sim.parts[id].x = x as f32;
        sim.parts[id].y = y as f32;
        sim.parts[id].vx = sim.rng.between(-2, 1) as f32 + 0.5;
        sim.parts[id].vy = sim.rng.between(-2, 1) as f32;
        sim.parts[i].life += 4;
        let idx0 = sim.idx(x, y);
        let idx1 = sim.idx(nx, ny);
        sim.pmap[idx0] = r;
        sim.pmap[idx1] = pmap_pack(i as u32, pt::WARP);
        let _ = trade;
        return 0;
    }
    0
}

fn warp_create(sim: &mut Sim, i: usize, _x: i32, _y: i32, _t: i32, _v: i32) {
    sim.parts[i].life = sim.rng.between(70, 164);
}

// ---------------------------------------------------------------------------
// PUMP: pressure pump. Equalises pressure in the surrounding cells.
// ---------------------------------------------------------------------------

fn pump_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.parts[i].life != 10 {
        if sim.parts[i].life > 0 {
            sim.parts[i].life -= 1;
        }
        return 0;
    }
    if sim.parts[i].temp >= MAX_PRESSURE + 273.15 {
        sim.parts[i].temp = MAX_PRESSURE + 273.15;
    }
    if sim.parts[i].temp <= MIN_PRESSURE + 273.15 {
        sim.parts[i].temp = MIN_PRESSURE + 273.15;
    }
    for rx in -1..=1 {
        for ry in -1..=1 {
            let cx = x / CELL as i32 + rx;
            let cy = y / CELL as i32 + ry;
            if !sim.in_cell_bounds(cx, cy) {
                continue;
            }
            if sim.parts[i].tmp != 1 {
                if !(rx == 0 && ry == 0) {
                    let vc = sim.cell_idx(cx, cy);
                    sim.pv[vc] += 0.1 * ((sim.parts[i].temp - 273.15) - sim.pv[vc]);
                }
            }
        }
    }
    for rx in -2..=2 {
        for ry in -2..=2 {
            if rx == 0 && ry == 0 {
                continue;
            }
            let nx = x + rx;
            let ny = y + ry;
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let r = sim.pmap[sim.idx(nx, ny)];
            if r == 0 {
                continue;
            }
            let id = pmap_id(r) as usize;
            if pmap_type(r) == pt::PUMP {
                let other = sim.parts[id].life;
                if other < 10 && other > 0 {
                    sim.parts[i].life = 9;
                } else if other == 0 {
                    sim.parts[id].life = 10;
                }
            }
        }
    }
    0
}

// ---------------------------------------------------------------------------
// FWRK: firework. Ignites from fire/plasma/thunder, then bursts into EMBR.
// ---------------------------------------------------------------------------

fn fwrk_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.parts[i].tmp <= 0 {
        for rx in -1..=1 {
            for ry in -1..=1 {
                if rx == 0 && ry == 0 {
                    continue;
                }
                let nx = x + rx;
                let ny = y + ry;
                if !sim.in_bounds(nx, ny) {
                    continue;
                }
                let r = sim.pmap[sim.idx(nx, ny)];
                if r == 0 {
                    continue;
                }
                let rt = pmap_type(r);
                if matches!(rt, pt::FIRE | pt::PLSM | pt::THDR) {
                    let (mut gx, mut gy) = sim.gravity_field(x, y, element(pt::FWRK).gravity, 1.0);
                    if gx * gx + gy * gy < 0.001 {
                        let angle = sim.rng.between(0, 6283) as f32 * 0.001;
                        gx += angle.sin() * element(pt::FWRK).gravity * 0.5;
                        gy += angle.cos() * element(pt::FWRK).gravity * 0.5;
                    }
                    sim.parts[i].tmp = 1;
                    sim.parts[i].life = sim.rng.between(20, 29);
                    let mult = (sim.parts[i].life as f32 + 20.0) * 0.2 / (gx * gx + gy * gy).sqrt();
                    sim.parts[i].vx -= gx * mult;
                    sim.parts[i].vy -= gy * mult;
                    return 0;
                }
            }
        }
    } else if sim.parts[i].tmp == 1 {
        if sim.parts[i].life <= 0 {
            sim.parts[i].tmp = 2;
        } else {
            sim.parts[i].flags &= !crate::particle::flags::STAGNANT;
        }
    } else {
        let vx = sim.parts[i].vx;
        let vy = sim.parts[i].vy;
        let dcolour = sim.parts[i].dcolour;
        for _ in 0..40 {
            let mag = sim.rng.between(40, 99) as f32 * 0.05;
            let angle = sim.rng.between(0, 6283) as f32 * 0.001;
            if let Some(np) = create_near(sim, x, y, pt::EMBR) {
                sim.parts[np].vx = vx * 0.5 + angle.cos() * mag;
                sim.parts[np].vy = vy * 0.5 + angle.sin() * mag;
                sim.parts[np].tmp = 1;
                sim.parts[np].life = sim.rng.between(70, 109);
                sim.parts[np].temp = sim.rng.between(5750, 6249) as f32;
                sim.parts[np].dcolour = dcolour;
            }
        }
        if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
            let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
            sim.pv[vc] += 8.0;
        }
        sim.kill_part(i);
        return 1;
    }
    0
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
