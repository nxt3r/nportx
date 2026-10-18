use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::app::App;

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(f.area());

    let body = Paragraph::new("Hello from NportX TUI — press q to quit")
        .block(Block::default().title(" NportX ").borders(Borders::ALL));
    f.render_widget(body, chunks[0]);

    let footer = Paragraph::new(app.status.as_str());
    f.render_widget(footer, chunks[1]);
}