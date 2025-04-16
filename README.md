# Photonos

Photonos is a wget-like tool that can render JavaScript-powered websites using a headless browser, allowing you to capture the fully rendered HTML content and take screenshots of modern web applications.

## Usage

Photonos works similar to wget, but with JavaScript rendering:

```bash
photonos https://example.com -o output.html --screenshot screenshot.png
```

## Installation

### Automatic Installation

You can install Photonos automatically with this command:

```bash
curl -sSL https://raw.githubusercontent.com/4383/photonos/main/bootstrap.sh | bash
```

The script will ask for your sudo password if necessary to install system dependencies. Do not run the command directly with sudo.

This command:
- Automatically detects your Linux distribution
- Installs all required system dependencies (asking for sudo if necessary)
- Installs Node.js if needed
- Installs Rust and Cargo if needed
- Installs Photonos from crates.io

After installation, you will be able to use Photonos from anywhere in the command line.

### Manual Installation

If you prefer to install Photonos manually, follow these steps for your specific Linux distribution:

#### Install dependencies

##### Debian / Ubuntu
```bash
# System dependencies
sudo apt-get update
sudo apt-get install -y libnss3 libnspr4 libatk1.0-0 libatk-bridge2.0-0 libatspi2.0-0 libxcomposite1 libxdamage1 libxfixes3 libxrandr2 libxkbcommon0 libasound2 curl

# Node.js (if not already installed)
sudo apt-get install -y nodejs npm
```

##### Fedora
```bash
# System dependencies
sudo dnf install -y atk cups-libs gtk3 libdrm libXcomposite libXdamage libXrandr libgbm libxshmfence alsa-lib pango cairo libX11-xcb xorg-x11-server-Xvfb nss libXcursor libXfixes libXi mesa-libEGL libXrender libXtst xdg-utils nss nspr at-spi2-atk at-spi2-core libxkbcommon mesa-libgbm curl

# Node.js (if not already installed)
sudo dnf install -y nodejs npm
```

##### Arch Linux
```bash
# System dependencies
sudo pacman -Sy --noconfirm nss nspr atk at-spi2-atk at-spi2-core libxcomposite libxdamage libxfixes libxrandr libxkbcommon alsa-lib curl

# Node.js (if not already installed)
sudo pacman -Sy --noconfirm nodejs npm
```

#### Install Rust and Cargo

If you don't have Rust installed already:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### Install Photonos

Once the dependencies are installed, you can install Photonos directly from crates.io:

```bash
cargo install photonos
```

> **Note:** Photonos includes all necessary Node.js dependencies and Playwright browser configuration in the package. No additional setup for Playwright is required after installation.

## Why so many dependencies?

While Rust programs are known for being standalone and having minimal dependencies, Photonos requires several system libraries because it integrates with Playwright, a Node.js-based browser automation tool. These dependencies are necessary for:

- **Browser Rendering**: Photonos uses Playwright's headless Chromium to fully render web pages with JavaScript, which requires many of the same libraries that Chrome itself needs.
- **System Integration**: The headless browser needs access to graphics libraries, fonts, and other system components even when running in headless mode.
- **Cross-platform compatibility**: The dependencies ensure that the browser functions correctly across different Linux distributions.

This hybrid approach (Rust + Node.js) combines the performance and reliability of Rust with the powerful web rendering capabilities of modern browsers through Playwright.

## Troubleshooting

If you can't run Photonos after installation, make sure that Cargo's bin directory is in your PATH:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

For persistent configuration, add this line to your `~/.bashrc`, `~/.zshrc`, or equivalent shell configuration file.

If you encounter rendering issues, make sure all system dependencies are correctly installed for your distribution.