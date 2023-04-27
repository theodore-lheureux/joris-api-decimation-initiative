use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::app::App;

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .constraints(vec![Constraint::Percentage(100), Constraint::Percentage(100)])
        .split(f.size());

    let block = Block::default().title(&*app.title).borders(Borders::ALL);
    f.render_widget(block, chunks[0]);

    let constraints = vec![Constraint::Percentage(100), Constraint::Percentage(90)];

    let chunks = Layout::default()
        .constraints(constraints)
        .horizontal_margin(1)
        .direction(Direction::Horizontal)
        .split(chunks[0]);
    
    draw_tasks(f, app, chunks[0])
}

fn draw_tasks<B>(f: &mut Frame<B>, app: &mut App, area: Rect)
where
    B: Backend,
{
    let tasks: Vec<ListItem> = app
        .tasks
        .items
        .iter()
        .map(|i| ListItem::new(vec![Spans::from(Span::raw(*i))]))
        .collect();

    let task_selected = app.tasks.state.selected() != None;

    let chunks = Layout::default()
        .constraints([Constraint::Percentage(100), Constraint::Percentage(100)].as_ref())
        .direction(Direction::Horizontal)
        .vertical_margin(2)
        .horizontal_margin(if task_selected { 1 } else { 3 })
        .split(area);

    let tasks = List::new(tasks)
        .block(Block::default())
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");
    f.render_stateful_widget(tasks, chunks[0], &mut app.tasks.state);
}