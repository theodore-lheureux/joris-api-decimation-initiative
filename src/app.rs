use reqwest::Client;
use tui::widgets::ListState;

use crate::{tasks_loader::Task, SERVER_URL};

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

pub struct App {
    pub title: String,
    pub client: Client,
    pub should_quit: bool,
    pub loading: bool,
    pub tasks: StatefulList<Task>,
    pub opened_task: Option<Task>,
    pub gauge_value: u16,
    pub enhanced_graphics: bool,
    pub throbber_state: throbber_widgets_tui::ThrobberState,
}

impl App {
    pub fn new(title: String, enhanced_graphics: bool, client: Client) -> App {
        App {
            title,
            should_quit: false,
            client,
            loading: true,
            tasks: StatefulList::with_items(vec![]),
            opened_task: None,
            gauge_value: 50,
            enhanced_graphics,
            throbber_state: throbber_widgets_tui::ThrobberState::default(),
        }
    }

    pub fn on_up(&mut self) {
        if let Some(_) = self.opened_task {
            return;
        }
        if self.loading {
            return;
        }
        self.tasks.previous();
    }

    pub fn on_down(&mut self) {
        if let Some(_) = self.opened_task {
            return;
        }
        if self.loading {
            return;
        }
        self.tasks.next();
    }

    pub fn on_right(&mut self) {
        if self.gauge_value == 100 {
            self.gauge_value = 0;
            return;
        }
        self.gauge_value += 1;
    }

    pub fn on_left(&mut self) {
        if self.gauge_value == 0 {
            self.gauge_value = 100;
            return;
        }
        self.gauge_value -= 1;
    }

    pub fn on_key(&mut self, c: char) {
        match c {
            _ => {}
        }
    }

    pub fn on_esc(&mut self) {
        if let Some(_) = self.opened_task {
            self.opened_task = None;
        } else {
            self.should_quit = true;
        }
    }

    pub async fn on_enter(&mut self) {
        if let Some(t) = self.opened_task.as_ref() {
            let url = format!("{}{}/{}/{}", SERVER_URL, "progress", t.id, self.gauge_value);

            let response = self.client.get(&url).send().await;

            if let Ok(response) = response {
                if response.status().as_u16() != 200 {
                    return;
                }
            }

            self.refresh_tasks().await;
            self.opened_task = None;
            return;
        }

        if let Some(i) = self.tasks.state.selected() {
            if let Some(task) = self.tasks.items.get(i) {
                self.gauge_value = task.percentage_done as u16;
                self.opened_task = Some(task.clone());
            }
        }
    }

    pub async fn refresh_tasks(&mut self) {
        let selected_index = self.tasks.state.selected().unwrap_or(0);

        self.loading = true;
        let tasks = crate::tasks_loader::scrape_tasks(0, 1000, &self.client).await;
        self.tasks = StatefulList::with_items(tasks);
        self.loading = false;

        if selected_index >= self.tasks.items.len() {
            return;
        }
        self.tasks.state.select(Some(selected_index));

    }

    pub fn on_tick(&mut self) {
        // Update progress
    }
}