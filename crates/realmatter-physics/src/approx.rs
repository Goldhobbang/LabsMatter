//! Approximation methods and their limitations (documentation module).
//!
//! This module documents the numerical approximations used by the physics
//! crate. It contains no runtime logic; constants and notes only.

/// Approximation notes.
pub mod notes {
    /// Heat conduction uses explicit Euler finite differences with a 4-neighbor
    /// Laplacian. Stability requires dt <= dx^2/(4*D). Diffusion coefficient
    /// is clamped to 0.24 for safety. Accuracy: first-order in time, second-order
    /// in space. Limitation: not suitable for very high conductivity materials
    /// at large dt; may need sub-stepping.
    pub const HEAT_CONDUCTION: &str = "explicit Euler, 4-neighbor Laplacian, coeff clamped 0.24";

    /// Ideal gas law P = rho*R_specific*T with R_specific approximated from
    /// density. Limitation: not a real EOS; ignores humidity, composition.
    pub const GAS_PRESSURE: &str = "ideal gas law, R_specific approximated";

    /// Gas diffusion uses temperature diffusion + pressure-driven cell swaps.
    /// Limitation: matter diffusion is discrete cell swaps, not continuous.
    pub const GAS_DIFFUSION: &str = "discrete cell swaps + temperature diffusion";

    /// Fluid pressure gradient uses finite differences on cell.pressure.
    /// Limitation: no full incompressibility solve (no Poisson); approximate.
    pub const FLUID_PRESSURE: &str = "finite-difference gradient, no Poisson solve";

    /// Buoyancy uses density difference vs reference (air) density.
    /// Limitation: Boussinesq approximation; not valid for large density ratios.
    pub const BUOYANCY: &str = "Boussinesq, reference density 1.225 kg/m3";

    /// Phase transitions clamp temperature at transition point but do not
    /// change material id (no latent heat consumption beyond clamping).
    /// Limitation: latent heat not fully accounted; material id unchanged.
    pub const PHASE_TRANSITION: &str = "temperature clamping only; no material id change";

    /// Reactions use simplified Arrhenius gate and fixed heat release.
    /// Limitation: not chemically accurate; no stoichiometry balancing.
    pub const REACTIONS: &str = "simplified Arrhenius gate, fixed heat release";

    /// Surface tension is a velocity smoothing at boundaries.
    /// Limitation: not a real surface tension model.
    pub const SURFACE_TENSION: &str = "velocity averaging at material boundaries";
}

/// Numerical error budget summary.
pub mod error_budgets {
    /// Heat conduction relative error target.
    pub const HEAT_REL_ERROR: f32 = 1e-2;
    /// Pressure relative error target.
    pub const PRESSURE_REL_ERROR: f32 = 1e-2;
    /// Velocity integration error target per step.
    pub const VELOCITY_ERROR: f32 = 1e-3;
}
