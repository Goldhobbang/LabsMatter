# A1 Status

- Current task: K5 editing / rendering / tools / decoration / inspection
- State: IMPLEMENTATION READY (gameplay acceptance delegated to Agent B)
- Last updated: 2026-11-08
- Branch: agent1/kernel
- K4 base: dbe2d76214b681fdf03b647200ac3e1e949d7339
- Worktrees: AICrew (agent1/kernel), AICrew-agent2 (agent2/physics),
  AICrew-agent3 (agent3/chemistry-elements)
- Notes:
  - Walls and decoration now render; energy particles render. Wall paint/erase
    and powered `eglow` are visible.
  - Decoration editor (per-particle dcolour) added, persisted in LMT2 v3.
  - apply_tool covers 11 reference tools (HEAT/COOL/AIR/VAC/PGRV/NGRV/MIX/CYCL/
    WIND/AMBP/AMBM); PGRV/NGRV await the A2 gravity-well solver.
  - Inspection API: get_cell_info JSON + per-field getters, world settings,
    entity head, decoration, render buffer (CPU-side).
  - Fixed empty-pixel temperature/HEAT bugs (`pmap_id(0)==0`).
  - Feature matrix: docs/tpt-parity/inventory/application-features.md.
  - Report: communication/agents/A1/reports/K5-A1-20261108.md
  - Playtest handoff: docs/tpt-parity/qa/K5-handoff.md
  - Agent 3 reconciled to K4 (no source diff); Agent 2 has no new physics.
  - Agent B K3 code-level + live browser audits committed; P0/P1 reconciled.
