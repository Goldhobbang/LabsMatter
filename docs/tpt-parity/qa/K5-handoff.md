# K5 Handoff — Independent Gameplay Verification (Agent B)

**Branch:** `agent3/chemistry-elements` (integration via `agent1/kernel`)
**Runtime:** experimental TPT engine, `?engine=tpt`. Production unchanged.
**Prior handoff:** `docs/tpt-parity/qa/K4-handoff.md`.

> K5 is implementation-ready, not gameplay-accepted. Only Agent B can accept it.

## 1. What K5 changed

| Element | Change |
|---|---|
| WIFI | Confirmed reference behaviour; needs a spark source to transmit |
| PIPE / PPIP | Real payload storage/transport, direction, intake/output, PRTI hand-off, PPIP triggers, auto-config |
| PSTN | Temperature-driven extension/retraction, `tmp3` flags, FRME push |
| SOAP | Bubble formation, chain pop, obstacle detach, second-order constraints |
| STKM / STKM2 | Hazard damage, leg phase, element-power spawn, death cleanup |
| FIGH | Nearest-target pursuit, melee, death cleanup |

## 2. Manual checklist

### WIFI
1. Build `BTRY -> METL wire -> WIFI` (transmitter).
2. Build `WIFI -> NSCN -> METL wire` at a distant point (receiver), same
   temperature (channel) as the transmitter.
3. Step. The receiver wire should spark (`SPRK` appears) while the transmitter
   has a live spark. `wireless_active()` should be ≥ 1.
4. Break the transmitter spark: the channel should deactivate within one tick.
5. Negative control: a lone WIFI with no spark must **not** transmit.

### PIPE / PPIP
1. Paint a PIPE line. After ~60 ticks it should build a `BRCK` border and turn
   red; wait for the colour gradient.
2. Erase one `BRCK` at the desired exit.
3. Drop `SAND`/`WATR` at the other end. It should travel along the pipe and
   emerge at the exit; the stored particle must not be lost.
4. Place `PRTI` at a pipe end: the payload should be transferred to the portal
   and reappear at the matching `PRTO`.
5. Spark a `PPIP` with `PSCN`/`NSCN`/`INST`: pause/reverse must toggle; the
   `BRCK` border should glow when paused.

### PSTN
1. `PSTN` base with `PSCN` spark on one side: the arm should extend by
   temperature (≈2 cells per frame) and displace particles, not delete them.
2. `NSCN` spark: the arm retracts.
3. Set `tmp3` flags (0x01 obstacle, 0x02 arm-limit, 0x04 retract-obstacle,
   0x08 retract-too-short) and confirm the expected abort/limit behaviour.
4. Put a `FRME` row in front: it should be pushed with the arm.

### SOAP
1. Paint SOAP under a pressure change (pressure tool or a small explosion): it
   should bubble and link into chains.
2. Drive it into a solid/wall when its life is exhausted: it should pop and
   detach cleanly (no dangling links/spin).
3. Break a link mid-chain: links must be repaired or cleaned up, not corrupt.

### STKM / STKM2 / FIGH
1. Spawn a stickman, walk left/right, jump; legs should alternate when walking.
2. Walk into `ACID`/`LAVA`: the entity dies. Walk into fire: damage then death.
3. Select an element power and hold the spawn key: the element should emit in
   front of the entity.
4. Spawn a fighter near a stickman: it should pursue, melee, and kill; the
   fighter count should remain correct. Spawn two players and two fighters:
   each should act independently.

### Generic / passive
- Paint several no-Update elements (OIL, METL, GAS, WAX, etc.): they should use
  reference default life/ctype/tmp and undergo their declared phase transitions
  (see `docs/tpt-parity/inventory/generic-passive-audit.md`).

## 3. Known remaining gaps (do not report as passing)

- PIPE: HEAC flood trigger and exact PIPE graphics emulation.
- PSTN: full recursive `MoveStack`/`FRME` shared-stack propagation.
- SOAP: full chain rewiring when reconnecting mid-chain.
- STKM/STKM2: reference leg-particle physics and portal interaction.
- FIGH: reference fighter element powers and multi-fighter brawling.

## 4. Build / run

```powershell
cargo build --workspace
cargo run -p realmatter-tpt --example list_unimpl   # expect 0
cargo build -p labsmatter-wasm --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/labsmatter_wasm.wasm `
  --out-dir web/pkg-tpt --target web --no-typescript
cd web; python -m http.server 5173
```
Open `http://localhost:5173/?engine=tpt`.

## 5. Evidence already gathered by A3 (not acceptance)

- WASM: WIFI active>0 with BTRY→METL→WIFI and receiver spark.
- WASM: PIPE absorbs adjacent SAND after auto-configuration.
- WASM: entity alive, fighter spawn, v3 save/load round-trip.
- Native: PIPE emits past an open end; PSTN 1→4→3; SOAP 10 bubbles/10 links;
  STKM dies to acid; FIGH closes to dx=1 and kills.
