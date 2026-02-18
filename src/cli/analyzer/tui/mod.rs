/// Terminal User Interface for interactive pattern analysis
///
/// This module provides the TUI for browsing proposed patterns,
/// previewing matching log entries, and saving rules.
mod app;
mod pattern_list;
mod preview;
mod save_dialog;

// Re-export the main entry point
pub use app::run_analyzer;
