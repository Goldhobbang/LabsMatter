# REALMATTER — Performance Budgets

**Version:** 0.1.0
**Owner:** A1

## Targets (mid-range hardware, release build)

- Frame rate: 60 FPS sustained.
- Tick budget: ≤ 8 ms per fixed step.
- Render budget: ≤ 8 ms per frame.
- Memory: ≤ 512 MB for default world (512×384).
- WASM binary: ≤ 2 MB gzipped.

## Measurement

- Benchmarks owned by A8 (Verification).
- Use `criterion` for micro-benchmarks; browser profiling for end-to-end.
- Regressions > 10% fail CI.

## Hot-path rules

- No allocations in tick hot path (reuse buffers).
- No `println!`/`dbg!` in release.
- Prefer SoA storage for cells in world crate.
- SIMD allowed only in audited modules.