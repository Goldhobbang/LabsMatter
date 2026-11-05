# TPT-QA-0004 — Air, gravity, electronics, and edges are not simulated

| | |
|---|---|
| Severity | P1 |
| Feature | Air, gravity modes, Newtonian gravity, ambient heat, edge modes, powered materials |
| LabsMatter | `b660f4adf9aea7dab697a0ead0aad2ec3c6ab226` |
| Browser | Chromium, Linux, WebGL2 |
| Status | Open |

## Reference

README at `2b7221a46d378589b4de7683f16d3ebc92be6bd9` lists W (gravity modes), Y (air modes), Ctrl+E (edge modes), N (Newtonian gravity), U (ambient heat), and the intro text describes electronics and powered materials. Element sources include ARAY, BCLN, BTRY, CLNE, and others. TPT was not executed.

## Actual

The right rail is categories, draw/select, three brush shapes, five view modes, grid, and HUD. There is no gravity cycle, air mode, edge mode, Newtonian toggle, or ambient-heat toggle. Pressing those keys does nothing (see TPT-QA-0003).

What did run: sand falls downward about one cell per frame; heat view colors lava and cold sand differently; empty cells stay the dark clear color. That is not air velocity or Newtonian gravity.

Pressure view and velocity view exist as keys 4 and 5. This session did not pixel-check them, so they are not passed and not failed here.

## Reproduce

1. Open the app.
2. Look for W/Y/N/U controls or press those keys.
3. Expected: the sim's gravity or air mode changes and the HUD or a menu says so.
4. Observed: no control and no mode change.

## Suspected component

No WASM air or gravity-mode API is hooked to the UI. Do not treat the missing buttons as a rendering bug.
