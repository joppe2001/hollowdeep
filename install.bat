@echo off
REM Hollowdeep Installation Script for Windows (Batch wrapper)
REM This script launches the PowerShell installer

echo ======================================
echo   Hollowdeep Installation Script
echo ======================================
echo.

REM Check if PowerShell is available
where powershell >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo [ERROR] PowerShell is required but not found.
    echo Please install PowerShell or run install.ps1 directly.
    pause
    exit /b 1
)

REM Run the PowerShell script
powershell -ExecutionPolicy Bypass -File "%~dp0install.ps1" %*

if %ERRORLEVEL% neq 0 (
    echo.
    echo Installation encountered an error.
    pause
    exit /b %ERRORLEVEL%
)

pause
