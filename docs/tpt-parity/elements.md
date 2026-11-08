# Element inventory

Reference tree: The Powder Toy `2b7221a46d378589b4de7683f16d3ebc92be6bd9`.

`src/simulation/elements/` contains **195** `.cpp` files. That count is not the number of selectable palette entries. Variants (for example BIZR / BIZRG / BIZRS), tools, and internal helpers share that folder. A full menu-section pass of `SimulationData` is still required before anyone claims "195 elements". This file records what LabsMatter can actually simulate.

## LabsMatter registry

Selectable ids now run 0–25 (26 palette entries). Added since the 17-id list: snow (melts), copper (spark conductor), nitro (liquid burst), hydrogen (light burning gas), CO2 (sinks when air is on), plant (replaces adjacent water), gold, dust, bomb (fireball at 600 K, stone survives). The reference tree is still far larger. This is not full element parity.

Selectable ids in the WASM material path and the palette:

| id | abbr | role | behavior beyond "sits there" |
|---:|---|---|---|
| 0 | VOID | tool | erase |
| 1 | STNE | solid wall | fixed, acid-resistant |
| 2 | SAND | powder | falls, piles, acid dissolves it |
| 3 | WATR | liquid | flows, freezes, boils |
| 4 | OIL | liquid | flows, burns |
| 5 | STM | gas | rises when air is on, condenses |
| 6 | FIRE | energy | ignites fuel, becomes smoke |
| 7 | WOOD | solid | burns, acid |
| 8 | IRON | solid conductor | heat, acid, carries SPRK |
| 9 | LAVA | liquid | hot, dense |
| 10 | ACID | liquid | deletes sand/wood/iron/salt/gunpowder |
| 11 | SALT | powder | falls |
| 12 | SMKE | gas | rises when air is on |
| 13 | ICE | solid | melts to water |
| 14 | GLAS | solid | acid-resistant |
| 15 | GUN | powder | falls, bursts when lit |
| 16 | SPRK | energy | moves onto adjacent iron, then becomes iron |

Everything else in the 195-file reference tree is **missing**. Electronics other than this spark-on-iron rule, portals, clones, life, pipes, and stickmen are not registered.

## Engine modes (this batch)

These are real WASM flags, default gravity down, air on, edge solid, Newtonian off, ambient off.

| Mode | Values | Effect |
|---|---|---|
| Gravity | down / off / radial | Powders fall, or stop falling (liquids only spread), or step toward the center |
| Air | on / off / press | Gas motion on, fully off, or pressure field without gas motion |
| Edge | void / solid / wrap | Border cells deleted, blocked, or wrapped to the opposite side |
| Newtonian | on / off | Powders step toward nearby mass |
| Ambient | on / off | Non-empty temperatures drift toward 295.15 K |

Not TPT-complete. No per-element ctype, no wall layer separate from stone, no powered electronics beyond SPRK on iron.
