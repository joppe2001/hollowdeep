# Hollowdeep Installation Script for Windows (PowerShell)

param(
    [string]$InstallDir = "$env:LOCALAPPDATA\Hollowdeep",
    [switch]$SkipRustInstall,
    [switch]$BuildOnly
)

$ErrorActionPreference = "Stop"

# Colors
function Write-Info { Write-Host "[INFO] $args" -ForegroundColor Green }
function Write-Warn { Write-Host "[WARN] $args" -ForegroundColor Yellow }
function Write-Err { Write-Host "[ERROR] $args" -ForegroundColor Red; exit 1 }

# Check if running as administrator (not required but noted)
function Test-Administrator {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($identity)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

# Check if Rust is installed
function Test-RustInstalled {
    try {
        $rustVersion = & rustc --version 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-Info "Rust is installed: $rustVersion"
            return $true
        }
    } catch {}
    return $false
}

# Install Rust via rustup
function Install-Rust {
    Write-Info "Downloading rustup installer..."

    $rustupUrl = "https://win.rustup.rs/x86_64"
    $rustupPath = "$env:TEMP\rustup-init.exe"

    try {
        Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath -UseBasicParsing
    } catch {
        Write-Err "Failed to download rustup: $_"
    }

    Write-Info "Installing Rust (this may take a few minutes)..."

    # Run rustup installer with default options
    Start-Process -FilePath $rustupPath -ArgumentList "-y" -Wait -NoNewWindow

    # Update PATH for current session
    $cargoPath = "$env:USERPROFILE\.cargo\bin"
    if ($env:PATH -notlike "*$cargoPath*") {
        $env:PATH = "$cargoPath;$env:PATH"
    }

    # Verify installation
    if (Test-RustInstalled) {
        Write-Info "Rust installed successfully"
    } else {
        Write-Warn "Rust was installed but may require a new terminal session"
        Write-Warn "Please restart your terminal and run this script again"
        exit 0
    }

    Remove-Item $rustupPath -Force -ErrorAction SilentlyContinue
}

# Check for Visual Studio Build Tools
function Test-BuildTools {
    $vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"

    if (Test-Path $vsWhere) {
        $installations = & $vsWhere -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -format json | ConvertFrom-Json
        if ($installations.Count -gt 0) {
            Write-Info "Visual Studio Build Tools found"
            return $true
        }
    }

    # Check for standalone Build Tools
    if (Test-Path "${env:ProgramFiles(x86)}\Microsoft Visual Studio\*\BuildTools") {
        Write-Info "Visual Studio Build Tools found"
        return $true
    }

    return $false
}

# Build the project
function Build-Project {
    Write-Info "Building Hollowdeep (release mode)..."

    try {
        & cargo build --release
        if ($LASTEXITCODE -ne 0) {
            Write-Err "Build failed with exit code $LASTEXITCODE"
        }
    } catch {
        Write-Err "Build failed: $_"
    }

    Write-Info "Build complete"
}

# Install to system
function Install-ToSystem {
    param([string]$DestDir)

    Write-Info "Installing to $DestDir..."

    $binDir = Join-Path $DestDir "bin"
    $dataDir = Join-Path $DestDir "data"

    # Create directories
    New-Item -ItemType Directory -Force -Path $binDir | Out-Null
    New-Item -ItemType Directory -Force -Path $dataDir | Out-Null

    # Copy binary
    Copy-Item "target\release\hollowdeep.exe" -Destination $binDir -Force
    Write-Info "Binary installed to $binDir\hollowdeep.exe"

    # Copy assets
    if (Test-Path "assets") {
        Copy-Item "assets\*" -Destination $dataDir -Recurse -Force
        Write-Info "Assets installed to $dataDir"
    }

    # Add to user PATH
    $userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($userPath -notlike "*$binDir*") {
        Write-Info "Adding $binDir to user PATH..."
        [Environment]::SetEnvironmentVariable("PATH", "$userPath;$binDir", "User")
        $env:PATH = "$env:PATH;$binDir"
        Write-Info "PATH updated (may require new terminal session)"
    }
}

# Create desktop shortcut
function New-DesktopShortcut {
    param([string]$TargetPath)

    $desktopPath = [Environment]::GetFolderPath("Desktop")
    $shortcutPath = Join-Path $desktopPath "Hollowdeep.lnk"

    $shell = New-Object -ComObject WScript.Shell
    $shortcut = $shell.CreateShortcut($shortcutPath)
    $shortcut.TargetPath = $TargetPath
    $shortcut.WorkingDirectory = Split-Path $TargetPath
    $shortcut.Description = "Hollowdeep - A grimdark terminal roguelike RPG"
    $shortcut.Save()

    Write-Info "Desktop shortcut created"
}

# Main
function Main {
    Write-Host ""
    Write-Host "======================================" -ForegroundColor Cyan
    Write-Host "  Hollowdeep Installation Script" -ForegroundColor Cyan
    Write-Host "======================================" -ForegroundColor Cyan
    Write-Host ""

    # Check for Rust
    if (-not (Test-RustInstalled)) {
        if ($SkipRustInstall) {
            Write-Err "Rust is not installed. Install it from https://rustup.rs"
        }

        Write-Warn "Rust is not installed"
        $response = Read-Host "Install Rust via rustup? [Y/n]"
        if ($response -eq "n" -or $response -eq "N") {
            Write-Err "Rust is required to build Hollowdeep"
        }
        Install-Rust
    }

    # Check for build tools
    if (-not (Test-BuildTools)) {
        Write-Warn "Visual Studio Build Tools not detected"
        Write-Host ""
        Write-Host "Rust on Windows requires the Visual Studio C++ Build Tools."
        Write-Host "Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/"
        Write-Host ""
        Write-Host "Select 'Desktop development with C++' workload during installation."
        Write-Host ""
        $response = Read-Host "Continue anyway? [y/N]"
        if ($response -ne "y" -and $response -ne "Y") {
            exit 1
        }
    }

    # Build
    Build-Project

    if ($BuildOnly) {
        Write-Info "Build complete. Run with: .\target\release\hollowdeep.exe"
        return
    }

    # Install
    Write-Host ""
    $response = Read-Host "Install to $InstallDir? [Y/n]"
    if ($response -ne "n" -and $response -ne "N") {
        Install-ToSystem -DestDir $InstallDir

        # Desktop shortcut
        $response = Read-Host "Create desktop shortcut? [Y/n]"
        if ($response -ne "n" -and $response -ne "N") {
            New-DesktopShortcut -TargetPath (Join-Path $InstallDir "bin\hollowdeep.exe")
        }
    } else {
        Write-Info "Skipping system installation"
        Write-Info "You can run the game with: .\target\release\hollowdeep.exe"
    }

    Write-Host ""
    Write-Host "======================================" -ForegroundColor Cyan
    Write-Host "  Installation Complete!" -ForegroundColor Cyan
    Write-Host "======================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Run 'hollowdeep' to start the game"
    Write-Host ""
    Write-Host "For the best experience, use Windows Terminal with a modern font."
    Write-Host "Recommended terminals: Windows Terminal, WezTerm"
    Write-Host ""
}

Main
