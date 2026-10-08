# Prints de divulgação

Imagens prontas para README, post, página da loja ou thread de divulgação.

**Como foram feitas:** as páginas em [`_gerador/`](_gerador) carregam o
`overlay.css`, o `overlay.js`, o `badges.js` e o `config.js` **de verdade**, com
o backend Tauri e o `tmi.js` trocados por dublês, e injetam mensagens de
exemplo. O que aparece nos prints é exatamente o que o app desenha na tela: as
mesmas medidas, o mesmo contorno, os mesmos selos, a mesma quebra de linha. O
que muda é só de onde vêm as mensagens — de um arquivo de exemplo, não da
Twitch.

Duas ressalvas, para ninguém ser pego de surpresa:

- **O cenário de fundo é desenho, não é jogo.** São duas cenas abstratas feitas
  em SVG (uma noturna e uma clara), escolhidas para mostrar que o texto continua
  legível sobre fundo escuro e sobre fundo claro. Nenhuma delas imita a
  interface de jogo nenhum. Se você for usar isso para divulgar, o ideal é
  refazer o print por cima de uma gameplay sua — e os PNGs transparentes abaixo
  existem justamente para isso.
- **O chat é fictício.** Nomes, mensagens e canais são inventados. `ofadinha` e
  `fairyland_live` aparecem como exemplo por serem do próprio projeto;
  `canal_convidado` é um nome de fachada. Nenhuma mensagem real de nenhuma
  pessoa real foi usada.

## Cenas (1920 × 1080)

| Arquivo | O que mostra |
| --- | --- |
| `capa.png` | Imagem de capa, 2560 × 1440. Dois chats, texto grande o bastante para continuar legível em miniatura. |
| `padrao.png` | A configuração de fábrica: 380 × 600, fonte 14, opacidade 85%, assinatura ligada, menções destacadas. |
| `fonte-grande.png` | Fonte 22 e opacidade cheia sobre cenário claro — é o print que responde "dá para ler enquanto jogo?". |
| `tres-chats.png` | Três chats ao mesmo tempo, cada um com o seu canal, o seu tamanho, a sua cor e o seu `@`. |
| `recolhido.png` | O overlay depois do tempo de inatividade: sobra a faixa de baixo e o resto da tela volta para o jogo. |
| `recolhido-comparativo.png` | O antes e o depois do recolhimento lado a lado, com legenda. É a figura para explicar o recurso. |
| `modo-mover.png` | Modo mover: moldura roxa, faixa de aviso com o nome do chat e o botão **Travar**. |
| `modo-mover-aviso.png` | O aviso de inatividade do modo mover — vermelho pulsando, congelado no pico. |
| `cores-personalizadas.png` | Texto em dourado e contorno do nome em roxo, para mostrar que a aparência é ajustável. |
| `sem-assinatura.png` | Assinatura desligada, faixa estreita no canto — o mínimo possível na tela. |

## Overlay em PNG transparente (2×)

Sem fundo nenhum: dá para jogar por cima de uma captura sua de gameplay no
editor de imagem e ter um print legítimo em dois minutos.

| Arquivo | Configuração |
| --- | --- |
| `overlay-transparente.png` | Padrão, 380 × 600. |
| `overlay-transparente-fonte-grande.png` | Fonte 22, opacidade cheia, 470 × 580. |
| `overlay-transparente-modo-mover.png` | Modo mover ligado, 480 × 600. |
| `overlay-transparente-recolhido.png` | Recolhido em 25% da altura. |

## Janela de configurações

| Arquivo | O que mostra |
| --- | --- |
| `configuracoes.png` | A janela no tamanho real (520 × 700), como ela abre pela bandeja. |
| `configuracoes-completa.png` | O painel inteiro numa imagem só, até a legenda dos selos e o aviso de privacidade. |

O estado mostrado ali é de mentira, mas é um estado válido: dois perfis, três
chats no perfil ativo, um deles escondido pelo interruptor. O caminho do
`settings.json` aparece com um usuário fictício.

## Refazer os prints

```sh
cd docs/prints/_gerador
bash gerar-prints.sh
```

Precisa do Microsoft Edge (ou Chrome) e de Python com Pillow — o Pillow só é
usado para recortar os PNGs transparentes. O script abre um navegador **sem
janela**, com perfil descartável: o seu navegador não é aberto nem tocado.

Para mudar o conteúdo do chat, mexa em [`_gerador/data.js`](_gerador/data.js);
para mudar tamanho, posição ou aparência de cada print, em
[`_gerador/scene.html`](_gerador/scene.html), onde ficam os presets. Nada disso
entra no `.exe`: é material de divulgação, não código do app.
