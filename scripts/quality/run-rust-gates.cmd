@echo off
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0run-rust-gates.ps1"
exit /b %errorlevel%
