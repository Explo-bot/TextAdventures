@echo off
rem Lanciatore per Windows: funziona anche con il doppio clic.
chcp 65001 >nul
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0gioca.ps1" %*
set CODICE=%ERRORLEVEL%
rem Pausa solo con il doppio clic: allora la riga di comando di cmd contiene il percorso completo del file.
echo %CMDCMDLINE% | find /i "%~f0" >nul && pause
exit /b %CODICE%
