use anyhow::Result;
use std::process::Command;

pub fn prepare_release(version: &str) -> Result<()> {
    println!("🚀 Preparing release v{}...", version);

    // Validate version format (semantic versioning)
    if !is_valid_semver(version) {
        anyhow::bail!(
            "Invalid version format: {}. Expected format: X.Y.Z",
            version
        );
    }

    // Check git status
    println!("🔍 Checking git status...");
    let output = Command::new("git")
        .args(&["status", "--porcelain"])
        .output()?;

    if !output.stdout.is_empty() {
        println!("⚠️  Warning: Working directory has uncommitted changes");
        println!("Consider committing or stashing changes before release.");
    }

    // Run all tests
    println!("🧪 Running all tests...");
    let status = Command::new("cargo")
        .arg("test")
        .arg("--all-features")
        .status()?;
    if !status.success() {
        anyhow::bail!("Tests failed - cannot prepare release");
    }

    // Build all targets in release mode
    println!("🔨 Building all release artifacts...");
    crate::tasks::build::build_all(true)?;

    // Generate documentation
    println!("📚 Generating documentation...");
    crate::tasks::docs::generate_docs(false)?;

    // Build for all platforms (may fail for some, that's OK)
    println!("🌐 Attempting cross-platform builds...");
    let _ = crate::tasks::build::build_all_cli(true);

    println!("\n✅ Release preparation completed for v{}", version);
    println!("\n📝 Next steps:");
    println!("  1. Review the changes and ensure everything works correctly");
    println!("  2. Update version in Cargo.toml to {}", version);
    println!("  3. Update CHANGELOG.md with release notes");
    println!(
        "  4. Commit changes: git commit -am 'chore: release v{}'",
        version
    );
    println!(
        "  5. Create git tag: git tag -a v{} -m 'Release v{}'",
        version, version
    );
    println!("  6. Push changes: git push origin main --tags");
    println!("\n📦 Release artifacts can be found in:");
    println!("  - target/<target>/release/logcheck-filter");
    println!("  - target/wasm32-unknown-unknown/release/logcheck_fluent_bit_filter.wasm");

    Ok(())
}

fn is_valid_semver(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return false;
    }

    parts.iter().all(|part| part.parse::<u32>().is_ok())
}
