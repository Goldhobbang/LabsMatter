# Verification guidelines — Agent 3 element families

Reference: The-Powder-Toy @ `2b7221a46d378589b4de7683f16d3ebc92be6bd9`
Implementation: `crates/realmatter-tpt/src/updates.rs` (`update_fn`, `create_fn`).

The project intentionally has no new automated test suite. Verify each family in
the real browser build after Agent 1 wires the module (or with the WASM
`step`/`paint` exports). Record the observed result and mark the family below.

## C01 — Thermal chemical reactions

| Element | Check | Expected |
|---|---|---|
| FIRE | Place WOOD next to FIRE | WOOD ignites, becomes FIRE, then SMKE |
| OIL / DESL | Ignite with FIRE | Fire spreads, smoke appears |
| H2 | Ignite with FIRE/PLSM | H2 → FIRE and heats neighbours |
| C5 | Heat a C5 line | C5 → CFLM (cold flame), pressure rises |
| IGNT | Place IGNT next to FIRE | IGNT emits EMBR/FIRE, life counts down |
| BANG | Heat above 673 K or touch FIRE | BANG detonates into FIRE/SMKE/EMBR |
| GUNP / BOMB / C5 | Ignite | Explosion, EMBR debris, pressure spike |

## C02 — State transitions

| Element | Check | Expected |
|---|---|---|
| WATR | Cool below 273 K / heat above 373 K | ICE / WTRV |
| ICEI / SNOW | Heat | WATR |
| LAVA | Cool | STNE or ctype target (BMTL, GLAS, ...) |
| METL / IRON / GOLD / TUNG | Heat above melting point | LAVA (with ctype) |
| QRTZ | Heat above 2573 K | LAVA(QRTZ); pressure shock → PQRT |
| CRMC | Cool under negative pressure | CLST |
| FRZZ / FRZW | Touch WATR | FRZW; FRZW eventually → ICEI(FRZW) |
| LNTG | Heat | NICE, then gas |

## C03 — Corrosion / dissolution

| Element | Check | Expected |
|---|---|---|
| ACID | Touch SAND/WOOD/IRON/SALT/GUNP | Target deleted, ACID life decreases; at life ≤ 50 becomes SMKE |
| ACID | Touch GLAS | No effect (GLAS is immune) |
| ACID | Touch LITH | LITH → H2 |
| CAUS | Touch matter | Corrodes like acid; GAS at pressure > 3 → RFRG |
| BASE | Touch ACID / CAUS | Neutralises into SLTW / kill |
| BASE | Touch OIL at concentration ≥ 70 | SOAP |
| IRON | Touch SALT/SLTW/WATR/O2/LO2 | Rusts to BMTL |
| GOLD | Touch rusted BMTL | BMTL → IRON |
| SLTW | Touch PLNT | Plant consumed |
| YEST | Heat 30–44 °C | Grows new YEST |

## C04 — Explosives

| Element | Check | Expected |
|---|---|---|
| BOMB | Touch any non-immune particle | 8-cell EMBR blast, pressure |
| C5 | Contact with cold matter / CFLM | CFLM flame |
| BANG | Heat / fire | Detonation chain |
| IGNT | Fire contact | EMBR/FIRE spray |
| FUSE / FSEP | Spark or heat | Plasma bursts, FUSE → FSEP |
| FWRK / FIRW | Ignite | EMBR firework burst |

## C05 — Radioactivity / nuclear

| Element | Check | Expected |
|---|---|---|
| URAN | Raise pressure | Temperature climbs |
| PLUT | Raise pressure | Emits NEUT |
| POLO | Time / PROT contact | Emits NEUT; at tmp2 ≥ 10 → PLUT |
| DEUT | Heat / compression | Life compresses then re-expands; implodes on PROT |
| NEUT | Fire at matter | Absorbed/converted per `can_move` table |
| ELEC | Touch water | O2 or H2; touch DEUT → life+1 |
| PROT | Particle collision | Decays to NBLE/CO2/O2/PLSM/POLO/URAN/PLUT/SING by energy |

## C06 — Electricity

| Element | Check | Expected |
|---|---|---|
| SPRK | Spark METL/IRON/GOLD/TUNG | Propagates, leaves ctype, cools |
| PSCN / NSCN | Spark through | Only conducts one polarity |
| BTRY | Place next to conductor | Conductor sparks |
| SWCH | Spark, or two red BRAYs diagonally | Toggles (life 10 ↔ 0) |
| LCRY | Spark / neighbour state | Bistable switch flips |
| NTCT / PTCT | Heat / cool | Conducts only in the right temperature band (via transition) |
| HSWC | life == 10 and neighbour HSWC | Heat conducts only when latched |
| INWR | Spark | Insulated wire, no propagation to neighbours |
| WIRE | Spark on PSCN head | Head/tail state machine |
| DTEC / TSNS / PSNS / LSNS / VSNS / LDTC | Spark / signal | Emit SPRK when detected |

## C07 — Powered materials / automation

| Element | Check | Expected |
|---|---|---|
| CLNE / PCLN | Set ctype, power PCLN | Clones ctype into empty neighbours |
| BCLN / PBCN | Directional fill | Fills along tmp direction |
| VOID / PVOD | Touch matter | Deletes matter (PVOD only when powered) |
| PUMP | Power it | Equalises pressure toward its temperature |
| GPMP | Power it | Adds gravity mass from temperature |
| GRAV | Fast movement | Flashes / stores life |
| GBMB | Touch matter | Positive then negative gravity mass |
| DLAY | Spark through | Delays spark by its temperature-derived life |
| STOR | Touch matter / spark | Stores then releases particle |
| PIPE | Pressure | Equalises along pipe axis |
| PRTI / PRTO | Portal pair | Stores/emits payloads (thread-local channel) |
| WIFI | Spark near WIFI | Wireless channel activates (transient) |
| ARAY / CRAY / DRAY | Spark | Emits BRAY / copies line |
| PSTN | Spark with PSCN/NSCN | Extends / retracts (simplified) |

## C08 — Energy particles / optics

| Element | Check | Expected |
|---|---|---|
| PHOT | Launch at QRTZ | Reflects; at ISOZ/ISZS splits; at RSST solidifies |
| BGLA | Photon | Refracts |
| FILT | Photon through | Wavelength/colour changes propagate to FILT chain |
| LIGH | Create | Branches into LIGH segments, ignites flammables, sparks conductors |
| THDR | Create | Sparks conductors, explodes on matter |
| ELEC | Create | Travels, reacts with matter |
| NEUT / PROT / DEUT | Create | Travel, react, decay |

## C09 — Special particle systems

| Element | Check | Expected |
|---|---|---|
| CLNE family | See C07 | — |
| BHOL / NBHL | Create | BHOL eats; NBHL attracts via gravity |
| WHOL / NWHL | Create | Adds positive/negative gravity mass |
| SING | Create | Attracts matter, grows, then explodes into PHOT/NEUT/ELEC |
| WARP | Create | Teleports neighbours |
| VIBR / BVBR | Heat / pressure | Absorbs energy, explodes into EXOT + ELEC + PHOT + BREC |
| ANAR | Create | Annihilates BIZR/CFLM neighbours |
| INVIS | Pressure | Detects pressure beyond tmp threshold |
| SHLD1–4 | Spark | Grows shield lattice, changes state |
| TTAN | Build a line | Blocks air when ≥ 2 aligned |
| RSSS | Create | Blocks air; clone ctype sync |
| RSST | Touch GUNP/BCOL | FIRW / FSEP |

## C10 — Life / biological

| Element | Check | Expected |
|---|---|---|
| LIFE | Paint | Game of Life generation per `GOL_RULES` |
| PLNT | Place near WATR | Grows, consumes water |
| VINE | Place | Grows and turns into PLNT |
| SEED | Place on SAND/SPNG, add water | Grows into PLNT after life > 200 |
| YEST | Heat | Grows / becomes DYST |
| VIRS / VRSS / VRSG | Contact | Infects neighbours; SOAP cures; PLSM burns |
| STKM / STKM2 / FIGH | — | **Blocked**: requires player controller |

## C11 — Remaining exotic

| Element | Check | Expected |
|---|---|---|
| EXOT | Create in LAVA(TTAN/GOLD) | Converts to LAVA(VIBR); decays to WARP |
| BIZR | Create | Blends dcolour into neighbours |
| ISOZ / ISZS | Low pressure | Emits PHOT |
| SLCN | Spark 4 cells away | Becomes SPRK |
| PTNM | Create | Shield growth, H2 fusion, ISZS→PLUT |
| LITH | Water / CO2 / sparks | Hydrogenation, carbonation, battery, explosion |
| MERC | Heat | Compresses/expands |
| GEL / SPNG / CONV | Water / pressure | Absorb, store, release water |
| CLST | Water / NITR | PSTS / BANG |
| BASE | See C03 | — |
| HEAC | Heat | Line-of-sight heat equaliser |
| CRMC | Pressure | CLST / LAVA |

## Regression checklist (run before marking a family PASS)

1. `cargo build --workspace` succeeds.
2. `cargo run -p realmatter-tpt --example list_unimpl` prints `0`.
3. Material IDs are unchanged (palette still loads; `docs/tpt-parity/inventory/elements.md`
   regenerated counts unchanged).
4. Save/load round-trips a world containing at least one element from each
   implemented family without material-ID corruption.
5. No panics when painting every implemented element and stepping 100 ticks.
6. Browser build (`web`) renders and steps without console errors.
