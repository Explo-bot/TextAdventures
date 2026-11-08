<#
.SYNOPSIS
  Installa (o disinstalla) il motore di avventure testuali "Avventura".
.DESCRIPTION
  Copia l'eseguibile, config.esempio.toml e le avventure nella cartella di destinazione
  e crea la cartella modelli\ per i file GGUF.
  Non tocca mai: modelli\ esistente, partite\ (stati e salvataggi), un config.toml esistente
  e le avventure aggiunte dall'utente.
.PARAMETER Destinazione
  Cartella di installazione (default: %LOCALAPPDATA%\Programs\Avventura).
.PARAMETER DaSorgente
  Compila con "cargo build --release" prima di copiare.
.PARAMETER ConModelloLocale
  Installa anche la variante con modello locale GGUF (avventura-cuda.exe).
  Con -DaSorgente la compila con --features cuda (o --features locale se CUDA manca).
.PARAMETER NoPath
  Non modifica il PATH dell'utente.
.PARAMETER Disinstalla
  Rimuove eseguibili e voce del PATH; chiede conferma separata per partite\, avventure\,
  config.toml e modelli\.
#>
[CmdletBinding()]
param(
    [string]$Destinazione = (Join-Path $env:LOCALAPPDATA 'Programs\Avventura'),
    [switch]$DaSorgente,
    [switch]$ConModelloLocale,
    [switch]$NoPath,
    [switch]$Disinstalla
)

$ErrorActionPreference = 'Stop'
$radice = Split-Path -Parent $MyInvocation.MyCommand.Path
$Destinazione = [System.IO.Path]::GetFullPath($Destinazione)

function Info($m)   { Write-Host $m -ForegroundColor Cyan }
function Ok($m)     { Write-Host $m -ForegroundColor Green }
function Avviso($m) { Write-Host $m -ForegroundColor Yellow }
function Errore($m) { Write-Host $m -ForegroundColor Red }

function Chiedi($domanda) {
    try {
        $r = Read-Host "$domanda (s/n)"
    } catch {
        return $false   # sessione non interattiva: risposta negativa
    }
    return ($r -match '^(s|si|sì|y|yes)$')
}

function Normalizza($p) { return $p.TrimEnd('\', '/') }

function Rimuovi-DaPath($cartella) {
    $attuale = [Environment]::GetEnvironmentVariable('Path', 'User')
    if (-not $attuale) { return $false }
    $voci = @($attuale -split ';' | Where-Object { $_ -ne '' })
    $nuove = @($voci | Where-Object { (Normalizza $_) -ine (Normalizza $cartella) })
    if ($nuove.Count -ne $voci.Count) {
        [Environment]::SetEnvironmentVariable('Path', ($nuove -join ';'), 'User')
        return $true
    }
    return $false
}

function Ha-GpuNvidia {
    if (-not (Get-Command nvidia-smi -ErrorAction SilentlyContinue)) { return $false }
    try {
        & nvidia-smi *> $null
        return ($LASTEXITCODE -eq 0)
    } catch { return $false }
}

function Ha-Cuda {
    if ($env:CUDA_PATH) { return $true }
    return [bool](Get-Command nvcc -ErrorAction SilentlyContinue)
}

$testoModelli = @"
Modelli locali (file GGUF)
=========================
Metti qui i file .gguf (ad esempio gemma-4-12b-it-Q4_K_M.gguf) e indica il percorso nel
profilo "gguf" di config.toml (campo percorso_modello).
Il fornitore "locale" funziona solo con la variante avventura-cuda.exe (GPU NVIDIA),
non con avventura.exe standard.
Alternativa senza file GGUF: Ollama o LM Studio (profili ollama / lmstudio).
Questa cartella non viene mai toccata dall'installatore.
"@

# ---------------------------------------------------------------- disinstallazione
if ($Disinstalla) {
    Info "Disinstallazione da: $Destinazione"
    if (-not (Test-Path -LiteralPath $Destinazione)) {
        Avviso 'Nessuna installazione trovata in questa cartella.'
    } else {
        foreach ($f in 'avventura.exe', 'avventura-cuda.exe', 'avventura-locale.exe', 'config.esempio.toml') {
            $p = Join-Path $Destinazione $f
            if (Test-Path -LiteralPath $p) {
                try { Remove-Item -LiteralPath $p -Force; Ok "Rimosso $f" }
                catch { Avviso "Impossibile rimuovere $f (in esecuzione?)." }
            }
        }
        if (Rimuovi-DaPath $Destinazione) { Ok 'Voce rimossa dal PATH utente.' }

        $partite = Join-Path $Destinazione 'partite'
        if (Test-Path -LiteralPath $partite) {
            if (Chiedi 'Cancellare anche partite\ (stati e SALVATAGGI)? Operazione irreversibile') {
                Remove-Item -LiteralPath $partite -Recurse -Force; Ok 'Cartella partite rimossa.'
            } else { Info 'partite\ conservata.' }
        }
        $avv = Join-Path $Destinazione 'avventure'
        if (Test-Path -LiteralPath $avv) {
            if (Chiedi 'Cancellare anche avventure\ (comprese quelle create da te)? Operazione irreversibile') {
                Remove-Item -LiteralPath $avv -Recurse -Force; Ok 'Cartella avventure rimossa.'
            } else { Info 'avventure\ conservata.' }
        }
        $cfg = Join-Path $Destinazione 'config.toml'
        if (Test-Path -LiteralPath $cfg) {
            if (Chiedi 'Cancellare anche config.toml?') { Remove-Item -LiteralPath $cfg -Force; Ok 'config.toml rimosso.' }
            else { Info 'config.toml conservato.' }
        }
        $mod = Join-Path $Destinazione 'modelli'
        if (Test-Path -LiteralPath $mod) {
            $contenuto = @(Get-ChildItem -LiteralPath $mod -Force | Where-Object { $_.Name -ne 'LEGGIMI.txt' })
            if ($contenuto.Count -eq 0) {
                Remove-Item -LiteralPath $mod -Recurse -Force
            } elseif (Chiedi 'Cancellare anche modelli\ (i tuoi file GGUF)? Operazione irreversibile') {
                Remove-Item -LiteralPath $mod -Recurse -Force; Ok 'Cartella modelli rimossa.'
            } else { Info 'modelli\ conservata.' }
        }
        # rimuove la cartella di installazione solo se vuota
        if ((Test-Path -LiteralPath $Destinazione) -and -not (Get-ChildItem -LiteralPath $Destinazione -Force)) {
            Remove-Item -LiteralPath $Destinazione -Force
        }
    }
    Ok 'Disinstallazione completata.'
    exit 0
}

# ---------------------------------------------------------------- sorgenti
$exe = $null
$exeLocale = $null
$nomeExeLocale = 'avventura-cuda.exe'
$fileConfig = $null
$cartellaAvv = $null

if ($DaSorgente) {
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Errore 'Impossibile compilare: "cargo" non trovato.'
        Write-Host 'Installa Rust con:  winget install Rustlang.Rustup' -ForegroundColor Yellow
        Write-Host 'e i Microsoft C++ Build Tools con il carico di lavoro "Sviluppo di applicazioni desktop con C++".' -ForegroundColor Yellow
        exit 1
    }
    $staging = Join-Path ([System.IO.Path]::GetTempPath()) ('avventura-build-' + [guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Force -Path $staging | Out-Null
    $prodotto = Join-Path $radice 'motore\target\release\avventura.exe'

    Info 'Compilazione in corso (cargo build --release)...'
    Push-Location (Join-Path $radice 'motore')
    try {
        & cargo build --release
        if ($LASTEXITCODE -ne 0) { Errore 'Compilazione fallita.'; exit 1 }
        $exe = Join-Path $staging 'avventura.exe'
        Copy-Item -LiteralPath $prodotto -Destination $exe -Force

        if ($ConModelloLocale) {
            if (Ha-Cuda) { $feature = 'cuda'; $nomeExeLocale = 'avventura-cuda.exe' }
            else {
                $feature = 'locale'; $nomeExeLocale = 'avventura-locale.exe'
                Avviso 'CUDA non trovato: compilo la variante solo CPU (feature "locale").'
            }
            Info "Compilazione della variante con modello locale (--features $feature)..."
            & cargo build --release --features $feature
            if ($LASTEXITCODE -ne 0) { Errore "Compilazione della variante $feature fallita."; exit 1 }
            $exeLocale = Join-Path $staging $nomeExeLocale
            Copy-Item -LiteralPath $prodotto -Destination $exeLocale -Force
        }
    } finally { Pop-Location }

    $cartellaAvv = Join-Path $radice 'avventure'
    if (-not (Test-Path -LiteralPath $cartellaAvv)) { $cartellaAvv = Join-Path $radice 'dist\avventure' }
    $fileConfig = Join-Path $radice 'config\config.esempio.toml'
    if (-not (Test-Path -LiteralPath $fileConfig)) { $fileConfig = Join-Path $radice 'dist\config.esempio.toml' }
} else {
    $exe = Join-Path $radice 'dist\avventura.exe'
    if (-not (Test-Path -LiteralPath $exe)) { $exe = Join-Path $radice 'avventura.exe' }

    foreach ($nome in 'avventura-cuda.exe', 'avventura-locale.exe') {
        foreach ($base in (Join-Path $radice 'dist'), $radice) {
            $c = Join-Path $base $nome
            if (-not $exeLocale -and (Test-Path -LiteralPath $c)) { $exeLocale = $c; $nomeExeLocale = $nome }
        }
    }

    $cartellaAvv = Join-Path $radice 'dist\avventure'
    if (-not (Test-Path -LiteralPath $cartellaAvv)) { $cartellaAvv = Join-Path $radice 'avventure' }
    foreach ($c in 'dist\config.esempio.toml', 'config.esempio.toml', 'config\config.esempio.toml') {
        if (-not $fileConfig -and (Test-Path -LiteralPath (Join-Path $radice $c))) { $fileConfig = Join-Path $radice $c }
    }

    # Proposta della variante locale se c'è una GPU NVIDIA
    if ($exeLocale -and -not $ConModelloLocale) {
        if ((Ha-GpuNvidia) -and (Chiedi "Rilevata una GPU NVIDIA: installo anche $nomeExeLocale (modelli GGUF locali)?")) {
            $ConModelloLocale = [System.Management.Automation.SwitchParameter]::new($true)
        }
    }
    if ($ConModelloLocale -and -not $exeLocale) {
        Avviso "$nomeExeLocale non disponibile nel pacchetto: installo solo la versione standard (usa -DaSorgente per compilarla)."
    }
}

if (-not (Test-Path -LiteralPath $exe)) {
    Errore "Eseguibile non trovato: $exe"
    Avviso 'Compila prima il programma oppure rilancia con -DaSorgente.'
    exit 1
}
if (-not (Test-Path -LiteralPath $cartellaAvv)) {
    Errore "Cartella delle avventure non trovata: $cartellaAvv"
    exit 1
}

# ---------------------------------------------------------------- copia
Info "Installazione in: $Destinazione"
New-Item -ItemType Directory -Force -Path $Destinazione | Out-Null

function Copia-Eseguibile($origine, $nome) {
    try {
        Copy-Item -LiteralPath $origine -Destination (Join-Path $Destinazione $nome) -Force
    } catch {
        Errore "Impossibile sovrascrivere ${nome}: è in esecuzione? Chiudi il gioco e riprova."
        exit 1
    }
    Ok "Copiato $nome."
}

Copia-Eseguibile $exe 'avventura.exe'
$varianteInstallata = $null
if ($ConModelloLocale -and $exeLocale) {
    Copia-Eseguibile $exeLocale $nomeExeLocale
    $varianteInstallata = $nomeExeLocale
}

# modelli\ : creata se manca, mai toccata se esiste
$cartellaModelli = Join-Path $Destinazione 'modelli'
if (-not (Test-Path -LiteralPath $cartellaModelli)) {
    New-Item -ItemType Directory -Force -Path $cartellaModelli | Out-Null
    $leggimi = Join-Path $cartellaModelli 'LEGGIMI.txt'
    $sorgenteLeggimi = Join-Path $radice 'dist\modelli\LEGGIMI.txt'
    if (Test-Path -LiteralPath $sorgenteLeggimi) { Copy-Item -LiteralPath $sorgenteLeggimi -Destination $leggimi }
    else { Set-Content -LiteralPath $leggimi -Value $testoModelli -Encoding UTF8 }
    Ok 'Creata la cartella modelli\ (qui vanno i file GGUF).'
} else {
    Info 'modelli\ esistente: non modificata.'
}

if ($fileConfig -and (Test-Path -LiteralPath $fileConfig)) {
    Copy-Item -LiteralPath $fileConfig -Destination (Join-Path $Destinazione 'config.esempio.toml') -Force
    $cfg = Join-Path $Destinazione 'config.toml'
    if (-not (Test-Path -LiteralPath $cfg)) {
        Copy-Item -LiteralPath $fileConfig -Destination $cfg
        Ok 'Creato config.toml da config.esempio.toml.'
    } else {
        Info 'config.toml esistente: non modificato.'
    }
} else {
    Avviso 'config.esempio.toml non trovato: saltato.'
}

# Avventure: sostituisce quelle del pacchetto (così non restano file di versioni
# precedenti), non tocca le altre.
$destAvv = Join-Path $Destinazione 'avventure'
New-Item -ItemType Directory -Force -Path $destAvv | Out-Null
# Versioni inglesi separate delle avventure di esempio, sostituite da un'unica
# versione con traduzioni/it.toml.
foreach ($vecchia in @('faro-punta-nera-en', 'space-chicken-en')) {
    $p = Join-Path $destAvv $vecchia
    if (Test-Path -LiteralPath $p) {
        Remove-Item -LiteralPath $p -Recurse -Force
        Info "Rimossa la vecchia versione separata: $vecchia"
    }
}
$contate = 0
foreach ($a in Get-ChildItem -LiteralPath $cartellaAvv -Directory) {
    $dest = Join-Path $destAvv $a.Name
    if (Test-Path -LiteralPath $dest) { Remove-Item -LiteralPath $dest -Recurse -Force }
    New-Item -ItemType Directory -Force -Path $dest | Out-Null
    foreach ($voce in Get-ChildItem -LiteralPath $a.FullName -Force) {
        Copy-Item -LiteralPath $voce.FullName -Destination $dest -Recurse -Force
    }
    $contate++
    Ok "Avventura aggiornata: $($a.Name)"
}
if ($contate -eq 0) { Avviso 'Nessuna avventura trovata nel pacchetto.' }
# partite\ non viene mai toccata.

if ($DaSorgente -and $staging -and (Test-Path -LiteralPath $staging)) {
    Remove-Item -LiteralPath $staging -Recurse -Force -ErrorAction SilentlyContinue
}

# ---------------------------------------------------------------- PATH
$pathAggiunto = $false
if (-not $NoPath) {
    $attuale = [Environment]::GetEnvironmentVariable('Path', 'User')
    $voci = @()
    if ($attuale) { $voci = @($attuale -split ';' | Where-Object { $_ -ne '' }) }
    $presente = @($voci | Where-Object { (Normalizza $_) -ieq (Normalizza $Destinazione) })
    if ($presente.Count -eq 0) {
        $nuovo = ($voci + $Destinazione) -join ';'
        [Environment]::SetEnvironmentVariable('Path', $nuovo, 'User')
        $pathAggiunto = $true
        Ok 'Cartella aggiunta al PATH utente.'
    } else {
        Info 'Cartella già presente nel PATH utente.'
    }
}

# ---------------------------------------------------------------- riepilogo
Write-Host ''
Ok '=== Installazione completata ==='
Write-Host 'Prossimi passi:' -ForegroundColor Cyan
Write-Host "  1. Scegli un profilo in  $(Join-Path $Destinazione 'config.toml')" -ForegroundColor White
Write-Host '       - cloud (OpenAI, Anthropic, Gemini): imposta la chiave, es.  setx OPENAI_API_KEY "la-tua-chiave"' -ForegroundColor Gray
Write-Host '         (poi riapri il terminale)' -ForegroundColor Gray
Write-Host '       - Ollama o LM Studio: nessuna chiave, basta che il server sia avviato' -ForegroundColor Gray
Write-Host "       - modello GGUF locale: copia il file in  $(Join-Path $Destinazione 'modelli\')  e usa il profilo gguf" -ForegroundColor Gray
if ($varianteInstallata) {
    Write-Host "         (e avvia  $varianteInstallata  invece di avventura.exe)" -ForegroundColor Gray
}
Write-Host '  2. Prova il collegamento:   avventura prova-ai' -ForegroundColor White
Write-Host '  3. Avvia il gioco:          avventura' -ForegroundColor White
Write-Host '     Oppure il bot Telegram:  avventura telegram' -ForegroundColor White
Write-Host "  4. Nuove avventure: copiale in  $(Join-Path $Destinazione 'avventure\<nome>\')  (con gioco.md e stanze\)." -ForegroundColor White
if ($pathAggiunto) { Avviso 'Apri un NUOVO terminale perché il PATH aggiornato abbia effetto.' }
Write-Host "Salvataggi e partite: $(Join-Path $Destinazione 'partite\') (creata dal programma al primo avvio)" -ForegroundColor Gray
