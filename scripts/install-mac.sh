#!/bin/bash

# ReadFlow Installation Script for macOS
# Supports: Intel and Apple Silicon Macs

set -e

VERSION="0.1.0"
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="$HOME/.config/readflow"
DATA_DIR="$HOME/.local/share/readflow"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}Installing ReadFlow v${VERSION} for macOS...${NC}"

# Detect architecture
ARCH=$(uname -m)
if [ "$ARCH" = "arm64" ]; then
    echo "Detected Apple Silicon Mac"
    HOMEBREW_PREFIX="/opt/homebrew"
else
    echo "Detected Intel Mac"
    HOMEBREW_PREFIX="/usr/local"
fi

# Check if Homebrew is installed
check_homebrew() {
    if ! command -v brew &> /dev/null; then
        echo -e "${YELLOW}Homebrew not found. Installing Homebrew...${NC}"
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    fi
}

# Install dependencies
install_deps() {
    echo "Installing dependencies..."
    brew install openssl@3
    
    # Check if Rust is installed
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
    
    # Build with SSL support
    OPENSSL_DIR="$HOMEBREW_PREFIX/opt/openssl@3" cargo build --release
    
    # Install binary
    sudo cp target/release/readflow $INSTALL_DIR/
    sudo chmod +x $INSTALL_DIR/readflow
    
    echo -e "${GREEN}Installed to $INSTALL_DIR/readflow${NC}"
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

# Create .app bundle for macOS
create_app_bundle() {
    echo "Creating macOS app bundle..."
    
    APP_DIR="$HOME/Applications/ReadFlow.app"
    mkdir -p "$APP_DIR/Contents/MacOS"
    mkdir -p "$APP_DIR/Contents/Resources"
    
    # Copy binary
    cp target/release/readflow "$APP_DIR/Contents/MacOS/ReadFlow"
    chmod +x "$APP_DIR/Contents/MacOS/ReadFlow"
    
    # Create Info.plist
    cat > "$APP_DIR/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>ReadFlow</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>CFBundleIdentifier</key>
    <string>com.readflow.app</string>
    <key>CFBundleExecutable</key>
    <string>ReadFlow</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleIconFile</key>
    <string></string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.utilities</string>
    <key>LSUIElement</key>
    <false/>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF
    
    echo -e "${GREEN}App bundle created at $APP_DIR${NC}"
}

# Main installation
main() {
    check_homebrew
    install_deps
    build_from_source
    setup_config
    create_app_bundle
    
    echo -e "${GREEN}Installation complete!${NC}"
    echo "Run 'readflow' or open ReadFlow.app to start"
}

main "$@"
