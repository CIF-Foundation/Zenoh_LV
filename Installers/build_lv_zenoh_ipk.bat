@echo off
rem Windows launcher for the lv_zenoh_ipk build.
rem
rem Requires WSL because the IPK is assembled with Linux tar and bash scripts.
rem Converts the repo path to a WSL path, then runs build_lv_zenoh_ipk.sh.
rem Output is written to Installers\output\lv-zenoh_<version>_<arch>.ipk
setlocal

set "INSTALLERS_DIR=%~dp0"
set "REPO_DIR=%INSTALLERS_DIR%.."

where wsl >nul 2>&1
if errorlevel 1 (
  echo Error: WSL is required to build the lv_zenoh_ipk package.
  exit /b 1
)

rem Translate the Windows repo path to a path WSL can use, e.g. /mnt/d/dev/Packages/Zenoh_LV
for /f "usebackq delims=" %%I in (`wsl wslpath -a "%REPO_DIR%"`) do set "WSL_REPO=%%I"

echo Building lv_zenoh_ipk via WSL...
wsl bash -lc "cd '%WSL_REPO%/Installers' && chmod +x build_lv_zenoh_ipk.sh build_ipk.sh opkg-utils/opkg-build normalize_ipk_line_endings.py && ./build_lv_zenoh_ipk.sh"
if errorlevel 1 (
  echo Error: IPK build failed.
  exit /b 1
)

echo IPK build completed. Output is in Installers\output\
endlocal
