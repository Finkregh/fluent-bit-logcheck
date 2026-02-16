use anyhow::Result;
use clap::{Parser, Subcommand};

mod tasks;

/// Development task runner for logcheck-fluent-bit-filter
#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Development task runner for logcheck-fluent-bit-filter")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build all targets (CLI, WASM filter, and shared library) for native platform
    BuildAll {
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },
    /// Build CLI binary for native platform
    BuildCli {
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },
    /// Build CLI for all supported platforms (requires cross-compilation setup)
    BuildAllCli {
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },
    /// Build WASM filter
    BuildWasm {
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },
    /// Build shared library plugin for native platform
    BuildPlugin {
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },
    /// Build plugin for all supported platforms (requires cross-compilation setup)
    BuildAllPlugin {
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },
    /// Install CLI to ~/.local/bin
    InstallCli,
    /// Run integration tests
    TestIntegration {
        /// Run with release builds
        #[arg(long)]
        release: bool,
    },
    /// Test WASM filter with JSON format (requires Docker)
    TestJson,
    /// Test WASM filter with MessagePack format (requires Docker)
    TestMsgpack,
    /// Generate documentation
    Docs {
        /// Open documentation in browser
        #[arg(long)]
        open: bool,
    },
    /// Prepare release
    Release {
        /// Version to release (e.g., 1.0.0)
        version: String,
    },
    /// Build Docker container
    Docker {
        /// Docker tag to use
        #[arg(long, default_value = "latest")]
        tag: String,
    },
    /// Run benchmarks
    Bench {
        /// Benchmark pattern to run
        pattern: Option<String>,
    },
    /// Run logcheck rules tests
    TestRules,
    /// Generate code coverage report using cargo-tarpaulin
    Coverage,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::BuildAll { release } => tasks::build::build_all(release),
        Commands::BuildCli { release } => tasks::build::build_cli(release),
        Commands::BuildAllCli { release } => tasks::build::build_all_cli(release),
        Commands::BuildWasm { release } => tasks::build::build_wasm(release),
        Commands::BuildPlugin { release } => tasks::build::build_plugin(release),
        Commands::BuildAllPlugin { release } => tasks::build::build_all_plugin(release),
        Commands::InstallCli => tasks::build::install_cli(),
        Commands::TestIntegration { release } => tasks::test::integration_tests(release),
        Commands::TestJson => tasks::test::test_json(),
        Commands::TestMsgpack => tasks::test::test_msgpack(),
        Commands::Docs { open } => tasks::docs::generate_docs(open),
        Commands::Release { version } => tasks::release::prepare_release(&version),
        Commands::Docker { tag } => tasks::docker::build_container(&tag),
        Commands::Bench { pattern } => tasks::bench::run_benchmarks(pattern.as_deref()),
        Commands::TestRules => tasks::test::test_rules(),
        Commands::Coverage => tasks::test::coverage(),
    }
}
