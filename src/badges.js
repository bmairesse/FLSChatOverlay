/* Selos do chat, desenhados em vetor.
 *
 * Compartilhado entre o overlay, que os exibe ao lado dos nomes, e a janela de
 * configurações, que mostra a legenda. O projeto não usa bundler, então os dois
 * HTML carregam este arquivo antes do seu próprio script e as definições ficam
 * disponíveis como globais.
 *
 * Por que desenhar em vez de baixar: o chat informa *quais* selos cada pessoa
 * tem, mas a arte oficial só sai do CDN da Twitch. Baixá-la abriria uma segunda
 * conexão de rede só por enfeite, e "uma única conexão" é parte do que este app
 * promete. De quebra, vetor não borra: a arte da Twitch tem 18px, feita para
 * página de navegador, e ficaria macia numa fonte de 48px sobre o jogo.
 *
 * A gramática visual é a mesma do site: quadrado arredondado colorido, símbolo
 * branco por cima. São ícones equivalentes, não idênticos — e o selo de
 * assinante personalizado de cada canal não é reproduzido. */

const SVG_NS = "http://www.w3.org/2000/svg";

const STAR =
  "M8 2.4L9.41 6.06L13.33 6.27L10.28 8.74L11.29 12.53L8 10.4" +
  "L4.71 12.53L5.72 8.74L2.67 6.27L6.59 6.06Z";

const BADGES = {
  broadcaster: {
    label: "Dono do canal",
    bg: "#e91916",
    // Ponto cheio: o "no ar" da câmera.
    d: "M8 4.6a3.4 3.4 0 100 6.8 3.4 3.4 0 000-6.8z",
  },
  staff: {
    label: "Equipe da Twitch",
    bg: "#3b3b45",
    d: "M8 2.2l5 1.8v4.2c0 3-2.6 4.8-5 5.6-2.4-.8-5-2.6-5-5.6V4z",
  },
  moderator: {
    label: "Moderador",
    bg: "#00ad03",
    // Espada.
    d: "M7 2.6h2v6.2h2v2H9v2.6H7v-2.6H5v-2h2z",
  },
  vip: {
    label: "VIP",
    bg: "#e005b9",
    d: "M8 2.4l5.2 5.6L8 13.6L2.8 8z",
  },
  partner: {
    label: "Parceiro",
    bg: "#9147ff",
    // Visto.
    d: "M6.7 12.4L2.6 8.3l1.7-1.7 2.4 2.4 4.9-4.9 1.7 1.7z",
  },
  founder: {
    label: "Fundador",
    bg: "#d9a441",
    // Mesma estrela do assinante: o que os separa é a cor.
    d: STAR,
  },
  subscriber: {
    label: "Assinante",
    bg: "#6441a5",
    d: STAR,
  },
  premium: {
    label: "Prime",
    bg: "#00a0d6",
    // Coroa.
    d: "M2.4 11.2L3.4 4.6l3 2.8L8 3.4l1.6 4 3-2.8 1 6.6z",
  },
  turbo: {
    label: "Turbo",
    bg: "#7a3fd6",
    // Raio.
    d: "M9.4 2.2L4 9.2h2.9L6 13.8l5.6-7.2H8.6z",
  },
};

// Papel no canal primeiro, depois status da conta — a mesma ordem da Twitch.
const BADGE_ORDER = [
  "broadcaster",
  "staff",
  "moderator",
  "vip",
  "partner",
  "founder",
  "subscriber",
  "premium",
  "turbo",
];

function badgeElement(def) {
  const svg = document.createElementNS(SVG_NS, "svg");
  svg.setAttribute("class", "badge");
  svg.setAttribute("viewBox", "0 0 16 16");
  svg.setAttribute("aria-hidden", "true");

  const bg = document.createElementNS(SVG_NS, "rect");
  bg.setAttribute("width", "16");
  bg.setAttribute("height", "16");
  bg.setAttribute("rx", "3");
  bg.setAttribute("fill", def.bg);
  svg.appendChild(bg);

  const glyph = document.createElementNS(SVG_NS, "path");
  glyph.setAttribute("d", def.d);
  glyph.setAttribute("fill", "#fff");
  svg.appendChild(glyph);

  return svg;
}
