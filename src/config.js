/* Janela de configurações. Escreve no estado do Rust, que persiste em disco,
 * mantém as janelas de overlay alinhadas com os chats do perfil ativo e avisa
 * cada overlay pelo evento `chat-changed`.
 *
 * Todo comando de escrita devolve o estado inteiro já sanitizado. Esta janela
 * nunca deduz o resultado de uma ação: ela redesenha a partir do que voltou.
 * É o que mantém a interface igual ao disco mesmo quando a mudança veio de
 * outro lugar — trocar de perfil pela bandeja, por exemplo. */

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const el = {
  profileSelect: document.getElementById("profile-select"),
  profileName: document.getElementById("profile-name"),
  profileAdd: document.getElementById("profile-add"),
  profileDup: document.getElementById("profile-dup"),
  profileDel: document.getElementById("profile-del"),
  chatList: document.getElementById("chat-list"),
  chatAdd: document.getElementById("chat-add"),
  chatDel: document.getElementById("chat-del"),
  chatHeading: document.getElementById("chat-heading"),
  chatName: document.getElementById("chat-name"),
  channel: document.getElementById("channel"),
  channelApply: document.getElementById("channel-apply"),
  mention: document.getElementById("mention"),
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
  confirmDialog: document.getElementById("confirm-dialog"),
  confirmTitle: document.getElementById("confirm-title"),
  confirmText: document.getElementById("confirm-text"),
  confirmOk: document.getElementById("confirm-ok"),
};

// Espelham os defaults do Rust (settings.rs). Só são usados pelos botões
// "Padrão"; a fonte da verdade continua sendo o backend.
const DEFAULT_TEXT_COLOR = "#ffffff";
const DEFAULT_NAME_OUTLINE_COLOR = "#000000";

/** Último estado recebido do backend. */
let store = null;
/** Chat em edição. É escolha desta janela, não vai para o disco. */
let selectedChatId = null;
let moveMode = false;

let toastTimer = null;

function toast(message, isError) {
  el.toast.textContent = message;
  el.toast.classList.toggle("error", Boolean(isError));
  el.toast.classList.add("show");
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => el.toast.classList.remove("show"), 1800);
}

/* Confirmação para o que apaga configuração. O mesmo <dialog> nativo do recado
 * da assinatura, com o texto trocado; a resposta sai do clique no botão, não do
 * `returnValue`, porque ele sobrevive ao fechamento e o Esc não o limpa. */
function askConfirm(title, text, okLabel) {
  return new Promise((resolve) => {
    el.confirmTitle.textContent = title;
    el.confirmText.textContent = text;
    el.confirmOk.textContent = okLabel;

    const onOk = () => {
      cleanup();
      resolve(true);
    };
    const onClose = () => {
      cleanup();
      resolve(false);
    };
    function cleanup() {
      el.confirmOk.removeEventListener("click", onOk);
      el.confirmDialog.removeEventListener("close", onClose);
    }

    el.confirmOk.addEventListener("click", onOk);
    el.confirmDialog.addEventListener("close", onClose);
    el.confirmDialog.showModal();
  });
}

/* ------------------------------------------------------------------ */
/* Leitura do estado                                                   */
/* ------------------------------------------------------------------ */

function activeProfile() {
  if (!store) return null;
  return (
    store.profiles.find((p) => p.id === store.active_profile) || store.profiles[0] || null
  );
}

function selectedChat() {
  const profile = activeProfile();
  if (!profile) return null;
  return profile.chats.find((c) => c.id === selectedChatId) || profile.chats[0] || null;
}

/* ------------------------------------------------------------------ */
/* Render                                                              */
/* ------------------------------------------------------------------ */

/** Sobrescrever um campo de texto enquanto ele está em foco seria hostil. */
function setValue(input, value) {
  if (document.activeElement !== input) input.value = value;
}

function render(next) {
  store = next;
  const profile = activeProfile();
  if (!profile) return;

  // O chat selecionado pode ter sumido (removido, ou o perfil mudou).
  const chat = selectedChat();
  selectedChatId = chat ? chat.id : null;

  renderProfiles(profile);
  renderChatList(profile);
  renderChat(chat);

  el.idleSecs.value = store.idle_warning_secs;
  renderButtons();
}

function renderProfiles(profile) {
  el.profileSelect.replaceChildren();
  for (const item of store.profiles) {
    const option = document.createElement("option");
    option.value = item.id;
    option.textContent = item.name;
    option.selected = item.id === profile.id;
    el.profileSelect.appendChild(option);
  }
  setValue(el.profileName, profile.name);
  el.profileDel.disabled = store.profiles.length <= 1;
}

/* Uma linha por chat, com duas ações separadas: o nome seleciona, o
 * interruptor mostra/esconde a janela daquele chat na hora.
 *
 * Separadas de propósito. Esconder um overlay é o que mais se faz no meio de
 * uma partida, e antes exigia selecionar o chat primeiro — dois passos, e o
 * primeiro deles trocava o que os ajustes lá embaixo mostram sem ninguém ter
 * pedido isso. */
function renderChatList(profile) {
  const rows = document.createDocumentFragment();
  for (const chat of profile.chats) {
    const row = document.createElement("li");
    row.className = "chat-row";
    row.classList.toggle("active", chat.id === selectedChatId);
    // A linha desbotada é o que diz, de uma olhada, quais janelas estão fora
    // da tela — sem precisar abrir um chat por vez para descobrir.
    row.classList.toggle("off", !chat.visible);

    const pick = document.createElement("button");
    pick.type = "button";
    pick.className = "chat-pick";

    const name = document.createElement("span");
    name.className = "chat-row-name";
    name.textContent = chat.name;
    pick.appendChild(name);

    const channel = document.createElement("span");
    channel.className = "chat-row-channel";
    channel.textContent = chat.channel ? `#${chat.channel}` : "sem canal";
    pick.appendChild(channel);

    pick.addEventListener("click", () => {
      selectedChatId = chat.id;
      render(store);
    });
    row.appendChild(pick);

    // `role="switch"` em vez de um checkbox: o estado real mora no Rust, e o
    // botão só mostra o que veio de lá. Um checkbox nativo viraria sozinho no
    // clique e ficaria adiantado em relação ao disco se o comando falhasse.
    const toggle = document.createElement("button");
    toggle.type = "button";
    toggle.className = "chat-toggle";
    toggle.setAttribute("role", "switch");
    toggle.setAttribute("aria-checked", String(chat.visible));
    const action = chat.visible
      ? `Esconder a janela de ${chat.name}`
      : `Mostrar a janela de ${chat.name}`;
    toggle.title = action;
    toggle.setAttribute("aria-label", action);

    const knob = document.createElement("span");
    knob.className = "knob";
    toggle.appendChild(knob);

    toggle.addEventListener("click", () => {
      run("set_chat_visible", { id: chat.id, visible: !chat.visible });
    });
    row.appendChild(toggle);

    rows.appendChild(row);
  }
  el.chatList.replaceChildren(rows);
  el.chatDel.disabled = profile.chats.length <= 1;
}

function renderChat(chat) {
  if (!chat) return;

  el.chatHeading.textContent = chat.channel
    ? `Chat selecionado — #${chat.channel}`
    : "Chat selecionado";

  setValue(el.chatName, chat.name);
  setValue(el.channel, chat.channel);
  setValue(el.mention, chat.mention);
  el.opacity.value = chat.opacity;
  el.opacityOut.textContent = `${Math.round(chat.opacity * 100)}%`;
  el.fontSize.value = chat.font_size;
  el.fontSizeOut.textContent = `${chat.font_size}px`;
  // <input type="color"> só aceita #rrggbb; o Rust já entrega nesse formato.
  el.textColor.value = chat.text_color;
  el.nameOutline.value = chat.name_outline_color;
  el.highlightMentions.checked = chat.highlight_channel_mentions;
  el.showHeader.checked = chat.show_header;
  el.maxMessages.value = chat.max_messages;
}

function renderButtons() {
  el.moveBtn.textContent = moveMode ? "Travar posição" : "Ativar mover";
  const profile = activeProfile();
  const anyVisible = profile ? profile.chats.some((c) => c.visible) : false;
  el.visibleBtn.textContent = anyVisible ? "Ocultar overlays" : "Mostrar overlays";
}

/* ------------------------------------------------------------------ */
/* Escrita                                                             */
/* ------------------------------------------------------------------ */

/** Roda um comando que devolve o estado inteiro e redesenha com a resposta. */
async function run(command, args, message) {
  try {
    const updated = await invoke(command, args);
    if (updated) render(updated);
    if (message) toast(message);
    return updated;
  } catch (err) {
    toast(String(err), true);
    // Uma ação recusada (remover o último chat, por exemplo) deixaria a tela
    // mostrando o que o usuário tentou fazer; redesenhar devolve o que é real.
    if (store) render(store);
    return null;
  }
}

function saveChat(patch, message) {
  if (!selectedChatId) return Promise.resolve(null);
  return run("save_chat", { id: selectedChatId, patch }, message);
}

/* ---------- perfis ---------- */

el.profileSelect.addEventListener("change", () => {
  run("switch_profile", { id: el.profileSelect.value });
});

el.profileAdd.addEventListener("click", async () => {
  const updated = await run("add_profile", { name: "" }, "Perfil criado");
  if (updated) selectFirstChat(updated);
});

el.profileDup.addEventListener("click", async () => {
  const profile = activeProfile();
  if (!profile) return;
  const updated = await run("duplicate_profile", { id: profile.id }, "Perfil duplicado");
  if (updated) selectFirstChat(updated);
});

el.profileDel.addEventListener("click", async () => {
  const profile = activeProfile();
  if (!profile) return;
  const ok = await askConfirm(
    "Excluir o perfil?",
    `"${profile.name}" e os ${profile.chats.length} chat(s) dele serão apagados. Não dá para desfazer.`,
    "Excluir perfil"
  );
  if (!ok) return;
  const updated = await run("remove_profile", { id: profile.id }, "Perfil excluído");
  if (updated) selectFirstChat(updated);
});

function applyProfileName() {
  const profile = activeProfile();
  if (!profile || el.profileName.value === profile.name) return;
  run("rename_profile", { id: profile.id, name: el.profileName.value });
}

el.profileName.addEventListener("blur", applyProfileName);
el.profileName.addEventListener("keydown", (event) => {
  if (event.key === "Enter") applyProfileName();
});

/** Depois de trocar/criar perfil, a seleção antiga não existe mais. */
function selectFirstChat(updated) {
  const profile =
    updated.profiles.find((p) => p.id === updated.active_profile) || updated.profiles[0];
  selectedChatId = profile && profile.chats.length ? profile.chats[0].id : null;
  render(updated);
}

/* ---------- chats ---------- */

el.chatAdd.addEventListener("click", async () => {
  const updated = await run("add_chat", {}, "Chat adicionado");
  if (!updated) return;
  const profile =
    updated.profiles.find((p) => p.id === updated.active_profile) || updated.profiles[0];
  // O chat novo é o último da lista; selecioná-lo é o próximo passo óbvio,
  // porque ele nasce sem canal.
  selectedChatId = profile.chats[profile.chats.length - 1].id;
  render(updated);
  el.channel.focus();
});

el.chatDel.addEventListener("click", async () => {
  const chat = selectedChat();
  if (!chat) return;
  const ok = await askConfirm(
    "Remover o chat?",
    `"${chat.name}" sai deste perfil e a janela dele fecha. Não dá para desfazer.`,
    "Remover chat"
  );
  if (!ok) return;
  selectedChatId = null;
  await run("remove_chat", { id: chat.id }, "Chat removido");
});

/* ---------- campos do chat ---------- */

function applyChatName() {
  saveChat({ name: el.chatName.value });
}

el.chatName.addEventListener("blur", applyChatName);
el.chatName.addEventListener("keydown", (event) => {
  if (event.key === "Enter") applyChatName();
});

function applyChannel() {
  saveChat({ channel: el.channel.value }, "Canal aplicado");
}

el.channelApply.addEventListener("click", applyChannel);
el.channel.addEventListener("keydown", (event) => {
  if (event.key === "Enter") applyChannel();
});
el.channel.addEventListener("blur", applyChannel);

function applyMention() {
  saveChat({ mention: el.mention.value });
}

el.mention.addEventListener("blur", applyMention);
el.mention.addEventListener("keydown", (event) => {
  if (event.key === "Enter") applyMention();
});

// Sliders escrevem muito: mostra na hora, grava com folga.
let sliderTimer = null;
function debouncedSave(patch) {
  clearTimeout(sliderTimer);
  sliderTimer = setTimeout(() => saveChat(patch), 180);
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
  saveChat({ text_color: DEFAULT_TEXT_COLOR }, "Cor do texto restaurada");
});

el.nameOutlineReset.addEventListener("click", () => {
  saveChat({ name_outline_color: DEFAULT_NAME_OUTLINE_COLOR }, "Contorno restaurado");
});

el.highlightMentions.addEventListener("change", () => {
  saveChat({ highlight_channel_mentions: el.highlightMentions.checked }, "Salvo");
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
    saveChat({ show_header: true }, "Assinatura ligada");
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
  saveChat({ show_header: false }, "Assinatura desligada");
});

el.maxMessages.addEventListener("change", () => {
  saveChat({ max_messages: Number(el.maxMessages.value) }, "Salvo");
});

/* ---------- campos do app ---------- */

el.idleSecs.addEventListener("change", () => {
  run("save_app", { patch: { idle_warning_secs: Number(el.idleSecs.value) } }, "Salvo");
});

/* ---------- ações ---------- */

el.moveBtn.addEventListener("click", async () => {
  try {
    await invoke("set_move_mode", { active: !moveMode });
  } catch (err) {
    toast(String(err), true);
  }
});

el.visibleBtn.addEventListener("click", async () => {
  const profile = activeProfile();
  const anyVisible = profile ? profile.chats.some((c) => c.visible) : false;
  try {
    // O backend responde com `state-changed`, que atualiza o botão.
    await invoke("set_overlay_visible", { visible: !anyVisible });
  } catch (err) {
    toast(String(err), true);
  }
});

listen("move-mode", (event) => {
  moveMode = event.payload.active;
  renderButtons();
});

listen("state-changed", (event) => render(event.payload));

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
    render(await invoke("get_state"));
    moveMode = await invoke("is_move_mode");
    renderButtons();
    el.configPath.textContent = await invoke("settings_file_path");
    if (!el.channel.value) el.channel.focus();
  } catch (err) {
    toast(String(err), true);
  }
})();
