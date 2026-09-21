@echo off
setlocal
cd /d "%~dp0"

if exist "target\release\microwriter.exe" (
    "target\release\microwriter.exe"
) else if exist "target\debug\microwriter.exe" (
    "target\debug\microwriter.exe"
) else (
    where cargo >nul 2>&1
    if errorlevel 1 (
        echo Microwriter is not built and Cargo was not found on PATH.
        echo Build the app with Rust installed, then double-click this file again.
        pause
        exit /b 1
    )
    cargo run --release
)

set "exit_code=%errorlevel%"
if not "%exit_code%"=="0" pause
exit /b %exit_code%
