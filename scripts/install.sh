#!/bin/bash

# ReadFlow Universal One-Click Installer
# Works on Linux, macOS, and Windows (via WSL/Git Bash)
# Usage: curl -sL https://raw.githubusercontent.com/readflow/readflow/main/install.sh | bash

set -e

VERSION="0.1.0"
REPO_URL="https://github.com/readflow/readflow"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}"
echo "╔═══════════════════════════════════════════════════════════╗"
echo "║                                                           ║"
echo "║   ███████╗ ██████╗ ██████╗  ██████╗  █████╗ ██████╗     ║"
echo "║   ██╔════╝██╔═══██╗██╔══██╗██╔════╝ ██╔══██╗██╔══██╗    ║"
echo "║   ███████╗██║   ██║██████╔╝██║  ███╗███████║██████╔╝    ║"
echo "║   ╚════██║██║   ██║██╔══██╗██║   ██║██╔══██║██╔══██╗    ║"
echo "║   ███████║╚██████╔╝██║  ██║╚██████╔╝██║  ██║██║  ██║    ║"
echo "║   ╚══════╝ ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝    ║"
echo "║                      v${VERSION}                              ║"
echo "║                                                           ║"
echo "║          Universal One-Click Installer                    ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)
            if [ -f /etc/os-release ]; then
                . /etc/os-release
                echo "linux-$ID"
            else
                echo "linux-unknown"
            fi
            ;;
        Darwin*)
            echo "macos"
            ;;
        CYGWIN*|MINGW*|MSYS*)
            echo "windows"
            ;;
        *)
            echo "unknown"
            ;;
    esac
}

OS=$(detect_os)
echo -e "${GREEN}Detected OS: $OS${NC}"

# Check prerequisites
check_prerequisites() {
    echo -e "${YELLOW}Checking prerequisites...${NC}"
    
    if ! command -v curl &> /dev/null; then
        echo "curl is required but not installed"
        exit 1
    fi
    
    if ! command -v git &> /dev/null; then
        echo "git is required but not installed"
        exit 1
    fi
}

# Install system dependencies
install_deps() {
    case $OS in
        linux-debian|linux-ubuntu|linux-linuxmint)
            echo "Installing dependencies for Debian/Ubuntu..."
            sudo apt-get update
            sudo apt-get install -y build-essential pkg-config libssl-dev
            ;;
        linux-arch|linux-manjaro)
            echo "Installing dependencies for Arch Linux..."
            sudo pacman -S --noconfirm base-devel openssl
            ;;
        linux-fedora)
            echo "Installing dependencies for Fedora..."
            sudo dnf install -y gcc gcc-c++ openssl-devel
            ;;
        macos)
            echo "Installing dependencies for macOS..."
            if command -v brew &> /dev/null; then
                brew install openssl@3
            fi
            ;;
    esac
}

# Install Rust
install_rust() {
    if ! command -v rustc &> /dev/null; then
        echo -e "${YELLOW}Installing Rust...${NC}"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    else
        echo -e "${GREEN}Rust is already installed${NC}"
    fi
}

# Build ReadFlow
build_readflow() {
    echo -e "${YELLOW}Building ReadFlow...${NC}"
    
    # Clone or use existing repo
    if [ -d "readflow" ]; then
        cd readflow
        git pull origin main 2>/dev/null || true
    else
        git clone "$REPO_URL"
        cd readflow
    fi
    
    cargo build --release
    
    INSTALL_DIR=""
    case $OS in
        linux-*)
            INSTALL_DIR="/usr/local/bin"
            sudo cp target/release/readflow $INSTALL_DIR/
            sudo chmod +x $INSTALL_DIR/readflow
            ;;
        macos)
            INSTALL_DIR="/usr/local/bin"
            sudo cp target/release/readflow $INSTALL_DIR/
            sudo chmod +x $INSTALL_DIR/readflow
            ;;
        windows)
            INSTALL_DIR="$LOCALAPPDATA/ReadFlow"
            mkdir -p "$INSTALL_DIR"
            cp target/release/readflow.exe "$INSTALL_DIR/"
            ;;
    esac
    
    echo -e "${GREEN}Installed to $INSTALL_DIR/readflow${NC}"
}

# Setup configuration
setup_config() {
    CONFIG_DIR=""
    case $OS in
        linux-*|macos)
            CONFIG_DIR="$HOME/.config/readflow"
            mkdir -p "$CONFIG_DIR"
            mkdir -p "$HOME/.local/share/readflow"
            ;;
        windows)
            CONFIG_DIR="$APPDATA/readflow"
            mkdir -p "$CONFIG_DIR"
            ;;
    esac
    
    if [ ! -f "$CONFIG_DIR/config.toml" ]; then
        cat > "$CONFIG_DIR/config.toml" << 'EOF'
# ReadFlow Configuration
theme = "dark"
default_url = ""
enable_cookies = true
EOF
    fi
    
    echo -e "${GREEN}Configuration created at $CONFIG_DIR${NC}"
}

# Main
main() {
    check_prerequisites
    install_deps
    install_rust
    build_readflow
    setup_config
    
    echo ""
    echo -e "${GREEN}╔═══════════════════════════════════════════════════════════╗"
    echo "║                                                           ║"
    echo "║   ✓ ReadFlow installed successfully!                      ║"
    echo "║                                                           ║"
    echo "║   Usage:                                                  ║"
    echo "║     readflow                              # Start       ║"
    echo "║     readflow --url https://example.com    # With URL    ║"
    echo "║     readflow --help                        # Help        ║"
    echo "║                                                           ║"
    echo "║   Keyboard shortcuts:                                        ║"
    echo "║     o    Open URL                                           ║"
    echo "║     h    Go back                                            ║"
    echo "║     l    Go forward                                         ║"
    echo "║     j/k  Scroll down/up                                     ║"
    echo "║     /    Search page                                        ║"
    echo "║     r    Reader mode                                        ║"
    echo "║     t    Toggle theme                                       ║"
    echo "║     b    Add bookmark                                        ║"
    echo "║     ?    Help                                               ║"
    echo "║     q    Quit                                               ║"
    echo "║                                                           ║"
    echo "╚═══════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

main "$@"
