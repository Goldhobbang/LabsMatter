# A1 Status

- Current task: K5 integration — Agent 3 merge + runtime routing verification
- State: implementation ready; gameplay acceptance delegated to Agent B
- Last updated: 2026-11-08
- Branch: agent1/kernel
- Worktrees: AICrew (agent1/kernel), AICrew-agent2 (agent2/physics),
  AICrew-agent3 (agent3/chemistry-elements)
- Notes:
  - Runtime routing P1 (Agent B): NOT present in the integrated build.
    `?engine=tpt` instantiates `TptAdapter` / `labsmatter-wasm`; `/` uses
    `WasmDirectAdapter` / `realmatter-wasm`. Verified in a real browser via
    `window.labsmatter.{requestedEngine,engine,adapter,wasmIdentity}` and the HUD
    (active adapter, `MISMATCH` guard). See
    `docs/tpt-parity/qa/runtime-routing-verification.md`.
  - Agent 3 branch (d096999) merged: WIFI, PIPE/PPIP, PSTN, SOAP, STKM/STKM2,
    FIGH. WASM rebuilt from merged source.
  - Fighter slot indexing fixed: `Sim::fighter_head(slot)`; `entity_head` uses it.
  - Loop mode now wraps energy particles (previously killed before `do_move`).
  - Partials retained: PIPE HEAC/graphics, PSTN FRME, SOAP reconnect, STKM legs/
    portal, FIGH powers/multi-fighter.
  - K5 editing/rendering/tools/decoration/inspection from the earlier report
    remain; see `docs/tpt-parity/inventory/application-features.md`.
  - Report: communication/agents/A1/reports/K5-A1-20261108.md
  - Handoff: docs/tpt-parity/qa/K5-handoff.md
