//! Pattern list widget for analyzer TUI

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct PatternList<'a> {
    items: &'a [String],
    selected: usize,
    focused: bool,
    wrap: bool,
    h_scroll: usize,
}

impl<'a> PatternList<'a> {
    pub fn new(
        items: &'a [String],
        selected: usize,
        focused: bool,
        wrap: bool,
        h_scroll: usize,
    ) -> Self {
        Self {
            items,
            selected,
            focused,
            wrap,
            h_scroll,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let items = self.build_items();
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Patterns (↑/↓ to navigate, Enter to save, q to quit)")
            .border_style(self.border_style());
        let mut widget = Paragraph::new(items).block(block);
        if self.wrap {
            widget = widget.wrap(ratatui::widgets::Wrap { trim: false });
        } else if self.h_scroll > 0 {
            widget = widget.scroll((0, self.h_scroll as u16));
        }
        f.render_widget(widget, area);
    }

    fn border_style(&self) -> Style {
        if self.focused {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        }
    }

    fn build_items(&self) -> Vec<Line<'static>> {
        if self.items.is_empty() {
            return vec![Line::from(Span::styled(
                "No patterns available",
                Style::default().fg(Color::DarkGray),
            ))];
        }

        self.items
            .iter()
            .enumerate()
            .map(|(idx, pattern)| {
                let selected = idx == self.selected;
                let style = if selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                let prefix = if selected { "> " } else { "  " };
                Line::from(Span::styled(format!("{}{}", prefix, pattern), style))
            })
            .collect()
    }
}
