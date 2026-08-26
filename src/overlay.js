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

  trimMessages();

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

function trimMessages() {
  const max = settings ? settings.max_messages : 80;
  while (el.chat.children.length > max) {
    el.chat.removeChild(el.chat.firstChild);
  }
}

function addMessage(tags, text, isAction) {
  const name = tags["display-name"] || tags.username || "?";

  const line = document.createElement("div");
  line.className = isAction ? "msg action" : "msg";

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
  el.chat.scrollTop = el.chat.scrollHeight;
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
