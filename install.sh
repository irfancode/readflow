#!/bin/bash

# ReadFlow Universal One-Click Installer/Uninstaller
# Works on Linux, macOS, and Windows (via WSL/Git Bash)
# Usage: curl -sL https://raw.githubusercontent.com/irfancode/readflow/main/install.sh | bash
# Uninstall: curl -sL https://raw.githubusercontent.com/irfancode/readflow/main/install.sh | bash -s -- --uninstall

set -e

VERSION="0.1.0"
REPO_URL="https://github.com/irfancode/readflow"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Check for uninstall flag
UNINSTALL=false
for arg in "$@"; do
    if [ "$arg" = "--uninstall" ] || [ "$arg" = "-u" ]; then
        UNINSTALL=true
    fi
done

echo -e "${BLUE}"
echo "    ___"
echo "   /   \_______"
echo "  |   /       \"
echo "  |   |       | |   ReadFlow v${VERSION}"
echo "   \_|       | |   Modern TUI Browser"
echo "      \_____/     One-Click Installer"
echo ""
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

# Uninstall ReadFlow
uninstall_readflow() {
    echo -e "${YELLOW}Uninstalling ReadFlow...${NC}"
    
    case $OS in
        linux-*)
            if [ -f /usr/local/bin/readflow ]; then
                sudo rm -f /usr/local/bin/readflow
                echo -e "${GREEN}Removed /usr/local/bin/readflow${NC}"
            elif [ -f ~/.local/bin/readflow ]; then
                rm -f ~/.local/bin/readflow
                echo -e "${GREEN}Removed ~/.local/bin/readflow${NC}"
            fi
            ;;
        macos)
            if [ -f /usr/local/bin/readflow ]; then
                sudo rm -f /usr/local/bin/readflow
                echo -e "${GREEN}Removed /usr/local/bin/readflow${NC}"
            elif [ -f ~/.local/bin/readflow ]; then
                rm -f ~/.local/bin/readflow
                echo -e "${GREEN}Removed ~/.local/bin/readflow${NC}"
            fi
            ;;
        windows)
            if [ -f "$LOCALAPPDATA/ReadFlow/readflow.exe" ]; then
                rm -f "$LOCALAPPDATA/ReadFlow/readflow.exe"
                echo -e "${GREEN}Removed $LOCALAPPDATA/ReadFlow/readflow.exe${NC}"
            fi
            ;;
    esac

    echo ""
    echo -e "${YELLOW}Do you want to remove all ReadFlow data (bookmarks, history, settings)?${NC}"
    echo -e "${YELLOW}This cannot be undone!${NC}"
    read -p "Type 'yes' to confirm: " confirm
    
    if [ "$confirm" = "yes" ]; then
        case $OS in
            linux-*|macos)
                rm -rf "$HOME/.config/readflow"
                rm -rf "$HOME/.local/share/readflow"
                rm -rf "$HOME/.cache/readflow"
                echo -e "${GREEN}Removed configuration and data${NC}"
                ;;
            windows)
                rm -rf "$APPDATA/readflow"
                rm -rf "$LOCALAPPDATA/ReadFlow"
                echo -e "${GREEN}Removed configuration and data${NC}"
                ;;
        esac
    else
        echo -e "${GREEN}Kept configuration and data${NC}"
    fi

    if [ -d "readflow" ]; then
        rm -rf readflow
        echo -e "${GREEN}Removed build directory${NC}"
    fi

    echo ""
    echo -e "${GREEN}╔═══════════════════════════════════════════════════════════╗"
    echo "║                                                           ║"
    echo "║   ✓ ReadFlow uninstalled successfully!                   ║"
    echo "║                                                           ║"
    echo "╚═══════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
    exit 0
}

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
    if [ "$UNINSTALL" = true ]; then
        uninstall_readflow
    fi
    
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
    echo "║   Uninstall:                                               ║"
    echo "║     curl -sL .../install.sh | bash -s -- --uninstall       ║"
    echo "║                                                           ║"
    echo "╚═══════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

main "$@"
