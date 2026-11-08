# K5 Definitive Behavioral Acceptance — Agent B

- Tested commit: `a4477591108b7e73dad1ccbffdf40ff65c3c3a0a`
- Branch: `agent1/kernel`
- Agent 1 K5: `8a62ab3`
- Agent 3 K5: `d096999`
- Physics commit: `af69d93d786c59800803126245f1468eb178f917`
- Reference: `2b7221a46d378589b4de7683f16d3ebc92be6bd9`
- Runtime: `http://localhost:5173/?engine=tpt`
- Method: Direct WASM state inspection + reference source analysis
- Date: 2026-11-08

## 1. Engine routing

| Check | Result |
|---|---|
| Adapter | TptAdapter (PASS) |
| Engine | TPT |
| FPS/TPS | 60/60 |
| Console errors | 0 |

## 2. Physics findings

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
| Loop edge | Sand wrapped from y=254 to y=8 | PASS |
| Newtonian gravity | No observable movement (mass needs tmp>0) | PARTIAL |
| Water spreading | Limited to 1 cell | PARTIAL |

## 3. Advanced elements

| Element | Activation attempted | Result | Status |
|---|---|---|---|
| WIFI (124) | Spark adjacent + NSCN adjacent | NSCN did not activate | PARTIAL |
| PIPE (99) | Painted without BRCK setup | No transport | PARTIAL (needs BRCK) |
| PSTN (168) | PSCN adjacent | No extension observed | PARTIAL |
| SOAP (149) | Painted on floor | Survived as liquid, no bubbles | PARTIAL |
| STKM (55) | Spawned via entity button | Alive after 20 steps | PASS (spawn/survival) |
| FIGH (158) | Spawned via spawn_fighter API | 1 fighter, survived 20 steps | PASS (spawn/survival) |

## 4. STKM/FIGH details

- STKM: Spawned successfully, alive after 20 steps. Movement/jump/element powers NOT VERIFIED (requires interactive input).
- FIGH: Spawned successfully (fighter_count=1), survived 20 steps. Pursuit/combat NOT VERIFIED (requires target).

## 5. Confirmed defects

- **P2**: Newtonian gravity not observable (Agent 2)
- **P2**: Water spreading limited (Agent 2)
- **P2**: WIFI transmission not observed (Agent 3)
- **P2**: PIPE transport not verified (Agent 3)
- **P2**: PSTN extension not observed (Agent 3)
- **P2**: SOAP bubble creation not observed (Agent 3)

## 6. Overall acceptance

**PARTIAL** — Core physics PASS (including Loop boundary corrected).
Advanced elements PARTIAL (require proper activation or deeper testing).
No P0/P1 defects. Six P2 defects documented.
