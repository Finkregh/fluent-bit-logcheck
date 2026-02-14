use anyhow::Result;
use std::process::Command;

pub fn run_benchmarks(pattern: Option<&str>) -> Result<()> {
    println!("🏃 Running benchmarks...");

    let mut cmd = Command::new("cargo");
    cmd.arg("bench");

    if let Some(p) = pattern {
        cmd.arg(p);
    }

    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("Benchmarks failed");
    }

    println!("✅ Benchmarks completed");
    Ok(())
}
