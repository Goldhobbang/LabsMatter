# Runtime Verification Guidelines

**Last updated:** M4 (`87f02f0` baseline → M4 commit)
**Scope:** manual and scripted verification of the LabsMatter browser sandbox.

These are *guidelines for a human or scripted operator*. Per project protocol no
automated test suites, fixtures, or mocks are added to the repository; the
commands below exercise the real artifacts.

## 0. Build and artifacts

```bash
cargo check --workspace          # expect: no errors
cargo build --workspace --release
cargo build -p realmatter-wasm --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/realmatter_wasm.wasm \
  --out-dir web/pkg --target web --no-typescript
```

Serve `web/` (e.g. `python -m http.server 5173 --directory web`) and open
`http://localhost:5173/index.html`. Console must show
`Using WASM simulation (real module)` and the renderer backend.

## 1. Determinism

1. Fresh page, paint a fixed stroke (e.g. sand at 3 known cells), run 300 ticks.
2. Reload, repeat the identical stroke and tick count.
3. Cell materials and temperatures must match exactly.

Sandboxed check: call `init`/`set_cell`/`step` in DevTools, snapshot
`get_cell_material` + `get_cell_temperature` via `tick_count()`, repeat, compare
bit-for-bit. Expected: identical (no RNG anywhere in the pipeline).

## 2. Conservation

1. Isolate a two-cell pair (lava 1500 K next to iron) away from walls.
2. Sum `temperature * density * specific_heat` over all non-empty cells.
3. Run 100+ ticks. Expected drift `< 1%` (measured 0.00%).

Fire scenes *increase* total energy by design (combustion enthalpy) — exclude
them from conservation checks.

## 3. Chunk-boundary interactions

1. Paint material straddling a 32×32 chunk boundary (e.g. x = 31/32).
2. Verify it falls/flows across the boundary without stalling.
3. Verify heat and fire cross boundaries (lava next to wood across a seam).
4. Verify a region that was empty and becomes active (paint into a previously
   empty quadrant) starts simulating on the next tick — no frozen regions.

## 4. Fire propagation

1. Paint a wood block (7) and a single fire cell (6) adjacent.
2. Expected: fire ignites the block within 1–2 ticks, spreads through it, wood
   is fully consumed, and fire converts to smoke (12) after ~8 s.

## 5. Heat transfer

1. Lava (1500 K) adjacent to iron: iron warms within 100 ticks.
2. Ice (263 K) placed in a warm region melts toward ambient, not below 0 K.
3. No cell should ever read NaN/Inf or negative temperature.

## 6. Save / load

1. Paint a distinctive pattern, note `tick_count()`.
2. Save (UI Save button → `.rmt` file), Reset, then Load.
3. Materials, temperatures, and `tick_count` must be restored.
4. Load a corrupted file (truncate bytes): expect a clean failure (no crash).

## 7. Performance

1. Console `FPS` should hold ~60 on the default scene.
2. Dense 256×256 scene (sand + water + wood + fire blocks) target: 45+ TPS
   (release/opt-level 3). Record `dense step ms` and TPS.
3. Sparse scene (a few painted cells): expect < 5 ms/step.
4. Measure painting separately: 10k `set_cell` calls should be ~2 ms.

Reference numbers (release build, 256×256, Node.js V8, WASM step only):

| Workload | avg | p95 | TPS |
|----------|-----|-----|-----|
| empty | 3.99 ms | 5.37 ms | 251 |
| sparse | 4.10 ms | 5.18 ms | 244 |
| dense mixed | 7.59 ms | 10.43 ms | 132 |
| full flood (all cells filled) | 17.41 ms | 20.16 ms | 57 |

Profile comparison (dense mixed, before the clone-elimination work):

| Workload | opt-level 3 | opt-level "z" |
|----------|-------------|---------------|
| dense 256² | 21.6 ms (46 TPS) | 41.1 ms (24 TPS) |
| empty 256² | 4.8 ms | 6.4 ms |
| artifact | 212.8 KB | 200.7 KB |

Note: `hud-state`, FPS, and TPS are repainted from the animation frame loop. In
a headless or throttled environment, read `btn-play[aria-pressed]` (updated
synchronously) rather than the HUD text when checking play/pause.

## 8. Long-running stability

1. Run an active scene (fire + falling sand + water) for 5+ minutes.
2. Watch for: memory growth (`getDiagnostics().memoryMB`), FPS decay, frozen
   regions, NaNs (temperature sidebar), and console errors.
3. Expected: bounded memory (WASM memory grows only to steady state), stable
   FPS, no errors.

## 9. Regression checklist (must all pass)

- Sand falls and settles on walls.
- Water flows sideways and settles.
- Walls never move.
- Eraser clears cells.
- Reset clears the world *and* the tick counter.
- Pause / resume / single-step / speed controls respond.
- No console errors during a 2-minute session.
