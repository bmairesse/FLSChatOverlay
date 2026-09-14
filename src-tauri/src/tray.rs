//! Ícone de bandeja e seu menu de contexto.
//!
//! O app vive aqui: fechar a janela de configurações não encerra nada.
//!
//! O menu é remontado inteiro a cada mudança de estado, em vez de ter os itens
//! guardados e atualizados um a um. A lista de perfis muda de tamanho, e um
//! menu remontado nunca fica fora de sincronia com o que está no disco.

use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Wry};

use crate::overlay;

const TRAY_ID: &str = "main";

const ID_CONFIG: &str = "config";
const ID_MOVE: &str = "move";
const ID_VISIBLE: &str = "visible";
const ID_QUIT: &str = "quit";
/// Prefixo dos itens de perfil. O que vem depois é o id do perfil.
const ID_PROFILE_PREFIX: &str = "profile:";

fn build_menu(app: &AppHandle) -> Result<Menu<Wry>, String> {
    let store = overlay::current_store(app);

    let config_item = MenuItem::with_id(app, ID_CONFIG, "Abrir configurações", true, None::<&str>)
        .map_err(|e| e.to_string())?;

    let mut profile_items: Vec<CheckMenuItem<Wry>> = Vec::new();
    for profile in &store.profiles {
        profile_items.push(
            CheckMenuItem::with_id(
                app,
                format!("{ID_PROFILE_PREFIX}{}", profile.id),
                &profile.name,
                true,
                profile.id == store.active_profile,
                None::<&str>,
            )
            .map_err(|e| e.to_string())?,
        );
    }
    let profile_refs: Vec<&dyn tauri::menu::IsMenuItem<Wry>> = profile_items
        .iter()
        .map(|item| item as &dyn tauri::menu::IsMenuItem<Wry>)
        .collect();
    let profiles_menu =
        Submenu::with_items(app, "Perfil", true, &profile_refs).map_err(|e| e.to_string())?;

    let move_text = if overlay::is_move_mode(app) {
        "Travar posição"
    } else {
        "Ativar mover"
    };
    let move_item = MenuItem::with_id(app, ID_MOVE, move_text, true, None::<&str>)
        .map_err(|e| e.to_string())?;

    let visible_text = if overlay::any_visible(app) {
        "Ocultar overlays"
    } else {
        "Mostrar overlays"
    };
    let visible_item = MenuItem::with_id(app, ID_VISIBLE, visible_text, true, None::<&str>)
        .map_err(|e| e.to_string())?;

    let sep_a = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let sep_b = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let quit_item =
        MenuItem::with_id(app, ID_QUIT, "Sair", true, None::<&str>).map_err(|e| e.to_string())?;

    Menu::with_items(
        app,
        &[
            &config_item,
            &sep_a,
            &profiles_menu,
            &move_item,
            &visible_item,
            &sep_b,
            &quit_item,
        ],
    )
    .map_err(|e| e.to_string())
}

pub fn build(app: &AppHandle) -> Result<(), String> {
    let menu = build_menu(app)?;

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("FLS Chat Overlay")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(handle_menu_event)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::DoubleClick { .. } = event {
                // Fora da thread do event loop, como no `handle_menu_event`.
                let app = tray.app_handle().clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(err) = overlay::open_config(&app) {
                        eprintln!("[tray] {err}");
                    }
                });
            }
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }

    builder.build(app).map_err(|e| e.to_string())?;
    Ok(())
}

/// O clique num item chega na thread do event loop, e nenhuma ação daqui pode
/// rodar nela: abrir a configuração e trocar de perfil criam janelas, e a
/// criação de uma WebView2 espera pela mensagem do Windows que só essa thread
/// despacharia — segurá-la trava o app. É o mesmo motivo que faz os comandos
/// de janela serem `async` em `commands.rs`.
fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    let app = app.clone();
    let id = event.id().as_ref().to_string();

    tauri::async_runtime::spawn(async move {
        if let Some(profile_id) = id.strip_prefix(ID_PROFILE_PREFIX) {
            // O item marcado do perfil ativo também dispara evento quando
            // clicado; trocar para o perfil em uso não faz nada além de
            // remontar o menu, o que devolve a marca para o lugar.
            if let Err(err) =
                crate::commands::switch_profile(app.clone(), profile_id.to_string()).await
            {
                eprintln!("[tray] {err}");
            }
            sync_menu(&app);
            return;
        }

        let result = match id.as_str() {
            ID_CONFIG => overlay::open_config(&app),
            ID_MOVE => overlay::toggle_move_mode(&app),
            ID_VISIBLE => overlay::set_all_visible(&app, !overlay::any_visible(&app)),
            ID_QUIT => {
                // Trava antes de sair para não deixar as janelas sem
                // click-through caso o app seja reaberto com o estado anterior.
                let _ = overlay::set_move_mode(&app, false);
                app.exit(0);
                Ok(())
            }
            _ => Ok(()),
        };
        if let Err(err) = result {
            eprintln!("[tray] {err}");
        }
    });
}

/// Reflete o estado atual no menu, remontando-o.
pub fn sync_menu(app: &AppHandle) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };
    match build_menu(app) {
        Ok(menu) => {
            let _ = tray.set_menu(Some(menu));
        }
        Err(err) => eprintln!("[tray] {err}"),
    }
}
