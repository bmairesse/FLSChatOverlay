/* Janela de configurações. Escreve no estado do Rust, que persiste em disco e
 * avisa o overlay pelo evento `settings-changed`. */

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const el = {
  channel: document.getElementById("channel"),
  channelApply: document.getElementById("channel-apply"),
  opacity: document.getElementById("opacity"),
  opacityOut: document.getElementById("opacity-out"),
  fontSize: document.getElementById("font-size"),
  fontSizeOut: document.getElementById("font-size-out"),
  textColor: document.getElementById("text-color"),
  textColorReset: document.getElementById("text-color-reset"),
  nameOutline: document.getElementById("name-outline-color"),
  nameOutlineReset: document.getElementById("name-outline-reset"),
  highlightMentions: document.getElementById("highlight-mentions"),
  showHeader: document.getElementById("show-header"),
  headerDialog: document.getElementById("header-dialog"),
  headerDisable: document.getElementById("header-disable"),
  maxMessages: document.getElementById("max-messages"),
  idleSecs: document.getElementById("idle-secs"),
  moveBtn: document.getElementById("move-btn"),
  visibleBtn: document.getElementById("visible-btn"),
  configPath: document.getElementById("config-path"),
  toast: document.getElementById("toast"),
};

// Espelham os defaults do Rust (settings.rs). Só são usados pelos botões
// "Padrão"; a fonte da verdade continua sendo o backend.
const DEFAULT_TEXT_COLOR = "#ffffff";
const DEFAULT_NAME_OUTLINE_COLOR = "#000000";

let toastTimer = null;

function toast(message, isError) {
  el.toast.textContent = message;
  el.toast.classList.toggle("error", Boolean(isError));
  el.toast.classList.add("show");
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => el.toast.classList.remove("show"), 1800);
}

/** Preenche os campos sem disparar os handlers de save. */
function render(settings) {
  // O campo de canal é o único que o usuário digita livremente: sobrescrever
  // enquanto ele digita seria hostil.
  if (document.activeElement !== el.channel) {
    el.channel.value = settings.channel;
  }
  el.opacity.value = settings.opacity;
  el.opacityOut.textContent = `${Math.round(settings.opacity * 100)}%`;
  el.fontSize.value = settings.font_size;
  el.fontSizeOut.textContent = `${settings.font_size}px`;
  // <input type="color"> só aceita #rrggbb; o Rust já entrega nesse formato.
  el.textColor.value = settings.text_color;
  el.nameOutline.value = settings.name_outline_color;
  el.highlightMentions.checked = settings.highlight_channel_mentions;
  el.showHeader.checked = settings.show_header;
  el.maxMessages.value = settings.max_messages;
  el.idleSecs.value = settings.idle_warning_secs;

  // A bandeja também mexe na visibilidade; o evento é a fonte da verdade.
  overlayVisible = settings.overlay_visible;
  renderButtons();
}

async function save(patch, message) {
  try {
    const updated = await invoke("save_settings", { patch });
    render(updated);
    if (message) toast(message);
    return updated;
  } catch (err) {
    toast(String(err), true);
    return null;
  }
}

/* ---------- campos ---------- */

function applyChannel() {
  save({ channel: el.channel.value }, "Canal aplicado");
}

el.channelApply.addEventListener("click", applyChannel);
el.channel.addEventListener("keydown", (event) => {
  if (event.key === "Enter") applyChannel();
});
el.channel.addEventListener("blur", applyChannel);

// Sliders escrevem muito: mostra na hora, grava com folga.
let sliderTimer = null;
function debouncedSave(patch) {
  clearTimeout(sliderTimer);
  sliderTimer = setTimeout(() => save(patch), 180);
}

el.opacity.addEventListener("input", () => {
  const value = Number(el.opacity.value);
  el.opacityOut.textContent = `${Math.round(value * 100)}%`;
  debouncedSave({ opacity: value });
});

el.fontSize.addEventListener("input", () => {
  const value = Number(el.fontSize.value);
  el.fontSizeOut.textContent = `${value}px`;
  debouncedSave({ font_size: value });
});

// Arrastar no seletor de cor dispara `input` continuamente, igual aos sliders.
el.textColor.addEventListener("input", () => {
  debouncedSave({ text_color: el.textColor.value });
});

el.nameOutline.addEventListener("input", () => {
  debouncedSave({ name_outline_color: el.nameOutline.value });
});

el.textColorReset.addEventListener("click", () => {
  save({ text_color: DEFAULT_TEXT_COLOR }, "Cor do texto restaurada");
});

el.nameOutlineReset.addEventListener("click", () => {
  save({ name_outline_color: DEFAULT_NAME_OUTLINE_COLOR }, "Contorno restaurado");
});

el.highlightMentions.addEventListener("change", () => {
  save({ highlight_channel_mentions: el.highlightMentions.checked }, "Salvo");
});

/* Desligar a assinatura passa por um recado do autor; ligar de volta, não —
 * quem está religando já ouviu o argumento.
 *
 * A caixa volta a marcada antes de o diálogo abrir: enquanto ele está na tela
 * nada foi salvo, e o que aparece precisa dizer isso. De quebra, fechar no Esc
 * ou no backdrop cai no mesmo lugar que "Deixar ligada", sem handler nenhum
 * para desfazer. */
el.showHeader.addEventListener("change", () => {
  if (el.showHeader.checked) {
    save({ show_header: true }, "Assinatura ligada");
    return;
  }
  el.showHeader.checked = true;
  el.headerDialog.showModal();
});

/* A decisão sai do clique no botão, não do evento `close` do <dialog>.
 *
 * O caminho óbvio seria ler o `returnValue` no `close`, mas ele é frágil por
 * dois motivos: o valor sobrevive ao fechamento e o Esc não o limpa, então a
 * resposta de uma vez decidiria a próxima; e o `close` chega depois, quando o
 * diálogo já saiu — em teste ele deixou de disparar. O clique é o que a pessoa
 * de fato fez. Fechar o diálogo continua sendo trabalho do `method="dialog"`
 * do form, sem JS. */
el.headerDisable.addEventListener("click", () => {
  el.showHeader.checked = false;
  save({ show_header: false }, "Assinatura desligada");
});

el.maxMessages.addEventListener("change", () => {
  save({ max_messages: Number(el.maxMessages.value) }, "Salvo");
});

el.idleSecs.addEventListener("change", () => {
  save({ idle_warning_secs: Number(el.idleSecs.value) }, "Salvo");
});

/* ---------- ações ---------- */

let moveMode = false;
let overlayVisible = true;

function renderButtons() {
  el.moveBtn.textContent = moveMode ? "Travar posição" : "Ativar mover";
  el.visibleBtn.textContent = overlayVisible ? "Ocultar overlay" : "Mostrar overlay";
}

el.moveBtn.addEventListener("click", async () => {
  try {
    await invoke("set_move_mode", { active: !moveMode });
  } catch (err) {
    toast(String(err), true);
  }
});

el.visibleBtn.addEventListener("click", async () => {
  try {
    // O backend responde com `settings-changed`, que atualiza o botão.
    await invoke("set_overlay_visible", { visible: !overlayVisible });
  } catch (err) {
    toast(String(err), true);
  }
});

listen("move-mode", (event) => {
  moveMode = event.payload.active;
  renderButtons();
});

listen("settings-changed", (event) => render(event.payload));

/* ---------- legenda dos selos ---------- */

/* Montada a partir das mesmas definições que o overlay usa para desenhar
   (badges.js). Se um selo mudar de cor ou de símbolo, a legenda acompanha
   sozinha — não existe segunda cópia para esquecer de atualizar. */
function renderBadgeLegend() {
  const list = document.getElementById("badge-legend");
  const items = document.createDocumentFragment();

  for (const name of BADGE_ORDER) {
    const def = BADGES[name];
    const item = document.createElement("li");
    item.appendChild(badgeElement(def));

    const label = document.createElement("span");
    label.textContent = def.label;
    item.appendChild(label);

    items.appendChild(item);
  }

  list.appendChild(items);
}

/* ---------- boot ---------- */

// Fora do bloco assíncrono: a legenda não depende do backend, então uma falha
// ao carregar as configurações não deve levá-la junto.
renderBadgeLegend();

(async () => {
  try {
    render(await invoke("get_settings"));
    moveMode = await invoke("is_move_mode");
    overlayVisible = await invoke("is_overlay_visible");
    renderButtons();
    el.configPath.textContent = await invoke("settings_file_path");
    if (!el.channel.value) el.channel.focus();
  } catch (err) {
    toast(String(err), true);
  }
})();
