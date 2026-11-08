# K3 Handoff — Independent Gameplay Verification (Agent B)

**Branch:** `agent1/kernel`
**Commit:** `952cc42bc1e5b9c698c20423b98d5b725b60c09e`
**Runtime:** experimental TPT engine, opt-in only. Production is unchanged.

> K3 is implementation-ready, **not** gameplay-accepted. Only Agent B can accept
> it after the checks below. Do not claim parity from compilation or from the
> author's own smoke test.

---

## 1. How to run

Already-built artifacts are committed (`web/pkg`, `web/pkg-tpt`). From a clean
checkout:

```powershell
cd web
python -m http.server 5173        # or: node scripts/build.mjs && serve dist/
```

- Production runtime: `http://localhost:5173/`
- **Experimental TPT runtime: `http://localhost:5173/?engine=tpt`**

Rebuild (optional):
```powershell
cargo build -p labsmatter-wasm --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/labsmatter_wasm.wasm `
  --out-dir web/pkg-tpt --target web --no-typescript
cd web; node scripts/build.mjs
```

The HUD shows `TPT` or `PROD` so the active engine is always visible.
`window.labsmatter` exposes `{ engine, sim, materials, renderer }` for probes.

## 2. Implemented systems

- Particle-list world: `pmap` + `photons` (ordinary + energy particles), walls
  (`bmap`), wall electricity (`emap`), air pressure/velocity, ambient heat,
  Newtonian gravity field.
- Tick pipeline: `BeforeSim` (sanitize → air → ambient heat → Newtonian gravity
  → EMP → tick++ → recalc/life → emap decay → block maps → GoL) →
  `UpdateParticles` (bounds/wall kill, air momentum, neighbourhood, velocity,
  transition, element update, movement) → `AfterSim`.
- Boundary modes: **Void** (particles die at the border), **Solid** (a wall ring
  is installed on the border CELLs; particles are blocked, not lost), **Loop**
  (particles wrap to the opposite side). Switch with the edge-mode key binding.
- Air/heat/gravity/phase transitions from `realmatter-tpt` (Agent 2 domain).
- Element update dispatch: 132/132 reference `Update` callbacks have handlers
  (126 dedicated + 6 partial/blocked; see `inventory/implementation-status.md`).
- Sim tools: heat/cool/pressure/vacuum/PGRV/NGRV/mix/cyclone/wind exposed as
  `apply_tool` (mirrors `realmatter-wasm`).
- Persistence: native v2/v3 (`LMT2`) plus legacy v1 (`RMT\0`) import with an
  explicit material-id mapping.
- Diagnostics: particle count, tick, per-cell material/temperature/pressure/
  velocity, world size, memory, load report.

## 3. Changed world behavior in K3

- **Solid edge mode now installs a border wall ring** (`WL_WALL` = 8) exactly like
  `Simulation::SetEdgeMode`. Previously particles reaching the outer 4-px margin
  were killed even in Solid mode.
- **`Sim::sanitize()` runs at the start of `BeforeSim`**: non-finite particle
  positions/velocities/temperatures and air fields are clamped. No-op for healthy
  state; prevents NaN propagation from element bugs.
- **IGNT** now spawns with `life = 3` (reference `DefaultProperties.life`).
- **Legacy v1 saves import** into the TPT runtime (see §6).
- **v3 save** additionally persists portal payloads, wireless channel state, and
  player/fighter entities.

## 4. New material behaviors

Handlers are dispatched per element in `crates/realmatter-tpt/src/updates.rs`.
Notable groups completed by Agent 3: explosives (BOMB, C4, TNT, DEST, FUSE),
gases (O2, CO2, H2, NBLE, WTRV, FOG), solids/powders (METL, GLAS, QRTZ, TUNG,
ROCK, BRCK, CONC), radioactive/nuclear (URAN, PLUT, DEUT, NEUT/PROT/PHOT),
electronics (SPRK, ELEC, PSCN, NSCN, SWCH, LCRY, ...), and special handlers
(portal PRTI/PRTO, clones, pistons, Soap links, etc.).

**Caveat:** portals, wireless channel state, piston stacks and soap links use
persistent `Sim` state (save format v3). Verified in the WASM bundle: portal
payload survives save/load, stickman locomotion moves without teleporting, a
sparked piston extends its arm. What remains unverified is gameplay feel and
the reference-specific element reactions (see §5).

## 5. Known unsupported / unverified

- Stickman/fighter entities and wireless channels: shared state and locomotion
  are implemented and persisted; the reference leg physics, element-power
  reactions and fighter combat still need Agent 3. `WIFI` channel latch is
  verified, but end-to-end triggering from a live spark needs gameplay testing.
- Electricity is partial; many conductors react but full circuit semantics
  (pistons, powered walls, detectors) are not proven.
- Game of Life (LIFE) rules are present (`gol.rs`, 23 rules) but unverified.
- Border behavior for energy particles in Void mode is untested.
- No headless-browser gameplay acceptance was performed by Agent 1 beyond the
  smoke checks in §8; Agent B must verify interactively.

## 6. Save compatibility

- v2/v3 native: `Save` writes `LMT2` magic; `Load` restores particles, walls,
  air fields, settings, RNG state, portal/wireless/entity state (v3).
- Legacy v1 (`RMT\0`, optionally `ZLIB`): imported by mapping each old material
  id through its canonical abbreviation to the current PT id (never numeric
  reuse). Temperature, velocity and lifetime (s→frames) are preserved; per-cell
  pressure is averaged into the air grid; decoration cells and v1 flags are not
  representable and are reported, not silently dropped.
- Unknown/corrupt blobs are rejected with the active world unchanged; the reason
  is exposed via `last_load_report()` and shown as a toast.

## 7. Expected observable physics

- Powder falls and piles; liquids spread; gases rise/disperse when air is on.
- `Solid` edge: a pile of sand stays in the world (count does not drop to 0).
- `Void` edge: falling matter leaves the world / is removed.
- `Loop` edge: matter wraps around edges.
- Air tools (wind/cyclone/pressure/vacuum) visibly push gases and light matter.
- Heat tool warms cells; lava/fire heat neighbours; water freezes/boils.
- Hover HUD shows real temperature/pressure/velocity (not invented values).

## 8. Agent 1 smoke evidence (not acceptance)

Chrome (headless, WebGL2), 1280x800, 256x256 world:

| Check | TPT | PROD |
|---|---|---|
| init / no console errors | ready, 0 errors | ready, 0 errors |
| paint SAND (render pixels) | 634 | 634 |
| Solid edge, 180 steps | 634 → 634, wall(0,0)=8 | 634 → 634 |
| apply_tool heat | 293.15 → 313.15 K | n/a |
| v2/v3 save→load | 635 → 635, report "loaded LabsMatter v2/v3 world" | 634 → 634 |
| bad blob | rejected, world unchanged, reason reported | rejected |

Node-level: v1 legacy blob (34 particles) migrated with correct PT ids
(2,6,13,14,15,44); Solid 200→200, Void 200→0, Loop 200→200; v3 round-trip.

## 9. Specific areas requiring gameplay retesting

1. Boundary: Solid/Void/Loop with powders, liquids, gases, and energy particles.
2. Air/gravity: wind, cyclone, pressure/vacuum tools; Newtonian gravity on/off.
3. Pressure propagation and heat transfer over time.
4. Electrical elements (SPRK/PSCN/NSCN/SWCH/LCRY) and powered walls.
5. Reactive pairs (fire+fuel, acid+metal, water+lava, salt+water, etc.).
6. Energy particles (PHOT/NEUT/ELEC) — creation, movement, absorption.
7. Save/load: v3 round-trip, v1 legacy import, cross-engine rejection.
8. Rendering: orientation, cursor alignment, zoom/pan, grid, heat/pressure/velocity views.
9. Reset and re-init after load (dimensions, buffer freshness).
10. Performance under a dense 50% world (see §10).

## 10. Performance reference (Node, release wasm, 128x128, edge solid)

| Scenario | Particles | Sim TPS | Sim+render TPS | ms/step |
|---|---:|---:|---:|---:|
| empty | 0 | 9,675 | 7,254 | 0.103 |
| sparse 10% | 1,256 | 3,475 | 2,833 | 0.288 |
| dense 50% | 6,276 | 749 | 682 | 1.335 |
| dense + Newtonian | 6,276 | 507 | 638 | 1.971 |
| dense + heat off | 6,276 | 701 | 666 | 1.427 |

WASM binary 548,596 bytes; runtime memory ~3.2 MB at these sizes. Browser showed
FPS 54–62 at 256x256. Dense 50% is ~700–750 TPS (still >10x the 60 TPS target);
Newtonian gravity is the heaviest add-on.
