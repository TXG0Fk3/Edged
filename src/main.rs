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

    let (target_w, target_h, target_x, target_y) = if let Some(ref target) = target_monitor {
        let scale = if target.scale > 0.0 { target.scale } else { 1.0 };
        (
            target.width as f32 / scale as f32,
            target.height as f32 / scale as f32,
            target.x as f32 / scale as f32,
            target.y as f32 / scale as f32,
        )
    } else {
        (
            screen_image.width() as f32,
            screen_image.height() as f32,
            0.0,
            0.0,
        )
    };

    apply_compositor_window_rules(target_monitor.as_ref());

    // Launch egui / eframe application
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id("edged")
            .with_title("Edged")
            .with_inner_size([target_w, target_h])
            .with_position([target_x, target_y])
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

/// Dynamic window rules to ensure tiling compositors (Hyprland, Sway) properly frame fullscreen overlays
fn apply_compositor_window_rules(target_monitor: Option<&monitor::MonitorInfo>) {
    if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
        let mut rules = vec![
            "float, class:^(edged)$".to_string(),
            "fullscreen, class:^(edged)$".to_string(),
            "noborder, class:^(edged)$".to_string(),
            "noshadow, class:^(edged)$".to_string(),
            "rounding 0, class:^(edged)$".to_string(),
            "noanim, class:^(edged)$".to_string(),
            "stayfocused, class:^(edged)$".to_string(),
            "pin, class:^(edged)$".to_string(),
            "move 0 0, class:^(edged)$".to_string(),
            "float, initialClass:^(edged)$".to_string(),
            "fullscreen, initialClass:^(edged)$".to_string(),
        ];
        if let Some(target) = target_monitor {
            rules.push(format!("monitor {}, class:^(edged)$", target.name));
        }
        for rule in rules {
            let _ = std::process::Command::new("hyprctl")
                .args(["keyword", "windowrulev2", &rule])
                .output();
        }
    } else if std::env::var_os("SWAYSOCK").is_some() {
        let _ = std::process::Command::new("swaymsg")
            .arg("for_window [app_id=\"edged\"] floating enable, fullscreen enable, border none")
            .output();
        if let Some(target) = target_monitor {
            let _ = std::process::Command::new("swaymsg")
                .arg(format!("for_window [app_id=\"edged\"] move to output {}", target.name))
                .output();
        }
    }
}
