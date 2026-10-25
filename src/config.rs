use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DefaultMode {
    Bounds,
    Spacing,
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CaptureMethod {
    Auto,
    Grim,
    Portal,
    Command,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SecondaryUnit {
    None,
    Inch,
    Cm,
    Mm,
    Pt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CopyFormat {
    Compact,
    Verbose,
    Css,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub default_mode: DefaultMode,
    pub capture_method: CaptureMethod,
    pub custom_capture_command: String,
    pub close_on_copy: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            default_mode: DefaultMode::Spacing,
            capture_method: CaptureMethod::Auto,
            custom_capture_command: String::new(),
            close_on_copy: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulerConfig {
    pub initial_tolerance: u8,
    pub tolerance_step: u8,
    pub draw_feet: bool,
    pub feet_size: f64,
    pub line_thickness: f64,
    pub draw_crosshair: bool,
}

impl Default for RulerConfig {
    fn default() -> Self {
        Self {
            initial_tolerance: 15,
            tolerance_step: 2,
            draw_feet: true,
            feet_size: 10.0,
            line_thickness: 2.0,
            draw_crosshair: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundsConfig {
    pub shift_pins_bounds: bool,
    pub border_thickness: f64,
}

impl Default for BoundsConfig {
    fn default() -> Self {
        Self {
            shift_pins_bounds: true,
            border_thickness: 2.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    pub overlay_opacity: f64,
    pub line_color: String,
    pub line_shadow_color: String,
    pub bounds_fill_color: String,
    pub bounds_border_color: String,
    pub label_bg_color: String,
    pub label_text_color: String,
    pub label_border_color: String,
    pub label_font_family: String,
    pub label_font_size: f64,
    pub label_border_radius: f64,
    pub label_padding_x: f64,
    pub label_padding_y: f64,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            overlay_opacity: 0.05,
            line_color: "#3B82F6".to_string(), // Modern vibrant blue
            line_shadow_color: "rgba(0, 0, 0, 0.6)".to_string(),
            bounds_fill_color: "rgba(59, 130, 246, 0.15)".to_string(),
            bounds_border_color: "#3B82F6".to_string(),
            label_bg_color: "rgba(15, 23, 42, 0.90)".to_string(),
            label_text_color: "#F8FAFC".to_string(),
            label_border_color: "#475569".to_string(),
            label_font_family: "sans-serif".to_string(),
            label_font_size: 12.0,
            label_border_radius: 5.0,
            label_padding_x: 7.0,
            label_padding_y: 3.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitsConfig {
    pub show_pixel_unit: bool,
    pub secondary_unit: SecondaryUnit,
    pub screen_dpi: f64,
}

impl Default for UnitsConfig {
    fn default() -> Self {
        Self {
            show_pixel_unit: true,
            secondary_unit: SecondaryUnit::None,
            screen_dpi: 96.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardConfig {
    pub format: CopyFormat,
}

impl Default for ClipboardConfig {
    fn default() -> Self {
        Self {
            format: CopyFormat::Compact,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub ruler: RulerConfig,
    #[serde(default)]
    pub bounds: BoundsConfig,
    #[serde(default)]
    pub appearance: AppearanceConfig,
    #[serde(default)]
    pub units: UnitsConfig,
    #[serde(default)]
    pub clipboard: ClipboardConfig,
}

pub const DEFAULT_CONFIG_COMMENTED: &str = r##"# ==============================================================================
# Edged - Screen Ruler para Wayland (Hyprland, GNOME, Sway, etc.)
# Arquivo de configuração: ~/.config/edged/config.toml
# ==============================================================================

[general]
# Modo padrão ao iniciar sem argumentos: "bounds", "spacing", "horizontal", "vertical"
default_mode = "spacing"

# Método de captura de tela: "auto", "grim", "portal", "command"
# - "auto": Tenta 'grim' primeiro (wlroots / Hyprland / Sway), com fallback para xdg-desktop-portal (GNOME / KDE)
# - "grim": Força uso do 'grim'
# - "portal": Força uso do portal XDG Desktop (PipeWire / Screenshot portal)
# - "command": Executa um comando customizado definido abaixo
capture_method = "auto"

# Comando customizado caso capture_method = "command"
# Use %s onde o caminho do arquivo temporário PNG de saída deve ser inserido.
# Exemplo: "wayshot --file %s"
custom_capture_command = ""

# Se deve fechar o aplicativo imediatamente após copiar a medida com o clique esquerdo
close_on_copy = true

[ruler]
# Tolerância inicial para edge detection (em porcentagem: 1 a 100)
# Quanto menor, mais sensível a pequenas variações de cor.
initial_tolerance = 15

# Variação da tolerância a cada "tick" de scroll da roda do mouse
tolerance_step = 2

# Desenhar "pés" (linhas perpendiculares nas pontas das réguas indicando os limites exatos)
draw_feet = true

# Tamanho dos pés em pixels
feet_size = 10.0

# Espessura da linha de medição
line_thickness = 2.0

# Exibir mira central sutil na posição exata do cursor
draw_crosshair = true

[bounds]
# No modo Bounds, manter Shift pressionado fixa retângulos anteriores na tela
shift_pins_bounds = true

# Espessura da borda do retângulo no modo Bounds
border_thickness = 2.0

[appearance]
# Opacidade do fundo escurecido da tela (0.0 = totalmente transparente, 0.05 = leve escurecimento)
overlay_opacity = 0.05

# Cores no formato hexadecimal "#RRGGBB", "#RRGGBBAA" ou "rgba(r, g, b, a)"
line_color = "#3B82F6"
line_shadow_color = "rgba(0, 0, 0, 0.6)"

bounds_fill_color = "rgba(59, 130, 246, 0.15)"
bounds_border_color = "#3B82F6"

# Estilo das etiquetas numéricas (badges)
label_bg_color = "rgba(15, 23, 42, 0.90)"
label_text_color = "#F8FAFC"
label_border_color = "#475569"
label_font_family = "sans-serif"
label_font_size = 12.0
label_border_radius = 5.0
label_padding_x = 7.0
label_padding_y = 3.5

[units]
# Exibir o sufixo "px" nas etiquetas
show_pixel_unit = true

# Unidade secundária opcional: "none", "inch", "cm", "mm", "pt"
secondary_unit = "none"

# DPI para cálculo da unidade secundária
screen_dpi = 96.0

[clipboard]
# Formato do texto copiado ao clicar:
# - "compact": ex: "1920x1080", "120"
# - "verbose": ex: "1920 × 1080 px", "120 px"
# - "css":     ex: "width: 1920px; height: 1080px;", "width: 120px;"
format = "compact"
"##;

impl Config {
    /// Determina o caminho padrão de configuração: ~/.config/edged/config.toml
    pub fn default_path() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("edged").join("config.toml")
        } else {
            PathBuf::from(".config/edged/config.toml")
        }
    }

    /// Carrega as configurações a partir do caminho informado ou do caminho padrão.
    /// Se o arquivo não existir, cria o diretório e o arquivo com os valores padrão comentados.
    pub fn load_or_create(custom_path: Option<&Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let path = match custom_path {
            Some(p) => p.to_path_buf(),
            None => Self::default_path(),
        };

        if !path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&path, DEFAULT_CONFIG_COMMENTED)?;
            log_info(&format!("Arquivo de configuração padrão criado em {:?}", path));
            let cfg: Config = toml::from_str(DEFAULT_CONFIG_COMMENTED)?;
            return Ok(cfg);
        }

        let content = fs::read_to_string(&path)?;
        match toml::from_str::<Config>(&content) {
            Ok(cfg) => Ok(cfg),
            Err(err) => {
                eprintln!(
                    "Aviso: Erro ao analisar {:?}: {}. Usando configurações padrão.",
                    path, err
                );
                let fallback: Config = toml::from_str(DEFAULT_CONFIG_COMMENTED)?;
                Ok(fallback)
            }
        }
    }
}

/// Utilitário simples para parsear cores em tuplas RGBA (0.0 a 1.0) para uso com Cairo.
pub fn parse_color(color_str: &str) -> (f64, f64, f64, f64) {
    let s = color_str.trim();

    if s.starts_with("rgba") || s.starts_with("rgb") {
        if let Some(start) = s.find('(') {
            if let Some(end) = s.rfind(')') {
                let parts: Vec<&str> = s[start + 1..end].split(',').map(|p| p.trim()).collect();
                if parts.len() >= 3 {
                    let r = parts[0].parse::<f64>().unwrap_or(0.0) / 255.0;
                    let g = parts[1].parse::<f64>().unwrap_or(0.0) / 255.0;
                    let b = parts[2].parse::<f64>().unwrap_or(0.0) / 255.0;
                    let a = if parts.len() >= 4 {
                        parts[3].parse::<f64>().unwrap_or(1.0)
                    } else {
                        1.0
                    };
                    return (r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0), a.clamp(0.0, 1.0));
                }
            }
        }
    }

    if s.starts_with('#') {
        let hex = &s[1..];
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap_or(0) as f64 / 255.0;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap_or(0) as f64 / 255.0;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap_or(0) as f64 / 255.0;
                return (r, g, b, 1.0);
            }
            4 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap_or(0) as f64 / 255.0;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap_or(0) as f64 / 255.0;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap_or(0) as f64 / 255.0;
                let a = u8::from_str_radix(&hex[3..4].repeat(2), 16).unwrap_or(255) as f64 / 255.0;
                return (r, g, b, a);
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f64 / 255.0;
                let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f64 / 255.0;
                let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f64 / 255.0;
                return (r, g, b, 1.0);
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f64 / 255.0;
                let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f64 / 255.0;
                let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f64 / 255.0;
                let a = u8::from_str_radix(&hex[6..8], 16).unwrap_or(255) as f64 / 255.0;
                return (r, g, b, a);
            }
            _ => {}
        }
    }

    // Cor padrão caso a formatação falhe (azul padrão)
    (0.23, 0.51, 0.96, 1.0)
}

fn log_info(msg: &str) {
    println!("[edged] {}", msg);
}
