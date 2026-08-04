@echo off
setlocal
cd /d "%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\windows-acceptance.ps1 -Mode Full -LogDirectory .\logs
if errorlevel 1 pause
