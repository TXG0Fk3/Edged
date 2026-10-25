use crate::config::{CaptureMethod, Config};
use ashpd::desktop::screenshot::Screenshot;
use image::RgbaImage;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug)]
pub enum CaptureError {
    GrimFailed(String),
    PortalFailed(String),
    CommandFailed(String),
    ImageDecodeFailed(String),
    IoError(std::io::Error),
    AllMethodsFailed,
}

impl std::fmt::Display for CaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaptureError::GrimFailed(e) => write!(f, "Grim capture failed: {}", e),
            CaptureError::PortalFailed(e) => write!(f, "Desktop portal capture failed: {}", e),
            CaptureError::CommandFailed(e) => write!(f, "Custom capture command failed: {}", e),
            CaptureError::ImageDecodeFailed(e) => write!(f, "Image decode failed: {}", e),
            CaptureError::IoError(e) => write!(f, "IO error: {}", e),
            CaptureError::AllMethodsFailed => write!(f, "All screen capture methods failed"),
        }
    }
}

impl std::error::Error for CaptureError {}

impl From<std::io::Error> for CaptureError {
    fn from(err: std::io::Error) -> Self {
        CaptureError::IoError(err)
    }
}

/// Executes screen capture according to the configured capture method
pub fn capture_screen(config: &Config) -> Result<RgbaImage, CaptureError> {
    match config.general.capture_method {
        CaptureMethod::Grim => capture_via_grim(),
        CaptureMethod::Portal => capture_via_portal(),
        CaptureMethod::Command => {
            if config.general.custom_capture_command.is_empty() {
                Err(CaptureError::CommandFailed(
                    "custom_capture_command is empty".to_string(),
                ))
            } else {
                capture_via_custom_command(&config.general.custom_capture_command)
            }
        }
        CaptureMethod::Auto => {
            // Try 'grim' first (instant on wlroots/Hyprland/Sway)
            match capture_via_grim() {
                Ok(img) => Ok(img),
                Err(grim_err) => {
                    // Fall back to xdg-desktop-portal (GNOME, KDE Plasma, etc.)
                    match capture_via_portal() {
                        Ok(img) => Ok(img),
                        Err(portal_err) => {
                            // If a custom command is configured, try it as a last resort
                            if !config.general.custom_capture_command.is_empty() {
                                capture_via_custom_command(&config.general.custom_capture_command)
                            } else {
                                eprintln!("[edged] Grim: {}", grim_err);
                                eprintln!("[edged] Portal: {}", portal_err);
                                Err(CaptureError::AllMethodsFailed)
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Captures the screen using the `grim` utility (Wayland screencopy)
pub fn capture_via_grim() -> Result<RgbaImage, CaptureError> {
    let temp_path = std::env::temp_dir().join(format!("edged_grim_{}.png", std::process::id()));

    let output = Command::new("grim")
        .arg("-t")
        .arg("png")
        .arg(&temp_path)
        .output();

    match output {
        Ok(out) => {
            if !out.status.success() {
                let stderr = String::from_utf8_lossy(&out.stderr);
                let _ = fs::remove_file(&temp_path);
                return Err(CaptureError::GrimFailed(stderr.trim().to_string()));
            }

            if !temp_path.exists() {
                return Err(CaptureError::GrimFailed(
                    "Grim output file was not created".to_string(),
                ));
            }

            let img = image::open(&temp_path)
                .map_err(|e| CaptureError::ImageDecodeFailed(e.to_string()))?;
            let _ = fs::remove_file(&temp_path);
            Ok(img.to_rgba8())
        }
        Err(e) => Err(CaptureError::GrimFailed(e.to_string())),
    }
}

/// Captures the screen via XDG Desktop Portal (org.freedesktop.portal.Screenshot)
pub fn capture_via_portal() -> Result<RgbaImage, CaptureError> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| CaptureError::PortalFailed(format!("Failed to create tokio runtime: {}", e)))?;

    let uri_string = rt.block_on(async {
        let request = Screenshot::request().interactive(false).modal(false);
        let response = request
            .send()
            .await
            .map_err(|e| format!("Portal call failed: {}", e))?
            .response()
            .map_err(|e| format!("Portal response failed: {}", e))?;
        Ok::<String, String>(response.uri().as_str().to_string())
    }).map_err(CaptureError::PortalFailed)?;

    // Convert file:// URI to a local filesystem path
    let file_path = uri_to_path(&uri_string)
        .ok_or_else(|| CaptureError::PortalFailed(format!("Invalid URI: {}", uri_string)))?;

    if !file_path.exists() {
        return Err(CaptureError::PortalFailed(format!(
            "Screenshot file not found at {:?}",
            file_path
        )));
    }

    let img = image::open(&file_path)
        .map_err(|e| CaptureError::ImageDecodeFailed(e.to_string()))?;

    // Clean up temporary screenshot file created by portal
    let _ = fs::remove_file(&file_path);

    Ok(img.to_rgba8())
}

/// Executes a custom screenshot capture command
fn capture_via_custom_command(cmd_template: &str) -> Result<RgbaImage, CaptureError> {
    let temp_path = std::env::temp_dir().join(format!("edged_custom_{}.png", std::process::id()));
    let temp_str = temp_path.to_string_lossy().to_string();

    let cmd_str = if cmd_template.contains("%s") {
        cmd_template.replace("%s", &temp_str)
    } else {
        format!("{} {}", cmd_template, temp_str)
    };

    let status = Command::new("sh")
        .arg("-c")
        .arg(&cmd_str)
        .status()
        .map_err(|e| CaptureError::CommandFailed(e.to_string()))?;

    if !status.success() {
        let _ = fs::remove_file(&temp_path);
        return Err(CaptureError::CommandFailed(format!(
            "Command exited with status code: {:?}",
            status.code()
        )));
    }

    if !temp_path.exists() {
        return Err(CaptureError::CommandFailed(
            "Custom command did not produce the expected output file".to_string(),
        ));
    }

    let img = image::open(&temp_path)
        .map_err(|e| CaptureError::ImageDecodeFailed(e.to_string()))?;
    let _ = fs::remove_file(&temp_path);

    Ok(img.to_rgba8())
}

/// Converts an RgbaImage to a Cairo ImageSurface for hardware-accelerated rendering
pub fn rgba_to_cairo_surface(img: &RgbaImage) -> Result<cairo::ImageSurface, Box<dyn std::error::Error>> {
    let (width, height) = img.dimensions();
    let mut surface = cairo::ImageSurface::create(cairo::Format::Rgb24, width as i32, height as i32)?;
    let stride = surface.stride() as usize;
    {
        let mut data = surface
            .data()
            .map_err(|e| format!("Failed to borrow surface data: {:?}", e))?;
        let raw = img.as_raw();
        for y in 0..height as usize {
            let src_row = y * (width as usize) * 4;
            let dst_row = y * stride;
            for x in 0..width as usize {
                let src_idx = src_row + x * 4;
                let dst_idx = dst_row + x * 4;
                // Cairo Rgb24 on host endianness (little-endian x86_64: BGRA)
                data[dst_idx] = raw[src_idx + 2];     // B
                data[dst_idx + 1] = raw[src_idx + 1]; // G
                data[dst_idx + 2] = raw[src_idx];     // R
                data[dst_idx + 3] = 0xFF;             // Unused / Alpha
            }
        }
    }
    Ok(surface)
}

/// Converts a file:// URI to a PathBuf with percent-decoding
fn uri_to_path(uri: &str) -> Option<PathBuf> {
    let stripped = uri.strip_prefix("file://")?;
    let decoded = url_decode(stripped);
    Some(PathBuf::from(decoded))
}

fn url_decode(input: &str) -> String {
    let mut result = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(val) = u8::from_str_radix(&input[i + 1..i + 3], 16) {
                result.push(val);
                i += 3;
                continue;
            }
        }
        result.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&result).into_owned()
}
