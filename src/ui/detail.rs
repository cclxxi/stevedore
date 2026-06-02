//! Detail panel for the selected container, shown beneath the list.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::App;
use crate::util::format_bytes;

/// Render details and live network/memory figures for the selection.
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title(" Details ");

    let Some(container) = app.selected_container() else {
        frame.render_widget(Paragraph::new("Nothing selected").block(block), area);
        return;
    };

    let stats = app.stats.get(&container.id);
    let ports = if container.ports.is_empty() {
        "—".to_string()
    } else {
        container.ports.clone()
    };

    let (cpu, mem, net) = match stats {
        Some(s) => (
            format!("{:.2} %", s.cpu_percent),
            format!(
                "{} / {} ({:.1} %)",
                format_bytes(s.mem_used),
                format_bytes(s.mem_limit),
                s.mem_percent
            ),
            format!("↓ {}  ↑ {}", format_bytes(s.net_rx), format_bytes(s.net_tx)),
        ),
        None => ("—".to_string(), "—".to_string(), "—".to_string()),
    };

    let lines = vec![
        field("Name", &container.name),
        field("Id", container.short_id()),
        field("Image", &container.image),
        field("Ports", &ports),
        field("CPU", &cpu),
        field("Memory", &mem),
        field("Network", &net),
    ];

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

/// A `label: value` line with a dimmed label.
fn field<'a>(label: &str, value: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(
            format!("{label:>8}: "),
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(value),
    ])
}
