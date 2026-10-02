@echo off
rem ==========================================================================
rem  microwriter launcher (Windows)
rem
rem  Double-click this file to start microwriter.
rem
rem  If microwriter has not been built on this PC yet, this launcher builds it
rem  once and shows friendly progress (a spinner, a percentage and a bar)
rem  instead of compiler output. The compiler output goes to target\build.log
rem  so it can be shared if setup ever fails.
rem
rem  A build that was interrupted (the window closed, Ctrl+C) or that failed
rem  leaves a broken half-built folder behind. The launcher detects that,
rem  closes this window, deletes the broken build folder, and opens a fresh
rem  window to build again from scratch.
rem ==========================================================================

setlocal
title microwriter
cd /d "%~dp0"

set "RELEASE_BIN=target\release\microwriter.exe"
set "DEBUG_BIN=target\debug\microwriter.exe"
set "BUILD_LOG=target\build.log"
set "BUILD_MARKER=target\build.complete"
set "PROGRESS_SCRIPT=scripts\build_progress.ps1"
set "CARGO=cargo"
set "REBUILD_FLAG=%TEMP%\microwriter_rebuild.flag"
set "RESTART_HELPER=%TEMP%\microwriter_restart.cmd"

rem Did the previous launch already restart this window? If so, this is the
rem fresh run and we must not restart again, or the launcher would loop.
set "RESTARTED=0"
if exist "%REBUILD_FLAG%" (
    set "RESTARTED=1"
    del /f /q "%REBUILD_FLAG%" >nul 2>&1
)

rem --- A build that did not finish? Start over in a clean window. --------
rem A finished build writes a marker beside the app. If the marker is missing
rem but build output is sitting there, the last setup was interrupted or
rem failed, so the folder may be half-written and broken: delete it and start
rem again. One restart only, so a genuine error cannot loop forever.
set "BROKEN=0"
if not exist "%BUILD_MARKER%" (
    if exist "target\release" set "BROKEN=1"
    if exist "target\debug" set "BROKEN=1"
    if exist "%RELEASE_BIN%" set "BROKEN=1"
    if exist "%DEBUG_BIN%" set "BROKEN=1"
)
if "%BROKEN%"=="1" if "%RESTARTED%"=="0" goto restart

rem --- Already built? Start right away, with no setup at all. --------------
if exist "%RELEASE_BIN%" if exist "%BUILD_MARKER%" goto run_release
if exist "%DEBUG_BIN%" if exist "%BUILD_MARKER%" goto run_debug

rem --- First run on this PC: build once, quietly. --------------------------
echo.
echo   microwriter - distraction-free writing environment
echo.
if "%RESTARTED%"=="1" (
    echo   Starting over with a clean build folder.
) else (
    echo   It looks like this is the first time you have opened microwriter here.
    echo   Let me set everything up for you. This only happens once.
)
echo.

where cargo >nul 2>&1
if not errorlevel 1 goto have_cargo
if exist "%USERPROFILE%\.cargo\bin\cargo.exe" set "CARGO=%USERPROFILE%\.cargo\bin\cargo.exe"
if not exist "%USERPROFILE%\.cargo\bin\cargo.exe" goto no_cargo

:have_cargo
if not exist target mkdir target

echo   Getting ready now. This usually takes one to three minutes.
echo   The window shows how far along it is. Writing starts by itself.
echo.

rem Start from a clean slate: withdraw the marker and clear the app's own
rem artifacts, so whatever a stopped build left behind cannot get in the way.
if exist "%BUILD_MARKER%" del /f /q "%BUILD_MARKER%" >nul 2>&1
"%CARGO%" clean -p microwriter >nul 2>&1

rem Friendly progress needs PowerShell; anything unexpected falls back to a
rem plain, silent build rather than failing in front of the user.
where powershell >nul 2>&1
if errorlevel 1 goto plain_build
if not exist "%PROGRESS_SCRIPT%" goto plain_build
powershell -NoProfile -Command "exit 0" >nul 2>&1
if errorlevel 1 goto plain_build

powershell -NoProfile -ExecutionPolicy Bypass -File "%PROGRESS_SCRIPT%" -Cargo "%CARGO%" -Log "%BUILD_LOG%"
if errorlevel 1 goto build_failed
goto built

:plain_build
"%CARGO%" build --release > "%BUILD_LOG%" 2>&1
if errorlevel 1 goto build_failed

:built
rem Record that setup finished, so an interrupted build can be told apart from
rem a complete one the next time this launcher runs.
echo done>"%BUILD_MARKER%"

echo.
echo   All set! Starting microwriter...
echo.
if exist "%RELEASE_BIN%" goto run_release

"%CARGO%" run --release --quiet
set "RC=%ERRORLEVEL%"
goto finish

:build_failed
rem The build did not finish. If this window has not already restarted once,
rem delete the broken build folder and try again in a fresh window; otherwise
rem show the message and stop.
if "%RESTARTED%"=="0" goto restart

echo.
echo   Sorry - setting microwriter up did not finish successfully.
echo   The details were saved in this file:
echo.
echo     target\build.log
echo.
echo   Sharing that file is all that is needed to get this sorted out.
echo.
set "RC=1"
goto finish

:restart
rem Close this window, wait a moment so nothing is holding the folder open,
rem delete the broken build folder, then open a fresh window and build again.
rem A small helper does that in the background after this window is gone.
echo.
echo   This setup did not finish. Closing this window, clearing the broken
echo   build folder, and starting over in a fresh window...
> "%REBUILD_FLAG%" echo restarted
> "%RESTART_HELPER%" echo @echo off
>>"%RESTART_HELPER%" echo timeout /t 2 /nobreak ^>nul
>>"%RESTART_HELPER%" echo rmdir /s /q "%~dp0target" ^>nul 2^>^&1
>>"%RESTART_HELPER%" echo start "" cmd /c "%~f0" %*
>>"%RESTART_HELPER%" echo del /q "%%~f0" ^>nul 2^>^&1
start "" /min cmd /c "%RESTART_HELPER%"
exit /b 0

:no_cargo
echo   microwriter needs a one-time setup with a free tool called Rust.
echo.
echo     1. Visit https://rustup.rs
echo     2. Follow the short instructions there, accepting the defaults.
echo     3. Open microwriter again - everything else happens automatically.
echo.
set "RC=1"
goto finish

:run_release
"%RELEASE_BIN%" %*
set "RC=%ERRORLEVEL%"
goto finish

:run_debug
"%DEBUG_BIN%" %*
set "RC=%ERRORLEVEL%"
goto finish

:finish
rem Keep the window open if something went wrong, so the message above can
rem actually be read before it disappears.
if not "%RC%"=="0" (
    echo.
    pause
)
endlocal & exit /b %RC%
