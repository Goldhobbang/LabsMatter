# K6 independent gameplay verification — Agent B

Independent QA. No implementation source was modified. No automated tests, fixtures, or mocks were added to the repository.

| | |
|---|---|
| Role | Agent B |
| Requested K6 baseline | Agent 1 final `5c334572e02c62e4d38101fff1aa1598b25885c5`; Agent 1 implementation `2c3bb906c714521a38796905a6a9042feeed6033`; Agent 3 merge `b250a6c0a1e258a24c7272ef627e00a6f3208a5b`; Agent 3 branch `2aae1c70f1c903ff1be46bf04a15c5407b5b678c`; prior K5 `a4477591108b7e73dad1ccbffdf40ff65c3c3a0a` |
| Build actually executed | See section A. Those five SHAs are not in `Goldhobbang/LabsMatter`. |
| TPT reference pin | `2b7221a46d378589b4de7683f16d3ebc92be6bd9` (commit exists; `src/simulation/elements/SPRK.cpp` returns HTTP 200). TPT was not executed. |
| Browser | Google Chrome 156.0.8078.47, Linux x86_64, headless, SwiftShader |
| When | 2026-11-09 00:49–00:52 Asia/Seoul |
| Overall K6 gameplay | **Not accepted.** The K6 engine is not in this repository, so it cannot be played. |

Status words used below mean only this:

- **PASS** — the reference behavior was observed on a build that contains the feature.
- **PARTIAL** — some of the behavior ran, and a specific reference behavior did not.
- **FAIL** — a correctly configured operation on the feature's own build missed the reference.
- **NOT VERIFIED** — the activation conditions were not established.
- **MISSING** — the feature is not in the tree that was tested.

K6 systems are **MISSING**. Observations of the older powder shell are recorded separately and are not K6 scores.

---

## A. Repository and environment

### Requested K6 commits are absent

`git cat-file -t` and `GET /repos/Goldhobbang/LabsMatter/commits/<sha>` both fail for every SHA in the assignment. GitHub returns HTTP 422, `No commit found for SHA`. After `git fetch --all --prune` the remote has only:

| Branch | SHA | Commit time (Asia/Seoul) | Subject |
|---|---|---|---|
| `main` | `8736292254d81b701de679849f699b8c0b3b1a77` | 2026-11-05 15:42 +0900 | Merge pull request #1 |
| `cursor/fix-play-bugs-9315` | same history as main | | |
| `cursor/tpt-parity-9315` | `76480ab093bdbfb7d2688b48b19e526fc60133cf` | 2026-11-08 20:45 | Keep powders falling through empty air |

No newer integration exists on this remote. `76480ab` is not a newer K6. It is an older single-engine powder shell. It does not contain Agent 3's K6 work, and it was not used as a silent substitute for K6 scoring.

Searched and not found anywhere on either tree:

- crate `realmatter-tpt`
- crate `labsmatter-wasm`
- `TptAdapter`
- `window.labsmatter`
- `set_cell_ex`
- `LMT2`
- `fighter_head`
- `get_cell_life`, `get_cell_ctype`, `get_cell_tmp`, glow-buffer export

Workspace at start: `/workspace`, branch `main`, clean, one worktree. The Windows checkouts named `AICrew`, `AICrew-agent2`, and `AICrew-agent3` are not on this machine. No other agent worktree was switched or reset.

### Documentation that was requested and is not in the repo

Absent on `main` and on `cursor/tpt-parity-9315`:

- `docs/tpt-parity/qa/K6-handoff.md`
- `docs/tpt-parity/qa/K5-handoff.md`
- `docs/tpt-parity/qa/runtime-routing-verification.md`
- `docs/tpt-parity/inventory/application-features.md`
- `docs/tpt-parity/inventory/elements.md`
- `docs/tpt-parity/inventory/implementation-status.md`
- `docs/tpt-parity/inventory/generic-passive-audit.md`
- `docs/tpt-parity/contracts/engine-v2.md`
- `docs/tpt-parity/contracts/agent3-transport-api.md`

`communication/` has A0–A8 protocol files only. There is no Agent B report path there. Historical gameplay notes live only on `cursor/tpt-parity-9315`:

- `docs/tpt-parity/qa/README.md` (session at `b660f4a`)
- `docs/tpt-parity/qa/TPT-QA-0001.md` … `TPT-QA-0004.md`
- `docs/tpt-parity/qa/RETEST-d82210d.md`
- `docs/tpt-parity/elements.md`

Those notes describe the powder shell. They are not evidence about K6. This session did not rewrite them.

### What was actually booted

Both trees were served with `python3 -m http.server` and opened in Chrome.

| URL | Tree | Result |
|---|---|---|
| `http://127.0.0.1:5173/?engine=tpt` | `76480ab` | App reaches `data-state=ready`. |
| `http://127.0.0.1:5174/?engine=tpt` | `8736292` (`main`) | App reaches `data-state=ready`. |

Runtime identity, read from the page after ready:

| Check | `76480ab` `/?engine=tpt` | `8736292` `/?engine=tpt` |
|---|---|---|
| `window.labsmatter` | `null` | `null` |
| Adapter constructed by `web/src/main.js` | `WasmDirectAdapter` only | `WasmDirectAdapter` only |
| WASM module URL | `/pkg/realmatter_wasm.js?sim=9` | `/pkg/realmatter_wasm.js` |
| WASM file | `realmatter_wasm_bg.wasm` (231103 bytes) | `realmatter-wasm` glue, no `sim_status` |
| `sim_status()` | `grav down air on edge solid n- u-` | `mod.sim_status is not a function` |
| World | 256×256 | 256×256 |
| HUD | `FPS 56  TPS 0  webgl2` then later `FPS 60` | `FPS 60 [webgl2] \| TPS 0` |
| Renderer | WebGL2. Console: `WebGPU init failed, falling back: No WebGPU adapter` | same warning |
| `navigator.gpu` | present | present |
| WebGPU session | **not established** | **not established** |

`?engine=tpt` changes the query string and nothing else. There is no second engine to fall back from, and there is no silent swap into a TPT adapter. The production page and the powder-shell page are different builds of the same adapter name.

K6 diagnostic fields (`requestedEngine`, `adapter`, `wasmIdentity`, `getCellInfo`, entity head, decoration, glow buffer) are not on the page.

The powder shell's live module is the same module the page painted into. A UI click and `get_cell_material` agree (next section). That check is about `76480ab`, not about a TPT world.

---

## How to read sections B–G

For every K6 item the status is **MISSING**: the source, the palette, and the WASM exports required to activate it are not in the tested commits. That is not a FAIL of a configured circuit, and it is not PARTIAL.

Under each heading, a "Nearest shell" note says what `76480ab` actually did when a related, smaller behavior exists. Those notes are not acceptance of K6.

Reference pin `2b7221a…` contains these element sources (GitHub contents listing, 214 directory entries): `SPRK.cpp`, `BTRY.cpp`, `METL.cpp`, `PSCN.cpp`, `NSCN.cpp`, `INST.cpp`, `INWR.cpp`, `WIFI.cpp`, `PIPE.cpp`, `PPIP.cpp`, `PUMP.cpp`, `GPMP.cpp`, `HSWC.cpp`, `PBCN.cpp`, `LCRY.cpp`, `EMP.cpp`, `WATR.cpp`, `SLTW.cpp`, `NTCT.cpp`, `PTCT.cpp`, `PSTN.cpp`, `FRME.cpp`, `SOAP.cpp`, `STKM.cpp`, `STKM2.cpp`, `FIGH.cpp`, `PHOT.cpp`, `NEUT.cpp`, `ELEC.cpp`, `HEAC.cpp`, `PRTI.cpp`, `STOR.cpp`, `GRVT.cpp`. `PGRV.cpp` and `NGRV.cpp` are not in that directory. TPT's binary was not run.

---

## B. Electrical system

### K6

| Behavior | Status | Why |
|---|---|---|
| BTRY → METL propagation | **MISSING** | No `BTRY` or `METL` in either palette or crate. |
| Spark lifetime and restoration to the original conductor | **MISSING** | K6 `SPRK` rewrite is not in the tree. |
| PSCN, NSCN, SWCH, INST, INWR | **MISSING** | No symbols, no palette entries. |
| WIFI channel transmission | **MISSING** | Cannot build `BTRY → METL → WIFI` / `WIFI → NSCN → METL`. Placing a raw unused id is not a WIFI test. |
| PPIP electrical pause, PUMP, GPMP, HSWC, PBCN, LCRY, EMP | **MISSING** | |
| WATR / SLTW / RSST durations | **MISSING** | `SLTW` and `RSST` are not registered. Water has no electrical update. |
| NTCT / PTCT | **MISSING** | |

No duplicate-spark, insulation-crossing, or tick-order defect is claimed for K6. The circuit was never loaded.

### Nearest shell (`76480ab` only)

`crates/realmatter-wasm/src/lib.rs` `step_spark` conducts material 16 through iron (8) and copper (18), one orthogonal step per frame. The painted cell's `vx` selects the restored id (`vx > 17` restores copper, otherwise iron). Conductors that the spark itself enters store their previous id in `vx`.

Setup: gravity off, a horizontal wire, one `set_cell(..., 16)`, then `step`. World 256×256. Same WASM instance the page had loaded.

Iron wire x=10..20 at y=40. Material ids before each of 14 steps:

```
16,8,8,8,8,8,8,8,8,8,8
8,16,8,8,8,8,8,8,8,8,8
8,8,16,8,8,8,8,8,8,8,8
… one cell per frame …
8,8,8,8,8,8,8,8,8,8,16
8,8,8,8,8,8,8,8,8,8,8
```

The pulse does not duplicate, does not bounce, and the wire is iron again after the pulse leaves. That matches this shell's own rule. It does not match TPT `SPRK.cpp` (per-element life, PSCN/NSCN, powered materials).

Copper wire x=10..16 at y=50, spark painted with `set_cell` (so `vx` stays 0):

```
16,18,18,18,18,18,18
8,16,18,18,18,18,18
8,18,16,18,18,18,18
8,18,18,16,18,18,18
8,18,18,18,16,18,18
8,18,18,18,18,16,18
8,18,18,18,18,18,16
8,18,18,18,18,18,18
```

Cells the pulse enters are restored to copper. The cell that was painted as spark becomes iron and stays iron. See NB-001 in the appendix. This is not scored as a K6 regression.

`set_cell` of ids 26, 29, 55, 60, 65, 149, and 158 stores that integer and `get_cell_material` reads it back. Id 65 is still there after 5 gravity-down steps. Those ids are not WIFI, PIPE, PSTN, SOAP, STKM, or FIGH. No update function references them.

---

## C. Specialized elements

All K6 items **MISSING**. Palette and source on `76480ab` stop at 26 materials (`VOID` through `BOMB`). `docs/tpt-parity/elements.md` on that branch says the same. Production `main` has a smaller inspector list (void, stone, sand, water, oil, steam, fire, wood, iron, lava, acid, salt, smoke, ice, glass, gunpowder) and no spark.

| Feature | K6 status | Activation attempted | Result |
|---|---|---|---|
| PIPE / PPIP init, BRCK, intake, payload, direction, temperature, life | **MISSING** | No element to place | |
| HEAC flood, powered pause/reverse, PRTI, STOR | **MISSING** | | |
| PSTN extend/retract, FRME push, `tmp3` | **MISSING** | No PSCN/NSCN source either | |
| SOAP links, reciprocal links, pop, cleanup | **MISSING** | | |
| STKM / STKM2 spawn, walk, jump, portal | **MISSING** | No stickman tool or keyboard control | |
| FIGH slot, pursuit, melee, cleanup | **MISSING** | | |

Nothing here is marked PARTIAL. The operating conditions (electricity, pipe geometry, fighter slots) cannot be built.

---

## D. Physics

### K6

| Topic | Status |
|---|---|
| K6 / Agent 2 solver, energy particles (PHOT, NEUT, ELEC), gravity wells (GRVT / the assignment's PGRV and NGRV) | **MISSING** from this remote. Agent 2's advanced physics was not in the reported K6 baseline, and it is not in `76480ab` either. |
| Loop-mode survival of energy particles (the K5 fix) | **NOT VERIFIED**. No energy particle exists to send across the border. |

### Nearest shell, observed this session

These are facts about `76480ab`. They are not K6 acceptance.

Reference for the expectations is this shell's `move_powder_step` / `move_liquid_step` (down, then diagonals, then sideways for liquids) plus the pinned TPT element files, which were not executed.

| Check | Setup | Result | Status on this shell |
|---|---|---|---|
| UI sand paint | Default brush `4×4 c`, click aimed at cell (40, 30). Cell size 4 px, camera at origin, zoom 1. | 49 sand cells, x 36–44, y 26–34. Center (40, 30) is material 2 at 293.15 K. | Paint writes the live world. |
| Sand fall | Eight `#btn-step` clicks. | Count stays 49. y moves from 26–34 to 34–42. `tick_count` is 8. | One cell per frame down. |
| Single sand, gravity down | `set_cell(40,10,2)`, 8 steps. | Sand at y=18. | PASS for this rule. |
| Gravity off | Mode 1, sand (40, 10), 12 steps. | Still material 2 at (40, 10). HUD `grav off`. | Stays. |
| Radial | Mode 2, sand (20, 128), edge solid. | x goes 20,21,22,23,24,25,26,27 over 7 steps. y stays 128. | Steps toward center on x. |
| Newtonian | Gravity off, newton on, gold 3×3 at (39–41, 39–41), sand (40, 33). Kernel in source is ±8 cells, threshold `abs > 1`. | Path (40,33)→(41,34)→(40,35)→(41,36)→(40,37)→(41,38), then stops beside the gold. Newton off: sand stays (40,33) for 6 steps. | Local pull works for this mass. It is the shell's sign sum, not a TPT 1/r² field. **PARTIAL** against the pin. |
| Open water column | Gravity down, 8 water cells at x=80, y=20–27, 30 steps. | Count 8, x stays 80, y=50–57. No widening while falling. | Down wins while the cell below is empty. |
| Water in a stone basin | Walls x=30 and x=50, floors y=20 and y=40, 5×5 water at (38–42, 30–34), gravity down, 40 steps. | Count stays 25. Bounding box goes from span x=4, y=4 to x=31–49 (span 18) and y=38–39 (span 1). | Spreads when downward motion is blocked. Not scored as TPT liquid parity. |
| Smoke, air on | (80, 40), 8 steps. | y=32. | Rises one cell per frame. |
| Smoke, air off | Same cell, 8 steps. | Stays at (80, 40). | |
| Wrap | Edge wrap, sand (40, 255), one step. Air happened to still be off from the previous call. | Sand at y=0, y=255 empty. | Powder crosses that border. |
| Void edge | Edge void, gravity off, sand at x=0 and x=5. | Border cell cleared, inner sand remains. | Border delete only. |
| Heat contact | Gravity off, air off, lava (100,100) at 1500 K beside iron (101,100) at 293.15 K. | After 1 step iron is 296.13 K and lava has moved to (99,100) at 1496.59 K. Iron does not rise further. By step 5 lava has left the searched window. | One conduction tick is real. Sustained contact was not held, because flat-liquid movement walks lava away. **NOT VERIFIED** as ongoing heat flow. |
| Pressure numbers | Empty, sand, and water. | 0 immediately after `set_cell`. After 20 steps all three read 101325, including an empty cell that was never painted. | No gradient in this setup. Pressure view therefore paints one color (section E). |
| Fire on wood | Gravity off, fire beside wood. | Wood is fire at 1100.1 K on the next step. Temperature then climbs 1102.7, 1105.0, 1107.0, 1108.8, 1110.4, 1111.8. | Adjacent fuel ignites. |
| Acid, sand, stone | Gravity off, acid (70,70), sand (71,70), stone (72,70), 10 steps. Search x=60–80, y=60–80. | Sand is gone. Acid is at (60,70) (moved). Stone remains at (72,70). | Sand does not survive contact. Stone does. |

Production `8736292`, direct `set_cell(40,10,2)` and 8 steps: sand is at y=18. That only shows the production engine still falls. It is not the TPT engine.

---

## E. Rendering

### K6 graphics stack

`GfxPixel`, `graphics.rs`, a separate glow buffer, fire/glow composition, four-neighbor bloom, and a gravity display mode are **MISSING**. WebGPU was **not** run. `navigator.gpu` exists in this Chrome, and `requestAdapter` fails with `No WebGPU adapter`. Claiming WebGPU PASS would be false.

### Nearest shell, CPU buffer and screenshot

`set_view_mode` clamps with `mode.min(4)`. Buffer is RGBA8, row-major. Samples after placing lava (10,10), sand (20,10), and water (30,10):

| Mode | Lava | Sand | Water | Empty |
|---|---|---|---|---|
| 0 Normal | 255,61,0,255 | 219,196,124,255 | 83,199,251,220 | 0,0,0,0 |
| 1 Fancy | 219,206,27,255 | 219,196,124,255 | 83,199,251,220 | 0,0,0,0 |
| 2 Heat | 207,255,37,255 | 14,0,188,255 | 14,0,188,255 | 0,0,0,0 |
| 3 Pressure | 40,80,200,255 | 40,80,200,255 | 40,80,200,255 | 0,0,0,0 |
| 4 Velocity | 40,180,80,255 | 40,180,80,255 | 40,180,80,255 | 0,0,0,0 |
| 5 and 9 | same as mode 4 | | | |

Normal, Fancy, and Heat change the sampled pixels in the way this shell's `rgba_for` describes. Fancy tints only the hot lava. Heat paints lava hot (207,255,37) and the cold cells cold (14,0,188). Pressure and velocity are constant because the underlying fields do not differ (pressure 101325, velocity ~0). Gravity view does not exist; modes above 4 alias to velocity.

Screenshots (WebGL2 compositor, not a second canvas context):

- `docs/tpt-parity/qa/evidence/nearest-76480ab-sand-normal.png` — sand pile, HUD `49 pts`, `Normal`, `webgl2`, rail `NRM`.
- `docs/tpt-parity/qa/evidence/nearest-76480ab-heat.png` — HUD `Heat`, three cells, hot vs cold colors. The cells are small; the CPU samples above are the measurement.
- `docs/tpt-parity/qa/evidence/main-8736292-engine-tpt.png` — production shell. The query was `?engine=tpt`. The page is still the production inspector. Log text in the panel says the real WASM module is in use. Backend line is webgl2.

High DPI: viewport 390×844, devicePixelRatio 2. Canvas CSS 390×661, backing store 780×1322. A click aimed at cell (20, 10) paints sand there (`get_cell_material(20,10) == 2`, 49 cells in the brush disk). Alignment for that click holds.

Not present, so not passed: particle glow, fire glow as a separate buffer, bloom, powered-wall glow, decoration blend, per-element TPT graphics.

---

## F. Editing and persistence

### K6

| Item | Status |
|---|---|
| `set_cell_ex` preservation of life, ctype, tmp, tmp2–4, flags, decoration | **MISSING**. The export does not exist. |
| SOAP links, PIPE payloads, portals, WIFI, STKM, FIGH across copy/paste | **MISSING**. The entities do not exist, so reference integrity cannot be tested. |
| LMT2 v3, LMT2 v2, legacy RMT v1 as specified for the experimental engine | **MISSING**. |
| RNG state, portal state, wireless state in the save | **NOT VERIFIED**. No fields to read. |

### Nearest shell

| Operation | Evidence | Status on this shell |
|---|---|---|
| Paint | Section D, 49 sand cells from one click. | Writes the live world. |
| Undo | After a fresh reset, one click paints 49 sand cells. `#btn-undo` leaves 0. `#btn-redo` restores 49. | Brush undo/redo works for this sand dab. |
| Line, rectangle, fill, stamp, rotate, mirror, eyedropper, copy/cut | Not re-run this session. | **NOT VERIFIED** here. Historical notes on `d82210d` are not reused as proof. |
| Save / load | `set_cell(3,4,8)` and `set_cell(4,4,3)`. `save_blob` length 7232. After `reset`, both cells are 0. `load_blob` returns true. Cells are 8 and 3. Iron temperature 293.15 K. | Material and that temperature survive this RMT blob. Velocity, life, ctype, and tmp are not in the API. |
| Corrupt load | Flip byte 4 of that blob. Place sand at (9,9). `load_blob` returns false. Sand at (9,9) remains. Iron at (3,4) from the previous successful load remains. | Rejected load does not apply. |

---

## G. Performance

K6's reported Node/WASM table (128×128, empty Normal 10695 sim TPS, dense 50% Normal 740) was **not** reproduced. Those numbers are for an engine that is not here. Different size and different code, so this session does not call a regression.

Measurements on `76480ab` in this Chrome, world 256×256, gravity off, view mode 0, `step` called from the page (simulation only):

| Workload | Steps | Wall time | Implied TPS |
|---|---:|---:|---:|
| Empty | 100 | 211.4 ms | 473 |
| About 10% cells set to sand, strided | 30 | 75.5 ms | 397 |
| About 50% cells set to sand, strided | 30 | 133.9 ms | 224 |

Browser loop, after placing a sparse sand field and pressing Play, Heat view still selected, about 450 particles: HUD read `FPS 60  TPS 60  webgl2` after 2.2 s. That TPS includes the page's 60 Hz cap (`fixed = 1/60`, at most 8 steps per frame), so it is not a saturated sim benchmark.

`performance.memory.usedJSHeapSize` after the sim-only bench was 11242817 bytes. One sample, not a leak test.

Console during all three pages: 9 messages, the only failure-shaped one is the WebGPU fallback. No shader compile error was logged.

Production idle screenshot shows a ready WASM panel. Its FPS counter read 60 with TPS 0 while paused.

---

## H. Confirmed defects

### K6-B-001 — P0 — K6 integration is not in the repository

| | |
|---|---|
| Owner | Agent 1, because only Agent 1 integrates. Agent 3's K6 commit is not reachable either. |
| Reference | The assignment's K6 SHAs and the experimental runtime: `TptAdapter`, `labsmatter-wasm`, `/?engine=tpt`. |
| Observed | None of the SHAs exist. Both running apps construct `WasmDirectAdapter` and `realmatter-wasm`. `window.labsmatter` is null. `?engine=tpt` does not select another engine. |
| Status | **MISSING**, and it blocks every K6 acceptance item. |

Reproduce:

1. `git cat-file -t 5c334572e02c62e4d38101fff1aa1598b25885c5` in `Goldhobbang/LabsMatter`, or `gh api repos/Goldhobbang/LabsMatter/commits/5c334572e02c62e4d38101fff1aa1598b25885c5`.
2. Expected: a commit that contains `TptAdapter` and Agent 3's K6 elements.
3. Observed: `fatal: git cat-file: could not get object info`, and GitHub `No commit found for SHA` (HTTP 422). The same for `2c3bb906…`, `b250a6c0…`, `2aae1c70…`, and `a4477591…`.
4. Serve `cursor/tpt-parity-9315` at `76480ab` and open `/?engine=tpt`. In the page, `window.labsmatter` is `null`. The HUD suffix is `webgl2`. Importing `/pkg/realmatter_wasm.js?sim=9` yields `sim_status` and no life/ctype/tmp/glow exports.

Do not treat the powder shell's iron spark as a failed K6 electrical rewrite. The rewrite is not loaded.

### Not filed as K6 defects

The historical routing P1 is not reopened against a K6 fix, because that fix is not in this remote. What this session saw is the absence of the route, which is K6-B-001, not a regression of a present router.

Nearest-shell notes in the appendix are outside K6 scoring.

---

## I. Unverified features

Unverified because the K6 build is absent, not because a setup was skipped:

- Every Priority 1 electrical behavior, including a valid WIFI channel.
- PIPE, PPIP, PSTN, FRME, SOAP, STKM, STKM2, FIGH, including link and slot integrity after delete.
- PHOT, NEUT, ELEC, refraction, and Loop-border survival of those particles.
- GRVT / gravity wells. `PGRV.cpp` and `NGRV.cpp` are not on the pin; `GRVT.cpp` is. Neither is in LabsMatter.
- K6 display modes beyond the five names in the powder shell, glow, bloom, decoration, powered walls.
- WebGPU on a machine that has an adapter.
- `set_cell_ex`, clipboard of special state, LMT2 v2/v3, atomic failure of an LMT2 file.
- K6 performance table.
- Line, rectangle, fill, stamp, rotate, mirror, eyedropper, and copy/cut on this session's build (not re-run).
- Sustained heat conduction while the hot cell is held in place.
- Full TPT liquid motion. The basin test only shows this shell's cellular spread.
- Any comparison that requires executing the pinned TPT binary. The pin was used as a file listing and as the named specification. The binary was not built or run.

Acid-versus-sand and the single conduction tick are observations of `76480ab` only.

---

## J. Acceptance decision

| Area | Decision |
|---|---|
| 1. Runtime stability of K6 | **Not acceptable.** The K6 runtime cannot be started. |
| 2. Engine routing | **Not acceptable.** `?engine=tpt` does not load `TptAdapter` or `labsmatter-wasm`. `window.labsmatter` is absent. Production `main` stays on `WasmDirectAdapter`, which is correct for production and is also what the experimental URL loads. |
| 3. Core physics | **No K6 decision.** The powder shell's sand, basin water, smoke, radial mode, local Newtonian pull, wrap, and void border ran. That is not the K6 solver. |
| 4. Electrical systems | **Not acceptable.** K6 SPRK and WIFI are missing. |
| 5. Special elements | **Not acceptable.** PIPE, PSTN, SOAP are missing. |
| 6. Entities | **Not acceptable.** STKM, STKM2, and FIGH are missing. |
| 7. Rendering | **Not acceptable** as K6 graphics. WebGL2 displayed the powder shell. WebGPU was not verified. |
| 8. Editing | **No K6 decision.** Sand undo/redo on the powder shell worked. `set_cell_ex` is missing. |
| 9. Persistence | **No K6 decision.** One RMT blob round-trip and one corrupt rejection worked on the powder shell. LMT2 is missing. |
| 10. Overall K6 gameplay readiness | **Not accepted.** |

### Highest-priority work

**Agent 1.** Put a commit that actually contains the K6 integration on `Goldhobbang/LabsMatter`. The tested commit has to include Agent 3's K6 implementation (`b250a6c0…` / `2aae1c70…` or a later merge that still contains it). Until `/?engine=tpt` constructs `TptAdapter` and `labsmatter-wasm`, and `window.labsmatter` identifies that pair, there is nothing for gameplay acceptance to pass.

**Agent 3.** Do not expect electrical, PIPE, PSTN, SOAP, STKM, or FIGH acceptance from this tree. The iron/copper spark in `76480ab` is a different program. Re-test only after Agent 1's integration is fetchable. Build WIFI with a real BTRY–METL feed and a temperature-matched receiver before calling it verified.

**Agent 2.** The reported K6 baseline did not include the remaining advanced physics, and this remote still does not. Newtonian gravity here is an 8-cell sign sum. Pressure in the probed setup sits at 101325 for empty and occupied cells alike. Neither result is a K6 pass or a K6 regression.

---

## Appendix — nearest shell only (`76480ab`)

Not part of the K6 score.

### NB-001 — Painted spark on copper becomes iron

Severity for this shell: P2. Owner: the powder-shell spark, not the missing K6 electrical rewrite.

`step_spark` restores a spark with `vx > 17` to copper and everything else to iron. `set_cell(16)` does not set `vx` to the conductor it replaced. A spark painted on the first copper cell therefore becomes iron. Sparks that propagate into copper store the old id in `vx` and return to copper. Trace is in section B.

### What this session does not reopen

`TPT-QA-0002` (frame step while running) was previously retested PASS at `d82210d`. This session used `#btn-step` while paused and did not repeat that shortcut. It is not re-failed.

`TPT-QA-0004` stays a powder-shell gap (air field, real Newtonian gravity, electronics beyond this spark). This session's basin, local gold pull, and iron wire agree with that earlier limit. They do not close it, and they do not move it onto the K6 list.

Sand still falls through empty air after `76480ab`: the UI pile moved down 8 cells in 8 frames, and a single cell moved from y=10 to y=18. That matches the subject of that commit.
