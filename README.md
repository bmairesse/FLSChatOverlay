# FLS Chat Overlay

Overlay flutuante que mostra o chat da Twitch **por cima do jogo**, para quem
joga em um monitor só e não pode desviar o olhar para a janela do navegador.

- Sempre no topo, inclusive sobre jogos em **fullscreen borderless**
- **Click-through por padrão**: os cliques atravessam o overlay e chegam no jogo
- Opacidade, fonte e posição ajustáveis
- Vive na bandeja do sistema
- **Nenhum login na Twitch. Nenhum token. Nenhuma senha.**

---

## "Isso é hack? Vai pegar ban?"

Não, e a resposta é auditável.

**O app só lê o chat, e o chat da Twitch é público.** Ler e escrever são coisas
diferentes na Twitch: você abre a página de qualquer streamer sem ter conta e já
vê o chat rolando. O login só é exigido para **escrever** ou para acessar dados
privados da sua conta.

Este overlay entra como espectador anônimo. O servidor IRC da Twitch aceita um
login fantasma no formato `justinfan<número>` — é o modo anônimo oficial da
própria Twitch para leitura. Não existe usuário nem senha; esse nome é só um
placeholder que o protocolo exige.

Na prática:

1. Você digita o **nome do canal** nas configurações (ex.: `gaules`) — uma vez só
2. O app conecta em `wss://irc-ws.chat.twitch.tv:443` como `justinfan…`
3. Manda `JOIN #gaules`
4. As mensagens começam a chegar

Não é link, não é login, e não se repete: o nome fica salvo e nas próximas vezes
você só abre o app.

**O que o app não faz:** não envia mensagens, não pede OAuth, não lê cookies do
navegador, não toca na sua conta, não injeta nada em processo nenhum de jogo. Ele
é uma janela transparente do Windows como qualquer outra.

**Como conferir:** o código está todo aqui. A conexão está em
[`src/overlay.js`](src/overlay.js); a política de rede (CSP) está travada em
[`src-tauri/tauri.conf.json`](src-tauri/tauri.conf.json) e só permite
`wss://irc-ws.chat.twitch.tv` — o app é incapaz de falar com qualquer outro
servidor. O `.exe` publicado sai do
[workflow público do GitHub Actions](.github/workflows/build.yml), com log aberto,
a partir deste mesmo código.

---

## Compatibilidade com anti-cheat

Anti-cheats como EAC, BattlEye e Vanguard não procuram por overlays — eles
procuram por **interação com o processo do jogo**. Este app não faz nada disso:

| Gatilho de anti-cheat | O app faz? |
| --- | --- |
| Injeção de DLL no processo do jogo | Não |
| `ReadProcessMemory` / `WriteProcessMemory` | Não |
| `OpenProcess` com handle no jogo | Não |
| Hook do renderizador (D3D/Vulkan) | Não |
| Hook de teclado (`SetWindowsHookEx`) | Não |
| Atalho global (`RegisterHotKey`) | Não |
| Driver de kernel | Não |
| Simulação de input (`SendInput`) | Não |

As únicas chamadas nativas do app são `GetWindowLongPtrW`, `SetWindowLongPtrW` e
`SetWindowPos`, todas aplicadas **na própria janela dele**, em
[`platform.rs`](src-tauri/src/platform.rs). Um `grep` por qualquer API que toque
outro processo não retorna nada no repositório inteiro. Para o sistema, isso
aqui é uma janela como a do Bloco de Notas com "sempre no topo" ligado — a mesma
categoria de Discord, Steam, Xbox Game Bar e da lupa do Windows.

Duas decisões de projeto reforçam isso, e são intencionais:

1. **Sem atalhos globais.** `RegisterHotKey` é uma API benigna, mas remover
   qualquer captura de teclado fora da janela do app elimina a discussão. O
   `Esc` que trava o modo mover é um evento de teclado da própria webview, que
   só chega quando o overlay está em foco.
2. **Sem suporte a fullscreen exclusivo.** É tecnicamente possível, e o único
   caminho é hookar o renderizador do jogo — que é exatamente o que anti-cheat
   vigia. A limitação é o preço da segurança, e é um preço barato: basta jogar
   em borderless.

Em ambiente de campeonato com admin (FACEIT, ESEA, ligas), a regra costuma ser
"feche tudo que não está na lista aprovada". Isso é política do torneio, não
detecção — mas vale saber.

---

## Instalação

Três caminhos, do mais simples ao mais paranoico. Todos entregam o mesmo app.

### 1. Microsoft Store

O caminho sem atrito: instala como qualquer outro app, atualiza sozinho e não
mostra aviso nenhum, porque o pacote é assinado pela própria Microsoft. Também
é o único caminho que passa por revisão de terceiro antes de chegar em você.

### 2. Instalador do GitHub Releases

Baixe o `.exe` na aba **Releases**. Ele sai do
[workflow público do CI](.github/workflows/build.yml), com log aberto, e o hash
SHA256 de cada build é impresso nesse log — dá para conferir que o arquivo
baixado é o mesmo que o CI produziu a partir deste código.

O instalador **não** é assinado digitalmente, então o SmartScreen vai avisar
"editor desconhecido" na primeira execução (*Mais informações → Executar assim
mesmo*). Isso vale para todo software independente sem certificado de assinatura
de código e não diz nada sobre o conteúdo do arquivo.

### 3. Compilar você mesmo

Se você não quer confiar em binário nenhum — nem no meu, nem no do CI — o
projeto compila com dois comandos. É o passo a passo logo abaixo.

## Compilar você mesmo

Pré-requisitos:

- [Rust](https://rustup.rs/) (stable, 1.77.2+)
- [Node.js](https://nodejs.org/) 18+
- **Microsoft Visual Studio C++ Build Tools** com o workload *Desktop
  development with C++* (o Tauri usa o linker MSVC)
- **WebView2 Runtime** — já vem no Windows 10/11 atualizado

Depois:

```bash
npm install
npm run dev     # desenvolvimento, com hot reload do frontend
npm run build   # gera o instalador em src-tauri/target/release/bundle/nsis/
```

O `npm install` traz o [tmi.js](https://tmijs.com/), publicado no npm apenas em
CommonJS para Node. O script `npm run vendor` (roda sozinho antes de dev/build)
converte esse pacote em um bundle de browser em `src/vendor/`, usando o esbuild.
Nada é carregado de CDN em tempo de execução.

Para gerar o pacote da Microsoft Store, veja
[`docs/microsoft-store.md`](docs/microsoft-store.md).

---

## Como usar

1. Abra o app. Na primeira execução a janela de configurações abre sozinha.
2. Digite o **nome do canal** e clique em *Aplicar*.
3. Ajuste opacidade e tamanho da fonte.
4. Para reposicionar o overlay: bandeja → **Ativar mover**.
5. Arraste pela janela, redimensione pelas bordas.
6. Bandeja → **Travar posição**, ou o botão *Travar* no banner, ou `Esc` com o
   overlay em foco.

Não existe atalho global: o app não registra nenhuma tecla fora da própria
janela. Isso é [deliberado](#compatibilidade-com-anti-cheat).

### Modo mover, em detalhe

Enquanto o modo mover está ativo, o click-through fica **desligado** — ou seja,
seus cliques param no overlay e não chegam no jogo. Esse é o estado perigoso de
esquecer ligado.

Por isso existe um watchdog: se você ficar **5 segundos sem mexer** no overlay e
ele ainda estiver com click-through desligado, a moldura começa a **pulsar em
vermelho** e o banner muda para `ATENÇÃO: MODO MOVER AINDA ATIVO — cliques NÃO
passam para o jogo`. Qualquer movimento reinicia a contagem e apaga o aviso.

O aviso **nunca** aparece com o click-through ligado — é a conjunção das duas
condições (parado **e** interativo).

O tempo de 5 segundos é configurável.

### Menu da bandeja

| Item | O que faz |
| --- | --- |
| Abrir configurações | Abre a janela de config (duplo-clique no ícone também) |
| Ativar mover / Travar posição | Alterna click-through e o modo de reposicionamento |
| Mostrar / Ocultar overlay | Some com o overlay sem fechar o app |
| Sair | Encerra de verdade |

Fechar a janela de configurações **não** encerra o app — ele continua na bandeja.

---

## Limitações conhecidas

- **Fullscreen exclusivo esconde o overlay.** Use **fullscreen borderless** /
  *windowed fullscreen* no jogo, ou modo janela — que é o que praticamente todo
  mundo que faz stream já usa, porque também evita o piscar do alt-tab. Nesses
  modos o overlay funciona normalmente. No fullscreen exclusivo ele
  simplesmente não aparece: não trava nada, não quebra o jogo. Suportar esse
  modo exigiria hookar o renderizador do jogo, e essa é uma linha que o projeto
  [não vai cruzar](#compatibilidade-com-anti-cheat).
- Outros overlays também querem ser o topmost (Discord, driver de vídeo). O app
  reafirma sua posição a cada ~3 segundos para não ficar por baixo.
- Windows é a plataforma alvo. A camada nativa está isolada em
  [`src-tauri/src/platform.rs`](src-tauri/src/platform.rs) com um fallback para
  os outros sistemas, mas macOS/Linux não são testados nem suportados por ora.

---

## Onde ficam as configurações

Em `%APPDATA%\com.fairylandstudios.chatoverlay\settings.json`. O caminho exato aparece
no rodapé da janela de configurações. É um JSON simples, sem nada sensível
dentro — pode abrir, editar e apagar à vontade.

---

## Mapa do código

```
src/                    frontend, HTML/CSS/JS puro (sem bundler)
  overlay.html/.css/.js overlay transparente + conexão com o chat
  config.html/.css/.js  janela de configurações
src-tauri/src/
  lib.rs                montagem do app, plugins, eventos de janela
  overlay.rs            criação das janelas, modo mover, watchdog de 5s
  platform.rs           flags nativas do Windows (click-through, topmost)
  tray.rs               ícone e menu da bandeja
  settings.rs           persistência em JSON no app config dir
  commands.rs           comandos expostos às webviews
msix/                   empacotamento para a Microsoft Store
  AppxManifest.xml      modelo do manifesto MSIX
  identity.json         identidade do app no Partner Center
  Assets/               logos e tiles da Store
scripts/
  gen_icons.py          gera ícones do app e assets da Store (sem dependências)
  vendor.mjs            gera o bundle de browser do tmi.js em src/vendor
  build-msix.ps1        monta o MSIX a partir do binário compilado
docs/
  microsoft-store.md    passo a passo da submissão à Store
```

### Por que o click-through tem duas camadas

O `set_ignore_cursor_events()` do Tauri resolve o caso comum, mas no Windows a
flag `WS_EX_TRANSPARENT` às vezes se perde em transições do WebView2. Como o
sintoma dessa falha é exatamente o pior cenário deste app — o overlay engolindo
cliques que deveriam ir para o jogo — as flags também são aplicadas direto pela
API do Windows em `platform.rs`, e reafirmadas periodicamente pelo watchdog.

---

## Roadmap

- [ ] Badges e emotes
- [ ] Filtros (ignorar bots, comandos, usuários)
- [ ] Temas / estilo do chat
- [ ] Fade automático de mensagens antigas

## Privacidade

O app não coleta, não armazena e não transmite dado pessoal nenhum. Detalhes em
[PRIVACY.md](PRIVACY.md).

## Licença

[MIT](LICENSE).
