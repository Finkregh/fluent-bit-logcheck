//! Save dialog for TUI

use crate::cli::analyzer::rule_writer::RuleCategory;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// State for the save dialog
pub struct SaveDialog {
    /// Pattern being saved
    pattern: String,
    /// Currently selected category
    selected_category: usize,
    /// Whether pattern is being edited
    editing_pattern: bool,
    /// Current cursor position in pattern (if editing)
    cursor_position: usize,
}

impl SaveDialog {
    /// Create new save dialog with pattern
    pub fn new(pattern: String) -> Self {
        Self {
            cursor_position: pattern.len(),
            pattern,
            selected_category: 1, // Default to Server
            editing_pattern: false,
        }
    }

    /// Get currently selected category
    pub fn selected_category(&self) -> RuleCategory {
        RuleCategory::all()[self.selected_category]
    }

    /// Get the pattern to save
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Start editing the pattern
    pub fn start_edit(&mut self) {
        self.editing_pattern = true;
        self.cursor_position = self.pattern.len();
    }

    /// Stop editing the pattern
    pub fn stop_edit(&mut self) {
        self.editing_pattern = false;
    }

    /// Check if currently editing
    pub fn is_editing(&self) -> bool {
        self.editing_pattern
    }

    /// Move category selection left
    pub fn category_left(&mut self) {
        if self.selected_category > 0 {
            self.selected_category -= 1;
        }
    }

    /// Move category selection right
    pub fn category_right(&mut self) {
        let max = RuleCategory::all().len() - 1;
        if self.selected_category < max {
            self.selected_category += 1;
        }
    }

    /// Add character to pattern (when editing)
    pub fn add_char(&mut self, c: char) {
        if self.editing_pattern {
            self.pattern.insert(self.cursor_position, c);
            self.cursor_position += 1;
        }
    }

    /// Delete character before cursor (backspace)
    pub fn backspace(&mut self) {
        if self.editing_pattern && self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.pattern.remove(self.cursor_position);
        }
    }

    /// Delete character at cursor (delete key)
    pub fn delete(&mut self) {
        if self.editing_pattern && self.cursor_position < self.pattern.len() {
            self.pattern.remove(self.cursor_position);
        }
    }

    /// Move cursor left
    pub fn cursor_left(&mut self) {
        if self.editing_pattern && self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Move cursor right
    pub fn cursor_right(&mut self) {
        if self.editing_pattern && self.cursor_position < self.pattern.len() {
            self.cursor_position += 1;
        }
    }

    /// Render the save dialog
    pub fn render(&self, f: &mut Frame, area: Rect, match_count: usize, target_path: &str) {
        // Create centered area for dialog
        let dialog_width = area.width.min(100);
        let dialog_height = 14;
        let x = (area.width.saturating_sub(dialog_width)) / 2;
        let y = (area.height.saturating_sub(dialog_height)) / 2;
        let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

        // Clear background
        let bg = Block::default()
            .style(Style::default().bg(Color::Black))
            .borders(Borders::ALL)
            .title("Save Rule")
            .border_style(Style::default().fg(Color::Yellow));
        f.render_widget(bg, dialog_area);

        // Split dialog into sections
        let inner = dialog_area.inner(ratatui::layout::Margin {
            vertical: 1,
            horizontal: 2,
        });

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Pattern display/edit
                Constraint::Length(3), // Category selection
                Constraint::Length(2), // Target path
                Constraint::Length(2), // Match count
                Constraint::Length(2), // Help text
            ])
            .split(inner);

        // Pattern section
        let pattern_title = if self.editing_pattern {
            "Regex Pattern (editing, Esc to finish):"
        } else {
            "Regex Pattern (e to edit):"
        };
        let pattern_style = if self.editing_pattern {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let mut pattern_text = self.pattern.clone();
        if self.editing_pattern {
            // Show cursor
            if self.cursor_position < pattern_text.len() {
                pattern_text.insert(self.cursor_position, '│');
            } else {
                pattern_text.push('│');
            }
        }

        let pattern_widget = Paragraph::new(pattern_text)
            .style(pattern_style)
            .wrap(Wrap { trim: false })
            .block(Block::default().title(pattern_title));
        f.render_widget(pattern_widget, chunks[0]);

        // Category selection
        let categories: Vec<Span> = RuleCategory::all()
            .iter()
            .enumerate()
            .flat_map(|(idx, cat)| {
                let is_selected = idx == self.selected_category;
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                let radio = if is_selected { "●" } else { "○" };

                vec![
                    Span::styled(format!(" {} ", radio), style),
                    Span::styled(cat.display_name(), style),
                    Span::raw("  "),
                ]
            })
            .collect();

        let category_widget = Paragraph::new(Line::from(categories))
            .block(Block::default().title("Category (←/→ to select):"));
        f.render_widget(category_widget, chunks[1]);

        // Target path
        let path_widget = Paragraph::new(target_path)
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().title("Target File:"));
        f.render_widget(path_widget, chunks[2]);

        // Match count
        let matches_text = format!("Matches: {} entries", match_count);
        let matches_widget = Paragraph::new(matches_text).style(Style::default().fg(Color::Green));
        f.render_widget(matches_widget, chunks[3]);

        // Help text
        let help_text = if self.editing_pattern {
            "←/→ to move cursor, Esc when done editing"
        } else {
            "[Enter] Save   [e] Edit pattern   [←/→] Change category   [Esc] Cancel"
        };
        let help_widget = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(help_widget, chunks[4]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_dialog_creation() {
        let dialog = SaveDialog::new("^test pattern$".to_string());
        assert_eq!(dialog.pattern(), "^test pattern$");
        assert_eq!(dialog.selected_category(), RuleCategory::Server);
        assert!(!dialog.is_editing());
    }

    #[test]
    fn test_category_navigation() {
        let mut dialog = SaveDialog::new("pattern".to_string());

        // Start at Server (index 1)
        assert_eq!(dialog.selected_category(), RuleCategory::Server);

        dialog.category_left();
        assert_eq!(dialog.selected_category(), RuleCategory::Violations);

        dialog.category_left();
        assert_eq!(dialog.selected_category(), RuleCategory::Violations); // Can't go further

        dialog.category_right();
        assert_eq!(dialog.selected_category(), RuleCategory::Server);

        dialog.category_right();
        assert_eq!(dialog.selected_category(), RuleCategory::Workstation);

        dialog.category_right();
        assert_eq!(dialog.selected_category(), RuleCategory::Local);

        dialog.category_right();
        assert_eq!(dialog.selected_category(), RuleCategory::Local); // Can't go further
    }

    #[test]
    fn test_pattern_editing() {
        let mut dialog = SaveDialog::new("test".to_string());

        assert!(!dialog.is_editing());

        dialog.start_edit();
        assert!(dialog.is_editing());
        assert_eq!(dialog.cursor_position, 4); // At end

        dialog.add_char('!');
        assert_eq!(dialog.pattern(), "test!");

        dialog.backspace();
        assert_eq!(dialog.pattern(), "test");

        dialog.stop_edit();
        assert!(!dialog.is_editing());
    }

    #[test]
    fn test_cursor_movement() {
        let mut dialog = SaveDialog::new("abc".to_string());
        dialog.start_edit();

        assert_eq!(dialog.cursor_position, 3);

        dialog.cursor_left();
        assert_eq!(dialog.cursor_position, 2);

        dialog.cursor_left();
        assert_eq!(dialog.cursor_position, 1);

        dialog.cursor_left();
        assert_eq!(dialog.cursor_position, 0);

        dialog.cursor_left();
        assert_eq!(dialog.cursor_position, 0); // Can't go further

        dialog.cursor_right();
        assert_eq!(dialog.cursor_position, 1);

        dialog.cursor_right();
        dialog.cursor_right();
        assert_eq!(dialog.cursor_position, 3);

        dialog.cursor_right();
        assert_eq!(dialog.cursor_position, 3); // Can't go further
    }

    #[test]
    fn test_insert_at_cursor() {
        let mut dialog = SaveDialog::new("ac".to_string());
        dialog.start_edit();

        dialog.cursor_left(); // Move to position 1
        dialog.add_char('b');

        assert_eq!(dialog.pattern(), "abc");
        assert_eq!(dialog.cursor_position, 2);
    }

    #[test]
    fn test_delete_operations() {
        let mut dialog = SaveDialog::new("abcd".to_string());
        dialog.start_edit();

        // Position at end (4)
        dialog.backspace(); // Remove 'd'
        assert_eq!(dialog.pattern(), "abc");
        assert_eq!(dialog.cursor_position, 3);

        // Move to position 1
        dialog.cursor_left();
        dialog.cursor_left();
        assert_eq!(dialog.cursor_position, 1);

        dialog.delete(); // Remove 'b'
        assert_eq!(dialog.pattern(), "ac");
        assert_eq!(dialog.cursor_position, 1);
    }
}
