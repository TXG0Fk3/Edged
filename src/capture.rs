use crate::config::{CaptureMethod, Config};
use crate::monitor::MonitorInfo;
use ashpd::desktop::screenshot::Screenshot;
use image::RgbaImage;
use std::fs;
use std::path::{Path, PathBuf};
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
pub fn capture_screen(
    config: &Config,
    target_monitor: Option<&MonitorInfo>,
) -> Result<RgbaImage, CaptureError> {
    // Determine configured method (priority to [capture] section, fallback to [general])
    let method = if config.capture.method != CaptureMethod::Auto {
        config.capture.method
    } else {
        config.general.capture_method
    };

    let custom_cmd = if !config.capture.command.is_empty() {
        &config.capture.command
    } else {
        &config.general.custom_capture_command
    };

    match method {
        CaptureMethod::Grim => {
            log::info!("Capturing screen via Grim...");
            capture_via_grim(target_monitor)
        }
        CaptureMethod::Portal => {
            log::info!("Capturing screen via XDG Desktop Portal...");
            capture_via_portal()
        }
        CaptureMethod::Command | CaptureMethod::Custom => {
            if custom_cmd.is_empty() {
                Err(CaptureError::CommandFailed(
                    "Custom capture command is empty. Configure 'command' in [capture] section.".to_string(),
                ))
            } else {
                log::info!("Capturing screen via custom command: '{}'", custom_cmd);
                capture_via_custom_command(custom_cmd, target_monitor)
            }
        }
        CaptureMethod::Auto => {
            // If custom command is set, prioritize it
            if !custom_cmd.is_empty() {
                log::info!("Auto-capture: using configured custom command: '{}'", custom_cmd);
                return capture_via_custom_command(custom_cmd, target_monitor);
            }

            // Try 'grim' first (instant on wlroots/Hyprland/Sway)
            log::debug!("Auto-capture: testing Grim...");
            match capture_via_grim(target_monitor) {
                Ok(img) => {
                    log::info!("Auto-capture: successfully captured via Grim");
                    Ok(img)
                }
                Err(grim_err) => {
                    log::debug!("Auto-capture: Grim unavailable ({}), checking alternatives...", grim_err);

                    // On Hyprland, try hyprshot before falling back to portal
                    if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
                        log::debug!("Auto-capture: Hyprland detected, attempting hyprshot...");
                        if let Ok(img) = capture_via_hyprshot(target_monitor) {
                            log::info!("Auto-capture: successfully captured via hyprshot");
                            return Ok(img);
                        }
                    }

                    // Fall back to xdg-desktop-portal (GNOME, KDE Plasma, etc.)
                    log::info!("Auto-capture: falling back to XDG Desktop Portal...");
                    match capture_via_portal() {
                        Ok(img) => {
                            log::info!("Auto-capture: successfully captured via Desktop Portal");
                            Ok(img)
                        }
                        Err(portal_err) => {
                            log::error!("All capture methods failed. Grim: {}, Portal: {}", grim_err, portal_err);
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

/// Captures the screen using the `hyprshot` utility (native to Hyprland)
pub fn capture_via_hyprshot(target_monitor: Option<&MonitorInfo>) -> Result<RgbaImage, CaptureError> {
    let mut cmd = Command::new("hyprshot");
    cmd.args(["-m", "output", "-s", "--raw"]);

    if let Some(monitor) = target_monitor {
        // hyprshot does not take monitor name directly on -m output without -o (which is output folder),
        // but it captures the focused output by default.
        let _ = monitor;
    }

    let output = cmd.output().map_err(|e| CaptureError::CommandFailed(format!("Failed to run hyprshot: {}", e)))?;
    if !output.status.success() || output.stdout.is_empty() {
        return Err(CaptureError::CommandFailed("hyprshot failed or produced empty output".to_string()));
    }

    let img = image::load_from_memory(&output.stdout)
        .map_err(|e| CaptureError::ImageDecodeFailed(format!("Failed to decode hyprshot output: {}", e)))?;

    Ok(img.to_rgba8())
}

/// Captures the screen using the `grim` utility (Wayland screencopy)
pub fn capture_via_grim(target_monitor: Option<&MonitorInfo>) -> Result<RgbaImage, CaptureError> {
    let temp_path = std::env::temp_dir().join(format!("edged_grim_{}.png", std::process::id()));

    // If a target monitor is specified, try targeting its output connector directly first
    if let Some(monitor) = target_monitor {
        let mut cmd = Command::new("grim");
        cmd.arg("-t").arg("png");
        cmd.arg("-o").arg(&monitor.name);
        cmd.arg(&temp_path);

        if let Ok(out) = cmd.output() {
            if out.status.success() && temp_path.exists() {
                if let Ok(img) = image::open(&temp_path) {
                    let _ = fs::remove_file(&temp_path);
                    return Ok(img.to_rgba8());
                }
            }
        }
        let _ = fs::remove_file(&temp_path);
    }

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

/// Expands placeholders in a custom capture command template
pub fn expand_command_placeholders(
    cmd_template: &str,
    temp_path: &Path,
    monitor: Option<&MonitorInfo>,
) -> String {
    let temp_str = temp_path.to_string_lossy();
    let temp_dir = temp_path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "/tmp".to_string());
    let temp_filename = temp_path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "edged_capture.png".to_string());

    let monitor_name = monitor.map(|m| m.name.as_str()).unwrap_or("");
    let monitor_w = monitor.map(|m| m.width.to_string()).unwrap_or_default();
    let monitor_h = monitor.map(|m| m.height.to_string()).unwrap_or_default();
    let monitor_x = monitor.map(|m| m.x.to_string()).unwrap_or_default();
    let monitor_y = monitor.map(|m| m.y.to_string()).unwrap_or_default();

    let mut cmd = cmd_template.to_string();

    let has_any_placeholder = cmd.contains("%s")
        || cmd.contains("%path")
        || cmd.contains("%d")
        || cmd.contains("%dir")
        || cmd.contains("%n")
        || cmd.contains("%filename")
        || cmd.contains("%o")
        || cmd.contains("%output")
        || cmd.contains("%w")
        || cmd.contains("%width")
        || cmd.contains("%h")
        || cmd.contains("%height")
        || cmd.contains("%x")
        || cmd.contains("%y");

    if has_any_placeholder {
        cmd = cmd
            .replace("%path", &temp_str)
            .replace("%s", &temp_str)
            .replace("%dir", &temp_dir)
            .replace("%d", &temp_dir)
            .replace("%filename", &temp_filename)
            .replace("%n", &temp_filename)
            .replace("%output", monitor_name)
            .replace("%o", monitor_name)
            .replace("%width", &monitor_w)
            .replace("%w", &monitor_w)
            .replace("%height", &monitor_h)
            .replace("%h", &monitor_h)
            .replace("%x", &monitor_x)
            .replace("%y", &monitor_y);
    } else {
        // If no placeholder is present, append the target file path
        cmd = format!("{} {}", cmd.trim(), temp_str);
    }

    cmd
}

/// Finds the newest image file inside a directory
pub fn find_newest_image_in_dir(dir: &Path) -> Option<PathBuf> {
    let mut candidates: Vec<(PathBuf, std::time::SystemTime)> = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if ext_lower == "png"
                        || ext_lower == "jpg"
                        || ext_lower == "jpeg"
                        || ext_lower == "webp"
                    {
                        let mtime = entry
                            .metadata()
                            .and_then(|m| m.modified())
                            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                        candidates.push((p, mtime));
                    }
                }
            }
        }
    }
    candidates.sort_by_key(|a| std::cmp::Reverse(a.1));
    candidates.first().map(|(p, _)| p.clone())
}

/// Executes a custom screenshot capture command with resilient fallback handling
fn capture_via_custom_command(
    cmd_template: &str,
    target_monitor: Option<&MonitorInfo>,
) -> Result<RgbaImage, CaptureError> {
    let temp_dir = std::env::temp_dir();
    let temp_filename = format!("edged_custom_{}.png", std::process::id());
    let temp_path = temp_dir.join(&temp_filename);

    // Ensure clean state before running command
    if temp_path.exists() {
        if temp_path.is_dir() {
            let _ = fs::remove_dir_all(&temp_path);
        } else {
            let _ = fs::remove_file(&temp_path);
        }
    }

    let cmd_str = expand_command_placeholders(cmd_template, &temp_path, target_monitor);

    let output = Command::new("sh")
        .arg("-c")
        .arg(&cmd_str)
        .output()
        .map_err(|e| CaptureError::CommandFailed(format!("Failed to execute command: {}", e)))?;

    let stderr_msg = String::from_utf8_lossy(&output.stderr).trim().to_string();

    // Look for the generated screenshot:
    // 1. Direct file at temp_path
    // 2. Or if a directory was created with the name temp_path (common when tools like hyprshot treat -o as folder),
    //    find the newest image inside that directory
    let (image_path, is_dir_cleanup) = if temp_path.is_file() {
        (Some(temp_path.clone()), false)
    } else if temp_path.is_dir() {
        match find_newest_image_in_dir(&temp_path) {
            Some(found) => (Some(found), true),
            None => (None, true),
        }
    } else {
        (None, false)
    };

    let result = match image_path {
        Some(path) => {
            if !output.status.success() {
                eprintln!(
                    "[edged] Note: Custom command exited with status {:?}, but a valid image was found. Proceeding. Details: {}",
                    output.status.code(),
                    stderr_msg
                );
            }
            match image::open(&path) {
                Ok(img) => Ok(img.to_rgba8()),
                Err(err) => Err(CaptureError::ImageDecodeFailed(format!(
                    "Failed to decode image at {:?}: {}",
                    path, err
                ))),
            }
        }
        None => {
            let error_details = if !stderr_msg.is_empty() {
                format!("Stderr: {}", stderr_msg)
            } else {
                format!("Exit code: {:?}", output.status.code())
            };
            Err(CaptureError::CommandFailed(format!(
                "Command did not generate an image at {:?}. {}",
                temp_path, error_details
            )))
        }
    };

    // Clean up temporary file or directory
    if temp_path.exists() {
        if is_dir_cleanup || temp_path.is_dir() {
            let _ = fs::remove_dir_all(&temp_path);
        } else {
            let _ = fs::remove_file(&temp_path);
        }
    }

    result
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
