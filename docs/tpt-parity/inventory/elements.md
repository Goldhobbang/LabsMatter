# Element inventory (generated from pinned TPT reference)

Reference: The-Powder-Toy/The-Powder-Toy @ 2b7221a46d378589b4de7683f16d3ebc92be6bd9

Total slots: 196 (live elements: 195, disabled slots: 1)
Enabled: 195, menu-visible: 175, selectable: 175

## Implementation status (source-derived classification)

| Status | Count | Meaning |
|---|---:|---|
| Full | 126 | reference `Update` + dedicated Rust handler |
| Partial | 6 | handler exists, blocked on an engine subsystem |
| Unimplemented | 0 | reference `Update` but no Rust handler |
| Generic | 48 | no `Update`; create/graphics/transition behaviour provided by the engine |
| Passive | 15 | no `Update`; generic movement only |
| Disabled | 1 | disabled reference slot |

`Full`/`Partial` count Rust **handlers**, not verified behaviours.
Independent gameplay verification is `no` for every element until
Agent B signs off (see `docs/tpt-parity/qa/`).

| id | abbr | name | section | state flags | enabled | menu | update | create | graphics | transition | status |
|---:|---|---|---:|---|---:|---:|---:|---:|---:|---:|---|
| 0 | DEFAULT_PT_NONE | NONE | 11 | - | 1 | 1 |  |  |  |  | Passive |
| 1 | DEFAULT_PT_DUST | DUST | 8 | PART | 1 | 1 |  |  |  |  | Passive |
| 2 | DEFAULT_PT_WATR | WATR | 7 | LIQUID,CONDUCTS,PHOTPASS,NEUTPASS,LIFE_DEC | 1 | 1 | Y |  |  | Y | Full |
| 3 | DEFAULT_PT_OIL | OIL | 7 | LIQUID,NEUTPASS | 1 | 1 |  |  |  | Y | Generic |
| 4 | DEFAULT_PT_FIRE | FIRE | 5 | GAS,LIFE_DEC,LIFE_KILL | 1 | 1 | Y | Y | Y | Y | Full |
| 5 | DEFAULT_PT_STNE | STNE | 8 | PART | 1 | 1 |  |  |  | Y | Generic |
| 6 | DEFAULT_PT_LAVA | LAVA | 7 | LIQUID,LIFE_DEC | 1 | 1 |  | Y | Y | Y | Generic |
| 7 | DEFAULT_PT_GUNP | GUN | 5 | PART | 1 | 1 |  |  |  | Y | Generic |
| 8 | DEFAULT_PT_NITR | NITR | 5 | LIQUID | 1 | 1 |  |  |  | Y | Generic |
| 9 | DEFAULT_PT_CLNE | CLNE | 11 | SOLID,PHOTPASS,NOCTYPEDRAW | 1 | 1 | Y |  |  |  | Full |
| 10 | DEFAULT_PT_GAS | GAS | 6 | GAS,NEUTPASS | 1 | 1 |  |  |  | Y | Generic |
| 11 | DEFAULT_PT_PLEX | C-4 | 5 | SOLID,NEUTPENETRATE | 1 | 1 |  |  |  | Y | Generic |
| 12 | DEFAULT_PT_GOO | GOO | 9 | SOLID,NEUTPENETRATE,LIFE_DEC,LIFE_KILL_DEC | 1 | 1 | Y |  |  |  | Full |
| 13 | DEFAULT_PT_ICEI | ICE | 9 | SOLID,NEUTPASS,LIFE_DEC | 1 | 1 | Y |  |  | Y | Full |
| 14 | DEFAULT_PT_METL | METL | 1 | SOLID,CONDUCTS,HOT_GLOW,LIFE_DEC | 1 | 1 |  |  |  | Y | Generic |
| 15 | DEFAULT_PT_SPRK | SPRK | 1 | SOLID,LIFE_DEC | 1 | 1 | Y |  | Y |  | Full |
| 16 | DEFAULT_PT_SNOW | SNOW | 8 | PART,NEUTPASS | 1 | 1 | Y |  |  | Y | Full |
| 17 | DEFAULT_PT_WOOD | WOOD | 9 | SOLID,NEUTPENETRATE | 1 | 1 | Y |  | Y | Y | Full |
| 18 | DEFAULT_PT_NEUT | NEUT | 10 | ENERGY,LIFE_DEC,LIFE_KILL_DEC | 1 | 1 | Y | Y | Y |  | Full |
| 19 | DEFAULT_PT_PLUT | PLUT | 10 | PART,NEUTPASS,RADIOACTIVE | 1 | 1 | Y |  |  |  | Full |
| 20 | DEFAULT_PT_PLNT | PLNT | 9 | SOLID,NEUTPENETRATE,LIFE_DEC | 1 | 1 | Y |  | Y | Y | Full |
| 21 | DEFAULT_PT_ACID | ACID | 7 | LIQUID,DEADLY | 1 | 1 | Y |  | Y |  | Full |
| 22 | DEFAULT_PT_VOID | VOID | 11 | SOLID | 1 | 1 |  |  |  |  | Passive |
| 23 | DEFAULT_PT_WTRV | WTRV | 6 | GAS | 1 | 1 | Y |  |  | Y | Full |
| 24 | DEFAULT_PT_CNCT | CNCT | 8 | PART,HOT_GLOW | 1 | 1 |  |  |  | Y | Generic |
| 25 | DEFAULT_PT_DSTW | DSTW | 7 | LIQUID,PHOTPASS,NEUTPASS | 1 | 1 | Y |  |  | Y | Full |
| 26 | DEFAULT_PT_SALT | SALT | 8 | PART | 1 | 1 |  |  |  | Y | Generic |
| 27 | DEFAULT_PT_SLTW | SLTW | 7 | LIQUID,CONDUCTS,PHOTPASS,NEUTPENETRATE,LIFE_DEC | 1 | 1 | Y |  |  | Y | Full |
| 28 | DEFAULT_PT_DMND | DMND | 11 | SOLID | 1 | 1 |  |  |  |  | Passive |
| 29 | DEFAULT_PT_BMTL | BMTL | 9 | SOLID,CONDUCTS,HOT_GLOW,LIFE_DEC | 1 | 1 | Y |  |  | Y | Full |
| 30 | DEFAULT_PT_BRMT | BRMT | 8 | PART,CONDUCTS,HOT_GLOW,LIFE_DEC | 1 | 1 | Y |  |  | Y | Full |
| 31 | DEFAULT_PT_PHOT | PHOT | 10 | ENERGY,PHOTPASS,LIFE_DEC,LIFE_KILL_DEC | 1 | 1 | Y | Y | Y |  | Full |
| 32 | DEFAULT_PT_URAN | URAN | 10 | PART,RADIOACTIVE | 1 | 1 | Y |  |  |  | Full |
| 33 | DEFAULT_PT_WAX | WAX | 9 | SOLID | 1 | 1 |  |  |  | Y | Generic |
| 34 | DEFAULT_PT_MWAX | MWAX | 7 | LIQUID | 1 | 1 |  |  |  | Y | Generic |
| 35 | DEFAULT_PT_PSCN | PSCN | 1 | SOLID,CONDUCTS,LIFE_DEC | 1 | 1 |  |  |  | Y | Generic |
| 36 | DEFAULT_PT_NSCN | NSCN | 1 | SOLID,CONDUCTS,LIFE_DEC | 1 | 1 |  |  |  | Y | Generic |
| 37 | DEFAULT_PT_LNTG | LN2 | 7 | LIQUID | 1 | 1 |  |  |  | Y | Generic |
| 38 | DEFAULT_PT_INSL | INSL | 1 | SOLID | 1 | 1 |  |  |  |  | Passive |
| 39 | DEFAULT_PT_BHOL | VACU | 11 | SOLID | 1 | 1 |  |  |  |  | Passive |
| 40 | DEFAULT_PT_WHOL | VENT | 11 | SOLID | 1 | 1 |  |  |  |  | Passive |
| 41 | DEFAULT_PT_RBDM | RBDM | 5 | SOLID,CONDUCTS,LIFE_DEC | 1 | 1 |  |  |  | Y | Generic |
| 42 | DEFAULT_PT_LRBD | LRBD | 5 | LIQUID,CONDUCTS,LIFE_DEC | 1 | 1 |  |  |  | Y | Generic |
| 43 | DEFAULT_PT_NTCT | NTCT | 1 | SOLID,CONDUCTS,LIFE_DEC | 1 | 1 | Y |  |  | Y | Full |
| 44 | DEFAULT_PT_SAND | SAND | 8 | PART | 1 | 1 |  |  |  | Y | Generic |
| 45 | DEFAULT_PT_GLAS | GLAS | 9 | SOLID,PHOTPASS,NEUTPASS,HOT_GLOW,SPARKSETTLE | 1 | 1 | Y | Y |  | Y | Full |
| 46 | DEFAULT_PT_PTCT | PTCT | 1 | SOLID,CONDUCTS,LIFE_DEC | 1 | 1 | Y |  |  | Y | Full |
| 47 | DEFAULT_PT_BGLA | BGLA | 8 | PART,PHOTPASS,NEUTPASS,DEADLY,HOT_GLOW | 1 | 1 |  |  |  | Y | Generic |
| 48 | DEFAULT_PT_THDR | THDR | 5 | PART | 1 | 1 | Y |  | Y |  | Full |
| 49 | DEFAULT_PT_PLSM | PLSM | 6 | GAS,LIFE_DEC,LIFE_KILL | 1 | 1 |  | Y | Y |  | Generic |
| 50 | DEFAULT_PT_ETRD | ETRD | 1 | SOLID,CONDUCTS,LIFE_DEC | 1 | 1 |  |  |  |  | Passive |
| 51 | DEFAULT_PT_NICE | NICE | 9 | SOLID | 1 | 1 |  |  |  | Y | Generic |
| 52 | DEFAULT_PT_NBLE | NBLE | 6 | GAS,CONDUCTS,LIFE_DEC | 1 | 1 | Y |  |  |  | Full |
| 53 | DEFAULT_PT_BTRY | BTRY | 1 | SOLID | 1 | 1 | Y |  |  | Y | Full |
| 54 | DEFAULT_PT_LCRY | LCRY | 2 | SOLID | 1 | 1 | Y |  | Y | Y | Full |
| 55 | DEFAULT_PT_STKM | STKM | 11 | NOCTYPEDRAW | 1 | 1 | Y | Y | Y | Y | Partial (shared locomotion implemented; reference legs/element-power reactions pending Agent 3) |
| 56 | DEFAULT_PT_SWCH | SWCH | 1 | SOLID | 1 | 1 | Y |  | Y |  | Full |
| 57 | DEFAULT_PT_SMKE | SMKE | 6 | GAS,LIFE_DEC,LIFE_KILL_DEC | 1 | 1 |  |  | Y | Y | Generic |
| 58 | DEFAULT_PT_DESL | DESL | 7 | LIQUID | 1 | 1 |  |  |  | Y | Generic |
| 59 | DEFAULT_PT_COAL | COAL | 9 | SOLID | 1 | 1 | Y |  | Y |  | Full |
| 60 | DEFAULT_PT_LO2 | LOXY | 7 | LIQUID | 1 | 1 |  |  |  | Y | Generic |
| 61 | DEFAULT_PT_O2 | OXYG | 6 | GAS | 1 | 1 | Y |  |  | Y | Full |
| 62 | DEFAULT_PT_INWR | INWR | 1 | SOLID,CONDUCTS,LIFE_DEC | 1 | 1 |  |  |  | Y | Generic |
| 63 | DEFAULT_PT_YEST | YEST | 8 | PART | 1 | 1 | Y |  |  | Y | Full |
| 64 | DEFAULT_PT_DYST | DYST | 8 | PART | 1 | 0 |  |  |  | Y | Generic |
| 65 | DEFAULT_PT_THRM | THRM | 5 | PART | 1 | 1 |  |  |  |  | Passive |
| 66 | DEFAULT_PT_GLOW | GLOW | 7 | LIQUID,PHOTPASS,LIFE_DEC | 1 | 1 | Y |  | Y |  | Full |
| 67 | DEFAULT_PT_BRCK | BRCK | 9 | SOLID,HOT_GLOW | 1 | 1 |  |  | Y | Y | Generic |
| 68 | DEFAULT_PT_HFLM | CFLM | 5 | GAS,LIFE_DEC,LIFE_KILL | 1 | 1 |  | Y | Y |  | Generic |
| 69 | DEFAULT_PT_FIRW | FIRW | 5 | PART,LIFE_DEC | 1 | 1 | Y |  | Y |  | Full |
| 70 | DEFAULT_PT_FUSE | FUSE | 5 | SOLID | 1 | 1 | Y |  |  |  | Full |
| 71 | DEFAULT_PT_FSEP | FSEP | 5 | PART | 1 | 1 | Y |  |  |  | Full |
| 72 | DEFAULT_PT_AMTR | AMTR | 10 | GAS | 1 | 1 | Y |  | Y |  | Full |
| 73 | DEFAULT_PT_BCOL | BCOL | 8 | PART | 1 | 1 |  |  |  |  | Passive |
| 74 | DEFAULT_PT_PCLN | PCLN | 2 | SOLID,PHOTPASS,NOCTYPEDRAW | 1 | 1 | Y |  | Y |  | Full |
| 75 | DEFAULT_PT_HSWC | HSWC | 2 | SOLID | 1 | 1 | Y |  | Y |  | Full |
| 76 | DEFAULT_PT_IRON | IRON | 9 | SOLID,CONDUCTS,HOT_GLOW,LIFE_DEC | 1 | 1 | Y |  |  | Y | Full |
| 77 | DEFAULT_PT_MORT | MORT | 11 | GAS | 1 | 0 | Y |  |  |  | Full |
| 78 | DEFAULT_PT_LIFE | LIFE | 12 | SOLID,LIFE | 1 | 0 |  | Y | Y |  | Generic |
| 79 | DEFAULT_PT_DLAY | DLAY | 2 | SOLID | 1 | 1 | Y |  | Y |  | Full |
| 80 | DEFAULT_PT_CO2 | CO2 | 6 | GAS | 1 | 1 | Y |  |  | Y | Full |
| 81 | DEFAULT_PT_DRIC | DRIC | 9 | SOLID | 1 | 1 |  |  |  | Y | Generic |
| 82 | DEFAULT_PT_CBNW | BUBW | 7 | LIQUID,CONDUCTS,NEUTPENETRATE,LIFE_DEC | 1 | 1 | Y |  | Y | Y | Full |
| 83 | DEFAULT_PT_STOR | STOR | 2 | SOLID,NOCTYPEDRAW | 1 | 1 | Y |  | Y |  | Full |
| 84 | DEFAULT_PT_PVOD | PVOD | 2 | SOLID | 1 | 1 | Y |  | Y |  | Full |
| 85 | DEFAULT_PT_CONV | CONV | 11 | SOLID,NOCTYPEDRAW | 1 | 1 | Y |  |  |  | Full |
| 86 | DEFAULT_PT_CAUS | CAUS | 6 | GAS,DEADLY | 1 | 1 | Y |  |  |  | Full |
| 87 | DEFAULT_PT_LIGH | LIGH | 5 | SOLID | 1 | 1 | Y | Y | Y |  | Full |
| 88 | DEFAULT_PT_TESC | TESC | 1 | SOLID,CONDUCTS,HOT_GLOW,LIFE_DEC | 1 | 1 |  | Y |  |  | Generic |
| 89 | DEFAULT_PT_DEST | DEST | 5 | PART,LIFE_DEC,LIFE_KILL_DEC | 1 | 1 | Y |  | Y |  | Full |
| 90 | DEFAULT_PT_SPNG | SPNG | 9 | SOLID | 1 | 1 | Y |  | Y | Y | Full |
| 91 | DEFAULT_PT_RIME | RIME | 9 | SOLID | 1 | 1 | Y |  |  | Y | Full |
| 92 | DEFAULT_PT_FOG | FOG | 6 | GAS,LIFE_DEC | 1 | 1 | Y |  |  | Y | Full |
| 93 | DEFAULT_PT_BCLN | BCLN | 11 | SOLID,PHOTPASS,LIFE_DEC,LIFE_KILL_DEC,NOCTYPEDRAW | 1 | 1 | Y |  |  |  | Full |
| 94 | DEFAULT_PT_LOVE | LOVE | 11 | SOLID | 1 | 0 |  |  |  |  | Passive |
| 95 | DEFAULT_PT_DEUT | DEUT | 10 | LIQUID,NEUTPASS | 1 | 1 | Y |  | Y |  | Full |
| 96 | DEFAULT_PT_WARP | WARP | 10 | GAS,LIFE_DEC,LIFE_KILL | 1 | 1 | Y | Y | Y |  | Full |
| 97 | DEFAULT_PT_PUMP | PUMP | 2 | SOLID | 1 | 1 | Y |  | Y |  | Full |
| 98 | DEFAULT_PT_FWRK | FWRK | 5 | PART,LIFE_DEC | 1 | 1 | Y |  |  |  | Full |
| 99 | DEFAULT_PT_PIPE | PIPE | 4 | SOLID,LIFE_DEC | 1 | 1 | Y |  | Y | Y | Partial (full payload flow across pipe segments pending Agent 3) |
| 100 | DEFAULT_PT_FRZZ | FRZZ | 8 | PART | 1 | 1 | Y |  |  | Y | Full |
| 101 | DEFAULT_PT_FRZW | FRZW | 7 | LIQUID,LIFE_DEC | 1 | 0 | Y |  |  | Y | Full |
| 102 | DEFAULT_PT_GRAV | GRAV | 8 | PART,LIFE_DEC | 1 | 1 | Y |  | Y |  | Full |
| 103 | DEFAULT_PT_BIZR | BIZR | 7 | LIQUID | 1 | 1 | Y |  | Y | Y | Full |
| 104 | DEFAULT_PT_BIZRG | BIZG | 6 | GAS | 1 | 0 |  |  |  | Y | Generic |
| 105 | DEFAULT_PT_BIZRS | BIZS | 9 | SOLID | 1 | 0 |  |  |  | Y | Generic |
| 106 | DEFAULT_PT_INST | INST | 1 | SOLID,LIFE_DEC | 1 | 1 |  |  |  |  | Generic |
| 107 | DEFAULT_PT_ISOZ | ISOZ | 10 | LIQUID,PHOTPASS,NEUTPENETRATE | 1 | 1 | Y |  |  | Y | Full |
| 108 | DEFAULT_PT_ISZS | ISZS | 10 | SOLID,PHOTPASS | 1 | 1 | Y |  |  | Y | Full |
| 109 | DEFAULT_PT_PRTI | PRTI | 11 | SOLID | 1 | 1 | Y |  | Y |  | Full |
| 110 | DEFAULT_PT_PRTO | PRTO | 11 | SOLID | 1 | 1 | Y |  | Y |  | Full |
| 111 | DEFAULT_PT_PSTE | PSTE | 7 | LIQUID | 1 | 1 |  |  |  | Y | Generic |
| 112 | DEFAULT_PT_PSTS | PSTS | 9 | SOLID | 1 | 0 |  |  |  | Y | Generic |
| 113 | DEFAULT_PT_ANAR | ANAR | 8 | PART | 1 | 1 | Y |  |  |  | Full |
| 114 | DEFAULT_PT_VINE | VINE | 9 | SOLID | 1 | 1 | Y |  | Y | Y | Full |
| 115 | DEFAULT_PT_INVIS | INVS | 3 | SOLID,PHOTPASS,NEUTPASS | 1 | 1 | Y |  | Y |  | Full |
| 116 | DEFAULT_PT_116 | EQVE | 11 | PART | 1 | 0 |  |  |  |  | Passive |
| 117 | DEFAULT_PT_SPAWN2 | SPWN2 | 9 | SOLID | 1 | 0 |  |  |  |  | Generic |
| 118 | DEFAULT_PT_SPAWN | SPWN | 9 | SOLID | 1 | 0 |  |  |  |  | Generic |
| 119 | DEFAULT_PT_SHLD1 | SHLD | 9 | SOLID,LIFE_DEC | 1 | 1 | Y |  |  | Y | Full |
| 120 | DEFAULT_PT_SHLD2 | SHD2 | 9 | SOLID,LIFE_DEC | 1 | 0 | Y |  |  | Y | Full |
| 121 | DEFAULT_PT_SHLD3 | SHD3 | 9 | SOLID,LIFE_DEC | 1 | 0 | Y |  |  | Y | Full |
| 122 | DEFAULT_PT_SHLD4 | SHD4 | 9 | SOLID,LIFE_DEC | 1 | 0 | Y |  |  | Y | Full |
| 123 | DEFAULT_PT_LOLZ | LOLZ | 11 | SOLID | 1 | 0 |  |  |  |  | Passive |
| 124 | DEFAULT_PT_WIFI | WIFI | 1 | SOLID | 1 | 1 | Y |  | Y | Y | Full |
| 125 | DEFAULT_PT_FILT | FILT | 9 | SOLID,PHOTPASS,LIFE_DEC,NOAMBHEAT | 1 | 1 |  | Y | Y |  | Generic |
| 126 | DEFAULT_PT_ARAY | ARAY | 1 | SOLID | 1 | 1 | Y |  |  |  | Full |
| 127 | DEFAULT_PT_BRAY | BRAY | 1 | SOLID,LIFE_DEC,LIFE_KILL | 1 | 0 |  |  | Y |  | Generic |
| 128 | DEFAULT_PT_STKM2 | STK2 | 11 | NOCTYPEDRAW | 1 | 1 | Y | Y |  | Y | Partial (shared locomotion implemented; reference legs/element-power reactions pending Agent 3) |
| 129 | DEFAULT_PT_BOMB | BOMB | 5 | PART,SPARKSETTLE | 1 | 1 | Y |  | Y |  | Full |
| 130 | DEFAULT_PT_C5 | C-5 | 5 | SOLID,PHOTPASS,NEUTPENETRATE,LIFE_DEC | 1 | 1 | Y |  | Y |  | Full |
| 131 | DEFAULT_PT_SING | SING | 10 | PART,LIFE_DEC | 1 | 1 | Y | Y |  |  | Full |
| 132 | DEFAULT_PT_QRTZ | QRTZ | 9 | SOLID,PHOTPASS,HOT_GLOW,LIFE_DEC | 1 | 1 | Y | Y | Y | Y | Full |
| 133 | DEFAULT_PT_PQRT | PQRT | 8 | PART,PHOTPASS,HOT_GLOW | 1 | 1 |  | Y |  | Y | Generic |
| 134 | DEFAULT_PT_EMP | EMP | 1 | SOLID,LIFE_DEC | 1 | 1 |  |  | Y |  | Generic |
| 135 | DEFAULT_PT_BREC | BREL | 8 | PART,CONDUCTS,HOT_GLOW,LIFE_DEC | 1 | 1 | Y |  |  |  | Full |
| 136 | DEFAULT_PT_ELEC | ELEC | 10 | ENERGY,LIFE_DEC,LIFE_KILL_DEC | 1 | 1 | Y | Y | Y |  | Full |
| 137 | DEFAULT_PT_ACEL | ACEL | 4 | SOLID | 1 | 1 | Y |  | Y |  | Full |
| 138 | DEFAULT_PT_DCEL | DCEL | 4 | SOLID | 1 | 1 | Y |  | Y |  | Full |
| 139 | DEFAULT_PT_BANG | TNT | 5 | SOLID,NEUTPENETRATE | 1 | 1 | Y |  |  |  | Full |
| 140 | DEFAULT_PT_IGNT | IGNC | 5 | SOLID,NEUTPENETRATE,LIFE_KILL,SPARKSETTLE | 1 | 1 | Y |  |  | Y | Full |
| 141 | DEFAULT_PT_BOYL | BOYL | 6 | GAS | 1 | 1 | Y |  |  |  | Full |
| 142 | DEFAULT_PT_GEL | GEL | 7 | LIQUID,NEUTPENETRATE,LIFE_DEC | 1 | 1 | Y |  | Y |  | Full |
| 143 | DEFAULT_PT_TRON | TRON | 11 | SOLID,LIFE_DEC,LIFE_KILL | 1 | 1 | Y | Y | Y |  | Full |
| 144 | DEFAULT_PT_TTAN | TTAN | 9 | SOLID,CONDUCTS,HOT_GLOW,LIFE_DEC | 1 | 1 | Y |  |  | Y | Full |
| 145 | DEFAULT_PT_EXOT | EXOT | 10 | LIQUID,NEUTPASS | 1 | 1 | Y |  | Y |  | Full |
| 147 | DEFAULT_PT_EMBR | EMBR | 5 | PART,LIFE_DEC,LIFE_KILL,SPARKSETTLE | 1 | 0 | Y |  | Y |  | Full |
| 148 | DEFAULT_PT_H2 | HYGN | 6 | GAS,PHOTPASS | 1 | 1 | Y |  |  |  | Full |
| 149 | DEFAULT_PT_SOAP | SOAP | 7 | LIQUID,NEUTPENETRATE,LIFE_DEC | 1 | 1 | Y |  | Y |  | Partial (links implemented; full bubble pop/detach semantics pending Agent 3) |
| 150 | DEFAULT_PT_NBHL | BHOL | 11 | SOLID | 1 | 1 | Y |  |  |  | Full |
| 151 | DEFAULT_PT_NWHL | WHOL | 11 | SOLID | 1 | 1 | Y |  |  |  | Full |
| 152 | DEFAULT_PT_MERC | MERC | 7 | LIQUID,CONDUCTS,NEUTABSORB,LIFE_DEC | 1 | 1 | Y |  |  |  | Full |
| 153 | DEFAULT_PT_PBCN | PBCN | 2 | SOLID,PHOTPASS,NOCTYPEDRAW | 1 | 1 | Y |  | Y |  | Full |
| 154 | DEFAULT_PT_GPMP | GPMP | 2 | SOLID | 1 | 1 | Y |  | Y |  | Full |
| 155 | DEFAULT_PT_CLST | CLST | 8 | PART | 1 | 1 | Y | Y | Y | Y | Full |
| 156 | DEFAULT_PT_WIRE | WWLD | 1 | SOLID | 1 | 1 | Y |  | Y |  | Full |
| 157 | DEFAULT_PT_GBMB | GBMB | 4 | PART,LIFE_DEC,LIFE_KILL_DEC | 1 | 1 | Y |  | Y |  | Full |
| 158 | DEFAULT_PT_FIGH | FIGH | 11 | NOCTYPEDRAW | 1 | 1 | Y |  |  | Y | Partial (shared locomotion implemented; fighter combat/AI pending Agent 3) |
| 159 | DEFAULT_PT_FRAY | FRAY | 4 | SOLID | 1 | 1 | Y |  |  |  | Full |
| 160 | DEFAULT_PT_RPEL | RPEL | 4 | SOLID | 1 | 1 | Y |  |  |  | Full |
| 161 | DEFAULT_PT_PPIP | PPIP | 2 | SOLID,LIFE_DEC | 1 | 1 |  |  |  |  | Generic |
| 162 | DEFAULT_PT_DTEC | DTEC | 3 | SOLID | 1 | 1 | Y |  |  |  | Full |
| 163 | DEFAULT_PT_DMG | DMG | 4 | PART,SPARKSETTLE | 1 | 1 | Y |  | Y |  | Full |
| 164 | DEFAULT_PT_TSNS | TSNS | 3 | SOLID | 1 | 1 | Y |  |  |  | Full |
| 165 | DEFAULT_PT_VIBR | VIBR | 10 | SOLID,LIFE_DEC | 1 | 1 | Y |  | Y |  | Full |
| 166 | DEFAULT_PT_BVBR | BVBR | 10 | PART,LIFE_DEC | 1 | 1 |  |  |  |  | Passive |
| 167 | DEFAULT_PT_CRAY | CRAY | 1 | SOLID | 1 | 1 | Y |  |  |  | Full |
| 168 | DEFAULT_PT_PSTN | PSTN | 4 | SOLID | 1 | 1 | Y |  | Y |  | Partial (uses shared MoveStack; FRME recursion and tmp3 flags pending Agent 3) |
| 169 | DEFAULT_PT_FRME | FRME | 4 | SOLID | 1 | 1 |  |  | Y |  | Generic |
| 170 | DEFAULT_PT_GOLD | GOLD | 9 | SOLID,CONDUCTS,NEUTPASS,HOT_GLOW,LIFE_DEC | 1 | 1 | Y |  | Y | Y | Full |
| 171 | DEFAULT_PT_TUNG | TUNG | 1 | SOLID,CONDUCTS,LIFE_DEC | 1 | 1 | Y | Y | Y |  | Full |
| 172 | DEFAULT_PT_PSNS | PSNS | 3 | SOLID | 1 | 1 | Y |  |  |  | Full |
| 173 | DEFAULT_PT_PROT | PROT | 10 | ENERGY | 1 | 1 | Y | Y | Y |  | Full |
| 174 | DEFAULT_PT_VIRS | VIRS | 7 | LIQUID,DEADLY | 1 | 1 | Y |  | Y | Y | Full |
| 175 | DEFAULT_PT_VRSS | VRSS | 9 | SOLID,DEADLY | 1 | 0 |  |  | Y | Y | Generic |
| 176 | DEFAULT_PT_VRSG | VRSG | 6 | GAS,DEADLY | 1 | 0 |  |  | Y | Y | Generic |
| 177 | DEFAULT_PT_GRVT | GRVT | 10 | ENERGY,LIFE_DEC,LIFE_KILL_DEC | 1 | 1 | Y | Y | Y |  | Full |
| 178 | DEFAULT_PT_DRAY | DRAY | 1 | SOLID | 1 | 1 | Y |  |  |  | Full |
| 179 | DEFAULT_PT_CRMC | CRMC | 9 | SOLID,NEUTPASS | 1 | 1 | Y | Y | Y | Y | Full |
| 180 | DEFAULT_PT_HEAC | HEAC | 9 | SOLID | 1 | 1 | Y |  |  |  | Full |
| 181 | DEFAULT_PT_SAWD | SAWD | 8 | PART,NEUTPASS | 1 | 1 |  |  |  |  | Passive |
| 182 | DEFAULT_PT_POLO | POLO | 10 | PART,NEUTPASS,DEADLY,RADIOACTIVE,LIFE_DEC | 1 | 1 | Y |  | Y | Y | Full |
| 183 | DEFAULT_PT_RFRG | RFRG | 6 | GAS,DEADLY | 1 | 1 | Y |  |  | Y | Full |
| 184 | DEFAULT_PT_RFGL | RFGL | 7 | LIQUID,DEADLY | 1 | 0 |  |  |  | Y | Generic |
| 185 | DEFAULT_PT_LSNS | LSNS | 3 | SOLID | 1 | 1 | Y |  |  |  | Full |
| 186 | DEFAULT_PT_LDTC | LDTC | 3 | SOLID,NOCTYPEDRAW | 1 | 1 | Y |  |  |  | Full |
| 187 | DEFAULT_PT_SLCN | SLCN | 8 | PART,CONDUCTS,HOT_GLOW,LIFE_DEC | 1 | 1 | Y | Y | Y | Y | Full |
| 188 | DEFAULT_PT_PTNM | PTNM | 9 | SOLID,CONDUCTS,HOT_GLOW,LIFE_DEC,SPARKSETTLE | 1 | 1 | Y | Y | Y | Y | Full |
| 189 | DEFAULT_PT_VSNS | VSNS | 3 | SOLID | 1 | 1 | Y |  |  |  | Full |
| 190 | DEFAULT_PT_ROCK | ROCK | 9 | SOLID,HOT_GLOW | 1 | 1 |  | Y | Y | Y | Generic |
| 191 | DEFAULT_PT_LITH | LITH | 5 | PART,LIFE_DEC | 1 | 1 | Y |  | Y | Y | Full |
| 192 | DEFAULT_PT_RSST | RSST | 7 | LIQUID,CONDUCTS,PHOTPASS,NEUTPASS,LIFE_DEC | 1 | 1 | Y |  |  |  | Full |
| 193 | DEFAULT_PT_RSSS | RSSS | 9 | SOLID,NEUTPASS | 1 | 1 | Y |  |  |  | Full |
| 194 | DEFAULT_PT_BASE | BASE | 7 | LIQUID,DEADLY | 1 | 1 | Y |  | Y |  | Full |
| 195 | DEFAULT_PT_SEED | SEED | 8 | PART,NEUTPASS | 1 | 1 | Y | Y | Y | Y | Full |
