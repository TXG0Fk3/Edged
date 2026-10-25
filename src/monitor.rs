use image::RgbaImage;
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Target monitor for ruler overlay
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MonitorTarget {
    /// Target the currently active / focused monitor (recommended default)
    Focused,
    /// Span ruler overlay across all connected monitors
    All,
    /// Target the primary monitor
    Primary,
    /// Target a specific monitor connector (e.g. "DP-1", "HDMI-A-1", "eDP-1")
    #[serde(untagged)]
    Named(String),
}

impl Default for MonitorTarget {
    fn default() -> Self {
        MonitorTarget::Focused
    }
}

impl std::str::FromStr for MonitorTarget {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        match trimmed.to_ascii_lowercase().as_str() {
            "focused" | "focus" | "current" => Ok(MonitorTarget::Focused),
            "all" | "span" => Ok(MonitorTarget::All),
            "primary" | "main" => Ok(MonitorTarget::Primary),
            _ => Ok(MonitorTarget::Named(trimmed.to_string())),
        }
    }
}

impl std::fmt::Display for MonitorTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MonitorTarget::Focused => write!(f, "focused"),
            MonitorTarget::All => write!(f, "all"),
            MonitorTarget::Primary => write!(f, "primary"),
            MonitorTarget::Named(name) => write!(f, "{}", name),
        }
    }
}

/// Metadata describing a physical or logical monitor
#[derive(Debug, Clone, PartialEq)]
pub struct MonitorInfo {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale: f64,
    pub is_primary: bool,
    pub is_focused: bool,
}

/// Queries compositor-specific APIs to detect monitor metadata, focused output, and primary output
pub fn detect_compositor_monitors() -> Vec<MonitorInfo> {
    // 1. Try Hyprland (hyprctl monitors -j)
    if let Ok(monitors) = detect_hyprland_monitors() {
        if !monitors.is_empty() {
            return monitors;
        }
    }

    // 2. Try Sway / wlroots (swaymsg -t get_outputs -r)
    if let Ok(monitors) = detect_sway_monitors() {
        if !monitors.is_empty() {
            return monitors;
        }
    }

    // 3. Try GNOME Mutter (org.gnome.Mutter.DisplayConfig)
    if let Ok(monitors) = detect_gnome_monitors() {
        if !monitors.is_empty() {
            return monitors;
        }
    }

    Vec::new()
}

fn detect_hyprland_monitors() -> Result<Vec<MonitorInfo>, Box<dyn std::error::Error>> {
    let output = Command::new("hyprctl")
        .args(["monitors", "-j"])
        .output()?;

    if !output.status.success() {
        return Err("hyprctl failed".into());
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let array = json.as_array().ok_or("Expected JSON array")?;

    let mut monitors = Vec::new();
    for item in array {
        let name = item["name"].as_str().unwrap_or("").to_string();
        if name.is_empty() {
            continue;
        }
        let x = item["x"].as_i64().unwrap_or(0) as i32;
        let y = item["y"].as_i64().unwrap_or(0) as i32;
        let width = item["width"].as_u64().unwrap_or(1920) as u32;
        let height = item["height"].as_u64().unwrap_or(1080) as u32;
        let scale = item["scale"].as_f64().unwrap_or(1.0);
        let is_focused = item["focused"].as_bool().unwrap_or(false);

        monitors.push(MonitorInfo {
            name,
            x,
            y,
            width,
            height,
            scale,
            is_primary: is_focused,
            is_focused,
        });
    }

    Ok(monitors)
}

fn detect_sway_monitors() -> Result<Vec<MonitorInfo>, Box<dyn std::error::Error>> {
    let output = Command::new("swaymsg")
        .args(["-t", "get_outputs", "-r"])
        .output()?;

    if !output.status.success() {
        return Err("swaymsg failed".into());
    }

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let array = json.as_array().ok_or("Expected JSON array")?;

    let mut monitors = Vec::new();
    for item in array {
        let name = item["name"].as_str().unwrap_or("").to_string();
        if name.is_empty() {
            continue;
        }
        let rect = &item["rect"];
        let x = rect["x"].as_i64().unwrap_or(0) as i32;
        let y = rect["y"].as_i64().unwrap_or(0) as i32;
        let width = rect["width"].as_u64().unwrap_or(1920) as u32;
        let height = rect["height"].as_u64().unwrap_or(1080) as u32;
        let scale = item["scale"].as_f64().unwrap_or(1.0);
        let is_focused = item["focused"].as_bool().unwrap_or(false);
        let is_primary = item["primary"].as_bool().unwrap_or(is_focused);

        monitors.push(MonitorInfo {
            name,
            x,
            y,
            width,
            height,
            scale,
            is_primary,
            is_focused,
        });
    }

    Ok(monitors)
}

fn detect_gnome_monitors() -> Result<Vec<MonitorInfo>, Box<dyn std::error::Error>> {
    let output = Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest",
            "org.gnome.Mutter.DisplayConfig",
            "--object-path",
            "/org/gnome/Mutter/DisplayConfig",
            "--method",
            "org.gnome.Mutter.DisplayConfig.GetCurrentState",
        ])
        .output()?;

    if !output.status.success() {
        return Err("gdbus Mutter DisplayConfig failed".into());
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut monitors = Vec::new();

    // Look for logical monitor definitions: (x, y, scale, transform, is_primary, [('connector', ...)])
    for line in text.split("('") {
        if let Some(conn_end) = line.find('\'') {
            let connector = &line[..conn_end];
            if connector.contains('-') || connector.starts_with("eDP") || connector.starts_with("DP") || connector.starts_with("HDMI") || connector.starts_with("Virtual") {
                let is_primary = text.contains(&format!("true, [('{}'", connector));
                monitors.push(MonitorInfo {
                    name: connector.to_string(),
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 0,
                    scale: 1.0,
                    is_primary,
                    is_focused: is_primary,
                });
            }
        }
    }

    Ok(monitors)
}

/// Collects complete monitor list from compositor metadata with fallback
pub fn get_all_monitors() -> Vec<MonitorInfo> {
    let monitors = detect_compositor_monitors();
    if monitors.is_empty() {
        vec![MonitorInfo {
            name: "default".to_string(),
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
            scale: 1.0,
            is_primary: true,
            is_focused: true,
        }]
    } else {
        monitors
    }
}

/// Resolves the target monitor based on user configuration or CLI override
pub fn resolve_target_monitor<'a>(
    target: &MonitorTarget,
    monitors: &'a [MonitorInfo],
) -> Option<&'a MonitorInfo> {
    if monitors.is_empty() {
        return None;
    }

    match target {
        MonitorTarget::Focused => {
            // 1. Look for monitor with is_focused == true
            if let Some(m) = monitors.iter().find(|m| m.is_focused) {
                return Some(m);
            }
            // 2. Fall back to is_primary == true
            if let Some(m) = monitors.iter().find(|m| m.is_primary) {
                return Some(m);
            }
            // 3. Fall back to first monitor
            monitors.first()
        }
        MonitorTarget::Primary => {
            if let Some(m) = monitors.iter().find(|m| m.is_primary) {
                return Some(m);
            }
            monitors.first()
        }
        MonitorTarget::Named(name) => {
            if let Some(m) = monitors.iter().find(|m| m.name.eq_ignore_ascii_case(name)) {
                return Some(m);
            }
            eprintln!(
                "[edged] Warning: Specified monitor '{}' not found. Available monitors: {}. Falling back to default.",
                name,
                monitors.iter().map(|m| m.name.as_str()).collect::<Vec<_>>().join(", ")
            );
            resolve_target_monitor(&MonitorTarget::Focused, monitors)
        }
        MonitorTarget::All => None,
    }
}

/// Automatically crops a screenshot image to the target monitor's bounds if the screenshot covers multiple monitors
pub fn crop_image_if_needed(
    image: RgbaImage,
    target: &MonitorInfo,
    all_monitors: &[MonitorInfo],
    crop_enabled: bool,
) -> RgbaImage {
    if !crop_enabled || all_monitors.len() <= 1 {
        return image;
    }

    let target_phys_w = (target.width as f64 * target.scale).round() as u32;
    let target_phys_h = (target.height as f64 * target.scale).round() as u32;

    // If the image already matches the target monitor physical dimensions (e.g. grim -o was used), no crop needed
    if image.width() == target_phys_w && image.height() == target_phys_h {
        return image;
    }

    // If the image is smaller than the target monitor, cannot crop safely
    if image.width() < target_phys_w && image.height() < target_phys_h {
        return image;
    }

    // Determine total logical desktop bounds
    let min_x = all_monitors.iter().map(|m| m.x).min().unwrap_or(0);
    let min_y = all_monitors.iter().map(|m| m.y).min().unwrap_or(0);
    let max_x = all_monitors
        .iter()
        .map(|m| m.x + m.width as i32)
        .max()
        .unwrap_or(target.width as i32);
    let max_y = all_monitors
        .iter()
        .map(|m| m.y + m.height as i32)
        .max()
        .unwrap_or(target.height as i32);

    let total_logical_w = (max_x - min_x).max(1) as f64;
    let total_logical_h = (max_y - min_y).max(1) as f64;

    let scale_x = image.width() as f64 / total_logical_w;
    let scale_y = image.height() as f64 / total_logical_h;

    let rel_x = (target.x - min_x) as f64;
    let rel_y = (target.y - min_y) as f64;

    let crop_x = (rel_x * scale_x).round() as u32;
    let crop_y = (rel_y * scale_y).round() as u32;
    let crop_w = (target.width as f64 * scale_x).round() as u32;
    let crop_h = (target.height as f64 * scale_y).round() as u32;

    // Bounds safety clamping
    if crop_x >= image.width() || crop_y >= image.height() {
        return image;
    }

    let final_w = crop_w.min(image.width() - crop_x);
    let final_h = crop_h.min(image.height() - crop_y);

    if final_w == 0 || final_h == 0 {
        return image;
    }

    println!(
        "[edged] Multi-monitor: cropped screenshot to target monitor '{}' ({}x{} at +{},+{})",
        target.name, final_w, final_h, crop_x, crop_y
    );

    image::imageops::crop_imm(&image, crop_x, crop_y, final_w, final_h).to_image()
}
