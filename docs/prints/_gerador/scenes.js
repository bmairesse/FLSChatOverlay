/* Cenários sintéticos de fundo.
 *
 * Não são capturas de jogo nenhum: são desenhos abstratos, feitos só para
 * mostrar como o overlay se comporta sobre cenário escuro e sobre cenário
 * claro. Nada aqui imita a interface de um jogo real. */

function pines(y, scale, fill, opacity, seed) {
  let out = `<g fill="${fill}" opacity="${opacity}">`;
  let x = -40;
  let n = seed;
  while (x < 1980) {
    n = (n * 1103515245 + 12345) % 2147483648;
    const r = n / 2147483648;
    x += 90 + r * 200;
    const h = (110 + ((n >> 7) % 110)) * scale;
    const w = (0.2 + 0.05 * (((n >> 3) % 5))) * h;
    const yy = y + ((n >> 5) % 26) - 13;
    out += `<path d="M${x.toFixed(1)} ${yy} L${(x - w).toFixed(1)} ${yy} L${x.toFixed(1)} ${(yy - h).toFixed(1)} L${(x + w).toFixed(1)} ${yy} Z"/>`;
    out += `<rect x="${(x - 2.5 * scale).toFixed(1)}" y="${yy - 4}" width="${5 * scale}" height="${12 * scale}"/>`;
  }
  return out + "</g>";
}

const SCENES = {
  noite: `
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1920 1080" preserveAspectRatio="xMidYMid slice"
     style="position:absolute;inset:0;width:100%;height:100%">
  <defs>
    <linearGradient id="sky" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#090d20"/><stop offset="0.42" stop-color="#231a44"/>
      <stop offset="0.7" stop-color="#5d3462"/><stop offset="1" stop-color="#a55a52"/>
    </linearGradient>
    <radialGradient id="glow" cx="0.72" cy="0.68" r="0.45">
      <stop offset="0" stop-color="#ffd9a0" stop-opacity="0.8"/>
      <stop offset="1" stop-color="#ffd9a0" stop-opacity="0"/>
    </radialGradient>
    <linearGradient id="far" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#4a3566"/><stop offset="1" stop-color="#31234b"/>
    </linearGradient>
    <linearGradient id="mid" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#231a3c"/><stop offset="1" stop-color="#16102a"/>
    </linearGradient>
    <linearGradient id="fog" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#b79ad0" stop-opacity="0"/>
      <stop offset="0.5" stop-color="#b79ad0" stop-opacity="0.22"/>
      <stop offset="1" stop-color="#b79ad0" stop-opacity="0"/>
    </linearGradient>
    <linearGradient id="ground" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#140e26"/><stop offset="1" stop-color="#05030c"/>
    </linearGradient>
    <radialGradient id="vig" cx="0.5" cy="0.5" r="0.78">
      <stop offset="0.55" stop-color="#000" stop-opacity="0"/>
      <stop offset="1" stop-color="#000" stop-opacity="0.5"/>
    </radialGradient>
  </defs>
  <rect width="1920" height="1080" fill="url(#sky)"/>
  <circle cx="1390" cy="640" r="540" fill="url(#glow)"/>
  <circle cx="1390" cy="640" r="44" fill="#ffe7c2" opacity="0.92"/>
  <g fill="#fff" opacity="0.6">
    <circle cx="180" cy="120" r="1.8"/><circle cx="420" cy="80" r="1.4"/>
    <circle cx="700" cy="180" r="1.6"/><circle cx="960" cy="60" r="1.2"/>
    <circle cx="1240" cy="150" r="1.7"/><circle cx="1600" cy="90" r="1.3"/>
    <circle cx="1820" cy="210" r="1.5"/><circle cx="300" cy="300" r="1.1"/>
    <circle cx="1100" cy="260" r="1.2"/><circle cx="560" cy="230" r="1"/>
    <circle cx="820" cy="330" r="1.3"/><circle cx="1500" cy="300" r="1"/>
  </g>
  <path d="M0 700 L230 520 L400 640 L580 460 L790 670 L920 580 L1080 700 L1300 610 L1520 700 L1740 620 L1920 690 L1920 1080 L0 1080 Z" fill="url(#far)" opacity="0.7"/>
  <rect x="0" y="640" width="1920" height="160" fill="url(#fog)"/>
  <path d="M0 800 L260 670 L470 780 L690 620 L900 790 L1150 690 L1400 800 L1700 720 L1920 810 L1920 1080 L0 1080 Z" fill="url(#mid)"/>
  ${pines(830, 0.75, "#120c22", 0.9, 7)}
  <path d="M0 880 L1920 845 L1920 1080 L0 1080 Z" fill="url(#ground)"/>
  ${pines(985, 1.35, "#07040f", 1, 21)}
  <rect width="1920" height="1080" fill="url(#vig)"/>
</svg>`,

  dia: `
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1920 1080" preserveAspectRatio="xMidYMid slice"
     style="position:absolute;inset:0;width:100%;height:100%">
  <defs>
    <linearGradient id="sky2" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#8ec8ff"/><stop offset="0.55" stop-color="#d9ecff"/>
      <stop offset="1" stop-color="#f8fcff"/>
    </linearGradient>
    <radialGradient id="sun" cx="0.2" cy="0.16" r="0.38">
      <stop offset="0" stop-color="#ffffff" stop-opacity="0.98"/>
      <stop offset="1" stop-color="#ffffff" stop-opacity="0"/>
    </radialGradient>
    <linearGradient id="snowfar" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#c9ddf2"/><stop offset="1" stop-color="#eaf3fc"/>
    </linearGradient>
    <linearGradient id="snownear" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#ffffff"/><stop offset="1" stop-color="#dfecf9"/>
    </linearGradient>
  </defs>
  <rect width="1920" height="1080" fill="url(#sky2)"/>
  <circle cx="390" cy="170" r="440" fill="url(#sun)"/>
  <g fill="#ffffff" opacity="0.9">
    <ellipse cx="1500" cy="220" rx="210" ry="56"/>
    <ellipse cx="1630" cy="188" rx="148" ry="44"/>
    <ellipse cx="1010" cy="140" rx="168" ry="42"/>
    <ellipse cx="620" cy="250" rx="140" ry="38"/>
  </g>
  <path d="M0 620 L300 450 L520 600 L780 410 L1020 610 L1300 460 L1560 610 L1920 500 L1920 1080 L0 1080 Z" fill="url(#snowfar)"/>
  ${pines(665, 0.5, "#a8c0d6", 0.7, 3)}
  <path d="M0 770 L340 690 L640 780 L980 680 L1320 790 L1660 710 L1920 780 L1920 1080 L0 1080 Z" fill="url(#snownear)"/>
  ${pines(905, 1.0, "#8aa7c2", 0.85, 15)}
  <g stroke="#cadcec" stroke-width="5" opacity="0.9" fill="none">
    <path d="M120 1080 C 380 950, 640 990, 920 880"/>
    <path d="M1380 1080 C 1520 970, 1720 950, 1920 910"/>
  </g>
</svg>`,
};
