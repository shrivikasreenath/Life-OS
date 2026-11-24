@echo off
title Life OS - Windows Host Wi-Fi Bridge
cd /d "%~dp0"
echo ============================================================
echo          LIFE OS - WINDOWS HOST WI-FI BRIDGE HELPER
echo ============================================================
echo Owner: Shrivika
echo Starting UDP server on port 18888...
echo.
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0WiFiBridge.ps1"
pause
