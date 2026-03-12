@echo off
call "C:\Program Files\Microsoft Visual Studio\2022\Community\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64
if errorlevel 1 exit /b 1
if not defined HTTP_PROXY set HTTP_PROXY=http://127.0.0.1:2080
if not defined HTTPS_PROXY set HTTPS_PROXY=http://127.0.0.1:2080
set PATH=C:\Users\Houpy\.cargo\bin;%PATH%
if "%~1"=="" (
  call npm exec tauri build
) else (
  call npm exec tauri build -- %*
)
