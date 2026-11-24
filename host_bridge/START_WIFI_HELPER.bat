@echo off
title Life OS — Windows Wi-Fi Bridge Helper
color 0B
echo ============================================================
echo          LIFE OS — WINDOWS HOST WI-FI BRIDGE HELPER         
echo ============================================================
echo   Project: Life OS (Bare-Metal Rust Operating System)
echo   Owner: Shrivika
echo.
echo Starting PowerShell Wi-Fi Bridge on UDP port 18888...
echo This helper scans real laptop physical Wi-Fi networks and
echo makes them available to the Life OS guest in VirtualBox.
echo.

powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0WiFiBridge.ps1"

pause
