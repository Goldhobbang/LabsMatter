# ADR-0001: Workspace Structure

**Status:** Accepted
**Date:** 2026-11-04
**Owner:** A1

## Context

REALMATTER needs a modular Rust/WASM codebase with clear ownership boundaries for parallel agent development.

## Decision

Use a Cargo workspace with one crate per domain module (core, world, physics, chemistry, renderer, ui, persist, wasm). `realmatter-core` holds shared types/traits and has no realmatter dependencies.

## Consequences

- Clear ownership and dependency direction.
- Parallel compilation.
- Agents can work independently within owned crates.