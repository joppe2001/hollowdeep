@echo off
REM Hollowdeep Windows Installer
REM Double-click to run, or execute from Command Prompt

powershell -ExecutionPolicy Bypass -Command ^
$ErrorActionPreference = 'Stop'; ^
function Write-Info { Write-Host '[INFO]' $args -ForegroundColor Green }; ^
function Write-Warn { Write-Host '[WARN]' $args -ForegroundColor Yellow }; ^
function Write-Err { Write-Host '[ERROR]' $args -ForegroundColor Red; exit 1 }; ^
Write-Host ''; ^
Write-Host '======================================' -ForegroundColor Cyan; ^
Write-Host '  Hollowdeep Windows Installer' -ForegroundColor Cyan; ^
Write-Host '======================================' -ForegroundColor Cyan; ^
Write-Host ''; ^
$rustInstalled = $false; ^
try { $v = rustc --version 2>$null; if ($LASTEXITCODE -eq 0) { Write-Info \"Rust installed: $v\"; $rustInstalled = $true } } catch {}; ^
if (-not $rustInstalled) { ^
    Write-Warn 'Rust is not installed'; ^
    $r = Read-Host 'Install Rust via rustup? [Y/n]'; ^
    if ($r -eq 'n') { Write-Err 'Rust is required' }; ^
    Write-Info 'Downloading rustup...'; ^
    Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile \"$env:TEMP\rustup-init.exe\"; ^
    Write-Info 'Installing Rust (this may take a few minutes)...'; ^
    Start-Process -FilePath \"$env:TEMP\rustup-init.exe\" -ArgumentList '-y' -Wait -NoNewWindow; ^
    $env:PATH = \"$env:USERPROFILE\.cargo\bin;$env:PATH\"; ^
    Remove-Item \"$env:TEMP\rustup-init.exe\" -Force -ErrorAction SilentlyContinue; ^
    Write-Info 'Rust installed - you may need to restart your terminal after this script' ^
}; ^
Write-Info 'Building Hollowdeep (release mode)...'; ^
cargo build --release; ^
if ($LASTEXITCODE -ne 0) { Write-Err 'Build failed' }; ^
Write-Info 'Build complete!'; ^
Write-Host ''; ^
Write-Host 'Run the game with: .\target\release\hollowdeep.exe' -ForegroundColor Green; ^
Write-Host ''; ^
Write-Host 'For best experience, use Windows Terminal or WezTerm.' -ForegroundColor Yellow; ^
pause

