use anyhow::Result;
use std::process::Command;

pub fn build_container(tag: &str) -> Result<()> {
    println!("🐳 Building Docker container with tag: {}", tag);

    let status = Command::new("docker")
        .arg("build")
        .arg("-t")
        .arg(format!("fluent-bit-logcheck:{}", tag))
        .arg(".")
        .status()?;

    if !status.success() {
        anyhow::bail!("Failed to build Docker container");
    }

    println!("✅ Docker container built successfully");
    Ok(())
}
