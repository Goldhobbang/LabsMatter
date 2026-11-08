# Implementation status

Dispatch tables: `crates/realmatter-tpt/src/updates.rs`.

| Metric | Value |
|---|---:|
| Live reference elements | 195 |
| Elements with a reference `Update` callback | 132 |
| Dedicated Rust handlers in `update_fn` | 123 |
| Blocked (missing subsystem) | 9 |
| Internal-only (no Update) | 63 |

## Blocked entries

| abbr | reason |
|---|---|
| STKM | requires player controller (`Simulation::player`) |
| PIPE | full flow requires `portalp` payload channel storage |
| PRTI | uses a thread-local portal channel (not part of the saved world) |
| PRTO | uses a thread-local portal channel (not part of the saved world) |
| WIFI | wireless channel state lives outside `Sim`; transient only |
| STKM2 | requires player controller (`Simulation::player2`) |
| SOAP | bubble/link physics simplified without particle-index links |
| FIGH | requires fighter registry (`Simulation::fighters[]`) |
| PSTN | simplified piston; full `MoveStack`/`FRME` interaction pending |
