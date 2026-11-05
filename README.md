# LabsMatter

A browser-based, offline-capable 2D falling-sand physics and chemistry sandbox.
Rust simulation core compiled to WebAssembly, JavaScript front end,
WebGPU → WebGL2 → Canvas2D rendering fallbacks. No external services, no paid
APIs, no backend, no accounts.

**Live site: <https://goldhobbang.github.io/LabsMatter/>**

## Quick start

```bash
# 1. Build the simulation core to WebAssembly
cargo build -p realmatter-wasm --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/realmatter_wasm.wasm \
  --out-dir web/pkg --target web --no-typescript

# 2. Build the static site (outputs web/dist)
npm --prefix web run build

# 3. Serve it (any static server)
python -m http.server 5173 --directory web/dist
#   or: python -m http.server 5173   (from inside web/)
```

Open <http://localhost:5173/index.html>. The console should report
`Using WASM simulation (real module)` and the active renderer backend.

The front end has no runtime dependencies and no bundler; `npm run build`
simply assembles `web/dist` from the static sources plus the generated WASM
package.

### Requirements

- Rust stable with the `wasm32-unknown-unknown` target
- `wasm-bindgen-cli` matching the `wasm-bindgen` version in `Cargo.lock`
- Python 3 or Node.js (any static file server)

## Deployment

`.github/workflows/deploy-pages.yml` builds the WASM module, generates
bindings, runs the static build, and publishes `web/dist` to GitHub Pages on
every push to `main` (and on manual `workflow_dispatch`).

The site is served from the `/LabsMatter/` subpath. All asset and WASM URLs are
resolved relative to the modules that load them, so the same build works at a
project subpath, at a domain root, and on localhost without configuration.

**One-time repository setting:** Settings → Pages → Build and deployment →
Source: **GitHub Actions**.

## What works

| Area | Status |
|------|--------|
| Particle simulation (powder, liquid, gas, plasma) | Working |
| Gravity, wall collision, density-driven sinking/swapping | Working |
| Heat conduction (energy-conserving, 0.000% measured drift) | Working |
| Phase transitions (temperature clamping, wetness) | Working |
| Combustion, fire propagation, fuel consumption, smoke | Working |
| Oxidation, dissolution | Working |
| Gas diffusion and buoyancy | Working |
| Paint / erase / line / rectangle / flood fill | Working |
| Pause / resume / single step / speed 0.25×–4× | Working |
| Zoom, pan, HiDPI input mapping | Working |
| Save / load (persist format v1, `.rmt` files) | Working |
| Deterministic simulation (no RNG) | Working |
| WebGL2 renderer | Working |
| Canvas2D renderer | Working |
| WebGPU renderer | Implemented; not verified on hardware |

16 materials are available: void, stone/wall, sand, water, oil, steam, fire,
wood, iron, lava, acid, salt, smoke, ice, glass, gunpowder.

## Architecture

> Internal Rust crate and WASM package names remain `realmatter-*` for
> compatibility with existing build tooling and save files. The product is
> **LabsMatter**.

```
crates/
  realmatter-core      shared types: Cell, materials, coords, world config,                       active-chunk mask, module traits
  realmatter-world     World grid, deterministic movement, tick scheduler
  realmatter-physics   heat, gas, fluid, reactions, transfer, stability
  realmatter-chemistry reaction registry, kinetics, schema, engine
  realmatter-persist   save/load, format v1, material manifests
  realmatter-renderer  (Rust-side renderer scaffolding)
  realmatter-ui        (Rust-side UI scaffolding)
  realmatter-wasm      wasm-bindgen boundary, owns the live World
web/                   offline front end (no build step)
docs/                  architecture, contracts, ADRs, verification
```

### Tick pipeline (`realmatter-wasm::step`)

Follows the approved phase order in `docs/contracts/simulation-tick.md`:

1. Numerical guards (NaN/Inf clamp)
2. Chemistry — registry-driven reactions
3. Physics forces — gas pressure/diffusion/buoyancy, fluid viscosity/pressure/
   buoyancy, heat conduction, phase transitions, reactions, energy/mass transfer
4. Movement — powder and liquid cellular automaton (deterministic, row-major)
5. Finalize — flag clearing, lifetimes, tick counter
6. Export — RGBA8 render buffer refresh

Active-chunk masking (`core::chunk_mask::ActiveMask`) skips empty regions while
preserving cross-chunk interactions; the mask is rebuilt every tick so any
written cell immediately wakes its neighborhood.

## Performance

Measured on a 256×256 world (65,536 cells), release build (`opt-level=3`),
Node.js V8, WASM step time only:

| Scene | avg | p95 | TPS |
|-------|-----|-----|-----|
| empty | 3.99 ms | 5.37 ms | 251 |
| sparse (floor + a few cells) | 4.10 ms | 5.18 ms | 244 |
| dense mixed (sand + water + burning wood) | 7.59 ms | 10.43 ms | 132 |
| full flood (every cell filled) | 17.41 ms | 20.16 ms | 57 |

Release profile comparison (dense mixed workload):

| Profile | step | TPS | artifact |
|---------|------|-----|----------|
| `release` (opt-level 3) | 21.6 ms | 46 | 212.8 KB |
| `release-s` | 37.8 ms | 26 | 198.8 KB |
| `release-z` | 41.1 ms | 24 | 200.7 KB |

`release` (speed) is the shipping configuration. Since the M4 measurements
above, eliminating per-cell `MaterialProps` clones (heap `String` allocation in
every physics lookup) and reusing the chemistry engine across ticks cut the
dense step to 7.6 ms (132 TPS).

Front end assets total ~90 KB of JS/CSS/HTML plus a 212 KB WASM module.

## Controls

| Input | Action |
|-------|--------|
| Left drag | Paint with the selected tool |
| `B` / `E` / `L` / `Q` / `F` | Brush / Eraser / Line / Rect / Fill |
| Space | Play / pause |
| `.` | Single step |
| `Ctrl+R` | Reset world |
| `G` / `H` | Grid / heat overlay |
| Arrows | Pan |
| `+` / `-` / `0` | Zoom in / out / reset view |
| `Esc` | Cancel stroke |
| `?` / `,` | Help / settings |
| Middle drag or Space+drag | Pan |

Save writes a `.rmt` blob; Load restores cells, temperatures, world dimensions,
and the tick counter.

## Persistence format

`persist` version 1, little-endian, CRC32-checksummed: magic `RMT\0`,
format version, bincode header (config, `tick_count`, `time_accumulator`,
label), material manifest, per-chunk cell snapshots (individually
zlib-compressed), recovery metadata, checksum. Corrupt or truncated blobs are
rejected without panicking.

## Limitations

- **WebGPU renderer is not verified on hardware.** The pipeline, texture
  upload, and shader are implemented, but the available verification
  environment (headless Chromium with software rendering) exposes no WebGPU
  adapter, so the application falls back to WebGL2. Treat WebGPU as
  *implemented, unverified*.
- **Heat overlay** shows colors only in live WASM mode; the streamed color
  buffer carries no per-cell temperatures, so the toggle logs a notice instead
  of tinting.
- Phase transitions clamp temperature and set wetness but do not consume latent
  heat or change material IDs (melting ice does not become water automatically).
- `heat::conduct` and `transfer::transfer_energy` both move heat (two
  overlapping but individually conservative models). Consolidating them would
  remove a pass and should be done with a behavior diff.
- Acid, salt, and gunpowder participate in the general material rules
  (density, flammability, conduction) but have no dedicated reaction chemistry.
- No automated test suite is checked in (project protocol); verification is
  manual per `docs/runtime-verification.md`.

## Documentation

- `docs/architecture.md`, `docs/contracts/**` — architecture and contracts
- `docs/runtime-verification.md` — verification guidelines and reference numbers
- `docs/adr/**` — architecture decision records
- `communication/**` — historical multi-agent coordination records

## License

MIT OR Apache-2.0
