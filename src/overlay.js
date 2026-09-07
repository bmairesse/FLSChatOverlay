/* Overlay: conecta anonimamente ao chat da Twitch e desenha as mensagens.
 *
 * Nenhum login, token ou cookie é usado em lugar nenhum deste arquivo. O tmi.js
 * sem `identity` entra como `justinfan<numero>`, que é o modo de leitura
 * anônima oficial do IRC da Twitch. O único dado que sai daqui é o nome do
 * canal, no comando JOIN.
 */

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const appWindow = window.__TAURI__.window.getCurrentWindow();

const el = {
  body: document.body,
  chat: document.getElementById("chat"),
  status: document.getElementById("status"),
  banner: document.getElementById("banner"),
  bannerText: document.getElementById("banner-text"),
  lockBtn: document.getElementById("lock-btn"),
  dragLayer: document.getElementById("drag-layer"),
};

let settings = null;
let moveMode = false;

/* ------------------------------------------------------------------ */
/* Configurações                                                       */
/* ------------------------------------------------------------------ */

function applySettings(next) {
  const previousChannel = settings ? settings.channel : null;
  settings = next;

  document.documentElement.style.setProperty("--chat-opacity", String(next.opacity));
  document.documentElement.style.setProperty("--font-size", `${next.font_size}px`);
  // Valores já validados como hex pelo sanitize() do Rust.
  document.documentElement.style.setProperty("--text-color", next.text_color);
  document.documentElement.style.setProperty("--name-outline", next.name_outline_color);

  el.body.classList.toggle("highlight-mentions", next.highlight_channel_mentions);
  el.body.classList.toggle("no-header", !next.show_header);

  trimMessages();
  // Fonte, assinatura e teto de mensagens mudam a altura do conteúdo sem mexer
  // na caixa do #chat, então o ResizeObserver não é acionado: quem ancora
  // nestes casos é esta chamada.
  pinToBottom();

  if (next.channel !== previousChannel) {
    connect(next.channel);
  }
}

/* ------------------------------------------------------------------ */
/* Render das mensagens                                                */
/* ------------------------------------------------------------------ */

// Cor estável para quem nunca escolheu uma na Twitch.
const FALLBACK_COLORS = [
  "#FF4A80", "#FF7F50", "#FFB300", "#7FD858", "#2ED9C3",
  "#4FA8FF", "#A970FF", "#FF69B4", "#00D084", "#F2545B",
];

function colorFor(name, tagColor) {
  if (tagColor) return tagColor;
  let hash = 0;
  for (let i = 0; i < name.length; i += 1) {
    hash = (hash * 31 + name.charCodeAt(i)) >>> 0;
  }
  return FALLBACK_COLORS[hash % FALLBACK_COLORS.length];
}

// `BADGES`, `BADGE_ORDER` e `badgeElement` vêm de badges.js, carregado antes
// deste arquivo — os mesmos desenhos alimentam a legenda das configurações.
function appendBadges(line, badges) {
  if (!badges) return;

  for (const name of BADGE_ORDER) {
    if (!Object.prototype.hasOwnProperty.call(badges, name)) continue;
    const def = BADGES[name];
    if (def) line.appendChild(badgeElement(def));
  }
}

/* O chat só faz sentido colado no fim: interessa a mensagem mais nova, e o
 * overlay é click-through — ninguém consegue rolar de volta para procurar o
 * que ficou escondido.
 *
 * `scrollTop` guarda uma distância em pixels, não uma âncora no texto. Quando a
 * janela é redimensionada ou a fonte muda, o conteúdo se refaz e essa mesma
 * distância passa a apontar para outro ponto: as linhas novas ficam abaixo da
 * borda de baixo, cortadas no meio e sem como voltar. Por isso tudo que mexe na
 * geometria termina aqui.
 *
 * Ler `scrollHeight` obriga o layout a se atualizar antes da conta, então
 * chamar logo depois de trocar uma custom property já enxerga a altura nova. */
function pinToBottom() {
  el.chat.scrollTop = el.chat.scrollHeight;
}

// Redimensionar a janela (ou ligar/desligar a assinatura) muda a caixa do
// #chat; estreitá-la ainda reembrulha as linhas, mudando a altura do conteúdo
// junto. O observer roda depois do novo layout e antes da pintura, então a
// âncora acompanha o arraste da alça quadro a quadro, sem piscar.
new ResizeObserver(pinToBottom).observe(el.chat);

function trimMessages() {
  const max = settings ? settings.max_messages : 80;
  while (el.chat.children.length > max) {
    el.chat.removeChild(el.chat.firstChild);
  }
}

/** A mensagem cita o canal? Ex.: `@rubini`, mas não `@rubinizinho`. */
function mentionsChannel(text) {
  const channel = settings && settings.channel;
  if (!channel) return false;

  const needle = `@${channel}`;
  const haystack = text.toLowerCase();

  for (let from = 0; ; from += 1) {
    const at = haystack.indexOf(needle, from);
    if (at === -1) return false;
    // `\b` não serve aqui: para o regex `_` é caractere de palavra, e nome da
    // Twitch pode terminar em `_`. Conferimos o caractere seguinte à mão.
    const after = haystack[at + needle.length];
    if (after === undefined || !/[a-z0-9_]/.test(after)) return true;
    from = at;
  }
}

function addMessage(tags, text, isAction) {
  const name = tags["display-name"] || tags.username || "?";

  const line = document.createElement("div");
  line.className = isAction ? "msg action" : "msg";

  // Mensagem realçada com pontos do canal: a Twitch marca com este msg-id.
  if (tags["msg-id"] === "highlighted-message") {
    line.classList.add("highlighted");
  }

  // A classe é sempre aplicada; quem decide se ela pinta algo é a classe do
  // <body>. Assim o botão da configuração afeta o que já está na tela.
  if (mentionsChannel(text)) {
    line.classList.add("mentions-channel");
  }

  appendBadges(line, tags.badges);

  const user = document.createElement("span");
  user.className = "user";
  user.style.color = colorFor(name.toLowerCase(), tags.color);
  user.textContent = name;
  line.appendChild(user);

  if (!isAction) {
    const sep = document.createElement("span");
    sep.className = "sep";
    sep.textContent = ":";
    line.appendChild(sep);
  }

  const body = document.createElement("span");
  body.className = "text";
  // textContent, não innerHTML: mensagem de chat é entrada de terceiros.
  body.textContent = ` ${text}`;
  line.appendChild(body);

  el.chat.appendChild(line);
  trimMessages();
  pinToBottom();
}

function setStatus(text) {
  el.status.textContent = text || "";
  el.body.classList.toggle("has-status", Boolean(text));
}

/* ------------------------------------------------------------------ */
/* Conexão com o chat                                                  */
/* ------------------------------------------------------------------ */

let client = null;
let generation = 0;

async function connect(channel) {
  generation += 1;
  const myGeneration = generation;

  if (client) {
    const previous = client;
    client = null;
    try {
      await previous.disconnect();
    } catch {
      /* já estava caindo; nada a fazer */
    }
  }

  el.chat.replaceChildren();

  if (!channel) {
    setStatus("Nenhum canal configurado — abra as configurações na bandeja.");
    return;
  }

  setStatus(`Conectando a #${channel}…`);

  // Sem `identity`: o tmi.js entra como justinfan (anônimo, somente leitura).
  const next = new tmi.Client({
    options: { skipUpdatingEmotesets: true },
    connection: { secure: true, reconnect: true },
    channels: [channel],
  });

  next.on("message", (_channel, tags, message, self) => {
    if (self || myGeneration !== generation) return;
    addMessage(tags, message, tags["message-type"] === "action");
  });

  next.on("connected", () => {
    if (myGeneration === generation) setStatus("");
  });

  next.on("disconnected", (reason) => {
    if (myGeneration === generation) {
      setStatus(`Desconectado (${reason || "sem motivo informado"}) — tentando de novo…`);
    }
  });

  client = next;

  try {
    await next.connect();
  } catch (err) {
    if (myGeneration === generation) {
      setStatus(`Falha ao conectar em #${channel}: ${err}`);
    }
  }
}

/* ------------------------------------------------------------------ */
/* Modo mover                                                          */
/* ------------------------------------------------------------------ */

const BANNER_IDLE = "MODO MOVER ATIVO — cliques NÃO passam para o jogo";
const BANNER_WARN = "ATENÇÃO: MODO MOVER AINDA ATIVO — cliques NÃO passam para o jogo";

function setMoveMode(active) {
  moveMode = active;
  el.body.classList.toggle("move-mode", active);
  if (!active) {
    el.body.classList.remove("warn");
    el.bannerText.textContent = BANNER_IDLE;
  }
}

function setWarning(active) {
  el.body.classList.toggle("warn", active);
  el.bannerText.textContent = active ? BANNER_WARN : BANNER_IDLE;
}

// O watchdog vive no Rust; aqui só avisamos que houve movimento. Throttle para
// não virar uma enxurrada de IPC a cada pixel do mouse.
let lastPing = 0;
function pingActivity() {
  if (!moveMode) return;
  const now = Date.now();
  if (now - lastPing < 250) return;
  lastPing = now;
  invoke("notify_overlay_activity").catch(() => {});
}

document.addEventListener("mousemove", pingActivity);
document.addEventListener("wheel", pingActivity, { passive: true });

el.dragLayer.addEventListener("mousedown", (event) => {
  if (event.button !== 0 || !moveMode) return;
  pingActivity();
  appWindow.startDragging().catch(() => {});
});

for (const handle of document.querySelectorAll(".handle")) {
  handle.addEventListener("mousedown", (event) => {
    if (event.button !== 0 || !moveMode) return;
    event.preventDefault();
    event.stopPropagation();
    pingActivity();
    appWindow.startResizeDragging(handle.dataset.dir).catch(() => {});
  });
}

el.lockBtn.addEventListener("click", () => {
  invoke("set_move_mode", { active: false }).catch(() => {});
});

// Esc trava sem precisar acertar o botão. É um keydown na própria webview do
// overlay, que só recebe teclas quando está focada em modo mover — não existe
// hook de teclado nem atalho global em lugar nenhum deste app.
document.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && moveMode) {
    invoke("set_move_mode", { active: false }).catch(() => {});
  }
});

/* ------------------------------------------------------------------ */
/* Boot                                                                */
/* ------------------------------------------------------------------ */

listen("settings-changed", (event) => applySettings(event.payload));
listen("move-mode", (event) => setMoveMode(event.payload.active));
listen("move-idle-warning", (event) => setWarning(event.payload.active));

invoke("get_settings")
  .then(applySettings)
  .catch((err) => setStatus(`Não foi possível ler as configurações: ${err}`));

invoke("is_move_mode")
  .then(setMoveMode)
  .catch(() => {});
