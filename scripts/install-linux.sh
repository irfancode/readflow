#!/bin/bash

# ReadFlow Installation Script for Linux
# Supports: Debian/Ubuntu, Arch Linux, Fedora, openSUSE

set -e

VERSION="0.1.0"
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/readflow"
DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/readflow"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}Installing ReadFlow v${VERSION} for Linux...${NC}"

# Detect package manager
detect_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        echo $ID
    elif [ -f /etc/arch-release ]; then
        echo "arch"
    elif [ -f /etc/debian_version ]; then
        echo "debian"
    elif [ -f /etc/fedora-release ]; then
        echo "fedora"
    else
        echo "unknown"
    fi
}

DISTRO=$(detect_distro)

# Check if Rust is installed
check_rust() {
    if ! command -v rustc &> /dev/null; then
        echo -e "${YELLOW}Rust not found. Installing Rust...${NC}"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi
}

# Build from source
build_from_source() {
    echo -e "${YELLOW}Building ReadFlow from source...${NC}"
    
    # Clone or use existing repo
    if [ -d "readflow" ]; then
        cd readflow
    else
        echo "Please ensure you have the readflow source code in the current directory"
        exit 1
    fi
    
    cargo build --release
    
    # Install binary
    sudo cp target/release/readflow $INSTALL_DIR/
    sudo chmod +x $INSTALL_DIR/readflow
    
    echo -e "${GREEN}Installed to $INSTALL_DIR/readflow${NC}"
}

# Install via package manager (if available)
install_apt() {
    echo "Installing dependencies..."
    sudo apt-get update
    sudo apt-get install -y build-essential pkg-config libssl-dev
    
    build_from_source
}

install_arch() {
    echo "Installing dependencies..."
    sudo pacman -S --noconfirm base-devel openssl
    
    build_from_source
}

install_dnf() {
    echo "Installing dependencies..."
    sudo dnf install -y gcc gcc-c++ openssl-devel
    
    build_from_source
}

# Create config directory
setup_config() {
    mkdir -p "$CONFIG_DIR"
    mkdir -p "$DATA_DIR"
    
    # Create default config if not exists
    if [ ! -f "$CONFIG_DIR/config.toml" ]; then
        cat > "$CONFIG_DIR/config.toml" << EOF
# ReadFlow Configuration
theme = "dark"
default_url = ""
enable_cookies = true
EOF
    fi
    
    echo -e "${GREEN}Configuration created at $CONFIG_DIR${NC}"
}

# Main installation
main() {
    check_rust
    
    case $DISTRO in
        debbian|ubuntu|linuxmint)
            install_apt
            ;;
        arch|manjaro)
            install_arch
            ;;
        fedora|rhel|centos)
            install_dnf
            ;;
        *)
            echo "Unsupported distribution. Building from source..."
            build_from_source
            ;;
    esac
    
    setup_config
    
    # Create desktop entry
    if [ -d "$HOME/.local/share/applications" ]; then
        cat > "$HOME/.local/share/applications/readflow.desktop" << EOF
[Desktop Entry]
Name=ReadFlow
Comment=A modern TUI browser
Exec=$INSTALL_DIR/readflow
Icon=terminal
Terminal=true
Type=Application
Categories=Network;WebBrowser;Utility;
EOF
    fi
    
    echo -e "${GREEN}Installation complete!${NC}"
    echo "Run 'readflow' to start"
}

main "$@"
