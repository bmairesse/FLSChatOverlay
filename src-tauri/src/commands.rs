//! Comandos expostos às webviews (config e overlay).

use serde::Deserialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::overlay::{self, AppState};
use crate::settings::Settings;

/// Atualização parcial: a janela de config manda só o que mudou.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct SettingsPatch {
    pub channel: Option<String>,
    pub opacity: Option<f64>,
    pub font_size: Option<u32>,
    pub text_color: Option<String>,
    pub name_outline_color: Option<String>,
    pub highlight_channel_mentions: Option<bool>,
    pub max_messages: Option<u32>,
    pub show_header: Option<bool>,
    pub idle_warning_secs: Option<u64>,
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Settings {
    overlay::current_settings(&app)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, patch: SettingsPatch) -> Result<Settings, String> {
    {
        let state = app.state::<AppState>();
        let mut s = state.settings.lock().unwrap();

        if let Some(v) = patch.channel {
            s.channel = v;
        }
        if let Some(v) = patch.opacity {
            s.opacity = v;
        }
        if let Some(v) = patch.font_size {
            s.font_size = v;
        }
        if let Some(v) = patch.text_color {
            s.text_color = v;
        }
        if let Some(v) = patch.name_outline_color {
            s.name_outline_color = v;
        }
        if let Some(v) = patch.highlight_channel_mentions {
            s.highlight_channel_mentions = v;
        }
        if let Some(v) = patch.max_messages {
            s.max_messages = v;
        }
        if let Some(v) = patch.show_header {
            s.show_header = v;
        }
        if let Some(v) = patch.idle_warning_secs {
            s.idle_warning_secs = v;
        }
        s.sanitize();
    }

    overlay::persist(&app)?;
    let updated = overlay::current_settings(&app);
    let _ = app.emit(overlay::EV_SETTINGS, updated.clone());
    Ok(updated)
}

#[tauri::command]
pub fn set_move_mode(app: AppHandle, active: bool) -> Result<(), String> {
    overlay::set_move_mode(&app, active)
}

#[tauri::command]
pub fn toggle_move_mode(app: AppHandle) -> Result<(), String> {
    overlay::toggle_move_mode(&app)
}

#[tauri::command]
pub fn is_move_mode(app: AppHandle) -> bool {
    overlay::is_move_mode(&app)
}

/// Chamado pelo overlay a cada movimento do mouse sobre a janela.
/// Reinicia o watchdog de inatividade.
#[tauri::command]
pub fn notify_overlay_activity(app: AppHandle) {
    if overlay::is_move_mode(&app) {
        overlay::note_activity(&app);
    }
}

#[tauri::command]
pub fn set_overlay_visible(app: AppHandle, visible: bool) -> Result<(), String> {
    overlay::set_overlay_visible(&app, visible)
}

#[tauri::command]
pub fn is_overlay_visible(app: AppHandle) -> bool {
    overlay::overlay_window(&app)
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false)
}

#[tauri::command]
pub fn open_config(app: AppHandle) -> Result<(), String> {
    overlay::open_config(&app)
}

/// Caminho do arquivo de configuração, mostrado na UI para o usuário poder
/// auditar/editar/apagar o que o app guarda.
#[tauri::command]
pub fn settings_file_path(app: AppHandle) -> Result<String, String> {
    crate::settings::settings_path(&app).map(|p| p.display().to_string())
}
