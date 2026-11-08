# Element inventory (generated from pinned TPT reference)

Reference: The-Powder-Toy/The-Powder-Toy @ 2b7221a46d378589b4de7683f16d3ebc92be6bd9

Total slots: 196 (live elements: 195, disabled slots: 1)
Enabled: 195, menu-visible: 175

## Implementation status

| Status | Count |
|---|---:|
| Implemented | 126 |
| Internal | 63 |
| Blocked | 6 |
| Disabled | 1 |

`Implemented` means a dedicated Rust handler exists in
`crates/realmatter-tpt/src/updates.rs` and mirrors the reference
source's core behaviour. `Blocked` entries still have a handler but are
limited by a missing engine subsystem; see `implementation-status.md`.

| id | abbr | name | section | state flags | enabled | menu | update | graphics | create | status |
|---:|---|---|---:|---|---:|---:|---:|---:|---:|---|
| 0 | DEFAULT_PT_NONE | NONE | 11 | - | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 1 | DEFAULT_PT_DUST | DUST | 8 | PART | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 2 | DEFAULT_PT_WATR | WATR | 7 | LIQUID,CONDUCTS,PHOTPASS,NEUTPASS,LIFE_DEC | 1 | 1 | Y |  |  | Implemented |
| 3 | DEFAULT_PT_OIL | OIL | 7 | LIQUID,NEUTPASS | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 4 | DEFAULT_PT_FIRE | FIRE | 5 | GAS,LIFE_DEC,LIFE_KILL | 1 | 1 | Y | Y | Y | Implemented |
| 5 | DEFAULT_PT_STNE | STNE | 8 | PART | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 6 | DEFAULT_PT_LAVA | LAVA | 7 | LIQUID,LIFE_DEC | 1 | 1 |  | Y | Y | Internal (no reference Update callback; generic movement/transition) |
| 7 | DEFAULT_PT_GUNP | GUN | 5 | PART | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 8 | DEFAULT_PT_NITR | NITR | 5 | LIQUID | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 9 | DEFAULT_PT_CLNE | CLNE | 11 | SOLID,PHOTPASS,NOCTYPEDRAW | 1 | 1 | Y |  |  | Implemented |
| 10 | DEFAULT_PT_GAS | GAS | 6 | GAS,NEUTPASS | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 11 | DEFAULT_PT_PLEX | C-4 | 5 | SOLID,NEUTPENETRATE | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 12 | DEFAULT_PT_GOO | GOO | 9 | SOLID,NEUTPENETRATE,LIFE_DEC,LIFE_KILL_DEC | 1 | 1 | Y |  |  | Implemented |
| 13 | DEFAULT_PT_ICEI | ICE | 9 | SOLID,NEUTPASS,LIFE_DEC | 1 | 1 | Y |  |  | Implemented |
| 14 | DEFAULT_PT_METL | METL | 1 | SOLID,CONDUCTS,HOT_GLOW,LIFE_DEC | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 15 | DEFAULT_PT_SPRK | SPRK | 1 | SOLID,LIFE_DEC | 1 | 1 | Y | Y |  | Implemented |
| 16 | DEFAULT_PT_SNOW | SNOW | 8 | PART,NEUTPASS | 1 | 1 | Y |  |  | Implemented |
| 17 | DEFAULT_PT_WOOD | WOOD | 9 | SOLID,NEUTPENETRATE | 1 | 1 | Y | Y |  | Implemented |
| 18 | DEFAULT_PT_NEUT | NEUT | 10 | ENERGY,LIFE_DEC,LIFE_KILL_DEC | 1 | 1 | Y | Y | Y | Implemented |
| 19 | DEFAULT_PT_PLUT | PLUT | 10 | PART,NEUTPASS,RADIOACTIVE | 1 | 1 | Y |  |  | Implemented |
| 20 | DEFAULT_PT_PLNT | PLNT | 9 | SOLID,NEUTPENETRATE,LIFE_DEC | 1 | 1 | Y | Y |  | Implemented |
| 21 | DEFAULT_PT_ACID | ACID | 7 | LIQUID,DEADLY | 1 | 1 | Y | Y |  | Implemented |
| 22 | DEFAULT_PT_VOID | VOID | 11 | SOLID | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 23 | DEFAULT_PT_WTRV | WTRV | 6 | GAS | 1 | 1 | Y |  |  | Implemented |
| 24 | DEFAULT_PT_CNCT | CNCT | 8 | PART,HOT_GLOW | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 25 | DEFAULT_PT_DSTW | DSTW | 7 | LIQUID,PHOTPASS,NEUTPASS | 1 | 1 | Y |  |  | Implemented |
| 26 | DEFAULT_PT_SALT | SALT | 8 | PART | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 27 | DEFAULT_PT_SLTW | SLTW | 7 | LIQUID,CONDUCTS,PHOTPASS,NEUTPENETRATE,LIFE_DEC | 1 | 1 | Y |  |  | Implemented |
| 28 | DEFAULT_PT_DMND | DMND | 11 | SOLID | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 29 | DEFAULT_PT_BMTL | BMTL | 9 | SOLID,CONDUCTS,HOT_GLOW,LIFE_DEC | 1 | 1 | Y |  |  | Implemented |
| 30 | DEFAULT_PT_BRMT | BRMT | 8 | PART,CONDUCTS,HOT_GLOW,LIFE_DEC | 1 | 1 | Y |  |  | Implemented |
| 31 | DEFAULT_PT_PHOT | PHOT | 10 | ENERGY,PHOTPASS,LIFE_DEC,LIFE_KILL_DEC | 1 | 1 | Y | Y | Y | Implemented |
| 32 | DEFAULT_PT_URAN | URAN | 10 | PART,RADIOACTIVE | 1 | 1 | Y |  |  | Implemented |
| 33 | DEFAULT_PT_WAX | WAX | 9 | SOLID | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 34 | DEFAULT_PT_MWAX | MWAX | 7 | LIQUID | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 35 | DEFAULT_PT_PSCN | PSCN | 1 | SOLID,CONDUCTS,LIFE_DEC | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 36 | DEFAULT_PT_NSCN | NSCN | 1 | SOLID,CONDUCTS,LIFE_DEC | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 37 | DEFAULT_PT_LNTG | LN2 | 7 | LIQUID | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 38 | DEFAULT_PT_INSL | INSL | 1 | SOLID | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 39 | DEFAULT_PT_BHOL | VACU | 11 | SOLID | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 40 | DEFAULT_PT_WHOL | VENT | 11 | SOLID | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 41 | DEFAULT_PT_RBDM | RBDM | 5 | SOLID,CONDUCTS,LIFE_DEC | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 42 | DEFAULT_PT_LRBD | LRBD | 5 | LIQUID,CONDUCTS,LIFE_DEC | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 43 | DEFAULT_PT_NTCT | NTCT | 1 | SOLID,CONDUCTS,LIFE_DEC | 1 | 1 | Y |  |  | Implemented |
| 44 | DEFAULT_PT_SAND | SAND | 8 | PART | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 45 | DEFAULT_PT_GLAS | GLAS | 9 | SOLID,PHOTPASS,NEUTPASS,HOT_GLOW,SPARKSETTLE | 1 | 1 | Y |  | Y | Implemented |
| 46 | DEFAULT_PT_PTCT | PTCT | 1 | SOLID,CONDUCTS,LIFE_DEC | 1 | 1 | Y |  |  | Implemented |
| 47 | DEFAULT_PT_BGLA | BGLA | 8 | PART,PHOTPASS,NEUTPASS,DEADLY,HOT_GLOW | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 48 | DEFAULT_PT_THDR | THDR | 5 | PART | 1 | 1 | Y | Y |  | Implemented |
| 49 | DEFAULT_PT_PLSM | PLSM | 6 | GAS,LIFE_DEC,LIFE_KILL | 1 | 1 |  | Y | Y | Internal (no reference Update callback; generic movement/transition) |
| 50 | DEFAULT_PT_ETRD | ETRD | 1 | SOLID,CONDUCTS,LIFE_DEC | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 51 | DEFAULT_PT_NICE | NICE | 9 | SOLID | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 52 | DEFAULT_PT_NBLE | NBLE | 6 | GAS,CONDUCTS,LIFE_DEC | 1 | 1 | Y |  |  | Implemented |
| 53 | DEFAULT_PT_BTRY | BTRY | 1 | SOLID | 1 | 1 | Y |  |  | Implemented |
| 54 | DEFAULT_PT_LCRY | LCRY | 2 | SOLID | 1 | 1 | Y | Y |  | Implemented |
| 55 | DEFAULT_PT_STKM | STKM | 11 | NOCTYPEDRAW | 1 | 1 | Y | Y | Y | Blocked (shared locomotion implemented; reference legs/element-power reactions pending Agent 3) |
| 56 | DEFAULT_PT_SWCH | SWCH | 1 | SOLID | 1 | 1 | Y | Y |  | Implemented |
| 57 | DEFAULT_PT_SMKE | SMKE | 6 | GAS,LIFE_DEC,LIFE_KILL_DEC | 1 | 1 |  | Y |  | Internal (no reference Update callback; generic movement/transition) |
| 58 | DEFAULT_PT_DESL | DESL | 7 | LIQUID | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 59 | DEFAULT_PT_COAL | COAL | 9 | SOLID | 1 | 1 | Y | Y |  | Implemented |
| 60 | DEFAULT_PT_LO2 | LOXY | 7 | LIQUID | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 61 | DEFAULT_PT_O2 | OXYG | 6 | GAS | 1 | 1 | Y |  |  | Implemented |
| 62 | DEFAULT_PT_INWR | INWR | 1 | SOLID,CONDUCTS,LIFE_DEC | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 63 | DEFAULT_PT_YEST | YEST | 8 | PART | 1 | 1 | Y |  |  | Implemented |
| 64 | DEFAULT_PT_DYST | DYST | 8 | PART | 1 | 0 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 65 | DEFAULT_PT_THRM | THRM | 5 | PART | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 66 | DEFAULT_PT_GLOW | GLOW | 7 | LIQUID,PHOTPASS,LIFE_DEC | 1 | 1 | Y | Y |  | Implemented |
| 67 | DEFAULT_PT_BRCK | BRCK | 9 | SOLID,HOT_GLOW | 1 | 1 |  | Y |  | Internal (no reference Update callback; generic movement/transition) |
| 68 | DEFAULT_PT_HFLM | CFLM | 5 | GAS,LIFE_DEC,LIFE_KILL | 1 | 1 |  | Y | Y | Internal (no reference Update callback; generic movement/transition) |
| 69 | DEFAULT_PT_FIRW | FIRW | 5 | PART,LIFE_DEC | 1 | 1 | Y | Y |  | Implemented |
| 70 | DEFAULT_PT_FUSE | FUSE | 5 | SOLID | 1 | 1 | Y |  |  | Implemented |
| 71 | DEFAULT_PT_FSEP | FSEP | 5 | PART | 1 | 1 | Y |  |  | Implemented |
| 72 | DEFAULT_PT_AMTR | AMTR | 10 | GAS | 1 | 1 | Y | Y |  | Implemented |
| 73 | DEFAULT_PT_BCOL | BCOL | 8 | PART | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 74 | DEFAULT_PT_PCLN | PCLN | 2 | SOLID,PHOTPASS,NOCTYPEDRAW | 1 | 1 | Y | Y |  | Implemented |
| 75 | DEFAULT_PT_HSWC | HSWC | 2 | SOLID | 1 | 1 | Y | Y |  | Implemented |
| 76 | DEFAULT_PT_IRON | IRON | 9 | SOLID,CONDUCTS,HOT_GLOW,LIFE_DEC | 1 | 1 | Y |  |  | Implemented |
| 77 | DEFAULT_PT_MORT | MORT | 11 | GAS | 1 | 0 | Y |  |  | Implemented |
| 78 | DEFAULT_PT_LIFE | LIFE | 12 | SOLID,LIFE | 1 | 0 |  | Y | Y | Internal (no reference Update callback; generic movement/transition) |
| 79 | DEFAULT_PT_DLAY | DLAY | 2 | SOLID | 1 | 1 | Y | Y |  | Implemented |
| 80 | DEFAULT_PT_CO2 | CO2 | 6 | GAS | 1 | 1 | Y |  |  | Implemented |
| 81 | DEFAULT_PT_DRIC | DRIC | 9 | SOLID | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 82 | DEFAULT_PT_CBNW | BUBW | 7 | LIQUID,CONDUCTS,NEUTPENETRATE,LIFE_DEC | 1 | 1 | Y | Y |  | Implemented |
| 83 | DEFAULT_PT_STOR | STOR | 2 | SOLID,NOCTYPEDRAW | 1 | 1 | Y | Y |  | Implemented |
| 84 | DEFAULT_PT_PVOD | PVOD | 2 | SOLID | 1 | 1 | Y | Y |  | Implemented |
| 85 | DEFAULT_PT_CONV | CONV | 11 | SOLID,NOCTYPEDRAW | 1 | 1 | Y |  |  | Implemented |
| 86 | DEFAULT_PT_CAUS | CAUS | 6 | GAS,DEADLY | 1 | 1 | Y |  |  | Implemented |
| 87 | DEFAULT_PT_LIGH | LIGH | 5 | SOLID | 1 | 1 | Y | Y | Y | Implemented |
| 88 | DEFAULT_PT_TESC | TESC | 1 | SOLID,CONDUCTS,HOT_GLOW,LIFE_DEC | 1 | 1 |  |  | Y | Internal (no reference Update callback; generic movement/transition) |
| 89 | DEFAULT_PT_DEST | DEST | 5 | PART,LIFE_DEC,LIFE_KILL_DEC | 1 | 1 | Y | Y |  | Implemented |
| 90 | DEFAULT_PT_SPNG | SPNG | 9 | SOLID | 1 | 1 | Y | Y |  | Implemented |
| 91 | DEFAULT_PT_RIME | RIME | 9 | SOLID | 1 | 1 | Y |  |  | Implemented |
| 92 | DEFAULT_PT_FOG | FOG | 6 | GAS,LIFE_DEC | 1 | 1 | Y |  |  | Implemented |
| 93 | DEFAULT_PT_BCLN | BCLN | 11 | SOLID,PHOTPASS,LIFE_DEC,LIFE_KILL_DEC,NOCTYPEDRAW | 1 | 1 | Y |  |  | Implemented |
| 94 | DEFAULT_PT_LOVE | LOVE | 11 | SOLID | 1 | 0 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 95 | DEFAULT_PT_DEUT | DEUT | 10 | LIQUID,NEUTPASS | 1 | 1 | Y | Y |  | Implemented |
| 96 | DEFAULT_PT_WARP | WARP | 10 | GAS,LIFE_DEC,LIFE_KILL | 1 | 1 | Y | Y | Y | Implemented |
| 97 | DEFAULT_PT_PUMP | PUMP | 2 | SOLID | 1 | 1 | Y | Y |  | Implemented |
| 98 | DEFAULT_PT_FWRK | FWRK | 5 | PART,LIFE_DEC | 1 | 1 | Y |  |  | Implemented |
| 99 | DEFAULT_PT_PIPE | PIPE | 4 | SOLID,LIFE_DEC | 1 | 1 | Y | Y |  | Blocked (full payload flow across pipe segments pending Agent 3) |
| 100 | DEFAULT_PT_FRZZ | FRZZ | 8 | PART | 1 | 1 | Y |  |  | Implemented |
| 101 | DEFAULT_PT_FRZW | FRZW | 7 | LIQUID,LIFE_DEC | 1 | 0 | Y |  |  | Implemented |
| 102 | DEFAULT_PT_GRAV | GRAV | 8 | PART,LIFE_DEC | 1 | 1 | Y | Y |  | Implemented |
| 103 | DEFAULT_PT_BIZR | BIZR | 7 | LIQUID | 1 | 1 | Y | Y |  | Implemented |
| 104 | DEFAULT_PT_BIZRG | BIZG | 6 | GAS | 1 | 0 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 105 | DEFAULT_PT_BIZRS | BIZS | 9 | SOLID | 1 | 0 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 106 | DEFAULT_PT_INST | INST | 1 | SOLID,LIFE_DEC | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 107 | DEFAULT_PT_ISOZ | ISOZ | 10 | LIQUID,PHOTPASS,NEUTPENETRATE | 1 | 1 | Y |  |  | Implemented |
| 108 | DEFAULT_PT_ISZS | ISZS | 10 | SOLID,PHOTPASS | 1 | 1 | Y |  |  | Implemented |
| 109 | DEFAULT_PT_PRTI | PRTI | 11 | SOLID | 1 | 1 | Y | Y |  | Implemented |
| 110 | DEFAULT_PT_PRTO | PRTO | 11 | SOLID | 1 | 1 | Y | Y |  | Implemented |
| 111 | DEFAULT_PT_PSTE | PSTE | 7 | LIQUID | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 112 | DEFAULT_PT_PSTS | PSTS | 9 | SOLID | 1 | 0 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 113 | DEFAULT_PT_ANAR | ANAR | 8 | PART | 1 | 1 | Y |  |  | Implemented |
| 114 | DEFAULT_PT_VINE | VINE | 9 | SOLID | 1 | 1 | Y | Y |  | Implemented |
| 115 | DEFAULT_PT_INVIS | INVS | 3 | SOLID,PHOTPASS,NEUTPASS | 1 | 1 | Y | Y |  | Implemented |
| 116 | DEFAULT_PT_116 | EQVE | 11 | PART | 1 | 0 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 117 | DEFAULT_PT_SPAWN2 | SPWN2 | 9 | SOLID | 1 | 0 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 118 | DEFAULT_PT_SPAWN | SPWN | 9 | SOLID | 1 | 0 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 119 | DEFAULT_PT_SHLD1 | SHLD | 9 | SOLID,LIFE_DEC | 1 | 1 | Y |  |  | Implemented |
| 120 | DEFAULT_PT_SHLD2 | SHD2 | 9 | SOLID,LIFE_DEC | 1 | 0 | Y |  |  | Implemented |
| 121 | DEFAULT_PT_SHLD3 | SHD3 | 9 | SOLID,LIFE_DEC | 1 | 0 | Y |  |  | Implemented |
| 122 | DEFAULT_PT_SHLD4 | SHD4 | 9 | SOLID,LIFE_DEC | 1 | 0 | Y |  |  | Implemented |
| 123 | DEFAULT_PT_LOLZ | LOLZ | 11 | SOLID | 1 | 0 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 124 | DEFAULT_PT_WIFI | WIFI | 1 | SOLID | 1 | 1 | Y | Y |  | Implemented |
| 125 | DEFAULT_PT_FILT | FILT | 9 | SOLID,PHOTPASS,LIFE_DEC,NOAMBHEAT | 1 | 1 |  | Y | Y | Internal (no reference Update callback; generic movement/transition) |
| 126 | DEFAULT_PT_ARAY | ARAY | 1 | SOLID | 1 | 1 | Y |  |  | Implemented |
| 127 | DEFAULT_PT_BRAY | BRAY | 1 | SOLID,LIFE_DEC,LIFE_KILL | 1 | 0 |  | Y |  | Internal (no reference Update callback; generic movement/transition) |
| 128 | DEFAULT_PT_STKM2 | STK2 | 11 | NOCTYPEDRAW | 1 | 1 | Y |  | Y | Blocked (shared locomotion implemented; reference legs/element-power reactions pending Agent 3) |
| 129 | DEFAULT_PT_BOMB | BOMB | 5 | PART,SPARKSETTLE | 1 | 1 | Y | Y |  | Implemented |
| 130 | DEFAULT_PT_C5 | C-5 | 5 | SOLID,PHOTPASS,NEUTPENETRATE,LIFE_DEC | 1 | 1 | Y | Y |  | Implemented |
| 131 | DEFAULT_PT_SING | SING | 10 | PART,LIFE_DEC | 1 | 1 | Y |  | Y | Implemented |
| 132 | DEFAULT_PT_QRTZ | QRTZ | 9 | SOLID,PHOTPASS,HOT_GLOW,LIFE_DEC | 1 | 1 | Y | Y | Y | Implemented |
| 133 | DEFAULT_PT_PQRT | PQRT | 8 | PART,PHOTPASS,HOT_GLOW | 1 | 1 |  |  | Y | Internal (no reference Update callback; generic movement/transition) |
| 134 | DEFAULT_PT_EMP | EMP | 1 | SOLID,LIFE_DEC | 1 | 1 |  | Y |  | Internal (no reference Update callback; generic movement/transition) |
| 135 | DEFAULT_PT_BREC | BREL | 8 | PART,CONDUCTS,HOT_GLOW,LIFE_DEC | 1 | 1 | Y |  |  | Implemented |
| 136 | DEFAULT_PT_ELEC | ELEC | 10 | ENERGY,LIFE_DEC,LIFE_KILL_DEC | 1 | 1 | Y | Y | Y | Implemented |
| 137 | DEFAULT_PT_ACEL | ACEL | 4 | SOLID | 1 | 1 | Y | Y |  | Implemented |
| 138 | DEFAULT_PT_DCEL | DCEL | 4 | SOLID | 1 | 1 | Y | Y |  | Implemented |
| 139 | DEFAULT_PT_BANG | TNT | 5 | SOLID,NEUTPENETRATE | 1 | 1 | Y |  |  | Implemented |
| 140 | DEFAULT_PT_IGNT | IGNC | 5 | SOLID,NEUTPENETRATE,LIFE_KILL,SPARKSETTLE | 1 | 1 | Y |  |  | Implemented |
| 141 | DEFAULT_PT_BOYL | BOYL | 6 | GAS | 1 | 1 | Y |  |  | Implemented |
| 142 | DEFAULT_PT_GEL | GEL | 7 | LIQUID,NEUTPENETRATE,LIFE_DEC | 1 | 1 | Y | Y |  | Implemented |
| 143 | DEFAULT_PT_TRON | TRON | 11 | SOLID,LIFE_DEC,LIFE_KILL | 1 | 1 | Y | Y | Y | Implemented |
| 144 | DEFAULT_PT_TTAN | TTAN | 9 | SOLID,CONDUCTS,HOT_GLOW,LIFE_DEC | 1 | 1 | Y |  |  | Implemented |
| 145 | DEFAULT_PT_EXOT | EXOT | 10 | LIQUID,NEUTPASS | 1 | 1 | Y | Y |  | Implemented |
| 147 | DEFAULT_PT_EMBR | EMBR | 5 | PART,LIFE_DEC,LIFE_KILL,SPARKSETTLE | 1 | 0 | Y | Y |  | Implemented |
| 148 | DEFAULT_PT_H2 | HYGN | 6 | GAS,PHOTPASS | 1 | 1 | Y |  |  | Implemented |
| 149 | DEFAULT_PT_SOAP | SOAP | 7 | LIQUID,NEUTPENETRATE,LIFE_DEC | 1 | 1 | Y | Y |  | Blocked (links implemented; full bubble pop/detach semantics pending Agent 3) |
| 150 | DEFAULT_PT_NBHL | BHOL | 11 | SOLID | 1 | 1 | Y |  |  | Implemented |
| 151 | DEFAULT_PT_NWHL | WHOL | 11 | SOLID | 1 | 1 | Y |  |  | Implemented |
| 152 | DEFAULT_PT_MERC | MERC | 7 | LIQUID,CONDUCTS,NEUTABSORB,LIFE_DEC | 1 | 1 | Y |  |  | Implemented |
| 153 | DEFAULT_PT_PBCN | PBCN | 2 | SOLID,PHOTPASS,NOCTYPEDRAW | 1 | 1 | Y | Y |  | Implemented |
| 154 | DEFAULT_PT_GPMP | GPMP | 2 | SOLID | 1 | 1 | Y | Y |  | Implemented |
| 155 | DEFAULT_PT_CLST | CLST | 8 | PART | 1 | 1 | Y | Y | Y | Implemented |
| 156 | DEFAULT_PT_WIRE | WWLD | 1 | SOLID | 1 | 1 | Y | Y |  | Implemented |
| 157 | DEFAULT_PT_GBMB | GBMB | 4 | PART,LIFE_DEC,LIFE_KILL_DEC | 1 | 1 | Y | Y |  | Implemented |
| 158 | DEFAULT_PT_FIGH | FIGH | 11 | NOCTYPEDRAW | 1 | 1 | Y |  |  | Blocked (shared locomotion implemented; fighter combat/AI pending Agent 3) |
| 159 | DEFAULT_PT_FRAY | FRAY | 4 | SOLID | 1 | 1 | Y |  |  | Implemented |
| 160 | DEFAULT_PT_RPEL | RPEL | 4 | SOLID | 1 | 1 | Y |  |  | Implemented |
| 161 | DEFAULT_PT_PPIP | PPIP | 2 | SOLID,LIFE_DEC | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 162 | DEFAULT_PT_DTEC | DTEC | 3 | SOLID | 1 | 1 | Y |  |  | Implemented |
| 163 | DEFAULT_PT_DMG | DMG | 4 | PART,SPARKSETTLE | 1 | 1 | Y | Y |  | Implemented |
| 164 | DEFAULT_PT_TSNS | TSNS | 3 | SOLID | 1 | 1 | Y |  |  | Implemented |
| 165 | DEFAULT_PT_VIBR | VIBR | 10 | SOLID,LIFE_DEC | 1 | 1 | Y | Y |  | Implemented |
| 166 | DEFAULT_PT_BVBR | BVBR | 10 | PART,LIFE_DEC | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 167 | DEFAULT_PT_CRAY | CRAY | 1 | SOLID | 1 | 1 | Y |  |  | Implemented |
| 168 | DEFAULT_PT_PSTN | PSTN | 4 | SOLID | 1 | 1 | Y | Y |  | Blocked (uses shared MoveStack; FRME recursion and tmp3 flags pending Agent 3) |
| 169 | DEFAULT_PT_FRME | FRME | 4 | SOLID | 1 | 1 |  | Y |  | Internal (no reference Update callback; generic movement/transition) |
| 170 | DEFAULT_PT_GOLD | GOLD | 9 | SOLID,CONDUCTS,NEUTPASS,HOT_GLOW,LIFE_DEC | 1 | 1 | Y | Y |  | Implemented |
| 171 | DEFAULT_PT_TUNG | TUNG | 1 | SOLID,CONDUCTS,LIFE_DEC | 1 | 1 | Y | Y | Y | Implemented |
| 172 | DEFAULT_PT_PSNS | PSNS | 3 | SOLID | 1 | 1 | Y |  |  | Implemented |
| 173 | DEFAULT_PT_PROT | PROT | 10 | ENERGY | 1 | 1 | Y | Y | Y | Implemented |
| 174 | DEFAULT_PT_VIRS | VIRS | 7 | LIQUID,DEADLY | 1 | 1 | Y | Y |  | Implemented |
| 175 | DEFAULT_PT_VRSS | VRSS | 9 | SOLID,DEADLY | 1 | 0 |  | Y |  | Internal (no reference Update callback; generic movement/transition) |
| 176 | DEFAULT_PT_VRSG | VRSG | 6 | GAS,DEADLY | 1 | 0 |  | Y |  | Internal (no reference Update callback; generic movement/transition) |
| 177 | DEFAULT_PT_GRVT | GRVT | 10 | ENERGY,LIFE_DEC,LIFE_KILL_DEC | 1 | 1 | Y | Y | Y | Implemented |
| 178 | DEFAULT_PT_DRAY | DRAY | 1 | SOLID | 1 | 1 | Y |  |  | Implemented |
| 179 | DEFAULT_PT_CRMC | CRMC | 9 | SOLID,NEUTPASS | 1 | 1 | Y | Y | Y | Implemented |
| 180 | DEFAULT_PT_HEAC | HEAC | 9 | SOLID | 1 | 1 | Y |  |  | Implemented |
| 181 | DEFAULT_PT_SAWD | SAWD | 8 | PART,NEUTPASS | 1 | 1 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 182 | DEFAULT_PT_POLO | POLO | 10 | PART,NEUTPASS,DEADLY,RADIOACTIVE,LIFE_DEC | 1 | 1 | Y | Y |  | Implemented |
| 183 | DEFAULT_PT_RFRG | RFRG | 6 | GAS,DEADLY | 1 | 1 | Y |  |  | Implemented |
| 184 | DEFAULT_PT_RFGL | RFGL | 7 | LIQUID,DEADLY | 1 | 0 |  |  |  | Internal (no reference Update callback; generic movement/transition) |
| 185 | DEFAULT_PT_LSNS | LSNS | 3 | SOLID | 1 | 1 | Y |  |  | Implemented |
| 186 | DEFAULT_PT_LDTC | LDTC | 3 | SOLID,NOCTYPEDRAW | 1 | 1 | Y |  |  | Implemented |
| 187 | DEFAULT_PT_SLCN | SLCN | 8 | PART,CONDUCTS,HOT_GLOW,LIFE_DEC | 1 | 1 | Y | Y | Y | Implemented |
| 188 | DEFAULT_PT_PTNM | PTNM | 9 | SOLID,CONDUCTS,HOT_GLOW,LIFE_DEC,SPARKSETTLE | 1 | 1 | Y | Y | Y | Implemented |
| 189 | DEFAULT_PT_VSNS | VSNS | 3 | SOLID | 1 | 1 | Y |  |  | Implemented |
| 190 | DEFAULT_PT_ROCK | ROCK | 9 | SOLID,HOT_GLOW | 1 | 1 |  | Y | Y | Internal (no reference Update callback; generic movement/transition) |
| 191 | DEFAULT_PT_LITH | LITH | 5 | PART,LIFE_DEC | 1 | 1 | Y | Y |  | Implemented |
| 192 | DEFAULT_PT_RSST | RSST | 7 | LIQUID,CONDUCTS,PHOTPASS,NEUTPASS,LIFE_DEC | 1 | 1 | Y |  |  | Implemented |
| 193 | DEFAULT_PT_RSSS | RSSS | 9 | SOLID,NEUTPASS | 1 | 1 | Y |  |  | Implemented |
| 194 | DEFAULT_PT_BASE | BASE | 7 | LIQUID,DEADLY | 1 | 1 | Y | Y |  | Implemented |
| 195 | DEFAULT_PT_SEED | SEED | 8 | PART,NEUTPASS | 1 | 1 | Y | Y | Y | Implemented |
