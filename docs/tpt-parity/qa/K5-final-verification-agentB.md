# K5 Final Independent Gameplay Acceptance — Agent B

- Tested commit: `a4477591108b7e73dad1ccbffdf40ff65c3c3a0a`
- Branch: `agent1/kernel`
- Agent 1 K5: `8a62ab3`
- Agent 3 K5: `d096999`
- Physics commit: `af69d93d786c59800803126245f1468eb178f917`
- Reference: `2b7221a46d378589b4de7683f16d3ebc92be6bd9`
- Runtime: `http://localhost:5173/?engine=tpt`
- Method: Direct WASM state inspection via TptAdapter
- Date: 2026-11-08

## 1. Engine routing (P0)

| Check | Result |
|---|---|
| Requested engine | tpt |
| Adapter | TptAdapter (PASS) |
| WASM module | labsmatter-wasm |
| HUD label | TPT |
| Console errors | 0 |
| Legacy engine isolated | YES (default URL uses WasmDirectAdapter) |

**Routing defect from K3: RESOLVED.**

## 2. Core physics

| System | Result | Status |
|---|---|---|
| Powder falling | Sand fell 30px in 30 steps | PASS |
| Negative pressure | Fire created -0.099 | PASS |
| Air velocity | Fire created vx=-0.027, vy=-0.207 | PASS |
| Heat transfer | Ice heated by lava, melted | PASS |
| Combustion | Fire burned wood | PASS |
| Acid dissolution | Acid dissolved sand | PASS |
| Electricity | Spark traveled metal chain | PASS |
| Gravity disabled | Sand stayed in place | PASS |
| Void edge | Particles killed at boundary | PASS |
| Loop edge | Particles not wrapped | PARTIAL |

## 3. Advanced elements

| Element | Paintable | Behavior | Status |
|---|---|---|---|
| WIFI (124) | YES | Stays, no observable signal | PARTIAL |
| PIPE (99) | YES | Stays, no transport observed | PARTIAL |
| PSTN (168) | YES | Stays, no extension observed | PARTIAL |
| SOAP (149) | YES | Disappears (pops) | PARTIAL |
| STKM (55) | YES | Disappears | PARTIAL |
| FIGH (158) | YES | Disappears | PARTIAL |

## 4. Performance

| Metric | Value |
|---|---|
| FPS | 60 |
| TPS | 60 |
| Console errors | 0 |

## 5. Confirmed defects

- **P2**: Loop edge mode doesn't wrap (Agent 1)
- **P2**: Newtonian gravity not observable (Agent 2)
- **P2**: Advanced elements partial (Agent 3)
- **P2**: Water spreading limited (Agent 2)

## 6. Overall K5 acceptance

**PARTIAL** — Core physics and chemistry verified. Frontend routing fixed.
Advanced elements remain partial. Persistence and UI features need verification.
No P0/P1 defects. Four P2 defects documented.
