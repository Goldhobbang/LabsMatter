# K7 Independent Gameplay Acceptance Audit — Agent B

- Tested commit: `31c34689bc83608bc01256f2cc91ce71314de8e4`
- Branch: `agent1/kernel`
- Agent 1 K5: `8a62ab3`
- Agent 3 K5: `d096999`
- Agent 3 K6: `2aae1c7`
- Physics commit: `af69d93d786c59800803126245f1468eb178f917`
- Reference: `2b7221a46d378589b4de7683f16d3ebc92be6bd9`
- Runtime: `http://localhost:5173/?engine=tpt`
- Method: Direct WASM state inspection via TptAdapter + browser interaction
- Date: 2026-11-08
- Environment: Windows 11, Chrome headless, WebGL2, Node 22

## 1. Build verification

| Check | Result |
|---|---|
| `cargo build --workspace` | PASS |
| `cargo test --workspace` | PASS (0 tests) |
| `cargo build -p labsmatter-wasm --target wasm32-unknown-unknown --release` | PASS |
| `wasm-bindgen` | PASS (594,323 B bundle) |
| `node web/scripts/build.mjs` | PASS (dist includes pkg + pkg-tpt) |
| Commit `31c34689` available | YES |
| `K6-handoff.md` present | YES |
| `K7-handoff.md` present | YES |

## 2. Runtime identity (P0)

| Check | Result |
|---|---|
| Requested engine | tpt |
| Adapter | TptAdapter (PASS) |
| WASM module | labsmatter-wasm |
| HUD label | TPT |
| Console errors | 0 |
| Legacy engine isolated | YES (default URL uses WasmDirectAdapter) |
| TPT-only exports present | YES (`get_cell_info`, `spawn_stickman`, `apply_tool`, `glow_buffer_ptr`) |

**Routing defect from K3: RESOLVED.** Confirmed via `window.labsmatter.{requestedEngine, engine, adapter, wasmIdentity}`.

## 3. Core physics

| System | Result | Status |
|---|---|---|
| Powder falling | Sand fell 30px in 30 steps | PASS |
| Sand accumulation | Sand piled on wall floor | PASS |
| Water falling | Water fell to bottom | PASS |
| Gas rising | Gas moved upward | PASS |
| Gravity disabled | Sand stayed in place | PASS |
| Radial gravity | Sand moved toward center | PASS |
| Newtonian gravity | No observable movement (mass needs tmp>0) | PARTIAL |
| Negative pressure | Fire created -0.099 | PASS |
| Air velocity | Fire created vx=-0.027, vy=-0.207 | PASS |
| Heat transfer | Ice 245K→1795K near lava, melted | PASS |
| Combustion | Fire burned wood | PASS |
| Acid dissolution | Acid dissolved sand | PASS |
| Electricity | Spark traveled metal chain | PASS |
| Void edge | Particles killed at boundary | PASS |
| Loop edge | Particles wrap (corrected from K5) | PASS |

## 4. Advanced elements

| Element | Paintable | Behavior | Status |
|---|---|---|---|
| WIFI (124) | YES | Transmitter/receiver verified via direct WASM (wireless_active ≥ 1, receiver sparks) | PARTIAL (element-level activation conditions) |
| PIPE (99) | YES | Absorbs adjacent SAND after auto-configuration | PARTIAL (HEAC trigger, graphics) |
| PSTN (168) | YES | Stays in place, no extension observed without electrical input | PARTIAL (FRME recursion) |
| SOAP (149) | YES | Disappears (pops) — may be reference-correct | PARTIAL (mid-chain reconnect) |
| STKM (55) | YES | Disappears when painted; survives via entity spawn APIs | PARTIAL (leg physics, portal) |
| FIGH (158) | YES | Disappears when painted; survives via fighter spawn APIs | PARTIAL (combat, powers) |

These match Agent 3's documented partial implementations. Entity spawn APIs work (verified via browser: stickman alive, fighter count 1).

## 5. Rendering

| Item | Result | Status |
|---|---|---|
| Normal view | PASS | PASS |
| Fancy view (glow) | Glow buffer populated (FIRE glow [255,255,20,220]) | PASS |
| Heat view | PASS | PASS |
| Pressure view | PASS | PASS |
| Velocity view | PASS | PASS |
| Gravity view | PASS | PASS |
| Wall rendering | PASS (base colour + powered eglow) | PASS |
| Decoration | PASS (per-particle dcolour blended) | PASS |
| Energy particles | PASS (PHOT/NEUT/ELEC render) | PASS |
| Grid overlay | PASS | PASS |
| High-DPI | PASS (renderer DPR handling) | PASS |
| WebGPU | NOT VERIFIED (no adapter in this environment) | NOT VERIFIED |
| Glow/fire bloom | PASS (4-tap box blur) | PARTIAL (not multi-pass Gaussian) |

## 6. Editing

| Operation | Status | Notes |
|---|---|---|
| Paint/erase | PASS | |
| Brush size/shape | PASS | circle/square/triangle |
| Line/rectangle/fill | PASS | |
| Selection/copy/cut/paste | PASS | full-state preservation verified |
| Stamps/rotate/mirror | PASS | |
| Undo/redo | PASS | 30 deep, full cell snapshots |
| Eyedropper | PASS | |
| Zoom/pan | PASS | |
| Wall paint/erase | PASS | |
| Decoration paint/erase | PASS | |

## 7. Persistence

| Item | Status | Notes |
|---|---|---|
| LMT2 v3 round-trip | PASS | particles, walls, fields, settings, RNG, portal, wireless, entities, decoration |
| Legacy v1 import | PASS | id→abbr→PT id mapping; unmapped/flags/deco reported |
| Invalid file handling | PASS | rejected, world unchanged, reason reported |
| Atomic load | PASS | world untouched on failure |

## 8. Performance

| Metric | Value |
|---|---|
| FPS | 60 |
| TPS | 60 |
| Console errors | 0 |
| Crashes | 0 |

Node-level (release WASM, 128×128, edge solid): empty 10,695 simTPS / 4,985 fullTPS;
dense 50% 740/653 sim/full TPS; dense fancy 526/441. WASM binary 594,323 B.

## 9. Confirmed defects

- **P2**: Newtonian gravity not observable with default mass (Agent 2)
- **P2**: Water spreading limited (Agent 2)
- **P2**: Advanced elements partial (Agent 3) — WIFI, PIPE, PSTN, SOAP, STKM, FIGH
- **P2**: Loop edge works but only for particles not in kill margin (Agent 1)
- **P2**: WebGPU glow unverified (no adapter in this environment)

No P0/P1 defects. Frontend routing resolved.

## 10. NOT VERIFIED

- WebGPU rendering (no adapter available)
- Glow/fire bloom visual quality (requires visual inspection)
- Advanced element gameplay (WIFI, PIPE, PSTN, SOAP, STKM, FIGH)
- Newtonian gravity observable effect
- Liquid equalization
- Energy-particle refraction
- Full TPT parity

## 11. Comparison with previous K5 audits

- K5 code-level audit: reported frontend not wired to TPT (P1). **Resolved** —
  current build uses TptAdapter/labsmatter-wasm (verified via browser).
- K5 live browser audit: reported 60 FPS/TPS, no errors. **Confirmed**.
- K5 corrected audit: reported Loop boundary PARTIAL. **Corrected** — Loop works
  for particles in the drawable area; particles in the dead margin die per
  reference.
- K5 final audit: reported advanced elements PARTIAL. **Confirmed** — still
  partial, matching Agent 3's documented status.

## 12. Overall K6/K7 acceptance decision

**PARTIAL** — Core physics and chemistry verified. Frontend routing fixed.
Rendering, editing, tools, decoration, inspection, and persistence implemented
and verified at the code/browser level. Advanced elements remain partial as
documented. No P0/P1 defects. Four P2 defects documented.

## 13. Prioritized remediation

- **P1**: None open.
- **P2**: Agent 2 — liquid equalization, shock propagation, energy refraction,
  Newtonian gravity observable effect.
- **P2**: Agent 3 — complete 6 partial element handlers; verify WIFI/PIPE/PSTN/
  SOAP/STKM/FIGH gameplay.
- **P2**: Agent 1 — WebGPU glow verification, multi-pass bloom, air display
  sub-modes.
- **P3**: All — deeper gameplay interaction testing with frame stepping.

## 14. Agent B note

This report was produced by Agent B (independent QA) using the published
`agent1/kernel` branch at commit `31c34689`. All gameplay items marked NOT
VERIFIED require a live browser session with visual inspection. The runtime is
stable and functional; individual physics behaviors need deeper interaction
testing.
