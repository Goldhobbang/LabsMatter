//! Game of Life cellular automata on `PT_LIFE` cells.
//!
//! Reference: `Simulation::SimulateGoL` in `Simulation.cpp:3443-3618`.

use crate::elements_generated::pt;
use crate::elements_generated::GOL_RULES;
use crate::elements_generated::WALLS;
use crate::sim::{pmap_id, pmap_type, Sim, CELL};

const NGOL: u32 = GOL_RULES.len() as u32;

// Wall id for DEFAULT_WL_STASIS (elements_generated: "DEFAULT_WL_STASIS" id 18).
const WL_STASIS: u8 = 18;

#[inline]
fn is_stasis_frozen(sim: &Sim, x: i32, y: i32) -> bool {
    debug_assert!(WALLS[WL_STASIS as usize].identifier == "DEFAULT_WL_STASIS");
    if sim.wall_at(x, y) != WL_STASIS {
        return false;
    }
    let cx = x / CELL as i32;
    let cy = y / CELL as i32;
    if !sim.in_cell_bounds(cx, cy) {
        return false;
    }
    // emap is per-cell; stasis frozen when emap < 8 (reference Simulation.cpp).
    sim.emap[sim.cell_idx(cx, cy)] < 8
}

/// `Simulation::SimulateGoL` (reference `Simulation.cpp:3443-3618`).
pub fn simulate_gol(sim: &mut Sim) {
    let width = sim.width as i32;
    let height = sim.height as i32;
    let inner_min = CELL as i32;
    let inner_max_x = width - inner_min;
    let inner_max_y = height - inner_min;
    let inner_w = width - 2 * inner_min;
    let inner_h = height - 2 * inner_min;

    // Scratch neighbour list: persistent in reference as `gol[YRES][XRES][5]`,
    // zeroed as consumed. Local zeroed per call starts each call zeroed.
    let mut gol: Vec<[u32; 5]> = vec![[0u32; 5]; sim.width * sim.height];

    // PASS 1 — per LIFE particle (reference Simulation.cpp:3463-3497).
    let part_count = sim.parts.len();
    for i in 0..part_count {
        if sim.parts[i].t != pt::LIFE {
            continue;
        }
        let x = (sim.parts[i].x + 0.5) as i32;
        let y = (sim.parts[i].y + 0.5) as i32;
        if x < inner_min || y < inner_min || x >= inner_max_x || y >= inner_max_y {
            continue;
        }
        let golnum0 = sim.parts[i].ctype as u32;
        let mut ruleset = golnum0;
        let mut golnum = golnum0;
        if golnum < NGOL {
            ruleset = GOL_RULES[golnum as usize].ruleset;
            golnum += 1; // +1 bias for builtins in neighbour list
        }
        let alive_threshold = (((ruleset >> 17) & 0xF) as i32) + 1;
        if sim.parts[i].tmp2 as i32 == alive_threshold {
            for yy in -1i32..=1 {
                for xx in -1i32..=1 {
                    if xx == 0 && yy == 0 {
                        continue;
                    }
                    // Toroidal wrap inside 2*CELL border (C++ % on non-negative value).
                    let ax = ((x + xx + width - 3 * inner_min) % inner_w) + inner_min;
                    let ay = ((y + yy + height - 3 * inner_min) % inner_h) + inner_min;
                    let aidx = sim.idx(ax, ay);
                    let r = sim.pmap[aidx];
                    if r != 0 && pmap_type(r) != pt::LIFE {
                        continue;
                    }
                    gol[aidx][0] = gol[aidx][0].wrapping_add(1 << 28);
                    for l in 0..5 {
                        let nr = gol[aidx][l] & 0x001F_FFFF;
                        if nr == golnum {
                            gol[aidx][l] = gol[aidx][l].wrapping_add(1 << 21);
                            break;
                        }
                        if nr == 0 {
                            gol[aidx][l] =
                                ((yy as u32 & 3) << 26) | ((xx as u32 & 3) << 24) | golnum;
                            break;
                        }
                    }
                }
            }
        } else {
            // Dying/non-alive-countdown state: decrement unless stasis-frozen.
            if !is_stasis_frozen(sim, x, y) && i < sim.parts.len() && sim.parts[i].t == pt::LIFE {
                sim.parts[i].tmp2 -= 1;
            }
        }
    }

    // PASS 2 — all cells (reference Simulation.cpp:3499-3593).
    for y in inner_min..inner_max_y {
        for x in inner_min..inner_max_x {
            let idx = sim.idx(x, y);
            let r = sim.pmap[idx];
            if r != 0 && pmap_type(r) != pt::LIFE {
                continue;
            }
            let nl0 = gol[idx][0];
            if r == 0 && nl0 == 0 {
                continue;
            }

            let neighbours = if nl0 != 0 { ((nl0 >> 28) & 7) + 1 } else { 0 };
            if !is_stasis_frozen(sim, x, y) {
                if r != 0 {
                    // Existing LIFE: survival check.
                    let id = pmap_id(r) as usize;
                    if id < sim.parts.len() && sim.parts[id].t == pt::LIFE {
                        let mut ruleset = sim.parts[id].ctype as u32;
                        if ruleset < NGOL {
                            ruleset = GOL_RULES[ruleset as usize].ruleset;
                        }
                        let alive_threshold = (((ruleset >> 17) & 0xF) as i32) + 1;
                        if ((ruleset >> neighbours) & 1) == 0
                            && sim.parts[id].tmp2 as i32 == alive_threshold
                        {
                            sim.parts[id].tmp2 -= 1;
                        }
                    }
                } else {
                    // Birth: exact contest (reference Simulation.cpp:3519-3582).
                    let mut golnum_to_create: u32 = u32::MAX;
                    let mut create_from_entry: u32 = 0;
                    let majority = neighbours / 2 + neighbours % 2;
                    for l in 0..5 {
                        let entry = gol[idx][l];
                        let mut golnum_l = entry & 0x001F_FFFF;
                        if golnum_l == 0 {
                            break;
                        }
                        let mut ruleset_l = golnum_l;
                        if golnum_l.wrapping_sub(1) < NGOL {
                            ruleset_l = GOL_RULES[(golnum_l - 1) as usize].ruleset;
                            golnum_l -= 1;
                        }
                        let pop = ((entry >> 21) & 7) + 1;
                        if ((ruleset_l >> (neighbours + 8)) & 1) != 0
                            && pop >= majority
                            && golnum_l < golnum_to_create
                        {
                            golnum_to_create = golnum_l;
                            create_from_entry = entry;
                        }
                    }
                    if golnum_to_create != u32::MAX {
                        if let Some(new_id) = sim.create_part(x, y, pt::LIFE) {
                            // Decode contributing neighbour offset from createFromEntry.
                            let mut xx = ((create_from_entry >> 24) & 3) as i32;
                            let mut yy = ((create_from_entry >> 26) & 3) as i32;
                            if xx == 3 { xx = -1; }
                            if yy == 3 { yy = -1; }
                            let ax = ((x - xx + width - 3 * inner_min) % inner_w) + inner_min;
                            let ay = ((y - yy + height - 3 * inner_min) % inner_h) + inner_min;
                            // Sample dcolour/tmp from parts[ID(pmap[ay][ax])] like reference.
                            let aidx = sim.idx(ax, ay);
                            let sr = sim.pmap[aidx];
                            let mut sdcolour = 0u32;
                            let mut stmp = 0i32;
                            if sr != 0 && pmap_type(sr) == pt::LIFE {
                                let sid = pmap_id(sr) as usize;
                                if sid < sim.parts.len() && sim.parts[sid].t == pt::LIFE {
                                    sdcolour = sim.parts[sid].dcolour;
                                    stmp = sim.parts[sid].tmp;
                                }
                            }
                            // Reference create_part(-1,x,y,LIFE, golnumToCreate|0x200000):
                            // LIFE create sets ctype = v & 0x1FFFFF, tmp2 = ((ruleset>>17)&0xF)+1
                            // and skips colour lookup because of 0x200000. create_part here
                            // uses create_fn with v=0, so set fields explicitly.
                            let birth_ruleset = if golnum_to_create < NGOL {
                                GOL_RULES[golnum_to_create as usize].ruleset
                            } else {
                                golnum_to_create
                            };
                            if new_id < sim.parts.len() {
                                sim.parts[new_id].ctype = golnum_to_create as i32;
                                sim.parts[new_id].tmp2 = (((birth_ruleset >> 17) & 0xF) + 1) as i32;
                                sim.parts[new_id].dcolour = sdcolour;
                                sim.parts[new_id].tmp = stmp;
                            }
                        }
                    }
                }
            }
            // Clear consumed entries (reference Simulation.cpp:3586-3592).
            for l in 0..5 {
                if gol[idx][l] == 0 {
                    break;
                }
                gol[idx][l] = 0;
            }
        }
    }

    // PASS 3 — kill LIFE with tmp2 <= 0 (reference Simulation.cpp:3595-3616).
    for y in inner_min..inner_max_y {
        for x in inner_min..inner_max_x {
            let idx = sim.idx(x, y);
            let r = sim.pmap[idx];
            if r != 0 && pmap_type(r) == pt::LIFE {
                let id = pmap_id(r) as usize;
                if id < sim.parts.len() && sim.parts[id].t == pt::LIFE && sim.parts[id].tmp2 <= 0 {
                    sim.kill_part(id);
                }
            }
        }
    }

    sim.gol_counter = 0;
}
