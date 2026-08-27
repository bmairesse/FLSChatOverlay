# Publicar na Microsoft Store

O bundler do Tauri gera `.exe` (NSIS) e `.msi`, mas **não gera MSIX**, que é o
formato exigido pela Store. Este projeto monta o MSIX por conta própria:

```
msix/AppxManifest.xml    modelo do manifesto, com placeholders
msix/identity.json       os valores de identidade do seu app no Partner Center
msix/Assets/             logos e tiles (gerados por scripts/gen_icons.py)
scripts/build-msix.ps1   monta o layout e chama o makeappx.exe
```

O pacote é pequeno: o Tauri embute o frontend dentro do binário e linka o loader
do WebView2 estaticamente, então o payload é o `.exe` mais as imagens.

---

## 1. Pré-requisitos

- Conta de desenvolvedor da Microsoft Store (você já tem)
- **Windows SDK** instalado, com o componente *Windows SDK Signing Tools for
  Desktop Apps* — é ele que traz `makeappx.exe` e `signtool.exe`. Costuma vir
  junto com o Visual Studio Build Tools que o Tauri já exige.

## 2. Reservar o nome no Partner Center

Em **Partner Center → Aplicativos e jogos → Novo produto → App MSIX ou PWA**,
reserve o nome.

Use **FLS Chat Overlay**.

> ⚠️ **Não coloque "Twitch" no nome.** Marca registrada de terceiro no nome do
> produto é motivo comum de reprovação na revisão da Store. O projeto inteiro já
> foi nomeado com isso em mente — "Twitch" aparece só onde descreve o serviço
> que o app lê ("overlay de chat para Twitch", na descrição da listagem), nunca
> como identidade do produto. Mantenha essa separação ao preencher a listagem.

## 3. Copiar os valores de identidade

Ainda no Partner Center, vá em **Gerenciamento de produto → Identidade do
produto**. Você vai encontrar três valores:

| No Partner Center | No `identity.json` |
| --- | --- |
| Nome do pacote | `identityName` |
| Publicador (`CN=...`) | `publisher` |
| Nome de exibição do publicador | `publisherDisplayName` |

Copie **exatamente**, incluindo o prefixo `CN=`. Qualquer divergência faz o
Partner Center recusar o upload.

## 4. Preencher `msix/identity.json`

```json
{
  "identityName": "12345Fulano.FLSChatOverlay",
  "publisher": "CN=ABCD1234-0000-0000-0000-00000000ABCD",
  "publisherDisplayName": "Fulano",
  "displayName": "FLS Chat Overlay"
}
```

Esses valores não são segredo — a identidade do publicador é pública em qualquer
app da Store. Pode commitar.

Enquanto houver `PREENCHER` no arquivo, o CI pula o empacotamento em vez de
falhar, para não quebrar o build de quem clonou o repo.

## 5. Gerar e testar localmente

```powershell
npm run build       # gera o executável
npm run msix:test   # empacota E assina com certificado autoassinado
```

O `-SelfSign` existe só para você conseguir instalar e testar na sua máquina. O
script imprime os dois comandos necessários:

```powershell
# como administrador, confia no certificado de teste
Import-Certificate -FilePath msix\out\selfsign.cer -CertStoreLocation Cert:\LocalMachine\TrustedPeople
Add-AppxPackage msix\out\FLSChatOverlay.msix
```

Vale testar instalado, não só em `npm run dev`: apps empacotados rodam com
redirecionamento de sistema de arquivos, então confirme que as configurações
salvam e recarregam direito.

Para desinstalar: `Get-AppxPackage *FLSChatOverlay* | Remove-AppxPackage`.

## 6. Gerar o pacote de envio

```powershell
npm run build
npm run msix        # sem assinatura
```

**Não assine o pacote que vai para a Store.** A Microsoft assina o pacote com o
certificado dela na publicação — é exatamente por isso que a Store resolve o
problema do "Editor desconhecido" sem você comprar certificado nenhum.

O arquivo sai em `msix/out/FLSChatOverlay.msix`. O CI também gera esse
mesmo artefato a cada build, no job `build`.

## 7. Enviar

Em **Envios → Pacotes**, arraste o `.msix`. Depois preencha:

**Classificação etária.** O app exibe chat ao vivo escrito por outras pessoas —
conteúdo gerado por usuário e não moderado por você. Declare isso no
questionário. Omitir é motivo de reprovação, e a pergunta é explícita.

**Política de privacidade.** Obrigatória para qualquer app com acesso à rede.
Use o [`PRIVACY.md`](../PRIVACY.md) do repositório — aponte para a URL do
GitHub, ou publique via GitHub Pages se preferir um endereço mais apresentável.

**Justificativa do `runFullTrust`.** A Store pede explicação para essa
capacidade restrita. Algo direto resolve:

> Aplicativo desktop Win32 (não UWP). Precisa de confiança total para criar uma
> janela sobreposta transparente com click-through (`WS_EX_LAYERED |
> WS_EX_TRANSPARENT`) e um ícone na área de notificação. Não acessa outros
> processos nem o sistema de arquivos fora do próprio diretório de configuração.

**Descrição.** Deixe claro que a leitura do chat é anônima e que não há login na
Twitch — é diferencial de confiança e evita perguntas do revisor.

## 8. Atualizações

O `build-msix.ps1` lê a versão de `src-tauri/tauri.conf.json` e acrescenta a
revisão zerada (`1.0.0` → `1.0.0.0`). A Store **exige** que o quarto componente
seja `0`; ela reserva esse campo para si.

Cada envio precisa de versão maior que a anterior. Suba o número em
`src-tauri/tauri.conf.json` (e, por consistência, em `package.json` e
`src-tauri/Cargo.toml`) antes de gerar o pacote.

## 9. WebView2

O app depende do **WebView2 Runtime**, que já vem instalado no Windows 11 e no
Windows 10 atualizado. Não há pacote de framework MSIX para ele, então não é
declarado como dependência no manifesto — é o mesmo comportamento do instalador
NSIS. Se algum revisor perguntar, é dependência de sistema, não de pacote.

---

## Problemas comuns

| Sintoma | Causa |
| --- | --- |
| Partner Center recusa o pacote por identidade | `identityName` ou `publisher` diferem do reservado. Confira caractere por caractere. |
| `makeappx` não encontrado | Falta o Windows SDK, ou falta o componente de signing tools. |
| Erro de versão no upload | O quarto componente precisa ser `0`, e a versão precisa ser maior que a do envio anterior. |
| `Add-AppxPackage` recusa o pacote de teste | O certificado autoassinado não foi importado em `TrustedPeople`, ou o `Subject` dele não bate com o `publisher` do manifesto. |
| Reprovação por nome | Marca de terceiro no nome do produto. Veja o aviso no passo 2. |
