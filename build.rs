use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    // Sets the build script output directory in OUT_DIR
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("built.rs");
    
    // Forces recompilation if a Git tag changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/tags");
    
    // Try to retrieve the version from Git tags
    let git_version = get_git_tag_version().unwrap_or_else(|| {
        // If no tag is found, use the version from Cargo.toml
        env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.1.0".to_string())
    });
    
    // Inform Cargo of the detected version
    println!("cargo:rustc-env=GIT_VERSION={}", git_version);
    
    // Collect project version information
    let mut opts = built::Options::default();
    opts.set_ci(true);
    
    built::write_built_file_with_opts(&opts, &Path::new("."), &dest_path)
        .expect("Failed to acquire build-time information");
}

/// Retrieves the version from Git tags
/// Returns the most recent version (numerically) if multiple tags are present
fn get_git_tag_version() -> Option<String> {
    let output = Command::new("git")
        .args(["tag", "--sort=-v:refname"])
        .output()
        .ok()?;
    
    if !output.status.success() {
        return None;
    }
    
    let tags = String::from_utf8(output.stdout).ok()?;
    
    // Take the most recent tag (the first in the sorted list)
    let latest_tag = tags.lines().next()?;
    
    // Clean up the tag to remove the 'v' prefix if present
    let version = if latest_tag.starts_with('v') {
        latest_tag.trim_start_matches('v').to_string()
    } else {
        latest_tag.to_string()
    };
    
    // Force recompilation if this tag is detected
    println!("cargo:warning=Using Git tag version: {}", version);
    
    Some(version)
}