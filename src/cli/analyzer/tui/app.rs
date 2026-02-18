/// Main TUI application for pattern analysis
use crate::cli::analyzer::pattern_grouper::PatternGroup;
use crate::cli::analyzer::rule_writer::{RuleWriter, RuleWriterConfig};
use crate::cli::Result;
use anyhow::Context;

use super::pattern_list::PatternList;
use super::preview::Preview;
use super::save_dialog::SaveDialog;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
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
    /// Showing full line modal
    Modal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusBox {
    Patterns,
    Preview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModalSource {
    Patterns,
    Preview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditMode {
    Select,
    Editing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReplacementType {
    Wildcard,
    Word,
    Digit,
    NonSpace,
    Alternation,
}

#[derive(Debug, Clone)]
struct EditSpan {
    start: usize,
    end: usize,
    replacement: ReplacementType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AlternationSource {
    Group,
    PreviewMatches,
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
    /// Focused box
    focused: FocusBox,
    /// Wrap mode for pattern list
    pattern_wrap: bool,
    /// Wrap mode for preview
    preview_wrap: bool,
    /// Horizontal scroll for pattern list
    pattern_h_scroll: usize,
    /// Horizontal scroll for preview
    preview_h_scroll: usize,
    /// Modal content
    modal_line: Option<String>,
    /// Modal source
    modal_source: Option<ModalSource>,
    /// Edit mode
    edit_mode: EditMode,
    /// Sample line for editing (from selected pattern)
    edit_line: Option<String>,
    /// Cursor position within edit line
    edit_cursor: usize,
    /// Selected spans (start, end) for replacement
    edit_spans: Vec<EditSpan>,
    /// Derived regex preview for editing mode
    edit_regex: Option<String>,
    /// Alternation source mode
    alternation_source: AlternationSource,
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
            focused: FocusBox::Patterns,
            pattern_wrap: false,
            preview_wrap: false,
            pattern_h_scroll: 0,
            preview_h_scroll: 0,
            modal_line: None,
            modal_source: None,
            edit_mode: EditMode::Select,
            edit_line: None,
            edit_cursor: 0,
            edit_spans: Vec::new(),
            edit_regex: None,
            alternation_source: AlternationSource::Group,
        }
    }

    fn next_pattern(&mut self) {
        if self.selected_pattern < self.patterns.len().saturating_sub(1) {
            self.selected_pattern += 1;
            self.preview_scroll = 0;
            self.preview_h_scroll = 0;
        }
    }

    fn previous_pattern(&mut self) {
        if self.selected_pattern > 0 {
            self.selected_pattern -= 1;
            self.preview_scroll = 0;
            self.preview_h_scroll = 0;
        }
    }

    fn scroll_preview_down(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_add(1);
    }

    fn scroll_preview_up(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_sub(1);
    }

    fn focus_next(&mut self) {
        self.focused = match self.focused {
            FocusBox::Patterns => FocusBox::Preview,
            FocusBox::Preview => FocusBox::Patterns,
        };
    }

    fn focus_prev(&mut self) {
        self.focus_next();
    }

    fn toggle_wrap(&mut self) {
        match self.focused {
            FocusBox::Patterns => self.pattern_wrap = !self.pattern_wrap,
            FocusBox::Preview => self.preview_wrap = !self.preview_wrap,
        }
    }

    fn scroll_left(&mut self) {
        match self.focused {
            FocusBox::Patterns => self.pattern_h_scroll = self.pattern_h_scroll.saturating_sub(1),
            FocusBox::Preview => self.preview_h_scroll = self.preview_h_scroll.saturating_sub(1),
        }
    }

    fn scroll_right(&mut self) {
        match self.focused {
            FocusBox::Patterns => self.pattern_h_scroll = self.pattern_h_scroll.saturating_add(1),
            FocusBox::Preview => self.preview_h_scroll = self.preview_h_scroll.saturating_add(1),
        }
    }

    fn open_modal_for_selected(&mut self) {
        match self.focused {
            FocusBox::Patterns => {
                if let Some(pattern) = self.patterns.get(self.selected_pattern) {
                    self.modal_line = Some(pattern.display());
                    self.modal_source = Some(ModalSource::Patterns);
                    self.state = AppState::Modal;
                }
            }
            FocusBox::Preview => {
                let matching_entries = self.get_matching_entries();
                if let Some(entry) = matching_entries.get(self.preview_scroll) {
                    self.modal_line = Some(entry.clone());
                    self.modal_source = Some(ModalSource::Preview);
                    self.state = AppState::Modal;
                }
            }
        }
    }

    fn close_modal(&mut self) {
        self.modal_line = None;
        self.modal_source = None;
        self.state = AppState::Browsing;
    }

    fn open_save_dialog(&mut self) {
        if self.edit_mode == EditMode::Editing {
            if let Some(ref regex) = self.edit_regex {
                self.save_dialog = Some(SaveDialog::new(regex.clone()));
                self.state = AppState::Saving;
            }
            return;
        }

        if self.selected_pattern < self.patterns.len() {
            let pattern = &self.patterns[self.selected_pattern];
            self.save_dialog = Some(SaveDialog::new(pattern.regex.clone()));
            self.state = AppState::Saving;
        }
    }

    fn enter_edit_mode(&mut self) {
        if self.selected_pattern >= self.patterns.len() {
            return;
        }
        let pattern = &self.patterns[self.selected_pattern];
        if let Some(&first_idx) = pattern.matching_indices.first() {
            if let Some(line) = self.log_entries.get(first_idx) {
                self.edit_mode = EditMode::Editing;
                self.edit_line = Some(line.clone());
                self.edit_cursor = 0;
                self.edit_spans.clear();
                self.edit_regex = Some(regex::escape(line));
                self.alternation_source = AlternationSource::Group;
            }
        }
    }

    fn exit_edit_mode(&mut self) {
        self.edit_mode = EditMode::Select;
        self.edit_line = None;
        self.edit_cursor = 0;
        self.edit_spans.clear();
        self.edit_regex = None;
    }

    fn move_edit_cursor_left(&mut self) {
        self.edit_cursor = self.edit_cursor.saturating_sub(1);
    }

    fn move_edit_cursor_right(&mut self) {
        if let Some(ref line) = self.edit_line {
            if self.edit_cursor < line.len() {
                self.edit_cursor += 1;
            }
        }
    }

    fn toggle_span_at_cursor(&mut self) {
        let Some(ref line) = self.edit_line else {
            return;
        };
        if self.edit_cursor >= line.len() {
            return;
        }

        if let Some((start, end)) = find_word_span(line, self.edit_cursor) {
            if let Some(idx) = self
                .edit_spans
                .iter()
                .position(|span| span.start == start && span.end == end)
            {
                self.edit_spans.remove(idx);
            } else {
                self.edit_spans.push(EditSpan {
                    start,
                    end,
                    replacement: ReplacementType::Wildcard,
                });
            }
            self.edit_spans.sort_by_key(|span| span.start);
            self.recompute_edit_regex();
        }
    }

    fn cycle_span_replacement(&mut self) {
        let Some((start, end)) = self.current_span_range() else {
            return;
        };
        if let Some(span) = self
            .edit_spans
            .iter_mut()
            .find(|span| span.start == start && span.end == end)
        {
            span.replacement = match span.replacement {
                ReplacementType::Wildcard => ReplacementType::Word,
                ReplacementType::Word => ReplacementType::Digit,
                ReplacementType::Digit => ReplacementType::NonSpace,
                ReplacementType::NonSpace => ReplacementType::Alternation,
                ReplacementType::Alternation => ReplacementType::Wildcard,
            };
            self.recompute_edit_regex();
        }
    }

    fn set_span_replacement(&mut self, replacement: ReplacementType) {
        let Some((start, end)) = self.current_span_range() else {
            return;
        };
        if let Some(span) = self
            .edit_spans
            .iter_mut()
            .find(|span| span.start == start && span.end == end)
        {
            span.replacement = replacement;
            self.recompute_edit_regex();
        }
    }

    fn toggle_alternation_source(&mut self) {
        self.alternation_source = match self.alternation_source {
            AlternationSource::Group => AlternationSource::PreviewMatches,
            AlternationSource::PreviewMatches => AlternationSource::Group,
        };
        self.recompute_edit_regex();
    }

    fn current_span_range(&self) -> Option<(usize, usize)> {
        let line = self.edit_line.as_ref()?;
        if self.edit_cursor >= line.len() {
            return None;
        }
        find_word_span(line, self.edit_cursor)
    }

    fn recompute_edit_regex(&mut self) {
        let Some(ref line) = self.edit_line else {
            return;
        };
        let source_lines = self.get_alternation_source_lines();
        self.edit_regex = Some(build_regex_from_spans(
            line,
            &self.edit_spans,
            &source_lines,
        ));
    }

    fn get_alternation_source_lines(&self) -> Vec<String> {
        if self.selected_pattern >= self.patterns.len() {
            return Vec::new();
        }
        match self.alternation_source {
            AlternationSource::Group => {
                let pattern = &self.patterns[self.selected_pattern];
                pattern
                    .matching_indices
                    .iter()
                    .filter_map(|&idx| self.log_entries.get(idx).cloned())
                    .collect()
            }
            AlternationSource::PreviewMatches => self.get_matching_entries(),
        }
    }

    fn cancel_save(&mut self) {
        self.save_dialog = None;
        self.state = AppState::Browsing;
    }

    fn get_matching_entries(&self) -> Vec<String> {
        if self.edit_mode == EditMode::Editing {
            if let Some(ref regex) = self.edit_regex {
                if let Ok(compiled) = regex::Regex::new(regex) {
                    return self
                        .log_entries
                        .iter()
                        .filter(|entry| compiled.is_match(entry))
                        .cloned()
                        .collect();
                }
            }
            return Vec::new();
        }

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

    fn current_match_count(&self) -> usize {
        self.get_matching_entries().len()
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
                    KeyCode::Char('e') => {
                        if app.focused == FocusBox::Patterns {
                            app.enter_edit_mode();
                        }
                    }
                    KeyCode::Char('x') => {
                        if app.edit_mode == EditMode::Editing {
                            app.exit_edit_mode();
                        }
                    }
                    KeyCode::Tab => app.focus_next(),
                    KeyCode::BackTab => app.focus_prev(),
                    KeyCode::Char('w') => app.toggle_wrap(),
                    KeyCode::Left | KeyCode::Char('h') => {
                        if app.edit_mode == EditMode::Editing {
                            app.move_edit_cursor_left();
                        } else {
                            app.scroll_left();
                        }
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        if app.edit_mode == EditMode::Editing {
                            app.move_edit_cursor_right();
                        } else {
                            app.scroll_right();
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if app.focused == FocusBox::Patterns {
                            app.next_pattern()
                        } else {
                            app.scroll_preview_down()
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if app.focused == FocusBox::Patterns {
                            app.previous_pattern()
                        } else {
                            app.scroll_preview_up()
                        }
                    }
                    KeyCode::PageDown => {
                        if app.focused == FocusBox::Preview {
                            app.scroll_preview_down()
                        }
                    }
                    KeyCode::PageUp => {
                        if app.focused == FocusBox::Preview {
                            app.scroll_preview_up()
                        }
                    }
                    KeyCode::Enter => {
                        if app.focused == FocusBox::Patterns {
                            app.open_save_dialog()
                        }
                    }
                    KeyCode::Char('m') => {
                        if app.edit_mode == EditMode::Editing && app.focused == FocusBox::Patterns {
                            app.toggle_span_at_cursor();
                        }
                    }
                    KeyCode::Char('r') => {
                        if app.edit_mode == EditMode::Editing && app.focused == FocusBox::Patterns {
                            app.cycle_span_replacement();
                        }
                    }
                    KeyCode::Char('1') => {
                        if app.edit_mode == EditMode::Editing && app.focused == FocusBox::Patterns {
                            app.set_span_replacement(ReplacementType::Wildcard);
                        }
                    }
                    KeyCode::Char('2') => {
                        if app.edit_mode == EditMode::Editing && app.focused == FocusBox::Patterns {
                            app.set_span_replacement(ReplacementType::Word);
                        }
                    }
                    KeyCode::Char('3') => {
                        if app.edit_mode == EditMode::Editing && app.focused == FocusBox::Patterns {
                            app.set_span_replacement(ReplacementType::Digit);
                        }
                    }
                    KeyCode::Char('4') => {
                        if app.edit_mode == EditMode::Editing && app.focused == FocusBox::Patterns {
                            app.set_span_replacement(ReplacementType::NonSpace);
                        }
                    }
                    KeyCode::Char('5') => {
                        if app.edit_mode == EditMode::Editing && app.focused == FocusBox::Patterns {
                            app.set_span_replacement(ReplacementType::Alternation);
                        }
                    }
                    KeyCode::Char('t') => {
                        if app.edit_mode == EditMode::Editing && app.focused == FocusBox::Patterns {
                            app.toggle_alternation_source();
                        }
                    }
                    KeyCode::Char(' ') => app.open_modal_for_selected(),
                    _ => {}
                },
                AppState::Modal => match key.code {
                    KeyCode::Esc | KeyCode::Char(' ') => app.close_modal(),
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
    let header = Paragraph::new(format!(
        "{} | Focus: {} | w wrap | h/l scroll | space full | tab switch | e edit | m mark | r cycle | 1-5 set | t alt-src | x finish",
        header_text,
        match app.focused {
            FocusBox::Patterns => "patterns",
            FocusBox::Preview => "preview",
        }
    ))
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(header, chunks[0]);

    let pattern_items = if app.edit_mode == EditMode::Editing {
        vec![app
            .edit_regex
            .clone()
            .unwrap_or_else(|| "<no regex>".to_string())]
    } else {
        app.patterns.iter().map(|p| p.display()).collect()
    };

    let pattern_list = PatternList::new(
        &pattern_items,
        if app.edit_mode == EditMode::Editing {
            0
        } else {
            app.selected_pattern
        },
        app.focused == FocusBox::Patterns,
        app.pattern_wrap,
        app.pattern_h_scroll,
    );
    pattern_list.render(f, chunks[1]);

    let matching_entries = app.get_matching_entries();
    let preview_regex = if app.edit_mode == EditMode::Editing {
        app.edit_regex
            .as_ref()
            .and_then(|regex| regex::Regex::new(regex).ok())
    } else {
        app.patterns
            .get(app.selected_pattern)
            .and_then(|pattern| pattern.compile().ok())
    };
    let preview = Preview::new(
        &matching_entries,
        app.preview_scroll,
        matching_entries.len(),
        preview_regex.as_ref(),
        app.focused == FocusBox::Preview,
        app.preview_wrap,
        app.preview_h_scroll,
    );
    preview.render(f, chunks[2]);

    // Render save dialog overlay if in Saving state
    if let AppState::Saving = app.state {
        if let Some(ref save_dialog) = app.save_dialog {
            let match_count = if app.edit_mode == EditMode::Editing {
                app.current_match_count()
            } else if app.selected_pattern < app.patterns.len() {
                app.patterns[app.selected_pattern].match_count
            } else {
                0
            };

            let config = RuleWriterConfig::default();
            let target_path = config.get_path(save_dialog.selected_category());

            save_dialog.render(f, f.area(), match_count, &target_path.to_string_lossy());
        }
    }

    // Render modal overlay if in Modal state
    if let AppState::Modal = app.state {
        if let Some(ref line) = app.modal_line {
            let width = f.area().width.min(120);
            let height = f.area().height.min(8);
            let x = (f.area().width.saturating_sub(width)) / 2;
            let y = (f.area().height.saturating_sub(height)) / 2;
            let modal_area = ratatui::layout::Rect::new(x, y, width, height);

            let title = match app.modal_source {
                Some(ModalSource::Patterns) => "Pattern details",
                Some(ModalSource::Preview) => "Preview line",
                None => "Details",
            };

            let block = Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Yellow));
            let text = Paragraph::new(line.as_str())
                .block(block)
                .wrap(ratatui::widgets::Wrap { trim: false })
                .style(Style::default().fg(Color::White));
            f.render_widget(text, modal_area);
        }
    }

    if app.edit_mode == EditMode::Editing {
        if let Some(ref line) = app.edit_line {
            let width = f.area().width.min(120);
            let height = f.area().height.min(7);
            let x = (f.area().width.saturating_sub(width)) / 2;
            let y = (f.area().height.saturating_sub(height)) / 2;
            let edit_area = ratatui::layout::Rect::new(x, y, width, height);

            let block = Block::default()
                .borders(Borders::ALL)
                .title(format!(
                    "Mark parts (m) | r cycle | 1-5 set | t alt-src ({}) | x finish",
                    match app.alternation_source {
                        AlternationSource::Group => "group",
                        AlternationSource::PreviewMatches => "preview",
                    }
                ))
                .border_style(Style::default().fg(Color::Yellow));

            let line_with_cursor = inject_cursor(line, app.edit_cursor);
            let text = Paragraph::new(line_with_cursor)
                .block(block)
                .wrap(ratatui::widgets::Wrap { trim: false });
            f.render_widget(text, edit_area);
        }
    }
}

fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || matches!(
            c,
            '_' | '-' | ':' | '(' | ')' | '=' | '.' | '/' | '[' | ']' | '@'
        )
}

fn find_word_span(line: &str, cursor: usize) -> Option<(usize, usize)> {
    let chars: Vec<(usize, char)> = line.char_indices().collect();
    let pos = chars
        .iter()
        .position(|(idx, _)| *idx == cursor)
        .or_else(|| chars.iter().position(|(idx, _)| *idx > cursor))?;

    if !is_word_char(chars[pos].1) {
        return None;
    }

    let mut start = pos;
    while start > 0 && is_word_char(chars[start - 1].1) {
        start -= 1;
    }

    let mut end = pos;
    while end + 1 < chars.len() && is_word_char(chars[end + 1].1) {
        end += 1;
    }

    let start_idx = chars[start].0;
    let end_idx = if end + 1 < chars.len() {
        chars[end + 1].0
    } else {
        line.len()
    };

    Some((start_idx, end_idx))
}

fn build_regex_from_spans(line: &str, spans: &[EditSpan], samples: &[String]) -> String {
    if spans.is_empty() {
        return format!("^{}$", regex::escape(line));
    }

    let mut regex = String::new();
    let mut cursor = 0;
    for span in spans {
        if span.start > cursor {
            regex.push_str(&regex::escape(&line[cursor..span.start]));
        }
        regex.push_str(&replacement_for_span(span, samples));
        cursor = span.end;
    }
    if cursor < line.len() {
        regex.push_str(&regex::escape(&line[cursor..]));
    }
    format!("^{}$", regex)
}

fn replacement_for_span(span: &EditSpan, samples: &[String]) -> String {
    match span.replacement {
        ReplacementType::Wildcard => ".*".to_string(),
        ReplacementType::Word => r"\w+".to_string(),
        ReplacementType::Digit => r"\d+".to_string(),
        ReplacementType::NonSpace => r"\S+".to_string(),
        ReplacementType::Alternation => build_alternation(span, samples),
    }
}

fn build_alternation(span: &EditSpan, samples: &[String]) -> String {
    let mut values = samples
        .iter()
        .filter_map(|line| line.get(span.start..span.end))
        .map(regex::escape)
        .collect::<Vec<_>>();

    values.sort();
    values.dedup();

    if values.is_empty() {
        return ".*".to_string();
    }
    format!("(?:{})", values.join("|"))
}

fn inject_cursor(line: &str, cursor: usize) -> String {
    let mut output = line.to_string();
    if cursor <= output.len() {
        output.insert(cursor, '│');
    } else {
        output.push('│');
    }
    output
}
