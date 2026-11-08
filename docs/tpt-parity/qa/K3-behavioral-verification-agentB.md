# K3 Behavioral Verification Addendum — Agent B

- Tested commit: `c06a86fb3c89659b583e8b3eec5c3c038f6810aa`
- Physics commit: `af69d93d786c59800803126245f1468eb178f917`
- Reference: `2b7221a46d378589b4de7683f16d3ebc92be6bd9`
- Runtime: TPT wasm (labsmatter-wasm) via direct module import
- Method: Direct WASM state inspection (get_cell_material, get_cell_temperature, get_cell_pressure, get_cell_vx/vy) + step()
- Date: 2026-11-08

## Verification method

Direct import of `/pkg-tpt/labsmatter_wasm.js` in browser context. State read via
`get_cell_material`, `get_cell_temperature`, `get_cell_pressure`, `get_cell_vx`,
`get_cell_vy`, `tick_count`. Simulation advanced via `step(1/60)`. No Canvas2D
access required. No mocks or test fixtures created.

## Physics findings

| System | Test | Result | Status |
|---|---|---|---|
| Powder falling | Sand (44) at (100,100) → after 30 steps at (100,130) | Sand fell 30 px | PASS |
| Sand accumulation | Sand on wall floor → accumulated at y=199 | Sand piled on floor | PASS |
| Water falling | Water (2) at (100,100) → fell to bottom | Water fell | PASS |
| Gas rising | Gas (10) at (100,60) → rose | Gas moved up | PASS |
| Gravity disabled | Sand with gravity mode 1 → stayed at (100,100) | No fall | PASS |
| Radial gravity | Sand near center → moved | Sand moved toward center | PASS |
| Newtonian gravity | Sand near GRVT mass → stayed | Force too weak / mass=0 (tmp=0) | PARTIAL |
| Negative pressure | Fire created pressure -0.099 | Negative pressure present | PASS |
| Air velocity | Fire created vx=-0.033, vy=-0.315 | Air coupling works | PASS |
| Heat transfer | Ice near lava: 245K → 1795K, melted | Heat conducted | PASS |
| Combustion | Fire + wood → wood consumed, temp 695K | Fire spread and burned | PASS |
| Acid dissolution | Acid + sand → sand dissolved | Acid works | PASS |
| Electricity | Spark on metal chain → traveled | Electrical propagation works | PASS |
| Void edge | Sand at (10,10) → killed | Void kills | PASS |
| Loop edge | Sand at (128,254) → killed (not wrapped) | Loop may not work near edge | PARTIAL |

## Chemistry findings

| System | Result | Status |
|---|---|---|
| Combustion | Fire ignites wood, wood burns and is consumed | PASS |
| Acid/corrosion | Acid dissolves sand | PASS |
| Electrical propagation | Spark travels along metal chain | PASS |

## Advanced elements

| System | Status | Notes |
|---|---|---|
| Portals (PRTI/PRTO) | NOT VERIFIED | requires interactive testing |
| WIFI | NOT VERIFIED | requires interactive testing |
| PIPE | NOT VERIFIED | requires interactive testing |
| PSTN | NOT VERIFIED | requires interactive testing |
| SOAP | NOT VERIFIED | requires interactive testing |
| STKM/STKM2/FIGH | NOT VERIFIED | requires interactive testing |

## UI and palette findings

| Item | Result |
|---|---|
| TPT runtime accessible via wasm | PASS |
| 195 elements registered | PASS (ptId 0-195) |
| No console errors | PASS |

## Critical finding: Frontend not wired to TPT

The frontend (`web/src/main.js`) always instantiates `WasmDirectAdapter` (legacy
realmatter-wasm) even when `?engine=tpt` is specified. The `TptAdapter` is imported
but never used. The `?engine=tpt` flag only changes the HUD label and palette,
not the actual simulation engine. The real TPT engine (labsmatter-wasm) is
accessible only via direct module import, not through the UI.

**Severity: P1 — Agent 1 (integration)**

## Remaining unverified

- Portals, WIFI, PIPE, PSTN, SOAP, STKM/STKM2/FIGH (requires UI interaction)
- Loop edge mode (particles killed before wrapping)
- Newtonian gravity observable effect (mass sources need tmp > 0)
- Water spreading (limited to 1 cell in test)
- Thermal view rendering

## P0/P1/P2 defects

- **P1**: Frontend not wired to TPT adapter (Agent 1)
- **P2**: Loop edge mode may not wrap particles near boundary (Agent 1)
- **P2**: Newtonian gravity force too weak to observe with default mass (Agent 2)
- **P2**: Water spreading limited (Agent 2)

## Final K3 gameplay acceptance

**PARTIAL** — Core physics (powder, liquid, gas, heat, combustion, acid, electricity,
gravity modes, negative pressure, air coupling) verified via direct WASM state
inspection. Advanced elements and UI integration remain unverified. Critical
frontend wiring issue prevents TPT gameplay through normal UI.
