//! Centered yes/no overlay confirming a destructive container action.

use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use super::center;
use crate::app::PendingAction;

/// Render the confirmation popup for `pending`, centered over `area`.
pub fn render(frame: &mut Frame, area: Rect, pending: &PendingAction) {
    let popup = center(area, Constraint::Length(54), Constraint::Length(5));

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Confirm ")
        .style(Style::default().bg(Color::Black));

    let prompt = Line::from(vec![
        Span::raw(format!("{} container ", capitalize(pending.action.verb()))),
        Span::styled(
            pending.container_name.as_str(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("?"),
    ]);
    let keys = Line::from(Span::styled(
        "y confirm · n / Esc cancel",
        Style::default().fg(Color::DarkGray),
    ));

    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(vec![prompt, Line::from(""), keys]).block(block),
        popup,
    );
}

/// Uppercase the first character of `s` (ASCII action verbs only).
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
