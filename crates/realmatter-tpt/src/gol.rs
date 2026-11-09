//! Game of Life cellular automata on `PT_LIFE` cells.

use crate::elements_generated::pt;
use crate::elements_generated::GOL_RULES;
use crate::sim::{pmap_id, pmap_type, Sim, CELL};

const NGOL: u32 = GOL_RULES.len() as u32;

/// Resolve the ruleset for a LIFE particle's `ctype` (golnum identity for
/// builtins, raw ruleset for custom rules), mirroring reference `SimulateGoL`.
#[inline]
fn resolve_ruleset(ctype: i32) -> (u32, u32) {
    let golnum = ctype as u32;
    if golnum < NGOL {
        (golnum, GOL_RULES[golnum as usize].ruleset)
    } else {
        (golnum, golnum)
    }
}

#[derive(Clone, Copy, Default)]
struct CellInfo {
    alive: bool,
    /// Species identity: builtin index (0-based) or raw custom ruleset.
    golnum: u32,
    ruleset: u32,
    dcolour: u32,
    tmp: i32,
}

/// `Simulation::SimulateGoL`.
///
/// Reference-grounded behavior implemented here: builtin/custom ruleset
/// resolution, multi-state countdown, neighbor-count birth/survival masks with
/// toroidal wrapping inside the 2*CELL border, birth from the true majority
/// species among the 8 alive neighbors (lowest golnum on ties, matching the
/// reference contest order), and inheritance of `dcolour`/`tmp` from the
/// sampled neighbor that contributed the winning species (reference
/// `createFromEntry` offset sampling).
///
/// Not implemented (requires gol grid / tick API changes): the reference's
/// per-cell 5-slot multi-species contest grid (`gol[y][x][5]`) that tracks
/// multiple competing species populations in the same empty cell across the
/// global grid. This implementation tallies the 8 neighbors directly, which
/// is sufficient for <=8 competing neighbors but is not the full 5-slot grid
/// structure (`SYS-GOL-MULTI`).
pub fn simulate_gol(sim: &mut Sim) {
    let w = sim.width as i32;
    let h = sim.height as i32;
    let inner_min = CELL as i32;
    let inner_w = w - 2 * CELL as i32;
    let inner_h = h - 2 * CELL as i32;

    let mut cells: Vec<CellInfo> = vec![CellInfo::default(); sim.width * sim.height];

    for i in 0..sim.parts.len() {
        let p = &sim.parts[i];
        if p.t != pt::LIFE {
            continue;
        }
        let x = (p.x + 0.5) as i32;
        let y = (p.y + 0.5) as i32;
        if x < inner_min || y < inner_min || x >= w - inner_min || y >= h - inner_min {
            continue;
        }
        let (golnum, ruleset) = resolve_ruleset(p.ctype);
        let states = ((ruleset >> 17) & 0xF) + 2;
        let idx = sim.idx(x, y);
        cells[idx] = CellInfo {
            alive: p.tmp2 as u32 + 1 >= states,
            golnum,
            ruleset,
            dcolour: p.dcolour,
            tmp: p.tmp,
        };
    }

    #[derive(Clone, Copy)]
    struct Tally {
        golnum: u32,
        ruleset: u32,
        count: u32,
        sample_dx: i32,
        sample_dy: i32,
    }

    let mut to_kill: Vec<usize> = Vec::new();
    let mut to_birth: Vec<(i32, i32, u32, u32, i32)> = Vec::new(); // x,y,golnum,dcolour,tmp

    for y in inner_min..h - inner_min {
        for x in inner_min..w - inner_min {
            let idx = sim.idx(x, y);
            let r = sim.pmap[idx];

            // Count alive neighbors and tally species (reference iterates yy/xx -1..1).
            let mut neighbours = 0u32;
            let mut tallies: [Option<Tally>; 8] = [None; 8];
            let mut ntally = 0usize;
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let ax = (x + dx + w - 3 * CELL as i32).rem_euclid(inner_w) + CELL as i32;
                    let ay = (y + dy + h - 3 * CELL as i32).rem_euclid(inner_h) + CELL as i32;
                    let nidx = sim.idx(ax, ay);
                    let nr = sim.pmap[nidx];
                    if nr != 0 && pmap_type(nr) != pt::LIFE {
                        continue;
                    }
                    let c = cells[nidx];
                    if !c.alive {
                        continue;
                    }
                    neighbours += 1;
                    let mut found = false;
                    for t in tallies[..ntally].iter_mut().flatten() {
                        if t.golnum == c.golnum {
                            t.count += 1;
                            found = true;
                            break;
                        }
                    }
                    if !found && ntally < tallies.len() {
                        // Store sample offset as the contributing neighbor delta
                        // (reference createFromEntry packs yy/xx of the adding
                        // neighbor; birth samples at x - xx, y - yy).
                        tallies[ntally] = Some(Tally {
                            golnum: c.golnum,
                            ruleset: c.ruleset,
                            count: 1,
                            sample_dx: dx,
                            sample_dy: dy,
                        });
                        ntally += 1;
                    }
                }
            }

            let n = neighbours.min(8);
            if r != 0 && pmap_type(r) == pt::LIFE {
                let id = pmap_id(r) as usize;
                if id >= sim.parts.len() {
                    continue;
                }
                let ruleset = cells[idx].ruleset;
                let stay = ruleset & 0x1FF;
                if (stay >> n) & 1 == 0 {
                    to_kill.push(id);
                }
            } else if r == 0 && neighbours > 0 {
                // True majority contest (reference): winner needs count >=
                // ceil(neighbours/2) and birth bit set; lowest golnum wins ties.
                let majority = neighbours / 2 + neighbours % 2;
                let mut best: Option<Tally> = None;
                for t in tallies[..ntally].iter().flatten() {
                    let begin = (t.ruleset >> 9) & 0x1FF;
                    if (begin >> n) & 1 == 0 || t.count < majority {
                        continue;
                    }
                    if best.map_or(true, |b| t.golnum < b.golnum) {
                        best = Some(*t);
                    }
                }
                if let Some(b) = best {
                    let ax = (x - b.sample_dx + w - 3 * CELL as i32).rem_euclid(inner_w) + CELL as i32;
                    let ay = (y - b.sample_dy + h - 3 * CELL as i32).rem_euclid(inner_h) + CELL as i32;
                    let sidx = sim.idx(ax, ay);
                    let sample = cells[sidx];
                    to_birth.push((x, y, b.golnum, sample.dcolour, sample.tmp));
                }
            }
        }
    }

    for id in to_kill {
        if id >= sim.parts.len() || sim.parts[id].t != pt::LIFE {
            continue;
        }
        sim.parts[id].tmp2 -= 1;
        if sim.parts[id].tmp2 < 0 {
            sim.kill_part(id);
        }
    }
    for (x, y, golnum, dcolour, tmp) in to_birth {
        if sim.pmap[sim.idx(x, y)] != 0 {
            continue;
        }
        if let Some(i) = sim.create_part(x, y, pt::LIFE) {
            let ruleset = if golnum < NGOL {
                GOL_RULES[golnum as usize].ruleset
            } else {
                golnum
            };
            let states = ((ruleset >> 17) & 0xF) + 2;
            sim.parts[i].tmp2 = states as i32 - 1;
            // Store builtin identity index or raw custom ruleset, matching
            // reference `LIFE::create` / birth (`golnumToCreate`).
            sim.parts[i].ctype = golnum as i32;
            sim.parts[i].dcolour = dcolour;
            sim.parts[i].tmp = tmp;
        }
    }
    sim.gol_counter = 0;
}
