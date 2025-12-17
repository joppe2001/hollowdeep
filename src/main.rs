//! Hollowdeep - Entry Point
//!
//! This is the main executable that initializes the terminal,
//! sets up the game, and runs the main loop.

use std::io;
use std::time::{Duration, Instant};
use std::fs::OpenOptions;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};

use hollowdeep::game::{Game, GameState};
use hollowdeep::ui::App;

/// Target frames per second for the game loop
const TARGET_FPS: u64 = 60;
const FRAME_TIME: Duration = Duration::from_millis(1000 / TARGET_FPS);

fn main() -> Result<()> {
    // Initialize logging to file (to avoid interfering with TUI)
    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("hollowdeep.log")
        .unwrap_or_else(|_| OpenOptions::new().write(true).open("/dev/null").unwrap());

    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    )
    .target(env_logger::Target::Pipe(Box::new(log_file)))
    .init();

    log::info!("Starting Hollowdeep v{}", env!("CARGO_PKG_VERSION"));

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create game and UI
    let mut app = App::new();
    let mut game = Game::new();

    // Run the game loop
    let result = run_game_loop(&mut terminal, &mut app, &mut game);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Report any errors
    if let Err(ref e) = result {
        log::error!("Game exited with error: {}", e);
        eprintln!("Error: {}", e);
    }

    log::info!("Hollowdeep shut down cleanly");
    result
}

/// Main game loop
fn run_game_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    game: &mut Game,
) -> Result<()> {
    let mut last_frame = Instant::now();

    loop {
        let frame_start = Instant::now();
        let delta = frame_start.duration_since(last_frame);
        last_frame = frame_start;

        // Handle input
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                // Only handle key press events, not releases
                if key.kind == KeyEventKind::Press {
                    match app.handle_input(key, game) {
                        Ok(should_quit) if should_quit => break,
                        Ok(_) => {}
                        Err(e) => log::warn!("Input handling error: {}", e),
                    }
                }
            }
        }

        // Update game state
        game.update(delta);

        // Render
        terminal.draw(|frame| {
            app.render(frame, game);
        })?;

        // Check if game wants to quit
        if matches!(game.state(), GameState::Quit) {
            break;
        }

        // Frame rate limiting
        let frame_time = frame_start.elapsed();
        if frame_time < FRAME_TIME {
            std::thread::sleep(FRAME_TIME - frame_time);
        }
    }

    Ok(())
}
