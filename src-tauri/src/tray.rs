//! Ícone de bandeja e seu menu de contexto.
//!
//! O app vive aqui: fechar a janela de configurações não encerra nada.

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};

use crate::overlay;

const ID_CONFIG: &str = "config";
const ID_MOVE: &str = "move";
const ID_VISIBLE: &str = "visible";
const ID_QUIT: &str = "quit";

/// Guardamos os itens dinâmicos para poder trocar o texto conforme o estado.
pub struct TrayItems {
    pub move_item: MenuItem<Wry>,
    pub visible_item: MenuItem<Wry>,
}

pub fn build(app: &AppHandle) -> Result<(), String> {
    let config_item = MenuItem::with_id(app, ID_CONFIG, "Abrir configurações", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let move_item = MenuItem::with_id(app, ID_MOVE, "Ativar mover", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let visible_item = MenuItem::with_id(app, ID_VISIBLE, "Ocultar overlay", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let sep = PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?;
    let quit_item =
        MenuItem::with_id(app, ID_QUIT, "Sair", true, None::<&str>).map_err(|e| e.to_string())?;

    let menu = Menu::with_items(
        app,
        &[&config_item, &move_item, &visible_item, &sep, &quit_item],
    )
    .map_err(|e| e.to_string())?;

    app.manage(TrayItems {
        move_item,
        visible_item,
    });

    let mut builder = TrayIconBuilder::with_id("main")
        .tooltip("FLS Chat Overlay")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(handle_menu_event)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::DoubleClick { .. } = event {
                let _ = overlay::open_config(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }

    builder.build(app).map_err(|e| e.to_string())?;
    sync_menu(app);
    Ok(())
}

fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    let app = app.clone();
    let result = match event.id().as_ref() {
        ID_CONFIG => overlay::open_config(&app),
        ID_MOVE => overlay::toggle_move_mode(&app),
        ID_VISIBLE => {
            let visible = overlay::overlay_window(&app)
                .and_then(|w| w.is_visible().ok())
                .unwrap_or(false);
            overlay::set_overlay_visible(&app, !visible)
        }
        ID_QUIT => {
            // Trava antes de sair para não deixar a janela sem click-through
            // caso o app seja reaberto com o estado anterior.
            let _ = overlay::set_move_mode(&app, false);
            app.exit(0);
            Ok(())
        }
        _ => Ok(()),
    };
    if let Err(err) = result {
        eprintln!("[tray] {err}");
    }
}

/// Reflete o estado atual nos rótulos do menu.
pub fn sync_menu(app: &AppHandle) {
    let Some(items) = app.try_state::<TrayItems>() else {
        return;
    };

    let move_text = if overlay::is_move_mode(app) {
        "Travar posição"
    } else {
        "Ativar mover"
    };
    let _ = items.move_item.set_text(move_text);

    let visible = overlay::overlay_window(app)
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false);
    let _ = items.visible_item.set_text(if visible {
        "Ocultar overlay"
    } else {
        "Mostrar overlay"
    });
}
