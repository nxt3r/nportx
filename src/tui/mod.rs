mod app;
mod ui;

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

pub use app::{App, Mode};

pub fn run(refresh_secs: u64) -> Result<()> {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal();
        original_hook(panic_info);
    }));

    let mut terminal = setup_terminal()?;
    let result = run_loop(&mut terminal, refresh_secs);
    restore_terminal()?;
    result
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    refresh_secs: u64,
) -> Result<()> {
    let mut app = App::new(Duration::from_secs(refresh_secs));
    app.refresh_ports();

    let tick = Duration::from_millis(200);

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        if event::poll(tick)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // Inside run_loop, inside `if key.kind == KeyEventKind::Press {`
match app.mode.clone() {
    Mode::Normal => match key.code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Down => app.next(),
        KeyCode::Up => app.prev(),
        KeyCode::Char('k') => app.begin_kill(),
        _ => {}
    },
    Mode::ConfirmKill { port, pid, process } => match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            app.confirm_kill(port, pid, &process);
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.status = "Kill cancelled".into();
        }
        _ => {}
    },
}
                }
            }
        }

        if app.last_refresh.elapsed() >= app.refresh_interval {
            app.refresh_ports();
        }

        if app.should_quit {
            return Ok(());
        }
    }
}