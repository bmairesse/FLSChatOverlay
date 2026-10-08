//! Comandos expostos às webviews (config e overlay).

use serde::Deserialize;
use tauri::{AppHandle, Manager, Window};

use crate::overlay::{self, AppState};
use crate::settings::{self, Chat, Profile, Store};

/// Atualização parcial de um chat: a janela de config manda só o que mudou.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct ChatPatch {
    pub name: Option<String>,
    pub channel: Option<String>,
    pub mention: Option<String>,
    pub opacity: Option<f64>,
    pub font_size: Option<u32>,
    pub text_color: Option<String>,
    pub name_outline_color: Option<String>,
    pub highlight_channel_mentions: Option<bool>,
    pub max_messages: Option<u32>,
    pub show_header: Option<bool>,
    pub fade_secs: Option<u64>,
    pub fade_percent: Option<u32>,
}

/// Atualização parcial do que é do app inteiro, não de um chat.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct AppPatch {
    pub idle_warning_secs: Option<u64>,
}

/// Roda `f` sobre o estado, sanitiza e grava.
///
/// Todo comando que escreve passa por aqui: é o que garante que nunca exista um
/// estado salvo sem os invariantes do `sanitize()`.
fn apply<F>(app: &AppHandle, f: F) -> Result<(), String>
where
    F: FnOnce(&mut Store) -> Result<(), String>,
{
    {
        let state = app.state::<AppState>();
        let mut store = state.store.lock().unwrap();
        f(&mut store)?;
        store.sanitize();
    }
    overlay::persist(app)
}

/// Mudança que só mexe no conteúdo dos chats. Avisa as janelas que já existem.
fn mutate<F>(app: &AppHandle, f: F) -> Result<Store, String>
where
    F: FnOnce(&mut Store) -> Result<(), String>,
{
    apply(app, f)?;
    overlay::push_chats(app);
    Ok(overlay::current_store(app))
}

/// Mudança que muda *quais* janelas devem existir: adicionar ou remover chat,
/// trocar de perfil. Só estas passam pelo `sync_windows`, que abre e fecha
/// janela e remonta o menu da bandeja — trabalho caro demais para acontecer a
/// cada passo de um slider.
fn mutate_windows<F>(app: &AppHandle, f: F) -> Result<Store, String>
where
    F: FnOnce(&mut Store) -> Result<(), String>,
{
    apply(app, f)?;
    overlay::sync_windows(app)?;
    Ok(overlay::current_store(app))
}

#[tauri::command]
pub fn get_state(app: AppHandle) -> Store {
    overlay::current_store(&app)
}

/// A configuração do chat de quem está perguntando. O id sai do label da
/// janela, não de um argumento: assim um overlay não consegue ler ou pedir a
/// configuração de outro.
#[tauri::command]
pub fn get_chat(app: AppHandle, window: Window) -> Result<Chat, String> {
    let id = overlay::chat_id_of(window.label()).ok_or("essa janela não é um overlay")?;
    overlay::current_store(&app)
        .chat(id)
        .cloned()
        .ok_or_else(|| "chat não encontrado".to_string())
}

#[tauri::command]
pub fn save_chat(app: AppHandle, id: String, patch: ChatPatch) -> Result<Store, String> {
    // O nome do chat aparece no submenu "Centralizar na tela" da bandeja, e o
    // nome automático sai do canal — as duas mudanças obrigam a remontar o
    // menu. Lido antes do `mutate`, que consome o patch.
    let renamed = patch.name.is_some() || patch.channel.is_some();
    let updated = mutate(&app, |store| {
        let chat = store.chat_mut(&id).ok_or("chat não encontrado")?;
        if let Some(v) = patch.name {
            chat.name = v;
        }
        if let Some(v) = patch.channel {
            chat.channel = v;
        }
        if let Some(v) = patch.mention {
            chat.mention = v;
        }
        if let Some(v) = patch.opacity {
            chat.opacity = v;
        }
        if let Some(v) = patch.font_size {
            chat.font_size = v;
        }
        if let Some(v) = patch.text_color {
            chat.text_color = v;
        }
        if let Some(v) = patch.name_outline_color {
            chat.name_outline_color = v;
        }
        if let Some(v) = patch.highlight_channel_mentions {
            chat.highlight_channel_mentions = v;
        }
        if let Some(v) = patch.max_messages {
            chat.max_messages = v;
        }
        if let Some(v) = patch.show_header {
            chat.show_header = v;
        }
        if let Some(v) = patch.fade_secs {
            chat.fade_secs = v;
        }
        if let Some(v) = patch.fade_percent {
            chat.fade_percent = v;
        }
        Ok(())
    })?;
    if renamed {
        crate::tray::sync_menu(&app);
    }
    Ok(updated)
}

#[tauri::command]
pub fn save_app(app: AppHandle, patch: AppPatch) -> Result<Store, String> {
    mutate(&app, |store| {
        if let Some(v) = patch.idle_warning_secs {
            store.idle_warning_secs = v;
        }
        Ok(())
    })
}

/// Comandos que criam ou fecham janela são `async` de propósito.
///
/// Um comando síncrono roda na thread do event loop. Criar uma WebView2 espera
/// pela mensagem do Windows que anuncia a criação — mensagem que só essa mesma
/// thread despacharia. O resultado é um travamento seco: a janela nunca nasce e
/// o app para de responder. Sendo `async`, o Tauri roda o comando no runtime
/// assíncrono e a thread do event loop fica livre para despachar.
///
/// Nada aqui dentro tem `.await`: o `async` existe só para escolher a thread.
///
/// Adiciona um chat ao perfil ativo. Nasce ao lado do último, deslocado, para
/// não abrir exatamente em cima dele.
#[tauri::command]
pub async fn add_chat(app: AppHandle) -> Result<Store, String> {
    mutate_windows(&app, |store| {
        let chats = store.active_chats_mut();
        let previous = chats.last().cloned().unwrap_or_default();
        // Canal em branco é o próximo passo do usuário, não um palpite nosso.
        // O resto herda a aparência do chat anterior, para não obrigar a
        // reajustar cor, fonte e opacidade a cada chat novo.
        chats.push(Chat {
            id: settings::new_id(settings::ID_CHAT),
            name: format!("Chat {}", chats.len() + 1),
            channel: String::new(),
            x: previous.x + 40,
            y: previous.y + 40,
            visible: true,
            ..previous
        });
        Ok(())
    })
}

/// Remove um chat. O último de um perfil não sai: um perfil sem chat nenhum
/// não tem como ser configurado de volta pela interface.
#[tauri::command]
pub async fn remove_chat(app: AppHandle, id: String) -> Result<Store, String> {
    mutate_windows(&app, |store| {
        let chats = store.active_chats_mut();
        if chats.len() <= 1 {
            return Err("o perfil precisa de pelo menos um chat".into());
        }
        let before = chats.len();
        chats.retain(|c| c.id != id);
        if chats.len() == before {
            return Err("chat não encontrado".into());
        }
        Ok(())
    })
}

#[tauri::command]
pub fn set_chat_visible(app: AppHandle, id: String, visible: bool) -> Result<Store, String> {
    overlay::set_chat_visible(&app, &id, visible)?;
    Ok(overlay::current_store(&app))
}

#[tauri::command]
pub async fn add_profile(app: AppHandle, name: String) -> Result<Store, String> {
    mutate_windows(&app, |store| {
        let profile = Profile {
            id: settings::new_id(settings::ID_PROFILE),
            name,
            chats: vec![Chat::default()],
        };
        // Perfil novo entra em uso na hora: quem criou quer configurá-lo.
        store.active_profile = profile.id.clone();
        store.profiles.push(profile);
        Ok(())
    })
}

/// Copia um perfil inteiro, com todos os chats e a aparência de cada um. É o
/// caminho curto para "quero o mesmo layout, só trocando os canais".
#[tauri::command]
pub async fn duplicate_profile(app: AppHandle, id: String) -> Result<Store, String> {
    mutate_windows(&app, |store| {
        let source = store
            .profiles
            .iter()
            .find(|p| p.id == id)
            .ok_or("perfil não encontrado")?;
        let copy = Profile {
            id: settings::new_id(settings::ID_PROFILE),
            name: format!("{} (cópia)", source.name),
            // Id em branco: o `sanitize()` gera um novo para cada chat, senão a
            // cópia disputaria o label de janela com o original.
            chats: source
                .chats
                .iter()
                .map(|c| Chat {
                    id: String::new(),
                    ..c.clone()
                })
                .collect(),
        };
        store.active_profile = copy.id.clone();
        store.profiles.push(copy);
        Ok(())
    })
}

#[tauri::command]
pub fn rename_profile(app: AppHandle, id: String, name: String) -> Result<Store, String> {
    let updated = mutate(&app, |store| {
        let profile = store.profile_mut(&id).ok_or("perfil não encontrado")?;
        profile.name = name;
        Ok(())
    })?;
    // Nenhuma janela muda, mas o nome do perfil aparece no menu da bandeja.
    crate::tray::sync_menu(&app);
    Ok(updated)
}

#[tauri::command]
pub async fn remove_profile(app: AppHandle, id: String) -> Result<Store, String> {
    mutate_windows(&app, |store| {
        if store.profiles.len() <= 1 {
            return Err("é preciso ter pelo menos um perfil".into());
        }
        let before = store.profiles.len();
        store.profiles.retain(|p| p.id != id);
        if store.profiles.len() == before {
            return Err("perfil não encontrado".into());
        }
        Ok(())
    })
}

/// Troca o perfil ativo: fecha as janelas do perfil anterior e abre as do novo.
#[tauri::command]
pub async fn switch_profile(app: AppHandle, id: String) -> Result<Store, String> {
    // A posição das janelas atuais é lida antes da troca. Sem isso, arrastar um
    // overlay e trocar de perfil sem travar a posição perderia o arrasto.
    overlay::capture_geometry(&app);
    mutate_windows(&app, |store| {
        if !store.profiles.iter().any(|p| p.id == id) {
            return Err("perfil não encontrado".into());
        }
        store.active_profile = id;
        Ok(())
    })
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
    overlay::set_all_visible(&app, visible)
}

#[tauri::command]
pub async fn open_config(app: AppHandle) -> Result<(), String> {
    overlay::open_config(&app)
}

/// Caminho do arquivo de configuração, mostrado na UI para o usuário poder
/// auditar/editar/apagar o que o app guarda.
#[tauri::command]
pub fn settings_file_path(app: AppHandle) -> Result<String, String> {
    crate::settings::settings_path(&app).map(|p| p.display().to_string())
}
