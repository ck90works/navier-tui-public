//! # The Ratatui UI Layer
//!
//! Provides rendering facilities to decouple physics from terminal TUI mapping.
//! We map the macroscopic fluid velocity and density to terminal colors and ASCII.
//!
//! **Design Note:** The physics loop is intentionally decoupled from the rendering
//! loop. We run multiple physics ticks per rendered frame to maintain fluid
//! responsiveness while keeping the terminal frame rate smooth (~30 FPS).

use ratatui::{
    backend::CrosstermBackend,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode, MouseEventKind, MouseButton},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io::{self, stdout},
    time::{Duration, Instant},
};

use crate::lbm::LbmEngine;

// ---------------------------------------------------------------------------
// Rendering configuration constants
// ---------------------------------------------------------------------------

/// Target interval between rendered frames (~30 FPS).
/// We render at 30 FPS but run physics at a higher rate for smooth motion.
const RENDER_INTERVAL: Duration = Duration::from_millis(33);

/// How many physics ticks to compute between each rendered frame.
/// More ticks = faster visible fluid motion without increasing GPU/terminal load.
const PHYSICS_TICKS_PER_FRAME: u32 = 4;

/// Inlet velocity (Mach number in lattice units). 0.08 is a safe value
/// that stays well below the LBM compressibility limit (~0.1–0.15).
const INLET_VELOCITY: f64 = 0.08;

// ---------------------------------------------------------------------------
// Velocity → character mapping
// ---------------------------------------------------------------------------

/// Maps a fluid velocity magnitude to an ASCII density character.
/// The thresholds are chosen to produce a visually pleasing gradient from
/// still fluid (blank) to high-speed flow (dense block).
fn map_velocity_to_char(ux: f64, uy: f64) -> char {
    let mag_sq = ux * ux + uy * uy;
    if mag_sq < 0.0005 {
        ' '
    } else if mag_sq < 0.002 {
        '·'
    } else if mag_sq < 0.005 {
        '.'
    } else if mag_sq < 0.01 {
        ':'
    } else if mag_sq < 0.02 {
        '~'
    } else if mag_sq < 0.04 {
        '='
    } else if mag_sq < 0.07 {
        '+'
    } else if mag_sq < 0.12 {
        '*'
    } else {
        '#'
    }
}

// ---------------------------------------------------------------------------
// Inlet forcing
// ---------------------------------------------------------------------------

/// Resets the left-most two columns to a steady rightward-flowing equilibrium.
/// This acts as a "wind tunnel inlet": fluid is continuously injected from the
/// left edge, driving the overall flow field.
fn apply_inlet_forcing(engine: &mut LbmEngine) {
    let height = engine.grid.height;
    // Skip top and bottom rows (they act as implicit wall boundaries)
    for y in 1..height.saturating_sub(1) {
        for col in 0..2 {
            for i in 0..9 {
                engine.grid.f[[col, y, i]] =
                    crate::lbm::equilibrium(1.0, INLET_VELOCITY, 0.0, i);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Main application loop
// ---------------------------------------------------------------------------

/// A decoupled TUI loop that runs the physics engine and renders it via Ratatui.
///
/// **Resize handling:** When a `Resize` event is detected, the engine is rebuilt
/// at the new terminal dimensions so the fluid always fills the full window.
pub fn run_app(mut engine: LbmEngine) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut last_render = Instant::now();
    let mut is_drawing_wall = false;
    let mut is_erasing_wall = false;

    loop {
        // ---- Render frame ----
        terminal.draw(|frame| {
            let size = frame.area();
            // Inner area accounts for the 1-cell border on each side
            let inner_w = (size.width.saturating_sub(2)) as usize;
            let inner_h = (size.height.saturating_sub(2)) as usize;

            let mut text_lines = Vec::with_capacity(inner_h);

            for y in 0..inner_h.min(engine.grid.height) {
                let mut spans = Vec::with_capacity(inner_w);
                for x in 0..inner_w.min(engine.grid.width) {
                    if engine.grid.solid[[x, y]] {
                        spans.push(Span::styled("█", Style::default().fg(Color::White)));
                    } else {
                        let vel_ux = engine.grid.ux[[x, y]];
                        let vel_uy = engine.grid.uy[[x, y]];
                        let ch = map_velocity_to_char(vel_ux, vel_uy);

                        // Color: colorblind-friendly viridis-inspired gradient.
                        // dark blue (slow) → cyan (moderate) → yellow (fast).
                        // This palette is safe for deuteranopia, protanopia, and tritanopia.
                        let speed = (vel_ux * vel_ux + vel_uy * vel_uy).sqrt();
                        let t = (speed * 12.0).min(1.0);

                        // Piecewise: dark blue(0) → cyan(0.5) → yellow(1.0)
                        let (r, g, b) = if t < 0.5 {
                            // Dark blue → Cyan
                            let s = t * 2.0;
                            ((30.0 * s) as u8, (50.0 + 170.0 * s) as u8, (100.0 + 155.0 * s) as u8)
                        } else {
                            // Cyan → Yellow
                            let s = (t - 0.5) * 2.0;
                            ((30.0 + 225.0 * s) as u8, (220.0 + 35.0 * s) as u8, (255.0 - 230.0 * s) as u8)
                        };

                        spans.push(Span::styled(
                            ch.to_string(),
                            Style::default().fg(Color::Rgb(r, g, b)),
                        ));
                    }
                }
                text_lines.push(Line::from(spans));
            }

            let paragraph = Paragraph::new(text_lines).block(
                Block::default()
                    .title(" Navier-TUI │ LBM Fluid Simulation │ Q to quit ")
                    .borders(Borders::ALL),
            );
            frame.render_widget(paragraph, size);
        })?;

        // ---- Process events (non-blocking) ----
        // We poll for a short duration so that we don't block the physics loop.
        if event::poll(Duration::from_millis(1))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                        break;
                    }
                }
                Event::Mouse(me) => {
                    match me.kind {
                        MouseEventKind::Down(MouseButton::Left) => is_drawing_wall = true,
                        MouseEventKind::Up(MouseButton::Left) => is_drawing_wall = false,
                        MouseEventKind::Down(MouseButton::Right) => is_erasing_wall = true,
                        MouseEventKind::Up(MouseButton::Right) => is_erasing_wall = false,
                        MouseEventKind::Drag(MouseButton::Left) => is_drawing_wall = true,
                        MouseEventKind::Drag(MouseButton::Right) => is_erasing_wall = true,
                        _ => {}
                    }

                    // Account for the 1-cell border offset
                    let mx = me.column.saturating_sub(1) as usize;
                    let my = me.row.saturating_sub(1) as usize;

                    if mx < engine.grid.width && my < engine.grid.height {
                        if is_drawing_wall {
                            engine.grid.solid[[mx, my]] = true;
                            for i in 0..9 {
                                engine.grid.f[[mx, my, i]] =
                                    crate::lbm::equilibrium(1.0, 0.0, 0.0, i);
                            }
                        } else if is_erasing_wall {
                            engine.grid.solid[[mx, my]] = false;
                        }
                    }
                }
                // ---- Bug 4 fix: Resize detection ----
                Event::Resize(w, h) => {
                    // Rebuild the entire engine at the new terminal size.
                    // We subtract 2 for the border cells on each axis.
                    let new_w = w.saturating_sub(2) as usize;
                    let new_h = h.saturating_sub(2) as usize;
                    if new_w > 4 && new_h > 4 {
                        engine = LbmEngine::new(new_w, new_h, engine.tau);
                    }
                }
                _ => {}
            }
        }

        // ---- Physics ticks (decoupled from render rate) ----
        if last_render.elapsed() >= RENDER_INTERVAL {
            for _ in 0..PHYSICS_TICKS_PER_FRAME {
                apply_inlet_forcing(&mut engine);
                engine.tick();
            }
            last_render = Instant::now();
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        event::DisableMouseCapture
    )?;
    Ok(())
}
