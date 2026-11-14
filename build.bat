@echo off
setlocal EnableExtensions EnableDelayedExpansion
cd /d "%~dp0"

where cargo >nul 2>&1
if errorlevel 1 (
  echo ERROR: cargo not found on PATH.
  echo Install Rust and add cargo to PATH, or check your Environment Variables.
  echo.
  pause
  exit /b 1
)

if not exist Cargo.toml (
  echo ERROR: Cargo.toml not found in current directory.
  echo.
  pause
  exit /b 1
)

if not exist version.ini (
  echo ERROR: version.ini not found in current directory.
  echo The version number is embedded at compile time.
  echo.
  pause
  exit /b 1
)

if not exist assets\icon.ico (
  echo ERROR: assets\icon.ico not found.
  echo The icon resource is generated from this file at compile time.
  echo.
  pause
  exit /b 1
)

set "OUTDIR=dist"
set "EXE_NAME=WU-Blocker.exe"
set "BUILT=target\release\wu-blocker.exe"

cls

if exist "%OUTDIR%" rmdir /s /q "%OUTDIR%" >nul 2>&1
mkdir "%OUTDIR%" 2>nul

cargo build --release
if errorlevel 1 goto :fail

if not exist "%BUILT%" goto :fail

copy /y "%BUILT%" "%OUTDIR%\%EXE_NAME%" >nul 2>&1
if not exist "%OUTDIR%\%EXE_NAME%" goto :fail

echo.
echo BUILD OK  -^> %OUTDIR%\%EXE_NAME%
echo.

timeout /t 3 /nobreak >nul
exit /b 0

:fail
echo.
echo BUILD FAILED
echo.
pause
exit /b 1
