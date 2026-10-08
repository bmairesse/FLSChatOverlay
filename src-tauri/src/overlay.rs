//! Criação das janelas e todo o ciclo do "modo mover".
//!
//! Cada chat do perfil ativo tem uma janela própria, de label `overlay-<id>`.
//! O modo mover, por outro lado, é do app inteiro: ele desliga o click-through
//! de todas as janelas de uma vez, porque o risco que ele cria (cliques que não
//! chegam no jogo) não é de uma janela específica — é da tela.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, PhysicalSize, WebviewUrl,
    WebviewWindow, WebviewWindowBuilder,
};

use crate::platform;
use crate::settings::{self, Chat, Store};

pub const CONFIG_LABEL: &str = "config";
/// Prefixo do label das janelas de overlay. O que vem depois é o id do chat.
/// A capability em `capabilities/default.json` usa o glob `overlay-*`.
pub const OVERLAY_PREFIX: &str = "overlay-";

// Eventos emitidos para as webviews.
/// Estado completo (perfis e chats). Vai para a janela de configurações.
pub const EV_STATE: &str = "state-changed";
/// Configuração de um chat só. Vai apenas para a janela daquele chat.
pub const EV_CHAT: &str = "chat-changed";
pub const EV_MOVE_MODE: &str = "move-mode";
pub const EV_IDLE_WARNING: &str = "move-idle-warning";

pub struct AppState {
    pub store: Mutex<Store>,
    /// true = click-through desligado, janelas interativas.
    pub move_mode: AtomicBool,
    /// Último instante em que houve movimento (drag, resize ou mouse sobre uma janela).
    pub last_activity: Mutex<Instant>,
    /// Evita re-emitir o aviso a cada tick do watchdog.
    pub warned: AtomicBool,
}

impl AppState {
    pub fn new(store: Store) -> Self {
        Self {
            store: Mutex::new(store),
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

/// Label de uma janela nova: `overlay-<id do chat>-<geração>`.
///
/// A geração não é enfeite. O `destroy()` de uma janela só agenda a remoção — o
/// label continua no manager do Tauri até o event loop processar o evento
/// `Destroyed`. Se o label fosse só `overlay-<id>`, trocar de perfil e voltar
/// depressa cairia em uma de duas armadilhas: ou o label antigo ainda estaria
/// lá e o chat ficaria sem janela nenhuma, ou a criação falharia por label
/// repetido. Com um label novo a cada criação, esse encontro não existe.
fn next_overlay_label(chat_id: &str) -> String {
    static GENERATION: AtomicU64 = AtomicU64::new(0);
    let generation = GENERATION.fetch_add(1, Ordering::Relaxed);
    format!("{OVERLAY_PREFIX}{chat_id}-{generation}")
}

/// Id do chat a partir do label. `None` para qualquer janela que não seja um
/// overlay — a de configurações, por exemplo.
pub fn chat_id_of(label: &str) -> Option<&str> {
    let rest = label.strip_prefix(OVERLAY_PREFIX)?;
    // O id não tem `-` (garantido pelo `sanitize`), então o último separa a
    // geração e o que vem antes é o id inteiro.
    rest.rsplit_once('-').map(|(id, _generation)| id)
}

pub fn is_overlay_label(label: &str) -> bool {
    label.starts_with(OVERLAY_PREFIX)
}

/// Todas as janelas de overlay abertas, com o id do chat de cada uma.
pub fn chat_windows(app: &AppHandle) -> Vec<(String, WebviewWindow)> {
    app.webview_windows()
        .into_iter()
        .filter_map(|(label, window)| chat_id_of(&label).map(|id| (id.to_string(), window)))
        .collect()
}

pub fn chat_window(app: &AppHandle, chat_id: &str) -> Option<WebviewWindow> {
    chat_windows(app)
        .into_iter()
        .find(|(id, _)| id == chat_id)
        .map(|(_, window)| window)
}

fn create_chat_window(app: &AppHandle, chat: &Chat) -> Result<WebviewWindow, String> {
    let window = WebviewWindowBuilder::new(
        app,
        next_overlay_label(&chat.id),
        WebviewUrl::App("overlay.html".into()),
    )
    .title(format!("Chat Overlay — {}", chat.name))
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .skip_taskbar(true)
    .resizable(true)
    .shadow(false)
    .focused(false)
    .visible(false)
    .inner_size(chat.width as f64, chat.height as f64)
    .build()
    .map_err(|e| format!("criando overlay de {}: {e}", chat.name))?;

    // Posição/tamanho em pixels físicos: é assim que lemos de volta na hora de
    // salvar, então usar a mesma unidade nos dois lados evita deriva em
    // monitores com escala diferente de 100%.
    let _ = window.set_position(PhysicalPosition::new(chat.x, chat.y));
    let _ = window.set_size(PhysicalSize::new(chat.width, chat.height));

    // Uma janela criada com o modo mover já ligado (chat adicionado no meio da
    // arrumação) nasce interativa, como as outras.
    platform::apply_click_through(&window, !is_move_mode(app))?;
    platform::raise_to_top(&window)?;

    if chat.visible {
        let _ = window.show();
    }
    Ok(window)
}

/// Deixa as janelas abertas iguais à lista de chats do perfil ativo: fecha o
/// que sobrou, cria o que falta. É o único caminho para criar/destruir overlay,
/// então trocar de perfil, adicionar e remover chat passam todos por aqui.
pub fn sync_windows(app: &AppHandle) -> Result<(), String> {
    let chats: Vec<Chat> = current_store(app).active_chats().to_vec();

    for (id, window) in chat_windows(app) {
        if !chats.iter().any(|c| c.id == id) {
            // `destroy` e não `close`: `close` só pede o fechamento, e a janela
            // ainda existiria se o mesmo label fosse recriado em seguida.
            let _ = window.destroy();
        }
    }

    for chat in &chats {
        match chat_window(app, &chat.id) {
            Some(window) => {
                let _ = window.set_title(&format!("Chat Overlay — {}", chat.name));
            }
            None => {
                create_chat_window(app, chat)?;
            }
        }
    }

    push_chats(app);
    crate::tray::sync_menu(app);
    Ok(())
}

/// Manda para cada janela de overlay a configuração do chat dela, e o estado
/// completo para a janela de configurações.
pub fn push_chats(app: &AppHandle) {
    let store = current_store(app);
    for (id, window) in chat_windows(app) {
        if let Some(chat) = store.chat(&id) {
            let _ = app.emit_to(window.label(), EV_CHAT, chat.clone());
        }
    }
    let _ = app.emit(EV_STATE, store);
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
        .inner_size(520.0, 700.0)
        .min_inner_size(440.0, 480.0)
        .resizable(true)
        .build()
        .map_err(|e| format!("criando janela de config: {e}"))?;
    let _ = win.set_size(LogicalSize::new(520.0, 700.0));
    let _ = win.set_focus();
    Ok(())
}

/// Alguma janela de overlay está na tela?
pub fn any_visible(app: &AppHandle) -> bool {
    chat_windows(app)
        .iter()
        .any(|(_, w)| w.is_visible().unwrap_or(false))
}

/// Mostra ou esconde a janela de um chat. O estado vai para o disco: é ele que
/// decide o que aparece na próxima abertura do app.
pub fn set_chat_visible(app: &AppHandle, chat_id: &str, visible: bool) -> Result<(), String> {
    let Some(window) = chat_window(app, chat_id) else {
        return Err("esse chat não está aberto".into());
    };

    if visible {
        window.show().map_err(|e| e.to_string())?;
        platform::raise_to_top(&window)?;
    } else {
        // Esconder a última janela durante o modo mover deixaria o usuário sem
        // como travar: não sobraria nada na tela para clicar.
        let last_one = chat_windows(app)
            .iter()
            .filter(|(id, w)| id != chat_id && w.is_visible().unwrap_or(false))
            .count()
            == 0;
        if last_one && is_move_mode(app) {
            set_move_mode(app, false)?;
        }
        window.hide().map_err(|e| e.to_string())?;
    }

    if let Some(chat) = app
        .state::<AppState>()
        .store
        .lock()
        .unwrap()
        .chat_mut(chat_id)
    {
        chat.visible = visible;
    }
    persist(app)?;
    push_chats(app);
    crate::tray::sync_menu(app);
    Ok(())
}

/// Mostra ou esconde todos os overlays do perfil ativo. É o que a bandeja usa.
pub fn set_all_visible(app: &AppHandle, visible: bool) -> Result<(), String> {
    if !visible && is_move_mode(app) {
        set_move_mode(app, false)?;
    }

    for (id, window) in chat_windows(app) {
        if visible {
            let _ = window.show();
            let _ = platform::raise_to_top(&window);
        } else {
            let _ = window.hide();
        }
        if let Some(chat) = app.state::<AppState>().store.lock().unwrap().chat_mut(&id) {
            chat.visible = visible;
        }
    }

    persist(app)?;
    push_chats(app);
    crate::tray::sync_menu(app);
    Ok(())
}

/// Centro do monitor em que a janela está, em pixels físicos.
///
/// Cai no monitor primário quando o Windows não sabe dizer em qual ela está —
/// que é exatamente o caso de uma janela largada fora de qualquer tela, o
/// motivo de este recurso existir.
fn centered_position(window: &WebviewWindow) -> Result<PhysicalPosition<i32>, String> {
    let monitor = match window
        .current_monitor()
        .map_err(|e| format!("monitor atual: {e}"))?
    {
        Some(monitor) => monitor,
        None => window
            .primary_monitor()
            .map_err(|e| format!("monitor primário: {e}"))?
            .ok_or("nenhum monitor disponível")?,
    };

    let origin = monitor.position();
    let area = *monitor.size();
    // `outer_size` e não a largura salva no chat: é a janela inteira que vai
    // para o centro, e é a posição externa que o `set_position` move.
    let size = window
        .outer_size()
        .map_err(|e| format!("tamanho da janela: {e}"))?;

    Ok(PhysicalPosition::new(
        origin.x + (area.width as i32 - size.width as i32) / 2,
        origin.y + (area.height as i32 - size.height as i32) / 2,
    ))
}

/// Põe no centro da tela a janela de um chat (`Some(id)`) ou de todos os chats
/// do perfil ativo (`None`).
///
/// É o resgate de um overlay que não dá para arrastar de volta: quem desliga um
/// monitor, troca a resolução ou muda o arranjo das telas fica com a janela
/// numa coordenada que o mouse não alcança, e o modo mover não ajuda — não tem
/// o que agarrar. Vale para janela escondida também: ela é movida onde está, e
/// continua escondida.
pub fn center_chats(app: &AppHandle, only: Option<&str>) -> Result<(), String> {
    // Antes de mexer em qualquer coisa: um arrasto em andamento nas *outras*
    // janelas seria perdido pelo `persist` do fim.
    capture_geometry(app);

    let windows: Vec<(String, WebviewWindow)> = chat_windows(app)
        .into_iter()
        .filter(|(id, _)| match only {
            Some(wanted) => wanted == id,
            None => true,
        })
        .collect();

    if windows.is_empty() {
        return Err("nenhum chat aberto".into());
    }

    for (id, window) in &windows {
        let position = centered_position(window)?;
        window
            .set_position(position)
            .map_err(|e| format!("centralizando {id}: {e}"))?;

        // A posição nova é gravada a partir do valor calculado, não lida de
        // volta da janela: fora do modo mover nada chamaria `capture_geometry`
        // depois daqui, e o `set_position` pode ainda não ter sido processado.
        if let Some(chat) = app.state::<AppState>().store.lock().unwrap().chat_mut(id) {
            chat.x = position.x;
            chat.y = position.y;
        }
    }

    persist(app)?;
    push_chats(app);
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

/// Liga (`true`) ou trava (`false`) o modo mover, em todas as janelas juntas.
pub fn set_move_mode(app: &AppHandle, active: bool) -> Result<(), String> {
    let windows = chat_windows(app);
    if windows.is_empty() {
        return Err("nenhum chat aberto".into());
    }

    // Não faz sentido mover algo invisível — mas também não faz sentido
    // reaparecer com um chat que o usuário escondeu de propósito. Só quando
    // *nada* está na tela o modo mover traz tudo de volta.
    if active && !any_visible(app) {
        for (id, window) in &windows {
            let _ = window.show();
            if let Some(chat) = app.state::<AppState>().store.lock().unwrap().chat_mut(id) {
                chat.visible = true;
            }
        }
    }

    // click-through é exatamente o inverso do modo mover.
    for (_, window) in &windows {
        if window.is_visible().unwrap_or(false) {
            platform::apply_click_through(window, !active)?;
            platform::raise_to_top(window)?;
        }
    }

    let idle_warning_secs = {
        let state = app.state::<AppState>();
        state.move_mode.store(active, Ordering::SeqCst);
        state.warned.store(false, Ordering::SeqCst);
        *state.last_activity.lock().unwrap() = Instant::now();
        // O binding não é enfeite: como última expressão do bloco, o MutexGuard
        // temporário sobreviveria ao `state` que ele empresta. Nomear o valor
        // solta o guard aqui, antes do fim do bloco.
        let secs = state.store.lock().unwrap().idle_warning_secs;
        secs
    };

    if active {
        if let Some((_, window)) = windows
            .iter()
            .find(|(_, w)| w.is_visible().unwrap_or(false))
        {
            let _ = window.set_focus();
        }
    } else {
        // Ao travar, a posição atual de cada janela vira a posição salva.
        capture_geometry(app);
    }
    persist(app)?;

    push_chats(app);
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

/// Copia posição/tamanho atuais de cada janela aberta para o estado em memória.
pub fn capture_geometry(app: &AppHandle) {
    let windows = chat_windows(app);
    let state = app.state::<AppState>();
    let mut store = state.store.lock().unwrap();
    for (id, window) in windows {
        let position = window.outer_position().ok();
        let size = window.inner_size().ok();
        let Some(chat) = store.chat_mut(&id) else {
            continue;
        };
        if let Some(position) = position {
            chat.x = position.x;
            chat.y = position.y;
        }
        if let Some(size) = size {
            chat.width = size.width;
            chat.height = size.height;
        }
        chat.sanitize();
    }
}

pub fn persist(app: &AppHandle) -> Result<(), String> {
    let snapshot = app.state::<AppState>().store.lock().unwrap().clone();
    settings::save(app, &snapshot)
}

pub fn current_store(app: &AppHandle) -> Store {
    app.state::<AppState>().store.lock().unwrap().clone()
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
                let idle_secs = state.store.lock().unwrap().idle_warning_secs;
                let idle_for = state.last_activity.lock().unwrap().elapsed();
                if idle_for >= Duration::from_secs(idle_secs)
                    && !state.warned.swap(true, Ordering::SeqCst)
                {
                    let _ = app.emit(EV_IDLE_WARNING, IdleWarningPayload { active: true });
                }
            }

            // Aproximadamente a cada 3 segundos.
            if ticks % 8 == 0 {
                let move_mode = state.move_mode.load(Ordering::SeqCst);
                for (_, window) in chat_windows(&app) {
                    if !window.is_visible().unwrap_or(false) {
                        continue;
                    }
                    let _ = platform::raise_to_top(&window);
                    // Reafirma o click-through: no Windows a flag se perde em
                    // algumas transições do WebView2, e o sintoma (overlay
                    // engolindo cliques do jogo) é justamente o que este app
                    // não pode deixar acontecer.
                    if !move_mode {
                        let _ = platform::apply_click_through(&window, true);
                    }
                }
            }
        }
    });
}
