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
            commands::get_state,
            commands::get_chat,
            commands::save_chat,
            commands::save_app,
            commands::add_chat,
            commands::remove_chat,
            commands::set_chat_visible,
            commands::add_profile,
            commands::duplicate_profile,
            commands::rename_profile,
            commands::remove_profile,
            commands::switch_profile,
            commands::set_move_mode,
            commands::toggle_move_mode,
            commands::is_move_mode,
            commands::notify_overlay_activity,
            commands::set_overlay_visible,
            commands::open_config,
            commands::settings_file_path,
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            let loaded = settings::load(&handle);
            app.manage(overlay::AppState::new(loaded.clone()));

            overlay::sync_windows(&handle)?;
            tray::build(&handle)?;
            // O menu nasce com o tray; a primeira montagem dentro do
            // `sync_windows` acima ainda não tinha ícone para pendurar.
            tray::sync_menu(&handle);
            overlay::spawn_watchdog(handle.clone());

            // Primeira execução: sem nenhum canal configurado o overlay não tem
            // o que mostrar, então abrimos a configuração direto.
            if loaded.active_chats().iter().all(|c| c.channel.is_empty()) {
                overlay::open_config(&handle)?;
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if !overlay::is_overlay_label(window.label()) {
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
            // bandeja. O mesmo vale para as janelas de overlay fechadas ao
            // trocar de perfil. Só sai de verdade quando `app.exit()` é
            // chamado, que é quando o `code` vem preenchido.
            if let tauri::RunEvent::ExitRequested { code, api, .. } = event {
                if code.is_none() {
                    api.prevent_exit();
                }
            }
        });
}
