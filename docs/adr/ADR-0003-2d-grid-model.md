# ADR-0003: 2D Grid Model

**Status:** Accepted
**Date:** 2026-11-04
**Owner:** A1

## Context

Physics/chemistry sandbox could be 2D or 3D. 3D is far more expensive in
browser WASM.

## Decision

Use a 2D grid of cells (falling-sand style) with chunking. 3D deferred to
future version.

## Consequences

- Tractable performance budget.
- Simpler rendering (2D WebGPU).
- Rigid bodies (3D) deferred.
