# Reaction inventory (source-derived)

Reference: The-Powder-Toy/The-Powder-Toy @ 2b7221a46d378589b4de7683f16d3ebc92be6bd9

## Authored interaction annotations (`//@` comments)

Every `//@` comment in `src/simulation/elements/*.cpp` is reproduced below,
grouped by the element source file that implements it. These annotations are
the reference authors' own description of each interaction and are matched by
the corresponding Rust handler in `crates/realmatter-tpt/src/updates.rs`.

### WATR (`WATR.cpp`)

- `WATR -> ICEI`
- `WATR -> WTRV`
- `WATR + SALT -> SLTW + SALT`
- `WATR + RBDM/LRBD -> FIRE + RBDM/LRBD`
- `WATR + SLTW -> 2xSLTW`
- `WATR + ROCK -> WATR + SAND/STNE`

### OIL (`OIL.cpp`)

- `OIL -> GAS`

### FIRE (`FIRE.cpp`)

- `FIRE -> PLSM`
- `PLSM(NBLE) -> NBLE`
- `PLSM + O2 + H2 -> WTRV(FIRE)`
- `FIRE + O2 + H2 -> WTRV(FIRE)`
- `FIRE -> SMKE`
- `LAVA(ROCK) -> LAVA(STNE)`
- `LAVA(ROCK) -> LAVA(BRMT/CNCT)`
- `LAVA(ROCK) -> LAVA(GOLD/QRTZ)`
- `LAVA(ROCK) -> LAVA(TTAN/IRON)`
- `LAVA(ROCK) -> LAVA(URAN/PLUT/TUNG)`
- `LAVA(GOLD) -> LAVA(PTNM)`
- `LAVA -> LAVA(ROCK)`
- `FIRE/PLSM/LAVA + THRM -> FIRE/PLSM/LAVA + LAVA(BMTL/THRM)`
- `LAVA(IRON) + COAL/BCOL -> LAVA(METL)`
- `LAVA + COAL/BCOL -> LAVA(SLCN)`
- `LAVA(QRTZ) + LAVA(CLST) -> 2xLAVA(CRMC)`
- `LAVA(SLCN) + O2 -> LAVA(SAND/CLST/PQRT/STNE)`
- `LAVA(SLCN) + LAVA(METL/BMTL) -> LAVA(NSCN) + LAVA(PSCN)`
- `LAVA(SLCN) + LAVA(SALT) -> LAVA(LITH)`
- `LAVA(ROCK) + LAVA(GOLD) -> 2xLAVA(GOLD)`

### STNE (`STNE.cpp`)

- `STNE -> LAVA`

### GUNP (`GUNP.cpp`)

- `GUNP -> FIRE`

### NITR (`NITR.cpp`)

- `NITR -> FIRE`

### GAS (`GAS.cpp`)

- `GAS -> OIL`
- `GAS -> FIRE`

### PLEX (`PLEX.cpp`)

- `PLEX -> FIRE`

### ICEI (`ICEI.cpp`)

- `ICEI -> SNOW`
- `ICEI + SALT/SLTW -> 2xSLTW`
- `ICEI + FRZZ -> ICEI + ICEI(FRZW)`

### METL (`METL.cpp`)

- `METL -> LAVA(METL)`

### SPRK (`SPRK.cpp`)

- `ETRD -> ETRD + PLSM`
- `NBLE -> PLSM(NBLE)`
- `TESC -> TESC + LIGH`
- `IRON + DSTW/SLTW/WATR -> IRON + O2/H2`

### SNOW (`SNOW.cpp`)

- `SNOW + SALT/SLTW -> 2xSLTW`

### WOOD (`WOOD.cpp`)

- `WOOD -> FIRE`
- `WOOD -> BCOL`

### NEUT (`NEUT.cpp`)

- `NEUT + WATR -> NEUT + DSTW`
- `NEUT + PLUT -> NEUT + URAN/LAVA(PLUT)`
- `NEUT + PLUT -> 2xNEUT`
- `NEUT + DEUT -> 2xNEUT`
- `NEUT + GUNP -> NEUT + DUST`
- `NEUT + DYST -> NEUT + YEST`
- `NEUT + YEST -> NEUT + DYST`
- `NEUT + PLEX -> NEUT + GOO`
- `NEUT + NITR -> NEUT + DESL`
- `NEUT + PLNT -> NEUT + WOOD`
- `NEUT + DESL/OIL -> NEUT + GAS`
- `NEUT + COAL -> NEUT + WOOD`
- `NEUT + BCOL -> NEUT + SAWD`
- `NEUT + DUST -> NEUT + FWRK`
- `NEUT + ACID -> NEUT + ISOZ`
- `NEUT + RFRG -> NEUT + GAS/CAUS`
- `NEUT + RSSS(ctype) -> ctype`
- `NEUT + RSSS -> RSST`
- `NEUT + BASE -> NEUT + LRBD`

### PLUT (`PLUT.cpp`)

- `PLUT -> NEUT`

### PLNT (`PLNT.cpp`)

- `PLNT -> FIRE`
- `PLNT -> WOOD`
- `PLNT -> GOO`
- `PLNT -> PLNT + SEED`
- `PLNT + WATR -> 2xPLNT`
- `PLNT + LAVA -> FIRE + LAVA`
- `PLNT + SMKE/CO2 -> PLNT + O2`

### ACID (`ACID.cpp`)

- `ACID + PLEX/NITR/GUNP/RBDM/LRBD -> 2xFIRE`
- `ACID + WTRV -> CAUS`
- `ACID + LITH -> ACID + H2`

### WTRV (`WTRV.cpp`)

- `WTRV + RBDM/LRBD -> FIRE + RBDM/LRBD`

### CNCT (`CNCT.cpp`)

- `CNCT -> LAVA(CNCT)`

### DSTW (`DSTW.cpp`)

- `DSTW -> ICEI(DSTW)`
- `DSTW -> WTRV`
- `DSTW + SALT -> 2xSLTW`
- `DSTW + SLTW -> 2xSLTW`
- `DSTW + WATR -> 2xWATR`
- `DSTW + RBDM/LRBD -> FIRE + RBDM/LRBD`
- `DSTW + SMKE -> BASE`

### SALT (`SALT.cpp`)

- `SALT -> LAVA(SALT)`

### SLTW (`SLTW.cpp`)

- `SLTW -> ICE(SLTW)`
- `SLTW + SALT -> 2xSLTW`
- `SLTW + RBDM/LRBD -> FIRE + RBDM/LRBD`

### BMTL (`BMTL.cpp`)

- `BMTL -> LAVA(BMTL)`
- `BMTL + METL/IRON -> 2xBMTL`
- `BMTL -> BRMT`

### BRMT (`BRMT.cpp`)

- `BRMT -> LAVA(BMTL)`
- `BRMT + BREC -> BRMT + THRM`
- `BRMT + BREC -> THRM + BREC`

### PHOT (`PHOT.cpp`)

- `PHOT + ISOZ/ISZS -> 2xPHOT`
- `PHOT + RSST(ctype) -> ctype`
- `PHOT + RSST -> RSSS`

### WAX (`WAX.cpp`)

- `WAX -> MWAX`

### MWAX (`MWAX.cpp`)

- `MWAX -> WAX`
- `MWAX -> FIRE`

### PSCN (`PSCN.cpp`)

- `PSCN -> LAVA(PSCN)`

### NSCN (`NSCN.cpp`)

- `NSCN -> LAVA(NSCN)`

### LNTG (`LNTG.cpp`)

- `LNTG -> NICE`

### RBDM (`RBDM.cpp`)

- `RBDM -> LRBD`

### LRBD (`LRBD.cpp`)

- `LRBD -> RBDM`
- `LRBD -> FIRE`

### NTCT (`NTCT.cpp`)

- `NTCT -> LAVA(NTCT)`

### SAND (`SAND.cpp`)

- `SAND -> LAVA(GLAS)`

### GLAS (`GLAS.cpp`)

- `GLAS -> LAVA(GLAS)`
- `GLAS -> BGLA`
- `GLAS -> GLAS + LNTG`

### PTCT (`PTCT.cpp`)

- `PTCT -> LAVA(PTCT)`

### BGLA (`BGLA.cpp`)

- `BGLA -> LAVA(GLAS)`

### THDR (`THDR.cpp`)

- `THDR -> FIRE`

### NICE (`NICE.cpp`)

- `NICE -> LNTG`

### NBLE (`NBLE.cpp`)

- `NBLE -> CO2 + NEUT + PHOT + sometimes ELEC + PLSM`

### BTRY (`BTRY.cpp`)

- `BTRY -> PLSM`

### LCRY (`LCRY.cpp`)

- `LCRY -> BGLA`

### STKM (`STKM.cpp`)

- `STKM -> FIRE`

### SMKE (`SMKE.cpp`)

- `SMKE -> FIRE`

### DESL (`DESL.cpp`)

- `DESL -> FIRE`
- `DESL -> FIRE`

### COAL (`COAL.cpp`)

- `COAL -> BCOL`

### LO2 (`LO2.cpp`)

- `LO2 -> O2`

### O2 (`O2.cpp`)

- `O2 -> LO2`
- `O2 + FIRE -> 2xFIRE`
- `O2 + PLSM -> FIRE + PLSM`
- `O2 -> BRMT + NEUT + PHOT + GRVT + sometimes PLSM`

### INWR (`INWR.cpp`)

- `INWR -> LAVA(INWR)`

### YEST (`YEST.cpp`)

- `YEST -> DYST`
- `YEST -> DYST`

### DYST (`DYST.cpp`)

- `DYST -> DUST`

### GLOW (`GLOW.cpp`)

- `GLOW + WATR -> DEUT`
- `GLOW + GEL -> RSST`

### BRCK (`BRCK.cpp`)

- `BRCK -> STNE`
- `BRCK -> LAVA(BRCK)`

### FIRW (`FIRW.cpp`)

- `FIRW -> EMBR`

### FUSE (`FUSE.cpp`)

- `FUSE -> FUSE + PLSM`
- `FUSE -> FSEP`

### FSEP (`FSEP.cpp`)

- `FSEP -> FSEP + PLSM`

### AMTR (`AMTR.cpp`)

- `AMTR + anything -> AMTR + PHOT`

### IRON (`IRON.cpp`)

- `IRON -> LAVA(IRON)`
- `IRON + SALT -> BMTL + SALT`
- `IRON + SLTW -> BMTL + SLTW`
- `IRON + WATR -> BMTL + WATR`
- `IRON + O2 -> BMTL + O2`
- `IRON + LO2 -> BMTL + LO2`

### CO2 (`CO2.cpp`)

- `CO2 -> DRIC`
- `CO2 -> WATR`
- `CO2 + WATR/DSTW -> WATR + CBNW`
- `CO2 + WATR/DSTW -> CBNW`
- `CO2 -> O2 + NEUT`
- `CO2 -> O2 + NEUT + ELEC`

### DRIC (`DRIC.cpp`)

- `DRIC -> CO2`

### CBNW (`CBNW.cpp`)

- `CBNW -> ICEI(CBNW)`
- `CBNW -> WTRV`
- `CBNW -> CO2`
- `CBNW -> CO2`
- `CBNW -> CO2`
- `CBNW + RBDM/LRBD -> CBNW + FIRE`

### CAUS (`CAUS.cpp`)

- `CAUS + GAS -> 2xRFRG`

### LIGH (`LIGH.cpp`)

- `LIGH + flammable -> LIGH + FIRE`
- `LIGH + DEUT/PLUT -> LIGH + NEUT`
- `LIGH + HEAC -> LIGH + LAVA(HEAC)`

### DEST (`DEST.cpp`)

- `DEST + PLUT/DEUT -> DEST + NEUT`
- `DEST + INSL -> DEST + PLSM`

### SPNG (`SPNG.cpp`)

- `SPNG -> FIRE`
- `SPNG + SLTW -> SPNG + SALT`
- `SPNG + CBNW -> SPNG + CO2`
- `SPNG + PSTE -> SPNG + CLST`
- `SPNG -> SPNG + WATR`
- `SPNG -> SPNG + WTRV`

### RIME (`RIME.cpp`)

- `RIME -> FOG`
- `RIME + FOG -> 2xFOG`

### FOG (`FOG.cpp`)

- `FOG -> WTRV`
- `FOG -> RIME`

### WARP (`WARP.cpp`)

- `WARP -> WARP + ELEC`

### FWRK (`FWRK.cpp`)

- `FWRK -> EMBR`

### PIPE (`PIPE.cpp`)

- `PIPE -> BRMT`
- `PIPE -> PIPE + BRCK`

### FRZZ (`FRZZ.cpp`)

- `FRZZ -> SNOW`
- `FRZZ + WATR -> FRZW`

### FRZW (`FRZW.cpp`)

- `FRZW -> ICEI(FRZW)`
- `FRZW + WATR -> 2xFRZW`
- `FRZW -> ICEI(FRZW)`

### BIZR (`BIZR.cpp`)

- `BIZR -> BIZRG`
- `BIZR -> BIZRS`

### BIZRG (`BIZRG.cpp`)

- `BIZRG -> BIZR`

### BIZRS (`BIZRS.cpp`)

- `BIZRS -> BIZR`

### ISOZ (`ISOZ.cpp`)

- `ISOZ -> ISZS`
- `ISOZ -> PHOT`

### ISZS (`ISZS.cpp`)

- `ISZS -> ISOZ`
- `ISZS -> PHOT`

### PSTE (`PSTE.cpp`)

- `PSTE -> PSTS`
- `PSTE -> BRCK`

### PSTS (`PSTS.cpp`)

- `PSTS -> PSTE`

### ANAR (`ANAR.cpp`)

- `ANAR + CFLM -> 2xCFLM`

### VINE (`VINE.cpp`)

- `VINE -> FIRE`
- `VINE -> PLNT`

### WIFI (`WIFI.cpp`)

- `WIFI -> BRMT`

### STKM2 (`STKM2.cpp`)

- `STKM2 -> FIRE`

### BOMB (`BOMB.cpp`)

- `BOMB -> EMBR`
- `BOMB -> EMBR`

### C5 (`C5.cpp`)

- `C5 -> CFLM`

### SING (`SING.cpp`)

- `SING -> PHOT`
- `SING -> NEUT`
- `SING -> ELEC`

### QRTZ (`QRTZ.cpp`)

- `QRTZ -> LAVA(QRTZ)`
- `QRTZ -> PQRT`
- `PQRT/QRTZ -> 2xQRTZ`

### PQRT (`PQRT.cpp`)

- `PQRT -> LAVA(QRTZ)`

### EMP (`EMP.cpp`)

- `EMP + PSCN/NSCN/PTCT/NTCT/INST/SWCH/WIRE -> EMP + BREC/NTCT`
- `EMP + METL -> EMP + BMTL/BRMT`
- `EMP + BMTL -> EMP + BRMT`
- `EMP + WIFI -> EMP + BREC`
- `EMP + SWCH -> EMP + BREC`
- `EMP + ARAY -> EMP + BREC`

### BREC (`BREC.cpp`)

- `BREC -> EXOT`

### ELEC (`ELEC.cpp`)

- `ELEC + GLAS -> EMBR + GLAS`
- `ELEC + WATR/DSTW/SLTW/CBNW -> O2/H2`
- `ELEC + PROT/NEUT -> H2`

### BANG (`BANG.cpp`)

- `BANG -> FIRE`
- `BANG -> SMKE`
- `BANG -> EMBR`

### IGNT (`IGNT.cpp`)

- `IGNT -> FIRE`
- `IGNT -> IGNT + EMBR`
- `IGNT -> IGNT + FIRE`

### BOYL (`BOYL.cpp`)

- `BOYL + WATR -> BOYL + FOG`
- `BOYL + O2 -> WATR`

### GEL (`GEL.cpp`)

- `GEL + PSTE -> GEL + CLST`
- `GEL + SLTW -> GEL + SALT`
- `GEL + CBNW -> GEL + CO2`

### TTAN (`TTAN.cpp`)

- `TTAN -> LAVA(TTAN)`

### EXOT (`EXOT.cpp`)

- `EXOT + LAVA(TTAN/GOLD) -> LAVA(VIBR)`
- `EXOT -> WARP`
- `EXOT -> WARP`
- `EXOT(PROT) -> CFLM`

### H2 (`H2.cpp`)

- `H2 + DESL -> OIL + WATR`
- `H2 + FIRE -> 2xFIRE`
- `H2 + PLSM/LAVA -> FIRE + PLSM/LAVA`
- `H2 -> NBLE + NEUT + PHOT`
- `H2 -> NBLE + NEUT + PHOT + ELEC`
- `H2 -> NBLE + NEUT + PHOT + ELEC + PLSM`

### CLST (`CLST.cpp`)

- `CLST -> LAVA(CLST)`
- `CLST + WATR -> PSTS`
- `CLST + NITR -> 2xBANG`

### DMG (`DMG.cpp`)

- `DMG + BMTL -> BRMT`
- `DMG + GLAS -> BGLA`
- `DMG + COAL -> BCOL`
- `DMG + QRTZ -> PQRT`
- `DMG + TUNG -> BRMT(TUNG)`
- `DMG + WOOD -> SAWD`

### VIBR (`VIBR.cpp`)

- `VIBR/BVBR -> EXOT + ELEC + PHOT + BREC`
- `VIBR/BVBR + EXOT -> 2xEXOT`
- `VIBR + ANAR -> BVBR`

### GOLD (`GOLD.cpp`)

- `GOLD -> LAVA(GOLD)`
- `GOLD + BMTL -> GOLD + IRON`

### TUNG (`TUNG.cpp`)

- `TUNG -> FIRE`
- `TUNG -> LAVA(TUNG)`
- `TUNG -> BRMT(TUNG)`

### PROT (`PROT.cpp`)

- `PROT + LCRY -> PHOT + LCRY`
- `PROT + TUNG -> AMTR`
- `PROT -> SING`
- `PROT -> PLUT`
- `PROT -> URAN`
- `PROT -> POLO`
- `PROT -> PLSM`
- `PROT -> O2`
- `PROT -> CO2`
- `PROT -> NBLE`

### VIRS (`VIRS.cpp`)

- `VIRS -> VRSS`
- `VIRS -> VRSG`
- `VIRS/VRSS/VRSG + PLSM -> 2xPLSM`

### VRSS (`VRSS.cpp`)

- `VRSS -> VIRS`

### VRSG (`VRSG.cpp`)

- `VRSG -> VIRS`

### CRMC (`CRMC.cpp`)

- `CRMC -> LAVA(CRMC)`
- `CRMC -> CLST`

### POLO (`POLO.cpp`)

- `POLO -> LAVA(POLO)`
- `POLO -> POLO + NEUT`
- `POLO -> POLO + NEUT`
- `POLO -> PLUT`

### RFRG (`RFRG.cpp`)

- `RFRG -> RFGL`

### RFGL (`RFGL.cpp`)

- `RFGL -> RFRG`

### SLCN (`SLCN.cpp`)

- `SLCN -> LAVA(SLCN)`

### PTNM (`PTNM.cpp`)

- `PTNM -> LAVA(PTNM)`
- `PTNM + WTRV + BCOL -> PTNM + OIL`
- `PTNM + H2 + DESL -> PTNM + OIL + WATR`
- `PTNM + H2 + O2 -> PTNM + 2xDSTW`
- `PTNM + 2xH2 -> PTNM + NBLE + NEUT + PHOT + maybe ELEC`
- `PTNM + ISZS/ISOZ -> PTNM + PLUT + PHOT`
- `PTNM + GAS -> PTNM + INSL`
- `PTNM + BREC -> PTNM + EXOT`
- `PTNM + SMKE -> PTNM + CO2`
- `PTNM + RSST -> PTNM + BIZR`

### ROCK (`ROCK.cpp`)

- `ROCK -> STNE`
- `ROCK -> LAVA(ROCK)`

### LITH (`LITH.cpp`)

- `LITH -> LAVA(LITH/GLAS)`
- `LITH -> LITH + FIRE`
- `LITH + SLTW/WATR/DSTW/CBNW -> LITH + WTRV`
- `LITH + SLTW/WTRV/WATR/DSTW/CBNW -> LITH + H2`
- `LITH + O2 -> 2xPLSM`
- `LITH -> LAVA(LITH)`
- `LITH -> LAVA(GLAS)`

### RSST (`RSST.cpp`)

- `RSST + GUNP -> FIRW`
- `RSST + BCOL -> FSEP`

### BASE (`BASE.cpp`)

- `BASE -> BOYL`
- `BASE -> ICE(BASE)`
- `BASE + WATR/DSTW/CBNW -> 2xBASE`
- `BASE + ACID -> 2xSLTW`
- `BASE + CAUS -> SLTW`
- `BASE + OIL -> SOAP`
- `BASE + GOO -> BASE + GEL`
- `BASE + BCOL -> BASE + GUNP`
- `BASE + LAVA(ROCK) -> MERC`
- `BASE + conductive solid -> BASE + BMTL`

### SEED (`SEED.cpp`)

- `SEED -> FIRE`
- `SEED -> MWAX`
- `SEED -> PLNT`

## Central state-transition edges

These transitions are applied by `transition_phase` in
`crates/realmatter-tpt/src/transition.rs` (heat/pressure driven).

| from | to (low temp) | to (high temp) | to (low pres) | to (high pres) |
|---|---|---|---|---|
| WATR | ICE | WTRV | - | - |
| OIL | - | GAS | - | - |
| FIRE | - | PLSM | - | - |
| STNE | - | LAVA | - | - |
| GUN | - | FIRE | - | - |
| NITR | - | FIRE | - | - |
| GAS | - | FIRE | - | OIL |
| C-4 | - | FIRE | - | - |
| ICE | - | special | - | SNOW |
| METL | - | LAVA | - | - |
| WOOD | - | FIRE | - | - |
| PLNT | - | FIRE | - | - |
| CNCT | - | LAVA | - | - |
| DSTW | ICE | WTRV | - | - |
| SALT | - | LAVA | - | - |
| SLTW | ICE | special | - | - |
| BMTL | - | LAVA | - | special |
| WAX | - | MWAX | - | - |
| MWAX | WAX | FIRE | - | - |
| PSCN | - | LAVA | - | - |
| NSCN | - | LAVA | - | - |
| LN2 | NICE | NONE | - | - |
| RBDM | - | LRBD | - | - |
| LRBD | RBDM | FIRE | - | - |
| NTCT | - | LAVA | - | - |
| SAND | - | LAVA | - | - |
| GLAS | - | LAVA | - | - |
| PTCT | - | LAVA | - | - |
| BGLA | - | LAVA | - | - |
| NICE | - | LN2 | - | - |
| BTRY | - | PLSM | - | - |
| LCRY | - | BGLA | - | - |
| STKM | - | FIRE | - | - |
| SMKE | - | FIRE | - | - |
| DESL | - | FIRE | - | FIRE |
| LOXY | - | OXYG | - | - |
| OXYG | LOXY | - | - | - |
| INWR | - | LAVA | - | - |
| YEST | - | DYST | - | - |
| DYST | - | DUST | - | - |
| BRCK | - | LAVA | - | STNE |
| IRON | - | LAVA | - | - |
| CO2 | DRIC | - | - | - |
| DRIC | - | CO2 | - | - |
| BUBW | ICE | WTRV | - | - |
| SPNG | - | FIRE | - | - |
| FOG | - | WTRV | - | - |
| PIPE | - | - | - | BRMT |
| FRZZ | ICE | FRZW | - | SNOW |
| FRZW | ICE | - | - | - |
| BIZR | BIZG | BIZS | - | - |
| BIZG | - | BIZR | - | - |
| BIZS | BIZR | - | - | - |
| ISOZ | ISZS | - | - | - |
| ISZS | - | ISOZ | - | - |
| PSTE | - | BRCK | - | PSTS |
| PSTS | - | - | PSTE | - |
| VINE | - | FIRE | - | - |
| SHLD | - | - | - | NONE |
| SHD2 | - | - | - | NONE |
| SHD3 | - | - | - | NONE |
| SHD4 | - | - | - | NONE |
| WIFI | - | - | - | BRMT |
| STK2 | - | FIRE | - | - |
| QRTZ | - | LAVA | - | - |
| PQRT | - | LAVA | - | - |
| IGNC | - | FIRE | - | - |
| TTAN | - | LAVA | - | - |
| CLST | - | LAVA | - | - |
| FIGH | - | FIRE | - | - |
| GOLD | - | LAVA | - | - |
| VIRS | VRSS | VRSG | - | - |
| VRSS | - | VIRS | - | - |
| VRSG | VIRS | - | - | - |
| POLO | - | LAVA | - | - |
| RFRG | - | - | - | RFGL |
| RFGL | - | - | RFRG | - |
| SLCN | - | LAVA | - | - |
| PTNM | - | LAVA | - | - |
| ROCK | - | LAVA | - | STNE |
| LITH | - | LAVA | - | - |
| SEED | - | FIRE | - | - |

## Special (`ST`) transitions

The reference marks these transitions `ST` and resolves them with element-specific
code rather than a fixed target type:

- **LAVA**: low_temperature
- **ICE**: high_temperature
- **SNOW**: high_temperature
- **WTRV**: low_temperature
- **SLTW**: high_temperature
- **BMTL**: high_pressure
- **BRMT**: high_temperature
- **RIME**: high_temperature
- **CRMC**: high_temperature

## Coverage summary

- Source files with `//@` annotations: 118
- Annotated interactions: 317
- Elements with a temperature/pressure transition edge: 82
