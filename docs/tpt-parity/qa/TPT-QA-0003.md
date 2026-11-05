# TPT-QA-0003 — Reference keys that do nothing or do something else

| | |
|---|---|
| Severity | P2 |
| Feature | Keyboard shortcuts |
| LabsMatter | `b660f4adf9aea7dab697a0ead0aad2ec3c6ab226` |
| Browser | Chromium, Linux, WebGL2 |
| Status | Open |

## Reference

README controls table at `2b7221a46d378589b4de7683f16d3ebc92be6bd9`. TPT was not executed.

## Actual

Observed on the Powder preset:

| Input | Reference | Observed |
|---|---|---|
| Alt+[ | Brush size decreases by 1 | Readout stayed `5×5 s` |
| G | Increase grid size | Button `GRID` gained `is-on`. Grid-size steps were not observed. Line pixels were not confirmed. |
| 0–9 | Ten view modes | Keys 1 and 3 switch Normal and Heat. 0 and 6–9 are not view modes. |
| R while pasting | Rotate stamp | No paste rotate control was found. |
| W, Y, Ctrl+E, N, U, B, Ctrl+B, backtick, `=` | Gravity, air, edges, Newtonian gravity, ambient heat, decorations, console, reset maps | No effect observed and no such controls are on the rail. |

These are one shortcut gap, not separate broken tools. The systems those keys drive are [TPT-QA-0004](TPT-QA-0004.md) and [TPT-QA-0001](TPT-QA-0001.md).

Working on this commit, so they are not part of this defect: Space, F-as-step, Tab, wheel brush, E search, H HUD, K stamp dialog, Ctrl+Z material undo, left/right draw, Shift line, Ctrl rectangle, middle-click sample.

## Reproduce

1. Powder keys preset.
2. Grow the brush with the wheel until the readout says `5×5`.
3. Press Alt+[.
4. Expected: `4×4` or a one-pixel step down.
5. Observed: readout unchanged.

## Suspected component

`web/src/shortcuts.js` powder preset. Alt+[ is not bound. G is bound to a boolean grid toggle.
