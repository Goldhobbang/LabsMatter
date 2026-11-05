# TPT-QA-0001 — Most reference elements are absent

| | |
|---|---|
| Severity | P1 |
| Feature | Element registry |
| LabsMatter | `b660f4adf9aea7dab697a0ead0aad2ec3c6ab226` |
| Browser | Chromium, Linux, WebGL2 |
| Status | Open |

## Reference

Pinned commit `2b7221a46d378589b4de7683f16d3ebc92be6bd9`, directory `src/simulation/elements/`: **195** `.cpp` files (ACEL, ACID, AMTR, ANAR, ARAY, BANG, BCLN, BOMB, CLNE, …). TPT was not executed here. The file list is the evidence that those elements exist in the reference.

## Actual

The running palette has 16 entries: VOID, STNE, SAND, WATR, OIL, STM, FIRE, WOOD, IRON, LAVA, ACID, SALT, SMKE, ICE, GLAS, GUN. Categories shown: PWD, LIQ, GAS, SOL, NRG, TOOL, ALL.

This session placed or sampled SAND, WATR, STNE, and OIL through the UI. The other 12 names were visible in code and, for some, in earlier heat/save checks, but they were not each exercised as palette clicks in this audit. None of the electronics, clone, portal, powered, life, pipe, or stickman sources have a palette entry.

## Reproduce

1. Serve this commit and open the app.
2. Click every category on the right rail.
3. Compare the abbreviations to `src/simulation/elements/*.cpp` at the pinned commit.

## Suspected component

`web/src/materials.js` and the WASM material table. Missing elements are not implemented, not merely hidden.
