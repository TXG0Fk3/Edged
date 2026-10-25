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
use eframe::egui;
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

    let all_monitors = monitor::get_all_monitors();
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
    let screen_image = if let Some(ref target) = target_monitor {
        if config.monitor.crop_to_monitor {
            monitor::crop_image_if_needed(raw_image, target, &all_monitors, true)
        } else {
            raw_image
        }
    } else {
        raw_image
    };

    let detector = Arc::new(EdgeDetector::new(Arc::new(screen_image.clone())));
    let config = Arc::new(config);

    // Launch egui / eframe application
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Edged")
            .with_decorations(false)
            .with_transparent(true)
            .with_fullscreen(true)
            .with_always_on_top()
            .with_active(true),
        ..Default::default()
    };

    let app = ui::EdgedApp::new(
        config,
        detector,
        Arc::new(screen_image),
        mode,
        initial_tolerance,
    );

    if let Err(e) = eframe::run_native(
        "Edged",
        native_options,
        Box::new(|_cc| Ok(Box::new(app))),
    ) {
        eprintln!("[edged] Error running egui application: {}", e);
        std::process::exit(1);
    }
}
