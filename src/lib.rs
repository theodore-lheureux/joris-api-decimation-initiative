use std::{
    error::Error,
    io,
    sync::{mpsc, Arc},
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
use tokio::sync::Mutex;
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

pub mod app;
pub mod tasks_loader;
pub mod ui;

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
pub enum IoEvent {
    GetTasks,
    SetTaskProgress { id: i64, progress: u32 },
}

pub async fn init_ui(
    client: Client,
    username: String,
) -> Result<(), Box<dyn std::error::Error>> {
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
                if let CEvent::Key(key) =
                    event::read().expect("can read events")
                {
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

    let (ntx, nrx) = mpsc::channel();

    let app = Arc::new(Mutex::new(App::new(
        "KickMyB Exploit | Signed In as: ".to_owned() + &username,
        enhanced_graphics,
        ntx
    )));


    let cloned_app = app.clone();

    std::thread::spawn(move || {
        start_network_thread(&cloned_app, nrx, client);
    });

    app.lock().await.refresh();

    run_app(&mut terminal, app, rx).await?;

    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: Arc<Mutex<App>>,
    rx: mpsc::Receiver<Event<KeyEvent>>,
) -> Result<(), Box<dyn Error>> {

    loop {
        let mut app = app.lock().await;

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
                            app.refresh();
                        }
                    }
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

#[tokio::main]
async fn start_network_thread(
    app: &Arc<Mutex<App>>,
    rx: std::sync::mpsc::Receiver<IoEvent>,
    client: Client,
) {
    loop {
        if let Ok(event) = rx.recv() {
            match event {
                IoEvent::GetTasks => {
                    let tasks = tasks_loader::scrape_tasks(2, 5000, &client).await;
                    let mut app = app.lock().await;
                    app.loading = false;
                    app.tasks.items = tasks;
                }
                IoEvent::SetTaskProgress { id, progress } => {
                    let url = format!("{}{}/{}/{}", SERVER_URL, "progress", id, progress);
                    let response = client.get(&url).send().await;

                    if let Ok(response) = response {
                        let mut app = app.lock().await;
                        let task = app.tasks.items.iter_mut().find(|t| t.id == id);

                        if let Some(task) = task {
                            task.percentage_done = progress as i32;
                        }

                        if response.status().as_u16() != 200 {
                            app.refresh();
                            return;
                        }
                    }
                }
            }
        }
    }
}
