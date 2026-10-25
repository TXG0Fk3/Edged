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

/// Executa a captura de tela de acordo com o método configurado
pub fn capture_screen(config: &Config) -> Result<RgbaImage, CaptureError> {
    match config.general.capture_method {
        CaptureMethod::Grim => capture_via_grim(),
        CaptureMethod::Portal => capture_via_portal(),
        CaptureMethod::Command => {
            if config.general.custom_capture_command.is_empty() {
                Err(CaptureError::CommandFailed(
                    "custom_capture_command não configurado".to_string(),
                ))
            } else {
                capture_via_custom_command(&config.general.custom_capture_command)
            }
        }
        CaptureMethod::Auto => {
            // Tenta 'grim' primeiro (comum e instantâneo em wlroots/Hyprland/Sway)
            match capture_via_grim() {
                Ok(img) => Ok(img),
                Err(grim_err) => {
                    // Fallback para xdg-desktop-portal (GNOME, KDE Plasma, etc.)
                    match capture_via_portal() {
                        Ok(img) => Ok(img),
                        Err(portal_err) => {
                            // Se houver comando customizado, tenta como último recurso
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

/// Captura a tela usando o utilitário `grim` (Wayland screencopy)
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
                    "Arquivo de saída do grim não foi gerado".to_string(),
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

/// Captura a tela via XDG Desktop Portal (org.freedesktop.portal.Screenshot)
pub fn capture_via_portal() -> Result<RgbaImage, CaptureError> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| CaptureError::PortalFailed(format!("Erro ao criar tokio runtime: {}", e)))?;

    let uri_string = rt.block_on(async {
        let request = Screenshot::request().interactive(false).modal(false);
        let response = request
            .send()
            .await
            .map_err(|e| format!("Falha na chamada do portal: {}", e))?
            .response()
            .map_err(|e| format!("Falha na resposta do portal: {}", e))?;
        Ok::<String, String>(response.uri().as_str().to_string())
    }).map_err(CaptureError::PortalFailed)?;

    // Converte URI file:// para caminho do sistema de arquivos
    let file_path = uri_to_path(&uri_string)
        .ok_or_else(|| CaptureError::PortalFailed(format!("URI inválido: {}", uri_string)))?;

    if !file_path.exists() {
        return Err(CaptureError::PortalFailed(format!(
            "Arquivo do screenshot não encontrado em {:?}",
            file_path
        )));
    }

    let img = image::open(&file_path)
        .map_err(|e| CaptureError::ImageDecodeFailed(e.to_string()))?;

    // Remove o arquivo de screenshot do portal para não poluir a pasta do usuário
    let _ = fs::remove_file(&file_path);

    Ok(img.to_rgba8())
}

/// Executa um comando de captura de tela personalizado
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
            "Comando terminou com código: {:?}",
            status.code()
        )));
    }

    if !temp_path.exists() {
        return Err(CaptureError::CommandFailed(
            "Comando customizado não gerou o arquivo esperado".to_string(),
        ));
    }

    let img = image::open(&temp_path)
        .map_err(|e| CaptureError::ImageDecodeFailed(e.to_string()))?;
    let _ = fs::remove_file(&temp_path);

    Ok(img.to_rgba8())
}

/// Converte um URI file:// para PathBuf com decodificação de porcentagem
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
