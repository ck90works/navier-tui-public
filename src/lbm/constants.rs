//! # Physical and Lattice Constants for the D2Q9 Model
//!
//! This module defines the core architectural constants of a D2Q9 Lattice Boltzmann Method simulation.
//! Instead of calculating these values dynamically during the simulation, which would degrade the
//! cache-locality and performance of the tight physics loop, they are statically defined here.
//!
//! ## D2Q9 Model
//! "D2Q9" defines a 2-dimensional lattice containing 9 discrete microscopic velocities per cell.
//! These velocities represent particles resting (0), moving ordinally (1-4), or moving
//! diagonally (5-8).

pub const Q: usize = 9;

/// The discrete velocities ($\vec{e}_i$) representing the direction vector $c_i$.
/// In a D2Q9 system, we move by integer lattice coordinates:
/// 0: Rest (0, 0)
/// 1-4: Ordinal directions (E, N, W, S) -> (1,0), (0,1), (-1,0), (0,-1)
/// 5-8: Diagonal directions (NE, NW, SW, SE) -> (1,1), (-1,1), (-1,-1), (1,-1)
pub const C_X: [i32; Q] = [0, 1, 0, -1, 0, 1, -1, -1, 1];
pub const C_Y: [i32; Q] = [0, 0, 1, 0, -1, 1, 1, -1, -1];

/// The lattice equilibrium probability weights ($w_i$).
/// These weights are derived via Hermite polynomial expansion of the Maxwell-Boltzmann
/// distribution. They determine how the mass distributes symmetrically in local thermal equilibrium.
pub const W: [f64; Q] = [
    4.0 / 9.0, // Rest particle
    1.0 / 9.0,
    1.0 / 9.0,
    1.0 / 9.0,
    1.0 / 9.0, // Ordinal particles
    1.0 / 36.0,
    1.0 / 36.0,
    1.0 / 36.0,
    1.0 / 36.0, // Diagonal particles
];

/// The opposite lattice direction index.
/// This array is crucial for the "bounce-back" boundary condition. When a particle hits a solid
/// wall, it reverses its path identically. `OPPOSITE[i]` quickly gives the opposing direction.
pub const OPPOSITE: [usize; Q] = [0, 3, 4, 1, 2, 7, 8, 5, 6];

/// Speed of sound squared in LBM units. Usually $c_s^2 = \frac{1}{3}$.
/// This avoids expensive divisions during the macroscopic equilibrium checks.
pub const _CS2: f64 = 1.0 / 3.0; // Prefixed with _ as currently implicitly used via fractions
