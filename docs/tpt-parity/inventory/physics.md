# Physics Inventory — LabsMatter TPT Parity

Reference: The-Powder-Toy/The-Powder-Toy @ `2b7221a46d378589b4de7683f16d3ebc92be6bd9`

Owner: Agent 2 (physics). Primary code: `crates/realmatter-physics/` (cell-based engine used by WASM) and `crates/realmatter-tpt/` (TPT-parity engine).

Status legend: `DONE` implemented & wired; `PARTIAL` implemented with deviations; `MISSING` not implemented; `N/A` not applicable to this engine.

---

## P01 — Particle motion

| ID | Mechanism | Ref source | Algorithm/equation | State vars | Inputs/outputs | Coupled | Boundary | Stability | Current impl | Missing | Proposed Rust | Perf | Status |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| P01.01 | Position/velocity integration | `Simulation.cpp` UpdateParticles | v += a*dt; x += v*dt (sub-pixel) | Particle.x/y/vx/vy | forces → position | gravity, air, collisions | edge mode (void/solid/loop) | clamp velocity, finite check | `realmatter-tpt/src/tick.rs` update_particles; `realmatter-world/src/movement.rs` | sub-pixel integration in cell engine | keep CA + velocity fields | O(N) | PARTIAL |
| P01.02 | Acceleration | `Simulation.cpp` | a = F/m | vx,vy | forces | gravity, pressure | — | clamp | `PhysicsEngine::apply_forces` | per-material mass | add mass term | O(N) | PARTIAL |
| P01.03 | Gravity response | `Simulation.cpp` gravity field | particle_grav * dir | vy | settings.gravity_mode | gravity solver | edge | clamp | `Sim::gravity_field` (tpt); WASM gravity mode | — | expose modes in physics | O(N) | DONE |
| P01.04 | Powder falling | `movement.rs` move_powder_step | CA: down, down-left/right | cell material | density | gravity | bottom edge | UPDATED flag | `realmatter-world::move_powder_step` | — | keep | O(N) | DONE |
| P01.05 | Granular accumulation | `movement.rs` | pile when blocked | — | density | — | walls | — | CA handles | — | — | O(N) | DONE |
| P01.06 | Granular lateral motion | `movement.rs` | down-left/right | — | — | — | — | — | CA handles | — | — | O(N) | DONE |
| P01.07 | Liquid spreading | `movement.rs` move_liquid_step | fall then sideways | — | — | — | — | — | CA handles | — | — | O(N) | DONE |
| P01.08 | Liquid equalization | `Simulation.cpp` water equal test | density swap | — | water_equal_test | — | — | — | not implemented | water_equal_test setting | add equalization pass | O(N) | MISSING |
| P01.09 | Gas motion | `gas.rs` diffuse | rise + spread | vy | buoyancy | air | top edge | UPDATED flag | `gas::diffuse` | — | keep | O(N) | DONE |
| P01.10 | Diffusion | `gas.rs` diffuse | random cell swaps | vx,vy | diffusion coeff | air | bounds | clamp | `gas::diffuse`, air diffusion | — | keep | O(N) | DONE |
| P01.11 | Buoyancy | `fluid.rs` apply_buoyancy, `gas.rs` apply_buoyancy | (rho_ref - rho)/rho_ref * g | vy | density, gravity | gravity | — | clamp | implemented | Boussinesq validity | document | O(N) | DONE |
| P01.12 | Collision response | `movement.rs` try_sink_swap | density-driven swap | — | density | — | walls | — | implemented | restitution/elasticity | add elasticity param | O(N) | PARTIAL |
| P01.13 | Elasticity/restitution | `Simulation.cpp` | bounce factor | vx,vy | material | walls | walls | clamp | not implemented | restitution coeff | add to MaterialProps | O(N) | MISSING |
| P01.14 | Friction/damping | `fluid.rs` apply_viscosity | vx*=damp | vx,vy | viscosity | — | — | finite | implemented | — | keep | O(N) | DONE |
| P01.15 | Viscosity | `fluid.rs` apply_viscosity | damp = 1 - viscosity*dt | vx,vy | viscosity | — | — | clamp | implemented | — | keep | O(N) | DONE |
| P01.16 | Drag | `air.rs` apply_air_to_particles | vx += advection*field | vx,vy | air field | air | — | clamp | implemented | — | keep | O(N) | DONE |
| P01.17 | Advection | `air.rs` advect_gases, integrate_velocity | cell displacement by field | vx,vy | air field | air | bounds | clamp | implemented | — | keep | O(N) | DONE |
| P01.18 | Density/weight displacement | `movement.rs` try_sink_swap | di > dj swap | — | density | — | — | — | implemented | — | keep | O(N) | DONE |
| P01.19 | Particle swapping | `movement.rs` try_move/try_sink_swap | swap cells | pmap | can_move table | — | bounds | UPDATED flag | implemented (tpt) | — | keep | O(N) | DONE |
| P01.20 | Stationary/stagnant | `Simulation.cpp` stasis wall | freeze | flags | wall | walls | stasis wall | — | implemented (tpt) | — | keep | O(N) | DONE |
| P01.21 | Mobility classes | `movement.rs` | powder/liquid/gas/plasma | state | MaterialState | — | — | — | implemented | — | keep | O(N) | DONE |
| P01.22 | Energy-particle movement | `tick.rs` update_particles | PHOT/NEUT/GRVT move | x,y,vx,vy | can_move table | walls | bounds | clamp | implemented (tpt) | — | keep | O(N) | DONE |

## P02 — Air pressure and fluid dynamics

| ID | Mechanism | Ref source | Algorithm | State vars | Inputs/outputs | Coupled | Boundary | Stability | Current impl | Missing | Proposed Rust | Perf | Status |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| P02.01 | Pressure field | `Air.cpp` update_air | pv grid | pv | velocity divergence | particles | edge damping | clamp ±256 | `air.rs` AirField.pv | — | keep | O(N) | DONE |
| P02.02 | Horizontal air velocity | `Air.cpp` | vx grid | vx | pressure gradient | particles | edge damping | clamp | implemented | — | keep | O(N) | DONE |
| P02.03 | Vertical air velocity | `Air.cpp` | vy grid | vy | pressure gradient | particles | edge damping | clamp | implemented | — | keep | O(N) | DONE |
| P02.04 | Pressure gradients | `Air.cpp` | dp from vx/vy divergence | pv | vx,vy | — | — | — | implemented | — | keep | O(N) | DONE |
| P02.05 | Airflow advection | `Air.cpp` | kernel + far advection | vx,vy,pv | — | walls | wall stop | clamp | implemented | — | keep | O(N) | DONE |
| P02.06 | Momentum propagation | `Air.cpp` | velocity from pressure | vx,vy | pv | — | — | — | implemented | — | keep | O(N) | DONE |
| P02.07 | Pressure-velocity coupling | `Air.cpp` | divergence→pressure, gradient→velocity | pv,vx,vy | — | — | — | — | implemented | — | keep | O(N) | DONE |
| P02.08 | Viscous damping | `Air.cpp` | AIR_VLOSS, AIR_PLOSS | vx,vy,pv | — | — | — | — | implemented | — | keep | O(N) | DONE |
| P02.09 | Air diffusion | `Air.cpp` | kernel smoothing | vx,vy,pv | — | walls | — | — | implemented | — | keep | O(N) | DONE |
| P02.10 | Air compression | `Air.cpp` | pressure from divergence | pv | vx,vy | — | — | clamp | implemented | — | keep | O(N) | DONE |
| P02.11 | Air-particle interaction | `tick.rs` update_particles | momentum injection, hot air, advection | vx,vy,pv | particle vx,vy | particles | — | clamp | `apply_air_to_particles` | — | keep | O(N) | DONE |
| P02.12 | Air drag on powders | `tick.rs` | advection * field | vx,vy | air field | air | — | clamp | implemented | — | keep | O(N) | DONE |
| P02.13 | Air drag on liquids | `tick.rs` | advection * field | vx,vy | air field | air | — | clamp | implemented | — | keep | O(N) | DONE |
| P02.14 | Air drag on gases | `air.rs` advect_gases | field displacement | x,y | air field | air | bounds | — | implemented | — | keep | O(N) | DONE |
| P02.15 | Airflow around walls | `Air.cpp` | blockair zeroing | bmap_blockair | walls | walls | — | — | implemented | permeability per wall type | add wall permeability | O(N) | PARTIAL |
| P02.16 | Fans/forced air | `Air.cpp` | fan adds fvx/fvy | vx,vy | wall FAN | walls | — | clamp | tpt: fan_active; physics: not | fan in physics AirField | add fan support | O(N) | PARTIAL |
| P02.17 | Vacuum/pressure tools | `simtools/VAC.cpp`, `AIR.cpp` | add_pressure | pv | tool input | air | bounds | clamp | wasm apply_tool | expose physics op | keep | O(N) | DONE |
| P02.18 | Air blocking/permeability | `Air.cpp` | bmap_blockair | bmap_blockair | walls, particles | walls | — | — | rebuild_blockair | per-wall permeability | add permeability table | O(N) | PARTIAL |
| P02.19 | Airflow boundary conditions | `Air.cpp` | edge damping, edge_pressure/velocity | pv,vx,vy | settings | — | edge | clamp | implemented (tpt); physics uses 0 edge | edge settings in physics | add edge params | O(N) | PARTIAL |
| P02.20 | Pressure-induced transitions | `Simulation.cpp` | pressure thresholds | pressure | — | chemistry | — | — | not implemented | pressure-dependent phase | add hooks | O(N) | MISSING |

## P03 — Air simulation modes

| ID | Mechanism | Ref source | Algorithm | State vars | Inputs/outputs | Coupled | Boundary | Stability | Current impl | Missing | Proposed Rust | Perf | Status |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| P03.01 | AIR_ON | `SimulationSettings.h` | full calc | — | — | — | — | — | `AirMode::On` | — | keep | O(1) | DONE |
| P03.02 | AIR_PRESSUREOFF | settings | velocity only, pv=0 | pv | — | — | — | — | `AirMode::PressureOff` | — | keep | O(1) | DONE |
| P03.03 | AIR_VELOCITYOFF | settings | pressure only, vx/vy=0 | vx,vy | — | — | — | — | `AirMode::VelocityOff` | — | keep | O(1) | DONE |
| P03.04 | AIR_OFF | settings | both zeroed | pv,vx,vy | — | — | — | — | `AirMode::Off` | — | keep | O(1) | DONE |
| P03.05 | AIR_NOUPDATE | settings | no update | — | — | — | — | — | `AirMode::NoUpdate` | — | keep | O(1) | DONE |

## P04 — Gravity

| ID | Mechanism | Ref source | Algorithm | State vars | Inputs/outputs | Coupled | Boundary | Stability | Current impl | Missing | Proposed Rust | Perf | Status |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| P04.01 | Vertical gravity | `Simulation.cpp` | particle_grav | vy | settings.gravity_mode | motion | — | clamp | implemented | — | keep | O(N) | DONE |
| P04.02 | Gravity disabled | settings | zero | — | — | — | — | — | implemented | — | keep | O(1) | DONE |
| P04.03 | Radial gravity | `Simulation.cpp` | toward center | vx,vy | — | motion | — | clamp | implemented (tpt) | — | keep | O(N) | DONE |
| P04.04 | Custom gravity | settings | custom vector | vx,vy | custom_gravity | motion | — | clamp | implemented (tpt) | — | keep | O(N) | DONE |
| P04.05 | Mass-dependent Newtonian | `gravity/Fft.cpp` | FFT convolution 1/r² | grav_mass, grav_force | mass grid | particles | mask (grav wall) | finite | Jacobi Poisson (physics & tpt) | FFT for fidelity | optional FFT module | O(N log N) | PARTIAL |
| P04.06 | Gravity field calc | `gravity/Fft.cpp` | kernel convolution | grav_force | mass | — | boundary zero | — | Jacobi | FFT | document deviation | O(N) | PARTIAL |
| P04.07 | Gravity acceleration | `Simulation.cpp` | gravity_field() | vx,vy | particle_grav, newton_grav | motion | — | clamp | implemented | — | keep | O(N) | DONE |
| P04.08 | Gravity-particle interaction | `tick.rs` | n.pgrav_x/y | vx,vy | gravity field | motion | — | clamp | implemented | — | keep | O(N) | DONE |
| P04.09 | Gravity sources/sinks | `elements/GRVT.cpp`, `SING.cpp` | element-specific | — | — | Agent 3 | — | — | Agent 3 | — | expose API | O(N) | PARTIAL |
| P04.10 | Gravity blocking | `walls` GRAV wall | grav_mask | grav_mask | walls | walls | grav wall | — | tpt: grav_mask; physics: no mask | add mask to physics | add mask param | O(N) | PARTIAL |
| P04.11 | Gravity boundary conditions | `gravity/Fft.cpp` | zero at border | phi | — | — | border | — | Jacobi Dirichlet | — | keep | O(N) | DONE |

## P05 — Air convection

| ID | Mechanism | Ref source | Algorithm | State vars | Inputs/outputs | Coupled | Boundary | Stability | Current impl | Missing | Proposed Rust | Perf | Status |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| P05.01 | Disabled convection | settings | none | — | — | — | — | — | `ConvectionMode::None` (tpt) | physics has no convection | add module | O(1) | PARTIAL |
| P05.02 | Legacy convection | `Air.cpp` update_airh | weight*grad(hv) | vx,vy | hv | air, gravity | bounds | clamp | tpt implemented | physics missing | add convection.rs | O(N) | PARTIAL |
| P05.03 | Boussinesq convection | `Air.cpp` update_airh | weight=(hv-ambient)/10000 | vx,vy | hv, ambient | air, gravity | bounds | clamp | tpt implemented | physics missing | add convection.rs | O(N) | PARTIAL |
| P05.04 | Thermal buoyancy | `Air.cpp` | hot air rises | vy | temperature | air | — | clamp | gas::apply_buoyancy | — | keep | O(N) | DONE |
| P05.05 | Airfield coupling | `Air.cpp` | hv advected by vx,vy | hv | air field | air | walls | clamp | tpt implemented | physics missing | add ambient heat field | O(N) | PARTIAL |

## P06 — Heat transfer

| ID | Mechanism | Ref source | Algorithm | State vars | Inputs/outputs | Coupled | Boundary | Stability | Current impl | Missing | Proposed Rust | Perf | Status |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| P06.01 | Particle temperature | `Particle.h` | temp field | temperature | — | heat | — | clamp 0..4000 | implemented | — | keep | O(N) | DONE |
| P06.02 | Heat capacity | `ElementDefs.h` | density*specific_heat*volume | — | material | — | — | — | implemented | — | keep | O(N) | DONE |
| P06.03 | Thermal conductivity | `ElementDefs.h` | k_eff harmonic mean | — | material | — | — | — | implemented | — | keep | O(N) | DONE |
| P06.04 | Neighbor heat transfer | `Simulation.cpp` | Fourier conduction | temperature | k, dt | — | bounds | coeff clamp | `heat::conduct` | — | keep | O(N) | DONE |
| P06.05 | Heat transfer between materials | `Simulation.cpp` | per-pair exchange | temperature | materials | — | — | — | implemented | — | keep | O(N) | DONE |
| P06.06 | Thermal exchange with ambient air | `Air.cpp` update_airh | hv coupling | hv, temperature | ambient | air | walls | clamp | simple loop in WASM | proper coupling | add ambient exchange | O(N) | PARTIAL |
| P06.07 | Heat advection | `Air.cpp` | hv advected | hv | air field | air | walls | clamp | tpt implemented | physics missing | add module | O(N) | PARTIAL |
| P06.08 | Thermal insulation | `Air.cpp` | bmap_blockairh | bmap_blockairh | walls, particles | walls | — | — | tpt implemented | physics missing | add insulation | O(N) | PARTIAL |
| P06.09 | Heat sources/sinks | `elements/HEAT.cpp`, `COOL` | add temperature | temperature | tool | — | bounds | clamp | wasm heat_cell | physics op | keep | O(N) | DONE |
| P06.10 | Temperature clamping | `Simulation.cpp` | restrict_flt | temperature | — | — | — | 0..9999 | stability::sanitize | — | keep | O(N) | DONE |
| P06.11 | Stable integration | `heat.rs` | explicit Euler, coeff clamp | temperature | dt | — | — | coeff≤0.25 | implemented | sub-stepping | document | O(N) | DONE |
| P06.12 | Material-specific heat props | `ElementDefs.h` | specific_heat, conductivity | — | material | — | — | — | implemented | — | keep | O(N) | DONE |
| P06.13 | Energy conservation | `heat.rs` | symmetric exchange | temperature | — | — | — | — | implemented | duplicate transfer_energy | consolidate | O(N) | PARTIAL |

## P07 — Thermodynamic transitions

| ID | Mechanism | Ref source | Algorithm | State vars | Inputs/outputs | Coupled | Boundary | Stability | Current impl | Missing | Proposed Rust | Perf | Status |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| P07.01 | Melting | `Simulation.cpp` | temp >= melting_point | material, temp | material props | heat | — | clamp | heat::phase_transitions (generic) | material id change | add transition table hook | O(N) | PARTIAL |
| P07.02 | Solidification | `Simulation.cpp` | temp <= melting_point | material, temp | material props | heat | — | clamp | implemented | material id change | add hook | O(N) | PARTIAL |
| P07.03 | Evaporation | `Simulation.cpp` | temp >= boiling_point | material, temp | material props | heat | — | clamp | implemented (water/steam) | generic | add hook | O(N) | PARTIAL |
| P07.04 | Condensation | `Simulation.cpp` | temp <= boiling_point | material, temp | material props | heat | — | clamp | implemented (water/steam) | generic | add hook | O(N) | PARTIAL |
| P07.05 | Sublimation | `Simulation.cpp` | solid→gas | material, temp | material props | heat | — | clamp | not implemented | sublimation path | add hook | O(N) | MISSING |
| P07.06 | Deposition | `Simulation.cpp` | gas→solid | material, temp | material props | heat | — | clamp | not implemented | deposition path | add hook | O(N) | MISSING |
| P07.07 | Pressure-dependent transitions | `Simulation.cpp` | pressure thresholds | pressure, temp | — | air | — | — | not implemented | pressure coupling | add hooks | O(N) | MISSING |
| P07.08 | Temperature-dependent transitions | `Simulation.cpp` | temp thresholds | temp | material props | heat | — | clamp | implemented | — | keep | O(N) | DONE |
| P07.09 | Latent heat | `Simulation.cpp` | consume/release L during phase change | temperature | latent_heat | heat | — | clamp | not implemented | latent heat accounting | add latent_heat to props + consume | O(N) | MISSING |
| P07.10 | Energy transfer during phase change | `Simulation.cpp` | dE = m*L | temperature | — | heat | — | — | not implemented | — | add | O(N) | MISSING |
| P07.11 | Ambient temperature interactions | `Simulation.cpp` | ambient heat | temperature | ambient | air | — | clamp | simple loop | proper coupling | add | O(N) | PARTIAL |

## P08 — Pressure and force effects

| ID | Mechanism | Ref source | Algorithm | State vars | Inputs/outputs | Coupled | Boundary | Stability | Current impl | Missing | Proposed Rust | Perf | Status |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| P08.01 | Pressure forces | `Simulation.cpp` | -grad(p)/rho | vx,vy | pressure | fluid | bounds | clamp | `fluid::apply_pressure_gradient` | — | keep | O(N) | DONE |
| P08.02 | Local pressure gradients | `Simulation.cpp` | finite diff | vx,vy | pressure | — | bounds | — | implemented | — | keep | O(N) | DONE |
| P08.03 | Shock propagation | `Simulation.cpp` | pressure wave | pv | — | air | — | clamp | not implemented | shock model | add to air solver | O(N) | MISSING |
| P08.04 | Pressure-induced movement | `Simulation.cpp` | pressure force | vx,vy | pressure | fluid | — | clamp | implemented | — | keep | O(N) | DONE |
| P08.05 | Pressure-induced phase transitions | `Simulation.cpp` | pressure thresholds | material | pressure | chemistry | — | — | not implemented | hooks | add | O(N) | MISSING |
| P08.06 | Explosive pressure impulses | `elements/BANG.cpp` | add pressure | pv | element | air | bounds | clamp | not implemented | element API | expose add_pressure | O(N) | PARTIAL |
| P08.07 | Air expansion | `gas.rs` | pressure expansion | vy | pressure | air | — | clamp | `gas::apply_buoyancy` | — | keep | O(N) | DONE |
| P08.08 | Negative pressure/vacuum | `simtools/VAC.cpp` | add_pressure negative | pv | tool | air | bounds | clamp | wasm apply_tool | physics op | keep | O(N) | DONE |
| P08.09 | Force-field interactions | `elements/ACEL.cpp` | velocity change | vx,vy | element | — | — | clamp | not implemented | element API | expose | O(N) | MISSING |
| P08.10 | Gravitational acceleration fields | `elements/GRAV.cpp` | gravity well | vx,vy | element | gravity | — | clamp | not implemented | element API | expose | O(N) | MISSING |

## P09 — Walls and boundaries

| ID | Mechanism | Ref source | Algorithm | State vars | Inputs/outputs | Coupled | Boundary | Stability | Current impl | Missing | Proposed Rust | Perf | Status |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| P09.01 | Solid boundaries | `Simulation.cpp` | wall blocks | bmap | walls | walls | — | — | implemented | — | keep | O(N) | DONE |
| P09.02 | Void boundaries | settings | kill at edge | — | edge_mode | — | edge | — | clear_border | — | keep | O(N) | DONE |
| P09.03 | Wrapping boundaries | settings | swap edges | — | edge_mode | — | edge | — | wrap_border | — | keep | O(N) | DONE |
| P09.04 | Wall particle permeability | `Simulation.cpp` wall_at | per-wall allow | — | wall type | walls | — | — | wall_blocks/wall_kills | — | keep | O(N) | DONE |
| P09.05 | Wall air permeability | `Air.cpp` | blockair | bmap_blockair | walls | air | — | — | rebuild_blockair | per-wall permeability | add table | O(N) | PARTIAL |
| P09.06 | Wall heat interaction | `Air.cpp` | bmap_blockairh | bmap_blockairh | walls, particles | heat | — | — | tpt implemented | physics missing | add | O(N) | PARTIAL |
| P09.07 | Energy-particle permeability | `Simulation.cpp` | wall allows energy | — | wall type | walls | — | — | wall_blocks | — | keep | O(N) | DONE |
| P09.08 | Stasis/gravity-blocking | `walls` | freeze, grav_mask | emap, grav_mask | walls | — | — | — | tpt implemented | physics missing | add | O(N) | PARTIAL |

## P10 — Energy and radiation transport physics

| ID | Mechanism | Ref source | Algorithm | State vars | Inputs/outputs | Coupled | Boundary | Stability | Current impl | Missing | Proposed Rust | Perf | Status |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| P10.01 | Energy particle velocity | `elements/PHOT.cpp` | vx,vy | vx,vy | — | motion | — | clamp | tpt | physics missing | add energy.rs | O(N) | PARTIAL |
| P10.02 | Propagation | `Simulation.cpp` | move along velocity | x,y | can_move | — | bounds | — | tpt | physics missing | add | O(N) | PARTIAL |
| P10.03 | Reflection | `elements/PHOT.cpp` | bounce | vx,vy | walls | walls | — | — | tpt (can_move) | physics missing | add | O(N) | PARTIAL |
| P10.04 | Refraction | `elements/PHOT.cpp` | bend | vx,vy | material | — | — | — | not implemented | refraction | add | O(N) | MISSING |
| P10.05 | Absorption | `elements/PHOT.cpp` | kill + heat | temperature | material | heat | — | — | tpt | physics missing | add | O(N) | PARTIAL |
| P10.06 | Transmission | `elements/PHOT.cpp` | pass through | — | material | — | — | — | tpt (can_move=2) | physics missing | add | O(N) | PARTIAL |
| P10.07 | Scattering | `elements/PHOT.cpp` | random deflect | vx,vy | material | — | — | — | not implemented | scattering | add | O(N) | MISSING |
| P10.08 | Particle-matter interaction | `Simulation.cpp` | can_move table | — | material | chemistry | — | — | tpt | physics missing | add | O(N) | PARTIAL |
| P10.09 | Energy deposition | `elements/PHOT.cpp` | heat on absorb | temperature | — | heat | — | clamp | tpt | physics missing | add | O(N) | PARTIAL |
| P10.10 | Temperature changes | `elements/PHOT.cpp` | heat | temperature | — | heat | — | clamp | tpt | physics missing | add | O(N) | PARTIAL |
| P10.11 | Pressure effects | `elements/PHOT.cpp` | — | pressure | — | air | — | — | not implemented | — | add | O(N) | MISSING |

## P11 — Material stability

| ID | Mechanism | Ref source | Algorithm | State vars | Inputs/outputs | Coupled | Boundary | Stability | Current impl | Missing | Proposed Rust | Perf | Status |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| P11.01 | NaN/inf guard | `Simulation.cpp` | sanitize | all fields | — | — | — | — | `stability::sanitize` | — | keep | O(N) | DONE |
| P11.02 | Invalid temperature guard | `Simulation.cpp` | clamp 0..4000 | temperature | — | — | — | — | implemented | — | keep | O(N) | DONE |
| P11.03 | Invalid pressure guard | `Simulation.cpp` | clamp 0.. | pressure | — | — | — | — | implemented | — | keep | O(N) | DONE |
| P11.04 | Runaway velocity guard | `Simulation.cpp` | clamp ±50 | vx,vy | — | — | — | — | `clamp_velocities` | — | keep | O(N) | DONE |
| P11.05 | Invalid field values | `Simulation.cpp` | finite check | all | — | — | — | — | implemented | — | keep | O(N) | DONE |
| P11.06 | Invalid positions | `Simulation.cpp` | bounds check | x,y | — | — | — | — | in_bounds | — | keep | O(N) | DONE |
| P11.07 | Numerical overflow | `Simulation.cpp` | clamp | all | — | — | — | — | implemented | — | keep | O(N) | DONE |
| P11.08 | Unstable propagation | `heat.rs` | coeff clamp | temperature | — | — | — | — | implemented | — | keep | O(N) | DONE |
| P11.09 | Frozen regions | `walls` stasis | skip | — | stasis wall | walls | — | — | tpt | physics missing | add | O(N) | PARTIAL |
| P11.10 | Tick-order corruption | `Simulation.cpp` | UPDATED flag | flags | — | — | — | — | implemented | — | keep | O(N) | DONE |

## P12 — Simulation tools

| ID | Mechanism | Ref source | Algorithm | State vars | Inputs/outputs | Coupled | Boundary | Stability | Current impl | Missing | Proposed Rust | Perf | Status |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| P12.01 | AIR tool | `simtools/AIR.cpp` | add velocity/pressure | pv,vx,vy | tool | air | bounds | clamp | wasm apply_tool kind 2/3/8 | physics op | keep | O(N) | DONE |
| P12.02 | AMBM tool | `simtools/AMBM.cpp` | decrease ambient temp | ambient | tool | air | — | clamp | wasm apply_tool kind 9 | physics op | keep | O(N) | DONE |
| P12.03 | AMBP tool | `simtools/AMBP.cpp` | increase ambient temp | ambient | tool | air | — | clamp | wasm apply_tool kind 10 | physics op | keep | O(N) | DONE |
| P12.04 | COOL tool | `simtools/COOL.cpp` | decrease temp | temperature | tool | heat | bounds | clamp | wasm apply_tool kind 1 | physics op | keep | O(N) | DONE |
| P12.05 | CYCL tool | `simtools/CYCL.cpp` | swirl air | vx,vy | tool | air | bounds | clamp | wasm apply_tool kind 7 | physics op | keep | O(N) | DONE |
| P12.06 | HEAT tool | `simtools/HEAT.cpp` | increase temp | temperature | tool | heat | bounds | clamp | wasm apply_tool kind 0 | physics op | keep | O(N) | DONE |
| P12.07 | MIX tool | `simtools/MIX.cpp` | mix particles | vx,vy | tool | motion | bounds | — | wasm apply_tool kind 6 | physics op | keep | O(N) | DONE |
| P12.08 | NGRV tool | `simtools/NGRV.cpp` | negative gravity well | vx,vy | tool | gravity | bounds | clamp | wasm apply_tool kind 5 | physics op | keep | O(N) | DONE |
| P12.09 | PGRV tool | `simtools/PGRV.cpp` | positive gravity well | vx,vy | tool | gravity | bounds | clamp | wasm apply_tool kind 4 | physics op | keep | O(N) | DONE |
| P12.10 | VAC tool | `simtools/VAC.cpp` | decrease pressure | pv | tool | air | bounds | clamp | wasm apply_tool kind 3 | physics op | keep | O(N) | DONE |
| P12.11 | WIND tool | `simtools/WIND.cpp` | air movement | vx,vy | tool | air | bounds | clamp | wasm apply_tool kind 8 | physics op | keep | O(N) | DONE |

---

## Additional mechanisms identified

| ID | Mechanism | Ref source | Notes | Status |
|---|---|---|---|---|
| A.01 | Ambient heat field (hv) | `Air.cpp` update_airh | separate field for ambient temperature transport | PARTIAL (tpt only) |
| A.02 | Vorticity confinement | `Air.cpp` | vorticity_coeff | DONE (tpt) |
| A.03 | Edge pressure/velocity settings | `SimulationSettings.h` | edge_pressure, edge_velocity_x/y | PARTIAL (tpt only) |
| A.04 | Water equal test | `Simulation.cpp` | water_equal_test | MISSING |
| A.05 | GoL simulation | `gol.rs` | Game of Life rules | DONE (tpt) |
| A.06 | EMP decoration | `Simulation.cpp` | emp_decor | DONE (tpt) |
| A.07 | Particle life decay | `tick.rs` recalc_and_life | life decrement/kill | DONE (tpt) |
| A.08 | Spark/electricity transport | `SPRK.cpp` | step_spark | DONE (wasm) |
| A.09 | Glow buffer | `Simulation.cpp` | rendering | DONE (tpt) |
| A.10 | Colour buffer | `Simulation.cpp` | rendering | DONE (tpt) |

---

## Integration contracts required (for Agent 1)

- `apply_external_force(cells, x, y, fx, fy)` — expose for elements (ACEL, GRAV, etc.)
- `read_pressure(x, y) -> f32`, `read_air_velocity(x, y) -> (f32, f32)` — for elements/tools
- `apply_air_impulse(x, y, dvx, dvy, dp)` — for fans, explosions
- `read_gravity(x, y) -> (f32, f32)` — for particles
- `apply_thermal_energy(x, y, dE)` — for heat sources
- `read_material_density(id)`, `read_material_conductivity(id)` — for heat/pressure
- `request_phase_transition(x, y, target_material)` — for chemistry
- `read/modify energy-particle motion` — for radiation
- `wall permeability queries` — for air/heat/particles

## Performance notes

- Air solver: O(cx*cy) per tick, cx=width/4. Dominant cost for large worlds.
- Gravity (Jacobi): O(cx*cy*iterations). 24 iterations. Consider FFT for fidelity but Jacobi is real-time safe.
- Heat conduction: O(w*h) per tick.
- Movement CA: O(w*h) per tick.
- Active-chunk masking reduces work in sparse worlds.

## Remaining reference differences (summary)

1. Newtonian gravity uses Jacobi Poisson instead of FFT convolution (documented deviation).
2. Phase transitions mostly clamp temperature; latent heat not fully accounted; material id changes only for water/ice/steam.
3. Elasticity/restitution not implemented.
4. Liquid equalization (water_equal_test) not implemented.
5. Shock propagation not implemented.
6. Refraction/scattering for energy particles not implemented.
7. Ambient heat field (hv) and convection only in tpt engine, not in cell-based physics crate.
8. Per-wall air/heat permeability not fully implemented in physics crate.
9. AMBM/AMBP tools not implemented.
10. Sub-pixel particle integration not in cell-based engine (CA-based).
