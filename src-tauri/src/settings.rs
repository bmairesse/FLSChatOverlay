use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

/// Tudo que o app persiste em disco.
///
/// Gravado como JSON em `%APPDATA%/<identifier>/settings.json` (app config dir).
/// Nenhum campo aqui é credencial: a leitura do chat é anônima.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Nome do canal da Twitch, sem `#` e sem URL. Ex.: "gaules".
    pub channel: String,
    /// Opacidade do conteúdo do overlay, 0.0 a 1.0.
    pub opacity: f64,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub font_size: u32,
    /// Quantas mensagens ficam na tela antes das antigas serem descartadas.
    pub max_messages: u32,
    /// Se o overlay começa visível.
    pub overlay_visible: bool,
    /// Segundos sem movimento até o aviso visual do modo mover.
    pub idle_warning_secs: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            channel: String::new(),
            opacity: 0.85,
            x: 40,
            y: 40,
            width: 380,
            height: 600,
            font_size: 14,
            max_messages: 80,
            overlay_visible: true,
            idle_warning_secs: 5,
        }
    }
}

impl Settings {
    /// Corrige valores fora de faixa vindos de um JSON editado à mão.
    pub fn sanitize(&mut self) {
        self.channel = normalize_channel(&self.channel);
        self.opacity = self.opacity.clamp(0.05, 1.0);
        self.width = self.width.clamp(160, 4000);
        self.height = self.height.clamp(120, 4000);
        self.font_size = self.font_size.clamp(8, 48);
        self.max_messages = self.max_messages.clamp(10, 500);
        self.idle_warning_secs = self.idle_warning_secs.clamp(1, 120);
    }
}

/// Aceita o que o usuário costuma colar (URL, `#canal`, maiúsculas) e devolve
/// só o nome do canal em minúsculas, que é o que o IRC espera.
pub fn normalize_channel(raw: &str) -> String {
    let mut s = raw.trim().to_lowercase();
    for prefix in [
        "https://www.twitch.tv/",
        "http://www.twitch.tv/",
        "https://twitch.tv/",
        "http://twitch.tv/",
        "www.twitch.tv/",
        "twitch.tv/",
    ] {
        if let Some(rest) = s.strip_prefix(prefix) {
            s = rest.to_string();
            break;
        }
    }
    // Descarta querystring/path residual de um link colado.
    s = s
        .split(['/', '?', '#'])
        .find(|part| !part.is_empty())
        .unwrap_or("")
        .to_string();
    s.retain(|c| c.is_ascii_alphanumeric() || c == '_');
    s
}

pub fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("não foi possível resolver o config dir: {e}"))?;
    Ok(dir.join("settings.json"))
}

pub fn load(app: &AppHandle) -> Settings {
    let mut settings = settings_path(app)
        .ok()
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|raw| serde_json::from_str::<Settings>(&raw).ok())
        .unwrap_or_default();
    settings.sanitize();
    settings
}

pub fn save(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    let path = settings_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("criando {}: {e}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("serializando settings: {e}"))?;
    fs::write(&path, json).map_err(|e| format!("gravando {}: {e}", path.display()))
}
