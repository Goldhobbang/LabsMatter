# K6 Handoff — Agent B gameplay verification (element fidelity)

**Branch:** `agent3/chemistry-elements` → integrate into `agent1/kernel`
**Commit:** `4285dea2456a8927834b4047bc649f5b42e5cdea`
**Runtime:** experimental TPT engine `?engine=tpt`. Production unchanged.
**Prior:** `K5-handoff.md` (core/editing/rendering/tools).

> K6 is implementation-ready for the six element families and the SPRK rewrite.
> It is **not** gameplay-accepted. Only Agent B can accept it.

## 1. What changed in K6

| Area | Change |
|---|---|
| SPRK | Faithful `SPRK.cpp` port: fire update, restore durations, SWCH/pump/latch/PPIP/EMP activation, sender/receiver conduction rules |
| PIPE/PPIP | HEAC flood to heat-conductive pipes; single-pixel init; PPIP pause/reverse from sparks |
| PSTN | Reference direction (adjacent arm / away from spark), FRME row discovery + push, extension into empty space |
| SOAP | Mid-chain rewiring (`ctype==7`) and lone-head splice |
| STKM/STKM2 | Feet in `legs[4/5]`,`legs[12/13]`, torso anchor, 2-cell jump, PRTI→PRTO teleport, element power |
| FIGH | Slot-aware head, player torso targeting, DUST default power + SPAWN rules, multi-fighter separation |

## 2. Manual checks

### SPRK / electrical
1. `BTRY → METL wire`: the spark should run to the far end without random gaps.
2. `PSCN spark → SWCH`: switch turns on (`life = 10`); `NSCN spark → SWCH` turns it off.
3. `PSCN/NSCN/INST spark → PPIP`: pipe unpauses/pauses/reverses; border glow follows pause.
4. Spark on `METL` next to `WATR/SLTW`: water briefly sparks then restores with the longer duration.
5. Spark next to flammable (GAS/OIL): should ignite (reference `Element_FIRE_update`).

### PIPE / PPIP
1. Paint a PIPE line; wait for the BRCK border + red colour gradient.
2. Put `HEAC` beside a configured pipe run: the whole run should become heat-conductive.
3. Erase one BRCK exit; feed SAND/WATR; it should travel and exit.
4. Spark a PPIP with `PSCN` then `NSCN`: pause state toggles; payload stops/resumes.
5. Single isolated PIPE should become a single-pixel pipe (`0x100`).

### PSTN
1. Place `PSTN`, a `FRME` row in front, and spark with `PSCN`: the frame row should
   move one cell and the piston should extend behind it.
2. Lone `PSTN` with no frame: spark extends into empty space (~2 cells/frame at
   300 K).
3. `NSCN` spark retracts the fresh arm.
4. `tmp3` flags: 0x01 obstacle aborts, 0x02 arm-limit clamps, 0x04/0x08 retract
   variants.

### SOAP
1. Pressure a SOAP line: bubbles form and link into chains.
2. Break a chain link: the reference rewiring should reconnect a lone head or
   join two chain ends without corrupt indices.
3. Drive SOAP into a wall/solid with life exhausted: pop and detach cleanly.

### STKM / STKM2
1. Walk left/right on BRCK: feet alternate (`legs[4/5]` vs `legs[12/13]`).
2. Jump: leaves the ground by ~2 cells.
3. Stand next to `PRTI` with a matching `PRTO`: the stickman teleports to the
   PRTO side.
4. Acid/lava kills; fire damages; element power spawns in the facing direction.

### FIGH
1. Spawn a fighter near a stickman: it should pursue and kill.
2. Spawn two fighters: they should separate when overlapping; both act.
3. Give the fighter a deadly element (when the UI exposes it) and confirm SPAWN
   emissions at close range; default DUST should fall back to melee.

## 3. Known remaining gaps (do not report as passing)
- PIPE graphics emulation of the stored particle.
- PSTN full recursive attached-structure stacks.
- SOAP boundary edge cases.
- STKM reference leg-particle solver (planned feet gravity/rotation).
- FIGH multi-fighter brawling and non-default powers unverified.

## 4. Build
```powershell
cargo build --workspace
cargo run -p realmatter-tpt --example list_unimpl   # 0
cargo build -p labsmatter-wasm --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/labsmatter_wasm.wasm `
  --out-dir web/pkg-tpt --target web --no-typescript
cd web; python -m http.server 5173
```

## 5. A3 evidence (not acceptance)
Native: SPRK far-end propagation; SWCH on; PPIP paused; PIPE+HEAC 6 conductive;
PSTN 1→4 + FRME 21→27; STKM jump 49→48; portal dx1/dy2; FIGH kill.
WASM: SPRK max 16 reached far end; WIFI active ≥1 + receiver spark; PIPE intake;
entity+fighter save/load.
