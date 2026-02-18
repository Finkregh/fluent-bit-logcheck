//! Preview widget for analyzer TUI

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use regex::Regex;

pub struct Preview<'a> {
    entries: &'a [String],
    scroll: usize,
    total_matches: usize,
    pattern: Option<&'a Regex>,
    focused: bool,
    wrap: bool,
    h_scroll: usize,
}

impl<'a> Preview<'a> {
    pub fn new(
        entries: &'a [String],
        scroll: usize,
        total_matches: usize,
        pattern: Option<&'a Regex>,
        focused: bool,
        wrap: bool,
        h_scroll: usize,
    ) -> Self {
        Self {
            entries,
            scroll,
            total_matches,
            pattern,
            focused,
            wrap,
            h_scroll,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let visible_height = area.height.saturating_sub(2) as usize;
        let lines = self
            .entries
            .iter()
            .skip(self.scroll)
            .take(visible_height)
            .map(|entry| self.render_line(entry))
            .collect::<Vec<_>>();

        let title = format!(
            "Preview ({} matches, PgUp/PgDn to scroll)",
            self.total_matches
        );
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(self.border_style());
        let mut widget = Paragraph::new(lines).block(block);
        if self.wrap {
            widget = widget.wrap(ratatui::widgets::Wrap { trim: false });
        } else if self.h_scroll > 0 {
            widget = widget.scroll((0, self.h_scroll as u16));
        }
        f.render_widget(widget, area);
    }

    fn render_line(&self, entry: &str) -> Line<'static> {
        let Some(regex) = self.pattern else {
            return Line::from(entry.to_string());
        };

        let highlight_style = Style::default()
            .fg(Color::Black)
            .bg(Color::Yellow)
            .add_modifier(Modifier::BOLD);

        let mut spans: Vec<Span> = Vec::new();
        let mut last_end = 0;

        for mat in regex.find_iter(entry) {
            if mat.start() > last_end {
                spans.push(Span::raw(entry[last_end..mat.start()].to_string()));
            }
            spans.push(Span::styled(mat.as_str().to_string(), highlight_style));
            last_end = mat.end();
        }

        if spans.is_empty() {
            return Line::from(entry.to_string());
        }

        if last_end < entry.len() {
            spans.push(Span::raw(entry[last_end..].to_string()));
        }

        Line::from(spans)
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
}
