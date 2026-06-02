//! The log viewer.
//!
//! Lines are windowed to the visible height by `scroll` (counted from the
//! bottom), so even a 5000-line buffer renders in constant time.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;

/// Render the log buffer for the active container.
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let logs = &app.logs;

    let mode = if logs.follow {
        Span::styled(
            " [FOLLOW] ",
            Style::default().fg(Color::Black).bg(Color::Green),
        )
    } else {
        Span::styled(
            format!(" [scroll +{}] ", logs.scroll),
            Style::default().fg(Color::Black).bg(Color::Yellow),
        )
    };
    let title = Line::from(vec![
        Span::styled(
            format!(" logs: {} ", logs.container_name),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        mode,
    ]);
    let block = Block::default().borders(Borders::ALL).title(title);

    // Inner height available for log text (area minus top/bottom borders).
    let height = area.height.saturating_sub(2) as usize;
    let total = logs.lines.len();
    let end = total.saturating_sub(logs.scroll);
    let start = end.saturating_sub(height);

    let lines: Vec<Line> = logs
        .lines
        .iter()
        .skip(start)
        .take(end - start)
        .map(|l| Line::from(l.as_str()))
        .collect();

    frame.render_widget(Paragraph::new(lines).block(block), area);
}
