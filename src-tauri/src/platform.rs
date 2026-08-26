//! Camada nativa por plataforma.
//!
//! O `set_ignore_cursor_events` do Tauri resolve o click-through na maioria dos
//! casos, mas no Windows ele nem sempre é suficiente sozinho (dependendo do
//! momento em que o webview é criado a flag se perde). Aqui reforçamos as flags
//! `WS_EX_LAYERED | WS_EX_TRANSPARENT` direto na window, e garantimos o
//! topmost real, que é o que faz o overlay aparecer sobre jogos em fullscreen
//! borderless.

use tauri::WebviewWindow;

#[cfg(windows)]
mod imp {
    use tauri::WebviewWindow;
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, HWND_TOPMOST,
        SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, WS_EX_LAYERED, WS_EX_NOACTIVATE,
        WS_EX_TRANSPARENT,
    };

    fn hwnd_of(window: &WebviewWindow) -> Result<HWND, String> {
        // `.0` é um ponteiro cru; passamos por `isize` para não acoplar este
        // crate à versão exata do crate `windows` que o Tauri usa por dentro.
        let raw = window
            .hwnd()
            .map_err(|e| format!("hwnd indisponível: {e}"))?
            .0 as isize;
        Ok(raw as HWND)
    }

    /// Liga/desliga `WS_EX_TRANSPARENT` (cliques atravessam) mantendo
    /// `WS_EX_LAYERED`, que é pré-requisito para a transparência funcionar.
    pub fn set_click_through(window: &WebviewWindow, enabled: bool) -> Result<(), String> {
        let hwnd = hwnd_of(window)?;
        unsafe {
            let current = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
            let next = if enabled {
                current | WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE
            } else {
                // Em modo mover a janela precisa receber clique e foco.
                (current | WS_EX_LAYERED) & !WS_EX_TRANSPARENT & !WS_EX_NOACTIVATE
            };
            if next != current {
                let _ = SetWindowLongPtrW(hwnd, GWL_EXSTYLE, next as isize);
            }
        }
        Ok(())
    }

    /// Reafirma o topmost. Jogos e outros overlays (Discord, driver de vídeo)
    /// disputam essa posição, então chamamos isso periodicamente.
    pub fn raise_to_top(window: &WebviewWindow) -> Result<(), String> {
        let hwnd = hwnd_of(window)?;
        unsafe {
            let _ = SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }
        Ok(())
    }
}

#[cfg(not(windows))]
mod imp {
    use tauri::WebviewWindow;

    pub fn set_click_through(_window: &WebviewWindow, _enabled: bool) -> Result<(), String> {
        Ok(())
    }

    pub fn raise_to_top(window: &WebviewWindow) -> Result<(), String> {
        window
            .set_always_on_top(true)
            .map_err(|e| format!("always_on_top: {e}"))
    }
}

/// Aplica click-through pelo caminho do Tauri e reforça nativamente.
pub fn apply_click_through(window: &WebviewWindow, enabled: bool) -> Result<(), String> {
    window
        .set_ignore_cursor_events(enabled)
        .map_err(|e| format!("set_ignore_cursor_events: {e}"))?;
    imp::set_click_through(window, enabled)
}

pub fn raise_to_top(window: &WebviewWindow) -> Result<(), String> {
    imp::raise_to_top(window)
}
