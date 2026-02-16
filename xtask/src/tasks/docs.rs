use anyhow::Result;
use std::process::Command;

pub fn generate_docs(open: bool) -> Result<()> {
    println!("📚 Generating documentation...");

    // Generate API documentation
    generate_api_docs(open)?;

    // Trigger CLI documentation generation (via build.rs)
    generate_cli_docs()?;

    println!("✅ All documentation generated successfully");
    Ok(())
}

fn generate_api_docs(open: bool) -> Result<()> {
    println!("📖 Generating API documentation...");

    let mut cmd = Command::new("cargo");
    cmd.arg("doc")
        .arg("--no-deps")
        .arg("--all-features")
        .arg("--document-private-items");

    if open {
        cmd.arg("--open");
    }

    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("Failed to generate API documentation");
    }

    println!("✅ API documentation generated");
    Ok(())
}

fn generate_cli_docs() -> Result<()> {
    println!("📝 Generating CLI documentation (markdown + man pages)...");

    // Build the CLI binary which triggers build.rs
    // build.rs automatically generates docs/cli-reference.md and docs/man/logcheck-filter.1
    let status = Command::new("cargo")
        .arg("build")
        .arg("--bin")
        .arg("logcheck-filter")
        .status()?;

    if !status.success() {
        anyhow::bail!("Failed to build CLI (needed for documentation generation)");
    }

    println!("✅ CLI documentation generated");
    println!("   - docs/cli-reference.md");
    println!("   - docs/man/logcheck-filter.1");
    Ok(())
}
