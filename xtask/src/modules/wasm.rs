use anyhow::{Context, Result};
use std::process::Command;
use std::path::PathBuf;

pub async fn build_wasm(_verbose: bool) -> Result<()> {
    println!("Building Spooky Maze WASM...");
    println!();

    // Find spooky-maze-wasm directory by checking common locations
    let wasm_dir = find_wasm_dir()?;

    if !wasm_dir.exists() {
        anyhow::bail!("spooky-maze-wasm directory not found at {:?}", wasm_dir);
    }

    // Check if wasm-pack is installed
    if !command_exists("wasm-pack") {
        println!("Installing wasm-pack...");
        let output = Command::new("sh")
            .arg("-c")
            .arg("curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh")
            .output()
            .context("Failed to install wasm-pack")?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to install wasm-pack: {}", error);
        }
        println!("wasm-pack installed");
    }

    // Build WASM with wasm-pack
    println!("Building WASM package...");
    let output = Command::new("wasm-pack")
        .arg("build")
        .arg("--target")
        .arg("web")
        .arg("--out-dir")
        .arg("pkg")
        .arg("--dev")
        .current_dir(&wasm_dir)
        .output()
        .context("Failed to build WASM")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("{}", stderr);
        anyhow::bail!("WASM build failed");
    }

    println!("WASM build complete!");
    println!("Output: {:}/pkg/", wasm_dir.display());
    println!();
    println!("To serve the files, run:");
    println!("  cargo xtask serve-wasm");

    Ok(())
}

pub async fn serve_wasm(_verbose: bool, port: Option<u16>) -> Result<()> {
    let port = port.unwrap_or(8000);

    // First build WASM
    build_wasm(_verbose).await?;

    println!();
    println!("Starting HTTP server on http://localhost:{}", port);
    println!("Press Ctrl+C to stop");
    println!();

    let wasm_dir = find_wasm_dir()?;

    // Try miniserve first (pure Rust, fast)
    if command_exists("miniserve") {
        println!("Using miniserve (pure Rust HTTP server)...");

        let mut child = Command::new("miniserve")
            .arg("--port")
            .arg(port.to_string())
            .arg("--index")
            .arg("index.html")
            .arg(&wasm_dir)
            .spawn()
            .context("Failed to start miniserve")?;

        let status = child.wait().context("Failed to wait for miniserve")?;
        if !status.success() {
            anyhow::bail!("Server exited with error");
        }
    } else {
        // Install and use miniserve
        println!("miniserve not found. Installing...");
        println!("   (This is a one-time installation)");

        let install_output = Command::new("cargo")
            .arg("install")
            .arg("miniserve")
            .output()
            .context("Failed to install miniserve")?;

        if !install_output.status.success() {
            let error = String::from_utf8_lossy(&install_output.stderr);
            anyhow::bail!("Failed to install miniserve: {}", error);
        }

        println!("miniserve installed. Starting server...");

        let mut child = Command::new("miniserve")
            .arg("--port")
            .arg(port.to_string())
            .arg("--index")
            .arg("index.html")
            .arg(&wasm_dir)
            .spawn()
            .context("Failed to start miniserve")?;

        let status = child.wait().context("Failed to wait for miniserve")?;
        if !status.success() {
            anyhow::bail!("Server exited with error");
        }
    }

    Ok(())
}

fn find_wasm_dir() -> Result<PathBuf> {
    // Try current directory first
    let current = PathBuf::from(".");
    let wasm_dir = current.join("spooky-maze-wasm");
    if wasm_dir.exists() {
        return Ok(wasm_dir);
    }

    // Try parent directory
    if let Some(parent) = current.parent() {
        let wasm_dir = parent.join("spooky-maze-wasm");
        if wasm_dir.exists() {
            return Ok(wasm_dir);
        }
    }

    // Try CARGO_MANIFEST_DIR approach
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let manifest_path = PathBuf::from(&manifest_dir);
        if let Some(parent) = manifest_path.parent() {
            let wasm_dir = parent.join("spooky-maze-wasm");
            if wasm_dir.exists() {
                return Ok(wasm_dir);
            }
        }

        // Try two levels up (xtask -> workspace -> spooky-maze-wasm)
        let manifest_path = PathBuf::from(&manifest_dir);
        if let Some(grandparent) = manifest_path.parent().and_then(|p| p.parent()) {
            let wasm_dir = grandparent.join("spooky-maze-wasm");
            if wasm_dir.exists() {
                return Ok(wasm_dir);
            }
        }
    }

    anyhow::bail!("Could not find spooky-maze-wasm directory")
}

fn command_exists(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
