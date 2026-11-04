//! Fixed-timestep tick scheduler (A2).
//!
//! Implements the phase order from `docs/contracts/simulation-tick.md`:
//! input → chemistry → physics forces → physics collisions → movement →
//! thermal → finalize → export. Each phase runs registered
//! [`SimulationModule`](realmatter_core::traits::SimulationModule)s in
//! registration order; phase boundaries are enforced by the scheduler.
//! Leftover time carries over via accumulator; max 4 substeps per frame.

use realmatter_core::cell::Cell;
use realmatter_core::error::Result;
use realmatter_core::traits::SimulationModule;
use realmatter_core::units::Time;
use realmatter_core::world_config::WorldConfig;

/// Tick phases in deterministic execution order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Phase {
    Input,
    Chemistry,
    PhysicsForces,
    PhysicsCollisions,
    Movement,
    Thermal,
    Finalize,
    Export,
}

impl Phase {
    pub const ORDER: [Phase; 8] = [
        Phase::Input,
        Phase::Chemistry,
        Phase::PhysicsForces,
        Phase::PhysicsCollisions,
        Phase::Movement,
        Phase::Thermal,
        Phase::Finalize,
        Phase::Export,
    ];
}

/// Per-tick statistics for diagnostics.
#[derive(Clone, Copy, Debug, Default)]
pub struct TickStats {
    pub tick: u64,
    pub dt: Time,
    pub substeps: u32,
}

/// A module registered for one phase.
struct Entry {
    phase: Phase,
    module: Box<dyn SimulationModule>,
}

/// Fixed-timestep scheduler over a cell slice.
pub struct TickScheduler {
    config: WorldConfig,
    entries: Vec<Entry>,
    accumulator: f64,
    tick: u64,
}

impl TickScheduler {
    pub fn new(config: WorldConfig) -> Result<Self> {
        config.validate()?;
        Ok(Self {
            config,
            entries: Vec::new(),
            accumulator: 0.0,
            tick: 0,
        })
    }

    /// Register a module for a phase. Order within a phase = registration order.
    pub fn register(&mut self, phase: Phase, module: Box<dyn SimulationModule>) {
        self.entries.push(Entry { phase, module });
        // Keep registration order stable within a phase (stable sort).
        self.entries.sort_by_key(|e| e.phase);
    }

    /// Advance by `elapsed` seconds; runs up to 4 fixed steps.
    /// `movement` and `finalize` are callbacks owned by the world grid so the
    /// scheduler never holds cell storage itself.
    pub fn advance(
        &mut self,
        elapsed: Time,
        cells: &mut [Cell],
        mut movement: impl FnMut(&mut [Cell], Time) -> Result<()>,
        mut finalize: impl FnMut(&mut [Cell]) -> Result<()>,
        mut export: impl FnMut(u64, &[Cell]) -> Result<()>,
    ) -> Result<TickStats> {
        self.accumulator += elapsed;
        let dt = self.config.fixed_dt;
        let mut substeps = 0u32;
        while self.accumulator >= dt && substeps < 4 {
            self.accumulator -= dt;
            self.step_once(dt, cells, &mut movement, &mut finalize, &mut export)?;
            substeps += 1;
        }
        if substeps == 4 && self.accumulator >= dt {
            self.accumulator = 0.0; // spiral-of-death guard
        }
        Ok(TickStats { tick: self.tick, dt, substeps })
    }

    fn step_once(
        &mut self,
        dt: Time,
        cells: &mut [Cell],
        movement: &mut impl FnMut(&mut [Cell], Time) -> Result<()>,
        finalize: &mut impl FnMut(&mut [Cell]) -> Result<()>,
        export: &mut impl FnMut(u64, &[Cell]) -> Result<()>,
    ) -> Result<()> {
        for phase in Phase::ORDER {
            match phase {
                Phase::Movement => movement(cells, dt)?,
                Phase::Finalize => finalize(cells)?,
                Phase::Export => export(self.tick, cells)?,
                _ => {
                    for entry in self.entries.iter_mut().filter(|e| e.phase == phase) {
                        entry.module.step(dt)?;
                    }
                }
            }
        }
        self.tick += 1;
        Ok(())
    }

    pub fn tick(&self) -> u64 {
        self.tick
    }
}
