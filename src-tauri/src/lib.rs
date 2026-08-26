mod commands;
mod overlay;
mod platform;
mod settings;
mod tray;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::get_settings,
            commands::save_settings,
            commands::set_move_mode,
            commands::toggle_move_mode,
            commands::is_move_mode,
            commands::notify_overlay_activity,
            commands::set_overlay_visible,
            commands::is_overlay_visible,
            commands::open_config,
            commands::settings_file_path,
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            let loaded = settings::load(&handle);
            app.manage(overlay::AppState::new(loaded.clone()));

            overlay::create_overlay(&handle, &loaded)?;
            tray::build(&handle)?;
            overlay::spawn_watchdog(handle.clone());

            // Primeira execução: sem canal configurado o overlay não tem o que
            // mostrar, então abrimos a configuração direto.
            if loaded.channel.is_empty() {
                overlay::open_config(&handle)?;
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() != overlay::OVERLAY_LABEL {
                return;
            }
            // Arrastar e redimensionar contam como movimento e reiniciam o
            // watchdog de inatividade.
            if matches!(
                event,
                tauri::WindowEvent::Moved(_) | tauri::WindowEvent::Resized(_)
            ) {
                let app = window.app_handle();
                if overlay::is_move_mode(app) {
                    overlay::note_activity(app);
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("erro ao inicializar o app Tauri")
        .run(|_app, event| {
            // Fechar a janela de configurações não encerra o app: ele vive na
            // bandeja. Só sai de verdade quando `app.exit()` é chamado, que é
            // quando o `code` vem preenchido.
            if let tauri::RunEvent::ExitRequested { code, api, .. } = event {
                if code.is_none() {
                    api.prevent_exit();
                }
            }
        });
}
