@echo off
setlocal
cd /d "%~dp0"
call npm run verify:all
if errorlevel 1 pause
