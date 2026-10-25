use crate::config::CopyFormat;
use crate::ruler::RulerMode;
use gdk4::prelude::*;
use std::process::Command;

pub fn format_measurement(
    mode: RulerMode,
    width: u32,
    height: u32,
    format: CopyFormat,
) -> String {
    match mode {
        RulerMode::Bounds | RulerMode::Spacing => match format {
            CopyFormat::Compact => format!("{}x{}", width, height),
            CopyFormat::Verbose => format!("{} × {} px", width, height),
            CopyFormat::Css => format!("width: {}px; height: {}px;", width, height),
        },
        RulerMode::Horizontal => match format {
            CopyFormat::Compact => format!("{}", width),
            CopyFormat::Verbose => format!("{} px", width),
            CopyFormat::Css => format!("width: {}px;", width),
        },
        RulerMode::Vertical => match format {
            CopyFormat::Compact => format!("{}", height),
            CopyFormat::Verbose => format!("{} px", height),
            CopyFormat::Css => format!("height: {}px;", height),
        },
    }
}

pub fn copy_to_clipboard(text: &str) {
    // 1. Tenta definir no clipboard do GTK4 se a sessão gráfica estiver ativa
    if let Some(display) = gdk4::Display::default() {
        display.clipboard().set_text(text);
    }

    // 2. Garante persistência em Wayland usando `wl-copy` caso esteja instalado
    let _ = Command::new("wl-copy")
        .arg("--")
        .arg(text)
        .spawn();

    println!("[edged] Copiado para o clipboard: {}", text);
}
