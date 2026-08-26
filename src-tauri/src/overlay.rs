//! Criação das janelas e todo o ciclo do "modo mover".

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, PhysicalSize, WebviewUrl,
    WebviewWindow, WebviewWindowBuilder,
};

use crate::platform;
use crate::settings::{self, Settings};

pub const OVERLAY_LABEL: &str = "overlay";
pub const CONFIG_LABEL: &str = "config";

// Eventos emitidos para as webviews.
pub const EV_SETTINGS: &str = "settings-changed";
pub const EV_MOVE_MODE: &str = "move-mode";
pub const EV_IDLE_WARNING: &str = "move-idle-warning";

pub struct AppState {
    pub settings: Mutex<Settings>,
    /// true = click-through desligado, janela interativa.
    pub move_mode: AtomicBool,
    /// Último instante em que houve movimento (drag, resize ou mouse sobre a janela).
    pub last_activity: Mutex<Instant>,
    /// Evita re-emitir o aviso a cada tick do watchdog.
    pub warned: AtomicBool,
}

impl AppState {
    pub fn new(settings: Settings) -> Self {
        Self {
            settings: Mutex::new(settings),
            move_mode: AtomicBool::new(false),
            last_activity: Mutex::new(Instant::now()),
            warned: AtomicBool::new(false),
        }
    }
}

#[derive(Clone, Serialize)]
pub struct MoveModePayload {
    pub active: bool,
    pub idle_warning_secs: u64,
}

#[derive(Clone, Serialize)]
pub struct IdleWarningPayload {
    pub active: bool,
}

pub fn overlay_window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(OVERLAY_LABEL)
}

pub fn create_overlay(app: &AppHandle, s: &Settings) -> Result<WebviewWindow, String> {
    let window =
        WebviewWindowBuilder::new(app, OVERLAY_LABEL, WebviewUrl::App("overlay.html".into()))
            .title("Chat Overlay")
            .decorations(false)
            .transparent(true)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(true)
            .shadow(false)
            .focused(false)
            .visible(false)
            .inner_size(s.width as f64, s.height as f64)
            .build()
            .map_err(|e| format!("criando overlay: {e}"))?;

    // Posição/tamanho em pixels físicos: é assim que lemos de volta na hora de
    // salvar, então usar a mesma unidade nos dois lados evita deriva em
    // monitores com escala diferente de 100%.
    let _ = window.set_position(PhysicalPosition::new(s.x, s.y));
    let _ = window.set_size(PhysicalSize::new(s.width, s.height));

    platform::apply_click_through(&window, true)?;
    platform::raise_to_top(&window)?;

    if s.overlay_visible {
        let _ = window.show();
    }
    Ok(window)
}

pub fn open_config(app: &AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window(CONFIG_LABEL) {
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
        return Ok(());
    }
    let win = WebviewWindowBuilder::new(app, CONFIG_LABEL, WebviewUrl::App("config.html".into()))
        .title("FLS Chat Overlay — Configurações")
        .inner_size(480.0, 660.0)
        .min_inner_size(420.0, 480.0)
        .resizable(true)
        .build()
        .map_err(|e| format!("criando janela de config: {e}"))?;
    let _ = win.set_size(LogicalSize::new(480.0, 660.0));
    let _ = win.set_focus();
    Ok(())
}

pub fn set_overlay_visible(app: &AppHandle, visible: bool) -> Result<(), String> {
    let Some(win) = overlay_window(app) else {
        return Ok(());
    };
    if visible {
        win.show().map_err(|e| e.to_string())?;
        platform::raise_to_top(&win)?;
    } else {
        // Ocultar durante o modo mover deixaria o usuário sem como travar.
        if is_move_mode(app) {
            set_move_mode(app, false)?;
        }
        win.hide().map_err(|e| e.to_string())?;
    }

    {
        let state = app.state::<AppState>();
        let mut s = state.settings.lock().unwrap();
        s.overlay_visible = visible;
    }
    persist(app)?;
    let _ = app.emit(EV_SETTINGS, current_settings(app));
    crate::tray::sync_menu(app);
    Ok(())
}

pub fn is_move_mode(app: &AppHandle) -> bool {
    app.state::<AppState>().move_mode.load(Ordering::SeqCst)
}

/// Registra que houve movimento agora: reinicia o watchdog e apaga o aviso.
pub fn note_activity(app: &AppHandle) {
    let state = app.state::<AppState>();
    *state.last_activity.lock().unwrap() = Instant::now();
    if state.warned.swap(false, Ordering::SeqCst) {
        let _ = app.emit(EV_IDLE_WARNING, IdleWarningPayload { active: false });
    }
}

/// Liga (`true`) ou trava (`false`) o modo mover.
pub fn set_move_mode(app: &AppHandle, active: bool) -> Result<(), String> {
    let Some(win) = overlay_window(app) else {
        return Err("overlay não existe".into());
    };

    // Não faz sentido mover algo invisível.
    if active && !win.is_visible().unwrap_or(false) {
        win.show().map_err(|e| e.to_string())?;
        let state = app.state::<AppState>();
        let mut s = state.settings.lock().unwrap();
        s.overlay_visible = true;
    }

    // click-through é exatamente o inverso do modo mover.
    platform::apply_click_through(&win, !active)?;
    platform::raise_to_top(&win)?;

    let idle_warning_secs = {
        let state = app.state::<AppState>();
        state.move_mode.store(active, Ordering::SeqCst);
        state.warned.store(false, Ordering::SeqCst);
        *state.last_activity.lock().unwrap() = Instant::now();
        state.settings.lock().unwrap().idle_warning_secs
    };

    if active {
        let _ = win.set_focus();
    } else {
        // Ao travar, a posição atual vira a posição salva.
        capture_geometry(app, &win);
    }
    persist(app)?;

    let _ = app.emit(EV_SETTINGS, current_settings(app));
    let _ = app.emit(EV_IDLE_WARNING, IdleWarningPayload { active: false });
    let _ = app.emit(
        EV_MOVE_MODE,
        MoveModePayload {
            active,
            idle_warning_secs,
        },
    );
    crate::tray::sync_menu(app);
    Ok(())
}

pub fn toggle_move_mode(app: &AppHandle) -> Result<(), String> {
    set_move_mode(app, !is_move_mode(app))
}

/// Copia posição/tamanho atuais da janela para o estado em memória.
pub fn capture_geometry(app: &AppHandle, win: &WebviewWindow) {
    let state = app.state::<AppState>();
    let mut s = state.settings.lock().unwrap();
    if let Ok(pos) = win.outer_position() {
        s.x = pos.x;
        s.y = pos.y;
    }
    if let Ok(size) = win.inner_size() {
        s.width = size.width;
        s.height = size.height;
    }
    s.sanitize();
}

pub fn persist(app: &AppHandle) -> Result<(), String> {
    let snapshot = app.state::<AppState>().settings.lock().unwrap().clone();
    settings::save(app, &snapshot)
}

pub fn current_settings(app: &AppHandle) -> Settings {
    app.state::<AppState>().settings.lock().unwrap().clone()
}

/// Watchdog do modo mover.
///
/// A regra pedida é uma conjunção: só avisa se estiver em modo mover **e** sem
/// movimento pelo tempo configurado. Com click-through ligado nunca avisa.
/// O mesmo laço reafirma o topmost de tempos em tempos, porque outros overlays
/// (Discord, driver de vídeo) roubam essa posição quando entram em cena.
pub fn spawn_watchdog(app: AppHandle) {
    std::thread::spawn(move || {
        let mut ticks: u32 = 0;
        loop {
            std::thread::sleep(Duration::from_millis(400));
            ticks = ticks.wrapping_add(1);

            let Some(state) = app.try_state::<AppState>() else {
                continue;
            };

            if state.move_mode.load(Ordering::SeqCst) {
                let idle_secs = state.settings.lock().unwrap().idle_warning_secs;
                let idle_for = state.last_activity.lock().unwrap().elapsed();
                if idle_for >= Duration::from_secs(idle_secs)
                    && !state.warned.swap(true, Ordering::SeqCst)
                {
                    let _ = app.emit(EV_IDLE_WARNING, IdleWarningPayload { active: true });
                }
            }

            // Aproximadamente a cada 3 segundos.
            if ticks % 8 == 0 {
                if let Some(win) = overlay_window(&app) {
                    if win.is_visible().unwrap_or(false) {
                        let _ = platform::raise_to_top(&win);
                        // Reafirma o click-through: no Windows a flag se perde
                        // em algumas transições do WebView2, e o sintoma
                        // (overlay engolindo cliques do jogo) é justamente o
                        // que este app não pode deixar acontecer.
                        if !state.move_mode.load(Ordering::SeqCst) {
                            let _ = platform::apply_click_through(&win, true);
                        }
                    }
                }
            }
        }
    });
}
