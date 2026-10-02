use crossterm::{
    cursor::SetCursorStyle,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};

mod app;
mod config;
mod dictionary;
mod drives;
mod editor;
mod export;
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
        SetCursorStyle::DefaultUserShape,
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

/// Match the terminal cursor shape to the `cursor_style` setting. The escape
/// is written only when the choice changes, so it is not re-sent every frame.
fn apply_cursor_style(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    style: &str,
    applied: &mut String,
) -> io::Result<()> {
    if applied == style {
        return Ok(());
    }
    let shape = match style {
        "beam" => SetCursorStyle::SteadyBar,
        "underline" => SetCursorStyle::SteadyUnderScore,
        _ => SetCursorStyle::SteadyBlock,
    };
    execute!(terminal.backend_mut(), shape)?;
    applied.clear();
    applied.push_str(style);
    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut applied_cursor = String::new();
    loop {
        apply_cursor_style(terminal, &app.config.cursor_style, &mut applied_cursor)?;
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

        // Writing statistics and the autosave tick
        app.tick();

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
