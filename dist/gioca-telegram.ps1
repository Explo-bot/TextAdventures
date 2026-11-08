<#
.SYNOPSIS
  Avvia il bot Telegram di "Avventura" in primo piano.
.DESCRIPTION
  Controlla il token (TELEGRAM_BOT_TOKEN) e gli utenti ammessi, esegue "avventura telegram --verifica"
  e poi "avventura telegram". Ctrl+C ferma il bot in modo ordinato.
.PARAMETER Profilo
  Profilo AI da usare per il bot.
.PARAMETER Config
  File di configurazione.
.PARAMETER Verifica
  Esegue solo i controlli ed esce.
.PARAMETER InstallaAvvioAutomatico
  Crea l'attività pianificata "Avventura - bot Telegram" che avvia il bot all'accesso dell'utente.
.PARAMETER RimuoviAvvioAutomatico
  Elimina l'attività pianificata.
#>
param(
    [string]$Profilo,
    [string]$Config,
    [switch]$Verifica,
    [switch]$InstallaAvvioAutomatico,
    [switch]$RimuoviAvvioAutomatico
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

$nomeAttivita = 'Avventura - bot Telegram'

if ($RimuoviAvvioAutomatico) {
    if (-not (Get-ScheduledTask -TaskName $nomeAttivita -ErrorAction SilentlyContinue)) {
        Write-Host "L'attività «$nomeAttivita» non esiste."; exit 0
    }
    if (-not (Chiedi "Elimino l'attività pianificata «$nomeAttivita»?")) { exit 1 }
    Unregister-ScheduledTask -TaskName $nomeAttivita -Confirm:$false
    Write-Host 'Attività eliminata.'; exit 0
}

if ($Config -and -not (Test-Path -LiteralPath $Config)) { Errore "File di configurazione non trovato: $Config"; exit 1 }
if ($Profilo -eq 'prova') { $env:AVVENTURA_FORNITORE = 'finto'; $Profilo = '' }
$argomentiComuni = @()
if ($Profilo) { $argomentiComuni += @('--profilo', $Profilo) }
if ($Config)  { $argomentiComuni += @('--config', (Resolve-Path -LiteralPath $Config).Path) }

# ------------------------------------------------------------ eseguibile
$exe = $null
$cmd = Get-Command avventura -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
if ($cmd) { $exe = $cmd.Source }
else {
    foreach ($p in @((Join-Path $radice 'avventura.exe'), (Join-Path $radice 'dist\avventura.exe'),
                     (Join-Path $radice 'motore\target\release\avventura.exe'))) {
        if (Test-Path -LiteralPath $p) { $exe = $p; break }
    }
}
if (-not $exe) { Errore 'Eseguibile "avventura" non trovato: compila con "cargo build --release" in motore\ o esegui installa.ps1.'; exit 1 }

# ------------------------------------------------------------ configurazione [telegram] (lettura minima)
$fileConfig = $null
if ($Config) { $fileConfig = $Config }
else {
    foreach ($p in @((Join-Path $radice 'config.toml'), (Join-Path $radice 'config\config.toml'),
                     (Join-Path (Split-Path -Parent $exe) 'config.toml'))) {
        if (Test-Path -LiteralPath $p) { $fileConfig = $p; break }
    }
}
$nomeVar = 'TELEGRAM_BOT_TOKEN'; $ammessi = ''; $invito = ''
if ($fileConfig) {
    $sezione = ''
    foreach ($riga in (Get-Content -LiteralPath $fileConfig -Encoding UTF8)) {
        $t = $riga.Trim()
        if ($t -match '^\[(.+?)\]\s*(#.*)?$') { $sezione = $Matches[1].Trim(); continue }
        if ($sezione -ne 'telegram') { continue }
        if ($t -match '^token_env\s*=\s*"([^"]+)"') { $nomeVar = $Matches[1] }
        elseif ($t -match '^utenti_ammessi\s*=\s*\[([^\]]*)\]') { $ammessi = $Matches[1].Trim() }
        elseif ($t -match '^codice_invito\s*=\s*"([^"]+)"') { $invito = $Matches[1] }
    }
}

# ------------------------------------------------------------ token
if (-not [Environment]::GetEnvironmentVariable($nomeVar)) {
    Avviso "La variabile $nomeVar (token del bot) non è impostata."
    Write-Host @"
Come ottenere il token:
  1. In Telegram scrivi a @BotFather e invia /newbot.
  2. Scegli un nome e un username (deve finire con "bot"): BotFather ti risponde con il token.
  3. Imposta la variabile d'ambiente (una volta sola) e apri un nuovo terminale:
       [Environment]::SetEnvironmentVariable("$nomeVar", "123456:ABC...", "User")
     Non scrivere il token in config.toml né in repository.
Per i dettagli vedi docs\MANUALE.md (capitolo sul bot Telegram).
"@
    exit 1
}

# ------------------------------------------------------------ utenti ammessi
$fileUtenti = Join-Path $radice 'partite\telegram\utenti.json'
$haUtentiSalvati = (Test-Path -LiteralPath $fileUtenti) -and ((Get-Content -LiteralPath $fileUtenti -Raw) -match '\d')
if (-not $ammessi -and -not $invito -and -not $haUtentiSalvati) {
    Avviso "Attenzione: utenti_ammessi è vuoto e non c'è un codice_invito: nessuno potrà giocare."
    Write-Host 'Per scoprire il tuo ID Telegram scrivi un messaggio al bot: risponde con il tuo ID.'
    Write-Host 'Poi inseriscilo in config.toml, sezione [telegram], per esempio: utenti_ammessi = [123456789]'
}

# ------------------------------------------------------------ avvio automatico (facoltativo)
if ($InstallaAvvioAutomatico) {
    $ps = (Get-Command pwsh -ErrorAction SilentlyContinue)
    $host_ = if ($ps) { $ps.Source } else { (Get-Command powershell).Source }
    $script = Join-Path $radice 'gioca-telegram.ps1'
    $parametri = "-NoProfile -ExecutionPolicy Bypass -WindowStyle Minimized -File `"$script`""
    if ($Profilo) { $parametri += " -Profilo `"$Profilo`"" }
    if ($Config)  { $parametri += " -Config `"$((Resolve-Path -LiteralPath $Config).Path)`"" }
    Write-Host "Creo l'attività «$nomeAttivita»: all'accesso di $env:USERNAME avvia"
    Write-Host "  $host_ $parametri"
    Write-Host "  cartella di lavoro: $radice"
    if (-not (Chiedi 'Procedo?')) { exit 1 }
    $azione = New-ScheduledTaskAction -Execute $host_ -Argument $parametri -WorkingDirectory $radice
    $trigger = New-ScheduledTaskTrigger -AtLogOn -User "$env:USERDOMAIN\$env:USERNAME"
    $impostazioni = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries `
        -ExecutionTimeLimit ([TimeSpan]::Zero) -RestartCount 3 -RestartInterval (New-TimeSpan -Minutes 1)
    Register-ScheduledTask -TaskName $nomeAttivita -Action $azione -Trigger $trigger -Settings $impostazioni `
        -Description 'Avvia il bot Telegram di Avventura' -Force | Out-Null
    Write-Host 'Attività creata. Per toglierla: .\gioca-telegram.ps1 -RimuoviAvvioAutomatico'
    exit 0
}

# ------------------------------------------------------------ avvio
Push-Location $radice
try {
    & $exe telegram --verifica @argomentiComuni
    if ($LASTEXITCODE -ne 0) { Errore 'Verifica non superata: sistema i problemi indicati sopra.'; exit 1 }
    if ($Verifica) { exit 0 }
    & $exe telegram @argomentiComuni
    $codice = $LASTEXITCODE
} finally { Pop-Location }
exit $codice
