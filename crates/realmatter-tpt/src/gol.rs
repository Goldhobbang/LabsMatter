//! Game of Life cellular automata on `PT_LIFE` cells.

use crate::element::element;
use crate::elements_generated::pt;
use crate::elements_generated::GOL_RULES;
use crate::sim::{pmap_id, pmap_type, Sim, CELL};

/// `Simulation::SimulateGoL` for a single species per cell.
///
/// The reference supports up to five competing species per cell with a
/// neighbour list; LabsMatter implements the population rule (birth/survival
/// masks) and multi-state countdown, and records the multi-species contest as
/// not yet implemented in `docs/tpt-parity/status.md` (`SYS-GOL-MULTI`).
pub fn simulate_gol(sim: &mut Sim) {
    let w = sim.width as i32;
    let h = sim.height as i32;
    let inner_min = CELL as i32;
    let inner_w = w - 2 * CELL as i32;

    // Snapshot of "alive" state per GoL cell.
    let mut counts: Vec<u32> = vec![0; sim.width * sim.height];
    let mut rules: Vec<u32> = vec![0; sim.width * sim.height];
    let mut alive: Vec<bool> = vec![false; sim.width * sim.height];

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
        let kind = p.ctype;
        let ruleset = if kind >= 0 && (kind as usize) < GOL_RULES.len() {
            GOL_RULES[kind as usize].ruleset
        } else {
            kind as u32
        };
        let states = ((ruleset >> 17) & 0xF) + 2;
        let idx = sim.idx(x, y);
        rules[idx] = ruleset;
        alive[idx] = p.tmp2 as u32 + 1 >= states;
    }

    // Count neighbours (wrapping inside the 2*CELL border, as the reference).
    for y in inner_min..h - inner_min {
        for x in inner_min..w - inner_min {
            let idx = sim.idx(x, y);
            if !alive[idx] && rules[idx] == 0 {
                // Only empty cells can later be born; still needs counts.
            }
            let mut n = 0u32;
            for dy in -1i32..2 {
                for dx in -1i32..2 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let ax = (x + dx + w - 3 * CELL as i32).rem_euclid(w - 2 * CELL as i32) + CELL as i32;
                    let ay = (y + dy + h - 3 * CELL as i32).rem_euclid(h - 2 * CELL as i32) + CELL as i32;
                    let nidx = sim.idx(ax, ay);
                    let r = sim.pmap[nidx];
                    if r != 0 && pmap_type(r) != pt::LIFE {
                        continue;
                    }
                    if alive[nidx] {
                        n += 1;
                    }
                }
            }
            counts[idx] = n;
        }
    }

    // Apply survival / birth.
    let mut to_kill: Vec<usize> = Vec::new();
    let mut to_birth: Vec<(i32, i32, u32)> = Vec::new();
    for y in inner_min..h - inner_min {
        for x in inner_min..w - inner_min {
            let idx = sim.idx(x, y);
            let r = sim.pmap[idx];
            let neighbours = counts[idx];
            let n = neighbours.min(8);
            if r != 0 && pmap_type(r) == pt::LIFE {
                let id = pmap_id(r) as usize;
                if id >= sim.parts.len() {
                    continue;
                }
                let ruleset = rules[idx];
                let stay = ruleset & 0x1FF;
                if (stay >> n) & 1 == 0 {
                    to_kill.push(id);
                }
            } else if r == 0 && neighbours > 0 {
                // Birth: find the majority neighbour ruleset.
                let mut best = 0u32;
                let mut best_count = 0u32;
                for dy in -1i32..2 {
                    for dx in -1i32..2 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let ax = (x + dx + w - 3 * CELL as i32).rem_euclid(w - 2 * CELL as i32)
                            + CELL as i32;
                        let ay = (y + dy + h - 3 * CELL as i32).rem_euclid(h - 2 * CELL as i32)
                            + CELL as i32;
                        let nidx = sim.idx(ax, ay);
                        if alive[nidx] {
                            let rr = rules[nidx];
                            let cnt = 1;
                            if cnt > best_count {
                                best = rr;
                                best_count = cnt;
                            }
                        }
                    }
                }
                if best != 0 {
                    let begin = (best >> 9) & 0x1FF;
                    if (begin >> n) & 1 != 0 {
                        to_birth.push((x, y, best));
                    }
                }
            }
        }
    }

    for id in to_kill {
        sim.parts[id].tmp2 -= 1;
        if sim.parts[id].tmp2 < 0 {
            sim.kill_part(id);
        }
    }
    for (x, y, ruleset) in to_birth {
        if sim.pmap[sim.idx(x, y)] != 0 {
            continue;
        }
        if let Some(i) = sim.create_part(x, y, pt::LIFE) {
            let states = ((ruleset >> 17) & 0xF) + 2;
            sim.parts[i].tmp2 = states as i32 - 1;
            // Store the raw ruleset in ctype for custom rules.
            sim.parts[i].ctype = ruleset as i32;
            let _ = element(pt::LIFE);
        }
    }
    sim.gol_counter = 0;
}
