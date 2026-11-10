//! Shared transport primitives: piston stacks and soap links.
//!
//! Both behaviours are declared in element sources (`PSTN.cpp`, `SOAP.cpp`) but
//! require world-level operations that must not be re-implemented inside an
//! element update:
//!
//! - `PSTN` needs to displace an ordered stack of particles through the world
//!   without deleting anything (`CanMoveStack` / `MoveStack`).
//! - `SOAP` needs persistent particle-to-particle links (`tmp`/`tmp2` indices)
//!   with attach/detach and a tension solve.
//!
//! Agent 1 owns these operations; Agent 3 calls them from the element handlers.
//!
//! ## Reference trace (PSTN.cpp / SOAP.cpp)
//!
//! | ref (quoted)                                                                                                             | ref line      | ours                        | verdict |
//! |--------------------------------------------------------------------------------------------------------------------------|---------------|-----------------------------|---------|
//! | `struct StackData { int pushed; int spaces; ... }`                                                                        | PSTN.cpp:53-62| `StackProbe`                | match (reshape: `pushed` count + `slots` walk record) |
//! | `CanMoveStack(..., bool retract, int block)`; `block && TYP(r) == block`                                                   | PSTN.cpp:234,246| `can_move_stack(.., retract, block)` | fixed (params added; `block` is a parameter, not a `StackData` field) |
//! | `IsWallBlocking(posX, posY, 0)`                                                                                           | PSTN.cpp:246  | `self.wall_blocks(.., 0)`   | fixed (was `wall_at(..) != 0`, which also blocked WL_DETECT / active WL_EWALL) |
//! | `currentPos < maxSize + amount && currentPos < XRES - 1` (particles AND spaces counted)                                     | PSTN.cpp:236  | loop head on `slots.len()`  | fixed (was particle-only count, no XRES-1 cap) |
//! | `currentPos - spaces < maxSize && (!retract || (TYP(r) == PT_FRME && posX == stackX && posY == stackY))`                    | PSTN.cpp:255-256| push guard                 | fixed (was push-then-stop past maxSize; no retract mode) |
//! | MoveStack extend: `possibleMovement++` per trailing free slot, each particle moves by the free slots ahead of it            | PSTN.cpp:382-400| `staggered_stack_move`      | fixed (was uniform shift by `min(spaces, amount)`) |
//! | MoveStack FRME head branch at `callDepth` 0, `sticky = !parts[..].tmp`, arm kill `j = 1..=amount`                           | PSTN.cpp:270-341 | `move_stack_ref`           | added (`sticky`/`callDepth` had no representation) |
//! | MoveStack retract branch: `(r) && TYP(r) != block && (sticky || TYP(r) != PT_FRME)` walk + unconditional backward move       | PSTN.cpp:342-378 | `move_stack_ref` retract   | added |
//! | `cancelOnObstacle`: FRME scan aborts with `return 0` / `amount = spaces`                                                    | PSTN.cpp:288-294,306-312 | `move_stack_ref`      | added |
//! | velocities: MoveStack never writes `vx`/`vy`                                                                                | PSTN.cpp:265-411 | untouched                  | match |
//! | arm create: `create_part(-3, pistonEndX + (nxi*j), ...)`; failure continues the loop; new arm `life = 1` + dcolour darken    | PSTN.cpp:190-206 | `extend_piston_arm(_full)` | fixed (was break-on-failure; dcolour port added) |
//! | arm kill: `kill_part(ID(sim->pmap[stack - j*dir]))`, exactly `amount` cells, no type check                                   | PSTN.cpp:333-335,367-369 | `retract_piston_arm` | deviation kept by design: we stop at the first non-`PSTN` cell (the reference would kill `ID(0)` on an empty cell) |
//! | `Element_SOAP_neighourLoop` wrap arithmetic `dx >= (XRES/2 - CELL)) dx -= XRES - 2*CELL`                                    | SOAP.cpp:8-17 | `soap_link_step` + updates.rs `soap_delta` | match |
//! | `validIndex(int i) { return i >= 0 && i < NPART; }`                                                                          | SOAP.cpp:68-70 | `i >= 0 && i < parts.len()` | match (parts pool is our NPART) |
//! | `Element_SOAP_detach`: front xor 0x04 on `tmp`, back xor 0x02 on `tmp2`, then `parts[i].ctype = 0` (tmp/tmp2 left dangling)   | SOAP.cpp:73-88 | `soap_detach`              | match (order kept) |
//! | `attach`: `i1.ctype |= 2; i1.tmp = i2; i2.ctype |= 4; i2.tmp2 = i1;` (mirror for the back slot)                              | SOAP.cpp:90-103 | `soap_attach`             | match (write order kept) |
//! | `d = 9/(pow(dx,2)+pow(dy,2)+9)-0.5` (primary); `81/(pow+81)-0.5` with `*0.5f` (secondary, applied to `tmp`/`ii` only)         | SOAP.cpp:247-270 | `soap_constraint` + updates.rs | match (secondary solve lives in updates.rs, reported there) |
//! | RNG draws in PSTN.cpp / SOAP.cpp                                                                                            | —             | none                        | match (0 draws both sides) |

use crate::elements_generated::pt;
use crate::sim::{pmap_id, pmap_pack, pmap_type, Sim, CELL};

/// Reference `MAX_FRAME` (PSTN.cpp:53):
/// `constexpr int MAX_FRAME = 0x0F;`
const MAX_FRAME: i32 = 0x0F;

/// Result of a stack probe. `pushed` are the particle slots that can move,
/// `spaces` is how many free cells exist ahead of them.
///
/// Reference `StackData` (PSTN.cpp:53-62):
/// `struct StackData { int pushed; int spaces; ... }` — `pushed` is
/// `currentPos - spaces`, the number of collected particles. The reference also
/// threads a `block` type filter *parameter* through `CanMoveStack`/`MoveStack`
/// (`static StackData CanMoveStack(..., bool retract, int block)`,
/// PSTN.cpp:7/234); it is not a field of `StackData`. `slots` mirrors the
/// reference `Element_PSTN_tempParts` scratch array (PSTN.cpp:249/258): `-1`
/// marks a free cell, anything else is a particle id, in walk order.
#[derive(Clone, Debug, Default)]
pub struct StackProbe {
    pub slots: Vec<i32>,
    pub pushed: i32,
    pub spaces: i32,
}

impl Sim {
    /// Reference `CanMoveStack` (PSTN.cpp:234-264):
    ///
    /// ```text
    /// static StackData CanMoveStack(Simulation * sim, int stackX, int stackY,
    ///     int directionX, int directionY, int maxSize, int amount, bool retract, int block)
    /// ...
    /// for (posX = stackX, posY = stackY; currentPos < maxSize + amount && currentPos < XRES - 1; ...)
    /// {
    ///     if (!(posX < XRES && posY < YRES && posX >= 0 && posY >= 0)) break;
    ///     int r = sim->pmap[posY][posX];
    ///     if (sim->IsWallBlocking(posX, posY, 0) || (block && TYP(r) == block))
    ///         return StackData(currentPos - spaces, spaces);
    ///     if (!r) { spaces++; ...tempParts[currentPos++] = -1; if (spaces >= amount) break; }
    ///     else {
    ///         if (currentPos - spaces < maxSize && (!retract || (TYP(r) == PT_FRME && posX == stackX && posY == stackY)))
    ///             sim->Element_PSTN_tempParts[currentPos++] = ID(r);
    ///         else return StackData(currentPos - spaces, spaces);
    ///     }
    /// }
    /// ```
    ///
    /// `block` is the piston's `ctype` type filter — particles of that type stop
    /// the walk untouched (PSTN.cpp:246). `retract` only lets a `FRME` at the
    /// very first walked cell be collected (PSTN.cpp:255-256). The wall test is
    /// `IsWallBlocking(posX, posY, 0)`, i.e. evaluated for element 0
    /// (PSTN.cpp:246): this drives `wall_blocks(.., 0)`, which lets WL_DETECT
    /// and electrically-opened WL_EWALL pass, unlike a bare `wall_at(..) != 0`.
    pub fn can_move_stack(
        &self,
        stack_x: i32,
        stack_y: i32,
        dir_x: i32,
        dir_y: i32,
        max_size: i32,
        amount: i32,
        retract: bool,
        block: i32,
    ) -> StackProbe {
        let mut probe = StackProbe::default();
        if amount <= 0 {
            // ref: PSTN.cpp:236-238 — `if (amount <= 0) return StackData(0, 0);`
            return probe;
        }
        let mut pos_x = stack_x;
        let mut pos_y = stack_y;
        // ref: PSTN.cpp:236 — the loop bound counts particles AND spaces:
        // `currentPos < maxSize + amount && currentPos < XRES - 1`
        let xres_cap = self.width as i32 - 1;
        while (probe.slots.len() as i32) < max_size + amount && (probe.slots.len() as i32) < xres_cap
        {
            if !self.in_bounds(pos_x, pos_y) {
                // ref: PSTN.cpp:240-242
                break;
            }
            let r = self.pmap_at(pos_x, pos_y);
            let typ = if r != 0 { pmap_type(r) } else { 0 };
            if self.wall_blocks(pos_x, pos_y, 0) || (block != 0 && typ == block as u16) {
                // ref: PSTN.cpp:245-247 — blocked walk keeps what it has:
                // `return StackData(currentPos - spaces, spaces);`
                probe.pushed = probe.slots.len() as i32 - probe.spaces;
                return probe;
            }
            if r == 0 {
                // ref: PSTN.cpp:248-253
                probe.spaces += 1;
                probe.slots.push(-1);
                if probe.spaces >= amount {
                    break;
                }
            } else {
                // ref: PSTN.cpp:254-261 — the particle is rejected (not
                // collected) as soon as the pushed count reaches `maxSize`,
                // and during retract only a `FRME` at the first cell counts.
                let pushed_count = probe.slots.len() as i32 - probe.spaces;
                if pushed_count < max_size
                    && (!retract || (typ == pt::FRME && pos_x == stack_x && pos_y == stack_y))
                {
                    probe.slots.push(pmap_id(r) as i32);
                } else {
                    probe.pushed = pushed_count;
                    return probe;
                }
            }
            pos_x += dir_x;
            pos_y += dir_y;
        }
        probe.pushed = probe.slots.len() as i32 - probe.spaces;
        probe
    }

    /// Extend-branch displacement shared by `move_stack` and `move_stack_ref`
    /// (ref: PSTN.cpp:375-410). Walking the probe from the far end,
    /// `possibleMovement` is incremented for every trailing free slot and each
    /// particle moves forward by exactly the number of free slots ahead of it,
    /// so interleaved gaps compress the stack (a uniform shift would overshoot).
    /// Velocities are never touched (ref: PSTN.cpp:265-411 — no `vx`/`vy`
    /// writes anywhere in `MoveStack`).
    fn staggered_stack_move(&mut self, probe: &StackProbe, dir_x: i32, dir_y: i32) -> (i32, bool) {
        let mut possible = 0i32;
        let mut moved_any = false;
        for &slot in probe.slots.iter().rev() {
            if slot < 0 {
                // ref: PSTN.cpp:383-387 — `if (jP < 0) { possibleMovement++; continue; }`
                possible += 1;
                continue;
            }
            if possible == 0 {
                // ref: PSTN.cpp:388-390 — `if (!possibleMovement) continue;`
                continue;
            }
            let id = slot as usize;
            if id >= self.parts.len() || self.parts[id].t == 0 {
                // Defensive: the reference assumes the probed particle is still
                // alive at this point.
                continue;
            }
            moved_any = true;
            // ref: PSTN.cpp:392-399 —
            // `int destX = srcX+directionX*possibleMovement, destY = srcY+directionY*possibleMovement;`
            let src_x = (self.parts[id].x + 0.5) as i32;
            let src_y = (self.parts[id].y + 0.5) as i32;
            let dst_x = src_x + dir_x * possible;
            let dst_y = src_y + dir_y * possible;
            if !self.in_bounds(dst_x, dst_y) {
                // Unreachable for probed slots (every destination is inside the
                // walked window); the reference writes without checking.
                continue;
            }
            let src_cell = self.idx(src_x, src_y);
            if self.pmap_at(src_x, src_y) != 0 && pmap_id(self.pmap[src_cell]) as usize == id {
                self.pmap[src_cell] = 0;
            }
            self.parts[id].x = dst_x as f32;
            self.parts[id].y = dst_y as f32;
            let dst_cell = self.idx(dst_x, dst_y);
            self.pmap[dst_cell] = pmap_pack(id as u32, self.parts[id].t);
        }
        (possible, moved_any)
    }

    /// Extend-only entry point kept for the current `updates.rs` PSTN flow:
    /// reference extend path with `block = 0` and no `cancelOnObstacle`; the
    /// FRME frame is handled by the caller (`push_frme_component`), which is
    /// why the `callDepth`-0 FRME head branch of `MoveStack` is not used here.
    /// Returns how many cells the front moved, or 0 when nothing moved so the
    /// caller's "empty space ahead" arm-creation fallback still fires.
    /// `move_stack_ref` is the reference-complete entry point.
    pub fn move_stack(
        &mut self,
        stack_x: i32,
        stack_y: i32,
        dir_x: i32,
        dir_y: i32,
        max_size: i32,
        amount: i32,
    ) -> i32 {
        let probe = self.can_move_stack(stack_x, stack_y, dir_x, dir_y, max_size, amount, false, 0);
        let (possible, moved_any) = self.staggered_stack_move(&probe, dir_x, dir_y);
        if moved_any {
            possible
        } else {
            0
        }
    }

    /// Reference `MoveStack` (PSTN.cpp:265-411):
    ///
    /// ```text
    /// static int MoveStack(Simulation * sim, int stackX, int stackY,
    ///     int directionX, int directionY, int maxSize, int amount,
    ///     bool retract, int block, bool sticky, int callDepth, bool cancelOnObstacle)
    /// ```
    ///
    /// - FRME head branch at `callDepth == 0` (PSTN.cpp:269-341): the piston
    ///   pushing/retracting whole frame rows; each FRME cell is re-issued as a
    ///   sub-move with `callDepth = 1` and `sticky = !parts[ID(..)].tmp`.
    /// - retract branch (PSTN.cpp:342-378): kills the arm (`callDepth == 0`
    ///   only) and drags `sticky`-linked particles backward by `amount`.
    /// - extend branch (PSTN.cpp:375-410): probed stack compression.
    ///
    /// `block` is the piston `ctype` filter, `cancelOnObstacle` aborts before
    /// moving anything when the FRME scan hits an obstacle (PSTN.cpp:288-294
    /// and 306-312), and `amount` is clamped to the observed free space there.
    pub fn move_stack_ref(
        &mut self,
        stack_x: i32,
        stack_y: i32,
        dir_x: i32,
        dir_y: i32,
        max_size: i32,
        amount: i32,
        retract: bool,
        block: i32,
        sticky: bool,
        call_depth: i32,
        cancel_on_obstacle: bool,
    ) -> i32 {
        let mut amount = amount;
        let stack_r = if self.in_bounds(stack_x, stack_y) {
            self.pmap_at(stack_x, stack_y)
        } else {
            0
        };
        if call_depth == 0 && stack_r != 0 && pmap_type(stack_r) == pt::FRME {
            // ref: PSTN.cpp:270-341 —
            // `int newY = !!directionX, newX = !!directionY;`
            // `int realDirectionX = retract ? -directionX : directionX;` ...
            let new_y = i32::from(dir_x != 0);
            let new_x = i32::from(dir_y != 0);
            let real_dir_x = if retract { -dir_x } else { dir_x };
            let real_dir_y = if retract { -dir_y } else { dir_y };
            let mut max_right = MAX_FRAME;
            let mut max_left = MAX_FRAME;
            // Forward scan: `for (int c = retract; c < MAX_FRAME; c++)`
            // (PSTN.cpp:279) — retracting starts past the stack cell.
            let mut c = i32::from(retract);
            while c < MAX_FRAME {
                let (px, py) = (stack_x + c * new_x, stack_y + c * new_y);
                let r = self.pmap_at(px, py);
                if self.in_bounds(px, py) && r != 0 && pmap_type(r) == pt::FRME {
                    let spaces = self
                        .can_move_stack(px, py, real_dir_x, real_dir_y, max_size, amount, retract, block)
                        .spaces;
                    if spaces < amount {
                        // ref: PSTN.cpp:287-294 — obstacle in the frame row:
                        // `if (cancelOnObstacle) return 0; amount = spaces;`
                        if cancel_on_obstacle {
                            return 0;
                        }
                        amount = spaces;
                    }
                    c += 1;
                } else {
                    // ref: PSTN.cpp:295-298 — `maxRight = c; break;`
                    max_right = c;
                    break;
                }
            }
            // Backward scan: `for (int c = 1; c < MAX_FRAME; c++)`
            // (PSTN.cpp:299).
            let mut c = 1;
            while c < MAX_FRAME {
                let (px, py) = (stack_x - c * new_x, stack_y - c * new_y);
                let r = self.pmap_at(px, py);
                if self.in_bounds(px, py) && r != 0 && pmap_type(r) == pt::FRME {
                    let spaces = self
                        .can_move_stack(px, py, real_dir_x, real_dir_y, max_size, amount, retract, block)
                        .spaces;
                    if spaces < amount {
                        // ref: PSTN.cpp:304-312
                        if cancel_on_obstacle {
                            return 0;
                        }
                        amount = spaces;
                    }
                    c += 1;
                } else {
                    // ref: PSTN.cpp:313-316 — `maxLeft = c; break;`
                    max_left = c;
                    break;
                }
            }
            // ref: PSTN.cpp:319-327 — "If the piston is pushing frame, iterate
            // out from the centre to the edge and push everything resting on
            // frame": `MoveStack(sim, posX, posY, directionX, directionY,
            // maxSize, amount, retract, block, !parts[ID(...)].tmp, 1, cancelOnObstacle);`
            for c in 1..max_right.max(0) {
                let (px, py) = (stack_x + c * new_x, stack_y + c * new_y);
                let r = self.pmap_at(px, py);
                if r == 0 || pmap_type(r) != pt::FRME {
                    continue; // scan-verified FRME; re-checked defensively
                }
                let fid = pmap_id(r) as usize;
                let sub_sticky = fid < self.parts.len() && self.parts[fid].tmp == 0;
                self.move_stack_ref(
                    px, py, dir_x, dir_y, max_size, amount, retract, block, sub_sticky, 1,
                    cancel_on_obstacle,
                );
            }
            for c in 1..max_left.max(0) {
                let (px, py) = (stack_x - c * new_x, stack_y - c * new_y);
                let r = self.pmap_at(px, py);
                if r == 0 || pmap_type(r) != pt::FRME {
                    continue;
                }
                let fid = pmap_id(r) as usize;
                let sub_sticky = fid < self.parts.len() && self.parts[fid].tmp == 0;
                self.move_stack_ref(
                    px, py, dir_x, dir_y, max_size, amount, retract, block, sub_sticky, 1,
                    cancel_on_obstacle,
                );
            }
            if retract {
                // ref: PSTN.cpp:331-335 —
                // "Remove arm section if retracting with FRME":
                // `for (int j = 1; j <= amount; j++) sim->kill_part(ID(sim->pmap[stackY+(directionY * -j)][stackX+(directionX * -j)]));`
                self.kill_arm_behind(stack_x, stack_y, dir_x, dir_y, amount);
            }
            // ref: PSTN.cpp:337-338 — tail call with
            // `sticky = !parts[ID(pmap[stackY][stackX])].tmp` at `callDepth = 1`.
            let tail_id = pmap_id(stack_r) as usize;
            let tail_sticky = tail_id < self.parts.len() && self.parts[tail_id].tmp == 0;
            return self.move_stack_ref(
                stack_x, stack_y, dir_x, dir_y, max_size, amount, retract, block, tail_sticky, 1,
                cancel_on_obstacle,
            );
        }
        if retract {
            // ref: PSTN.cpp:342-378 — retract branch.
            let mut found_parts = false;
            if call_depth == 0 {
                // ref: PSTN.cpp:344-348 —
                // "Remove arm section if retracting without FRME":
                // `if (!callDepth) for (int j = 1; j <= amount; j++) sim->kill_part(...)`
                self.kill_arm_behind(stack_x, stack_y, dir_x, dir_y, amount);
            }
            let mut slots: Vec<i32> = Vec::new();
            let (mut px, mut py) = (stack_x, stack_y);
            let xres_cap = self.width as i32 - 1;
            while (slots.len() as i32) < max_size && (slots.len() as i32) < xres_cap {
                if !self.in_bounds(px, py) {
                    // ref: PSTN.cpp:351-355
                    break;
                }
                let r = self.pmap_at(px, py);
                if r == 0 {
                    break;
                }
                let typ = pmap_type(r);
                if block != 0 && typ == block as u16 {
                    // ref: PSTN.cpp:356-359 — `(!r || TYP(r) == block || (!sticky && TYP(r) != PT_FRME)) → break`
                    break;
                }
                if !sticky && typ != pt::FRME {
                    break;
                }
                let id = pmap_id(r) as usize;
                if id >= self.parts.len() {
                    break;
                }
                found_parts = true;
                slots.push(id as i32);
                px += dir_x;
                py += dir_y;
            }
            if found_parts {
                // ref: PSTN.cpp:362-377 — "Move particles":
                // `int destX = srcX-directionX*amount, destY = srcY-directionY*amount;`
                // with no destination check and no free-space logic — the arm
                // killed above is the room being reclaimed.
                for &sid in &slots {
                    let id = sid as usize;
                    let src_x = (self.parts[id].x + 0.5) as i32;
                    let src_y = (self.parts[id].y + 0.5) as i32;
                    let dst_x = src_x - dir_x * amount;
                    let dst_y = src_y - dir_y * amount;
                    if !self.in_bounds(dst_x, dst_y) {
                        continue; // defensive: reference writes unchecked
                    }
                    let src_cell = self.idx(src_x, src_y);
                    if self.pmap_at(src_x, src_y) != 0
                        && pmap_id(self.pmap[src_cell]) as usize == id
                    {
                        self.pmap[src_cell] = 0;
                    }
                    self.parts[id].x = dst_x as f32;
                    self.parts[id].y = dst_y as f32;
                    let dst_cell = self.idx(dst_x, dst_y);
                    self.pmap[dst_cell] = pmap_pack(id as u32, self.parts[id].t);
                }
                return amount;
            }
            return 0;
        }
        // ref: PSTN.cpp:375-410 — extend branch: `StackData stackData =
        // CanMoveStack(...)` then the staggered compression above (the return
        // value is `possibleMovement`, the total observed free space, even when
        // nothing moved).
        let probe = self.can_move_stack(stack_x, stack_y, dir_x, dir_y, max_size, amount, false, block);
        if probe.slots.is_empty() {
            return 0;
        }
        let (possible, _) = self.staggered_stack_move(&probe, dir_x, dir_y);
        possible
    }

    /// Arm removal shared by the retract paths (ref: PSTN.cpp:333-335 and
    /// 367-369):
    /// `for (int j = 1; j <= amount; j++) sim->kill_part(ID(sim->pmap[stackY + (directionY * -j)][stackX + (directionX * -j)]));`
    ///
    /// The reference kills exactly `amount` cells with no type test — on an
    /// empty cell `ID(0)` would kill particle 0. We stop at the first cell that
    /// is not an occupied map entry; the guard is a deliberate deviation.
    fn kill_arm_behind(&mut self, stack_x: i32, stack_y: i32, dir_x: i32, dir_y: i32, amount: i32) {
        for j in 1..=amount {
            let px = stack_x - dir_x * j;
            let py = stack_y - dir_y * j;
            if !self.in_bounds(px, py) {
                break;
            }
            let r = self.pmap_at(px, py);
            if r == 0 {
                break;
            }
            let id = pmap_id(r) as usize;
            if id < self.parts.len() {
                self.kill_part(id);
            }
        }
    }

    /// Reference arm creation (PSTN.cpp:190-206):
    ///
    /// ```text
    /// int newSpace = MoveStack(sim, pistonEndX, pistonEndY, ...);
    /// if (newSpace) {
    ///     for (int j = 0; j < newSpace; j++) {
    ///         int nr = sim->create_part(-3, pistonEndX + (nxi * j), pistonEndY + (nyi * j), PT_PSTN);
    ///         if (nr > -1) { parts[nr].life = 1; ... }
    ///     }
    /// ```
    ///
    /// Current signature kept for the `updates.rs` call sites (they pass the
    /// contiguous arm end, so "ahead of the end" equals the reference
    /// `pistonEnd + j*dir` window); `extend_piston_arm_full` adds the reference
    /// dcolour darkening of new arm sections.
    pub fn extend_piston_arm(
        &mut self,
        x: i32,
        y: i32,
        dir_x: i32,
        dir_y: i32,
        count: i32,
        life: i32,
    ) -> i32 {
        self.extend_piston_arm_full(x, y, dir_x, dir_y, count, life, 0)
    }

    /// As `extend_piston_arm`, plus `dcolour` darkening copied from the piston
    /// (ref: PSTN.cpp:198-202 — `parts[nr].dcolour = (colour & 0xFF000000) | max((colour & 0xFF0000) - 0x3C0000, 0) | ...`).
    /// A failed creation (occupied cell) does not abort the walk: the
    /// reference loop simply skips that cell (PSTN.cpp:195-205).
    pub fn extend_piston_arm_full(
        &mut self,
        x: i32,
        y: i32,
        dir_x: i32,
        dir_y: i32,
        count: i32,
        life: i32,
        dcolour: u32,
    ) -> i32 {
        let mut created = 0;
        let mut tx = x;
        let mut ty = y;
        for _ in 0..count.max(0) {
            tx += dir_x;
            ty += dir_y;
            if !self.in_bounds(tx, ty) {
                break;
            }
            let cell = self.idx(tx, ty);
            if self.pmap[cell] != 0 || self.photons[cell] != 0 || self.wall_at(tx, ty) != 0 {
                // Occupied or walled: the reference `create_part(-3, ...)`
                // would overwrite the cell without checking (PSTN.cpp:193);
                // skipping instead is a deliberate safety deviation. Either way
                // the walk continues — it does not break (PSTN.cpp:194-205).
                continue;
            }
            if let Some(np) = self.create_part(tx, ty, pt::PSTN) {
                self.parts[np].life = life;
                if dcolour != 0 {
                    // ref: PSTN.cpp:198-202
                    let colour = dcolour;
                    self.parts[np].dcolour = (colour & 0xFF00_0000)
                        | (colour & 0xFF_0000).saturating_sub(0x3C_0000)
                        | (colour & 0xFF00).saturating_sub(0x3C_00)
                        | (colour & 0xFF).saturating_sub(0x3C);
                }
                created += 1;
            }
        }
        created
    }

    /// Arm removal for retraction (ref: PSTN.cpp:344-348 —
    /// `if (!callDepth) for (int j = 1; j <= amount; j++) sim->kill_part(ID(sim->pmap[stackY + (directionY * -j)][stackX + (directionX * -j)]));`
    /// — exactly `amount` cells, no type check; on an empty cell `ID(0)`
    /// would kill particle 0).
    ///
    /// Deliberate deviation: we stop at the first cell that is not a live
    /// `PSTN` arm section and return the number of sections actually removed,
    /// which is what the current `updates.rs` retract flow consumes. The
    /// reference-complete path (arm kill inside `move_stack_ref`) is available
    /// for call-site migration.
    pub fn retract_piston_arm(&mut self, x: i32, y: i32, dir_x: i32, dir_y: i32, count: i32) -> i32 {
        let mut removed = 0;
        let mut tx = x;
        let mut ty = y;
        for _ in 0..count.max(0) {
            tx -= dir_x;
            ty -= dir_y;
            if !self.in_bounds(tx, ty) {
                break;
            }
            let r = self.pmap_at(tx, ty);
            if r == 0 || pmap_type(r) != pt::PSTN {
                break;
            }
            let id = pmap_id(r) as usize;
            if id >= self.parts.len() {
                break;
            }
            self.kill_part(id);
            removed += 1;
        }
        removed
    }

    // -- SOAP links --------------------------------------------------------

    /// True when `i` currently has a "first mate" link (ref: SOAP.cpp:180+ —
    /// `//0x02 - first mate yes/no`).
    #[inline]
    pub fn soap_has_front(&self, i: usize) -> bool {
        i < self.parts.len() && (self.parts[i].ctype & 0x02) != 0
    }

    /// True when `i` currently has a "back mate" link (ref: SOAP.cpp:181 —
    /// `//0x04 - "back" mate yes/no`).
    #[inline]
    pub fn soap_has_back(&self, i: usize) -> bool {
        i < self.parts.len() && (self.parts[i].ctype & 0x04) != 0
    }

    /// Reference `Element_SOAP_detach` (SOAP.cpp:73-88): sever both links of
    /// `i` and clear the reciprocal flags so no dangling index survives.
    ///
    /// ```text
    /// if ((sim->parts[i].ctype&2) == 2 && validIndex(sim->parts[i].tmp) && sim->parts[sim->parts[i].tmp].type == PT_SOAP) {
    ///     if ((sim->parts[sim->parts[i].tmp].ctype&4) == 4) sim->parts[sim->parts[i].tmp].ctype ^= 4;
    /// }
    /// if ((sim->parts[i].ctype&4) == 4 && validIndex(sim->parts[i].tmp2) && ...) {
    ///     if ((sim->parts[sim->parts[i].tmp2].ctype&2) == 2) sim->parts[sim->parts[i].tmp2].ctype ^= 2;
    /// }
    /// sim->parts[i].ctype = 0;
    /// ```
    /// Order is front (`tmp`, xor 0x04 on the partner) then back (`tmp2`, xor
    /// 0x02); `i.tmp`/`i.tmp2` are deliberately left dangling. The SOAP-type
    /// guard on `i` is a safety addition (all reference callers are on SOAP
    /// particles; PRTI callers in updates.rs included).
    pub fn soap_detach(&mut self, i: usize) {
        if i >= self.parts.len() || self.parts[i].t != pt::SOAP {
            return;
        }
        let front = self.parts[i].tmp;
        let back = self.parts[i].tmp2;
        if self.parts[i].ctype & 0x02 != 0
            && front >= 0
            && (front as usize) < self.parts.len()
            && self.parts[front as usize].t == pt::SOAP
            && self.parts[front as usize].ctype & 0x04 != 0
        {
            self.parts[front as usize].ctype ^= 0x04;
        }
        if self.parts[i].ctype & 0x04 != 0
            && back >= 0
            && (back as usize) < self.parts.len()
            && self.parts[back as usize].t == pt::SOAP
            && self.parts[back as usize].ctype & 0x02 != 0
        {
            self.parts[back as usize].ctype ^= 0x02;
        }
        self.parts[i].ctype = 0;
    }

    /// Reference `attach` (SOAP.cpp:90-103): link `i1` to `i2` in the first
    /// free direction.
    ///
    /// ```text
    /// if (!(parts[i2].ctype&4)) {
    ///     parts[i1].ctype |= 2; parts[i1].tmp = i2;
    ///     parts[i2].ctype |= 4; parts[i2].tmp2 = i1;
    /// } else if (!(parts[i2].ctype&2)) {
    ///     parts[i1].ctype |= 4; parts[i1].tmp2 = i2;
    ///     parts[i2].ctype |= 2; parts[i2].tmp = i1;
    /// }
    /// ```
    /// Write order kept as-is; the extra bounds/self-link guards are safety
    /// additions.
    pub fn soap_attach(&mut self, i1: usize, i2: usize) {
        if i1 >= self.parts.len() || i2 >= self.parts.len() {
            return;
        }
        if i1 == i2 {
            return;
        }
        if self.parts[i2].ctype & 0x04 == 0 {
            self.parts[i1].ctype |= 0x02;
            self.parts[i1].tmp = i2 as i32;
            self.parts[i2].ctype |= 0x04;
            self.parts[i2].tmp2 = i1 as i32;
        } else if self.parts[i2].ctype & 0x02 == 0 {
            self.parts[i1].ctype |= 0x04;
            self.parts[i1].tmp2 = i2 as i32;
            self.parts[i2].ctype |= 0x02;
            self.parts[i2].tmp = i1 as i32;
        }
    }

    /// One SOAP constraint solve between two linked particles (ref:
    /// SOAP.cpp:247-256):
    ///
    /// ```text
    /// Element_SOAP_neighourLoop(sim, dx, dy);
    /// d = 9/(pow(dx, 2)+pow(dy, 2)+9)-0.5;
    /// parts[parts[i].tmp].vx -= dx*d;
    /// parts[parts[i].tmp].vy -= dy*d;
    /// parts[i].vx += dx*d;
    /// parts[i].vy += dy*d;
    /// ```
    ///
    /// `stiffness` is the coefficient scaled by the caller (`soap_constraint`
    /// for the primary solve; the secondary 81-constant solve with the `*0.5f`
    /// factors lives in `updates.rs`). The edge-loop wrap mirrors
    /// `Element_SOAP_neighourLoop` (SOAP.cpp:8-17): `dx >= (XRES/2 - CELL) →
    /// dx -= XRES - 2*CELL` and symmetrically.
    pub fn soap_link_step(&mut self, i: usize, j: usize, stiffness: f32) {
        if i >= self.parts.len() || j >= self.parts.len() {
            return;
        }
        let mut dx = self.parts[i].x - self.parts[j].x;
        let mut dy = self.parts[i].y - self.parts[j].y;
        if self.settings.edge_mode == crate::sim::EdgeMode::Loop {
            let hx = self.width as f32 / 2.0 - CELL as f32;
            let hy = self.height as f32 / 2.0 - CELL as f32;
            let span_x = self.width as f32 - 2.0 * CELL as f32;
            let span_y = self.height as f32 - 2.0 * CELL as f32;
            if dx >= hx { dx -= span_x; }
            if dy >= hy { dy -= span_y; }
            if dx < -hx { dx += span_x; }
            if dy < -hy { dy += span_y; }
        }
        let d = stiffness;
        self.parts[j].vx -= dx * d;
        self.parts[j].vy -= dy * d;
        self.parts[i].vx += dx * d;
        self.parts[i].vy += dy * d;
    }

    /// Primary SOAP tension coefficient (ref: SOAP.cpp:249 —
    /// `d = 9/(pow(dx, 2)+pow(dy, 2)+9)-0.5;`). The secondary coefficient
    /// (ref: SOAP.cpp:263 — `d = 81/(pow(dx, 2)+pow(dy, 2)+81)-0.5;`) is
    /// computed inline in `updates.rs` together with its `*0.5f` factors.
    pub fn soap_constraint(dx: f32, dy: f32) -> f32 {
        9.0 / (dx * dx + dy * dy + 9.0) - 0.5
    }

    /// Map a world pixel to its air cell index (used by transport helpers).
    #[inline]
    pub fn air_cell_of(&self, x: i32, y: i32) -> Option<usize> {
        let cx = x / CELL as i32;
        let cy = y / CELL as i32;
        if self.in_cell_bounds(cx, cy) {
            Some(self.cell_idx(cx, cy))
        } else {
            None
        }
    }
}
