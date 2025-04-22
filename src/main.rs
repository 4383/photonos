use clap::Parser;
use std::fs::File;
use std::io::Write;

// Corrected imports for chromiumoxide
use chromiumoxide::{Browser, browser::BrowserConfig};
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use futures::StreamExt;
use tokio::runtime::Runtime;
use anyhow::{Result, Context, anyhow};
use std::path::Path;

// Include the built information generated at compile time
pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

// Utiliser la version Git comme version de l'application
const VERSION: &str = env!("GIT_VERSION");

/// Photonos - A wget-like tool that renders JavaScript with Chromium
#[derive(Parser)]
#[command(name = "photonos")]
#[command(version = VERSION)]
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
    
    /// Enable Chrome browser installation check
    #[arg(long)]
    check_browser: bool,
    
    /// Custom user-agent string
    #[arg(short = 'A', long, value_name = "STRING")]
    user_agent: Option<String>,
}

// Check if Chrome/Chromium is installed
fn check_chrome_installation() -> bool {
    println!("Checking for Chrome/Chromium installation...");
    println!("Note: Chrome/Chromium is now the only external dependency required");
    
    // Check if Chrome or Chromium is installed
    let chrome_paths = [
        "/usr/bin/google-chrome",
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser"
    ];
    
    for path in &chrome_paths {
        if std::path::Path::new(path).exists() {
            println!("✅ Found browser at: {}", path);
            return true;
        }
    }
    
    // Otherwise suggest installation
    eprintln!("❌ Chrome or Chromium not found. Please install one of them:");
    eprintln!("For Debian/Ubuntu:");
    eprintln!("    sudo apt-get install chromium-browser");
    eprintln!("\nFor Fedora:");
    eprintln!("    sudo dnf install chromium");
    eprintln!("\nFor Arch Linux:");
    eprintln!("    sudo pacman -S chromium");
    
    false
}

// New function to render the page with chromiumoxide
async fn render_page(url: &str, screenshot_path: Option<&str>, user_agent: Option<&str>) -> Result<String> {
    // Configure and launch the browser with appropriate configuration
    let browser_config = BrowserConfig::builder()
        // Additional arguments to improve stability
        .args(vec![
            "--no-sandbox", 
            "--disable-setuid-sandbox",
            "--disable-gpu", 
            "--disable-dev-shm-usage",
            "--disable-web-security",
            "--allow-running-insecure-content",
            "--disable-popup-blocking"
        ])
        .build()
        .map_err(|e: String| anyhow!(e))?;
        
    let (mut browser, mut handler) = Browser::launch(browser_config).await?;
    
    // The error message can be ignored as it doesn't affect functionality
    let browser_handle = tokio::task::spawn(async move {
        while let Some(event) = handler.next().await {
            if let Err(e) = event {
                // Ignore this specific error that is just an internal decoding problem
                if !e.to_string().contains("data did not match any variant of untagged enum Message") {
                    eprintln!("Browser error: {}", e);
                }
            }
        }
    });
    
    // Create a new page
    let page = browser.new_page("about:blank").await?;
    
    // Configure the window size
    let viewport_params = chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams::builder()
        .width(1280)
        .height(800)
        .device_scale_factor(1.0)
        .mobile(false)
        .build()
        .map_err(|e| anyhow!(e))?;
    
    page.execute(viewport_params).await?;
    
    // Increase the default timeout for requests
    let user_agent_value = user_agent.unwrap_or("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36");
    
    // Log the user agent being used
    if let Some(custom_ua) = user_agent {
        println!("Using custom User-Agent: {}", custom_ua);
    }
    
    let timeout_params = chromiumoxide::cdp::browser_protocol::network::SetUserAgentOverrideParams::builder()
        .user_agent(user_agent_value)
        .accept_language("en-US,en;q=0.9")
        .platform("Linux")
        .build()
        .map_err(|e| anyhow!(e))?;
    
    page.execute(timeout_params).await?;
    
    // Enable JavaScript explicitly
    let js_params = chromiumoxide::cdp::browser_protocol::emulation::SetScriptExecutionDisabledParams::builder()
        .value(false)
        .build()
        .map_err(|e| anyhow!(e))?;
    
    page.execute(js_params).await?;
    
    println!("Navigating to: {}", url);
    
    // Navigate to the specified URL with a longer timeout
    page.goto(url).await?;
    
    // Wait for the page to load with a longer delay (30 seconds)
    println!("Waiting for page to load completely...");
    
    // First wait for initial DOM loading
    page.evaluate("() => document.readyState").await?;
    
    // Then wait for all scripts to execute and the page to be stable
    // Wait 10 seconds before continuing
    page.evaluate(r#"
        () => new Promise((resolve) => {
            const checkComplete = () => {
                // Check if the page is fully loaded
                if (document.readyState === 'complete') {
                    // Wait a bit more for async scripts
                    setTimeout(() => {
                        // Check if there are any fetch or XMLHttpRequest requests in progress
                        setTimeout(resolve, 2000); // Wait 2 more seconds to be sure
                    }, 3000); // Wait 3 seconds after complete loading
                } else {
                    // Check again in 500ms
                    setTimeout(checkComplete, 500);
                }
            };
            
            checkComplete();
            
            // Maximum timeout of 30 seconds
            setTimeout(resolve, 30000);
        })
    "#).await?;
    
    println!("Page loaded, capturing content...");
    
    // Capture a screenshot if requested
    if let Some(screenshot) = screenshot_path {
        let format = match Path::new(screenshot).extension().and_then(|ext| ext.to_str()) {
            Some("jpg") | Some("jpeg") => CaptureScreenshotFormat::Jpeg,
            _ => CaptureScreenshotFormat::Png,
        };
        
        println!("Taking screenshot: {}", screenshot);
        
        let screenshot_bytes = page.screenshot(
            chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotParams::builder()
                .format(format)
                // Try to add the full_page parameter correctly
                .capture_beyond_viewport(true)
                .build()
        ).await?;
        
        // Write bytes to a file
        std::fs::write(screenshot, &screenshot_bytes)?;
    }
    
    // Get the HTML content of the page
    let html = page.content().await?;
    
    // Close the browser
    browser.close().await?;
    browser_handle.await?;
    
    Ok(html)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Check Chrome installation if requested
    if cli.check_browser && !check_chrome_installation() {
        std::process::exit(1);
    }

    // Create and run the Tokio runtime
    let rt = Runtime::new().context("Failed to create Tokio runtime")?;
    
    // Run the page rendering in the runtime
    let html = rt.block_on(async {
        let screenshot_path = cli.screenshot.as_deref();
        render_page(&cli.url, screenshot_path, cli.user_agent.as_deref()).await
    })?;

    // Write the HTML to a file
    let mut file = File::create(&cli.output).context("Failed to create output file")?;
    file.write_all(html.as_bytes())
        .context("Failed to write HTML to file")?;

    println!("✅ HTML saved to: {}", cli.output);

    if let Some(screenshot) = &cli.screenshot {
        println!("📸 Screenshot saved to: {}", screenshot);
    }
    
    Ok(())
}
