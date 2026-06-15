//! Centered help overlay listing all keybindings.

use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use super::center;

/// Render the help popup centered over the given area.
pub fn render(frame: &mut Frame, area: Rect) {
    let popup = center(area, Constraint::Length(46), Constraint::Length(21));

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Help ")
        .style(Style::default().bg(Color::Black));

    let lines = vec![
        section("Navigation"),
        key("↑ / k, ↓ / j", "move selection / scroll"),
        key("g / G", "jump to top / bottom"),
        key("a", "toggle all / running only"),
        Line::from(""),
        section("Containers"),
        key("Enter / l", "open logs"),
        key("S / s", "start / stop"),
        key("r", "restart"),
        key("x / d", "remove (confirm)"),
        Line::from(""),
        section("Logs"),
        key("PgUp / PgDn", "page up / down"),
        key("f", "toggle follow"),
        key("Esc / q", "back to list"),
        Line::from(""),
        section("General"),
        key("? ", "toggle this help"),
        key("q", "quit"),
    ];

    frame.render_widget(Clear, popup);
    frame.render_widget(Paragraph::new(lines).block(block), popup);
}

fn section(title: &str) -> Line<'_> {
    Line::from(Span::styled(
        title,
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))
}

fn key<'a>(keys: &'a str, desc: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("  {keys:<14}"), Style::default().fg(Color::Yellow)),
        Span::raw(desc),
    ])
}
