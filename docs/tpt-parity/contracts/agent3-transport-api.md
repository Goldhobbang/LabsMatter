# Contract: Shared Transport / Entity API for Agent 3

**Owner:** A1 (world infrastructure) · **Consumer:** A3 (PIPE, PSTN, SOAP, STKM,
STKM2, FIGH) · **Status:** stable, integrated into `realmatter-tpt::Sim`.

These helpers live in the authoritative `Sim` and are already used by the
corresponding element handlers. A3 must call them instead of adding duplicate
storage or thread-local state.

## 1. Portals — `crate::portal`

```rust
pub const PORTAL_CHANNELS: usize = 101;
pub const PORTAL_DIRECTIONS: usize = 8;
pub const PORTAL_SLOTS: usize = 80;

pub struct PortalPayload {
    pub t: u16, pub life: i32, pub ctype: i32,
    pub x: f32, pub y: f32, pub vx: f32, pub vy: f32, pub temp: f32,
    pub tmp: i32, pub tmp2: i32, pub tmp3: i32, pub tmp4: i32,
    pub flags: u32, pub dcolour: u32,
}
impl PortalPayload {
    pub fn is_empty(&self) -> bool;              // t == 0
    pub fn from_particle(p: &Particle) -> Self;
    pub fn apply_to(&self, p: &mut Particle, keep_velocity_if_zero: bool);
}
pub struct PortalStore { /* dense-lazy */ }
impl PortalStore {
    pub fn store(&mut self, channel: usize, direction: usize, payload: PortalPayload) -> bool;
    pub fn peek(&self, channel: usize, direction: usize) -> Option<PortalPayload>;
    pub fn take(&mut self, channel: usize, direction: usize) -> Option<PortalPayload>;
    pub fn channel_occupied(&self, channel: usize) -> bool;
    pub fn iter_nonempty(&self) -> impl Iterator<Item = (usize, usize, usize, PortalPayload)>;
    pub fn restore(&mut self, channel: usize, direction: usize, slot: usize, payload: PortalPayload);
    pub fn len(&self) -> usize;
    pub fn clear(&mut self);
}
```

`Sim.portal: PortalStore` is persisted in the v3 save. Channel derivation is
`portal_channel(sim, particle_id)` in `updates.rs`; keep PRTI/PRTO/PIPE using the
same helper so sender and receiver agree.

## 2. Wireless — `crate::wireless`

```rust
pub struct WirelessStore { /* [ch][0] current, [ch][1] next */ }
impl WirelessStore {
    pub fn is_active(&self, channel: usize) -> bool;
    pub fn activate_next(&mut self, channel: usize);
    pub fn note_spark(&mut self, channel: usize) -> bool; // true when a spark was seen
    pub fn advance(&mut self);                            // called from Sim::after_sim
    pub fn active_count(&self) -> usize;
    pub fn iter_active(&self) -> impl Iterator<Item = usize>;
    pub fn clear(&mut self);
}
```

`Sim.after_sim()` promotes the "next" frame to "current" once per tick. WIFI
must call `note_spark(ch)` when an adjacent conductor carries a fresh spark, and
`activate_next(ch)` when it should transmit. Do not call `advance()` from an
element handler.

## 3. Entities — `crate::entity`

```rust
pub const MAX_FIGHTERS: usize = 100;
pub mod input { pub const LEFT: u8 = 1; pub const RIGHT: u8 = 2; pub const JUMP: u8 = 4; pub const SPAWN: u8 = 8; }

pub struct PlayerState {
    pub comm: u8, pub pcomm: u8, pub elem: i32,
    pub legs: [f32; 16], pub accs: [f32; 8],
    pub spwn: u8, pub frames: u32, pub rocket_boots: bool, pub fan: bool, pub spawn_id: i32,
}
pub enum Entity { Player1, Player2, Fighter(usize) }
pub struct StepReport { pub moved: bool, pub grounded: bool, pub died: bool }

impl Sim {
    pub fn player_state(&self, which: Entity) -> Option<&PlayerState>;
    pub fn player_state_mut(&mut self, which: Entity) -> Option<&mut PlayerState>;
    pub fn entity_head(&self, which: Entity) -> Option<usize>; // re-resolved, safe
    pub fn spawn_entity(&mut self, which: Entity, x: i32, y: i32) -> Option<usize>;
    pub fn despawn_entity(&mut self, which: Entity);
    pub fn set_entity_input(&mut self, which: Entity, comm: u8);
    pub fn step_entity(&mut self, which: Entity) -> StepReport;
    pub fn step_entity_at(&mut self, which: Entity, head: usize) -> StepReport;
    pub fn spawn_fighter(&mut self, x: i32, y: i32) -> Option<(usize, usize)>; // (slot, head)
    pub fn despawn_fighter(&mut self, slot: usize);
}
```

**Index stability:** `recalc()` compacts `parts`, so a raw head index can move.
Always fetch `entity_head(which)` at the start of a tick (it re-resolves against
`player.spwn` / fighter slots) instead of caching the index across ticks.

## 4. Stack movement / pistons / SOAP — `crate::transport`

```rust
pub struct StackProbe { pub pushed: Vec<usize>, pub spaces: i32 }

impl Sim {
    pub fn can_move_stack(&self, stack_x: i32, stack_y: i32, dir_x: i32, dir_y: i32,
                          max_size: i32, amount: i32) -> StackProbe;
    pub fn move_stack(&mut self, stack_x: i32, stack_y: i32, dir_x: i32, dir_y: i32,
                      max_size: i32, amount: i32) -> i32; // shifted cells, 0 = blocked
    pub fn extend_piston_arm(&mut self, x: i32, y: i32, dir_x: i32, dir_y: i32,
                             count: i32, life: i32) -> i32; // created count
    pub fn retract_piston_arm(&mut self, x: i32, y: i32, dir_x: i32, dir_y: i32,
                              count: i32) -> i32; // removed count

    pub fn soap_has_front(&self, i: usize) -> bool; // ctype bit 0x02
    pub fn soap_has_back(&self, i: usize) -> bool;  // ctype bit 0x04
    pub fn soap_detach(&mut self, i: usize);
    pub fn soap_attach(&mut self, i1: usize, i2: usize);
    pub fn soap_link_step(&mut self, i: usize, j: usize, stiffness: f32);
    pub fn soap_constraint(dx: f32, dy: f32) -> f32;
}
```

SOAP link bits live in `Particle.ctype` (`0x02` front, `0x04` back), which is
already persisted by the v3 save. Do not add a second link table.

## 5. WASM surface (already exported)

`spawn_stickman`, `despawn_stickman`, `set_stickman_input`, `set_stickman_element`,
`stickman_alive`, `spawn_fighter`, `despawn_fighter`, `fighter_count`,
`set_fighter_input`, `portal_count`, `wireless_active`.

Browser controls call these; A3 does not need new bindings to complete the
element behaviour.

## 6. What A3 must still implement (element logic only)

| Element | Remaining work |
|---|---|
| PIPE | full payload transfer between pipe segments and PRTI hand-off, using `Sim.portal` |
| PSTN | `FRME` recursion and `tmp3` flags (`FLAG_EXT_OBSTACLE`, `FLAG_EXT_ARMLIMIT`, `FLAG_RET_OBSTACLE`, `FLAG_RET_TOOSHORT`) |
| SOAP | bubble pop/detach edge cases and multi-link chains via the `soap_*` helpers |
| STKM/STKM2 | reference leg physics, element-power reactions, portal interaction |
| FIGH | fighter combat/AI on top of `step_entity` |

A1 will not implement these behaviours; A1 maintains the storage, tick ordering,
persistence and WASM/frontend access.
