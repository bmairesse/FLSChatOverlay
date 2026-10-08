#!/usr/bin/env bash
#
# Gera os prints de divulgação a partir do código real do app.
#
# Não é build nem teste: nada aqui entra no .exe. As páginas deste diretório
# carregam o overlay.css, o overlay.js, o badges.js e o config.js de verdade,
# com o backend Tauri e o tmi.js trocados por dublês, e injetam mensagens de
# exemplo. O que sai é exatamente o que o app desenha — o que muda é só de onde
# vêm as mensagens e a configuração.
#
# Requer: Microsoft Edge (ou Chrome) e Python com Pillow (só para recortar os
# PNGs transparentes). Rode de dentro deste diretório:
#
#   bash gerar-prints.sh
#
set -u

cd "$(dirname "$0")"
HERE="$(pwd -W 2>/dev/null || pwd)"
OUT="$(cd .. && (pwd -W 2>/dev/null || pwd))"

BROWSER=""
for candidate in \
  "/c/Program Files (x86)/Microsoft/Edge/Application/msedge.exe" \
  "/c/Program Files/Microsoft/Edge/Application/msedge.exe" \
  "/c/Program Files/Google/Chrome/Application/chrome.exe" \
  "/c/Program Files (x86)/Google/Chrome/Application/chrome.exe"; do
  [ -x "$candidate" ] && BROWSER="$candidate" && break
done
if [ -z "$BROWSER" ]; then
  echo "Nenhum Edge ou Chrome encontrado." >&2
  exit 1
fi

# Perfil descartável: o navegador do usuário não é tocado.
PROFILE="$(mktemp -d)"
trap 'rm -rf "$PROFILE"' EXIT

shot() { # nome largura altura escala url [flags extras]
  local name="$1" w="$2" h="$3" s="$4" url="$5"; shift 5
  "$BROWSER" --headless=new --disable-gpu --hide-scrollbars --allow-file-access-from-files \
    --user-data-dir="$PROFILE" --window-size="$w,$h" --force-device-scale-factor="$s" \
    --virtual-time-budget=6000 --screenshot="$OUT/$name.png" "$@" "$url" >/dev/null 2>&1
  echo "  $name.png"
}

# Overlay sozinho, em PNG transparente. A janela do headless tem largura mínima
# (~490px), e pedir menos que isso faz o texto quebrar numa largura e ser
# recortado em outra: por isso a página é grande e o recorte vem depois.
solo() { # nome largura altura payload
  local name="$1" w="$2" h="$3" payload="$4"
  "$BROWSER" --headless=new --disable-gpu --hide-scrollbars --allow-file-access-from-files \
    --user-data-dir="$PROFILE" --window-size=1200,1400 --force-device-scale-factor=2 \
    --default-background-color=00000000 --virtual-time-budget=6000 \
    --screenshot="$OUT/$name.png" "file:///$HERE/solo.html?w=$w&h=$h&c=$payload" >/dev/null 2>&1
  python - "$OUT/$name.png" "$w" "$h" <<'PY'
import sys
from PIL import Image
path, w, h = sys.argv[1], int(sys.argv[2]) * 2, int(sys.argv[3]) * 2
Image.open(path).crop((0, 0, w, h)).save(path)
PY
  echo "  $name.png"
}

cena() { shot "$2" 1920 1080 1 "file:///$HERE/scene.html?p=$1"; }

echo "Cenas (1920x1080):"
cena padrao padrao
cena fonte_grande fonte-grande
cena tres_chats tres-chats
cena recolhido recolhido
cena comparativo_recolhimento recolhido-comparativo
cena modo_mover modo-mover
cena modo_mover_aviso modo-mover-aviso
cena cores cores-personalizadas
cena sem_assinatura sem-assinatura

echo "Capa (2560x1440):"
shot capa 1280 720 2 "file:///$HERE/scene.html?p=hero"

echo "Overlay em PNG transparente (2x):"
solo overlay-transparente 380 600 \
  "%7B%22chat%22%3A%7B%22width%22%3A380%2C%22height%22%3A600%7D%7D"
solo overlay-transparente-fonte-grande 470 580 \
  "%7B%22chat%22%3A%7B%22width%22%3A470%2C%22height%22%3A580%2C%22font_size%22%3A22%2C%22opacity%22%3A1%7D%7D"
solo overlay-transparente-modo-mover 480 600 \
  "%7B%22chat%22%3A%7B%22width%22%3A480%2C%22height%22%3A600%2C%22name%22%3A%22Minha%20live%22%7D%2C%22move%22%3Atrue%7D"
solo overlay-transparente-recolhido 380 600 \
  "%7B%22chat%22%3A%7B%22width%22%3A380%2C%22height%22%3A600%2C%22fade_secs%22%3A20%2C%22fade_percent%22%3A25%7D%2C%22faded%22%3Atrue%7D"

echo "Janela de configuracoes (2x):"
# A altura do painel inteiro sai do próprio DOM: chumbar o número aqui deixaria
# sobra preta embaixo assim que qualquer texto da configuração mudasse.
ALTURA=$("$BROWSER" --headless=new --disable-gpu --hide-scrollbars --allow-file-access-from-files \
  --user-data-dir="$PROFILE" --window-size=520,400 --virtual-time-budget=5000 \
  --dump-dom "file:///$HERE/config-shot.html" 2>/dev/null |
  grep -o 'data-height="[0-9]*"' | head -1 | grep -o '[0-9]*')
ALTURA=${ALTURA:-2600}
shot configuracoes 640 800 2 "file:///$HERE/window.html?h=700"
shot configuracoes-completa 600 $((ALTURA + 80)) 2 "file:///$HERE/window.html?h=$ALTURA"

echo
echo "Prints em: $OUT"
