mod capture;
mod cli;
mod clipboard;
mod config;
mod ruler;
mod ui;

use clap::Parser;
use cli::Cli;
use config::{Config, DefaultMode};
use gtk4::prelude::*;
use ruler::{EdgeDetector, RulerMode};
use std::sync::Arc;

fn main() {
    let cli = Cli::parse();

    // Load or initialize configuration (~/.config/edged/config.toml)
    let config = match Config::load_or_create(cli.config.as_deref()) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("[edged] Error loading configuration: {}", err);
            std::process::exit(1);
        }
    };

    // Determine operating mode (CLI flags take priority, fallback to config.toml)
    let mode = if cli.bounds {
        RulerMode::Bounds
    } else if cli.spacing {
        RulerMode::Spacing
    } else if cli.horizontal {
        RulerMode::Horizontal
    } else if cli.vertical {
        RulerMode::Vertical
    } else {
        match config.general.default_mode {
            DefaultMode::Bounds => RulerMode::Bounds,
            DefaultMode::Spacing => RulerMode::Spacing,
            DefaultMode::Horizontal => RulerMode::Horizontal,
            DefaultMode::Vertical => RulerMode::Vertical,
        }
    };

    let initial_tolerance = cli
        .tolerance
        .unwrap_or(config.ruler.initial_tolerance)
        .clamp(1, 100);

    // Capture screen before creating any overlay window
    let screen_image = match capture::capture_screen(&config) {
        Ok(img) => img,
        Err(err) => {
            eprintln!("[edged] Failed to capture screen: {}", err);
            eprintln!(
                "[edged] Hint: Ensure either 'grim' (for wlroots/Hyprland) or xdg-desktop-portal is installed and running."
            );
            std::process::exit(1);
        }
    };

    // Convert captured frame to Cairo surface for rendering background
    let cairo_surface = match capture::rgba_to_cairo_surface(&screen_image) {
        Ok(s) => Arc::new(s),
        Err(err) => {
            eprintln!("[edged] Failed to create Cairo image surface: {}", err);
            std::process::exit(1);
        }
    };

    let detector = Arc::new(EdgeDetector::new(Arc::new(screen_image)));
    let config = Arc::new(config);

    // Initialize GTK4 application for Wayland overlay
    let app = gtk4::Application::builder()
        .application_id("com.github.edged.screenruler")
        .build();

    let config_clone = config.clone();
    let detector_clone = detector.clone();
    let surface_clone = cairo_surface.clone();

    app.connect_activate(move |application| {
        ui::create_and_show_overlay(
            application,
            config_clone.clone(),
            detector_clone.clone(),
            surface_clone.clone(),
            mode,
            initial_tolerance,
        );
    });

    // Run GTK application without forwarding command-line arguments
    app.run_with_args::<&str>(&[]);
}
