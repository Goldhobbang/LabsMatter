# A2 → A5 Handoff Requirements (WASM Integration)

- Owner: A0
- Date: 2026-11-04
- Target: A2 to deliver, A5 to consume

## Required Deliverables
1. **WASM boundary contract document** (in `docs/` or `communication/`) defining:
   - Exports: `init`, `tick`, `reset`, `set_material`, `paint`, etc.
   - Imports/callbacks if any
   - Types passed across boundary
   - Error handling

2. **Interface signatures**
   - Use `wasm-bindgen` compatible types
   - Coordinate system (2D world coords)
   - Units

3. **Build artifacts**
   - wasm32-unknown-unknown build target working
   - `realmatter-wasm` crate compiles

4. **Ownership boundaries**
   - A5 uses adapters only, no direct core modifications
   - Contract versioning

## Acceptance Criteria
- A5 can bind without inventing APIs
- Contract reviewed by A0
- Matches core types
