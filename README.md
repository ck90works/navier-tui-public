# navier-tui

**navier-tui** is a high-performance, real-time 2D fluid dynamics simulator built for the terminal. Instead of solving macroscopic Navier-Stokes equations, it utilizes the Lattice Boltzmann Method (LBM) to model fluid at a mesoscopic level. Users can interact with the simulation in real-time, placing solid boundaries (obstacles) and injecting dye to visualize complex flow phenomena directly in their standard terminal emulator.

## Motivation & Origin

This project was born out of a deep childhood curiosity for physics and mathematical problems. While my career grew into product development, software engineering, and a deep-dive into DevOps and Cloud architecture, my true hobby has always been **connecting new dots**. I love using my creative mind to merge seemingly opposite worlds—taking highly abstract concepts like computational fluid dynamics and rendering them inside the raw, text-based terminal environment. Programming is a kind of art, and building something this challenging leads to valuable insight into parallel computing and memory performance.

**Transparency Note:** This project was developed with the extensive help of AI, specifically **Gemini 3.1 Pro**. Collaborating with Gemini enabled the rapid translation of complex physics formulas into highly optimized, lock-free Rust code.

## System Architecture

* **Physics Engine (LBM):** Utilizes the D2Q9 (2 dimensions, 9 discrete velocities) lattice model. 
* **State Management:** Employs a double-buffering technique for the distribution functions ($f_{old}$ and $f_{new}$) to ensure mathematical correctness during the streaming phase. Data is stored entirely in Data-Oriented Design (DoD) structures using `ndarray`.
* **Rendering Layer:** A decoupled TUI loop that reads the macroscopic grid and maps velocity magnitudes to ASCII densities with a colorblind-safe dark blue→cyan→yellow heatmap (dark blue = slow, yellow = fast).
* **Input Handling:** An asynchronous event listener that captures mouse clicks and key presses to dynamically mutate the grid state (e.g., painting obstacles).

## Dependencies

* `rust` = "1.93.1" (Rust Edition 2024)
* `ratatui` (For terminal UI layout and widget rendering)
* `crossterm` (For backend terminal manipulation and event handling)
* `rayon` (For parallelizing the LBM lattice updates)
* `ndarray` (For efficient multi-dimensional array manipulation and cache locality)

## Getting Started

To run the simulation:

```bash
cargo run --release
```

**Controls:**
- **Mouse Left Click & Drag:** Draw solid boundaries inside the fluid field.
- **Mouse Right Click & Drag:** Erase solid boundaries.
- **Q or ESC:** Quit the simulation.

**Color Legend:** The heatmap visualizes **velocity magnitude** using a colorblind-friendly palette — 🟦 dark blue = slow/stagnant, 🟦 cyan = moderate, 🟨 yellow = fast flow. Narrow gaps between obstacles appear yellow because the fluid accelerates through constrictions (Venturi effect), which is physically correct.

You can build a standalone executable under `.\target\release\` with:

```bash
cargo build --release
```
