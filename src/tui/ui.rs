use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState},
    Frame,
};

use super::app::{App, Mode};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(f.area());

    let header = Row::new(vec!["PORT", "PID", "PROCESS", "MEMORY"])
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(0);

    let rows: Vec<Row> = app
        .visible()
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

    // Draw Search bar if typing, otherwise draw normal footer
    if app.searching {
        let text = format!("/{}", app.search);
        let footer = Paragraph::new(text).style(Style::default().bg(Color::Blue));
        f.render_widget(footer, chunks[1]);
    } else {
        let footer = Paragraph::new(app.status.as_str());
        f.render_widget(footer, chunks[1]);
    }

    // Draw Modal
    if let Mode::ConfirmKill { port, pid, process } = &app.mode {
        let area = centered_rect(50, 20, f.area());
        let msg = format!("Kill {process} (PID {pid}) on port {port}? [y/N]");
        
        let modal = Paragraph::new(msg)
            .block(Block::default().title(" Confirm ").borders(Borders::ALL))
            .style(Style::default().bg(Color::DarkGray));
            
        f.render_widget(Clear, area);
        f.render_widget(modal, area);
    }
}

fn format_memory(bytes: u64) -> String {
    const MB: u64 = 1024 * 1024;
    format!("{:.1} MB", bytes as f64 / MB as f64)
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}