/* Conteúdo de exemplo para os prints. Nomes de espectadores são fictícios. */

const C = {
  purple: "#a970ff", blue: "#4fa8ff", green: "#00d084", red: "#f2545b",
  gold: "#ffb300", pink: "#ff69b4", cyan: "#2ed9c3", orange: "#ff7f50",
};

function m(name, text, opts = {}) {
  return {
    tags: {
      "display-name": name,
      color: opts.color || null,
      badges: opts.badges || null,
      "msg-id": opts.highlighted ? "highlighted-message" : undefined,
    },
    text,
    action: Boolean(opts.action),
  };
}

/* Chat principal: live de jogo, canal do próprio dono do overlay. */
const SCRIPT_MAIN = [
  m("Marina_Q", "boa noite gente 💜", { color: C.pink, badges: { vip: "1", subscriber: "12" } }),
  m("bidu_dev", "esse clutch foi insano kkkkkk", { color: C.blue, badges: { subscriber: "6", premium: "1" } }),
  m("TioSteve", "primeira vez aqui, canal muito bom", { color: C.green }),
  m("nanda_plays", "@ofadinha qual mouse vc usa?", { color: C.gold, badges: { subscriber: "3" } }),
  m("ofadinha", "é um modelo antigo, já respondo no !setup", { color: C.purple, badges: { broadcaster: "1" } }),
  m("k4ssio", "KEKW", { color: C.orange }),
  m("LariMod", "lembrando: sem spoiler no chat", { color: C.red, badges: { moderator: "1", subscriber: "24" } }),
  m("pedro_h", "esse overlay aí é o que?", { color: C.cyan }),
  m("Marina_Q", "@ofadinha o chat na tela ficou top demais", { color: C.pink, badges: { vip: "1", subscriber: "12" } }),
  m("gabs_", "jogando com um monitor só é sofrido kkk", { badges: { premium: "1" } }),
  m("ZeCarlos77", "acabei de assinar!! 💜", { color: C.gold, badges: { founder: "1", subscriber: "1" } }),
  m("ofadinha", "valeu demais, Zé!", { color: C.purple, badges: { broadcaster: "1" } }),
  m("ruivinha_ttv", "destacou minha mensagem com pontos", { color: C.red, badges: { subscriber: "9" }, highlighted: true }),
  m("joao_turbo", "dá pra abrir dois chats ao mesmo tempo?", { color: C.blue, badges: { turbo: "1", subscriber: "2" } }),
  m("bidu_dev", "dá sim, um por canal", { color: C.blue, badges: { subscriber: "6", premium: "1" } }),
  m("mari_cwb", "entra na fila aí", { color: C.cyan }),
  m("TioSteve", "gg", { color: C.green }),
  m("nanda_plays", "vamo pro próximo mapa", { color: C.gold, badges: { subscriber: "3" } }),
  m("k4ssio", "sobe o volume do jogo", { color: C.orange }),
  m("LariMod", "clipou aquele lance?", { color: C.red, badges: { moderator: "1", subscriber: "24" } }),
  m("pedro_h", "esse round foi virado no 1x3", { color: C.cyan }),
  m("gabs_", "PogChamp", { badges: { premium: "1" } }),
  m("Marina_Q", "boa!!", { color: C.pink, badges: { vip: "1", subscriber: "12" } }),
  m("bidu_dev", "quero ver o replay", { color: C.blue, badges: { subscriber: "6", premium: "1" } }),
];

/* Segundo chat: live de outra pessoa, com o @ do usuário destacado. */
const SCRIPT_GUEST = [
  m("Equipe_Twitch", "boa live pra vocês", { color: C.purple, badges: { staff: "1" } }),
  m("vitinho", "cadê o comando do sorteio", { color: C.blue }),
  m("fairyland_live", "sorteio em 10 minutos, fiquem ligados", { color: C.pink, badges: { broadcaster: "1", partner: "1" } }),
  m("carol_rpg", "@ofadinha vem jogar com a gente depois", { color: C.gold, badges: { subscriber: "4" } }),
  m("duda__", "kkkkkkkk", { color: C.cyan }),
  m("Rafa_Mod", "chat, leiam as regras 📌", { color: C.green, badges: { moderator: "1" } }),
  m("nick_alves", "esse boss é impossível", { color: C.orange, badges: { premium: "1" } }),
  m("carol_rpg", "tenta pelo lado esquerdo", { color: C.gold, badges: { subscriber: "4" } }),
  m("vitinho", "GG", { color: C.blue }),
  m("lele_ops", "cheguei agora, perdi muito?", { color: C.red }),
  m("duda__", "só a parte boa kkkk", { color: C.cyan }),
];

/* Terceiro chat: conversa mais calma, fonte maior. */
const SCRIPT_CALM = [
  m("sol_dias", "que trilha é essa?", { color: C.cyan }),
  m("mestre_dos_dados", "rolou 20 natural!", { color: C.gold, badges: { subscriber: "18" } }),
  m("bia_v", "não acredito kkkk", { color: C.pink }),
  m("canal_convidado", "acerto crítico, pode narrar", { color: C.green, badges: { broadcaster: "1" } }),
  m("sol_dias", "épico", { color: C.cyan }),
  m("tom_barros", "entra na história", { color: C.blue, badges: { premium: "1" } }),
];

const SCRIPT_ACTION = [
  m("bidu_dev", "entrou na sala", { color: C.blue, action: true, badges: { subscriber: "6" } }),
];
