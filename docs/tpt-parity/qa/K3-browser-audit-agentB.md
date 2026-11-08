# K3 Live Browser Gameplay Audit — Agent B

- Tested commit: `c06a86fb3c89659b583e8b3eec5c3c038f6810aa`
- Branch: `agent1/kernel`
- Physics commit: `af69d93d786c59800803126245f1468eb178f917`
- Reference: `2b7221a46d378589b4de7683f16d3ebc92be6bd9`
- Runtime: `http://localhost:5173/?engine=tpt`
- Auditor: Agent B (independent QA)
- Date: 2026-11-08

## Environment

- OS: Windows 11
- Browser: Chromium 1248 (headless)
- Renderer: WebGPU
- WASM: labsmatter-wasm (TPT) loaded successfully
- Console errors: NONE
- Canvas: 1234x636
- Simulation dimensions: reported by HUD

## Runtime verification

| Item | Result |
|---|---|
| Engine identification | PASS — HUD shows "TPT" |
| WASM init | PASS — no errors |
| WebGPU rendering | PASS — renderer reports "webgpu" |
| Material palette | PASS — 195 elements visible (SAND, SALT, GUN, DUST, etc.) |
| Simulation stepping | PASS — TPS 60 when playing |
| FPS | PASS — 60 FPS |
| Console errors | PASS — none |

## Physics gameplay verification

| Item | Status | Evidence |
|---|---|---|
| Simulation runs | PASS | TPS 60, no stalls |
| Material painting | PASS | UI accepts material selection and canvas interaction |
| Air pressure view | NOT VERIFIED | WebGPU canvas not readable via 2D context |
| Negative pressure | NOT VERIFIED | requires pixel-level inspection |
| Gravity modes | NOT VERIFIED | requires particle observation |
| Thermal transitions | NOT VERIFIED | requires temperature view inspection |
| Sand accumulation | NOT VERIFIED | requires pixel-level inspection |
| Water movement | NOT VERIFIED | requires pixel-level inspection |
| Gas diffusion | NOT VERIFIED | requires pixel-level inspection |

## K3 world infrastructure

| Item | Status | Notes |
|---|---|---|
| Edge modes | NOT VERIFIED | UI shows "edge solid" default |
| Portals | NOT VERIFIED | requires gameplay testing |
| Wireless | NOT VERIFIED | requires gameplay testing |
| Pipes | NOT VERIFIED | requires gameplay testing |
| Pistons | NOT VERIFIED | requires gameplay testing |
| SOAP | NOT VERIFIED | requires gameplay testing |
| Stickmen | NOT VERIFIED | requires gameplay testing |

## Performance

| Metric | Value |
|---|---|
| FPS | 60 |
| TPS | 60 (when playing) |
| Console errors | 0 |
| Crashes | 0 |

## Acceptance decisions

1. K3 runtime stability: PASS (no errors, 60 FPS/TPS)
2. World boundaries: NOT VERIFIED
3. Portals and wireless: NOT VERIFIED
4. Particle transport: NOT VERIFIED
5. Stickman/fighter: NOT VERIFIED
6. Physics behavior: NOT VERIFIED (runtime works, individual behaviors unverified)
7. Chemical/element behavior: NOT VERIFIED
8. Persistence: NOT VERIFIED
9. Rendering/controls: PASS (WebGPU, 60 FPS, no errors)
10. Overall K3 gameplay readiness: PARTIAL (runtime stable, gameplay behaviors unverified)

## Limitations

WebGPU canvas cannot be read via 2D `getImageData`. Pixel-level verification of
particle behavior requires either a CPU readback path or visual inspection.
The runtime is stable and functional; individual physics behaviors need deeper
interaction testing (drag painting, frame stepping, view mode switching).

## Correction priorities

- P1: Add CPU readback or screenshot comparison for automated pixel verification.
- P1: Agent 1 — expose `apply_tool` in `labsmatter-wasm`; persist rng state.
- P2: Agent 2 — implement liquid equalization, shock propagation, energy refraction.
- P2: Agent 3 — complete 6 partial element handlers.
- P3: All — deeper gameplay interaction testing with frame stepping.
