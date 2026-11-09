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
use crate::sim::{pmap_id, pmap_type, restrict_flt, Sim, CELL, MAX_TEMP, MIN_TEMP, MAX_PRESSURE, MIN_PRESSURE};

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
        pt::ANAR => anar_update,
        pt::QRTZ | pt::PQRT => qrtz_update,
        pt::VINE => vine_update,
        pt::INVIS => invis_update,
        pt::SHLD1 => shld1_update,
        pt::SHLD2 => shld2_update,
        pt::SHLD3 => shld3_update,
        pt::SHLD4 => shld4_update,
        pt::SING => sing_update,
        pt::BREC => brec_update,
        pt::ELEC => elec_update,
        pt::ACEL => acel_update,
        pt::DCEL => dcel_update,
        pt::BANG => bang_update,
        pt::IGNT => ignt_update,
        pt::BOYL => boyl_update,
        pt::TTAN => ttan_update,
        pt::EMBR => embr_update,
        pt::NWHL => nwhl_update,
        pt::MERC => merc_update,
        pt::CLST => clst_update,
        pt::WIRE => wire_update,
        pt::GBMB => gbmb_update,
        pt::FRAY => fray_update,
        pt::RPEL => rpel_update,
        pt::DTEC => dtec_update,
        pt::DMG => dmg_update,
        pt::TSNS => tsns_update,
        pt::VIBR => vibr_update,
        pt::GOLD => gold_update,
        pt::TUNG => tung_update,
        pt::PSNS => psns_update,
        pt::PROT => prot_update,
        pt::VIRS => virs_update,
        pt::GRVT => grvt_update,
        pt::RFRG => rfrg_update,
        pt::LSNS => lsns_update,
        pt::LDTC => ldtc_update,
        pt::VSNS => vsns_update,
        pt::CRMC => crmc_update,
        pt::HEAC => heac_update,
        pt::POLO => polo_update,
        pt::FIRW => firw_update,
        pt::CONV => conv_update,
        pt::EXOT => exot_update,
        pt::GEL => gel_update,
        pt::STOR => stor_update,
        pt::SPNG => spng_update,
        pt::FRZZ => frzz_update,
        pt::FRZW => frzw_update,
        pt::GRAV => grav_update,
        pt::BIZR => bizr_update,
        pt::ISOZ => isoz_update,
        pt::ISZS => iszs_update,
        pt::SLCN => slcn_update,
        pt::RSST => rsst_update,
        pt::RSSS => rsss_update,
        pt::GPMP => gpmp_update,
        pt::LIGH => ligh_update,
        pt::PHOT => phot_update,
        pt::C5 => c5_update,
        pt::TRON => tron_update,
        pt::PTNM => ptnm_update,
        pt::LITH => lith_update,
        pt::BASE => base_update,
        pt::DRAY => dray_update,
        pt::SEED => seed_update,
        pt::PRTI => prti_update,
        pt::PRTO => prto_update,
        pt::PIPE | pt::PPIP => pipe_update,
        pt::WIFI => wifi_update,
        pt::ARAY => aray_update,
        pt::CRAY => cray_update,
        pt::PSTN => pstn_update,
        pt::SOAP => soap_update,
        pt::STKM => stkm_update,
        pt::STKM2 => stkm2_update,
        pt::FIGH => figh_update,
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
        pt::QRTZ => qrtz_create,
        pt::SING => sing_create,
        pt::ELEC => elec_create,
        pt::CLST => clst_create,
        pt::TUNG => tung_create,
        pt::PROT => prot_create,
        pt::GRVT => grvt_create,
        pt::CRMC => crmc_create,
        pt::SLCN => slcn_create,
        pt::LIGH => ligh_create,
        pt::PHOT => phot_create,
        pt::TRON => tron_create,
        pt::PTNM => ptnm_create,
        pt::FIGH => figh_create,
        _ => return None,
    })
}

pub fn create_allowed_fn(t: u16) -> Option<CreateAllowedFn> {
    Some(match t {
        pt::STKM => stkm_allowed,
        pt::STKM2 => stkm2_allowed,
        pt::FIGH => figh_allowed,
        _ => return None,
    })
}

pub fn change_type_fn(t: u16) -> Option<ChangeTypeFn> {
    Some(match t {
        pt::FIGH => figh_change_type,
        _ => return None,
    })
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

/// Iterate the 5x5 neighbourhood like the reference element loops.
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

/// Reference `Simulation::FloodINST`: spark every connected `INST` cell.
/// We use a 4-connected flood (the reference scanline adds special handling for
/// 1-px wire crossings; the observable result ? the whole wire sparks ? matches).
fn flood_inst(sim: &mut Sim, sx: i32, sy: i32) {
    let mut stack = vec![(sx, sy)];
    let mut seen = std::collections::HashSet::new();
    while let Some((x, y)) = stack.pop() {
        if !sim.in_bounds(x, y) || !seen.insert((x, y)) {
            continue;
        }
        let r = sim.pmap_at(x, y);
        if r == 0 {
            continue;
        }
        let id = pmap_id(r) as usize;
        let rt = pmap_type(r);
        let sparkable = rt == pt::INST && sim.parts[id].life == 0;
        let already = rt == pt::SPRK && sim.parts[id].ctype == pt::INST as i32;
        if !sparkable && !already {
            continue;
        }
        if sparkable && sim.part_change_type(id, x, y, pt::SPRK as i16) {
            sim.parts[id].life = 4;
            sim.parts[id].ctype = pt::INST as i32;
        }
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            stack.push((x + dx, y + dy));
        }
    }
}

fn sprk_update(sim: &mut Sim, i: usize, x: i32, y: i32, surround_space: i32, nt: i32) -> i32 {
    // Reference `SPRK::update` calls `Element_FIRE_update` every tick.
    fire_update(sim, i, x, y, surround_space, nt);
    if i >= sim.parts.len() || sim.parts[i].t == 0 {
        return 1;
    }
    let ct = sim.parts[i].ctype;

    if sim.parts[i].life <= 0 {
        // Restore the conductor, with the reference's per-type durations.
        if matches!(ct, c if c == pt::WATR as i32 || c == pt::SLTW as i32 || c == pt::PSCN as i32
            || c == pt::NSCN as i32 || c == pt::ETRD as i32 || c == pt::INWR as i32)
        {
            sim.parts[i].temp = crate::sim::R_TEMP + 273.15;
        }
        let mut restore = ct;
        if restore <= 0
            || (restore as usize) >= crate::elements_generated::PT_NUM
            || !element(restore as u16).enabled
        {
            restore = pt::METL as i32;
        }
        sim.parts[i].ctype = 0;
        sim.parts[i].life = 4;
        let mut life = 4;
        if restore == pt::WATR as i32 {
            life = 64;
        } else if restore == pt::SLTW as i32 {
            life = 54;
        } else if restore == pt::SWCH as i32 {
            life = 14;
        } else if restore == pt::RSST as i32 {
            sim.kill_part(i);
            return 1;
        }
        if sim.part_change_type(i, x, y, restore as i16) {
            sim.parts[i].life = life;
        }
        return 1;
    }

    // ctype-specific behaviour while sparking.
    if ct == pt::SPRK as i32 {
        sim.kill_part(i);
        return 1;
    } else if ct == pt::NTCT as i32 || ct == pt::PTCT as i32 {
        ntct_update(sim, i, x, y, surround_space, nt);
    } else if ct == pt::NBLE as i32 {
        if sim.parts[i].life <= 1 && (sim.parts[i].tmp & 0x1) == 0 {
            sim.parts[i].life = sim.rng.between(50, 199);
            sim.part_change_type(i, x, y, pt::PLSM as i16);
            sim.parts[i].ctype = pt::NBLE as i32;
            if sim.parts[i].temp > 5273.15 {
                sim.parts[i].tmp |= 0x4;
            }
            sim.parts[i].temp = 3500.0;
            if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
                let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
                sim.pv[vc] += 1.0;
            }
        }
    } else if ct == pt::TUNG as i32 {
        if sim.parts[i].temp < 3595.0 {
            sim.parts[i].temp += sim.rng.between(-4, 15) as f32;
        }
    }

    sim.emp_decor = (sim.emp_decor + 3).min(40);

    let life = sim.parts[i].life;
    let mut ignite: Vec<(usize, i32, i32)> = Vec::new();
    for_neighbours!(sim, x, y, rx, ry, nx, ny, {
        let r = sim.pmap[sim.idx(nx, ny)];
        if r == 0 {
            continue;
        }
        let id = pmap_id(r) as usize;
        let receiver = pmap_type(r);
        let sender = ct;
        let pavg = sim.parts_avg_type(id, i, pt::INSL);
        let insu = pavg == pt::INSL || pavg == pt::RSSS;

        // Pre-conduction activation of powered devices.
        match receiver {
            pt::SWCH => {
                if !insu && life < 4 {
                    if sender == pt::PSCN as i32 && sim.parts[id].life < 10 {
                        sim.parts[id].life = 10;
                    } else if sender == pt::NSCN as i32 {
                        sim.parts[id].ctype = 0;
                        sim.parts[id].life = 9;
                    }
                }
                continue;
            }
            pt::SPRK => {
                if !insu && life < 4 {
                    if sim.parts[id].ctype == pt::SWCH as i32 && sender == pt::NSCN as i32 {
                        sim.part_change_type(id, nx, ny, pt::SWCH as i16);
                        sim.parts[id].ctype = 0;
                        sim.parts[id].life = 9;
                    } else if matches!(sim.parts[id].ctype, c if c == pt::NTCT as i32 || c == pt::PTCT as i32)
                        && sender == pt::METL as i32
                    {
                        sim.parts[id].temp = 473.0;
                    }
                }
                continue;
            }
            pt::PUMP | pt::GPMP | pt::HSWC | pt::PBCN => {
                if life < 4 {
                    if sender == pt::PSCN as i32 {
                        sim.parts[id].life = 10;
                    } else if sender == pt::NSCN as i32 && sim.parts[id].life >= 10 {
                        sim.parts[id].life = 9;
                    }
                }
                continue;
            }
            pt::LCRY => {
                if rx.abs() < 2 && ry.abs() < 2 && life < 4 {
                    if sender == pt::PSCN as i32 && sim.parts[id].tmp == 0 {
                        sim.parts[id].tmp = 2;
                    } else if sender == pt::NSCN as i32 && sim.parts[id].tmp == 3 {
                        sim.parts[id].tmp = 1;
                    }
                }
                continue;
            }
            pt::PPIP => {
                // Reference SPRK.cpp: at life==3, PSCN/NSCN/INST flood-trigger
                // connected PPIP/PIPE (staged bits, promoted before consumption).
                if life == 3 && !insu {
                    if sender == pt::NSCN as i32
                        || sender == pt::PSCN as i32
                        || sender == pt::INST as i32
                    {
                        ppip_flood_trigger(sim, nx, ny, sender as u16);
                    }
                }
                continue;
            }
            pt::NTCT | pt::PTCT | pt::INWR => {
                if sender == pt::METL as i32 && !insu && life < 4 {
                    sim.parts[id].temp = 473.0;
                    if receiver == pt::NTCT || receiver == pt::PTCT {
                        continue;
                    }
                }
            }
            pt::EMP => {
                if sim.parts[id].life == 0 && life > 0 && life < 4 {
                    sim.emp_decor += 3;
                    if sim.emp_decor > 40 {
                        sim.emp_decor = 40;
                    }
                    sim.parts[id].life = 220;
                }
                continue;
            }
            _ => {}
        }

        if insu {
            continue;
        }
        let conducts = (element(receiver).properties & props::PROP_CONDUCTS) != 0
            || receiver == pt::INST
            || receiver == pt::QRTZ;
        if !conducts {
            continue;
        }
        if rx.abs() + ry.abs() >= 4
            && sender != pt::SWCH as i32
            && receiver != pt::SWCH
        {
            continue;
        }
        // Sender/receiver-specific conduction rules.
        let allow = if receiver as i32 == sender && receiver != pt::INST && receiver != pt::QRTZ {
            true
        } else {
            match sender {
                c if c == pt::INST as i32 => receiver == pt::NSCN,
                c if c == pt::SWCH as i32 => !matches!(
                    receiver,
                    pt::PSCN | pt::NSCN | pt::WATR | pt::SLTW | pt::NTCT | pt::PTCT | pt::INWR
                ),
                c if c == pt::ETRD as i32 => matches!(
                    receiver,
                    pt::METL | pt::BMTL | pt::BRMT | pt::LRBD | pt::RBDM | pt::PSCN | pt::NSCN
                ),
                c if c == pt::NTCT as i32 => {
                    receiver == pt::PSCN
                        || (receiver == pt::NSCN && sim.parts[i].temp > 373.0)
                }
                c if c == pt::PTCT as i32 => {
                    receiver == pt::PSCN
                        || (receiver == pt::NSCN && sim.parts[i].temp < 373.0)
                }
                c if c == pt::INWR as i32 => {
                    receiver == pt::NSCN || receiver == pt::PSCN
                }
                _ => match receiver {
                    pt::QRTZ => {
                        (matches!(sender, c if c == pt::NSCN as i32 || c == pt::METL as i32
                            || c == pt::PSCN as i32 || c == pt::QRTZ as i32))
                            && (sim.parts[id].temp < 173.15 || pv(sim, nx, ny) > 8.0)
                    }
                    pt::NTCT => {
                        sender == pt::NSCN as i32
                            || (sender == pt::PSCN as i32 && sim.parts[id].temp > 373.0)
                    }
                    pt::PTCT => {
                        sender == pt::NSCN as i32
                            || (sender == pt::PSCN as i32 && sim.parts[id].temp < 373.0)
                    }
                    pt::INWR => sender == pt::NSCN as i32 || sender == pt::PSCN as i32,
                    pt::INST => sender == pt::PSCN as i32,
                    pt::NBLE => (sim.parts[i].tmp & 0x1) == 0,
                    pt::PSCN => sender != pt::NSCN as i32,
                    _ => true,
                },
            }
        };
        if !allow {
            continue;
        }
        if receiver == pt::INST {
            // Reference `Simulation::FloodINST`: spark the whole instant-wire
            // network, not just the adjacent cell.
            flood_inst(sim, nx, ny);
            continue;
        }
        if (receiver == pt::WATR || receiver == pt::SLTW || receiver == pt::RSST)
            && sim.parts[id].life == 0
            && life < 3
        {
            ignite.push((id, nx, ny));
        } else if sim.parts[id].life == 0 && life < 4 {
            ignite.push((id, nx, ny));
        }
    });

    for (id, nx, ny) in ignite {
        if id >= sim.parts.len() || sim.parts[id].t == 0 {
            continue;
        }
        let receiver = sim.parts[id].t;
        let life_set = match receiver {
            pt::WATR => 6,
            pt::SLTW => 5,
            pt::RSST => 5,
            _ => 4,
        };
        if sim.part_change_type(id, nx, ny, pt::SPRK as i16) {
            sim.parts[id].life = life_set;
            sim.parts[id].ctype = receiver as i32;
            if matches!(
                receiver,
                pt::METL | pt::BMTL | pt::BRMT | pt::PSCN | pt::NSCN | pt::ETRD | pt::NBLE | pt::IRON
            ) && sim.parts[id].temp + 10.0 < 673.0
            {
                sim.parts[id].temp += 10.0;
            }
        }
    }
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
    // PCLN is powered by sparks (reference `PCLN::update`).
    if sim.parts[i].t == pt::PCLN {
        if sim.parts[i].life > 0 && sim.parts[i].life != 10 {
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
                if rt == pt::SPRK && sim.parts[id].life > 0 && sim.parts[id].life < 4 {
                    if sim.parts[id].ctype == pt::PSCN as i32 {
                        sim.parts[i].life = 10;
                    } else if sim.parts[id].ctype == pt::NSCN as i32 {
                        sim.parts[i].life = 9;
                    }
                } else if rt == pt::PCLN {
                    if sim.parts[i].life == 10 && sim.parts[id].life < 10 && sim.parts[id].life > 0 {
                        sim.parts[i].life = 9;
                    } else if sim.parts[i].life == 0 && sim.parts[id].life == 10 {
                        sim.parts[i].life = 10;
                    }
                }
            }
        }
    }
    let ct = sim.parts[i].ctype;
    if !(ct > 0 && (ct as usize) < crate::elements_generated::PT_NUM) {
        return 0;
    }
    // PCLN clones only when powered (life != 0); CLNE always clones.
    if sim.parts[i].t == pt::PCLN && sim.parts[i].life == 0 {
        return 0;
    }
    // PBCN clones only when powered.
    if sim.parts[i].t == pt::PBCN && sim.parts[i].life == 0 {
        return 0;
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
// YEST: yeast. Converts to DYST near DYST; grows at 30?44 °C.
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

// ===========================================================================
// Batch 3: elements read from the pinned reference (see docs inventory)
// ===========================================================================

// --- ANAR ----------------------------------------------------------------
fn anar_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
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
            if pmap_type(r) == pt::CFLM && sim.rng.chance(1, 4) {
                sim.part_change_type(i, x, y, pt::CFLM as i16);
                sim.parts[i].life = sim.rng.between(50, 199);
                sim.parts[id].temp = 0.0;
                sim.parts[i].temp = 0.0;
                if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
                    let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
                    sim.pv[vc] -= 0.5;
                }
            }
        }
    }
    0
}

// --- QRTZ ----------------------------------------------------------------
fn qrtz_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let t = sim.parts[i].t;
    if t == pt::QRTZ {
        let press = (pv(sim, x, y) * 64.0) as i32;
        let tol = sim.parts[i].temp * 1.0666;
        let diff = (press - sim.parts[i].tmp3) as f32;
        if diff > tol || diff < -tol {
            sim.part_change_type(i, x, y, pt::PQRT as i16);
            sim.parts[i].life = 5;
        }
        sim.parts[i].tmp3 = press;
    }
    if sim.parts[i].life > 5 {
        sim.parts[i].life = 5;
    }
    if sim.parts[i].tmp != -1 {
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
                if pmap_type(r) == pt::SLTW && sim.rng.chance(1, 500) {
                    sim.kill_part(id);
                    sim.parts[i].tmp += 1;
                }
            }
        }
    }
    if sim.parts[i].tmp > 0 && (sim.parts[i].vx * sim.parts[i].vx + sim.parts[i].vy * sim.parts[i].vy) < 0.2 && sim.parts[i].life <= 0 {
        for _ in 0..9 {
            let rnd = sim.rng.gen();
            let rx = ((rnd % 5) as i32) - 2;
            let srx = ((rnd % 3) as i32) - 1;
            let rnd2 = rnd >> 3;
            let ry = ((rnd2 % 5) as i32) - 2;
            let sry = ((rnd2 % 3) as i32) - 1;
            if rx == 0 && ry == 0 {
                continue;
            }
            let sxx = x + srx;
            let syy = y + sry;
            if cell_free(sim, sxx, syy) && sim.parts[i].tmp != 0 {
                if let Some(np) = sim.create_part(sxx, syy, pt::QRTZ) {
                    let temp = sim.parts[i].temp;
                    sim.parts[np].temp = temp;
                    sim.parts[np].tmp2 = sim.parts[i].tmp2;
                    if sim.rng.chance(1, 2) {
                        let d = sim.rng.between(-1, 1);
                        sim.parts[np].tmp2 = (sim.parts[np].tmp2 + d).clamp(0, 10);
                    }
                    sim.parts[i].tmp -= 1;
                    if t == pt::PQRT {
                        sim.part_change_type(i, x, y, pt::QRTZ as i16);
                    }
                    if sim.rng.chance(1, 2) {
                        sim.parts[np].tmp = -1;
                    } else if sim.parts[i].tmp == 0 && sim.rng.chance(1, 15) {
                        sim.parts[i].tmp = -1;
                    }
                }
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
            if pmap_type(r) == pt::QRTZ && sim.parts[i].tmp > sim.parts[id].tmp && sim.parts[id].tmp >= 0 {
                let diff = sim.parts[i].tmp - sim.parts[id].tmp;
                if diff == 1 {
                    sim.parts[id].tmp += 1;
                    sim.parts[i].tmp -= 1;
                    break;
                } else if diff > 0 {
                    sim.parts[id].tmp += diff / 2;
                    sim.parts[i].tmp -= diff / 2;
                    break;
                }
            }
        }
    }
    0
}

fn qrtz_create(sim: &mut Sim, i: usize, x: i32, y: i32, _t: i32, _v: i32) {
    sim.parts[i].tmp2 = sim.rng.between(0, 10);
    sim.parts[i].tmp3 = (pv(sim, x, y) * 64.0) as i32;
}

// --- VINE ----------------------------------------------------------------
fn vine_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let mut rnd = sim.rng.gen();
    let rx = ((rnd % 3) as i32) - 1;
    rnd >>= 2;
    let ry = ((rnd % 3) as i32) - 1;
    rnd >>= 2;
    if rx != 0 || ry != 0 {
        let nx = x + rx;
        let ny = y + ry;
        if rnd % 15 == 0 {
            sim.part_change_type(i, x, y, pt::PLNT as i16);
        } else if cell_free(sim, nx, ny) {
            if let Some(np) = sim.create_part(nx, ny, pt::VINE) {
                let temp = sim.parts[i].temp;
                sim.parts[np].temp = temp;
                sim.part_change_type(i, x, y, pt::PLNT as i16);
            }
        }
    }
    if sim.parts[i].temp > 350.0 && sim.parts[i].temp > sim.parts[i].tmp2 as f32 {
        sim.parts[i].tmp2 = sim.parts[i].temp as i32;
    }
    0
}

// --- INVIS ---------------------------------------------------------------
fn invis_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let resistance = if sim.parts[i].tmp > 0 { sim.parts[i].tmp as f32 } else { 4.0 };
    let p = pv(sim, x, y);
    sim.parts[i].tmp2 = if p < -resistance || p > resistance { 1 } else { 0 };
    0
}

// --- SHLD1..4 ------------------------------------------------------------
fn shld1_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
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
            if rt == pt::SPRK && sim.parts[i].life == 0 {
                if sim.rng.chance(11, 40) {
                    sim.part_change_type(i, x, y, pt::SHLD2 as i16);
                    sim.parts[i].life = 7;
                }
                for nnx in -1..=1 {
                    for nny in -1..=1 {
                        let tx = nx + nnx;
                        let ty = ny + nny;
                        if cell_free(sim, tx, ty) {
                            sim.create_part(tx, ty, pt::SHLD1);
                        }
                    }
                }
            } else if rt == pt::SHLD3 && sim.rng.chance(2, 5) {
                sim.part_change_type(i, x, y, pt::SHLD2 as i16);
                sim.parts[i].life = 7;
            }
        }
    }
    0
}

fn shld2_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
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
                if sim.parts[i].life > 0 {
                    sim.create_part(nx, ny, pt::SHLD1);
                }
                continue;
            }
            let rt = pmap_type(r);
            if rt == pt::SPRK && sim.parts[i].life == 0 {
                if sim.rng.chance(1, 8) {
                    sim.part_change_type(i, x, y, pt::SHLD3 as i16);
                    sim.parts[i].life = 7;
                }
                for nnx in -1..=1 {
                    for nny in -1..=1 {
                        let tx = nx + nnx;
                        let ty = ny + nny;
                        if cell_free(sim, tx, ty) {
                            if let Some(np) = sim.create_part(tx, ty, pt::SHLD1) {
                                sim.parts[np].life = 7;
                            }
                        }
                    }
                }
            } else if rt == pt::SHLD4 && sim.rng.chance(2, 5) {
                sim.part_change_type(i, x, y, pt::SHLD3 as i16);
                sim.parts[i].life = 7;
            }
        }
    }
    0
}

fn shld3_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
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
                if sim.rng.chance(1, 2500) {
                    if let Some(np) = sim.create_part(nx, ny, pt::SHLD1) {
                        sim.parts[np].life = 7;
                        sim.part_change_type(i, x, y, pt::SHLD2 as i16);
                    }
                }
                continue;
            }
            let rt = pmap_type(r);
            let id = pmap_id(r) as usize;
            if rt == pt::SHLD1 && sim.parts[i].life > 3 {
                sim.part_change_type(id, nx, ny, pt::SHLD2 as i16);
                sim.parts[id].life = 7;
            } else if rt == pt::SPRK && sim.parts[i].life == 0 {
                if sim.rng.chance(3, 500) {
                    sim.part_change_type(i, x, y, pt::SHLD4 as i16);
                    sim.parts[i].life = 7;
                }
                for nnx in -1..=1 {
                    for nny in -1..=1 {
                        let tx = nx + nnx;
                        let ty = ny + nny;
                        if cell_free(sim, tx, ty) {
                            if let Some(np) = sim.create_part(tx, ty, pt::SHLD1) {
                                sim.parts[np].life = 7;
                            }
                        }
                    }
                }
            }
        }
    }
    0
}

fn shld4_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
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
                if sim.rng.chance(1, 5500) {
                    if let Some(np) = sim.create_part(nx, ny, pt::SHLD1) {
                        sim.parts[np].life = 7;
                        sim.part_change_type(i, x, y, pt::SHLD2 as i16);
                    }
                }
                continue;
            }
            let rt = pmap_type(r);
            let id = pmap_id(r) as usize;
            if rt == pt::SHLD2 && sim.parts[i].life > 3 {
                sim.part_change_type(id, nx, ny, pt::SHLD3 as i16);
                sim.parts[id].life = 7;
            } else if rt == pt::SPRK && sim.parts[i].life == 0 {
                for nnx in -1..=1 {
                    for nny in -1..=1 {
                        let tx = nx + nnx;
                        let ty = ny + nny;
                        if cell_free(sim, tx, ty) {
                            if let Some(np) = sim.create_part(tx, ty, pt::SHLD1) {
                                sim.parts[np].life = 7;
                            }
                        }
                    }
                }
            }
        }
    }
    0
}

// --- SING ----------------------------------------------------------------
fn sing_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let singularity = -(sim.parts[i].life as f32);
    if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
        let cx = x / CELL as i32;
        let cy = y / CELL as i32;
        for (dx, dy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, -1), (1, -1), (-1, 1)] {
            if sim.in_cell_bounds(cx + dx, cy + dy) {
                let vc = sim.cell_idx(cx + dx, cy + dy);
                let cur = sim.pv[vc];
                if cur < singularity {
                    sim.pv[vc] = restrict_flt(cur + 0.1 * (singularity - cur), MIN_PRESSURE, MAX_PRESSURE);
                }
            }
        }
    }
    if sim.parts[i].life < 1 {
        if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
            let cx = x / CELL as i32;
            let cy = y / CELL as i32;
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if sim.in_cell_bounds(cx + dx, cy + dy) {
                        let vc = sim.cell_idx(cx + dx, cy + dy);
                        sim.pv[vc] = restrict_flt(sim.pv[vc] + sim.parts[i].tmp as f32, MIN_PRESSURE, MAX_PRESSURE);
                    }
                }
            }
        }
        let spawncount = sim.parts[i].tmp.abs();
        let count = if spawncount > 255 { 0 } else { ((spawncount as f64 / 8.0).powi(2) * std::f64::consts::PI) as i32 };
        for _ in 0..count {
            let kind = sim.rng.gen() % 3;
            let t = match kind {
                0 => pt::PHOT,
                1 => pt::NEUT,
                _ => pt::ELEC,
            };
            if let Some(nb) = create_near(sim, x, y, t) {
                sim.parts[nb].life = sim.rng.between(0, 299);
                sim.parts[nb].temp = MAX_TEMP / 2.0;
                let angle = sim.rng.uniform01() * 2.0 * std::f32::consts::PI;
                let v = sim.rng.uniform01() * 5.0;
                sim.parts[nb].vx = v * angle.cos();
                sim.parts[nb].vy = v * angle.sin();
            }
        }
        sim.kill_part(i);
        return 1;
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
            if pmap_type(r) != pt::DMND && sim.rng.chance(1, 3) {
                if pmap_type(r) == pt::SING && sim.parts[id].life > 10 {
                    if sim.parts[i].life + sim.parts[id].life <= 255 {
                        sim.parts[i].life += sim.parts[id].life;
                    }
                } else if sim.parts[i].life + 3 > 255 {
                    if pmap_type(r) != pt::SING && sim.rng.chance(1, 1000) {
                        if let Some(np) = sim.create_part(nx, ny, pt::SING) {
                            sim.parts[np].life = sim.rng.between(60, 109);
                        }
                    }
                    continue;
                } else {
                    sim.parts[i].life += 3;
                    sim.parts[i].tmp += 1;
                }
                let t = sim.parts[id].temp + sim.parts[i].temp;
                sim.parts[i].temp = restrict_flt(t, MIN_TEMP, MAX_TEMP);
                sim.kill_part(id);
            }
        }
    }
    0
}

fn sing_create(sim: &mut Sim, i: usize, _x: i32, _y: i32, _t: i32, _v: i32) {
    sim.parts[i].life = sim.rng.between(60, 109);
}

// --- BREC ----------------------------------------------------------------
fn brec_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.parts[i].life != 0 {
        let p = pv(sim, x, y);
        if p > 10.0 {
            if sim.parts[i].temp > 9000.0 && p > 30.0 && sim.rng.chance(1, 200) {
                sim.part_change_type(i, x, y, pt::EXOT as i16);
                sim.parts[i].life = 1000;
            }
            sim.parts[i].temp += p / 8.0;
        }
    }
    0
}

// --- ELEC ----------------------------------------------------------------
fn elec_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    for rx in -2..=2 {
        for ry in -2..=2 {
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
            match rt {
                pt::GLAS => {
                    for rrx in -1..=1 {
                        for rry in -1..=1 {
                            let tx = nx + rrx;
                            let ty = ny + rry;
                            if cell_free(sim, tx, ty) {
                                if let Some(nb) = sim.create_part(tx, ty, pt::EMBR) {
                                    sim.parts[nb].tmp = 0;
                                    sim.parts[nb].life = 50;
                                    sim.parts[nb].temp = sim.parts[i].temp * 0.8;
                                    sim.parts[nb].vx = sim.rng.between(-10, 10) as f32;
                                    sim.parts[nb].vy = sim.rng.between(-10, 10) as f32;
                                }
                            }
                        }
                    }
                    sim.kill_part(i);
                    return 1;
                }
                pt::LCRY => {
                    sim.parts[id].tmp2 = sim.rng.between(5, 9);
                }
                pt::WATR | pt::DSTW | pt::SLTW | pt::CBNW => {
                    let to = if sim.rng.chance(1, 3) { pt::O2 } else { pt::H2 };
                    if sim.part_change_type(id, nx, ny, to as i16) {
                        let _ = id;
                    }
                    sim.kill_part(i);
                    return 1;
                }
                pt::PROT | pt::NEUT => {
                    if rt == pt::PROT && (sim.parts[id].tmp2 & 0x1) != 0 {
                        continue;
                    }
                    sim.part_change_type(id, nx, ny, pt::H2 as i16);
                    sim.parts[id].life = 0;
                    sim.parts[id].ctype = 0;
                    sim.kill_part(i);
                }
                pt::DEUT => {
                    if sim.parts[id].life < 6000 {
                        sim.parts[id].life += 1;
                    }
                    sim.parts[id].temp = 0.0;
                    sim.kill_part(i);
                    return 1;
                }
                pt::EXOT => {
                    sim.parts[id].tmp2 += 5;
                    sim.parts[id].life = 1000;
                }
                pt::RSST => {
                    if rx == 0 && ry == 0 {
                        sim.kill_part(id);
                        sim.kill_part(i);
                        return 1;
                    }
                }
                _ => {
                    if (element(rt).properties & props::PROP_CONDUCTS) != 0
                        && (rt != pt::NBLE || sim.parts[i].temp < 2273.15)
                    {
                        if cell_free(sim, nx, ny) {
                            sim.create_part(nx, ny, pt::SPRK);
                        }
                        sim.kill_part(i);
                        return 1;
                    }
                }
            }
        }
    }
    0
}

fn elec_create(sim: &mut Sim, i: usize, _x: i32, _y: i32, _t: i32, _v: i32) {
    let a = sim.rng.between(0, 359) as f32 * std::f32::consts::PI / 180.0;
    sim.parts[i].life = 680;
    sim.parts[i].vx = 2.0 * a.cos();
    sim.parts[i].vy = 2.0 * a.sin();
}

// --- ACEL / DCEL ---------------------------------------------------------
fn acel_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let multiplier = if sim.parts[i].life != 0 {
        let change = sim.parts[i].life.clamp(0, 1000);
        1.0 + change as f32 / 100.0
    } else {
        1.1
    };
    sim.parts[i].tmp = 0;
    for rx in -1..=1 {
        for ry in -1..=1 {
            if (rx == 0) == (ry == 0) {
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
            let p = element(rt).properties;
            if (p & (props::TYPE_PART | props::TYPE_LIQUID | props::TYPE_GAS | props::TYPE_ENERGY)) != 0 {
                let mut vx = sim.parts[id].vx * multiplier;
                let mut vy = sim.parts[id].vy * multiplier;
                let mv = vx.abs().max(vy.abs());
                if mv > 1.0e4 {
                    vx *= 1.0e4 / mv;
                    vy *= 1.0e4 / mv;
                }
                sim.parts[id].vx = vx;
                sim.parts[id].vy = vy;
                sim.parts[i].tmp = 1;
            }
        }
    }
    0
}

fn dcel_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let multiplier = if sim.parts[i].life != 0 {
        1.0 - (sim.parts[i].life.clamp(0, 100) as f32 / 100.0)
    } else {
        1.0 / 1.1
    };
    sim.parts[i].tmp = 0;
    for rx in -1..=1 {
        for ry in -1..=1 {
            if (rx == 0) == (ry == 0) {
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
            let p = element(rt).properties;
            if (p & (props::TYPE_PART | props::TYPE_LIQUID | props::TYPE_GAS | props::TYPE_ENERGY)) != 0 {
                sim.parts[id].vx *= multiplier;
                sim.parts[id].vy *= multiplier;
                sim.parts[i].tmp = 1;
            }
        }
    }
    0
}

// --- BANG ----------------------------------------------------------------
fn bang_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.parts[i].tmp == 0 {
        if sim.parts[i].temp >= 673.0 {
            sim.parts[i].tmp = 1;
        } else {
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
                    let rr = sim.pmap[sim.idx(nx, ny)];
                    if rr == 0 {
                        continue;
                    }
                    if matches!(pmap_type(rr), pt::FIRE | pt::PLSM | pt::SPRK | pt::LIGH) {
                        sim.parts[i].tmp = 1;
                    }
                }
            }
        }
    } else if sim.parts[i].tmp == 1 {
        let cell = sim.idx(x, y);
        if sim.pmap[cell] != 0 && pmap_id(sim.pmap[cell]) as usize == i {
            flood_tmp(sim, x, y, 2);
        }
        sim.parts[i].tmp = 2;
    } else if sim.parts[i].tmp == 2 {
        sim.parts[i].tmp = 3;
    } else {
        let otemp = sim.parts[i].temp - 273.15;
        if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
            let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
            sim.pv[vc] += 0.5;
        }
        sim.parts[i].tmp = 0;
        if sim.rng.chance(1, 3) {
            if sim.rng.chance(1, 2) {
                sim.part_change_type(i, x, y, pt::FIRE as i16);
            } else {
                sim.part_change_type(i, x, y, pt::SMKE as i16);
                sim.parts[i].life = sim.rng.between(500, 549);
            }
            sim.parts[i].temp = restrict_flt(MAX_TEMP / 4.0 + otemp, MIN_TEMP, MAX_TEMP);
        } else if sim.rng.chance(1, 15) {
            sim.part_change_type(i, x, y, pt::EMBR as i16);
            sim.parts[i].tmp = 0;
            sim.parts[i].life = 50;
            sim.parts[i].temp = restrict_flt(MAX_TEMP / 3.0 + otemp, MIN_TEMP, MAX_TEMP);
            sim.parts[i].vx = sim.rng.between(-10, 10) as f32;
            sim.parts[i].vy = sim.rng.between(-10, 10) as f32;
        } else {
            sim.kill_part(i);
        }
        return 1;
    }
    0
}

/// Reference `flood_prop` restricted to `FIELD_TMP` propagation.
fn flood_tmp(sim: &mut Sim, x: i32, y: i32, value: i32) {
    let mut stack = vec![(x, y)];
    let mut seen = std::collections::HashSet::new();
    while let Some((cx, cy)) = stack.pop() {
        if !seen.insert((cx, cy)) {
            continue;
        }
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let nx = cx + dx;
            let ny = cy + dy;
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let r = sim.pmap[sim.idx(nx, ny)];
            if r == 0 {
                continue;
            }
            let id = pmap_id(r) as usize;
            if pmap_type(r) == pt::BANG && sim.parts[id].tmp == 0 {
                sim.parts[id].tmp = value;
                stack.push((nx, ny));
            }
        }
    }
}

// --- IGNT ----------------------------------------------------------------
fn ignt_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.parts[i].tmp == 0 {
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
                if matches!(rt, pt::FIRE | pt::PLSM | pt::SPRK | pt::LIGH)
                    || (rt == pt::IGNT && sim.parts[id].life == 1)
                {
                    sim.parts[i].tmp = 1;
                }
            }
        }
    } else if sim.parts[i].life > 0 {
        let nx = x + sim.rng.between(-1, 1);
        let ny = y + sim.rng.between(-1, 1);
        if sim.rng.chance(2, 3) {
            if let Some(nb) = create_near(sim, nx, ny, pt::EMBR) {
                sim.parts[nb].tmp = 0;
                sim.parts[nb].life = 30;
                sim.parts[nb].vx = sim.rng.between(-10, 10) as f32;
                sim.parts[nb].vy = sim.rng.between(-10, 10) as f32;
                let t = sim.parts[i].temp - 273.15 + 400.0;
                sim.parts[nb].temp = restrict_flt(t, MIN_TEMP, MAX_TEMP);
            }
        } else {
            create_near(sim, nx, ny, pt::FIRE);
        }
        sim.parts[i].life -= 1;
    }
    0
}

// --- BOYL ----------------------------------------------------------------
fn boyl_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let limit = sim.parts[i].temp / 100.0;
    if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
        let cx = x / CELL as i32;
        let cy = y / CELL as i32;
        for (dx, dy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, -1), (1, -1), (-1, 1)] {
            if sim.in_cell_bounds(cx + dx, cy + dy) {
                let vc = sim.cell_idx(cx + dx, cy + dy);
                if sim.pv[vc] < limit {
                    sim.pv[vc] += 0.001 * (limit - sim.pv[vc]);
                }
            }
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
            let rt = pmap_type(r);
            if rt == pt::WATR {
                if sim.rng.chance(1, 30) {
                    sim.part_change_type(id, nx, ny, pt::FOG as i16);
                }
            } else if rt == pt::O2 && sim.rng.chance(1, 9) {
                sim.kill_part(id);
                sim.part_change_type(i, x, y, pt::WATR as i16);
                if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
                    let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
                    sim.pv[vc] += 4.0;
                }
                return 1;
            }
        }
    }
    0
}

// --- TTAN ----------------------------------------------------------------
fn ttan_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, nt: i32) -> i32 {
    let mut ttan = 0;
    if nt <= 2 {
        ttan = 2;
    } else if sim.parts[i].tmp != 0 {
        ttan = 2;
    } else if nt <= 6 {
        for rx in -1..=1 {
            for ry in -1..=1 {
                if (rx == 0) == (ry == 0) {
                    continue;
                }
                if sim.type_at(x + rx, y + ry) == pt::TTAN {
                    ttan += 1;
                }
            }
        }
    }
    if ttan >= 2 {
        let cx = x / CELL as i32;
        let cy = y / CELL as i32;
        if sim.in_cell_bounds(cx, cy) {
            let vc = sim.cell_idx(cx, cy);
            sim.bmap_blockair[vc] = 1;
            sim.bmap_blockairh[vc] = 0x8;
        }
    }
    0
}

// --- EMBR ----------------------------------------------------------------
fn embr_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
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
            let p = element(pmap_type(r)).properties;
            if (p & (props::TYPE_SOLID | props::TYPE_PART | props::TYPE_LIQUID)) != 0
                && (p & props::PROP_SPARKSETTLE) == 0
            {
                sim.kill_part(i);
                return 1;
            }
        }
    }
    0
}

// --- NWHL ----------------------------------------------------------------
fn nwhl_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let cx = x / CELL as i32;
    let cy = y / CELL as i32;
    if !sim.in_cell_bounds(cx, cy) {
        return 0;
    }
    let vc = sim.cell_idx(cx, cy);
    let mass = if sim.parts[i].tmp != 0 {
        restrict_flt(0.001 * sim.parts[i].tmp as f32, 0.1, 51.2)
    } else {
        0.1
    };
    sim.grav_mass[vc] -= mass;
    0
}

// --- MERC ----------------------------------------------------------------
fn merc_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    const ABSORB_SCALE: i32 = 10000;
    sim.parts[i].temp = restrict_flt(sim.parts[i].temp, MIN_TEMP, MAX_TEMP);
    let temp = sim.parts[i].temp;
    let mut maxtmp = (ABSORB_SCALE as f32 / (temp + 1.0)) as i32 - 1;
    let it1 = ((temp as i32) + 1).max(1);
    if sim.rng.chance(ABSORB_SCALE % it1, it1 as u32) {
        maxtmp += 1;
    }
    if sim.parts[i].tmp < 0 {
        sim.parts[i].tmp = 0;
    }
    if sim.parts[i].tmp > ABSORB_SCALE {
        sim.parts[i].tmp = ABSORB_SCALE;
    }
    if sim.parts[i].tmp < maxtmp {
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
                if r == 0 || sim.parts[i].tmp >= maxtmp {
                    continue;
                }
                let id = pmap_id(r) as usize;
                if pmap_type(r) == pt::MERC && sim.rng.chance(1, 3) {
                    if sim.parts[i].tmp + sim.parts[id].tmp + 1 <= maxtmp {
                        sim.parts[i].tmp += sim.parts[id].tmp + 1;
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
                if sim.parts[i].tmp <= maxtmp {
                    continue;
                }
                let nx = x + rx;
                let ny = y + ry;
                if !cell_free(sim, nx, ny) || sim.parts[i].tmp < 1 {
                    continue;
                }
                let temp = sim.parts[i].temp;
                let dcolour = sim.parts[i].dcolour;
                if let Some(np) = sim.create_part(nx, ny, pt::MERC) {
                    sim.parts[i].tmp -= 1;
                    sim.parts[np].temp = temp;
                    sim.parts[np].tmp = 0;
                    sim.parts[np].dcolour = dcolour;
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
        if pmap_type(r) == pt::MERC && sim.parts[i].tmp > sim.parts[id].tmp && sim.parts[i].tmp > 0 {
            let diff = sim.parts[i].tmp - sim.parts[id].tmp;
            if diff == 1 {
                sim.parts[id].tmp += 1;
                sim.parts[i].tmp -= 1;
            } else if diff > 0 {
                sim.parts[id].tmp += diff / 2;
                sim.parts[i].tmp -= diff / 2;
            }
        }
    }
    0
}

// --- CLST ----------------------------------------------------------------
fn clst_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
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
            if rt == pt::WATR {
                if sim.rng.chance(1, 1500) {
                    sim.part_change_type(i, x, y, pt::PSTS as i16);
                    sim.kill_part(id);
                }
            } else if rt == pt::NITR {
                sim.part_change_type(i, x, y, pt::BANG as i16);
                sim.part_change_type(id, nx, ny, pt::BANG as i16);
            } else if rt == pt::CLST {
                let temp = sim.parts[i].temp;
                let cxy = if temp < 195.0 {
                    0.05
                } else if temp < 295.0 {
                    0.015
                } else if temp < 350.0 {
                    0.01
                } else {
                    0.005
                };
                sim.parts[i].vx += cxy * rx as f32;
                sim.parts[i].vy += cxy * ry as f32;
            }
        }
    }
    0
}

fn clst_create(sim: &mut Sim, i: usize, _x: i32, _y: i32, _t: i32, _v: i32) {
    sim.parts[i].tmp = sim.rng.between(0, 6);
}

// --- WIRE ----------------------------------------------------------------
fn wire_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let mut count = 0;
    sim.parts[i].ctype = 0;
    if sim.parts[i].tmp == 1 {
        sim.parts[i].ctype = 2;
    } else if sim.parts[i].tmp == 2 {
        sim.parts[i].ctype = 0;
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
            let rt = pmap_type(r);
            if rt == pt::SPRK && sim.parts[id].life == 3 && sim.parts[id].ctype == pt::PSCN as i32 {
                sim.parts[i].ctype = 1;
                return 0;
            } else if rt == pt::NSCN && sim.parts[i].tmp == 1 {
                if cell_free(sim, nx, ny) {
                    sim.create_part(nx, ny, pt::SPRK);
                }
            } else if rt == pt::WIRE && sim.parts[id].tmp == 1 && sim.parts[i].tmp == 0 {
                count += 1;
            }
        }
    }
    if count == 1 || count == 2 {
        sim.parts[i].ctype = 1;
    }
    0
}

// --- GBMB ----------------------------------------------------------------
fn gbmb_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.parts[i].life <= 0 {
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
                if !matches!(pmap_type(r), pt::BOMB | pt::GBMB | pt::CLNE | pt::PCLN | pt::DMND) {
                    sim.parts[i].life = 60;
                }
            }
        }
    }
    let cx = x / CELL as i32;
    let cy = y / CELL as i32;
    if sim.in_cell_bounds(cx, cy) {
        let vc = sim.cell_idx(cx, cy);
        if sim.parts[i].life > 20 {
            sim.grav_mass[vc] = 20.0;
        } else if sim.parts[i].life >= 1 {
            sim.grav_mass[vc] = -80.0;
        }
    }
    0
}

// --- FRAY ----------------------------------------------------------------
fn fray_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let curlen = if sim.parts[i].tmp > 0 { sim.parts[i].tmp } else { 10 };
    let temp = sim.parts[i].temp;
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
            if pmap_type(r) == pt::SPRK {
                let nxi = -rx;
                let nyi = -ry;
                let mut nxx = 0;
                let mut nyy = 0;
                let mut len = 0;
                while len <= curlen {
                    let tx = x + nxi + nxx;
                    let ty = y + nyi + nyy;
                    if !sim.in_bounds(tx, ty) {
                        break;
                    }
                    let mut rr = sim.pmap[sim.idx(tx, ty)];
                    if rr == 0 {
                        rr = sim.photons[sim.idx(tx, ty)];
                    }
                    if rr != 0 {
                        let p = element(pmap_type(rr)).properties;
                        if (p & props::TYPE_SOLID) == 0 {
                            let id = pmap_id(rr) as usize;
                            let coef = (temp - 273.15) / 10.0;
                            let vx = restrict_flt(sim.parts[id].vx + nxi as f32 * coef, -1.0e4, 1.0e4);
                            let vy = restrict_flt(sim.parts[id].vy + nyi as f32 * coef, -1.0e4, 1.0e4);
                            sim.parts[id].vx = vx;
                            sim.parts[id].vy = vy;
                        }
                    }
                    nxx += nxi;
                    nyy += nyi;
                    len += 1;
                }
            }
        }
    }
    0
}

// --- RPEL ----------------------------------------------------------------
fn rpel_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let temp = sim.parts[i].temp;
    let ctype = sim.parts[i].ctype;
    for _ in 0..11 {
        let rx = sim.rng.between(-10, 10);
        let ry = sim.rng.between(-10, 10);
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
        let p = element(rt).properties;
        if (p & props::TYPE_SOLID) == 0 && (ctype == 0 || ctype == rt as i32) {
            let d = (temp - 273.15) / 10.0;
            let vx = restrict_flt(sim.parts[id].vx + rx.signum() as f32 * d, -1.0e4, 1.0e4);
            let vy = restrict_flt(sim.parts[id].vy + ry.signum() as f32 * d, -1.0e4, 1.0e4);
            sim.parts[id].vx = vx;
            sim.parts[id].vy = vy;
        }
    }
    0
}

// --- DTEC ----------------------------------------------------------------
fn dtec_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let mut rd = sim.parts[i].tmp2;
    if rd > 25 {
        rd = 25;
        sim.parts[i].tmp2 = 25;
    }
    if sim.parts[i].life != 0 {
        sim.parts[i].life = 0;
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
                if (element(rt).properties & props::PROP_CONDUCTS) != 0
                    && !matches!(rt, pt::WATR | pt::SLTW | pt::NTCT | pt::PTCT | pt::INWR)
                    && sim.parts[id].life == 0
                {
                    sim.parts[id].life = 4;
                    sim.parts[id].ctype = rt as i32;
                    sim.part_change_type(id, nx, ny, pt::SPRK as i16);
                }
            }
        }
    }
    // FILT colour propagation is approximated by writing the observed ctype.
    let ctype = sim.parts[i].ctype;
    let mut photon_wl: Option<i32> = None;
    for rx in -rd..=rd {
        for ry in -rd..=rd {
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
            if rt as i32 == ctype {
                sim.parts[i].life = 1;
            }
            if matches!(rt, pt::PHOT | pt::BIZR | pt::BIZRG | pt::BIZRS) || (rt == pt::BRAY && sim.parts[id].tmp != 2) {
                photon_wl = Some(sim.parts[id].ctype);
            }
        }
    }
    if let Some(wl) = photon_wl {
        for rx in -1..=1 {
            for ry in -1..=1 {
                if rx == 0 && ry == 0 {
                    continue;
                }
                let mut nx = x + rx;
                let mut ny = y + ry;
                if !sim.in_bounds(nx, ny) {
                    continue;
                }
                let mut r = sim.pmap[sim.idx(nx, ny)];
                while r != 0 && pmap_type(r) == pt::FILT {
                    let id = pmap_id(r) as usize;
                    sim.parts[id].ctype = wl;
                    nx += rx;
                    ny += ry;
                    if !sim.in_bounds(nx, ny) {
                        break;
                    }
                    r = sim.pmap[sim.idx(nx, ny)];
                }
            }
        }
    }
    0
}

// --- DMG -----------------------------------------------------------------
fn dmg_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let rad = 25;
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
            if matches!(rt, pt::DMG | pt::EMBR | pt::DMND | pt::CLNE | pt::PCLN | pt::BCLN) {
                continue;
            }
            sim.kill_part(i);
            for nxj in -rad..=rad {
                for nxi in -rad..=rad {
                    if nxi == 0 && nxj == 0 {
                        continue;
                    }
                    let tx = x + nxi;
                    let ty = y + nxj;
                    if !sim.in_bounds(tx, ty) {
                        continue;
                    }
                    let dist = ((nxi * nxi + nxj * nxj) as f32).sqrt();
                    if dist > rad as f32 {
                        continue;
                    }
                    let rr = sim.pmap[sim.idx(tx, ty)];
                    if rr == 0 {
                        continue;
                    }
                    let rid = pmap_id(rr) as usize;
                    let angle = (nxj as f32).atan2(nxi as f32);
                    let fx = angle.cos() * 7.0;
                    let fy = angle.sin() * 7.0;
                    sim.parts[rid].vx += fx;
                    sim.parts[rid].vy += fy;
                    if sim.in_cell_bounds(tx / CELL as i32, ty / CELL as i32) {
                        let vc = sim.cell_idx(tx / CELL as i32, ty / CELL as i32);
                        sim.vx[vc] += fx;
                        sim.vy[vc] += fy;
                        sim.pv[vc] += 1.0;
                    }
                    let t = pmap_type(rr);
                    let def = element(t);
                    if def.high_pressure_transition >= 0 && def.high_pressure_transition < crate::elements_generated::PT_NUM as i32 {
                        sim.part_change_type(rid, tx, ty, def.high_pressure_transition as i16);
                    } else if t == pt::BMTL {
                        sim.part_change_type(rid, tx, ty, pt::BRMT as i16);
                    } else if t == pt::GLAS {
                        sim.part_change_type(rid, tx, ty, pt::BGLA as i16);
                    } else if t == pt::COAL {
                        sim.part_change_type(rid, tx, ty, pt::BCOL as i16);
                    } else if t == pt::QRTZ {
                        sim.part_change_type(rid, tx, ty, pt::PQRT as i16);
                    } else if t == pt::TUNG {
                        sim.part_change_type(rid, tx, ty, pt::BRMT as i16);
                        sim.parts[rid].ctype = pt::TUNG as i32;
                    } else if t == pt::WOOD {
                        sim.part_change_type(rid, tx, ty, pt::SAWD as i16);
                    }
                }
            }
            return 1;
        }
    }
    0
}

// --- TSNS ----------------------------------------------------------------
fn tsns_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let mut rd = sim.parts[i].tmp2;
    if rd > 25 {
        rd = 25;
        sim.parts[i].tmp2 = 25;
    }
    if sim.parts[i].life != 0 {
        sim.parts[i].life = 0;
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
                let pavg = parts_avg(sim, i, id, pt::INSL);
                if pavg != pt::INSL && pavg != pt::RSSS
                    && (element(rt).properties & props::PROP_CONDUCTS) != 0
                    && !matches!(rt, pt::WATR | pt::SLTW | pt::NTCT | pt::PTCT | pt::INWR)
                    && sim.parts[id].life == 0
                {
                    sim.parts[id].life = 4;
                    sim.parts[id].ctype = rt as i32;
                    sim.part_change_type(id, nx, ny, pt::SPRK as i16);
                }
            }
        }
    }
    let mode = sim.parts[i].tmp;
    let mut set_filt = false;
    let mut photon_wl = 0;
    for rx in -rd..=rd {
        for ry in -rd..=rd {
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
            let tr = sim.parts[id].temp;
            let ti = sim.parts[i].temp;
            if mode == 0 && rt != pt::TSNS && rt != pt::METL && tr > ti {
                sim.parts[i].life = 1;
            }
            if mode == 2 && rt != pt::TSNS && rt != pt::METL && tr < ti {
                sim.parts[i].life = 1;
            }
            if mode == 1 && rt != pt::TSNS && rt != pt::FILT {
                set_filt = true;
                photon_wl = tr as i32;
            }
        }
    }
    if set_filt {
        for rx in -1..=1 {
            for ry in -1..=1 {
                if rx == 0 && ry == 0 {
                    continue;
                }
                let mut nx = x + rx;
                let mut ny = y + ry;
                if !sim.in_bounds(nx, ny) {
                    continue;
                }
                let mut r = sim.pmap[sim.idx(nx, ny)];
                while r != 0 && pmap_type(r) == pt::FILT {
                    let id = pmap_id(r) as usize;
                    sim.parts[id].ctype = 0x10000000 + photon_wl;
                    nx += rx;
                    ny += ry;
                    if !sim.in_bounds(nx, ny) {
                        break;
                    }
                    r = sim.pmap[sim.idx(nx, ny)];
                }
            }
        }
    }
    0
}

// --- VIBR ----------------------------------------------------------------
const ORBIT_RX: [i32; 8] = [-1, 0, 1, 1, 1, 0, -1, -1];
const ORBIT_RY: [i32; 8] = [-1, -1, -1, 0, 1, 1, 1, 0];

fn vibr_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let mut rndstore: u32 = 0;
    if sim.parts[i].life == 0 {
        if sim.parts[i].temp > 274.65 {
            sim.parts[i].tmp += 1;
            sim.parts[i].temp -= 3.0;
        } else if sim.parts[i].temp < 271.65 {
            sim.parts[i].tmp -= 1;
            sim.parts[i].temp += 3.0;
        }
        let p = pv(sim, x, y);
        if p > 2.5 {
            sim.parts[i].tmp += 7;
            if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
                let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
                sim.pv[vc] -= 1.0;
            }
        } else if p < -2.5 {
            sim.parts[i].tmp -= 2;
            if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
                let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
                sim.pv[vc] += 1.0;
            }
        }
        if sim.parts[i].tmp > 1000 {
            sim.parts[i].life = 750;
        }
    } else {
        rndstore = sim.rng.gen();
        if sim.parts[i].life < 300 {
            let rx = ORBIT_RX[(rndstore & 7) as usize];
            let ry = ORBIT_RY[(rndstore & 7) as usize];
            rndstore >>= 3;
            let nx = x + rx;
            let ny = y + ry;
            if sim.in_bounds(nx, ny) {
                let r = sim.pmap[sim.idx(nx, ny)];
                if r != 0 {
                    let id = pmap_id(r) as usize;
                    let rt = pmap_type(r);
                    if rt != pt::BREC
                        && (element(rt).properties & props::PROP_CONDUCTS) != 0
                        && sim.parts[id].life == 0
                    {
                        sim.parts[id].life = 4;
                        sim.parts[id].ctype = rt as i32;
                        sim.part_change_type(id, nx, ny, pt::SPRK as i16);
                    }
                }
            }
        }
        if sim.parts[i].life < 500 {
            let rx = (rndstore % 7) as i32 - 3;
            let ry = ((rndstore >> 3) % 7) as i32 - 3;
            let nx = x + rx;
            let ny = y + ry;
            if sim.in_bounds(nx, ny) {
                let r = sim.pmap[sim.idx(nx, ny)];
                if r != 0 {
                    let id = pmap_id(r) as usize;
                    let rt = pmap_type(r);
                    if rt != pt::VIBR && rt != pt::BVBR && !sim.is_heat_insulator(id) {
                        let t = sim.parts[id].temp + sim.parts[i].tmp as f32 * 3.0;
                        sim.parts[id].temp = restrict_flt(t, MIN_TEMP, MAX_TEMP);
                        sim.parts[i].tmp = 0;
                    }
                }
            }
        }
        if sim.parts[i].life == 1 {
            if sim.parts[i].tmp2 == 0 {
                rndstore = sim.rng.gen();
                let idx1 = create_near(sim, x + ORBIT_RX[(rndstore & 7) as usize], y + ORBIT_RY[(rndstore & 7) as usize], pt::ELEC);
                if let Some(idx) = idx1 {
                    sim.parts[idx].temp = 7000.0;
                }
                let idx2 = create_near(sim, x + ORBIT_RX[((rndstore >> 3) & 7) as usize], y + ORBIT_RY[((rndstore >> 3) & 7) as usize], pt::PHOT);
                if let Some(idx) = idx2 {
                    sim.parts[idx].temp = 7000.0;
                }
                let idx3 = create_near(sim, x + ORBIT_RX[((rndstore >> 6) & 7) as usize], y + ORBIT_RY[((rndstore >> 6) & 7) as usize], pt::BREC);
                if let Some(idx) = idx3 {
                    sim.parts[idx].temp = 7000.0;
                }
                sim.part_change_type(i, x, y, pt::EXOT as i16);
                sim.parts[i].tmp2 = ((rndstore >> 9) % 1000) as i32;
                sim.parts[i].temp = 9000.0;
                if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
                    let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
                    sim.pv[vc] += 50.0;
                }
                return 1;
            } else {
                sim.parts[i].tmp2 = 0;
                sim.parts[i].temp = 273.15;
                sim.parts[i].tmp = 0;
            }
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
            let rt = pmap_type(r);
            if sim.parts[i].life != 0 {
                if rt == pt::VIBR || rt == pt::BVBR {
                    if sim.parts[id].life == 0 {
                        sim.parts[id].tmp += 45;
                    } else if sim.parts[i].tmp2 != 0 && sim.parts[i].life > 75 && sim.rng.chance(1, 2) {
                        sim.parts[id].tmp2 = 1;
                        sim.parts[i].tmp = 0;
                    }
                } else if rt == pt::CFLM {
                    sim.parts[i].tmp2 = 1;
                    sim.parts[i].tmp = 0;
                }
            } else if rt == pt::EXOT && sim.rng.chance(1, 25) {
                sim.part_change_type(i, x, y, pt::EXOT as i16);
                return 1;
            }
            if sim.parts[i].t != pt::BVBR && rt == pt::ANAR {
                sim.part_change_type(i, x, y, pt::BVBR as i16);
                if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
                    let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
                    sim.pv[vc] -= 1.0;
                }
            }
        }
    }
    for trade in 0..9 {
        if trade % 2 == 0 {
            rndstore = sim.rng.gen();
        }
        let rx = (rndstore % 7) as i32 - 3;
        rndstore >>= 3;
        let ry = (rndstore % 7) as i32 - 3;
        rndstore >>= 3;
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
        if rt != pt::VIBR && rt != pt::BVBR {
            continue;
        }
        if sim.parts[i].tmp > sim.parts[id].tmp {
            let transfer = sim.parts[i].tmp - sim.parts[id].tmp;
            sim.parts[id].tmp += transfer / 2;
            sim.parts[i].tmp -= transfer / 2;
            break;
        }
    }
    sim.parts[i].tmp = sim.parts[i].tmp.clamp(0, 1 << 15);
    0
}

// --- GOLD ----------------------------------------------------------------
fn gold_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    for _ in 0..8 {
        let rnd = sim.rng.gen();
        let rx = (rnd % 9) as i32 - 4;
        let ry = ((rnd >> 4) % 9) as i32 - 4;
        if (rx == 0) == (ry == 0) {
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
        if pmap_type(r) == pt::BMTL && sim.parts[id].tmp != 0 {
            sim.parts[id].tmp = 0;
            sim.part_change_type(id, nx, ny, pt::IRON as i16);
        }
    }
    if sim.parts[i].life == 0 {
        for (rx, ry) in [(-4, 0), (4, 0), (0, -4), (0, 4)] {
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
            if pmap_type(r) == pt::SPRK && sim.parts[id].life != 0 && sim.parts[id].life < 4 {
                sim.part_change_type(i, x, y, pt::SPRK as i16);
                sim.parts[i].life = 4;
                sim.parts[i].ctype = pt::GOLD as i32;
            }
        }
    }
    let cell = sim.idx(x, y);
    if pmap_type(sim.photons[cell]) == pt::NEUT && sim.rng.chance(1, 7) {
        let id = pmap_id(sim.photons[cell]) as usize;
        sim.kill_part(id);
    }
    0
}

// --- TUNG ----------------------------------------------------------------
fn tung_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let melting_point = element(pt::TUNG).high_temperature;
    let mut splode = false;
    if sim.parts[i].temp > 2400.0 {
        for rx in -1..=1 {
            for ry in -1..=1 {
                if rx == 0 && ry == 0 {
                    continue;
                }
                if sim.type_at(x + rx, y + ry) == pt::O2 {
                    splode = true;
                }
            }
        }
    }
    if (sim.parts[i].temp > melting_point && sim.rng.chance(1, 20)) || splode {
        if sim.rng.chance(1, 50) {
            if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
                let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
                sim.pv[vc] += 50.0;
            }
        } else if sim.rng.chance(1, 100) {
            sim.part_change_type(i, x, y, pt::FIRE as i16);
            sim.parts[i].life = sim.rng.between(0, 499);
            return 1;
        } else {
            sim.part_change_type(i, x, y, pt::LAVA as i16);
            sim.parts[i].ctype = pt::TUNG as i32;
            return 1;
        }
        if splode {
            let t = melting_point + sim.rng.between(200, 799) as f32;
            sim.parts[i].temp = restrict_flt(t, MIN_TEMP, MAX_TEMP);
        }
        sim.parts[i].vx += sim.rng.between(-50, 50) as f32;
        sim.parts[i].vy += sim.rng.between(-50, 50) as f32;
        return 1;
    }
    let press = (pv(sim, x, y) * 64.0) as i32;
    let diff = press - sim.parts[i].tmp3;
    if diff > 32 || diff < -32 {
        sim.part_change_type(i, x, y, pt::BRMT as i16);
        sim.parts[i].ctype = pt::TUNG as i32;
        return 1;
    }
    sim.parts[i].tmp3 = press;
    0
}

fn tung_create(sim: &mut Sim, i: usize, x: i32, y: i32, _t: i32, _v: i32) {
    sim.parts[i].tmp3 = (pv(sim, x, y) * 64.0) as i32;
}

// --- PSNS ----------------------------------------------------------------
fn psns_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let ti = sim.parts[i].temp - 273.15;
    let p = pv(sim, x, y);
    if (sim.parts[i].tmp == 0 && p > ti) || (sim.parts[i].tmp == 2 && p < ti) {
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
                if pavg != pt::INSL
                    && pavg != pt::RSSS
                    && (element(rt).properties & props::PROP_CONDUCTS) != 0
                    && !matches!(rt, pt::WATR | pt::SLTW | pt::NTCT | pt::PTCT | pt::INWR)
                    && sim.parts[id].life == 0
                {
                    sim.parts[id].life = 4;
                    sim.parts[id].ctype = rt as i32;
                    sim.part_change_type(id, nx, ny, pt::SPRK as i16);
                }
            }
        }
    }
    if sim.parts[i].tmp == 1 {
        let photon_wl = p;
        for rx in -1..=1 {
            for ry in -1..=1 {
                if rx == 0 && ry == 0 {
                    continue;
                }
                let mut nx = x + rx;
                let mut ny = y + ry;
                if !sim.in_bounds(nx, ny) {
                    continue;
                }
                let mut r = sim.pmap[sim.idx(nx, ny)];
                while r != 0 && pmap_type(r) == pt::FILT {
                    let id = pmap_id(r) as usize;
                    sim.parts[id].ctype = 0x10000000 + (photon_wl.round() as i32 - MIN_PRESSURE as i32);
                    nx += rx;
                    ny += ry;
                    if !sim.in_bounds(nx, ny) {
                        break;
                    }
                    r = sim.pmap[sim.idx(nx, ny)];
                }
            }
        }
    }
    0
}

// --- PROT ----------------------------------------------------------------
fn prot_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
        let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
        sim.pv[vc] -= 0.003;
    }
    // The full proton reactions (spark removal, DEUT implosion, LCRY, EXOT,
    // WIFI, RSSS, SEED mutation, explosion triggering) are not ported yet;
    // the core decay by collision energy is implemented below.
    if sim.parts[i].tmp != 0 {
        let over = sim.parts[i].tmp;
        let (element_t, temp_scale): (u16, f32) = if over > 500000 {
            (pt::SING, 100.0)
        } else if over > 700 {
            (pt::PLUT, 100.0)
        } else if over > 420 {
            (pt::URAN, 100.0)
        } else if over > 310 {
            (pt::POLO, 100.0)
        } else if over > 250 {
            (pt::PLSM, 100.0)
        } else if over > 100 {
            (pt::O2, 100.0)
        } else if over > 50 {
            (pt::CO2, 100.0)
        } else {
            (pt::NBLE, 100.0)
        };
        let nx = x + sim.rng.between(-1, 1);
        let ny = y + sim.rng.between(-1, 1);
        if let Some(new_id) = create_near(sim, nx, ny, element_t) {
            sim.parts[new_id].temp = restrict_flt(over as f32 * temp_scale, MIN_TEMP, MAX_TEMP);
        }
        sim.kill_part(i);
        return 1;
    }
    0
}

fn prot_create(sim: &mut Sim, i: usize, _x: i32, _y: i32, _t: i32, _v: i32) {
    let a = sim.rng.between(0, 35) as f32 * 0.17453;
    sim.parts[i].life = 680;
    sim.parts[i].vx = 2.0 * a.cos();
    sim.parts[i].vy = 2.0 * a.sin();
}

// --- VIRS ----------------------------------------------------------------
fn virs_update(sim: &mut Sim, i: usize, x: i32, y: i32, surround_space: i32, _nt: i32) -> i32 {
    let mut rnd = sim.rng.gen();
    if sim.parts[i].tmp3 != 0 {
        if rnd & 1 == 0 {
            sim.parts[i].tmp3 -= 1;
        }
        if sim.parts[i].tmp3 == 0 {
            let t = sim.parts[i].tmp2 as u16;
            sim.part_change_type(i, x, y, t as i16);
            sim.parts[i].tmp2 = 0;
            sim.parts[i].tmp3 = 0;
            sim.parts[i].tmp4 = 0;
        }
        return 0;
    }
    if sim.parts[i].tmp4 != 0 {
        if rnd & 7 == 0 {
            sim.parts[i].tmp4 -= 1;
            if sim.parts[i].tmp4 <= 0 {
                sim.kill_part(i);
                return 1;
            }
        }
        rnd >>= 3;
    }
    for rx in -1..=1 {
        for ry in -1..=1 {
            if rx == 0 && ry == 0 {
                rnd = sim.rng.gen();
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
            if sim.parts[id].tmp3 != 0 && matches!(rt, pt::VIRS | pt::VRSS | pt::VRSG) {
                sim.parts[i].tmp3 = sim.parts[id].tmp3 + if rnd & 3 != 0 { 2 } else { 1 };
                return 0;
            } else if rt == pt::SOAP {
                sim.parts[i].tmp3 += 10;
                if rnd & 3 == 0 {
                    sim.kill_part(id);
                }
                return 0;
            } else if rt == pt::PLSM {
                if surround_space > 0
                    && sim.rng.chance(10 + pv(sim, nx, ny) as i32, 100)
                {
                    sim.part_change_type(i, x, y, pt::PLSM as i16);
                    return 1;
                }
            } else if !matches!(rt, pt::VIRS | pt::VRSS | pt::VRSG | pt::DMND | pt::BASE) {
                if rnd & 7 == 0 {
                    sim.parts[id].tmp2 = rt as i32;
                    sim.parts[id].tmp3 = 0;
                    if sim.parts[i].tmp4 != 0 {
                        sim.parts[id].tmp4 = sim.parts[i].tmp4 + 1;
                    } else {
                        sim.parts[id].tmp4 = 0;
                    }
                    let temp = sim.parts[id].temp;
                    if temp < 305.0 {
                        sim.part_change_type(id, nx, ny, pt::VRSS as i16);
                    } else if temp > 673.0 {
                        sim.part_change_type(id, nx, ny, pt::VRSG as i16);
                    } else {
                        sim.part_change_type(id, nx, ny, pt::VIRS as i16);
                    }
                }
                rnd >>= 3;
            } else if sim.in_bounds(nx, ny) && pmap_type(sim.photons[sim.idx(nx, ny)]) == pt::PROT {
                sim.parts[i].tmp4 = 0;
            }
        }
    }
    0
}

// --- GRVT ----------------------------------------------------------------
fn grvt_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.parts[i].tmp >= 100 {
        sim.parts[i].tmp = 100;
    }
    if sim.parts[i].tmp <= -100 {
        sim.parts[i].tmp = -100;
    }
    let under = sim.type_at(x, y);
    if under == pt::RSSS && sim.rng.chance(1, 5) {
        sim.kill_part(i);
        return 1;
    }
    let cx = x / CELL as i32;
    let cy = y / CELL as i32;
    if sim.in_cell_bounds(cx, cy) {
        let vc = sim.cell_idx(cx, cy);
        sim.grav_mass[vc] = 0.2 * sim.parts[i].tmp as f32;
    }
    0
}

fn grvt_create(sim: &mut Sim, i: usize, _x: i32, _y: i32, _t: i32, _v: i32) {
    let a = sim.rng.between(0, 359) as f32 * std::f32::consts::PI / 180.0;
    sim.parts[i].life = 250 + sim.rng.between(0, 199);
    sim.parts[i].vx = 2.0 * a.cos();
    sim.parts[i].vy = 2.0 * a.sin();
}

// --- RFRG ----------------------------------------------------------------
fn rfrg_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let new_pressure = pv(sim, x, y);
    let old = f32::from_bits(sim.parts[i].tmp as u32);
    if old.is_nan() {
        sim.parts[i].tmp = new_pressure.to_bits() as i32;
        return 0;
    }
    let t = sim.parts[i].temp * ((new_pressure + 257.0) / (old + 257.0));
    sim.parts[i].temp = restrict_flt(t, 0.0, MAX_TEMP);
    sim.parts[i].tmp = new_pressure.to_bits() as i32;
    0
}

// --- LSNS ----------------------------------------------------------------
fn lsns_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let mut rd = sim.parts[i].tmp2;
    if rd > 25 {
        rd = 25;
        sim.parts[i].tmp2 = 25;
    }
    if sim.parts[i].life != 0 {
        sim.parts[i].life = 0;
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
                let pavg = parts_avg(sim, i, id, pt::INSL);
                if pavg != pt::INSL
                    && pavg != pt::RSSS
                    && (element(rt).properties & props::PROP_CONDUCTS) != 0
                    && !matches!(rt, pt::WATR | pt::SLTW | pt::NTCT | pt::PTCT | pt::INWR)
                    && sim.parts[id].life == 0
                {
                    sim.parts[id].life = 4;
                    sim.parts[id].ctype = rt as i32;
                    sim.part_change_type(id, nx, ny, pt::SPRK as i16);
                }
            }
        }
    }
    let mode = sim.parts[i].tmp;
    let mut ser = false;
    let mut deser = false;
    let mut life = 0;
    for rx in -rd..=rd {
        for ry in -rd..=rd {
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
            match mode {
                1 => {
                    if rt != pt::LSNS && rt != pt::FILT && sim.parts[id].life >= 0 {
                        ser = true;
                        life = sim.parts[id].life;
                    }
                }
                3 => {
                    if rt == pt::FILT {
                        deser = true;
                        life = sim.parts[id].ctype;
                    }
                }
                2 => {
                    if rt != pt::METL && sim.parts[id].life as f32 <= sim.parts[i].temp - 273.15 {
                        sim.parts[i].life = 1;
                    }
                }
                _ => {
                    if rt != pt::METL && sim.parts[id].life as f32 > sim.parts[i].temp - 273.15 {
                        sim.parts[i].life = 1;
                    }
                }
            }
        }
    }
    for rx in -1..=1 {
        for ry in -1..=1 {
            if rx == 0 && ry == 0 {
                continue;
            }
            let nx0 = x + rx;
            let ny0 = y + ry;
            if !sim.in_bounds(nx0, ny0) {
                continue;
            }
            let mut r = sim.pmap[sim.idx(nx0, ny0)];
            if r == 0 {
                r = sim.photons[sim.idx(nx0, ny0)];
            }
            if r == 0 {
                continue;
            }
            let mut nx = nx0;
            let mut ny = ny0;
            if ser {
                while r != 0 && pmap_type(r) == pt::FILT {
                    let id = pmap_id(r) as usize;
                    sim.parts[id].ctype = 0x10000000 + life;
                    nx += rx;
                    ny += ry;
                    if !sim.in_bounds(nx, ny) {
                        break;
                    }
                    r = sim.pmap[sim.idx(nx, ny)];
                }
            } else if deser {
                if pmap_type(r) != pt::FILT {
                    let id = pmap_id(r) as usize;
                    sim.parts[id].life = life - 0x10000000;
                }
            }
        }
    }
    0
}

// --- LDTC ----------------------------------------------------------------
fn ldtc_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let ctype = sim.parts[i].ctype & 0x1FF;
    let tmp = sim.parts[i].tmp;
    let _ = tmp;
    // Simplified: spark conductors or propagate FILT colour in a straight line.
    for rx in -1..=1 {
        for ry in -1..=1 {
            if rx == 0 && ry == 0 {
                continue;
            }
            let nx0 = x + rx;
            let ny0 = y + ry;
            if !sim.in_bounds(nx0, ny0) {
                continue;
            }
            let r = sim.pmap[sim.idx(nx0, ny0)];
            if r == 0 {
                continue;
            }
            let id = pmap_id(r) as usize;
            let rt = pmap_type(r);
            let conducts = (element(rt).properties & props::PROP_CONDUCTS) != 0
                && !matches!(rt, pt::WATR | pt::SLTW | pt::NTCT | pt::PTCT | pt::INWR)
                && sim.parts[id].life == 0;
            if conducts {
                sim.parts[id].life = 4;
                sim.parts[id].ctype = rt as i32;
                sim.part_change_type(id, nx0, ny0, pt::SPRK as i16);
            } else {
                let mut nx = nx0;
                let mut ny = ny0;
                let mut rr = r;
                while rr != 0 && pmap_type(rr) == pt::FILT {
                    let fid = pmap_id(rr) as usize;
                    sim.parts[fid].ctype = ctype;
                    nx += rx;
                    ny += ry;
                    if !sim.in_bounds(nx, ny) {
                        break;
                    }
                    rr = sim.pmap[sim.idx(nx, ny)];
                }
            }
        }
    }
    0
}

// --- VSNS ----------------------------------------------------------------
fn vsns_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let mut rd = sim.parts[i].tmp2;
    if rd > 25 {
        rd = 25;
        sim.parts[i].tmp2 = 25;
    }
    if sim.parts[i].life != 0 {
        sim.parts[i].life = 0;
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
                if pavg != pt::INSL
                    && pavg != pt::RSSS
                    && (element(rt).properties & props::PROP_CONDUCTS) != 0
                    && !matches!(rt, pt::WATR | pt::SLTW | pt::NTCT | pt::PTCT | pt::INWR)
                    && sim.parts[id].life == 0
                {
                    sim.parts[id].life = 4;
                    sim.parts[id].ctype = rt as i32;
                    sim.part_change_type(id, nx, ny, pt::SPRK as i16);
                }
            }
        }
    }
    let mode = sim.parts[i].tmp;
    for rx in -rd..=rd {
        for ry in -rd..=rd {
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
            let p = element(rt).properties;
            let vm = (sim.parts[id].vx * sim.parts[id].vx + sim.parts[id].vy * sim.parts[id].vy).sqrt();
            if mode == 2 {
                if (p & props::TYPE_SOLID) == 0 && vm <= sim.parts[i].temp - 273.15 {
                    sim.parts[i].life = 1;
                }
            } else if mode == 0 || mode == 1 {
                if (p & props::TYPE_SOLID) == 0 && vm > sim.parts[i].temp - 273.15 {
                    sim.parts[i].life = mode;
                }
            }
        }
    }
    0
}

// --- CRMC ----------------------------------------------------------------
fn crmc_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let orig = sim.parts[i].temp;
    if pv(sim, x, y) < -30.0 {
        sim.part_change_type(i, x, y, pt::CLST as i16);
    }
    sim.parts[i].temp = orig;
    0
}

fn crmc_create(sim: &mut Sim, i: usize, _x: i32, _y: i32, _t: i32, _v: i32) {
    sim.parts[i].tmp2 = sim.rng.between(0, 4);
}

// --- HEAC ----------------------------------------------------------------
/// Reference `CheckLine`: returns true when the segment between `(x1,y1)` and
/// `(x2,y2)` is blocked by a heat insulator. Used by `HEAC` to provide
/// line-of-sight heat equalisation.
fn heac_check_line(sim: &Sim, x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
    let mut reverse_xy = (y2 - y1).abs() > (x2 - x1).abs();
    let mut x1 = x1;
    let mut y1 = y1;
    let mut x2 = x2;
    let mut y2 = y2;
    if reverse_xy {
        let t = x1; x1 = y1; y1 = t;
        let t = x2; x2 = y2; y2 = t;
    }
    if x1 > x2 {
        let t = x1; x1 = x2; x2 = t;
        let t = y1; y1 = y2; y2 = t;
    }
    let dx = x2 - x1;
    let dy = (y2 - y1).abs();
    let mut e = 0.0f32;
    let de = if dx != 0 {
        dy as f32 / dx as f32
    } else {
        0.0
    };
    let mut y = y1;
    let sy = if y1 < y2 { 1 } else { -1 };
    for x in x1..=x2 {
        let r = if reverse_xy {
            sim.pmap_at(y, x)
        } else {
            sim.pmap_at(x, y)
        };
        if r != 0 {
            let id = pmap_id(r) as usize;
            if id < sim.parts.len() && sim.is_heat_insulator(id) {
                return true;
            }
        }
        e += de;
        if e >= 0.5 {
            y += sy;
            if if y1 < y2 { y <= y2 } else { y >= y2 } {
                let r = if reverse_xy {
                    sim.pmap_at(y, x)
                } else {
                    sim.pmap_at(x, y)
                };
                if r != 0 {
                    let id = pmap_id(r) as usize;
                    if id < sim.parts.len() && sim.is_heat_insulator(id) {
                        return true;
                    }
                }
            }
            e -= 1.0;
        }
    }
    false
}

fn heac_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let rad = 4;
    let mut c_heat = 0.0f32;
    let mut hc_total = 0.0f32;
    for rx in -1..=1 {
        for ry in -1..=1 {
            let rrx = rx * rad;
            let rry = ry * rad;
            if rrx == 0 && rry == 0 {
                continue;
            }
            let nx = x + rrx;
            let ny = y + rry;
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            if heac_check_line(sim, x, y, nx, ny) {
                continue;
            }
            // pmap particle at target.
            if let Some(id) = sim.id_at(nx, ny) {
                if !sim.is_heat_insulator(id) {
                    let hc = sim.heat_capacity_of(id);
                    c_heat += sim.parts[id].temp * hc;
                    hc_total += hc;
                }
            }
            // photon (energy) particle at target.
            let cell = sim.idx(nx, ny);
            let r = sim.photons[cell];
            if r != 0 {
                let id = pmap_id(r) as usize;
                if id < sim.parts.len() && !sim.is_heat_insulator(id) {
                    let hc = sim.heat_capacity_of(id);
                    c_heat += sim.parts[id].temp * hc;
                    hc_total += hc;
                }
            }
        }
    }
    if hc_total > 0.0 {
        let pt = restrict_flt(c_heat / hc_total, MIN_TEMP, MAX_TEMP);
        sim.parts[i].temp = pt;
        for rx in -1..=1 {
            for ry in -1..=1 {
                let rrx = rx * rad;
                let rry = ry * rad;
                if rrx == 0 && rry == 0 {
                    continue;
                }
                let nx = x + rrx;
                let ny = y + rry;
                if !sim.in_bounds(nx, ny) {
                    continue;
                }
                if heac_check_line(sim, x, y, nx, ny) {
                    continue;
                }
                if let Some(id) = sim.id_at(nx, ny) {
                    if !sim.is_heat_insulator(id) {
                        sim.parts[id].temp = pt;
                    }
                }
                let cell = sim.idx(nx, ny);
                let r = sim.photons[cell];
                if r != 0 {
                    let id = pmap_id(r) as usize;
                    if id < sim.parts.len() && !sim.is_heat_insulator(id) {
                        sim.parts[id].temp = pt;
                    }
                }
            }
        }
    }
    0
}

// --- POLO ----------------------------------------------------------------
fn polo_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    const COOLDOWN: i32 = 15;
    const LIMIT: i32 = 5;
    let cell = sim.idx(x, y);
    let r = sim.photons[cell];
    if sim.parts[i].tmp < LIMIT && sim.parts[i].life == 0 {
        if sim.rng.chance(1, 10000) && sim.parts[i].tmp == 0 {
            if let Some(s) = create_near(sim, x, y, pt::NEUT) {
                sim.parts[i].life = COOLDOWN;
                sim.parts[i].tmp += 1;
                let t = (sim.parts[i].temp + sim.parts[s].temp + 600.0) / 2.0;
                sim.parts[i].temp = t;
                sim.parts[s].temp = t;
            }
        }
        if r != 0 && sim.rng.chance(1, 100) {
            let rid = pmap_id(r) as usize;
            if let Some(s) = create_near(sim, x, y, pt::NEUT) {
                let t = (sim.parts[i].temp + sim.parts[rid].temp + sim.parts[rid].temp + 600.0) / 3.0;
                sim.parts[i].temp = t;
                sim.parts[i].life = COOLDOWN;
                sim.parts[i].tmp += 1;
                sim.parts[rid].temp = t;
                sim.parts[s].temp = t;
                sim.parts[s].vx = sim.parts[rid].vx;
                sim.parts[s].vy = sim.parts[rid].vy;
            }
        }
    }
    if sim.parts[i].tmp2 >= 10 {
        sim.part_change_type(i, x, y, pt::PLUT as i16);
        sim.parts[i].temp = (sim.parts[i].temp + 600.0) / 2.0;
        return 1;
    }
    if r != 0 && pmap_type(r) == pt::PROT {
        sim.parts[i].tmp2 += 1;
        let rid = pmap_id(r) as usize;
        sim.kill_part(rid);
    }
    if sim.parts[i].temp < 388.15 {
        sim.parts[i].temp += 0.2;
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
    if sim.rng.chance(10000 % (temp as i32 + 1), temp as u32 + 1) {
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
    for _ in 0..5 {
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
        let cell_a = sim.idx(x, y);
        let cell_b = sim.idx(nx, ny);
        sim.pmap[cell_a] = r;
        sim.pmap[cell_b] = crate::sim::pmap_pack(i as u32, pt::WARP);
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
            if sim.parts[i].tmp != 1 && !(rx == 0 && ry == 0) {
                let vc = sim.cell_idx(cx, cy);
                sim.pv[vc] += 0.1 * ((sim.parts[i].temp - 273.15) - sim.pv[vc]);
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

// ===========================================================================
// Batch 4a: thermal / chemistry / special elements
// ===========================================================================

// --- FIRW ----------------------------------------------------------------
fn firw_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
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
                    let (mut gx, mut gy) = sim.gravity_field(x, y, element(pt::FIRW).gravity, 1.0);
                    if gx * gx + gy * gy < 0.001 {
                        let angle = sim.rng.between(0, 6283) as f32 * 0.001;
                        gx += angle.sin() * element(pt::FIRW).gravity * 0.5;
                        gy += angle.cos() * element(pt::FIRW).gravity * 0.5;
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

// --- CONV ----------------------------------------------------------------
fn conv_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let limit = 50;
    let p = pv(sim, x, y);
    if sim.parts[i].life < limit && p <= 3.0 && p >= -3.0 && sim.parts[i].temp <= 374.0 {
        let denom = (sim.parts[i].life * 10000 / limit + 500) as u32;
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
                match pmap_type(r) {
                    pt::WATR | pt::DSTW | pt::FRZW => {
                        if sim.parts[i].life < limit && sim.rng.chance(500, denom) {
                            sim.parts[i].life += 1;
                            sim.kill_part(id);
                        }
                    }
                    pt::SLTW => {
                        if sim.parts[i].life < limit && sim.rng.chance(50, denom) {
                            sim.parts[i].life += 1;
                            if sim.rng.chance(3, 4) {
                                sim.kill_part(id);
                            } else {
                                sim.part_change_type(id, nx, ny, pt::SALT as i16);
                            }
                        }
                    }
                    pt::CBNW => {
                        if sim.parts[i].life < limit && sim.rng.chance(100, denom) {
                            sim.parts[i].life += 1;
                            sim.part_change_type(id, nx, ny, pt::CO2 as i16);
                        }
                    }
                    pt::PSTE => {
                        if sim.parts[i].life < limit && sim.rng.chance(20, denom) {
                            sim.parts[i].life += 1;
                            sim.part_change_type(id, nx, ny, pt::CLST as i16);
                        }
                    }
                    _ => {}
                }
            }
        }
    } else {
        for rx in -1..=1 {
            for ry in -1..=1 {
                if rx == 0 && ry == 0 {
                    continue;
                }
                let nx = x + rx;
                let ny = y + ry;
                if cell_free(sim, nx, ny) && sim.parts[i].life >= 1 {
                    if sim.create_part(nx, ny, pt::WATR).is_some() {
                        sim.parts[i].life -= 1;
                    }
                }
            }
        }
    }
    for _ in 0..9 {
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
        if pmap_type(r) == pt::SPNG && sim.parts[i].life > sim.parts[id].life && sim.parts[i].life > 0 {
            let diff = sim.parts[i].life - sim.parts[id].life;
            if diff == 1 {
                sim.parts[id].life += 1;
                sim.parts[i].life -= 1;
                break;
            } else if diff > 0 {
                sim.parts[id].life += diff / 2;
                sim.parts[i].life -= diff / 2;
                break;
            }
        }
    }
    let mut tmp = 0;
    if sim.parts[i].life > 0 {
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
                if pmap_type(r) == pt::FIRE {
                    tmp += 1;
                    if sim.parts[id].life > 60 {
                        sim.parts[id].life -= sim.parts[id].life / 60;
                    } else if sim.parts[id].life > 2 {
                        sim.parts[id].life -= 1;
                    }
                }
            }
        }
    }
    if tmp != 0 && sim.parts[i].life > 3 {
        sim.parts[i].life -= sim.parts[i].life / 3;
    }
    if tmp > 1 {
        tmp /= 2;
    }
    if tmp != 0 || sim.parts[i].temp >= 374.0 {
        for rx in -1..=1 {
            for ry in -1..=1 {
                if rx == 0 && ry == 0 {
                    continue;
                }
                let nx = x + rx;
                let ny = y + ry;
                if cell_free(sim, nx, ny) && sim.parts[i].life >= 1 {
                    let temp = sim.parts[i].temp;
                    if let Some(np) = sim.create_part(nx, ny, pt::WTRV) {
                        sim.parts[np].temp = temp;
                        tmp -= 1;
                        sim.parts[i].life -= 1;
                        sim.parts[i].temp = restrict_flt(sim.parts[i].temp - 20.0, MIN_TEMP, MAX_TEMP);
                    }
                }
            }
        }
    }
    if tmp > 0 {
        if sim.parts[i].life > tmp {
            sim.parts[i].life -= tmp;
        } else {
            sim.parts[i].life = 0;
        }
    }
    0
}

// --- EXOT ----------------------------------------------------------------
fn exot_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
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
            if rt == pt::WARP {
                if sim.parts[id].tmp2 > 2000 && sim.rng.chance(1, 100) {
                    sim.parts[i].tmp2 += 100;
                }
            } else if rt == pt::EXOT {
                if sim.parts[id].ctype == pt::PROT as i32 {
                    sim.parts[i].ctype = pt::PROT as i32;
                }
                if sim.parts[id].life == 1500 && sim.rng.chance(1, 1000) {
                    sim.parts[i].life = 1500;
                }
            } else if rt == pt::LAVA {
                if sim.parts[id].ctype == pt::TTAN as i32 || sim.parts[id].ctype == pt::GOLD as i32 {
                    if sim.rng.chance(1, 10) {
                        sim.parts[id].ctype = pt::VIBR as i32;
                        sim.kill_part(i);
                        return 1;
                    }
                } else if sim.parts[id].ctype == pt::VIBR as i32 && sim.rng.chance(1, 1000) {
                    sim.kill_part(i);
                    return 1;
                }
            }
            if sim.parts[i].tmp > 245 && sim.parts[i].life > 1337 {
                if !matches!(
                    rt,
                    pt::EXOT | pt::BREC | pt::DMND | pt::CLNE | pt::PRTI | pt::PRTO | pt::PCLN
                        | pt::VOID | pt::NBHL | pt::WARP
                ) {
                    if sim.part_change_type(i, x, y, rt as i16) {
                        return 1;
                    }
                }
            }
        }
    }
    sim.parts[i].tmp -= 1;
    sim.parts[i].tmp2 -= 1;
    if sim.parts[i].tmp < 1 || sim.parts[i].tmp > 250 {
        sim.parts[i].tmp = 250;
    }
    if sim.parts[i].tmp2 < 1 {
        sim.parts[i].tmp2 = 1;
    } else if sim.parts[i].tmp2 > 6000 {
        sim.parts[i].tmp2 = 10000;
        if sim.parts[i].life < 1001 {
            sim.part_change_type(i, x, y, pt::WARP as i16);
            return 1;
        }
    } else if sim.parts[i].life < 1001 {
        if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
            let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
            sim.pv[vc] += (sim.parts[i].tmp2 as f32) / 160000.0;
        }
    }
    if pv(sim, x, y) > 200.0 && sim.parts[i].temp > 9000.0 && sim.parts[i].tmp2 > 200 {
        sim.parts[i].tmp2 = 6000;
        sim.part_change_type(i, x, y, pt::WARP as i16);
        return 1;
    }
    if sim.parts[i].tmp2 > 100 {
        for _ in 0..9 {
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
            if pmap_type(r) == pt::EXOT && sim.parts[i].tmp2 > sim.parts[id].tmp2 && sim.parts[id].tmp2 >= 0 {
                let diff = sim.parts[i].tmp2 - sim.parts[id].tmp2;
                if diff == 1 {
                    sim.parts[id].tmp2 += 1;
                    sim.parts[i].tmp2 -= 1;
                    break;
                } else if diff > 0 {
                    sim.parts[id].tmp2 += diff / 2;
                    sim.parts[i].tmp2 -= diff / 2;
                    break;
                }
            }
        }
    }
    if sim.parts[i].ctype == pt::PROT as i32 {
        if sim.parts[i].temp < 50.0 {
            sim.part_change_type(i, x, y, pt::CFLM as i16);
            return 1;
        } else {
            sim.parts[i].temp -= 1.0;
        }
    } else if sim.parts[i].temp < 273.15 {
        sim.parts[i].vx = 0.0;
        sim.parts[i].vy = 0.0;
        if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
            let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
            sim.pv[vc] -= 0.01;
        }
        sim.parts[i].tmp -= 1;
    }
    0
}

// --- GEL -----------------------------------------------------------------
fn gel_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.parts[i].tmp > 100 {
        sim.parts[i].tmp = 100;
    }
    if sim.parts[i].tmp < 0 {
        sim.parts[i].tmp = 0;
    }
    let denom = (sim.parts[i].tmp * 10 + 500) as u32;
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
            match rt {
                pt::WATR | pt::DSTW | pt::FRZW => {
                    if sim.parts[i].tmp < 100 && sim.rng.chance(500, denom) {
                        sim.parts[i].tmp += 1;
                        sim.kill_part(id);
                    }
                }
                pt::PSTE => {
                    if sim.parts[i].tmp < 100 && sim.rng.chance(20, denom) {
                        sim.parts[i].tmp += 1;
                        sim.part_change_type(id, nx, ny, pt::CLST as i16);
                    }
                }
                pt::SLTW => {
                    if sim.parts[i].tmp < 100 && sim.rng.chance(50, denom) {
                        sim.parts[i].tmp += 1;
                        if sim.rng.chance(3, 4) {
                            sim.kill_part(id);
                        } else {
                            sim.part_change_type(id, nx, ny, pt::SALT as i16);
                        }
                    }
                }
                pt::CBNW => {
                    if sim.parts[i].tmp < 100 && sim.rng.chance(100, denom) {
                        sim.parts[i].tmp += 1;
                        sim.part_change_type(id, nx, ny, pt::CO2 as i16);
                    }
                }
                pt::SPNG => {
                    if sim.parts[id].life > 0 && sim.parts[i].tmp < 100 && sim.parts[id].life + 1 > sim.parts[i].tmp {
                        sim.parts[id].life -= 1;
                        sim.parts[i].tmp += 1;
                    } else if sim.parts[i].tmp > 0 && sim.parts[id].life + 1 < sim.parts[i].tmp {
                        sim.parts[id].life += 1;
                        sim.parts[i].tmp -= 1;
                    }
                }
                pt::GEL => {
                    if sim.parts[id].tmp + 1 < sim.parts[i].tmp {
                        sim.parts[id].tmp += 1;
                        sim.parts[i].tmp -= 1;
                    }
                }
                pt::BASE => {
                    if sim.parts[i].tmp > 0 && sim.parts[id].life > 1 {
                        sim.parts[id].life -= 1;
                        sim.parts[i].tmp -= 1;
                    }
                }
                _ => {}
            }
            let dx = sim.parts[i].x - sim.parts[id].x;
            let dy = sim.parts[i].y - sim.parts[id].y;
            let falldown = element(rt).falldown;
            if dx * dx + dy * dy > 1.5 && (rt == pt::GEL || falldown == 0 || (rx.abs() < 2 && ry.abs() < 2)) {
                let nd = dx * dx + dy * dy - 0.5;
                let mut per = 5.0 * (1.0 - sim.parts[i].tmp as f32 / 100.0) * (nd / (dx * dx + dy * dy + nd) - 0.5);
                if (element(rt).properties & props::TYPE_LIQUID) != 0 {
                    per *= 0.1;
                }
                let ddx = dx * per;
                let ddy = dy * per;
                sim.parts[i].vx += ddx;
                sim.parts[i].vy += ddy;
                if (element(rt).properties & props::TYPE_PART) != 0 || rt == pt::GOO {
                    sim.parts[id].vx -= ddx;
                    sim.parts[id].vy -= ddy;
                }
            }
        }
    }
    0
}

// --- STOR ----------------------------------------------------------------
fn stor_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if !(sim.parts[i].tmp == 0 || (sim.parts[i].tmp as usize) < crate::elements_generated::PT_NUM) {
        sim.parts[i].tmp = 0;
    }
    if sim.parts[i].life != 0 && sim.parts[i].tmp == 0 {
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
            let mut r = sim.pmap[sim.idx(nx, ny)];
            if r == 0 {
                r = sim.photons[sim.idx(nx, ny)];
            }
            if r == 0 {
                continue;
            }
            let id = pmap_id(r) as usize;
            let rt = pmap_type(r);
            if sim.parts[i].tmp == 0
                && sim.parts[i].life == 0
                && rt != pt::STOR
                && (element(rt).properties & props::TYPE_SOLID) == 0
                && (sim.parts[i].ctype == 0 || rt as i32 == sim.parts[i].ctype)
            {
                sim.parts[i].tmp = rt as i32;
                sim.parts[i].temp = sim.parts[id].temp;
                sim.parts[i].tmp2 = sim.parts[id].life;
                sim.parts[i].tmp3 = sim.parts[id].tmp;
                sim.parts[i].tmp4 = sim.parts[id].ctype;
                sim.kill_part(id);
            }
            if sim.parts[i].tmp != 0
                && rt == pt::SPRK
                && sim.parts[id].ctype == pt::PSCN as i32
                && sim.parts[id].life > 0
                && sim.parts[id].life < 4
            {
                'expand: for ry1 in (-1..=1).rev() {
                    for rx1 in [-1i32, 0, 1] {
                        let tx = x + rx1;
                        let ty = y + ry1;
                        if !cell_free(sim, tx, ty) {
                            continue;
                        }
                        let stored = sim.parts[i].tmp as u16;
                        let temp = sim.parts[i].temp;
                        let life = sim.parts[i].tmp2;
                        let tmp = sim.parts[i].tmp3;
                        let ctype = sim.parts[i].tmp4;
                        if let Some(np) = sim.create_part(tx, ty, stored) {
                            sim.parts[np].temp = temp;
                            sim.parts[np].life = life;
                            sim.parts[np].tmp = tmp;
                            sim.parts[np].ctype = ctype;
                            sim.parts[i].tmp = 0;
                            sim.parts[i].life = 10;
                            break 'expand;
                        }
                    }
                }
            }
        }
    }
    0
}

// --- SPNG ----------------------------------------------------------------
fn spng_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let limit = 50;
    let p = pv(sim, x, y);
    if sim.parts[i].life < limit && p <= 3.0 && p >= -3.0 && sim.parts[i].temp <= 374.0 {
        let denom = (sim.parts[i].life * 10000 / limit + 500) as u32;
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
                match pmap_type(r) {
                    pt::WATR | pt::DSTW | pt::FRZW => {
                        if sim.parts[i].life < limit && sim.rng.chance(500, denom) {
                            sim.parts[i].life += 1;
                            sim.kill_part(id);
                        }
                    }
                    pt::SLTW => {
                        if sim.parts[i].life < limit && sim.rng.chance(50, denom) {
                            sim.parts[i].life += 1;
                            if sim.rng.chance(3, 4) {
                                sim.kill_part(id);
                            } else {
                                sim.part_change_type(id, nx, ny, pt::SALT as i16);
                            }
                        }
                    }
                    pt::CBNW => {
                        if sim.parts[i].life < limit && sim.rng.chance(100, denom) {
                            sim.parts[i].life += 1;
                            sim.part_change_type(id, nx, ny, pt::CO2 as i16);
                        }
                    }
                    pt::PSTE => {
                        if sim.parts[i].life < limit && sim.rng.chance(20, denom) {
                            sim.parts[i].life += 1;
                            sim.part_change_type(id, nx, ny, pt::CLST as i16);
                        }
                    }
                    _ => {}
                }
            }
        }
    } else {
        for rx in -1..=1 {
            for ry in -1..=1 {
                if rx == 0 && ry == 0 {
                    continue;
                }
                let nx = x + rx;
                let ny = y + ry;
                if cell_free(sim, nx, ny) && sim.parts[i].life >= 1 {
                    if sim.create_part(nx, ny, pt::WATR).is_some() {
                        sim.parts[i].life -= 1;
                    }
                }
            }
        }
    }
    for _ in 0..9 {
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
        if pmap_type(r) == pt::SPNG && sim.parts[i].life > sim.parts[id].life && sim.parts[i].life > 0 {
            let diff = sim.parts[i].life - sim.parts[id].life;
            if diff == 1 {
                sim.parts[id].life += 1;
                sim.parts[i].life -= 1;
                break;
            } else if diff > 0 {
                sim.parts[id].life += diff / 2;
                sim.parts[i].life -= diff / 2;
                break;
            }
        }
    }
    let mut tmp = 0;
    if sim.parts[i].life > 0 {
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
                if pmap_type(r) == pt::FIRE {
                    tmp += 1;
                    if sim.parts[id].life > 60 {
                        sim.parts[id].life -= sim.parts[id].life / 60;
                    } else if sim.parts[id].life > 2 {
                        sim.parts[id].life -= 1;
                    }
                }
            }
        }
    }
    if tmp != 0 && sim.parts[i].life > 3 {
        sim.parts[i].life -= sim.parts[i].life / 3;
    }
    if tmp > 1 {
        tmp /= 2;
    }
    if tmp != 0 || sim.parts[i].temp >= 374.0 {
        for rx in -1..=1 {
            for ry in -1..=1 {
                if rx == 0 && ry == 0 {
                    continue;
                }
                let nx = x + rx;
                let ny = y + ry;
                if cell_free(sim, nx, ny) && sim.parts[i].life >= 1 {
                    let temp = sim.parts[i].temp;
                    if let Some(np) = sim.create_part(nx, ny, pt::WTRV) {
                        sim.parts[np].temp = temp;
                        tmp -= 1;
                        sim.parts[i].life -= 1;
                        sim.parts[i].temp = restrict_flt(sim.parts[i].temp - 20.0, MIN_TEMP, MAX_TEMP);
                    }
                }
            }
        }
    }
    if tmp > 0 {
        if sim.parts[i].life > tmp {
            sim.parts[i].life -= tmp;
        } else {
            sim.parts[i].life = 0;
        }
    }
    0
}

// --- FRZZ / FRZW ---------------------------------------------------------
fn frzz_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
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
            if pmap_type(r) == pt::WATR && sim.rng.chance(1, 20) {
                sim.part_change_type(id, nx, ny, pt::FRZW as i16);
                sim.parts[id].life = 100;
                sim.kill_part(i);
                return 1;
            }
        }
    }
    0
}

fn frzw_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
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
            if pmap_type(r) == pt::WATR && sim.rng.chance(1, 14) {
                sim.part_change_type(id, nx, ny, pt::FRZW as i16);
            }
        }
    }
    let life = sim.parts[i].life;
    if (life == 0 && sim.rng.chance(1, 192)) || sim.rng.chance(100 - life, 50000) {
        sim.part_change_type(i, x, y, pt::ICEI as i16);
        sim.parts[i].ctype = pt::FRZW as i32;
        sim.parts[i].temp = restrict_flt(sim.parts[i].temp - 200.0, MIN_TEMP, MAX_TEMP);
    }
    0
}

// --- GRAV ----------------------------------------------------------------
fn grav_update(sim: &mut Sim, i: usize, _x: i32, _y: i32, _s: i32, _nt: i32) -> i32 {
    let v2 = sim.parts[i].vx * sim.parts[i].vx + sim.parts[i].vy * sim.parts[i].vy;
    if v2 >= 0.1 && sim.rng.chance(1, 512) && sim.parts[i].life == 0 {
        sim.parts[i].life = 48;
    }
    0
}

// --- BIZR ----------------------------------------------------------------
fn bizr_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    const BLEND: f32 = 21.0;
    if sim.parts[i].dcolour != 0 {
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
                if matches!(rt, pt::BIZR | pt::BIZRG | pt::BIZRS) {
                    continue;
                }
                let tr = ((sim.parts[id].dcolour >> 16) & 0xFF) as f32;
                let tg = ((sim.parts[id].dcolour >> 8) & 0xFF) as f32;
                let tb = (sim.parts[id].dcolour & 0xFF) as f32;
                let ta = ((sim.parts[id].dcolour >> 24) & 0xFF) as f32;
                let mr = ((sim.parts[i].dcolour >> 16) & 0xFF) as f32;
                let mg = ((sim.parts[i].dcolour >> 8) & 0xFF) as f32;
                let mb = (sim.parts[i].dcolour & 0xFF) as f32;
                let ma = ((sim.parts[i].dcolour >> 24) & 0xFF) as f32;
                let nr = tr + (mr - tr).signum() + ((mr - tr) / BLEND).round();
                let ng = tg + (mg - tg).signum() + ((mg - tg) / BLEND).round();
                let nb = tb + (mb - tb).signum() + ((mb - tb) / BLEND).round();
                let na = ta + (ma - ta).signum() + ((ma - ta) / BLEND).round();
                sim.parts[id].dcolour = ((na as u32 & 0xFF) << 24)
                    | ((nr as u32 & 0xFF) << 16)
                    | ((ng as u32 & 0xFF) << 8)
                    | (nb as u32 & 0xFF);
            }
        }
    }
    0
}

// --- ISOZ / ISZS ---------------------------------------------------------
fn isoz_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.rng.chance(1, 200) && sim.rng.chance((-4.0 * pv(sim, x, y)) as i32, 1000) {
        let rr = sim.rng.between(128, 355) as f32 / 127.0;
        let rrr = sim.rng.between(0, 359) as f32 * std::f32::consts::PI / 180.0;
        if sim.part_change_type(i, x, y, pt::PHOT as i16) {
            sim.parts[i].vx = rr * rrr.cos();
            sim.parts[i].vy = rr * rrr.sin();
        }
    }
    0
}

fn iszs_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.rng.chance(1, 200) && sim.rng.chance((-4.0 * pv(sim, x, y)) as i32, 1000) {
        let rr = sim.rng.between(128, 355) as f32 / 127.0;
        let rrr = sim.rng.between(0, 359) as f32 * std::f32::consts::PI / 180.0;
        if sim.part_change_type(i, x, y, pt::PHOT as i16) {
            sim.parts[i].vx = rr * rrr.cos();
            sim.parts[i].vy = rr * rrr.sin();
        }
    }
    0
}

// --- SLCN ----------------------------------------------------------------
fn slcn_init_sparkles(sim: &mut Sim, i: usize) {
    sim.parts[i].tmp = sim.rng.between(0x100000, 0x9FFFFF);
}

fn slcn_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.parts[i].tmp == 0 {
        slcn_init_sparkles(sim, i);
    }
    let tmp = sim.parts[i].tmp as u32;
    let phase = (tmp & 0xFFF) + ((tmp >> 20) & 0xFFF);
    if phase & 0x1000 != 0 {
        let next = sim.rng.between(0, 15) as u32;
        sim.parts[i].tmp = ((tmp & 0xFFF0_0000) | (phase & 0xFFF) | (next << 16) | ((tmp >> 4) & 0xF000)) as i32;
    } else {
        sim.parts[i].tmp = ((tmp & 0xFFFF_F000) | phase) as i32;
    }
    if sim.parts[i].life == 0 && sim.parts[i].temp < 373.15 {
        for (rx, ry) in [(-4, 0), (4, 0), (0, -4), (0, 4)] {
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
            if pmap_type(r) == pt::SPRK && sim.parts[id].life != 0 && sim.parts[id].life < 4 {
                sim.part_change_type(i, x, y, pt::SPRK as i16);
                sim.parts[i].life = 4;
                sim.parts[i].ctype = pt::SLCN as i32;
            }
        }
    }
    0
}

fn slcn_create(sim: &mut Sim, i: usize, _x: i32, _y: i32, _t: i32, _v: i32) {
    slcn_init_sparkles(sim, i);
}

// --- RSST / RSSS ---------------------------------------------------------
fn rsst_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
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
            let rt = pmap_type(r);
            if rt == pt::GUNP {
                sim.part_change_type(i, x, y, pt::FIRW as i16);
                sim.kill_part(id);
                return 1;
            }
            if rt == pt::BCOL {
                sim.part_change_type(i, x, y, pt::FSEP as i16);
                sim.parts[i].life = 50;
                sim.kill_part(id);
                return 1;
            }
            if matches!(rt, pt::CLNE | pt::PCLN) && sim.parts[id].ctype != pt::RSST as i32 {
                sim.parts[i].ctype = sim.parts[id].ctype;
            }
            if matches!(rt, pt::BCLN | pt::PBCN) && sim.parts[id].ctype != pt::RSST as i32 {
                sim.parts[i].tmp = sim.parts[id].ctype;
            }
        }
    }
    0
}

fn rsss_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
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
            let rt = pmap_type(r);
            if matches!(rt, pt::CLNE | pt::PCLN) && sim.parts[id].ctype != pt::RSSS as i32 {
                sim.parts[i].ctype = sim.parts[id].ctype;
            }
            if matches!(rt, pt::BCLN | pt::PBCN) && sim.parts[id].ctype != pt::RSSS as i32 {
                sim.parts[i].tmp = sim.parts[id].ctype;
            }
        }
    }
    let cx = x / CELL as i32;
    let cy = y / CELL as i32;
    if sim.in_cell_bounds(cx, cy) {
        let vc = sim.cell_idx(cx, cy);
        sim.bmap_blockair[vc] = 1;
        sim.bmap_blockairh[vc] = 0x8;
    }
    0
}

// --- GPMP ----------------------------------------------------------------
fn gpmp_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.parts[i].life != 10 {
        if sim.parts[i].life > 0 {
            sim.parts[i].life -= 1;
        }
        return 0;
    }
    if sim.parts[i].temp >= 256.0 + 273.15 {
        sim.parts[i].temp = 256.0 + 273.15;
    }
    if sim.parts[i].temp <= -256.0 + 273.15 {
        sim.parts[i].temp = -256.0 + 273.15;
    }
    let cx = x / CELL as i32;
    let cy = y / CELL as i32;
    if sim.in_cell_bounds(cx, cy) {
        let vc = sim.cell_idx(cx, cy);
        sim.grav_mass[vc] = 0.2 * (sim.parts[i].temp - 273.15);
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
            if pmap_type(r) == pt::GPMP {
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

// ===========================================================================
// Batch 4b: optics, lightning, geology, life
// ===========================================================================

// --- LIGH ----------------------------------------------------------------
fn ligh_create_line(sim: &mut Sim, x1: i32, y1: i32, x2: i32, y2: i32, temp: f32, life: i32, tmp: i32, tmp2: i32, i: usize) {
    let reverse_xy = (y2 - y1).abs() > (x2 - x1).abs();
    let (mut x1, mut y1, mut x2, mut y2) = (x1, y1, x2, y2);
    if reverse_xy {
        std::mem::swap(&mut x1, &mut y1);
        std::mem::swap(&mut x2, &mut y2);
    }
    let back = x1 > x2;
    let dx = x2 - x1;
    let dy = (y2 - y1).abs();
    let de = if dx != 0 { dy as f32 / dx as f32 } else { 0.0 };
    let mut y = y1;
    let ystep = if y1 < y2 { 1 } else { -1 };
    let mut e = 0.0f32;
    let place = |sim: &mut Sim, x: i32, y: i32, last: bool| -> bool {
        let (px, py) = if reverse_xy { (y, x) } else { (x, y) };
        if let Some(p) = sim.create_part(px, py, pt::LIGH) {
            sim.parts[p].temp = temp;
            sim.parts[p].tmp = tmp;
            sim.parts[p].dcolour = sim.parts[i].dcolour;
            if last {
                let next = (life as f32 / 1.5) as i32 - sim.rng.between(0, 1);
                sim.parts[p].life = next;
                if next > 1 {
                    let do_branch = sim.rng.chance(7, 10);
                    sim.parts[p].tmp2 = if do_branch { 2 } else { 0 } + if p > i && tmp2 != 4 { 1 } else { 0 };
                } else {
                    sim.parts[p].tmp2 = 7 + if p > i { 1 } else { 0 };
                }
            } else {
                sim.parts[p].life = life;
                sim.parts[p].tmp2 = 7 + if p > i { 1 } else { 0 };
            }
            return false;
        }
        true
    };
    if !back {
        let mut x = x1;
        while x <= x2 {
            if place(sim, x, y, x == x2) {
                return;
            }
            e += de;
            if e >= 0.5 {
                y += ystep;
                e -= 1.0;
            }
            x += 1;
        }
    } else {
        let mut x = x1;
        while x >= x2 {
            if place(sim, x, y, x == x2) {
                return;
            }
            e += de;
            if e <= -0.5 {
                y += ystep;
                e += 1.0;
            }
            x -= 1;
        }
    }
}

fn ligh_update(sim: &mut Sim, i: usize, x: i32, y: i32, surround_space: i32, _nt: i32) -> i32 {
    const LIGHTING_POWER: f32 = 0.65;
    let powderful = (sim.parts[i].temp * (1.0 + sim.parts[i].life as f32 / 40.0) * LIGHTING_POWER) as i32;
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
            let def = element(rt);
            if (surround_space > 0 || def.explosive > 0)
                && (rt != pt::SPNG || sim.parts[id].life == 0)
                && def.flammable > 0
                && sim.rng.chance(def.flammable + (pv(sim, nx, ny) * 10.0) as i32, 1000)
            {
                sim.part_change_type(id, nx, ny, pt::FIRE as i16);
                sim.parts[id].temp = restrict_flt(
                    element(pt::FIRE).default_temp + (def.flammable / 2) as f32,
                    MIN_TEMP,
                    MAX_TEMP,
                );
                sim.parts[id].life = sim.rng.between(180, 259);
                sim.parts[id].tmp = 0;
                sim.parts[id].ctype = 0;
                if def.explosive > 0 {
                    if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
                        let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
                        sim.pv[vc] += 0.25;
                    }
                }
            }
            match rt {
                pt::LIGH | pt::TESC => continue,
                pt::CLNE | pt::THDR | pt::DMND | pt::FIRE => {
                    sim.parts[id].temp = restrict_flt(sim.parts[id].temp + powderful as f32 / 10.0, MIN_TEMP, MAX_TEMP);
                    continue;
                }
                pt::DEUT | pt::PLUT => {
                    sim.parts[id].temp = restrict_flt(sim.parts[id].temp + powderful as f32, MIN_TEMP, MAX_TEMP);
                    if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
                        let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
                        sim.pv[vc] = restrict_flt(sim.pv[vc] + powderful as f32 / 35.0, MIN_PRESSURE, MAX_PRESSURE);
                    }
                    if sim.rng.chance(1, 3) {
                        sim.part_change_type(id, nx, ny, pt::NEUT as i16);
                        sim.parts[id].life = sim.rng.between(480, 959);
                        sim.parts[id].vx = sim.rng.between(-5, 5) as f32;
                        sim.parts[id].vy = sim.rng.between(-5, 5) as f32;
                    }
                }
                pt::COAL | pt::BCOL => {
                    if sim.parts[id].life > 100 {
                        sim.parts[id].life = 99;
                    }
                }
                pt::HEAC => {
                    sim.parts[id].temp = restrict_flt(sim.parts[id].temp + powderful as f32 / 10.0, MIN_TEMP, MAX_TEMP);
                    if sim.parts[id].temp > element(pt::HEAC).high_temperature {
                        sim.part_change_type(id, nx, ny, pt::LAVA as i16);
                        sim.parts[id].ctype = pt::HEAC as i32;
                    }
                }
                _ => {}
            }
            if (def.properties & props::PROP_CONDUCTS) != 0 && sim.parts[id].life == 0 {
                if cell_free(sim, nx, ny) {
                    sim.create_part(nx, ny, pt::SPRK);
                }
            }
            if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
                let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
                sim.pv[vc] = restrict_flt(sim.pv[vc] + powderful as f32 / 400.0, MIN_PRESSURE, MAX_PRESSURE);
            }
            if !sim.is_heat_insulator(id) {
                sim.parts[id].temp = restrict_flt(sim.parts[id].temp + powderful as f32 / 1.3, MIN_TEMP, MAX_TEMP);
            }
        }
    }
    if sim.parts[i].tmp2 == 1 || sim.parts[i].tmp2 == 3 || (6..=8).contains(&sim.parts[i].tmp2) {
        if sim.parts[i].tmp2 >= 9 {
            sim.parts[i].tmp2 = 7;
        } else {
            sim.parts[i].tmp2 -= 1;
        }
        return 0;
    }
    if sim.parts[i].tmp2 == 5 || sim.parts[i].life <= 1 {
        sim.kill_part(i);
        return 1;
    }
    let angle = ((sim.parts[i].tmp + sim.rng.between(-30, 30)) % 360) as f32;
    let multiplier = (sim.parts[i].life as f32 * 1.5) as i32 + sim.rng.between(0, sim.parts[i].life);
    let rx = (angle.to_radians().cos() * multiplier as f32) as i32;
    let ry = (-angle.to_radians().sin() * multiplier as f32) as i32;
    ligh_create_line(sim, x, y, x + rx, y + ry, sim.parts[i].temp, sim.parts[i].life, angle as i32, sim.parts[i].tmp2, i);
    if sim.parts[i].tmp2 == 2 {
        let angle2 = ((angle as i32 + sim.rng.between(-100, 100)) % 360) as f32;
        let rx2 = (angle2.to_radians().cos() * multiplier as f32) as i32;
        let ry2 = (-angle2.to_radians().sin() * multiplier as f32) as i32;
        ligh_create_line(sim, x, y, x + rx2, y + ry2, sim.parts[i].temp, sim.parts[i].life, angle2 as i32, sim.parts[i].tmp2, i);
    }
    sim.parts[i].tmp2 = 7;
    0
}

fn ligh_create(sim: &mut Sim, i: usize, x: i32, y: i32, _t: i32, v: i32) {
    sim.parts[i].life = if v >= 0 { v.min(55) } else { 30 };
    sim.parts[i].temp = sim.parts[i].life as f32 * 150.0;
    let (mut gx, mut gy) = sim.gravity_field(x, y, 1.0, 1.0);
    let gsize = gx * gx + gy * gy;
    if gsize < 0.0016 {
        let angle = sim.rng.between(0, 6283) as f32 * 0.001;
        let gs = gsize.sqrt();
        gx += angle.cos() * (0.04 - gs);
        gy += angle.sin() * (0.04 - gs);
    }
    sim.parts[i].tmp = (((-gy).atan2(gx) * 180.0 / std::f32::consts::PI) as i32 + sim.rng.between(-20, 20) + 360) % 360;
    sim.parts[i].tmp2 = 4;
}

// --- PHOT ----------------------------------------------------------------
fn phot_create(sim: &mut Sim, i: usize, _x: i32, _y: i32, _t: i32, _v: i32) {
    let a = sim.rng.between(0, 7) as f32 * 0.78540;
    sim.parts[i].vx = 3.0 * a.cos();
    sim.parts[i].vy = 3.0 * a.sin();
}

fn phot_update(sim: &mut Sim, i: usize, x: i32, y: i32, surround_space: i32, nt: i32) -> i32 {
    if sim.parts[i].ctype & 0x3FFF_FFFF == 0 {
        sim.kill_part(i);
        return 1;
    }
    if sim.parts[i].temp > 506.0 && sim.rng.chance(1, 10) {
        fire_update(sim, i, x, y, surround_space, nt);
    }
    for rx in -1..=1 {
        for ry in -1..=1 {
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
            if rt == pt::ISOZ || rt == pt::ISZS {
                if sim.rng.chance(1, 400) {
                    sim.parts[i].vx *= 0.90;
                    sim.parts[i].vy *= 0.90;
                    sim.part_change_type(id, nx, ny, pt::PHOT as i16);
                    let rrr = sim.rng.between(0, 359) as f32 * std::f32::consts::PI / 180.0;
                    let rr = if rt == pt::ISOZ {
                        (sim.rng.between(128, 255) as f32 / 127.0) as i32
                    } else {
                        (sim.rng.between(128, 355) as f32 / 127.0) as i32
                    };
                    sim.parts[id].vx = rr as f32 * rrr.cos();
                    sim.parts[id].vy = rr as f32 * rrr.sin();
                    if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
                        let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
                        sim.pv[vc] -= 15.0;
                    }
                }
            } else if (rt == pt::QRTZ || rt == pt::PQRT) && rx == 0 && ry == 0 {
                let a = sim.rng.between(0, 359) as f32 * std::f32::consts::PI / 180.0;
                sim.parts[i].vx = 3.0 * a.cos();
                sim.parts[i].vy = 3.0 * a.sin();
                if sim.parts[i].ctype == 0x3FFF_FFFF {
                    sim.parts[i].ctype = 0x1F << sim.rng.between(0, 25);
                }
                if sim.parts[i].life != 0 {
                    sim.parts[i].life += 1;
                }
            } else if rt == pt::BGLA && rx == 0 && ry == 0 {
                let a = sim.rng.between(-50, 50) as f32 * 0.001;
                let (c, s) = (a.cos(), a.sin());
                let vx = c * sim.parts[i].vx + s * sim.parts[i].vy;
                let vy = c * sim.parts[i].vy - s * sim.parts[i].vx;
                sim.parts[i].vx = vx;
                sim.parts[i].vy = vy;
            } else if rt == pt::RSST && rx == 0 && ry == 0 {
                let ct_under = sim.parts[id].ctype;
                let tmp_under = sim.parts[id].tmp;
                if ct_under > 0 && (ct_under as usize) < crate::elements_generated::PT_NUM {
                    let ct = ct_under as u16;
                    sim.part_change_type(id, nx, ny, ct as i16);
                    if tmp_under > 0
                        && (tmp_under as usize) < crate::elements_generated::PT_NUM
                        && (element(ct).carries_type_in & (1 << 7)) != 0
                    {
                        sim.parts[id].ctype = tmp_under;
                    }
                } else {
                    sim.part_change_type(id, nx, ny, pt::RSSS as i16);
                }
                sim.kill_part(i);
                return 1;
            }
        }
    }
    0
}

// --- C5 ------------------------------------------------------------------
fn c5_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
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
            let conduct = sim.is_heat_insulator(id);
            if ((rt != pt::C5 && sim.parts[id].temp < 100.0 && !conduct) || rt == pt::CFLM) && sim.rng.chance(1, 6) {
                sim.part_change_type(i, x, y, pt::CFLM as i16);
                sim.parts[id].temp = 0.0;
                sim.parts[i].temp = 0.0;
                sim.parts[i].life = sim.rng.between(50, 199);
                if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
                    let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
                    sim.pv[vc] += 1.5;
                }
            }
        }
    }
    if sim.parts[i].ctype != 0 && sim.parts[i].life == 0 {
        let vx = ((sim.parts[i].tmp << 16) >> 16) as f32 / 255.0;
        let vy = (sim.parts[i].tmp >> 16) as f32 / 255.0;
        let dx = ((sim.parts[i].tmp2 << 16) >> 16) as f32 / 255.0;
        let dy = (sim.parts[i].tmp2 >> 16) as f32 / 255.0;
        if let Some(p) = create_near(sim, x, y, pt::PHOT) {
            sim.parts[p].ctype = sim.parts[i].ctype;
            sim.parts[p].x += dx;
            sim.parts[p].y += dy;
            sim.parts[p].vx = vx;
            sim.parts[p].vy = vy;
            sim.parts[p].temp = sim.parts[i].temp;
        }
        sim.parts[i].ctype = 0;
        sim.parts[i].tmp = 0;
        sim.parts[i].tmp2 = 0;
    }
    0
}

// --- TRON ----------------------------------------------------------------
const TRON_RX: [i32; 4] = [-1, 0, 1, 0];
const TRON_RY: [i32; 4] = [0, -1, 0, 1];
const TRON_HEAD: i32 = 0x0001;
const TRON_NOGROW: i32 = 0x0002;
const TRON_WAIT: i32 = 0x0004;
const TRON_NODIE: i32 = 0x0008;
const TRON_DEATH: i32 = 0x0010;
const TRON_NORANDOM: i32 = 0x0001_0000;

fn tron_can_move(sim: &Sim, r: u32, len: i32) -> bool {
    let rt = pmap_type(r);
    let id = pmap_id(r) as usize;
    if r == 0
        || (rt == pt::SWCH && sim.parts[id].life >= 10)
        || (rt == pt::INVIS && sim.parts[id].tmp2 == 1)
    {
        return true;
    }
    let p = element(rt).properties;
    if (((p & props::PROP_LIFE_KILL_DEC) != 0 && sim.parts[id].life > 0)
        || ((p & props::PROP_LIFE_KILL) != 0 && (p & props::PROP_LIFE_DEC) != 0))
        && sim.parts[id].life < len
    {
        return true;
    }
    false
}

fn tron_try_move(sim: &Sim, x: i32, y: i32, dir: i32, len: i32) -> i32 {
    let mut count = 0;
    let (mut rx, mut ry) = (x, y);
    for k in 1..=len {
        rx += TRON_RX[dir as usize];
        ry += TRON_RY[dir as usize];
        if !sim.in_bounds(rx, ry) {
            break;
        }
        let r = sim.pmap[sim.idx(rx, ry)];
        let wall = sim.wall_at(rx, ry);
        if tron_can_move(sim, r, k - 1)
            && wall == 0
            && ry >= CELL as i32 && rx >= CELL as i32
            && ry < sim.height as i32 - CELL as i32
            && rx < sim.width as i32 - CELL as i32
        {
            count += 1;
            let mut tx = rx - TRON_RY[dir as usize];
            let mut ty = ry - TRON_RX[dir as usize];
            let mut j = 1;
            while (tx - rx).abs() < (len - k) && (ty - ry).abs() < (len - k) {
                if !sim.in_bounds(tx, ty) {
                    break;
                }
                let rr = sim.pmap[sim.idx(tx, ty)];
                if tron_can_move(sim, rr, j + k - 1) && sim.wall_at(tx, ty) == 0 {
                    if j == len - k {
                        return len + 1;
                    }
                    count += 1;
                } else {
                    break;
                }
                tx -= TRON_RY[dir as usize];
                ty -= TRON_RX[dir as usize];
                j += 1;
            }
            let mut tx = rx + TRON_RY[dir as usize];
            let mut ty = ry + TRON_RX[dir as usize];
            let mut j = 1;
            while (tx - rx).abs() < (len - k) && (ty - ry).abs() < (len - k) {
                if !sim.in_bounds(tx, ty) {
                    break;
                }
                let rr = sim.pmap[sim.idx(tx, ty)];
                if tron_can_move(sim, rr, j + k - 1) && sim.wall_at(tx, ty) == 0 {
                    if j == len - k {
                        return len + 1;
                    }
                    count += 1;
                } else {
                    break;
                }
                tx += TRON_RY[dir as usize];
                ty += TRON_RX[dir as usize];
                j += 1;
            }
        } else {
            break;
        }
    }
    count
}

fn tron_new_head(sim: &mut Sim, x: i32, y: i32, i: usize, direction: i32) -> i32 {
    if !cell_free(sim, x, y) {
        return -1;
    }
    let props = sim.parts[i].tmp;
    let mut tail_len = sim.parts[i].tmp2;
    let mut life = sim.parts[i].life;
    if let Some(np) = sim.create_part(x, y, pt::TRON) {
        if life >= 100 {
            if props & TRON_NOGROW == 0 {
                tail_len += 1;
            }
            life = 5;
        }
        let hue = props & 0xF800;
        let add = props & (TRON_NOGROW | TRON_NODIE | TRON_NORANDOM);
        let mut pt = TRON_HEAD | (direction << 5) | add | hue;
        if np > i {
            pt |= TRON_WAIT;
        }
        sim.parts[np].tmp = pt;
        sim.parts[np].ctype = sim.parts[i].ctype;
        sim.parts[np].tmp2 = tail_len;
        sim.parts[np].life = life + 2;
        1
    } else {
        -1
    }
}

fn tron_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.parts[i].tmp & TRON_WAIT != 0 {
        sim.parts[i].tmp &= !TRON_WAIT;
        return 0;
    }
    if sim.parts[i].tmp & TRON_HEAD != 0 {
        let mut direction = (sim.parts[i].tmp >> 5) & 0x3;
        let original = direction;
        let random = sim.rng.between(0, 339);
        if (random == 1 || random == 3) && sim.parts[i].tmp & TRON_NORANDOM == 0 {
            direction = (direction + random) % 4;
        }
        let tmp2 = sim.parts[i].tmp2;
        let first = tron_try_move(sim, x, y, direction, tmp2);
        let mut seconddir = direction;
        let mut lastdir = direction;
        let mut second = 0;
        let mut last = 0;
        if first < tmp2 {
            if sim.parts[i].tmp & TRON_NORANDOM != 0 {
                seconddir = (direction + 1) % 4;
                lastdir = (direction + 3) % 4;
            } else if original != direction {
                seconddir = original;
                lastdir = (direction + 2) % 4;
            } else {
                seconddir = (direction + (sim.rng.between(0, 1) * 2) + 1) % 4;
                lastdir = (seconddir + 2) % 4;
            }
            second = tron_try_move(sim, x, y, seconddir, tmp2);
            last = tron_try_move(sim, x, y, lastdir, tmp2);
        }
        if second > first {
            direction = seconddir;
        }
        if last > second && last > first {
            direction = lastdir;
        }
        if tron_new_head(sim, x + TRON_RX[direction as usize], y + TRON_RY[direction as usize], i, direction) == -1 {
            sim.parts[i].tmp |= TRON_DEATH;
        }
        sim.parts[i].life = tmp2;
        sim.parts[i].tmp &= 0xF818;
    } else if sim.parts[i].tmp & TRON_NODIE != 0 {
        sim.parts[i].life += 1;
    }
    0
}

fn tron_create(sim: &mut Sim, i: usize, _x: i32, _y: i32, _t: i32, _v: i32) {
    let randhue = sim.rng.between(0, 359);
    let randomdir = sim.rng.between(0, 3);
    sim.parts[i].tmp = 1 | (randomdir << 5) | (randhue << 7);
    sim.parts[i].tmp2 = 4;
    sim.parts[i].life = 5;
}

// --- PTNM ----------------------------------------------------------------
fn ptnm_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.parts[i].life == 0 {
        for (rx, ry) in [(-4, 0), (4, 0), (0, -4), (0, 4)] {
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
            if pmap_type(r) == pt::SPRK && sim.parts[id].life != 0 && sim.parts[id].life < 4 {
                sim.part_change_type(i, x, y, pt::SPRK as i16);
                sim.parts[i].life = 4;
                sim.parts[i].ctype = pt::PTNM as i32;
            }
        }
    }
    let mut hygn1_id: i32 = -1;
    let mut wtrv1_id: i32 = -1;
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
            let rt = pmap_type(r);
            if rt == pt::H2 && hygn1_id < 0 {
                hygn1_id = id as i32;
            }
            if rt == pt::WTRV && wtrv1_id < 0 {
                wtrv1_id = id as i32;
            }
            if sim.parts[id].life == 0 && matches!(rt, pt::SHLD1 | pt::SHLD2 | pt::SHLD3) {
                let next = match rt {
                    pt::SHLD1 => pt::SHLD2,
                    pt::SHLD2 => pt::SHLD3,
                    _ => pt::SHLD4,
                };
                sim.part_change_type(id, nx, ny, next as i16);
                sim.parts[id].life = 7;
                continue;
            }
            if rt == pt::ISZS || rt == pt::ISOZ {
                sim.part_change_type(id, nx, ny, pt::PLUT as i16);
                create_near(sim, nx, ny, pt::PHOT);
                continue;
            }
            let prob = (sim.parts[i].temp / (273.15 + 1500.0)).min(1.0);
            let prob = prob * prob;
            if sim.rng.uniform01() <= prob {
                match rt {
                    pt::GAS => {
                        if sim.parts[id].temp >= 473.15 && pv(sim, nx, ny) > 2.0 {
                            sim.part_change_type(id, nx, ny, pt::INSL as i16);
                            sim.parts[i].temp += 60.0;
                        }
                    }
                    pt::BREC => {
                        if sim.parts[id].temp > 1273.15 && pv(sim, nx, ny) > 50.0 {
                            sim.part_change_type(id, nx, ny, pt::EXOT as i16);
                            sim.parts[id].temp -= 30.0;
                            sim.parts[i].temp -= 30.0;
                        }
                    }
                    pt::SMKE => {
                        sim.part_change_type(id, nx, ny, pt::CO2 as i16);
                    }
                    pt::RSST => {
                        sim.part_change_type(id, nx, ny, pt::BIZR as i16);
                    }
                    _ => {}
                }
            }
        }
    }
    if hygn1_id >= 0 {
        let hid = hygn1_id as usize;
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
                if r == 0 || pmap_id(r) as usize == hid {
                    continue;
                }
                let id = pmap_id(r) as usize;
                let rt = pmap_type(r);
                if rt == pt::DESL {
                    sim.part_change_type(id, nx, ny, pt::WATR as i16);
                    sim.part_change_type(hid, (sim.parts[hid].x + 0.5) as i32, (sim.parts[hid].y + 0.5) as i32, pt::OIL as i16);
                    return 0;
                }
                if rt == pt::O2 && sim.parts[i].life == 0 {
                    sim.part_change_type(id, nx, ny, pt::DSTW as i16);
                    sim.part_change_type(hid, (sim.parts[hid].x + 0.5) as i32, (sim.parts[hid].y + 0.5) as i32, pt::DSTW as i16);
                    sim.parts[id].temp += 5.0;
                    sim.parts[hid].temp += 5.0;
                    sim.parts[i].ctype = pt::PTNM as i32;
                    sim.parts[i].life = 4;
                    sim.part_change_type(i, x, y, pt::SPRK as i16);
                    return 0;
                }
                if rt == pt::H2
                    && sim.rng.chance(1, 1000)
                    && sim.parts[id].temp > 773.15
                    && sim.parts[hid].temp > 773.15
                {
                    sim.part_change_type(id, nx, ny, pt::NBLE as i16);
                    let hx = (sim.parts[hid].x + 0.5) as i32;
                    let hy = (sim.parts[hid].y + 0.5) as i32;
                    sim.part_change_type(hid, hx, hy, pt::NEUT as i16);
                    sim.parts[id].temp += 1000.0;
                    sim.parts[hid].temp += 1000.0;
                    if sim.in_cell_bounds(nx / CELL as i32, ny / CELL as i32) {
                        let vc = sim.cell_idx(nx / CELL as i32, ny / CELL as i32);
                        sim.pv[vc] += 10.0;
                    }
                    if let Some(j) = create_near(sim, nx, ny, pt::PHOT) {
                        sim.parts[j].ctype = 0x7C0000;
                        sim.parts[j].temp = sim.parts[id].temp;
                        sim.parts[j].tmp = 0x1;
                    }
                    if sim.rng.chance(1, 10) {
                        if let Some(j) = create_near(sim, nx, ny, pt::ELEC) {
                            sim.parts[j].temp = sim.parts[id].temp;
                        }
                    }
                    return 0;
                }
            }
        }
    }
    if wtrv1_id >= 0 {
        let wid = wtrv1_id as usize;
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
                if r == 0 || pmap_id(r) as usize == wid {
                    continue;
                }
                let id = pmap_id(r) as usize;
                if pmap_type(r) == pt::BCOL
                    && sim.parts[id].temp > 473.15
                    && sim.parts[wid].temp > 473.15
                    && pv(sim, nx, ny) > 7.0
                {
                    sim.part_change_type(id, nx, ny, pt::OIL as i16);
                    sim.kill_part(wid);
                    return 0;
                }
            }
        }
    }
    0
}

fn ptnm_create(sim: &mut Sim, i: usize, _x: i32, _y: i32, _t: i32, _v: i32) {
    if sim.rng.chance(1, 15) {
        sim.parts[i].tmp = 1;
    }
}

// --- LITH ----------------------------------------------------------------
fn lith_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.parts[i].ctype < 0 {
        sim.parts[i].ctype = 0;
    }
    let mut charged = false;
    let mut discharged = false;
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
                if sim.parts[i].life > 1012 && sim.rng.chance(1, 10) {
                    sim.create_part(nx, ny, pt::FIRE);
                }
                continue;
            }
            let id = pmap_id(r) as usize;
            let rt = pmap_type(r);
            let pavg = parts_avg(sim, i, id, pt::INSL);
            match rt {
                pt::SLTW | pt::WTRV | pt::WATR | pt::DSTW | pt::CBNW => {
                    if sim.parts[i].life > 1016 {
                        sim.part_change_type(id, nx, ny, pt::WTRV as i16);
                        sim.parts[id].temp = 440.0;
                        continue;
                    }
                    if sim.parts[i].tmp + sim.parts[i].tmp2 >= 10 {
                        continue;
                    }
                    sim.part_change_type(id, nx, ny, pt::H2 as i16);
                    if sim.parts[i].temp > 440.0 {
                        let stored = sim.parts[i].ctype.min(24);
                        sim.parts[i].life = 1024 + if stored > 24 { 96 } else { stored * 4 };
                        sim.parts[i].tmp = 10;
                    } else {
                        let se = sim.parts[i].ctype as f32;
                        sim.parts[i].temp = restrict_flt(sim.parts[i].temp + 20.365 + se * se * 1.5, MIN_TEMP, MAX_TEMP);
                        sim.parts[i].tmp += 1;
                    }
                }
                pt::CO2 => {
                    if sim.parts[i].tmp + sim.parts[i].tmp2 >= 10 {
                        continue;
                    }
                    sim.kill_part(id);
                    sim.parts[i].tmp2 += 1;
                }
                pt::SPRK => {
                    if pavg == pt::INSL || pavg == pt::RSSS {
                        continue;
                    }
                    if sim.parts[i].tmp + sim.parts[i].tmp2 >= 5 {
                        continue;
                    }
                    if sim.parts[id].ctype == pt::PSCN as i32
                        && sim.parts[id].life == 3
                        && !charged
                        && sim.parts[i].life == 0
                    {
                        charged = true;
                    }
                }
                pt::NSCN => {
                    if pavg == pt::INSL || pavg == pt::RSSS {
                        continue;
                    }
                    if sim.parts[id].life == 0 && sim.parts[i].ctype > 0 && sim.parts[i].life == 0 {
                        sim.part_change_type(id, nx, ny, pt::SPRK as i16);
                        sim.parts[id].life = 4;
                        sim.parts[id].ctype = pt::NSCN as i32;
                        discharged = true;
                    }
                }
                pt::FIRE => {
                    if sim.parts[i].temp > 440.0 && sim.rng.chance(1, 40) && sim.parts[i].tmp < 6 {
                        sim.parts[i].life = 1013;
                        sim.parts[i].tmp += 1;
                    }
                }
                pt::O2 => {
                    if sim.parts[i].life > 1000 && sim.rng.chance(1, 10) {
                        sim.part_change_type(i, x, y, pt::PLSM as i16);
                        sim.part_change_type(id, nx, ny, pt::PLSM as i16);
                        if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
                            let vc = sim.cell_idx(x / CELL as i32, y / CELL as i32);
                            sim.pv[vc] += 4.0;
                        }
                        return 0;
                    }
                }
                _ => {}
            }
        }
    }
    if charged {
        sim.parts[i].ctype += 1;
        sim.parts[i].life = 8;
    }
    if discharged {
        sim.parts[i].ctype -= 1;
        sim.parts[i].life = 8;
    }
    for _ in 0..9 {
        let rx = sim.rng.between(-3, 3);
        let ry = sim.rng.between(-3, 3);
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
        if pmap_type(r) != pt::LITH {
            continue;
        }
        if sim.parts[i].life < 1000 && sim.parts[i].ctype > 90 && sim.parts[id].life > 1000 {
            sim.parts[i].life = 1024;
        }
        if sim.parts[i].ctype > sim.parts[id].ctype {
            let mut transfer = sim.parts[i].ctype - sim.parts[id].ctype;
            transfer -= transfer / 2;
            sim.parts[id].ctype += transfer;
            sim.parts[i].ctype -= transfer;
            break;
        }
    }
    if sim.parts[i].life < 1000 && sim.parts[i].ctype >= 100 {
        sim.parts[i].life = 1024;
    }
    if sim.parts[i].life == 1000 {
        sim.parts[i].life = 0;
        sim.part_change_type(i, x, y, pt::LAVA as i16);
        if sim.parts[i].tmp2 < 3 {
            let se = sim.parts[i].ctype as f32;
            sim.parts[i].temp = restrict_flt(500.0 + se * 10.0, MIN_TEMP, MAX_TEMP);
            sim.parts[i].ctype = pt::LITH as i32;
        } else {
            let se = sim.parts[i].ctype as f32;
            sim.parts[i].temp = restrict_flt(2000.0 + se * 10.0, MIN_TEMP, MAX_TEMP);
            sim.parts[i].ctype = pt::GLAS as i32;
        }
    }
    if sim.parts[i].life >= 1024 && sim.parts[i].life != 1024 {
        sim.parts[i].life = 1024;
    }
    // Advance the burn timer toward the explosion threshold.
    if sim.parts[i].life >= 1013 && sim.parts[i].life < 1024 {
        sim.parts[i].life += 1;
        if sim.parts[i].life >= 1024 {
            sim.parts[i].life = 1000;
        }
    }
    0
}

// --- BASE ----------------------------------------------------------------
fn base_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    sim.parts[i].tmp = 0;
    if sim.parts[i].life < 1 {
        sim.parts[i].life = 1;
    }
    if sim.parts[i].life > 100 {
        sim.parts[i].life = 100;
    }
    let pres = pv(sim, x, y);
    if sim.parts[i].life < 100 && pres < 10.0 && sim.parts[i].temp > 393.15 && sim.rng.chance(1, 20) {
        if sim.rng.chance(1, (sim.parts[i].life + 1) as u32) {
            sim.part_change_type(i, x, y, pt::BOYL as i16);
            return 1;
        } else {
            let life = sim.parts[i].life;
            sim.parts[i].life += 1;
            sim.parts[i].temp -= 20.0 / life as f32;
        }
    }
    if sim.parts[i].temp < 273.15 - sim.parts[i].life as f32 / 4.0 {
        sim.part_change_type(i, x, y, pt::ICEI as i16);
        sim.parts[i].ctype = pt::BASE as i32;
        sim.parts[i].life = 0;
        return 1;
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
            let rt = pmap_type(r);
            let skip = matches!(
                rt,
                pt::BASE | pt::SALT | pt::SLTW | pt::BOYL | pt::MERC | pt::BMTL | pt::BRMT | pt::SOAP
                    | pt::CLNE | pt::PCLN
            ) || (rt == pt::ICEI && sim.parts[id].ctype == pt::BASE as i32)
                || (rt == pt::SNOW && sim.parts[id].ctype == pt::BASE as i32)
                || (rt == pt::SPRK && sim.parts[id].ctype == pt::BMTL as i32)
                || (rt == pt::SPRK && sim.parts[id].ctype == pt::BRMT as i32);
            if skip {
                continue;
            }
            if sim.parts[i].life > 1 && matches!(rt, pt::WATR | pt::DSTW | pt::CBNW) {
                if sim.rng.chance(1, 20) {
                    let saturh = sim.parts[i].life / 2;
                    sim.part_change_type(id, nx, ny, pt::BASE as i16);
                    sim.parts[id].life = saturh;
                    sim.parts[id].temp += saturh as f32 / 10.0;
                    sim.parts[i].life -= saturh;
                }
            } else if rt == pt::ACID {
                if sim.parts[id].life > 50 && sim.parts[i].life > 0 {
                    sim.parts[id].life -= 1;
                    sim.parts[i].life -= 1;
                }
                if sim.parts[id].life <= 50 {
                    sim.part_change_type(id, nx, ny, pt::SLTW as i16);
                }
                if sim.parts[i].life <= 0 {
                    sim.part_change_type(i, x, y, pt::SLTW as i16);
                    return 1;
                }
            } else if rt == pt::CAUS {
                if sim.parts[id].life > 50 && sim.parts[i].life > 0 {
                    sim.parts[id].life -= 1;
                    sim.parts[i].life -= 1;
                }
                if sim.parts[id].life <= 50 {
                    sim.kill_part(id);
                }
                if sim.parts[i].life <= 0 {
                    sim.part_change_type(i, x, y, pt::SLTW as i16);
                    return 1;
                }
            } else if sim.parts[i].life >= 70 && rt == pt::OIL {
                sim.part_change_type(i, x, y, pt::SOAP as i16);
                sim.kill_part(id);
                return 1;
            } else if sim.parts[i].life > 1 && rt == pt::GOO {
                sim.part_change_type(id, nx, ny, pt::GEL as i16);
                sim.parts[i].life -= 1;
            } else if sim.parts[i].life > 1 && rt == pt::BCOL {
                sim.part_change_type(id, nx, ny, pt::GUNP as i16);
                sim.parts[i].life -= 1;
            } else if rt == pt::LAVA && sim.parts[id].ctype == pt::ROCK as i32 && pres >= 10.0 && sim.rng.chance(1, 1000) {
                sim.part_change_type(i, x, y, pt::MERC as i16);
                sim.kill_part(id);
                return 1;
            } else if sim.parts[i].life >= 10
                && (element(rt).properties & (props::TYPE_SOLID | props::PROP_CONDUCTS))
                    == (props::TYPE_SOLID | props::PROP_CONDUCTS)
                && sim.rng.chance(1, 10)
            {
                sim.part_change_type(id, nx, ny, pt::BMTL as i16);
                sim.parts[id].tmp = sim.rng.between(20, 29);
                sim.parts[i].life -= 1;
                sim.parts[i].tmp = 1;
            } else if element(rt).hardness > 0
                && element(rt).hardness < 50
                && sim.parts[i].life >= 2 * element(rt).hardness
                && sim.rng.chance(50 - element(rt).hardness, 1000)
            {
                sim.kill_part(id);
                sim.parts[i].life -= 2;
                sim.parts[i].tmp = 1;
            }
        }
    }
    for _ in 0..2 {
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
        let id = pmap_id(r) as usize;
        if pmap_type(r) == pt::BASE && sim.parts[i].life > sim.parts[id].life && sim.parts[i].life > 1 {
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

// --- DRAY ----------------------------------------------------------------
fn dray_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    // Simplified: DRAY copies the first particle it sees in a straight line when
    // sparked. Full overwrite/energy semantics match the reference closely.
    let ctype = sim.parts[i].ctype & 0x1FF;
    let copy_len = sim.parts[i].tmp;
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
            if pmap_type(r) == pt::SPRK && sim.parts[id].life == 3 {
                let overwrite = sim.parts[id].ctype == pt::PSCN as i32;
                let mut xc = x - rx;
                let mut yc = y - ry;
                let mut found = false;
                let mut remaining = if copy_len > 0 { copy_len } else { 0 };
                while sim.in_bounds(xc, yc) {
                    let rr = sim.pmap[sim.idx(xc, yc)];
                    if rr != 0 {
                        found = true;
                        break;
                    }
                    if copy_len > 0 {
                        remaining -= 1;
                        if remaining <= 0 {
                            break;
                        }
                    }
                    xc -= rx;
                    yc -= ry;
                }
                if found {
                    let src = sim.pmap[sim.idx(xc, yc)];
                    let st = pmap_type(src);
                    if st != 0 && st != pt::SPRK && (st as i32) == ctype || (ctype == 0 && st != 0) {
                        let tx = x + rx;
                        let ty = y + ry;
                        if overwrite && !cell_free(sim, tx, ty) {
                            if let Some(old) = sim.id_at(tx, ty) {
                                sim.kill_part(old);
                            }
                        }
                        if cell_free(sim, tx, ty) {
                            sim.create_part(tx, ty, st);
                        }
                    }
                }
            }
        }
    }
    0
}

// --- SEED (simplified growth model) --------------------------------------
fn seed_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let temp = sim.parts[i].temp;
    if temp > 320.0 && pv(sim, x, y) > 50.0 && sim.rng.chance(1, 150) {
        sim.part_change_type(i, x, y, pt::MWAX as i16);
        sim.parts[i].ctype = 0;
        sim.parts[i].life = 0;
        sim.parts[i].tmp = 0;
        sim.parts[i].tmp2 = 0;
        sim.parts[i].tmp3 = 0;
        sim.parts[i].tmp4 = 0;
        return 0;
    }
    if !(278.15..=343.15).contains(&temp) {
        sim.parts[i].life = 0;
        return 0;
    }
    let mut water = sim.parts[i].ctype;
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
            match pmap_type(r) {
                pt::WATR => {
                    if water < 31 {
                        sim.kill_part(id);
                        water += 1;
                        sim.parts[i].life = 0;
                    }
                }
                pt::DEUT => {
                    if water < 255 {
                        sim.kill_part(id);
                        water += 1;
                        sim.parts[i].life = 0;
                    }
                }
                pt::CBNW | pt::DSTW => {
                    if water < 3 {
                        sim.kill_part(id);
                        water += 1;
                        sim.parts[i].life = 0;
                    }
                }
                pt::SLTW => {
                    if water > 0 {
                        sim.kill_part(id);
                        water -= 1;
                        sim.parts[i].life = 0;
                    }
                }
                pt::SPNG => {
                    if water < 15 && sim.parts[id].life > 0 {
                        sim.parts[id].life -= 1;
                        water += 1;
                        sim.parts[i].life = 0;
                    }
                }
                _ => {}
            }
        }
    }
    sim.parts[i].ctype = water;
    if water > 3 {
        let below = sim.type_at(x, y + 1);
        let above_free = cell_free(sim, x, y - 1);
        if matches!(below, pt::SAND | pt::SPNG) && above_free {
            if sim.parts[i].life > 200 {
                sim.part_change_type(i, x, y, pt::PLNT as i16);
                sim.parts[i].life = 15 * water;
                return 1;
            } else {
                sim.parts[i].life += 1;
            }
        } else {
            sim.parts[i].life = 0;
        }
    }
    0
}

// ===========================================================================
// Batch 4c: portals, wireless, rays, pistons, stickmen fallbacks
// ===========================================================================

// Portal payloads now live in `Sim::portal` (world layer). This keeps them
// serialisable and independent of particle iteration order.
use crate::portal::{PortalPayload, PORTAL_CHANNELS};

const PORTAL_RX: [i32; 8] = [-1, 0, 1, 1, 1, 0, -1, -1];
const PORTAL_RY: [i32; 8] = [-1, -1, -1, 0, 1, 1, 1, 0];

fn portal_channel(sim: &Sim, i: usize) -> usize {
    let t = ((sim.parts[i].temp - 73.15) / 100.0 + 1.0) as i32;
    t.clamp(0, PORTAL_CHANNELS as i32 - 1) as usize
}

// --- PRTI ----------------------------------------------------------------
fn prti_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let ch = portal_channel(sim, i);
    sim.parts[i].tmp = ch as i32;
    for count in 0..8 {
        let rx = PORTAL_RX[count];
        let ry = PORTAL_RY[count];
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
        let p = element(rt).properties;
        if !((p & (props::TYPE_PART | props::TYPE_LIQUID | props::TYPE_GAS | props::TYPE_ENERGY)) != 0
            || rt == pt::SPRK
            || rt == pt::STOR)
        {
            continue;
        }
        if matches!(rt, pt::STKM | pt::STKM2 | pt::FIGH) {
            continue;
        }
        let payload = PortalPayload::from_particle(&sim.parts[id]);
        if sim.portal.store(ch, count, payload) {
            if rt == pt::SPRK {
                let ct = sim.parts[id].ctype;
                sim.part_change_type(id, nx, ny, ct as i16);
            } else {
                sim.kill_part(id);
            }
        }
    }
    0
}

// --- PRTO ----------------------------------------------------------------
fn prto_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let ch = portal_channel(sim, i);
    sim.parts[i].tmp = ch as i32;
    for count in 0..8 {
        let rx = PORTAL_RX[count];
        let ry = PORTAL_RY[count];
        let nx = x + rx;
        let ny = y + ry;
        if !sim.in_bounds(nx, ny) {
            continue;
        }
        if !cell_free(sim, nx, ny) {
            continue;
        }
        let random = ((count as i32 + sim.rng.between(-1, 1) + 4) % 8) as usize;
        if let Some(p) = sim.portal.take(ch, random) {
            if let Some(np) = sim.create_part(nx, ny, p.t) {
                p.apply_to(&mut sim.parts[np], true);
                sim.parts[np].x = nx as f32;
                sim.parts[np].y = ny as f32;
            }
            break;
        }
    }
    0
}

// --- PIPE / PPIP ---------------------------------------------------------
// Faithful port of `PIPE.cpp` / `PPIP.cpp`. A pipe stores one particle in
// `ctype` (type), `temp`, `tmp2` (life), `tmp3` (tmp), `tmp4` (ctype). The
// low `tmp` bits hold the direction/color flags used to route the payload.
const PIPE_OFFSETS: [(i32, i32); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];
const PFLAG_CAN_CONDUCT: i32 = 0x0000_0001;
const PFLAG_PARTICLE_DECO: i32 = 0x0000_0002;
const PFLAG_NORMALSPEED: i32 = 0x0001_0000;
const PFLAG_INITIALIZING: i32 = 0x0002_0000;
const PFLAG_COLOR_RED: i32 = 0x0004_0000;
const PFLAG_COLOR_GREEN: i32 = 0x0008_0000;
const PFLAG_COLOR_BLUE: i32 = 0x000C_0000;
const PFLAG_COLORS: i32 = 0x000C_0000;
const PPIP_REVERSED: i32 = 0x0100_0000;
const PPIP_PAUSED: i32 = 0x0200_0000;
const PPIP_TRIGGER_REVERSE: i32 = 0x0400_0000;
const PPIP_TRIGGER_OFF: i32 = 0x0800_0000;
const PPIP_TRIGGER_ON: i32 = 0x1000_0000;
const PPIP_TRIGGERS: i32 = 0x1C00_0000;
/// Direction of travel stored in the pipe's free bits 20..22 (index into
/// `PIPE_OFFSETS`). Kept away from the reference flag bits.
const PIPE_DIR_MASK: i32 = 0x0070_0000;
const PIPE_DIR_SHIFT: i32 = 20;
const PIPE_OPPOSITE: [usize; 8] = [7, 6, 5, 4, 3, 2, 1, 0];

fn pipe_dir(flags: i32) -> usize {
    ((flags & PIPE_DIR_MASK) >> PIPE_DIR_SHIFT) as usize & 7
}

fn pipe_set_dir(flags: &mut i32, dir: usize) {
    *flags = (*flags & !PIPE_DIR_MASK) | (((dir & 7) as i32) << PIPE_DIR_SHIFT);
}

fn pipe_next_color(flags: i32) -> i32 {
    match flags & PFLAG_COLORS {
        PFLAG_COLOR_RED => PFLAG_COLOR_BLUE,
        PFLAG_COLOR_BLUE => PFLAG_COLOR_GREEN,
        PFLAG_COLOR_GREEN => PFLAG_COLOR_RED,
        _ => PFLAG_COLOR_GREEN,
    }
}

fn pipe_prev_color(flags: i32) -> i32 {
    match flags & PFLAG_COLORS {
        PFLAG_COLOR_RED => PFLAG_COLOR_GREEN,
        PFLAG_COLOR_GREEN => PFLAG_COLOR_BLUE,
        PFLAG_COLOR_BLUE => PFLAG_COLOR_RED,
        _ => PFLAG_COLOR_RED,
    }
}

/// Copy the stored payload of `pipe` onto a freshly created particle.
fn pipe_to_part(sim: &mut Sim, pipe: usize, part: usize) {
    let t = sim.parts[pipe].ctype;
    if t <= 0 || (t as usize) >= crate::elements_generated::PT_NUM {
        return;
    }
    let temp = sim.parts[pipe].temp;
    let life = sim.parts[pipe].tmp2;
    let tmp = sim.parts[pipe].tmp3;
    let ctype = sim.parts[pipe].tmp4;
    let deco = sim.parts[pipe].dcolour;
    let particle_deco = sim.parts[pipe].tmp & PFLAG_PARTICLE_DECO != 0;
    sim.parts[pipe].ctype = 0;
    sim.parts[part].temp = temp;
    sim.parts[part].life = life;
    sim.parts[part].tmp = tmp;
    sim.parts[part].ctype = ctype;
    if particle_deco && deco != 0 {
        sim.parts[part].dcolour = deco;
        sim.parts[pipe].dcolour = 0;
        sim.parts[pipe].tmp &= !PFLAG_PARTICLE_DECO;
    }
    let def = element(t as u16);
    if (def.properties & props::TYPE_ENERGY) == 0 {
        sim.parts[part].vx = 0.0;
        sim.parts[part].vy = 0.0;
    }
}

/// Store a live particle into a pipe, preserving type/life/tmp/ctype/temp.
fn part_to_pipe(sim: &mut Sim, part: usize, pipe: usize) {
    let t = sim.parts[part].t;
    if t == 0 {
        return;
    }
    let temp = sim.parts[part].temp;
    let life = sim.parts[part].life;
    let tmp = sim.parts[part].tmp;
    let ctype = sim.parts[part].ctype;
    let deco = sim.parts[part].dcolour;
    sim.parts[pipe].ctype = t as i32;
    sim.parts[pipe].temp = temp;
    sim.parts[pipe].tmp2 = life;
    sim.parts[pipe].tmp3 = tmp;
    sim.parts[pipe].tmp4 = ctype;
    if deco != 0 && sim.parts[pipe].dcolour == 0 {
        sim.parts[pipe].dcolour = deco;
        sim.parts[pipe].tmp |= PFLAG_PARTICLE_DECO;
    }
}

/// Transfer a stored payload between adjacent pipes.
fn pipe_to_pipe(sim: &mut Sim, src: usize, dest: usize) {
    let ctype = sim.parts[src].ctype;
    let temp = sim.parts[src].temp;
    let life = sim.parts[src].tmp2;
    let tmp = sim.parts[src].tmp3;
    let ctype2 = sim.parts[src].tmp4;
    let deco = sim.parts[src].dcolour;
    let particle_deco = sim.parts[src].tmp & PFLAG_PARTICLE_DECO != 0;
    sim.parts[src].ctype = 0;
    sim.parts[dest].ctype = ctype;
    sim.parts[dest].temp = temp;
    sim.parts[dest].tmp2 = life;
    sim.parts[dest].tmp3 = tmp;
    sim.parts[dest].tmp4 = ctype2;
    if particle_deco && deco != 0 {
        if sim.parts[dest].dcolour == 0 {
            sim.parts[dest].dcolour = deco;
            sim.parts[dest].tmp |= PFLAG_PARTICLE_DECO;
        }
        sim.parts[src].dcolour = 0;
        sim.parts[src].tmp &= !PFLAG_PARTICLE_DECO;
    }
}

/// Reference `PIPE::pushParticle` (depth limited to 2 hops per frame), with a
/// deterministic travel direction stored in free `tmp` bits so a straight pipe
/// does not oscillate among identically coloured neighbours.
fn pipe_push(sim: &mut Sim, i: usize, count: i32, original: usize) {
    if count >= 2 || sim.parts[i].ctype == 0 {
        return;
    }
    let notctype = pipe_next_color(sim.parts[i].tmp);
    let x = (sim.parts[i].x + 0.5) as i32;
    let y = (sim.parts[i].y + 0.5) as i32;
    let single = sim.parts[i].tmp & 0x200 != 0;
    let dir = pipe_dir(sim.parts[i].tmp);
    if single {
        let coords = 7 - ((sim.parts[i].tmp >> 10) & 7);
        let (rx, ry) = PIPE_OFFSETS[(coords & 7) as usize];
        let nx = x + rx;
        let ny = y + ry;
        if sim.in_bounds(nx, ny) {
            let r = sim.pmap_at(nx, ny);
            if r != 0 {
                let id = pmap_id(r) as usize;
                let rt = pmap_type(r);
                if (rt == pt::PIPE || rt == pt::PPIP)
                    && (sim.parts[id].tmp & PFLAG_COLORS) != notctype
                    && sim.parts[id].ctype == 0
                {
                    pipe_to_pipe(sim, i, id);
                    let d = PIPE_OPPOSITE[(coords & 7) as usize];
                    pipe_set_dir(&mut sim.parts[id].tmp, d);
                    if id > original {
                        sim.parts[id].flags |= PFLAG_NORMALSPEED as u32;
                    }
                    pipe_push(sim, id, count + 1, original);
                }
            }
        }
        return;
    }
    // Prefer continuing straight, then small turns, then any eligible neighbour.
    let mut order = [dir; 8];
    for k in 0..8usize {
        order[k] = match k {
            0 => dir,
            1 => (dir + 1) & 7,
            2 => (dir + 7) & 7,
            3 => (dir + 2) & 7,
            4 => (dir + 6) & 7,
            5 => (dir + 3) & 7,
            6 => (dir + 5) & 7,
            _ => dir ^ 4,
        };
    }
    for idx in order {
        let (rx, ry) = PIPE_OFFSETS[idx];
        let nx = x + rx;
        let ny = y + ry;
        if !sim.in_bounds(nx, ny) {
            continue;
        }
        let r = sim.pmap_at(nx, ny);
        if r == 0 {
            continue;
        }
        let id = pmap_id(r) as usize;
        let rt = pmap_type(r);
        if (rt == pt::PIPE || rt == pt::PPIP)
            && (sim.parts[id].tmp & PFLAG_COLORS) != notctype
            && sim.parts[id].ctype == 0
        {
            pipe_to_pipe(sim, i, id);
            let d = PIPE_OPPOSITE[idx];
            pipe_set_dir(&mut sim.parts[id].tmp, d);
            if id > original {
                sim.parts[id].flags |= PFLAG_NORMALSPEED as u32;
            }
            pipe_push(sim, id, count + 1, original);
            return;
        } else if rt == pt::PRTI {
            let ch = portal_channel(sim, id);
            let dir = count.max(0) as usize % crate::portal::PORTAL_DIRECTIONS;
            let payload = crate::portal::PortalPayload {
                t: sim.parts[i].ctype as u16,
                life: sim.parts[i].tmp2,
                ctype: sim.parts[i].tmp4,
                x: nx as f32,
                y: ny as f32,
                vx: 0.0,
                vy: 0.0,
                temp: sim.parts[i].temp,
                tmp: sim.parts[i].tmp3,
                tmp2: 0,
                tmp3: 0,
                tmp4: 0,
                flags: 0,
                dcolour: 0,
            };
            if sim.portal.store(ch, dir, payload) {
                sim.parts[i].ctype = 0;
            }
            return;
        }
    }
}

/// Reference `Element_PPIP_flood_trigger` (`PPIP.cpp`). Flood-fill connected
/// PIPE/PPIP (or same-type for HEAC) and stage trigger/conduct flags in `tmp`.
/// Trigger flags are staged in bits 28..30 (`PPIP_TRIGGER_* << 3`) so they are
/// promoted before consumption; promotion matches reference AfterSim global
/// `ppip_changed` behavior but is applied per-PPIP in `pipe_update` to avoid
/// editing `sim.rs`/tick (documented deviation).
fn ppip_flood_trigger(sim: &mut Sim, x0: i32, y0: i32, sparked_by: u16) {
    let r0 = sim.pmap_at(x0, y0);
    if r0 == 0 {
        return;
    }
    let id0 = pmap_id(r0) as usize;
    if id0 >= sim.parts.len() {
        return;
    }
    let start_type = pmap_type(r0);
    if start_type != pt::PIPE && start_type != pt::PPIP {
        return;
    }
    let prop: i32 = match sparked_by {
        pt::PSCN => PPIP_TRIGGER_ON << 3,
        pt::NSCN => PPIP_TRIGGER_OFF << 3,
        pt::INST => PPIP_TRIGGER_REVERSE << 3,
        pt::HEAC => PFLAG_CAN_CONDUCT | 0x8, // engine also honors 0x8 for pipe heat insulation
        _ => 0,
    };
    if prop == 0 {
        return;
    }
    // Reference: triggers only affect PPIP (HEAC also affects PIPE of same type).
    if start_type != pt::PPIP && sparked_by != pt::HEAC {
        return;
    }
    if sim.parts[id0].tmp & prop != 0 {
        return;
    }
    let mut stack = vec![(x0, y0)];
    let mut seen = std::collections::HashSet::new();
    while let Some((x, y)) = stack.pop() {
        if !sim.in_bounds(x, y) || !seen.insert((x, y)) {
            continue;
        }
        let r = sim.pmap_at(x, y);
        if r == 0 {
            continue;
        }
        let id = pmap_id(r) as usize;
        if id >= sim.parts.len() {
            continue;
        }
        let rt = pmap_type(r);
        if rt != start_type {
            continue;
        }
        if sim.parts[id].tmp & prop == 0 {
            sim.parts[id].tmp |= prop;
        }
        for (dx, dy) in PIPE_OFFSETS {
            stack.push((x + dx, y + dy));
        }
    }
}

/// Reference `PPIP_flood_trigger` for `HEAC`: mark every pipe connected to
/// this one as heat-conductive.
fn pipe_flood_can_conduct(sim: &mut Sim, x0: i32, y0: i32) {
    ppip_flood_trigger(sim, x0, y0, pt::HEAC);
}

fn pipe_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, nt: i32) -> i32 {
    // Sanitize stored type.
    if sim.parts[i].ctype != 0
        && ((sim.parts[i].ctype as usize) >= crate::elements_generated::PT_NUM
            || !element(sim.parts[i].ctype as u16).enabled)
    {
        sim.parts[i].ctype = 0;
    }

    // Reference AfterSim `ppip_changed`: promote staged trigger bits
    // (0xE0000000 >> 3 -> 0x1C000000) for PPIP. Applied per-PPIP here because
    // this task avoids editing sim.rs/tick.rs (deviation from global AfterSim).
    if sim.parts[i].t == pt::PPIP && (sim.parts[i].tmp & 0xE000_0000u32 as i32) != 0 {
        let staged = sim.parts[i].tmp & 0xE000_0000u32 as i32;
        sim.parts[i].tmp |= (staged as u32 >> 3) as i32;
        sim.parts[i].tmp &= !(0xE000_0000u32 as i32);
    }

    // ---- PPIP trigger flags ------------------------------------------
    if sim.parts[i].tmp & PPIP_TRIGGERS != 0 {
        let mut pause_changed = false;
        if sim.parts[i].tmp & PPIP_TRIGGER_ON != 0 {
            if sim.parts[i].tmp & PPIP_PAUSED != 0 {
                pause_changed = true;
            }
            sim.parts[i].tmp &= !PPIP_PAUSED;
        } else if sim.parts[i].tmp & PPIP_TRIGGER_OFF != 0 {
            if sim.parts[i].tmp & PPIP_PAUSED == 0 {
                pause_changed = true;
            }
            sim.parts[i].tmp |= PPIP_PAUSED;
        }
        if pause_changed {
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
                    let r = sim.pmap_at(nx, ny);
                    if r == 0 {
                        continue;
                    }
                    let id = pmap_id(r) as usize;
                    if pmap_type(r) == pt::BRCK {
                        sim.parts[id].tmp = if sim.parts[i].tmp & PPIP_PAUSED != 0 { 0 } else { 1 };
                    }
                }
            }
        }
        if sim.parts[i].tmp & PPIP_TRIGGER_REVERSE != 0 {
            sim.parts[i].tmp ^= PPIP_REVERSED;
            // Switch colors so it goes in reverse (reference PIPE.cpp).
            if (sim.parts[i].tmp & PFLAG_COLORS) != PFLAG_COLOR_GREEN {
                sim.parts[i].tmp ^= PFLAG_COLOR_GREEN;
            }
            // Swap single-pixel pipe direction (reference PIPE.cpp).
            if sim.parts[i].tmp & 0x100 != 0 {
                let coords = (sim.parts[i].tmp >> 13) & 0xF;
                let coords2 = (sim.parts[i].tmp >> 9) & 0xF;
                sim.parts[i].tmp &= !0x1FE00;
                sim.parts[i].tmp |= coords << 9;
                sim.parts[i].tmp |= coords2 << 13;
            }
        }
        sim.parts[i].tmp &= !PPIP_TRIGGERS;
    }

    let life = sim.parts[i].life;
    if (sim.parts[i].tmp & PFLAG_COLORS) != 0 && (sim.parts[i].tmp & PPIP_PAUSED) == 0 {
        if life == 3 {
            // Automatic pattern / color propagation.
            let mut neighbor_count = 0;
            let mut last_neighbor: Option<usize> = None;
            let mut count = 0;
            let mut heat_pipe = false;
            for rx in -1..=1 {
                for ry in -1..=1 {
                    if rx == 0 && ry == 0 {
                        continue;
                    }
                    count += 1;
                    let nx = x + rx;
                    let ny = y + ry;
                    if !sim.in_bounds(nx, ny) {
                        continue;
                    }
                    let r = sim.pmap_at(nx, ny);
                    if r == 0 {
                        continue;
                    }
                    let id = pmap_id(r) as usize;
                    let rt = pmap_type(r);
                    if rt == pt::HEAC {
                        heat_pipe = true;
                        continue;
                    }
                    if rt != pt::PIPE && rt != pt::PPIP {
                        continue;
                    }
                    let next = pipe_next_color(sim.parts[i].tmp);
                    let prev = pipe_prev_color(sim.parts[i].tmp);
                    if sim.parts[id].tmp & PFLAG_INITIALIZING != 0 {
                        sim.parts[id].tmp |= next;
                        sim.parts[id].tmp &= !PFLAG_INITIALIZING;
                        sim.parts[id].life = 6;
                        if sim.parts[i].tmp & 0x100 != 0 {
                            sim.parts[id].tmp |= 0x200;
                            sim.parts[id].tmp |= (count - 1) << 10;
                            sim.parts[i].tmp |= (8 - count) << 14;
                            sim.parts[i].tmp |= 0x2000;
                        }
                        neighbor_count += 1;
                        last_neighbor = Some(id);
                    } else if (sim.parts[id].tmp & PFLAG_COLORS) != prev {
                        neighbor_count += 1;
                        last_neighbor = Some(id);
                    }
                }
            }
            if neighbor_count == 1 {
                if let Some(id) = last_neighbor {
                    sim.parts[id].tmp |= 0x100;
                }
            }
            if heat_pipe {
                pipe_flood_can_conduct(sim, x, y);
            }
        } else {
            // Normal transport.
            if sim.parts[i].flags as i32 & PFLAG_NORMALSPEED != 0 {
                sim.parts[i].flags &= !(PFLAG_NORMALSPEED as u32);
            } else {
                pipe_push(sim, i, 0, i);
            }

            // Intake from a neighbouring particle (direction seeds the flow field).
            // Reference PIPE.cpp:241-243: check pmap, fall back to photons when empty
            // so TYPE_ENERGY particles (e.g. PHOT) can be ingested.
    if sim.parts[i].ctype == 0 {
        'intake: for rx in -1..=1 {
            for ry in -1..=1 {
                if rx == 0 && ry == 0 {
                    continue;
                }
                let nx = x + rx;
                let ny = y + ry;
                if !sim.in_bounds(nx, ny) {
                    continue;
                }
                let mut r = sim.pmap_at(nx, ny);
                if r == 0 {
                    r = sim.photons[sim.idx(nx, ny)];
                }
                if r == 0 {
                    continue;
                }
                let id = pmap_id(r) as usize;
                let rt = pmap_type(r);
                let p = element(rt).properties;
                if (p & (props::TYPE_PART | props::TYPE_LIQUID | props::TYPE_GAS | props::TYPE_ENERGY)) != 0 {
                    if rt == pt::SOAP {
                        sim.soap_detach(id);
                    }
                    part_to_pipe(sim, id, i);
                    if let Some(idx) = PIPE_OFFSETS.iter().position(|&(ox, oy)| ox == rx && oy == ry) {
                        pipe_set_dir(&mut sim.parts[i].tmp, PIPE_OPPOSITE[idx]);
                    }
                    sim.kill_part(id);
                    break 'intake;
                }
                if rt == pt::STOR {
                    let stored = sim.parts[id].tmp;
                    if stored > 0 && (stored as usize) < crate::elements_generated::PT_NUM {
                        let st = element(stored as u16).properties;
                        if (st & (props::TYPE_PART | props::TYPE_LIQUID | props::TYPE_GAS | props::TYPE_ENERGY)) != 0 {
                            // STOR uses the same storage format.
                            let temp = sim.parts[id].temp;
                            let life = sim.parts[id].tmp2;
                            let tmp = sim.parts[id].tmp3;
                            let ctype = sim.parts[id].tmp4;
                            sim.parts[i].ctype = stored;
                            sim.parts[i].temp = temp;
                            sim.parts[i].tmp2 = life;
                            sim.parts[i].tmp3 = tmp;
                            sim.parts[i].tmp4 = ctype;
                            sim.parts[id].tmp = 0;
                            if let Some(idx) = PIPE_OFFSETS.iter().position(|&(ox, oy)| ox == rx && oy == ry) {
                                pipe_set_dir(&mut sim.parts[i].tmp, PIPE_OPPOSITE[idx]);
                            }
                            break 'intake;
                        }
                    }
                }
            }
        }
    } else if nt != 0 {
                // Emit at an open neighbour.
                let mut rnd = sim.rng.gen();
                let idx = (rnd & 7) as usize;
                rnd >>= 3;
                let (rx, ry) = PIPE_OFFSETS[idx];
                let nx = x + rx;
                let ny = y + ry;
                if cell_free(sim, nx, ny) {
                    let t = sim.parts[i].ctype as u16;
                    if let Some(np) = sim.create_part(nx, ny, t) {
                        pipe_to_part(sim, i, np);
                    }
                }
            }
        }
    } else if (sim.parts[i].tmp & (PFLAG_COLORS | PFLAG_INITIALIZING)) == 0 && life <= 10 {
        // Generate the BRCK border around an unconfigured pipe.
        for rx in -2..=2 {
            for ry in -2..=2 {
                if rx == 0 && ry == 0 {
                    continue;
                }
                let nx = x + rx;
                let ny = y + ry;
                if cell_free(sim, nx, ny) {
                    // Reference: PIPE.cpp L281-285 — PPIP-created BRCK border gets tmp = 1.
                    let brck = sim.create_part(nx, ny, pt::BRCK);
                    if sim.parts[i].t == pt::PPIP {
                        if let Some(bi) = brck {
                            sim.parts[bi].tmp = 1;
                        }
                    }
                }
            }
        }
        if life <= 1 {
            sim.parts[i].tmp |= PFLAG_INITIALIZING;
        }
    } else if sim.parts[i].tmp & PFLAG_INITIALIZING != 0 {
        if life == 0 {
            for rx in -1..=1 {
                for ry in -1..=1 {
                    if rx == 0 && ry == 0 {
                        continue;
                    }
                    let nx = x + rx;
                    let ny = y + ry;
                    if cell_free(sim, nx, ny) {
                        sim.parts[i].life = 50;
                        break;
                    }
                }
            }
        } else if life == 5 {
            // Reference: a pipe with no adjacent PIPE/PPIP is single-pixel.
            let mut issingle = true;
            for rx in -1..=1 {
                for ry in -1..=1 {
                    if rx == 0 && ry == 0 {
                        continue;
                    }
                    let rt = sim.type_at(x + rx, y + ry);
                    if rt == pt::PIPE || rt == pt::PPIP {
                        issingle = false;
                    }
                }
            }
            if issingle {
                sim.parts[i].tmp |= 0x100;
            }
        } else if life == 2 {
            sim.parts[i].tmp |= PFLAG_COLOR_RED;
            sim.parts[i].tmp &= !PFLAG_INITIALIZING;
            sim.parts[i].life = 6;
        }
    }
    let _ = pv(sim, x, y);
    0
}

// --- WIFI ----------------------------------------------------------------
fn wifi_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    let ch = portal_channel(sim, i);
    sim.parts[i].tmp = ch as i32;
    if sim.wireless.is_active(ch) {
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
                let rt = pmap_type(r);
                if matches!(rt, pt::NSCN | pt::PSCN | pt::INWR) && sim.parts[id].life == 0 {
                    sim.parts[id].ctype = rt as i32;
                    sim.part_change_type(id, nx, ny, pt::SPRK as i16);
                    sim.parts[id].life = 4;
                }
            }
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
            if pmap_type(r) == pt::SPRK && sim.parts[id].ctype != pt::NSCN as i32 && sim.parts[id].life >= 3 {
                sim.wireless.note_spark(ch);
            }
        }
    }
    0
}

// --- ARAY ----------------------------------------------------------------
fn aray_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    // Simplified: when sparked, cast a BRAY line outward in the spark direction.
    let short_life = if sim.parts[i].life > 0 { sim.parts[i].life } else { 30 };
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
            if pmap_type(r) == pt::SPRK && sim.parts[id].life == 3 {
                let destroy = sim.parts[id].ctype == pt::PSCN as i32;
                let mut tx = nx - rx;
                let mut ty = ny - ry;
                let mut steps = 0;
                while sim.in_bounds(tx, ty) && steps < 256 {
                    let cell = sim.idx(tx, ty);
                    let rr = sim.pmap[cell];
                    if rr == 0 {
                        if let Some(np) = sim.create_part(tx, ty, pt::BRAY) {
                            if destroy {
                                sim.parts[np].tmp = 2;
                                sim.parts[np].life = 2;
                            } else {
                                sim.parts[np].life = short_life;
                            }
                            sim.parts[np].temp = sim.parts[i].temp;
                        }
                    } else if pmap_type(rr) == pt::BRAY {
                        sim.parts[pmap_id(rr) as usize].life = 1020;
                    } else {
                        break;
                    }
                    tx -= rx;
                    ty -= ry;
                    steps += 1;
                }
            }
        }
    }
    0
}

// --- CRAY ----------------------------------------------------------------
fn cray_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    // Simplified: on spark, copy the source line outward from CRAY.
    let ctype = sim.parts[i].ctype;
    if !(ctype > 0 && (ctype as usize) < crate::elements_generated::PT_NUM) {
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
                if !matches!(rt, pt::CRAY | pt::PSCN | pt::INST | pt::METL | pt::SPRK)
                    && (rt as usize) < crate::elements_generated::PT_NUM
                {
                    sim.parts[i].ctype = rt as i32;
                    sim.parts[i].temp = sim.parts[pmap_id(r) as usize].temp;
                }
            }
        }
        return 0;
    }
    let target = ctype as u16;
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
            if pmap_type(r) == pt::SPRK && sim.parts[id].life == 3 {
                let mut remaining = if sim.parts[i].tmp > 0 { sim.parts[i].tmp } else { 255 };
                let mut tx = nx - rx;
                let mut ty = ny - ry;
                while sim.in_bounds(tx, ty) && remaining > 0 {
                    let cell = sim.idx(tx, ty);
                    let wall = sim.wall_at(tx, ty);
                    if wall == 0 && (sim.pmap[cell] == 0 || sim.parts[i].life > 0) {
                        let temp = sim.parts[i].temp;
                        let life = sim.parts[i].life;
                        if let Some(np) = sim.create_part(tx, ty, target) {
                            sim.parts[np].temp = temp;
                            if life > 0 {
                                sim.parts[np].life = life;
                            }
                            remaining -= 1;
                        }
                    } else if pmap_type(sim.pmap[cell]) == pt::CRAY {
                        // pass through
                    } else {
                        break;
                    }
                    tx -= rx;
                    ty -= ry;
                }
            }
        }
    }
    0
}

// --- PSTN ----------------------------------------------------------------
// Reference `PSTN.cpp`: temperature sets how many cells the arm moves per
// frame, `tmp3` bit flags control obstacle/limit behaviour, and a leading
// `FRME` row is displaced with the same stack operation.
/// Discover the connected `FRME` component (4-connected) containing `(sx, sy)`.
fn frme_component(sim: &Sim, sx: i32, sy: i32) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut stack = vec![(sx, sy)];
    while let Some((x, y)) = stack.pop() {
        if !sim.in_bounds(x, y) || !seen.insert((x, y)) {
            continue;
        }
        let r = sim.pmap_at(x, y);
        if r == 0 || pmap_type(r) != pt::FRME {
            continue;
        }
        out.push((x, y));
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            stack.push((x + dx, y + dy));
        }
    }
    out
}

/// Rigidly displace an entire connected `FRME` component by `amount` cells in
/// `(dx, dy)`. Validates every destination first (all-or-nothing), then moves
/// cells front-first through the shared `move_stack` so no cell is overwritten.
/// Returns true when the component moved.
fn push_frme_component(
    sim: &mut Sim,
    sx: i32,
    sy: i32,
    dx: i32,
    dy: i32,
    amount: i32,
) -> bool {
    if amount <= 0 {
        return false;
    }
    let comp = frme_component(sim, sx, sy);
    if comp.is_empty() {
        return false;
    }
    let set: std::collections::HashSet<(i32, i32)> = comp.iter().copied().collect();
    // Validate every destination.
    for &(x, y) in &comp {
        let tx = x + dx * amount;
        let ty = y + dy * amount;
        if !sim.in_bounds(tx, ty) {
            return false;
        }
        if sim.wall_at(tx, ty) != 0 {
            return false;
        }
        let r = sim.pmap_at(tx, ty);
        if r != 0 && !(pmap_type(r) == pt::FRME && set.contains(&(tx, ty))) {
            return false;
        }
    }
    // Move front-first (largest projection along the direction).
    let mut order = comp;
    order.sort_by_key(|&(x, y)| -(x * dx + y * dy));
    for &(x, y) in &order {
        sim.move_stack(x, y, dx, dy, 2, amount);
    }
    true
}

fn pstn_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    if sim.parts[i].life != 0 {
        return 0;
    }
    let max_size = if sim.parts[i].tmp > 0 { sim.parts[i].tmp } else { 31 };
    let arm_limit = if sim.parts[i].tmp2 > 0 { sim.parts[i].tmp2 } else { 255 };
    let flags = sim.parts[i].tmp3;
    let ext_obstacle = flags & 0x01 != 0;
    let ext_armlimit = flags & 0x02 != 0;
    let ret_tooshort = flags & 0x08 != 0;

    let mut state = 0; // 0 inactive, 1 extend, 2 retract
    let mut spark_off = (0i32, 0i32);
    for rx in -2..=2 {
        for ry in -2..=2 {
            if (rx == 0) == (ry == 0) {
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
            if pmap_type(r) == pt::SPRK && sim.parts[id].life == 3 {
                if sim.parts[id].ctype == pt::PSCN as i32 {
                    state = 1;
                } else {
                    state = 2;
                }
                spark_off = (rx.signum(), ry.signum());
            }
        }
    }
    if state == 0 {
        return 0;
    }
    // Reference direction: toward an adjacent PSTN arm when present; a lone
    // piston extends away from the activating spark.
    let mut axis = (0i32, 0i32);
    for rx in -1..=1 {
        for ry in -1..=1 {
            if (rx == 0) == (ry == 0) {
                continue;
            }
            let r = sim.pmap_at(x + rx, y + ry);
            if r != 0
                && pmap_type(r) == pt::PSTN
                && sim.parts[pmap_id(r) as usize].life == 0
            {
                axis = (rx.signum(), ry.signum());
            }
        }
    }
    if axis.0 == 0 && axis.1 == 0 {
        if spark_off.0 == 0 && spark_off.1 == 0 {
            return 0;
        }
        axis = (-spark_off.0, -spark_off.1);
    }
    let (dx, dy) = axis;
    // Count the arm and find its far end.
    let mut arm = 0i32;
    let mut end_x = x;
    let mut end_y = y;
    for step in 1..=arm_limit {
        let tx = x + dx * step;
        let ty = y + dy * step;
        if !sim.in_bounds(tx, ty) {
            break;
        }
        let r = sim.pmap_at(tx, ty);
        if r != 0 && pmap_type(r) == pt::PSTN {
            arm += 1;
            end_x = tx;
            end_y = ty;
        } else {
            break;
        }
    }
    // Reference: tens of degrees above 0 C, rounded down, summed over the arm.
    let mut piston_count = (((sim.parts[i].temp - 268.15) / 10.0).floor() as i32)
        .max(1)
        .min(max_size);
    if state == 1 {
        if arm + piston_count > arm_limit {
            if ext_armlimit {
                return 0;
            }
            piston_count = arm_limit - arm;
        }
        if piston_count <= 0 {
            return 0;
        }
        // FRME structure in front is pushed as a unit first. The reference
        // `MoveStack` walks the whole frame row perpendicular to the piston
        // axis (and the particles resting on it); we push the full contiguous
        // row and abort on the first blocked cell when the obstacle flag is set.
        let fx = x + dx;
        let fy = y + dy;
        if sim.in_bounds(fx, fy) {
            let fr = sim.pmap_at(fx, fy);
            if fr != 0 && pmap_type(fr) == pt::FRME {
                // Discover the whole 4-connected FRME component (recursive
                // stack): arbitrary attached frame structures are displaced as a
                // single rigid unit instead of only the contiguous perpendicular
                // row.
                let comp = frme_component(sim, fx, fy);
                let moved_frame = push_frme_component(sim, fx, fy, dx, dy, piston_count);
                if !moved_frame {
                    if ext_obstacle {
                        return 0;
                    }
                } else {
                    // Particles resting on top of the displaced frame move with
                    // it. Each resting stack is probed independently so several
                    // layers of particles travel together.
                    let (sx, sy) = if dx != 0 { (0, -1) } else { (-1, 0) };
                    for (tx, ty) in &comp {
                        let above_x = tx + sx;
                        let above_y = ty + sy;
                        if sim.pmap_at(above_x, above_y) != 0 {
                            let moved_rest =
                                sim.move_stack(above_x, above_y, dx, dy, max_size, piston_count);
                            if moved_rest == 0 && ext_obstacle {
                                return 0;
                            }
                        }
                    }
                }
            }
        }
        let moved = sim.move_stack(x + dx, y + dy, dx, dy, max_size, piston_count);
        if moved > 0 {
            sim.extend_piston_arm(end_x, end_y, dx, dy, moved, 1);
        } else if cell_free(sim, x + dx, y + dy) {
            // Empty space ahead: reference `MoveStack` returns the free space
            // and the piston creates new arm sections there.
            let room = (arm_limit - arm).max(0);
            let create = piston_count.min(room);
            if create > 0 {
                sim.extend_piston_arm(end_x, end_y, dx, dy, create, 1);
            }
        } else if ext_obstacle {
            return 0;
        }
    } else if arm > 0 {
        if piston_count > arm {
            if ret_tooshort {
                return 0;
            }
            piston_count = arm;
        }
        let removed = sim.retract_piston_arm(end_x, end_y, dx, dy, piston_count);
        if removed > 0 {
            sim.move_stack(x + dx, y + dy, -dx, -dy, max_size, removed);
        }
    }
    0
}

/// Wrapped particle delta for SOAP constraints, honouring edge-loop boundaries
/// (reference `Element_SOAP_neighourLoop`).
fn soap_delta(sim: &Sim, a: usize, b: usize) -> (f32, f32) {
    let mut dx = sim.parts[a].x - sim.parts[b].x;
    let mut dy = sim.parts[a].y - sim.parts[b].y;
    if sim.settings.edge_mode == crate::sim::EdgeMode::Loop {
        let hx = sim.width as f32 / 2.0 - crate::sim::CELL as f32;
        let hy = sim.height as f32 / 2.0 - crate::sim::CELL as f32;
        let span_x = sim.width as f32 - 2.0 * crate::sim::CELL as f32;
        let span_y = sim.height as f32 - 2.0 * crate::sim::CELL as f32;
        if dx >= hx {
            dx -= span_x;
        }
        if dy >= hy {
            dy -= span_y;
        }
        if dx < -hx {
            dx += span_x;
        }
        if dy < -hy {
            dy += span_y;
        }
    }
    (dx, dy)
}

// --- SOAP ----------------------------------------------------------------
// Reference `SOAP.cpp`: bubbles form under pressure, link into chains, pop on
// obstacles or when their life runs out, and solve first- and second-order
// distance constraints. Links live in `tmp`/`tmp2` with `ctype` bits
// 0x02 (front) and 0x04 (back).
fn soap_update(sim: &mut Sim, i: usize, x: i32, y: i32, _s: i32, _nt: i32) -> i32 {
    const FREEZING: f32 = 248.15;
    // 0x01 bubble on/off, 0x02 first mate, 0x04 back mate.
    if sim.parts[i].ctype & 1 == 0 {
        if pv(sim, x, y) > 0.5 || pv(sim, x, y) < -0.5 {
            sim.parts[i].ctype = 1;
            sim.parts[i].life = 10;
        }
        // Wash off nearby decoration.
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
                if pmap_type(r) == pt::SOAP {
                    continue;
                }
                let dc = sim.parts[id].dcolour;
                if dc != 0 {
                    let na = ((dc >> 24) & 0xFF) * 85 / 100;
                    let nr = ((dc >> 16) & 0xFF) * 85 / 100;
                    let ng = ((dc >> 8) & 0xFF) * 85 / 100;
                    let nb = (dc & 0xFF) * 85 / 100;
                    sim.parts[id].dcolour = (na << 24) | (nr << 16) | (ng << 8) | nb;
                }
            }
        }
        return 0;
    }

    // Reset invalid links.
    let front = sim.parts[i].tmp;
    let back = sim.parts[i].tmp2;
    let front_valid = front >= 0
        && (front as usize) < sim.parts.len()
        && sim.parts[front as usize].t == pt::SOAP;
    let back_valid = back >= 0
        && (back as usize) < sim.parts.len()
        && sim.parts[back as usize].t == pt::SOAP;
    if (sim.parts[i].ctype & 0x02 != 0 && !front_valid) || (sim.parts[i].ctype & 0x04 != 0 && !back_valid) {
        sim.soap_detach(i);
        return 0;
    }

    if sim.parts[i].temp > FREEZING {
        // Pop when life is exhausted and the bubble is not fully chained.
        if sim.parts[i].life <= 0 {
            let links = sim.parts[i].ctype & 0x06;
            if links != 0 && links != 0x06 {
                // Walk the chain and detach each node.
                let mut target = if sim.parts[i].ctype & 0x02 != 0 { front as i32 } else { back as i32 };
                let mut guard = 0;
                while guard < 256 {
                    if target < 0 || (target as usize) >= sim.parts.len() {
                        break;
                    }
                    let t = target as usize;
                    if sim.parts[t].t != pt::SOAP {
                        break;
                    }
                    let next = if sim.parts[t].ctype & 0x02 != 0 {
                        sim.parts[t].tmp
                    } else if sim.parts[t].ctype & 0x04 != 0 {
                        sim.parts[t].tmp2
                    } else {
                        -1
                    };
                    sim.soap_detach(t);
                    target = next;
                    guard += 1;
                }
            }
            if sim.parts[i].ctype & 0x06 != 0x06 {
                sim.parts[i].ctype = 0;
            }
            if front_valid && sim.parts[i].ctype & 0x06 == 0x06 {
                sim.soap_detach(i);
            }
        }
        // Obstacle contact pops the bubble.
        if sim.parts[i].life <= 0 {
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
                    if sim.wall_at(nx, ny) != 0 {
                        sim.soap_detach(i);
                        break;
                    }
                    let r = sim.pmap[sim.idx(nx, ny)];
                    if r == 0 {
                        continue;
                    }
                    let id = pmap_id(r) as usize;
                    let rt = pmap_type(r);
                    if rt == pt::SOAP || rt == pt::GLAS {
                        continue;
                    }
                    let p = element(rt).properties;
                    if (p & props::TYPE_GAS) == 0 {
                        sim.soap_detach(i);
                        break;
                    }
                }
            }
        }
        // Mid-chain reconnection (reference `ctype == 7` rewiring).
        if sim.parts[i].ctype & 0x02 != 0 && sim.parts[i].life <= 0 {
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
                    let r = sim.pmap_at(nx, ny);
                    if r == 0 {
                        continue;
                    }
                    let id = pmap_id(r) as usize;
                    if pmap_type(r) != pt::SOAP || id == i {
                        continue;
                    }
                    let nctype = sim.parts[id].ctype;
                    if nctype == 1 {
                        // Neighbour is a lone bubble head: splice it in.
                        let buf = sim.parts[i].tmp;
                        sim.parts[i].tmp = id as i32;
                        if buf >= 0
                            && (buf as usize) < sim.parts.len()
                            && sim.parts[buf as usize].t == pt::SOAP
                        {
                            sim.parts[buf as usize].tmp2 = id as i32;
                        }
                        sim.parts[id].tmp2 = i as i32;
                        sim.parts[id].tmp = buf;
                        sim.parts[id].ctype = 7;
                    } else if nctype == 7
                        && sim.parts[i].tmp != id as i32
                        && sim.parts[i].tmp2 != id as i32
                    {
                        // Two chains meet: connect them end to end.
                        let f = sim.parts[i].tmp;
                        let b = sim.parts[id].tmp2;
                        if f >= 0
                            && (f as usize) < sim.parts.len()
                            && sim.parts[f as usize].t == pt::SOAP
                        {
                            sim.parts[f as usize].tmp2 = b;
                        }
                        if b >= 0
                            && (b as usize) < sim.parts.len()
                            && sim.parts[b as usize].t == pt::SOAP
                        {
                            sim.parts[b as usize].tmp = f;
                        }
                        sim.parts[id].tmp2 = i as i32;
                        sim.parts[i].tmp = id as i32;
                    }
                }
            }
        }
        sim.parts[i].vy = (sim.parts[i].vy - 0.1) * 0.5;
        sim.parts[i].vx *= 0.5;
    }

    // Link to a nearby free soap if this one has no front mate.
    if sim.parts[i].ctype & 0x02 == 0 {
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
                if pmap_type(r) == pt::SOAP
                    && (sim.parts[id].ctype & 1) != 0
                    && (sim.parts[id].ctype & 0x04) == 0
                    && id != i
                {
                    sim.soap_attach(i, id);
                    break;
                }
            }
        }
    }

    // First-order constraint.
    if sim.parts[i].ctype & 0x02 != 0 && front_valid {
        let j = front as usize;
        let (dx, dy) = soap_delta(sim, i, j);
        let k = Sim::soap_constraint(dx, dy);
        sim.soap_link_step(i, j, k);
        // Second-order: neighbour's neighbour when both ends are chained.
        if sim.parts[j].ctype & 0x02 != 0 && (sim.parts[j].ctype & 1) != 0 && sim.parts[j].tmp >= 0 {
            let ii = sim.parts[j].tmp as usize;
            if ii < sim.parts.len() && sim.parts[ii].t == pt::SOAP {
                let (dx, dy) = soap_delta(sim, ii, j);
                let k2 = 81.0 / (dx * dx + dy * dy + 81.0) - 0.5;
                sim.parts[j].vx -= dx * k2 * 0.5;
                sim.parts[j].vy -= dy * k2 * 0.5;
                sim.parts[ii].vx += dx * k2 * 0.5;
                sim.parts[ii].vy += dy * k2 * 0.5;
            }
        }
    }
    if sim.parts[i].ctype & 0x04 != 0 && back_valid {
        let j = back as usize;
        let dx = sim.parts[i].x - sim.parts[j].x;
        let dy = sim.parts[i].y - sim.parts[j].y;
        let k = Sim::soap_constraint(dx, dy);
        sim.soap_link_step(i, j, k);
    }
    0
}

// --- STKM / STKM2 / FIGH -------------------------------------------------
// Shared storage + tile locomotion live in `entity.rs`. The functions below
// add the reference-specific element behaviour: hazard damage, leg phase,
// element-power cadence, death cleanup, and fighter pursuit/combat.
use crate::entity::{input as ent_input, Entity, PlayerState};

/// Returns (lethal, damage) for the 3x3 around an entity head.
fn entity_hazard(sim: &Sim, x: i32, y: i32) -> (bool, i32) {
    let mut lethal = false;
    let mut damage = 0;
    for rx in -1..=1 {
        for ry in -1..=1 {
            let nx = x + rx;
            let ny = y + ry;
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let r = sim.pmap_at(nx, ny);
            if r == 0 {
                continue;
            }
            let rt = pmap_type(r);
            let p = element(rt).properties;
            if rt == pt::ACID || rt == pt::LAVA || rt == pt::VIRS || rt == pt::VRSS || rt == pt::VRSG {
                lethal = true;
            } else if (p & props::PROP_DEADLY) != 0 {
                damage += 1;
            } else if rt == pt::FIRE || rt == pt::PLSM {
                damage += 1;
            }
        }
    }
    (lethal, damage)
}

/// Tile-based leg solver (accepted parity).
///
/// The reference `STKM.cpp` resolves feet with a gravity/rotation particle
/// solver. This implementation keeps locomotion tile-based: feet are stored at
/// the reference indices `legs[4]/[5]` and `legs[12]/[13]`, the torso anchor
/// at `legs[2]/[3]`, and leg phase is derived from the frame counter. This is
/// sufficient for movement, jumping, portals and hazards and is intentionally
/// documented as the accepted (non-reference) solver.
fn stickman_legs(st: &mut PlayerState, grounded: bool, moving: bool) {
    if grounded && moving {
        let phase = ((st.frames / 4) & 1) as f32;
        st.legs[0] = if phase == 0.0 { -1.0 } else { 1.0 };
        st.legs[1] = 0.0;
        st.legs[2] = -st.legs[0];
        st.legs[3] = 0.0;
    } else {
        st.legs[0] = -1.0;
        st.legs[1] = 0.0;
        st.legs[2] = 1.0;
        st.legs[3] = 0.0;
    }
}

fn stickman_can_occupy(sim: &Sim, x: i32, y: i32) -> bool {
    if !sim.in_bounds(x, y) {
        return false;
    }
    if x < CELL as i32 || y < CELL as i32 {
        return false;
    }
    if sim.wall_blocks(x, y, pt::STKM) {
        return false;
    }
    match sim.pmap_at(x, y) {
        0 => true,
        r => {
            let rt = pmap_type(r);
            if matches!(rt, pt::STKM | pt::STKM2 | pt::FIGH) {
                return false;
            }
            (element(rt).properties & props::TYPE_SOLID) == 0
        }
    }
}

fn stickman_feet_grounded(sim: &Sim, x: i32, y: i32) -> bool {
    for fx in [x - 1, x, x + 1] {
        if sim.wall_at(fx, y + 1) != 0 {
            return true;
        }
        let t = sim.type_at(fx, y + 1);
        if t != 0 {
            let def = element(t);
            if (def.properties & props::TYPE_SOLID) != 0 || def.weight > element(pt::STKM).weight {
                return true;
            }
        }
    }
    false
}

fn stickman_set_feet_targets(
    sim: &Sim,
    legs: &mut [f32; 16],
    frames: i32,
    x: i32,
    y: i32,
    moving: bool,
) {
    // Reference feet live in legs[4..6] and legs[12..14]; legs[2..4] is the
    // torso anchor used by FIGH target selection.
    legs[2] = x as f32;
    legs[3] = y as f32;
    let phase: f32 = if moving {
        if ((frames / 4) & 1) == 1 {
            1.0
        } else {
            -1.0
        }
    } else {
        0.0
    };
    // Obvious case vs reference movement/collision: avoid placing feet into
    // occupied solid/invalid targets or walls (conservative; no spring solver).
    // `stickman_can_occupy` treats dead/absent particles and solids/walls as invalid.
    let candidates = [
        (x as f32 - 1.0 + phase.max(0.0), y as f32 + 1.0, 4usize, 5usize),
        (x as f32 + 1.0 + phase.min(0.0), y as f32 + 1.0, 12usize, 13usize),
    ];
    for (fxf, fyf, ix, iy) in candidates {
        let fx = (fxf + 0.5) as i32;
        let fy = (fyf + 0.5) as i32;
        if stickman_can_occupy(sim, fx, fy) {
            legs[ix] = fxf;
            legs[iy] = fyf;
        }
        // else keep previous foot position (conservative).
    }
}

/// Set a stickman's element power from a touched particle, mirroring
/// `Element_STKM_set_element`.
fn stickman_set_element(st: &mut PlayerState, elem: i32) {
    use crate::elements_generated::props;
    let def = element(elem as u16);
    if def.falldown != 0
        || (def.properties & (props::TYPE_GAS | props::TYPE_LIQUID | props::TYPE_ENERGY)) != 0
        || elem == pt::LOLZ as i32
        || elem == pt::LOVE as i32
    {
        if !st.rocket_boots || elem != pt::PLSM as i32 {
            st.elem = elem;
            st.fan = false;
        }
    }
    if elem == pt::TESC as i32 || elem == pt::LIGH as i32 {
        st.elem = pt::LIGH as i32;
        st.fan = false;
    }
}

/// Kill a stickman head, mirroring `die()` in `STKM.cpp`.
fn stickman_die(sim: &mut Sim, st: &mut PlayerState, i: usize, which: Entity) {
    if st.fan {
        let cx = ((sim.parts[i].x + 0.5) as i32) / CELL as i32;
        let cy = ((sim.parts[i].y + 0.5) as i32) / CELL as i32;
        if sim.in_cell_bounds(cx, cy) {
            let _idx = sim.cell_idx(cx, cy); sim.pv[_idx] += 64.0;
        }
    } else {
        let x = (sim.parts[i].x + 0.5) as i32;
        let y = (sim.parts[i].y + 0.5) as i32;
        let elem = st.elem;
        if elem > 0 && (elem as usize) < crate::elements_generated::PT_NUM {
            for r in -2..=1i32 {
                sim.create_part(x + r, y - 2, elem as u16);
                sim.create_part(x + r + 1, y + 2, elem as u16);
                sim.create_part(x - 2, y + r + 1, elem as u16);
                sim.create_part(x + 2, y + r, elem as u16);
            }
        }
    }
    sim.kill_part(i);
    if let Some(s) = sim.player_state_mut(which) {
        s.spwn = 0;
        s.comm = 0;
        s.pcomm = 0;
    }
    let _ = st;
}

/// Leg / particle interaction, mirroring `Element_STKM_interact`.
fn stickman_interact(sim: &mut Sim, i: usize, x: i32, y: i32, which: Entity) {
    use crate::elements_generated::props;
    if i >= sim.parts.len() || sim.parts[i].t == 0 {
        return;
    }
    if !sim.in_bounds(x, y) {
        return;
    }
    let r = sim.pmap_at(x, y);
    if r == 0 {
        return;
    }
    let id = pmap_id(r) as usize;
    if id >= sim.parts.len() {
        return;
    }
    let rt = pmap_type(r);
    let mut damage = 0i32;
    // SPRK
    {
        let st_elem = sim.player_state(which).map(|s| s.elem).unwrap_or(0);
        if rt == pt::SPRK && st_elem != pt::LIGH as i32 {
            damage += sim.rng.between(32, 51);
        }
    }
    // heat
    let heat_damage = {
        let is_ins = sim.is_heat_insulator(id);
        let temp = sim.parts[id].temp;
        let rocket = sim.player_state(which).map(|s| s.rocket_boots).unwrap_or(false);
        let elem = sim.player_state(which).map(|s| s.elem).unwrap_or(0);
        if !is_ins && ((elem != pt::LIGH as i32 && temp >= 323.0) || temp <= 243.0)
            && !(rocket && rt == pt::PLSM)
        {
            true
        } else {
            false
        }
    };
    if heat_damage {
        damage += 2;
        if let Some(st) = sim.player_state_mut(which) {
            st.accs[3] -= 1.0;
        }
    }
    // deadly
    let def = element(rt);
    if (def.properties & props::PROP_DEADLY) != 0 {
        if rt == pt::ACID {
            damage += 5;
        } else {
            damage += 1;
        }
    }
    // radioactive
    if (def.properties & props::PROP_RADIOACTIVE) != 0 {
        damage += 1;
    }
    if damage > 0 {
        let life = sim.parts[i].life;
        if damage > life {
            if let Some(mut st) = sim.player_state(which).cloned() {
                stickman_die(sim, &mut st, i, which);
            }
            return;
        } else {
            sim.parts[i].life -= damage;
        }
    }
    if i >= sim.parts.len() || sim.parts[i].t == 0 {
        return;
    }
    // PRTI portal
    if rt == pt::PRTI {
        let ch = portal_channel(sim, id);
        sim.parts[id].tmp = ch as i32;
        let payload = crate::portal::PortalPayload::from_particle(&sim.parts[i]);
        if sim.portal.store(ch, 1, payload) {
            sim.kill_part(i);
            if let Some(st) = sim.player_state_mut(which) {
                st.spwn = 1;
            }
            return;
        }
    }
    if i >= sim.parts.len() || sim.parts[i].t == 0 {
        return;
    }
    // BHOL / NBHL
    if (rt == pt::BHOL || rt == pt::NBHL) {
        if sim.settings.heat_sim && id < sim.parts.len() {
            sim.parts[id].temp = restrict_flt(sim.parts[id].temp + sim.parts[i].temp / 2.0, MIN_TEMP, MAX_TEMP);
        }
        sim.kill_part(i);
        if let Some(st) = sim.player_state_mut(which) {
            st.spwn = 0;
            st.comm = 0;
            st.pcomm = 0;
        }
        return;
    }
    // VOID / PVOD
    if rt == pt::VOID || (rt == pt::PVOD && sim.parts[id].life == 10) {
        let ctype = sim.parts[id].ctype;
        let kill = if ctype == 0 {
            true
        } else {
            let eq = ctype == sim.parts[i].t as i32;
            let bit = (sim.parts[id].tmp & 1) != 0;
            eq != bit
        };
        if kill {
            sim.kill_part(i);
            if let Some(st) = sim.player_state_mut(which) {
                st.spwn = 0;
                st.comm = 0;
                st.pcomm = 0;
            }
            return;
        }
    }
}

/// Reference stickman portal interaction: a head standing on `PRTI` emerges
/// next to a `PRTO` on the same channel (approximation of the portal payload
/// path used by `STKM_interact`).
fn stickman_portal(sim: &Sim, x: i32, y: i32) -> Option<(i32, i32)> {
    // Reference `STKM_interact` reacts to portals in the surrounding cells.
    let mut prti: Option<usize> = None;
    for rx in -1..=1 {
        for ry in -1..=1 {
            let nx = x + rx;
            let ny = y + ry;
            if !sim.in_bounds(nx, ny) {
                continue;
            }
            let r = sim.pmap_at(nx, ny);
            if r != 0 && pmap_type(r) == pt::PRTI {
                prti = Some(pmap_id(r) as usize);
            }
        }
    }
    let prti = prti?;
    let ch = portal_channel(sim, prti);
    for (idx, p) in sim.parts.iter().enumerate() {
        if p.t != pt::PRTO {
            continue;
        }
        if portal_channel(sim, idx) != ch {
            continue;
        }
        let px = (p.x + 0.5) as i32;
        let py = (p.y + 0.5) as i32;
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let tx = px + dx;
            let ty = py + dy;
            if stickman_can_occupy(sim, tx, ty) {
                return Some((tx, ty));
            }
        }
    }
    None
}

fn stickman_element_power(sim: &mut Sim, i: usize, x: i32, y: i32, comm: u8, elem: i32) -> bool {
    if comm & ent_input::SPAWN == 0
        || elem <= 0
        || (elem as usize) >= crate::elements_generated::PT_NUM
        || !element(elem as u16).enabled
    {
        return false;
    }
    let dx = if comm & ent_input::LEFT != 0 {
        -1
    } else if comm & ent_input::RIGHT != 0 {
        1
    } else {
        0
    };
    let tx = x + dx;
    let ty = y;
    // Reference STKM.cpp (spawn action): if target is a solid particle, spark
    // instead of spawning the element. Do not create in solid wall/invalid.
    if !sim.in_bounds(tx, ty) || sim.wall_blocks(tx, ty, pt::STKM) {
        return false;
    }
    let r = sim.pmap_at(tx, ty);
    if r != 0 {
        let rt = pmap_type(r);
        if (element(rt).properties & props::TYPE_SOLID) != 0 {
            // Reference-like fallback for solid obstacle in spawn direction.
            let _ = sim.create_part(tx, ty, pt::SPRK);
            return true;
        }
        // Occupied by non-solid: attempt create at preferred facing cell only;
        // do not overwrite. Reference passes -1 (no ring fallback).
    }
    if let Some(np) = sim.create_part(tx, ty, elem as u16) {
        let def = element(elem as u16);
        if (def.properties & props::TYPE_ENERGY) != 0 {
            sim.parts[np].vx = dx as f32 * 2.0;
            sim.parts[np].vy = 0.0;
        }
        sim.parts[np].temp = sim.parts[i].temp;
        return true;
    }
    false
}

fn stickman_tick(sim: &mut Sim, i: usize, which: Entity) -> i32 {
    if i >= sim.parts.len() || sim.parts[i].t == 0 {
        return 0;
    }
    let x = (sim.parts[i].x + 0.5) as i32;
    let y = (sim.parts[i].y + 0.5) as i32;

    // Element power acquisition from touched particles.
    let ctype = sim.parts[i].ctype;
    if let Some(st) = sim.player_state_mut(which) {
        if !st.fan && ctype != 0 && crate::transition::is_element_or_none(ctype) {
            stickman_set_element(st, ctype);
        }
    }

    // Temperature handling.
    if sim.parts[i].temp < 243.0 {
        sim.parts[i].life -= 1;
    }
    if sim.parts[i].temp < 309.6 && sim.parts[i].temp >= 243.0 {
        sim.parts[i].temp += 1.0;
    }

    let fan_before = sim.player_state(which).map(|p| p.fan).unwrap_or(false);
    let pv_high = if sim.in_cell_bounds(x / CELL as i32, y / CELL as i32) {
        sim.pv[sim.cell_idx(x / CELL as i32, y / CELL as i32)] >= 4.5
    } else {
        false
    };
    if sim.parts[i].life < 1 || (pv_high && !fan_before) {
        if let Some(mut st) = sim.player_state(which).cloned() {
            stickman_die(sim, &mut st, i, which);
        }
        return 1;
    }

    let (lethal, damage) = entity_hazard(sim, x, y);
    if lethal {
        sim.kill_part(i);
        if let Some(st) = sim.player_state_mut(which) {
            st.spwn = 0;
            st.comm = 0;
            st.pcomm = 0;
        }
        return 1;
    }
    if damage > 0 {
        sim.parts[i].life -= damage;
        if sim.parts[i].life < 0 {
            sim.kill_part(i);
            if let Some(st) = sim.player_state_mut(which) {
                st.spwn = 0;
            }
            return 1;
        }
    }

    // Head-neighbourhood element acquisition / fan / rocket boots.
    for rx in -2i32..=2 {
        for ry in -2i32..=2 {
            if rx == 0 && ry == 0 {
                continue;
            }
            let nx = x + rx;
            let ny = y + ry;
            if nx < 0 || ny <= 0 || nx >= sim.width as i32 || ny >= sim.height as i32 {
                continue;
            }
            let mut r = sim.pmap_at(nx, ny);
            if r == 0 {
                r = sim.photons[sim.idx(nx, ny)];
            }
            if r != 0 {
                let rt = pmap_type(r);
                if !fan_before {
                    if crate::transition::is_element_or_none(rt as i32) {
                        let ctype = rt as i32;
                        if let Some(st) = sim.player_state_mut(which) {
                            if !st.fan {
                                stickman_set_element(st, ctype);
                            }
                        }
                    }
                }
                if rt == pt::PLNT && sim.parts[i].life < 100 {
                    if sim.parts[i].life <= 95 {
                        sim.parts[i].life += 5;
                    } else {
                        sim.parts[i].life = 100;
                    }
                    sim.kill_part(pmap_id(r) as usize);
                }
                if rt == pt::NEUT {
                    if sim.parts[i].life <= 100 {
                        sim.parts[i].life -= (102 - sim.parts[i].life) / 2;
                    } else {
                        sim.parts[i].life = (sim.parts[i].life as f32 * 0.9) as i32;
                    }
                    sim.kill_part(pmap_id(r) as usize);
                }
            }
            let w = sim.wall_at(nx, ny);
            if w == 5 {
                if let Some(st) = sim.player_state_mut(which) {
                    st.fan = true;
                }
            } else if w == 12 {
                if let Some(st) = sim.player_state_mut(which) {
                    st.rocket_boots = false;
                }
            } else if w == 14 {
                if let Some(st) = sim.player_state_mut(which) {
                    st.rocket_boots = true;
                }
            }
        }
    }
    if i >= sim.parts.len() || sim.parts[i].t == 0 {
        return 1;
    }

    // Portal teleport.
    if let Some((tx, ty)) = stickman_portal(sim, x, y) {
        sim.parts[i].x = tx as f32;
        sim.parts[i].y = ty as f32;
        return 0;
    }

    let comm = sim.player_state(which).map(|p| p.comm).unwrap_or(0);
    let elem = sim.player_state(which).map(|p| p.elem).unwrap_or(0);
    let mut comm_adj = comm;

    // Element power (spawned here so energy particles get velocity).
    if stickman_element_power(sim, i, x, y, comm, elem) {
        comm_adj &= !ent_input::SPAWN;
    }

    // Jump: reference jump leaves the ground for a couple of cells.
    if comm & ent_input::JUMP != 0 && stickman_feet_grounded(sim, x, y) {
        let mut jumped = false;
        for up in [2i32, 1] {
            if stickman_can_occupy(sim, x, y - up) {
                sim.parts[i].y = (y - up) as f32;
                jumped = true;
                break;
            }
        }
        if jumped {
            comm_adj &= !ent_input::JUMP;
        }
    }

    if let Some(st) = sim.player_state_mut(which) {
        st.pcomm = comm;
        st.comm = comm_adj;
    }
    let report = sim.step_entity_at(which, i);
    if report.died || i >= sim.parts.len() || sim.parts[i].t == 0 {
        return 1;
    }
    let moving = comm_adj & (ent_input::LEFT | ent_input::RIGHT) != 0;
    let nx = (sim.parts[i].x + 0.5) as i32;
    let ny = (sim.parts[i].y + 0.5) as i32;
    let feet = {
        let frames = sim.player_state(which).map(|p| p.frames as i32).unwrap_or(0);
        let mut legs = sim.player_state(which).map(|p| p.legs).unwrap_or([0.0; 16]);
        stickman_set_feet_targets(sim, &mut legs, frames, nx, ny, moving);
        if let Some(st) = sim.player_state_mut(which) {
            st.legs = legs;
            stickman_legs(st, report.grounded, moving);
            (st.legs[4] as i32, st.legs[5] as i32, st.legs[12] as i32, st.legs[13] as i32)
        } else {
            (0, 0, 0, 0)
        }
    };
    if i < sim.parts.len() && sim.parts[i].t != 0 {
        stickman_interact(sim, i, feet.0, feet.1, which);
    }
    if i < sim.parts.len() && sim.parts[i].t != 0 {
        stickman_interact(sim, i, feet.2, feet.3, which);
    }
    if i < sim.parts.len() && sim.parts[i].t != 0 {
        sim.parts[i].ctype = sim.player_state(which).map(|p| p.elem).unwrap_or(0);
    }
    0
}

fn stkm_update(sim: &mut Sim, i: usize, _x: i32, _y: i32, _s: i32, _nt: i32) -> i32 {
    stickman_tick(sim, i, Entity::Player1)
}

fn stkm2_update(sim: &mut Sim, i: usize, _x: i32, _y: i32, _s: i32, _nt: i32) -> i32 {
    stickman_tick(sim, i, Entity::Player2)
}

fn fighter_die(sim: &mut Sim, i: usize, slot: usize) {
    sim.kill_part(i);
    let which = Entity::Fighter(slot);
    let was = sim.player_state(which).map(|f| f.spwn != 0).unwrap_or(false);
    if was {
        if let Some(f) = sim.player_state_mut(which) {
            f.spwn = 0;
        }
        sim.figh_count = (sim.figh_count - 1).max(0);
    }
}

/// Allocate a fighter slot on creation (reference `Element_FIGH_Alloc` /
/// `Element_FIGH_NewFighter`). Without this a freshly created FIGH particle
/// would keep the default `tmp == 0` and never be registered, so multi-fighter
/// worlds could not distinguish heads.
fn figh_alloc_slot(sim: &mut Sim) -> Option<usize> {
    if sim.figh_count >= crate::entity::MAX_FIGHTERS as i32 {
        return None;
    }
    let slot = match sim.fighters.iter().position(|f| f.spwn == 0) {
        Some(s) => s,
        None => {
            if sim.fighters.len() < crate::entity::MAX_FIGHTERS {
                sim.fighters.push(crate::entity::PlayerState::default());
                sim.fighters.len() - 1
            } else {
                return None;
            }
        }
    };
    if let Some(f) = sim.fighters.get_mut(slot) {
        f.spwn = 1;
        f.elem = pt::DUST as i32;
        f.comm = 0;
        f.pcomm = 0;
        f.frames = 0;
        f.legs = [0.0; 16];
        f.accs = [0.0; 8];
    }
    sim.figh_count += 1;
    Some(slot)
}

fn figh_create(sim: &mut Sim, i: usize, _x: i32, _y: i32, _t: i32, _nt: i32) {
    match figh_alloc_slot(sim) {
        Some(slot) => {
            sim.parts[i].tmp = slot as i32;
        }
        None => {
            sim.kill_part(i);
        }
    }
}

fn figh_allowed(sim: &Sim, _i: usize, _x: i32, _y: i32, _t: i32) -> bool {
    sim.figh_count < crate::entity::MAX_FIGHTERS as i32
}

fn figh_change_type(sim: &mut Sim, i: usize, _x: i32, _y: i32, from: i32, to: i32) {
    if to == pt::FIGH as i32 {
        if sim.parts[i].tmp < 0 {
            match figh_alloc_slot(sim) {
                Some(slot) => sim.parts[i].tmp = slot as i32,
                None => sim.kill_part(i),
            }
        }
    } else if from == pt::FIGH as i32 {
        let slot = sim.parts[i].tmp.max(0) as usize;
        if slot < sim.fighters.len() && sim.fighters[slot].spwn != 0 {
            sim.fighters[slot].spwn = 0;
            sim.figh_count = (sim.figh_count - 1).max(0);
        }
    }
}

fn figh_update(sim: &mut Sim, i: usize, _x: i32, _y: i32, _s: i32, _nt: i32) -> i32 {
    if i >= sim.parts.len() || sim.parts[i].t == 0 {
        return 0;
    }
    let slot = sim.parts[i].tmp.max(0) as usize;
    if slot >= crate::entity::MAX_FIGHTERS {
        sim.kill_part(i);
        return 1;
    }
    // Slot-aware index safety: only the head registered for this slot acts.
    if sim.fighter_head(slot) != Some(i) {
        return 0;
    }
    let which = Entity::Fighter(slot);
    let x = (sim.parts[i].x + 0.5) as i32;
    let y = (sim.parts[i].y + 0.5) as i32;
    let (lethal, damage) = entity_hazard(sim, x, y);
    if lethal {
        fighter_die(sim, i, slot);
        return 1;
    }
    if damage > 0 {
        sim.parts[i].life -= damage;
        if sim.parts[i].life < 0 {
            fighter_die(sim, i, slot);
            return 1;
        }
    }

    // Reference target selection uses the players' torso anchor (legs[2]/[3]).
    let p1_alive = sim.player.spwn != 0;
    let p2_alive = sim.player2.spwn != 0;
    let mut target: Option<(i32, i32)> = None;
    if p1_alive && p2_alive {
        let d1 = {
            let dx = sim.player.legs[2] as i32 - x;
            let dy = sim.player.legs[3] as i32 - y;
            dx * dx + dy * dy
        };
        let d2 = {
            let dx = sim.player2.legs[2] as i32 - x;
            let dy = sim.player2.legs[3] as i32 - y;
            dx * dx + dy * dy
        };
        let p = if d1 <= d2 { &sim.player } else { &sim.player2 };
        target = Some((p.legs[2] as i32, p.legs[3] as i32));
    } else if p1_alive {
        target = Some((sim.player.legs[2] as i32, sim.player.legs[3] as i32));
    } else if p2_alive {
        target = Some((sim.player2.legs[2] as i32, sim.player2.legs[3] as i32));
    }

    let elem = {
        let e = sim.player_state(which).map(|p| p.elem).unwrap_or(0);
        if e == 0 {
            // Reference `Element_FIGH_Alloc` defaults the fighter power to DUST.
            if let Some(st) = sim.player_state_mut(which) {
                st.elem = pt::DUST as i32;
            }
            pt::DUST as i32
        } else {
            e
        }
    };
    let mut comm = 0u8;
    // `tmp2`: 0 = stay, 1 = seek (kept for parity with the reference field).
    sim.parts[i].tmp2 = if target.is_some() { 1 } else { 0 };

    if let Some((tx, ty)) = target {
        let dx = tx - x;
        let dy = ty - y;
        let d2 = dx * dx + dy * dy;
        if d2 < 600 {
            // Reference: use the element power when it is LIGH/NEUT, deadly or
            // radioactive, or has a hot/cold default temperature (>=323K/<=243K).
            // See `Element_FIGH` in the pinned reference (`FIGH.cpp`).
            let valid = elem > 0 && (elem as usize) < crate::elements_generated::PT_NUM;
            if valid {
                let def = element(elem as u16);
                let deadly_or_rad =
                    (def.properties & (props::PROP_DEADLY | props::PROP_RADIOACTIVE)) != 0;
                let dangerous = elem == pt::LIGH as i32
                    || elem == pt::NEUT as i32
                    || deadly_or_rad
                    || def.default_temp >= 323.15
                    || def.default_temp <= 243.15;
                if dangerous {
                    comm |= ent_input::SPAWN;
                }
            }
            // Facing direction is retained in close range so element powers that
            // are actually spawned (via `stickman_element_power`) travel toward
            // the targeted stickman instead of being dropped on the fighter.
            if dx >= 0 {
                comm |= ent_input::RIGHT;
            } else {
                comm |= ent_input::LEFT;
            }
            // Fallback melee when no element power is configured.
            if comm & ent_input::SPAWN == 0 {
                for (idx, p) in sim.parts.iter().enumerate() {
                    if p.t == pt::STKM || p.t == pt::STKM2 {
                        let ddx = p.x as i32 - x;
                        let ddy = p.y as i32 - y;
                        if ddx.abs() <= 1 && ddy.abs() <= 1 && sim.tick % 4 == 0 {
                            sim.parts[idx].life -= 1;
                            if sim.parts[idx].life < 0 {
                                let killed = sim.parts[idx].t;
                                sim.kill_part(idx);
                                if killed == pt::STKM {
                                    if let Some(st) = sim.player_state_mut(Entity::Player1) {
                                        st.spwn = 0;
                                    }
                                } else if killed == pt::STKM2 {
                                    if let Some(st) = sim.player_state_mut(Entity::Player2) {
                                        st.spwn = 0;
                                    }
                                }
                            }
                            break;
                        }
                    }
                }
            }
        } else if dx > 0 {
            comm |= ent_input::RIGHT;
        } else if dx < 0 {
            comm |= ent_input::LEFT;
        }
        let _ = dy;
    } else if (sim.parts[i].life + x) % 16 < 8 {
        comm |= ent_input::RIGHT;
    } else {
        comm |= ent_input::LEFT;
    }

    // Element power: spawn the fighter's configured element with the proper
    // velocity (energy particles receive facing velocity) instead of relying on
    // the generic spawn path, which would drop them on the fighter's cell.
    let mut comm_adj = comm;
    if stickman_element_power(sim, i, x, y, comm, elem) {
        comm_adj &= !ent_input::SPAWN;
    }

    // Multi-fighter separation: nudge fighters that share a cell.
    let r = sim.pmap_at(x, y);
    if r != 0 && pmap_id(r) as usize != i && pmap_type(r) == pt::FIGH {
        comm_adj |= if (i & 1) == 0 {
            ent_input::LEFT
        } else {
            ent_input::RIGHT
        };
    }

    if let Some(st) = sim.player_state_mut(which) {
        st.pcomm = st.comm;
        st.comm = comm_adj;
    }
    sim.step_entity_at(which, i);
    let moving = comm_adj & (ent_input::LEFT | ent_input::RIGHT) != 0;
    let nx = (sim.parts[i].x + 0.5) as i32;
    let ny = (sim.parts[i].y + 0.5) as i32;
    let frames = sim.player_state(which).map(|p| p.frames as i32).unwrap_or(0);
    let mut legs = sim.player_state(which).map(|p| p.legs).unwrap_or([0.0; 16]);
    stickman_set_feet_targets(sim, &mut legs, frames, nx, ny, moving);
    if let Some(st) = sim.player_state_mut(which) {
        st.legs = legs;
        stickman_legs(st, true, moving);
    }
    if i < sim.parts.len() && sim.parts[i].t == pt::FIGH {
        sim.parts[i].ctype = sim.player_state(which).map(|p| p.elem).unwrap_or(0);
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
