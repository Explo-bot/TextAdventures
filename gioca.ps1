<#
.SYNOPSIS
  Avvia il gioco "Avventura" (menu di scelta dell'avventura e della partita).
.DESCRIPTION
  Cerca l'eseguibile (PATH, dist\, motore\target\release\), fa controllare al motore configurazione,
  chiave e raggiungibilità del server AI (avventura prova-ai --solo-config), poi lancia
  "avventura gioca" con le opzioni indicate e restituisce il suo codice di uscita.
.PARAMETER Avventura
  Id dell'avventura (nome della sottocartella di avventure\). Omesso: menu di scelta.
.PARAMETER Carica
  Nome di un salvataggio da caricare subito.
.PARAMETER Nuova
  Inizia subito una nuova partita.
.PARAMETER Debug
  Log dettagliato e comandi /state e /raw.
.PARAMETER Profilo
  Profilo AI da usare ([profili.NOME] nel config; vedi "avventura profili").
.PARAMETER Offline
  Usa l'AI simulata (come il profilo "prova"), senza rete e senza chiave.
.PARAMETER Config
  File di configurazione.
.PARAMETER Timeout
  Attesa massima della risposta dell'AI, in secondi.
.PARAMETER Lingua
  Lingua dell'interfaccia per questa sessione: it, en, fr, es o de (senza salvarla).
#>
param(
    [string]$Avventura,
    [string]$Carica,
    [switch]$Nuova,
    [switch]$Debug,
    [string]$Profilo,
    [switch]$Offline,
    [string]$Config,
    [int]$Timeout = 0,
    [ValidateSet('it', 'en', 'fr', 'es', 'de')]
    [string]$Lingua
)

$ErrorActionPreference = 'Stop'
$radice = Split-Path -Parent $MyInvocation.MyCommand.Path
try { [Console]::OutputEncoding = [Text.Encoding]::UTF8 } catch {}
try { chcp 65001 > $null } catch {}

function Avviso($m) { Write-Host $m -ForegroundColor Yellow }
function Errore($m) { Write-Host $m -ForegroundColor Red }
function Chiedi($domanda) {
    try { $r = Read-Host "$domanda (s/n)" } catch { return $false }
    return ($r -match '^(s|si|sì|y|yes)$')
}

if ($Nuova -and $Carica) { Errore 'Le opzioni -Nuova e -Carica non possono essere usate insieme.'; exit 2 }
if ($Timeout -lt 0 -or $Timeout -gt 86400) { Errore '-Timeout deve essere tra 1 e 86400 secondi.'; exit 2 }
if ($Offline -and $Profilo -and $Profilo -ne 'prova') { Errore '-Offline e -Profilo non possono essere usati insieme.'; exit 2 }

# ------------------------------------------------------------ eseguibile
function Trova-Eseguibile([string[]]$nomi) {
    foreach ($nome in $nomi) {
        $cmd = Get-Command $nome -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($cmd) { return $cmd.Source }
    }
    foreach ($nome in $nomi) {
        foreach ($cartella in @($radice, (Join-Path $radice 'dist'), (Join-Path $radice 'motore\target\release'))) {
            $p = Join-Path $cartella "$nome.exe"
            if (Test-Path -LiteralPath $p) { return $p }
        }
    }
    return $null
}

# -Offline: AI simulata. Senza dipendere dall'esistenza di [profili.prova] nel config.
if ($Offline -or $Profilo -eq 'prova') {
    $env:AVVENTURA_FORNITORE = 'finto'
    $Profilo = ''
}
if ($Config -and -not (Test-Path -LiteralPath $Config)) { Errore "File di configurazione non trovato: $Config"; exit 1 }
function Argomenti-Comuni {
    $a = @()
    if ($Profilo) { $a += @('--profilo', $Profilo) }
    if ($Config)  { $a += @('--config', (Resolve-Path -LiteralPath $Config).Path) }
    return ,$a
}
$argomentiComuni = Argomenti-Comuni

$exe = Trova-Eseguibile @('avventura')
if (-not $exe) {
    Avviso 'Eseguibile "avventura" non trovato (PATH, dist\, motore\target\release\).'
    $manifest = Join-Path $radice 'motore\Cargo.toml'
    if (-not (Test-Path -LiteralPath $manifest)) {
        Errore 'Sorgenti non presenti: esegui installa.ps1 o scarica un pacchetto già compilato.'; exit 1
    }
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Errore 'Rust (cargo) non è installato: vedi https://rustup.rs oppure usa il pacchetto già compilato.'; exit 1
    }
    if (-not (Chiedi 'Compilo ora con "cargo build --release"?')) { exit 1 }
    Push-Location (Join-Path $radice 'motore')
    try { & cargo build --release; $esito = $LASTEXITCODE } finally { Pop-Location }
    if ($esito -ne 0) { Errore 'Compilazione fallita.'; exit 1 }
    $exe = Trova-Eseguibile @('avventura')
    if (-not $exe) { Errore 'Compilazione riuscita ma eseguibile non trovato.'; exit 1 }
}

# Esegue l'eseguibile e restituisce stdout+stderr e codice di uscita senza che Windows PowerShell 5.1
# trasformi lo stderr in errori rossi.
function Esegui-Catturando([string]$eseguibile, [string[]]$argomenti) {
    $psi = New-Object Diagnostics.ProcessStartInfo
    $psi.FileName = $eseguibile
    $psi.Arguments = (($argomenti | ForEach-Object { if ($_ -match '[\s"]') { '"' + ($_ -replace '"', '\"') + '"' } else { $_ } }) -join ' ')
    $psi.WorkingDirectory = $radice
    $psi.UseShellExecute = $false
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.StandardOutputEncoding = [Text.Encoding]::UTF8
    $psi.StandardErrorEncoding = [Text.Encoding]::UTF8
    $proc = [Diagnostics.Process]::Start($psi)
    $errTask = $proc.StandardError.ReadToEndAsync()
    $out = $proc.StandardOutput.ReadToEnd()
    $proc.WaitForExit()
    return @{ Testo = $out + $errTask.Result; Codice = $proc.ExitCode }
}
# Il profilo scelto usa il fornitore "locale" (modello GGUF)? Serve la variante con llama.cpp.
function Usa-Locale([string]$eseguibile) {
    if ($env:AVVENTURA_FORNITORE) { return ($env:AVVENTURA_FORNITORE.Trim().ToLower() -eq 'locale') }
    try {
        $r = Esegui-Catturando $eseguibile (@('profili') + $argomentiComuni)
        foreach ($riga in ($r.Testo -split "`r?`n")) { if ($riga -match '^\s*\*\s' -and $riga -match 'fornitore locale\b') { return $true } }
    } catch {}
    return $false
}
if (Usa-Locale $exe) {
    $variante = Trova-Eseguibile @('avventura-cuda', 'avventura-locale')
    if ($variante) { $exe = $variante }
    else { Avviso 'Il profilo usa il fornitore "locale" ma non trovo avventura-cuda.exe / avventura-locale.exe: l''eseguibile standard ti darà una spiegazione.' }
}

# ------------------------------------------------------------ controlli preliminari (delegati al motore)
$controllo = Esegui-Catturando $exe (@('prova-ai', '--solo-config') + $argomentiComuni)
$testo = $controllo.Testo
$esito = $controllo.Codice
if ($esito -ne 0) {
    Write-Host $testo.TrimEnd()
    if ($Offline) { exit 1 }
    if (Chiedi 'Avvio in modalità offline (AI simulata, profilo "prova")?') {
        $env:AVVENTURA_FORNITORE = 'finto'
        $Profilo = ''
        $argomentiComuni = Argomenti-Comuni
    } else {
        Errore 'Sistema la configurazione e riprova (vedi docs\MANUALE.md).'; exit 1
    }
}

# ------------------------------------------------------------ avvio
$argomenti = @('gioca')
if ($Avventura) { $argomenti += @('--avventura', $Avventura) }
if ($Carica)    { $argomenti += @('--carica', $Carica) }
if ($Nuova)     { $argomenti += '--nuova' }
if ($Debug)     { $argomenti += '--debug' }
if ($Timeout -gt 0) { $argomenti += @('--timeout', "$Timeout") }
if ($Lingua)    { $argomenti += @('--lingua', $Lingua) }
$argomenti += $argomentiComuni

Push-Location $radice
try { & $exe @argomenti; $codice = $LASTEXITCODE } finally { Pop-Location }
exit $codice
