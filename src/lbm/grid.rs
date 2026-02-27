//! # Lattice Grid Definitions
//!
//! This module establishes the layout of the Lattice Boltzmann domain. By strictly
//! adhering to Data-Oriented Design (DoD), we enforce linear, contiguous chunks
//! of memory rather than scattered pointers (Object-Oriented/AoS structures).
//! We use `ndarray` as it maps perfectly to standard BLAS/Linear configurations.

use super::constants::Q;
use ndarray::{Array2, Array3};

/// Represents a Struct of Arrays (SoA) layout for our 2D grid of properties.
/// We utilize `ndarray` arrays explicitly as requested by the architecture. We track macroscopic
/// values (density, x-velocity, y-velocity) and microscopic values (f).
///
/// **Why SoA over AoS?**
/// If we grouped `struct Cell { f: [f64; 9], rho: f64, ux: f64, uy: f64 }` into a large
/// array, fetching `rho` across an entire row for rendering would pull unwanted `f` elements
/// into the CPU cache line (Cache Thrashing). Keeping them separate allows our computational
/// loops to pull exactly what they need linearly.
pub struct LatticeGrid {
    pub width: usize,
    pub height: usize,

    /// Microscopic Distribution Functions ($f_i$).
    /// A 3D array: [X_COORD, Y_COORD, Q_INDEX].
    /// Represents the probability of particles existing with velocity $c_i$ at $(x,y)$.
    pub f: Array3<f64>,

    /// Microscopic Distribution Functions for the NEXT step.
    /// This resolves race conditions during the "streaming" stage. Reading from `f` and
    /// simultaneously writing back into `f` adjacent cells would scramble states iteratively.
    pub f_new: Array3<f64>,

    /// Macroscopic density ($\rho$).
    pub rho: Array2<f64>,

    /// Macroscopic velocity X ($u_x$).
    pub ux: Array2<f64>,

    /// Macroscopic velocity Y ($u_y$).
    pub uy: Array2<f64>,

    /// Passive scalar field (Dye) advecting alongside the velocity field.
    pub dye: Array2<f64>,

    /// Solid boundary geometry flags. `true` = wall, `false` = fluid.
    pub solid: Array2<bool>,
}

impl LatticeGrid {
    /// Zero-allocation constructor: pre-allocates all necessary arrays on startup.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            f: Array3::zeros((width, height, Q)),
            f_new: Array3::zeros((width, height, Q)),
            rho: Array2::ones((width, height)),
            ux: Array2::zeros((width, height)),
            uy: Array2::zeros((width, height)),
            dye: Array2::zeros((width, height)),
            solid: Array2::from_elem((width, height), false),
        }
    }
}
