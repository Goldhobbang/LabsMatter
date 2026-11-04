# ADR-0002: Numerical Precision

**Status:** Accepted
**Date:** 2026-11-04
**Owner:** A1

## Context

Need balance between performance (WASM) and determinism/accuracy.

## Decision

- Spatial state (positions, velocities): f32.
- Time accumulation: f64.
- Per-tick delta: f64 (Time).
- Mass/density/temperature: f32.
- Accumulators summing many small values: f64 internally, cast to f32 at write.

## Consequences

- Good WASM performance and WebGPU compatibility.
- Deterministic on single target.
- Cross-platform bitwise equality is best-effort.