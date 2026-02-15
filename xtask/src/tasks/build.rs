use anyhow::Result;
use std::process::Command;

/// Platform targets supported for CLI builds
const CLI_TARGETS: &[&str] = &[
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
];

/// MUSL targets for static Linux builds
const MUSL_TARGETS: &[&str] = &["x86_64-unknown-linux-musl", "aarch64-unknown-linux-musl"];

/// Windows targets
const WINDOWS_TARGETS: &[&str] = &["x86_64-pc-windows-msvc"];

/// Detect native target triple
fn detect_native_target() -> String {
    // Use rustc to detect the host triple
    let output = Command::new("rustc")
        .arg("-vV")
        .output()
        .expect("Failed to run rustc");

    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines() {
        if line.starts_with("host: ") {
            return line.trim_start_matches("host: ").to_string();
        }
    }

    // Fallback to x86_64-unknown-linux-gnu
    "x86_64-unknown-linux-gnu".to_string()
}

pub fn build_all(release: bool) -> Result<()> {
    println!(
        "🔨 Building all targets{}...",
        if release { " (release)" } else { "" }
    );

    // Build CLI for native target
    build_cli(release)?;

    // Build WASM filter
    build_wasm(release)?;

    // Build plugin for native target
    build_plugin(release)?;

    println!("✅ All targets built successfully");
    Ok(())
}

pub fn build_cli(release: bool) -> Result<()> {
    let target = detect_native_target();
    println!(
        "🔨 Building CLI binary for {}{}...",
        target,
        if release { " (release)" } else { "" }
    );

    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--bin")
        .arg("logcheck-filter")
        .arg("--target")
        .arg(&target);

    if release {
        cmd.arg("--release");
    }

    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("Failed to build CLI binary for {}", target);
    }

    println!("✅ CLI binary built successfully for {}", target);
    Ok(())
}

/// Build CLI for all supported platforms (requires cross-compilation setup)
pub fn build_all_cli(release: bool) -> Result<()> {
    println!("🔨 Building CLI for all platforms...");

    let mut failed_targets = Vec::new();
    let mut all_targets = Vec::new();
    all_targets.extend_from_slice(CLI_TARGETS);
    all_targets.extend_from_slice(MUSL_TARGETS);
    all_targets.extend_from_slice(WINDOWS_TARGETS);

    for target in all_targets {
        println!("  Building for {}...", target);

        let mut cmd = Command::new("cargo");
        cmd.arg("build")
            .arg("--bin")
            .arg("logcheck-filter")
            .arg("--target")
            .arg(target);

        if release {
            cmd.arg("--release");
        }

        match cmd.status() {
            Ok(status) if status.success() => {
                println!("    ✅ {}", target);
            }
            _ => {
                println!("    ❌ Failed: {}", target);
                failed_targets.push(target);
            }
        }
    }

    if !failed_targets.is_empty() {
        println!("\n⚠️  Some targets failed to build:");
        for target in &failed_targets {
            println!("  - {}", target);
        }
        println!("\nNote: Cross-compilation may require additional toolchains.");
        println!("Install with: rustup target add <target>");
    } else {
        println!("\n✅ All CLI targets built successfully");
    }

    Ok(())
}

pub fn build_wasm(release: bool) -> Result<()> {
    println!(
        "🔨 Building WASM filter{}...",
        if release { " (release)" } else { "" }
    );

    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--lib")
        .arg("--target")
        .arg("wasm32-unknown-unknown");

    if release {
        cmd.arg("--release");
    }

    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("Failed to build WASM filter");
    }

    println!("✅ WASM filter built successfully");
    Ok(())
}

/// Build shared library plugin for native target
pub fn build_plugin(release: bool) -> Result<()> {
    let target = detect_native_target();
    println!(
        "🔨 Building shared library plugin for {}{}...",
        target,
        if release { " (release)" } else { "" }
    );

    let mut cmd = Command::new("cargo");
    cmd.arg("build").arg("--lib").arg("--target").arg(&target);

    if release {
        cmd.arg("--release");
    }

    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("Failed to build plugin for {}", target);
    }

    println!("✅ Plugin built successfully for {}", target);
    Ok(())
}

/// Build plugin for all supported platforms (requires cross-compilation setup)
pub fn build_all_plugin(release: bool) -> Result<()> {
    println!("🔨 Building plugin for all platforms...");

    let mut failed_targets = Vec::new();
    let mut all_targets = Vec::new();
    all_targets.extend_from_slice(CLI_TARGETS);
    all_targets.extend_from_slice(MUSL_TARGETS);

    for target in all_targets {
        println!("  Building plugin for {}...", target);

        let mut cmd = Command::new("cargo");
        cmd.arg("build").arg("--lib").arg("--target").arg(target);

        if release {
            cmd.arg("--release");
        }

        match cmd.status() {
            Ok(status) if status.success() => {
                println!("    ✅ {}", target);
            }
            _ => {
                println!("    ❌ Failed: {}", target);
                failed_targets.push(target);
            }
        }
    }

    if !failed_targets.is_empty() {
        println!("\n⚠️  Some targets failed to build:");
        for target in &failed_targets {
            println!("  - {}", target);
        }
        println!("\nNote: Cross-compilation may require additional toolchains.");
        println!("Install with: rustup target add <target>");
    } else {
        println!("\n✅ All plugin targets built successfully");
    }

    Ok(())
}

/// Install CLI to ~/.local/bin
pub fn install_cli() -> Result<()> {
    println!("📦 Installing CLI to ~/.local/bin...");

    // First build in release mode
    build_cli(true)?;

    let target = detect_native_target();
    let home = std::env::var("HOME")?;
    let install_dir = format!("{}/.local/bin", home);

    // Create install directory if it doesn't exist
    std::fs::create_dir_all(&install_dir)?;

    // Determine binary extension
    let binary_name = if cfg!(windows) {
        "logcheck-filter.exe"
    } else {
        "logcheck-filter"
    };

    // Copy binary
    let mode = "release";
    let source = format!("target/{}/{}/{}", target, mode, binary_name);
    let dest = format!("{}/{}", install_dir, binary_name);

    std::fs::copy(&source, &dest)?;

    println!("✅ Installed logcheck-filter to {}", install_dir);
    Ok(())
}
