use crate::config::CopyFormat;
use crate::ruler::RulerMode;
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
    // 1. Copy via arboard (cross-platform / Wayland data-control)
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        let _ = clipboard.set_text(text);
    }

    // WHY wl-copy fallback:
    // On Wayland, when a process terminates, its clipboard data-source is destroyed by the compositor
    // unless an external clipboard manager takes ownership. Because Edged follows an immediate one-shot
    // termination model (exits instantly on click by default), spawning `wl-copy` ensures the copied
    // measurement text survives process exit and remains in the system clipboard.
    let _ = Command::new("wl-copy")
        .arg("--")
        .arg(text)
        .spawn();

    log::info!("Copied to clipboard: '{}'", text);
}
