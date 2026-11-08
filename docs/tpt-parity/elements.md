# Element inventory

Reference tree: The Powder Toy `2b7221a46d378589b4de7683f16d3ebc92be6bd9`.

`src/simulation/elements/` contains **195** `.cpp` files. That count is not the number of selectable palette entries. Variants (for example BIZR / BIZRG / BIZRS), tools, and internal helpers share that folder. A full menu-section pass of `SimulationData` is still required before anyone claims "195 elements". This file records what LabsMatter can actually simulate.

## LabsMatter registry

Selectable ids now run 0–61. Ids 0–25 are the powder shell. Ids 26–61 are the K7 electronics and specials. The reference tree is still larger. This is not full element parity.

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
| 16 | SPRK | energy | Sits on a conductor for several frames, then restores it. Jumps one empty cell unless INSL is the midpoint. |

Ids 26–61 are registered in the WASM material table and the palette. They are element updates, not a claim of reference parity. Gameplay acceptance stays with Agent B.

| id | abbr | role | what the update does | remaining difference from the pinned source |
|---:|---|---|---|---|
| 26 | METL | conductor | Four-frame spark, then a four-frame cooldown | No ETRD/BMTL neighbors. No random metal ignition exists in the pinned SPRK either |
| 27 | BTRY | source | Sparks conductors in the chebyshev-2 neighborhood except water, salt water, NTCT, PTCT, INWR | Does not spark elements this build does not have |
| 28 | PSCN | conductor | Sends to conductors and switches devices on. Rejects NSCN | — |
| 29 | NSCN | conductor | Receives. Does not pass back into PSCN. Switches devices off | — |
| 30 | INST | wire | PSCN floods the connected wire in one frame, including the crossed-wire skip. NSCN receives next frame | Flood does not reproduce the reference's edge margin |
| 31 | INWR | conductor | Only PSCN, NSCN, and other INWR | — |
| 32 | WIFI | channel | Temperature channel, one-frame delay, sparks adjacent PSCN/NSCN/INWR | Channel count is 100. Timing can be one frame earlier than a shifted flag |
| 33 | SWCH | switch | Conducts only while life is 10. PSCN turns it on, NSCN turns it off | Graphics are a flat green while on |
| 34 | PUMP | device | While life is 10, orthogonal pressure steps toward the pump temperature | Pressure is an offset from 101325 Pa. The next physics pass resets non-gas pressure |
| 35 | GPMP | device | While on, adds a velocity bias to adjacent mobile cells and spreads the on-state | No `set_gravity_bias(x, y, mass)` on the physics solver. This is not `gravIn.mass` |
| 36 | HSWC | device | While on, averages temperature with neighbors and spreads the on-state | Not the full heat-conduction kernel |
| 37 | PBCN | clone | Copies while life is 10. Disappears after repeated copies | High-pressure death timer is a copy counter (`tmp2 > 24`) |
| 38 | LCRY | crystal | PSCN/NSCN state machine with the reference switch fallthrough | Color is a brightness ramp, not the reference gradient |
| 39 | PPIP | pipe | PIPE transport plus a pause bit. PSCN clears it, NSCN sets it | Flood is immediate. Reference uses shifted flags |
| 40 | PIPE | pipe | Phase push, then eject. Empty pipe ingests one adjacent mobile cell | Not the reference 600-line state machine, RNG retries, or portal speed boost |
| 41 | EMP | pulse | A spark starts life 220. While life is above 180, nearby sparks clear and metal can become BRMT | Not the reference binomial probability |
| 42 | SLTW | liquid | Spark birth life 5, restore cooldown 54, temperature reset | Same conductor rules as water |
| 43 | RSST | liquid | Spark birth life 5. At life 0 the cell is deleted | — |
| 44 | NTCT | thermistor | Passes PSCN/NSCN only while hotter than 100 C. Metal heats it | — |
| 45 | PTCT | thermistor | Inverse of NTCT | — |
| 46 | INSL | insulator | Blocks a spark whose midpoint is this cell | Does not block orthogonal contact, matching `parts_avg` |
| 47 | PSTN | piston | Two or more pistons. PSCN extends, any other spark retracts. `tmp3` flags cancel a blocked move | One piston does nothing. This is the source shaft-plus-frame-line, not an arbitrary frame flood |
| 48 | FRME | frame | A straight perpendicular line, at most 15 cells either side, pushes the stack in front of each cell | `tmp == 0` is sticky. Curved or branching frames are not one connected component |
| 49 | STKM | player | Independent leg points, foot gravity, constraints, jump, element use, portal exit | No rocket boots, no subpixel collision, head velocity is damped by 0.9. Input is J / semicolon / I / O |
| 50 | STK2 | player | Separate player record from STKM | Numpad 4 / 6 / 8 / 5. Same movement gaps as STKM |
| 51 | FIGH | fighter | Chases the nearer living player and uses its element up close. Slots are reused | Does not target other fighters. The pinned FIGH update does not either. No pressure death, because solids sit at atmosphere |
| 52 | SOAP | liquid | Reciprocal links, middle-link stitch, wrap distance, stale index drop | Bubble start also accepts a pressure offset, because signed `pv` is not stored. No decoration wash, no oil foam |
| 53 | CLNE | clone | Copies `ctype`, or a sampled neighbor, into one empty orthogonal cell | — |
| 54 | PCLN | clone | Same as CLNE while life is 10 | Not BCLN |
| 55 | PRTI | portal | Stores one adjacent mobile cell on the temperature channel | Buffer cap 8 |
| 56 | PRTO | portal | Emits into an empty orthogonal neighbor. Stickmen exit here | — |
| 57 | PLUT | solid | A neutron heats it and can spawn a side neutron | Not the full reaction table |
| 58 | NEUT | energy | Steps by `tmp` direction. Glass and water pass it. Plutonium is the special hit | Direction is 0 right, 1 down, 2 left, 3 up |
| 59 | PHOT | energy | Passes glass and water. Anything else absorbs it and warms | — |
| 60 | ELEC | energy | Ignites the first conductor and dies. Passes glass and water | — |
| 61 | BRMT | solid | EMP product. Does not conduct | Not a sparkable broken-metal original |

Ids 0–15 and 17–25 keep the powder-shell behavior already on this branch. Gold is still a falling powder, not a conductor. Drawing SPRK onto a conductor stores that conductor. Drawing SPRK onto empty is rejected. A second STKM or STK2 is rejected.

## Classification

Implementation completeness is separate from gameplay acceptance. Only Agent B can accept gameplay.

| group | count | meaning |
|---|---:|---|
| Palette ids with an update or a powder-shell rule | 62 | ids 0–61 |
| Reference Update callbacks ported as their own handler | 36 | ids 26–61 |
| Handlers with a listed remaining difference | 36 | none of these are marked Full |
| Reference `.cpp` files with no handler | 159 | 195 minus the 36 new handlers. The older 26 ids are a powder shell, not those files |

The prompt's "195 live / 126 Full / 6 Partial" inventory is not this tree. Those K6 commits are not in the repository.

## Engine modes (this batch)

These are real WASM flags, default gravity down, air on, edge solid, Newtonian off, ambient off.

| Mode | Values | Effect |
|---|---|---|
| Gravity | down / off / radial | Powders fall, or stop falling (liquids only spread), or step toward the center |
| Air | on / off / press | Gas motion on, fully off, or pressure field without gas motion |
| Edge | void / solid / wrap | Border cells deleted, blocked, or wrapped to the opposite side |
| Newtonian | on / off | Powders step toward nearby mass |
| Ambient | on / off | Non-empty temperatures drift toward 295.15 K |

Not TPT-complete. There is no wall layer separate from stone. Particle fields `ctype`, `life`, `tmp`, `tmp2`, and `tmp3` live beside the cell, in the chemistry element state, and ride along in a save trailer the persist decoder does not read.
