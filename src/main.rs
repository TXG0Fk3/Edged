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

    // Carrega ou inicializa a configuração (~/.config/edged/config.toml)
    let config = match Config::load_or_create(cli.config.as_deref()) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("[edged] Erro ao carregar configuração: {}", err);
            std::process::exit(1);
        }
    };

    // Determina o modo de operação (prioridade para flags CLI, fallback para config.toml)
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

    // Captura da tela antes de abrir qualquer janela de overlay
    let screen_image = match capture::capture_screen(&config) {
        Ok(img) => Arc::new(img),
        Err(err) => {
            eprintln!("[edged] Falha ao capturar a tela: {}", err);
            eprintln!(
                "[edged] Dica: Certifique-se de que o utilitário 'grim' (wlroots/Hyprland) ou o serviço xdg-desktop-portal esteja disponível."
            );
            std::process::exit(1);
        }
    };

    let detector = Arc::new(EdgeDetector::new(screen_image));
    let config = Arc::new(config);

    // Inicializa a aplicação GTK4 para desenhar o overlay Wayland
    let app = gtk4::Application::builder()
        .application_id("com.github.edged.screenruler")
        .build();

    let config_clone = config.clone();
    let detector_clone = detector.clone();

    app.connect_activate(move |application| {
        ui::create_and_show_overlay(
            application,
            config_clone.clone(),
            detector_clone.clone(),
            mode,
            initial_tolerance,
        );
    });

    // Executa a aplicação GTK sem repassar os argumentos da linha de comando
    app.run_with_args::<&str>(&[]);
}
