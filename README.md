# Photonos

Photonos is a wget-like tool that can render JavaScript-powered websites using a headless browser, allowing you to capture the fully rendered HTML content and take screenshots of modern web applications.

## Installation

Once the requirements are met (see [Requirements](#requirements) section below), you can install Photonos directly from crates.io:

```bash
cargo install photonos
```

## Usage

Photonos works similar to wget, but with JavaScript rendering:

```bash
photonos https://example.com -o output.html --screenshot screenshot.png
```

## Requirements

Photonos has two main dependencies:

### 1. Chrome/Chromium

Chrome or Chromium is required as it's the browser engine used for rendering web pages. Install it for your distribution:

#### Debian / Ubuntu
```bash
sudo apt-get update
sudo apt-get install -y chromium-browser
```

#### Fedora
```bash
sudo dnf install -y chromium
```

#### Arch Linux
```bash
sudo pacman -S chromium
```

### 2. Rust and Cargo

Rust and Cargo are needed to build and install Photonos:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

## Why does Photonos need Chromium?

Photonos uses chromiumoxide, a pure Rust library to control the Chrome/Chromium browser. This allows it to:

- Render modern web pages with JavaScript support
- Take screenshots exactly as they would appear in a browser
- Interact with dynamic content

The pure Rust approach leverages the power of modern browser rendering while maintaining the performance benefits of Rust.

## Troubleshooting

If you can't run Photonos after installation, make sure that Cargo's bin directory is in your PATH:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

For persistent configuration, add this line to your `~/.bashrc`, `~/.zshrc`, or equivalent shell configuration file.