/// Main TUI application for pattern analysis
use crate::cli::analyzer::pattern_grouper::PatternGroup;
use crate::cli::analyzer::rule_writer::{RuleWriter, RuleWriterConfig};
use crate::cli::Result;
use anyhow::Context;

use super::save_dialog::SaveDialog;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io;

/// Application state
enum AppState {
    /// Browsing patterns
    Browsing,
    /// Showing save dialog
    Saving,
}

/// Main TUI application
pub struct App {
    /// Current state
    state: AppState,
    /// Pattern groups to display
    patterns: Vec<PatternGroup>,
    /// All log entries for preview
    log_entries: Vec<String>,
    /// Currently selected pattern index
    selected_pattern: usize,
    /// Preview scroll offset
    preview_scroll: usize,
    /// Save dialog state
    save_dialog: Option<SaveDialog>,
}

impl App {
    fn new(patterns: Vec<PatternGroup>, log_entries: &[String]) -> Self {
        Self {
            state: AppState::Browsing,
            patterns,
            log_entries: log_entries.to_vec(),
            selected_pattern: 0,
            preview_scroll: 0,
            save_dialog: None,
        }
    }

    fn next_pattern(&mut self) {
        if self.selected_pattern < self.patterns.len().saturating_sub(1) {
            self.selected_pattern += 1;
            self.preview_scroll = 0;
        }
    }

    fn previous_pattern(&mut self) {
        if self.selected_pattern > 0 {
            self.selected_pattern -= 1;
            self.preview_scroll = 0;
        }
    }

    fn scroll_preview_down(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_add(1);
    }

    fn scroll_preview_up(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_sub(1);
    }

    fn open_save_dialog(&mut self) {
        if self.selected_pattern < self.patterns.len() {
            let pattern = &self.patterns[self.selected_pattern];
            self.save_dialog = Some(SaveDialog::new(pattern.regex.clone()));
            self.state = AppState::Saving;
        }
    }

    fn cancel_save(&mut self) {
        self.save_dialog = None;
        self.state = AppState::Browsing;
    }

    fn get_matching_entries(&self) -> Vec<String> {
        if self.selected_pattern >= self.patterns.len() {
            return Vec::new();
        }

        let pattern = &self.patterns[self.selected_pattern];
        pattern
            .matching_indices
            .iter()
            .filter_map(|&idx| self.log_entries.get(idx).cloned())
            .collect()
    }
}

/// Run the interactive analyzer TUI
pub fn run_analyzer(patterns: Vec<PatternGroup>, log_entries: &[String]) -> Result<()> {
    // Setup terminal
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .context("Failed to setup terminal")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    // Create app
    let app = App::new(patterns, log_entries);
    let result = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode().context("Failed to disable raw mode")?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .context("Failed to restore terminal")?;
    terminal.show_cursor().context("Failed to show cursor")?;

    // Handle success message if returned
    match result {
        Ok(Some(success_msg)) => {
            eprintln!("{}", success_msg);
            Ok(())
        }
        Ok(None) => Ok(()),
        Err(e) => Err(e),
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<Option<String>> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match app.state {
                AppState::Browsing => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(None),
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(None)
                    }
                    KeyCode::Down | KeyCode::Char('j') => app.next_pattern(),
                    KeyCode::Up | KeyCode::Char('k') => app.previous_pattern(),
                    KeyCode::PageDown => app.scroll_preview_down(),
                    KeyCode::PageUp => app.scroll_preview_up(),
                    KeyCode::Enter => app.open_save_dialog(),
                    _ => {}
                },
                AppState::Saving => {
                    if let Some(ref mut save_dialog) = app.save_dialog {
                        match key.code {
                            KeyCode::Esc => {
                                if save_dialog.is_editing() {
                                    save_dialog.stop_edit();
                                } else {
                                    app.cancel_save();
                                }
                            }
                            KeyCode::Char('e') => {
                                if !save_dialog.is_editing() {
                                    save_dialog.start_edit();
                                }
                            }
                            KeyCode::Left => {
                                if save_dialog.is_editing() {
                                    save_dialog.cursor_left();
                                } else {
                                    save_dialog.category_left();
                                }
                            }
                            KeyCode::Right => {
                                if save_dialog.is_editing() {
                                    save_dialog.cursor_right();
                                } else {
                                    save_dialog.category_right();
                                }
                            }
                            KeyCode::Char(c) => {
                                if save_dialog.is_editing() {
                                    save_dialog.add_char(c);
                                }
                            }
                            KeyCode::Backspace => {
                                if save_dialog.is_editing() {
                                    save_dialog.backspace();
                                }
                            }
                            KeyCode::Delete => {
                                if save_dialog.is_editing() {
                                    save_dialog.delete();
                                }
                            }
                            KeyCode::Enter => {
                                if !save_dialog.is_editing() {
                                    // Save the pattern
                                    let pattern = save_dialog.pattern().to_string();
                                    let category = save_dialog.selected_category();
                                    let match_count = if app.selected_pattern < app.patterns.len() {
                                        app.patterns[app.selected_pattern].match_count
                                    } else {
                                        0
                                    };

                                    let writer = RuleWriter::with_default_config();
                                    match writer.save_rule(category, &pattern, match_count) {
                                        Ok(path) => {
                                            let success_msg = format!(
                                                "Rule saved successfully to: {}",
                                                path.display()
                                            );
                                            return Ok(Some(success_msg));
                                        }
                                        Err(_) => {
                                            // For now, just cancel the dialog on error
                                            // Later we can improve error display
                                            app.cancel_save();
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // Header
            Constraint::Percentage(40), // Pattern list
            Constraint::Percentage(60), // Preview
        ])
        .split(f.area());

    // Header
    let header_text = format!(
        "Logcheck Analyzer - {} unmatched entries, {} proposed patterns",
        app.log_entries.len(),
        app.patterns.len()
    );
    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(header, chunks[0]);

    // Pattern list
    let pattern_items: Vec<Line> = app
        .patterns
        .iter()
        .enumerate()
        .map(|(idx, pattern)| {
            let style = if idx == app.selected_pattern {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let prefix = if idx == app.selected_pattern {
                "> "
            } else {
                "  "
            };
            Line::from(Span::styled(
                format!("{}{}", prefix, pattern.display()),
                style,
            ))
        })
        .collect();

    let pattern_list = Paragraph::new(pattern_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Patterns (↑/↓ to navigate, Enter to save, q to quit)"),
    );
    f.render_widget(pattern_list, chunks[1]);

    // Preview
    let matching_entries = app.get_matching_entries();
    let preview_lines: Vec<Line> = matching_entries
        .iter()
        .skip(app.preview_scroll)
        .take(chunks[2].height as usize - 2)
        .map(|entry| Line::from(entry.as_str()))
        .collect();

    let preview =
        Paragraph::new(preview_lines).block(Block::default().borders(Borders::ALL).title(format!(
            "Preview ({} matches, PgUp/PgDn to scroll)",
            matching_entries.len()
        )));
    f.render_widget(preview, chunks[2]);

    // Render save dialog overlay if in Saving state
    if let AppState::Saving = app.state {
        if let Some(ref save_dialog) = app.save_dialog {
            let match_count = if app.selected_pattern < app.patterns.len() {
                app.patterns[app.selected_pattern].match_count
            } else {
                0
            };

            let config = RuleWriterConfig::default();
            let target_path = config.get_path(save_dialog.selected_category());

            save_dialog.render(f, f.area(), match_count, &target_path.to_string_lossy());
        }
    }
}
