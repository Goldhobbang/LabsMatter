# Retest of `d82210de12c8ad7ee0826455b71dfe213934fb2d`

Independent play. Implementation files were not modified. Original reports TPT-QA-0001 through 0004 were not edited.

| | |
|---|---|
| Commit | `d82210de12c8ad7ee0826455b71dfe213934fb2d` (HEAD matched) |
| Reference | TPT `2b7221a46d378589b4de7683f16d3ebc92be6bd9` README controls. TPT binary was not run. |
| Browser | HeadlessChrome 156, Linux x86_64, WebGL2 |
| WebGPU | NOT_VERIFIED. Console: `WebGPU init failed, falling back: No WebGPU adapter` |
| Viewport | 1280×800 DPR 1, then 390×844 DPR 2 |

## Defect table

| Defect | Previous | New | Evidence | Remaining gap |
|---|---|---|---|---|
| TPT-QA-0002 `F` | FAIL | **PASS** | Running sim: Play button `Pause`, tick 0. After `F`: button `Play`, tick 1. Next `F` while paused: sand y 9 to 10, button stays `Play`. | None for this shortcut. |
| TPT-QA-0003 keys | FAIL | **PARTIAL** | `Alt+[` : `4×4 c` to `3×3 c`. `W` `Y` `N` `U` `Ctrl+E` change `sim_status` and the effects below. `G` then `G`: HUD grid 16 then 32. `Shift+G`: 16. Stamp `R`: horizontal pair placed at (20,20)(21,20); after `R`, (20,20)(20,21). | `B` still opens nothing. Grid lines: 0 mid-gray samples on an empty frame, so the overlay is not shown on void. `Shift+R` / `Ctrl+Shift+R` flips were not exercised. |
| TPT-QA-0004 physics | FAIL | **PARTIAL** | Gravity off: sand stays (40,10) for 12 steps. Gravity down: (40,18) after 8 steps. Radial from (20,80): (26,80), toward center on x. Air on: smoke (80,40) to (80,32). Air off: stays (80,40). Void edge: border sand gone. Solid edge: sand remains at x=0 and falls to y=45. Wrap: sand from y=255 found at (40,2). Newtonian, gravity off: lone sand (26,40) moves to x=29 next to a block. Ambient, gravity off: lava 1500 K to 973 K after 100 steps. SPRK on iron: columns 11,12,13,14,15. | Not TPT air (no velocity field carried into powders). Newtonian is a local sign sum, not 1/r². Ambient has no latent heat. Electricity is spark-on-iron only. Do not close the P1. |
| TPT-QA-0001 elements | FAIL | **FAIL** | `docs/tpt-parity/elements.md` lists 17 selectable ids (0–16) and says the 195 `.cpp` files are not 195 menu entries. Palette still has no portals, clones, life, pipes, or stickmen. | Element parity is not met. |

## Regressions

No regression found in the checks that completed:

- DPR 2, 390×844: click CSS (80,40) centers sand on cell (20,10). Canvas 390×661, backing store 780×1322.
- Iron (3,4) save then load restores material 8.
- Default gravity still moves sand down after modes are set back.
- Stamp save of a 1×2 sand line restores that shape (`dx` 0 and 1).

Undo, water spread, and middle-click sample were not repeated as full scripts on this commit. They were not seen to break during the stamp and draw setup. Not re-failed.

## Still open for Agent A

1. P1 TPT-QA-0001: elements beyond the 17.
2. P1 TPT-QA-0004: air velocity, real Newtonian gravity, electronics other than SPRK-on-iron, latent heat. Keep the modes; do not treat them as done.
3. P2 grid: size keys work; lines were not visible on empty space.
4. P2 `B` decoration editor: still missing. Do not add an empty dialog.
5. Not verified here: pressure and velocity view pixels, Fancy mode, flood fill, copy/cut, screenshot file, stamp flip keys, Canvas2D, touch gestures.

Acceptance of the whole game: **not accepted**.
