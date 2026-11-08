# TPT-QA-0002 — F steps but does not pause

| | |
|---|---|
| Severity | P2 |
| Feature | Shortcut `F` |
| LabsMatter | `b660f4adf9aea7dab697a0ead0aad2ec3c6ab226` |
| Browser | Chromium, Linux, WebGL2, 1280×800, DPR 1 |
| Status | Open |

## Reference

README at `2b7221a46d378589b4de7683f16d3ebc92be6bd9`:

> F — Pause and step to next frame

TPT itself was not run.

## Actual

With the sim already running, the Play button read `Pause`. Dispatching `KeyF` left the button as `Pause`. `F` does advance a paused sim: sand moved from about y=12 to y=42 across 30 `F` presses, one cell per press, downward.

So single-step works. The pause half of the shortcut does not.

## Reproduce

1. Open the app. Press Space so the button says Pause.
2. Press F.
3. Expected: button returns to Play and exactly one frame runs.
4. Observed: button stays Pause.

## Suspected component

`web/src/main.js` action `step` only calls `sim.step`. It does not set `playing` to false.
