mod capture;
mod cli;
mod clipboard;
mod config;
mod monitor;
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
    let mut config = match Config::load_or_create(cli.config.as_deref()) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("[edged] Error loading configuration: {}", err);
            std::process::exit(1);
        }
    };

    // CLI --monitor flag overrides configuration
    if let Some(ref mon) = cli.monitor {
        config.monitor.target = mon.parse().unwrap();
    }

    // Initialize GTK4 and Wayland display
    if let Err(err) = gtk4::init() {
        eprintln!("[edged] Warning: Failed to initialize GTK4 runtime: {}", err);
    }

    let display = gdk4::Display::default().expect("Wayland display not found");
    let all_monitors = monitor::get_all_monitors(&display);
    let target_monitor = monitor::resolve_target_monitor(&config.monitor.target, &all_monitors).cloned();

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
    let raw_image = match capture::capture_screen(&config, target_monitor.as_ref()) {
        Ok(img) => img,
        Err(err) => {
            eprintln!("[edged] Failed to capture screen: {}", err);
            eprintln!(
                "[edged] Hint: Ensure either 'grim' (for wlroots/Hyprland) or xdg-desktop-portal is installed and running."
            );
            std::process::exit(1);
        }
    };

    // If targeting a single monitor, crop multi-monitor screenshots to target geometry
    let (screen_image, is_monitor_cropped) = if let Some(ref target) = target_monitor {
        if config.monitor.crop_to_monitor {
            let cropped = monitor::crop_image_if_needed(raw_image, target, &all_monitors, true);
            (cropped, true)
        } else {
            (raw_image, false)
        }
    } else {
        (raw_image, false)
    };

    let target_monitor_name = target_monitor.map(|m| m.name);

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
            target_monitor_name.clone(),
            is_monitor_cropped,
        );
    });

    // Run GTK application without forwarding command-line arguments
    app.run_with_args::<&str>(&[]);
}
