<#
.SYNOPSIS
  Empacota o executável já compilado do Tauri em um MSIX para a Microsoft Store.

.DESCRIPTION
  O bundler do Tauri não gera MSIX, então montamos o pacote à mão: um layout com
  o .exe, os assets visuais e o AppxManifest.xml, selado pelo makeappx.exe do
  Windows SDK.

  O payload é pequeno de propósito — o Tauri embute o frontend no próprio
  binário e linka o loader do WebView2 estaticamente, então o pacote é
  basicamente um .exe mais imagens.

  Rode `npm run build` antes (ou use -Build para encadear os dois).

.PARAMETER Build
  Roda `npm run build` antes de empacotar.

.PARAMETER SelfSign
  Assina o pacote com um certificado autoassinado, só para instalar e testar
  localmente. Para a Store NÃO use isto: o pacote é enviado sem assinatura e a
  própria Microsoft assina na publicação.

.PARAMETER SkipIfUnconfigured
  Sai com sucesso (em vez de erro) se identity.json ainda tiver placeholders.
  Usado pelo CI, para o build não quebrar em quem clonou o repo.

.EXAMPLE
  npm run build
  pwsh scripts/build-msix.ps1

.EXAMPLE
  pwsh scripts/build-msix.ps1 -Build -SelfSign
#>

[CmdletBinding()]
param(
  [switch]$Build,
  [switch]$SelfSign,
  [switch]$SkipIfUnconfigured
)

$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot
$msixDir = Join-Path $root 'msix'
$outDir = Join-Path $msixDir 'out'
$stageDir = Join-Path $outDir 'stage'

function Fail([string]$message) {
  Write-Error $message
  exit 1
}

# ---------------------------------------------------------------------------
# Identidade
# ---------------------------------------------------------------------------

$identityPath = Join-Path $msixDir 'identity.json'
if (-not (Test-Path $identityPath)) { Fail "não encontrei $identityPath" }

$identity = Get-Content $identityPath -Raw | ConvertFrom-Json

$required = @('identityName', 'publisher', 'publisherDisplayName', 'displayName')
foreach ($field in $required) {
  $value = $identity.$field
  if ([string]::IsNullOrWhiteSpace($value) -or $value -like '*PREENCHER*') {
    $note = "msix/identity.json ainda não foi preenchido (campo '$field'). Veja docs/microsoft-store.md."
    if ($SkipIfUnconfigured) {
      Write-Host "[msix] $note"
      Write-Host '[msix] pulando o empacotamento.'
      exit 0
    }
    Fail $note
  }
}

if ($identity.publisher -notmatch '^CN=') {
  Fail "publisher deve começar com 'CN=' e ser a string exata do Partner Center. Valor atual: $($identity.publisher)"
}

# ---------------------------------------------------------------------------
# Versão — vem do tauri.conf.json, fonte única da verdade
# ---------------------------------------------------------------------------

$confPath = Join-Path $root 'src-tauri/tauri.conf.json'
$conf = Get-Content $confPath -Raw | ConvertFrom-Json
$semver = $conf.version

if ($semver -notmatch '^\d+\.\d+\.\d+$') {
  Fail "version em tauri.conf.json precisa ser X.Y.Z; encontrei '$semver'"
}

# A Store exige quatro componentes com a revisão zerada: ela reserva o último
# campo para as próprias republicações.
$version = "$semver.0"
Write-Host "[msix] versão do pacote: $version"

# ---------------------------------------------------------------------------
# Ferramentas do Windows SDK
# ---------------------------------------------------------------------------

function Find-SdkTool([string]$name) {
  $existing = Get-Command $name -ErrorAction SilentlyContinue
  if ($existing) { return $existing.Source }

  $bases = @(
    "${env:ProgramFiles(x86)}\Windows Kits\10\bin",
    "$env:ProgramFiles\Windows Kits\10\bin"
  ) | Where-Object { $_ -and (Test-Path $_) }

  foreach ($base in $bases) {
    $found = Get-ChildItem -Path $base -Recurse -Filter $name -ErrorAction SilentlyContinue |
      Where-Object { $_.FullName -match '\\(x64|x86)\\' } |
      Sort-Object FullName -Descending |
      Select-Object -First 1
    if ($found) { return $found.FullName }
  }
  return $null
}

$makeappx = Find-SdkTool 'makeappx.exe'
if (-not $makeappx) {
  Fail 'makeappx.exe não encontrado. Instale o Windows SDK (componente "Windows SDK Signing Tools for Desktop Apps").'
}
Write-Host "[msix] makeappx: $makeappx"

# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------

if ($Build) {
  Write-Host '[msix] rodando npm run build...'
  Push-Location $root
  try {
    npm run build
    if ($LASTEXITCODE -ne 0) { Fail 'npm run build falhou' }
  } finally {
    Pop-Location
  }
}

# O nome do binário varia conforme o Tauri/Cargo, então procuramos em vez de
# assumir. Descartamos artefatos intermediários do cargo (deps/, build/).
$releaseDir = Join-Path $root 'src-tauri/target/release'
if (-not (Test-Path $releaseDir)) { Fail "não encontrei $releaseDir. Rode npm run build antes." }

$exe = Get-ChildItem -Path $releaseDir -Filter *.exe -File |
  Where-Object { $_.Name -notmatch '^(build|deps)' } |
  Sort-Object Length -Descending |
  Select-Object -First 1

if (-not $exe) { Fail "nenhum .exe em $releaseDir. Rode npm run build antes." }
Write-Host "[msix] executável: $($exe.FullName)"

# ---------------------------------------------------------------------------
# Layout do pacote
# ---------------------------------------------------------------------------

if (Test-Path $stageDir) { Remove-Item $stageDir -Recurse -Force }
New-Item -ItemType Directory -Path $stageDir -Force | Out-Null

# Nome fixo no layout: o manifesto referencia este nome, independente de como o
# Cargo batizou o binário.
Copy-Item $exe.FullName (Join-Path $stageDir 'FLSChatOverlay.exe')

$assetsSrc = Join-Path $msixDir 'Assets'
if (-not (Test-Path $assetsSrc)) {
  Fail "não encontrei $assetsSrc. Rode 'python scripts/gen_icons.py'."
}
Copy-Item $assetsSrc (Join-Path $stageDir 'Assets') -Recurse

$manifest = Get-Content (Join-Path $msixDir 'AppxManifest.xml') -Raw
$manifest = $manifest.Replace('{IDENTITY_NAME}', $identity.identityName)
$manifest = $manifest.Replace('{PUBLISHER}', $identity.publisher)
$manifest = $manifest.Replace('{VERSION}', $version)
$manifest = $manifest.Replace('{PUBLISHER_DISPLAY_NAME}', $identity.publisherDisplayName)
$manifest = $manifest.Replace('{DISPLAY_NAME}', $identity.displayName)

if ($manifest -match '\{[A-Z_]+\}') {
  Fail "sobrou placeholder não substituído no manifesto: $($Matches[0])"
}

# UTF-8 sem BOM: o makeappx recusa o manifesto com BOM em algumas versões.
[System.IO.File]::WriteAllText(
  (Join-Path $stageDir 'AppxManifest.xml'),
  $manifest,
  (New-Object System.Text.UTF8Encoding($false))
)

# ---------------------------------------------------------------------------
# Empacotar
# ---------------------------------------------------------------------------

$msixPath = Join-Path $outDir 'FLSChatOverlay.msix'
Write-Host '[msix] empacotando...'
& $makeappx pack /d $stageDir /p $msixPath /o
if ($LASTEXITCODE -ne 0) { Fail 'makeappx pack falhou' }

Write-Host "[msix] gerado: $msixPath"
Write-Host "[msix] sha256: $((Get-FileHash $msixPath -Algorithm SHA256).Hash)"

# ---------------------------------------------------------------------------
# Assinatura local (apenas para teste)
# ---------------------------------------------------------------------------

if ($SelfSign) {
  $signtool = Find-SdkTool 'signtool.exe'
  if (-not $signtool) { Fail 'signtool.exe não encontrado no Windows SDK.' }

  Write-Host '[msix] gerando certificado autoassinado para teste local...'
  # O Subject precisa bater exatamente com o Publisher do manifesto, senão a
  # instalação é recusada.
  $cert = New-SelfSignedCertificate `
    -Type Custom `
    -Subject $identity.publisher `
    -KeyUsage DigitalSignature `
    -CertStoreLocation 'Cert:\CurrentUser\My' `
    -TextExtension @('2.5.29.37={text}1.3.6.1.5.5.7.3.3', '2.5.29.19={text}')

  $pfxPath = Join-Path $outDir 'selfsign.pfx'
  $password = ConvertTo-SecureString -String 'msix-local-test' -Force -AsPlainText
  Export-PfxCertificate -Cert $cert -FilePath $pfxPath -Password $password | Out-Null

  & $signtool sign /fd SHA256 /f $pfxPath /p 'msix-local-test' $msixPath
  if ($LASTEXITCODE -ne 0) { Fail 'signtool falhou' }

  $cerPath = Join-Path $outDir 'selfsign.cer'
  Export-Certificate -Cert $cert -FilePath $cerPath | Out-Null

  Write-Host ''
  Write-Host '[msix] pacote assinado para TESTE LOCAL. Para instalar:'
  Write-Host "  Import-Certificate -FilePath '$cerPath' -CertStoreLocation Cert:\LocalMachine\TrustedPeople   # como admin"
  Write-Host "  Add-AppxPackage '$msixPath'"
  Write-Host ''
  Write-Host '[msix] NÃO envie este pacote assinado para a Store — gere um sem -SelfSign.'
}
