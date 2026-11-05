# LabsMatter — TPT Walls, Tools and GoL Rules

> Generated from the pinned reference. Machine-readable form in
> `docs/tpt-parity/data/tpt-reference.json`.

## Walls (19)

| ID | Identifier | Name | Colour | Glow | Drawstyle | Description |
|---|---|---|---|---|---|---|
| 0 | `DEFAULT_WL_ERASE` | ERASE | #%06x | #%06x | 0 | Erases walls. |
| 1 | `DEFAULT_WL_CNDTW` | CONDUCTIVE WALL | #%06x | #%06x | 0 | Blocks everything. Conductive. |
| 2 | `DEFAULT_WL_EWALL` | EWALL | #%06x | #%06x | 0 | E-Wall. Becomes transparent when electricity is connected. |
| 3 | `DEFAULT_WL_DTECT` | DETECTOR | #%06x | #%06x | 1 | Detector. Generates electricity when a particle is inside. |
| 4 | `DEFAULT_WL_STRM` | STREAMLINE | #%06x | #%06x | 0 | Streamline. Creates a line that follows air movement. |
| 5 | `DEFAULT_WL_FAN` | FAN | #%06x | #%06x | 1 | Fan. Accelerates air. Use the line tool to set direction and strength. |
| 6 | `DEFAULT_WL_LIQD` | LIQUID WALL | #%06x | #%06x | 2 | Allows liquids, blocks all other particles. Conductive. |
| 7 | `DEFAULT_WL_ABSRB` | ABSORB WALL | #%06x | #%06x | 1 | Absorbs particles but lets air currents through. |
| 8 | `DEFAULT_WL_WALL` | WALL | #%06x | #%06x | 3 | Basic wall, blocks everything. |
| 9 | `DEFAULT_WL_AIR` | AIRONLY WALL | #%06x | #%06x | 1 | Allows air, but blocks all particles. |
| 10 | `DEFAULT_WL_POWDR` | POWDER WALL | #%06x | #%06x | 1 | Allows powders, blocks all other particles. |
| 11 | `DEFAULT_WL_CNDTR` | CONDUCTOR | #%06x | #%06x | 2 | Conductor. Allows all particles to pass through and conducts electricity. |
| 12 | `DEFAULT_WL_EHOLE` | EHOLE | #%06x | #%06x | 0 | E-Hole. absorbs particles, releases them when powered. |
| 13 | `DEFAULT_WL_GAS` | GAS WALL | #%06x | #%06x | 1 | Allows gases, blocks all other particles. |
| 14 | `DEFAULT_WL_GRVTY` | GRAVITY WALL | #%06x | #%06x | 4 | Gravity wall. Newtonian Gravity has no effect inside a box drawn with this. |
| 15 | `DEFAULT_WL_ENRGY` | ENERGY WALL | #%06x | #%06x | 4 | Allows energy particles, blocks all other particles. |
| 16 | `DEFAULT_WL_NOAIR` | AIRBLOCK WALL | #%06x | #%06x | 1 | Allows all particles, but blocks air. |
| 17 | `DEFAULT_WL_ERASEA` | ERASEALL | #%06x | #%06x | 0 | Erases walls, particles, and signs. |
| 18 | `DEFAULT_WL_STASIS` | STASIS WALL | #%06x | #%06x | 0 | Freezes particles inside the wall in place until powered. |

Drawstyle values are interpreted in `src/graphics/Renderer.cpp`; the
`drawstyle` field selects which wall texture generator runs.

## Simulation tools (11)

| ID | Tool | Identifier | Colour | Hooks implemented | Description |
|---|---|---|---|---|---|
| 0 | HEAT | `DEFAULT_TOOL_HEAT` | #ffdd00 | Perform | Heats the targeted element. |
| 1 | COOL | `DEFAULT_TOOL_COOL` | #00ddff | Perform | Cools the targeted element. |
| 2 | AIR | `DEFAULT_TOOL_AIR` | #ffffff | Perform | Air, creates airflow and pressure. |
| 3 | VAC | `DEFAULT_TOOL_VAC` | #303030 | Perform | Vacuum, reduces air pressure. |
| 4 | PGRV | `DEFAULT_TOOL_PGRV` | #ccccff | Perform | Creates a short-lasting gravity well. |
| 5 | NGRV | `DEFAULT_TOOL_NGRV` | #aaccff | Perform | Creates a short-lasting negative gravity well. |
| 6 | MIX | `DEFAULT_TOOL_MIX` | #ffd090 | Perform | Mixes particles. |
| 7 | CYCL | `DEFAULT_TOOL_CYCL` | #132f5b | Perform | Cyclone, produces swirling air currents |
| 8 | AMBM | `DEFAULT_TOOL_AMBM` | #00ddff | Perform | Decreases ambient air temperature. |
| 9 | AMBP | `DEFAULT_TOOL_AMBP` | #ffdd00 | Perform | Increases ambient air temperature. |
| 10 | WIND | `DEFAULT_TOOL_WIND` | #404040 | PerformDrag, PerformDrawLine | Creates air movement. |

## Built-in Game of Life rules (23)

| ID | Name | ctype | Rule bits | Stay | Begin | States | Colour | Description |
|---|---|---|---|---|---|---|---|---|
| 0 | GOL | `GT_GOL` | `0x0080C` | 000001100 | 000000100 | 2 | #0cac00 | Game Of Life: Begin 3/Stay 23 |
| 1 | HLIF | `GT_HLIF` | `0x0480C` | 000001100 | 000100100 | 2 | #ff0000 | High Life: B36/S23 |
| 2 | ASIM | `GT_ASIM` | `0x038F0` | 011110000 | 000011100 | 2 | #0000ff | Assimilation: B345/S4567 |
| 3 | 2X2 | `GT_2x2` | `0x04826` | 000100110 | 000100100 | 2 | #ffff00 | 2X2: B36/S125 |
| 4 | DANI | `GT_DANI` | `0x1C9D8` | 111011000 | 011100100 | 2 | #00ffff | Day and Night: B3678/S34678 |
| 5 | AMOE | `GT_AMOE` | `0x0A92A` | 100101010 | 001010100 | 2 | #ff00ff | Amoeba: B357/S1358 |
| 6 | MOVE | `GT_MOVE` | `0x14834` | 000110100 | 010100100 | 2 | #ffffff | 'Move' particles. Does not move things.. it is a life type: B368/S245 |
| 7 | PGOL | `GT_PGOL` | `0x0A90C` | 100001100 | 001010100 | 2 | #e05010 | Pseudo Life: B357/S238 |
| 8 | DMOE | `GT_DMOE` | `0x1E9E0` | 111100000 | 011110100 | 2 | #500000 | Diamoeba: B35678/S5678 |
| 9 | 3-4 | `GT_34` | `0x01818` | 000011000 | 000001100 | 2 | #500050 | 3-4: B34/S34 |
| 10 | LLIF | `GT_LLIF` | `0x03820` | 000100000 | 000011100 | 2 | #505050 | Long Life: B345/S5 |
| 11 | STAN | `GT_STAN` | `0x1C9EC` | 111101100 | 011100100 | 2 | #5000ff | Stains: B3678/S235678 |
| 12 | SEED | `GT_SEED` | `0x00400` | 000000000 | 000000010 | 2 | #fbec7d | Seeds: B2/S |
| 13 | MAZE | `GT_MAZE` | `0x0083E` | 000111110 | 000000100 | 2 | #a8e4a0 | Maze: B3/S12345 |
| 14 | COAG | `GT_COAG` | `0x189EC` | 111101100 | 011000100 | 2 | #9acd32 | Coagulations: B378/S235678 |
| 15 | WALL | `GT_WALL` | `0x1F03C` | 000111100 | 011111000 | 2 | #0047ab | Walled cities: B45678/S2345 |
| 16 | GNAR | `GT_GNAR` | `0x00202` | 000000010 | 000000001 | 2 | #e5b73b | Gnarl: B1/S1 |
| 17 | REPL | `GT_REPL` | `0x0AAAA` | 010101010 | 001010101 | 2 | #259588 | Replicator: B1357/S1357 |
| 18 | MYST | `GT_MYST` | `0x139E1` | 111100001 | 010011100 | 2 | #0c3c00 | Mystery: B3458/S05678 |
| 19 | LOTE | `GT_LOTE` | `0x48938` | 100111000 | 001000100 | 4 | #ff0000 | Living on the Edge: B37/S3458/4 |
| 20 | FRG2 | `GT_FRG2` | `0x20816` | 000010110 | 100000100 | 3 | #006432 | Like Frogs rule: B3/S124/3 |
| 21 | STAR | `GT_STAR` | `0x98478` | 001111000 | 011000010 | 6 | #000040 | Like Star Wars rule: B278/S3456/6 |
| 22 | FROG | `GT_FROG` | `0x21806` | 000000110 | 100001100 | 3 | #006400 | Frogs: B34/S12/3 |
