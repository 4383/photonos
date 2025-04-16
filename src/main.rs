use clap::Parser;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Photonos - a wget-like tool that renders JavaScript using Playwright
#[derive(Parser)]
#[command(name = "photonos")]
#[command(version = "0.1.0")]
#[command(about = "Renders JS-powered pages like a browser and saves the HTML", long_about = None)]
struct Cli {
    /// URL to fetch
    url: String,

    /// HTML output file
    #[arg(short, long, default_value = "output.html")]
    output: String,

    /// Optional screenshot output file
    #[arg(long)]
    screenshot: Option<String>,
    
    /// Enable dependency checks
    #[arg(long)]
    check_dependencies: bool,
    
    /// Enable Playwright browser installation check
    #[arg(long)]
    check_browser: bool,
}

fn check_dependencies() -> bool {
    println!("Checking for required dependencies...");
    
    // Detect the Linux distribution
    let is_fedora = std::fs::metadata("/etc/fedora-release").is_ok();
    let is_debian = std::fs::metadata("/etc/debian_version").is_ok();
    
    if is_fedora {
        return check_fedora_dependencies();
    } else if is_debian {
        return check_debian_dependencies();
    } else {
        // For other distributions, show a generic message
        eprintln!("Unsupported Linux distribution. Please manually install the required dependencies:");
        eprintln!("For Debian/Ubuntu:");
        eprintln!("    sudo apt-get install libnss3 libnspr4 libatk1.0-0 libatk-bridge2.0-0 libatspi2.0-0 libxcomposite1 libxdamage1 libxfixes3 libxrandr2 libxkbcommon0 libasound2");
        eprintln!("\nFor Fedora:");
        eprintln!("    sudo dnf install nss nspr atk at-spi2-atk at-spi2-core libXcomposite libXdamage libXfixes libXrandr libxkbcommon alsa-lib mesa-libgbm");
        
        return false;
    }
}

// Check if Playwright browsers are installed
fn check_playwright_browsers() -> bool {
    println!("Checking Playwright browser installation...");
    
    // Check if the node_modules directory exists
    let node_modules_path = PathBuf::from("scripts/node_modules");
    if !node_modules_path.exists() {
        eprintln!("❌ Playwright dependencies not found. Please run:");
        eprintln!("    cd scripts && npm install");
        return false;
    }
    
    // Path to the chromium browser that Playwright expects
    // This path can differ based on the system and Playwright version, so better check for missing browsers
    let playwright_command = Command::new("node")
        .current_dir("scripts")
        .arg("-e")
        .arg("const { chromium } = require('playwright-core'); chromium.executablePath();")
        .output();
    
    match playwright_command {
        Ok(_) => true,
        Err(_) => {
            eprintln!("❌ Playwright browsers not installed. Please run:");
            eprintln!("    cd scripts && npx playwright install chromium");
            false
        }
    }
}

fn check_debian_dependencies() -> bool {
    let dependencies = [
        "libnss3", "libnspr4", "libatk1.0-0", "libatk-bridge2.0-0",
        "libatspi2.0-0", "libxcomposite1", "libxdamage1", "libxfixes3",
        "libxrandr2", "libxkbcommon0", "libasound2"
    ];
    
    let mut missing_deps = Vec::new();
    
    for dep in &dependencies {
        let status = Command::new("dpkg")
            .args(["-s", dep])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
            
        match status {
            Ok(exit_status) => {
                if !exit_status.success() {
                    missing_deps.push(*dep);
                }
            },
            Err(_) => {
                missing_deps.push(*dep);
            }
        }
    }
    
    if !missing_deps.is_empty() {
        eprintln!("❌ Missing dependencies required for browser automation:");
        eprintln!("Please install them with the following command:");
        
        let deps_str = missing_deps.join(" ");
        eprintln!("    sudo apt-get install {}", deps_str);
        
        eprintln!("\nOr use Playwright to install all dependencies:");
        eprintln!("    sudo npx playwright install-deps");
        
        return false;
    }
    
    true
}

fn check_fedora_dependencies() -> bool {
    let dependencies = [
        "nss", "nspr", "atk", "at-spi2-atk", "at-spi2-core", 
        "libXcomposite", "libXdamage", "libXfixes", "libXrandr", 
        "libxkbcommon", "alsa-lib", "mesa-libgbm"
    ];
    
    let mut missing_deps = Vec::new();
    
    for dep in &dependencies {
        let status = Command::new("rpm")
            .args(["-q", dep])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
            
        match status {
            Ok(exit_status) => {
                if !exit_status.success() {
                    missing_deps.push(*dep);
                }
            },
            Err(_) => {
                missing_deps.push(*dep);
            }
        }
    }
    
    if !missing_deps.is_empty() {
        eprintln!("❌ Missing dependencies required for browser automation on Fedora:");
        eprintln!("Please install them with the following command:");
        
        let deps_str = missing_deps.join(" ");
        eprintln!("    sudo dnf install {}", deps_str);
        
        eprintln!("\nOr use Playwright to install all dependencies:");
        eprintln!("    sudo npx playwright install-deps");
        
        return false;
    }
    
    true
}

fn main() {
    let cli = Cli::parse();

    // Check for required dependencies only if explicitly requested
    if cli.check_dependencies && !check_dependencies() {
        std::process::exit(1);
    }
    
    // Check for Playwright browser installation only if explicitly requested
    if cli.check_browser && !check_playwright_browsers() {
        std::process::exit(1);
    }

    // Path to the packaged Playwright binary
    let render_bin = PathBuf::from("scripts/render-bin");

    // Command construction
    let mut command = Command::new(render_bin);
    command.arg(&cli.url);

    if let Some(screenshot_path) = &cli.screenshot {
        command.arg(screenshot_path);
    }

    let output = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("❌ Failed to execute JavaScript engine");

    if !output.status.success() {
        eprintln!("❌ JS rendering failed. Code: {}", output.status);
        
        // Print stderr output to help with debugging
        if !output.stderr.is_empty() {
            eprintln!("Error details:");
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            
            // Check if the error is about missing browsers and provide helpful instructions
            let error_str = String::from_utf8_lossy(&output.stderr);
            if error_str.contains("Executable doesn't exist") || error_str.contains("Please run the following command to download new browsers") {
                eprintln!("\n❓ It looks like Playwright browsers are not installed.");
                eprintln!("Run the following commands to install them:");
                eprintln!("    cd scripts");
                eprintln!("    npm install");
                eprintln!("    npx playwright install chromium");
                eprintln!("\nOr run photonos with --skip-browser-check to bypass this check.");
            }
        }
        
        std::process::exit(1);
    }

    let mut file = File::create(&cli.output).expect("❌ Unable to create output file");
    file.write_all(&output.stdout)
        .expect("❌ Unable to write HTML");

    println!("✅ HTML saved to: {}", cli.output);

    if let Some(screenshot) = &cli.screenshot {
        println!("📸 Screenshot saved to: {}", screenshot);
    }
}
