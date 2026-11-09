@echo off
cd /d "%~dp0"
if exist "umbra.exe" (
    start "" "umbra.exe" %*
) else if exist "target\release\umbra.exe" (
    start "" "target\release\umbra.exe" %*
) else (
    echo Building Umbra...
    cargo run --release -- %*
)
