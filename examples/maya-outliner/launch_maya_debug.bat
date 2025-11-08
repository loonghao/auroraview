@echo off
REM Maya Debug Launcher
REM This script kills Maya, rebuilds AuroraView, and launches Maya with proper PYTHONPATH

echo ==========================================
echo Maya Debug Workflow
echo ==========================================
echo.

REM Get the project root directory (2 levels up from this script)
set SCRIPT_DIR=%~dp0
cd /d "%SCRIPT_DIR%..\.."
set PROJECT_ROOT=%CD%

echo [1/4] Killing all Maya processes...
taskkill /F /IM maya.exe 2>nul
if %errorlevel% equ 0 (
    echo [OK] Maya processes terminated
) else (
    echo [OK] No Maya processes running
)
echo.

echo [2/4] Rebuilding Rust core...
call just rebuild-core
if %errorlevel% neq 0 (
    echo [ERROR] Failed to rebuild core
    pause
    exit /b 1
)
echo.

echo [3/4] Setting up environment...
set PYTHONPATH=%PROJECT_ROOT%\python
echo PYTHONPATH=%PYTHONPATH%
echo.

echo [4/4] Launching Maya 2024...
echo Starting Maya with PYTHONPATH set...
start "" /D "C:\Program Files\Autodesk\Maya2024\bin" maya.exe
echo.

echo ==========================================
echo Maya launched with AuroraView in PYTHONPATH
echo ==========================================
echo.
echo In Maya Script Editor, run:
echo   import sys
echo   sys.path.append(r'%PROJECT_ROOT%\examples\maya-outliner')
echo   from maya_integration import maya_outliner
echo   maya_outliner.main()
echo.
echo Press any key to close this window...
pause >nul

