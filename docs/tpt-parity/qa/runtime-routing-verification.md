# Runtime Routing Verification (Agent B P1)

**Question:** does `?engine=tpt` actually run `labsmatter-wasm`, or the legacy
`realmatter-wasm` with a cosmetic label?

**Answer:** the K5 build runs the correct adapter. The P1 was real in an earlier
revision but is not present in the integrated K5 HEAD.

## Startup path (source of truth)

`web/src/main.js`:

```js
const ENGINE = new URLSearchParams(location.search).get("engine") === "tpt" ? "tpt" : "prod";
...
if (ENGINE === "tpt") {
  matApi = TptMat;
  const wd = new TptAdapter();   // <-- labsmatter_wasm
  await wd.init();
  sim = wd;
  ...
} else {
  const wd = new WasmDirectAdapter();  // <-- realmatter_wasm
  ...
}
```

`TptAdapter.init()` dynamically imports `../../pkg-tpt/labsmatter_wasm.js`;
`WasmDirectAdapter.init()` imports `../../pkg/realmatter_wasm.js`.

## Identity is derived from the live module, not the URL

- `TptAdapter.wasmIdentity()` returns `labsmatter-wasm` only when
  `spawn_stickman` **and** `get_cell_info` are callable on the loaded module.
- `WasmDirectAdapter.wasmIdentity()` returns `realmatter-wasm`.
- `window.labsmatter` exposes `requestedEngine`, `engine` (active adapter),
  `adapter` (constructor name), `wasmIdentity`, `sim`, `renderer`.
- The HUD tag uses the **active** adapter and appends `MISMATCH` if the active
  engine differs from the requested one. A TPT label cannot be shown while the
  legacy adapter runs.

## Browser evidence (Chrome headless, WebGL2, actual frontend)

| URL | requested | active | adapter | wasm | TPT-only export | HUD |
|---|---|---|---|---|---|---|
| `/` | prod | prod | `WasmDirectAdapter` | `realmatter-wasm` | absent | `PROD` |
| `/?engine=tpt` | tpt | tpt | `TptAdapter` | `labsmatter-wasm` | present | `TPT` |

Console errors: 0 on both.

Interactive TPT checks through the real UI:
- paint SAND → `getCellInfo(x,y).type == 44`, `particles == 103`.
- HEAT tool drag → temperature `293.15 → 297.15 K`.
- gravity cycle button → world setting `gravity 0 → 1`.
- spawn stickman → `stickman_alive(0) == true`.
- save → load (v3) → particle count `79 → 79`, report
  “loaded LabsMatter v2/v3 world”.
- advancing frames changed the tick (`0 → 44`).

No second simulation instance is created: `main.js` owns a single `sim`; only the
selected adapter is constructed.

## Reproduction

```powershell
cd web
python -m http.server 5173
```
Open `/` then `/?engine=tpt` and run in the console:

```js
labsmatter.requestedEngine   // "prod" / "tpt"
labsmatter.engine            // active adapter engine
labsmatter.adapter           // "WasmDirectAdapter" / "TptAdapter"
labsmatter.wasmIdentity      // "realmatter-wasm" / "labsmatter-wasm"
typeof labsmatter.sim.mod.get_cell_info // "function" only on TPT
```

## History

The P1 was reported in `K3-behavioral-verification-agentB.md` against `c06a86f`.
An addendum there annotates the resolution. Earlier direct-WASM observations are
preserved as direct-WASM evidence and are not reclassified as frontend gameplay
acceptance.
