#!/usr/bin/env bash
set -euo pipefail

# Hollowdeep Installation Script for Unix (Linux/macOS)

INSTALL_DIR="${INSTALL_DIR:-$HOME/.local}"
BIN_DIR="$INSTALL_DIR/bin"
DATA_DIR="$INSTALL_DIR/share/hollowdeep"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info() { echo -e "${GREEN}[INFO]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)  OS="linux";;
        Darwin*) OS="macos";;
        *)       error "Unsupported operating system: $(uname -s)";;
    esac
    info "Detected OS: $OS"
}

# Check if Rust is installed
check_rust() {
    if command -v rustc &> /dev/null; then
        RUST_VERSION=$(rustc --version | cut -d' ' -f2)
        info "Rust $RUST_VERSION is installed"
        return 0
    else
        return 1
    fi
}

# Install Rust via rustup
install_rust() {
    info "Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    info "Rust installed successfully"
}

# Check for required dependencies
check_dependencies() {
    local missing=()

    # Check for basic build tools
    if ! command -v cc &> /dev/null && ! command -v gcc &> /dev/null && ! command -v clang &> /dev/null; then
        missing+=("C compiler (gcc/clang)")
    fi

    # Linux-specific dependencies for audio (ALSA)
    if [[ "$OS" == "linux" ]]; then
        if ! pkg-config --exists alsa 2>/dev/null; then
            missing+=("alsa-lib (libasound2-dev on Debian/Ubuntu, alsa-lib-devel on Fedora)")
        fi
    fi

    if [[ ${#missing[@]} -gt 0 ]]; then
        warn "Missing dependencies:"
        for dep in "${missing[@]}"; do
            echo "  - $dep"
        done
        echo ""
        echo "Install them using your package manager:"
        echo "  Debian/Ubuntu: sudo apt install build-essential libasound2-dev"
        echo "  Fedora:        sudo dnf install gcc alsa-lib-devel"
        echo "  Arch:          sudo pacman -S base-devel alsa-lib"
        echo "  macOS:         xcode-select --install"
        echo ""
        read -p "Continue anyway? [y/N] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
}

# Build the project
build_project() {
    info "Building Hollowdeep (release mode)..."
    cargo build --release
    info "Build complete"
}

# Install to system
install_to_system() {
    info "Installing to $INSTALL_DIR..."

    # Create directories
    mkdir -p "$BIN_DIR"
    mkdir -p "$DATA_DIR"

    # Copy binary
    cp target/release/hollowdeep "$BIN_DIR/"
    chmod +x "$BIN_DIR/hollowdeep"

    # Copy assets
    if [[ -d "assets" ]]; then
        cp -r assets/* "$DATA_DIR/"
        info "Assets installed to $DATA_DIR"
    fi

    info "Binary installed to $BIN_DIR/hollowdeep"

    # Check if bin dir is in PATH
    if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
        warn "$BIN_DIR is not in your PATH"
        echo "Add the following to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
        echo "  export PATH=\"\$PATH:$BIN_DIR\""
    fi
}

# Main
main() {
    echo "======================================"
    echo "  Hollowdeep Installation Script"
    echo "======================================"
    echo ""

    detect_os

    # Check/install Rust
    if ! check_rust; then
        warn "Rust is not installed"
        read -p "Install Rust via rustup? [Y/n] " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Nn]$ ]]; then
            error "Rust is required to build Hollowdeep"
        fi
        install_rust
    fi

    check_dependencies
    build_project

    echo ""
    read -p "Install to $INSTALL_DIR? [Y/n] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Nn]$ ]]; then
        install_to_system
    else
        info "Skipping system installation"
        info "You can run the game with: ./target/release/hollowdeep"
    fi

    echo ""
    echo "======================================"
    echo "  Installation Complete!"
    echo "======================================"
    echo ""
    echo "Run 'hollowdeep' to start the game"
    echo ""
    echo "For the best experience, use a terminal with true color support:"
    echo "  - Ghostty, Kitty, WezTerm, or iTerm2 for sprite rendering"
    echo ""
}

main "$@"
