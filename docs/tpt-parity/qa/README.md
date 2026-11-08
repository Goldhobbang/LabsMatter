# LabsMatter gameplay QA — session 1

Independent play of the running app. No source fixes in this session.

| | |
|---|---|
| Role | Agent B |
| LabsMatter commit | `b660f4adf9aea7dab697a0ead0aad2ec3c6ab226` |
| Branch | `cursor/tpt-parity-9315` |
| TPT reference | `2b7221a46d378589b4de7683f16d3ebc92be6bd9` |
| TPT executed | No. Expected behavior is from that commit's README controls table and `src/simulation/elements/*.cpp` (195 files). |
| Browser | Chromium, Linux, WebGL2. WebGPU adapter not present (`WebGPU init failed, falling back`). |
| Viewports | 1280×800 at DPR 1; 390×844 at DPR 2 |
| Live site | Not this commit. https://goldhobbang.github.io/LabsMatter/ was not used as evidence. |
| Acceptance | **Not accepted.** |

`docs/tpt-parity/` had no Agent A inventory. Counts below are from the pinned TPT tree and from play, not from an inventory file.

## Coverage

| Area | Applicable | Verified pass | Failed | Missing / not built | Not verified |
|---|---:|---:|---:|---:|---:|
| TPT element sources | 195 | 0 full parity | — | 179 not in the palette | 16 present, only some played |
| Palette materials played through the UI | 16 | 4 placed or sampled (SAND, WATR, STNE, OIL) | 0 | — | 12 not clicked |
| README shortcuts | 40 listed rows | 14 observed | 2 | most air/gravity/decoration keys absent | rest |
| Display modes 0–9 | 10 | Heat (key 3) and Normal (key 1) | — | 0 and 6–9 | Fancy, pressure, velocity not pixel-checked |
| Renderers | 3 | WebGL2 | 0 | — | WebGPU, Canvas2D |

Played and observed:

- Left drag places sand under the cursor (49 cells, brush `4×4 c`, center matches `css/4`).
- Thirty `F` steps move that sand from about y=12 to y=42 (down).
- Water places. Right drag with the default secondary (VOID) clears it.
- Palette right-click sets secondary to OIL.
- Wheel changes `4×4 c` to `5×5 c`. Tab changes the shape letter to `s`.
- Shift-drag draws a sand line (41 cells on one row). Ctrl-drag fills a rectangle (256 cells). Ctrl+Z removes that rectangle.
- Middle click samples STNE into the primary chip.
- `E` focuses element search. `H` hides the HUD. `K` opens the stamp dialog. Space starts the sim (`Play` becomes `Pause`).
- Heat view: lava pixel `(207,255,37)`, cold sand `(14,0,188)`, empty canvas stays the clear color `(5,8,15)`. HUD reads `Heat`.
- Iron at (3,4) survives save blob then reset then load.
- After ~0.7s of play, HUD showed FPS 61 on WebGL2. TPS was still 0 because the one-second window had not elapsed.
- 390×844, DPR 2, fresh load: canvas CSS 390×661, backing store 780×1322. Click at CSS (80,40) lands on cell (20,10), the center of the 4-radius circle. Rail and palette remain on screen.

## Open defects

| ID | Severity | Summary |
|---|---|---|
| [TPT-QA-0001](TPT-QA-0001.md) | P1 | 179 reference elements and whole categories are not in the game |
| [TPT-QA-0002](TPT-QA-0002.md) | P2 | `F` steps but does not pause a running sim |
| [TPT-QA-0003](TPT-QA-0003.md) | P2 | Many README keys do nothing (`Alt+[`, gravity, air, decorations, stamp rotate) |
| [TPT-QA-0004](TPT-QA-0004.md) | P1 | No air, Newtonian gravity, ambient heat, electronics, or edge modes |

## Not claimed

Flood fill, stamp place/rotate, copy/cut, screenshot file contents, Fancy/pressure/velocity colors, grid lines at 1× zoom, Canvas2D, WebGPU, touch-only gestures, and the public GitHub Pages build were not given a pass.

---

## Experimental TPT runtime handoffs (current)

The session-1 notes above describe the **production** engine. The experimental
TPT-parity runtime is documented separately and must be tested through
`?engine=tpt`:

- **K3 handoff (browser + build steps + boundary/persistence checks):**
  [`K3-handoff.md`](K3-handoff.md)
- **K3 independent audit (code-level, no browser):**
  [`K3-retest-agentB.md`](K3-retest-agentB.md)
- **K4 handoff (entity controls, sim tools, default properties, persistence):**
  [`K4-handoff.md`](K4-handoff.md)

Run: `cd web; python -m http.server 5173` then open
`http://localhost:5173/?engine=tpt`.
- **K5 handoff (walls/decoration rendering, tools, inspection API):**
  [`K5-handoff.md`](K5-handoff.md)
- **K7 handoff (graphics, glow/bloom, gravity view, full-state editing, inspection):**
  [`K7-handoff.md`](K7-handoff.md)
