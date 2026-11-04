# ADR-0002: Floating Point Precision

**Status:** Accepted
**Date:** 2026-11-04
**Owner:** A1

## Context

Need balance between performance (WASM/WebGPU) and numerical stability.

## Decision

- Spatial state (position, velocity, force): `f32`.
- Time accumulation: `f64`.
- Temperature, pressure, density: `f32`.
- No fast-math reassociation.

## Consequences

- WebGPU-compatible (f32).
- Deterministic across platforms (IEEE-754).
- Some precision loss in long simulations; acceptable for sandbox.
