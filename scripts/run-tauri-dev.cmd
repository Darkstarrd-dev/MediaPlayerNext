@echo off
call "C:\Program Files\Microsoft Visual Studio\2022\Community\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64
if errorlevel 1 exit /b 1
if not defined HTTP_PROXY set HTTP_PROXY=http://127.0.0.1:3066
if not defined HTTPS_PROXY set HTTPS_PROXY=http://127.0.0.1:3066
if not defined http_proxy set http_proxy=%HTTP_PROXY%
if not defined https_proxy set https_proxy=%HTTPS_PROXY%
if not defined ALL_PROXY set ALL_PROXY=%HTTP_PROXY%
if not defined all_proxy set all_proxy=%HTTP_PROXY%
set PATH=C:\Users\Houpy\.cargo\bin;%PATH%
call npm exec tauri dev
