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
    /// Cor do texto das mensagens, em hex `#rrggbb`. O nome de quem escreve não
    /// entra aqui: ele usa a cor que a própria pessoa escolheu na Twitch.
    pub text_color: String,
    /// Cor do contorno do nome, em hex `#rrggbb`. O contorno não pode ser
    /// desligado — é ele que mantém o nome legível sobre cenário claro.
    pub name_outline_color: String,
    /// Marca as linhas que citam `@canal`, como o chat do site faz. Mensagens
    /// realçadas com pontos do canal são marcadas independentemente disto.
    pub highlight_channel_mentions: bool,
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
            text_color: DEFAULT_TEXT_COLOR.to_string(),
            name_outline_color: DEFAULT_NAME_OUTLINE_COLOR.to_string(),
            highlight_channel_mentions: true,
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
        self.text_color = normalize_hex_color(&self.text_color, DEFAULT_TEXT_COLOR);
        self.name_outline_color =
            normalize_hex_color(&self.name_outline_color, DEFAULT_NAME_OUTLINE_COLOR);
        self.max_messages = self.max_messages.clamp(10, 500);
        self.idle_warning_secs = self.idle_warning_secs.clamp(1, 120);
    }
}

pub const DEFAULT_TEXT_COLOR: &str = "#ffffff";
pub const DEFAULT_NAME_OUTLINE_COLOR: &str = "#000000";

/// Devolve sempre `#rrggbb` minúsculo, caindo no padrão se o valor não for uma
/// cor hex válida. A forma curta `#abc` é expandida para `#aabbcc`.
///
/// Os seis dígitos não são preciosismo: o `<input type="color">` da janela de
/// configurações só aceita esse formato e cai para preto, calado, diante de
/// qualquer outro.
///
/// E validar não é só arrumar formatação — o valor vai parar direto em uma
/// custom property do CSS no overlay, então um settings.json editado à mão não
/// pode injetar texto arbitrário ali.
pub fn normalize_hex_color(raw: &str, fallback: &str) -> String {
    let trimmed = raw.trim().to_lowercase();
    let hex = trimmed.strip_prefix('#').unwrap_or(&trimmed);

    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return fallback.to_string();
    }

    match hex.len() {
        6 => format!("#{hex}"),
        3 => {
            let mut out = String::with_capacity(7);
            out.push('#');
            for c in hex.chars() {
                out.push(c);
                out.push(c);
            }
            out
        }
        _ => fallback.to_string(),
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
