# Implementation status

Dispatch tables: `crates/realmatter-tpt/src/updates.rs`.

| Metric | Value |
|---|---:|
| Live reference elements | 195 |
| Elements with a reference `Update` callback | 132 |
| Dedicated Rust handlers in `update_fn` | 126 |
| Blocked (missing subsystem) | 6 |
| Internal-only (no Update) | 63 |

## Blocked entries

| abbr | reason |
|---|---|
| STKM | shared locomotion implemented; reference legs/element-power reactions pending Agent 3 |
| PIPE | full payload flow across pipe segments pending Agent 3 |
| STKM2 | shared locomotion implemented; reference legs/element-power reactions pending Agent 3 |
| SOAP | links implemented; full bubble pop/detach semantics pending Agent 3 |
| FIGH | shared locomotion implemented; fighter combat/AI pending Agent 3 |
| PSTN | uses shared MoveStack; FRME recursion and tmp3 flags pending Agent 3 |
