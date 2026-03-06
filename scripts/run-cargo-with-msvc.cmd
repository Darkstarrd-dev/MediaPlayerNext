@echo off
call "C:\Program Files\Microsoft Visual Studio\2022\Community\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64
if errorlevel 1 exit /b 1
set PATH=C:\Users\Houpy\.cargo\bin;%PATH%
call "C:\Users\Houpy\.cargo\bin\cargo.exe" %*
