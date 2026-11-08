# Generic / passive element audit (K5)

Elements with no `Update` callback but with source-defined `create`,
`graphics`, or temperature/pressure transition behaviour. The engine must
provide those behaviours centrally; this audit records what each relies on.

| abbr | create | graphics | transitions | engine support |
|---|---:|---:|---|---|
| NONE |  |  | - | movement only |
| DUST |  |  | - | movement only |
| OIL |  |  | highT | transition_phase |
| STNE |  |  | highT | transition_phase |
| LAVA | Y | Y | lowT | element_defaults.rs, base color render, transition_phase |
| GUNP |  |  | highT | transition_phase |
| NITR |  |  | highT | transition_phase |
| GAS |  |  | highT,highP | transition_phase |
| PLEX |  |  | highT | transition_phase |
| METL |  |  | highT | transition_phase |
| VOID |  |  | - | movement only |
| CNCT |  |  | highT | transition_phase |
| SALT |  |  | highT | transition_phase |
| DMND |  |  | - | movement only |
| WAX |  |  | highT | transition_phase |
| MWAX |  |  | lowT,highT | transition_phase |
| PSCN |  |  | highT | transition_phase |
| NSCN |  |  | highT | transition_phase |
| LNTG |  |  | lowT,highT | transition_phase |
| INSL |  |  | - | movement only |
| BHOL |  |  | - | movement only |
| WHOL |  |  | - | movement only |
| RBDM |  |  | highT | transition_phase |
| LRBD |  |  | lowT,highT | transition_phase |
| SAND |  |  | highT | transition_phase |
| BGLA |  |  | highT | transition_phase |
| PLSM | Y | Y | - | element_defaults.rs, base color render |
| ETRD |  |  | - | movement only |
| NICE |  |  | highT | transition_phase |
| SMKE |  | Y | highT | base color render, transition_phase |
| DESL |  |  | highT,highP | transition_phase |
| LO2 |  |  | highT | transition_phase |
| INWR |  |  | highT | transition_phase |
| DYST |  |  | highT | transition_phase |
| THRM |  |  | - | movement only |
| BRCK |  | Y | highT,highP | base color render, transition_phase |
| CFLM | Y | Y | - | element_defaults.rs, base color render |
| BCOL |  |  | - | movement only |
| LIFE | Y | Y | - | element_defaults.rs, base color render |
| DRIC |  |  | highT | transition_phase |
| TESC | Y |  | - | element_defaults.rs |
| LOVE |  |  | - | movement only |
| BIZRG |  |  | highT | transition_phase |
| BIZRS |  |  | lowT | transition_phase |
| INST |  |  | - | movement only |
| PSTE |  |  | highT,highP | transition_phase |
| PSTS |  |  | lowP | transition_phase |
| E116 |  |  | - | movement only |
| SPAWN2 |  |  | - | movement only |
| SPAWN |  |  | - | movement only |
| LOLZ |  |  | - | movement only |
| FILT | Y | Y | - | element_defaults.rs, base color render |
| BRAY |  | Y | - | base color render |
| PQRT | Y |  | highT | element_defaults.rs, transition_phase |
| EMP |  | Y | - | base color render |
| PPIP |  |  | - | movement only |
| BVBR |  |  | - | movement only |
| FRME |  | Y | - | base color render |
| VRSS |  | Y | highT | base color render, transition_phase |
| VRSG |  | Y | lowT | base color render, transition_phase |
| SAWD |  |  | - | movement only |
| RFGL |  |  | lowP | transition_phase |
| ROCK | Y | Y | highT,highP | element_defaults.rs, base color render, transition_phase |

The K5 audit found no Generic/Passive element whose source declares a
create/graphics/transition behaviour that is missing from the engine:
reference `DefaultProperties` are applied by `element_defaults.rs` at
particle creation, base colours come from the generated element table, and
all declared temperature/pressure transitions are applied by
`transition_phase`.
