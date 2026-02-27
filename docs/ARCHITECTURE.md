# System Architecture: `navier-tui`

## Overview
`navier-tui` is a high-performance terminal UI (TUI) application designed for simulating real-time 2D fluid dynamics via the Lattice Boltzmann Method (LBM).

## Core Principles
1. **Data-Oriented Design**: We avoid Arrays of Structs (AoS) and opt for a flat structure utilizing `ndarray` to pre-allocate state. This guarantees cache locality during the linear traversal inside the physics loop.
2. **Double Buffering**: Operations read from a "read" grid ($f_{old}$) and write to a "write" grid ($f_{new}$), swapping pointers/references at the end of each tick to remain strictly immutable and mathematically sound during streaming.
3. **Decoupled Architecture**: 
   - **Physics Loop**: Operates deterministically and independently of frame rendering. 
   - **Render Loop**: Samples macroscopic states to render ASCII representations within `ratatui`.

## Component Breakdown

### 1. The Lattice Engine (Math & Physics)
The core simulation runs the `D2Q9` lattice model. 
- **Discrete Velocities ($\vec{e}_i$)**: Represents the 9 directional vectors connecting grid nodes.
- **Weights ($w_i$)**: Defines the equilibrium probability constants.
- **Relaxation ($\tau$)**: Controls the fluid's kinematic viscosity. Update rule uses the BGK relaxation model.
- **Streaming**: Propagates the cell distribution to adjacent nodes based on discrete velocities.

### 2. Rendering & Input (TUI)
- **Crossterm**: Intercepts keyboard and mouse signals (for drawing barriers or dye). Switches the terminal emulator to raw mode.
- **Ratatui**: Renders velocity magnitude as a colorblind-friendly dark blue→cyan→yellow heatmap (dark blue = slow, yellow = fast) over ASCII density characters. Narrow constrictions appear yellow due to the Venturi effect.

### 3. Concurrency
Utilizes the `rayon` crate for row-wise parallelism of grid updates to maximize core utilization.
