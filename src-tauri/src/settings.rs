use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

/// Versão do formato gravado em disco. Serve para o `load()` saber que um
/// arquivo antigo precisa ser convertido antes de ser usado.
pub const STORE_VERSION: u32 = 2;

pub const DEFAULT_TEXT_COLOR: &str = "#ffffff";
pub const DEFAULT_NAME_OUTLINE_COLOR: &str = "#000000";

/// Um chat: um canal e toda a aparência e posição da janela que o mostra.
///
/// Cada chat vira uma janela de overlay própria, com label `overlay-<id>`.
/// Nenhum campo aqui é credencial: a leitura do chat continua anônima.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Chat {
    /// Identidade estável do chat. Também compõe o label da janela, então só
    /// entram caracteres que o Tauri aceita em label (prefixo + hex).
    pub id: String,
    /// Rótulo mostrado na configuração. Não vai para a tela do overlay.
    pub name: String,
    /// Nome do canal da Twitch, sem `#` e sem URL. Ex.: "gaules".
    pub channel: String,
    /// O `@` destacado nas mensagens deste chat. Vazio = usa o próprio canal.
    ///
    /// Existe porque quem abre vários chats ao mesmo tempo raramente quer ver
    /// só as menções ao dono de cada canal: quer ver as menções a si mesmo — e
    /// o nome dele não é o nome do canal quando o overlay está na live de
    /// outra pessoa.
    pub mention: String,
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
    /// Marca as linhas que citam o `@` deste chat, como o chat do site faz.
    /// Mensagens realçadas com pontos do canal são marcadas independentemente.
    pub highlight_channel_mentions: bool,
    /// Quantas mensagens ficam na tela antes das antigas serem descartadas.
    pub max_messages: u32,
    /// Mostra a assinatura fixa no topo do overlay. Ligado por padrão: é a
    /// única forma de quem vê a live descobrir de onde o overlay veio. Quem
    /// preferir a tela limpa pode desligar — a decisão é do usuário.
    pub show_header: bool,
    /// Segundos sem mensagem nova até o overlay recolher para a faixa de baixo.
    /// `0` desliga o recolhimento — é o padrão, e deixa o overlay exatamente
    /// como era antes desta opção existir.
    pub fade_secs: u64,
    /// Quanto da altura da janela continua visível depois do fade, em por
    /// cento. `100` também desliga: não sobra nada para esconder.
    ///
    /// A conta é sobre a janela inteira, não só sobre as mensagens: a
    /// assinatura desce junto e passa a ocupar o topo da faixa que restou, de
    /// forma que o que o usuário vê depois do fade mede exatamente esta
    /// porcentagem da altura que ele configurou.
    pub fade_percent: u32,
    /// Se a janela deste chat começa visível.
    pub visible: bool,
}

impl Default for Chat {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            channel: String::new(),
            mention: String::new(),
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
            show_header: true,
            fade_secs: 0,
            fade_percent: 100,
            visible: true,
        }
    }
}

impl Chat {
    /// Corrige valores fora de faixa vindos de um JSON editado à mão.
    pub fn sanitize(&mut self) {
        self.name = self.name.trim().to_string();
        self.channel = normalize_channel(&self.channel);
        self.mention = normalize_channel(&self.mention);
        self.opacity = self.opacity.clamp(0.05, 1.0);
        self.width = self.width.clamp(160, 4000);
        self.height = self.height.clamp(120, 4000);
        self.font_size = self.font_size.clamp(8, 48);
        self.text_color = normalize_hex_color(&self.text_color, DEFAULT_TEXT_COLOR);
        self.name_outline_color =
            normalize_hex_color(&self.name_outline_color, DEFAULT_NAME_OUTLINE_COLOR);
        self.max_messages = self.max_messages.clamp(10, 500);
        self.fade_secs = self.fade_secs.min(600);
        // O piso não é 0: uma faixa de 1% some junto com as mensagens e o
        // resultado é indistinguível de um overlay quebrado. Quem quer a tela
        // limpa desliga a janela pela bandeja, que é a ferramenta para isso.
        self.fade_percent = self.fade_percent.clamp(5, 100);
    }
}

/// Um conjunto nomeado de chats. Trocar de perfil troca todas as janelas de
/// overlay de uma vez — é o "uma cena por jogo" que o OBS tem.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub chats: Vec<Chat>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            chats: vec![Chat::default()],
        }
    }
}

/// Tudo que o app persiste em disco.
///
/// Gravado como JSON em `%APPDATA%/<identifier>/settings.json` (app config dir).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Store {
    pub version: u32,
    /// Id do perfil atualmente em uso.
    pub active_profile: String,
    pub profiles: Vec<Profile>,
    /// Segundos sem movimento até o aviso visual do modo mover. É do app
    /// inteiro, não de um chat: o modo mover vale para todas as janelas juntas.
    pub idle_warning_secs: u64,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            version: STORE_VERSION,
            active_profile: String::new(),
            profiles: vec![Profile::default()],
            idle_warning_secs: 5,
        }
    }
}

impl Store {
    /// Garante os invariantes de que o resto do app depende: sempre existe um
    /// perfil, sempre existe um chat dentro dele, todo id é único e não vazio,
    /// e `active_profile` aponta para um perfil que existe.
    pub fn sanitize(&mut self) {
        self.version = STORE_VERSION;
        self.idle_warning_secs = self.idle_warning_secs.clamp(1, 120);

        if self.profiles.is_empty() {
            self.profiles.push(Profile::default());
        }

        // Um conjunto só para perfis e chats: id repetido entre os dois nunca
        // acontece na prática, mas um arquivo copiado à mão pode acabar com
        // duas janelas disputando o mesmo label.
        let mut seen: Vec<String> = Vec::new();

        for (profile_index, profile) in self.profiles.iter_mut().enumerate() {
            profile.name = profile.name.trim().to_string();
            if !valid_id(&profile.id) || seen.contains(&profile.id) {
                profile.id = new_id(ID_PROFILE);
            }
            seen.push(profile.id.clone());
            if profile.name.is_empty() {
                profile.name = format!("Perfil {}", profile_index + 1);
            }
            if profile.chats.is_empty() {
                profile.chats.push(Chat::default());
            }
            for (chat_index, chat) in profile.chats.iter_mut().enumerate() {
                if !valid_id(&chat.id) || seen.contains(&chat.id) {
                    chat.id = new_id(ID_CHAT);
                }
                seen.push(chat.id.clone());
                chat.sanitize();
                if chat.name.is_empty() {
                    chat.name = default_chat_name(chat, chat_index);
                }
            }
        }

        if !self.profiles.iter().any(|p| p.id == self.active_profile) {
            self.active_profile = self.profiles[0].id.clone();
        }
    }

    pub fn active_index(&self) -> usize {
        self.profiles
            .iter()
            .position(|p| p.id == self.active_profile)
            .unwrap_or(0)
    }

    pub fn active_chats(&self) -> &[Chat] {
        self.profiles
            .get(self.active_index())
            .map(|p| p.chats.as_slice())
            .unwrap_or(&[])
    }

    pub fn active_chats_mut(&mut self) -> &mut Vec<Chat> {
        let index = self.active_index();
        &mut self.profiles[index].chats
    }

    /// Busca em todos os perfis, não só no ativo: a janela de configuração
    /// pode mexer em um chat logo depois de o perfil ativo mudar por outro
    /// caminho (bandeja), e o pedido em voo não deve virar erro.
    pub fn chat(&self, id: &str) -> Option<&Chat> {
        self.profiles
            .iter()
            .flat_map(|p| p.chats.iter())
            .find(|c| c.id == id)
    }

    pub fn chat_mut(&mut self, id: &str) -> Option<&mut Chat> {
        self.profiles
            .iter_mut()
            .flat_map(|p| p.chats.iter_mut())
            .find(|c| c.id == id)
    }

    pub fn profile_mut(&mut self, id: &str) -> Option<&mut Profile> {
        self.profiles.iter_mut().find(|p| p.id == id)
    }
}

/// Nome automático de um chat: o canal, quando já se sabe qual é.
fn default_chat_name(chat: &Chat, index: usize) -> String {
    if chat.channel.is_empty() {
        format!("Chat {}", index + 1)
    } else {
        chat.channel.clone()
    }
}

pub const ID_PROFILE: char = 'p';
pub const ID_CHAT: char = 'c';

/// Um id só vale se couber num label de janela do Tauri e não atrapalhar o
/// `overlay-<id>-<geração>` que o label usa. Letras, dígitos e `_`, nada mais:
/// um id com espaço ou `/` vindo de um arquivo editado à mão faria a criação da
/// janela falhar, e um com `-` confundiria a leitura do label de volta.
fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Id curto e único dentro da execução. Não precisa ser imprevisível — só
/// estável no arquivo e válido como label de janela do Tauri.
pub fn new_id(kind: char) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    format!("{kind}{millis:x}{n:x}")
}

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

/// Aceita o que o usuário costuma colar (URL, `#canal`, `@nome`, maiúsculas) e
/// devolve só o nome em minúsculas, que é o que o IRC espera.
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

/* ------------------------------------------------------------------ */
/* Formato antigo (1.x): um chat só, com os campos na raiz do arquivo  */
/* ------------------------------------------------------------------ */

/// Espelha o `Settings` da versão 1.x. Só existe para migrar o arquivo de quem
/// já usava o app antes dos perfis — nada no código novo lê esta struct.
#[derive(Debug, Deserialize)]
#[serde(default)]
struct LegacySettings {
    channel: String,
    opacity: f64,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    font_size: u32,
    text_color: String,
    name_outline_color: String,
    highlight_channel_mentions: bool,
    max_messages: u32,
    show_header: bool,
    overlay_visible: bool,
    idle_warning_secs: u64,
}

impl Default for LegacySettings {
    fn default() -> Self {
        let chat = Chat::default();
        Self {
            channel: chat.channel,
            opacity: chat.opacity,
            x: chat.x,
            y: chat.y,
            width: chat.width,
            height: chat.height,
            font_size: chat.font_size,
            text_color: chat.text_color,
            name_outline_color: chat.name_outline_color,
            highlight_channel_mentions: chat.highlight_channel_mentions,
            max_messages: chat.max_messages,
            show_header: chat.show_header,
            overlay_visible: chat.visible,
            idle_warning_secs: 5,
        }
    }
}

impl LegacySettings {
    fn into_store(self) -> Store {
        let chat = Chat {
            channel: self.channel,
            opacity: self.opacity,
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
            font_size: self.font_size,
            text_color: self.text_color,
            name_outline_color: self.name_outline_color,
            highlight_channel_mentions: self.highlight_channel_mentions,
            max_messages: self.max_messages,
            show_header: self.show_header,
            visible: self.overlay_visible,
            ..Chat::default()
        };
        Store {
            version: STORE_VERSION,
            active_profile: String::new(),
            profiles: vec![Profile {
                id: String::new(),
                name: "Padrão".to_string(),
                chats: vec![chat],
            }],
            idle_warning_secs: self.idle_warning_secs,
        }
    }
}

/// Decide entre o formato novo e o antigo pelo conteúdo, não pela versão: um
/// arquivo da 1.x não tem campo `version` nenhum para consultar.
///
/// O `bool` diz se houve conversão — é o que faz o `load()` guardar uma cópia
/// do arquivo original.
fn parse(raw: &str) -> (Store, bool) {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(raw) else {
        return (Store::default(), false);
    };
    if value.get("profiles").is_some() {
        return (
            serde_json::from_value::<Store>(value).unwrap_or_default(),
            false,
        );
    }
    let store = serde_json::from_value::<LegacySettings>(value)
        .map(LegacySettings::into_store)
        .unwrap_or_default();
    (store, true)
}

pub fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|e| format!("não foi possível resolver o config dir: {e}"))?;
    Ok(dir.join("settings.json"))
}

pub fn load(app: &AppHandle) -> Store {
    let path = settings_path(app).ok();
    let raw = path.as_ref().and_then(|p| fs::read_to_string(p).ok());

    let (mut store, migrated) = match raw {
        Some(raw) => parse(&raw),
        None => (Store::default(), false),
    };

    // A conversão da 1.x é só de ida: a primeira gravação já sai no formato
    // novo, e uma versão antiga do app não saberia ler de volta. A cópia é o
    // caminho de volta de quem precisar desinstalar e voltar para a 1.x.
    if migrated {
        if let Some(path) = &path {
            let backup = path.with_file_name("settings.v1.json");
            if !backup.exists() {
                let _ = fs::copy(path, &backup);
            }
        }
    }

    store.sanitize();
    store
}

pub fn save(app: &AppHandle, store: &Store) -> Result<(), String> {
    let path = settings_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("criando {}: {e}", parent.display()))?;
    }
    let json =
        serde_json::to_string_pretty(store).map_err(|e| format!("serializando settings: {e}"))?;
    fs::write(&path, json).map_err(|e| format!("gravando {}: {e}", path.display()))
}
