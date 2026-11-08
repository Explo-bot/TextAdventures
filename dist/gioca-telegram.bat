@echo off
rem Avvia il bot Telegram (finestra di cmd: Ctrl+C per fermarlo).
chcp 65001 >nul
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0gioca-telegram.ps1" %*
set CODICE=%ERRORLEVEL%
echo %CMDCMDLINE% | find /i "%~f0" >nul && pause
exit /b %CODICE%
