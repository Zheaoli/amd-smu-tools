mod app;
mod ui;

use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = match App::new(Duration::from_millis(500)) {
        Ok(a) => a,
        Err(e) => {
            // Restore terminal before printing error
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    // Initial data fetch
    app.tick();

    // Run event loop
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    while app.running {
        terminal.draw(|f| ui::draw(f, app))?;

        // Poll for events with timeout matching refresh interval
        if event::poll(app.interval)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => app.quit(),
                        KeyCode::Char('t') => app.toggle_temps(),
                        KeyCode::Char('p') => app.toggle_power(),
                        KeyCode::Char('f') => app.toggle_freq(),
                        KeyCode::Char('+') | KeyCode::Char('=') => app.decrease_interval(),
                        KeyCode::Char('-') => app.increase_interval(),
                        _ => {}
                    }
                }
            }
        }

        // Refresh data
        app.tick();
    }

    Ok(())
}
