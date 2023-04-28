use std::{
    error::Error,
    io,
    sync::mpsc,
    thread::{self},
    time::{Duration, Instant},
};

use app::App;
use crossterm::{
    event::{self, EnableMouseCapture, Event as CEvent, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen},
};
use reqwest::Client;
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

pub mod app;
pub mod ui;
pub mod tasks_loader;

pub const SERVER_URL: &str = "http://localhost:8080/api/";

struct Cli {
    tick_rate: u64,
    enhanced_graphics: bool,
}

#[derive(Debug, Clone, Copy)]
enum Event<I> {
    Input(I),
    Tick,
}

pub async fn init_ui(client: Client, username: String) -> Result<(), Box<dyn std::error::Error>> {
    let cli: Cli = Cli {
        tick_rate: 250,
        enhanced_graphics: true,
    };
    let tick_rate = Duration::from_millis(cli.tick_rate);
    let enhanced_graphics = cli.enhanced_graphics;

    run(tick_rate, enhanced_graphics, client, username).await?;

    Ok(())
}

pub async fn run(
    tick_rate: Duration,
    enhanced_graphics: bool,
    client: Client,
    username: String,
) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let app = App::new(
        "KickMyB Exploit | Signed In as: ".to_owned() + &username,
        enhanced_graphics,
        client,
    );

    run_app(&mut terminal, app, rx).await?;

    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    rx: mpsc::Receiver<Event<KeyEvent>>,
) -> Result<(), Box<dyn Error>> {
    app.refresh_tasks().await;

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        match rx.recv()? {
            Event::Input(event) => {
                if event.kind != event::KeyEventKind::Press {
                    continue;
                }
                match event.code {
                    KeyCode::Char(c) => {
                        app.on_key(c);

                        if c == 'r' {
                            app.refresh_tasks().await;
                        }
                    },
                    KeyCode::Esc => app.on_esc(),
                    KeyCode::Up => app.on_up(),
                    KeyCode::Down => app.on_down(),
                    KeyCode::Left => app.on_left(),
                    KeyCode::Right => app.on_right(),
                    KeyCode::Enter => app.on_enter().await,
                    _ => {}
                }
            }
            Event::Tick => app.on_tick(),
        }
        if app.should_quit {
            disable_raw_mode()?;
            terminal.clear()?;
            terminal.show_cursor()?;
            break;
        }
    }

    Ok(())
}
