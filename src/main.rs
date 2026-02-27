//! # navier-tui: A Terminal-based Lattice Boltzmann Fluid Dynamics Simulator
//!
//! This is the primary entry point for the simulation.
//! The goal of this application is twofold:
//! 1. Simulate computationally demanding fluid dynamics (D2Q9 LBM) on the CPU.
//! 2. Display the simulated macroscopic variables intuitively in the terminal using `ratatui`.
//!
//! By strictly following Data-Oriented Design (DoD) principles and Struct-of-Arrays (SoA) layout
//! inside the `LbmEngine`, we decouple the computationally heavy physics from this UI rendering layer.

pub mod lbm;
pub mod ui;

use lbm::LbmEngine;
use ui::run_app;

fn main() -> Result<(), std::io::Error> {
    // 1. Detect the current terminal dimensions so the fluid grid fills the
    //    entire window from the very first frame. We subtract 2 on each axis
    //    to account for the Ratatui border widget.
    let (term_w, term_h) = crossterm::terminal::size()?;
    let grid_w = (term_w.saturating_sub(2) as usize).max(10);
    let grid_h = (term_h.saturating_sub(2) as usize).max(10);

    // 2. The relaxation time `tau = 0.6` controls the kinematic viscosity.
    //    Viscosity = (tau - 0.5) / 3. At tau = 0.6, viscosity ≈ 0.033, producing
    //    a moderately viscous fluid that is numerically stable yet still shows
    //    interesting vortex phenomena at reasonable flow speeds.
    let engine = LbmEngine::new(grid_w, grid_h, 0.6);

    // 3. Hand off to the decoupled render + physics loop.
    run_app(engine)?;

    Ok(())
}
