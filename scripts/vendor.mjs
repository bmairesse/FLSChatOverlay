/* Gera o bundle de browser do tmi.js em `src/vendor/`.
 *
 * O frontend não usa bundler — é HTML/CSS/JS puro servido pelo Tauri. O tmi.js,
 * porém, é publicado no npm só em CommonJS para Node: o pacote traz `index.js` e
 * `lib/`, e nenhuma build de browser (a pasta `dist/` que a documentação dele
 * cita não existe no pacote, e a URL equivalente no unpkg responde 404).
 *
 * Então convertemos o pacote de `node_modules` na hora do dev/build. Carregar de
 * um CDN resolveria também, mas abriria a CSP e tiraria a garantia de build
 * offline — que é o ponto do projeto. */

import { existsSync, mkdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { build } from "esbuild";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const entry = join(root, "node_modules", "tmi.js", "index.js");
const outDir = join(root, "src", "vendor");
const outFile = join(outDir, "tmi.min.js");

if (!existsSync(entry)) {
  console.error(
    "[vendor] tmi.js não encontrado em node_modules.\n" +
      "         Rode `npm install` antes de `npm run dev` ou `npm run build`."
  );
  process.exit(1);
}

mkdirSync(outDir, { recursive: true });

// `platform: browser` faz o esbuild respeitar o campo `browser` do package.json
// do tmi.js, que anula `ws` e `node-fetch`: no navegador a lib usa o WebSocket
// nativo. `globalName: tmi` reproduz a global que overlay.html espera do
// <script>, e que overlay.js consome como `new tmi.Client(...)`.
await build({
  entryPoints: [entry],
  outfile: outFile,
  bundle: true,
  minify: true,
  format: "iife",
  globalName: "tmi",
  platform: "browser",
  target: "es2020",
  legalComments: "inline",
});

console.log(`[vendor] ${entry} -> ${outFile}`);
