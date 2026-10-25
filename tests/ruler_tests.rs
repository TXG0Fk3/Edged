use edged::config::{parse_color, Config, CopyFormat, DEFAULT_CONFIG_COMMENTED};
use edged::monitor::{crop_image_if_needed, MonitorInfo, MonitorTarget};
use edged::ruler::bounds::Rect;
use edged::ruler::{EdgeDetector, RulerMode};
use edged::ui::matches_key;
use egui::Key;
use image::{Rgba, RgbaImage};
use std::sync::Arc;

#[test]
fn test_default_config_validity() {
    let cfg: Config = toml::from_str(DEFAULT_CONFIG_COMMENTED).expect("Default config must parse");
    assert_eq!(cfg.ruler.initial_tolerance, 15);
    assert_eq!(cfg.ruler.tolerance_step, 2);
    assert!(cfg.ruler.draw_feet);
    assert!(cfg.bounds.shift_pins_bounds);
    assert!(cfg.general.close_on_copy);
    assert_eq!(cfg.shortcuts.cycle_mode_key, "Tab");
    assert_eq!(cfg.shortcuts.mode_bounds_key, "1");
    assert_eq!(cfg.shortcuts.mode_spacing_key, "2");
    assert!(cfg.shortcuts.shift_scroll_cycles_mode);
    assert!(cfg.shortcuts.ctrl_scroll_cycles_mode);
    assert_eq!(cfg.monitor.target, MonitorTarget::Focused);
    assert!(cfg.monitor.crop_to_monitor);
    assert!(cfg.logging.enabled);
    assert_eq!(cfg.logging.retention_days, 4);
    assert_eq!(cfg.logging.level, "info");
    assert!(cfg.logging.log_to_console);
    assert!(cfg.logging.log_to_file);
}

#[test]
fn test_load_or_create_config() {
    let temp_dir = std::env::temp_dir().join(format!("edged_test_{}", std::process::id()));
    let config_file = temp_dir.join("config.toml");

    let cfg = Config::load_or_create(Some(&config_file)).expect("Should create config");
    assert!(config_file.exists());
    assert_eq!(cfg.ruler.initial_tolerance, 15);

    // Clean up
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_mode_cycling() {
    assert_eq!(RulerMode::Spacing.next(), RulerMode::Bounds);
    assert_eq!(RulerMode::Bounds.next(), RulerMode::Horizontal);
    assert_eq!(RulerMode::Horizontal.next(), RulerMode::Vertical);
    assert_eq!(RulerMode::Vertical.next(), RulerMode::Spacing);

    assert_eq!(RulerMode::Spacing.prev(), RulerMode::Vertical);
    assert_eq!(RulerMode::Vertical.prev(), RulerMode::Horizontal);
    assert_eq!(RulerMode::Horizontal.prev(), RulerMode::Bounds);
    assert_eq!(RulerMode::Bounds.prev(), RulerMode::Spacing);
}

#[test]
fn test_key_matching() {
    assert!(matches_key(&Key::Tab, "Tab"));
    assert!(matches_key(&Key::Tab, "tab"));

    assert!(matches_key(&Key::Num1, "1"));
    assert!(matches_key(&Key::Num2, "2"));
    assert!(matches_key(&Key::B, "b"));
    assert!(matches_key(&Key::B, "B"));
    assert!(matches_key(&Key::S, "s"));
    assert!(matches_key(&Key::S, "S"));
    assert!(!matches_key(&Key::S, "b"));
}

#[test]
fn test_color_parsing() {
    assert_eq!(parse_color("#FFF"), (1.0, 1.0, 1.0, 1.0));
    assert_eq!(parse_color("#000"), (0.0, 0.0, 0.0, 1.0));
    assert_eq!(parse_color("#FF0000"), (1.0, 0.0, 0.0, 1.0));
    assert_eq!(parse_color("#00FF0080"), (0.0, 1.0, 0.0, 128.0 / 255.0));
    assert_eq!(
        parse_color("rgba(255, 0, 0, 0.5)"),
        (1.0, 0.0, 0.0, 0.5)
    );
    assert_eq!(parse_color("rgb(0, 255, 0)"), (0.0, 1.0, 0.0, 1.0));
}

#[test]
fn test_edge_detection_box() {
    // Create 100x100 image with black background (0,0,0)
    // and a red box/button (255,0,0) from x=20..=79 (width 60) and y=30..=69 (height 40)
    let mut img = RgbaImage::from_pixel(100, 100, Rgba([0, 0, 0, 255]));

    for y in 30..=69 {
        for x in 20..=79 {
            img.put_pixel(x, y, Rgba([255, 0, 0, 255]));
        }
    }

    let detector = EdgeDetector::new(Arc::new(img));

    // Cursor inside red box at (50, 50)
    let m = detector.detect_edges(50, 50, 15);
    assert_eq!(m.left_x, 20);
    assert_eq!(m.right_x, 79);
    assert_eq!(m.top_y, 30);
    assert_eq!(m.bottom_y, 69);
    assert_eq!(m.width, 60);
    assert_eq!(m.height, 40);

    // Cursor on black background at (10, 50)
    // Should stop at left edge of the screen (x=0) and left boundary of red box (x=19)
    let m_bg = detector.detect_edges(10, 50, 15);
    assert_eq!(m_bg.left_x, 0);
    assert_eq!(m_bg.right_x, 19);
    assert_eq!(m_bg.width, 20);
}

#[test]
fn test_edge_detection_tolerance() {
    // Image with linear gradient
    let mut img = RgbaImage::new(100, 100);
    for y in 0..100 {
        for x in 0..100 {
            let val = (x * 2) as u8; // 0 to 198
            img.put_pixel(x, y, Rgba([val, val, val, 255]));
        }
    }

    let detector = EdgeDetector::new(Arc::new(img));

    // Low tolerance (5%): narrow horizontal extent
    let m_low = detector.detect_edges(50, 50, 5);

    // High tolerance (30%): wide horizontal extent
    let m_high = detector.detect_edges(50, 50, 30);

    assert!(m_high.width > m_low.width);
}

#[test]
fn test_rect_bounds() {
    let r = Rect::from_points(10.0, 20.0, 60.0, 80.0);
    assert_eq!(r.x, 10.0);
    assert_eq!(r.y, 20.0);
    assert_eq!(r.width, 51.0);
    assert_eq!(r.height, 61.0);

    // Drag in reverse direction (bottom-right to top-left)
    let r2 = Rect::from_points(60.0, 80.0, 10.0, 20.0);
    assert_eq!(r2.x, 10.0);
    assert_eq!(r2.y, 20.0);
    assert_eq!(r2.width, 51.0);
    assert_eq!(r2.height, 61.0);
}

#[test]
fn test_clipboard_formatting() {
    use edged::clipboard::format_measurement;

    let text_compact = format_measurement(RulerMode::Spacing, 120, 80, CopyFormat::Compact);
    assert_eq!(text_compact, "120x80");

    let text_verbose = format_measurement(RulerMode::Spacing, 120, 80, CopyFormat::Verbose);
    assert_eq!(text_verbose, "120 × 80 px");

    let text_css = format_measurement(RulerMode::Spacing, 120, 80, CopyFormat::Css);
    assert_eq!(text_css, "width: 120px; height: 80px;");

    let text_h = format_measurement(RulerMode::Horizontal, 250, 50, CopyFormat::Compact);
    assert_eq!(text_h, "250");

    let text_v = format_measurement(RulerMode::Vertical, 50, 300, CopyFormat::Verbose);
    assert_eq!(text_v, "300 px");
}

#[test]
fn test_egui_color_image_conversion() {
    let mut img = RgbaImage::new(10, 10);
    img.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
    let size = [img.width() as usize, img.height() as usize];
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw());
    assert_eq!(color_image.size, [10, 10]);
}

#[test]
fn test_expand_command_placeholders() {
    use edged::capture::expand_command_placeholders;
    use std::path::Path;

    let target = Path::new("/tmp/test_dir/capture_123.png");

    // Standard %s
    let cmd1 = expand_command_placeholders("grim %s", target, None);
    assert_eq!(cmd1, "grim /tmp/test_dir/capture_123.png");

    // %path alias
    let cmd2 = expand_command_placeholders("wayshot %path", target, None);
    assert_eq!(cmd2, "wayshot /tmp/test_dir/capture_123.png");

    // Directory (%d) and filename (%n) for tools like hyprshot
    let cmd3 = expand_command_placeholders("hyprshot -m output -s -o %d -f %n", target, None);
    assert_eq!(
        cmd3,
        "hyprshot -m output -s -o /tmp/test_dir -f capture_123.png"
    );

    // Named %dir and %filename aliases
    let cmd4 = expand_command_placeholders("custom --dir %dir --file %filename", target, None);
    assert_eq!(
        cmd4,
        "custom --dir /tmp/test_dir --file capture_123.png"
    );

    // Fallback when no placeholders are present (appends target path)
    let cmd5 = expand_command_placeholders("grim", target, None);
    assert_eq!(cmd5, "grim /tmp/test_dir/capture_123.png");

    // Monitor placeholders: %o, %w, %h, %x, %y
    let monitor = MonitorInfo {
        name: "DP-1".to_string(),
        x: 1920,
        y: 0,
        width: 2560,
        height: 1440,
        scale: 1.0,
        is_primary: false,
        is_focused: true,
    };

    let cmd6 = expand_command_placeholders("grim -o %o %s", target, Some(&monitor));
    assert_eq!(cmd6, "grim -o DP-1 /tmp/test_dir/capture_123.png");

    let cmd7 = expand_command_placeholders("grim -g \"%x,%y %wx%h\" %s", target, Some(&monitor));
    assert_eq!(cmd7, "grim -g \"1920,0 2560x1440\" /tmp/test_dir/capture_123.png");
}

#[test]
fn test_monitor_target_parsing() {
    assert_eq!("focused".parse::<MonitorTarget>().unwrap(), MonitorTarget::Focused);
    assert_eq!("focus".parse::<MonitorTarget>().unwrap(), MonitorTarget::Focused);
    assert_eq!("all".parse::<MonitorTarget>().unwrap(), MonitorTarget::All);
    assert_eq!("primary".parse::<MonitorTarget>().unwrap(), MonitorTarget::Primary);
    assert_eq!("DP-1".parse::<MonitorTarget>().unwrap(), MonitorTarget::Named("DP-1".to_string()));
    assert_eq!("HDMI-A-1".parse::<MonitorTarget>().unwrap(), MonitorTarget::Named("HDMI-A-1".to_string()));
}

#[test]
fn test_monitor_image_crop() {
    // Simulate 2 monitors:
    // Mon 1: 1920x1080 at (0, 0), black background (0, 0, 0)
    // Mon 2: 1920x1080 at (1920, 0), red background (255, 0, 0)
    // Total desktop image: 3840x1080
    let mut desktop = RgbaImage::new(3840, 1080);
    for y in 0..1080 {
        for x in 0..1920 {
            desktop.put_pixel(x, y, Rgba([0, 0, 0, 255]));
        }
        for x in 1920..3840 {
            desktop.put_pixel(x, y, Rgba([255, 0, 0, 255]));
        }
    }

    let mon1 = MonitorInfo {
        name: "eDP-1".to_string(),
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
        scale: 1.0,
        is_primary: true,
        is_focused: false,
    };

    let mon2 = MonitorInfo {
        name: "DP-1".to_string(),
        x: 1920,
        y: 0,
        width: 1920,
        height: 1080,
        scale: 1.0,
        is_primary: false,
        is_focused: true,
    };

    let all_monitors = vec![mon1.clone(), mon2.clone()];

    // Crop to focused monitor (mon2, which is all red)
    let cropped = crop_image_if_needed(desktop, &mon2, &all_monitors, true);
    assert_eq!(cropped.width(), 1920);
    assert_eq!(cropped.height(), 1080);
    // Verify pixels are all red from monitor 2
    assert_eq!(cropped.get_pixel(0, 0), &Rgba([255, 0, 0, 255]));
    assert_eq!(cropped.get_pixel(1919, 1079), &Rgba([255, 0, 0, 255]));
}

#[test]
fn test_single_monitor_portal_padding_trim() {
    let mut oversized = RgbaImage::new(2400, 1350);
    oversized.put_pixel(100, 100, Rgba([0, 255, 0, 255]));

    let target = MonitorInfo {
        name: "DP-1".to_string(),
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
        scale: 1.25,
        is_primary: true,
        is_focused: true,
    };

    let all_monitors = vec![target.clone()];
    let trimmed = crop_image_if_needed(oversized, &target, &all_monitors, true);
    assert_eq!(trimmed.width(), 1920);
    assert_eq!(trimmed.height(), 1080);
    assert_eq!(trimmed.get_pixel(100, 100), &Rgba([0, 255, 0, 255]));
}

#[test]
fn test_find_newest_image_in_dir() {
    use edged::capture::find_newest_image_in_dir;
    use std::fs;

    let temp_dir = std::env::temp_dir().join(format!("edged_img_test_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).expect("Create temp dir");

    let file1 = temp_dir.join("older.png");
    let file2 = temp_dir.join("newer.png");
    let non_img = temp_dir.join("log.txt");

    fs::write(&file1, b"old").expect("write file1");
    std::thread::sleep(std::time::Duration::from_millis(50));
    fs::write(&file2, b"new").expect("write file2");
    fs::write(&non_img, b"not an image").expect("write non_img");

    let newest = find_newest_image_in_dir(&temp_dir);
    assert!(newest.is_some());
    assert_eq!(newest.unwrap().file_name().unwrap(), "newer.png");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_log_retention_prune_four_days() {
    use edged::logger::prune_old_logs;
    use std::fs;

    let temp_dir = std::env::temp_dir().join(format!("edged_log_prune_test_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).expect("Create temp dir");

    // Create log files with different date signatures
    let expired_log1 = temp_dir.join("edged-2020-01-01.log"); // Over 6 years old
    let expired_log2 = temp_dir.join("edged-2026-05-10.log"); // Months old
    let recent_log = temp_dir.join("edged-2026-10-25.log");  // Today
    let latest_symlink = temp_dir.join("latest.log");
    let non_log = temp_dir.join("keepme.txt");

    fs::write(&expired_log1, b"old log 1").expect("write expired1");
    fs::write(&expired_log2, b"old log 2").expect("write expired2");
    fs::write(&recent_log, b"today log").expect("write recent");
    fs::write(&latest_symlink, b"pointer to latest").expect("write latest");
    fs::write(&non_log, b"important notes").expect("write non_log");

    // Retention is strictly 4 days
    let (pruned_count, _) = prune_old_logs(&temp_dir, 4);

    assert_eq!(pruned_count, 2, "Expected 2 expired log files to be pruned");
    assert!(!expired_log1.exists(), "expired_log1 should be deleted");
    assert!(!expired_log2.exists(), "expired_log2 should be deleted");
    assert!(recent_log.exists(), "recent_log must be preserved");
    assert!(latest_symlink.exists(), "latest.log must not be deleted by retention pruning");
    assert!(non_log.exists(), "non_log file must not be touched");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_edge_detection_at_boundaries() {
    let mut img = RgbaImage::from_pixel(100, 100, Rgba([255, 255, 255, 255]));
    for y in 10..=20 {
        for x in 10..=20 {
            img.put_pixel(x, y, Rgba([0, 0, 0, 255]));
        }
    }
    let detector = EdgeDetector::new(Arc::new(img));

    // Test top-left corner (0, 0)
    let m_corner = detector.detect_edges(0, 0, 10);
    assert_eq!(m_corner.left_x, 0);
    assert_eq!(m_corner.top_y, 0);
    assert_eq!(m_corner.right_x, 99);

    // Test bottom-right corner (99, 99)
    let m_br = detector.detect_edges(99, 99, 10);
    assert_eq!(m_br.right_x, 99);
    assert_eq!(m_br.bottom_y, 99);

    // Test out-of-bounds cursor clamping gracefully without panicking
    let m_oob = detector.detect_edges(999, 999, 10);
    assert_eq!(m_oob.cursor_x, 99);
    assert_eq!(m_oob.cursor_y, 99);
}

#[test]
fn test_edge_detection_single_pixel_image() {
    let img = RgbaImage::from_pixel(1, 1, Rgba([100, 150, 200, 255]));
    let detector = EdgeDetector::new(Arc::new(img));

    let m = detector.detect_edges(0, 0, 15);
    assert_eq!(m.left_x, 0);
    assert_eq!(m.right_x, 0);
    assert_eq!(m.top_y, 0);
    assert_eq!(m.bottom_y, 0);
    assert_eq!(m.width, 1);
    assert_eq!(m.height, 1);
}

#[test]
fn test_edge_detection_uniform_canvas() {
    let img = RgbaImage::from_pixel(50, 50, Rgba([30, 40, 50, 255]));
    let detector = EdgeDetector::new(Arc::new(img));

    let m = detector.detect_edges(25, 25, 10);
    assert_eq!(m.left_x, 0);
    assert_eq!(m.right_x, 49);
    assert_eq!(m.top_y, 0);
    assert_eq!(m.bottom_y, 49);
    assert_eq!(m.width, 50);
    assert_eq!(m.height, 50);
}

#[test]
fn test_edge_detection_thin_lines_and_barriers() {
    let mut img = RgbaImage::from_pixel(100, 100, Rgba([255, 255, 255, 255]));
    // 1px black horizontal line at y = 40
    for x in 0..100 {
        img.put_pixel(x, 40, Rgba([0, 0, 0, 255]));
    }
    // 1px black vertical line at x = 60
    for y in 0..100 {
        img.put_pixel(60, y, Rgba([0, 0, 0, 255]));
    }

    let detector = EdgeDetector::new(Arc::new(img));

    // Cursor at (20, 20) inside the top-left quadrant [0..59, 0..39]
    let m = detector.detect_edges(20, 20, 15);
    assert_eq!(m.left_x, 0);
    assert_eq!(m.right_x, 59); // Stops right before vertical line at 60
    assert_eq!(m.top_y, 0);
    assert_eq!(m.bottom_y, 39); // Stops right before horizontal line at 40
    assert_eq!(m.width, 60);
    assert_eq!(m.height, 40);
}

#[test]
fn test_edge_measurement_empty() {
    use edged::ruler::EdgeMeasurement;
    let m = EdgeMeasurement::empty(42, 84);
    assert_eq!(m.cursor_x, 42);
    assert_eq!(m.cursor_y, 84);
    assert_eq!(m.left_x, 42);
    assert_eq!(m.right_x, 42);
    assert_eq!(m.top_y, 84);
    assert_eq!(m.bottom_y, 84);
    assert_eq!(m.width, 1);
    assert_eq!(m.height, 1);
}

#[test]
fn test_rect_contains_and_center() {
    let r = Rect::new(10.0, 20.0, 50.0, 30.0);
    assert_eq!(r.center(), (35.0, 35.0));

    // Points inside
    assert!(r.contains(10.0, 20.0)); // Top-left corner
    assert!(r.contains(60.0, 50.0)); // Bottom-right corner (10+50, 20+30)
    assert!(r.contains(35.0, 35.0)); // Center
    assert!(r.contains(25.0, 30.0)); // Arbitrary interior point

    // Points outside
    assert!(!r.contains(9.9, 20.0));  // Left
    assert!(!r.contains(60.1, 30.0)); // Right
    assert!(!r.contains(20.0, 19.9)); // Above
    assert!(!r.contains(20.0, 50.1)); // Below
}

#[test]
fn test_rect_single_point_drag() {
    let r = Rect::from_points(50.0, 50.0, 50.0, 50.0);
    assert_eq!(r.x, 50.0);
    assert_eq!(r.y, 50.0);
    assert_eq!(r.width, 1.0);
    assert_eq!(r.height, 1.0);
    assert_eq!(r.center(), (50.5, 50.5));
}

#[test]
fn test_resolve_target_monitor_fallbacks() {
    use edged::monitor::resolve_target_monitor;

    // Case 1: Empty list returns None
    assert_eq!(resolve_target_monitor(&MonitorTarget::Focused, &[]), None);

    let m1 = MonitorInfo {
        name: "eDP-1".to_string(),
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
        scale: 1.0,
        is_primary: false,
        is_focused: false,
    };

    let m2 = MonitorInfo {
        name: "DP-1".to_string(),
        x: 1920,
        y: 0,
        width: 2560,
        height: 1440,
        scale: 1.0,
        is_primary: true,
        is_focused: false,
    };

    let m3 = MonitorInfo {
        name: "HDMI-A-1".to_string(),
        x: 4480,
        y: 0,
        width: 1920,
        height: 1080,
        scale: 1.0,
        is_primary: false,
        is_focused: true,
    };

    let monitors = vec![m1.clone(), m2.clone(), m3.clone()];

    // Focused returns m3
    let res_focused = resolve_target_monitor(&MonitorTarget::Focused, &monitors);
    assert_eq!(res_focused.unwrap().name, "HDMI-A-1");

    // Primary returns m2
    let res_primary = resolve_target_monitor(&MonitorTarget::Primary, &monitors);
    assert_eq!(res_primary.unwrap().name, "DP-1");

    // Named matching (case-insensitive)
    let res_named = resolve_target_monitor(&MonitorTarget::Named("hdmi-a-1".to_string()), &monitors);
    assert_eq!(res_named.unwrap().name, "HDMI-A-1");

    // Named not found -> falls back to focused
    let res_unknown = resolve_target_monitor(&MonitorTarget::Named("VIRTUAL-99".to_string()), &monitors);
    assert_eq!(res_unknown.unwrap().name, "HDMI-A-1");

    // All target returns None
    assert_eq!(resolve_target_monitor(&MonitorTarget::All, &monitors), None);

    // Fallback when neither focused nor primary exist -> returns first
    let neutral1 = MonitorInfo {
        name: "A".to_string(),
        x: 0,
        y: 0,
        width: 100,
        height: 100,
        scale: 1.0,
        is_primary: false,
        is_focused: false,
    };
    let neutral2 = MonitorInfo {
        name: "B".to_string(),
        x: 100,
        y: 0,
        width: 100,
        height: 100,
        scale: 1.0,
        is_primary: false,
        is_focused: false,
    };
    let neutrals = vec![neutral1, neutral2];
    assert_eq!(resolve_target_monitor(&MonitorTarget::Focused, &neutrals).unwrap().name, "A");
    assert_eq!(resolve_target_monitor(&MonitorTarget::Primary, &neutrals).unwrap().name, "A");
}

#[test]
fn test_multi_monitor_vertical_crop() {
    // 2 stacked monitors (1920x1080 stacked vertically -> 1920x2160)
    let mut desktop = RgbaImage::new(1920, 2160);
    // Top monitor (0..1080) filled with blue
    for y in 0..1080 {
        for x in 0..1920 {
            desktop.put_pixel(x, y, Rgba([0, 0, 255, 255]));
        }
    }
    // Bottom monitor (1080..2160) filled with green
    for y in 1080..2160 {
        for x in 0..1920 {
            desktop.put_pixel(x, y, Rgba([0, 255, 0, 255]));
        }
    }

    let mon_top = MonitorInfo {
        name: "DP-1".to_string(),
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
        scale: 1.0,
        is_primary: true,
        is_focused: false,
    };

    let mon_bottom = MonitorInfo {
        name: "DP-2".to_string(),
        x: 0,
        y: 1080,
        width: 1920,
        height: 1080,
        scale: 1.0,
        is_primary: false,
        is_focused: true,
    };

    let all_monitors = vec![mon_top.clone(), mon_bottom.clone()];

    // Crop to bottom monitor
    let cropped = crop_image_if_needed(desktop.clone(), &mon_bottom, &all_monitors, true);
    assert_eq!(cropped.width(), 1920);
    assert_eq!(cropped.height(), 1080);
    assert_eq!(cropped.get_pixel(0, 0), &Rgba([0, 255, 0, 255]));
    assert_eq!(cropped.get_pixel(1919, 1079), &Rgba([0, 255, 0, 255]));

    // When crop is disabled, full composite is preserved
    let untouched = crop_image_if_needed(desktop, &mon_bottom, &all_monitors, false);
    assert_eq!(untouched.width(), 1920);
    assert_eq!(untouched.height(), 2160);
}

#[test]
fn test_color_parsing_resilience() {
    // 3-digit hex
    assert_eq!(parse_color("#123"), (0x11 as f64 / 255.0, 0x22 as f64 / 255.0, 0x33 as f64 / 255.0, 1.0));

    // 4-digit hex
    assert_eq!(parse_color("#1234"), (0x11 as f64 / 255.0, 0x22 as f64 / 255.0, 0x33 as f64 / 255.0, 0x44 as f64 / 255.0));

    // 6-digit hex
    assert_eq!(parse_color("#aabbcc"), (0xaa as f64 / 255.0, 0xbb as f64 / 255.0, 0xcc as f64 / 255.0, 1.0));

    // 8-digit hex
    assert_eq!(parse_color("#aabbccdd"), (0xaa as f64 / 255.0, 0xbb as f64 / 255.0, 0xcc as f64 / 255.0, 0xdd as f64 / 255.0));

    // RGB / RGBA variations with and without spaces
    assert_eq!(parse_color("rgb(10, 20, 30)"), (10.0 / 255.0, 20.0 / 255.0, 30.0 / 255.0, 1.0));
    assert_eq!(parse_color("rgba(10,20,30,0.5)"), (10.0 / 255.0, 20.0 / 255.0, 30.0 / 255.0, 0.5));

    // Clamping of out-of-range values
    assert_eq!(parse_color("rgba(300, -20, 255, 1.5)"), (1.0, 0.0, 1.0, 1.0));

    // Corrupted / invalid input returns fallback blue
    let fallback = (0.23, 0.51, 0.96, 1.0);
    assert_eq!(parse_color(""), fallback);
    assert_eq!(parse_color("not_a_color"), fallback);
    assert_eq!(parse_color("#GGGGGG"), fallback);
    assert_eq!(parse_color("#12"), fallback);
    assert_eq!(parse_color("rgba()"), fallback);
}

#[test]
fn test_clipboard_formatting_all_modes_and_formats() {
    use edged::clipboard::format_measurement;

    // Bounds mode
    assert_eq!(format_measurement(RulerMode::Bounds, 100, 200, CopyFormat::Compact), "100x200");
    assert_eq!(format_measurement(RulerMode::Bounds, 100, 200, CopyFormat::Verbose), "100 × 200 px");
    assert_eq!(format_measurement(RulerMode::Bounds, 100, 200, CopyFormat::Css), "width: 100px; height: 200px;");

    // Spacing mode
    assert_eq!(format_measurement(RulerMode::Spacing, 100, 200, CopyFormat::Compact), "100x200");
    assert_eq!(format_measurement(RulerMode::Spacing, 100, 200, CopyFormat::Verbose), "100 × 200 px");
    assert_eq!(format_measurement(RulerMode::Spacing, 100, 200, CopyFormat::Css), "width: 100px; height: 200px;");

    // Horizontal mode
    assert_eq!(format_measurement(RulerMode::Horizontal, 100, 200, CopyFormat::Compact), "100");
    assert_eq!(format_measurement(RulerMode::Horizontal, 100, 200, CopyFormat::Verbose), "100 px");
    assert_eq!(format_measurement(RulerMode::Horizontal, 100, 200, CopyFormat::Css), "width: 100px;");

    // Vertical mode
    assert_eq!(format_measurement(RulerMode::Vertical, 100, 200, CopyFormat::Compact), "200");
    assert_eq!(format_measurement(RulerMode::Vertical, 100, 200, CopyFormat::Verbose), "200 px");
    assert_eq!(format_measurement(RulerMode::Vertical, 100, 200, CopyFormat::Css), "height: 200px;");
}

#[test]
fn test_logger_level_parsing() {
    use edged::logger::parse_level_filter;
    use log::LevelFilter;

    assert_eq!(parse_level_filter("off"), LevelFilter::Off);
    assert_eq!(parse_level_filter("OFF"), LevelFilter::Off);
    assert_eq!(parse_level_filter("error"), LevelFilter::Error);
    assert_eq!(parse_level_filter("ERROR"), LevelFilter::Error);
    assert_eq!(parse_level_filter("warn"), LevelFilter::Warn);
    assert_eq!(parse_level_filter("warning"), LevelFilter::Warn);
    assert_eq!(parse_level_filter("WARN"), LevelFilter::Warn);
    assert_eq!(parse_level_filter("info"), LevelFilter::Info);
    assert_eq!(parse_level_filter("INFO"), LevelFilter::Info);
    assert_eq!(parse_level_filter("debug"), LevelFilter::Debug);
    assert_eq!(parse_level_filter("DEBUG"), LevelFilter::Debug);
    assert_eq!(parse_level_filter("trace"), LevelFilter::Trace);
    assert_eq!(parse_level_filter("TRACE"), LevelFilter::Trace);
    assert_eq!(parse_level_filter("unknown"), LevelFilter::Info);
    assert_eq!(parse_level_filter(""), LevelFilter::Info);
}

#[test]
fn test_logger_pruner_ignores_subdirectories_and_foreign_files() {
    use edged::logger::prune_old_logs;
    use std::fs;

    let temp_dir = std::env::temp_dir().join(format!("edged_sub_prune_test_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).expect("Create temp dir");

    let sub_dir = temp_dir.join("nested_folder");
    fs::create_dir_all(&sub_dir).expect("Create sub dir");

    let foreign_file = temp_dir.join("other-2020-01-01.log");
    let non_log_file = temp_dir.join("edged-notes.txt");
    fs::write(&foreign_file, b"foreign log").expect("write foreign");
    fs::write(&non_log_file, b"notes").expect("write notes");

    let (pruned, _) = prune_old_logs(&temp_dir, 4);
    assert_eq!(pruned, 0);
    assert!(sub_dir.exists(), "Subdirectory must never be deleted");
    assert!(foreign_file.exists(), "Foreign log prefix must not be touched");
    assert!(non_log_file.exists(), "Non-.log extension must not be touched");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_capture_error_display_formatting() {
    use edged::capture::CaptureError;

    let err1 = CaptureError::GrimFailed("exit code 1".to_string());
    assert_eq!(format!("{}", err1), "Grim capture failed: exit code 1");

    let err2 = CaptureError::PortalFailed("Access denied".to_string());
    assert_eq!(format!("{}", err2), "Desktop portal capture failed: Access denied");

    let err3 = CaptureError::CommandFailed("binary not found".to_string());
    assert_eq!(format!("{}", err3), "Custom capture command failed: binary not found");

    let err4 = CaptureError::ImageDecodeFailed("unexpected EOF".to_string());
    assert_eq!(format!("{}", err4), "Image decode failed: unexpected EOF");

    let err5 = CaptureError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, "file missing"));
    assert_eq!(format!("{}", err5), "IO error: file missing");

    let err6 = CaptureError::AllMethodsFailed;
    assert_eq!(format!("{}", err6), "All screen capture methods failed");
}

#[test]
fn test_config_deserialization_from_empty_string() {
    let cfg: Config = toml::from_str("").expect("Empty TOML string should deserialize using defaults");
    assert!(cfg.general.close_on_copy);
    assert_eq!(cfg.ruler.initial_tolerance, 15);
    assert_eq!(cfg.logging.retention_days, 4);
    assert!(cfg.monitor.crop_to_monitor);
}

#[test]
fn test_config_corrupted_fallback() {
    let temp_dir = std::env::temp_dir().join(format!("edged_corrupt_cfg_{}", std::process::id()));
    let config_file = temp_dir.join("config.toml");
    std::fs::create_dir_all(&temp_dir).expect("create dir");

    // Write completely invalid syntax
    std::fs::write(&config_file, b"[general\ninvalid = = =").expect("write corrupt cfg");

    let cfg = Config::load_or_create(Some(&config_file)).expect("Corrupted config should fallback to defaults");
    assert!(cfg.general.close_on_copy);
    assert_eq!(cfg.ruler.initial_tolerance, 15);
    assert_eq!(cfg.logging.retention_days, 4);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_ruler_mode_names() {
    assert_eq!(RulerMode::Bounds.name(), "Bounds (Rectangle)");
    assert_eq!(RulerMode::Spacing.name(), "Spacing (Cross)");
    assert_eq!(RulerMode::Horizontal.name(), "Horizontal");
    assert_eq!(RulerMode::Vertical.name(), "Vertical");
}
