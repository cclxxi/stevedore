//! The container list table.

use ratatui::Frame;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};

use crate::app::App;
use crate::docker::model::{Container, ContainerState};
use crate::util::format_bytes;

/// Render the filtered container list with live CPU/MEM columns.
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(" Containers ");

    let visible = app.visible_indices();
    if visible.is_empty() {
        let hint = if app.show_all {
            "No containers found."
        } else {
            "No running containers. Press 'a' to show all."
        };
        frame.render_widget(ratatui::widgets::Paragraph::new(hint).block(block), area);
        return;
    }

    let header = Row::new(
        ["NAME", "IMAGE", "STATE", "CPU %", "MEM", "STATUS"]
            .into_iter()
            .map(|h| Cell::from(h).style(Style::default().add_modifier(Modifier::BOLD))),
    )
    .style(Style::default().fg(Color::DarkGray));

    let rows = visible.iter().map(|&i| {
        let container = &app.containers[i];
        row_for(container, app)
    });

    let widths = [
        Constraint::Percentage(22),
        Constraint::Percentage(30),
        Constraint::Length(11),
        Constraint::Length(7),
        Constraint::Length(18),
        Constraint::Percentage(20),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .row_highlight_style(
            Style::default()
                .bg(Color::Rgb(40, 40, 60))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▌ ");

    let mut state = TableState::default().with_selected(Some(app.selected));
    frame.render_stateful_widget(table, area, &mut state);
}

/// Build a single table row for a container.
fn row_for<'a>(container: &'a Container, app: &App) -> Row<'a> {
    let stats = app.stats.get(&container.id);
    let cpu = stats
        .map(|s| format!("{:.1}", s.cpu_percent))
        .unwrap_or_else(|| "-".to_string());
    let mem = stats
        .map(|s| {
            if s.mem_limit > 0 {
                format!("{} ({:.0}%)", format_bytes(s.mem_used), s.mem_percent)
            } else {
                format_bytes(s.mem_used)
            }
        })
        .unwrap_or_else(|| "-".to_string());

    Row::new(vec![
        Cell::from(container.name.as_str()),
        Cell::from(container.image.as_str()),
        Cell::from(Text::styled(
            container.state.label(),
            Style::default().fg(state_color(container.state)),
        )),
        Cell::from(cpu),
        Cell::from(mem),
        Cell::from(container.status.as_str()),
    ])
}

/// Color a container's state for quick visual scanning.
fn state_color(state: ContainerState) -> Color {
    match state {
        ContainerState::Running => Color::Green,
        ContainerState::Restarting => Color::Yellow,
        ContainerState::Paused => Color::Blue,
        ContainerState::Created => Color::Cyan,
        ContainerState::Exited => Color::Gray,
        ContainerState::Dead | ContainerState::Removing => Color::Red,
        ContainerState::Unknown => Color::DarkGray,
    }
}
