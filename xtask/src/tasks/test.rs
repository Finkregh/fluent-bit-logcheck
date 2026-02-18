use anyhow::Result;
use std::process::Command;

const DOCKER_IMAGE: &str = "fluent/fluent-bit:latest";

pub fn integration_tests(release: bool) -> Result<()> {
    println!(
        "🧪 Running integration tests{}...",
        if release { " (release)" } else { "" }
    );

    let mut cmd = Command::new("cargo");
    cmd.arg("test");

    if release {
        cmd.arg("--release");
    }

    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("Integration tests failed");
    }

    println!("✅ Integration tests passed");
    Ok(())
}

/// Ensure Docker image is available
fn ensure_docker_image() -> Result<()> {
    println!("🐳 Checking Docker image: {}", DOCKER_IMAGE);

    // Check if image exists
    let check = Command::new("docker")
        .arg("images")
        .arg("-q")
        .arg(DOCKER_IMAGE)
        .output()?;

    if check.stdout.is_empty() {
        println!("📥 Pulling Docker image...");
        let status = Command::new("docker")
            .arg("pull")
            .arg(DOCKER_IMAGE)
            .status()?;

        if !status.success() {
            anyhow::bail!("Failed to pull Docker image");
        }
    }

    Ok(())
}

/// Test WASM filter with JSON format
pub fn test_json() -> Result<()> {
    println!("🧪 Testing WASM filter with JSON format...");

    // Ensure Docker image is available
    ensure_docker_image()?;

    // Build WASM first
    crate::tasks::build::build_wasm(true)?;

    // Get current directory for bind mount
    let pwd = std::env::current_dir()?;
    let wasm_path = pwd.join("target/wasm32-unknown-unknown/release");

    println!("🐳 Running Fluent-Bit with WASM filter...");
    let status = Command::new("docker")
        .arg("run")
        .arg("--rm")
        .arg("--mount")
        .arg(format!(
            "type=bind,source={},target=/build_out",
            wasm_path.display()
        ))
        .arg(DOCKER_IMAGE)
        .arg("/opt/fluent-bit/bin/fluent-bit")
        .arg("-i")
        .arg("dummy")
        .arg("-F")
        .arg("wasm")
        .arg("-p")
        .arg("event_format=json")
        .arg("-p")
        .arg("wasm_path=/build_out/logcheck_fluent_bit_filter.wasm")
        .arg("-p")
        .arg("function_name=hello_world__json")
        .arg("-m")
        .arg("*")
        .arg("-o")
        .arg("stdout")
        .arg("-m")
        .arg("*")
        .status()?;

    if !status.success() {
        anyhow::bail!("WASM filter test (JSON) failed");
    }

    println!("✅ WASM filter test (JSON) passed");
    Ok(())
}

/// Test WASM filter with MessagePack format
pub fn test_msgpack() -> Result<()> {
    println!("🧪 Testing WASM filter with MessagePack format...");

    // Ensure Docker image is available
    ensure_docker_image()?;

    // Build WASM first
    crate::tasks::build::build_wasm(true)?;

    // Get current directory for bind mount
    let pwd = std::env::current_dir()?;
    let wasm_path = pwd.join("target/wasm32-unknown-unknown/release");

    println!("🐳 Running Fluent-Bit with WASM filter...");
    let status = Command::new("docker")
        .arg("run")
        .arg("--rm")
        .arg("--mount")
        .arg(format!(
            "type=bind,source={},target=/build_out",
            wasm_path.display()
        ))
        .arg(DOCKER_IMAGE)
        .arg("/opt/fluent-bit/bin/fluent-bit")
        .arg("-i")
        .arg("dummy")
        .arg("-F")
        .arg("wasm")
        .arg("-p")
        .arg("event_format=msgpack")
        .arg("-p")
        .arg("wasm_path=/build_out/logcheck_fluent_bit_filter.wasm")
        .arg("-p")
        .arg("function_name=hello_world__msgpack")
        .arg("-m")
        .arg("*")
        .arg("-o")
        .arg("stdout")
        .arg("-m")
        .arg("*")
        .status()?;

    if !status.success() {
        anyhow::bail!("WASM filter test (MessagePack) failed");
    }

    println!("✅ WASM filter test (MessagePack) passed");
    Ok(())
}

/// Run tests with logcheck rules (integration testing with system logcheck-database)
pub fn test_rules() -> Result<()> {
    println!("🧪 Running logcheck rules tests...");

    let target = crate::tasks::build::detect_native_target();
    println!("🔎 Using target for tests: {}", target);
    let status = Command::new("cargo")
        .arg("test")
        .arg("--lib")
        .arg("--target")
        .arg(&target)
        .status()?;

    if !status.success() {
        anyhow::bail!("Logcheck rules tests failed");
    }

    println!("✅ Logcheck rules tests passed");
    Ok(())
}

/// Run code coverage with tarpaulin (Linux) or llvm-cov fallback (non-Linux)
pub fn coverage() -> Result<()> {
    println!("📊 Running code coverage analysis...");

    let target = crate::tasks::build::detect_native_target();

    if target.contains("linux") {
        println!("🔎 Using target for coverage: {}", target);
        let status = Command::new("cargo")
            .arg("tarpaulin")
            .arg("--target")
            .arg(&target)
            .arg("--workspace")
            .arg("--timeout")
            .arg("300")
            .arg("--out")
            .arg("xml")
            .arg("--out")
            .arg("lcov")
            .arg("--output-dir")
            .arg("target/coverage/")
            .arg("--exclude-files")
            .arg("target/*")
            .arg("--exclude-files")
            .arg("build.rs")
            .arg("--all-features")
            .status()?;

        if !status.success() {
            anyhow::bail!("Code coverage analysis failed");
        }

        println!("✅ Code coverage analysis completed");
        println!("   Reports available in target/coverage/");
        return Ok(());
    } else {
        println!(
            "ℹ️ Non-Linux host detected ({}). Attempting cargo-llvm-cov fallback...",
            target
        );
        // Try cargo-llvm-cov as a cross-platform fallback
        let has_llvm_cov = Command::new("cargo")
            .arg("llvm-cov")
            .arg("--version")
            .output()
            .is_ok();
        if has_llvm_cov {
            let _ = std::fs::create_dir_all("target/coverage/");
            let status = Command::new("cargo")
                .arg("llvm-cov")
                .arg("--workspace")
                .arg("--all-features")
                .arg("--lcov")
                .arg("--output-path")
                .arg("target/coverage/lcov.info")
                .status()?;
            if !status.success() {
                anyhow::bail!("cargo-llvm-cov coverage failed");
            }
            println!("✅ llvm-cov coverage completed (LCOV only)");
            println!("   LCOV report at target/coverage/lcov.info");
            println!("   Note: Cobertura XML not generated on non-Linux hosts by xtask.");
            return Ok(());
        } else {
            anyhow::bail!(
                "cargo-tarpaulin supports Linux only. Run coverage on a Linux host or install cargo-llvm-cov for a local fallback.\n- Linux: cargo xtask coverage\n- macOS/Windows: cargo install cargo-llvm-cov && cargo xtask coverage"
            );
        }
    }
}
