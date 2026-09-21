use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};

mod app;
mod config;
mod editor;
mod states;
mod storage;
mod ui;

use app::App;

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize application and run the event loop. Keep the result until
    // terminal cleanup has completed so an initialization or runtime error
    // cannot leave the user's terminal in raw/alternate-screen mode.
    let mut app = App::new();
    let tick_rate = Duration::from_millis(16); // ~60fps
    let res = app
        .init()
        .and_then(|_| run_app(&mut terminal, &mut app, tick_rate));

    // Always restore the terminal before returning an application error.
    let cleanup_result = restore_terminal(&mut terminal);

    cleanup_result?;

    // Save state after terminal cleanup so a storage failure cannot strand
    // the user's terminal in raw/alternate-screen mode.
    if let Err(err) = res {
        eprintln!("Error: {err:?}");
        let _ = app.save_state();
        return Err(err);
    }
    app.save_state()?;
    Ok(())
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    tick_rate: Duration,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if event::poll(tick_rate)? {
            match event::read()? {
                Event::Key(key) => {
                    if (key.kind == KeyEventKind::Press || key.kind == KeyEventKind::Repeat)
                        && !app.handle_key(key)
                    {
                        break;
                    }
                }
                Event::Resize(_, _) => {
                    // Terminal resize — ratatui picks up the new size on the next draw
                }
                _ => {}
            }
        }

        // Autosave tick
        app.autosave_tick();

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
