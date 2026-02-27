//! # The Lattice Boltzmann Physics Engine
//!
//! Coordinates the transition of the `LatticeGrid` states across simulation ticks.

pub mod constants;
pub mod grid;

use constants::*;
use grid::LatticeGrid;

/// Computes the discrete equilibrium distribution function ($f_i^{eq}$).
/// This is the backbone of the BGK collision operator. Macroscopic properties
/// are plugged into a Maxwellian equilibrium derivation.
#[inline(always)]
pub fn equilibrium(rho: f64, ux: f64, uy: f64, i: usize) -> f64 {
    let cu = (C_X[i] as f64) * ux + (C_Y[i] as f64) * uy;
    let u2 = ux * ux + uy * uy;
    W[i] * rho * (1.0 + 3.0 * cu + 4.5 * cu * cu - 1.5 * u2)
}

/// A centralized state manager for our Lattice Simulation.
pub struct LbmEngine {
    pub grid: LatticeGrid,
    /// Kinematic viscosity relaxation time.
    pub tau: f64,
}

impl LbmEngine {
    pub fn new(width: usize, height: usize, tau: f64) -> Self {
        let mut grid = LatticeGrid::new(width, height);

        // Initialize equilibrium weights into f
        for x in 0..width {
            for y in 0..height {
                for i in 0..Q {
                    let eq = equilibrium(1.0, 0.0, 0.0, i);
                    grid.f[[x, y, i]] = eq;
                    grid.f_new[[x, y, i]] = eq;
                }
            }
        }

        Self { grid, tau }
    }
    /// The core physical step. Contains zero dynamic memory allocations.
    ///
    /// The algorithm is split into two passes:
    /// 1. **Collision**: Compute macroscopic density/velocity from distributions,
    ///    then relax distributions toward local equilibrium (BGK operator).
    ///    Solid nodes are **skipped** — their distributions are managed exclusively
    ///    by the bounce-back rule in the streaming pass.
    /// 2. **Streaming (Pull Method)**: Each fluid node pulls post-collision
    ///    distributions from upstream neighbors. Solid nodes perform a simple
    ///    bounce-back reversal so that mass is correctly reflected on the next tick.
    pub fn tick(&mut self) {
        let width = self.grid.width;
        let height = self.grid.height;
        let omega = 1.0 / self.tau;

        let LatticeGrid { f, f_new, rho, ux, uy, solid, .. } = &mut self.grid;

        use ndarray::{Zip, Axis};

        // --- 1. COLLISION PASS & MACROSCOPIC UPDATES ---
        // We zip the `solid` flag alongside the distribution lanes so that solid
        // cells are cleanly skipped. This prevents the BGK relaxation from
        // corrupting mass stored inside wall nodes.
        Zip::from(f.lanes_mut(Axis(2)))
            .and(&mut *rho)
            .and(&mut *ux)
            .and(&mut *uy)
            .and(&*solid)
            .par_for_each(|mut f_q, rho_xy, ux_xy, uy_xy, &is_solid| {
                // Solid cells do not participate in collision.
                if is_solid {
                    return;
                }

                let mut local_rho = 0.0;
                let mut local_ux = 0.0;
                let mut local_uy = 0.0;

                for i in 0..Q {
                    let fi = f_q[i];
                    local_rho += fi;
                    local_ux += fi * C_X[i] as f64;
                    local_uy += fi * C_Y[i] as f64;
                }

                if local_rho > 0.0 {
                    local_ux /= local_rho;
                    local_uy /= local_rho;
                }

                *rho_xy = local_rho;
                *ux_xy = local_ux;
                *uy_xy = local_uy;

                for i in 0..Q {
                    let f_eq = equilibrium(local_rho, local_ux, local_uy, i);
                    f_q[i] = f_q[i] * (1.0 - omega) + f_eq * omega;
                }
            });

        // --- 2. STREAMING PASS (PULL METHOD) ---
        // Fluid nodes pull post-collision distributions from their upstream neighbors.
        // Solid nodes perform full-way bounce-back: each direction's distribution is
        // simply reflected to the opposite direction. This ensures no stale data
        // remains in `f_new` for solid cells after the swap.
        Zip::indexed(f_new.lanes_mut(Axis(2)))
            .and(&*solid)
            .par_for_each(|(x, y), mut f_new_q, &is_solid| {
                if is_solid {
                    // Full-way bounce-back for solid nodes: reverse every direction.
                    for i in 0..Q {
                        f_new_q[OPPOSITE[i]] = f[[x, y, i]];
                    }
                } else {
                    for i in 0..Q {
                        let src_x = (x as i32 - C_X[i]).rem_euclid(width as i32) as usize;
                        let src_y = (y as i32 - C_Y[i]).rem_euclid(height as i32) as usize;

                        if solid[[src_x, src_y]] {
                            // The upstream neighbor is a wall — the particle bounced
                            // back from there, so we pull the opposite direction
                            // from our own cell's post-collision state.
                            let opp = OPPOSITE[i];
                            f_new_q[i] = f[[x, y, opp]];
                        } else {
                            f_new_q[i] = f[[src_x, src_y, i]];
                        }
                    }
                }
            });

        // Pointer swap: no heap allocation, just swaps internal buffer references.
        std::mem::swap(f, f_new);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests conservation of mass.
    /// LBM is mathematically guaranteed to preserve exact sums of the probability distributions
    /// over non-open boundaries over any discrete tick interval.
    #[test]
    fn test_mass_conservation() {
        let width = 20;
        let height = 20;
        let mut engine = LbmEngine::new(width, height, 0.6);

        // Introduce artificial velocity vectors to prove robustness over streaming
        engine.grid.f[[10, 10, 1]] += 0.5;
        engine.grid.f[[10, 10, 5]] += 0.3;

        // Establish known initial chaotic mass
        let initial_mass: f64 = engine.grid.f.sum();

        engine.tick();

        let final_mass: f64 = engine.grid.f.sum();

        // Compare using epsilon for precision errors
        let epsilon = 1e-10;
        assert!(
            (initial_mass - final_mass).abs() < epsilon,
            "Mass violated! Started with {}, got {} after 1 loop tick.",
            initial_mass,
            final_mass
        );
    }
}
