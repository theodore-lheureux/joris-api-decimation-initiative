use std::{io, time::Duration, error::Error};

use app::App;
use argh::FromArgs;
use crossterm::{
    event::EnableMouseCapture,
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
};
use tui::{
    backend::{CrosstermBackend, Backend},
    widgets::{Block, Borders},
    Terminal,
};

pub mod app;

pub const SERVER_URL: &str = "http://localhost:8080/api/";

/// Demo
#[derive(Debug, FromArgs)]



struct Cli {
    /// time in ms between two ticks.
    #[argh(option, default = "250")]
    tick_rate: u64,
    /// whether unicode symbols are used to improve the overall look of the app
    #[argh(option, default = "true")]
    enhanced_graphics: bool,
}

pub fn init_ui() -> Result<(), Box<dyn std::error::Error>> {
    let cli: Cli = argh::from_env();
    let tick_rate = Duration::from_millis(cli.tick_rate);

    run(tick_rate, enhanced_graphics)

    Ok(())
}

pub fn run(tick_rate: Duration, enhanced_graphics: bool) -> Result<(), Box<dyn Error>> {
    // setup terminalenable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new("Termion demo", enhanced_graphics);
    run_app(&mut terminal, app, tick_rate)?;

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> Result<(), Box<dyn Error>> {
    let events = events(tick_rate);
    loop {
        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default().title("KickMyB Exploit").borders(Borders::ALL);
            f.render_widget(block, size);
        })?;

        if crossterm::event::poll(tick_rate)? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                events
            }
        }

        std::thread::sleep(tick_rate);
    }

    Ok(())
}

enum Event {
    Input(Key),
    Tick,
}

fn events(tick_rate: Duration) -> impl Iterator<Item = Event> {
    let tick_rate = tick_rate;
    let tick = std::thread::spawn(move || loop {
        std::thread::sleep(tick_rate);
        tick
    });

    let input = std::thread::spawn(move || loop {
        if crossterm::event::poll(tick_rate).unwrap() {
            if let crossterm::event::Event::Key(key) = crossterm::event::read().unwrap() {
                input
            }
        }
    });

    tick.merge(input)
}


