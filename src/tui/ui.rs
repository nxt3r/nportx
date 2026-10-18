use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

use super::app::App;

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(f.area());

    // Table Header
    let header = Row::new(vec!["PORT", "PID", "PROCESS", "MEMORY"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(0);

    // Table Rows
    let rows: Vec<Row> = app
        .ports
        .iter()
        .map(|p| {
            let color = if p.process.to_lowercase().contains("system")
                || p.process.to_lowercase().contains("svchost")
            {
                Color::Red
            } else {
                Color::Green
            };
            
            Row::new(vec![
                Cell::from(p.port.to_string()),
                Cell::from(p.pid.to_string()),
                Cell::from(p.process.clone()),
                Cell::from(format_memory(p.memory_bytes)),
            ])
            .style(Style::default().fg(color))
        })
        .collect();

    let widths = [
        Constraint::Length(8),
        Constraint::Length(8),
        Constraint::Min(20),
        Constraint::Length(12),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().title(" NportX — Listening Ports ").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("▶ ");

    let mut state = TableState::default();
    state.select(Some(app.selected));
    f.render_stateful_widget(table, chunks[0], &mut state);

    // Footer Status
    let footer = Paragraph::new(app.status.as_str());
    f.render_widget(footer, chunks[1]);
}

fn format_memory(bytes: u64) -> String {
    const MB: u64 = 1024 * 1024;
    format!("{:.1} MB", bytes as f64 / MB as f64)
}