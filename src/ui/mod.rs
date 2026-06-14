//! Rendering layer. Pure functions of [`App`] → frame; no state mutation here.

mod confirm;
mod containers;
mod detail;
mod help;
mod logs;

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::{App, View};

/// Accent color used for the title bar and highlights.
const ACCENT: Color = Color::Cyan;

/// Top-level render entry point called once per frame.
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::vertical([
        Constraint::Length(1), // header
        Constraint::Min(1),    // body
        Constraint::Length(1), // footer
    ])
    .split(area);

    render_header(frame, app, chunks[0]);

    match app.view {
        View::List => {
            let body =
                Layout::vertical([Constraint::Min(3), Constraint::Length(8)]).split(chunks[1]);
            containers::render(frame, app, body[0]);
            detail::render(frame, app, body[1]);
        }
        View::Logs => logs::render(frame, app, chunks[1]),
    }

    render_footer(frame, app, chunks[2]);

    if app.show_help {
        help::render(frame, area);
    }
    if let Some(pending) = &app.confirm {
        confirm::render(frame, area, pending);
    }
}

/// Center a fixed-size rect inside `area`. Shared by the modal overlays.
pub(super) fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [h] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [v] = Layout::vertical([vertical]).flex(Flex::Center).areas(h);
    v
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let visible = app.visible_indices().len();
    let total = app.containers.len();
    let filter = if app.show_all { "all" } else { "running" };
    let line = Line::from(vec![
        Span::styled(
            " stevedore ",
            Style::default()
                .fg(Color::Black)
                .bg(ACCENT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("  Docker {}  ", app.docker_version)),
        Span::styled(
            format!("[{filter}: {visible}/{total}]"),
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(status) = &app.status {
        let line = Line::from(Span::styled(
            format!(" {status}"),
            Style::default().fg(Color::Yellow),
        ));
        frame.render_widget(Paragraph::new(line), area);
        return;
    }

    let hint = match app.view {
        View::List => " ↑↓/jk move · a all · enter logs · s/S/r/x actions · ? help · q quit",
        View::Logs => " ↑↓/jk scroll · PgUp/PgDn page · g/G top/bottom · f follow · q back",
    };
    let line = Line::from(Span::styled(hint, Style::default().fg(Color::DarkGray)));
    frame.render_widget(Paragraph::new(line), area);
}
