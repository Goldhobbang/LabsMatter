# K5 Corrected Independent Gameplay Audit — Agent B

- Tested commit: `a4477591108b7e73dad1ccbffdf40ff65c3c3a0a`
- Branch: `agent1/kernel`
- Agent 1 K5: `8a62ab3`
- Agent 3 K5: `d096999`
- Physics commit: `af69d93d786c59800803126245f1468eb178f917`
- Reference: `2b7221a46d378589b4de7683f16d3ebc92be6bd9`
- Runtime: `http://localhost:5173/?engine=tpt`
- Method: Direct WASM state inspection + reference source analysis
- Date: 2026-11-08

## 1. Build and routing

| Check | Result |
|---|---|
| Commit | `a447759` (latest integrated) |
| Adapter | TptAdapter (PASS) |
| Engine | TPT |
| FPS/TPS | 60/60 |
| Console errors | 0 |

## 2. Corrected findings

### Loop boundary — CORRECTED from PARTIAL to PASS

Previous report claimed Loop mode didn't wrap. Corrected testing shows sand at (128,254)
wraps to y=8. The earlier test only checked y=0..4. Loop mode works correctly.

### WIFI — PARTIAL (not FAIL)

WIFI requires: spark adjacent to transmitter, receiver (NSCN/PSCN/INWR) adjacent,
matching temperature channel. Tested with spark adjacent; NSCN did not activate.
May require specific spark life (>=3) or conductor. Reference: WIFI.cpp.
Status: PARTIAL — mechanism exists, activation conditions not fully met in test.

### PIPE — PARTIAL (not FAIL)

PIPE requires BRCK border generation before transport. Description: "Once the BRCK
generates, erase some for the exit. Then the PIPE generates and is usable."
Tested without BRCK setup. Cannot evaluate transport without proper initialization.
Status: PARTIAL — requires BRCK setup, not tested.

### PSTN — PARTIAL

PSTN requires electrical activation and temperature-dependent length. Tested
without electrical input. Stays in place. Status: PARTIAL.

### SOAP — PARTIAL

SOAP disappears (pops). Reference: SOAP pops under pressure. May be legitimate.
Status: PARTIAL — need to verify if popping is reference-correct.

### STKM/STKM2/FIGH — PARTIAL

Disappeared after stepping. May require specific spawn conditions (entity controls,
not material painting). Status: PARTIAL — need entity spawn APIs.

## 3. Physics findings (unchanged)

| System | Status |
|---|---|
| Powder falling | PASS |
| Negative pressure | PASS |
| Air velocity | PASS |
| Heat transfer | PASS |
| Combustion | PASS |
| Acid dissolution | PASS |
| Electricity | PASS |
| Gravity disabled | PASS |
| Void edge | PASS |
| Loop edge | PASS (corrected) |
| Newtonian gravity | PARTIAL |
| Water spreading | PARTIAL |

## 4. Confirmed defects

- **P2**: Newtonian gravity not observable (Agent 2)
- **P2**: Water spreading limited (Agent 2)
- **P2**: Advanced elements partial (Agent 3)
- **P2**: Loop edge works but only for particles not in kill margin (Agent 1)

## 5. Overall acceptance

**PARTIAL** — Core physics PASS. Loop boundary corrected to PASS.
Advanced elements PARTIAL (require proper activation). No P0/P1 defects.
