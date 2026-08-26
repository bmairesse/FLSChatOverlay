/* Copia o bundle de browser do tmi.js para `src/vendor/`.
 *
 * O frontend não usa bundler — é HTML/CSS/JS puro servido pelo Tauri. Em vez de
 * carregar a lib de um CDN (o que abriria a CSP e tiraria a garantia de build
 * offline), copiamos o arquivo de `node_modules` na hora do dev/build. */

import { copyFileSync, existsSync, mkdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const outDir = join(root, "src", "vendor");
const outFile = join(outDir, "tmi.min.js");

const candidates = [
  join(root, "node_modules", "tmi.js", "dist", "tmi.min.js"),
  join(root, "node_modules", "tmi.js", "dist", "tmi.js"),
];

const source = candidates.find((path) => existsSync(path));

if (!source) {
  console.error(
    "[vendor] tmi.js não encontrado em node_modules.\n" +
      "         Rode `npm install` antes de `npm run dev` ou `npm run build`."
  );
  process.exit(1);
}

mkdirSync(outDir, { recursive: true });
copyFileSync(source, outFile);
console.log(`[vendor] ${source} -> ${outFile}`);
