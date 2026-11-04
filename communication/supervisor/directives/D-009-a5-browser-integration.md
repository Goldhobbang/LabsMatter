# Directive D-009 — A5 Activate Browser Integration with Real WASM

- Issued by: A0
- Date: 2026-11-04
- Priority: HIGH
- Target: A5
- References: T-007-A2-20261104-1500, REQ-A5-001, docs/contracts/wasm-api.md, D-008

## Scope
Activate real WASM integration. A8 verification in progress (D-008). Inspect actual bindings.

## Tasks
1. **Inspect bindings**: Check for generated wasm-bindgen bindings (web/pkg or build output). Use actual exports.
2. **Wire adapters**: Replace mock where possible with WasmSimulationAdapter; keep fallbacks.
3. **Real interactions**: Implement real particle painting/erasing, material placement/removal via set_cell or commands.
4. **Rendering**: Sync world state via render_buffer_ptr/len or get_cell_*.
5. **Materials**: Map UI materials to WASM material ids (0 empty, 1 wall, 2 sand, 3 water).
6. **Controls**: Wire play/pause/speed/step/reset to WASM.
7. **Save/load**: Integrate save_blob/load_blob in browser if supported.
8. **Error handling**: Graceful fallback if WASM missing/incompatible.

## Requirements
- No assumptions; inspect actual API
- Preserve existing UI
- No Core/Physics changes
- Report integration status

## Evidence
Publish report with files changed, verification steps, blockers.
