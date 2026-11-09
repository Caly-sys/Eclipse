@echo off
cd /d "%~dp0"
if exist "eclipse.exe" (
    start "" "eclipse.exe" %*
) else if exist "umbra.exe" (
    start "" "umbra.exe" %*
) else if exist "target\release\eclipse.exe" (
    start "" "target\release\eclipse.exe" %*
) else if exist "target\release\umbra.exe" (
    start "" "target\release\umbra.exe" %*
) else (
    echo Building Eclipse...
    cargo run --release -- %*
)
