use edged::config::{parse_color, Config, CopyFormat, DEFAULT_CONFIG_COMMENTED};
use edged::ruler::bounds::Rect;
use edged::ruler::{EdgeDetector, RulerMode};
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
    // Cria imagem 100x100 com fundo preto (0,0,0)
    // e um botão/caixa vermelha (255,0,0) de x=20..=79 (largura 60) e y=30..=69 (altura 40)
    let mut img = RgbaImage::from_pixel(100, 100, Rgba([0, 0, 0, 255]));

    for y in 30..=69 {
        for x in 20..=79 {
            img.put_pixel(x, y, Rgba([255, 0, 0, 255]));
        }
    }

    let detector = EdgeDetector::new(Arc::new(img));

    // Cursor dentro da caixa vermelha em (50, 50)
    let m = detector.detect_edges(50, 50, 15);
    assert_eq!(m.left_x, 20);
    assert_eq!(m.right_x, 79);
    assert_eq!(m.top_y, 30);
    assert_eq!(m.bottom_y, 69);
    assert_eq!(m.width, 60);
    assert_eq!(m.height, 40);

    // Cursor no fundo preto em (10, 50)
    // Deve parar na borda esquerda da caixa (x=19) e na borda esquerda da tela (x=0)
    let m_bg = detector.detect_edges(10, 50, 15);
    assert_eq!(m_bg.left_x, 0);
    assert_eq!(m_bg.right_x, 19);
    assert_eq!(m_bg.width, 20);
}

#[test]
fn test_edge_detection_tolerance() {
    // Imagem com gradiente suave
    let mut img = RgbaImage::new(100, 100);
    for y in 0..100 {
        for x in 0..100 {
            let val = (x * 2) as u8; // 0 a 198
            img.put_pixel(x, y, Rgba([val, val, val, 255]));
        }
    }

    let detector = EdgeDetector::new(Arc::new(img));

    // Com tolerância baixa (5%), o alcance horizontal é menor
    let m_low = detector.detect_edges(50, 50, 5);

    // Com tolerância alta (30%), o alcance horizontal é muito maior
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

    // Arrastar na direção oposta (de baixo-direita para cima-esquerda)
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
