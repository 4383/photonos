#!/bin/bash

# Colors for messages
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}=================================================${NC}"
echo -e "${BLUE}      Automatic Installation of Photonos      ${NC}"
echo -e "${BLUE}=================================================${NC}"

# Basic dependencies for bootstrap
check_command() {
    command -v $1 >/dev/null 2>&1 || { 
        echo -e "${RED}❌ Command $1 is not installed. Installation required.${NC}"
        return 1
    }
    return 0
}

# Check if the user is root
check_root() {
    if [ "$(id -u)" != "0" ]; then
        return 1
    fi
    return 0
}

# Check if sudo is available
check_sudo() {
    command -v sudo >/dev/null 2>&1 || {
        echo -e "${RED}❌ sudo is not available and you are not root.${NC}"
        echo -e "${RED}   Please run this script as root or install sudo.${NC}"
        return 1
    }
    return 0
}

# Function to execute a command with sudo if necessary
run_with_sudo() {
    if check_root; then
        "$@"
    else
        if ! check_sudo; then
            echo -e "${RED}❌ Unable to obtain administrative privileges. Installation cancelled.${NC}"
            exit 1
        fi
        echo -e "${YELLOW}Requesting privilege elevation to install system dependencies...${NC}"
        sudo "$@"
    fi
}

# Detect Linux distribution
detect_distro() {
    if [ -f /etc/fedora-release ]; then
        echo "fedora"
    elif [ -f /etc/debian_version ]; then
        echo "debian"
    elif [ -f /etc/lsb-release ] && grep -q "Ubuntu" /etc/lsb-release; then
        echo "ubuntu"
    elif [ -f /etc/arch-release ]; then
        echo "arch"
    else
        echo "unknown"
    fi
}

# Install system dependencies based on distribution
install_dependencies() {
    local distro=$(detect_distro)
    
    echo -e "${BLUE}Detected distribution: $distro${NC}"
    echo -e "${YELLOW}Administrative privileges will be needed to install system dependencies.${NC}"
    
    case $distro in
        fedora)
            echo -e "${BLUE}Installing dependencies for Fedora...${NC}"
            run_with_sudo dnf install -y atk cups-libs gtk3 libdrm libXcomposite libXdamage libXrandr libgbm libxshmfence alsa-lib pango cairo libX11-xcb xorg-x11-server-Xvfb nss libXcursor libXfixes libXi mesa-libEGL libXrender libXtst xdg-utils nss nspr atk at-spi2-atk at-spi2-core libxkbcommon mesa-libgbm curl nodejs npm
            ;;
        debian|ubuntu)
            echo -e "${BLUE}Installing dependencies for Debian/Ubuntu...${NC}"
            run_with_sudo apt-get update
            run_with_sudo apt-get install -y libnss3 libnspr4 libatk1.0-0 libatk-bridge2.0-0 libatspi2.0-0 libxcomposite1 libxdamage1 libxfixes3 libxrandr2 libxkbcommon0 libasound2 curl nodejs npm
            ;;
        arch)
            echo -e "${BLUE}Installing dependencies for Arch Linux...${NC}"
            run_with_sudo pacman -Sy --noconfirm nss nspr atk at-spi2-atk at-spi2-core libxcomposite libxdamage libxfixes libxrandr libxkbcommon alsa-lib curl nodejs npm
            ;;
        *)
            echo -e "${YELLOW}Unrecognized distribution. Manual installation of dependencies required.${NC}"
            echo -e "${YELLOW}For Debian/Ubuntu:${NC}"
            echo "sudo apt-get install libnss3 libnspr4 libatk1.0-0 libatk-bridge2.0-0 libatspi2.0-0 libxcomposite1 libxdamage1 libxfixes3 libxrandr2 libxkbcommon0 libasound2 curl nodejs npm"
            echo -e "${YELLOW}For Fedora:${NC}"
            echo "sudo dnf install -y atk cups-libs gtk3 libdrm libXcomposite libXdamage libXrandr libgbm libxshmfence alsa-lib pango cairo libX11-xcb xorg-x11-server-Xvfb nss libXcursor libXfixes libXi mesa-libEGL libXrender libXtst xdg-utils nss nspr atk at-spi2-atk at-spi2-core libxkbcommon mesa-libgbm curl nodejs npm"
            ;;
    esac
}

# Installation of Rust and Cargo
install_rust_and_cargo() {
    echo -e "${BLUE}Checking for Rust and Cargo...${NC}"
    
    if ! check_command rustc || ! check_command cargo; then
        echo -e "${YELLOW}Rust/Cargo is not installed. Installing...${NC}"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        # Source the file to use Rust immediately
        if [ -f "$HOME/.cargo/env" ]; then
            source "$HOME/.cargo/env"
        else
            echo -e "${YELLOW}⚠️ Rust installation completed, but you'll need to restart your terminal or run:${NC}"
            echo -e "${BLUE}   source \$HOME/.cargo/env${NC}"
            echo -e "${YELLOW}   before continuing.${NC}"
            exit 1
        fi
    fi
}

# Install and build scripts/render-bin
build_render_binary() {
    echo -e "${BLUE}Building render-bin binary...${NC}"
    
    # Check if npm is installed
    if ! check_command npm; then
        echo -e "${RED}❌ npm is not installed. Please install Node.js and npm.${NC}"
        return 1
    }
    
    # Navigate to scripts directory
    echo -e "${BLUE}Installing Node.js dependencies...${NC}"
    cd scripts || { echo -e "${RED}❌ scripts directory not found!${NC}"; return 1; }
    
    # Install dependencies like the user did locally
    echo -e "${BLUE}Installing npm dependencies...${NC}"
    npm install || { echo -e "${RED}❌ Failed to install Node.js dependencies.${NC}"; return 1; }
    
    # Install Playwright and its dependencies
    echo -e "${BLUE}Installing Playwright...${NC}"
    npx playwright install || { echo -e "${RED}❌ Failed to install Playwright.${NC}"; return 1; }
    
    echo -e "${BLUE}Installing Playwright dependencies...${NC}"
    npx playwright install-deps || { echo -e "${RED}❌ Failed to install Playwright dependencies.${NC}"; return 1; }
    
    echo -e "${BLUE}Installing Chromium...${NC}"
    npx playwright install chromium || { echo -e "${RED}❌ Failed to install Chromium.${NC}"; return 1; }
    
    # Install pkg globally if not already installed
    echo -e "${BLUE}Checking for pkg...${NC}"
    if ! npm list -g pkg > /dev/null 2>&1; then
        echo -e "${BLUE}Installing pkg globally...${NC}"
        npm install -g pkg || { echo -e "${RED}❌ Failed to install pkg globally.${NC}"; return 1; }
    fi
    
    # Build the binary using pkg
    echo -e "${BLUE}Building render-bin using pkg...${NC}"
    pkg . --targets node18-linux-x64 --output render-bin || { echo -e "${RED}❌ Failed to build render-bin.${NC}"; return 1; }
    
    # Check if render-bin was created successfully
    if [ -f "render-bin" ]; then
        echo -e "${GREEN}✅ render-bin successfully built!${NC}"
        # Make the binary executable
        chmod +x render-bin || { echo -e "${YELLOW}⚠️ Failed to make render-bin executable.${NC}"; }
    else
        echo -e "${RED}❌ render-bin was not created.${NC}"
        return 1
    fi
    
    # Return to original directory
    cd - > /dev/null
    return 0
}

# Installing Photonos via cargo
install_photonos() {
    echo -e "${BLUE}Installing Photonos from crates.io...${NC}"
    if ! check_command cargo; then
        echo -e "${RED}❌ Cargo is not available. Please restart your terminal after installing Rust.${NC}"
        exit 1
    fi
    
    cargo install photonos
    
    if [ $? -ne 0 ]; then
        echo -e "${RED}❌ Photonos installation failed.${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✅ Photonos has been successfully installed!${NC}"
    echo -e "${GREEN}Executable available at: $HOME/.cargo/bin/photonos${NC}"
    
    # Check if PATH includes the cargo directory
    if ! echo "$PATH" | grep -q "$HOME/.cargo/bin"; then
        echo -e "${YELLOW}NOTE: Make sure that $HOME/.cargo/bin is in your PATH${NC}"
        echo -e "${YELLOW}You can add the following line to your .bashrc or .profile:${NC}"
        echo -e "${BLUE}export PATH=\"\$HOME/.cargo/bin:\$PATH\"${NC}"
    fi
}

# Main function
main() {
    # Install system dependencies
    install_dependencies
    
    # Install Rust and Cargo
    install_rust_and_cargo
    
    # Build the render-bin binary
    build_render_binary
    
    # Install Photonos from crates.io
    install_photonos
    
    echo -e "${GREEN}==================================================${NC}"
    echo -e "${GREEN}✅ Photonos installation completed successfully!${NC}"
    echo -e "${GREEN}You can run Photonos with:${NC}"
    echo -e "${BLUE}photonos URL -o output.html --screenshot screenshot.png${NC}"
    echo -e "${GREEN}==================================================${NC}"
}

# Execute the main function
main "$@"