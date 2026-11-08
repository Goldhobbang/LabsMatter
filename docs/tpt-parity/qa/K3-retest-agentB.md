# K3 Independent Gameplay Acceptance Audit — Agent B

- Tested commit: `c06a86fb3c89659b583e8b3eec5c3c038f6810aa`
- Branch: `agent1/kernel`
- Reference: `2b7221a46d378589b4de7683f16d3ebc92be6bd9`
- Runtime: `?engine=tpt` (experimental TPT WASM)
- Auditor: Agent B (independent QA)
- Date: 2026-11-08
- Environment: Windows, code-level audit (no live browser session available)

## Scope limitation

This audit was performed as a **code-level and build-level review**. No live browser
session was launched, so all gameplay-observable items are marked **NOT VERIFIED**
unless they can be confirmed from source. Build and static checks are reported as
verified.

## 1. Build verification

| Check | Result |
|---|---|
| `cargo build` (workspace) | PASS |
| `cargo build -p labsmatter-wasm --target wasm32-unknown-unknown` | PASS |
| `cargo build -p realmatter-tpt` | PASS |
| Commit `c06a86f` available | YES |
| `K3-handoff.md` present | NO (missing) |

## 2. Runtime verification

| Item | Status | Notes |
|---|---|---|
| Engine identification TPT | NOT VERIFIED | requires browser |
| WASM init | NOT VERIFIED | requires browser |
| WebGL2 rendering | NOT VERIFIED | requires browser |
| Paint/erase | NOT VERIFIED | requires browser |
| Stepping | NOT VERIFIED | requires browser |
| Material IDs | NOT VERIFIED | requires browser |
| Console errors | NOT VERIFIED | requires browser |

## 3. World boundary verification

| Mode | Status | Notes |
|---|---|---|
| Solid | NOT VERIFIED | code: `apply_edge_mode` installs wall ring; `clear_border`/`wrap_border` exist |
| Void | NOT VERIFIED | code: `clear_border` empties border cells |
| Loop | NOT VERIFIED | code: `wrap_border` swaps edges |

## 4. Advanced elements

| System | Status | Notes |
|---|---|---|
| Portals (PRTI/PRTO) | NOT VERIFIED | code present in `entity.rs`/`portal.rs` |
| Pipes (PIPE) | NOT VERIFIED | code present |
| Wireless (WIFI) | NOT VERIFIED | code present in `wireless.rs` |
| Pistons (PSTN) | NOT VERIFIED | code present |
| SOAP | NOT VERIFIED | code present |
| Stickmen (STKM/STKM2/FIGH) | NOT VERIFIED | code present |

## 5. Physics regression

| Item | Status | Notes |
|---|---|---|
| Powder movement | NOT VERIFIED | code: `movement_phase` handles powders |
| Liquid behavior | NOT VERIFIED | code: liquid spreading in movement |
| Gas movement | NOT VERIFIED | code: gas diffusion in element updates |
| Air pressure | NOT VERIFIED | code: `update_air` implemented |
| Air velocity | NOT VERIFIED | code: `vx`/`vy` fields |
| Negative pressure | PARTIAL | code uses signed `MIN_PRESSURE..MAX_PRESSURE`; physics sanitize fixed |
| Gravity modes | NOT VERIFIED | code: vertical/off/radial/custom in `gravity_field` |
| Newtonian gravity | NOT VERIFIED | Jacobi approximation (documented deviation) |
| Ambient heat | NOT VERIFIED | code: `update_airh`, `hv` field |
| Thermal transitions | NOT VERIFIED | code: `transition_phase` |
| Electrical propagation | NOT VERIFIED | code: `step_spark` in wasm |
| Energy-particle movement | NOT VERIFIED | code: `move_energy` |

## 6. Persistence

| Item | Status | Notes |
|---|---|---|
| LMT2 v2/v3 round-trip | NOT VERIFIED | requires browser |
| Legacy v1 import | NOT VERIFIED | requires browser |
| Invalid file handling | NOT VERIFIED | requires browser |

## 7. Element behavior audit

Implementation claims (195 elements, 132 handlers) are not independently verified.
Code-level: `updates.rs` contains handlers; `elements_generated.rs` defines 195 elements.

## 8. Performance

| Metric | Status |
|---|---|
| FPS | NOT VERIFIED |
| TPS | NOT VERIFIED |

## 9. Acceptance decisions

1. K3 runtime stability: NOT VERIFIED (no browser session)
2. World boundaries: NOT VERIFIED
3. Portals and wireless: NOT VERIFIED
4. Particle transport: NOT VERIFIED
5. Stickman/fighter: NOT VERIFIED
6. Physics behavior: NOT VERIFIED (code present, gameplay unverified)
7. Chemical/element behavior: NOT VERIFIED
8. Persistence: NOT VERIFIED
9. Rendering/controls: NOT VERIFIED
10. Overall K3 gameplay readiness: NOT VERIFIED

## Prioritized correction plan

- **P0**: Provide `K3-handoff.md` (missing). Enable browser-based QA by documenting serve/build steps.
- **P1**: Agent 1 — expose `apply_tool` in `labsmatter-wasm`; persist rng state.
- **P1**: Agent 2 — implement liquid equalization, shock propagation, energy refraction.
- **P2**: Agent 3 — complete 6 partial element handlers; verify portal/pipe/wireless gameplay.
- **P2**: All — run actual browser verification before claiming parity.

## Note

This report cannot grant acceptance. A live browser session is required to verify
observable gameplay behavior. All gameplay items are marked NOT VERIFIED.
