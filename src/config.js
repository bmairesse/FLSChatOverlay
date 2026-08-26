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
  maxMessages: document.getElementById("max-messages"),
  idleSecs: document.getElementById("idle-secs"),
  moveBtn: document.getElementById("move-btn"),
  visibleBtn: document.getElementById("visible-btn"),
  configPath: document.getElementById("config-path"),
  toast: document.getElementById("toast"),
};

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

/* ---------- boot ---------- */

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
